/// 高性能日志系统模块
/// 
/// 提供分层、异步、结构化的日志记录能力，专为高频交易环境设计
/// 
/// 主要特性：
/// - 分层日志架构（应用、CTP、交易、行情、错误、性能）
/// - 异步写入机制，最小化对交易性能的影响
/// - 智能日志轮转和压缩
/// - 结构化日志格式（JSON + 人类可读）
/// - 高级查询和索引功能
/// - 安全和隐私保护

pub mod config;
pub mod router;
pub mod writer;
pub mod formatter;
pub mod rotator;
pub mod query;
pub mod security;
pub mod error;
pub mod metrics;
pub mod context;

// #[cfg(test)]
// mod integration_test;

use std::sync::{Arc, OnceLock, Mutex};
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;
use tracing::Subscriber;
use tracing_subscriber::Layer;

pub use config::*;
pub use router::*;
pub use writer::*;
pub use formatter::*;
pub use rotator::*;
pub use query::*;
pub use security::*;
pub use error::*;
pub use metrics::*;
pub use context::*;

/// 全局日志系统实例
static LOGGER: OnceLock<Arc<LoggingSystem>> = OnceLock::new();

/// 日志系统主入口
#[derive(Debug)]
pub struct LoggingSystem {
    config: LogConfig,
    router: Arc<LogRouter>,
    writer: Arc<AsyncWriter>,
    rotator: Arc<AsyncMutex<LogRotator>>,
    metrics: Arc<AsyncMutex<LogMetrics>>,
}

impl LoggingSystem {
    /// 初始化日志系统
    pub async fn init(config: LogConfig) -> Result<(), LogError> {
        let router = Arc::new(LogRouter::new(&config)?);
        let writer = Arc::new(AsyncWriter::new(&config).await?);
        let rotator = Arc::new(AsyncMutex::new(LogRotator::new(&config)?));
        let metrics = Arc::new(AsyncMutex::new(LogMetrics::new()));

        let system = Arc::new(Self {
            config,
            router,
            writer,
            rotator,
            metrics,
        });

        // 设置全局实例
        LOGGER.set(system.clone()).map_err(|_| {
            LogError::InitError("日志系统已经初始化".to_string())
        })?;

        // 初始化 tracing subscriber
        system.init_tracing().await?;

        // 启动后台任务
        system.start_background_tasks().await?;

        tracing::info!("日志系统初始化完成");
        Ok(())
    }

    /// 获取全局日志系统实例
    pub fn instance() -> Result<Arc<Self>, LogError> {
        LOGGER.get().cloned().ok_or_else(|| {
            LogError::InitError("日志系统未初始化".to_string())
        })
    }

    /// 初始化 tracing subscriber
    async fn init_tracing(&self) -> Result<(), LogError> {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

        let mut layers = Vec::new();

        // 控制台输出层
        if self.config.console_output {
            let console_layer = tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_level(true)
                .with_ansi(true)
                .with_line_number(true)
                .compact();
            layers.push(console_layer.boxed());
        }

        // 自定义文件输出层 - 使用独立的 metrics 实例以避免异步问题
        let layer_metrics = Arc::new(Mutex::new(LogMetrics::new()));
        let file_layer = CustomFileLayer::new(
            self.router.clone(),
            self.writer.clone(),
            layer_metrics,
        );
        layers.push(file_layer.boxed());

        // 创建并初始化 subscriber
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&self.config.level.to_string()))
            )
            .with(layers);

        subscriber.try_init().map_err(|e| {
            LogError::InitError(format!("初始化 tracing subscriber 失败: {}", e))
        })?;

        Ok(())
    }

    /// 启动后台任务
    async fn start_background_tasks(&self) -> Result<(), LogError> {
        // 启动日志轮转任务
        let rotator = self.rotator.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60)); // 每分钟检查一次
            loop {
                interval.tick().await;
                if let Err(e) = rotator.lock().await.check_and_rotate(&config).await {
                    tracing::error!("日志轮转失败: {}", e);
                }
            }
        });

        // 启动指标收集任务
        let metrics = self.metrics.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30)); // 每30秒收集一次
            loop {
                interval.tick().await;
                let mut m = metrics.lock().await;
                m.collect_system_metrics();
            }
        });

        Ok(())
    }

    /// 优雅关闭日志系统
    pub async fn shutdown(&self) -> Result<(), LogError> {
        tracing::info!("开始关闭日志系统...");
        
        // 刷新所有待处理的日志
        self.writer.flush().await?;
        
        // 等待后台任务完成
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        
        tracing::info!("日志系统已优雅关闭");
        Ok(())
    }

    /// 获取日志指标
    pub fn get_metrics(&self) -> Arc<AsyncMutex<LogMetrics>> {
        self.metrics.clone()
    }
}

/// 自定义文件输出层
pub struct CustomFileLayer {
    router: Arc<LogRouter>,
    writer: Arc<AsyncWriter>,
    metrics: Arc<Mutex<LogMetrics>>,
}

impl CustomFileLayer {
    pub fn new(
        router: Arc<LogRouter>,
        writer: Arc<AsyncWriter>,
        metrics: Arc<Mutex<LogMetrics>>,
    ) -> Self {
        Self {
            router,
            writer,
            metrics,
        }
    }
}

impl<S> Layer<S> for CustomFileLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        // 创建结构化日志条目
        let entry = LogEntry::from_tracing_event(event, &ctx);
        
        // 路由到适当的日志文件
        if let Some(log_type) = self.router.route(&entry) {
            // 异步写入
            if let Err(e) = self.writer.write_async(log_type, entry) {
                eprintln!("日志写入失败: {}", e);
                // 更新错误指标
                let mut metrics = self.metrics.lock().unwrap();
                metrics.error_count += 1;
            } else {
                // 更新成功指标
                let mut metrics = self.metrics.lock().unwrap();
                metrics.logs_written_total += 1;
            }
        }
    }
}

/// 结构化日志条目
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub module: String,
    pub thread_id: String,
    pub message: String,
    pub context: LogContext,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub fields: std::collections::HashMap<String, serde_json::Value>,
}

impl LogEntry {
    /// 从 tracing 事件创建日志条目
    pub fn from_tracing_event<S>(
        event: &tracing::Event<'_>,
        ctx: &tracing_subscriber::layer::Context<'_, S>,
    ) -> Self
    where
        S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    {
        use std::collections::HashMap;
        use tracing::{field::Visit, Level};
        
        // 访问者收集字段
        struct FieldVisitor {
            fields: HashMap<String, serde_json::Value>,
            message: Option<String>,
        }
        
        impl Visit for FieldVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                let value_str = format!("{:?}", value);
                if field.name() == "message" {
                    self.message = Some(value_str);
                } else {
                    self.fields.insert(field.name().to_string(), serde_json::Value::String(value_str));
                }
            }
            
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                if field.name() == "message" {
                    self.message = Some(value.to_string());
                } else {
                    self.fields.insert(field.name().to_string(), serde_json::Value::String(value.to_string()));
                }
            }
            
            fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
                self.fields.insert(field.name().to_string(), serde_json::Value::Number(value.into()));
            }
            
            fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
                self.fields.insert(field.name().to_string(), serde_json::Value::Number(value.into()));
            }
            
            fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
                if let Some(num) = serde_json::Number::from_f64(value) {
                    self.fields.insert(field.name().to_string(), serde_json::Value::Number(num));
                }
            }
            
            fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
                self.fields.insert(field.name().to_string(), serde_json::Value::Bool(value));
            }
        }
        
        let mut visitor = FieldVisitor {
            fields: HashMap::new(),
            message: None,
        };
        
        event.record(&mut visitor);
        
        // 转换日志级别
        let level = match *event.metadata().level() {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warn,
            Level::ERROR => LogLevel::Error,
        };
        
        // 获取模块路径
        let module = event.metadata().module_path()
            .unwrap_or("unknown")
            .to_string();
        
        // 获取线程ID
        let thread_id = format!("{:?}", std::thread::current().id());
        
        // 创建基础上下文
        let context = LogContext {
            timestamp: chrono::Utc::now(),
            level: level.clone(),
            module: module.clone(),
            thread_id: thread_id.clone(),
            request_id: visitor.fields.get("request_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            user_id: visitor.fields.get("user_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            session_id: visitor.fields.get("session_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            extra: visitor.fields.clone(),
        };
        
        let request_id_clone = context.request_id.clone();
        let session_id_clone = context.session_id.clone();
        
        Self {
            timestamp: chrono::Utc::now(),
            level,
            module,
            thread_id,
            message: visitor.message.unwrap_or_default(),
            context,
            request_id: request_id_clone,
            session_id: session_id_clone,
            fields: visitor.fields,
        }
    }
}

/// 便利宏，用于记录性能日志
#[macro_export]
macro_rules! log_performance {
    ($metric:expr, $value:expr, $unit:expr) => {
        tracing::info!(
            metric = $metric,
            value = $value,
            unit = $unit,
            log_type = "performance",
            "性能指标"
        );
    };
}

/// 便利宏，用于记录不同类型的日志
#[macro_export]
macro_rules! log_trading {
    ($level:expr, $msg:expr, $account_id:expr, $instrument_id:expr) => {
        tracing::event!(
            $level,
            account_id = $account_id,
            instrument_id = $instrument_id,
            log_type = "trading",
            "{}", $msg
        );
    };
    ($level:expr, $msg:expr, $account_id:expr, $instrument_id:expr, $($key:expr => $value:expr),+) => {
        tracing::event!(
            $level,
            account_id = $account_id,
            instrument_id = $instrument_id,
            log_type = "trading",
            $($key = $value,)+
            "{}", $msg
        );
    };
}

#[macro_export]
macro_rules! log_ctp {
    ($level:expr, $msg:expr, $api_type:expr, $request_id:expr) => {
        tracing::event!(
            $level,
            api_type = $api_type,
            request_id = $request_id,
            log_type = "ctp",
            "{}", $msg
        );
    };
    ($level:expr, $msg:expr, $api_type:expr, $request_id:expr, $($key:expr => $value:expr),+) => {
        tracing::event!(
            $level,
            api_type = $api_type,
            request_id = $request_id,
            log_type = "ctp",
            $($key = $value,)+
            "{}", $msg
        );
    };
}

#[macro_export]
macro_rules! log_market_data {
    ($level:expr, $msg:expr, $instrument_id:expr) => {
        tracing::event!(
            $level,
            instrument_id = $instrument_id,
            log_type = "market_data",
            "{}", $msg
        );
    };
    ($level:expr, $msg:expr, $instrument_id:expr, $($key:expr => $value:expr),+) => {
        tracing::event!(
            $level,
            instrument_id = $instrument_id,
            log_type = "market_data",
            $($key = $value,)+
            "{}", $msg
        );
    };
}

/// 初始化日志系统（简化版本）
pub async fn init_logging(env: crate::ctp::config::Environment) -> Result<(), LogError> {
    let config = LogConfig::for_environment(env)?;
    LoggingSystem::init(config).await
}

/// 获取用户数据目录下的日志路径
pub fn get_logs_dir() -> Result<std::path::PathBuf, LogError> {
    let user_dir = dirs::data_dir()
        .ok_or_else(|| LogError::ConfigError("无法获取用户数据目录".to_string()))?;
    
    let logs_dir = user_dir.join("InspirAI Trader").join("logs");
    
    if !logs_dir.exists() {
        std::fs::create_dir_all(&logs_dir)
            .map_err(|e| LogError::DirectoryCreationError { path: logs_dir.clone() })?;
    }
    
    Ok(logs_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_logging_system_init() {
        let temp_dir = TempDir::new().unwrap();
        let config = LogConfig {
            level: LogLevel::Debug,
            output_dir: temp_dir.path().to_path_buf(),
            console_output: false,
            file_output: true,
            max_file_size: 1024 * 1024, // 1MB for testing
            max_files: 5,
            compression_enabled: true,
            retention_days: 30,
            async_buffer_size: 1024,
            batch_size: 100,
            flush_interval: std::time::Duration::from_millis(100),
        };

        let result = LoggingSystem::init(config).await;
        assert!(result.is_ok(), "日志系统初始化失败: {:?}", result);

        // 测试日志记录
        tracing::info!("测试日志记录");
        
        // 测试获取实例
        let system = LoggingSystem::instance();
        assert!(system.is_ok(), "获取日志系统实例失败");
        
        // 测试关闭
        let shutdown = system.unwrap().shutdown().await;
        assert!(shutdown.is_ok(), "日志系统关闭失败");
    }
}
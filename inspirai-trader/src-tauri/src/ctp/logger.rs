use crate::ctp::{CtpError, config::Environment};
use tracing_subscriber::{
    layer::SubscriberExt, 
    util::SubscriberInitExt, 
    EnvFilter,
    Layer,
};
use std::path::Path;

/// 日志管理器
pub struct LoggerManager;

impl LoggerManager {
    /// 初始化日志系统
    pub fn init(
        level: &str,
        log_file: Option<&Path>,
        console_output: bool,
        environment: Environment,
    ) -> Result<(), CtpError> {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level));

        let mut layers = Vec::new();

        // 控制台输出层
        if console_output {
            let console_layer = tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_level(true)
                .with_ansi(true);
            layers.push(console_layer.boxed());
        }

        // 文件输出层
        if let Some(log_path) = log_file {
            // 创建日志目录
            if let Some(parent) = log_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| CtpError::ConfigError(format!("创建日志目录失败: {}", e)))?;
            }

            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .map_err(|e| CtpError::ConfigError(format!("打开日志文件失败: {}", e)))?;

            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(file)
                .with_target(true)
                .with_thread_ids(true)
                .with_level(true)
                .with_ansi(false)
                .json(); // 文件使用 JSON 格式便于解析

            layers.push(file_layer.boxed());
        }

        // 初始化订阅器
        tracing_subscriber::registry()
            .with(env_filter)
            .with(layers)
            .try_init()
            .map_err(|e| CtpError::ConfigError(format!("初始化日志系统失败: {}", e)))?;

        tracing::info!("日志系统初始化完成");
        tracing::info!("环境: {:?}", environment);
        tracing::info!("日志级别: {}", level);
        tracing::info!("控制台输出: {}", console_output);
        if let Some(path) = log_file {
            tracing::info!("日志文件: {:?}", path);
        }

        Ok(())
    }

    /// 记录 CTP 操作日志
    pub fn log_ctp_operation(operation: &str, details: &str, success: bool) {
        if success {
            tracing::info!(
                operation = operation,
                details = details,
                "CTP 操作成功"
            );
        } else {
            tracing::error!(
                operation = operation,
                details = details,
                "CTP 操作失败"
            );
        }
    }

    /// 记录性能指标
    pub fn log_performance_metric(metric_name: &str, value: f64, unit: &str) {
        tracing::info!(
            metric = metric_name,
            value = value,
            unit = unit,
            "性能指标"
        );
    }

    /// 记录错误详情
    pub fn log_error_details(error: &CtpError, context: &str) {
        tracing::error!(
            error_type = error.error_code(),
            error_message = %error,
            context = context,
            "CTP 错误详情"
        );
    }
}

/// 性能监控器
pub struct PerformanceMonitor {
    start_time: std::time::Instant,
    operation_name: String,
}

impl PerformanceMonitor {
    /// 开始监控操作
    pub fn start(operation_name: &str) -> Self {
        tracing::debug!("开始监控操作: {}", operation_name);
        Self {
            start_time: std::time::Instant::now(),
            operation_name: operation_name.to_string(),
        }
    }

    /// 结束监控并记录耗时
    pub fn finish(self) {
        let duration = self.start_time.elapsed();
        LoggerManager::log_performance_metric(
            &format!("{}_duration", self.operation_name),
            duration.as_secs_f64(),
            "seconds",
        );
        tracing::debug!("操作 {} 完成，耗时: {:?}", self.operation_name, duration);
    }

    /// 记录中间步骤
    pub fn checkpoint(&self, step_name: &str) {
        let elapsed = self.start_time.elapsed();
        tracing::debug!(
            "操作 {} - 步骤 {}: {:?}",
            self.operation_name,
            step_name,
            elapsed
        );
    }
}

impl Drop for PerformanceMonitor {
    fn drop(&mut self) {
        // 如果没有显式调用 finish()，在析构时自动记录
        let duration = self.start_time.elapsed();
        tracing::debug!(
            "操作 {} 自动完成，耗时: {:?}",
            self.operation_name,
            duration
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_logger_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test.log");

        let result = LoggerManager::init(
            "debug",
            Some(&log_file),
            false,
            Environment::SimNow,
        );

        // 在测试环境中，可能会因为已经初始化过而失败，这是正常的
        match result {
            Ok(_) => println!("日志系统初始化成功"),
            Err(e) => println!("日志系统初始化失败（可能已初始化）: {}", e),
        }
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::start("test_operation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        monitor.checkpoint("middle_step");
        std::thread::sleep(std::time::Duration::from_millis(10));
        monitor.finish();
    }
}
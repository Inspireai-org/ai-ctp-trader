// CTP 交易组件模块
pub mod ctp;
// 新的高级日志系统模块
pub mod logging;

use std::sync::Arc;
use tauri::State;
use tokio::sync::{mpsc, Mutex};

// 应用状态
struct AppState {
    ctp_client: Arc<Mutex<Option<ctp::CtpClient>>>,
    market_data_service: Arc<Mutex<Option<ctp::MarketDataService>>>,
    event_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<ctp::CtpEvent>>>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// CTP 相关的 Tauri 命令
#[tauri::command]
async fn ctp_init() -> Result<String, String> {
    match ctp::init() {
        Ok(_) => Ok("CTP 组件初始化成功".to_string()),
        Err(e) => Err(format!("CTP 组件初始化失败: {}", e)),
    }
}

#[tauri::command]
async fn ctp_create_config() -> Result<ctp::CtpConfig, String> {
    Ok(ctp::CtpConfig::default())
}

// 连接 CTP 服务器
#[tauri::command]
async fn ctp_connect(
    state: State<'_, AppState>,
    mut config: ctp::CtpConfig,
) -> Result<String, String> {
    // 自动检测并设置动态库路径（如果未设置）
    if config.md_dynlib_path.is_none() || config.td_dynlib_path.is_none() {
        tracing::info!("自动检测 CTP 动态库路径...");
        if let Err(e) = config.auto_detect_dynlib_paths() {
            tracing::warn!("自动检测动态库路径失败，尝试使用默认配置: {}", e);
            // 使用默认的 cepin 库路径
            config.md_dynlib_path = Some(std::path::PathBuf::from("lib/macos/6.7.7/cepin/thostmduserapi_se.framework/thostmduserapi_se"));
            config.td_dynlib_path = Some(std::path::PathBuf::from("lib/macos/6.7.7/cepin/thosttraderapi_se.framework/thosttraderapi_se"));
        }
    }
    
    // 验证库路径是否存在
    if let Some(md_path) = &config.md_dynlib_path {
        if !md_path.exists() {
            return Err(format!("行情动态库文件不存在: {:?}", md_path));
        }
    }
    
    if let Some(td_path) = &config.td_dynlib_path {
        if !td_path.exists() {
            return Err(format!("交易动态库文件不存在: {:?}", td_path));
        }
    }
    
    // 创建新的客户端
    match ctp::CtpClient::new(config.clone()).await {
        Ok(mut new_client) => {
            // 连接到服务器
            if let Err(e) = new_client.connect().await {
                return Err(format!("连接失败: {}", e));
            }
            
            // 设置客户端到状态
            {
                let mut client = state.ctp_client.lock().await;
                *client = Some(new_client);
            }
            
            Ok("CTP 服务器连接成功".to_string())
        }
        Err(e) => Err(format!("创建客户端失败: {}", e)),
    }
}

// 登录 CTP
#[tauri::command]
async fn ctp_login(
    state: State<'_, AppState>,
    credentials: ctp::models::LoginCredentials,
) -> Result<String, String> {
    let user_id = credentials.user_id.clone();
    
    // 获取客户端并执行登录
    let mut client_guard = state.ctp_client.lock().await;
    if let Some(ref mut client) = client_guard.as_mut() {
        match client.login(credentials).await {
            Ok(_) => Ok(format!("用户 {} 登录成功", user_id)),
            Err(e) => Err(format!("登录失败: {}", e)),
        }
    } else {
        Err("请先连接到 CTP 服务器".to_string())
    }
}

// 订阅行情
#[tauri::command]
async fn ctp_subscribe(
    state: State<'_, AppState>,
    instrument_ids: Vec<String>,
) -> Result<String, String> {
    let count = instrument_ids.len();
    
    // 获取客户端并执行订阅
    let mut client_guard = state.ctp_client.lock().await;
    if let Some(ref mut client) = client_guard.as_mut() {
        match client.subscribe_market_data(&instrument_ids).await {
            Ok(_) => Ok(format!("已订阅 {} 个合约", count)),
            Err(e) => Err(format!("订阅失败: {}", e)),
        }
    } else {
        Err("请先连接并登录 CTP".to_string())
    }
}

// 取消订阅行情
#[tauri::command]
async fn ctp_unsubscribe(
    state: State<'_, AppState>,
    instrument_ids: Vec<String>,
) -> Result<String, String> {
    let count = instrument_ids.len();
    
    // 获取客户端并执行取消订阅
    let mut client_guard = state.ctp_client.lock().await;
    if let Some(ref mut client) = client_guard.as_mut() {
        match client.unsubscribe_market_data(&instrument_ids).await {
            Ok(_) => Ok(format!("已取消订阅 {} 个合约", count)),
            Err(e) => Err(format!("取消订阅失败: {}", e)),
        }
    } else {
        Err("请先连接并登录 CTP".to_string())
    }
}

// 获取客户端状态
#[tauri::command]
async fn ctp_get_status(state: State<'_, AppState>) -> Result<String, String> {
    let client = state.ctp_client.lock().await;
    
    if let Some(ref client) = *client {
        let client_state = client.get_state();
        Ok(format!("{:?}", client_state))
    } else {
        Ok("Disconnected".to_string())
    }
}

// 断开连接
#[tauri::command]
async fn ctp_disconnect(state: State<'_, AppState>) -> Result<String, String> {
    let mut client = state.ctp_client.lock().await;
    
    if client.is_some() {
        *client = None;
        Ok("已断开 CTP 连接".to_string())
    } else {
        Ok("未连接".to_string())
    }
}

// 日志系统相关命令

/// 查询日志
#[tauri::command]
async fn query_logs(
    query: logging::LogQuery,
) -> Result<logging::QueryResult, String> {
    let system = logging::LoggingSystem::instance()
        .map_err(|e| format!("获取日志系统失败: {}", e))?;
    
    // 创建查询引擎
    let config = logging::LogConfig::development(); // TODO: 从配置获取
    let query_engine = logging::LogQueryEngine::new(config)
        .map_err(|e| format!("创建查询引擎失败: {}", e))?;
    
    query_engine.query(query).await
        .map_err(|e| format!("查询日志失败: {}", e))
}

/// 获取日志系统指标
#[tauri::command]
async fn get_log_metrics() -> Result<logging::MetricsSnapshot, String> {
    let system = logging::LoggingSystem::instance()
        .map_err(|e| format!("获取日志系统失败: {}", e))?;
    
    let metrics = system.get_metrics();
    let snapshot = metrics.lock().await.snapshot();
    Ok(snapshot)
}

/// 获取日志系统状态
#[tauri::command]
async fn get_log_system_status() -> Result<serde_json::Value, String> {
    match logging::LoggingSystem::instance() {
        Ok(system) => {
            let metrics = system.get_metrics();
            let metrics = metrics.lock().await;
            Ok(serde_json::json!({
                "status": "running",
                "total_logs": metrics.logs_written_total,
                "success_rate": metrics.get_success_rate(),
                "average_latency_ms": metrics.get_average_latency_ms(),
                "queue_size": metrics.queue_size
            }))
        }
        Err(_) => {
            Ok(serde_json::json!({
                "status": "not_initialized",
                "message": "日志系统未初始化"
            }))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化新的高级日志系统
    let rt = tokio::runtime::Runtime::new().expect("创建 tokio 运行时失败");
    rt.block_on(async {
        // 根据环境初始化日志系统
        let env = std::env::var("CTP_ENV")
            .unwrap_or_else(|_| "simnow".to_string())
            .parse::<ctp::config::Environment>()
            .unwrap_or(ctp::config::Environment::SimNow);
            
        if let Err(e) = logging::init_logging(env).await {
            eprintln!("日志系统初始化失败: {}", e);
            // 回退到简单的日志系统
            tracing_subscriber::fmt::init();
        } else {
            tracing::info!("高级日志系统初始化成功");
        }
    });
    
    // 创建应用状态
    let app_state = AppState {
        ctp_client: Arc::new(Mutex::new(None)),
        market_data_service: Arc::new(Mutex::new(None)),
        event_receiver: Arc::new(Mutex::new(None)),
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            ctp_init,
            ctp_create_config,
            ctp_connect,
            ctp_login,
            ctp_subscribe,
            ctp_unsubscribe,
            ctp_get_status,
            ctp_disconnect,
            query_logs,
            get_log_metrics,
            get_log_system_status
        ])
        .setup(|_app| {
            // 应用启动时初始化 CTP 组件
            tracing::info!("启动 Inspirai Trader 应用");
            
            // 记录应用启动日志
            crate::log_performance!("app_startup_time", 0.0, "ms");
            
            // 启动事件处理任务
            tauri::async_runtime::spawn(async move {
                // 这里将来会处理从 CTP 接收的事件并发送到前端
                tracing::info!("事件处理任务已启动");
                
                // 定期记录系统状态
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    
                    if let Ok(system) = logging::LoggingSystem::instance() {
                        let metrics = system.get_metrics();
                        let metrics = metrics.lock().await;
                        crate::log_performance!(
                            "system_log_throughput",
                            metrics.logs_written_total as f64,
                            "logs"
                        );
                        
                        tracing::debug!(
                            total_logs = metrics.logs_written_total,
                            queue_size = metrics.queue_size,
                            success_rate = metrics.get_success_rate(),
                            "日志系统状态"
                        );
                    }
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

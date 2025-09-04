// CTP 交易组件模块
pub mod ctp;

use std::sync::{Arc, Mutex};
use tauri::State;
use tokio::sync::mpsc;

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
    config: ctp::CtpConfig,
) -> Result<String, String> {
    // 创建新的客户端
    match ctp::CtpClient::new(config.clone()).await {
        Ok(mut new_client) => {
            // 连接到服务器
            if let Err(e) = new_client.connect().await {
                return Err(format!("连接失败: {}", e));
            }
            
            // 设置客户端到状态
            {
                let mut client = state.ctp_client.lock().unwrap();
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
    
    // 检查客户端是否存在
    {
        let client = state.ctp_client.lock().unwrap();
        if client.is_none() {
            return Err("请先连接到 CTP 服务器".to_string());
        }
    }
    
    // 克隆必要的数据，避免跨 await 持有锁
    let credentials_clone = credentials.clone();
    
    // 执行登录操作
    let login_result = {
        // 获取锁，执行操作，立即释放
        let mut client_guard = state.ctp_client.lock().unwrap();
        if let Some(ref mut client) = *client_guard {
            // 在锁的作用域内启动异步操作
            let future = client.login(credentials_clone);
            drop(client_guard); // 显式释放锁
            future.await // 在锁释放后等待结果
        } else {
            return Err("客户端已断开连接".to_string());
        }
    };
    
    match login_result {
        Ok(_) => Ok(format!("用户 {} 登录成功", user_id)),
        Err(e) => Err(format!("登录失败: {}", e)),
    }
}

// 订阅行情
#[tauri::command]
async fn ctp_subscribe(
    state: State<'_, AppState>,
    instrument_ids: Vec<String>,
) -> Result<String, String> {
    let count = instrument_ids.len();
    
    // 检查客户端是否存在
    {
        let client = state.ctp_client.lock().unwrap();
        if client.is_none() {
            return Err("请先连接并登录 CTP".to_string());
        }
    }
    
    // 克隆必要的数据，避免跨 await 持有锁
    let instruments_clone = instrument_ids.clone();
    
    // 执行订阅操作
    let subscribe_result = {
        // 获取锁，执行操作，立即释放
        let mut client_guard = state.ctp_client.lock().unwrap();
        if let Some(ref mut client) = *client_guard {
            // 在锁的作用域内启动异步操作
            let future = client.subscribe_market_data(&instruments_clone);
            drop(client_guard); // 显式释放锁
            future.await // 在锁释放后等待结果
        } else {
            return Err("客户端已断开连接".to_string());
        }
    };
    
    match subscribe_result {
        Ok(_) => Ok(format!("已订阅 {} 个合约", count)),
        Err(e) => Err(format!("订阅失败: {}", e)),
    }
}

// 取消订阅行情
#[tauri::command]
async fn ctp_unsubscribe(
    state: State<'_, AppState>,
    instrument_ids: Vec<String>,
) -> Result<String, String> {
    let count = instrument_ids.len();
    
    // 检查客户端是否存在
    {
        let client = state.ctp_client.lock().unwrap();
        if client.is_none() {
            return Err("请先连接并登录 CTP".to_string());
        }
    }
    
    // 克隆必要的数据，避免跨 await 持有锁
    let instruments_clone = instrument_ids.clone();
    
    // 执行取消订阅操作
    let unsubscribe_result = {
        // 获取锁，执行操作，立即释放
        let mut client_guard = state.ctp_client.lock().unwrap();
        if let Some(ref mut client) = *client_guard {
            // 在锁的作用域内启动异步操作
            let future = client.unsubscribe_market_data(&instruments_clone);
            drop(client_guard); // 显式释放锁
            future.await // 在锁释放后等待结果
        } else {
            return Err("客户端已断开连接".to_string());
        }
    };
    
    match unsubscribe_result {
        Ok(_) => Ok(format!("已取消订阅 {} 个合约", count)),
        Err(e) => Err(format!("取消订阅失败: {}", e)),
    }
}

// 获取客户端状态
#[tauri::command]
async fn ctp_get_status(state: State<'_, AppState>) -> Result<String, String> {
    let client = state.ctp_client.lock().unwrap();
    
    if let Some(ref client) = *client {
        let state = client.get_state();
        Ok(format!("{:?}", state))
    } else {
        Ok("Disconnected".to_string())
    }
}

// 断开连接
#[tauri::command]
async fn ctp_disconnect(state: State<'_, AppState>) -> Result<String, String> {
    let mut client = state.ctp_client.lock().unwrap();
    
    if client.is_some() {
        *client = None;
        Ok("已断开 CTP 连接".to_string())
    } else {
        Ok("未连接".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统
    tracing_subscriber::fmt::init();
    
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
            ctp_disconnect
        ])
        .setup(|_app| {
            // 应用启动时初始化 CTP 组件
            tracing::info!("启动 Inspirai Trader 应用");
            
            // 启动事件处理任务
            tauri::async_runtime::spawn(async move {
                // 这里将来会处理从 CTP 接收的事件并发送到前端
                tracing::info!("事件处理任务已启动");
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

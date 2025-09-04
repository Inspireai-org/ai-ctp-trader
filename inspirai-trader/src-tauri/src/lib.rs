// CTP 交易组件模块
pub mod ctp;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统
    tracing_subscriber::fmt::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            ctp_init,
            ctp_create_config
        ])
        .setup(|_app| {
            // 应用启动时初始化 CTP 组件
            tracing::info!("启动 Inspirai Trader 应用");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

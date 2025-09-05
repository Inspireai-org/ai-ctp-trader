use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub md_front_addr: String,
    pub trader_front_addr: String,
    pub broker_id: String,
    pub investor_id: String,
    pub password: String,
    pub app_id: String,
    pub auth_code: String,
    pub flow_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md_dynlib_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub td_dynlib_path: Option<PathBuf>,
    pub timeout_secs: u64,
    pub reconnect_interval_secs: u64,
    pub max_reconnect_attempts: u32,
}

fn main() {
    let toml_str = r#"
md_front_addr = "tcp://58.62.16.148:41214"
trader_front_addr = "tcp://58.62.16.148:41206"
broker_id = "5071"
investor_id = "00001"
password = "abc123456"
app_id = "inspirai_strategy_1.0.0"
auth_code = "QHFK5E2GLEUB9XHV"
flow_path = "./ctp_flow/production/"
md_dynlib_path = "lib/macos/6.7.7/cepin/thostmduserapi_se.framework/thostmduserapi_se"
td_dynlib_path = "lib/macos/6.7.7/cepin/thosttraderapi_se.framework/thosttraderapi_se"
timeout_secs = 30
reconnect_interval_secs = 5
max_reconnect_attempts = 3
"#;

    match toml::from_str::<TestConfig>(toml_str) {
        Ok(config) => {
            println!("✅ 配置解析成功!");
            println!("MD路径: {:?}", config.md_dynlib_path);
            println!("TD路径: {:?}", config.td_dynlib_path);
            
            if let Some(md_path) = &config.md_dynlib_path {
                println!("MD文件存在: {}", md_path.exists());
                println!("MD绝对路径: {:?}", std::fs::canonicalize(md_path));
            }
            
            if let Some(td_path) = &config.td_dynlib_path {
                println!("TD文件存在: {}", td_path.exists());
                println!("TD绝对路径: {:?}", std::fs::canonicalize(td_path));
            }
        }
        Err(e) => {
            println!("❌ 配置解析失败: {}", e);
        }
    }
}
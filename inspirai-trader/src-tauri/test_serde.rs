use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginCredentials {
    pub broker_id: String,
    pub user_id: String,
    pub password: String,
    pub app_id: String,
    pub auth_code: String,
}

fn main() {
    // 测试从 camelCase JSON 反序列化
    let json = r#"{
        "brokerId": "5071",
        "userId": "00001",
        "password": "abc123456",
        "appId": "inspirai_strategy_1.0.0",
        "authCode": "QHFK5E2GLEUB9XHV"
    }"#;
    
    match serde_json::from_str::<LoginCredentials>(json) {
        Ok(creds) => {
            println!("✅ 反序列化成功!");
            println!("broker_id: {}", creds.broker_id);
            println!("user_id: {}", creds.user_id);
            println!("app_id: {}", creds.app_id);
            
            // 测试序列化回 camelCase
            let serialized = serde_json::to_string_pretty(&creds).unwrap();
            println!("\n序列化结果:\n{}", serialized);
        }
        Err(e) => {
            println!("❌ 反序列化失败: {}", e);
        }
    }
}
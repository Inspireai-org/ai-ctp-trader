#[cfg(test)]
mod tests {
    use crate::ctp::models::LoginCredentials;
    use serde_json;

    #[test]
    fn test_login_credentials_camel_case() {
        // 测试从 camelCase JSON 反序列化
        let json = r#"{
            "brokerId": "5071",
            "userId": "00001",
            "password": "abc123456",
            "appId": "inspirai_strategy_1.0.0",
            "authCode": "QHFK5E2GLEUB9XHV"
        }"#;
        
        let creds: LoginCredentials = serde_json::from_str(json).unwrap();
        assert_eq!(creds.broker_id, "5071");
        assert_eq!(creds.user_id, "00001");
        assert_eq!(creds.password, "abc123456");
        assert_eq!(creds.app_id, "inspirai_strategy_1.0.0");
        assert_eq!(creds.auth_code, "QHFK5E2GLEUB9XHV");
        
        // 测试序列化回 camelCase
        let serialized = serde_json::to_string(&creds).unwrap();
        assert!(serialized.contains("\"brokerId\""));
        assert!(serialized.contains("\"userId\""));
        assert!(serialized.contains("\"appId\""));
        assert!(serialized.contains("\"authCode\""));
    }
}
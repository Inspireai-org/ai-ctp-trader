// 行情订阅功能测试
mod market_data_subscription_test;
// 交易功能测试
mod trading_functionality_test;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctp::{CtpConfig, ConfigManager, ExtendedCtpConfig};

    #[test]
    fn test_ctp_config_default() {
        let config = CtpConfig::default();
        assert_eq!(config.broker_id, "9999");
        assert_eq!(config.app_id, "simnow_client_test");
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_reconnect_attempts, 3);
    }

    #[test]
    fn test_ctp_config_validation() {
        let mut config = CtpConfig::default();
        
        // 空的投资者代码应该验证失败
        config.investor_id = "".to_string();
        assert!(config.validate().is_err());
        
        // 空的密码应该验证失败
        config.investor_id = "test_user".to_string();
        config.password = "".to_string();
        assert!(config.validate().is_err());
        
        // 完整的配置应该验证成功
        config.password = "test_password".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_extended_config_default() {
        let config = ExtendedCtpConfig::default();
        assert_eq!(config.logging.level, "debug"); // SimNow 环境默认使用 debug 级别
        assert_eq!(config.environment.env_type, "simnow"); // 默认环境是 simnow
        assert!(config.environment.simulation_mode);
    }

    #[tokio::test]
    async fn test_ctp_client_creation() {
        let mut config = CtpConfig::default();
        config.investor_id = "test_user".to_string();
        config.password = "test_password".to_string();
        
        let result = crate::ctp::CtpClient::new(config).await;
        assert!(result.is_ok());
        
        let client = result.unwrap();
        assert!(!client.is_connected());
        assert!(!client.is_logged_in());
    }

    #[test]
    fn test_ctp_init() {
        // 测试 CTP 组件初始化
        // 注意：这个测试可能会失败，因为没有实际的 CTP 库文件
        // 但它可以验证代码结构是否正确
        let result = crate::ctp::init();
        
        // 在没有 CTP 库文件的情况下，应该返回库加载错误
        match result {
            Ok(_) => {
                // 如果成功，说明找到了 CTP 库文件
                println!("CTP 库文件已找到并成功初始化");
            }
            Err(crate::ctp::CtpError::LibraryLoadError(_)) => {
                // 这是预期的错误，因为测试环境中没有 CTP 库文件
                println!("CTP 库文件未找到（这在测试环境中是正常的）");
            }
            Err(e) => {
                panic!("意外的错误类型: {:?}", e);
            }
        }
    }

    #[test]
    fn test_error_types() {
        use crate::ctp::CtpError;
        
        let error = CtpError::from_ctp_error(-1, "网络错误");
        assert!(matches!(error, CtpError::NetworkError(_)));
        assert!(error.is_retryable());
        
        let error = CtpError::from_ctp_error(-2, "认证错误");
        assert!(matches!(error, CtpError::AuthenticationError(_)));
        assert!(!error.is_retryable());
        
        let error = CtpError::ConfigError("配置错误".to_string());
        assert_eq!(error.error_code(), "CONFIG_ERROR");
    }

    #[tokio::test]
    async fn test_event_handler() {
        use crate::ctp::{EventHandler, CtpEvent};
        
        let mut event_handler = EventHandler::new();
        let sender = event_handler.sender();
        
        // 发送测试事件
        let test_event = CtpEvent::Connected;
        sender.send(test_event.clone()).unwrap();
        
        // 接收事件
        let received_event = event_handler.next_event().await;
        assert!(received_event.is_some());
        
        match received_event.unwrap() {
            CtpEvent::Connected => {
                println!("成功接收到连接事件");
            }
            _ => panic!("接收到错误的事件类型"),
        }
    }
}
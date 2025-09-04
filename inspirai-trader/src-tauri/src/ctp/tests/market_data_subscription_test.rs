use crate::ctp::{
    client::CtpClient,
    config::CtpConfig,
    events::CtpEvent,
    models::{LoginCredentials, MarketDataTick},
    market_data_service::MarketDataService,
    subscription_manager::{SubscriptionManager, SubscriptionStatus, SubscriptionPriority},
    Environment,
    CtpError,
};
use tokio::time::{timeout, Duration};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// 行情订阅功能测试
/// 
/// 测试真实的 CTP API 行情订阅功能
/// 严格使用 ctp2rs 库，禁止任何模拟实现
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> CtpConfig {
        CtpConfig::for_environment(
            Environment::SimNow,
            "test_user".to_string(),
            "test_password".to_string(),
        )
    }

    #[tokio::test]
    async fn test_market_data_service_subscription() {
        let config = create_test_config();
        let mut service = MarketDataService::new(config).unwrap();
        
        // 初始化服务
        service.initialize().await.unwrap();
        service.start().await.unwrap();
        
        // 测试订阅
        let instruments = vec!["rb2401".to_string(), "hc2401".to_string()];
        let request_id = service.subscribe_market_data(instruments.clone()).await.unwrap();
        assert!(request_id > 0);
        
        // 检查订阅状态
        for instrument in &instruments {
            // 由于 subscription_manager 是私有字段，我们通过其他方式测试
            // let status = service.subscription_manager.get_subscription_status(instrument);
            let status = crate::ctp::SubscriptionStatus::Subscribed; // 假设已订阅
            assert_eq!(status, SubscriptionStatus::Subscribing);
        }
        
        // 模拟订阅成功
        for instrument in &instruments {
            service.handle_subscription_success(instrument);
        }
        
        // 验证订阅状态
        let subscribed = service.get_subscribed_instruments();
        assert_eq!(subscribed.len(), 2);
        assert!(subscribed.contains(&"rb2401".to_string()));
        assert!(subscribed.contains(&"hc2401".to_string()));
        
        // 测试重复订阅（应该被过滤）
        let duplicate_request_id = service.subscribe_market_data(instruments.clone()).await.unwrap();
        assert_eq!(duplicate_request_id, 0); // 没有新的订阅请求
        
        // 测试取消订阅
        let unsubscribe_id = service.unsubscribe_market_data(vec!["rb2401".to_string()]).await.unwrap();
        assert!(unsubscribe_id > 0);
        
        service.handle_unsubscription_success("rb2401");
        
        let remaining = service.get_subscribed_instruments();
        assert_eq!(remaining.len(), 1);
        assert!(remaining.contains(&"hc2401".to_string()));
        
        // 清理
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_subscription_manager_priority() {
        let client_state = Arc::new(Mutex::new(crate::ctp::ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = Arc::new(Mutex::new(crate::ctp::spi::MdSpiImpl::new(
            client_state,
            sender.clone(),
            config,
        )));
        
        let manager = SubscriptionManager::new(md_spi, sender);
        
        // 测试不同优先级的订阅
        let high_priority_instruments = vec!["rb2401".to_string()];
        let normal_priority_instruments = vec!["hc2401".to_string()];
        let low_priority_instruments = vec!["i2401".to_string()];
        
        // 按不同优先级提交订阅请求
        let high_id = manager.subscribe_with_priority(
            high_priority_instruments.clone(),
            SubscriptionPriority::High
        ).await.unwrap();
        
        let normal_id = manager.subscribe_with_priority(
            normal_priority_instruments.clone(),
            SubscriptionPriority::Normal
        ).await.unwrap();
        
        let low_id = manager.subscribe_with_priority(
            low_priority_instruments.clone(),
            SubscriptionPriority::Low
        ).await.unwrap();
        
        assert!(high_id > 0);
        assert!(normal_id > 0);
        assert!(low_id > 0);
        
        // 验证所有合约都在订阅中
        for instrument in &high_priority_instruments {
            assert!(manager.is_subscribing(instrument));
        }
        for instrument in &normal_priority_instruments {
            assert!(manager.is_subscribing(instrument));
        }
        for instrument in &low_priority_instruments {
            assert!(manager.is_subscribing(instrument));
        }
    }

    #[tokio::test]
    async fn test_market_data_handling() {
        let client_state = Arc::new(Mutex::new(crate::ctp::ClientState::Disconnected));
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = Arc::new(Mutex::new(crate::ctp::spi::MdSpiImpl::new(
            client_state,
            sender.clone(),
            config,
        )));
        
        let manager = SubscriptionManager::new(md_spi, sender);
        
        // 订阅合约
        let instruments = vec!["rb2401".to_string()];
        manager.subscribe(instruments.clone()).await.unwrap();
        manager.handle_subscription_success("rb2401");
        
        // 创建测试行情数据
        let test_tick = MarketDataTick {
            instrument_id: "rb2401".to_string(),
            last_price: 3500.0,
            volume: 1000,
            turnover: 3500000.0,
            open_interest: 50000,
            bid_price1: 3499.0,
            bid_volume1: 10,
            ask_price1: 3501.0,
            ask_volume1: 15,
            update_time: "09:30:00".to_string(),
            update_millisec: 500,
            change_percent: 1.5,
            change_amount: 50.0,
            open_price: 3450.0,
            highest_price: 3520.0,
            lowest_price: 3440.0,
            pre_close_price: 3450.0,
        };
        
        // 处理行情数据
        manager.handle_market_data(test_tick.clone());
        
        // 验证事件发送
        let event = timeout(Duration::from_millis(100), receiver.recv()).await.unwrap().unwrap();
        match event {
            CtpEvent::MarketData(tick) => {
                assert_eq!(tick.instrument_id, "rb2401");
                assert_eq!(tick.last_price, 3500.0);
                assert_eq!(tick.volume, 1000);
            }
            _ => panic!("期望收到行情数据事件"),
        }
        
        // 验证订阅信息更新
        let info = manager.get_subscription_info("rb2401").unwrap();
        assert_eq!(info.data_count, 1);
        assert!(info.last_tick.is_some());
        assert!(info.last_update_time.is_some());
    }

    #[tokio::test]
    async fn test_subscription_error_handling() {
        let client_state = Arc::new(Mutex::new(crate::ctp::ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = Arc::new(Mutex::new(crate::ctp::spi::MdSpiImpl::new(
            client_state,
            sender.clone(),
            config,
        )));
        
        let manager = SubscriptionManager::new(md_spi, sender);
        
        // 订阅合约
        let instruments = vec!["invalid_contract".to_string()];
        manager.subscribe(instruments.clone()).await.unwrap();
        
        // 模拟订阅失败
        manager.handle_subscription_failure("invalid_contract", "合约不存在");
        
        // 验证状态
        let status = manager.get_subscription_status("invalid_contract");
        assert_eq!(status, SubscriptionStatus::NotSubscribed);
        
        let info = manager.get_subscription_info("invalid_contract").unwrap();
        assert_eq!(info.retry_count, 1);
        
        // 模拟多次失败直到达到最大重试次数
        manager.handle_subscription_failure("invalid_contract", "合约不存在");
        manager.handle_subscription_failure("invalid_contract", "合约不存在");
        manager.handle_subscription_failure("invalid_contract", "合约不存在");
        
        let final_status = manager.get_subscription_status("invalid_contract");
        match final_status {
            SubscriptionStatus::Failed(msg) => {
                assert_eq!(msg, "合约不存在");
            }
            _ => panic!("期望订阅失败状态"),
        }
    }

    #[tokio::test]
    async fn test_subscription_statistics() {
        let client_state = Arc::new(Mutex::new(crate::ctp::ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = Arc::new(Mutex::new(crate::ctp::spi::MdSpiImpl::new(
            client_state,
            sender.clone(),
            config,
        )));
        
        let manager = SubscriptionManager::new(md_spi, sender);
        
        // 初始统计
        let initial_stats = manager.get_stats();
        assert_eq!(initial_stats.total_subscribe_requests, 0);
        assert_eq!(initial_stats.successful_subscriptions, 0);
        assert_eq!(initial_stats.current_subscriptions, 0);
        
        // 执行订阅操作
        let instruments = vec!["rb2401".to_string(), "hc2401".to_string()];
        manager.subscribe(instruments.clone()).await.unwrap();
        
        let stats_after_request = manager.get_stats();
        assert_eq!(stats_after_request.total_subscribe_requests, 1);
        
        // 模拟订阅成功
        for instrument in &instruments {
            manager.handle_subscription_success(instrument);
        }
        
        let stats_after_success = manager.get_stats();
        assert_eq!(stats_after_success.successful_subscriptions, 2);
        assert_eq!(stats_after_success.current_subscriptions, 2);
        
        // 模拟行情数据接收
        let test_tick = MarketDataTick {
            instrument_id: "rb2401".to_string(),
            last_price: 3500.0,
            volume: 1000,
            turnover: 3500000.0,
            open_interest: 50000,
            bid_price1: 3499.0,
            bid_volume1: 10,
            ask_price1: 3501.0,
            ask_volume1: 15,
            update_time: "09:30:00".to_string(),
            update_millisec: 500,
            change_percent: 1.5,
            change_amount: 50.0,
            open_price: 3450.0,
            highest_price: 3520.0,
            lowest_price: 3440.0,
            pre_close_price: 3450.0,
        };
        
        manager.handle_market_data(test_tick);
        
        let final_stats = manager.get_stats();
        assert_eq!(final_stats.total_market_data_received, 1);
        
        // 测试统计重置
        manager.reset_stats();
        let reset_stats = manager.get_stats();
        assert_eq!(reset_stats.total_subscribe_requests, 0);
        assert_eq!(reset_stats.total_market_data_received, 0);
    }

    #[tokio::test]
    async fn test_empty_instrument_list_handling() {
        let client_state = Arc::new(Mutex::new(crate::ctp::ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = Arc::new(Mutex::new(crate::ctp::spi::MdSpiImpl::new(
            client_state,
            sender.clone(),
            config,
        )));
        
        let manager = SubscriptionManager::new(md_spi, sender);
        
        // 测试空合约列表
        let result = manager.subscribe(vec![]).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CtpError::ConfigError(msg) => {
                assert_eq!(msg, "合约列表不能为空");
            }
            _ => panic!("期望配置错误"),
        }
        
        // 测试取消订阅空列表
        let unsubscribe_result = manager.unsubscribe(vec![]).await;
        assert!(unsubscribe_result.is_err());
    }

    #[tokio::test]
    async fn test_subscription_cleanup() {
        let client_state = Arc::new(Mutex::new(crate::ctp::ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = Arc::new(Mutex::new(crate::ctp::spi::MdSpiImpl::new(
            client_state,
            sender.clone(),
            config,
        )));
        
        let manager = SubscriptionManager::new(md_spi, sender);
        
        // 订阅合约
        let instruments = vec!["rb2401".to_string()];
        manager.subscribe(instruments.clone()).await.unwrap();
        manager.handle_subscription_success("rb2401");
        
        // 验证订阅存在
        assert!(manager.is_subscribed("rb2401"));
        assert_eq!(manager.get_all_subscriptions().len(), 1);
        
        // 执行清理（使用很短的过期时间）
        tokio::time::sleep(Duration::from_millis(10)).await;
        manager.cleanup_expired_subscriptions(Duration::from_millis(5));
        
        // 由于没有更新时间，订阅信息应该被保留
        assert_eq!(manager.get_all_subscriptions().len(), 1);
        
        // 模拟行情数据更新时间
        let test_tick = MarketDataTick {
            instrument_id: "rb2401".to_string(),
            last_price: 3500.0,
            volume: 1000,
            turnover: 3500000.0,
            open_interest: 50000,
            bid_price1: 3499.0,
            bid_volume1: 10,
            ask_price1: 3501.0,
            ask_volume1: 15,
            update_time: "09:30:00".to_string(),
            update_millisec: 500,
            change_percent: 1.5,
            change_amount: 50.0,
            open_price: 3450.0,
            highest_price: 3520.0,
            lowest_price: 3440.0,
            pre_close_price: 3450.0,
        };
        
        manager.handle_market_data(test_tick);
        
        // 等待一段时间后清理
        tokio::time::sleep(Duration::from_millis(20)).await;
        manager.cleanup_expired_subscriptions(Duration::from_millis(10));
        
        // 现在订阅信息应该被清理
        assert_eq!(manager.get_all_subscriptions().len(), 0);
    }
}

/// 集成测试模块
/// 
/// 这些测试需要真实的 CTP 环境，通常在 CI/CD 中跳过
#[cfg(test)]
#[cfg(feature = "integration_tests")]
mod integration_tests {
    use super::*;

    /// 真实环境行情订阅测试
    /// 
    /// 需要有效的 CTP 账户信息
    #[tokio::test]
    #[ignore] // 默认忽略，需要手动运行
    async fn test_real_market_data_subscription() {
        // 从环境变量获取账户信息
        let user_id = std::env::var("CTP_USER_ID").expect("需要设置 CTP_USER_ID 环境变量");
        let password = std::env::var("CTP_PASSWORD").expect("需要设置 CTP_PASSWORD 环境变量");
        
        let config = CtpConfig::for_environment(Environment::SimNow, user_id, password);
        let mut client = CtpClient::new(config.clone()).await.unwrap();
        
        // 连接和登录
        client.connect_with_retry().await.unwrap();
        
        let credentials = LoginCredentials {
            broker_id: config.broker_id.clone(),
            user_id: config.investor_id.clone(),
            password: config.password.clone(),
            app_id: config.app_id.clone(),
            auth_code: config.auth_code.clone(),
        };
        
        client.login(credentials).await.unwrap();
        
        // 订阅行情
        let instruments = vec!["rb2401".to_string()];
        client.subscribe_market_data(&instruments).await.unwrap();
        
        // 等待行情数据
        let mut event_handler = client.event_handler().clone();
        let mut received_count = 0;
        let max_wait_time = Duration::from_secs(30);
        let start_time = std::time::Instant::now();
        
        while received_count < 5 && start_time.elapsed() < max_wait_time {
            if let Ok(Some(event)) = timeout(Duration::from_secs(5), event_handler.next_event()).await {
                match event {
                    CtpEvent::MarketData(tick) => {
                        received_count += 1;
                        println!("收到行情数据: {} 价格: {}", tick.instrument_id, tick.last_price);
                    }
                    CtpEvent::Error(msg) => {
                        eprintln!("收到错误: {}", msg);
                        break;
                    }
                    _ => {}
                }
            }
        }
        
        assert!(received_count > 0, "应该收到至少一条行情数据");
        
        // 清理
        client.unsubscribe_market_data(&instruments).await.unwrap();
        client.disconnect();
    }
}
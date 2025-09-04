use crate::ctp::{
    CtpClient, CtpConfig, Environment, QueryService, QueryOptions, QueryType,
    CtpEvent, EventHandler, AccountInfo, Position, TradeRecord, OrderStatus,
    PositionDirection, OrderDirection, OffsetFlag, OrderStatusType,
};
use tokio::time::{sleep, Duration};
use std::sync::Arc;

/// 查询功能测试
/// 
/// 测试 CTP 组件的各种查询功能是否正常工作
#[cfg(test)]
mod tests {
    use super::*;

    /// 测试查询服务创建
    #[tokio::test]
    async fn test_query_service_creation() {
        let config = create_test_config();
        let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();
        
        let query_service = QueryService::new(config, sender);
        
        // 验证初始状态
        assert!(query_service.get_query_state(QueryType::Account).is_none());
        assert!(query_service.get_query_state(QueryType::Positions).is_none());
        assert!(query_service.get_query_state(QueryType::Trades).is_none());
        assert!(query_service.get_query_state(QueryType::Orders).is_none());
        assert!(query_service.get_query_state(QueryType::Settlement).is_none());
    }

    /// 测试查询缓存功能
    #[tokio::test]
    async fn test_query_cache() {
        let config = create_test_config();
        let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();
        
        let query_service = QueryService::new(config, sender);
        
        // 创建测试数据
        let test_account = create_test_account();
        let test_positions = vec![create_test_position()];
        let test_trades = vec![create_test_trade()];
        let test_orders = vec![create_test_order()];
        let test_settlement = "测试结算信息内容".to_string();
        
        // 测试缓存功能
        query_service.handle_event(&CtpEvent::QueryAccountResult(test_account.clone()));
        query_service.handle_event(&CtpEvent::QueryPositionsResult(test_positions.clone()));
        query_service.handle_event(&CtpEvent::QueryTradesResult(test_trades.clone()));
        query_service.handle_event(&CtpEvent::QueryOrdersResult(test_orders.clone()));
        query_service.handle_event(&CtpEvent::QuerySettlementResult(test_settlement.clone()));
        
        // 验证缓存是否生效（这里需要访问私有方法，实际测试中可能需要调整）
        // 由于缓存方法是私有的，这里只能测试公共接口
        
        // 测试清空缓存
        query_service.clear_cache();
        query_service.clear_cache_by_type(QueryType::Account);
    }

    /// 测试客户端查询方法
    #[tokio::test]
    async fn test_client_query_methods() {
        let config = create_test_config();
        
        // 注意：这个测试需要真实的 CTP 连接，在单元测试中可能会失败
        // 在实际环境中，应该使用模拟的 API 管理器
        
        let result = CtpClient::new(config).await;
        if let Ok(mut client) = result {
            // 测试查询方法（这些方法在未连接状态下会返回错误）
            let account_result = client.query_account().await;
            assert!(account_result.is_err()); // 未登录状态应该返回错误
            
            let positions_result = client.query_positions().await;
            assert!(positions_result.is_err()); // 未登录状态应该返回错误
            
            let trades_result = client.query_trades(None).await;
            assert!(trades_result.is_err()); // 未登录状态应该返回错误
            
            let orders_result = client.query_orders(None).await;
            assert!(orders_result.is_err()); // 未登录状态应该返回错误
            
            let settlement_result = client.query_settlement_info(None).await;
            assert!(settlement_result.is_err()); // 未登录状态应该返回错误
            
            let confirm_result = client.confirm_settlement_info().await;
            assert!(confirm_result.is_err()); // 未登录状态应该返回错误
        }
    }

    /// 测试查询选项
    #[test]
    fn test_query_options() {
        let default_options = QueryOptions::default();
        assert!(default_options.use_cache);
        assert!(default_options.cache_ttl.is_none());
        assert!(default_options.timeout_secs.is_none());
        assert!(default_options.instrument_id.is_none());
        assert!(default_options.trading_day.is_none());
        
        let custom_options = QueryOptions {
            use_cache: false,
            cache_ttl: Some(300),
            timeout_secs: Some(60),
            instrument_id: Some("rb2401".to_string()),
            trading_day: Some("20241201".to_string()),
        };
        
        assert!(!custom_options.use_cache);
        assert_eq!(custom_options.cache_ttl, Some(300));
        assert_eq!(custom_options.timeout_secs, Some(60));
        assert_eq!(custom_options.instrument_id, Some("rb2401".to_string()));
        assert_eq!(custom_options.trading_day, Some("20241201".to_string()));
    }

    /// 测试查询类型枚举
    #[test]
    fn test_query_type_enum() {
        let types = vec![
            QueryType::Account,
            QueryType::Positions,
            QueryType::Trades,
            QueryType::Orders,
            QueryType::Settlement,
        ];
        
        // 测试枚举的基本功能
        for query_type in types {
            assert_eq!(query_type, query_type);
            let debug_str = format!("{:?}", query_type);
            assert!(!debug_str.is_empty());
        }
    }

    /// 测试事件处理
    #[tokio::test]
    async fn test_event_handling() {
        let config = create_test_config();
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        
        let query_service = QueryService::new(config, sender.clone());
        
        // 发送测试事件
        let test_events = vec![
            CtpEvent::QueryAccountResult(create_test_account()),
            CtpEvent::QueryPositionsResult(vec![create_test_position()]),
            CtpEvent::QueryTradesResult(vec![create_test_trade()]),
            CtpEvent::QueryOrdersResult(vec![create_test_order()]),
            CtpEvent::QuerySettlementResult("测试结算信息".to_string()),
            CtpEvent::SettlementConfirmed,
        ];
        
        for event in test_events {
            sender.send(event.clone()).unwrap();
            query_service.handle_event(&event);
        }
        
        // 验证事件是否被正确处理
        let mut event_count = 0;
        while let Ok(event) = receiver.try_recv() {
            event_count += 1;
            match event {
                CtpEvent::QueryAccountResult(_) |
                CtpEvent::QueryPositionsResult(_) |
                CtpEvent::QueryTradesResult(_) |
                CtpEvent::QueryOrdersResult(_) |
                CtpEvent::QuerySettlementResult(_) |
                CtpEvent::SettlementConfirmed => {
                    // 事件处理正常
                }
                _ => {
                    panic!("收到意外的事件类型");
                }
            }
        }
        
        assert_eq!(event_count, 6);
    }

    // 辅助函数

    fn create_test_config() -> CtpConfig {
        CtpConfig::for_environment(
            Environment::Sim,
            "test_user".to_string(),
            "test_password".to_string(),
        )
    }

    fn create_test_account() -> AccountInfo {
        AccountInfo {
            account_id: "test_account".to_string(),
            available: 100000.0,
            balance: 120000.0,
            frozen_margin: 5000.0,
            frozen_commission: 100.0,
            curr_margin: 15000.0,
            commission: 50.0,
            close_profit: 1000.0,
            position_profit: 500.0,
            risk_ratio: 12.5,
        }
    }

    fn create_test_position() -> Position {
        Position {
            instrument_id: "rb2401".to_string(),
            direction: PositionDirection::Long,
            total_position: 10,
            yesterday_position: 5,
            today_position: 5,
            open_cost: 35000.0,
            position_cost: 35000.0,
            margin: 7000.0,
            unrealized_pnl: 500.0,
            realized_pnl: 0.0,
        }
    }

    fn create_test_trade() -> TradeRecord {
        TradeRecord {
            trade_id: "test_trade_001".to_string(),
            order_id: "test_order_001".to_string(),
            instrument_id: "rb2401".to_string(),
            direction: OrderDirection::Buy,
            offset_flag: OffsetFlag::Open,
            price: 3500.0,
            volume: 5,
            trade_time: "09:30:15".to_string(),
        }
    }

    fn create_test_order() -> OrderStatus {
        OrderStatus {
            order_id: "test_order_001".to_string(),
            instrument_id: "rb2401".to_string(),
            direction: OrderDirection::Buy,
            offset_flag: OffsetFlag::Open,
            limit_price: 3500.0,
            volume_total_original: 5,
            volume_traded: 5,
            volume_total: 0,
            status: OrderStatusType::AllTraded,
            insert_time: "09:30:10".to_string(),
            update_time: "09:30:15".to_string(),
            status_msg: None,
        }
    }
}

/// 集成测试
/// 
/// 这些测试需要真实的 CTP 环境，通常在 CI/CD 中跳过
#[cfg(test)]
#[cfg(feature = "integration_tests")]
mod integration_tests {
    use super::*;
    use std::env;

    /// 测试真实的查询功能
    /// 
    /// 需要设置环境变量：
    /// - CTP_USER_ID: CTP 用户ID
    /// - CTP_PASSWORD: CTP 密码
    #[tokio::test]
    #[ignore] // 默认忽略，需要手动运行
    async fn test_real_query_functionality() -> Result<(), Box<dyn std::error::Error>> {
        // 从环境变量获取测试凭据
        let user_id = env::var("CTP_USER_ID").expect("请设置 CTP_USER_ID 环境变量");
        let password = env::var("CTP_PASSWORD").expect("请设置 CTP_PASSWORD 环境变量");
        
        // 创建配置
        let config = CtpConfig::for_environment(
            Environment::Sim,
            user_id,
            password,
        );
        
        // 创建客户端
        let mut client = CtpClient::new(config.clone()).await?;
        
        // 创建查询服务
        let query_service = QueryService::new(config.clone(), client.event_sender());
        
        // 连接和登录
        client.connect_with_retry().await?;
        
        let credentials = crate::ctp::LoginCredentials {
            broker_id: config.broker_id.clone(),
            user_id: config.investor_id.clone(),
            password: config.password.clone(),
            app_id: config.app_id.clone(),
            auth_code: config.auth_code.clone(),
        };
        
        client.login(credentials).await?;
        
        // 等待登录完成
        sleep(Duration::from_secs(3)).await;
        
        // 测试各种查询
        println!("测试账户查询...");
        client.query_account().await?;
        sleep(Duration::from_secs(2)).await;
        
        println!("测试持仓查询...");
        client.query_positions().await?;
        sleep(Duration::from_secs(2)).await;
        
        println!("测试成交记录查询...");
        client.query_trades(None).await?;
        sleep(Duration::from_secs(2)).await;
        
        println!("测试报单记录查询...");
        client.query_orders(None).await?;
        sleep(Duration::from_secs(2)).await;
        
        println!("测试结算信息查询...");
        client.query_settlement_info(None).await?;
        sleep(Duration::from_secs(3)).await;
        
        println!("测试结算信息确认...");
        client.confirm_settlement_info().await?;
        sleep(Duration::from_secs(2)).await;
        
        // 检查查询状态
        let all_states = query_service.get_all_query_states();
        for (query_type, state) in all_states {
            println!("查询类型: {:?}, 查询次数: {}, 最后查询: {:?}", 
                query_type, state.query_count, state.last_query_time);
        }
        
        // 断开连接
        client.disconnect();
        
        Ok(())
    }
}
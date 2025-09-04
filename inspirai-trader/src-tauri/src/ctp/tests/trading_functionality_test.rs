use crate::ctp::{
    CtpConfig, Environment,
    models::{OrderRequest, OrderDirection, OffsetFlag, OrderType, TimeCondition},
    trading_service::TradingService,
    utils::DataConverter,
};
use std::sync::Arc;
use tokio::sync::mpsc;

/// 测试交易功能的核心逻辑
/// 
/// 这些测试验证交易服务的各项功能，包括：
/// 1. 订单验证
/// 2. 数据转换
/// 3. 订单管理
/// 4. 错误处理
#[cfg(test)]
mod tests {
    use super::*;

    /// 创建测试用的交易服务
    fn create_test_trading_service() -> TradingService {
        let config = CtpConfig::for_environment(
            Environment::SimNow,
            "test_user".to_string(),
            "test_password".to_string(),
        );
        
        let client_state = Arc::new(std::sync::Mutex::new(
            crate::ctp::ClientState::LoggedIn
        ));
        
        let (event_sender, _) = mpsc::unbounded_channel();
        
        TradingService::new(config, client_state, event_sender)
    }

    /// 创建测试订单
    fn create_test_order() -> OrderRequest {
        OrderRequest {
            instrument_id: "rb2501".to_string(),
            direction: OrderDirection::Buy,
            offset_flag: OffsetFlag::Open,
            price: 3500.0,
            volume: 1,
            order_type: OrderType::Limit,
            time_condition: TimeCondition::GFD,
        }
    }

    #[tokio::test]
    async fn test_order_validation() {
        let trading_service = create_test_trading_service();
        
        // 测试有效订单
        let valid_order = create_test_order();
        let result = trading_service.submit_order(valid_order, None).await;
        assert!(result.is_ok(), "有效订单应该通过验证");
        
        // 测试无效订单 - 空合约代码
        let invalid_order = OrderRequest {
            instrument_id: "".to_string(),
            direction: OrderDirection::Buy,
            offset_flag: OffsetFlag::Open,
            price: 3500.0,
            volume: 1,
            order_type: OrderType::Limit,
            time_condition: TimeCondition::GFD,
        };
        
        let result = trading_service.submit_order(invalid_order, None).await;
        assert!(result.is_err(), "空合约代码的订单应该被拒绝");
        
        // 测试无效订单 - 零数量
        let invalid_order = OrderRequest {
            instrument_id: "rb2501".to_string(),
            direction: OrderDirection::Buy,
            offset_flag: OffsetFlag::Open,
            price: 3500.0,
            volume: 0,
            order_type: OrderType::Limit,
            time_condition: TimeCondition::GFD,
        };
        
        let result = trading_service.submit_order(invalid_order, None).await;
        assert!(result.is_err(), "零数量的订单应该被拒绝");
        
        // 测试无效订单 - 负价格的限价单
        let invalid_order = OrderRequest {
            instrument_id: "rb2501".to_string(),
            direction: OrderDirection::Buy,
            offset_flag: OffsetFlag::Open,
            price: -100.0,
            volume: 1,
            order_type: OrderType::Limit,
            time_condition: TimeCondition::GFD,
        };
        
        let result = trading_service.submit_order(invalid_order, None).await;
        assert!(result.is_err(), "负价格的限价单应该被拒绝");
    }

    #[tokio::test]
    async fn test_order_management() {
        let trading_service = create_test_trading_service();
        
        // 提交订单
        let order = create_test_order();
        let order_id = trading_service.submit_order(order, None).await
            .expect("订单提交应该成功");
        
        // 查询订单
        let order_info = trading_service.query_order(&order_id).await;
        assert!(order_info.is_ok(), "应该能够查询到刚提交的订单");
        
        let order_status = order_info.unwrap();
        assert_eq!(order_status.order_id, order_id, "订单ID应该匹配");
        assert_eq!(order_status.instrument_id, "rb2501", "合约代码应该匹配");
        
        // 查询活动订单
        let active_orders = trading_service.query_active_orders().await
            .expect("查询活动订单应该成功");
        assert!(!active_orders.is_empty(), "应该有活动订单");
        
        // 撤销订单
        let cancel_result = trading_service.cancel_order(&order_id, None).await;
        assert!(cancel_result.is_ok(), "撤单应该成功");
    }

    #[tokio::test]
    async fn test_data_conversion() {
        // 测试订单请求转换
        let order = create_test_order();
        let ctp_order = DataConverter::convert_order_request(
            &order,
            "9999",
            "test_user",
            "000001",
        );
        
        assert!(ctp_order.is_ok(), "订单转换应该成功");
        
        let ctp_order = ctp_order.unwrap();
        assert_eq!(ctp_order.LimitPrice, 3500.0, "价格应该正确转换");
        assert_eq!(ctp_order.VolumeTotalOriginal, 1, "数量应该正确转换");
        assert_eq!(ctp_order.Direction, '0' as i8, "买入方向应该正确转换");
        assert_eq!(ctp_order.CombOffsetFlag[0], '0' as i8, "开仓标志应该正确转换");
    }

    #[tokio::test]
    async fn test_trading_statistics() {
        let trading_service = create_test_trading_service();
        
        // 初始统计应该为零
        let initial_stats = trading_service.get_stats();
        assert_eq!(initial_stats.total_orders, 0, "初始订单数应该为0");
        assert_eq!(initial_stats.success_orders, 0, "初始成功订单数应该为0");
        
        // 提交几个订单
        for i in 0..3 {
            let mut order = create_test_order();
            order.instrument_id = format!("rb250{}", i + 1);
            let _ = trading_service.submit_order(order, None).await;
        }
        
        // 检查统计更新
        let updated_stats = trading_service.get_stats();
        assert_eq!(updated_stats.total_orders, 3, "应该有3个订单");
    }

    #[tokio::test]
    async fn test_query_operations() {
        let trading_service = create_test_trading_service();
        
        // 测试查询账户信息
        let account_result = trading_service.query_account(None).await;
        // 由于没有真实的API连接，这里可能返回错误或默认值
        // 主要测试方法调用不会panic
        
        // 测试查询持仓信息
        let positions_result = trading_service.query_positions(None).await;
        assert!(positions_result.is_ok(), "查询持仓应该不会出错");
        
        // 测试查询成交记录
        let trades_result = trading_service.query_trades(None, None).await;
        assert!(trades_result.is_ok(), "查询成交记录应该不会出错");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let trading_service = create_test_trading_service();
        
        // 测试查询不存在的订单
        let result = trading_service.query_order("non_existent_order").await;
        assert!(result.is_err(), "查询不存在的订单应该返回错误");
        
        // 测试撤销不存在的订单
        let result = trading_service.cancel_order("non_existent_order", None).await;
        assert!(result.is_err(), "撤销不存在的订单应该返回错误");
    }

    #[test]
    fn test_order_status_validation() {
        let trading_service = create_test_trading_service();
        
        // 测试可撤销状态
        let order_status = crate::ctp::models::OrderStatus {
            order_id: "test_order".to_string(),
            instrument_id: "rb2501".to_string(),
            direction: OrderDirection::Buy,
            offset_flag: OffsetFlag::Open,
            limit_price: 3500.0,
            volume_total_original: 1,
            volume_traded: 0,
            volume_total: 1,
            status: crate::ctp::models::OrderStatusType::NoTradeQueueing,
            insert_time: "09:30:00".to_string(),
            update_time: "09:30:00".to_string(),
            status_msg: None,
        };
        
        assert!(trading_service.can_cancel(&order_status), "排队中的订单应该可以撤销");
        
        // 测试不可撤销状态
        let mut completed_order = order_status.clone();
        completed_order.status = crate::ctp::models::OrderStatusType::AllTraded;
        
        assert!(!trading_service.can_cancel(&completed_order), "已成交的订单不应该可以撤销");
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let trading_service = Arc::new(create_test_trading_service());
        
        // 并发提交多个订单
        let mut handles = vec![];
        
        for i in 0..5 {
            let service = trading_service.clone();
            let handle = tokio::spawn(async move {
                let mut order = create_test_order();
                order.instrument_id = format!("rb250{}", i + 1);
                service.submit_order(order, None).await
            });
            handles.push(handle);
        }
        
        // 等待所有订单完成
        let mut success_count = 0;
        for handle in handles {
            if let Ok(result) = handle.await {
                if result.is_ok() {
                    success_count += 1;
                }
            }
        }
        
        assert_eq!(success_count, 5, "所有并发订单都应该成功");
        
        // 检查最终统计
        let final_stats = trading_service.get_stats();
        assert_eq!(final_stats.total_orders, 5, "应该有5个订单");
    }
}
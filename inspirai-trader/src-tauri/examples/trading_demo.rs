use inspirai_trader_lib::ctp::{
    CtpClient, CtpConfig, Environment,
    models::{OrderRequest, OrderDirection, OffsetFlag, OrderType, TimeCondition, LoginCredentials},
    trading_service::TradingService,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn};

/// 交易功能演示程序
/// 
/// 演示如何使用真实的 CTP API 进行交易操作：
/// 1. 连接和登录
/// 2. 提交订单
/// 3. 撤销订单
/// 4. 查询账户和持仓
/// 5. 查询成交记录
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();

    info!("启动 CTP 交易功能演示程序");

    // 从环境变量获取登录凭据
    let user_id = std::env::var("CTP_USER_ID")
        .unwrap_or_else(|_| "your_user_id".to_string());
    let password = std::env::var("CTP_PASSWORD")
        .unwrap_or_else(|_| "your_password".to_string());

    if user_id == "your_user_id" || password == "your_password" {
        error!("请设置环境变量 CTP_USER_ID 和 CTP_PASSWORD");
        error!("例如: export CTP_USER_ID=your_user_id");
        error!("例如: export CTP_PASSWORD=your_password");
        return Ok(());
    }

    // 创建 CTP 配置（使用 SimNow 环境）
    let config = CtpConfig::for_environment(Environment::SimNow, user_id.clone(), password.clone());
    
    info!("使用配置:");
    info!("  环境: {:?}", config.environment);
    info!("  经纪商: {}", config.broker_id);
    info!("  用户: {}", user_id);
    info!("  行情服务器: {}", config.md_front_addr);
    info!("  交易服务器: {}", config.trader_front_addr);

    // 创建 CTP 客户端
    let mut client = CtpClient::new(config.clone()).await?;

    // 连接到 CTP 服务器
    info!("连接到 CTP 服务器...");
    if let Err(e) = client.connect_with_retry().await {
        error!("连接失败: {}", e);
        return Err(e.into());
    }

    // 用户登录
    info!("用户登录...");
    let credentials = LoginCredentials {
        broker_id: config.broker_id.clone(),
        user_id: user_id.clone(),
        password: password.clone(),
        app_id: config.app_id.clone(),
        auth_code: config.auth_code.clone(),
    };

    let login_response = client.login(credentials).await?;
    info!("登录成功: {:?}", login_response);

    // 等待一段时间确保登录完成
    sleep(Duration::from_secs(2)).await;

    // 创建交易服务
    let client_state = Arc::new(std::sync::Mutex::new(
        inspirai_trader_lib::ctp::ClientState::LoggedIn
    ));
    let (event_sender, mut event_receiver) = tokio::sync::mpsc::unbounded_channel();
    
    let trading_service = TradingService::new(
        config.clone(),
        client_state,
        event_sender,
    );

    // 启动事件处理任务
    let event_handler = tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            info!("收到事件: {:?}", event);
        }
    });

    // 演示交易功能
    demo_trading_operations(&client, &trading_service).await?;

    // 断开连接
    info!("断开连接...");
    // client.disconnect();

    // 等待事件处理完成
    event_handler.abort();

    info!("演示程序结束");
    Ok(())
}

/// 演示交易操作
async fn demo_trading_operations(
    client: &CtpClient,
    trading_service: &TradingService,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== 开始交易功能演示 ===");

    // 1. 查询账户信息
    info!("1. 查询账户信息");
    match trading_service.query_account(None).await {
        Ok(account) => {
            info!("账户信息: 余额={:.2}, 可用={:.2}, 风险度={:.2}%", 
                account.balance, account.available, account.risk_ratio);
        }
        Err(e) => warn!("查询账户信息失败: {}", e),
    }

    sleep(Duration::from_secs(1)).await;

    // 2. 查询持仓信息
    info!("2. 查询持仓信息");
    match trading_service.query_positions(None).await {
        Ok(positions) => {
            if positions.is_empty() {
                info!("当前无持仓");
            } else {
                for position in positions {
                    info!("持仓: {} {} {}手 成本={:.2} 盈亏={:.2}", 
                        position.instrument_id, position.direction, 
                        position.total_position, position.position_cost, position.unrealized_pnl);
                }
            }
        }
        Err(e) => warn!("查询持仓信息失败: {}", e),
    }

    sleep(Duration::from_secs(1)).await;

    // 3. 提交测试订单（使用一个不太可能成交的价格）
    info!("3. 提交测试订单");
    let test_order = OrderRequest {
        instrument_id: "rb2501".to_string(), // 螺纹钢主力合约
        direction: OrderDirection::Buy,
        offset_flag: OffsetFlag::Open,
        price: 3000.0, // 使用一个较低的价格，不太可能成交
        volume: 1,
        order_type: OrderType::Limit,
        time_condition: TimeCondition::GFD,
    };

    match trading_service.submit_order(test_order, None).await {
        Ok(order_id) => {
            info!("订单提交成功，订单ID: {}", order_id);
            
            // 等待一段时间
            sleep(Duration::from_secs(2)).await;
            
            // 4. 查询订单状态
            info!("4. 查询订单状态");
            match trading_service.query_order(&order_id).await {
                Ok(order_status) => {
                    info!("订单状态: {} {:?} {}手@{:.2} 状态={:?}", 
                        order_status.instrument_id, order_status.direction,
                        order_status.volume_total_original, order_status.limit_price,
                        order_status.status);
                }
                Err(e) => warn!("查询订单状态失败: {}", e),
            }
            
            sleep(Duration::from_secs(1)).await;
            
            // 5. 撤销订单
            info!("5. 撤销订单");
            match trading_service.cancel_order(&order_id, None).await {
                Ok(_) => {
                    info!("撤单请求已发送");
                    
                    // 等待撤单完成
                    sleep(Duration::from_secs(2)).await;
                    
                    // 再次查询订单状态
                    match trading_service.query_order(&order_id).await {
                        Ok(order_status) => {
                            info!("撤单后订单状态: {:?}", order_status.status);
                        }
                        Err(e) => warn!("查询撤单后订单状态失败: {}", e),
                    }
                }
                Err(e) => warn!("撤单失败: {}", e),
            }
        }
        Err(e) => warn!("订单提交失败: {}", e),
    }

    sleep(Duration::from_secs(1)).await;

    // 6. 查询成交记录
    info!("6. 查询成交记录");
    match trading_service.query_trades(None, None).await {
        Ok(trades) => {
            if trades.is_empty() {
                info!("当前无成交记录");
            } else {
                for trade in trades {
                    info!("成交: {} {} {}手@{:.2} 时间={}", 
                        trade.instrument_id, trade.direction, 
                        trade.volume, trade.price, trade.trade_time);
                }
            }
        }
        Err(e) => warn!("查询成交记录失败: {}", e),
    }

    // 7. 查询活动订单
    info!("7. 查询活动订单");
    match trading_service.query_active_orders().await {
        Ok(orders) => {
            if orders.is_empty() {
                info!("当前无活动订单");
            } else {
                for order in orders {
                    info!("活动订单: {} {} {}手@{:.2} 状态={:?}", 
                        order.instrument_id, order.direction,
                        order.volume_total_original, order.limit_price, order.status);
                }
            }
        }
        Err(e) => warn!("查询活动订单失败: {}", e),
    }

    // 8. 获取交易统计
    info!("8. 获取交易统计");
    let stats = trading_service.get_stats();
    info!("交易统计: 总订单={}, 成功={}, 失败={}, 成交={}, 成交额={:.2}", 
        stats.total_orders, stats.success_orders, stats.failed_orders,
        stats.total_trades, stats.today_turnover);

    info!("=== 交易功能演示完成 ===");
    Ok(())
}
use inspirai_trader_lib::ctp::{
    client::CtpClient,
    config::CtpConfig,
    events::CtpEvent,
    models::LoginCredentials,
    Environment,
};
use tokio::time::{timeout, Duration};
use tracing::{info, error, warn};

/// 行情订阅示例
/// 
/// 演示如何使用真实的 CTP API 进行行情数据订阅
/// 严格使用 ctp2rs 库，禁止任何模拟实现
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();

    info!("开始行情订阅示例");

    // 创建 SimNow 环境配置
    let config = CtpConfig::for_environment(
        Environment::SimNow,
        "your_user_id".to_string(),    // 替换为实际的用户ID
        "your_password".to_string(),   // 替换为实际的密码
    );

    // 创建 CTP 客户端
    let mut client = CtpClient::new(config.clone()).await?;
    info!("CTP 客户端创建成功");

    // 连接到 CTP 服务器
    info!("连接到 CTP 服务器...");
    match client.connect_with_retry().await {
        Ok(_) => info!("CTP 服务器连接成功"),
        Err(e) => {
            error!("CTP 服务器连接失败: {}", e);
            return Err(e.into());
        }
    }

    // 用户登录
    let credentials = LoginCredentials {
        broker_id: config.broker_id.clone(),
        user_id: config.investor_id.clone(),
        password: config.password.clone(),
        app_id: config.app_id.clone(),
        auth_code: config.auth_code.clone(),
    };

    info!("用户登录...");
    match client.login(credentials).await {
        Ok(login_response) => {
            info!("用户登录成功");
            info!("交易日: {}", login_response.trading_day);
            info!("登录时间: {}", login_response.login_time);
        }
        Err(e) => {
            error!("用户登录失败: {}", e);
            return Err(e.into());
        }
    }

    // 订阅行情数据
    let instruments = vec![
        "rb2401".to_string(),  // 螺纹钢主力合约
        "hc2401".to_string(),  // 热卷主力合约
        "i2401".to_string(),   // 铁矿石主力合约
    ];

    info!("订阅行情数据，合约: {:?}", instruments);
    match client.subscribe_market_data(&instruments).await {
        Ok(_) => info!("行情订阅请求发送成功"),
        Err(e) => {
            error!("行情订阅失败: {}", e);
            return Err(e.into());
        }
    }

    // 启动事件处理循环
    info!("开始接收行情数据...");
    let mut event_handler = client.event_handler().clone();
    let mut market_data_count = 0;
    let max_market_data = 100; // 接收100条行情数据后退出

    // 设置超时时间
    let event_timeout = Duration::from_secs(60);
    let start_time = std::time::Instant::now();

    while market_data_count < max_market_data {
        match timeout(event_timeout, event_handler.next_event()).await {
            Ok(Some(event)) => {
                match event {
                    CtpEvent::MarketData(tick) => {
                        market_data_count += 1;
                        info!(
                            "行情数据 #{}: {} 最新价: {:.2} 成交量: {} 时间: {}",
                            market_data_count,
                            tick.instrument_id,
                            tick.last_price,
                            tick.volume,
                            tick.update_time
                        );

                        // 显示更多行情信息
                        if market_data_count % 10 == 0 {
                            info!(
                                "详细行情 - 买一价: {:.2} 买一量: {} 卖一价: {:.2} 卖一量: {} 涨跌幅: {:.2}%",
                                tick.bid_price1,
                                tick.bid_volume1,
                                tick.ask_price1,
                                tick.ask_volume1,
                                tick.change_percent
                            );
                        }
                    }
                    CtpEvent::Connected => {
                        info!("收到连接成功事件");
                    }
                    CtpEvent::Disconnected => {
                        warn!("收到连接断开事件");
                        break;
                    }
                    CtpEvent::LoginSuccess(response) => {
                        info!("收到登录成功事件: {}", response.user_id);
                    }
                    CtpEvent::LoginFailed(msg) => {
                        error!("收到登录失败事件: {}", msg);
                        break;
                    }
                    CtpEvent::Error(msg) => {
                        error!("收到错误事件: {}", msg);
                    }
                    _ => {
                        // 其他事件
                    }
                }
            }
            Ok(None) => {
                warn!("事件通道已关闭");
                break;
            }
            Err(_) => {
                warn!("等待事件超时");
                break;
            }
        }

        // 检查总运行时间
        if start_time.elapsed() > Duration::from_secs(300) {
            info!("示例运行时间达到5分钟，退出");
            break;
        }
    }

    // 取消订阅
    info!("取消行情订阅...");
    match client.unsubscribe_market_data(&instruments).await {
        Ok(_) => info!("行情取消订阅成功"),
        Err(e) => error!("行情取消订阅失败: {}", e),
    }

    // 断开连接
    info!("断开 CTP 连接...");
    client.disconnect();

    info!("行情订阅示例完成，共接收 {} 条行情数据", market_data_count);
    Ok(())
}

/// 订阅状态管理示例
async fn subscription_management_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("开始订阅状态管理示例");

    let config = CtpConfig::for_environment(
        Environment::SimNow,
        "your_user_id".to_string(),
        "your_password".to_string(),
    );

    let mut client = CtpClient::new(config.clone()).await?;

    // 连接和登录
    client.connect_with_retry().await?;
    let credentials = LoginCredentials {
        broker_id: config.broker_id.clone(),
        user_id: config.investor_id.clone(),
        password: config.password.clone(),
        app_id: config.app_id.clone(),
        auth_code: config.auth_code.clone(),
    };
    client.login(credentials).await?;

    // 分批订阅合约
    let batch1 = vec!["rb2401".to_string(), "hc2401".to_string()];
    let batch2 = vec!["i2401".to_string(), "j2401".to_string()];

    info!("订阅第一批合约: {:?}", batch1);
    client.subscribe_market_data(&batch1).await?;

    // 等待一段时间
    tokio::time::sleep(Duration::from_secs(5)).await;

    info!("订阅第二批合约: {:?}", batch2);
    client.subscribe_market_data(&batch2).await?;

    // 检查订阅状态
    let subscribed = client.get_subscribed_instruments();
    info!("当前已订阅合约: {:?}", subscribed);

    // 等待一段时间
    tokio::time::sleep(Duration::from_secs(10)).await;

    // 取消部分订阅
    info!("取消第一批合约订阅: {:?}", batch1);
    client.unsubscribe_market_data(&batch1).await?;

    let remaining = client.get_subscribed_instruments();
    info!("剩余订阅合约: {:?}", remaining);

    // 清理
    client.unsubscribe_market_data(&batch2).await?;
    client.disconnect();

    info!("订阅状态管理示例完成");
    Ok(())
}

/// 错误处理示例
async fn error_handling_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("开始错误处理示例");

    let config = CtpConfig::for_environment(
        Environment::SimNow,
        "invalid_user".to_string(),    // 故意使用无效用户
        "invalid_password".to_string(), // 故意使用无效密码
    );

    let mut client = CtpClient::new(config.clone()).await?;

    // 尝试连接（可能失败）
    match client.connect_with_retry().await {
        Ok(_) => {
            info!("连接成功");
            
            // 尝试登录（应该失败）
            let credentials = LoginCredentials {
                broker_id: config.broker_id.clone(),
                user_id: config.investor_id.clone(),
                password: config.password.clone(),
                app_id: config.app_id.clone(),
                auth_code: config.auth_code.clone(),
            };

            match client.login(credentials).await {
                Ok(_) => info!("登录成功（意外）"),
                Err(e) => {
                    error!("登录失败（预期）: {}", e);
                    
                    // 演示错误恢复
                    info!("尝试错误恢复...");
                    match client.handle_auth_failure(&e.to_string()).await {
                        Ok(_) => info!("错误恢复成功"),
                        Err(recovery_error) => error!("错误恢复失败: {}", recovery_error),
                    }
                }
            }
        }
        Err(e) => {
            error!("连接失败: {}", e);
        }
    }

    client.disconnect();
    info!("错误处理示例完成");
    Ok(())
}
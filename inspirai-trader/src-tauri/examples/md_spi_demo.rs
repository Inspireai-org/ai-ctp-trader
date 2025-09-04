use inspirai_trader_lib::ctp::{
    CtpConfig, Environment, MdSpiImpl, MarketDataManager, 
    ClientState, CtpEvent, PriceChangeFilter, VolumeFilter
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

/// MdSpi 演示程序
/// 
/// 展示如何使用 MdSpiImpl 和 MarketDataManager 处理行情数据
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    println!("=== CTP 行情 SPI 演示程序 ===");
    
    // 创建配置
    let config = CtpConfig {
        environment: Environment::SimNow,
        broker_id: "9999".to_string(),
        investor_id: "demo_user".to_string(),
        password: "demo_pass".to_string(),
        app_id: "demo_app".to_string(),
        auth_code: "0000000000000000".to_string(),
        md_front_addr: "tcp://180.168.146.187:10131".to_string(),
        trader_front_addr: "tcp://180.168.146.187:10130".to_string(),
        flow_path: "./demo_flow".to_string(),
        md_dynlib_path: None,
        td_dynlib_path: None,
        timeout_secs: 30,
        reconnect_interval_secs: 5,
        max_reconnect_attempts: 3,
    };
    
    println!("配置信息:");
    println!("  环境: {:?}", config.environment);
    println!("  经纪商: {}", config.broker_id);
    println!("  用户: {}", config.investor_id);
    println!("  行情服务器: {}", config.md_front_addr);
    
    // 创建事件通道
    let (event_sender, mut event_receiver) = mpsc::unbounded_channel();
    
    // 创建客户端状态
    let client_state = Arc::new(Mutex::new(ClientState::Disconnected));
    
    // 创建 MdSpi 实例
    let md_spi = Arc::new(Mutex::new(MdSpiImpl::new(
        client_state.clone(),
        event_sender.clone(),
        config.clone(),
    )));
    
    println!("\n=== 创建行情数据管理器 ===");
    
    // 创建行情数据管理器
    let market_data_manager = MarketDataManager::new(
        md_spi.clone(),
        event_sender.clone(),
    );
    
    // 添加数据过滤器
    println!("添加数据过滤器...");
    market_data_manager.add_filter(Box::new(PriceChangeFilter::new(0.1))); // 0.1% 价格变动过滤器
    market_data_manager.add_filter(Box::new(VolumeFilter::new(10))); // 最小成交量 10 手
    
    // 模拟连接过程
    println!("\n=== 模拟连接和登录过程 ===");
    
    // 模拟前置连接
    {
        let mut spi = md_spi.lock().unwrap();
        spi.on_front_connected();
    }
    
    // 等待连接事件
    if let Some(event) = event_receiver.recv().await {
        match event {
            CtpEvent::Connected => println!("✓ 前置连接成功"),
            _ => println!("✗ 收到意外事件: {:?}", event),
        }
    }
    
    // 模拟登录成功
    sleep(Duration::from_millis(100)).await;
    
    // 这里应该模拟登录响应，但由于结构体定义的限制，我们直接发送登录成功事件
    let login_response = inspirai_trader_lib::ctp::LoginResponse {
        trading_day: "20241203".to_string(),
        login_time: "09:30:00".to_string(),
        broker_id: config.broker_id.clone(),
        user_id: config.investor_id.clone(),
        system_name: "CTP演示系统".to_string(),
        front_id: 1,
        session_id: 1,
        max_order_ref: "1".to_string(),
    };
    
    event_sender.send(CtpEvent::LoginSuccess(login_response))?;
    
    if let Some(event) = event_receiver.recv().await {
        match event {
            CtpEvent::LoginSuccess(response) => {
                println!("✓ 登录成功");
                println!("  交易日: {}", response.trading_day);
                println!("  登录时间: {}", response.login_time);
                println!("  系统名称: {}", response.system_name);
            }
            _ => println!("✗ 收到意外事件: {:?}", event),
        }
    }
    
    println!("\n=== 订阅行情数据 ===");
    
    // 订阅行情数据
    let instruments = vec![
        "rb2405".to_string(),
        "hc2405".to_string(),
        "i2405".to_string(),
    ];
    
    println!("订阅合约: {:?}", instruments);
    market_data_manager.subscribe_market_data(&instruments).await?;
    
    let subscribed = market_data_manager.get_subscribed_instruments();
    println!("已订阅合约数量: {}", subscribed.len());
    for instrument in &subscribed {
        println!("  - {}", instrument);
    }
    
    println!("\n=== 模拟行情数据推送 ===");
    
    // 模拟行情数据
    let sample_ticks = vec![
        create_sample_tick("rb2405", 3500.0, 100, "09:30:01"),
        create_sample_tick("rb2405", 3501.0, 150, "09:30:02"),
        create_sample_tick("hc2405", 3200.0, 80, "09:30:01"),
        create_sample_tick("hc2405", 3205.0, 120, "09:30:03"),
        create_sample_tick("i2405", 800.0, 200, "09:30:02"),
    ];
    
    // 处理模拟行情数据
    for tick in sample_ticks {
        println!("处理行情数据: {} @ {} (成交量: {})", 
            tick.instrument_id, tick.last_price, tick.volume);
        market_data_manager.handle_market_data(tick);
    }
    
    // 接收并显示事件
    println!("\n=== 接收行情事件 ===");
    let mut event_count = 0;
    let max_events = 10;
    
    while event_count < max_events {
        tokio::select! {
            event = event_receiver.recv() => {
                if let Some(event) = event {
                    match event {
                        CtpEvent::MarketData(tick) => {
                            println!("📈 行情数据: {} 最新价: {} 成交量: {} 时间: {}", 
                                tick.instrument_id, tick.last_price, tick.volume, tick.update_time);
                            event_count += 1;
                        }
                        _ => {
                            println!("📨 其他事件: {:?}", event);
                        }
                    }
                } else {
                    break;
                }
            }
            _ = sleep(Duration::from_millis(100)) => {
                // 超时，继续等待
                if event_count == 0 {
                    println!("等待行情事件...");
                }
            }
        }
        
        if event_count >= max_events {
            break;
        }
    }
    
    println!("\n=== 显示统计信息 ===");
    
    let stats = market_data_manager.get_stats();
    println!("统计信息:");
    println!("  总接收: {}", stats.total_received);
    println!("  总过滤: {}", stats.total_filtered);
    println!("  总发送: {}", stats.total_sent);
    println!("  按合约统计:");
    for (instrument, count) in &stats.by_instrument {
        println!("    {}: {}", instrument, count);
    }
    
    // 显示缓存的行情数据
    println!("\n=== 缓存的行情数据 ===");
    let cached_data = market_data_manager.get_all_cached_market_data();
    for (instrument, tick) in cached_data {
        println!("  {}: 最新价 {} @ {}", instrument, tick.last_price, tick.update_time);
    }
    
    println!("\n=== 取消订阅测试 ===");
    
    // 取消部分订阅
    market_data_manager.unsubscribe_market_data(&vec!["rb2405".to_string()]).await?;
    
    let remaining = market_data_manager.get_subscribed_instruments();
    println!("剩余订阅合约: {:?}", remaining);
    
    println!("\n=== 演示完成 ===");
    println!("MdSpi 回调处理功能演示完成！");
    
    Ok(())
}

/// 创建示例行情数据
fn create_sample_tick(instrument_id: &str, price: f64, volume: i64, time: &str) -> inspirai_trader_lib::ctp::MarketDataTick {
    inspirai_trader_lib::ctp::MarketDataTick {
        instrument_id: instrument_id.to_string(),
        last_price: price,
        volume,
        turnover: price * volume as f64,
        open_interest: 10000,
        bid_price1: price - 1.0,
        bid_volume1: 50,
        ask_price1: price + 1.0,
        ask_volume1: 50,
        update_time: time.to_string(),
        update_millisec: 0,
        change_percent: 0.5,
        change_amount: price * 0.005,
        open_price: price - 2.0,
        highest_price: price + 5.0,
        lowest_price: price - 5.0,
        pre_close_price: price - 1.0,
    }
}
use inspirai_trader::ctp::{
    CtpClient, CtpConfig, Environment, QueryService, QueryOptions, QueryType,
    CtpEvent, EventHandler, DefaultEventListener, EventListener,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn};

/// 查询功能演示程序
/// 
/// 演示如何使用 CTP 组件的查询功能：
/// 1. 账户资金查询
/// 2. 持仓信息查询
/// 3. 成交记录查询
/// 4. 结算单查询和确认
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("=== CTP 查询功能演示程序 ===");

    // 创建配置（使用 SimNow 环境）
    let config = CtpConfig::for_environment(
        Environment::Sim,
        "your_user_id".to_string(),
        "your_password".to_string(),
    );

    // 创建 CTP 客户端
    let mut client = CtpClient::new(config.clone()).await?;
    
    // 创建查询服务
    let query_service = QueryService::new(
        config.clone(),
        client.event_sender(),
    );

    // 创建事件监听器
    let event_listener = Arc::new(QueryEventListener::new());
    
    info!("1. 连接到 CTP 服务器...");
    client.connect_with_retry().await?;

    info!("2. 用户登录...");
    let credentials = crate::ctp::LoginCredentials {
        broker_id: config.broker_id.clone(),
        user_id: config.investor_id.clone(),
        password: config.password.clone(),
        app_id: config.app_id.clone(),
        auth_code: config.auth_code.clone(),
    };
    
    let login_response = client.login(credentials).await?;
    info!("登录成功: {}", login_response.user_id);

    // 等待一段时间确保登录完成
    sleep(Duration::from_secs(2)).await;

    info!("3. 开始查询演示...");

    // 演示账户查询
    demo_account_query(&mut client, &query_service).await?;
    
    // 演示持仓查询
    demo_positions_query(&mut client, &query_service).await?;
    
    // 演示成交记录查询
    demo_trades_query(&mut client, &query_service).await?;
    
    // 演示报单记录查询
    demo_orders_query(&mut client, &query_service).await?;
    
    // 演示结算信息查询
    demo_settlement_query(&mut client, &query_service).await?;

    info!("4. 查询演示完成");

    // 处理事件
    let mut event_handler = client.event_handler().clone();
    tokio::spawn(async move {
        while let Some(event) = event_handler.next_event().await {
            query_service.handle_event(&event);
            handle_event(&event, &event_listener);
        }
    });

    // 保持连接一段时间
    sleep(Duration::from_secs(10)).await;

    info!("5. 断开连接");
    client.disconnect();

    Ok(())
}

/// 演示账户查询
async fn demo_account_query(
    client: &mut CtpClient,
    query_service: &QueryService,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("--- 账户查询演示 ---");
    
    // 发起账户查询
    client.query_account().await?;
    info!("账户查询请求已发送");
    
    // 等待查询结果
    sleep(Duration::from_secs(2)).await;
    
    // 检查查询状态
    if let Some(state) = query_service.get_query_state(QueryType::Account) {
        info!("账户查询状态: 查询次数={}, 最后查询时间={:?}", 
            state.query_count, state.last_query_time);
    }
    
    Ok(())
}

/// 演示持仓查询
async fn demo_positions_query(
    client: &mut CtpClient,
    query_service: &QueryService,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("--- 持仓查询演示 ---");
    
    // 发起持仓查询
    client.query_positions().await?;
    info!("持仓查询请求已发送");
    
    // 等待查询结果
    sleep(Duration::from_secs(2)).await;
    
    // 检查查询状态
    if let Some(state) = query_service.get_query_state(QueryType::Positions) {
        info!("持仓查询状态: 查询次数={}, 最后查询时间={:?}", 
            state.query_count, state.last_query_time);
    }
    
    Ok(())
}

/// 演示成交记录查询
async fn demo_trades_query(
    client: &mut CtpClient,
    query_service: &QueryService,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("--- 成交记录查询演示 ---");
    
    // 发起成交记录查询
    client.query_trades(None).await?;
    info!("成交记录查询请求已发送");
    
    // 等待查询结果
    sleep(Duration::from_secs(2)).await;
    
    // 检查查询状态
    if let Some(state) = query_service.get_query_state(QueryType::Trades) {
        info!("成交记录查询状态: 查询次数={}, 最后查询时间={:?}", 
            state.query_count, state.last_query_time);
    }
    
    Ok(())
}

/// 演示报单记录查询
async fn demo_orders_query(
    client: &mut CtpClient,
    query_service: &QueryService,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("--- 报单记录查询演示 ---");
    
    // 发起报单记录查询
    client.query_orders(None).await?;
    info!("报单记录查询请求已发送");
    
    // 等待查询结果
    sleep(Duration::from_secs(2)).await;
    
    // 检查查询状态
    if let Some(state) = query_service.get_query_state(QueryType::Orders) {
        info!("报单记录查询状态: 查询次数={}, 最后查询时间={:?}", 
            state.query_count, state.last_query_time);
    }
    
    Ok(())
}

/// 演示结算信息查询
async fn demo_settlement_query(
    client: &mut CtpClient,
    query_service: &QueryService,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("--- 结算信息查询演示 ---");
    
    // 发起结算信息查询
    client.query_settlement_info(None).await?;
    info!("结算信息查询请求已发送");
    
    // 等待查询结果
    sleep(Duration::from_secs(3)).await;
    
    // 确认结算信息
    client.confirm_settlement_info().await?;
    info!("结算信息确认请求已发送");
    
    // 等待确认结果
    sleep(Duration::from_secs(2)).await;
    
    // 检查查询状态
    if let Some(state) = query_service.get_query_state(QueryType::Settlement) {
        info!("结算信息查询状态: 查询次数={}, 最后查询时间={:?}", 
            state.query_count, state.last_query_time);
    }
    
    Ok(())
}

/// 查询事件监听器
struct QueryEventListener;

impl QueryEventListener {
    fn new() -> Self {
        Self
    }
}

impl EventListener for QueryEventListener {
    fn on_query_account_result(&self, account: &crate::ctp::AccountInfo) {
        info!("📊 账户查询结果:");
        info!("  账户ID: {}", account.account_id);
        info!("  账户余额: {:.2}", account.balance);
        info!("  可用资金: {:.2}", account.available);
        info!("  占用保证金: {:.2}", account.curr_margin);
        info!("  风险度: {:.2}%", account.risk_ratio);
    }
    
    fn on_query_positions_result(&self, positions: &[crate::ctp::Position]) {
        info!("📈 持仓查询结果: {} 个合约", positions.len());
        for position in positions {
            info!("  合约: {} | 方向: {:?} | 总仓: {} | 今仓: {} | 昨仓: {} | 盈亏: {:.2}",
                position.instrument_id,
                position.direction,
                position.total_position,
                position.today_position,
                position.yesterday_position,
                position.unrealized_pnl
            );
        }
    }
    
    fn on_query_trades_result(&self, trades: &[crate::ctp::TradeRecord]) {
        info!("💰 成交查询结果: {} 条记录", trades.len());
        for trade in trades.iter().take(5) { // 只显示前5条
            info!("  成交: {} | 合约: {} | 方向: {:?} | 价格: {:.2} | 数量: {} | 时间: {}",
                trade.trade_id,
                trade.instrument_id,
                trade.direction,
                trade.price,
                trade.volume,
                trade.trade_time
            );
        }
        if trades.len() > 5 {
            info!("  ... 还有 {} 条记录", trades.len() - 5);
        }
    }
    
    fn on_query_orders_result(&self, orders: &[crate::ctp::OrderStatus]) {
        info!("📋 报单查询结果: {} 条记录", orders.len());
        for order in orders.iter().take(5) { // 只显示前5条
            info!("  报单: {} | 合约: {} | 方向: {:?} | 价格: {:.2} | 数量: {} | 状态: {:?}",
                order.order_id,
                order.instrument_id,
                order.direction,
                order.limit_price,
                order.volume_total_original,
                order.status
            );
        }
        if orders.len() > 5 {
            info!("  ... 还有 {} 条记录", orders.len() - 5);
        }
    }
    
    fn on_query_settlement_result(&self, content: &str) {
        info!("📄 结算信息查询结果: {} 字符", content.len());
        if !content.is_empty() {
            // 显示结算信息的前几行
            let lines: Vec<&str> = content.lines().take(10).collect();
            info!("结算信息内容（前10行）:");
            for (i, line) in lines.iter().enumerate() {
                info!("  {}: {}", i + 1, line);
            }
            if content.lines().count() > 10 {
                info!("  ... 还有 {} 行", content.lines().count() - 10);
            }
        }
    }
    
    fn on_settlement_confirmed(&self) {
        info!("✅ 结算信息确认成功");
    }
    
    fn on_error(&self, error: &crate::ctp::CtpError) {
        error!("❌ CTP 错误: {}", error);
    }
}

/// 处理事件
fn handle_event(event: &CtpEvent, listener: &Arc<QueryEventListener>) {
    match event {
        CtpEvent::QueryAccountResult(account) => {
            listener.on_query_account_result(account);
        }
        CtpEvent::QueryPositionsResult(positions) => {
            listener.on_query_positions_result(positions);
        }
        CtpEvent::QueryTradesResult(trades) => {
            listener.on_query_trades_result(trades);
        }
        CtpEvent::QueryOrdersResult(orders) => {
            listener.on_query_orders_result(orders);
        }
        CtpEvent::QuerySettlementResult(content) => {
            listener.on_query_settlement_result(content);
        }
        CtpEvent::SettlementConfirmed => {
            listener.on_settlement_confirmed();
        }
        CtpEvent::Error(msg) => {
            let error = crate::ctp::CtpError::Unknown(msg.clone());
            listener.on_error(&error);
        }
        _ => {}
    }
}
# CTP 查询功能使用指南

本文档介绍如何使用 CTP 交易组件的查询功能，包括账户资金查询、持仓信息查询、成交记录查询和结算单查询。

## 功能概述

CTP 查询功能提供以下查询能力：

1. **账户资金查询** - 查询账户余额、可用资金、占用保证金等信息
2. **持仓信息查询** - 查询所有合约的持仓详情
3. **成交记录查询** - 查询历史成交记录
4. **报单记录查询** - 查询历史报单记录
5. **结算信息查询** - 查询和确认结算单

## 基本使用方法

### 1. 创建客户端和查询服务

```rust
use inspirai_trader::ctp::{
    CtpClient, CtpConfig, Environment, QueryService, QueryOptions,
    CtpEvent, EventHandler,
};

// 创建配置
let config = CtpConfig::for_environment(
    Environment::Sim,
    "your_user_id".to_string(),
    "your_password".to_string(),
);

// 创建客户端
let mut client = CtpClient::new(config.clone()).await?;

// 创建查询服务
let query_service = QueryService::new(config, client.event_sender());
```

### 2. 连接和登录

```rust
// 连接到服务器
client.connect_with_retry().await?;

// 用户登录
let credentials = LoginCredentials {
    broker_id: config.broker_id.clone(),
    user_id: config.investor_id.clone(),
    password: config.password.clone(),
    app_id: config.app_id.clone(),
    auth_code: config.auth_code.clone(),
};

client.login(credentials).await?;
```

### 3. 执行查询操作

#### 账户资金查询

```rust
// 发起账户查询
client.query_account().await?;

// 查询结果将通过事件回调返回
// 监听 CtpEvent::QueryAccountResult 事件
```

#### 持仓信息查询

```rust
// 查询所有持仓
client.query_positions().await?;

// 查询结果将通过事件回调返回
// 监听 CtpEvent::QueryPositionsResult 事件
```

#### 成交记录查询

```rust
// 查询所有成交记录
client.query_trades(None).await?;

// 查询指定合约的成交记录
client.query_trades(Some("rb2401")).await?;

// 查询结果将通过事件回调返回
// 监听 CtpEvent::QueryTradesResult 事件
```

#### 报单记录查询

```rust
// 查询所有报单记录
client.query_orders(None).await?;

// 查询指定合约的报单记录
client.query_orders(Some("rb2401")).await?;

// 查询结果将通过事件回调返回
// 监听 CtpEvent::QueryOrdersResult 事件
```

#### 结算信息查询和确认

```rust
// 查询结算信息
client.query_settlement_info(None).await?;

// 查询指定交易日的结算信息
client.query_settlement_info(Some("20241201")).await?;

// 确认结算信息
client.confirm_settlement_info().await?;

// 查询结果将通过 CtpEvent::QuerySettlementResult 事件返回
// 确认结果将通过 CtpEvent::SettlementConfirmed 事件返回
```

## 事件处理

查询结果通过事件系统异步返回，需要实现事件监听器来处理查询结果：

```rust
use inspirai_trader::ctp::{EventListener, AccountInfo, Position, TradeRecord, OrderStatus};

struct MyEventListener;

impl EventListener for MyEventListener {
    fn on_query_account_result(&self, account: &AccountInfo) {
        println!("账户余额: {:.2}", account.balance);
        println!("可用资金: {:.2}", account.available);
        println!("风险度: {:.2}%", account.risk_ratio);
    }
    
    fn on_query_positions_result(&self, positions: &[Position]) {
        println!("持仓数量: {}", positions.len());
        for position in positions {
            println!("合约: {} | 方向: {:?} | 数量: {}", 
                position.instrument_id, position.direction, position.total_position);
        }
    }
    
    fn on_query_trades_result(&self, trades: &[TradeRecord]) {
        println!("成交记录数量: {}", trades.len());
        for trade in trades {
            println!("成交: {} | 价格: {:.2} | 数量: {}", 
                trade.trade_id, trade.price, trade.volume);
        }
    }
    
    fn on_query_orders_result(&self, orders: &[OrderStatus]) {
        println!("报单记录数量: {}", orders.len());
        for order in orders {
            println!("报单: {} | 状态: {:?}", 
                order.order_id, order.status);
        }
    }
    
    fn on_query_settlement_result(&self, content: &str) {
        println!("结算信息长度: {} 字符", content.len());
        // 处理结算信息内容
    }
    
    fn on_settlement_confirmed(&self) {
        println!("结算信息确认成功");
    }
}
```

## 查询服务高级功能

### 使用查询选项

```rust
use inspirai_trader::ctp::QueryOptions;

// 创建查询选项
let options = QueryOptions {
    use_cache: true,           // 使用缓存
    cache_ttl: Some(300),      // 缓存有效期 5 分钟
    timeout_secs: Some(60),    // 查询超时 60 秒
    instrument_id: Some("rb2401".to_string()), // 指定合约
    trading_day: Some("20241201".to_string()),  // 指定交易日
};

// 使用查询服务进行查询（带缓存）
let account = query_service.query_account(options.clone()).await?;
let positions = query_service.query_positions(options.clone()).await?;
```

### 查询状态监控

```rust
use inspirai_trader::ctp::QueryType;

// 获取查询状态
if let Some(state) = query_service.get_query_state(QueryType::Account) {
    println!("账户查询状态:");
    println!("  正在查询: {}", state.is_querying);
    println!("  查询次数: {}", state.query_count);
    println!("  最后查询时间: {:?}", state.last_query_time);
    if let Some(error) = &state.last_error {
        println!("  最后错误: {}", error);
    }
}

// 获取所有查询状态
let all_states = query_service.get_all_query_states();
for (query_type, state) in all_states {
    println!("{:?}: 查询次数 {}", query_type, state.query_count);
}
```

### 缓存管理

```rust
// 清空所有缓存
query_service.clear_cache();

// 清空指定类型的缓存
query_service.clear_cache_by_type(QueryType::Account);
query_service.clear_cache_by_type(QueryType::Positions);
```

## 完整示例

```rust
use inspirai_trader::ctp::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt().init();

    // 创建配置
    let config = CtpConfig::for_environment(
        Environment::Sim,
        "your_user_id".to_string(),
        "your_password".to_string(),
    );

    // 创建客户端和查询服务
    let mut client = CtpClient::new(config.clone()).await?;
    let query_service = QueryService::new(config.clone(), client.event_sender());

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
    sleep(Duration::from_secs(2)).await;

    // 执行查询
    println!("查询账户信息...");
    client.query_account().await?;
    sleep(Duration::from_secs(2)).await;

    println!("查询持仓信息...");
    client.query_positions().await?;
    sleep(Duration::from_secs(2)).await;

    println!("查询成交记录...");
    client.query_trades(None).await?;
    sleep(Duration::from_secs(2)).await;

    println!("查询结算信息...");
    client.query_settlement_info(None).await?;
    sleep(Duration::from_secs(3)).await;

    println!("确认结算信息...");
    client.confirm_settlement_info().await?;
    sleep(Duration::from_secs(2)).await;

    // 处理事件
    let mut event_handler = client.event_handler().clone();
    let listener = Arc::new(MyEventListener);
    
    tokio::spawn(async move {
        while let Some(event) = event_handler.next_event().await {
            query_service.handle_event(&event);
            handle_query_event(&event, &listener);
        }
    });

    // 保持连接
    sleep(Duration::from_secs(10)).await;
    
    client.disconnect();
    Ok(())
}

fn handle_query_event(event: &CtpEvent, listener: &Arc<MyEventListener>) {
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
        _ => {}
    }
}
```

## 注意事项

1. **查询频率限制**: CTP 对查询频率有限制，建议在查询之间添加适当的延时
2. **异步处理**: 所有查询都是异步的，结果通过事件回调返回
3. **错误处理**: 查询可能失败，需要监听错误事件并进行适当处理
4. **缓存使用**: 合理使用缓存可以减少不必要的查询请求
5. **结算确认**: 每个交易日开始前需要确认前一日的结算信息

## 故障排除

### 常见错误

1. **未登录错误**: 确保在查询前已成功登录
2. **查询超时**: 检查网络连接和服务器状态
3. **权限错误**: 确认账户有相应的查询权限
4. **频率限制**: 避免过于频繁的查询请求

### 调试技巧

1. 启用详细日志记录
2. 监控查询状态和错误信息
3. 检查事件处理逻辑
4. 验证配置参数的正确性

## 相关文档

- [CTP API 官方文档](http://www.sfit.com.cn/5_2_DocumentDown.htm)
- [ctp2rs 库文档](https://docs.rs/ctp2rs/)
- [项目 README](../../../README.md)
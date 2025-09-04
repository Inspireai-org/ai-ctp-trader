use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use crate::ctp::{CtpError, models::*};

/// CTP 事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CtpEvent {
    /// 连接成功
    Connected,
    /// 连接断开
    Disconnected,
    /// 需要登录（由 SPI 回调触发）
    LoginRequired,
    /// 登录成功
    LoginSuccess(LoginResponse),
    /// 登录失败
    LoginFailed(String),
    /// 行情数据更新
    MarketData(MarketDataTick),
    /// 订单状态更新
    OrderUpdate(OrderStatus),
    /// 成交记录更新
    TradeUpdate(TradeRecord),
    /// 账户信息更新
    AccountUpdate(AccountInfo),
    /// 持仓信息更新
    PositionUpdate(Vec<Position>),
    /// 错误事件
    Error(String),
}

/// 事件处理器
pub struct EventHandler {
    sender: mpsc::UnboundedSender<CtpEvent>,
    receiver: mpsc::UnboundedReceiver<CtpEvent>,
}

impl EventHandler {
    /// 创建新的事件处理器
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }

    /// 获取事件发送器的克隆
    pub fn sender(&self) -> mpsc::UnboundedSender<CtpEvent> {
        self.sender.clone()
    }

    /// 发送事件
    pub fn send_event(&self, event: CtpEvent) -> Result<(), CtpError> {
        self.sender
            .send(event)
            .map_err(|e| CtpError::Unknown(format!("发送事件失败: {}", e)))
    }

    /// 接收下一个事件
    pub async fn next_event(&mut self) -> Option<CtpEvent> {
        self.receiver.recv().await
    }

    /// 尝试接收事件（非阻塞）
    pub fn try_recv_event(&mut self) -> Result<CtpEvent, mpsc::error::TryRecvError> {
        self.receiver.try_recv()
    }

    /// 创建事件订阅器
    pub fn subscribe(&self) -> mpsc::UnboundedReceiver<CtpEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        // 这里需要实现广播机制，暂时返回一个新的接收器
        // 在实际实现中，应该使用 tokio::sync::broadcast 来支持多个订阅者
        rx
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// 事件监听器 trait
pub trait EventListener: Send + Sync {
    /// 处理行情数据事件
    fn on_market_data(&self, _tick: &MarketDataTick) {}
    
    /// 处理订单更新事件
    fn on_order_update(&self, _order: &OrderStatus) {}
    
    /// 处理成交记录事件
    fn on_trade_update(&self, _trade: &TradeRecord) {}
    
    /// 处理账户更新事件
    fn on_account_update(&self, _account: &AccountInfo) {}
    
    /// 处理持仓更新事件
    fn on_position_update(&self, _positions: &[Position]) {}
    
    /// 处理错误事件
    fn on_error(&self, _error: &CtpError) {}
    
    /// 处理连接状态变化
    fn on_connection_changed(&self, _connected: bool) {}
}

/// 默认事件监听器实现
pub struct DefaultEventListener;

impl EventListener for DefaultEventListener {
    fn on_market_data(&self, tick: &MarketDataTick) {
        tracing::debug!("收到行情数据: {}", tick.instrument_id);
    }
    
    fn on_order_update(&self, order: &OrderStatus) {
        tracing::info!("订单状态更新: {} - {:?}", order.order_id, order.status);
    }
    
    fn on_trade_update(&self, trade: &TradeRecord) {
        tracing::info!("成交记录: {} - {}", trade.trade_id, trade.volume);
    }
    
    fn on_account_update(&self, account: &AccountInfo) {
        tracing::info!("账户更新: 可用资金 {}", account.available);
    }
    
    fn on_position_update(&self, positions: &[Position]) {
        tracing::info!("持仓更新: {} 个合约", positions.len());
    }
    
    fn on_error(&self, error: &CtpError) {
        tracing::error!("CTP 错误: {}", error);
    }
    
    fn on_connection_changed(&self, connected: bool) {
        if connected {
            tracing::info!("CTP 连接已建立");
        } else {
            tracing::warn!("CTP 连接已断开");
        }
    }
}
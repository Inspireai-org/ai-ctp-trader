use crate::ctp::{
    CtpError, OrderRequest, OrderStatus, OrderStatusType, TradeRecord,
    OrderDirection, OffsetFlag, OrderType, TimeCondition,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use tracing::{info, warn, error, debug};

/// 订单管理器
pub struct OrderManager {
    /// 所有订单
    orders: Arc<Mutex<HashMap<String, OrderInfo>>>,
    /// 活动订单（未完成）
    active_orders: Arc<Mutex<HashMap<String, String>>>,
    /// 成交记录
    trades: Arc<Mutex<Vec<TradeRecord>>>,
    /// 订单统计
    stats: Arc<Mutex<OrderStats>>,
}

/// 订单信息
#[derive(Debug, Clone)]
pub struct OrderInfo {
    /// 订单状态
    pub status: OrderStatus,
    /// 创建时间
    pub create_time: Instant,
    /// 最后更新时间
    pub last_update: Instant,
    /// 重试次数
    pub retry_count: u32,
    /// 相关成交记录
    pub trades: Vec<TradeRecord>,
}

/// 订单统计
#[derive(Debug, Clone, Default)]
pub struct OrderStats {
    /// 总订单数
    pub total_orders: u64,
    /// 成功订单数
    pub success_orders: u64,
    /// 失败订单数
    pub failed_orders: u64,
    /// 撤销订单数
    pub canceled_orders: u64,
    /// 总成交数
    pub total_trades: u64,
    /// 今日成交额
    pub today_turnover: f64,
}

impl OrderManager {
    pub fn new() -> Self {
        Self {
            orders: Arc::new(Mutex::new(HashMap::new())),
            active_orders: Arc::new(Mutex::new(HashMap::new())),
            trades: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(OrderStats::default())),
        }
    }

    /// 添加新订单
    pub fn add_order(&self, order: OrderStatus) -> Result<(), CtpError> {
        let order_id = order.order_id.clone();
        
        let order_info = OrderInfo {
            status: order.clone(),
            create_time: Instant::now(),
            last_update: Instant::now(),
            retry_count: 0,
            trades: Vec::new(),
        };
        
        self.orders.lock().unwrap().insert(order_id.clone(), order_info);
        
        // 如果是活动订单，加入活动列表
        if self.is_active_status(order.status) {
            self.active_orders.lock().unwrap()
                .insert(order_id.clone(), order.instrument_id.clone());
        }
        
        // 更新统计
        let mut stats = self.stats.lock().unwrap();
        stats.total_orders += 1;
        
        info!("添加订单: {} 合约={} 状态={:?}", 
            order_id, order.instrument_id, order.status);
        
        Ok(())
    }

    /// 更新订单状态
    pub fn update_order(&self, order: OrderStatus) -> Result<(), CtpError> {
        let order_id = order.order_id.clone();
        
        let mut orders = self.orders.lock().unwrap();
        
        if let Some(order_info) = orders.get_mut(&order_id) {
            let old_status = order_info.status.status;
            order_info.status = order.clone();
            order_info.last_update = Instant::now();
            
            // 更新活动订单列表
            if !self.is_active_status(order.status) {
                self.active_orders.lock().unwrap().remove(&order_id);
                
                // 更新统计
                let mut stats = self.stats.lock().unwrap();
                match order.status {
                    OrderStatusType::AllTraded => stats.success_orders += 1,
                    OrderStatusType::Canceled => stats.canceled_orders += 1,
                    OrderStatusType::Unknown => stats.failed_orders += 1,
                    _ => {}
                }
            }
            
            debug!("更新订单: {} 状态={:?} -> {:?}", 
                order_id, old_status, order.status);
        } else {
            // 如果订单不存在，创建新订单
            self.add_order(order)?;
        }
        
        Ok(())
    }

    /// 添加成交记录
    pub fn add_trade(&self, trade: TradeRecord) -> Result<(), CtpError> {
        let order_id = trade.order_id.clone();
        
        // 添加到总成交列表
        self.trades.lock().unwrap().push(trade.clone());
        
        // 关联到对应订单
        let mut orders = self.orders.lock().unwrap();
        if let Some(order_info) = orders.get_mut(&order_id) {
            order_info.trades.push(trade.clone());
            order_info.last_update = Instant::now();
        }
        
        // 更新统计
        let mut stats = self.stats.lock().unwrap();
        stats.total_trades += 1;
        stats.today_turnover += trade.price * trade.volume as f64;
        
        info!("添加成交: {} 合约={} {}手@{}", 
            trade.trade_id, trade.instrument_id, trade.volume, trade.price);
        
        Ok(())
    }

    /// 获取订单信息
    pub fn get_order(&self, order_id: &str) -> Option<OrderInfo> {
        self.orders.lock().unwrap().get(order_id).cloned()
    }

    /// 获取所有活动订单
    pub fn get_active_orders(&self) -> Vec<OrderStatus> {
        let orders = self.orders.lock().unwrap();
        let active = self.active_orders.lock().unwrap();
        
        active.keys()
            .filter_map(|id| orders.get(id))
            .map(|info| info.status.clone())
            .collect()
    }

    /// 获取订单的成交记录
    pub fn get_order_trades(&self, order_id: &str) -> Vec<TradeRecord> {
        self.orders.lock().unwrap()
            .get(order_id)
            .map(|info| info.trades.clone())
            .unwrap_or_default()
    }

    /// 获取今日成交
    pub fn get_today_trades(&self) -> Vec<TradeRecord> {
        self.trades.lock().unwrap().clone()
    }

    /// 获取订单统计
    pub fn get_stats(&self) -> OrderStats {
        self.stats.lock().unwrap().clone()
    }

    /// 验证订单请求
    pub fn validate_order(&self, order: &OrderRequest) -> Result<(), CtpError> {
        // 基本验证
        if order.instrument_id.is_empty() {
            return Err(CtpError::ValidationError("合约代码不能为空".to_string()));
        }
        
        if order.volume <= 0 {
            return Err(CtpError::ValidationError("委托数量必须大于0".to_string()));
        }
        
        if order.price <= 0.0 && order.order_type == OrderType::Limit {
            return Err(CtpError::ValidationError("限价单价格必须大于0".to_string()));
        }
        
        // TODO: 添加更多验证规则
        // - 风险控制检查
        // - 资金充足性检查
        // - 持仓可平检查
        
        Ok(())
    }

    /// 判断是否为活动状态
    fn is_active_status(&self, status: OrderStatusType) -> bool {
        matches!(
            status,
            OrderStatusType::Unknown
                | OrderStatusType::PartTradedQueueing
                | OrderStatusType::PartTradedNotQueueing
                | OrderStatusType::NoTradeQueueing
                | OrderStatusType::NoTradeNotQueueing
                | OrderStatusType::Touched
        )
    }

    /// 清理过期订单
    pub fn cleanup_expired_orders(&self, expire_duration: Duration) {
        let now = Instant::now();
        let mut orders = self.orders.lock().unwrap();
        let mut active = self.active_orders.lock().unwrap();
        
        let expired: Vec<String> = orders
            .iter()
            .filter(|(_, info)| {
                !self.is_active_status(info.status.status)
                    && now.duration_since(info.last_update) > expire_duration
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in expired {
            orders.remove(&id);
            active.remove(&id);
            debug!("清理过期订单: {}", id);
        }
    }
}
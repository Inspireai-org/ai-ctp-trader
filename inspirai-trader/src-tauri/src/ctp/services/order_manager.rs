use crate::ctp::{CtpError, CtpEvent, models::{OrderRequest, OrderStatus, OrderDirection, OrderOffsetFlag, OrderPriceType}};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, debug, error};
use std::time::{Duration, Instant};
use chrono::{DateTime, Local};

/// 订单管理器
pub struct OrderManager {
    /// 活动订单映射 (order_ref -> order_status)
    active_orders: Arc<RwLock<HashMap<String, OrderStatus>>>,
    /// 历史订单映射 (order_ref -> order_status)
    history_orders: Arc<RwLock<HashMap<String, OrderStatus>>>,
    /// 订单引用计数器
    order_ref_counter: Arc<Mutex<u32>>,
    /// 前置编号
    front_id: i32,
    /// 会话编号
    session_id: i32,
    /// 最大订单引用
    max_order_ref: Arc<Mutex<String>>,
    /// 事件发送器
    event_sender: mpsc::UnboundedSender<CtpEvent>,
    /// 订单统计
    statistics: Arc<RwLock<OrderStatistics>>,
    /// 风险控制参数
    risk_control: Arc<RwLock<RiskControl>>,
}

/// 订单统计
#[derive(Debug, Default, Clone)]
pub struct OrderStatistics {
    pub total_orders: u64,
    pub active_orders: usize,
    pub filled_orders: u64,
    pub cancelled_orders: u64,
    pub rejected_orders: u64,
    pub partial_filled_orders: u64,
    pub total_volume: u64,
    pub total_filled_volume: u64,
    pub average_fill_time_ms: f64,
    pub success_rate: f64,
}

/// 风险控制参数
#[derive(Debug, Clone)]
pub struct RiskControl {
    /// 最大持仓量
    pub max_position_size: u32,
    /// 单笔最大下单量
    pub max_order_volume: u32,
    /// 每分钟最大下单次数
    pub max_orders_per_minute: u32,
    /// 每日最大亏损限制
    pub daily_loss_limit: f64,
    /// 是否启用风控
    pub enabled: bool,
    /// 订单时间记录
    order_timestamps: Vec<Instant>,
}

impl Default for RiskControl {
    fn default() -> Self {
        Self {
            max_position_size: 100,
            max_order_volume: 10,
            max_orders_per_minute: 60,
            daily_loss_limit: 50000.0,
            enabled: true,
            order_timestamps: Vec::new(),
        }
    }
}

impl OrderManager {
    /// 创建新的订单管理器
    pub fn new(
        event_sender: mpsc::UnboundedSender<CtpEvent>,
        front_id: i32,
        session_id: i32,
    ) -> Self {
        Self {
            active_orders: Arc::new(RwLock::new(HashMap::new())),
            history_orders: Arc::new(RwLock::new(HashMap::new())),
            order_ref_counter: Arc::new(Mutex::new(1)),
            front_id,
            session_id,
            max_order_ref: Arc::new(Mutex::new("0".to_string())),
            event_sender,
            statistics: Arc::new(RwLock::new(OrderStatistics::default())),
            risk_control: Arc::new(RwLock::new(RiskControl::default())),
        }
    }

    /// 生成新的订单引用
    pub fn generate_order_ref(&self) -> String {
        let mut counter = self.order_ref_counter.lock().unwrap();
        let order_ref = format!("{:012}", *counter);
        *counter += 1;
        
        // 更新最大订单引用
        *self.max_order_ref.lock().unwrap() = order_ref.clone();
        
        debug!("生成订单引用: {}", order_ref);
        order_ref
    }

    /// 创建订单请求
    pub async fn create_order_request(
        &self,
        instrument_id: String,
        direction: OrderDirection,
        offset: OrderOffsetFlag,
        price: f64,
        volume: u32,
        price_type: OrderPriceType,
    ) -> Result<OrderRequest, CtpError> {
        // 风险控制检查
        if let Err(e) = self.check_risk_control(volume).await {
            error!("风控检查失败: {}", e);
            return Err(e);
        }

        let order_ref = self.generate_order_ref();
        
        let request = OrderRequest {
            instrument_id: instrument_id.clone(),
            order_ref: order_ref.clone(),
            direction,
            offset_flag: offset,
            price,
            volume,
            order_type: crate::ctp::models::OrderType::Limit,
            price_type,
            time_condition: crate::ctp::models::OrderTimeCondition::GFD,
            volume_condition: crate::ctp::models::OrderVolumeCondition::Any,
            min_volume: 1,
            contingent_condition: crate::ctp::models::OrderContingentCondition::Immediately,
            stop_price: 0.0,
            force_close_reason: crate::ctp::models::OrderForceCloseReason::NotForceClose,
            is_auto_suspend: false,
        };

        // 创建初始订单状态
        let order_status = OrderStatus {
            order_ref: order_ref.clone(),
            order_id: String::new(),
            instrument_id,
            direction,
            offset_flag: offset,
            price,
            limit_price: price,
            volume,
            volume_total_original: volume as i32,
            volume_traded: 0,
            volume_left: volume,
            volume_total: volume as i32,
            status: crate::ctp::models::OrderStatusType::Unknown,
            submit_time: Local::now(),
            insert_time: Local::now().format("%Y%m%d %H:%M:%S").to_string(),
            update_time: Local::now(),
            front_id: self.front_id,
            session_id: self.session_id,
            order_sys_id: String::new(),
            status_msg: String::new(),
            is_local: false,
            frozen_margin: 0.0,
            frozen_commission: 0.0,
        };

        // 添加到活动订单
        {
            let mut active = self.active_orders.write().await;
            active.insert(order_ref.clone(), order_status.clone());
        }

        // 更新统计
        {
            let mut stats = self.statistics.write().await;
            stats.total_orders += 1;
            stats.active_orders = self.active_orders.read().await.len();
            stats.total_volume += volume as u64;
        }

        // 发送订单创建事件
        if let Err(e) = self.event_sender.send(CtpEvent::OrderUpdate(order_status)) {
            error!("发送订单创建事件失败: {}", e);
        }

        info!("创建订单请求: {} {} {} @ {} x {}", 
            request.instrument_id, 
            if direction == OrderDirection::Buy { "买入" } else { "卖出" },
            match offset {
                OrderOffsetFlag::Open => "开仓",
                OrderOffsetFlag::Close => "平仓",
                OrderOffsetFlag::CloseToday => "平今",
                OrderOffsetFlag::CloseYesterday => "平昨",
                _ => "未知",
            },
            price,
            volume
        );

        Ok(request)
    }

    /// 更新订单状态
    pub async fn update_order_status(&self, order_status: OrderStatus) -> Result<(), CtpError> {
        let order_ref = order_status.order_ref.clone();
        
        // 更新活动订单
        {
            let mut active = self.active_orders.write().await;
            
            // 检查订单是否已完成
            let is_final = matches!(
                order_status.status,
                crate::ctp::models::OrderStatusType::AllTraded |
                crate::ctp::models::OrderStatusType::Cancelled
            );
            
            if is_final {
                // 移动到历史订单
                if let Some(mut order) = active.remove(&order_ref) {
                    order.status = order_status.status.clone();
                    order.update_time = Local::now();
                    order.volume_traded = order_status.volume_traded;
                    order.volume_left = order_status.volume_left;
                    order.status_msg = order_status.status_msg.clone();
                    
                    let mut history = self.history_orders.write().await;
                    history.insert(order_ref.clone(), order.clone());
                    
                    // 更新统计
                    let mut stats = self.statistics.write().await;
                    stats.active_orders = active.len();
                    
                    match order_status.status {
                        crate::ctp::models::OrderStatusType::AllTraded => {
                            stats.filled_orders += 1;
                            stats.total_filled_volume += order_status.volume_traded as u64;
                        }
                        crate::ctp::models::OrderStatusType::Cancelled => {
                            stats.cancelled_orders += 1;
                        }
                        _ => {}
                    }
                    
                    // 计算成功率
                    let total = stats.filled_orders + stats.cancelled_orders + stats.rejected_orders;
                    if total > 0 {
                        stats.success_rate = (stats.filled_orders as f64) / (total as f64) * 100.0;
                    }
                }
            } else {
                // 更新活动订单
                if let Some(order) = active.get_mut(&order_ref) {
                    order.status = order_status.status.clone();
                    order.update_time = Local::now();
                    order.volume_traded = order_status.volume_traded;
                    order.volume_left = order_status.volume_left;
                    order.order_sys_id = order_status.order_sys_id.clone();
                    order.status_msg = order_status.status_msg.clone();
                    
                    if order_status.volume_traded > 0 && order_status.volume_left > 0 {
                        let mut stats = self.statistics.write().await;
                        stats.partial_filled_orders += 1;
                    }
                }
            }
        }

        // 发送订单更新事件
        if let Err(e) = self.event_sender.send(CtpEvent::OrderUpdate(order_status.clone())) {
            error!("发送订单更新事件失败: {}", e);
        }

        debug!("更新订单状态: {} -> {:?}", order_ref, order_status.status);
        Ok(())
    }

    /// 获取活动订单
    pub async fn get_active_order(&self, order_ref: &str) -> Option<OrderStatus> {
        self.active_orders.read().await.get(order_ref).cloned()
    }

    /// 获取所有活动订单
    pub async fn get_all_active_orders(&self) -> Vec<OrderStatus> {
        self.active_orders.read().await.values().cloned().collect()
    }

    /// 获取历史订单
    pub async fn get_history_order(&self, order_ref: &str) -> Option<OrderStatus> {
        self.history_orders.read().await.get(order_ref).cloned()
    }

    /// 获取所有历史订单
    pub async fn get_all_history_orders(&self) -> Vec<OrderStatus> {
        self.history_orders.read().await.values().cloned().collect()
    }

    /// 获取统计信息
    pub async fn get_statistics(&self) -> OrderStatistics {
        self.statistics.read().await.clone()
    }

    /// 设置风控参数
    pub async fn set_risk_control(&self, risk_control: RiskControl) {
        *self.risk_control.write().await = risk_control;
        info!("更新风控参数");
    }

    /// 检查风险控制
    async fn check_risk_control(&self, volume: u32) -> Result<(), CtpError> {
        let mut risk = self.risk_control.write().await;
        
        if !risk.enabled {
            return Ok(());
        }

        // 检查单笔下单量
        if volume > risk.max_order_volume {
            return Err(CtpError::RiskControl(format!(
                "单笔下单量 {} 超过限制 {}",
                volume, risk.max_order_volume
            )));
        }

        // 检查下单频率
        let now = Instant::now();
        risk.order_timestamps.retain(|&t| now.duration_since(t) < Duration::from_secs(60));
        
        if risk.order_timestamps.len() >= risk.max_orders_per_minute as usize {
            return Err(CtpError::RiskControl(format!(
                "下单频率超过限制: 每分钟最多 {} 笔",
                risk.max_orders_per_minute
            )));
        }
        
        risk.order_timestamps.push(now);
        
        Ok(())
    }

    /// 清空所有订单
    pub async fn clear_all_orders(&self) {
        self.active_orders.write().await.clear();
        self.history_orders.write().await.clear();
        info!("清空所有订单");
    }

    /// 取消订单
    pub async fn cancel_order(&self, order_ref: &str) -> Result<(), CtpError> {
        let order = self.get_active_order(order_ref).await
            .ok_or_else(|| CtpError::NotFound(format!("订单不存在: {}", order_ref)))?;
        
        if matches!(
            order.status,
            crate::ctp::models::OrderStatusType::AllTraded |
            crate::ctp::models::OrderStatusType::Cancelled
        ) {
            return Err(CtpError::StateError(format!("订单已完成，无法撤销: {}", order_ref)));
        }
        
        info!("请求撤销订单: {}", order_ref);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_order_manager_creation() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let manager = OrderManager::new(tx, 1, 1001);
        
        assert_eq!(manager.front_id, 1);
        assert_eq!(manager.session_id, 1001);
        assert!(manager.get_all_active_orders().await.is_empty());
    }

    #[tokio::test]
    async fn test_order_ref_generation() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let manager = OrderManager::new(tx, 1, 1001);
        
        let ref1 = manager.generate_order_ref();
        let ref2 = manager.generate_order_ref();
        let ref3 = manager.generate_order_ref();
        
        assert_eq!(ref1, "000000000001");
        assert_eq!(ref2, "000000000002");
        assert_eq!(ref3, "000000000003");
    }

    #[tokio::test]
    async fn test_risk_control() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let manager = OrderManager::new(tx, 1, 1001);
        
        // 设置风控参数
        let mut risk = RiskControl::default();
        risk.max_order_volume = 5;
        manager.set_risk_control(risk).await;
        
        // 测试超过限制的订单
        let result = manager.create_order_request(
            "rb2401".to_string(),
            OrderDirection::Buy,
            OrderOffsetFlag::Open,
            4500.0,
            10, // 超过限制
            OrderPriceType::Limit,
        ).await;
        
        assert!(result.is_err());
    }
}
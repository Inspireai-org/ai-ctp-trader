use crate::ctp::{CtpError, CtpEvent, models::*};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, debug, error};

/// 交易服务
pub struct TradingService {
    /// 订单管理器
    order_manager: Arc<super::order_manager::OrderManager>,
    /// 事件发送器
    event_sender: mpsc::UnboundedSender<CtpEvent>,
    /// 服务状态
    is_running: Arc<RwLock<bool>>,
}

impl TradingService {
    /// 创建新的交易服务
    pub fn new(
        event_sender: mpsc::UnboundedSender<CtpEvent>,
        front_id: i32,
        session_id: i32,
    ) -> Self {
        let order_manager = Arc::new(super::order_manager::OrderManager::new(
            event_sender.clone(),
            front_id,
            session_id,
        ));

        Self {
            order_manager,
            event_sender,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// 启动交易服务
    pub async fn start(&self) -> Result<(), CtpError> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(CtpError::StateError("交易服务已经在运行".to_string()));
        }
        
        *is_running = true;
        info!("交易服务已启动");
        Ok(())
    }

    /// 停止交易服务
    pub async fn stop(&self) -> Result<(), CtpError> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Err(CtpError::StateError("交易服务未运行".to_string()));
        }
        
        *is_running = false;
        info!("交易服务已停止");
        Ok(())
    }

    /// 下单
    pub async fn place_order(
        &self,
        instrument_id: String,
        direction: OrderDirection,
        offset: OffsetFlag,
        price: f64,
        volume: u32,
    ) -> Result<String, CtpError> {
        // 检查服务状态
        if !*self.is_running.read().await {
            return Err(CtpError::StateError("交易服务未运行".to_string()));
        }

        // 创建订单请求
        let order_request = self.order_manager.create_order_request(
            instrument_id,
            direction,
            offset,
            price,
            volume,
            OrderPriceType::Limit,
        ).await?;

        let order_ref = order_request.order_ref.clone();
        
        // TODO: 发送订单到CTP API
        
        info!("订单已提交: {}", order_ref);
        Ok(order_ref)
    }

    /// 撤单
    pub async fn cancel_order(&self, order_ref: &str) -> Result<(), CtpError> {
        // 检查服务状态
        if !*self.is_running.read().await {
            return Err(CtpError::StateError("交易服务未运行".to_string()));
        }

        self.order_manager.cancel_order(order_ref).await?;
        
        // TODO: 发送撤单请求到CTP API
        
        info!("撤单请求已发送: {}", order_ref);
        Ok(())
    }

    /// 获取活动订单
    pub async fn get_active_orders(&self) -> Vec<OrderStatus> {
        self.order_manager.get_all_active_orders().await
    }

    /// 获取历史订单
    pub async fn get_history_orders(&self) -> Vec<OrderStatus> {
        self.order_manager.get_all_history_orders().await
    }

    /// 获取订单管理器
    pub fn get_order_manager(&self) -> Arc<super::order_manager::OrderManager> {
        self.order_manager.clone()
    }
}
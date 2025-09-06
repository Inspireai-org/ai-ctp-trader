use crate::ctp::{
    CtpError, CtpEvent, ClientState, TraderSpiImpl, OrderManager,
    OrderRequest, OrderStatus, OrderAction, TradeRecord, Position, AccountInfo,
    AccountService, PositionManager, SettlementManager, AccountSummary,
    config::CtpConfig,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use tracing::{info, warn, error, debug};

/// 交易服务
pub struct TradingService {
    /// 交易SPI实例
    trader_spi: Arc<Mutex<TraderSpiImpl>>,
    /// 订单管理器
    order_manager: OrderManager,
    /// 账户服务
    account_service: AccountService,
    /// 持仓管理器
    position_manager: PositionManager,
    /// 结算管理器
    settlement_manager: SettlementManager,
    /// 事件发送器
    event_sender: mpsc::UnboundedSender<CtpEvent>,
    /// 客户端状态
    client_state: Arc<Mutex<ClientState>>,
    /// 配置信息
    config: CtpConfig,
    /// 服务状态
    service_state: Arc<Mutex<ServiceState>>,
}

/// 服务状态
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceState {
    /// 未初始化
    Uninitialized,
    /// 初始化中
    Initializing,
    /// 已初始化
    Initialized,
    /// 运行中
    Running,
    /// 已停止
    Stopped,
    /// 错误状态
    Error(String),
}

/// 交易服务统计
#[derive(Debug, Clone, Default)]
pub struct TradingStats {
    /// 总订单数
    pub total_orders: u64,
    /// 成功订单数
    pub success_orders: u64,
    /// 失败订单数
    pub failed_orders: u64,
    /// 总成交数
    pub total_trades: u64,
    /// 今日成交额
    pub today_turnover: f64,
    /// 服务启动时间
    pub start_time: Option<Instant>,
    /// 最后活动时间
    pub last_activity: Option<Instant>,
}

impl TradingService {
    /// 创建交易服务
    pub fn new(
        config: CtpConfig,
        client_state: Arc<Mutex<ClientState>>,
        event_sender: mpsc::UnboundedSender<CtpEvent>,
    ) -> Self {
        let trader_spi = Arc::new(Mutex::new(TraderSpiImpl::new(
            client_state.clone(),
            event_sender.clone(),
            config.clone(),
        )));
        
        Self {
            trader_spi,
            order_manager: OrderManager::new(),
            account_service: AccountService::new(config.clone()),
            position_manager: PositionManager::new(),
            settlement_manager: SettlementManager::new(),
            event_sender,
            client_state,
            config,
            service_state: Arc::new(Mutex::new(ServiceState::Uninitialized)),
        }
    }

    /// 初始化服务
    pub async fn initialize(&self) -> Result<(), CtpError> {
        info!("初始化交易服务");
        *self.service_state.lock().unwrap() = ServiceState::Initializing;
        
        // 初始化各组件
        // TODO: 连接到交易前置
        
        *self.service_state.lock().unwrap() = ServiceState::Initialized;
        info!("交易服务初始化完成");
        
        Ok(())
    }

    /// 启动服务
    pub async fn start(&self) -> Result<(), CtpError> {
        if *self.service_state.lock().unwrap() != ServiceState::Initialized {
            return Err(CtpError::StateError("服务未初始化".to_string()));
        }
        
        info!("启动交易服务");
        *self.service_state.lock().unwrap() = ServiceState::Running;
        
        // TODO: 启动后台任务
        
        Ok(())
    }

    /// 停止服务
    pub async fn stop(&self) -> Result<(), CtpError> {
        info!("停止交易服务");
        *self.service_state.lock().unwrap() = ServiceState::Stopped;
        
        Ok(())
    }

    /// 提交订单
    pub async fn submit_order(&self, order: OrderRequest, trader_api: Option<Arc<ctp2rs::v1alpha1::TraderApi>>) -> Result<String, CtpError> {
        // 验证订单
        self.order_manager.validate_order(&order)?;
        
        // 生成订单引用
        let order_ref = self.trader_spi.lock().unwrap().next_order_ref();
        
        info!("提交订单: {} 合约={} 方向={:?} {}手@{}", 
            order_ref, order.instrument_id, order.direction, order.volume, order.price);
        
        // 创建订单状态
        let order_status = OrderStatus {
            order_ref: order_ref.clone(),
            order_id: order_ref.clone(),
            instrument_id: order.instrument_id.clone(),
            direction: order.direction,
            offset_flag: order.offset_flag,
            price: order.price,
            limit_price: order.price,
            volume: order.volume as u32,
            volume_total_original: order.volume as i32,
            volume_traded: 0,
            volume_left: order.volume as u32,
            volume_total: order.volume as i32,
            status: crate::ctp::models::OrderStatusType::Unknown,
            submit_time: chrono::Local::now(),
            insert_time: chrono::Local::now().format("%H:%M:%S").to_string(),
            update_time: chrono::Local::now(),
            front_id: 0,
            session_id: 0,
            order_sys_id: String::new(),
            status_msg: "待提交".to_string(),
            is_local: true,
            frozen_margin: 0.0,
            frozen_commission: 0.0,
        };
        
        // 添加到订单管理器
        self.order_manager.add_order(order_status)?;
        
        // 使用真实的 CTP API 提交订单
        if let Some(api) = trader_api {
            // 将业务订单转换为 CTP 订单结构
            let ctp_order = crate::ctp::utils::DataConverter::convert_order_request(
                &order,
                &self.config.broker_id,
                &self.config.investor_id,
                &order_ref,
            )?;
            
            let request_id = chrono::Utc::now().timestamp_millis() as i32 % 1000000;
            
            info!("发送报单录入请求，订单引用: {}, 请求ID: {}", order_ref, request_id);
            
            // 调用 ctp2rs TraderApi 提交订单
            let mut ctp_order_mut = ctp_order;
            let result = api.req_order_insert(&mut ctp_order_mut, request_id);
            
            if result != 0 {
                return Err(CtpError::CtpApiError {
                    code: result,
                    message: "报单录入请求发送失败".to_string(),
                });
            }
            
            info!("报单录入请求已发送，订单引用: {}", order_ref);
        } else {
            warn!("交易 API 未提供，订单将仅在本地记录");
        }
        
        Ok(order_ref)
    }

    /// 撤销订单
    pub async fn cancel_order(&self, order_id: &str, trader_api: Option<Arc<ctp2rs::v1alpha1::TraderApi>>) -> Result<(), CtpError> {
        info!("撤销订单: {}", order_id);
        
        // 获取订单信息
        let order_info = self.order_manager.get_order(order_id)
            .ok_or_else(|| CtpError::NotFound(format!("订单不存在: {}", order_id)))?;
        
        // 检查订单状态
        if !self.can_cancel(&order_info.status) {
            return Err(CtpError::StateError(
                format!("订单状态不允许撤销: {:?}", order_info.status.status)
            ));
        }
        
        // 使用真实的 CTP API 撤销订单
        if let Some(api) = trader_api {
            // 创建撤单请求
            let mut order_action = ctp2rs::v1alpha1::CThostFtdcInputOrderActionField::default();
            
            // 使用 ctp2rs 提供的字符串赋值工具
            use ctp2rs::ffi::AssignFromString;
            order_action.BrokerID.assign_from_str(&self.config.broker_id);
            order_action.InvestorID.assign_from_str(&self.config.investor_id);
            order_action.OrderRef.assign_from_str(order_id);
            order_action.InstrumentID.assign_from_str(&order_info.status.instrument_id);
            
            // 设置撤单标志
            order_action.ActionFlag = '0' as i8; // 删除
            order_action.FrontID = 1; // 前置编号，应该从登录响应中获取
            order_action.SessionID = 1; // 会话编号，应该从登录响应中获取
            
            let request_id = chrono::Utc::now().timestamp_millis() as i32 % 1000000;
            
            info!("发送报单操作请求，订单引用: {}, 请求ID: {}", order_id, request_id);
            
            // 调用 ctp2rs TraderApi 撤销订单
            let result = api.req_order_action(&mut order_action, request_id);
            
            if result != 0 {
                return Err(CtpError::CtpApiError {
                    code: result,
                    message: "报单操作请求发送失败".to_string(),
                });
            }
            
            info!("报单操作请求已发送，订单引用: {}", order_id);
        } else {
            warn!("交易 API 未提供，撤单将仅在本地记录");
        }
        
        Ok(())
    }

    /// 查询订单
    pub async fn query_order(&self, order_id: &str) -> Result<OrderStatus, CtpError> {
        self.order_manager.get_order(order_id)
            .map(|info| info.status)
            .ok_or_else(|| CtpError::NotFound(format!("订单不存在: {}", order_id)))
    }

    /// 查询所有活动订单
    pub async fn query_active_orders(&self) -> Result<Vec<OrderStatus>, CtpError> {
        Ok(self.order_manager.get_active_orders())
    }

    /// 查询成交记录
    pub async fn query_trades(&self, order_id: Option<&str>, trader_api: Option<Arc<ctp2rs::v1alpha1::TraderApi>>) -> Result<Vec<TradeRecord>, CtpError> {
        // 使用真实的 CTP API 查询成交记录
        if let Some(api) = trader_api {
            // 创建成交查询请求
            let mut qry_req = ctp2rs::v1alpha1::CThostFtdcQryTradeField::default();
            
            // 使用 ctp2rs 提供的字符串赋值工具
            use ctp2rs::ffi::AssignFromString;
            qry_req.BrokerID.assign_from_str(&self.config.broker_id);
            qry_req.InvestorID.assign_from_str(&self.config.investor_id);
            
            // 如果指定了订单ID，则查询特定订单的成交
            // 注意：CThostFtdcQryTradeField 没有 OrderRef 字段，需要使用其他字段
            // 这里我们暂时跳过特定订单的过滤，在回调中处理
            if let Some(_id) = order_id {
                // TODO: 根据 CTP API 文档确定正确的字段来过滤特定订单的成交
                // qry_req.TradeID.assign_from_str(id);
            }
            
            let request_id = chrono::Utc::now().timestamp_millis() as i32 % 1000000;
            
            info!("发送成交查询请求，请求ID: {}", request_id);
            
            // 调用 ctp2rs TraderApi 查询成交
            let result = api.req_qry_trade(&mut qry_req, request_id);
            
            if result != 0 {
                return Err(CtpError::CtpApiError {
                    code: result,
                    message: "成交查询请求发送失败".to_string(),
                });
            }
            
            info!("成交查询请求已发送");
        }
        
        // 返回本地缓存的成交记录
        if let Some(id) = order_id {
            Ok(self.order_manager.get_order_trades(id))
        } else {
            Ok(self.order_manager.get_today_trades())
        }
    }

    /// 查询持仓
    pub async fn query_positions(&self, trader_api: Option<Arc<ctp2rs::v1alpha1::TraderApi>>) -> Result<Vec<Position>, CtpError> {
        // 使用真实的 CTP API 查询持仓信息
        if let Some(api) = trader_api {
            // 创建投资者持仓查询请求
            let mut qry_req = ctp2rs::v1alpha1::CThostFtdcQryInvestorPositionField::default();
            
            // 使用 ctp2rs 提供的字符串赋值工具
            use ctp2rs::ffi::AssignFromString;
            qry_req.BrokerID.assign_from_str(&self.config.broker_id);
            qry_req.InvestorID.assign_from_str(&self.config.investor_id);
            // InstrumentID 留空表示查询所有合约的持仓
            
            let request_id = chrono::Utc::now().timestamp_millis() as i32 % 1000000;
            
            info!("发送投资者持仓查询请求，请求ID: {}", request_id);
            
            // 调用 ctp2rs TraderApi 查询投资者持仓
            let result = api.req_qry_investor_position(&mut qry_req, request_id);
            
            if result != 0 {
                return Err(CtpError::CtpApiError {
                    code: result,
                    message: "投资者持仓查询请求发送失败".to_string(),
                });
            }
            
            info!("投资者持仓查询请求已发送");
        }
        
        // 返回本地缓存的持仓信息
        Ok(self.trader_spi.lock().unwrap().get_all_positions())
    }

    /// 查询账户信息
    pub async fn query_account(&self, trader_api: Option<Arc<ctp2rs::v1alpha1::TraderApi>>) -> Result<AccountInfo, CtpError> {
        // 使用真实的 CTP API 查询账户信息
        if let Some(api) = trader_api {
            // 创建资金账户查询请求
            let mut qry_req = ctp2rs::v1alpha1::CThostFtdcQryTradingAccountField::default();
            
            // 使用 ctp2rs 提供的字符串赋值工具
            use ctp2rs::ffi::AssignFromString;
            qry_req.BrokerID.assign_from_str(&self.config.broker_id);
            qry_req.InvestorID.assign_from_str(&self.config.investor_id);
            
            let request_id = chrono::Utc::now().timestamp_millis() as i32 % 1000000;
            
            info!("发送资金账户查询请求，请求ID: {}", request_id);
            
            // 调用 ctp2rs TraderApi 查询资金账户
            let result = api.req_qry_trading_account(&mut qry_req, request_id);
            
            if result != 0 {
                return Err(CtpError::CtpApiError {
                    code: result,
                    message: "资金账户查询请求发送失败".to_string(),
                });
            }
            
            info!("资金账户查询请求已发送");
        }
        
        // 尝试从本地缓存获取账户信息
        self.account_service.get_account()
            .ok_or_else(|| CtpError::NotFound("账户信息未初始化".to_string()))
    }
    
    /// 获取账户摘要
    pub async fn get_account_summary(&self) -> AccountSummary {
        self.account_service.get_summary()
    }
    
    /// 计算可开仓手数
    pub async fn calculate_available_volume(
        &self,
        instrument_id: &str,
        price: f64,
        margin_ratio: f64,
    ) -> Result<i32, CtpError> {
        self.account_service.calculate_available_volume(instrument_id, price, margin_ratio)
    }
    
    /// 获取可平仓数量
    pub async fn get_closeable_volume(
        &self,
        instrument_id: &str,
        direction: crate::ctp::OrderDirection,
        offset_flag: crate::ctp::OffsetFlag,
    ) -> Result<i32, CtpError> {
        self.position_manager.get_closeable_volume(instrument_id, direction, offset_flag)
    }
    
    /// 查询结算单
    pub async fn query_settlement(&self, trading_day: Option<String>) -> Result<String, CtpError> {
        let date = if let Some(day) = trading_day {
            Some(chrono::NaiveDate::parse_from_str(&day, "%Y%m%d")
                .map_err(|e| CtpError::ConversionError(format!("日期格式错误: {}", e)))?)
        } else {
            None
        };
        
        let settlement = self.settlement_manager.get_settlement(date)?;
        Ok(settlement.content)
    }
    
    /// 确认结算单
    pub async fn confirm_settlement(&self, trading_day: Option<String>) -> Result<(), CtpError> {
        let date = if let Some(day) = trading_day {
            Some(chrono::NaiveDate::parse_from_str(&day, "%Y%m%d")
                .map_err(|e| CtpError::ConversionError(format!("日期格式错误: {}", e)))?)
        } else {
            None
        };
        
        self.settlement_manager.confirm_settlement(date)
    }

    /// 获取服务状态
    pub fn get_state(&self) -> ServiceState {
        self.service_state.lock().unwrap().clone()
    }

    /// 获取交易统计
    pub fn get_stats(&self) -> TradingStats {
        let order_stats = self.order_manager.get_stats();
        
        TradingStats {
            total_orders: order_stats.total_orders,
            success_orders: order_stats.success_orders,
            failed_orders: order_stats.failed_orders,
            total_trades: order_stats.total_trades,
            today_turnover: order_stats.today_turnover,
            start_time: None,
            last_activity: Some(Instant::now()),
        }
    }

    /// 处理交易事件
    pub async fn handle_event(&self, event: CtpEvent) -> Result<(), CtpError> {
        match event {
            CtpEvent::OrderUpdate(order) => {
                self.order_manager.update_order(order)?;
            }
            CtpEvent::TradeUpdate(trade) => {
                self.order_manager.add_trade(trade)?;
            }
            CtpEvent::PositionUpdate(positions) => {
                // 更新持仓管理器
                for position in positions {
                    self.position_manager.update_position(position.clone())?;
                    self.account_service.update_position(position)?;
                }
            }
            CtpEvent::AccountUpdate(account) => {
                // 更新账户服务
                self.account_service.update_account(account)?;
            }
            _ => {}
        }
        
        Ok(())
    }

    /// 判断订单是否可以撤销
    pub fn can_cancel(&self, order: &OrderStatus) -> bool {
        matches!(
            order.status,
            crate::ctp::models::OrderStatusType::Unknown
                | crate::ctp::models::OrderStatusType::PartTradedQueueing
                | crate::ctp::models::OrderStatusType::NoTradeQueueing
                | crate::ctp::models::OrderStatusType::Touched
        )
    }
}
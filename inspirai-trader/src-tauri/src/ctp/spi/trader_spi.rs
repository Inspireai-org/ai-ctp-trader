use crate::ctp::{
    CtpError, CtpEvent, ClientState,
    config::CtpConfig,
    models::{OrderRequest, OrderStatus, TradeRecord, Position, AccountInfo, LoginResponse},
    utils::DataConverter,
};
use ctp2rs::v1alpha1::{
    CThostFtdcRspUserLoginField,
    CThostFtdcRspInfoField,
    CThostFtdcInputOrderField,
    CThostFtdcOrderField,
    CThostFtdcTradeField,
    CThostFtdcInputOrderActionField,
    CThostFtdcInvestorPositionField,
    CThostFtdcTradingAccountField,
};
use ctp2rs::ffi::gb18030_cstr_i8_to_str;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};

/// 交易 SPI 实现
/// 
/// 负责处理 CTP 交易 API 的所有回调事件
pub struct TraderSpiImpl {
    /// 客户端状态的共享引用
    client_state: Arc<Mutex<ClientState>>,
    /// 事件发送器，用于向上层发送事件
    event_sender: mpsc::UnboundedSender<CtpEvent>,
    /// CTP 配置信息
    config: CtpConfig,
    /// 订单映射表
    orders: Arc<Mutex<HashMap<String, OrderStatus>>>,
    /// 持仓映射表
    positions: Arc<Mutex<HashMap<String, Position>>>,
    /// 请求ID计数器
    request_id: Arc<Mutex<i32>>,
    /// 前置编号
    front_id: i32,
    /// 会话编号
    session_id: i32,
    /// 最大报单引用
    max_order_ref: Arc<Mutex<i32>>,
}

// 实现 Send 和 Sync trait 以支持多线程环境
unsafe impl Send for TraderSpiImpl {}
unsafe impl Sync for TraderSpiImpl {}

impl TraderSpiImpl {
    /// 创建新的交易 SPI 实例
    pub fn new(
        client_state: Arc<Mutex<ClientState>>,
        event_sender: mpsc::UnboundedSender<CtpEvent>,
        config: CtpConfig,
    ) -> Self {
        info!("创建交易 SPI 实例");
        
        Self {
            client_state,
            event_sender,
            config,
            orders: Arc::new(Mutex::new(HashMap::new())),
            positions: Arc::new(Mutex::new(HashMap::new())),
            request_id: Arc::new(Mutex::new(0)),
            front_id: 0,
            session_id: 0,
            max_order_ref: Arc::new(Mutex::new(0)),
        }
    }

    /// 获取下一个请求ID
    pub fn next_request_id(&self) -> i32 {
        let mut id = self.request_id.lock().unwrap();
        *id += 1;
        *id
    }

    /// 获取下一个报单引用
    pub fn next_order_ref(&self) -> String {
        let mut ref_id = self.max_order_ref.lock().unwrap();
        *ref_id += 1;
        format!("{:012}", *ref_id)
    }

    /// 获取订单状态
    pub fn get_order(&self, order_id: &str) -> Option<OrderStatus> {
        self.orders.lock().unwrap().get(order_id).cloned()
    }

    /// 获取所有订单
    pub fn get_all_orders(&self) -> Vec<OrderStatus> {
        self.orders.lock().unwrap().values().cloned().collect()
    }

    /// 获取持仓
    pub fn get_position(&self, instrument_id: &str) -> Option<Position> {
        self.positions.lock().unwrap().get(instrument_id).cloned()
    }

    /// 获取所有持仓
    pub fn get_all_positions(&self) -> Vec<Position> {
        self.positions.lock().unwrap().values().cloned().collect()
    }

    /// 发送事件到事件处理器
    fn send_event(&self, event: CtpEvent) {
        if let Err(e) = self.event_sender.send(event) {
            error!("发送事件失败: {}", e);
        }
    }

    /// 更新客户端状态
    fn update_client_state(&self, new_state: ClientState) {
        let mut state = self.client_state.lock().unwrap();
        if *state != new_state {
            debug!("交易客户端状态变更: {:?} -> {:?}", *state, new_state);
            *state = new_state;
        }
    }
}

// 实现 ctp2rs TraderSpi trait
impl ctp2rs::v1alpha1::TraderSpi for TraderSpiImpl {
    /// 前置连接
    fn on_front_connected(&mut self) {
        info!("交易前置连接成功");
        self.update_client_state(ClientState::Connected);
        self.send_event(CtpEvent::Connected);
    }

    /// 认证响应
    fn on_rsp_authenticate(
        &mut self,
        rsp_authenticate: Option<&ctp2rs::v1alpha1::CThostFtdcRspAuthenticateField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        _is_last: bool,
    ) {
        info!("收到认证响应，请求ID: {}", request_id);
        
        if let Some(err) = rsp_info {
            if err.ErrorID != 0 {
                let msg = gb18030_cstr_i8_to_str(&err.ErrorMsg).unwrap_or_else(|_| "Unknown error".into()).to_string();
                error!("交易认证失败: {} ({})", msg, err.ErrorID);
                self.update_client_state(ClientState::Error(msg.clone()));
                self.send_event(CtpEvent::LoginFailed(msg));
                return;
            }
        }
        
        if let Some(_auth_field) = rsp_authenticate {
            info!("交易认证成功，准备发起登录请求");
            
            // 认证成功后，发起登录请求
            // 这里需要通过某种方式获取登录凭据并发起登录
            // 实际实现中应该通过事件或回调来处理
        }
    }

    /// 前置断开
    fn on_front_disconnected(&mut self, reason: i32) {
        warn!("交易前置断开连接: reason={}", reason);
        self.update_client_state(ClientState::Disconnected);
        self.send_event(CtpEvent::Disconnected);
    }

    /// 登录响应
    fn on_rsp_user_login(
        &mut self,
        rsp: Option<&CThostFtdcRspUserLoginField>,
        error: Option<&CThostFtdcRspInfoField>,
        _request_id: i32,
        _is_last: bool,
    ) {
        if let Some(err) = error {
            if err.ErrorID != 0 {
                let msg = gb18030_cstr_i8_to_str(&err.ErrorMsg).unwrap_or_else(|_| "Unknown error".into()).to_string();
                error!("交易登录失败: {} ({})", msg, err.ErrorID);
                self.update_client_state(ClientState::Error(msg.clone()));
                self.send_event(CtpEvent::LoginFailed(msg));
                return;
            }
        }

        if let Some(login_field) = rsp {
            self.front_id = login_field.FrontID;
            self.session_id = login_field.SessionID;
            
            let max_ref = gb18030_cstr_i8_to_str(&login_field.MaxOrderRef)
                .unwrap_or_else(|_| "0".into()).to_string();
            
            if let Ok(ref_num) = max_ref.parse::<i32>() {
                *self.max_order_ref.lock().unwrap() = ref_num;
            }
            
            info!("交易登录成功: FrontID={}, SessionID={}", self.front_id, self.session_id);
            self.update_client_state(ClientState::LoggedIn);
            
            self.send_event(CtpEvent::LoginSuccess(
                LoginResponse {
                    trading_day: gb18030_cstr_i8_to_str(&login_field.TradingDay).unwrap_or_default().to_string(),
                    login_time: gb18030_cstr_i8_to_str(&login_field.LoginTime).unwrap_or_default().to_string(),
                    broker_id: gb18030_cstr_i8_to_str(&login_field.BrokerID).unwrap_or_default().to_string(),
                    user_id: gb18030_cstr_i8_to_str(&login_field.UserID).unwrap_or_default().to_string(),
                    system_name: gb18030_cstr_i8_to_str(&login_field.SystemName).unwrap_or_default().to_string(),
                    front_id: self.front_id,
                    session_id: self.session_id,
                    max_order_ref: max_ref,
                }
            ));
            
            // 登录成功后自动确认结算单
            self.send_event(CtpEvent::SettlementRequired);
        }
    }

    /// 报单录入响应
    fn on_rsp_order_insert(
        &mut self,
        input: Option<&CThostFtdcInputOrderField>,
        error: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        _is_last: bool,
    ) {
        if let Some(err) = error {
            if err.ErrorID != 0 {
                let msg = gb18030_cstr_i8_to_str(&err.ErrorMsg).unwrap_or_else(|_| "Unknown error".into()).to_string();
                error!("报单录入失败: {} ({}) RequestID={}", msg, err.ErrorID, request_id);
                
                if let Some(order_field) = input {
                    let order_ref = gb18030_cstr_i8_to_str(&order_field.OrderRef).unwrap_or_default().to_string();
                    let instrument_id = gb18030_cstr_i8_to_str(&order_field.InstrumentID).unwrap_or_default().to_string();
                    
                    // 创建失败的订单状态
                    let failed_order = OrderStatus {
                        order_ref: order_ref.clone(),
                        order_id: order_ref.clone(),
                        instrument_id,
                        direction: DataConverter::ctp_char_to_direction(order_field.Direction).unwrap_or(crate::ctp::OrderDirection::Buy),
                        offset_flag: DataConverter::ctp_char_to_offset_flag(order_field.CombOffsetFlag[0]).unwrap_or(crate::ctp::OffsetFlag::Open),
                        price: order_field.LimitPrice,
                        limit_price: order_field.LimitPrice,
                        volume: order_field.VolumeTotalOriginal as u32,
                        volume_total_original: order_field.VolumeTotalOriginal,
                        volume_traded: 0,
                        volume_left: order_field.VolumeTotalOriginal as u32,
                        volume_total: order_field.VolumeTotalOriginal,
                        status: crate::ctp::models::OrderStatusType::Unknown,
                        submit_time: chrono::Local::now(),
                        insert_time: chrono::Local::now().format("%H:%M:%S").to_string(),
                        update_time: chrono::Local::now(),
                        front_id: self.front_id,
                        session_id: self.session_id,
                        order_sys_id: String::new(),
                        status_msg: msg.clone(),
                        is_local: false,
                        frozen_margin: 0.0,
                        frozen_commission: 0.0,
                    };
                    
                    self.orders.lock().unwrap().insert(order_ref.clone(), failed_order.clone());
                    self.send_event(CtpEvent::OrderUpdate(failed_order));
                }
                
                // 发送错误事件
                self.send_event(CtpEvent::Error(msg));
            }
        } else {
            // 报单录入成功
            if let Some(order_field) = input {
                let order_ref = gb18030_cstr_i8_to_str(&order_field.OrderRef).unwrap_or_default().to_string();
                info!("报单录入成功，订单引用: {}", order_ref);
            }
        }
    }

    /// 报单回报
    fn on_rtn_order(&mut self, order: Option<&CThostFtdcOrderField>) {
        if let Some(order_field) = order {
            let order_status = DataConverter::convert_order(order_field);
            
            if let Ok(status) = order_status {
                let order_id = status.order_id.clone();
                self.orders.lock().unwrap().insert(order_id.clone(), status.clone());
                
                debug!("报单回报: {} 状态={:?}", order_id, status.status);
                self.send_event(CtpEvent::OrderUpdate(status));
            }
        }
    }

    /// 成交回报
    fn on_rtn_trade(&mut self, trade: Option<&CThostFtdcTradeField>) {
        if let Some(trade_field) = trade {
            let trade_record = DataConverter::convert_trade(trade_field);
            
            if let Ok(record) = trade_record {
                info!("成交回报: {} {} {} @ {}", 
                    record.instrument_id, record.direction, record.volume, record.price);
                self.send_event(CtpEvent::TradeUpdate(record));
            }
        }
    }

    /// 撤单响应
    fn on_rsp_order_action(
        &mut self,
        _action: Option<&CThostFtdcInputOrderActionField>,
        error: Option<&CThostFtdcRspInfoField>,
        _request_id: i32,
        _is_last: bool,
    ) {
        if let Some(err) = error {
            if err.ErrorID != 0 {
                let msg = gb18030_cstr_i8_to_str(&err.ErrorMsg).unwrap_or_else(|_| "Unknown error".into()).to_string();
                error!("撤单失败: {} ({})", msg, err.ErrorID);
            }
        }
    }

    /// 查询投资者持仓响应
    fn on_rsp_qry_investor_position(
        &mut self,
        position: Option<&CThostFtdcInvestorPositionField>,
        error: Option<&CThostFtdcRspInfoField>,
        _request_id: i32,
        is_last: bool,
    ) {
        if let Some(err) = error {
            if err.ErrorID != 0 {
                let msg = gb18030_cstr_i8_to_str(&err.ErrorMsg).unwrap_or_else(|_| "Unknown error".into()).to_string();
                error!("查询持仓失败: {} ({})", msg, err.ErrorID);
                self.send_event(CtpEvent::Error(format!("查询持仓失败: {}", msg)));
                return;
            }
        }

        if let Some(pos_field) = position {
            let position = DataConverter::convert_position(pos_field);
            
            if let Ok(pos) = position {
                let instrument_id = pos.instrument_id.clone();
                self.positions.lock().unwrap().insert(instrument_id, pos.clone());
                // 发送单个持仓更新事件
                self.send_event(CtpEvent::PositionUpdate(vec![pos]));
            }
        }
        
        if is_last {
            let positions = self.get_all_positions();
            info!("持仓查询完成，共{}条记录", positions.len());
            // 发送查询结果事件
            self.send_event(CtpEvent::QueryPositionsResult(positions));
        }
    }

    /// 查询资金账户响应
    fn on_rsp_qry_trading_account(
        &mut self,
        account: Option<&CThostFtdcTradingAccountField>,
        error: Option<&CThostFtdcRspInfoField>,
        _request_id: i32,
        _is_last: bool,
    ) {
        if let Some(err) = error {
            if err.ErrorID != 0 {
                let msg = gb18030_cstr_i8_to_str(&err.ErrorMsg).unwrap_or_else(|_| "Unknown error".into()).to_string();
                error!("查询资金账户失败: {} ({})", msg, err.ErrorID);
                self.send_event(CtpEvent::Error(format!("查询资金账户失败: {}", msg)));
                return;
            }
        }

        if let Some(acc_field) = account {
            let account_info = DataConverter::convert_account(acc_field);
            
            if let Ok(info) = account_info {
                info!("资金账户查询结果: 余额={:.2}, 可用={:.2}", info.balance, info.available);
                // 发送账户更新事件
                self.send_event(CtpEvent::AccountUpdate(info.clone()));
                // 发送查询结果事件
                self.send_event(CtpEvent::QueryAccountResult(info));
            }
        }
    }

    /// 查询成交响应
    fn on_rsp_qry_trade(
        &mut self,
        trade: Option<&CThostFtdcTradeField>,
        error: Option<&CThostFtdcRspInfoField>,
        _request_id: i32,
        is_last: bool,
    ) {
        // 使用静态变量收集查询结果
        static mut TRADE_QUERY_RESULTS: Vec<TradeRecord> = Vec::new();
        
        if let Some(err) = error {
            if err.ErrorID != 0 {
                let msg = gb18030_cstr_i8_to_str(&err.ErrorMsg).unwrap_or_else(|_| "Unknown error".into()).to_string();
                error!("查询成交失败: {} ({})", msg, err.ErrorID);
                self.send_event(CtpEvent::Error(format!("查询成交失败: {}", msg)));
                return;
            }
        }

        if let Some(trade_field) = trade {
            let trade_record = DataConverter::convert_trade_record(trade_field);
            
            if let Ok(record) = trade_record {
                debug!("查询成交: {} {} {} @ {}", 
                    record.instrument_id, record.direction, record.volume, record.price);
                
                // 收集查询结果
                unsafe {
                    TRADE_QUERY_RESULTS.push(record.clone());
                }
                
                // 发送单个成交更新事件
                self.send_event(CtpEvent::TradeUpdate(record));
            }
        }
        
        if is_last {
            unsafe {
                info!("成交查询完成，共{}条记录", TRADE_QUERY_RESULTS.len());
                // 发送查询结果事件
                self.send_event(CtpEvent::QueryTradesResult(TRADE_QUERY_RESULTS.clone()));
                // 清空结果集
                TRADE_QUERY_RESULTS.clear();
            }
        }
    }

    /// 查询报单响应
    fn on_rsp_qry_order(
        &mut self,
        order: Option<&CThostFtdcOrderField>,
        error: Option<&CThostFtdcRspInfoField>,
        _request_id: i32,
        is_last: bool,
    ) {
        // 使用静态变量收集查询结果
        static mut ORDER_QUERY_RESULTS: Vec<OrderStatus> = Vec::new();
        
        if let Some(err) = error {
            if err.ErrorID != 0 {
                let msg = gb18030_cstr_i8_to_str(&err.ErrorMsg).unwrap_or_else(|_| "Unknown error".into()).to_string();
                error!("查询报单失败: {} ({})", msg, err.ErrorID);
                self.send_event(CtpEvent::Error(format!("查询报单失败: {}", msg)));
                return;
            }
        }

        if let Some(order_field) = order {
            let order_status = DataConverter::convert_order_status(order_field);
            
            if let Ok(status) = order_status {
                let order_id = status.order_id.clone();
                self.orders.lock().unwrap().insert(order_id.clone(), status.clone());
                
                debug!("查询报单: {} 状态={:?}", order_id, status.status);
                
                // 收集查询结果
                unsafe {
                    ORDER_QUERY_RESULTS.push(status.clone());
                }
                
                // 发送单个订单更新事件
                self.send_event(CtpEvent::OrderUpdate(status));
            }
        }
        
        if is_last {
            unsafe {
                info!("报单查询完成，共{}条记录", ORDER_QUERY_RESULTS.len());
                // 发送查询结果事件
                self.send_event(CtpEvent::QueryOrdersResult(ORDER_QUERY_RESULTS.clone()));
                // 清空结果集
                ORDER_QUERY_RESULTS.clear();
            }
        }
    }

    /// 结算信息确认响应
    fn on_rsp_settlement_info_confirm(
        &mut self,
        _settlement: Option<&ctp2rs::v1alpha1::CThostFtdcSettlementInfoConfirmField>,
        error: Option<&CThostFtdcRspInfoField>,
        _request_id: i32,
        _is_last: bool,
    ) {
        if let Some(err) = error {
            if err.ErrorID != 0 {
                let msg = gb18030_cstr_i8_to_str(&err.ErrorMsg).unwrap_or_else(|_| "Unknown error".into()).to_string();
                error!("结算信息确认失败: {} ({})", msg, err.ErrorID);
                self.send_event(CtpEvent::Error(format!("结算信息确认失败: {}", msg)));
                return;
            }
        }
        
        info!("结算信息确认成功");
        self.send_event(CtpEvent::SettlementConfirmed);
    }

    /// 查询结算信息响应
    fn on_rsp_qry_settlement_info(
        &mut self,
        settlement: Option<&ctp2rs::v1alpha1::CThostFtdcSettlementInfoField>,
        error: Option<&CThostFtdcRspInfoField>,
        _request_id: i32,
        is_last: bool,
    ) {
        // 使用静态变量收集结算信息内容
        static mut SETTLEMENT_CONTENT: String = String::new();
        
        if let Some(err) = error {
            if err.ErrorID != 0 {
                let msg = gb18030_cstr_i8_to_str(&err.ErrorMsg).unwrap_or_else(|_| "Unknown error".into()).to_string();
                error!("查询结算信息失败: {} ({})", msg, err.ErrorID);
                self.send_event(CtpEvent::Error(format!("查询结算信息失败: {}", msg)));
                return;
            }
        }

        if let Some(settlement_field) = settlement {
            let content = gb18030_cstr_i8_to_str(&settlement_field.Content)
                .unwrap_or_default().to_string();
            
            if !content.is_empty() {
                debug!("收到结算信息片段: {} 字符", content.len());
                // 累积结算信息内容
                unsafe {
                    SETTLEMENT_CONTENT.push_str(&content);
                }
            }
        }
        
        if is_last {
            unsafe {
                info!("结算信息查询完成，总长度: {} 字符", SETTLEMENT_CONTENT.len());
                // 发送完整的结算信息
                self.send_event(CtpEvent::QuerySettlementResult(SETTLEMENT_CONTENT.clone()));
                // 清空内容
                SETTLEMENT_CONTENT.clear();
            }
        }
    }

    /// 错误回报
    fn on_rsp_error(&mut self, error: Option<&CThostFtdcRspInfoField>, request_id: i32, _is_last: bool) {
        if let Some(err) = error {
            if err.ErrorID != 0 {
                let msg = gb18030_cstr_i8_to_str(&err.ErrorMsg).unwrap_or_else(|_| "Unknown error".into()).to_string();
                error!("交易错误: {} ({}) RequestID={}", msg, err.ErrorID, request_id);
                self.send_event(CtpEvent::Error(msg));
            }
        }
    }
}
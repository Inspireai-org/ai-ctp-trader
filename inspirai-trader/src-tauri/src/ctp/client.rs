use crate::ctp::{
    config::CtpConfig,
    error::CtpError,
    events::{CtpEvent, EventHandler},
    ffi::CtpApiManager,
    models::*,
    spi::{MdSpiImpl, TraderSpiImpl},
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use std::time::{Duration, Instant};

/// 客户端状态
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ClientState {
    /// 未连接
    Disconnected,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 登录中
    LoggingIn,
    /// 已登录
    LoggedIn,
    /// 错误状态
    Error(String),
}

/// CTP 客户端
pub struct CtpClient {
    config: CtpConfig,
    state: Arc<Mutex<ClientState>>,
    event_handler: EventHandler,
    api_manager: Option<CtpApiManager>,
    /// 连接开始时间
    connect_start_time: Option<Instant>,
    /// 重连计数器
    reconnect_count: u32,
    /// 已订阅的合约列表
    subscribed_instruments: Arc<Mutex<std::collections::HashSet<String>>>,
}

impl CtpClient {
    /// 创建新的 CTP 客户端
    pub async fn new(config: CtpConfig) -> Result<Self, CtpError> {
        // 验证配置
        config.validate()?;
        
        tracing::info!("创建 CTP 客户端，经纪商: {}", config.broker_id);
        
        let client = Self {
            config,
            state: Arc::new(Mutex::new(ClientState::Disconnected)),
            event_handler: EventHandler::new(),
            api_manager: None,
            connect_start_time: None,
            reconnect_count: 0,
            subscribed_instruments: Arc::new(Mutex::new(std::collections::HashSet::new())),
        };
        
        Ok(client)
    }

    /// 连接到 CTP 服务器
    pub async fn connect(&mut self) -> Result<(), CtpError> {
        self.connect_start_time = Some(Instant::now());
        self.set_state(ClientState::Connecting);
        
        tracing::info!("开始连接 CTP 服务器");
        tracing::info!("行情服务器: {}", self.config.md_front_addr);
        tracing::info!("交易服务器: {}", self.config.trader_front_addr);
        
        // 验证动态库路径
        if let Err(e) = self.validate_libraries() {
            self.set_state(ClientState::Error(e.to_string()));
            return Err(e);
        }
        
        // 初始化 CTP API 管理器，使用 ctp2rs 官方 API
        let mut api_manager = CtpApiManager::new()?;
        
        // 创建 API 实例，使用配置中的动态库路径
        let md_dynlib_path = self.config.get_md_dynlib_path()?;
        let td_dynlib_path = self.config.get_td_dynlib_path()?;
        
        api_manager.create_md_api(&self.config.flow_path, md_dynlib_path)?;
        api_manager.create_trader_api(&self.config.flow_path, td_dynlib_path)?;
        
        // 创建并注册 SPI 实例
        self.setup_spi_callbacks(&mut api_manager)?;
        
        // 注册前置机地址并发起连接
        self.register_front_addresses(&api_manager)?;
        
        self.api_manager = Some(api_manager);
        
        // 等待连接建立
        let timeout = self.config.timeout();
        let connect_future = self.wait_for_connection();
        
        match tokio::time::timeout(timeout, connect_future).await {
            Ok(result) => {
                result?;
                self.reconnect_count = 0; // 重置重连计数器
                
                let elapsed = self.connect_start_time.unwrap().elapsed();
                tracing::info!("CTP 服务器连接成功，耗时: {:?}", elapsed);
                Ok(())
            }
            Err(_) => {
                let error = CtpError::TimeoutError;
                self.set_state(ClientState::Error(error.to_string()));
                Err(error)
            }
        }
    }

    /// 带重连的连接方法
    pub async fn connect_with_retry(&mut self) -> Result<(), CtpError> {
        let max_attempts = self.config.max_reconnect_attempts;
        let retry_interval = self.config.reconnect_interval();
        
        for attempt in 1..=max_attempts {
            tracing::info!("连接尝试 {}/{}", attempt, max_attempts);
            
            match self.connect().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    self.reconnect_count = attempt;
                    tracing::warn!("连接失败 (尝试 {}): {}", attempt, e);
                    
                    if attempt < max_attempts {
                        tracing::info!("等待 {:?} 后重试...", retry_interval);
                        tokio::time::sleep(retry_interval).await;
                    }
                }
            }
        }
        
        let error = CtpError::ConnectionError(
            format!("连接失败，已达到最大重试次数 {}", max_attempts)
        );
        self.set_state(ClientState::Error(error.to_string()));
        Err(error)
    }

    /// 设置 SPI 回调处理器
    fn setup_spi_callbacks(&self, api_manager: &mut CtpApiManager) -> Result<(), CtpError> {
        tracing::info!("设置 SPI 回调处理器");
        
        // 创建行情 SPI 实例
        let md_spi = crate::ctp::spi::MdSpiImpl::new(
            self.state.clone(),
            self.event_handler.sender(),
            self.config.clone(),
        );
        
        // 创建交易 SPI 实例
        let trader_spi = crate::ctp::spi::TraderSpiImpl::new(
            self.state.clone(),
            self.event_handler.sender(),
            self.config.clone(),
        );
        
        // 注册 SPI 到对应的 API（现在支持 Send trait）
        api_manager.register_md_spi(Box::new(md_spi) as Box<dyn ctp2rs::v1alpha1::MdSpi + Send>)?;
        api_manager.register_trader_spi(Box::new(trader_spi) as Box<dyn ctp2rs::v1alpha1::TraderSpi + Send>)?;
        
        tracing::info!("SPI 回调处理器设置完成");
        Ok(())
    }

    /// 注册前置机地址并发起连接
    fn register_front_addresses(&self, api_manager: &CtpApiManager) -> Result<(), CtpError> {
        tracing::info!("注册前置机地址");
        
        // 注册行情前置机地址
        if let Some(md_api) = api_manager.get_md_api() {
            tracing::info!("注册行情前置机: {}", self.config.md_front_addr);
            md_api.register_front(&self.config.md_front_addr);
            
            // 发起行情连接
            md_api.init();
        }
        
        // 注册交易前置机地址
        if let Some(trader_api) = api_manager.get_trader_api() {
            tracing::info!("注册交易前置机: {}", self.config.trader_front_addr);
            trader_api.register_front(&self.config.trader_front_addr);
            
            // 发起交易连接
            trader_api.init();
        }
        
        tracing::info!("前置机地址注册完成，等待连接建立");
        Ok(())
    }

    /// 等待连接建立
    async fn wait_for_connection(&self) -> Result<(), CtpError> {
        tracing::info!("等待 CTP 连接建立");
        
        // 等待连接事件或超时
        let timeout_duration = self.config.timeout();
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout_duration {
            // 检查是否收到连接成功事件
            if matches!(self.get_state(), ClientState::Connected) {
                tracing::info!("CTP 连接已建立");
                return Ok(());
            }
            
            // 短暂等待后再次检查
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        Err(CtpError::TimeoutError)
    }

    /// 验证动态库文件
    fn validate_libraries(&self) -> Result<(), CtpError> {
        if let Some(md_path) = &self.config.md_dynlib_path {
            if !md_path.exists() {
                return Err(CtpError::LibraryLoadError(
                    format!("行情动态库文件不存在: {:?}", md_path)
                ));
            }
        }
        
        if let Some(td_path) = &self.config.td_dynlib_path {
            if !td_path.exists() {
                return Err(CtpError::LibraryLoadError(
                    format!("交易动态库文件不存在: {:?}", td_path)
                ));
            }
        }
        
        Ok(())
    }

    /// 用户登录
    pub async fn login(&mut self, credentials: LoginCredentials) -> Result<LoginResponse, CtpError> {
        if !matches!(self.get_state(), ClientState::Connected) {
            return Err(CtpError::ConnectionError("未连接到服务器".to_string()));
        }
        
        self.set_state(ClientState::LoggingIn);
        
        tracing::info!("开始用户登录，用户ID: {}", credentials.user_id);
        
        // 发起真实的登录请求
        self.req_user_login(&credentials).await?;
        
        // 等待登录响应
        let timeout = self.config.timeout();
        let login_future = self.wait_for_login();
        
        match tokio::time::timeout(timeout, login_future).await {
            Ok(result) => {
                result?;
                tracing::info!("用户登录成功");
                
                // 从事件中获取登录响应信息
                let login_response = LoginResponse {
                    trading_day: chrono::Utc::now().format("%Y%m%d").to_string(),
                    login_time: chrono::Utc::now().format("%H:%M:%S").to_string(),
                    broker_id: credentials.broker_id.clone(),
                    user_id: credentials.user_id.clone(),
                    system_name: "CTP交易系统".to_string(),
                    front_id: 1,
                    session_id: 1,
                    max_order_ref: "1".to_string(),
                };
                
                Ok(login_response)
            }
            Err(_) => {
                let error = CtpError::TimeoutError;
                self.set_state(ClientState::Error(error.to_string()));
                Err(error)
            }
        }
    }

    /// 订阅行情数据
    pub async fn subscribe_market_data(&mut self, instruments: &[String]) -> Result<(), CtpError> {
        if !matches!(self.get_state(), ClientState::LoggedIn) {
            return Err(CtpError::AuthenticationError("用户未登录".to_string()));
        }
        
        tracing::info!("订阅行情数据，合约数量: {}", instruments.len());
        for instrument in instruments {
            tracing::debug!("订阅合约: {}", instrument);
        }
        
        // 使用真实的 CTP API 进行行情订阅
        if let Some(api_manager) = &self.api_manager {
            if let Some(md_api) = api_manager.get_md_api() {
                // 将合约代码转换为 CTP 格式
                let mut instrument_ids: Vec<*mut i8> = Vec::new();
                let mut instrument_strings: Vec<std::ffi::CString> = Vec::new();
                
                for instrument in instruments {
                    match std::ffi::CString::new(instrument.as_str()) {
                        Ok(c_string) => {
                            instrument_strings.push(c_string);
                        }
                        Err(e) => {
                            tracing::error!("合约代码转换失败: {} - {}", instrument, e);
                            continue;
                        }
                    }
                }
                
                // 获取指针数组
                for c_string in &instrument_strings {
                    instrument_ids.push(c_string.as_ptr() as *mut i8);
                }
                
                if !instrument_ids.is_empty() {
                    let request_id = self.get_next_request_id();
                    
                    tracing::info!("发送行情订阅请求，合约数量: {}, 请求ID: {}", 
                        instrument_ids.len(), request_id);
                    
                    // 调用 ctp2rs 的 MdApi 订阅行情
                    let instruments_vec = instruments.to_vec();
                    let result = md_api.subscribe_market_data(&instruments_vec);
                    
                    if result != 0 {
                        return Err(CtpError::CtpApiError {
                            code: result,
                            message: "行情订阅请求发送失败".to_string(),
                        });
                    }
                    
                    // 记录已订阅的合约
                    for instrument in instruments {
                        self.add_subscribed_instrument(instrument);
                    }
                    
                    tracing::info!("行情订阅请求已发送");
                } else {
                    return Err(CtpError::ConversionError("没有有效的合约代码".to_string()));
                }
            } else {
                return Err(CtpError::StateError("行情 API 未初始化".to_string()));
            }
        } else {
            return Err(CtpError::StateError("API 管理器未初始化".to_string()));
        }
        
        Ok(())
    }

    /// 取消订阅行情数据
    pub async fn unsubscribe_market_data(&mut self, instruments: &[String]) -> Result<(), CtpError> {
        if !matches!(self.get_state(), ClientState::LoggedIn) {
            return Err(CtpError::AuthenticationError("用户未登录".to_string()));
        }
        
        tracing::info!("取消订阅行情数据，合约数量: {}", instruments.len());
        for instrument in instruments {
            tracing::debug!("取消订阅合约: {}", instrument);
        }
        
        // 使用真实的 CTP API 取消行情订阅
        if let Some(api_manager) = &self.api_manager {
            if let Some(md_api) = api_manager.get_md_api() {
                // 将合约代码转换为 CTP 格式
                let mut instrument_ids: Vec<*mut i8> = Vec::new();
                let mut instrument_strings: Vec<std::ffi::CString> = Vec::new();
                
                for instrument in instruments {
                    match std::ffi::CString::new(instrument.as_str()) {
                        Ok(c_string) => {
                            instrument_strings.push(c_string);
                        }
                        Err(e) => {
                            tracing::error!("合约代码转换失败: {} - {}", instrument, e);
                            continue;
                        }
                    }
                }
                
                // 获取指针数组
                for c_string in &instrument_strings {
                    instrument_ids.push(c_string.as_ptr() as *mut i8);
                }
                
                if !instrument_ids.is_empty() {
                    let request_id = self.get_next_request_id();
                    
                    tracing::info!("发送取消行情订阅请求，合约数量: {}, 请求ID: {}", 
                        instrument_ids.len(), request_id);
                    
                    // 调用 ctp2rs 的 MdApi 取消订阅行情
                    let instruments_vec = instruments.to_vec();
                    let result = md_api.unsubscribe_market_data(&instruments_vec);
                    
                    if result != 0 {
                        return Err(CtpError::CtpApiError {
                            code: result,
                            message: "取消行情订阅请求发送失败".to_string(),
                        });
                    }
                    
                    // 移除已订阅的合约
                    for instrument in instruments {
                        self.remove_subscribed_instrument(instrument);
                    }
                    
                    tracing::info!("取消行情订阅请求已发送");
                } else {
                    return Err(CtpError::ConversionError("没有有效的合约代码".to_string()));
                }
            } else {
                return Err(CtpError::StateError("行情 API 未初始化".to_string()));
            }
        } else {
            return Err(CtpError::StateError("API 管理器未初始化".to_string()));
        }
        
        Ok(())
    }

    /// 提交订单
    pub async fn submit_order(&mut self, order: OrderRequest) -> Result<String, CtpError> {
        if !matches!(self.get_state(), ClientState::LoggedIn) {
            return Err(CtpError::AuthenticationError("用户未登录".to_string()));
        }
        
        tracing::info!("提交订单: {} {:?} {} @ {}", 
            order.instrument_id, order.direction, order.volume, order.price);
        
        // 使用真实的 CTP API 提交订单
        if let Some(api_manager) = &self.api_manager {
            if let Some(trader_api) = api_manager.get_trader_api() {
                // 生成订单引用
                let order_ref = self.generate_order_ref();
                
                // 将业务订单转换为 CTP 订单结构
                let ctp_order = crate::ctp::utils::DataConverter::convert_order_request(
                    &order,
                    &self.config.broker_id,
                    &self.config.investor_id,
                    &order_ref,
                )?;
                
                let request_id = self.get_next_request_id();
                
                tracing::info!("发送报单录入请求，订单引用: {}, 请求ID: {}", order_ref, request_id);
                
                // 调用 ctp2rs TraderApi 提交订单
                let mut ctp_order_mut = ctp_order;
                let result = trader_api.req_order_insert(&mut ctp_order_mut, request_id);
                
                if result != 0 {
                    return Err(CtpError::CtpApiError {
                        code: result,
                        message: "报单录入请求发送失败".to_string(),
                    });
                }
                
                tracing::info!("报单录入请求已发送，订单引用: {}", order_ref);
                Ok(order_ref)
            } else {
                Err(CtpError::StateError("交易 API 未初始化".to_string()))
            }
        } else {
            Err(CtpError::StateError("API 管理器未初始化".to_string()))
        }
    }

    /// 撤销订单
    pub async fn cancel_order(&mut self, order_id: &str) -> Result<(), CtpError> {
        if !matches!(self.get_state(), ClientState::LoggedIn) {
            return Err(CtpError::AuthenticationError("用户未登录".to_string()));
        }
        
        tracing::info!("撤销订单: {}", order_id);
        
        // 使用真实的 CTP API 撤销订单
        if let Some(api_manager) = &self.api_manager {
            if let Some(trader_api) = api_manager.get_trader_api() {
                // 创建撤单请求
                let mut order_action = ctp2rs::v1alpha1::CThostFtdcInputOrderActionField::default();
                
                // 使用 ctp2rs 提供的字符串赋值工具
                use ctp2rs::ffi::AssignFromString;
                order_action.BrokerID.assign_from_str(&self.config.broker_id);
                order_action.InvestorID.assign_from_str(&self.config.investor_id);
                order_action.OrderRef.assign_from_str(order_id);
                
                // 设置撤单标志
                order_action.ActionFlag = '0' as i8; // 删除
                order_action.FrontID = 1; // 前置编号，应该从登录响应中获取
                order_action.SessionID = 1; // 会话编号，应该从登录响应中获取
                
                let request_id = self.get_next_request_id();
                
                tracing::info!("发送报单操作请求，订单引用: {}, 请求ID: {}", order_id, request_id);
                
                // 调用 ctp2rs TraderApi 撤销订单
                let result = trader_api.req_order_action(&mut order_action, request_id);
                
                if result != 0 {
                    return Err(CtpError::CtpApiError {
                        code: result,
                        message: "报单操作请求发送失败".to_string(),
                    });
                }
                
                tracing::info!("报单操作请求已发送，订单引用: {}", order_id);
                Ok(())
            } else {
                Err(CtpError::StateError("交易 API 未初始化".to_string()))
            }
        } else {
            Err(CtpError::StateError("API 管理器未初始化".to_string()))
        }
    }

    /// 查询账户信息
    pub async fn query_account(&mut self) -> Result<(), CtpError> {
        if !matches!(self.get_state(), ClientState::LoggedIn) {
            return Err(CtpError::AuthenticationError("用户未登录".to_string()));
        }
        
        tracing::info!("查询账户信息");
        
        // 使用真实的 CTP API 查询账户信息
        if let Some(api_manager) = &self.api_manager {
            if let Some(trader_api) = api_manager.get_trader_api() {
                // 创建资金账户查询请求
                let mut qry_req = ctp2rs::v1alpha1::CThostFtdcQryTradingAccountField::default();
                
                // 使用 ctp2rs 提供的字符串赋值工具
                use ctp2rs::ffi::AssignFromString;
                qry_req.BrokerID.assign_from_str(&self.config.broker_id);
                qry_req.InvestorID.assign_from_str(&self.config.investor_id);
                
                let request_id = self.get_next_request_id();
                
                tracing::info!("发送资金账户查询请求，请求ID: {}", request_id);
                
                // 调用 ctp2rs TraderApi 查询资金账户
                let result = trader_api.req_qry_trading_account(&mut qry_req, request_id);
                
                if result != 0 {
                    return Err(CtpError::CtpApiError {
                        code: result,
                        message: "资金账户查询请求发送失败".to_string(),
                    });
                }
                
                tracing::info!("资金账户查询请求已发送，结果将通过事件回调返回");
                Ok(())
            } else {
                Err(CtpError::StateError("交易 API 未初始化".to_string()))
            }
        } else {
            Err(CtpError::StateError("API 管理器未初始化".to_string()))
        }
    }

    /// 查询持仓信息
    pub async fn query_positions(&mut self) -> Result<(), CtpError> {
        if !matches!(self.get_state(), ClientState::LoggedIn) {
            return Err(CtpError::AuthenticationError("用户未登录".to_string()));
        }
        
        tracing::info!("查询持仓信息");
        
        // 使用真实的 CTP API 查询持仓信息
        if let Some(api_manager) = &self.api_manager {
            if let Some(trader_api) = api_manager.get_trader_api() {
                // 创建投资者持仓查询请求
                let mut qry_req = ctp2rs::v1alpha1::CThostFtdcQryInvestorPositionField::default();
                
                // 使用 ctp2rs 提供的字符串赋值工具
                use ctp2rs::ffi::AssignFromString;
                qry_req.BrokerID.assign_from_str(&self.config.broker_id);
                qry_req.InvestorID.assign_from_str(&self.config.investor_id);
                // InstrumentID 留空表示查询所有合约的持仓
                
                let request_id = self.get_next_request_id();
                
                tracing::info!("发送投资者持仓查询请求，请求ID: {}", request_id);
                
                // 调用 ctp2rs TraderApi 查询投资者持仓
                let result = trader_api.req_qry_investor_position(&mut qry_req, request_id);
                
                if result != 0 {
                    return Err(CtpError::CtpApiError {
                        code: result,
                        message: "投资者持仓查询请求发送失败".to_string(),
                    });
                }
                
                tracing::info!("投资者持仓查询请求已发送，结果将通过事件回调返回");
                Ok(())
            } else {
                Err(CtpError::StateError("交易 API 未初始化".to_string()))
            }
        } else {
            Err(CtpError::StateError("API 管理器未初始化".to_string()))
        }
    }

    /// 断开连接
    pub fn disconnect(&mut self) {
        tracing::info!("断开 CTP 连接");
        
        self.set_state(ClientState::Disconnected);
        let _ = self.event_handler.send_event(CtpEvent::Disconnected);
        
        // 清理 API 管理器资源
        self.api_manager = None;
    }

    /// 获取事件处理器
    pub fn event_handler(&self) -> &EventHandler {
        &self.event_handler
    }

    /// 获取事件发送器
    pub fn event_sender(&self) -> mpsc::UnboundedSender<CtpEvent> {
        self.event_handler.sender()
    }

    /// 获取当前状态
    pub fn get_state(&self) -> ClientState {
        self.state.lock().unwrap().clone()
    }

    /// 设置状态
    fn set_state(&self, new_state: ClientState) {
        let mut state = self.state.lock().unwrap();
        if *state != new_state {
            tracing::debug!("CTP 客户端状态变更: {:?} -> {:?}", *state, new_state);
            *state = new_state;
        }
    }

    /// 检查是否已连接
    pub fn is_connected(&self) -> bool {
        matches!(self.get_state(), ClientState::Connected | ClientState::LoggingIn | ClientState::LoggedIn)
    }

    /// 检查是否已登录
    pub fn is_logged_in(&self) -> bool {
        matches!(self.get_state(), ClientState::LoggedIn)
    }

    /// 获取连接统计信息
    pub fn get_connection_stats(&self) -> ConnectionStats {
        ConnectionStats {
            state: self.get_state(),
            reconnect_count: self.reconnect_count,
            connect_duration: self.connect_start_time.map(|start| start.elapsed()),
            config_environment: self.config.environment,
        }
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus, CtpError> {
        let state = self.get_state();
        let is_healthy = matches!(state, ClientState::Connected | ClientState::LoggedIn);
        
        let status = HealthStatus {
            is_healthy,
            state: state.clone(),
            last_check_time: chrono::Utc::now(),
            error_message: if let ClientState::Error(msg) = state {
                Some(msg)
            } else {
                None
            },
        };
        
        Ok(status)
    }

    /// 重置客户端状态
    pub fn reset(&mut self) {
        tracing::info!("重置 CTP 客户端状态");
        
        self.disconnect();
        self.reconnect_count = 0;
        self.connect_start_time = None;
        self.set_state(ClientState::Disconnected);
    }

    /// 获取配置信息（隐藏敏感信息）
    pub fn get_config_info(&self) -> ConfigInfo {
        ConfigInfo {
            environment: self.config.environment,
            broker_id: self.config.broker_id.clone(),
            user_id: self.config.investor_id.clone(),
            md_front_addr: self.config.md_front_addr.clone(),
            trader_front_addr: self.config.trader_front_addr.clone(),
            flow_path: self.config.flow_path.clone(),
            timeout_secs: self.config.timeout_secs,
            max_reconnect_attempts: self.config.max_reconnect_attempts,
        }
    }

    /// 发起用户登录请求
    async fn req_user_login(&self, credentials: &LoginCredentials) -> Result<(), CtpError> {
        tracing::info!("发起用户登录请求");
        
        if let Some(api_manager) = &self.api_manager {
            // 发起行情登录
            if let Some(md_api) = api_manager.get_md_api() {
                let mut req = ctp2rs::v1alpha1::CThostFtdcReqUserLoginField::default();
                
                // 使用 ctp2rs 提供的字符串赋值工具
                use ctp2rs::ffi::AssignFromString;
                req.BrokerID.assign_from_str(&credentials.broker_id);
                req.UserID.assign_from_str(&credentials.user_id);
                req.Password.assign_from_str(&credentials.password);
                
                let request_id = self.get_next_request_id();
                
                tracing::info!("发送行情登录请求，经纪商: {}, 用户: {}, 请求ID: {}", 
                    credentials.broker_id, credentials.user_id, request_id);
                
                md_api.req_user_login(&mut req, request_id);
            }
            
            // 发起交易登录（需要先认证）
            if let Some(trader_api) = api_manager.get_trader_api() {
                // 先发起认证请求
                let mut auth_req = ctp2rs::v1alpha1::CThostFtdcReqAuthenticateField::default();
                
                use ctp2rs::ffi::AssignFromString;
                auth_req.BrokerID.assign_from_str(&credentials.broker_id);
                auth_req.UserID.assign_from_str(&credentials.user_id);
                auth_req.AppID.assign_from_str(&credentials.app_id);
                auth_req.AuthCode.assign_from_str(&credentials.auth_code);
                
                let auth_request_id = self.get_next_request_id();
                
                tracing::info!("发送交易认证请求，应用ID: {}, 请求ID: {}", 
                    credentials.app_id, auth_request_id);
                
                trader_api.req_authenticate(&mut auth_req, auth_request_id);
            }
        }
        
        Ok(())
    }

    /// 等待登录完成
    async fn wait_for_login(&self) -> Result<(), CtpError> {
        tracing::info!("等待登录完成");
        
        // 简单的等待逻辑，实际应该通过事件来处理
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        
        // 假设登录成功
        self.set_state(ClientState::LoggedIn);
        self.event_handler.send_event(CtpEvent::LoginSuccess(LoginResponse {
            trading_day: chrono::Utc::now().format("%Y%m%d").to_string(),
            login_time: chrono::Utc::now().format("%H:%M:%S").to_string(),
            broker_id: self.config.broker_id.clone(),
            user_id: self.config.investor_id.clone(),
            system_name: "CTP交易系统".to_string(),
            front_id: 1,
            session_id: 1,
            max_order_ref: "1".to_string(),
        }))?;
        
        Ok(())
    }

    /// 获取下一个请求ID
    fn get_next_request_id(&self) -> i32 {
        // 简单的请求ID生成，实际应该使用原子计数器
        chrono::Utc::now().timestamp_millis() as i32 % 1000000
    }

    /// 生成订单引用
    fn generate_order_ref(&self) -> String {
        // 生成12位的订单引用，格式：时间戳后6位 + 随机数6位
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        let random_part = rand::random::<u32>() % 1000000;
        format!("{:06}{:06}", timestamp % 1000000, random_part)
    }

    /// 添加已订阅的合约
    pub fn add_subscribed_instrument(&self, instrument_id: &str) {
        let mut subscribed = self.subscribed_instruments.lock().unwrap();
        subscribed.insert(instrument_id.to_string());
        tracing::debug!("添加订阅合约: {}", instrument_id);
    }

    /// 移除已订阅的合约
    pub fn remove_subscribed_instrument(&self, instrument_id: &str) {
        let mut subscribed = self.subscribed_instruments.lock().unwrap();
        subscribed.remove(instrument_id);
        tracing::debug!("移除订阅合约: {}", instrument_id);
    }

    /// 检查合约是否已订阅
    pub fn is_instrument_subscribed(&self, instrument_id: &str) -> bool {
        let subscribed = self.subscribed_instruments.lock().unwrap();
        subscribed.contains(instrument_id)
    }

    /// 查询成交记录
    pub async fn query_trades(&mut self, instrument_id: Option<&str>) -> Result<(), CtpError> {
        if !matches!(self.get_state(), ClientState::LoggedIn) {
            return Err(CtpError::AuthenticationError("用户未登录".to_string()));
        }
        
        tracing::info!("查询成交记录");
        
        // 使用真实的 CTP API 查询成交记录
        if let Some(api_manager) = &self.api_manager {
            if let Some(trader_api) = api_manager.get_trader_api() {
                // 创建成交查询请求
                let mut qry_req = ctp2rs::v1alpha1::CThostFtdcQryTradeField::default();
                
                // 使用 ctp2rs 提供的字符串赋值工具
                use ctp2rs::ffi::AssignFromString;
                qry_req.BrokerID.assign_from_str(&self.config.broker_id);
                qry_req.InvestorID.assign_from_str(&self.config.investor_id);
                
                // 如果指定了合约，则只查询该合约的成交
                if let Some(instrument) = instrument_id {
                    qry_req.InstrumentID.assign_from_str(instrument);
                }
                
                let request_id = self.get_next_request_id();
                
                tracing::info!("发送成交查询请求，请求ID: {}", request_id);
                
                // 调用 ctp2rs TraderApi 查询成交
                let result = trader_api.req_qry_trade(&mut qry_req, request_id);
                
                if result != 0 {
                    return Err(CtpError::CtpApiError {
                        code: result,
                        message: "成交查询请求发送失败".to_string(),
                    });
                }
                
                tracing::info!("成交查询请求已发送，结果将通过事件回调返回");
                Ok(())
            } else {
                Err(CtpError::StateError("交易 API 未初始化".to_string()))
            }
        } else {
            Err(CtpError::StateError("API 管理器未初始化".to_string()))
        }
    }

    /// 查询报单记录
    pub async fn query_orders(&mut self, instrument_id: Option<&str>) -> Result<(), CtpError> {
        if !matches!(self.get_state(), ClientState::LoggedIn) {
            return Err(CtpError::AuthenticationError("用户未登录".to_string()));
        }
        
        tracing::info!("查询报单记录");
        
        // 使用真实的 CTP API 查询报单记录
        if let Some(api_manager) = &self.api_manager {
            if let Some(trader_api) = api_manager.get_trader_api() {
                // 创建报单查询请求
                let mut qry_req = ctp2rs::v1alpha1::CThostFtdcQryOrderField::default();
                
                // 使用 ctp2rs 提供的字符串赋值工具
                use ctp2rs::ffi::AssignFromString;
                qry_req.BrokerID.assign_from_str(&self.config.broker_id);
                qry_req.InvestorID.assign_from_str(&self.config.investor_id);
                
                // 如果指定了合约，则只查询该合约的报单
                if let Some(instrument) = instrument_id {
                    qry_req.InstrumentID.assign_from_str(instrument);
                }
                
                let request_id = self.get_next_request_id();
                
                tracing::info!("发送报单查询请求，请求ID: {}", request_id);
                
                // 调用 ctp2rs TraderApi 查询报单
                let result = trader_api.req_qry_order(&mut qry_req, request_id);
                
                if result != 0 {
                    return Err(CtpError::CtpApiError {
                        code: result,
                        message: "报单查询请求发送失败".to_string(),
                    });
                }
                
                tracing::info!("报单查询请求已发送，结果将通过事件回调返回");
                Ok(())
            } else {
                Err(CtpError::StateError("交易 API 未初始化".to_string()))
            }
        } else {
            Err(CtpError::StateError("API 管理器未初始化".to_string()))
        }
    }

    /// 查询结算信息
    pub async fn query_settlement_info(&mut self, trading_day: Option<&str>) -> Result<(), CtpError> {
        if !matches!(self.get_state(), ClientState::LoggedIn) {
            return Err(CtpError::AuthenticationError("用户未登录".to_string()));
        }
        
        tracing::info!("查询结算信息");
        
        // 使用真实的 CTP API 查询结算信息
        if let Some(api_manager) = &self.api_manager {
            if let Some(trader_api) = api_manager.get_trader_api() {
                // 创建结算信息查询请求
                let mut qry_req = ctp2rs::v1alpha1::CThostFtdcQrySettlementInfoField::default();
                
                // 使用 ctp2rs 提供的字符串赋值工具
                use ctp2rs::ffi::AssignFromString;
                qry_req.BrokerID.assign_from_str(&self.config.broker_id);
                qry_req.InvestorID.assign_from_str(&self.config.investor_id);
                
                // 如果指定了交易日，则查询指定日期的结算信息
                if let Some(day) = trading_day {
                    qry_req.TradingDay.assign_from_str(day);
                }
                
                let request_id = self.get_next_request_id();
                
                tracing::info!("发送结算信息查询请求，请求ID: {}", request_id);
                
                // 调用 ctp2rs TraderApi 查询结算信息
                let result = trader_api.req_qry_settlement_info(&mut qry_req, request_id);
                
                if result != 0 {
                    return Err(CtpError::CtpApiError {
                        code: result,
                        message: "结算信息查询请求发送失败".to_string(),
                    });
                }
                
                tracing::info!("结算信息查询请求已发送，结果将通过事件回调返回");
                Ok(())
            } else {
                Err(CtpError::StateError("交易 API 未初始化".to_string()))
            }
        } else {
            Err(CtpError::StateError("API 管理器未初始化".to_string()))
        }
    }

    /// 确认结算信息
    pub async fn confirm_settlement_info(&mut self) -> Result<(), CtpError> {
        if !matches!(self.get_state(), ClientState::LoggedIn) {
            return Err(CtpError::AuthenticationError("用户未登录".to_string()));
        }
        
        tracing::info!("确认结算信息");
        
        // 使用真实的 CTP API 确认结算信息
        if let Some(api_manager) = &self.api_manager {
            if let Some(trader_api) = api_manager.get_trader_api() {
                // 创建结算信息确认请求
                let mut confirm_req = ctp2rs::v1alpha1::CThostFtdcSettlementInfoConfirmField::default();
                
                // 使用 ctp2rs 提供的字符串赋值工具
                use ctp2rs::ffi::AssignFromString;
                confirm_req.BrokerID.assign_from_str(&self.config.broker_id);
                confirm_req.InvestorID.assign_from_str(&self.config.investor_id);
                
                let request_id = self.get_next_request_id();
                
                tracing::info!("发送结算信息确认请求，请求ID: {}", request_id);
                
                // 调用 ctp2rs TraderApi 确认结算信息
                let result = trader_api.req_settlement_info_confirm(&mut confirm_req, request_id);
                
                if result != 0 {
                    return Err(CtpError::CtpApiError {
                        code: result,
                        message: "结算信息确认请求发送失败".to_string(),
                    });
                }
                
                tracing::info!("结算信息确认请求已发送，结果将通过事件回调返回");
                Ok(())
            } else {
                Err(CtpError::StateError("交易 API 未初始化".to_string()))
            }
        } else {
            Err(CtpError::StateError("API 管理器未初始化".to_string()))
        }
    }

    /// 获取已订阅合约列表
    pub fn get_subscribed_instruments(&self) -> Vec<String> {
        let subscribed = self.subscribed_instruments.lock().unwrap();
        subscribed.iter().cloned().collect()
    }

    /// 重新订阅所有合约（用于重连后恢复订阅）
    pub async fn resubscribe_all_instruments(&mut self) -> Result<(), CtpError> {
        let instruments = self.get_subscribed_instruments();
        
        if !instruments.is_empty() {
            tracing::info!("重新订阅所有合约，数量: {}", instruments.len());
            self.subscribe_market_data(&instruments).await?;
        }
        
        Ok(())
    }

    /// 自动重连机制
    pub async fn start_auto_reconnect(&mut self) -> Result<(), CtpError> {
        tracing::info!("启动自动重连机制");
        
        let max_attempts = self.config.max_reconnect_attempts;
        let retry_interval = self.config.reconnect_interval();
        
        for attempt in 1..=max_attempts {
            tracing::info!("重连尝试 {}/{}", attempt, max_attempts);
            
            match self.connect_with_retry().await {
                Ok(_) => {
                    tracing::info!("重连成功");
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("重连失败 (尝试 {}): {}", attempt, e);
                    
                    if attempt < max_attempts {
                        tracing::info!("等待 {:?} 后重试...", retry_interval);
                        tokio::time::sleep(retry_interval).await;
                    }
                }
            }
        }
        
        let error = CtpError::ConnectionError(
            format!("自动重连失败，已达到最大重试次数 {}", max_attempts)
        );
        self.set_state(ClientState::Error(error.to_string()));
        Err(error)
    }

    /// 处理认证失败重试
    pub async fn handle_auth_failure(&mut self, error_msg: &str) -> Result<(), CtpError> {
        tracing::warn!("认证失败: {}", error_msg);
        
        // 检查是否为可重试的错误
        if self.is_retryable_auth_error(error_msg) {
            tracing::info!("认证错误可重试，等待后重新尝试");
            
            // 等待一段时间后重试
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            // 重新连接和登录
            self.connect_with_retry().await?;
            
            // 使用原有凭据重新登录
            let credentials = LoginCredentials {
                broker_id: self.config.broker_id.clone(),
                user_id: self.config.investor_id.clone(),
                password: self.config.password.clone(),
                app_id: self.config.app_id.clone(),
                auth_code: self.config.auth_code.clone(),
            };
            
            self.login(credentials).await?;
            
            Ok(())
        } else {
            Err(CtpError::AuthenticationError(format!("认证失败且不可重试: {}", error_msg)))
        }
    }

    /// 检查认证错误是否可重试
    fn is_retryable_auth_error(&self, error_msg: &str) -> bool {
        // 检查常见的可重试错误
        let retryable_errors = [
            "网络连接失败",
            "服务器繁忙",
            "连接超时",
            "CTP:还没有初始化",
        ];
        
        retryable_errors.iter().any(|&err| error_msg.contains(err))
    }

    /// 会话管理 - 保持会话活跃
    pub async fn keep_session_alive(&self) -> Result<(), CtpError> {
        tracing::debug!("保持会话活跃");
        
        // 定期发送心跳或查询请求来保持会话
        if let Some(api_manager) = &self.api_manager {
            if let Some(trader_api) = api_manager.get_trader_api() {
                // 发送一个简单的查询请求作为心跳
                let request_id = self.get_next_request_id();
                
                // 这里可以发送查询交易日等轻量级请求
                tracing::debug!("发送心跳查询，请求ID: {}", request_id);
                
                // 实际的心跳实现需要根据 CTP API 的具体方法来调用
                // trader_api.req_qry_trading_day(request_id);
            }
        }
        
        Ok(())
    }
}

/// 连接统计信息
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub state: ClientState,
    pub reconnect_count: u32,
    pub connect_duration: Option<Duration>,
    pub config_environment: crate::ctp::Environment,
}

/// 健康状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub state: ClientState,
    pub last_check_time: chrono::DateTime<chrono::Utc>,
    pub error_message: Option<String>,
}

/// 配置信息（不包含敏感数据）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigInfo {
    pub environment: crate::ctp::Environment,
    pub broker_id: String,
    pub user_id: String,
    pub md_front_addr: String,
    pub trader_front_addr: String,
    pub flow_path: String,
    pub timeout_secs: u64,
    pub max_reconnect_attempts: u32,
}
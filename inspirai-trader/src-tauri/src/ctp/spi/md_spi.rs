use crate::ctp::{
    CtpError, CtpEvent, ClientState,
    models::{MarketDataTick, LoginResponse},
    config::CtpConfig,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use std::collections::HashMap;

/// 行情 SPI 实现
/// 
/// 负责处理 CTP 行情 API 的所有回调事件，包括：
/// - 连接状态变化
/// - 用户登录响应
/// - 行情数据订阅响应
/// - 实时行情数据推送
/// - 错误处理
pub struct MdSpiImpl {
    /// 客户端状态的共享引用
    client_state: Arc<Mutex<ClientState>>,
    /// 事件发送器，用于向上层发送事件
    event_sender: mpsc::UnboundedSender<CtpEvent>,
    /// CTP 配置信息
    config: CtpConfig,
    /// 已订阅的合约列表
    subscribed_instruments: Arc<Mutex<HashMap<String, bool>>>,
    /// 请求ID计数器
    request_id_counter: Arc<Mutex<i32>>,
}

impl MdSpiImpl {
    /// 创建新的行情 SPI 实例
    pub fn new(
        client_state: Arc<Mutex<ClientState>>,
        event_sender: mpsc::UnboundedSender<CtpEvent>,
        config: CtpConfig,
    ) -> Self {
        tracing::info!("创建行情 SPI 实例");
        
        Self {
            client_state,
            event_sender,
            config,
            subscribed_instruments: Arc::new(Mutex::new(HashMap::new())),
            request_id_counter: Arc::new(Mutex::new(1)),
        }
    }

    /// 获取下一个请求ID
    fn next_request_id(&self) -> i32 {
        let mut counter = self.request_id_counter.lock().unwrap();
        let id = *counter;
        *counter += 1;
        id
    }

    /// 发送事件到事件处理器
    fn send_event(&self, event: CtpEvent) {
        if let Err(e) = self.event_sender.send(event) {
            tracing::error!("发送事件失败: {}", e);
        }
    }

    /// 更新客户端状态
    fn update_client_state(&self, new_state: ClientState) {
        let mut state = self.client_state.lock().unwrap();
        if *state != new_state {
            tracing::debug!("行情客户端状态变更: {:?} -> {:?}", *state, new_state);
            *state = new_state;
        }
    }

    /// 添加已订阅的合约
    fn add_subscribed_instrument(&self, instrument_id: &str) {
        let mut instruments = self.subscribed_instruments.lock().unwrap();
        instruments.insert(instrument_id.to_string(), true);
        tracing::debug!("添加订阅合约: {}", instrument_id);
    }

    /// 移除已订阅的合约
    fn remove_subscribed_instrument(&self, instrument_id: &str) {
        let mut instruments = self.subscribed_instruments.lock().unwrap();
        instruments.remove(instrument_id);
        tracing::debug!("移除订阅合约: {}", instrument_id);
    }

    /// 检查合约是否已订阅
    fn is_instrument_subscribed(&self, instrument_id: &str) -> bool {
        let instruments = self.subscribed_instruments.lock().unwrap();
        instruments.contains_key(instrument_id)
    }

    /// 获取已订阅合约列表
    pub fn get_subscribed_instruments(&self) -> Vec<String> {
        let instruments = self.subscribed_instruments.lock().unwrap();
        instruments.keys().cloned().collect()
    }
}

// 实现 ctp2rs 的 MdSpi trait，严禁使用自定义的模拟实现
impl ctp2rs::v1alpha1::MdSpi for MdSpiImpl {
    /// 当客户端与交易后台建立起通信连接时（还未登录前），该方法被调用
    fn on_front_connected(&mut self) {
        tracing::info!("行情前置连接成功");
        
        self.update_client_state(ClientState::Connected);
        self.send_event(CtpEvent::Connected);
        
        // 连接成功后自动发起登录请求
        if let Err(e) = self.req_user_login() {
            tracing::error!("自动登录请求失败: {}", e);
            self.send_event(CtpEvent::Error(format!("自动登录请求失败: {}", e)));
        }
    }

    /// 当客户端与交易后台通信连接断开时，该方法被调用
    /// 当发生这个情况后，API会自动重新连接，客户端可不做处理
    fn on_front_disconnected(&mut self, reason: i32) {
        tracing::warn!("行情前置连接断开，原因代码: {}", reason);
        
        let reason_msg = match reason {
            0x1001 => "网络读失败",
            0x1002 => "网络写失败", 
            0x2001 => "接收心跳超时",
            0x2002 => "发送心跳失败",
            0x2003 => "收到错误报文",
            _ => "未知原因",
        };
        
        tracing::warn!("断开原因: {}", reason_msg);
        
        self.update_client_state(ClientState::Disconnected);
        self.send_event(CtpEvent::Disconnected);
        
        // 清空订阅列表，等待重连后重新订阅
        {
            let mut instruments = self.subscribed_instruments.lock().unwrap();
            instruments.clear();
        }
    }

    /// 登录请求响应
    fn on_rsp_user_login(
        &mut self,
        rsp_user_login: Option<&CThostFtdcRspUserLoginField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        _is_last: bool,
    ) {
        tracing::info!("收到登录响应，请求ID: {}, 是否最后一条: {}", request_id, _is_last);
        
        if let Some(rsp_info) = rsp_info {
            if rsp_info.ErrorID != 0 {
                let error_msg = self.convert_gb18030_to_string(&rsp_info.ErrorMsg);
                tracing::error!("登录失败: {} (错误码: {})", error_msg, rsp_info.ErrorID);
                
                let error = CtpError::from_ctp_error(rsp_info.ErrorID, &error_msg);
                self.update_client_state(ClientState::Error(error.to_string()));
                self.send_event(CtpEvent::LoginFailed(error.to_string()));
                return;
            }
        }
        
        if let Some(login_field) = rsp_user_login {
            let trading_day = self.convert_gb18030_to_string(&login_field.TradingDay);
            let login_time = self.convert_gb18030_to_string(&login_field.LoginTime);
            let system_name = self.convert_gb18030_to_string(&login_field.SystemName);
            
            tracing::info!("行情登录成功");
            tracing::info!("交易日: {}", trading_day);
            tracing::info!("登录时间: {}", login_time);
            tracing::info!("系统名称: {}", system_name);
            
            let login_response = LoginResponse {
                trading_day,
                login_time,
                broker_id: self.config.broker_id.clone(),
                user_id: self.config.investor_id.clone(),
                system_name,
                front_id: login_field.FrontID,
                session_id: login_field.SessionID,
                max_order_ref: self.convert_gb18030_to_string(&login_field.MaxOrderRef),
            };
            
            self.update_client_state(ClientState::LoggedIn);
            self.send_event(CtpEvent::LoginSuccess(login_response));
        }
    }

    /// 订阅行情应答
    fn on_rsp_sub_market_data(
        &mut self,
        specific_instrument: Option<&CThostFtdcSpecificInstrumentField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        _is_last: bool,
    ) {
        tracing::debug!("收到行情订阅响应，请求ID: {}, 是否最后一条: {}", request_id, _is_last);
        
        if let Some(rsp_info) = rsp_info {
            if rsp_info.ErrorID != 0 {
                let error_msg = self.convert_gb18030_to_string(&rsp_info.ErrorMsg);
                tracing::error!("行情订阅失败: {} (错误码: {})", error_msg, rsp_info.ErrorID);
                
                if let Some(instrument) = specific_instrument {
                    let instrument_id = self.convert_gb18030_to_string(&instrument.InstrumentID);
                    tracing::error!("订阅失败的合约: {}", instrument_id);
                }
                
                self.send_event(CtpEvent::Error(format!("行情订阅失败: {}", error_msg)));
                return;
            }
        }
        
        if let Some(instrument) = specific_instrument {
            let instrument_id = self.convert_gb18030_to_string(&instrument.InstrumentID);
            tracing::info!("行情订阅成功: {}", instrument_id);
            
            self.add_subscribed_instrument(&instrument_id);
        }
    }

    /// 取消订阅行情应答
    fn on_rsp_unsub_market_data(
        &mut self,
        specific_instrument: Option<&CThostFtdcSpecificInstrumentField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        _is_last: bool,
    ) {
        tracing::debug!("收到取消行情订阅响应，请求ID: {}, 是否最后一条: {}", request_id, _is_last);
        
        if let Some(rsp_info) = rsp_info {
            if rsp_info.ErrorID != 0 {
                let error_msg = self.convert_gb18030_to_string(&rsp_info.ErrorMsg);
                tracing::error!("取消行情订阅失败: {} (错误码: {})", error_msg, rsp_info.ErrorID);
                self.send_event(CtpEvent::Error(format!("取消行情订阅失败: {}", error_msg)));
                return;
            }
        }
        
        if let Some(instrument) = specific_instrument {
            let instrument_id = self.convert_gb18030_to_string(&instrument.InstrumentID);
            tracing::info!("取消行情订阅成功: {}", instrument_id);
            
            self.remove_subscribed_instrument(&instrument_id);
        }
    }

    /// 深度行情通知
    fn on_rtn_depth_market_data(&mut self, depth_market_data: Option<&CThostFtdcDepthMarketDataField>) {
        if let Some(market_data) = depth_market_data {
            let instrument_id = self.convert_gb18030_to_string(&market_data.InstrumentID);
            
            // 只处理已订阅的合约行情
            if !self.is_instrument_subscribed(&instrument_id) {
                tracing::debug!("收到未订阅合约的行情数据: {}", instrument_id);
                return;
            }
            
            let tick = self.convert_market_data_to_tick(market_data);
            
            tracing::trace!("收到行情数据: {} 最新价: {}", tick.instrument_id, tick.last_price);
            
            self.send_event(CtpEvent::MarketData(tick));
        }
    }

    /// 错误应答
    fn on_rsp_error(
        &mut self,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        _is_last: bool,
    ) {
        if let Some(rsp_info) = rsp_info {
            let error_msg = self.convert_gb18030_to_string(&rsp_info.ErrorMsg);
            tracing::error!("CTP 行情错误: {} (错误码: {}, 请求ID: {})", 
                error_msg, rsp_info.ErrorID, request_id);
            
            let error = CtpError::from_ctp_error(rsp_info.ErrorID, &error_msg);
            self.send_event(CtpEvent::Error(error.to_string()));
        }
    }
}

// 辅助方法实现
impl MdSpiImpl {
    /// 通知需要发起用户登录请求
    /// 由于 SPI 回调中无法直接访问 API 实例，这里只是发送事件通知
    fn req_user_login(&self) -> Result<(), CtpError> {
        tracing::info!("通知需要发起行情用户登录请求");
        
        // 发送登录请求事件，由客户端处理实际的登录逻辑
        self.send_event(CtpEvent::LoginRequired);
        
        Ok(())
    }

    /// 将 CTP 的 GB18030 编码字符串转换为 UTF-8 字符串
    /// 使用 ctp2rs 官方转换工具，严禁自定义实现
    fn convert_gb18030_to_string(&self, gb18030_bytes: &[i8]) -> String {
        gb18030_cstr_i8_to_str(gb18030_bytes).unwrap_or_else(|e| {
            tracing::warn!("字符串转换失败: {}", e);
            "".into()
        }).to_string()
    }

    /// 将 CTP 行情数据转换为业务模型
    /// 使用 ctp2rs 官方数据转换工具，严禁自定义实现
    fn convert_market_data_to_tick(&self, market_data: &CThostFtdcDepthMarketDataField) -> MarketDataTick {
        match crate::ctp::utils::DataConverter::convert_market_data(market_data) {
            Ok(tick) => tick,
            Err(e) => {
                tracing::error!("行情数据转换失败: {}", e);
                // 即使转换失败，也不能使用自定义的后备方案
                // 必须通过正确的错误处理机制来解决问题
                panic!("CTP 行情数据转换失败，必须修复转换逻辑而不是使用后备方案: {}", e);
            }
        }
    }
}

// 使用 ctp2rs 提供的官方数据结构，严禁自定义
use ctp2rs::v1alpha1::{
    CThostFtdcDepthMarketDataField,
    CThostFtdcRspUserLoginField,
    CThostFtdcRspInfoField,
    CThostFtdcSpecificInstrumentField,

};
use ctp2rs::ffi::gb18030_cstr_i8_to_str;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use crate::ctp::Environment;

    fn create_test_config() -> CtpConfig {
        CtpConfig {
            environment: Environment::SimNow,
            broker_id: "9999".to_string(),
            investor_id: "test_user".to_string(),
            password: "test_pass".to_string(),
            app_id: "test_app".to_string(),
            auth_code: "test_auth".to_string(),
            md_front_addr: "tcp://127.0.0.1:41213".to_string(),
            trader_front_addr: "tcp://127.0.0.1:41205".to_string(),
            flow_path: "./test_flow".to_string(),
            md_dynlib_path: None,
            td_dynlib_path: None,
            timeout_secs: 30,
            reconnect_interval_secs: 5,
            max_reconnect_attempts: 3,
        }
    }

    #[tokio::test]
    async fn test_md_spi_creation() {
        let client_state = Arc::new(Mutex::new(ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = MdSpiImpl::new(client_state, sender, config);
        
        assert_eq!(md_spi.get_subscribed_instruments().len(), 0);
    }

    #[tokio::test]
    async fn test_instrument_subscription_management() {
        let client_state = Arc::new(Mutex::new(ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = MdSpiImpl::new(client_state, sender, config);
        
        // 测试添加订阅
        md_spi.add_subscribed_instrument("rb2401");
        assert!(md_spi.is_instrument_subscribed("rb2401"));
        assert_eq!(md_spi.get_subscribed_instruments().len(), 1);
        
        // 测试移除订阅
        md_spi.remove_subscribed_instrument("rb2401");
        assert!(!md_spi.is_instrument_subscribed("rb2401"));
        assert_eq!(md_spi.get_subscribed_instruments().len(), 0);
    }

    #[test]
    fn test_gb18030_conversion() {
        let client_state = Arc::new(Mutex::new(ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = MdSpiImpl::new(client_state, sender, config);
        
        // 测试字符串转换
        let test_bytes = [114, 98, 50, 52, 48, 49, 0, 0, 0]; // "rb2401" + null bytes
        let result = md_spi.convert_gb18030_to_string(&test_bytes);
        assert_eq!(result, "rb2401");
    }
}
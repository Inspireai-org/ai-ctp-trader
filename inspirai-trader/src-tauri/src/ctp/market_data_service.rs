use crate::ctp::{
    CtpError, CtpEvent, MdSpiImpl, MarketDataManager, SubscriptionManager,
    ClientState, MarketDataTick, SubscriptionInfo,
    SubscriptionPriority, MarketDataFilter, MarketDataStats, SubscriptionStats,
    config::CtpConfig,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use std::collections::HashMap;

/// 行情数据服务
/// 
/// 整合了 MdSpi、MarketDataManager 和 SubscriptionManager 的功能
/// 提供统一的行情数据订阅和管理接口
pub struct MarketDataService {
    /// 行情 SPI 实例
    md_spi: Arc<Mutex<MdSpiImpl>>,
    /// 行情数据管理器
    market_data_manager: MarketDataManager,
    /// 订阅管理器
    subscription_manager: SubscriptionManager,
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

/// 行情数据服务统计信息
#[derive(Debug, Clone)]
pub struct ServiceStats {
    /// 服务状态
    pub service_state: ServiceState,
    /// 启动时间
    pub start_time: Option<Instant>,
    /// 运行时长
    pub uptime: Option<Duration>,
    /// 行情数据统计
    pub market_data_stats: MarketDataStats,
    /// 订阅统计
    pub subscription_stats: SubscriptionStats,
    /// 当前订阅数量
    pub active_subscriptions: usize,
    /// 最后活动时间
    pub last_activity_time: Option<Instant>,
}

impl MarketDataService {
    /// 创建新的行情数据服务
    pub fn new(config: CtpConfig) -> Result<Self, CtpError> {
        // 创建事件通道
        let (event_sender, _event_receiver) = mpsc::unbounded_channel();
        
        // 创建客户端状态
        let client_state = Arc::new(Mutex::new(ClientState::Disconnected));
        
        // 创建 MdSpi 实例
        let md_spi = Arc::new(Mutex::new(MdSpiImpl::new(
            client_state.clone(),
            event_sender.clone(),
            config.clone(),
        )));
        
        // 创建行情数据管理器
        let market_data_manager = MarketDataManager::new(
            md_spi.clone(),
            event_sender.clone(),
        );
        
        // 创建订阅管理器
        let subscription_manager = SubscriptionManager::new(
            md_spi.clone(),
            event_sender.clone(),
        );
        
        Ok(Self {
            md_spi,
            market_data_manager,
            subscription_manager,
            event_sender,
            client_state,
            config,
            service_state: Arc::new(Mutex::new(ServiceState::Uninitialized)),
        })
    }

    /// 初始化服务
    pub async fn initialize(&mut self) -> Result<(), CtpError> {
        self.set_service_state(ServiceState::Initializing);
        
        tracing::info!("初始化行情数据服务");
        
        // 这里可以添加初始化逻辑，比如：
        // - 验证配置
        // - 检查网络连接
        // - 预加载合约信息等
        
        self.set_service_state(ServiceState::Initialized);
        tracing::info!("行情数据服务初始化完成");
        
        Ok(())
    }

    /// 启动服务
    pub async fn start(&mut self) -> Result<(), CtpError> {
        if !matches!(self.get_service_state(), ServiceState::Initialized) {
            return Err(CtpError::StateError("服务未初始化".to_string()));
        }
        
        self.set_service_state(ServiceState::Running);
        tracing::info!("启动行情数据服务");
        
        // 启动后台任务
        self.start_background_tasks().await?;
        
        Ok(())
    }

    /// 停止服务
    pub async fn stop(&mut self) -> Result<(), CtpError> {
        tracing::info!("停止行情数据服务");
        
        // 取消所有订阅
        let subscribed = self.subscription_manager.get_subscribed_instruments();
        if !subscribed.is_empty() {
            tracing::info!("取消所有订阅，数量: {}", subscribed.len());
            self.subscription_manager.unsubscribe(subscribed).await?;
        }
        
        // 清理资源
        self.market_data_manager.clear_cache();
        self.subscription_manager.reset_stats();
        
        self.set_service_state(ServiceState::Stopped);
        tracing::info!("行情数据服务已停止");
        
        Ok(())
    }

    /// 订阅行情数据
    pub async fn subscribe_market_data(&self, instruments: Vec<String>) -> Result<u32, CtpError> {
        self.check_service_running()?;
        
        tracing::info!("订阅行情数据，合约数量: {}", instruments.len());
        
        // 过滤掉已经订阅的合约
        let mut new_instruments = Vec::new();
        for instrument in &instruments {
            if !self.subscription_manager.is_subscribed(instrument) {
                new_instruments.push(instrument.clone());
            } else {
                tracing::debug!("合约 {} 已订阅，跳过", instrument);
            }
        }
        
        if new_instruments.is_empty() {
            tracing::info!("所有合约都已订阅，无需重复订阅");
            return Ok(0);
        }
        
        // 使用订阅管理器处理订阅
        let request_id = self.subscription_manager.subscribe(new_instruments.clone()).await?;
        
        // 同时在行情数据管理器中记录订阅
        self.market_data_manager.subscribe_market_data(&new_instruments).await?;
        
        Ok(request_id)
    }

    /// 使用优先级订阅行情数据
    pub async fn subscribe_market_data_with_priority(
        &self,
        instruments: Vec<String>,
        priority: SubscriptionPriority,
    ) -> Result<u32, CtpError> {
        self.check_service_running()?;
        
        tracing::info!("订阅行情数据（优先级: {:?}），合约数量: {}", priority, instruments.len());
        
        let request_id = self.subscription_manager
            .subscribe_with_priority(instruments.clone(), priority).await?;
        
        self.market_data_manager.subscribe_market_data(&instruments).await?;
        
        Ok(request_id)
    }

    /// 取消订阅行情数据
    pub async fn unsubscribe_market_data(&self, instruments: Vec<String>) -> Result<u32, CtpError> {
        self.check_service_running()?;
        
        tracing::info!("取消订阅行情数据，合约数量: {}", instruments.len());
        
        let request_id = self.subscription_manager.unsubscribe(instruments.clone()).await?;
        self.market_data_manager.unsubscribe_market_data(&instruments).await?;
        
        Ok(request_id)
    }

    /// 获取订阅信息
    pub fn get_subscription_info(&self, instrument_id: &str) -> Option<SubscriptionInfo> {
        self.subscription_manager.get_subscription_info(instrument_id)
    }

    /// 获取所有订阅信息
    pub fn get_all_subscriptions(&self) -> HashMap<String, SubscriptionInfo> {
        self.subscription_manager.get_all_subscriptions()
    }

    /// 获取已订阅的合约列表
    pub fn get_subscribed_instruments(&self) -> Vec<String> {
        self.subscription_manager.get_subscribed_instruments()
    }

    /// 获取缓存的行情数据
    pub fn get_cached_market_data(&self, instrument_id: &str) -> Option<MarketDataTick> {
        self.market_data_manager.get_cached_market_data(instrument_id)
    }

    /// 获取所有缓存的行情数据
    pub fn get_all_cached_market_data(&self) -> HashMap<String, MarketDataTick> {
        self.market_data_manager.get_all_cached_market_data()
    }

    /// 添加数据过滤器
    pub fn add_filter(&self, filter: Box<dyn MarketDataFilter + Send + Sync>) {
        self.market_data_manager.add_filter(filter);
    }

    /// 清除所有过滤器
    pub fn clear_filters(&self) {
        self.market_data_manager.clear_filters();
    }

    /// 获取服务统计信息
    pub fn get_service_stats(&self) -> ServiceStats {
        let service_state = self.get_service_state();
        let market_data_stats = self.market_data_manager.get_stats();
        let subscription_stats = self.subscription_manager.get_stats();
        let active_subscriptions = self.get_subscribed_instruments().len();
        
        ServiceStats {
            service_state,
            start_time: None, // TODO: 记录启动时间
            uptime: None,     // TODO: 计算运行时长
            market_data_stats: market_data_stats.clone(),
            subscription_stats,
            active_subscriptions,
            last_activity_time: market_data_stats.last_update_time.clone(),
        }
    }

    /// 处理行情数据
    pub fn handle_market_data(&self, tick: MarketDataTick) {
        // 通过订阅管理器处理
        self.subscription_manager.handle_market_data(tick.clone());
        
        // 通过行情数据管理器处理
        self.market_data_manager.handle_market_data(tick);
    }

    /// 处理订阅成功
    pub fn handle_subscription_success(&self, instrument_id: &str) {
        self.subscription_manager.handle_subscription_success(instrument_id);
    }

    /// 处理订阅失败
    pub fn handle_subscription_failure(&self, instrument_id: &str, error_msg: &str) {
        self.subscription_manager.handle_subscription_failure(instrument_id, error_msg);
    }

    /// 处理取消订阅成功
    pub fn handle_unsubscription_success(&self, instrument_id: &str) {
        self.subscription_manager.handle_unsubscription_success(instrument_id);
    }

    /// 获取事件发送器
    pub fn event_sender(&self) -> mpsc::UnboundedSender<CtpEvent> {
        self.event_sender.clone()
    }

    /// 获取客户端状态
    pub fn get_client_state(&self) -> ClientState {
        let state = self.client_state.lock().unwrap();
        state.clone()
    }

    /// 健康检查
    pub fn health_check(&self) -> Result<(), CtpError> {
        let service_state = self.get_service_state();
        
        match service_state {
            ServiceState::Running => Ok(()),
            ServiceState::Error(msg) => Err(CtpError::StateError(msg)),
            ServiceState::Stopped => Err(CtpError::StateError("服务已停止".to_string())),
            _ => Err(CtpError::StateError(format!("服务状态异常: {:?}", service_state))),
        }
    }

    // 私有方法

    /// 设置服务状态
    fn set_service_state(&self, state: ServiceState) {
        let mut service_state = self.service_state.lock().unwrap();
        if *service_state != state {
            tracing::debug!("服务状态变更: {:?} -> {:?}", *service_state, state);
            *service_state = state;
        }
    }

    /// 获取服务状态
    fn get_service_state(&self) -> ServiceState {
        let service_state = self.service_state.lock().unwrap();
        service_state.clone()
    }

    /// 检查服务是否正在运行
    fn check_service_running(&self) -> Result<(), CtpError> {
        let state = self.get_service_state();
        if !matches!(state, ServiceState::Running) {
            return Err(CtpError::StateError(
                format!("服务未运行，当前状态: {:?}", state)
            ));
        }
        Ok(())
    }

    /// 启动后台任务
    async fn start_background_tasks(&self) -> Result<(), CtpError> {
        // 启动定期清理任务
        self.start_cleanup_task().await;
        
        // 启动健康检查任务
        self.start_health_check_task().await;
        
        Ok(())
    }

    /// 启动清理任务
    async fn start_cleanup_task(&self) {
        // 暂时不启动后台任务，避免 Clone 问题
        // TODO: 在后续版本中实现更好的后台任务管理
        tracing::info!("清理任务已配置（暂未启动）");
    }

    /// 启动健康检查任务
    async fn start_health_check_task(&self) {
        // 暂时不启动后台任务，避免 Clone 问题
        // TODO: 在后续版本中实现更好的后台任务管理
        tracing::info!("健康检查任务已配置（暂未启动）");
    }
}

// MarketDataService 不实现 Clone，因为它包含不可克隆的资源

#[cfg(test)]
mod tests {
    use super::*;
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
    async fn test_market_data_service_lifecycle() {
        let config = create_test_config();
        let mut service = MarketDataService::new(config).unwrap();
        
        // 测试初始化
        assert!(matches!(service.get_service_state(), ServiceState::Uninitialized));
        
        service.initialize().await.unwrap();
        assert!(matches!(service.get_service_state(), ServiceState::Initialized));
        
        // 测试启动
        service.start().await.unwrap();
        assert!(matches!(service.get_service_state(), ServiceState::Running));
        
        // 测试健康检查
        service.health_check().unwrap();
        
        // 测试停止
        service.stop().await.unwrap();
        assert!(matches!(service.get_service_state(), ServiceState::Stopped));
    }

    #[tokio::test]
    async fn test_subscription_workflow() {
        let config = create_test_config();
        let mut service = MarketDataService::new(config).unwrap();
        
        service.initialize().await.unwrap();
        service.start().await.unwrap();
        
        // 测试订阅
        let instruments = vec!["rb2401".to_string(), "hc2401".to_string()];
        let request_id = service.subscribe_market_data(instruments.clone()).await.unwrap();
        assert!(request_id > 0);
        
        // 模拟订阅成功
        for instrument in &instruments {
            service.handle_subscription_success(instrument);
        }
        
        let subscribed = service.get_subscribed_instruments();
        assert_eq!(subscribed.len(), 2);
        
        // 测试取消订阅
        let unsubscribe_result = service.unsubscribe_market_data(vec!["rb2401".to_string()]).await;
        assert!(unsubscribe_result.is_ok());
        
        service.handle_unsubscription_success("rb2401");
        
        let remaining = service.get_subscribed_instruments();
        assert_eq!(remaining.len(), 1);
        
        service.stop().await.unwrap();
    }
}
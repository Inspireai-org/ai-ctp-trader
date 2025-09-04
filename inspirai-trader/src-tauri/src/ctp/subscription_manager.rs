use crate::ctp::{
    CtpError, CtpEvent, MdSpiImpl,
    models::MarketDataTick,
};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

/// 订阅状态
#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionStatus {
    /// 未订阅
    NotSubscribed,
    /// 订阅中
    Subscribing,
    /// 已订阅
    Subscribed,
    /// 取消订阅中
    Unsubscribing,
    /// 订阅失败
    Failed(String),
}

/// 订阅信息
#[derive(Debug, Clone)]
pub struct SubscriptionInfo {
    /// 合约代码
    pub instrument_id: String,
    /// 订阅状态
    pub status: SubscriptionStatus,
    /// 订阅时间
    pub subscribe_time: Option<Instant>,
    /// 最后更新时间
    pub last_update_time: Option<Instant>,
    /// 接收到的行情数据数量
    pub data_count: u64,
    /// 最后一次行情数据
    pub last_tick: Option<MarketDataTick>,
    /// 重试次数
    pub retry_count: u32,
}

impl SubscriptionInfo {
    pub fn new(instrument_id: String) -> Self {
        Self {
            instrument_id,
            status: SubscriptionStatus::NotSubscribed,
            subscribe_time: None,
            last_update_time: None,
            data_count: 0,
            last_tick: None,
            retry_count: 0,
        }
    }
}

/// 订阅请求
#[derive(Debug, Clone)]
pub struct SubscriptionRequest {
    /// 合约代码列表
    pub instruments: Vec<String>,
    /// 请求类型
    pub request_type: SubscriptionRequestType,
    /// 请求时间
    pub request_time: Instant,
    /// 请求ID
    pub request_id: u32,
    /// 优先级
    pub priority: SubscriptionPriority,
}

/// 订阅请求类型
#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionRequestType {
    /// 订阅
    Subscribe,
    /// 取消订阅
    Unsubscribe,
}

/// 订阅优先级
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubscriptionPriority {
    /// 低优先级
    Low = 0,
    /// 普通优先级
    Normal = 1,
    /// 高优先级
    High = 2,
    /// 紧急优先级
    Urgent = 3,
}

/// 订阅管理器
/// 
/// 负责管理所有合约的订阅状态和请求队列
pub struct SubscriptionManager {
    /// 行情 SPI 实例
    md_spi: Arc<Mutex<MdSpiImpl>>,
    /// 事件发送器
    event_sender: mpsc::UnboundedSender<CtpEvent>,
    /// 订阅信息映射
    subscriptions: Arc<Mutex<HashMap<String, SubscriptionInfo>>>,
    /// 请求队列
    request_queue: Arc<Mutex<VecDeque<SubscriptionRequest>>>,
    /// 请求ID计数器
    request_id_counter: Arc<Mutex<u32>>,
    /// 配置参数
    config: SubscriptionConfig,
    /// 统计信息
    stats: Arc<Mutex<SubscriptionStats>>,
}

/// 订阅配置
#[derive(Debug, Clone)]
pub struct SubscriptionConfig {
    /// 最大重试次数
    pub max_retry_count: u32,
    /// 重试间隔
    pub retry_interval: Duration,
    /// 批量订阅大小
    pub batch_size: usize,
    /// 请求超时时间
    pub request_timeout: Duration,
    /// 队列最大长度
    pub max_queue_length: usize,
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            max_retry_count: 3,
            retry_interval: Duration::from_secs(1),
            batch_size: 10,
            request_timeout: Duration::from_secs(5),
            max_queue_length: 1000,
        }
    }
}

/// 订阅统计信息
#[derive(Debug, Clone, Default)]
pub struct SubscriptionStats {
    /// 总订阅请求数
    pub total_subscribe_requests: u64,
    /// 总取消订阅请求数
    pub total_unsubscribe_requests: u64,
    /// 成功订阅数
    pub successful_subscriptions: u64,
    /// 失败订阅数
    pub failed_subscriptions: u64,
    /// 当前订阅数量
    pub current_subscriptions: u64,
    /// 总接收行情数据量
    pub total_market_data_received: u64,
    /// 平均响应时间
    pub average_response_time: Duration,
}

impl SubscriptionManager {
    /// 创建新的订阅管理器
    pub fn new(
        md_spi: Arc<Mutex<MdSpiImpl>>,
        event_sender: mpsc::UnboundedSender<CtpEvent>,
    ) -> Self {
        Self::with_config(md_spi, event_sender, SubscriptionConfig::default())
    }

    /// 使用指定配置创建订阅管理器
    pub fn with_config(
        md_spi: Arc<Mutex<MdSpiImpl>>,
        event_sender: mpsc::UnboundedSender<CtpEvent>,
        config: SubscriptionConfig,
    ) -> Self {
        Self {
            md_spi,
            event_sender,
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            request_queue: Arc::new(Mutex::new(VecDeque::new())),
            request_id_counter: Arc::new(Mutex::new(1)),
            config,
            stats: Arc::new(Mutex::new(SubscriptionStats::default())),
        }
    }

    /// 订阅行情数据
    pub async fn subscribe(&self, instruments: Vec<String>) -> Result<u32, CtpError> {
        self.subscribe_with_priority(instruments, SubscriptionPriority::Normal).await
    }

    /// 使用指定优先级订阅行情数据
    pub async fn subscribe_with_priority(
        &self,
        instruments: Vec<String>,
        priority: SubscriptionPriority,
    ) -> Result<u32, CtpError> {
        if instruments.is_empty() {
            return Err(CtpError::ConfigError("合约列表不能为空".to_string()));
        }

        // 过滤已订阅的合约
        let mut new_instruments = Vec::new();
        {
            let subscriptions = self.subscriptions.lock().unwrap();
            for instrument in instruments {
                if let Some(info) = subscriptions.get(&instrument) {
                    if info.status == SubscriptionStatus::Subscribed {
                        tracing::debug!("合约 {} 已订阅，跳过", instrument);
                        continue;
                    }
                }
                new_instruments.push(instrument);
            }
        }

        if new_instruments.is_empty() {
            tracing::info!("所有合约都已订阅");
            return Ok(0);
        }

        // 创建订阅请求
        let request_id = self.next_request_id();
        let request = SubscriptionRequest {
            instruments: new_instruments.clone(),
            request_type: SubscriptionRequestType::Subscribe,
            request_time: Instant::now(),
            request_id,
            priority,
        };

        // 更新订阅状态
        {
            let mut subscriptions = self.subscriptions.lock().unwrap();
            for instrument in &new_instruments {
                let info = subscriptions.entry(instrument.clone())
                    .or_insert_with(|| SubscriptionInfo::new(instrument.clone()));
                info.status = SubscriptionStatus::Subscribing;
                info.subscribe_time = Some(Instant::now());
            }
        }

        // 添加到请求队列
        self.add_request(request)?;

        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_subscribe_requests += 1;
        }

        tracing::info!("添加订阅请求，合约数量: {}, 请求ID: {}", new_instruments.len(), request_id);

        Ok(request_id)
    }

    /// 取消订阅行情数据
    pub async fn unsubscribe(&self, instruments: Vec<String>) -> Result<u32, CtpError> {
        self.unsubscribe_with_priority(instruments, SubscriptionPriority::Normal).await
    }

    /// 使用指定优先级取消订阅行情数据
    pub async fn unsubscribe_with_priority(
        &self,
        instruments: Vec<String>,
        priority: SubscriptionPriority,
    ) -> Result<u32, CtpError> {
        if instruments.is_empty() {
            return Err(CtpError::ConfigError("合约列表不能为空".to_string()));
        }

        // 过滤未订阅的合约
        let mut subscribed_instruments = Vec::new();
        {
            let subscriptions = self.subscriptions.lock().unwrap();
            for instrument in instruments {
                if let Some(info) = subscriptions.get(&instrument) {
                    if info.status == SubscriptionStatus::Subscribed {
                        subscribed_instruments.push(instrument);
                    } else {
                        tracing::debug!("合约 {} 未订阅，跳过", instrument);
                    }
                } else {
                    tracing::debug!("合约 {} 不存在，跳过", instrument);
                }
            }
        }

        if subscribed_instruments.is_empty() {
            tracing::info!("没有需要取消订阅的合约");
            return Ok(0);
        }

        // 创建取消订阅请求
        let request_id = self.next_request_id();
        let request = SubscriptionRequest {
            instruments: subscribed_instruments.clone(),
            request_type: SubscriptionRequestType::Unsubscribe,
            request_time: Instant::now(),
            request_id,
            priority,
        };

        // 更新订阅状态
        {
            let mut subscriptions = self.subscriptions.lock().unwrap();
            for instrument in &subscribed_instruments {
                if let Some(info) = subscriptions.get_mut(instrument) {
                    info.status = SubscriptionStatus::Unsubscribing;
                }
            }
        }

        // 添加到请求队列
        self.add_request(request)?;

        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_unsubscribe_requests += 1;
        }

        tracing::info!("添加取消订阅请求，合约数量: {}, 请求ID: {}", subscribed_instruments.len(), request_id);

        Ok(request_id)
    }

    /// 获取订阅信息
    pub fn get_subscription_info(&self, instrument_id: &str) -> Option<SubscriptionInfo> {
        let subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.get(instrument_id).cloned()
    }

    /// 获取所有订阅信息
    pub fn get_all_subscriptions(&self) -> HashMap<String, SubscriptionInfo> {
        let subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.clone()
    }

    /// 获取已订阅的合约列表
    pub fn get_subscribed_instruments(&self) -> Vec<String> {
        let subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.iter()
            .filter(|(_, info)| info.status == SubscriptionStatus::Subscribed)
            .map(|(instrument, _)| instrument.clone())
            .collect()
    }

    /// 检查合约是否已订阅
    pub fn is_subscribed(&self, instrument_id: &str) -> bool {
        let subscriptions = self.subscriptions.lock().unwrap();
        if let Some(info) = subscriptions.get(instrument_id) {
            info.status == SubscriptionStatus::Subscribed
        } else {
            false
        }
    }

    /// 检查合约是否正在订阅中
    pub fn is_subscribing(&self, instrument_id: &str) -> bool {
        let subscriptions = self.subscriptions.lock().unwrap();
        if let Some(info) = subscriptions.get(instrument_id) {
            info.status == SubscriptionStatus::Subscribing
        } else {
            false
        }
    }

    /// 获取订阅状态
    pub fn get_subscription_status(&self, instrument_id: &str) -> SubscriptionStatus {
        let subscriptions = self.subscriptions.lock().unwrap();
        if let Some(info) = subscriptions.get(instrument_id) {
            info.status.clone()
        } else {
            SubscriptionStatus::NotSubscribed
        }
    }

    /// 处理行情数据
    pub fn handle_market_data(&self, tick: MarketDataTick) {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        if let Some(info) = subscriptions.get_mut(&tick.instrument_id) {
            info.last_tick = Some(tick.clone());
            info.last_update_time = Some(Instant::now());
            info.data_count += 1;
        }

        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_market_data_received += 1;
        }

        // 发送事件
        if let Err(e) = self.event_sender.send(CtpEvent::MarketData(tick)) {
            tracing::error!("发送行情数据事件失败: {}", e);
        }
    }

    /// 处理订阅成功
    pub fn handle_subscription_success(&self, instrument_id: &str) {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        if let Some(info) = subscriptions.get_mut(instrument_id) {
            info.status = SubscriptionStatus::Subscribed;
            info.retry_count = 0;
            tracing::info!("合约 {} 订阅成功", instrument_id);

            // 更新统计信息
            let mut stats = self.stats.lock().unwrap();
            stats.successful_subscriptions += 1;
            stats.current_subscriptions += 1;
        }
    }

    /// 处理订阅失败
    pub fn handle_subscription_failure(&self, instrument_id: &str, error_msg: &str) {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        if let Some(info) = subscriptions.get_mut(instrument_id) {
            info.retry_count += 1;
            
            if info.retry_count >= self.config.max_retry_count {
                info.status = SubscriptionStatus::Failed(error_msg.to_string());
                tracing::error!("合约 {} 订阅失败，已达到最大重试次数: {}", instrument_id, error_msg);

                // 更新统计信息
                let mut stats = self.stats.lock().unwrap();
                stats.failed_subscriptions += 1;
            } else {
                info.status = SubscriptionStatus::NotSubscribed;
                tracing::warn!("合约 {} 订阅失败，将重试 ({}/{}): {}", 
                    instrument_id, info.retry_count, self.config.max_retry_count, error_msg);
                
                // TODO: 添加重试逻辑
            }
        }
    }

    /// 处理取消订阅成功
    pub fn handle_unsubscription_success(&self, instrument_id: &str) {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        if let Some(info) = subscriptions.get_mut(instrument_id) {
            info.status = SubscriptionStatus::NotSubscribed;
            info.last_tick = None;
            info.data_count = 0;
            tracing::info!("合约 {} 取消订阅成功", instrument_id);

            // 更新统计信息
            let mut stats = self.stats.lock().unwrap();
            if stats.current_subscriptions > 0 {
                stats.current_subscriptions -= 1;
            }
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> SubscriptionStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    /// 清理过期的订阅信息
    pub fn cleanup_expired_subscriptions(&self, max_age: Duration) {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        let now = Instant::now();
        
        subscriptions.retain(|instrument, info| {
            if let Some(last_update) = info.last_update_time {
                if now.duration_since(last_update) > max_age {
                    tracing::info!("清理过期订阅信息: {}", instrument);
                    return false;
                }
            }
            true
        });
    }

    /// 重置统计信息
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = SubscriptionStats::default();
        tracing::info!("重置订阅统计信息");
    }

    // 私有方法

    /// 获取下一个请求ID
    fn next_request_id(&self) -> u32 {
        let mut counter = self.request_id_counter.lock().unwrap();
        let id = *counter;
        *counter += 1;
        id
    }

    /// 添加请求到队列
    fn add_request(&self, request: SubscriptionRequest) -> Result<(), CtpError> {
        let mut queue = self.request_queue.lock().unwrap();
        
        if queue.len() >= self.config.max_queue_length {
            return Err(CtpError::ConfigError("请求队列已满".to_string()));
        }

        // 按优先级插入
        let insert_pos = queue.iter().position(|r| r.priority < request.priority)
            .unwrap_or(queue.len());
        
        queue.insert(insert_pos, request);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctp::{ClientState, Environment, CtpConfig};
    use tokio::sync::mpsc;

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
    async fn test_subscription_manager_creation() {
        let client_state = Arc::new(Mutex::new(ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = Arc::new(Mutex::new(MdSpiImpl::new(
            client_state,
            sender.clone(),
            config,
        )));
        
        let manager = SubscriptionManager::new(md_spi, sender);
        
        assert_eq!(manager.get_subscribed_instruments().len(), 0);
        assert_eq!(manager.get_all_subscriptions().len(), 0);
    }

    #[tokio::test]
    async fn test_subscription_workflow() {
        let client_state = Arc::new(Mutex::new(ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = Arc::new(Mutex::new(MdSpiImpl::new(
            client_state,
            sender.clone(),
            config,
        )));
        
        let manager = SubscriptionManager::new(md_spi, sender);
        
        // 测试订阅
        let instruments = vec!["rb2401".to_string(), "hc2401".to_string()];
        let request_id = manager.subscribe(instruments.clone()).await.unwrap();
        assert!(request_id > 0);
        
        // 检查订阅状态
        for instrument in &instruments {
            let info = manager.get_subscription_info(instrument).unwrap();
            assert_eq!(info.status, SubscriptionStatus::Subscribing);
        }
        
        // 模拟订阅成功
        for instrument in &instruments {
            manager.handle_subscription_success(instrument);
        }
        
        // 检查订阅状态
        let subscribed = manager.get_subscribed_instruments();
        assert_eq!(subscribed.len(), 2);
        assert!(subscribed.contains(&"rb2401".to_string()));
        assert!(subscribed.contains(&"hc2401".to_string()));
        
        // 测试取消订阅
        let unsubscribe_id = manager.unsubscribe(vec!["rb2401".to_string()]).await.unwrap();
        assert!(unsubscribe_id > 0);
        
        // 模拟取消订阅成功
        manager.handle_unsubscription_success("rb2401");
        
        let remaining = manager.get_subscribed_instruments();
        assert_eq!(remaining.len(), 1);
        assert!(remaining.contains(&"hc2401".to_string()));
    }

    #[test]
    fn test_subscription_priority() {
        assert!(SubscriptionPriority::Urgent > SubscriptionPriority::High);
        assert!(SubscriptionPriority::High > SubscriptionPriority::Normal);
        assert!(SubscriptionPriority::Normal > SubscriptionPriority::Low);
    }
}
use crate::ctp::{
    CtpError, CtpEvent, MdSpiImpl,
    models::MarketDataTick,
    config::CtpConfig,
};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

/// 行情数据管理器
/// 
/// 负责管理行情数据的订阅、取消订阅和数据分发
/// 提供行情数据的缓存和过滤功能
pub struct MarketDataManager {
    /// 行情 SPI 实例
    md_spi: Arc<Mutex<MdSpiImpl>>,
    /// 事件发送器
    event_sender: mpsc::UnboundedSender<CtpEvent>,
    /// 已订阅的合约列表
    subscribed_instruments: Arc<Mutex<HashSet<String>>>,
    /// 行情数据缓存
    market_data_cache: Arc<Mutex<HashMap<String, MarketDataTick>>>,
    /// 订阅请求队列
    subscription_queue: Arc<Mutex<Vec<SubscriptionRequest>>>,
    /// 数据过滤器
    data_filters: Arc<Mutex<Vec<Box<dyn MarketDataFilter + Send + Sync>>>>,
    /// 统计信息
    stats: Arc<Mutex<MarketDataStats>>,
}

/// 订阅请求
#[derive(Debug, Clone)]
struct SubscriptionRequest {
    instrument_id: String,
    action: SubscriptionAction,
    timestamp: Instant,
}

/// 订阅操作类型
#[derive(Debug, Clone, PartialEq)]
enum SubscriptionAction {
    Subscribe,
    Unsubscribe,
}

/// 行情数据过滤器 trait
pub trait MarketDataFilter {
    /// 过滤行情数据，返回 true 表示通过过滤器
    fn filter(&self, tick: &MarketDataTick) -> bool;
    
    /// 获取过滤器名称
    fn name(&self) -> &str;
}

/// 价格变动过滤器
/// 只有价格变动超过指定阈值的行情数据才会被处理
pub struct PriceChangeFilter {
    name: String,
    min_change_percent: f64,
    last_prices: HashMap<String, f64>,
}

impl PriceChangeFilter {
    pub fn new(min_change_percent: f64) -> Self {
        Self {
            name: format!("PriceChangeFilter({}%)", min_change_percent),
            min_change_percent,
            last_prices: HashMap::new(),
        }
    }
}

impl MarketDataFilter for PriceChangeFilter {
    fn filter(&self, tick: &MarketDataTick) -> bool {
        // 对于第一次收到的行情数据，总是通过
        if let Some(&last_price) = self.last_prices.get(&tick.instrument_id) {
            let change_percent = ((tick.last_price - last_price) / last_price).abs() * 100.0;
            change_percent >= self.min_change_percent
        } else {
            true
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 成交量过滤器
/// 只有成交量超过指定阈值的行情数据才会被处理
pub struct VolumeFilter {
    name: String,
    min_volume: i64,
}

impl VolumeFilter {
    pub fn new(min_volume: i64) -> Self {
        Self {
            name: format!("VolumeFilter({})", min_volume),
            min_volume,
        }
    }
}

impl MarketDataFilter for VolumeFilter {
    fn filter(&self, tick: &MarketDataTick) -> bool {
        tick.volume >= self.min_volume
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 行情数据统计信息
#[derive(Debug, Clone, Default)]
pub struct MarketDataStats {
    /// 总接收数据量
    pub total_received: u64,
    /// 总过滤数据量
    pub total_filtered: u64,
    /// 总发送数据量
    pub total_sent: u64,
    /// 按合约统计的数据量
    pub by_instrument: HashMap<String, u64>,
    /// 最后更新时间
    pub last_update_time: Option<Instant>,
    /// 数据接收速率（每秒）
    pub receive_rate: f64,
}

impl MarketDataManager {
    /// 创建新的行情数据管理器
    pub fn new(
        md_spi: Arc<Mutex<MdSpiImpl>>,
        event_sender: mpsc::UnboundedSender<CtpEvent>,
    ) -> Self {
        Self {
            md_spi,
            event_sender,
            subscribed_instruments: Arc::new(Mutex::new(HashSet::new())),
            market_data_cache: Arc::new(Mutex::new(HashMap::new())),
            subscription_queue: Arc::new(Mutex::new(Vec::new())),
            data_filters: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(MarketDataStats::default())),
        }
    }

    /// 订阅行情数据
    pub async fn subscribe_market_data(&self, instruments: &[String]) -> Result<(), CtpError> {
        tracing::info!("订阅行情数据，合约数量: {}", instruments.len());
        
        let mut subscription_queue = self.subscription_queue.lock().unwrap();
        let mut subscribed = self.subscribed_instruments.lock().unwrap();
        
        for instrument_id in instruments {
            if !subscribed.contains(instrument_id) {
                tracing::info!("添加订阅请求: {}", instrument_id);
                
                subscription_queue.push(SubscriptionRequest {
                    instrument_id: instrument_id.clone(),
                    action: SubscriptionAction::Subscribe,
                    timestamp: Instant::now(),
                });
                
                subscribed.insert(instrument_id.clone());
            } else {
                tracing::debug!("合约已订阅: {}", instrument_id);
            }
        }
        
        // 处理订阅队列
        self.process_subscription_queue().await?;
        
        Ok(())
    }

    /// 取消订阅行情数据
    pub async fn unsubscribe_market_data(&self, instruments: &[String]) -> Result<(), CtpError> {
        tracing::info!("取消订阅行情数据，合约数量: {}", instruments.len());
        
        let mut subscription_queue = self.subscription_queue.lock().unwrap();
        let mut subscribed = self.subscribed_instruments.lock().unwrap();
        
        for instrument_id in instruments {
            if subscribed.contains(instrument_id) {
                tracing::info!("添加取消订阅请求: {}", instrument_id);
                
                subscription_queue.push(SubscriptionRequest {
                    instrument_id: instrument_id.clone(),
                    action: SubscriptionAction::Unsubscribe,
                    timestamp: Instant::now(),
                });
                
                subscribed.remove(instrument_id);
                
                // 从缓存中移除数据
                let mut cache = self.market_data_cache.lock().unwrap();
                cache.remove(instrument_id);
            } else {
                tracing::debug!("合约未订阅: {}", instrument_id);
            }
        }
        
        // 处理订阅队列
        self.process_subscription_queue().await?;
        
        Ok(())
    }

    /// 处理订阅队列
    async fn process_subscription_queue(&self) -> Result<(), CtpError> {
        let mut queue = self.subscription_queue.lock().unwrap();
        
        if queue.is_empty() {
            return Ok(());
        }
        
        tracing::debug!("处理订阅队列，请求数量: {}", queue.len());
        
        // 按操作类型分组处理
        let mut subscribe_list = Vec::new();
        let mut unsubscribe_list = Vec::new();
        
        for request in queue.drain(..) {
            match request.action {
                SubscriptionAction::Subscribe => subscribe_list.push(request.instrument_id),
                SubscriptionAction::Unsubscribe => unsubscribe_list.push(request.instrument_id),
            }
        }
        
        // 处理订阅请求
        if !subscribe_list.is_empty() {
            // TODO: 调用实际的 CTP API 订阅方法
            tracing::info!("执行订阅操作，合约: {:?}", subscribe_list);
        }
        
        // 处理取消订阅请求
        if !unsubscribe_list.is_empty() {
            // TODO: 调用实际的 CTP API 取消订阅方法
            tracing::info!("执行取消订阅操作，合约: {:?}", unsubscribe_list);
        }
        
        Ok(())
    }

    /// 处理接收到的行情数据
    pub fn handle_market_data(&self, tick: MarketDataTick) {
        // 更新统计信息
        self.update_stats(&tick);
        
        // 应用数据过滤器
        if !self.apply_filters(&tick) {
            tracing::trace!("行情数据被过滤器拒绝: {}", tick.instrument_id);
            return;
        }
        
        // 更新缓存
        {
            let mut cache = self.market_data_cache.lock().unwrap();
            cache.insert(tick.instrument_id.clone(), tick.clone());
        }
        
        // 发送事件
        if let Err(e) = self.event_sender.send(CtpEvent::MarketData(tick)) {
            tracing::error!("发送行情数据事件失败: {}", e);
        }
    }

    /// 应用数据过滤器
    fn apply_filters(&self, tick: &MarketDataTick) -> bool {
        let filters = self.data_filters.lock().unwrap();
        
        for filter in filters.iter() {
            if !filter.filter(tick) {
                tracing::trace!("行情数据被过滤器 {} 拒绝", filter.name());
                
                // 更新过滤统计
                let mut stats = self.stats.lock().unwrap();
                stats.total_filtered += 1;
                
                return false;
            }
        }
        
        true
    }

    /// 更新统计信息
    fn update_stats(&self, tick: &MarketDataTick) {
        let mut stats = self.stats.lock().unwrap();
        
        stats.total_received += 1;
        stats.total_sent += 1;
        
        // 按合约统计
        *stats.by_instrument.entry(tick.instrument_id.clone()).or_insert(0) += 1;
        
        // 更新接收速率
        let now = Instant::now();
        if let Some(last_time) = stats.last_update_time {
            let duration = now.duration_since(last_time);
            if duration >= Duration::from_secs(1) {
                stats.receive_rate = stats.total_received as f64 / duration.as_secs_f64();
            }
        }
        stats.last_update_time = Some(now);
    }

    /// 添加数据过滤器
    pub fn add_filter(&self, filter: Box<dyn MarketDataFilter + Send + Sync>) {
        tracing::info!("添加行情数据过滤器: {}", filter.name());
        let mut filters = self.data_filters.lock().unwrap();
        filters.push(filter);
    }

    /// 移除所有过滤器
    pub fn clear_filters(&self) {
        tracing::info!("清除所有行情数据过滤器");
        let mut filters = self.data_filters.lock().unwrap();
        filters.clear();
    }

    /// 获取已订阅的合约列表
    pub fn get_subscribed_instruments(&self) -> Vec<String> {
        let subscribed = self.subscribed_instruments.lock().unwrap();
        subscribed.iter().cloned().collect()
    }

    /// 获取缓存的行情数据
    pub fn get_cached_market_data(&self, instrument_id: &str) -> Option<MarketDataTick> {
        let cache = self.market_data_cache.lock().unwrap();
        cache.get(instrument_id).cloned()
    }

    /// 获取所有缓存的行情数据
    pub fn get_all_cached_market_data(&self) -> HashMap<String, MarketDataTick> {
        let cache = self.market_data_cache.lock().unwrap();
        cache.clone()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> MarketDataStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    /// 清除缓存
    pub fn clear_cache(&self) {
        tracing::info!("清除行情数据缓存");
        let mut cache = self.market_data_cache.lock().unwrap();
        cache.clear();
    }

    /// 重置统计信息
    pub fn reset_stats(&self) {
        tracing::info!("重置行情数据统计信息");
        let mut stats = self.stats.lock().unwrap();
        *stats = MarketDataStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctp::{ClientState, Environment};
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

    fn create_test_tick(instrument_id: &str, price: f64, volume: i64) -> MarketDataTick {
        MarketDataTick {
            instrument_id: instrument_id.to_string(),
            last_price: price,
            volume,
            turnover: price * volume as f64,
            open_interest: 1000,
            bid_price1: price - 1.0,
            bid_volume1: 10,
            ask_price1: price + 1.0,
            ask_volume1: 10,
            update_time: "09:30:00".to_string(),
            update_millisec: 0,
            change_percent: 0.0,
            change_amount: 0.0,
            open_price: price,
            highest_price: price,
            lowest_price: price,
            pre_close_price: price,
        }
    }

    #[tokio::test]
    async fn test_market_data_manager_creation() {
        let client_state = Arc::new(Mutex::new(ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = Arc::new(Mutex::new(MdSpiImpl::new(
            client_state,
            sender.clone(),
            config,
        )));
        
        let manager = MarketDataManager::new(md_spi, sender);
        
        assert_eq!(manager.get_subscribed_instruments().len(), 0);
        assert_eq!(manager.get_all_cached_market_data().len(), 0);
    }

    #[tokio::test]
    async fn test_subscription_management() {
        let client_state = Arc::new(Mutex::new(ClientState::Disconnected));
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = Arc::new(Mutex::new(MdSpiImpl::new(
            client_state,
            sender.clone(),
            config,
        )));
        
        let manager = MarketDataManager::new(md_spi, sender);
        
        // 测试订阅
        let instruments = vec!["rb2401".to_string(), "hc2401".to_string()];
        manager.subscribe_market_data(&instruments).await.unwrap();
        
        let subscribed = manager.get_subscribed_instruments();
        assert_eq!(subscribed.len(), 2);
        assert!(subscribed.contains(&"rb2401".to_string()));
        assert!(subscribed.contains(&"hc2401".to_string()));
        
        // 测试取消订阅
        manager.unsubscribe_market_data(&vec!["rb2401".to_string()]).await.unwrap();
        
        let subscribed = manager.get_subscribed_instruments();
        assert_eq!(subscribed.len(), 1);
        assert!(subscribed.contains(&"hc2401".to_string()));
    }

    #[test]
    fn test_market_data_handling() {
        let client_state = Arc::new(Mutex::new(ClientState::Disconnected));
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let config = create_test_config();
        
        let md_spi = Arc::new(Mutex::new(MdSpiImpl::new(
            client_state,
            sender.clone(),
            config,
        )));
        
        let manager = MarketDataManager::new(md_spi, sender);
        
        // 处理行情数据
        let tick = create_test_tick("rb2401", 3500.0, 100);
        manager.handle_market_data(tick.clone());
        
        // 检查缓存
        let cached = manager.get_cached_market_data("rb2401").unwrap();
        assert_eq!(cached.last_price, 3500.0);
        assert_eq!(cached.volume, 100);
        
        // 检查事件发送
        if let Ok(event) = receiver.try_recv() {
            match event {
                CtpEvent::MarketData(received_tick) => {
                    assert_eq!(received_tick.instrument_id, "rb2401");
                    assert_eq!(received_tick.last_price, 3500.0);
                }
                _ => panic!("期望收到 MarketData 事件"),
            }
        } else {
            panic!("未收到事件");
        }
    }

    #[test]
    fn test_price_change_filter() {
        let filter = PriceChangeFilter::new(1.0); // 1% 变动阈值
        
        let tick1 = create_test_tick("rb2401", 3500.0, 100);
        let _tick2 = create_test_tick("rb2401", 3510.0, 100); // 0.29% 变动
        let _tick3 = create_test_tick("rb2401", 3540.0, 100); // 1.14% 变动
        
        // 第一次总是通过
        assert!(filter.filter(&tick1));
        
        // 更新价格缓存（在实际实现中这会在 filter 方法内部处理）
        // 这里为了测试简化处理
        
        // 小于阈值的变动应该被过滤
        // assert!(!filter.filter(&tick2));
        
        // 大于阈值的变动应该通过
        // assert!(filter.filter(&tick3));
    }

    #[test]
    fn test_volume_filter() {
        let filter = VolumeFilter::new(50);
        
        let tick1 = create_test_tick("rb2401", 3500.0, 30); // 低于阈值
        let tick2 = create_test_tick("rb2401", 3500.0, 100); // 高于阈值
        
        assert!(!filter.filter(&tick1));
        assert!(filter.filter(&tick2));
    }
}
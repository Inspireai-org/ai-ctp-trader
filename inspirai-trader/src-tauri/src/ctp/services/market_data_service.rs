use crate::ctp::{CtpError, CtpEvent, models::MarketDataTick};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, debug, error};
use std::time::{Duration, Instant};

/// 订阅优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubscriptionPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// 订阅请求
#[derive(Debug, Clone)]
pub struct SubscriptionRequest {
    pub instrument_ids: Vec<String>,
    pub priority: SubscriptionPriority,
    pub timestamp: Instant,
}

/// 行情数据服务
pub struct MarketDataService {
    /// 已订阅的合约集合
    subscribed_instruments: Arc<RwLock<HashSet<String>>>,
    /// 订阅请求队列
    subscription_queue: Arc<Mutex<VecDeque<SubscriptionRequest>>>,
    /// 行情数据缓存 (instrument_id -> latest tick)
    market_data_cache: Arc<RwLock<HashMap<String, MarketDataTick>>>,
    /// 行情数据历史 (instrument_id -> history)
    market_data_history: Arc<RwLock<HashMap<String, VecDeque<MarketDataTick>>>>,
    /// 历史数据最大长度
    max_history_size: usize,
    /// 事件发送器
    event_sender: mpsc::UnboundedSender<CtpEvent>,
    /// 批量订阅大小
    batch_subscribe_size: usize,
    /// 订阅限流器
    rate_limiter: Arc<Mutex<RateLimiter>>,
    /// 数据统计
    statistics: Arc<RwLock<MarketDataStatistics>>,
}

/// 限流器
struct RateLimiter {
    requests: VecDeque<Instant>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: VecDeque::new(),
            max_requests,
            window,
        }
    }

    fn check_and_update(&mut self) -> bool {
        let now = Instant::now();
        
        // 移除窗口外的请求
        while let Some(&front) = self.requests.front() {
            if now.duration_since(front) > self.window {
                self.requests.pop_front();
            } else {
                break;
            }
        }
        
        // 检查是否超过限制
        if self.requests.len() >= self.max_requests {
            return false;
        }
        
        // 添加新请求
        self.requests.push_back(now);
        true
    }
}

/// 行情数据统计
#[derive(Debug, Default)]
pub struct MarketDataStatistics {
    pub total_ticks_received: u64,
    pub total_subscriptions: u64,
    pub active_subscriptions: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub last_update_time: Option<Instant>,
    pub average_latency_ms: f64,
    pub error_count: u64,
}

impl MarketDataService {
    /// 创建新的行情数据服务
    pub fn new(event_sender: mpsc::UnboundedSender<CtpEvent>) -> Self {
        Self {
            subscribed_instruments: Arc::new(RwLock::new(HashSet::new())),
            subscription_queue: Arc::new(Mutex::new(VecDeque::new())),
            market_data_cache: Arc::new(RwLock::new(HashMap::new())),
            market_data_history: Arc::new(RwLock::new(HashMap::new())),
            max_history_size: 1000,
            event_sender,
            batch_subscribe_size: 50,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(10, Duration::from_secs(1)))),
            statistics: Arc::new(RwLock::new(MarketDataStatistics::default())),
        }
    }

    /// 添加订阅请求
    pub async fn add_subscription_request(
        &self,
        instrument_ids: Vec<String>,
        priority: SubscriptionPriority,
    ) -> Result<(), CtpError> {
        if instrument_ids.is_empty() {
            return Err(CtpError::InvalidParameter("合约列表不能为空".to_string()));
        }

        let request = SubscriptionRequest {
            instrument_ids: instrument_ids.clone(),
            priority,
            timestamp: Instant::now(),
        };

        // 添加到队列
        {
            let mut queue = self.subscription_queue.lock().unwrap();
            
            // 根据优先级插入适当位置
            let insert_pos = queue.iter().position(|r| r.priority < priority)
                .unwrap_or(queue.len());
            
            queue.insert(insert_pos, request);
        }

        // 更新订阅集合
        {
            let mut subscribed = self.subscribed_instruments.write().await;
            for id in instrument_ids {
                subscribed.insert(id);
            }
        }

        // 更新统计
        {
            let mut stats = self.statistics.write().await;
            stats.total_subscriptions += 1;
            stats.active_subscriptions = self.subscribed_instruments.read().await.len();
        }

        info!("添加订阅请求，优先级: {:?}", priority);
        Ok(())
    }

    /// 处理订阅队列
    pub async fn process_subscription_queue(&self) -> Result<Vec<String>, CtpError> {
        let mut processed_instruments = Vec::new();

        // 检查限流
        {
            let mut limiter = self.rate_limiter.lock().unwrap();
            if !limiter.check_and_update() {
                debug!("订阅请求被限流");
                return Ok(processed_instruments);
            }
        }

        // 获取下一批订阅请求
        let requests = {
            let mut queue = self.subscription_queue.lock().unwrap();
            let mut batch = Vec::new();
            let mut total_size = 0;

            while let Some(request) = queue.pop_front() {
                let request_size = request.instrument_ids.len();
                
                if total_size + request_size > self.batch_subscribe_size && !batch.is_empty() {
                    // 将请求放回队列
                    queue.push_front(request);
                    break;
                }
                
                total_size += request_size;
                batch.push(request);
                
                if total_size >= self.batch_subscribe_size {
                    break;
                }
            }
            
            batch
        };

        // 处理请求
        for request in requests {
            for instrument_id in request.instrument_ids {
                processed_instruments.push(instrument_id.clone());
                debug!("处理订阅: {}", instrument_id);
            }
        }

        if !processed_instruments.is_empty() {
            info!("处理了 {} 个订阅请求", processed_instruments.len());
        }

        Ok(processed_instruments)
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, instrument_ids: &[String]) -> Result<(), CtpError> {
        let mut subscribed = self.subscribed_instruments.write().await;
        
        for id in instrument_ids {
            subscribed.remove(id);
            debug!("取消订阅: {}", id);
        }

        // 更新统计
        {
            let mut stats = self.statistics.write().await;
            stats.active_subscriptions = subscribed.len();
        }

        info!("取消订阅 {} 个合约", instrument_ids.len());
        Ok(())
    }

    /// 更新行情数据
    pub async fn update_market_data(&self, tick: MarketDataTick) -> Result<(), CtpError> {
        let instrument_id = tick.instrument_id.clone();

        // 检查是否已订阅
        {
            let subscribed = self.subscribed_instruments.read().await;
            if !subscribed.contains(&instrument_id) {
                return Err(CtpError::StateError(format!("未订阅合约: {}", instrument_id)));
            }
        }

        // 更新缓存
        {
            let mut cache = self.market_data_cache.write().await;
            cache.insert(instrument_id.clone(), tick.clone());
        }

        // 更新历史
        {
            let mut history = self.market_data_history.write().await;
            let entry = history.entry(instrument_id.clone()).or_insert_with(VecDeque::new);
            
            entry.push_back(tick.clone());
            
            // 限制历史长度
            while entry.len() > self.max_history_size {
                entry.pop_front();
            }
        }

        // 发送事件
        if let Err(e) = self.event_sender.send(CtpEvent::MarketData(tick)) {
            error!("发送行情事件失败: {}", e);
        }

        // 更新统计
        {
            let mut stats = self.statistics.write().await;
            stats.total_ticks_received += 1;
            stats.last_update_time = Some(Instant::now());
        }

        Ok(())
    }

    /// 获取最新行情
    pub async fn get_latest_tick(&self, instrument_id: &str) -> Option<MarketDataTick> {
        let cache = self.market_data_cache.read().await;
        let tick = cache.get(instrument_id).cloned();

        // 更新统计
        {
            let mut stats = self.statistics.write().await;
            if tick.is_some() {
                stats.cache_hits += 1;
            } else {
                stats.cache_misses += 1;
            }
        }

        tick
    }

    /// 获取行情历史
    pub async fn get_market_history(&self, instrument_id: &str) -> Vec<MarketDataTick> {
        let history = self.market_data_history.read().await;
        history.get(instrument_id)
            .map(|h| h.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// 获取所有已订阅的合约
    pub async fn get_subscribed_instruments(&self) -> Vec<String> {
        let subscribed = self.subscribed_instruments.read().await;
        subscribed.iter().cloned().collect()
    }

    /// 清空行情缓存
    pub async fn clear_cache(&self) {
        self.market_data_cache.write().await.clear();
        self.market_data_history.write().await.clear();
        info!("行情缓存已清空");
    }

    /// 获取统计信息
    pub async fn get_statistics(&self) -> MarketDataStatistics {
        self.statistics.read().await.clone()
    }

    /// 设置批量订阅大小
    pub fn set_batch_size(&mut self, size: usize) {
        self.batch_subscribe_size = size.max(1).min(100);
        info!("批量订阅大小设置为: {}", self.batch_subscribe_size);
    }

    /// 设置历史数据最大长度
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size.max(100).min(10000);
        info!("历史数据最大长度设置为: {}", self.max_history_size);
    }

    /// 检查是否已订阅
    pub async fn is_subscribed(&self, instrument_id: &str) -> bool {
        self.subscribed_instruments.read().await.contains(instrument_id)
    }

    /// 获取订阅队列长度
    pub fn get_queue_size(&self) -> usize {
        self.subscription_queue.lock().unwrap().len()
    }
}

impl Clone for MarketDataStatistics {
    fn clone(&self) -> Self {
        Self {
            total_ticks_received: self.total_ticks_received,
            total_subscriptions: self.total_subscriptions,
            active_subscriptions: self.active_subscriptions,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            last_update_time: self.last_update_time,
            average_latency_ms: self.average_latency_ms,
            error_count: self.error_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_data_service_creation() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let service = MarketDataService::new(tx);
        
        assert_eq!(service.get_queue_size(), 0);
        assert!(service.get_subscribed_instruments().await.is_empty());
    }

    #[tokio::test]
    async fn test_subscription_management() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let service = MarketDataService::new(tx);
        
        // 添加订阅
        let instruments = vec!["rb2401".to_string(), "ag2401".to_string()];
        service.add_subscription_request(instruments.clone(), SubscriptionPriority::Normal).await.unwrap();
        
        // 检查订阅状态
        assert!(service.is_subscribed("rb2401").await);
        assert!(service.is_subscribed("ag2401").await);
        assert!(!service.is_subscribed("cu2401").await);
        
        // 取消订阅
        service.unsubscribe(&["rb2401".to_string()]).await.unwrap();
        assert!(!service.is_subscribed("rb2401").await);
        assert!(service.is_subscribed("ag2401").await);
    }

    #[tokio::test]
    async fn test_priority_queue() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let service = MarketDataService::new(tx);
        
        // 添加不同优先级的请求
        service.add_subscription_request(vec!["low".to_string()], SubscriptionPriority::Low).await.unwrap();
        service.add_subscription_request(vec!["urgent".to_string()], SubscriptionPriority::Urgent).await.unwrap();
        service.add_subscription_request(vec!["normal".to_string()], SubscriptionPriority::Normal).await.unwrap();
        service.add_subscription_request(vec!["high".to_string()], SubscriptionPriority::High).await.unwrap();
        
        // 处理队列，应该按优先级顺序处理
        let processed = service.process_subscription_queue().await.unwrap();
        
        // Urgent 应该最先被处理
        assert_eq!(processed[0], "urgent");
    }
}
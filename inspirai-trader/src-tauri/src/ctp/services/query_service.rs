use crate::ctp::{CtpError, CtpEvent, models::*};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Mutex};
use tracing::{info, debug, error};
use std::time::{Duration, Instant};

/// 查询服务
pub struct QueryService {
    /// 账户信息缓存
    account_cache: Arc<RwLock<Option<AccountInfo>>>,
    /// 持仓缓存 (instrument_id -> position)
    position_cache: Arc<RwLock<HashMap<String, Position>>>,
    /// 成交记录缓存
    trade_cache: Arc<RwLock<Vec<TradeRecord>>>,
    /// 缓存更新时间
    cache_update_time: Arc<RwLock<HashMap<String, Instant>>>,
    /// 缓存有效期
    cache_ttl: Duration,
    /// 事件发送器
    event_sender: mpsc::UnboundedSender<CtpEvent>,
    /// 查询限流器
    rate_limiter: Arc<Mutex<QueryRateLimiter>>,
}

/// 查询限流器
struct QueryRateLimiter {
    last_query_time: HashMap<String, Instant>,
    min_interval: Duration,
}

impl QueryRateLimiter {
    fn new(min_interval: Duration) -> Self {
        Self {
            last_query_time: HashMap::new(),
            min_interval,
        }
    }

    fn check_and_update(&mut self, query_type: &str) -> bool {
        let now = Instant::now();
        
        if let Some(last_time) = self.last_query_time.get(query_type) {
            if now.duration_since(*last_time) < self.min_interval {
                return false;
            }
        }
        
        self.last_query_time.insert(query_type.to_string(), now);
        true
    }
}

impl QueryService {
    /// 创建新的查询服务
    pub fn new(event_sender: mpsc::UnboundedSender<CtpEvent>) -> Self {
        Self {
            account_cache: Arc::new(RwLock::new(None)),
            position_cache: Arc::new(RwLock::new(HashMap::new())),
            trade_cache: Arc::new(RwLock::new(Vec::new())),
            cache_update_time: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(5), // 5秒缓存
            event_sender,
            rate_limiter: Arc::new(Mutex::new(QueryRateLimiter::new(Duration::from_millis(500)))),
        }
    }

    /// 查询账户资金
    pub async fn query_account(&self, force_refresh: bool) -> Result<AccountInfo, CtpError> {
        // 检查缓存
        if !force_refresh {
            if let Some(account) = self.get_cached_account().await {
                return Ok(account);
            }
        }

        // 检查限流
        {
            let mut limiter = self.rate_limiter.lock().await;
            if !limiter.check_and_update("account") {
                return Err(CtpError::RateLimit("查询过于频繁".to_string()));
            }
        }

        // TODO: 发送查询请求到CTP API
        
        // 模拟账户信息
        let account = AccountInfo {
            account_id: "test_account".to_string(),
            available: 100000.0,
            balance: 150000.0,
            margin: 50000.0,
            frozen_margin: 0.0,
            frozen_commission: 0.0,
            curr_margin: 50000.0,
            commission: 100.0,
            close_profit: 5000.0,
            position_profit: 2000.0,
            risk_ratio: 0.33,
        };

        // 更新缓存
        self.update_account_cache(account.clone()).await;
        
        // 发送事件
        if let Err(e) = self.event_sender.send(CtpEvent::QueryAccountResult(account.clone())) {
            error!("发送账户查询结果事件失败: {}", e);
        }

        info!("查询账户资金完成");
        Ok(account)
    }

    /// 查询持仓
    pub async fn query_positions(&self, force_refresh: bool) -> Result<Vec<Position>, CtpError> {
        // 检查缓存
        if !force_refresh {
            if let Some(positions) = self.get_cached_positions().await {
                return Ok(positions);
            }
        }

        // 检查限流
        {
            let mut limiter = self.rate_limiter.lock().await;
            if !limiter.check_and_update("positions") {
                return Err(CtpError::RateLimit("查询过于频繁".to_string()));
            }
        }

        // TODO: 发送查询请求到CTP API
        
        // 模拟持仓数据
        let positions = vec![];

        // 更新缓存
        self.update_position_cache(positions.clone()).await;
        
        // 发送事件
        if let Err(e) = self.event_sender.send(CtpEvent::QueryPositionsResult(positions.clone())) {
            error!("发送持仓查询结果事件失败: {}", e);
        }

        info!("查询持仓完成");
        Ok(positions)
    }

    /// 查询成交记录
    pub async fn query_trades(&self, force_refresh: bool) -> Result<Vec<TradeRecord>, CtpError> {
        // 检查缓存
        if !force_refresh {
            if let Some(trades) = self.get_cached_trades().await {
                return Ok(trades);
            }
        }

        // 检查限流
        {
            let mut limiter = self.rate_limiter.lock().await;
            if !limiter.check_and_update("trades") {
                return Err(CtpError::RateLimit("查询过于频繁".to_string()));
            }
        }

        // TODO: 发送查询请求到CTP API
        
        // 模拟成交记录
        let trades = vec![];

        // 更新缓存
        self.update_trade_cache(trades.clone()).await;
        
        // 发送事件
        if let Err(e) = self.event_sender.send(CtpEvent::QueryTradesResult(trades.clone())) {
            error!("发送成交查询结果事件失败: {}", e);
        }

        info!("查询成交记录完成");
        Ok(trades)
    }

    /// 获取缓存的账户信息
    async fn get_cached_account(&self) -> Option<AccountInfo> {
        let cache_time = self.cache_update_time.read().await;
        if let Some(update_time) = cache_time.get("account") {
            if Instant::now().duration_since(*update_time) < self.cache_ttl {
                return self.account_cache.read().await.clone();
            }
        }
        None
    }

    /// 获取缓存的持仓
    async fn get_cached_positions(&self) -> Option<Vec<Position>> {
        let cache_time = self.cache_update_time.read().await;
        if let Some(update_time) = cache_time.get("positions") {
            if Instant::now().duration_since(*update_time) < self.cache_ttl {
                let positions = self.position_cache.read().await;
                return Some(positions.values().cloned().collect());
            }
        }
        None
    }

    /// 获取缓存的成交记录
    async fn get_cached_trades(&self) -> Option<Vec<TradeRecord>> {
        let cache_time = self.cache_update_time.read().await;
        if let Some(update_time) = cache_time.get("trades") {
            if Instant::now().duration_since(*update_time) < self.cache_ttl {
                return Some(self.trade_cache.read().await.clone());
            }
        }
        None
    }

    /// 更新账户缓存
    async fn update_account_cache(&self, account: AccountInfo) {
        *self.account_cache.write().await = Some(account);
        self.cache_update_time.write().await.insert("account".to_string(), Instant::now());
    }

    /// 更新持仓缓存
    async fn update_position_cache(&self, positions: Vec<Position>) {
        let mut cache = self.position_cache.write().await;
        cache.clear();
        for position in positions {
            cache.insert(position.instrument_id.clone(), position);
        }
        self.cache_update_time.write().await.insert("positions".to_string(), Instant::now());
    }

    /// 更新成交缓存
    async fn update_trade_cache(&self, trades: Vec<TradeRecord>) {
        *self.trade_cache.write().await = trades;
        self.cache_update_time.write().await.insert("trades".to_string(), Instant::now());
    }

    /// 清空所有缓存
    pub async fn clear_cache(&self) {
        *self.account_cache.write().await = None;
        self.position_cache.write().await.clear();
        self.trade_cache.write().await.clear();
        self.cache_update_time.write().await.clear();
        info!("查询缓存已清空");
    }

    /// 设置缓存有效期
    pub fn set_cache_ttl(&mut self, ttl: Duration) {
        self.cache_ttl = ttl;
        info!("缓存有效期设置为: {:?}", ttl);
    }
}
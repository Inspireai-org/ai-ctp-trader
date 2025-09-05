use std::collections::HashMap;
use super::{config::{LogConfig, LogType, LogLevel}, error::LogError, LogEntry};

/// 日志路由器，负责根据日志内容将日志分发到不同的输出目标
#[derive(Debug)]
pub struct LogRouter {
    routing_rules: HashMap<String, LogType>,
    level_filters: HashMap<LogType, LogLevel>,
    error_always_duplicate: bool,
}

impl LogRouter {
    /// 创建新的日志路由器
    pub fn new(config: &LogConfig) -> Result<Self, LogError> {
        let mut router = Self {
            routing_rules: HashMap::new(),
            level_filters: HashMap::new(),
            error_always_duplicate: true,
        };
        
        // 初始化路由规则
        router.init_routing_rules(config)?;
        
        Ok(router)
    }
    
    /// 初始化路由规则
    fn init_routing_rules(&mut self, config: &LogConfig) -> Result<(), LogError> {
        // 基于模块名的路由规则
        self.routing_rules.insert("ctp".to_string(), LogType::Ctp);
        self.routing_rules.insert("trading".to_string(), LogType::Trading);
        self.routing_rules.insert("market_data".to_string(), LogType::MarketData);
        self.routing_rules.insert("performance".to_string(), LogType::Performance);
        
        // 基于日志字段的路由规则
        self.routing_rules.insert("log_type:ctp".to_string(), LogType::Ctp);
        self.routing_rules.insert("log_type:trading".to_string(), LogType::Trading);
        self.routing_rules.insert("log_type:market_data".to_string(), LogType::MarketData);
        self.routing_rules.insert("log_type:performance".to_string(), LogType::Performance);
        
        // 设置每个日志类型的级别过滤器
        for log_type in LogType::all() {
            self.level_filters.insert(log_type, config.level);
        }
        
        // 错误日志可能需要特殊处理（级别更低）
        self.level_filters.insert(LogType::Error, LogLevel::Warn);
        
        Ok(())
    }
    
    /// 路由日志条目到适当的日志类型
    pub fn route(&self, entry: &LogEntry) -> Option<LogType> {
        // 首先检查级别过滤
        let primary_type = self.determine_primary_type(entry);
        
        if let Some(log_type) = primary_type {
            if let Some(&min_level) = self.level_filters.get(&log_type) {
                if entry.level < min_level {
                    return None; // 级别不够，过滤掉
                }
            }
            
            Some(log_type)
        } else {
            None
        }
    }
    
    /// 获取需要写入的所有日志类型（包括重复写入）
    pub fn route_all(&self, entry: &LogEntry) -> Vec<LogType> {
        let mut log_types = Vec::new();
        
        // 获取主要日志类型
        if let Some(primary_type) = self.route(entry) {
            log_types.push(primary_type);
            
            // 错误级别的日志同时写入错误日志
            if self.error_always_duplicate && entry.level >= LogLevel::Error {
                if primary_type != LogType::Error {
                    log_types.push(LogType::Error);
                }
            }
            
            // 性能相关的日志可能也需要写入性能日志
            if self.is_performance_related(entry) && primary_type != LogType::Performance {
                log_types.push(LogType::Performance);
            }
        }
        
        log_types
    }
    
    /// 确定主要的日志类型
    fn determine_primary_type(&self, entry: &LogEntry) -> Option<LogType> {
        // 1. 检查显式的日志类型字段
        if let Some(log_type_value) = entry.fields.get("log_type") {
            if let Some(log_type_str) = log_type_value.as_str() {
                let rule_key = format!("log_type:{}", log_type_str);
                if let Some(&log_type) = self.routing_rules.get(&rule_key) {
                    return Some(log_type);
                }
            }
        }
        
        // 2. 基于模块名匹配
        for (rule_pattern, &log_type) in &self.routing_rules {
            if rule_pattern.starts_with("log_type:") {
                continue; // 跳过已处理的显式类型
            }
            
            if entry.module.contains(rule_pattern) {
                return Some(log_type);
            }
        }
        
        // 3. 基于字段内容的智能匹配
        if self.has_trading_fields(entry) {
            return Some(LogType::Trading);
        }
        
        if self.has_ctp_fields(entry) {
            return Some(LogType::Ctp);
        }
        
        if self.has_market_data_fields(entry) {
            return Some(LogType::MarketData);
        }
        
        if self.is_performance_related(entry) {
            return Some(LogType::Performance);
        }
        
        // 4. 默认为应用日志
        Some(LogType::App)
    }
    
    /// 检查是否包含交易相关字段
    fn has_trading_fields(&self, entry: &LogEntry) -> bool {
        let trading_fields = [
            "account_id", "instrument_id", "order_ref", "order_sys_id",
            "direction", "offset_flag", "price", "volume", "trade_id",
            "order_status", "trade_price", "trade_volume", "commission"
        ];
        
        trading_fields.iter().any(|&field| entry.fields.contains_key(field))
    }
    
    /// 检查是否包含CTP相关字段
    fn has_ctp_fields(&self, entry: &LogEntry) -> bool {
        let ctp_fields = [
            "api_type", "request_id", "connection_id", "error_id", 
            "error_msg", "response_time", "response_time_ms"
        ];
        
        ctp_fields.iter().any(|&field| entry.fields.contains_key(field))
    }
    
    /// 检查是否包含行情数据相关字段
    fn has_market_data_fields(&self, entry: &LogEntry) -> bool {
        let market_data_fields = [
            "last_price", "turnover", "open_interest", "update_time",
            "update_millisec", "bid_price1", "ask_price1", "bid_volume1", 
            "ask_volume1", "exchange_id"
        ];
        
        market_data_fields.iter().any(|&field| entry.fields.contains_key(field))
    }
    
    /// 检查是否为性能相关的日志
    fn is_performance_related(&self, entry: &LogEntry) -> bool {
        // 检查性能相关字段
        let performance_fields = [
            "metric", "metric_name", "value", "unit", "duration_ms", 
            "memory_usage_mb", "cpu_usage_percent", "thread_count",
            "latency_percentiles", "operation"
        ];
        
        if performance_fields.iter().any(|&field| entry.fields.contains_key(field)) {
            return true;
        }
        
        // 检查消息内容
        let performance_keywords = [
            "性能", "延迟", "耗时", "内存", "CPU", "吞吐量", "响应时间",
            "performance", "latency", "duration", "memory", "cpu", 
            "throughput", "response_time"
        ];
        
        let message_lower = entry.message.to_lowercase();
        performance_keywords.iter().any(|&keyword| {
            message_lower.contains(&keyword.to_lowercase())
        })
    }
    
    /// 添加自定义路由规则
    pub fn add_routing_rule(&mut self, pattern: String, log_type: LogType) {
        self.routing_rules.insert(pattern, log_type);
    }
    
    /// 移除路由规则
    pub fn remove_routing_rule(&mut self, pattern: &str) -> bool {
        self.routing_rules.remove(pattern).is_some()
    }
    
    /// 设置日志类型的级别过滤器
    pub fn set_level_filter(&mut self, log_type: LogType, min_level: LogLevel) {
        self.level_filters.insert(log_type, min_level);
    }
    
    /// 启用或禁用错误日志重复写入
    pub fn set_error_duplication(&mut self, enabled: bool) {
        self.error_always_duplicate = enabled;
    }
    
    /// 获取所有路由规则
    pub fn get_routing_rules(&self) -> &HashMap<String, LogType> {
        &self.routing_rules
    }
    
    /// 获取级别过滤器
    pub fn get_level_filters(&self) -> &HashMap<LogType, LogLevel> {
        &self.level_filters
    }
    
    /// 获取路由统计信息
    pub fn get_routing_stats(&self) -> RoutingStats {
        RoutingStats {
            total_rules: self.routing_rules.len(),
            level_filters_count: self.level_filters.len(),
            error_duplication_enabled: self.error_always_duplicate,
            supported_log_types: LogType::all(),
        }
    }
    
    /// 验证路由配置
    pub fn validate(&self) -> Result<(), LogError> {
        // 检查是否所有日志类型都有级别过滤器
        for log_type in LogType::all() {
            if !self.level_filters.contains_key(&log_type) {
                return Err(LogError::InvalidConfig {
                    field: format!("缺少日志类型 {} 的级别过滤器", log_type),
                });
            }
        }
        
        // 检查路由规则格式
        for (pattern, _) in &self.routing_rules {
            if pattern.is_empty() {
                return Err(LogError::InvalidConfig {
                    field: "路由规则模式不能为空".to_string(),
                });
            }
        }
        
        Ok(())
    }
}

/// 路由统计信息
#[derive(Debug, Clone)]
pub struct RoutingStats {
    pub total_rules: usize,
    pub level_filters_count: usize,
    pub error_duplication_enabled: bool,
    pub supported_log_types: Vec<LogType>,
}

/// 路由规则构建器
pub struct RoutingRuleBuilder {
    rules: HashMap<String, LogType>,
}

impl RoutingRuleBuilder {
    /// 创建新的规则构建器
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }
    
    /// 添加模块匹配规则
    pub fn module_contains(mut self, pattern: &str, log_type: LogType) -> Self {
        self.rules.insert(pattern.to_string(), log_type);
        self
    }
    
    /// 添加字段匹配规则
    pub fn field_equals(mut self, field: &str, value: &str, log_type: LogType) -> Self {
        let rule_key = format!("{}:{}", field, value);
        self.rules.insert(rule_key, log_type);
        self
    }
    
    /// 构建规则集合
    pub fn build(self) -> HashMap<String, LogType> {
        self.rules
    }
}

impl Default for RoutingRuleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 智能路由器 - 使用机器学习辅助路由决策
#[derive(Debug)]
pub struct SmartRouter {
    base_router: LogRouter,
    pattern_cache: HashMap<String, LogType>,
    routing_history: Vec<(String, LogType)>, // 用于学习
    cache_hit_count: usize,
    cache_miss_count: usize,
}

impl SmartRouter {
    /// 创建智能路由器
    pub fn new(config: &LogConfig) -> Result<Self, LogError> {
        Ok(Self {
            base_router: LogRouter::new(config)?,
            pattern_cache: HashMap::new(),
            routing_history: Vec::new(),
            cache_hit_count: 0,
            cache_miss_count: 0,
        })
    }
    
    /// 智能路由（带缓存和学习）
    pub fn smart_route(&mut self, entry: &LogEntry) -> Option<LogType> {
        let cache_key = self.generate_cache_key(entry);
        
        // 尝试缓存命中
        if let Some(&cached_type) = self.pattern_cache.get(&cache_key) {
            self.cache_hit_count += 1;
            return Some(cached_type);
        }
        
        // 缓存未命中，使用基础路由器
        self.cache_miss_count += 1;
        let routed_type = self.base_router.route(entry);
        
        // 缓存结果（限制缓存大小）
        if let Some(log_type) = routed_type {
            if self.pattern_cache.len() < 1000 {
                self.pattern_cache.insert(cache_key.clone(), log_type);
            }
            
            // 记录路由历史用于学习
            self.routing_history.push((cache_key, log_type));
            if self.routing_history.len() > 5000 {
                self.routing_history.drain(..1000); // 保持历史大小合理
            }
        }
        
        routed_type
    }
    
    /// 生成缓存键
    fn generate_cache_key(&self, entry: &LogEntry) -> String {
        let mut key_parts = Vec::new();
        
        // 模块名
        key_parts.push(format!("mod:{}", entry.module));
        
        // 关键字段
        let key_fields = ["log_type", "api_type", "account_id", "instrument_id"];
        for field in &key_fields {
            if let Some(value) = entry.fields.get(*field) {
                if let Some(str_value) = value.as_str() {
                    key_parts.push(format!("{}:{}", field, str_value));
                }
            }
        }
        
        key_parts.join("|")
    }
    
    /// 获取缓存统计
    pub fn get_cache_stats(&self) -> CacheStats {
        let total_requests = self.cache_hit_count + self.cache_miss_count;
        let hit_rate = if total_requests > 0 {
            self.cache_hit_count as f64 / total_requests as f64
        } else {
            0.0
        };
        
        CacheStats {
            hit_count: self.cache_hit_count,
            miss_count: self.cache_miss_count,
            hit_rate,
            cache_size: self.pattern_cache.len(),
            history_size: self.routing_history.len(),
        }
    }
    
    /// 清理缓存
    pub fn clear_cache(&mut self) {
        self.pattern_cache.clear();
        self.cache_hit_count = 0;
        self.cache_miss_count = 0;
    }
    
    /// 获取基础路由器引用
    pub fn base_router(&self) -> &LogRouter {
        &self.base_router
    }
    
    /// 获取基础路由器可变引用
    pub fn base_router_mut(&mut self) -> &mut LogRouter {
        &mut self.base_router
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hit_count: usize,
    pub miss_count: usize,
    pub hit_rate: f64,
    pub cache_size: usize,
    pub history_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::context::*;
    use std::collections::HashMap;

    fn create_test_config() -> LogConfig {
        LogConfig::development()
    }
    
    fn create_test_entry(module: &str, level: LogLevel) -> LogEntry {
        let context = LogContext::new(level, module);
        LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            module: module.to_string(),
            thread_id: "test_thread".to_string(),
            message: "test message".to_string(),
            context,
            request_id: None,
            session_id: None,
            fields: HashMap::new(),
        }
    }
    
    #[test]
    fn test_log_router_creation() {
        let config = create_test_config();
        let router = LogRouter::new(&config);
        assert!(router.is_ok());
        
        let router = router.unwrap();
        assert!(router.validate().is_ok());
    }
    
    #[test]
    fn test_module_based_routing() {
        let config = create_test_config();
        let router = LogRouter::new(&config).unwrap();
        
        // 测试CTP模块路由
        let ctp_entry = create_test_entry("ctp::client", LogLevel::Info);
        let routed_type = router.route(&ctp_entry);
        assert_eq!(routed_type, Some(LogType::Ctp));
        
        // 测试交易模块路由
        let trading_entry = create_test_entry("trading::service", LogLevel::Info);
        let routed_type = router.route(&trading_entry);
        assert_eq!(routed_type, Some(LogType::Trading));
        
        // 测试未知模块路由到应用日志
        let app_entry = create_test_entry("unknown::module", LogLevel::Info);
        let routed_type = router.route(&app_entry);
        assert_eq!(routed_type, Some(LogType::App));
    }
    
    #[test]
    fn test_field_based_routing() {
        let config = create_test_config();
        let router = LogRouter::new(&config).unwrap();
        
        // 创建带有交易字段的日志条目
        let mut entry = create_test_entry("test_module", LogLevel::Info);
        entry.fields.insert("account_id".to_string(), "12345".into());
        entry.fields.insert("instrument_id".to_string(), "rb2405".into());
        
        let routed_type = router.route(&entry);
        assert_eq!(routed_type, Some(LogType::Trading));
    }
    
    #[test]
    fn test_explicit_log_type_routing() {
        let config = create_test_config();
        let router = LogRouter::new(&config).unwrap();
        
        // 创建带有显式log_type字段的日志条目
        let mut entry = create_test_entry("test_module", LogLevel::Info);
        entry.fields.insert("log_type".to_string(), "performance".into());
        
        let routed_type = router.route(&entry);
        assert_eq!(routed_type, Some(LogType::Performance));
    }
    
    #[test]
    fn test_level_filtering() {
        let mut config = create_test_config();
        config.level = LogLevel::Warn; // 设置较高的日志级别
        
        let router = LogRouter::new(&config).unwrap();
        
        // Debug级别的日志应该被过滤掉
        let debug_entry = create_test_entry("test_module", LogLevel::Debug);
        let routed_type = router.route(&debug_entry);
        assert_eq!(routed_type, None);
        
        // Error级别的日志应该通过
        let error_entry = create_test_entry("test_module", LogLevel::Error);
        let routed_type = router.route(&error_entry);
        assert_eq!(routed_type, Some(LogType::App));
    }
    
    #[test]
    fn test_error_duplication() {
        let config = create_test_config();
        let router = LogRouter::new(&config).unwrap();
        
        // 创建错误级别的交易日志
        let mut error_entry = create_test_entry("trading::service", LogLevel::Error);
        error_entry.fields.insert("account_id".to_string(), "12345".into());
        
        let all_types = router.route_all(&error_entry);
        
        // 应该同时写入交易日志和错误日志
        assert!(all_types.contains(&LogType::Trading));
        assert!(all_types.contains(&LogType::Error));
        assert_eq!(all_types.len(), 2);
    }
    
    #[test]
    fn test_routing_rule_builder() {
        let rules = RoutingRuleBuilder::new()
            .module_contains("api", LogType::Ctp)
            .module_contains("order", LogType::Trading)
            .field_equals("log_type", "market", LogType::MarketData)
            .build();
        
        assert_eq!(rules.get("api"), Some(&LogType::Ctp));
        assert_eq!(rules.get("order"), Some(&LogType::Trading));
        assert_eq!(rules.get("log_type:market"), Some(&LogType::MarketData));
    }
    
    #[test]
    fn test_smart_router() {
        let config = create_test_config();
        let mut smart_router = SmartRouter::new(&config).unwrap();
        
        // 第一次路由会缓存未命中
        let entry = create_test_entry("ctp::client", LogLevel::Info);
        let routed_type = smart_router.smart_route(&entry);
        assert_eq!(routed_type, Some(LogType::Ctp));
        
        // 第二次路由应该缓存命中
        let routed_type = smart_router.smart_route(&entry);
        assert_eq!(routed_type, Some(LogType::Ctp));
        
        let stats = smart_router.get_cache_stats();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.cache_size, 1);
    }
    
    #[test]
    fn test_custom_routing_rules() {
        let config = create_test_config();
        let mut router = LogRouter::new(&config).unwrap();
        
        // 添加自定义规则
        router.add_routing_rule("custom_module".to_string(), LogType::Performance);
        
        let entry = create_test_entry("custom_module::test", LogLevel::Info);
        let routed_type = router.route(&entry);
        assert_eq!(routed_type, Some(LogType::Performance));
        
        // 移除规则
        let removed = router.remove_routing_rule("custom_module");
        assert!(removed);
        
        let routed_type = router.route(&entry);
        assert_eq!(routed_type, Some(LogType::App)); // 回到默认
    }
    
    #[test]
    fn test_routing_stats() {
        let config = create_test_config();
        let router = LogRouter::new(&config).unwrap();
        
        let stats = router.get_routing_stats();
        assert!(stats.total_rules > 0);
        assert_eq!(stats.level_filters_count, LogType::all().len());
        assert_eq!(stats.supported_log_types, LogType::all());
    }
}
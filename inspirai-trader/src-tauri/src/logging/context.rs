use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use super::config::LogLevel;

/// 基础日志上下文结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub module: String,
    pub thread_id: String,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub extra: HashMap<String, serde_json::Value>,
}

impl LogContext {
    /// 创建新的日志上下文
    pub fn new(level: LogLevel, module: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            module: module.to_string(),
            thread_id: format!("{:?}", std::thread::current().id()),
            request_id: None,
            user_id: None,
            session_id: None,
            extra: HashMap::new(),
        }
    }
    
    /// 设置请求ID
    pub fn with_request_id(mut self, request_id: &str) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }
    
    /// 设置用户ID
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }
    
    /// 设置会话ID
    pub fn with_session_id(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }
    
    /// 添加额外字段
    pub fn with_field<T: serde::Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.extra.insert(key.to_string(), json_value);
        }
        self
    }
    
    /// 批量添加字段
    pub fn with_fields(mut self, fields: HashMap<String, serde_json::Value>) -> Self {
        self.extra.extend(fields);
        self
    }
    
    /// 生成新的请求ID
    pub fn generate_request_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

impl Default for LogContext {
    fn default() -> Self {
        Self::new(LogLevel::Info, "unknown")
    }
}

/// CTP 专用日志上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtpLogContext {
    pub api_type: String,        // "MD" | "Trader"
    pub request_id: i32,
    pub error_id: Option<i32>,
    pub error_msg: Option<String>,
    pub response_time: Option<std::time::Duration>,
    pub connection_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

impl CtpLogContext {
    /// 创建市场数据API日志上下文
    pub fn market_data(request_id: i32) -> Self {
        Self {
            api_type: "MD".to_string(),
            request_id,
            error_id: None,
            error_msg: None,
            response_time: None,
            connection_id: None,
            user_id: None,
            session_id: None,
        }
    }
    
    /// 创建交易API日志上下文
    pub fn trader(request_id: i32) -> Self {
        Self {
            api_type: "Trader".to_string(),
            request_id,
            error_id: None,
            error_msg: None,
            response_time: None,
            connection_id: None,
            user_id: None,
            session_id: None,
        }
    }
    
    /// 设置错误信息
    pub fn with_error(mut self, error_id: i32, error_msg: &str) -> Self {
        self.error_id = Some(error_id);
        self.error_msg = Some(error_msg.to_string());
        self
    }
    
    /// 设置响应时间
    pub fn with_response_time(mut self, duration: std::time::Duration) -> Self {
        self.response_time = Some(duration);
        self
    }
    
    /// 设置连接ID
    pub fn with_connection_id(mut self, connection_id: &str) -> Self {
        self.connection_id = Some(connection_id.to_string());
        self
    }
    
    /// 设置用户ID
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }
    
    /// 设置会话ID
    pub fn with_session_id(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }
    
    /// 转换为通用上下文
    pub fn to_log_context(&self, level: LogLevel, module: &str) -> LogContext {
        let mut context = LogContext::new(level, module);
        
        context.extra.insert("api_type".to_string(), self.api_type.clone().into());
        context.extra.insert("request_id".to_string(), self.request_id.into());
        
        if let Some(error_id) = self.error_id {
            context.extra.insert("error_id".to_string(), error_id.into());
        }
        
        if let Some(error_msg) = &self.error_msg {
            context.extra.insert("error_msg".to_string(), error_msg.clone().into());
        }
        
        if let Some(response_time) = &self.response_time {
            context.extra.insert("response_time_ms".to_string(), (response_time.as_millis() as u64).into());
        }
        
        if let Some(connection_id) = &self.connection_id {
            context.extra.insert("connection_id".to_string(), connection_id.clone().into());
            context.session_id = Some(connection_id.clone());
        }
        
        if let Some(user_id) = &self.user_id {
            context.user_id = Some(user_id.clone());
        }
        
        if let Some(session_id) = &self.session_id {
            context.session_id = Some(session_id.clone());
        }
        
        context
    }
}

/// 交易专用日志上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingLogContext {
    pub account_id: String,
    pub instrument_id: String,
    pub order_ref: Option<String>,
    pub order_sys_id: Option<String>,
    pub direction: Option<String>,      // "BUY" | "SELL"
    pub offset_flag: Option<String>,    // "OPEN" | "CLOSE" | "CLOSE_TODAY" | "CLOSE_YESTERDAY"
    pub price: Option<f64>,
    pub volume: Option<i32>,
    pub order_status: Option<String>,
    pub trade_id: Option<String>,
    pub trade_price: Option<f64>,
    pub trade_volume: Option<i32>,
    pub commission: Option<f64>,
    pub error_id: Option<i32>,
    pub error_msg: Option<String>,
}

impl TradingLogContext {
    /// 创建订单日志上下文
    pub fn order(account_id: &str, instrument_id: &str) -> Self {
        Self {
            account_id: account_id.to_string(),
            instrument_id: instrument_id.to_string(),
            order_ref: None,
            order_sys_id: None,
            direction: None,
            offset_flag: None,
            price: None,
            volume: None,
            order_status: None,
            trade_id: None,
            trade_price: None,
            trade_volume: None,
            commission: None,
            error_id: None,
            error_msg: None,
        }
    }
    
    /// 创建成交日志上下文
    pub fn trade(account_id: &str, instrument_id: &str) -> Self {
        Self::order(account_id, instrument_id)
    }
    
    /// 设置订单信息
    pub fn with_order_info(
        mut self,
        order_ref: &str,
        direction: &str,
        offset_flag: &str,
        price: f64,
        volume: i32,
    ) -> Self {
        self.order_ref = Some(order_ref.to_string());
        self.direction = Some(direction.to_string());
        self.offset_flag = Some(offset_flag.to_string());
        self.price = Some(price);
        self.volume = Some(volume);
        self
    }
    
    /// 设置订单系统ID
    pub fn with_order_sys_id(mut self, order_sys_id: &str) -> Self {
        self.order_sys_id = Some(order_sys_id.to_string());
        self
    }
    
    /// 设置订单状态
    pub fn with_order_status(mut self, status: &str) -> Self {
        self.order_status = Some(status.to_string());
        self
    }
    
    /// 设置成交信息
    pub fn with_trade_info(
        mut self,
        trade_id: &str,
        trade_price: f64,
        trade_volume: i32,
        commission: Option<f64>,
    ) -> Self {
        self.trade_id = Some(trade_id.to_string());
        self.trade_price = Some(trade_price);
        self.trade_volume = Some(trade_volume);
        self.commission = commission;
        self
    }
    
    /// 设置错误信息
    pub fn with_error(mut self, error_id: i32, error_msg: &str) -> Self {
        self.error_id = Some(error_id);
        self.error_msg = Some(error_msg.to_string());
        self
    }
    
    /// 转换为通用上下文
    pub fn to_log_context(&self, level: LogLevel, module: &str) -> LogContext {
        let mut context = LogContext::new(level, module);
        
        // 添加交易相关字段
        context.extra.insert("account_id".to_string(), self.account_id.clone().into());
        context.extra.insert("instrument_id".to_string(), self.instrument_id.clone().into());
        
        if let Some(order_ref) = &self.order_ref {
            context.extra.insert("order_ref".to_string(), order_ref.clone().into());
        }
        
        if let Some(order_sys_id) = &self.order_sys_id {
            context.extra.insert("order_sys_id".to_string(), order_sys_id.clone().into());
        }
        
        if let Some(direction) = &self.direction {
            context.extra.insert("direction".to_string(), direction.clone().into());
        }
        
        if let Some(offset_flag) = &self.offset_flag {
            context.extra.insert("offset_flag".to_string(), offset_flag.clone().into());
        }
        
        if let Some(price) = self.price {
            context.extra.insert("price".to_string(), price.into());
        }
        
        if let Some(volume) = self.volume {
            context.extra.insert("volume".to_string(), volume.into());
        }
        
        if let Some(order_status) = &self.order_status {
            context.extra.insert("order_status".to_string(), order_status.clone().into());
        }
        
        if let Some(trade_id) = &self.trade_id {
            context.extra.insert("trade_id".to_string(), trade_id.clone().into());
        }
        
        if let Some(trade_price) = self.trade_price {
            context.extra.insert("trade_price".to_string(), trade_price.into());
        }
        
        if let Some(trade_volume) = self.trade_volume {
            context.extra.insert("trade_volume".to_string(), trade_volume.into());
        }
        
        if let Some(commission) = self.commission {
            context.extra.insert("commission".to_string(), commission.into());
        }
        
        if let Some(error_id) = self.error_id {
            context.extra.insert("error_id".to_string(), error_id.into());
        }
        
        if let Some(error_msg) = &self.error_msg {
            context.extra.insert("error_msg".to_string(), error_msg.clone().into());
        }
        
        context
    }
}

/// 行情数据专用日志上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataLogContext {
    pub instrument_id: String,
    pub exchange_id: Option<String>,
    pub last_price: Option<f64>,
    pub volume: Option<i32>,
    pub turnover: Option<f64>,
    pub open_interest: Option<i32>,
    pub update_time: Option<String>,
    pub update_millisec: Option<i32>,
    pub bid_price1: Option<f64>,
    pub ask_price1: Option<f64>,
    pub bid_volume1: Option<i32>,
    pub ask_volume1: Option<i32>,
}

impl MarketDataLogContext {
    /// 创建行情数据日志上下文
    pub fn new(instrument_id: &str) -> Self {
        Self {
            instrument_id: instrument_id.to_string(),
            exchange_id: None,
            last_price: None,
            volume: None,
            turnover: None,
            open_interest: None,
            update_time: None,
            update_millisec: None,
            bid_price1: None,
            ask_price1: None,
            bid_volume1: None,
            ask_volume1: None,
        }
    }
    
    /// 设置基础行情信息
    pub fn with_basic_info(
        mut self,
        last_price: f64,
        volume: i32,
        turnover: f64,
        open_interest: i32,
    ) -> Self {
        self.last_price = Some(last_price);
        self.volume = Some(volume);
        self.turnover = Some(turnover);
        self.open_interest = Some(open_interest);
        self
    }
    
    /// 设置更新时间
    pub fn with_update_time(mut self, update_time: &str, update_millisec: i32) -> Self {
        self.update_time = Some(update_time.to_string());
        self.update_millisec = Some(update_millisec);
        self
    }
    
    /// 设置盘口信息
    pub fn with_depth_info(
        mut self,
        bid_price1: f64,
        ask_price1: f64,
        bid_volume1: i32,
        ask_volume1: i32,
    ) -> Self {
        self.bid_price1 = Some(bid_price1);
        self.ask_price1 = Some(ask_price1);
        self.bid_volume1 = Some(bid_volume1);
        self.ask_volume1 = Some(ask_volume1);
        self
    }
    
    /// 设置交易所ID
    pub fn with_exchange_id(mut self, exchange_id: &str) -> Self {
        self.exchange_id = Some(exchange_id.to_string());
        self
    }
    
    /// 转换为通用上下文
    pub fn to_log_context(&self, level: LogLevel, module: &str) -> LogContext {
        let mut context = LogContext::new(level, module);
        
        // 添加行情数据相关字段
        context.extra.insert("instrument_id".to_string(), self.instrument_id.clone().into());
        
        if let Some(exchange_id) = &self.exchange_id {
            context.extra.insert("exchange_id".to_string(), exchange_id.clone().into());
        }
        
        if let Some(last_price) = self.last_price {
            context.extra.insert("last_price".to_string(), last_price.into());
        }
        
        if let Some(volume) = self.volume {
            context.extra.insert("volume".to_string(), volume.into());
        }
        
        if let Some(turnover) = self.turnover {
            context.extra.insert("turnover".to_string(), turnover.into());
        }
        
        if let Some(open_interest) = self.open_interest {
            context.extra.insert("open_interest".to_string(), open_interest.into());
        }
        
        if let Some(update_time) = &self.update_time {
            context.extra.insert("update_time".to_string(), update_time.clone().into());
        }
        
        if let Some(update_millisec) = self.update_millisec {
            context.extra.insert("update_millisec".to_string(), update_millisec.into());
        }
        
        if let Some(bid_price1) = self.bid_price1 {
            context.extra.insert("bid_price1".to_string(), bid_price1.into());
        }
        
        if let Some(ask_price1) = self.ask_price1 {
            context.extra.insert("ask_price1".to_string(), ask_price1.into());
        }
        
        if let Some(bid_volume1) = self.bid_volume1 {
            context.extra.insert("bid_volume1".to_string(), bid_volume1.into());
        }
        
        if let Some(ask_volume1) = self.ask_volume1 {
            context.extra.insert("ask_volume1".to_string(), ask_volume1.into());
        }
        
        context
    }
}

/// 性能监控日志上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceLogContext {
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
    pub operation: Option<String>,
    pub duration_ms: Option<u64>,
    pub memory_usage_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
    pub thread_count: Option<usize>,
    pub latency_percentiles: Option<HashMap<String, f64>>,
}

impl PerformanceLogContext {
    /// 创建性能指标日志上下文
    pub fn metric(name: &str, value: f64, unit: &str) -> Self {
        Self {
            metric_name: name.to_string(),
            value,
            unit: unit.to_string(),
            operation: None,
            duration_ms: None,
            memory_usage_mb: None,
            cpu_usage_percent: None,
            thread_count: None,
            latency_percentiles: None,
        }
    }
    
    /// 设置操作信息
    pub fn with_operation(mut self, operation: &str, duration_ms: u64) -> Self {
        self.operation = Some(operation.to_string());
        self.duration_ms = Some(duration_ms);
        self
    }
    
    /// 设置系统资源信息
    pub fn with_system_info(
        mut self,
        memory_usage_mb: f64,
        cpu_usage_percent: f64,
        thread_count: usize,
    ) -> Self {
        self.memory_usage_mb = Some(memory_usage_mb);
        self.cpu_usage_percent = Some(cpu_usage_percent);
        self.thread_count = Some(thread_count);
        self
    }
    
    /// 设置延迟百分位数
    pub fn with_latency_percentiles(mut self, percentiles: HashMap<String, f64>) -> Self {
        self.latency_percentiles = Some(percentiles);
        self
    }
    
    /// 转换为通用上下文
    pub fn to_log_context(&self, level: LogLevel, module: &str) -> LogContext {
        let mut context = LogContext::new(level, module);
        
        // 添加性能相关字段
        context.extra.insert("metric_name".to_string(), self.metric_name.clone().into());
        context.extra.insert("value".to_string(), self.value.into());
        context.extra.insert("unit".to_string(), self.unit.clone().into());
        
        if let Some(operation) = &self.operation {
            context.extra.insert("operation".to_string(), operation.clone().into());
        }
        
        if let Some(duration_ms) = self.duration_ms {
            context.extra.insert("duration_ms".to_string(), duration_ms.into());
        }
        
        if let Some(memory_usage_mb) = self.memory_usage_mb {
            context.extra.insert("memory_usage_mb".to_string(), memory_usage_mb.into());
        }
        
        if let Some(cpu_usage_percent) = self.cpu_usage_percent {
            context.extra.insert("cpu_usage_percent".to_string(), cpu_usage_percent.into());
        }
        
        if let Some(thread_count) = self.thread_count {
            context.extra.insert("thread_count".to_string(), thread_count.into());
        }
        
        if let Some(percentiles) = &self.latency_percentiles {
            context.extra.insert("latency_percentiles".to_string(), 
                serde_json::to_value(percentiles).unwrap_or_default());
        }
        
        context
    }
}

/// 上下文构建器
pub struct ContextBuilder {
    context: LogContext,
}

impl ContextBuilder {
    /// 创建新的上下文构建器
    pub fn new(level: LogLevel, module: &str) -> Self {
        Self {
            context: LogContext::new(level, module),
        }
    }
    
    /// 设置请求ID
    pub fn request_id(mut self, request_id: &str) -> Self {
        self.context.request_id = Some(request_id.to_string());
        self
    }
    
    /// 设置用户ID
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.context.user_id = Some(user_id.to_string());
        self
    }
    
    /// 设置会话ID
    pub fn session_id(mut self, session_id: &str) -> Self {
        self.context.session_id = Some(session_id.to_string());
        self
    }
    
    /// 添加字段
    pub fn field<T: serde::Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.context.extra.insert(key.to_string(), json_value);
        }
        self
    }
    
    /// 批量添加字段
    pub fn fields(mut self, fields: HashMap<String, serde_json::Value>) -> Self {
        self.context.extra.extend(fields);
        self
    }
    
    /// 构建上下文
    pub fn build(self) -> LogContext {
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_context() {
        let context = LogContext::new(LogLevel::Info, "test_module")
            .with_request_id("req_123")
            .with_user_id("user_456")
            .with_session_id("sess_789")
            .with_field("custom_field", "custom_value");
        
        assert_eq!(context.level, LogLevel::Info);
        assert_eq!(context.module, "test_module");
        assert_eq!(context.request_id, Some("req_123".to_string()));
        assert_eq!(context.user_id, Some("user_456".to_string()));
        assert_eq!(context.session_id, Some("sess_789".to_string()));
        assert!(context.extra.contains_key("custom_field"));
    }
    
    #[test]
    fn test_ctp_log_context() {
        let ctp_context = CtpLogContext::market_data(123)
            .with_error(1001, "连接失败")
            .with_response_time(std::time::Duration::from_millis(250))
            .with_user_id("test_user");
        
        assert_eq!(ctp_context.api_type, "MD");
        assert_eq!(ctp_context.request_id, 123);
        assert_eq!(ctp_context.error_id, Some(1001));
        assert_eq!(ctp_context.error_msg, Some("连接失败".to_string()));
        
        let log_context = ctp_context.to_log_context(LogLevel::Error, "ctp_module");
        assert_eq!(log_context.level, LogLevel::Error);
        assert_eq!(log_context.module, "ctp_module");
        assert!(log_context.extra.contains_key("api_type"));
        assert!(log_context.extra.contains_key("error_id"));
    }
    
    #[test]
    fn test_trading_log_context() {
        let trading_context = TradingLogContext::order("account123", "rb2405")
            .with_order_info("order_001", "BUY", "OPEN", 3850.0, 1)
            .with_order_sys_id("sys_001")
            .with_trade_info("trade_001", 3855.0, 1, Some(10.5));
        
        assert_eq!(trading_context.account_id, "account123");
        assert_eq!(trading_context.instrument_id, "rb2405");
        assert_eq!(trading_context.direction, Some("BUY".to_string()));
        assert_eq!(trading_context.price, Some(3850.0));
        assert_eq!(trading_context.trade_price, Some(3855.0));
        assert_eq!(trading_context.commission, Some(10.5));
        
        let log_context = trading_context.to_log_context(LogLevel::Info, "trading_module");
        assert!(log_context.extra.contains_key("account_id"));
        assert!(log_context.extra.contains_key("direction"));
        assert!(log_context.extra.contains_key("trade_price"));
    }
    
    #[test]
    fn test_market_data_log_context() {
        let md_context = MarketDataLogContext::new("rb2405")
            .with_basic_info(3850.0, 12345, 47520000.0, 150000)
            .with_depth_info(3849.0, 3851.0, 10, 8)
            .with_update_time("09:30:15", 500)
            .with_exchange_id("SHFE");
        
        assert_eq!(md_context.instrument_id, "rb2405");
        assert_eq!(md_context.last_price, Some(3850.0));
        assert_eq!(md_context.bid_price1, Some(3849.0));
        assert_eq!(md_context.ask_price1, Some(3851.0));
        assert_eq!(md_context.exchange_id, Some("SHFE".to_string()));
        
        let log_context = md_context.to_log_context(LogLevel::Debug, "market_data_module");
        assert!(log_context.extra.contains_key("instrument_id"));
        assert!(log_context.extra.contains_key("last_price"));
        assert!(log_context.extra.contains_key("exchange_id"));
    }
    
    #[test]
    fn test_performance_log_context() {
        let mut percentiles = HashMap::new();
        percentiles.insert("p50".to_string(), 10.5);
        percentiles.insert("p95".to_string(), 50.2);
        percentiles.insert("p99".to_string(), 100.8);
        
        let perf_context = PerformanceLogContext::metric("latency", 25.5, "ms")
            .with_operation("order_submission", 1500)
            .with_system_info(256.5, 15.2, 12)
            .with_latency_percentiles(percentiles);
        
        assert_eq!(perf_context.metric_name, "latency");
        assert_eq!(perf_context.value, 25.5);
        assert_eq!(perf_context.unit, "ms");
        assert_eq!(perf_context.memory_usage_mb, Some(256.5));
        assert_eq!(perf_context.thread_count, Some(12));
        
        let log_context = perf_context.to_log_context(LogLevel::Info, "performance_module");
        assert!(log_context.extra.contains_key("metric_name"));
        assert!(log_context.extra.contains_key("memory_usage_mb"));
        assert!(log_context.extra.contains_key("latency_percentiles"));
    }
    
    #[test]
    fn test_context_builder() {
        let mut fields = HashMap::new();
        fields.insert("field1".to_string(), serde_json::Value::String("value1".to_string()));
        fields.insert("field2".to_string(), serde_json::Value::Number(42.into()));
        
        let context = ContextBuilder::new(LogLevel::Warn, "builder_module")
            .request_id("req_builder")
            .user_id("user_builder")
            .session_id("sess_builder")
            .field("custom", "builder_value")
            .fields(fields)
            .build();
        
        assert_eq!(context.level, LogLevel::Warn);
        assert_eq!(context.module, "builder_module");
        assert_eq!(context.request_id, Some("req_builder".to_string()));
        assert!(context.extra.contains_key("custom"));
        assert!(context.extra.contains_key("field1"));
        assert!(context.extra.contains_key("field2"));
    }
    
    #[test]
    fn test_request_id_generation() {
        let id1 = LogContext::generate_request_id();
        let id2 = LogContext::generate_request_id();
        
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID v4 长度
        assert_eq!(id2.len(), 36);
    }
}
// CTP 交易组件模块
// 基于 ctp2rs 库的高级封装

pub mod client;
pub mod config;
pub mod config_manager;
pub mod error;
pub mod events;
pub mod models;
pub mod ffi;
pub mod ctp_sys;
pub mod logger;
pub mod spi;
pub mod utils;
pub mod market_data_manager;
pub mod subscription_manager;
pub mod market_data_service;
pub mod order_manager;
pub mod trading_service;
pub mod account_service;
pub mod position_manager;
pub mod settlement_manager;
pub mod query_service;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod production_config_test;

#[cfg(test)]
mod simple_production_test;

pub use client::{CtpClient, ClientState, ConnectionStats, HealthStatus, ConfigInfo};
pub use config::{CtpConfig, Environment};
pub use config_manager::{ConfigManager, ExtendedCtpConfig};
pub use error::CtpError;
pub use events::{CtpEvent, EventHandler, EventListener, DefaultEventListener};
pub use logger::{LoggerManager, PerformanceMonitor};
pub use models::*;
pub use spi::{MdSpiImpl, TraderSpiImpl};
pub use utils::{DataConverter, gb18030_to_utf8, utf8_to_gb18030};
pub use market_data_manager::{MarketDataManager, MarketDataFilter, MarketDataStats, PriceChangeFilter, VolumeFilter};
pub use subscription_manager::{SubscriptionManager, SubscriptionInfo, SubscriptionStatus, SubscriptionConfig, SubscriptionStats, SubscriptionPriority};
pub use market_data_service::{MarketDataService, ServiceState, ServiceStats};
pub use order_manager::{OrderManager, OrderInfo, OrderStats};
pub use trading_service::{TradingService, TradingStats};
pub use account_service::{AccountService, FundStats, RiskMetrics, RiskStatus, AccountSummary};
pub use position_manager::{PositionManager, PositionDetail, PositionStats};
pub use settlement_manager::{SettlementManager, Settlement, SettlementSummary, SettlementReport};
pub use query_service::{QueryService, QueryType, QueryState, QueryCache, QueryOptions};

/// CTP 组件版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 初始化 CTP 组件
pub fn init() -> Result<(), CtpError> {
    tracing::info!("初始化 CTP 交易组件 v{}", VERSION);
    
    // 检查 CTP 动态库是否可用
    ffi::check_ctp_libraries()?;
    
    tracing::info!("CTP 交易组件初始化完成");
    Ok(())
}

/// 使用配置初始化 CTP 组件
pub fn init_with_config(config: &ExtendedCtpConfig) -> Result<(), CtpError> {
    // 初始化日志系统
    LoggerManager::init(
        &config.logging.level,
        Some(std::path::Path::new(&config.logging.file_path)),
        config.logging.console,
        config.ctp.environment,
    )?;
    
    // 初始化组件
    init()?;
    
    tracing::info!("使用配置初始化 CTP 组件完成");
    tracing::info!("环境: {:?}", config.ctp.environment);
    tracing::info!("经纪商: {}", config.ctp.broker_id);
    
    Ok(())
}
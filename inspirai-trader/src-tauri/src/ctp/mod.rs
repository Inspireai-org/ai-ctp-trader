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
pub mod services;
pub mod market_data_manager;
pub mod subscription_manager;
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

#[cfg(test)]
mod test_path_issue;

#[cfg(test)]
mod test_serde;

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
pub use services::market_data_service::MarketDataService;
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
    
    // 尝试使用默认配置检查动态库
    let default_config = CtpConfig::default();
    if let (Some(md_path), Some(td_path)) = (&default_config.md_dynlib_path, &default_config.td_dynlib_path) {
        if md_path.exists() && td_path.exists() {
            ffi::check_ctp_libraries(md_path, td_path)?;
        } else {
            tracing::warn!("CTP 动态库文件不存在，跳过检查");
        }
    } else {
        tracing::warn!("未设置 CTP 动态库路径，跳过检查");
    }
    
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
    
    // 使用配置中的库路径检查动态库
    if let (Some(md_path), Some(td_path)) = (&config.ctp.md_dynlib_path, &config.ctp.td_dynlib_path) {
        ffi::check_ctp_libraries(md_path, td_path)?;
    } else {
        tracing::warn!("配置中未设置 CTP 动态库路径，跳过检查");
    }
    
    tracing::info!("使用配置初始化 CTP 组件完成");
    tracing::info!("环境: {:?}", config.ctp.environment);
    tracing::info!("经纪商: {}", config.ctp.broker_id);
    
    Ok(())
}
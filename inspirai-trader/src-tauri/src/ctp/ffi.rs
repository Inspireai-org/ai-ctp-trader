use crate::ctp::CtpError;

// 使用 ctp2rs 提供的官方 API，严禁自定义 FFI 绑定
use ctp2rs::v1alpha1::{MdApi, TraderApi};
use std::sync::Arc;


/// CTP API 管理器
/// 
/// 使用 ctp2rs 提供的官方 API，严禁自定义 FFI 实现
/// 这个结构体只是对 ctp2rs API 的简单封装，不包含任何自定义的 FFI 逻辑
pub struct CtpApiManager {
    md_api: Option<Arc<MdApi>>,
    trader_api: Option<Arc<TraderApi>>,
    md_spi: Option<Box<dyn ctp2rs::v1alpha1::MdSpi>>,
    trader_spi: Option<Box<dyn ctp2rs::v1alpha1::TraderSpi>>,
}

impl CtpApiManager {
    /// 创建新的 CTP API 管理器
    pub fn new() -> Result<Self, CtpError> {
        Ok(Self {
            md_api: None,
            trader_api: None,
            md_spi: None,
            trader_spi: None,
        })
    }

    /// 创建行情 API 实例
    /// 使用 ctp2rs 官方 API，严禁自定义实现
    pub fn create_md_api(&mut self, flow_path: &str) -> Result<(), CtpError> {
        tracing::info!("使用 ctp2rs 创建行情 API，流文件路径: {}", flow_path);
        
        // 创建流文件目录
        if let Err(e) = std::fs::create_dir_all(flow_path) {
            return Err(CtpError::IoError(e));
        }
        
        // 使用 ctp2rs 官方 API 创建行情 API 实例
        // 需要指定动态库路径，这里使用默认路径
        let dynlib_path = std::env::current_dir()
            .unwrap_or_default()
            .join("lib/macos/6.7.7/product/thostmduserapi_se.framework/thostmduserapi_se");
        
        let api = MdApi::create_api(
            dynlib_path,
            flow_path,
            false, // is_using_udp
            false, // is_multicast
        );
        
        self.md_api = Some(Arc::new(api));
        tracing::info!("行情 API 创建成功");
        Ok(())
    }

    /// 创建交易 API 实例
    /// 使用 ctp2rs 官方 API，严禁自定义实现
    pub fn create_trader_api(&mut self, flow_path: &str) -> Result<(), CtpError> {
        tracing::info!("使用 ctp2rs 创建交易 API，流文件路径: {}", flow_path);
        
        // 创建流文件目录
        if let Err(e) = std::fs::create_dir_all(flow_path) {
            return Err(CtpError::IoError(e));
        }
        
        // 使用 ctp2rs 官方 API 创建交易 API 实例
        // 需要指定动态库路径，这里使用默认路径
        let dynlib_path = std::env::current_dir()
            .unwrap_or_default()
            .join("lib/macos/6.7.7/product/thosttraderapi_se.framework/thosttraderapi_se");
        
        let api = TraderApi::create_api(dynlib_path, flow_path);
        
        self.trader_api = Some(Arc::new(api));
        tracing::info!("交易 API 创建成功");
        Ok(())
    }

    /// 获取行情 API 实例
    pub fn get_md_api(&self) -> Option<Arc<MdApi>> {
        self.md_api.clone()
    }

    /// 获取交易 API 实例
    pub fn get_trader_api(&self) -> Option<Arc<TraderApi>> {
        self.trader_api.clone()
    }

    /// 检查行情 API 是否已创建
    pub fn is_md_api_ready(&self) -> bool {
        self.md_api.is_some()
    }

    /// 检查交易 API 是否已创建
    pub fn is_trader_api_ready(&self) -> bool {
        self.trader_api.is_some()
    }

    /// 注册行情 SPI
    pub fn register_md_spi(&mut self, spi: Box<dyn ctp2rs::v1alpha1::MdSpi>) -> Result<(), CtpError> {
        tracing::info!("注册行情 SPI");
        
        if let Some(md_api) = &self.md_api {
            // ctp2rs 的 register_spi 方法需要原始指针
            let spi_ptr = Box::into_raw(spi);
            
            md_api.register_spi(spi_ptr);
            self.md_spi = Some(unsafe { Box::from_raw(spi_ptr) });
            tracing::info!("行情 SPI 注册成功");
            Ok(())
        } else {
            return Err(CtpError::StateError("行情 API 未创建".to_string()));
        }
    }

    /// 注册交易 SPI
    pub fn register_trader_spi(&mut self, spi: Box<dyn ctp2rs::v1alpha1::TraderSpi>) -> Result<(), CtpError> {
        tracing::info!("注册交易 SPI");
        
        if let Some(trader_api) = &self.trader_api {
            // ctp2rs 的 register_spi 方法需要原始指针
            let spi_ptr = Box::into_raw(spi);
            
            trader_api.register_spi(spi_ptr);
            self.trader_spi = Some(unsafe { Box::from_raw(spi_ptr) });
            tracing::info!("交易 SPI 注册成功");
            Ok(())
        } else {
            return Err(CtpError::StateError("交易 API 未创建".to_string()));
        }
    }
}

/// 检查 CTP 动态库是否可用
/// 使用 ctp2rs 的库检查机制，严禁自定义实现
pub fn check_ctp_libraries() -> Result<(), CtpError> {
    tracing::info!("检查 CTP 动态库可用性");
    
    // 尝试创建临时的 API 实例来验证库是否可用
    let temp_flow_path = std::env::temp_dir().join("ctp_lib_check");
    
    // 检查行情库
    let md_dynlib_path = std::env::current_dir()
        .unwrap_or_default()
        .join("lib/macos/6.7.7/product/thostmduserapi_se.framework/thostmduserapi_se");
    
    let _md_api = MdApi::create_api(
        md_dynlib_path,
        temp_flow_path.to_str().unwrap_or("/tmp/ctp_check"),
        false,
        false,
    );
    tracing::info!("行情库检查通过");
    
    // 检查交易库
    let td_dynlib_path = std::env::current_dir()
        .unwrap_or_default()
        .join("lib/macos/6.7.7/product/thosttraderapi_se.framework/thosttraderapi_se");
    
    let _trader_api = TraderApi::create_api(
        td_dynlib_path,
        temp_flow_path.to_str().unwrap_or("/tmp/ctp_check"),
    );
    tracing::info!("交易库检查通过");
    
    // 清理临时目录
    let _ = std::fs::remove_dir_all(&temp_flow_path);
    
    tracing::info!("CTP 动态库检查通过");
    Ok(())
}
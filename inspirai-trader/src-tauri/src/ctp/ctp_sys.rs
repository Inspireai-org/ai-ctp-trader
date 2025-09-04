// CTP API 系统级绑定
// 这个模块包含了对 CTP C++ API 的直接绑定

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// 尝试包含 bindgen 生成的绑定
// 如果生成失败，使用手动绑定
#[cfg(all(target_os = "macos", feature = "use_bindgen"))]
include!(concat!(env!("OUT_DIR"), "/ctp_bindings.rs"));

// 如果 bindgen 生成失败，使用手动定义的基本接口
#[cfg(not(feature = "use_bindgen"))]
pub mod manual_bindings {
    use std::os::raw::{c_char, c_int, c_void};
    
    // CTP API 版本
    pub const THOST_FTDC_VERSION: &str = "6.7.7";
    
    // 基本的 API 函数指针类型
    pub type CreateFtdcMdApi = unsafe extern "C" fn(pszFlowPath: *const c_char) -> *mut c_void;
    pub type CreateFtdcTraderApi = unsafe extern "C" fn(pszFlowPath: *const c_char) -> *mut c_void;
    
    // 响应信息结构
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct CThostFtdcRspInfoField {
        pub ErrorID: c_int,
        pub ErrorMsg: [c_char; 81],
    }
    
    // 用户登录请求
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct CThostFtdcReqUserLoginField {
        pub TradingDay: [c_char; 9],
        pub BrokerID: [c_char; 11],
        pub UserID: [c_char; 16],
        pub Password: [c_char; 41],
        pub UserProductInfo: [c_char; 11],
        pub InterfaceProductInfo: [c_char; 11],
        pub ProtocolInfo: [c_char; 11],
        pub MacAddress: [c_char; 21],
        pub OneTimePassword: [c_char; 41],
        pub ClientIPAddress: [c_char; 16],
        pub LoginRemark: [c_char; 36],
        pub ClientIPPort: c_int,
    }
    
    // 用户登录响应
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct CThostFtdcRspUserLoginField {
        pub TradingDay: [c_char; 9],
        pub LoginTime: [c_char; 9],
        pub BrokerID: [c_char; 11],
        pub UserID: [c_char; 16],
        pub SystemName: [c_char; 41],
        pub FrontID: c_int,
        pub SessionID: c_int,
        pub MaxOrderRef: [c_char; 13],
        pub SHFETime: [c_char; 9],
        pub DCETime: [c_char; 9],
        pub CZCETime: [c_char; 9],
        pub FFEXTime: [c_char; 9],
        pub INETime: [c_char; 9],
    }
}

// 导出手动绑定（用于没有 bindgen 的情况）
#[cfg(not(feature = "use_bindgen"))]
pub use manual_bindings::*;
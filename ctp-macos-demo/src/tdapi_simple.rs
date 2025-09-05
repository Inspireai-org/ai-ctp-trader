#![allow(unused_variables)]
use std::{sync::Arc, thread, time::Duration};

use ctp2rs::{
    ffi::{gb18030_cstr_i8_to_str, AssignFromString, WrapToString},
    print_rsp_info,
    v1alpha1::{
        CThostFtdcReqUserLoginField, CThostFtdcRspInfoField,
        CThostFtdcRspUserLoginField, CThostFtdcReqAuthenticateField,
        CThostFtdcRspAuthenticateField,
        TraderApi, TraderSpi,
    },
};

use crate::CtpConfig;

pub struct TraderSpiImpl {
    pub(crate) tdapi: Arc<TraderApi>,
    pub(crate) config: CtpConfig,
}

impl TraderSpi for TraderSpiImpl {
    fn on_front_connected(&mut self) {
        println!("[TD] 交易前置服务器连接成功");
        
        // 发送认证请求
        let mut auth_req = CThostFtdcReqAuthenticateField::default();
        auth_req.BrokerID.assign_from_str(&self.config.broker_id);
        auth_req.UserID.assign_from_str(&self.config.user_id);
        auth_req.AppID.assign_from_str(&self.config.app_id);
        auth_req.AuthCode.assign_from_str(&self.config.auth_code);
        
        let ret = self.tdapi.req_authenticate(&mut auth_req, 1);
        println!("[TD] 发送认证请求: {}", ret);
        println!("[TD] 认证参数:");
        println!("  BrokerID: {}", self.config.broker_id);
        println!("  UserID: {}", self.config.user_id);
        println!("  AppID: {}", self.config.app_id);
        println!("  AuthCode: {}", self.config.auth_code);
    }

    fn on_front_disconnected(&mut self, reason: i32) {
        println!("[TD] 交易前置服务器连接断开，原因: {}", reason);
    }

    fn on_rsp_authenticate(
        &mut self,
        rsp_authenticate_field: Option<&CThostFtdcRspAuthenticateField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool,
    ) {
        println!("[TD] 收到认证响应");
        print_rsp_info!(rsp_info);
        
        if let Some(auth_field) = rsp_authenticate_field {
            println!("[TD] 认证成功:");
            println!("  经纪商: {}", auth_field.BrokerID.to_string());
            println!("  用户: {}", auth_field.UserID.to_string());
            println!("  应用ID: {}", auth_field.AppID.to_string());
        }

        // 认证成功后不继续登录，只测试认证流程
        println!("[TD] 认证测试完成");
    }

    fn on_rsp_error(
        &mut self,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool,
    ) {
        print_rsp_info!(rsp_info);
        if let Some(error) = rsp_info {
            println!("[TD] 错误响应: {} - {}", 
                error.ErrorID, 
                gb18030_cstr_i8_to_str(&error.ErrorMsg).unwrap_or("Unknown error".into())
            );
        }
    }
}

pub fn run_td_simple(config: CtpConfig) {
    println!("\n=== 启动交易接口（仅测试认证） ===");
    println!("动态库路径: {}", config.td_dynlib_path.to_string_lossy());
    println!("前置地址: {}", config.td_front_address);
    println!("经纪商: {}", config.broker_id);
    println!("用户: {}", config.user_id);
    println!("AppID: {}", config.app_id);
    println!("AuthCode: {}", config.auth_code);

    // 确保流文件目录存在
    std::fs::create_dir_all("ctp_td_flow").expect("Failed to create flow directory");
    
    // 获取绝对路径
    let dynlib_path = std::fs::canonicalize(&config.td_dynlib_path)
        .unwrap_or_else(|_| config.td_dynlib_path.clone());
    
    println!("使用动态库: {}", dynlib_path.display());

    // 创建 API 实例
    let tdapi = TraderApi::create_api(&dynlib_path, "ctp_td_flow");
    let tdapi = Arc::new(tdapi);

    // 克隆配置数据
    let front_address = config.td_front_address.clone();

    // 创建 SPI 实例
    let td_spi = TraderSpiImpl {
        tdapi: Arc::clone(&tdapi),
        config,
    };

    // 注册 SPI
    let mut td_spi = td_spi;
    tdapi.register_spi(&mut td_spi);

    // 注册前置地址
    tdapi.register_front(&front_address);

    // 初始化连接
    tdapi.init();

    println!("[TD] 等待认证结果...");
    
    // 等待一段时间以观察认证结果
    thread::sleep(Duration::from_secs(10));
    
    println!("[TD] 测试结束");
}
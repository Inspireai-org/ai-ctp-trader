#![allow(unused_variables)]
use std::{sync::Arc, thread, time::Duration};

use ctp2rs::{
    ffi::{gb18030_cstr_i8_to_str, AssignFromString, WrapToString},
    print_rsp_info,
    v1alpha1::{
        CThostFtdcDepthMarketDataField, CThostFtdcReqUserLoginField, CThostFtdcRspInfoField,
        CThostFtdcRspUserLoginField, CThostFtdcSpecificInstrumentField, MdApi, MdSpi,
    },
};

use crate::CtpConfig;

pub struct MdSpiImpl {
    pub(crate) mdapi: Arc<MdApi>,
    pub(crate) config: CtpConfig,
}

impl MdSpi for MdSpiImpl {
    fn on_front_connected(&mut self) {
        println!("[MD] 前置服务器连接成功");
        let mut req = CThostFtdcReqUserLoginField::default();
        req.UserID.assign_from_str(&self.config.user_id);
        req.Password.assign_from_str(&self.config.password);
        req.BrokerID.assign_from_str(&self.config.broker_id);
        
        let ret = self.mdapi.req_user_login(&mut req, 1);
        println!("[MD] 发送登录请求: {}", ret);
    }

    fn on_front_disconnected(&mut self, reason: i32) {
        println!("[MD] 前置服务器连接断开，原因: {}", reason);
    }

    fn on_rsp_user_login(
        &mut self,
        rsp_user_login: Option<&CThostFtdcRspUserLoginField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool,
    ) {
        print_rsp_info!(rsp_info);
        
        if let Some(login_field) = rsp_user_login {
            println!("[MD] 登录成功:");
            println!("  交易日: {}", login_field.TradingDay.to_string());
            println!("  登录时间: {}", login_field.LoginTime.to_string());
            println!("  经纪商: {}", login_field.BrokerID.to_string());
            println!("  用户: {}", login_field.UserID.to_string());
        }

        if is_last {
            // 订阅行情
            let instruments = vec![
                "ag2512".to_string(),
                "au2512".to_string(),
                "rb2510".to_string(),
            ];
            
            println!("[MD] 订阅合约: {:?}", instruments);
            self.mdapi.subscribe_market_data(&instruments);
        }
    }

    fn on_rsp_sub_market_data(
        &mut self,
        specific_instrument: Option<&CThostFtdcSpecificInstrumentField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool,
    ) {
        print_rsp_info!(rsp_info);
        
        if let Some(instrument) = specific_instrument {
            println!(
                "[MD] 订阅成功: {}",
                instrument.InstrumentID.to_string()
            );
        }
    }

    fn on_rtn_depth_market_data(
        &mut self,
        depth_market_data: Option<&CThostFtdcDepthMarketDataField>,
    ) {
        if let Some(data) = depth_market_data {
            println!(
                "[MD] 行情 {} {} {} | {} | 最新价: {} | 买一: {} @ {} | 卖一: {} @ {} | 成交量: {}",
                data.TradingDay.to_string(),
                data.UpdateTime.to_string(),
                data.UpdateMillisec,
                data.InstrumentID.to_string(),
                data.LastPrice,
                data.BidPrice1,
                data.BidVolume1,
                data.AskPrice1,
                data.AskVolume1,
                data.Volume,
            );
        }
    }

    fn on_rsp_error(
        &mut self,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool,
    ) {
        print_rsp_info!(rsp_info);
        if let Some(error) = rsp_info {
            println!("[MD] 错误响应: {} - {}", 
                error.ErrorID, 
                gb18030_cstr_i8_to_str(&error.ErrorMsg).unwrap_or("Unknown error".into())
            );
        }
    }
}

pub fn run_md(config: CtpConfig) {
    println!("\n=== 启动行情接口 ===");
    println!("动态库路径: {}", config.md_dynlib_path.to_string_lossy());
    println!("前置地址: {}", config.md_front_address);

    // 确保流文件目录存在
    std::fs::create_dir_all("ctp_md_flow").expect("Failed to create flow directory");
    
    // 获取绝对路径
    let dynlib_path = std::fs::canonicalize(&config.md_dynlib_path)
        .unwrap_or_else(|_| config.md_dynlib_path.clone());
    
    println!("使用动态库: {}", dynlib_path.display());

    // 创建 API 实例，使用正确的流文件路径
    let mdapi = MdApi::create_api(&dynlib_path, "ctp_md_flow", false, false);
    let mdapi = Arc::new(mdapi);

    // 克隆配置数据
    let front_address = config.md_front_address.clone();

    // 创建 SPI 实例
    let md_spi = MdSpiImpl {
        mdapi: Arc::clone(&mdapi),
        config,
    };

    // 注册 SPI
    let mut md_spi = md_spi;
    mdapi.register_spi(&mut md_spi);

    // 注册前置地址
    mdapi.register_front(&front_address);

    // 初始化连接
    mdapi.init();

    println!("[MD] 等待行情数据...");
    
    // 保持线程运行
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
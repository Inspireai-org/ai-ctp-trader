#![allow(unused_variables)]
use std::{sync::Arc, thread, time::Duration};

use ctp2rs::{
    ffi::{gb18030_cstr_i8_to_str, AssignFromString, WrapToString},
    print_rsp_info,
    v1alpha1::{
        CThostFtdcReqUserLoginField, CThostFtdcRspInfoField,
        CThostFtdcRspUserLoginField, CThostFtdcSettlementInfoConfirmField,
        CThostFtdcRspAuthenticateField, CThostFtdcReqAuthenticateField,
        CThostFtdcInputOrderField, CThostFtdcOrderField,
        CThostFtdcTradeField, CThostFtdcInputOrderActionField,
        TraderApi, TraderSpi,
        // 使用常量值代替枚举
        THOST_FTDC_D_Buy,
        THOST_FTDC_OF_Open,
        THOST_FTDC_HF_Speculation,
        THOST_FTDC_CC_Immediately,
        THOST_FTDC_FCC_NotForceClose,
        THOST_FTDC_TC_IOC,  // 修正为正确的名称
        THOST_FTDC_VC_CV,
        THOST_FTDC_OPT_LimitPrice,
    },
};

use crate::CtpConfig;

pub struct TraderSpiImpl {
    pub(crate) tdapi: Arc<TraderApi>,
    pub(crate) config: CtpConfig,
    pub(crate) front_id: i32,
    pub(crate) session_id: i32,
    pub(crate) order_ref: i32,
    pub(crate) is_logged_in: bool,
    pub(crate) is_settled: bool,
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
        print_rsp_info!(rsp_info);
        
        if let Some(auth_field) = rsp_authenticate_field {
            println!("[TD] 认证成功:");
            println!("  经纪商: {}", auth_field.BrokerID.to_string());
            println!("  用户: {}", auth_field.UserID.to_string());
        }

        // 认证成功后登录
        let mut req = CThostFtdcReqUserLoginField::default();
        req.UserID.assign_from_str(&self.config.user_id);
        req.Password.assign_from_str(&self.config.password);
        req.BrokerID.assign_from_str(&self.config.broker_id);
        
        // 使用空的系统信息
        let system_info = [0i8; 273];
        let ret = self.tdapi.req_user_login(&mut req, 2, 0, system_info);
        println!("[TD] 发送登录请求: {}", ret);
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
            println!("[TD] 登录成功:");
            println!("  交易日: {}", login_field.TradingDay.to_string());
            println!("  登录时间: {}", login_field.LoginTime.to_string());
            println!("  经纪商: {}", login_field.BrokerID.to_string());
            println!("  用户: {}", login_field.UserID.to_string());
            println!("  前置编号: {}", login_field.FrontID);
            println!("  会话编号: {}", login_field.SessionID);
            println!("  最大报单引用: {}", login_field.MaxOrderRef.to_string());
            
            // 保存会话信息
            self.front_id = login_field.FrontID;
            self.session_id = login_field.SessionID;
            // 解析最大报单引用
            if let Ok(max_ref) = login_field.MaxOrderRef.to_string().parse::<i32>() {
                self.order_ref = max_ref + 1;
            } else {
                self.order_ref = 1;
            }
            self.is_logged_in = true;
        }

        // 登录成功后确认结算单
        let mut req = CThostFtdcSettlementInfoConfirmField::default();
        req.BrokerID.assign_from_str(&self.config.broker_id);
        req.InvestorID.assign_from_str(&self.config.user_id);
        
        let ret = self.tdapi.req_settlement_info_confirm(&mut req, 3);
        println!("[TD] 确认结算单: {}", ret);
    }

    fn on_rsp_settlement_info_confirm(
        &mut self,
        settlement_info_confirm: Option<&CThostFtdcSettlementInfoConfirmField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool,
    ) {
        print_rsp_info!(rsp_info);
        
        if let Some(confirm_field) = settlement_info_confirm {
            println!("[TD] 结算单确认成功:");
            println!("  确认日期: {}", confirm_field.ConfirmDate.to_string());
            println!("  确认时间: {}", confirm_field.ConfirmTime.to_string());
            self.is_settled = true;
        }
        
        println!("[TD] 交易系统就绪，可以进行交易操作");
        
        // 发送测试订单
        self.send_test_order();
    }

    fn on_rsp_order_insert(
        &mut self,
        input_order: Option<&CThostFtdcInputOrderField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool,
    ) {
        println!("[TD] 收到报单响应");
        print_rsp_info!(rsp_info);
        
        if let Some(order) = input_order {
            println!("[TD] 报单信息:");
            println!("  合约: {}", order.InstrumentID.to_string());
            println!("  报单引用: {}", order.OrderRef.to_string());
            println!("  价格: {}", order.LimitPrice);
            println!("  数量: {}", order.VolumeTotalOriginal);
        }
    }

    fn on_rtn_order(&mut self, order: Option<&CThostFtdcOrderField>) {
        if let Some(order) = order {
            println!("[TD] 报单回报:");
            println!("  合约: {}", order.InstrumentID.to_string());
            println!("  报单引用: {}", order.OrderRef.to_string());
            println!("  报单状态: {}", order.OrderStatus as u8 as char);
            println!("  成交数量: {}", order.VolumeTraded);
            println!("  剩余数量: {}", order.VolumeTotal);
            println!("  价格: {}", order.LimitPrice);
            println!("  状态信息: {}", order.StatusMsg.to_string());
        }
    }

    fn on_rtn_trade(&mut self, trade: Option<&CThostFtdcTradeField>) {
        if let Some(trade) = trade {
            println!("[TD] 成交回报:");
            println!("  合约: {}", trade.InstrumentID.to_string());
            println!("  成交编号: {}", trade.TradeID.to_string());
            println!("  报单引用: {}", trade.OrderRef.to_string());
            println!("  成交价格: {}", trade.Price);
            println!("  成交数量: {}", trade.Volume);
            println!("  成交时间: {}", trade.TradeTime.to_string());
        }
    }

    fn on_err_rtn_order_insert(
        &mut self,
        input_order: Option<&CThostFtdcInputOrderField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
    ) {
        println!("[TD] 报单录入错误:");
        print_rsp_info!(rsp_info);
        
        if let Some(order) = input_order {
            println!("[TD] 错误报单信息:");
            println!("  合约: {}", order.InstrumentID.to_string());
            println!("  报单引用: {}", order.OrderRef.to_string());
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
            println!("[TD] 错误响应: {} - {}", 
                error.ErrorID, 
                gb18030_cstr_i8_to_str(&error.ErrorMsg).unwrap_or("Unknown error".into())
            );
        }
    }
}

impl TraderSpiImpl {
    fn send_test_order(&mut self) {
        if !self.is_logged_in || !self.is_settled {
            println!("[TD] 系统未就绪，无法下单");
            return;
        }

        println!("\n[TD] === 发送测试订单 ===");
        
        let mut order = CThostFtdcInputOrderField::default();
        
        // 基本字段
        order.BrokerID.assign_from_str(&self.config.broker_id);
        order.InvestorID.assign_from_str(&self.config.user_id);
        
        // 合约代码 - 使用活跃的期货合约
        order.InstrumentID.assign_from_str("rb2510");  // 螺纹钢2510
        
        // 报单引用
        let order_ref = format!("{:012}", self.order_ref);
        order.OrderRef.assign_from_str(&order_ref);
        self.order_ref += 1;
        
        // 交易方向和开平标志
        order.Direction = THOST_FTDC_D_Buy as i8;  // 买入
        order.CombOffsetFlag[0] = THOST_FTDC_OF_Open as i8;  // 开仓
        
        // 投机套保标志
        order.CombHedgeFlag[0] = THOST_FTDC_HF_Speculation as i8;  // 投机
        
        // 价格和数量
        order.LimitPrice = 3600.0;  // 限价
        order.VolumeTotalOriginal = 1;  // 数量
        
        // 订单价格类型
        order.OrderPriceType = THOST_FTDC_OPT_LimitPrice as i8;  // 限价单
        
        // 有效期类型
        order.TimeCondition = THOST_FTDC_TC_IOC as i8;  // 立即成交否则撤销
        order.VolumeCondition = THOST_FTDC_VC_CV as i8;  // 全部数量
        order.MinVolume = 1;
        
        // 触发条件
        order.ContingentCondition = THOST_FTDC_CC_Immediately as i8;  // 立即
        
        // 强平标志
        order.ForceCloseReason = THOST_FTDC_FCC_NotForceClose as i8;  // 非强平
        
        // 自动挂起标志
        order.IsAutoSuspend = 0;
        
        // 用户强评标志
        order.UserForceClose = 0;
        
        println!("[TD] 下单参数:");
        println!("  合约: {}", "rb2510");
        println!("  方向: 买入");
        println!("  开平: 开仓");
        println!("  价格: {}", order.LimitPrice);
        println!("  数量: {}", order.VolumeTotalOriginal);
        println!("  报单引用: {}", order_ref);
        
        let ret = self.tdapi.req_order_insert(&mut order, 10);
        println!("[TD] 发送报单请求，返回: {}", ret);
        
        if ret == 0 {
            println!("[TD] 报单请求发送成功，等待交易所响应...");
        } else {
            println!("[TD] 报单请求发送失败，错误码: {}", ret);
        }
    }
}

pub fn run_td_order(config: CtpConfig) {
    println!("\n=== 启动交易接口（含下单测试） ===");
    println!("动态库路径: {}", config.td_dynlib_path.to_string_lossy());
    println!("前置地址: {}", config.td_front_address);

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
        front_id: 0,
        session_id: 0,
        order_ref: 1,
        is_logged_in: false,
        is_settled: false,
    };

    // 注册 SPI
    let mut td_spi = td_spi;
    tdapi.register_spi(&mut td_spi);

    // 订阅流 - 直接使用枚举值
    // 由于类型问题，我们不订阅流（在测试环境不是必须的）
    // tdapi.subscribe_public_topic(0);
    // tdapi.subscribe_private_topic(0);

    // 注册前置地址
    tdapi.register_front(&front_address);

    // 初始化连接
    tdapi.init();

    println!("[TD] 等待交易连接...");
    
    // 保持线程运行
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
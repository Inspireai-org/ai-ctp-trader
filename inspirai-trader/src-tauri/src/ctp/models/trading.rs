use serde::{Deserialize, Serialize};

// 订单输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderInput {
    pub instrument_id: String,
    pub direction: String, // Buy/Sell
    pub offset: String, // Open/Close/CloseToday/CloseYesterday
    pub price: f64,
    pub volume: u32,
    pub order_type: String, // Limit/Market/Stop/StopLimit
    pub time_condition: String, // IOC/GFS/GFD/GTD/GTC/GFA
    pub volume_condition: String, // Any/Min/All
    pub min_volume: u32,
    pub contingent_condition: String, // Immediately/Touch/TouchProfit
    pub stop_price: f64,
    pub force_close_reason: String, // NotForceClose/LackDeposit/ClientOverPositionLimit
    pub is_auto_suspend: bool,
}

// 订单引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRef {
    pub order_ref: String,
    pub front_id: i32,
    pub session_id: i32,
}

// 成交记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub order_ref: String,
    pub instrument_id: String,
    pub direction: String,
    pub offset: String,
    pub price: f64,
    pub volume: u32,
    pub trade_time: String,
    pub trade_type: String,
    pub exchange_id: String,
    pub commission: f64,
}

// 合约信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentInfo {
    pub instrument_id: String,
    pub exchange_id: String,
    pub instrument_name: String,
    pub product_id: String,
    pub product_class: String,
    pub delivery_year: i32,
    pub delivery_month: i32,
    pub max_market_order_volume: i32,
    pub min_market_order_volume: i32,
    pub max_limit_order_volume: i32,
    pub min_limit_order_volume: i32,
    pub volume_multiple: i32,
    pub price_tick: f64,
    pub create_date: String,
    pub open_date: String,
    pub expire_date: String,
    pub start_delivery_date: String,
    pub end_delivery_date: String,
    pub is_trading: bool,
    pub underlying_instrument: String,
    pub strike_price: f64,
    pub underlying_multiple: f64,
    pub long_margin_ratio: f64,
    pub short_margin_ratio: f64,
}

// 手续费率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionRate {
    pub instrument_id: String,
    pub open_ratio_by_money: f64,
    pub open_ratio_by_volume: f64,
    pub close_ratio_by_money: f64,
    pub close_ratio_by_volume: f64,
    pub close_today_ratio_by_money: f64,
    pub close_today_ratio_by_volume: f64,
}

// 保证金率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginRate {
    pub instrument_id: String,
    pub long_margin_ratio_by_money: f64,
    pub long_margin_ratio_by_volume: f64,
    pub short_margin_ratio_by_money: f64,
    pub short_margin_ratio_by_volume: f64,
}

// 行情数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub instrument_id: String,
    pub exchange_id: String,
    pub last_price: f64,
    pub pre_settlement_price: f64,
    pub pre_close_price: f64,
    pub pre_open_interest: f64,
    pub open_price: f64,
    pub highest_price: f64,
    pub lowest_price: f64,
    pub volume: i32,
    pub turnover: f64,
    pub open_interest: f64,
    pub close_price: f64,
    pub settlement_price: f64,
    pub upper_limit_price: f64,
    pub lower_limit_price: f64,
    pub bid_price: f64,
    pub bid_volume: i32,
    pub ask_price: f64,
    pub ask_volume: i32,
    pub average_price: f64,
    pub update_time: String,
    pub update_millisec: i32,
    pub trading_day: String,
}

// 市场数据订阅
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataSubscription {
    pub instruments: Vec<String>,
    pub priority: String, // Low/Normal/High/Urgent
}

// 风险参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskParams {
    pub max_position_ratio: f64,
    pub max_single_loss: f64,
    pub max_daily_loss: f64,
    pub max_order_volume: i32,
    pub position_limit: std::collections::HashMap<String, i32>,
    pub forbidden_instruments: Vec<String>,
    pub auto_stop_loss: bool,
    pub stop_loss_ratio: f64,
    pub auto_take_profit: bool,
    pub take_profit_ratio: f64,
}
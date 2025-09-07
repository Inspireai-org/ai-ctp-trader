use serde::{Deserialize, Serialize};
// 暂时允许未使用的导入，这些将在后续任务中使用
#[allow(unused_imports)]
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// 重新导出 trading 模块的类型
pub mod trading;
pub use trading::{
    OrderInput, OrderRef, Trade, InstrumentInfo, 
    CommissionRate, MarginRate, MarketData, 
    MarketDataSubscription, RiskParams
};

/// 登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub trading_day: String,
    pub login_time: String,
    pub broker_id: String,
    pub user_id: String,
    pub system_name: String,
    pub front_id: i32,
    pub session_id: i32,
    pub max_order_ref: String,
}

/// 行情数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataTick {
    /// 合约代码
    pub instrument_id: String,
    /// 最新价
    pub last_price: f64,
    /// 成交量
    pub volume: i64,
    /// 成交额
    pub turnover: f64,
    /// 持仓量
    pub open_interest: i64,
    /// 买一价
    pub bid_price1: f64,
    /// 买一量
    pub bid_volume1: i32,
    /// 卖一价
    pub ask_price1: f64,
    /// 卖一量
    pub ask_volume1: i32,
    /// 更新时间
    pub update_time: String,
    /// 更新毫秒
    pub update_millisec: i32,
    /// 涨跌幅
    pub change_percent: f64,
    /// 涨跌额
    pub change_amount: f64,
    /// 今开盘
    pub open_price: f64,
    /// 最高价
    pub highest_price: f64,
    /// 最低价
    pub lowest_price: f64,
    /// 昨收盘
    pub pre_close_price: f64,
}

/// 买卖方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderDirection {
    /// 买入
    Buy,
    /// 卖出
    Sell,
}

impl std::fmt::Display for OrderDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderDirection::Buy => write!(f, "买入"),
            OrderDirection::Sell => write!(f, "卖出"),
        }
    }
}

/// 开平仓标志
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OffsetFlag {
    /// 开仓
    Open,
    /// 平仓
    Close,
    /// 平今
    CloseToday,
    /// 平昨
    CloseYesterday,
}

/// 订单类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    /// 限价单
    Limit,
    /// 市价单
    Market,
    /// 条件单
    Conditional,
}

/// 时间条件
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeCondition {
    /// 立即完成，否则撤销
    IOC,
    /// 全部成交或撤销
    FOK,
    /// 当日有效
    GFD,
}

/// 订单状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatusType {
    /// 未知
    Unknown,
    /// 全部成交
    AllTraded,
    /// 部分成交还在队列中
    PartTradedQueueing,
    /// 部分成交不在队列中
    PartTradedNotQueueing,
    /// 未成交还在队列中
    NoTradeQueueing,
    /// 未成交不在队列中
    NoTradeNotQueueing,
    /// 撤单
    Canceled,
    /// 撤单（别名）
    Cancelled,
    /// 触发
    Touched,
}

/// 订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    /// 合约代码
    pub instrument_id: String,
    /// 订单引用
    pub order_ref: String,
    /// 买卖方向
    pub direction: OrderDirection,
    /// 开平仓标志
    pub offset_flag: OffsetFlag,
    /// 价格
    pub price: f64,
    /// 数量
    pub volume: u32,
    /// 订单类型
    pub order_type: OrderType,
    /// 价格类型
    pub price_type: OrderPriceType,
    /// 时间条件
    pub time_condition: OrderTimeCondition,
    /// 成交量条件
    pub volume_condition: OrderVolumeCondition,
    /// 最小成交量
    pub min_volume: u32,
    /// 触发条件
    pub contingent_condition: OrderContingentCondition,
    /// 止损价
    pub stop_price: f64,
    /// 强平原因
    pub force_close_reason: OrderForceCloseReason,
    /// 自动挂起标志
    pub is_auto_suspend: bool,
}

/// 撤单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAction {
    /// 订单号
    pub order_id: String,
    /// 合约代码
    pub instrument_id: String,
    /// 操作类型（撤单）
    pub action_flag: ActionFlag,
}

/// 操作标志
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionFlag {
    /// 删除（撤单）
    Delete,
}

/// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatus {
    /// 订单引用
    pub order_ref: String,
    /// 订单号
    pub order_id: String,
    /// 合约代码
    pub instrument_id: String,
    /// 买卖方向
    pub direction: OrderDirection,
    /// 开平仓标志
    pub offset_flag: OffsetFlag,
    /// 价格
    pub price: f64,
    /// 委托价格
    pub limit_price: f64,
    /// 数量
    pub volume: u32,
    /// 委托数量
    pub volume_total_original: i32,
    /// 成交数量
    pub volume_traded: u32,
    /// 剩余数量
    pub volume_left: u32,
    /// 剩余数量（兼容旧字段）
    pub volume_total: i32,
    /// 订单状态
    pub status: OrderStatusType,
    /// 提交时间
    pub submit_time: chrono::DateTime<chrono::Local>,
    /// 委托时间
    pub insert_time: String,
    /// 更新时间
    pub update_time: chrono::DateTime<chrono::Local>,
    /// 前置编号
    pub front_id: i32,
    /// 会话编号
    pub session_id: i32,
    /// 系统订单号
    pub order_sys_id: String,
    /// 状态信息
    pub status_msg: String,
    /// 是否本地订单
    pub is_local: bool,
    /// 冻结保证金
    pub frozen_margin: f64,
    /// 冻结手续费
    pub frozen_commission: f64,
}

/// 成交记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    /// 成交编号
    pub trade_id: String,
    /// 订单号
    pub order_id: String,
    /// 合约代码
    pub instrument_id: String,
    /// 买卖方向
    pub direction: OrderDirection,
    /// 开平仓标志
    pub offset_flag: OffsetFlag,
    /// 成交价格
    pub price: f64,
    /// 成交数量
    pub volume: i32,
    /// 成交时间
    pub trade_time: String,
}

/// 持仓方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PositionDirection {
    /// 多头
    Long,
    /// 空头
    Short,
}

impl std::fmt::Display for PositionDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionDirection::Long => write!(f, "多头"),
            PositionDirection::Short => write!(f, "空头"),
        }
    }
}

/// 持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// 合约代码
    pub instrument_id: String,
    /// 持仓方向
    pub direction: PositionDirection,
    /// 总持仓
    pub total_position: i32,
    /// 昨持仓
    pub yesterday_position: i32,
    /// 今持仓
    pub today_position: i32,
    /// 开仓成本
    pub open_cost: f64,
    /// 持仓成本
    pub position_cost: f64,
    /// 占用保证金
    pub margin: f64,
    /// 浮动盈亏
    pub unrealized_pnl: f64,
    /// 平仓盈亏
    pub realized_pnl: f64,
}

/// 账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// 账户代码
    pub account_id: String,
    /// 可用资金
    pub available: f64,
    /// 账户余额
    pub balance: f64,
    /// 保证金
    pub margin: f64,
    /// 冻结资金
    pub frozen_margin: f64,
    /// 冻结手续费
    pub frozen_commission: f64,
    /// 当前保证金总额
    pub curr_margin: f64,
    /// 手续费
    pub commission: f64,
    /// 平仓盈亏
    pub close_profit: f64,
    /// 持仓盈亏
    pub position_profit: f64,
    /// 风险度
    pub risk_ratio: f64,
}

/// 登录凭据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginCredentials {
    pub broker_id: String,
    pub user_id: String,
    pub password: String,
    pub app_id: String,
    pub auth_code: String,
}

/// 订单价格类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderPriceType {
    /// 限价
    Limit,
    /// 市价
    Market,
    /// 最优价
    Best,
    /// 最新价
    LastPrice,
}

/// 订单时间条件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderTimeCondition {
    /// 立即完成，否则撤销
    IOC,
    /// 本节有效
    GFS,
    /// 当日有效
    GFD,
    /// 指定日期前有效
    GTD,
    /// 撤销前有效
    GTC,
    /// 集合竞价有效
    GFA,
}

/// 订单成交量条件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderVolumeCondition {
    /// 任意数量
    Any,
    /// 最小数量
    Min,
    /// 全部数量
    All,
}

/// 订单触发条件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderContingentCondition {
    /// 立即
    Immediately,
    /// 止损
    Touch,
    /// 止赢
    TouchProfit,
    /// 预埋单
    ParkedOrder,
}

/// 强平原因
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderForceCloseReason {
    /// 非强平
    NotForceClose,
    /// 资金不足
    LackDeposit,
    /// 客户超仓
    ClientOverPositionLimit,
    /// 会员超仓
    MemberOverPositionLimit,
    /// 持仓非整数倍
    NotMultiple,
    /// 违规
    Violation,
    /// 其它
    Other,
}

/// 使用 OffsetFlag 作为 OrderOffsetFlag 的别名
pub type OrderOffsetFlag = OffsetFlag;
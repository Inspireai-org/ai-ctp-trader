use crate::ctp::{
    models::*,
    CtpError,
};

// 使用 ctp2rs 提供的官方数据结构和转换工具
use ctp2rs::v1alpha1::{
    CThostFtdcDepthMarketDataField,
    CThostFtdcInputOrderField,
    CThostFtdcOrderField,
    CThostFtdcTradeField,
    CThostFtdcInvestorPositionField,
    CThostFtdcTradingAccountField,
};
use ctp2rs::ffi::{gb18030_cstr_i8_to_str, AssignFromString, WrapToString};

/// 数据转换工具
/// 
/// 负责在 CTP 原生数据结构和业务模型之间进行转换
/// 严格使用 ctp2rs 提供的官方工具，禁止自定义实现
pub struct DataConverter;

impl DataConverter {
    /// 将 CTP 行情数据转换为业务模型
    /// 使用 ctp2rs 官方数据结构和转换工具
    pub fn convert_market_data(ctp_data: &CThostFtdcDepthMarketDataField) -> Result<MarketDataTick, CtpError> {
        // 使用 ctp2rs 官方字符串转换工具
        let instrument_id = gb18030_cstr_i8_to_str(&ctp_data.InstrumentID)
            .map_err(|e| CtpError::ConversionError(format!("合约代码转换失败: {}", e)))?.to_string();
        let update_time = gb18030_cstr_i8_to_str(&ctp_data.UpdateTime)
            .map_err(|e| CtpError::ConversionError(format!("更新时间转换失败: {}", e)))?.to_string();
        
        // 计算涨跌幅和涨跌额
        let change_amount = if ctp_data.PreClosePrice > 0.0 {
            ctp_data.LastPrice - ctp_data.PreClosePrice
        } else {
            0.0
        };
        
        let change_percent = if ctp_data.PreClosePrice > 0.0 {
            (change_amount / ctp_data.PreClosePrice) * 100.0
        } else {
            0.0
        };
        
        Ok(MarketDataTick {
            instrument_id,
            last_price: ctp_data.LastPrice,
            volume: ctp_data.Volume as i64,
            turnover: ctp_data.Turnover,
            open_interest: ctp_data.OpenInterest as i64,
            bid_price1: ctp_data.BidPrice1,
            bid_volume1: ctp_data.BidVolume1,
            ask_price1: ctp_data.AskPrice1,
            ask_volume1: ctp_data.AskVolume1,
            update_time,
            update_millisec: ctp_data.UpdateMillisec,
            change_percent,
            change_amount,
            open_price: ctp_data.OpenPrice,
            highest_price: ctp_data.HighestPrice,
            lowest_price: ctp_data.LowestPrice,
            pre_close_price: ctp_data.PreClosePrice,
        })
    }

    /// 将业务订单请求转换为 CTP 结构体
    /// 使用 ctp2rs 官方数据结构和字符串赋值工具
    pub fn convert_order_request(
        order: &OrderRequest,
        broker_id: &str,
        investor_id: &str,
        order_ref: &str,
    ) -> Result<CThostFtdcInputOrderField, CtpError> {
        let mut ctp_order = CThostFtdcInputOrderField::default();
        
        // 使用 ctp2rs 官方字符串赋值工具
        ctp_order.BrokerID.assign_from_str(broker_id);
        ctp_order.InvestorID.assign_from_str(investor_id);
        ctp_order.InstrumentID.assign_from_str(&order.instrument_id);
        ctp_order.OrderRef.assign_from_str(order_ref);
        
        // 订单参数
        ctp_order.Direction = Self::direction_to_ctp_char(order.direction);
        ctp_order.CombOffsetFlag[0] = Self::offset_flag_to_ctp_char(order.offset_flag);
        ctp_order.LimitPrice = order.price;
        ctp_order.VolumeTotalOriginal = order.volume;
        ctp_order.OrderPriceType = Self::order_type_to_ctp_char(order.order_type);
        ctp_order.TimeCondition = Self::time_condition_to_ctp_char(order.time_condition);
        
        // 其他必要字段
        ctp_order.CombHedgeFlag[0] = '1' as i8; // 投机
        ctp_order.ContingentCondition = '1' as i8; // 立即
        ctp_order.ForceCloseReason = '0' as i8; // 非强平
        ctp_order.IsAutoSuspend = 0; // 不自动挂起
        ctp_order.UserForceClose = 0; // 非用户强平
        ctp_order.VolumeCondition = '1' as i8; // 任何数量
        ctp_order.MinVolume = 1; // 最小成交量
        
        Ok(ctp_order)
    }

    /// 将 CTP 订单转换为订单状态（简化版本，用于 TraderSpi）
    pub fn convert_order(ctp_order: &CThostFtdcOrderField) -> Result<OrderStatus, CtpError> {
        Self::convert_order_status(ctp_order)
    }

    /// 将 CTP 成交转换为成交记录（简化版本，用于 TraderSpi）
    pub fn convert_trade(ctp_trade: &CThostFtdcTradeField) -> Result<TradeRecord, CtpError> {
        Self::convert_trade_record(ctp_trade)
    }

    /// 将 CTP 持仓转换为持仓信息（简化版本，用于 TraderSpi）
    pub fn convert_position(ctp_position: &CThostFtdcInvestorPositionField) -> Result<Position, CtpError> {
        Self::convert_position_info(ctp_position)
    }

    /// 将 CTP 账户转换为账户信息（简化版本，用于 TraderSpi）
    pub fn convert_account(ctp_account: &CThostFtdcTradingAccountField) -> Result<AccountInfo, CtpError> {
        Self::convert_account_info(ctp_account)
    }

    /// 将 CTP 订单状态转换为业务模型
    /// 使用 ctp2rs 官方字符串转换工具
    pub fn convert_order_status(ctp_order: &CThostFtdcOrderField) -> Result<OrderStatus, CtpError> {
        Ok(OrderStatus {
            order_id: gb18030_cstr_i8_to_str(&ctp_order.OrderRef)
                .map_err(|e| CtpError::ConversionError(format!("订单号转换失败: {}", e)))?.to_string(),
            instrument_id: gb18030_cstr_i8_to_str(&ctp_order.InstrumentID)
                .map_err(|e| CtpError::ConversionError(format!("合约代码转换失败: {}", e)))?.to_string(),
            direction: Self::ctp_char_to_direction(ctp_order.Direction)?,
            offset_flag: Self::ctp_char_to_offset_flag(ctp_order.CombOffsetFlag[0])?,
            limit_price: ctp_order.LimitPrice,
            volume_total_original: ctp_order.VolumeTotalOriginal,
            volume_traded: ctp_order.VolumeTraded,
            volume_total: ctp_order.VolumeTotal,
            status: Self::ctp_char_to_order_status(ctp_order.OrderStatus)?,
            insert_time: gb18030_cstr_i8_to_str(&ctp_order.InsertTime)
                .map_err(|e| CtpError::ConversionError(format!("插入时间转换失败: {}", e)))?.to_string(),
            update_time: gb18030_cstr_i8_to_str(&ctp_order.UpdateTime)
                .map_err(|e| CtpError::ConversionError(format!("更新时间转换失败: {}", e)))?.to_string(),
            status_msg: gb18030_cstr_i8_to_str(&ctp_order.StatusMsg)
                .ok()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty()),
        })
    }

    /// 将 CTP 成交记录转换为业务模型
    /// 使用 ctp2rs 官方字符串转换工具
    pub fn convert_trade_record(ctp_trade: &CThostFtdcTradeField) -> Result<TradeRecord, CtpError> {
        Ok(TradeRecord {
            trade_id: gb18030_cstr_i8_to_str(&ctp_trade.TradeID)
                .map_err(|e| CtpError::ConversionError(format!("成交编号转换失败: {}", e)))?.to_string(),
            order_id: gb18030_cstr_i8_to_str(&ctp_trade.OrderRef)
                .map_err(|e| CtpError::ConversionError(format!("订单号转换失败: {}", e)))?.to_string(),
            instrument_id: gb18030_cstr_i8_to_str(&ctp_trade.InstrumentID)
                .map_err(|e| CtpError::ConversionError(format!("合约代码转换失败: {}", e)))?.to_string(),
            direction: Self::ctp_char_to_direction(ctp_trade.Direction)?,
            offset_flag: Self::ctp_char_to_offset_flag(ctp_trade.OffsetFlag)?,
            price: ctp_trade.Price,
            volume: ctp_trade.Volume,
            trade_time: gb18030_cstr_i8_to_str(&ctp_trade.TradeTime)
                .map_err(|e| CtpError::ConversionError(format!("成交时间转换失败: {}", e)))?.to_string(),
        })
    }

    /// 将 CTP 持仓信息转换为业务模型
    /// 使用 ctp2rs 官方字符串转换工具
    pub fn convert_position_info(ctp_position: &CThostFtdcInvestorPositionField) -> Result<Position, CtpError> {
        let direction = if ctp_position.PosiDirection == '2' as i8 {
            PositionDirection::Long
        } else {
            PositionDirection::Short
        };

        Ok(Position {
            instrument_id: gb18030_cstr_i8_to_str(&ctp_position.InstrumentID)
                .map_err(|e| CtpError::ConversionError(format!("合约代码转换失败: {}", e)))?.to_string(),
            direction,
            total_position: ctp_position.Position,
            yesterday_position: ctp_position.YdPosition,
            today_position: ctp_position.TodayPosition,
            open_cost: ctp_position.OpenCost,
            position_cost: ctp_position.PositionCost,
            margin: ctp_position.UseMargin,
            unrealized_pnl: ctp_position.PositionProfit,
            realized_pnl: ctp_position.CloseProfit,
        })
    }

    /// 将 CTP 账户信息转换为业务模型
    /// 使用 ctp2rs 官方字符串转换工具
    pub fn convert_account_info(ctp_account: &CThostFtdcTradingAccountField) -> Result<AccountInfo, CtpError> {
        Ok(AccountInfo {
            account_id: gb18030_cstr_i8_to_str(&ctp_account.AccountID)
                .map_err(|e| CtpError::ConversionError(format!("账户ID转换失败: {}", e)))?.to_string(),
            available: ctp_account.Available,
            balance: ctp_account.Balance,
            frozen_margin: ctp_account.FrozenMargin,
            frozen_commission: ctp_account.FrozenCommission,
            curr_margin: ctp_account.CurrMargin,
            commission: ctp_account.Commission,
            close_profit: ctp_account.CloseProfit,
            position_profit: ctp_account.PositionProfit,
            risk_ratio: if ctp_account.Balance > 0.0 {
                ctp_account.CurrMargin / ctp_account.Balance * 100.0
            } else {
                0.0
            },
        })
    }

    // 辅助转换方法 - 使用 ctp2rs 官方工具，禁止自定义实现

    /// 买卖方向转换
    pub fn direction_to_ctp_char(direction: OrderDirection) -> i8 {
        match direction {
            OrderDirection::Buy => '0' as i8,
            OrderDirection::Sell => '1' as i8,
        }
    }

    pub fn ctp_char_to_direction(ctp_char: i8) -> Result<OrderDirection, CtpError> {
        match ctp_char as u8 as char {
            '0' => Ok(OrderDirection::Buy),
            '1' => Ok(OrderDirection::Sell),
            _ => Err(CtpError::ConversionError(
                format!("未知的买卖方向: {}", ctp_char)
            )),
        }
    }

    /// 开平仓标志转换
    pub fn offset_flag_to_ctp_char(offset_flag: OffsetFlag) -> i8 {
        match offset_flag {
            OffsetFlag::Open => '0' as i8,
            OffsetFlag::Close => '1' as i8,
            OffsetFlag::CloseToday => '3' as i8,
            OffsetFlag::CloseYesterday => '4' as i8,
        }
    }

    pub fn ctp_char_to_offset_flag(ctp_char: i8) -> Result<OffsetFlag, CtpError> {
        match ctp_char as u8 as char {
            '0' => Ok(OffsetFlag::Open),
            '1' => Ok(OffsetFlag::Close),
            '3' => Ok(OffsetFlag::CloseToday),
            '4' => Ok(OffsetFlag::CloseYesterday),
            _ => Err(CtpError::ConversionError(
                format!("未知的开平仓标志: {}", ctp_char)
            )),
        }
    }

    /// 订单类型转换
    fn order_type_to_ctp_char(order_type: OrderType) -> i8 {
        match order_type {
            OrderType::Limit => '2' as i8,
            OrderType::Market => '1' as i8,
            OrderType::Conditional => '9' as i8,
        }
    }

    /// 时间条件转换
    fn time_condition_to_ctp_char(time_condition: TimeCondition) -> i8 {
        match time_condition {
            TimeCondition::IOC => '1' as i8,
            TimeCondition::FOK => '4' as i8,
            TimeCondition::GFD => '3' as i8,
        }
    }

    /// 订单状态转换
    fn ctp_char_to_order_status(ctp_char: i8) -> Result<OrderStatusType, CtpError> {
        match ctp_char as u8 as char {
            '0' => Ok(OrderStatusType::AllTraded),
            '1' => Ok(OrderStatusType::PartTradedQueueing),
            '2' => Ok(OrderStatusType::PartTradedNotQueueing),
            '3' => Ok(OrderStatusType::NoTradeQueueing),
            '4' => Ok(OrderStatusType::NoTradeNotQueueing),
            '5' => Ok(OrderStatusType::Canceled),
            'a' => Ok(OrderStatusType::Unknown),
            _ => Ok(OrderStatusType::Unknown),
        }
    }
}

// 所有 CTP 数据结构都使用 ctp2rs 提供的官方定义
// 严禁自定义 CTP 结构体，必须使用 ctp2rs::v1alpha1 中的官方结构体

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_conversion() {
        assert_eq!(DataConverter::direction_to_ctp_char(OrderDirection::Buy), '0' as i8);
        assert_eq!(DataConverter::direction_to_ctp_char(OrderDirection::Sell), '1' as i8);
        
        assert_eq!(DataConverter::ctp_char_to_direction('0' as i8).unwrap(), OrderDirection::Buy);
        assert_eq!(DataConverter::ctp_char_to_direction('1' as i8).unwrap(), OrderDirection::Sell);
    }

    #[test]
    fn test_offset_flag_conversion() {
        assert_eq!(DataConverter::offset_flag_to_ctp_char(OffsetFlag::Open), '0' as i8);
        assert_eq!(DataConverter::offset_flag_to_ctp_char(OffsetFlag::Close), '1' as i8);
        
        assert_eq!(DataConverter::ctp_char_to_offset_flag('0' as i8).unwrap(), OffsetFlag::Open);
        assert_eq!(DataConverter::ctp_char_to_offset_flag('1' as i8).unwrap(), OffsetFlag::Close);
    }

    #[test]
    fn test_order_type_conversion() {
        assert_eq!(DataConverter::order_type_to_ctp_char(OrderType::Limit), '2' as i8);
        assert_eq!(DataConverter::order_type_to_ctp_char(OrderType::Market), '1' as i8);
    }
}
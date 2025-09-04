use crate::ctp::CtpError;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::{DateTime, Local, NaiveDate};
use tracing::{info, warn, debug};

/// 结算管理器
pub struct SettlementManager {
    /// 结算单存储 (date -> settlement)
    settlements: Arc<Mutex<HashMap<NaiveDate, Settlement>>>,
    /// 当前交易日
    current_trading_day: Arc<Mutex<Option<NaiveDate>>>,
    /// 结算确认状态
    confirmation_status: Arc<Mutex<HashMap<NaiveDate, bool>>>,
}

/// 结算单
#[derive(Debug, Clone)]
pub struct Settlement {
    /// 交易日
    pub trading_day: NaiveDate,
    /// 结算内容
    pub content: String,
    /// 生成时间
    pub generate_time: DateTime<Local>,
    /// 是否已确认
    pub confirmed: bool,
    /// 确认时间
    pub confirm_time: Option<DateTime<Local>>,
    /// 结算摘要
    pub summary: SettlementSummary,
}

/// 结算摘要
#[derive(Debug, Clone, Default)]
pub struct SettlementSummary {
    /// 期初权益
    pub prev_balance: f64,
    /// 期末权益
    pub balance: f64,
    /// 平仓盈亏
    pub close_profit: f64,
    /// 持仓盈亏
    pub position_profit: f64,
    /// 手续费
    pub commission: f64,
    /// 入金
    pub deposit: f64,
    /// 出金
    pub withdraw: f64,
    /// 当日盈亏
    pub daily_profit: f64,
    /// 风险度
    pub risk_ratio: f64,
}

impl SettlementManager {
    pub fn new() -> Self {
        Self {
            settlements: Arc::new(Mutex::new(HashMap::new())),
            current_trading_day: Arc::new(Mutex::new(None)),
            confirmation_status: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 设置当前交易日
    pub fn set_trading_day(&self, trading_day: &str) -> Result<(), CtpError> {
        let date = NaiveDate::parse_from_str(trading_day, "%Y%m%d")
            .map_err(|e| CtpError::ConversionError(format!("交易日期格式错误: {}", e)))?;
        
        *self.current_trading_day.lock().unwrap() = Some(date);
        info!("设置交易日: {}", trading_day);
        
        Ok(())
    }

    /// 保存结算单
    pub fn save_settlement(&self, content: String) -> Result<(), CtpError> {
        let trading_day = self.current_trading_day.lock().unwrap()
            .ok_or_else(|| CtpError::StateError("交易日未设置".to_string()))?;
        
        // 解析结算单内容
        let summary = self.parse_settlement_content(&content)?;
        
        let settlement = Settlement {
            trading_day,
            content: content.clone(),
            generate_time: Local::now(),
            confirmed: false,
            confirm_time: None,
            summary,
        };
        
        self.settlements.lock().unwrap()
            .insert(trading_day, settlement);
        
        info!("保存结算单: {}", trading_day);
        
        Ok(())
    }

    /// 确认结算单
    pub fn confirm_settlement(&self, trading_day: Option<NaiveDate>) -> Result<(), CtpError> {
        let date = trading_day.or_else(|| *self.current_trading_day.lock().unwrap())
            .ok_or_else(|| CtpError::StateError("交易日未指定".to_string()))?;
        
        let mut settlements = self.settlements.lock().unwrap();
        
        let settlement = settlements.get_mut(&date)
            .ok_or_else(|| CtpError::NotFound(format!("结算单不存在: {}", date)))?;
        
        if settlement.confirmed {
            return Err(CtpError::StateError("结算单已确认".to_string()));
        }
        
        settlement.confirmed = true;
        settlement.confirm_time = Some(Local::now());
        
        self.confirmation_status.lock().unwrap()
            .insert(date, true);
        
        info!("确认结算单: {}", date);
        
        Ok(())
    }

    /// 获取结算单
    pub fn get_settlement(&self, trading_day: Option<NaiveDate>) -> Result<Settlement, CtpError> {
        let date = trading_day.or_else(|| *self.current_trading_day.lock().unwrap())
            .ok_or_else(|| CtpError::StateError("交易日未指定".to_string()))?;
        
        self.settlements.lock().unwrap()
            .get(&date)
            .cloned()
            .ok_or_else(|| CtpError::NotFound(format!("结算单不存在: {}", date)))
    }

    /// 获取最近N天的结算单
    pub fn get_recent_settlements(&self, days: usize) -> Vec<Settlement> {
        let settlements = self.settlements.lock().unwrap();
        
        let mut dates: Vec<_> = settlements.keys().cloned().collect();
        dates.sort_by(|a, b| b.cmp(a)); // 降序排序
        
        dates.into_iter()
            .take(days)
            .filter_map(|date| settlements.get(&date).cloned())
            .collect()
    }

    /// 检查结算确认状态
    pub fn is_settlement_confirmed(&self, trading_day: Option<NaiveDate>) -> bool {
        let date = trading_day.or_else(|| *self.current_trading_day.lock().unwrap());
        
        if let Some(d) = date {
            self.confirmation_status.lock().unwrap()
                .get(&d)
                .copied()
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// 解析结算单内容
    fn parse_settlement_content(&self, content: &str) -> Result<SettlementSummary, CtpError> {
        let mut summary = SettlementSummary::default();
        
        // 简单的解析逻辑，实际需要根据结算单格式调整
        for line in content.lines() {
            if line.contains("期初权益") {
                summary.prev_balance = self.extract_number(line);
            } else if line.contains("期末权益") {
                summary.balance = self.extract_number(line);
            } else if line.contains("平仓盈亏") {
                summary.close_profit = self.extract_number(line);
            } else if line.contains("持仓盈亏") {
                summary.position_profit = self.extract_number(line);
            } else if line.contains("手续费") {
                summary.commission = self.extract_number(line);
            } else if line.contains("入金") {
                summary.deposit = self.extract_number(line);
            } else if line.contains("出金") {
                summary.withdraw = self.extract_number(line);
            } else if line.contains("风险度") {
                summary.risk_ratio = self.extract_number(line);
            }
        }
        
        // 计算当日盈亏
        summary.daily_profit = summary.close_profit + summary.position_profit - summary.commission;
        
        Ok(summary)
    }

    /// 从文本行提取数字
    fn extract_number(&self, line: &str) -> f64 {
        // 简单的数字提取逻辑
        line.split_whitespace()
            .filter_map(|s| s.parse::<f64>().ok())
            .next()
            .unwrap_or(0.0)
    }

    /// 生成结算报告
    pub fn generate_report(&self, start_date: NaiveDate, end_date: NaiveDate) -> SettlementReport {
        let settlements = self.settlements.lock().unwrap();
        
        let mut report = SettlementReport::default();
        report.start_date = start_date;
        report.end_date = end_date;
        
        for (date, settlement) in settlements.iter() {
            if date >= &start_date && date <= &end_date {
                report.total_days += 1;
                report.total_profit += settlement.summary.daily_profit;
                report.total_commission += settlement.summary.commission;
                report.total_deposit += settlement.summary.deposit;
                report.total_withdraw += settlement.summary.withdraw;
                
                if settlement.summary.daily_profit > 0.0 {
                    report.profit_days += 1;
                    report.max_daily_profit = report.max_daily_profit.max(settlement.summary.daily_profit);
                } else {
                    report.loss_days += 1;
                    report.max_daily_loss = report.max_daily_loss.min(settlement.summary.daily_profit);
                }
            }
        }
        
        if report.total_days > 0 {
            report.win_rate = (report.profit_days as f64) / (report.total_days as f64);
            report.avg_daily_profit = report.total_profit / (report.total_days as f64);
        }
        
        report
    }

    /// 清空结算数据
    pub fn clear(&self) {
        self.settlements.lock().unwrap().clear();
        self.confirmation_status.lock().unwrap().clear();
        *self.current_trading_day.lock().unwrap() = None;
        info!("清空结算数据");
    }
}

/// 结算报告
#[derive(Debug, Clone, Default)]
pub struct SettlementReport {
    /// 开始日期
    pub start_date: NaiveDate,
    /// 结束日期
    pub end_date: NaiveDate,
    /// 总天数
    pub total_days: i32,
    /// 盈利天数
    pub profit_days: i32,
    /// 亏损天数
    pub loss_days: i32,
    /// 胜率
    pub win_rate: f64,
    /// 总盈亏
    pub total_profit: f64,
    /// 总手续费
    pub total_commission: f64,
    /// 总入金
    pub total_deposit: f64,
    /// 总出金
    pub total_withdraw: f64,
    /// 日均盈亏
    pub avg_daily_profit: f64,
    /// 最大日盈利
    pub max_daily_profit: f64,
    /// 最大日亏损
    pub max_daily_loss: f64,
}
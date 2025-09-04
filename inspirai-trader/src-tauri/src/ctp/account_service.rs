use crate::ctp::{
    CtpError, CtpEvent, ClientState, AccountInfo, Position,
    config::CtpConfig,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use tracing::{info, warn, error, debug};

/// 账户服务
pub struct AccountService {
    /// 账户信息
    account_info: Arc<Mutex<Option<AccountInfo>>>,
    /// 持仓信息
    positions: Arc<Mutex<HashMap<String, Position>>>,
    /// 资金统计
    fund_stats: Arc<Mutex<FundStats>>,
    /// 风险指标
    risk_metrics: Arc<Mutex<RiskMetrics>>,
    /// 最后更新时间
    last_update: Arc<Mutex<Option<Instant>>>,
    /// 配置
    config: CtpConfig,
}

/// 资金统计
#[derive(Debug, Clone, Default)]
pub struct FundStats {
    /// 初始资金
    pub initial_balance: f64,
    /// 当前余额
    pub current_balance: f64,
    /// 可用资金
    pub available: f64,
    /// 冻结保证金
    pub frozen_margin: f64,
    /// 冻结手续费
    pub frozen_commission: f64,
    /// 当前保证金
    pub curr_margin: f64,
    /// 手续费
    pub commission: f64,
    /// 平仓盈亏
    pub close_profit: f64,
    /// 持仓盈亏
    pub position_profit: f64,
    /// 今日盈亏
    pub today_profit: f64,
    /// 累计盈亏
    pub total_profit: f64,
}

/// 风险指标
#[derive(Debug, Clone, Default)]
pub struct RiskMetrics {
    /// 风险度（保证金占用率）
    pub risk_ratio: f64,
    /// 可用资金比例
    pub available_ratio: f64,
    /// 持仓盈亏比例
    pub position_profit_ratio: f64,
    /// 最大回撤
    pub max_drawdown: f64,
    /// 警戒线（风险度）
    pub warning_level: f64,
    /// 强平线（风险度）
    pub force_close_level: f64,
}

impl AccountService {
    pub fn new(config: CtpConfig) -> Self {
        let mut risk_metrics = RiskMetrics::default();
        risk_metrics.warning_level = 0.8;  // 80% 警戒
        risk_metrics.force_close_level = 0.9;  // 90% 强平
        
        Self {
            account_info: Arc::new(Mutex::new(None)),
            positions: Arc::new(Mutex::new(HashMap::new())),
            fund_stats: Arc::new(Mutex::new(FundStats::default())),
            risk_metrics: Arc::new(Mutex::new(risk_metrics)),
            last_update: Arc::new(Mutex::new(None)),
            config,
        }
    }

    /// 更新账户信息
    pub fn update_account(&self, account: AccountInfo) -> Result<(), CtpError> {
        let balance = account.balance;
        let available = account.available;
        
        // 更新账户信息
        *self.account_info.lock().unwrap() = Some(account.clone());
        
        // 更新资金统计
        let mut stats = self.fund_stats.lock().unwrap();
        if stats.initial_balance == 0.0 {
            stats.initial_balance = balance;
        }
        stats.current_balance = balance;
        stats.available = available;
        stats.frozen_margin = account.frozen_margin;
        stats.frozen_commission = account.frozen_commission;
        stats.curr_margin = account.curr_margin;
        stats.commission = account.commission;
        stats.close_profit = account.close_profit;
        stats.position_profit = account.position_profit;
        stats.today_profit = account.close_profit + account.position_profit;
        stats.total_profit = balance - stats.initial_balance;
        
        // 更新风险指标
        self.update_risk_metrics(&account)?;
        
        // 更新时间戳
        *self.last_update.lock().unwrap() = Some(Instant::now());
        
        info!("账户更新: 余额={:.2}, 可用={:.2}, 风险度={:.2}%", 
            balance, available, account.risk_ratio);
        
        Ok(())
    }

    /// 更新持仓信息
    pub fn update_position(&self, position: Position) -> Result<(), CtpError> {
        let instrument_id = position.instrument_id.clone();
        
        self.positions.lock().unwrap()
            .insert(instrument_id.clone(), position.clone());
        
        debug!("持仓更新: {} 方向={:?} 总仓={} 盈亏={:.2}", 
            instrument_id, position.direction, position.total_position, position.unrealized_pnl);
        
        Ok(())
    }

    /// 批量更新持仓
    pub fn update_positions(&self, positions: Vec<Position>) -> Result<(), CtpError> {
        for position in positions {
            self.update_position(position)?;
        }
        Ok(())
    }

    /// 获取账户信息
    pub fn get_account(&self) -> Option<AccountInfo> {
        self.account_info.lock().unwrap().clone()
    }

    /// 获取资金统计
    pub fn get_fund_stats(&self) -> FundStats {
        self.fund_stats.lock().unwrap().clone()
    }

    /// 获取风险指标
    pub fn get_risk_metrics(&self) -> RiskMetrics {
        self.risk_metrics.lock().unwrap().clone()
    }

    /// 获取所有持仓
    pub fn get_positions(&self) -> Vec<Position> {
        self.positions.lock().unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// 获取指定合约持仓
    pub fn get_position(&self, instrument_id: &str) -> Option<Position> {
        self.positions.lock().unwrap()
            .get(instrument_id)
            .cloned()
    }

    /// 计算可开仓手数
    pub fn calculate_available_volume(
        &self, 
        instrument_id: &str,
        price: f64,
        margin_ratio: f64,
    ) -> Result<i32, CtpError> {
        let account = self.get_account()
            .ok_or_else(|| CtpError::NotFound("账户信息未初始化".to_string()))?;
        
        // 可用资金
        let available = account.available;
        
        // 每手保证金 = 价格 * 合约乘数 * 保证金率
        // 这里假设合约乘数为10（需要根据具体合约调整）
        let contract_multiplier = 10.0;
        let margin_per_lot = price * contract_multiplier * margin_ratio;
        
        // 可开手数 = 可用资金 / 每手保证金
        let available_volume = (available / margin_per_lot).floor() as i32;
        
        Ok(available_volume)
    }

    /// 检查风险状态
    pub fn check_risk_status(&self) -> RiskStatus {
        let metrics = self.get_risk_metrics();
        
        if metrics.risk_ratio >= metrics.force_close_level {
            RiskStatus::ForceClose
        } else if metrics.risk_ratio >= metrics.warning_level {
            RiskStatus::Warning
        } else {
            RiskStatus::Normal
        }
    }

    /// 更新风险指标
    fn update_risk_metrics(&self, account: &AccountInfo) -> Result<(), CtpError> {
        let mut metrics = self.risk_metrics.lock().unwrap();
        
        metrics.risk_ratio = account.risk_ratio / 100.0;
        
        if account.balance > 0.0 {
            metrics.available_ratio = account.available / account.balance;
            metrics.position_profit_ratio = account.position_profit / account.balance;
        }
        
        // 计算最大回撤
        let stats = self.fund_stats.lock().unwrap();
        if stats.initial_balance > 0.0 {
            let current_drawdown = (stats.initial_balance - account.balance) / stats.initial_balance;
            if current_drawdown > metrics.max_drawdown {
                metrics.max_drawdown = current_drawdown;
            }
        }
        
        Ok(())
    }

    /// 清空账户数据
    pub fn clear(&self) {
        *self.account_info.lock().unwrap() = None;
        self.positions.lock().unwrap().clear();
        *self.fund_stats.lock().unwrap() = FundStats::default();
        *self.last_update.lock().unwrap() = None;
        
        let mut metrics = self.risk_metrics.lock().unwrap();
        *metrics = RiskMetrics::default();
        metrics.warning_level = 0.8;
        metrics.force_close_level = 0.9;
    }

    /// 获取账户摘要
    pub fn get_summary(&self) -> AccountSummary {
        let account = self.get_account();
        let stats = self.get_fund_stats();
        let metrics = self.get_risk_metrics();
        let positions = self.get_positions();
        
        AccountSummary {
            balance: account.as_ref().map(|a| a.balance).unwrap_or(0.0),
            available: account.as_ref().map(|a| a.available).unwrap_or(0.0),
            margin: stats.curr_margin,
            position_profit: stats.position_profit,
            close_profit: stats.close_profit,
            today_profit: stats.today_profit,
            total_profit: stats.total_profit,
            risk_ratio: metrics.risk_ratio,
            position_count: positions.len(),
            last_update: *self.last_update.lock().unwrap(),
        }
    }
}

/// 风险状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskStatus {
    /// 正常
    Normal,
    /// 警戒
    Warning,
    /// 强平
    ForceClose,
}

/// 账户摘要
#[derive(Debug, Clone)]
pub struct AccountSummary {
    /// 账户余额
    pub balance: f64,
    /// 可用资金
    pub available: f64,
    /// 占用保证金
    pub margin: f64,
    /// 持仓盈亏
    pub position_profit: f64,
    /// 平仓盈亏
    pub close_profit: f64,
    /// 今日盈亏
    pub today_profit: f64,
    /// 累计盈亏
    pub total_profit: f64,
    /// 风险度
    pub risk_ratio: f64,
    /// 持仓数量
    pub position_count: usize,
    /// 最后更新时间
    pub last_update: Option<Instant>,
}
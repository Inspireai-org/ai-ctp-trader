use crate::ctp::{
    CtpError, Position, PositionDirection, OrderDirection, OffsetFlag,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tracing::{info, warn, debug};

/// 持仓管理器
pub struct PositionManager {
    /// 持仓映射表 (instrument_id -> direction -> position)
    positions: Arc<Mutex<HashMap<String, HashMap<PositionDirection, PositionDetail>>>>,
    /// 持仓统计
    stats: Arc<Mutex<PositionStats>>,
}

/// 持仓详情
#[derive(Debug, Clone)]
pub struct PositionDetail {
    /// 基础持仓信息
    pub position: Position,
    /// 今仓可平
    pub today_closeable: i32,
    /// 昨仓可平
    pub yesterday_closeable: i32,
    /// 冻结数量
    pub frozen_volume: i32,
    /// 平均开仓价
    pub avg_open_price: f64,
    /// 最新价
    pub last_price: f64,
    /// 浮动盈亏
    pub floating_pnl: f64,
}

/// 持仓统计
#[derive(Debug, Clone, Default)]
pub struct PositionStats {
    /// 总持仓数量
    pub total_positions: i32,
    /// 多头持仓数
    pub long_positions: i32,
    /// 空头持仓数
    pub short_positions: i32,
    /// 总占用保证金
    pub total_margin: f64,
    /// 总浮动盈亏
    pub total_floating_pnl: f64,
    /// 总平仓盈亏
    pub total_close_pnl: f64,
    /// 持仓合约数
    pub instrument_count: usize,
}

impl PositionManager {
    pub fn new() -> Self {
        Self {
            positions: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(PositionStats::default())),
        }
    }

    /// 更新持仓
    pub fn update_position(&self, position: Position) -> Result<(), CtpError> {
        let mut positions = self.positions.lock().unwrap();
        
        let instrument_positions = positions
            .entry(position.instrument_id.clone())
            .or_insert_with(HashMap::new);
        
        let detail = PositionDetail {
            today_closeable: position.today_position,
            yesterday_closeable: position.yesterday_position,
            frozen_volume: 0,
            avg_open_price: if position.total_position > 0 {
                position.position_cost / (position.total_position as f64)
            } else {
                0.0
            },
            last_price: 0.0,
            floating_pnl: position.unrealized_pnl,
            position: position.clone(),
        };
        
        instrument_positions.insert(position.direction, detail);
        
        // 更新统计
        self.update_stats();
        
        debug!("持仓更新: {} {:?} 总={} 今={} 昨={}", 
            position.instrument_id, position.direction,
            position.total_position, position.today_position, position.yesterday_position);
        
        Ok(())
    }

    /// 批量更新持仓
    pub fn update_positions(&self, positions: Vec<Position>) -> Result<(), CtpError> {
        for position in positions {
            self.update_position(position)?;
        }
        Ok(())
    }

    /// 获取可平仓数量
    pub fn get_closeable_volume(
        &self,
        instrument_id: &str,
        direction: OrderDirection,
        offset_flag: OffsetFlag,
    ) -> Result<i32, CtpError> {
        let positions = self.positions.lock().unwrap();
        
        // 平仓方向相反
        let position_direction = match direction {
            OrderDirection::Buy => PositionDirection::Short,
            OrderDirection::Sell => PositionDirection::Long,
        };
        
        let instrument_positions = positions
            .get(instrument_id)
            .ok_or_else(|| CtpError::NotFound(format!("无持仓: {}", instrument_id)))?;
        
        let detail = instrument_positions
            .get(&position_direction)
            .ok_or_else(|| CtpError::NotFound(format!("无{}持仓", position_direction)))?;
        
        let closeable = match offset_flag {
            OffsetFlag::Close => detail.today_closeable + detail.yesterday_closeable,
            OffsetFlag::CloseToday => detail.today_closeable,
            OffsetFlag::CloseYesterday => detail.yesterday_closeable,
            OffsetFlag::Open => 0,
        };
        
        Ok(closeable - detail.frozen_volume)
    }

    /// 冻结持仓
    pub fn freeze_position(
        &self,
        instrument_id: &str,
        direction: PositionDirection,
        volume: i32,
    ) -> Result<(), CtpError> {
        let mut positions = self.positions.lock().unwrap();
        
        let instrument_positions = positions
            .get_mut(instrument_id)
            .ok_or_else(|| CtpError::NotFound(format!("无持仓: {}", instrument_id)))?;
        
        let detail = instrument_positions
            .get_mut(&direction)
            .ok_or_else(|| CtpError::NotFound(format!("无{}持仓", direction)))?;
        
        let available = detail.today_closeable + detail.yesterday_closeable - detail.frozen_volume;
        if volume > available {
            return Err(CtpError::ValidationError(
                format!("可平仓不足: 需要{}, 可用{}", volume, available)
            ));
        }
        
        detail.frozen_volume += volume;
        
        debug!("冻结持仓: {} {:?} {}手", instrument_id, direction, volume);
        
        Ok(())
    }

    /// 解冻持仓
    pub fn unfreeze_position(
        &self,
        instrument_id: &str,
        direction: PositionDirection,
        volume: i32,
    ) -> Result<(), CtpError> {
        let mut positions = self.positions.lock().unwrap();
        
        if let Some(instrument_positions) = positions.get_mut(instrument_id) {
            if let Some(detail) = instrument_positions.get_mut(&direction) {
                detail.frozen_volume = (detail.frozen_volume - volume).max(0);
                debug!("解冻持仓: {} {:?} {}手", instrument_id, direction, volume);
            }
        }
        
        Ok(())
    }

    /// 更新最新价
    pub fn update_last_price(&self, instrument_id: &str, price: f64) {
        let mut positions = self.positions.lock().unwrap();
        
        if let Some(instrument_positions) = positions.get_mut(instrument_id) {
            for (direction, detail) in instrument_positions.iter_mut() {
                detail.last_price = price;
                
                // 重新计算浮动盈亏
                let multiplier = 10.0; // TODO: 从合约信息获取
                let volume = detail.position.total_position as f64;
                
                detail.floating_pnl = match direction {
                    PositionDirection::Long => {
                        (price - detail.avg_open_price) * volume * multiplier
                    }
                    PositionDirection::Short => {
                        (detail.avg_open_price - price) * volume * multiplier
                    }
                };
                
                detail.position.unrealized_pnl = detail.floating_pnl;
            }
        }
        
        self.update_stats();
    }

    /// 获取所有持仓
    pub fn get_all_positions(&self) -> Vec<PositionDetail> {
        let positions = self.positions.lock().unwrap();
        
        positions
            .values()
            .flat_map(|instrument_positions| instrument_positions.values().cloned())
            .collect()
    }

    /// 获取指定合约持仓
    pub fn get_position(
        &self,
        instrument_id: &str,
        direction: PositionDirection,
    ) -> Option<PositionDetail> {
        self.positions.lock().unwrap()
            .get(instrument_id)?
            .get(&direction)
            .cloned()
    }

    /// 获取合约所有方向持仓
    pub fn get_instrument_positions(&self, instrument_id: &str) -> Vec<PositionDetail> {
        self.positions.lock().unwrap()
            .get(instrument_id)
            .map(|positions| positions.values().cloned().collect())
            .unwrap_or_default()
    }

    /// 获取持仓统计
    pub fn get_stats(&self) -> PositionStats {
        self.stats.lock().unwrap().clone()
    }

    /// 清空持仓
    pub fn clear(&self) {
        self.positions.lock().unwrap().clear();
        *self.stats.lock().unwrap() = PositionStats::default();
        info!("清空所有持仓");
    }

    /// 更新统计信息
    fn update_stats(&self) {
        let positions = self.positions.lock().unwrap();
        let mut stats = PositionStats::default();
        
        stats.instrument_count = positions.len();
        
        for instrument_positions in positions.values() {
            for (direction, detail) in instrument_positions {
                stats.total_positions += detail.position.total_position;
                stats.total_margin += detail.position.margin;
                stats.total_floating_pnl += detail.floating_pnl;
                stats.total_close_pnl += detail.position.realized_pnl;
                
                match direction {
                    PositionDirection::Long => stats.long_positions += detail.position.total_position,
                    PositionDirection::Short => stats.short_positions += detail.position.total_position,
                }
            }
        }
        
        *self.stats.lock().unwrap() = stats;
    }

    /// 获取净持仓
    pub fn get_net_position(&self, instrument_id: &str) -> i32 {
        let positions = self.positions.lock().unwrap();
        
        if let Some(instrument_positions) = positions.get(instrument_id) {
            let long = instrument_positions
                .get(&PositionDirection::Long)
                .map(|d| d.position.total_position)
                .unwrap_or(0);
                
            let short = instrument_positions
                .get(&PositionDirection::Short)
                .map(|d| d.position.total_position)
                .unwrap_or(0);
                
            long - short
        } else {
            0
        }
    }
}
import { create } from 'zustand';
import { subscribeWithSelector, persist } from 'zustand/middleware';
import {
  OrderRequest,
  OrderAction,
  OrderStatus,
  TradeRecord,
  Position,
  AccountInfo,
  OrderDirection,
  OffsetFlag,
  OrderType,
  TimeCondition,
  OrderStatusType,
  PositionDirection,
  CtpError,
  CtpErrorType,
  AppEventType,
} from '../types';
import { eventBus } from './eventBus';

/**
 * 交易统计信息
 */
export interface TradingStats {
  /** 今日下单数 */
  todayOrderCount: number;
  /** 今日成交数 */
  todayTradeCount: number;
  /** 今日成交金额 */
  todayTurnover: number;
  /** 今日盈亏 */
  todayPnl: number;
  /** 总盈亏 */
  totalPnl: number;
  /** 胜率 */
  winRate: number;
  /** 最大回撤 */
  maxDrawdown: number;
}

/**
 * 快速交易配置
 */
export interface QuickTradeConfig {
  /** 默认手数 */
  defaultVolume: number;
  /** 是否启用一键平仓 */
  enableOneClickClose: boolean;
  /** 是否启用反手 */
  enableReverse: boolean;
  /** 默认订单类型 */
  defaultOrderType: OrderType;
  /** 默认时间条件 */
  defaultTimeCondition: TimeCondition;
  /** 价格偏移档位 */
  priceOffset: number;
}

/**
 * 风险控制配置
 */
export interface RiskControlConfig {
  /** 最大持仓手数 */
  maxPosition: number;
  /** 最大单笔下单手数 */
  maxOrderVolume: number;
  /** 最大日内亏损 */
  maxDailyLoss: number;
  /** 风险度预警阈值 */
  riskRatioWarning: number;
  /** 风险度强平阈值 */
  riskRatioForceClose: number;
  /** 是否启用风险控制 */
  enableRiskControl: boolean;
}

/**
 * 交易状态接口
 */
interface TradingState {
  // 账户信息
  accountInfo: AccountInfo | null;
  lastAccountUpdateTime: number | null;
  
  // 订单管理
  orders: Map<string, OrderStatus>;
  pendingOrders: Map<string, OrderRequest>; // 待提交的订单
  
  // 持仓管理
  positions: Map<string, Position>;
  
  // 成交记录
  trades: Map<string, TradeRecord>;
  todayTrades: TradeRecord[];
  
  // 交易统计
  stats: TradingStats;
  
  // 配置
  quickTradeConfig: QuickTradeConfig;
  riskControlConfig: RiskControlConfig;
  
  // 状态标志
  isTrading: boolean;
  lastError: CtpError | null;
  
  // Actions - 账户管理
  updateAccountInfo: (account: AccountInfo) => void;
  refreshAccountInfo: () => Promise<void>;
  
  // Actions - 订单管理
  submitOrder: (order: OrderRequest) => Promise<string>;
  cancelOrder: (orderId: string) => Promise<void>;
  cancelAllOrders: (instrumentId?: string) => Promise<void>;
  updateOrderStatus: (order: OrderStatus) => void;
  
  // Actions - 持仓管理
  updatePosition: (position: Position) => void;
  updatePositions: (positions: Position[]) => void;
  closePosition: (instrumentId: string, direction: PositionDirection, volume?: number) => Promise<void>;
  closeAllPositions: (instrumentId?: string) => Promise<void>;
  
  // Actions - 成交记录
  addTradeRecord: (trade: TradeRecord) => void;
  updateTradeRecords: (trades: TradeRecord[]) => void;
  
  // Actions - 快速交易
  quickBuy: (instrumentId: string, price?: number, volume?: number) => Promise<string>;
  quickSell: (instrumentId: string, price?: number, volume?: number) => Promise<string>;
  quickClose: (instrumentId: string, direction: PositionDirection) => Promise<void>;
  quickReverse: (instrumentId: string, currentDirection: PositionDirection) => Promise<void>;
  
  // Actions - 配置管理
  updateQuickTradeConfig: (config: Partial<QuickTradeConfig>) => void;
  updateRiskControlConfig: (config: Partial<RiskControlConfig>) => void;
  
  // Actions - 数据查询
  getOrder: (orderId: string) => OrderStatus | undefined;
  getPosition: (instrumentId: string, direction?: PositionDirection) => Position | undefined;
  getPositions: (instrumentId?: string) => Position[];
  getTodayTrades: (instrumentId?: string) => TradeRecord[];
  
  // Actions - 风险控制
  checkRiskControl: (order: OrderRequest) => { allowed: boolean; reason?: string };
  calculateRiskRatio: () => number;
  
  // Actions - 统计计算
  updateStats: () => void;
  
  // Actions - 错误处理
  setError: (error: CtpError) => void;
  clearError: () => void;
  
  // Actions - 重置
  reset: () => void;
}

/**
 * 默认配置
 */
const defaultQuickTradeConfig: QuickTradeConfig = {
  defaultVolume: 1,
  enableOneClickClose: true,
  enableReverse: true,
  defaultOrderType: OrderType.LIMIT,
  defaultTimeCondition: TimeCondition.GFD,
  priceOffset: 1,
};

const defaultRiskControlConfig: RiskControlConfig = {
  maxPosition: 100,
  maxOrderVolume: 10,
  maxDailyLoss: 10000,
  riskRatioWarning: 0.8,
  riskRatioForceClose: 0.95,
  enableRiskControl: true,
};

const defaultStats: TradingStats = {
  todayOrderCount: 0,
  todayTradeCount: 0,
  todayTurnover: 0,
  todayPnl: 0,
  totalPnl: 0,
  winRate: 0,
  maxDrawdown: 0,
};

/**
 * 交易状态管理 Store
 */
export const useTradingStore = create<TradingState>()(
  persist(
    subscribeWithSelector((set, get) => ({
    // 初始状态
    accountInfo: null,
    lastAccountUpdateTime: null,
    orders: new Map(),
    pendingOrders: new Map(),
    positions: new Map(),
    trades: new Map(),
    todayTrades: [],
    stats: defaultStats,
    quickTradeConfig: defaultQuickTradeConfig,
    riskControlConfig: defaultRiskControlConfig,
    isTrading: false,
    lastError: null,

    // 账户管理
    updateAccountInfo: (account) => {
      set({
        accountInfo: account,
        lastAccountUpdateTime: Date.now(),
      });
      
      // 发布账户信息更新事件
      eventBus.emit(AppEventType.ACCOUNT_UPDATED, account);
      
      // 更新统计信息
      get().updateStats();
    },

    refreshAccountInfo: async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const account = await invoke<AccountInfo>('query_account_info');
        get().updateAccountInfo(account);
      } catch (error) {
        const ctpError: CtpError = {
          type: CtpErrorType.CTP_API_ERROR,
          message: error instanceof Error ? error.message : '查询账户信息失败',
          timestamp: Date.now(),
        };
        get().setError(ctpError);
        throw error;
      }
    },

    // 订单管理
    submitOrder: async (order) => {
      const { riskControlConfig, checkRiskControl } = get();
      
      // 风险控制检查
      if (riskControlConfig.enableRiskControl) {
        const riskCheck = checkRiskControl(order);
        if (!riskCheck.allowed) {
          const error: CtpError = {
            type: CtpErrorType.STATE_ERROR,
            message: `风险控制: ${riskCheck.reason}`,
            timestamp: Date.now(),
          };
          get().setError(error);
          throw new Error(riskCheck.reason);
        }
      }

      try {
        set({ isTrading: true });
        
        // 生成临时订单ID
        const tempOrderId = `temp_${Date.now()}`;
        const { pendingOrders } = get();
        const updatedPendingOrders = new Map(pendingOrders);
        updatedPendingOrders.set(tempOrderId, order);
        set({ pendingOrders: updatedPendingOrders });

        // 调用 Tauri 命令提交订单
        const { invoke } = await import('@tauri-apps/api/core');
        const orderId = await invoke<string>('submit_order', { order });
        
        // 移除待提交订单，添加到正式订单列表
        updatedPendingOrders.delete(tempOrderId);
        set({ pendingOrders: updatedPendingOrders });
        
        // 更新统计
        const { stats } = get();
        set({
          stats: {
            ...stats,
            todayOrderCount: stats.todayOrderCount + 1,
          },
        });
        
        return orderId;
        
      } catch (error) {
        const ctpError: CtpError = {
          type: CtpErrorType.CTP_API_ERROR,
          message: error instanceof Error ? error.message : '提交订单失败',
          timestamp: Date.now(),
        };
        get().setError(ctpError);
        throw error;
      } finally {
        set({ isTrading: false });
      }
    },

    cancelOrder: async (orderId) => {
      try {
        const { orders } = get();
        const order = orders.get(orderId);
        if (!order) {
          throw new Error('订单不存在');
        }

        const action: OrderAction = {
          orderId,
          instrumentId: order.instrumentId,
          actionFlag: 'Delete' as any,
        };

        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('cancel_order', { action });
        
      } catch (error) {
        const ctpError: CtpError = {
          type: CtpErrorType.CTP_API_ERROR,
          message: error instanceof Error ? error.message : '撤单失败',
          timestamp: Date.now(),
        };
        get().setError(ctpError);
        throw error;
      }
    },

    cancelAllOrders: async (instrumentId) => {
      const { orders } = get();
      const ordersToCancel = Array.from(orders.values()).filter(order => {
        const isActive = order.status === OrderStatusType.NO_TRADE_QUEUEING || 
                        order.status === OrderStatusType.PART_TRADED_QUEUEING;
        return isActive && (!instrumentId || order.instrumentId === instrumentId);
      });

      const cancelPromises = ordersToCancel.map(order => get().cancelOrder(order.orderId));
      
      try {
        await Promise.all(cancelPromises);
      } catch (error) {
        // 部分撤单可能失败，这里只记录错误但不抛出
        console.error('批量撤单部分失败:', error);
      }
    },

    updateOrderStatus: (order) => {
      const { orders } = get();
      const updatedOrders = new Map(orders);
      updatedOrders.set(order.orderId, order);
      set({ orders: updatedOrders });

      // 发布订单状态更新事件
      eventBus.emit(AppEventType.ORDER_UPDATED, order);
    },

    // 持仓管理
    updatePosition: (position) => {
      const { positions } = get();
      const updatedPositions = new Map(positions);
      const key = `${position.instrumentId}_${position.direction}`;
      updatedPositions.set(key, position);
      set({ positions: updatedPositions });
      
      // 发布持仓更新事件
      eventBus.emit(AppEventType.POSITION_UPDATED, position);
      
      // 更新统计信息
      get().updateStats();
    },

    updatePositions: (positionList) => {
      const updatedPositions = new Map();
      positionList.forEach(position => {
        const key = `${position.instrumentId}_${position.direction}`;
        updatedPositions.set(key, position);
      });
      set({ positions: updatedPositions });
      
      // 更新统计信息
      get().updateStats();
    },

    closePosition: async (instrumentId, direction, volume) => {
      try {
        const { positions } = get();
        const key = `${instrumentId}_${direction}`;
        const position = positions.get(key);
        
        if (!position) {
          throw new Error('持仓不存在');
        }

        const closeVolume = volume || position.totalPosition;
        const closeDirection = direction === PositionDirection.LONG ? OrderDirection.SELL : OrderDirection.BUY;
        
        const order: OrderRequest = {
          instrumentId,
          direction: closeDirection,
          offsetFlag: OffsetFlag.CLOSE,
          price: 0, // 市价单
          volume: closeVolume,
          orderType: OrderType.MARKET,
          timeCondition: TimeCondition.IOC,
        };

        await get().submitOrder(order);
        
      } catch (error) {
        const ctpError: CtpError = {
          type: CtpErrorType.CTP_API_ERROR,
          message: error instanceof Error ? error.message : '平仓失败',
          timestamp: Date.now(),
        };
        get().setError(ctpError);
        throw error;
      }
    },

    closeAllPositions: async (instrumentId) => {
      const { positions } = get();
      const positionsToClose = Array.from(positions.values()).filter(position => {
        return position.totalPosition > 0 && (!instrumentId || position.instrumentId === instrumentId);
      });

      const closePromises = positionsToClose.map(position => 
        get().closePosition(position.instrumentId, position.direction)
      );
      
      try {
        await Promise.all(closePromises);
      } catch (error) {
        console.error('批量平仓部分失败:', error);
      }
    },

    // 成交记录
    addTradeRecord: (trade) => {
      const { trades, todayTrades } = get();
      const updatedTrades = new Map(trades);
      updatedTrades.set(trade.tradeId, trade);
      
      // 检查是否是今日成交
      const today = new Date().toDateString();
      const tradeDate = new Date(trade.tradeTime).toDateString();
      const updatedTodayTrades = tradeDate === today ? [...todayTrades, trade] : todayTrades;
      
      set({ 
        trades: updatedTrades,
        todayTrades: updatedTodayTrades,
      });
      
      // 发布成交记录更新事件
      eventBus.emit(AppEventType.TRADE_UPDATED, trade);
      
      // 更新统计
      const { stats } = get();
      if (tradeDate === today) {
        set({
          stats: {
            ...stats,
            todayTradeCount: stats.todayTradeCount + 1,
            todayTurnover: stats.todayTurnover + (trade.price * trade.volume),
          },
        });
      }
      
      get().updateStats();
    },

    updateTradeRecords: (tradeList) => {
      const updatedTrades = new Map();
      const today = new Date().toDateString();
      const updatedTodayTrades: TradeRecord[] = [];
      
      tradeList.forEach(trade => {
        updatedTrades.set(trade.tradeId, trade);
        const tradeDate = new Date(trade.tradeTime).toDateString();
        if (tradeDate === today) {
          updatedTodayTrades.push(trade);
        }
      });
      
      set({ 
        trades: updatedTrades,
        todayTrades: updatedTodayTrades,
      });
      
      get().updateStats();
    },

    // 快速交易
    quickBuy: async (instrumentId, price, volume) => {
      const { quickTradeConfig } = get();
      
      const order: OrderRequest = {
        instrumentId,
        direction: OrderDirection.BUY,
        offsetFlag: OffsetFlag.OPEN,
        price: price || 0,
        volume: volume || quickTradeConfig.defaultVolume,
        orderType: price ? OrderType.LIMIT : OrderType.MARKET,
        timeCondition: quickTradeConfig.defaultTimeCondition,
      };

      return get().submitOrder(order);
    },

    quickSell: async (instrumentId, price, volume) => {
      const { quickTradeConfig } = get();
      
      const order: OrderRequest = {
        instrumentId,
        direction: OrderDirection.SELL,
        offsetFlag: OffsetFlag.OPEN,
        price: price || 0,
        volume: volume || quickTradeConfig.defaultVolume,
        orderType: price ? OrderType.LIMIT : OrderType.MARKET,
        timeCondition: quickTradeConfig.defaultTimeCondition,
      };

      return get().submitOrder(order);
    },

    quickClose: async (instrumentId, direction) => {
      await get().closePosition(instrumentId, direction);
    },

    quickReverse: async (instrumentId, currentDirection) => {
      const { positions } = get();
      const key = `${instrumentId}_${currentDirection}`;
      const position = positions.get(key);
      
      if (!position) {
        throw new Error('持仓不存在');
      }

      // 先平仓
      await get().closePosition(instrumentId, currentDirection);
      
      // 再反向开仓
      const reverseDirection = currentDirection === PositionDirection.LONG ? 
        OrderDirection.SELL : OrderDirection.BUY;
      
      const order: OrderRequest = {
        instrumentId,
        direction: reverseDirection,
        offsetFlag: OffsetFlag.OPEN,
        price: 0, // 市价单
        volume: position.totalPosition,
        orderType: OrderType.MARKET,
        timeCondition: TimeCondition.IOC,
      };

      await get().submitOrder(order);
    },

    // 配置管理
    updateQuickTradeConfig: (config) => {
      const { quickTradeConfig } = get();
      set({ quickTradeConfig: { ...quickTradeConfig, ...config } });
    },

    updateRiskControlConfig: (config) => {
      const { riskControlConfig } = get();
      set({ riskControlConfig: { ...riskControlConfig, ...config } });
    },

    // 数据查询
    getOrder: (orderId) => {
      const { orders } = get();
      return orders.get(orderId);
    },

    getPosition: (instrumentId, direction) => {
      const { positions } = get();
      if (direction) {
        const key = `${instrumentId}_${direction}`;
        return positions.get(key);
      } else {
        // 返回该合约的任意方向持仓
        return Array.from(positions.values()).find(pos => pos.instrumentId === instrumentId);
      }
    },

    getPositions: (instrumentId) => {
      const { positions } = get();
      const result = Array.from(positions.values());
      return instrumentId ? result.filter(pos => pos.instrumentId === instrumentId) : result;
    },

    getTodayTrades: (instrumentId) => {
      const { todayTrades } = get();
      return instrumentId ? todayTrades.filter(trade => trade.instrumentId === instrumentId) : todayTrades;
    },

    // 风险控制
    checkRiskControl: (order) => {
      const { riskControlConfig, accountInfo } = get();
      
      if (!riskControlConfig.enableRiskControl) {
        return { allowed: true };
      }

      // 检查单笔下单手数
      if (order.volume > riskControlConfig.maxOrderVolume) {
        return { 
          allowed: false, 
          reason: `单笔下单手数超限，最大允许${riskControlConfig.maxOrderVolume}手` 
        };
      }

      // 检查最大持仓
      const currentPosition = get().getPosition(order.instrumentId, 
        order.direction === OrderDirection.BUY ? PositionDirection.LONG : PositionDirection.SHORT
      );
      const currentVolume = currentPosition?.totalPosition || 0;
      
      if (order.offsetFlag === OffsetFlag.OPEN && 
          currentVolume + order.volume > riskControlConfig.maxPosition) {
        return { 
          allowed: false, 
          reason: `持仓手数超限，最大允许${riskControlConfig.maxPosition}手` 
        };
      }

      // 检查风险度
      if (accountInfo) {
        const riskRatio = get().calculateRiskRatio();
        if (riskRatio > riskControlConfig.riskRatioWarning) {
          return { 
            allowed: false, 
            reason: `风险度过高(${(riskRatio * 100).toFixed(2)}%)，请降低仓位` 
          };
        }
      }

      return { allowed: true };
    },

    calculateRiskRatio: () => {
      const { accountInfo } = get();
      if (!accountInfo || accountInfo.balance <= 0) {
        return 0;
      }
      return accountInfo.currMargin / accountInfo.balance;
    },

    // 统计计算
    updateStats: () => {
      const { todayTrades, positions, accountInfo } = get();
      
      // 计算今日盈亏
      const todayPnl = todayTrades.reduce((sum, trade) => {
        // 这里需要根据开平仓计算实际盈亏，简化处理
        return sum + (trade.direction === OrderDirection.BUY ? -1 : 1) * trade.price * trade.volume;
      }, 0);

      // 计算持仓盈亏
      const positionPnl = Array.from(positions.values()).reduce((sum, pos) => {
        return sum + pos.unrealizedPnl;
      }, 0);

      // 计算胜率（简化处理）
      const winTrades = todayTrades.filter(trade => {
        // 这里需要更复杂的逻辑来判断盈亏，简化处理
        return trade.direction === OrderDirection.SELL; // 假设卖出为盈利
      }).length;
      const winRate = todayTrades.length > 0 ? winTrades / todayTrades.length : 0;

      const updatedStats: TradingStats = {
        todayOrderCount: get().stats.todayOrderCount,
        todayTradeCount: todayTrades.length,
        todayTurnover: todayTrades.reduce((sum, trade) => sum + trade.price * trade.volume, 0),
        todayPnl,
        totalPnl: (accountInfo?.closeProfit || 0) + positionPnl,
        winRate,
        maxDrawdown: 0, // 需要历史数据计算
      };

      set({ stats: updatedStats });
    },

    // 错误处理
    setError: (error) => {
      set({ lastError: error });
      
      // 发布错误事件
      eventBus.emit(AppEventType.ERROR_OCCURRED, error);
    },

    clearError: () => {
      set({ lastError: null });
    },

    // 重置
    reset: () => {
      set({
        accountInfo: null,
        lastAccountUpdateTime: null,
        orders: new Map(),
        pendingOrders: new Map(),
        positions: new Map(),
        trades: new Map(),
        todayTrades: [],
        stats: defaultStats,
        quickTradeConfig: defaultQuickTradeConfig,
        riskControlConfig: defaultRiskControlConfig,
        isTrading: false,
        lastError: null,
      });
    },
  })),
  {
    name: 'trading-store',
    partialize: (state) => ({
      // 持久化配置和统计信息
      quickTradeConfig: state.quickTradeConfig,
      riskControlConfig: state.riskControlConfig,
      stats: state.stats,
      // 不持久化实时交易数据，重启后需要重新查询
    }),
    onRehydrateStorage: () => (state) => {
      if (state) {
        // 重置运行时状态
        state.accountInfo = null;
        state.lastAccountUpdateTime = null;
        state.orders = new Map();
        state.pendingOrders = new Map();
        state.positions = new Map();
        state.trades = new Map();
        state.todayTrades = [];
        state.isTrading = false;
        state.lastError = null;
      }
    },
  })
);

/**
 * 交易相关的 Hook
 */
export const useTrading = () => {
  const store = useTradingStore();
  
  return {
    // 状态
    accountInfo: store.accountInfo,
    isTrading: store.isTrading,
    lastError: store.lastError,
    stats: store.stats,
    
    // 数据
    orders: store.orders,
    positions: store.positions,
    todayTrades: store.todayTrades,
    
    // 配置
    quickTradeConfig: store.quickTradeConfig,
    riskControlConfig: store.riskControlConfig,
    
    // 方法
    submitOrder: store.submitOrder,
    cancelOrder: store.cancelOrder,
    cancelAllOrders: store.cancelAllOrders,
    closePosition: store.closePosition,
    closeAllPositions: store.closeAllPositions,
    
    // 快速交易
    quickBuy: store.quickBuy,
    quickSell: store.quickSell,
    quickClose: store.quickClose,
    quickReverse: store.quickReverse,
    
    // 查询
    getOrder: store.getOrder,
    getPosition: store.getPosition,
    getPositions: store.getPositions,
    getTodayTrades: store.getTodayTrades,
    
    // 风险控制
    checkRiskControl: store.checkRiskControl,
    calculateRiskRatio: store.calculateRiskRatio,
    
    // 配置
    updateQuickTradeConfig: store.updateQuickTradeConfig,
    updateRiskControlConfig: store.updateRiskControlConfig,
    
    // 错误处理
    clearError: store.clearError,
  };
};

/**
 * 获取特定合约持仓的 Hook
 */
export const useInstrumentPositions = (instrumentId: string) => {
  return useTradingStore(state => {
    return Array.from(state.positions.values()).filter(pos => pos.instrumentId === instrumentId);
  });
};

/**
 * 获取活跃订单的 Hook
 */
export const useActiveOrders = (instrumentId?: string) => {
  return useTradingStore(state => {
    const activeOrders = Array.from(state.orders.values()).filter(order => {
      const isActive = order.status === OrderStatusType.NO_TRADE_QUEUEING || 
                      order.status === OrderStatusType.PART_TRADED_QUEUEING;
      return isActive && (!instrumentId || order.instrumentId === instrumentId);
    });
    return activeOrders;
  });
};
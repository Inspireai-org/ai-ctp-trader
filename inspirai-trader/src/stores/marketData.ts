import { create } from 'zustand';
import { subscribeWithSelector, persist } from 'zustand/middleware';
import { 
  MarketDataTick, 
  KlineData, 
  TimeFrame, 
  ConnectionStatus, 
  ContractInfo,
  AppEventType 
} from '../types';
import { eventBus } from './eventBus';

/**
 * 行情订阅状态
 */
export interface SubscriptionStatus {
  /** 合约代码 */
  instrumentId: string;
  /** 是否已订阅 */
  subscribed: boolean;
  /** 订阅时间 */
  subscribeTime?: number;
  /** 最后更新时间 */
  lastUpdateTime?: number;
  /** 错误信息 */
  error?: string;
}

/**
 * 行情数据状态接口
 */
interface MarketDataState {
  // 连接状态
  connectionStatus: ConnectionStatus;
  lastHeartbeat: number | null;
  
  // 行情数据
  ticks: Map<string, MarketDataTick>;
  klineData: Map<string, Map<TimeFrame, KlineData[]>>;
  
  // 订阅管理
  subscriptions: Map<string, SubscriptionStatus>;
  watchlist: string[]; // 自选合约列表
  
  // 合约信息
  contracts: Map<string, ContractInfo>;
  
  // 数据统计
  totalTicksReceived: number;
  lastTickTime: number | null;
  
  // Actions - 连接管理
  setConnectionStatus: (status: ConnectionStatus) => void;
  updateHeartbeat: () => void;
  
  // Actions - 行情数据
  updateTick: (tick: MarketDataTick) => void;
  updateKlineData: (instrumentId: string, timeframe: TimeFrame, data: KlineData[]) => void;
  appendKlineData: (instrumentId: string, timeframe: TimeFrame, data: KlineData) => void;
  
  // Actions - 订阅管理
  subscribe: (instrumentId: string) => Promise<void>;
  unsubscribe: (instrumentId: string) => Promise<void>;
  batchSubscribe: (instrumentIds: string[]) => Promise<void>;
  batchUnsubscribe: (instrumentIds: string[]) => Promise<void>;
  updateSubscriptionStatus: (instrumentId: string, status: Partial<SubscriptionStatus>) => void;
  
  // Actions - 自选管理
  addToWatchlist: (instrumentId: string) => void;
  removeFromWatchlist: (instrumentId: string) => void;
  reorderWatchlist: (instrumentIds: string[]) => void;
  
  // Actions - 合约信息
  updateContract: (contract: ContractInfo) => void;
  updateContracts: (contracts: ContractInfo[]) => void;
  
  // Actions - 数据查询
  getTick: (instrumentId: string) => MarketDataTick | undefined;
  getKlineData: (instrumentId: string, timeframe: TimeFrame) => KlineData[] | undefined;
  getLatestKline: (instrumentId: string, timeframe: TimeFrame) => KlineData | undefined;
  isSubscribed: (instrumentId: string) => boolean;
  
  // Actions - 数据清理
  clearTicks: () => void;
  clearKlineData: (instrumentId?: string) => void;
  clearOldData: (maxAge: number) => void;
  
  // Actions - 重置
  reset: () => void;
}

/**
 * 行情数据状态管理 Store
 */
export const useMarketDataStore = create<MarketDataState>()(
  persist(
    subscribeWithSelector((set, get) => ({
    // 初始状态
    connectionStatus: ConnectionStatus.DISCONNECTED,
    lastHeartbeat: null,
    ticks: new Map(),
    klineData: new Map(),
    subscriptions: new Map(),
    watchlist: [],
    contracts: new Map(),
    totalTicksReceived: 0,
    lastTickTime: null,

    // 连接管理
    setConnectionStatus: (status) => {
      const prevStatus = get().connectionStatus;
      set({ connectionStatus: status });
      
      // 发布连接状态变化事件
      if (prevStatus !== status) {
        eventBus.emit(AppEventType.CONNECTION_CHANGED, { status, prevStatus });
      }
      
      // 连接断开时清理订阅状态
      if (status === ConnectionStatus.DISCONNECTED) {
        const { subscriptions } = get();
        const updatedSubscriptions = new Map(subscriptions);
        updatedSubscriptions.forEach((sub, instrumentId) => {
          updatedSubscriptions.set(instrumentId, {
            ...sub,
            subscribed: false,
            error: '连接已断开',
          });
        });
        set({ subscriptions: updatedSubscriptions });
      }
    },

    updateHeartbeat: () => {
      set({ lastHeartbeat: Date.now() });
    },

    // 行情数据更新
    updateTick: (tick) => {
      const { ticks, totalTicksReceived } = get();
      const updatedTicks = new Map(ticks);
      updatedTicks.set(tick.instrumentId, tick);
      
      set({
        ticks: updatedTicks,
        totalTicksReceived: totalTicksReceived + 1,
        lastTickTime: Date.now(),
      });

      // 发布行情数据更新事件
      eventBus.emit(AppEventType.MARKET_DATA_UPDATED, tick);

      // 更新订阅状态的最后更新时间
      const { subscriptions } = get();
      const subscription = subscriptions.get(tick.instrumentId);
      if (subscription) {
        const updatedSubscriptions = new Map(subscriptions);
        updatedSubscriptions.set(tick.instrumentId, {
          ...subscription,
          lastUpdateTime: Date.now(),
        });
        set({ subscriptions: updatedSubscriptions });
      }
    },

    updateKlineData: (instrumentId, timeframe, data) => {
      const { klineData } = get();
      const updatedKlineData = new Map(klineData);
      
      if (!updatedKlineData.has(instrumentId)) {
        updatedKlineData.set(instrumentId, new Map());
      }
      
      const instrumentData = updatedKlineData.get(instrumentId)!;
      instrumentData.set(timeframe, [...data]);
      
      set({ klineData: updatedKlineData });

      // 发布K线数据更新事件
      eventBus.emit(AppEventType.KLINE_DATA_UPDATED, { instrumentId, timeframe, data });
    },

    appendKlineData: (instrumentId, timeframe, data) => {
      const { klineData } = get();
      const updatedKlineData = new Map(klineData);
      
      if (!updatedKlineData.has(instrumentId)) {
        updatedKlineData.set(instrumentId, new Map());
      }
      
      const instrumentData = updatedKlineData.get(instrumentId)!;
      const existingData = instrumentData.get(timeframe) || [];
      
      // 检查是否是更新最后一根K线还是添加新K线
      if (existingData.length > 0) {
        const lastKline = existingData[existingData.length - 1];
        if (lastKline && lastKline.timestamp === data.timestamp) {
          // 更新最后一根K线
          existingData[existingData.length - 1] = data;
        } else {
          // 添加新K线
          existingData.push(data);
        }
      } else {
        existingData.push(data);
      }
      
      instrumentData.set(timeframe, existingData);
      set({ klineData: updatedKlineData });
    },

    // 订阅管理
    subscribe: async (instrumentId) => {
      const { subscriptions } = get();
      const updatedSubscriptions = new Map(subscriptions);
      
      // 更新订阅状态为订阅中
      updatedSubscriptions.set(instrumentId, {
        instrumentId,
        subscribed: false,
        subscribeTime: Date.now(),
      });
      set({ subscriptions: updatedSubscriptions });

      try {
        // 调用 Tauri 命令订阅行情
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('subscribe_market_data', { instrumentIds: [instrumentId] });
        
        // 更新订阅状态为已订阅
        updatedSubscriptions.set(instrumentId, {
          instrumentId,
          subscribed: true,
          subscribeTime: Date.now(),
        });
        set({ subscriptions: updatedSubscriptions });
        
      } catch (error) {
        // 订阅失败，更新错误状态
        updatedSubscriptions.set(instrumentId, {
          instrumentId,
          subscribed: false,
          subscribeTime: Date.now(),
          error: error instanceof Error ? error.message : '订阅失败',
        });
        set({ subscriptions: updatedSubscriptions });
        throw error;
      }
    },

    unsubscribe: async (instrumentId) => {
      const { subscriptions } = get();
      const updatedSubscriptions = new Map(subscriptions);
      
      try {
        // 调用 Tauri 命令取消订阅
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('unsubscribe_market_data', { instrumentIds: [instrumentId] });
        
        // 移除订阅状态
        updatedSubscriptions.delete(instrumentId);
        set({ subscriptions: updatedSubscriptions });
        
      } catch (error) {
        // 取消订阅失败，更新错误状态
        const subscription = subscriptions.get(instrumentId);
        if (subscription) {
          updatedSubscriptions.set(instrumentId, {
            ...subscription,
            error: error instanceof Error ? error.message : '取消订阅失败',
          });
          set({ subscriptions: updatedSubscriptions });
        }
        throw error;
      }
    },

    batchSubscribe: async (instrumentIds) => {
      const { subscriptions } = get();
      const updatedSubscriptions = new Map(subscriptions);
      
      // 批量更新订阅状态
      instrumentIds.forEach(instrumentId => {
        updatedSubscriptions.set(instrumentId, {
          instrumentId,
          subscribed: false,
          subscribeTime: Date.now(),
        });
      });
      set({ subscriptions: updatedSubscriptions });

      try {
        // 调用 Tauri 命令批量订阅
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('subscribe_market_data', { instrumentIds });
        
        // 批量更新为已订阅状态
        instrumentIds.forEach(instrumentId => {
          updatedSubscriptions.set(instrumentId, {
            instrumentId,
            subscribed: true,
            subscribeTime: Date.now(),
          });
        });
        set({ subscriptions: updatedSubscriptions });
        
      } catch (error) {
        // 批量订阅失败
        instrumentIds.forEach(instrumentId => {
          updatedSubscriptions.set(instrumentId, {
            instrumentId,
            subscribed: false,
            subscribeTime: Date.now(),
            error: error instanceof Error ? error.message : '批量订阅失败',
          });
        });
        set({ subscriptions: updatedSubscriptions });
        throw error;
      }
    },

    batchUnsubscribe: async (instrumentIds) => {
      const { subscriptions } = get();
      const updatedSubscriptions = new Map(subscriptions);
      
      try {
        // 调用 Tauri 命令批量取消订阅
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('unsubscribe_market_data', { instrumentIds });
        
        // 批量移除订阅状态
        instrumentIds.forEach(instrumentId => {
          updatedSubscriptions.delete(instrumentId);
        });
        set({ subscriptions: updatedSubscriptions });
        
      } catch (error) {
        // 批量取消订阅失败
        instrumentIds.forEach(instrumentId => {
          const subscription = subscriptions.get(instrumentId);
          if (subscription) {
            updatedSubscriptions.set(instrumentId, {
              ...subscription,
              error: error instanceof Error ? error.message : '批量取消订阅失败',
            });
          }
        });
        set({ subscriptions: updatedSubscriptions });
        throw error;
      }
    },

    updateSubscriptionStatus: (instrumentId, status) => {
      const { subscriptions } = get();
      const updatedSubscriptions = new Map(subscriptions);
      const existing = subscriptions.get(instrumentId);
      
      if (existing) {
        updatedSubscriptions.set(instrumentId, { ...existing, ...status });
        set({ subscriptions: updatedSubscriptions });
      }
    },

    // 自选管理
    addToWatchlist: (instrumentId) => {
      const { watchlist } = get();
      if (!watchlist.includes(instrumentId)) {
        set({ watchlist: [...watchlist, instrumentId] });
      }
    },

    removeFromWatchlist: (instrumentId) => {
      const { watchlist } = get();
      set({ watchlist: watchlist.filter(id => id !== instrumentId) });
    },

    reorderWatchlist: (instrumentIds) => {
      set({ watchlist: instrumentIds });
    },

    // 合约信息
    updateContract: (contract) => {
      const { contracts } = get();
      const updatedContracts = new Map(contracts);
      updatedContracts.set(contract.instrumentId, contract);
      set({ contracts: updatedContracts });
    },

    updateContracts: (contractList) => {
      const { contracts } = get();
      const updatedContracts = new Map(contracts);
      contractList.forEach(contract => {
        updatedContracts.set(contract.instrumentId, contract);
      });
      set({ contracts: updatedContracts });
    },

    // 数据查询
    getTick: (instrumentId) => {
      const { ticks } = get();
      return ticks.get(instrumentId);
    },

    getKlineData: (instrumentId, timeframe) => {
      const { klineData } = get();
      const instrumentData = klineData.get(instrumentId);
      return instrumentData?.get(timeframe);
    },

    getLatestKline: (instrumentId, timeframe) => {
      const data = get().getKlineData(instrumentId, timeframe);
      return data && data.length > 0 ? data[data.length - 1] : undefined;
    },

    isSubscribed: (instrumentId) => {
      const { subscriptions } = get();
      const subscription = subscriptions.get(instrumentId);
      return subscription?.subscribed || false;
    },

    // 数据清理
    clearTicks: () => {
      set({ ticks: new Map() });
    },

    clearKlineData: (instrumentId) => {
      const { klineData } = get();
      if (instrumentId) {
        const updatedKlineData = new Map(klineData);
        updatedKlineData.delete(instrumentId);
        set({ klineData: updatedKlineData });
      } else {
        set({ klineData: new Map() });
      }
    },

    clearOldData: (maxAge) => {
      const now = Date.now();
      const { ticks, klineData } = get();
      
      // 清理过期的行情数据
      const updatedTicks = new Map();
      ticks.forEach((tick, instrumentId) => {
        const tickTime = new Date(tick.updateTime).getTime();
        if (now - tickTime < maxAge) {
          updatedTicks.set(instrumentId, tick);
        }
      });
      
      // 清理过期的K线数据
      const updatedKlineData = new Map();
      klineData.forEach((instrumentData, instrumentId) => {
        const updatedInstrumentData = new Map();
        instrumentData.forEach((data, timeframe) => {
          const filteredData = data.filter(kline => now - kline.timestamp < maxAge);
          if (filteredData.length > 0) {
            updatedInstrumentData.set(timeframe, filteredData);
          }
        });
        if (updatedInstrumentData.size > 0) {
          updatedKlineData.set(instrumentId, updatedInstrumentData);
        }
      });
      
      set({ 
        ticks: updatedTicks,
        klineData: updatedKlineData,
      });
    },

    // 重置
    reset: () => {
      set({
        connectionStatus: ConnectionStatus.DISCONNECTED,
        lastHeartbeat: null,
        ticks: new Map(),
        klineData: new Map(),
        subscriptions: new Map(),
        watchlist: [],
        contracts: new Map(),
        totalTicksReceived: 0,
        lastTickTime: null,
      });
    },
  })),
  {
    name: 'market-data-store',
    partialize: (state) => ({
      // 只持久化必要的数据
      watchlist: state.watchlist,
      subscriptions: Array.from(state.subscriptions.entries()).map(([key, value]) => [key, {
        instrumentId: value.instrumentId,
        subscribed: false, // 重启后需要重新订阅
        subscribeTime: value.subscribeTime,
      }]),
      contracts: Array.from(state.contracts.entries()),
    }),
    onRehydrateStorage: () => (state) => {
      if (state) {
        // 重新构建 Map 对象
        if (Array.isArray(state.subscriptions)) {
          state.subscriptions = new Map(state.subscriptions as any);
        }
        if (Array.isArray(state.contracts)) {
          state.contracts = new Map(state.contracts as any);
        }
        
        // 重置运行时状态
        state.connectionStatus = ConnectionStatus.DISCONNECTED;
        state.lastHeartbeat = null;
        state.ticks = new Map();
        state.klineData = new Map();
        state.totalTicksReceived = 0;
        state.lastTickTime = null;
      }
    },
  })
);

/**
 * 行情数据相关的 Hook
 */
export const useMarketData = () => {
  const store = useMarketDataStore();
  
  return {
    // 状态
    connectionStatus: store.connectionStatus,
    lastHeartbeat: store.lastHeartbeat,
    totalTicksReceived: store.totalTicksReceived,
    lastTickTime: store.lastTickTime,
    
    // 数据
    ticks: store.ticks,
    watchlist: store.watchlist,
    subscriptions: store.subscriptions,
    
    // 方法
    getTick: store.getTick,
    getKlineData: store.getKlineData,
    getLatestKline: store.getLatestKline,
    isSubscribed: store.isSubscribed,
    
    // 订阅管理
    subscribe: store.subscribe,
    unsubscribe: store.unsubscribe,
    batchSubscribe: store.batchSubscribe,
    batchUnsubscribe: store.batchUnsubscribe,
    
    // 自选管理
    addToWatchlist: store.addToWatchlist,
    removeFromWatchlist: store.removeFromWatchlist,
    reorderWatchlist: store.reorderWatchlist,
  };
};

/**
 * 获取特定合约行情的 Hook
 */
export const useInstrumentTick = (instrumentId: string) => {
  return useMarketDataStore(state => state.ticks.get(instrumentId));
};

/**
 * 获取特定合约K线数据的 Hook
 */
export const useInstrumentKline = (instrumentId: string, timeframe: TimeFrame) => {
  return useMarketDataStore(state => {
    const instrumentData = state.klineData.get(instrumentId);
    return instrumentData?.get(timeframe);
  });
};

/**
 * 获取自选列表行情的 Hook
 */
export const useWatchlistTicks = () => {
  return useMarketDataStore(state => {
    const watchlistTicks: MarketDataTick[] = [];
    state.watchlist.forEach(instrumentId => {
      const tick = state.ticks.get(instrumentId);
      if (tick) {
        watchlistTicks.push(tick);
      }
    });
    return watchlistTicks;
  });
};
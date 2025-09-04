// UI 状态管理
export { useUIStore, useTheme, useLayout, usePreferences } from './ui';

// 行情数据状态管理
export { 
  useMarketDataStore, 
  useMarketData, 
  useInstrumentTick, 
  useInstrumentKline, 
  useWatchlistTicks 
} from './marketData';

// 交易状态管理
export { 
  useTradingStore, 
  useTrading, 
  useInstrumentPositions, 
  useActiveOrders 
} from './trading';

// 事件总线
export { 
  eventBus, 
  useEventListener, 
  useEventEmitter, 
  useEventHistory 
} from './eventBus';

// 状态管理器
export { 
  stateManager, 
  useStateManager 
} from './stateManager';

// 初始化和系统状态
export { 
  initializeStores, 
  cleanupStores, 
  getSystemStatus, 
  useSystemStatus 
} from './initialize';

// 其他 Store（后续添加）
// export { useAccountStore } from './account';

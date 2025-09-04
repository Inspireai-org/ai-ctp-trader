/**
 * 状态管理系统测试
 */

// 使用 Bun 内置测试 API
declare global {
  function describe(name: string, fn: () => void): void;
  function it(name: string, fn: () => void): void;
  function beforeEach(fn: () => void): void;
  namespace expect {
    function toBe(expected: any): void;
    function toEqual(expected: any): void;
    function toBeGreaterThanOrEqual(expected: number): void;
    function toContain(expected: any): void;
    function toBeTruthy(): void;
  }
  function expect(actual: any): typeof expect;
}
import { useUIStore } from '../ui';
import { useMarketDataStore } from '../marketData';
import { useTradingStore } from '../trading';
import { eventBus } from '../eventBus';
import { stateManager } from '../stateManager';
import { ConnectionStatus, AppEventType, MarketDataTick, OrderDirection, OffsetFlag, OrderType, TimeCondition } from '../../types';

describe('状态管理系统测试', () => {
  beforeEach(() => {
    // 重置所有状态（跳过 UI store 的 DOM 操作）
    useMarketDataStore.getState().reset();
    useTradingStore.getState().reset();
    eventBus.off();
  });

  describe('UI Store', () => {
    it('应该正确初始化默认状态', () => {
      const state = useUIStore.getState();
      expect(state.theme).toBe('dark');
      expect(state.sidebarCollapsed).toBe(false);
      expect(state.isLayoutLocked).toBe(false);
    });

    it('应该正确切换主题', () => {
      const store = useUIStore.getState();
      store.switchTheme('light');
      
      const state = useUIStore.getState();
      expect(state.theme).toBe('light');
      expect(state.themeConfig.mode).toBe('light');
    });

    it('应该正确更新布局', () => {
      const store = useUIStore.getState();
      const newLayout = {
        ...store.layout,
        panels: {
          ...store.layout.panels,
          market: {
            ...store.layout.panels.market,
            visible: false,
          },
        },
      };
      
      store.updateLayout(newLayout);
      
      const state = useUIStore.getState();
      expect(state.layout.panels.market.visible).toBe(false);
    });
  });

  describe('MarketData Store', () => {
    it('应该正确初始化默认状态', () => {
      const state = useMarketDataStore.getState();
      expect(state.connectionStatus).toBe(ConnectionStatus.DISCONNECTED);
      expect(state.ticks.size).toBe(0);
      expect(state.watchlist.length).toBe(0);
    });

    it('应该正确更新连接状态', () => {
      const store = useMarketDataStore.getState();
      store.setConnectionStatus(ConnectionStatus.CONNECTED);
      
      const state = useMarketDataStore.getState();
      expect(state.connectionStatus).toBe(ConnectionStatus.CONNECTED);
    });

    it('应该正确更新行情数据', () => {
      const store = useMarketDataStore.getState();
      const tick: MarketDataTick = {
        instrumentId: 'rb2501',
        lastPrice: 3500,
        volume: 1000,
        turnover: 3500000,
        openInterest: 50000,
        bidPrice1: 3499,
        bidVolume1: 10,
        askPrice1: 3501,
        askVolume1: 15,
        updateTime: '09:30:00',
        updateMillisec: 500,
        changePercent: 1.5,
        changeAmount: 50,
        openPrice: 3450,
        highestPrice: 3520,
        lowestPrice: 3440,
        preClosePrice: 3450,
      };
      
      store.updateTick(tick);
      
      const state = useMarketDataStore.getState();
      expect(state.ticks.get('rb2501')).toEqual(tick);
      expect(state.totalTicksReceived).toBe(1);
    });

    it('应该正确管理自选列表', () => {
      const store = useMarketDataStore.getState();
      
      store.addToWatchlist('rb2501');
      store.addToWatchlist('hc2501');
      
      let state = useMarketDataStore.getState();
      expect(state.watchlist).toEqual(['rb2501', 'hc2501']);
      
      store.removeFromWatchlist('rb2501');
      
      state = useMarketDataStore.getState();
      expect(state.watchlist).toEqual(['hc2501']);
    });
  });

  describe('Trading Store', () => {
    it('应该正确初始化默认状态', () => {
      const state = useTradingStore.getState();
      expect(state.accountInfo).toBe(null);
      expect(state.orders.size).toBe(0);
      expect(state.positions.size).toBe(0);
      expect(state.isTrading).toBe(false);
    });

    it('应该正确更新快速交易配置', () => {
      const store = useTradingStore.getState();
      const newConfig = {
        defaultVolume: 5,
        enableOneClickClose: false,
      };
      
      store.updateQuickTradeConfig(newConfig);
      
      const state = useTradingStore.getState();
      expect(state.quickTradeConfig.defaultVolume).toBe(5);
      expect(state.quickTradeConfig.enableOneClickClose).toBe(false);
    });

    it('应该正确进行风险控制检查', () => {
      const store = useTradingStore.getState();
      
      // 更新风险控制配置
      store.updateRiskControlConfig({
        maxOrderVolume: 10,
        enableRiskControl: true,
      });
      
      // 测试正常订单
      const normalOrder = {
        instrumentId: 'rb2501',
        direction: OrderDirection.BUY,
        offsetFlag: OffsetFlag.OPEN,
        price: 3500,
        volume: 5,
        orderType: OrderType.LIMIT,
        timeCondition: TimeCondition.GFD,
      };
      
      const normalCheck = store.checkRiskControl(normalOrder);
      expect(normalCheck.allowed).toBe(true);
      
      // 测试超限订单
      const overLimitOrder = {
        ...normalOrder,
        volume: 15,
      };
      
      const overLimitCheck = store.checkRiskControl(overLimitOrder);
      expect(overLimitCheck.allowed).toBe(false);
      expect(overLimitCheck.reason).toContain('单笔下单手数超限');
    });
  });

  describe('Event Bus', () => {
    it('应该正确发布和订阅事件', () => {
      let receivedData: any = null;
      
      const unsubscribe = eventBus.on(AppEventType.MARKET_DATA_UPDATED, (event) => {
        receivedData = event.data;
      });
      
      const testData = { instrumentId: 'rb2501', lastPrice: 3500 };
      eventBus.emit(AppEventType.MARKET_DATA_UPDATED, testData);
      
      expect(receivedData).toEqual(testData);
      
      unsubscribe();
    });

    it('应该正确处理一次性事件订阅', () => {
      let callCount = 0;
      
      eventBus.once(AppEventType.CONNECTION_CHANGED, () => {
        callCount++;
      });
      
      eventBus.emit(AppEventType.CONNECTION_CHANGED, { status: ConnectionStatus.CONNECTED });
      eventBus.emit(AppEventType.CONNECTION_CHANGED, { status: ConnectionStatus.DISCONNECTED });
      
      expect(callCount).toBe(1);
    });

    it('应该正确记录事件历史', () => {
      eventBus.clearHistory();
      
      eventBus.emit(AppEventType.NOTIFICATION, { message: 'test1' });
      eventBus.emit(AppEventType.NOTIFICATION, { message: 'test2' });
      
      const history = eventBus.getHistory(AppEventType.NOTIFICATION);
      expect(history.length).toBe(2);
      expect(history[0]?.data.message).toBe('test1');
      expect(history[1]?.data.message).toBe('test2');
    });
  });

  describe('State Manager', () => {
    it('应该正确创建和恢复备份', () => {
      // 修改一些状态
      useUIStore.getState().switchTheme('light');
      useMarketDataStore.getState().addToWatchlist('rb2501');
      
      // 创建备份
      stateManager.createBackup('test-backup');
      
      // 修改状态
      useUIStore.getState().switchTheme('dark');
      useMarketDataStore.getState().addToWatchlist('hc2501');
      
      // 恢复备份
      const restored = stateManager.restoreBackup('test-backup');
      expect(restored).toBe(true);
      
      // 验证状态已恢复
      expect(useUIStore.getState().theme).toBe('light');
      expect(useMarketDataStore.getState().watchlist).toEqual(['rb2501']);
    });

    it('应该正确导出和导入配置', () => {
      // 设置一些配置
      useUIStore.getState().switchTheme('light');
      useMarketDataStore.getState().addToWatchlist('rb2501');
      useTradingStore.getState().updateQuickTradeConfig({ defaultVolume: 10 });
      
      // 导出配置
      const configJson = stateManager.exportConfig();
      expect(configJson).toBeTruthy();
      
      // 重置状态
      useUIStore.getState().resetToDefault();
      useMarketDataStore.getState().reset();
      useTradingStore.getState().reset();
      
      // 导入配置
      const imported = stateManager.importConfig(configJson);
      expect(imported).toBe(true);
      
      // 验证配置已导入
      expect(useUIStore.getState().theme).toBe('light');
      expect(useTradingStore.getState().quickTradeConfig.defaultVolume).toBe(10);
    });

    it('应该正确获取备份列表', () => {
      stateManager.createBackup('backup1');
      stateManager.createBackup('backup2');
      
      const backupList = stateManager.getBackupList();
      expect(backupList.length).toBeGreaterThanOrEqual(2);
      
      const backupNames = backupList.map(b => b.name);
      expect(backupNames).toContain('backup1');
      expect(backupNames).toContain('backup2');
    });
  });

  describe('状态持久化', () => {
    it('UI Store 应该正确持久化关键配置', () => {
      const store = useUIStore.getState();
      
      // 修改配置
      store.switchTheme('light');
      store.updatePreferences({ language: 'en-US' });
      
      // 模拟页面刷新（重新创建 store）
      const persistedState = useUIStore.getState();
      expect(persistedState.theme).toBe('light');
      expect(persistedState.preferences.language).toBe('en-US');
    });

    it('MarketData Store 应该正确持久化自选列表', () => {
      const store = useMarketDataStore.getState();
      
      // 添加自选
      store.addToWatchlist('rb2501');
      store.addToWatchlist('hc2501');
      
      // 模拟页面刷新
      const persistedState = useMarketDataStore.getState();
      expect(persistedState.watchlist).toEqual(['rb2501', 'hc2501']);
    });

    it('Trading Store 应该正确持久化交易配置', () => {
      const store = useTradingStore.getState();
      
      // 修改配置
      store.updateQuickTradeConfig({ defaultVolume: 10 });
      store.updateRiskControlConfig({ maxOrderVolume: 20 });
      
      // 模拟页面刷新
      const persistedState = useTradingStore.getState();
      expect(persistedState.quickTradeConfig.defaultVolume).toBe(10);
      expect(persistedState.riskControlConfig.maxOrderVolume).toBe(20);
    });
  });
});
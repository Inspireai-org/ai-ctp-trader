import { useUIStore } from './ui';
import { useMarketDataStore } from './marketData';
import { useTradingStore } from './trading';
import { eventBus } from './eventBus';
import { AppEventType, ConnectionStatus } from '../types';

/**
 * 状态备份接口
 */
interface StateBackup {
  timestamp: number;
  version: string;
  ui: any;
  marketData: any;
  trading: any;
}

/**
 * 状态管理器类
 */
class StateManager {
  private backups: Map<string, StateBackup> = new Map();
  private maxBackups = 10;
  private version = '1.0.0';

  /**
   * 初始化状态管理器
   */
  initialize(): void {
    // 监听连接状态变化
    eventBus.on(AppEventType.CONNECTION_CHANGED, (event) => {
      const { status } = event.data;
      
      if (status === ConnectionStatus.CONNECTED) {
        this.onConnected();
      } else if (status === ConnectionStatus.DISCONNECTED) {
        this.onDisconnected();
      }
    });

    // 监听错误事件
    eventBus.on(AppEventType.ERROR_OCCURRED, (event) => {
      console.error('应用错误:', event.data);
      // 可以在这里实现错误恢复逻辑
    });

    // 定期清理过期数据
    setInterval(() => {
      this.cleanupExpiredData();
    }, 5 * 60 * 1000); // 每5分钟清理一次
  }

  /**
   * 创建状态备份
   */
  createBackup(name: string): void {
    const backup: StateBackup = {
      timestamp: Date.now(),
      version: this.version,
      ui: this.getUIState(),
      marketData: this.getMarketDataState(),
      trading: this.getTradingState(),
    };

    this.backups.set(name, backup);

    // 限制备份数量
    if (this.backups.size > this.maxBackups) {
      const oldestKey = Array.from(this.backups.keys())[0];
      if (oldestKey) {
        this.backups.delete(oldestKey);
      }
    }

    eventBus.emit(AppEventType.NOTIFICATION, {
      type: 'success',
      message: `状态备份 "${name}" 创建成功`,
    });
  }

  /**
   * 恢复状态备份
   */
  restoreBackup(name: string): boolean {
    const backup = this.backups.get(name);
    if (!backup) {
      eventBus.emit(AppEventType.ERROR_OCCURRED, {
        message: `备份 "${name}" 不存在`,
      });
      return false;
    }

    try {
      // 恢复各个状态
      this.restoreUIState(backup.ui);
      this.restoreMarketDataState(backup.marketData);
      this.restoreTradingState(backup.trading);

      eventBus.emit(AppEventType.NOTIFICATION, {
        type: 'success',
        message: `状态备份 "${name}" 恢复成功`,
      });

      return true;
    } catch (error) {
      eventBus.emit(AppEventType.ERROR_OCCURRED, {
        message: `恢复备份失败: ${error instanceof Error ? error.message : '未知错误'}`,
      });
      return false;
    }
  }

  /**
   * 删除状态备份
   */
  deleteBackup(name: string): boolean {
    const deleted = this.backups.delete(name);
    if (deleted) {
      eventBus.emit(AppEventType.NOTIFICATION, {
        type: 'info',
        message: `备份 "${name}" 已删除`,
      });
    }
    return deleted;
  }

  /**
   * 获取所有备份列表
   */
  getBackupList(): Array<{ name: string; timestamp: number; version: string }> {
    return Array.from(this.backups.entries()).map(([name, backup]) => ({
      name,
      timestamp: backup.timestamp,
      version: backup.version,
    }));
  }

  /**
   * 重置所有状态到默认值
   */
  resetAllStates(): void {
    try {
      useUIStore.getState().resetToDefault();
      useMarketDataStore.getState().reset();
      useTradingStore.getState().reset();

      eventBus.emit(AppEventType.NOTIFICATION, {
        type: 'success',
        message: '所有状态已重置到默认值',
      });
    } catch (error) {
      eventBus.emit(AppEventType.ERROR_OCCURRED, {
        message: `重置状态失败: ${error instanceof Error ? error.message : '未知错误'}`,
      });
    }
  }

  /**
   * 导出状态配置
   */
  exportConfig(): string {
    const config = {
      timestamp: Date.now(),
      version: this.version,
      ui: this.getUIState(),
      marketData: {
        watchlist: useMarketDataStore.getState().watchlist,
        subscriptions: Array.from(useMarketDataStore.getState().subscriptions.entries()),
      },
      trading: {
        quickTradeConfig: useTradingStore.getState().quickTradeConfig,
        riskControlConfig: useTradingStore.getState().riskControlConfig,
      },
    };

    return JSON.stringify(config, null, 2);
  }

  /**
   * 导入状态配置
   */
  importConfig(configJson: string): boolean {
    try {
      const config = JSON.parse(configJson);
      
      // 验证配置格式
      if (!config.version || !config.ui || !config.marketData || !config.trading) {
        throw new Error('配置格式无效');
      }

      // 导入配置
      this.restoreUIState(config.ui);
      
      // 导入行情配置
      const marketDataStore = useMarketDataStore.getState();
      if (config.marketData.watchlist) {
        config.marketData.watchlist.forEach((instrumentId: string) => {
          marketDataStore.addToWatchlist(instrumentId);
        });
      }

      // 导入交易配置
      const tradingStore = useTradingStore.getState();
      if (config.trading.quickTradeConfig) {
        tradingStore.updateQuickTradeConfig(config.trading.quickTradeConfig);
      }
      if (config.trading.riskControlConfig) {
        tradingStore.updateRiskControlConfig(config.trading.riskControlConfig);
      }

      eventBus.emit(AppEventType.NOTIFICATION, {
        type: 'success',
        message: '配置导入成功',
      });

      return true;
    } catch (error) {
      eventBus.emit(AppEventType.ERROR_OCCURRED, {
        message: `导入配置失败: ${error instanceof Error ? error.message : '未知错误'}`,
      });
      return false;
    }
  }

  /**
   * 连接成功时的处理
   */
  private onConnected(): void {
    // 自动创建连接成功时的备份
    this.createBackup(`auto_connected_${new Date().toISOString()}`);
    
    // 重新订阅行情数据
    const marketDataStore = useMarketDataStore.getState();
    const { watchlist } = marketDataStore;
    
    if (watchlist.length > 0) {
      marketDataStore.batchSubscribe(watchlist).catch(error => {
        console.error('重新订阅行情失败:', error);
      });
    }
  }

  /**
   * 连接断开时的处理
   */
  private onDisconnected(): void {
    // 清理实时数据但保留配置
    const marketDataStore = useMarketDataStore.getState();
    marketDataStore.setConnectionStatus(ConnectionStatus.DISCONNECTED);
  }

  /**
   * 清理过期数据
   */
  private cleanupExpiredData(): void {
    const now = Date.now();
    const maxAge = 24 * 60 * 60 * 1000; // 24小时

    // 清理行情数据中的过期数据
    useMarketDataStore.getState().clearOldData(maxAge);

    // 清理过期的自动备份
    const expiredBackups: string[] = [];
    this.backups.forEach((backup, name) => {
      if (name.startsWith('auto_') && now - backup.timestamp > maxAge) {
        expiredBackups.push(name);
      }
    });

    expiredBackups.forEach(name => {
      this.backups.delete(name);
    });
  }

  /**
   * 获取UI状态
   */
  private getUIState(): any {
    const state = useUIStore.getState();
    return {
      theme: state.theme,
      themeConfig: state.themeConfig,
      layout: state.layout,
      preferences: state.preferences,
      sidebarCollapsed: state.sidebarCollapsed,
    };
  }

  /**
   * 获取行情数据状态
   */
  private getMarketDataState(): any {
    const state = useMarketDataStore.getState();
    return {
      watchlist: state.watchlist,
      subscriptions: Array.from(state.subscriptions.entries()),
    };
  }

  /**
   * 获取交易状态
   */
  private getTradingState(): any {
    const state = useTradingStore.getState();
    return {
      quickTradeConfig: state.quickTradeConfig,
      riskControlConfig: state.riskControlConfig,
      stats: state.stats,
    };
  }

  /**
   * 恢复UI状态
   */
  private restoreUIState(uiState: any): void {
    const store = useUIStore.getState();
    
    if (uiState.theme) {
      store.switchTheme(uiState.theme);
    }
    if (uiState.themeConfig) {
      store.updateThemeConfig(uiState.themeConfig);
    }
    if (uiState.layout) {
      store.updateLayout(uiState.layout);
    }
    if (uiState.preferences) {
      store.updatePreferences(uiState.preferences);
    }
    if (typeof uiState.sidebarCollapsed === 'boolean') {
      if (uiState.sidebarCollapsed !== store.sidebarCollapsed) {
        store.toggleSidebar();
      }
    }
  }

  /**
   * 恢复行情数据状态
   */
  private restoreMarketDataState(marketDataState: any): void {
    const store = useMarketDataStore.getState();
    
    if (marketDataState.watchlist) {
      store.reorderWatchlist(marketDataState.watchlist);
    }
  }

  /**
   * 恢复交易状态
   */
  private restoreTradingState(tradingState: any): void {
    const store = useTradingStore.getState();
    
    if (tradingState.quickTradeConfig) {
      store.updateQuickTradeConfig(tradingState.quickTradeConfig);
    }
    if (tradingState.riskControlConfig) {
      store.updateRiskControlConfig(tradingState.riskControlConfig);
    }
  }
}

/**
 * 全局状态管理器实例
 */
export const stateManager = new StateManager();

/**
 * React Hook 用于状态管理操作
 */
export const useStateManager = () => {
  return {
    createBackup: stateManager.createBackup.bind(stateManager),
    restoreBackup: stateManager.restoreBackup.bind(stateManager),
    deleteBackup: stateManager.deleteBackup.bind(stateManager),
    getBackupList: stateManager.getBackupList.bind(stateManager),
    resetAllStates: stateManager.resetAllStates.bind(stateManager),
    exportConfig: stateManager.exportConfig.bind(stateManager),
    importConfig: stateManager.importConfig.bind(stateManager),
  };
};
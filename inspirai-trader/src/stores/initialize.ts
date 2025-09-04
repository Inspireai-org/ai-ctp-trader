import { useUIStore } from './ui';
import { useMarketDataStore } from './marketData';
import { useTradingStore } from './trading';
import { stateManager } from './stateManager';
import { eventBus } from './eventBus';
import { AppEventType } from '../types';

/**
 * 初始化状态管理系统
 */
export const initializeStores = async (): Promise<void> => {
  try {
    console.log('正在初始化状态管理系统...');

    // 1. 初始化状态管理器
    stateManager.initialize();

    // 2. 初始化UI主题
    await useUIStore.getState().initializeTheme();

    // 3. 设置全局错误处理
    setupGlobalErrorHandling();

    // 4. 设置状态同步监听器
    setupStateSyncListeners();

    // 5. 创建初始化备份
    stateManager.createBackup('initialization');

    console.log('状态管理系统初始化完成');

    // 发布初始化完成事件
    eventBus.emit(AppEventType.NOTIFICATION, {
      type: 'success',
      message: '应用初始化完成',
    });

  } catch (error) {
    console.error('状态管理系统初始化失败:', error);
    
    eventBus.emit(AppEventType.ERROR_OCCURRED, {
      message: `初始化失败: ${error instanceof Error ? error.message : '未知错误'}`,
    });
    
    throw error;
  }
};

/**
 * 设置全局错误处理
 */
const setupGlobalErrorHandling = (): void => {
  // 监听未捕获的Promise错误
  window.addEventListener('unhandledrejection', (event) => {
    console.error('未捕获的Promise错误:', event.reason);
    
    eventBus.emit(AppEventType.ERROR_OCCURRED, {
      message: `未捕获的错误: ${event.reason}`,
      type: 'unhandledrejection',
    });
    
    // 阻止默认的错误处理
    event.preventDefault();
  });

  // 监听全局JavaScript错误
  window.addEventListener('error', (event) => {
    console.error('全局JavaScript错误:', event.error);
    
    eventBus.emit(AppEventType.ERROR_OCCURRED, {
      message: `JavaScript错误: ${event.message}`,
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
      type: 'javascript',
    });
  });
};

/**
 * 设置状态同步监听器
 */
const setupStateSyncListeners = (): void => {
  // 监听连接状态变化
  eventBus.on(AppEventType.CONNECTION_CHANGED, (event) => {
    const { status } = event.data;
    console.log(`连接状态变化: ${status}`);
    
    // 可以在这里添加连接状态变化的处理逻辑
  });

  // 监听行情数据更新
  eventBus.on(AppEventType.MARKET_DATA_UPDATED, () => {
    // 可以在这里添加行情数据更新的处理逻辑
    // 例如：更新相关的计算、触发预警等
  });

  // 监听交易事件
  eventBus.on(AppEventType.ORDER_UPDATED, (event) => {
    const order = event.data;
    console.log(`订单状态更新: ${order.orderId} - ${order.status}`);
  });

  eventBus.on(AppEventType.POSITION_UPDATED, (event) => {
    const position = event.data;
    console.log(`持仓更新: ${position.instrumentId} - ${position.direction}`);
  });

  eventBus.on(AppEventType.TRADE_UPDATED, (event) => {
    const trade = event.data;
    console.log(`成交记录: ${trade.tradeId} - ${trade.instrumentId}`);
  });

  // 监听账户信息更新
  eventBus.on(AppEventType.ACCOUNT_UPDATED, (event) => {
    const account = event.data;
    console.log(`账户信息更新: 可用资金 ${account.available}`);
  });

  // 监听错误事件
  eventBus.on(AppEventType.ERROR_OCCURRED, (event) => {
    const error = event.data;
    console.error('应用错误:', error);
    
    // 可以在这里添加错误处理逻辑
    // 例如：显示错误通知、记录错误日志等
  });

  // 监听通知事件
  eventBus.on(AppEventType.NOTIFICATION, (event) => {
    const notification = event.data;
    console.log(`通知: [${notification.type}] ${notification.message}`);
    
    // 可以在这里集成通知系统
    // 例如：显示Toast通知、系统通知等
  });
};

/**
 * 清理状态管理系统
 */
export const cleanupStores = (): void => {
  try {
    console.log('正在清理状态管理系统...');

    // 创建清理前的备份
    stateManager.createBackup('before_cleanup');

    // 清理事件监听器
    eventBus.off();

    // 重置所有状态
    useUIStore.getState().resetToDefault();
    useMarketDataStore.getState().reset();
    useTradingStore.getState().reset();

    console.log('状态管理系统清理完成');

  } catch (error) {
    console.error('状态管理系统清理失败:', error);
  }
};

/**
 * 获取系统状态信息
 */
export const getSystemStatus = () => {
  const uiStore = useUIStore.getState();
  const marketDataStore = useMarketDataStore.getState();
  const tradingStore = useTradingStore.getState();

  return {
    ui: {
      theme: uiStore.theme,
      layoutLocked: uiStore.isLayoutLocked,
      sidebarCollapsed: uiStore.sidebarCollapsed,
    },
    marketData: {
      connectionStatus: marketDataStore.connectionStatus,
      subscriptionsCount: marketDataStore.subscriptions.size,
      watchlistCount: marketDataStore.watchlist.length,
      ticksCount: marketDataStore.ticks.size,
      totalTicksReceived: marketDataStore.totalTicksReceived,
    },
    trading: {
      hasAccountInfo: !!tradingStore.accountInfo,
      ordersCount: tradingStore.orders.size,
      positionsCount: tradingStore.positions.size,
      todayTradesCount: tradingStore.todayTrades.length,
      isTrading: tradingStore.isTrading,
      hasError: !!tradingStore.lastError,
    },
    events: {
      listenerCount: eventBus.getListenerCount(),
    },
    backups: {
      count: stateManager.getBackupList().length,
    },
  };
};

/**
 * React Hook 用于获取系统状态
 */
export const useSystemStatus = () => {
  const [status, setStatus] = React.useState(getSystemStatus());

  React.useEffect(() => {
    const updateStatus = () => {
      setStatus(getSystemStatus());
    };

    // 定期更新状态
    const interval = setInterval(updateStatus, 5000);

    // 监听关键事件来立即更新状态
    const unsubscribers = [
      eventBus.on(AppEventType.CONNECTION_CHANGED, updateStatus),
      eventBus.on(AppEventType.ACCOUNT_UPDATED, updateStatus),
      eventBus.on(AppEventType.ERROR_OCCURRED, updateStatus),
    ];

    return () => {
      clearInterval(interval);
      unsubscribers.forEach(unsub => unsub());
    };
  }, []);

  return status;
};

// 导入 React
import * as React from 'react';
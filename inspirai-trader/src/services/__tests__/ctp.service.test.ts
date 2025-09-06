/**
 * CTP 服务层测试
 * 
 * 测试 CTP 服务管理器的核心功能
 */

import { describe, it, expect, beforeEach, afterEach, mock } from 'bun:test';

// Mock Tauri API
const mockInvoke = mock(() => Promise.resolve());
const mockListen = mock(() => Promise.resolve(() => {}));

// Mock Tauri modules
mock.module('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

mock.module('@tauri-apps/api/event', () => ({
  listen: mockListen,
}));

// 动态导入以避免模块加载问题
let CtpServiceManager: any;
let ClientState: any;
let Environment: any;

describe('CtpServiceManager', () => {
  let service: any;

  beforeEach(async () => {
    // 动态导入模块
    const ctpModule = await import('../ctp.service');
    const typesModule = await import('../../types');
    
    CtpServiceManager = ctpModule.CtpServiceManager;
    ClientState = typesModule.ClientState;
    Environment = typesModule.Environment;
    
    service = CtpServiceManager.getInstance();
    mockInvoke.mockClear();
    mockListen.mockClear();
  });

  afterEach(async () => {
    if (service && service.removeAllListeners) {
      await service.removeAllListeners();
    }
  });

  describe('单例模式', () => {
    it('应该返回同一个实例', () => {
      const instance1 = CtpServiceManager.getInstance();
      const instance2 = CtpServiceManager.getInstance();
      expect(instance1).toBe(instance2);
    });
  });

  describe('配置管理', () => {
    it('应该能够从预设创建配置', () => {
      const config = service.createConfigFromPreset('gzqh_test', {
        userId: 'test_user',
        password: 'test_password',
      });

      expect(config.environment).toBe('gzqh_test');
      expect(config.broker_id).toBe('5071');
      expect(config.investor_id).toBe('test_user');
      expect(config.password).toBe('test_password');
      expect(config.auth_code).toBe('QHFK5E2GLEUB9XHV');
    });

    it('应该能够获取默认配置', () => {
      const config = service.getDefaultConfig({
        userId: 'default_user',
        password: 'default_password',
      });

      expect(config.environment).toBe('gzqh_test');
      expect(config.investor_id).toBe('default_user');
      expect(config.password).toBe('default_password');
    });

    it('应该在预设不存在时抛出错误', () => {
      expect(() => {
        service.createConfigFromPreset('non_existent_preset');
      }).toThrow('未找到预设配置: non_existent_preset');
    });
  });

  describe('初始化', () => {
    it('应该能够初始化服务', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      
      await service.init();
      
      expect(mockInvoke).toHaveBeenCalledWith('ctp_init');
    });

    it('应该处理初始化错误', async () => {
      const error = new Error('初始化失败');
      mockInvoke.mockRejectedValueOnce(error);
      
      await expect(service.init()).rejects.toThrow();
    });
  });

  describe('连接管理', () => {
    const mockConfig = {
      environment: 'gzqh_test' as Environment,
      md_front_addr: 'tcp://test:41214',
      trader_front_addr: 'tcp://test:41206',
      broker_id: '5071',
      investor_id: 'test_user',
      password: 'test_password',
      app_id: 'test_app',
      auth_code: 'test_auth',
      flow_path: './test_flow/',
      timeout_secs: 30,
      reconnect_interval_secs: 5,
      max_reconnect_attempts: 3,
    };

    it('应该能够连接到服务器', async () => {
      mockInvoke.mockResolvedValueOnce(undefined); // init
      mockInvoke.mockResolvedValueOnce(undefined); // connect
      
      await service.connect(mockConfig);
      
      expect(mockInvoke).toHaveBeenCalledWith('ctp_connect', { config: mockConfig });
    });

    it('应该能够断开连接', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      
      await service.disconnect();
      
      expect(mockInvoke).toHaveBeenCalledWith('ctp_disconnect');
    });

    it('应该能够检查连接状态', async () => {
      mockInvoke.mockResolvedValueOnce(true);
      
      const connected = await service.isConnected();
      
      expect(connected).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('ctp_is_connected');
    });

    it('应该能够获取客户端状态', async () => {
      const mockState = ClientState.CONNECTED;
      mockInvoke.mockResolvedValueOnce(mockState);
      
      const state = await service.getState();
      
      expect(state).toBe(mockState);
      expect(mockInvoke).toHaveBeenCalledWith('ctp_get_state');
    });
  });

  describe('行情数据', () => {
    it('应该能够订阅行情数据', async () => {
      mockInvoke.mockResolvedValueOnce(true); // isLoggedIn
      mockInvoke.mockResolvedValueOnce(undefined); // subscribe
      
      const instruments = ['rb2510', 'au2412'];
      await service.subscribeMarketData(instruments);
      
      expect(mockInvoke).toHaveBeenCalledWith('ctp_subscribe_market_data', { instruments });
    });

    it('应该能够取消订阅行情数据', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      
      const instruments = ['rb2510'];
      await service.unsubscribeMarketData(instruments);
      
      expect(mockInvoke).toHaveBeenCalledWith('ctp_unsubscribe_market_data', { instruments });
    });

    it('应该能够检查合约订阅状态', async () => {
      mockInvoke.mockResolvedValueOnce(true);
      
      const subscribed = await service.isInstrumentSubscribed('rb2510');
      
      expect(subscribed).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('ctp_is_instrument_subscribed', { instrumentId: 'rb2510' });
    });
  });

  describe('交易功能', () => {
    const mockOrder = {
      instrumentId: 'rb2510',
      direction: 'Buy' as const,
      offsetFlag: 'Open' as const,
      price: 3600,
      volume: 1,
      orderType: 'Limit' as const,
      timeCondition: 'GFD' as const,
    };

    it('应该能够提交订单', async () => {
      mockInvoke.mockResolvedValueOnce(true); // isLoggedIn
      mockInvoke.mockResolvedValueOnce('order_123');
      
      const orderId = await service.submitOrder(mockOrder);
      
      expect(orderId).toBe('order_123');
      expect(mockInvoke).toHaveBeenCalledWith('ctp_submit_order', { order: mockOrder });
    });

    it('应该能够撤销订单', async () => {
      mockInvoke.mockResolvedValueOnce(true); // isLoggedIn
      mockInvoke.mockResolvedValueOnce(undefined);
      
      await service.cancelOrder('order_123');
      
      expect(mockInvoke).toHaveBeenCalledWith('ctp_cancel_order', { orderId: 'order_123' });
    });
  });

  describe('查询功能', () => {
    it('应该能够查询账户信息', async () => {
      mockInvoke.mockResolvedValueOnce(true); // isLoggedIn
      mockInvoke.mockResolvedValueOnce(undefined);
      
      await service.queryAccount();
      
      expect(mockInvoke).toHaveBeenCalledWith('ctp_query_account');
    });

    it('应该能够查询持仓信息', async () => {
      mockInvoke.mockResolvedValueOnce(true); // isLoggedIn
      mockInvoke.mockResolvedValueOnce(undefined);
      
      await service.queryPositions();
      
      expect(mockInvoke).toHaveBeenCalledWith('ctp_query_positions');
    });

    it('应该能够查询成交记录', async () => {
      mockInvoke.mockResolvedValueOnce(true); // isLoggedIn
      mockInvoke.mockResolvedValueOnce(undefined);
      
      await service.queryTrades('rb2510');
      
      expect(mockInvoke).toHaveBeenCalledWith('ctp_query_trades', { instrumentId: 'rb2510' });
    });

    it('应该能够查询订单状态', async () => {
      mockInvoke.mockResolvedValueOnce(true); // isLoggedIn
      mockInvoke.mockResolvedValueOnce(undefined);
      
      await service.queryOrders();
      
      expect(mockInvoke).toHaveBeenCalledWith('ctp_query_orders', { instrumentId: undefined });
    });
  });

  describe('事件监听', () => {
    it('应该能够监听行情数据', async () => {
      const mockCallback = mock(() => {});
      const mockUnlisten = mock(() => {});
      mockListen.mockResolvedValueOnce(mockUnlisten);
      
      const unlisten = await service.listenToMarketData(mockCallback);
      
      expect(mockListen).toHaveBeenCalledWith('market-data', expect.any(Function));
      expect(unlisten).toBe(mockUnlisten);
    });

    it('应该能够监听连接状态', async () => {
      const mockCallback = mock(() => {});
      const mockUnlisten = mock(() => {});
      mockListen.mockResolvedValueOnce(mockUnlisten);
      
      const unlisten = await service.listenToConnectionStatus(mockCallback);
      
      expect(mockListen).toHaveBeenCalledWith('connection-status', expect.any(Function));
      expect(unlisten).toBe(mockUnlisten);
    });

    it('应该能够移除所有监听器', async () => {
      const mockUnlisten1 = mock(() => Promise.resolve());
      const mockUnlisten2 = mock(() => Promise.resolve());
      
      // 添加一些监听器
      service['eventListeners'].set('test1', mockUnlisten1);
      service['eventListeners'].set('test2', mockUnlisten2);
      
      await service.removeAllListeners();
      
      expect(mockUnlisten1).toHaveBeenCalled();
      expect(mockUnlisten2).toHaveBeenCalled();
      expect(service['eventListeners'].size).toBe(0);
    });
  });

  describe('错误处理', () => {
    it('应该处理连接错误', async () => {
      const error = new Error('连接失败');
      mockInvoke.mockRejectedValueOnce(error);
      
      await expect(service.connect({} as any)).rejects.toThrow();
    });

    it('应该处理查询错误', async () => {
      mockInvoke.mockResolvedValueOnce(false); // isLoggedIn
      
      await expect(service.queryAccount()).rejects.toThrow('CTP 未登录，请先登录');
    });
  });

  describe('服务配置', () => {
    it('应该能够获取服务配置', () => {
      const config = service.getServiceConfig();
      
      expect(config.defaultRetry.maxRetries).toBe(3);
      expect(config.connectionTimeout).toBe(30000);
      expect(config.queryTimeout).toBe(10000);
    });

    it('应该能够更新服务配置', () => {
      service.updateServiceConfig({
        defaultRetry: {
          maxRetries: 5,
          baseDelay: 2000,
          exponentialBackoff: true,
        },
      });
      
      const config = service.getServiceConfig();
      expect(config.defaultRetry.maxRetries).toBe(5);
      expect(config.defaultRetry.baseDelay).toBe(2000);
    });
  });
});
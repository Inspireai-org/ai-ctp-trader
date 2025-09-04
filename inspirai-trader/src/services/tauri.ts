/**
 * Tauri 服务层
 * 
 * 封装所有 Tauri 命令调用，提供类型安全的 API 接口
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
  CtpConfig,
  LoginCredentials,
  LoginResponse,
  OrderRequest,

  MarketDataTick,
  Position,
  AccountInfo,
  TradeRecord,
  OrderStatus,
  CtpEvent,

  ClientState,
  ConnectionStats,
  HealthStatus,
  ConfigInfo,
  CtpError,
} from '../types';
import { ErrorHandler, withRetry } from './errorHandler';

/**
 * Tauri 命令调用结果类型
 */
export interface TauriResult<T> {
  success: boolean;
  data?: T;
  error?: string;
}

/**
 * CTP 服务类
 * 
 * 提供与后端 CTP 系统交互的所有方法
 */
export class CtpService {
  private eventListeners: Map<string, UnlistenFn> = new Map();

  // ============================================================================
  // 基础连接和配置方法
  // ============================================================================

  /**
   * 初始化 CTP 组件
   */
  async init(): Promise<string> {
    try {
      const result = await invoke<string>('ctp_init');
      return result;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 创建默认配置
   */
  async createConfig(): Promise<CtpConfig> {
    try {
      const result = await invoke<CtpConfig>('ctp_create_config');
      return result;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 连接到 CTP 服务器
   */
  async connect(config: CtpConfig): Promise<void> {
    try {
      await invoke<void>('ctp_connect', { config });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 带重连的连接方法
   */
  async connectWithRetry(config: CtpConfig): Promise<void> {
    try {
      await invoke<void>('ctp_connect_with_retry', { config });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 带重试的连接方法
   */
  connectWithRetryWrapper = withRetry(
    async (config: CtpConfig) => {
      await this.connect(config);
    },
    {
      maxRetries: 3,
      baseDelay: 2000,
      onRetry: (error, attempt) => {
        console.log(`连接重试 ${attempt}/3: ${error.message}`);
      },
    }
  );

  /**
   * 带重试的登录方法
   */
  loginWithRetry = withRetry(
    async (credentials: LoginCredentials) => {
      return await this.login(credentials);
    },
    {
      maxRetries: 2,
      baseDelay: 1000,
      onRetry: (error, attempt) => {
        console.log(`登录重试 ${attempt}/2: ${error.message}`);
      },
    }
  );

  /**
   * 断开连接
   */
  async disconnect(): Promise<void> {
    try {
      await invoke<void>('ctp_disconnect');
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 用户登录
   */
  async login(credentials: LoginCredentials): Promise<LoginResponse> {
    try {
      const result = await invoke<LoginResponse>('ctp_login', { credentials });
      return result;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 获取客户端状态
   */
  async getState(): Promise<ClientState> {
    try {
      const result = await invoke<ClientState>('ctp_get_state');
      return result;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 检查是否已连接
   */
  async isConnected(): Promise<boolean> {
    try {
      const result = await invoke<boolean>('ctp_is_connected');
      return result;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 检查是否已登录
   */
  async isLoggedIn(): Promise<boolean> {
    try {
      const result = await invoke<boolean>('ctp_is_logged_in');
      return result;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 获取连接统计信息
   */
  async getConnectionStats(): Promise<ConnectionStats> {
    try {
      const result = await invoke<ConnectionStats>('ctp_get_connection_stats');
      return result;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 健康检查
   */
  async healthCheck(): Promise<HealthStatus> {
    try {
      const result = await invoke<HealthStatus>('ctp_health_check');
      return result;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 重置客户端状态
   */
  async reset(): Promise<void> {
    try {
      await invoke<void>('ctp_reset');
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 获取配置信息（隐藏敏感信息）
   */
  async getConfigInfo(): Promise<ConfigInfo> {
    try {
      const result = await invoke<ConfigInfo>('ctp_get_config_info');
      return result;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  // ============================================================================
  // 行情数据方法
  // ============================================================================

  /**
   * 订阅行情数据
   */
  async subscribeMarketData(instruments: string[]): Promise<void> {
    try {
      await invoke<void>('ctp_subscribe_market_data', { instruments });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 取消订阅行情数据
   */
  async unsubscribeMarketData(instruments: string[]): Promise<void> {
    try {
      await invoke<void>('ctp_unsubscribe_market_data', { instruments });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 检查合约是否已订阅
   */
  async isInstrumentSubscribed(instrumentId: string): Promise<boolean> {
    try {
      const result = await invoke<boolean>('ctp_is_instrument_subscribed', { instrumentId });
      return result;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  // ============================================================================
  // 交易方法
  // ============================================================================

  /**
   * 提交订单
   */
  async submitOrder(order: OrderRequest): Promise<string> {
    try {
      const result = await invoke<string>('ctp_submit_order', { order });
      return result;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 撤销订单
   */
  async cancelOrder(orderId: string): Promise<void> {
    try {
      await invoke<void>('ctp_cancel_order', { orderId });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  // ============================================================================
  // 查询方法
  // ============================================================================

  /**
   * 查询账户信息
   */
  async queryAccount(): Promise<void> {
    try {
      await invoke<void>('ctp_query_account');
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 查询持仓信息
   */
  async queryPositions(): Promise<void> {
    try {
      await invoke<void>('ctp_query_positions');
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 查询成交记录
   */
  async queryTrades(instrumentId?: string): Promise<void> {
    try {
      await invoke<void>('ctp_query_trades', { instrumentId });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * 查询订单状态
   */
  async queryOrders(instrumentId?: string): Promise<void> {
    try {
      await invoke<void>('ctp_query_orders', { instrumentId });
    } catch (error) {
      throw this.handleError(error);
    }
  }

  // ============================================================================
  // 事件监听方法
  // ============================================================================

  /**
   * 监听 CTP 事件
   */
  async listenToCtpEvents(callback: (event: CtpEvent) => void): Promise<UnlistenFn> {
    const unlisten = await listen<CtpEvent>('ctp-event', (event) => {
      callback(event.payload);
    });

    this.eventListeners.set('ctp-event', unlisten);
    return unlisten;
  }

  /**
   * 监听行情数据更新
   */
  async listenToMarketData(callback: (data: MarketDataTick) => void): Promise<UnlistenFn> {
    const unlisten = await listen<MarketDataTick>('market-data', (event) => {
      callback(event.payload);
    });

    this.eventListeners.set('market-data', unlisten);
    return unlisten;
  }

  /**
   * 监听账户信息更新
   */
  async listenToAccountInfo(callback: (data: AccountInfo) => void): Promise<UnlistenFn> {
    const unlisten = await listen<AccountInfo>('account-info', (event) => {
      callback(event.payload);
    });

    this.eventListeners.set('account-info', unlisten);
    return unlisten;
  }

  /**
   * 监听持仓信息更新
   */
  async listenToPositionInfo(callback: (data: Position) => void): Promise<UnlistenFn> {
    const unlisten = await listen<Position>('position-info', (event) => {
      callback(event.payload);
    });

    this.eventListeners.set('position-info', unlisten);
    return unlisten;
  }

  /**
   * 监听订单状态更新
   */
  async listenToOrderStatus(callback: (data: OrderStatus) => void): Promise<UnlistenFn> {
    const unlisten = await listen<OrderStatus>('order-status', (event) => {
      callback(event.payload);
    });

    this.eventListeners.set('order-status', unlisten);
    return unlisten;
  }

  /**
   * 监听成交记录更新
   */
  async listenToTradeInfo(callback: (data: TradeRecord) => void): Promise<UnlistenFn> {
    const unlisten = await listen<TradeRecord>('trade-info', (event) => {
      callback(event.payload);
    });

    this.eventListeners.set('trade-info', unlisten);
    return unlisten;
  }

  /**
   * 监听连接状态变化
   */
  async listenToConnectionStatus(callback: (state: ClientState) => void): Promise<UnlistenFn> {
    const unlisten = await listen<ClientState>('connection-status', (event) => {
      callback(event.payload);
    });

    this.eventListeners.set('connection-status', unlisten);
    return unlisten;
  }

  /**
   * 监听错误事件
   */
  async listenToErrors(callback: (error: CtpError) => void): Promise<UnlistenFn> {
    const unlisten = await listen<CtpError>('ctp-error', (event) => {
      callback(event.payload);
    });

    this.eventListeners.set('ctp-error', unlisten);
    return unlisten;
  }

  // ============================================================================
  // 工具方法
  // ============================================================================

  /**
   * 移除所有事件监听器
   */
  async removeAllListeners(): Promise<void> {
    for (const [eventName, unlisten] of this.eventListeners) {
      try {
        await unlisten();
        console.log(`已移除事件监听器: ${eventName}`);
      } catch (error) {
        console.error(`移除事件监听器失败: ${eventName}`, error);
      }
    }
    this.eventListeners.clear();
  }

  /**
   * 移除指定事件监听器
   */
  async removeListener(eventName: string): Promise<void> {
    const unlisten = this.eventListeners.get(eventName);
    if (unlisten) {
      try {
        await unlisten();
        this.eventListeners.delete(eventName);
        console.log(`已移除事件监听器: ${eventName}`);
      } catch (error) {
        console.error(`移除事件监听器失败: ${eventName}`, error);
      }
    }
  }

  /**
   * 处理错误
   */
  private handleError(error: any): CtpError {
    const ctpError = ErrorHandler.toCtpError(error);
    ErrorHandler.logError(ctpError, 'Tauri 命令调用');
    return ctpError;
  }

  /**
   * 格式化错误消息
   */
  formatError(error: CtpError): string {
    return ErrorHandler.formatError(error);
  }

  /**
   * 获取用户友好的错误消息
   */
  getUserFriendlyMessage(error: CtpError): string {
    return ErrorHandler.getUserFriendlyMessage(error);
  }

  /**
   * 检查错误类型
   */
  isConnectionError(error: CtpError): boolean {
    return ErrorHandler.isConnectionError(error);
  }

  /**
   * 检查是否为认证错误
   */
  isAuthenticationError(error: CtpError): boolean {
    return ErrorHandler.isAuthenticationError(error);
  }

  /**
   * 检查是否为配置错误
   */
  isConfigError(error: CtpError): boolean {
    return ErrorHandler.isConfigError(error);
  }

  /**
   * 检查是否为超时错误
   */
  isTimeoutError(error: CtpError): boolean {
    return ErrorHandler.isTimeoutError(error);
  }

  /**
   * 检查是否为可重试的错误
   */
  isRetryableError(error: CtpError): boolean {
    return ErrorHandler.isRetryableError(error);
  }

  /**
   * 检查是否为致命错误
   */
  isFatalError(error: CtpError): boolean {
    return ErrorHandler.isFatalError(error);
  }
}

// 创建单例实例
export const ctpService = new CtpService();

// 导出默认实例
export default ctpService;
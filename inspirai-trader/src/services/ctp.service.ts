/**
 * CTP 服务层管理器
 * 
 * 提供完整的 CTP 交易系统接口，包括：
 * - 连接管理和状态监控
 * - 行情数据订阅和处理
 * - 交易订单管理
 * - 账户和持仓查询
 * - 错误处理和重试机制
 * - 事件监听和通知
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
  CtpConfig,
  LoginCredentials,
  LoginResponse,
  OrderRequest,
  OrderAction,
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
  CtpErrorType,
  Environment,
} from '../types';
import { ErrorHandler } from './errorHandler';
import { CtpPreset, getPreset, getDefaultPreset } from '../config/ctp-presets';

/**
 * 重试配置接口
 */
interface RetryConfig {
  /** 最大重试次数 */
  maxRetries: number;
  /** 基础延迟时间（毫秒） */
  baseDelay: number;
  /** 是否启用指数退避 */
  exponentialBackoff: boolean;
  /** 重试回调 */
  onRetry?: (error: CtpError, attempt: number) => void;
}

/**
 * 服务配置接口
 */
interface ServiceConfig {
  /** 默认重试配置 */
  defaultRetry: RetryConfig;
  /** 连接超时时间（毫秒） */
  connectionTimeout: number;
  /** 查询超时时间（毫秒） */
  queryTimeout: number;
  /** 事件监听器清理间隔（毫秒） */
  listenerCleanupInterval: number;
}

/**
 * CTP 服务管理器
 * 
 * 单例模式，提供统一的 CTP 服务接口
 */
export class CtpServiceManager {
  private static instance: CtpServiceManager;
  private eventListeners: Map<string, UnlistenFn> = new Map();
  private isInitialized = false;
  private currentConfig: CtpConfig | null = null;
  private serviceConfig: ServiceConfig;

  private constructor() {
    // 初始化服务配置
    this.serviceConfig = {
      defaultRetry: {
        maxRetries: 3,
        baseDelay: 1000,
        exponentialBackoff: true,
      },
      connectionTimeout: 30000,
      queryTimeout: 10000,
      listenerCleanupInterval: 60000,
    };

    // 设置重试回调（避免循环引用）
    this.serviceConfig.defaultRetry.onRetry = (error, attempt) => {
      console.log(`CTP 操作重试 ${attempt}/${this.serviceConfig.defaultRetry.maxRetries}: ${error.message}`);
    };
  }

  /**
   * 获取单例实例
   */
  static getInstance(): CtpServiceManager {
    if (!CtpServiceManager.instance) {
      CtpServiceManager.instance = new CtpServiceManager();
    }
    return CtpServiceManager.instance;
  }

  // ============================================================================
  // 初始化和配置方法
  // ============================================================================

  /**
   * 初始化 CTP 服务
   */
  async init(): Promise<void> {
    if (this.isInitialized) {
      console.log('CTP 服务已初始化');
      return;
    }

    try {
      await invoke<void>('ctp_init');
      this.isInitialized = true;
      console.log('CTP 服务初始化成功');
    } catch (error) {
      const ctpError = this.handleError(error, 'CTP 服务初始化');
      throw ctpError;
    }
  }

  /**
   * 创建默认配置
   */
  async createConfig(): Promise<CtpConfig> {
    try {
      const config = await invoke<CtpConfig>('ctp_create_config');
      return config;
    } catch (error) {
      throw this.handleError(error, '创建默认配置');
    }
  }

  /**
   * 从预设创建配置
   */
  createConfigFromPreset(presetKey: string, credentials?: { userId: string; password: string }): CtpConfig {
    const preset = getPreset(presetKey);
    if (!preset) {
      throw new Error(`未找到预设配置: ${presetKey}`);
    }

    return this.convertPresetToConfig(preset, credentials);
  }

  /**
   * 获取默认配置
   */
  getDefaultConfig(credentials?: { userId: string; password: string }): CtpConfig {
    const preset = getDefaultPreset();
    return this.convertPresetToConfig(preset, credentials);
  }

  /**
   * 转换预设配置为 CTP 配置
   */
  private convertPresetToConfig(preset: CtpPreset, credentials?: { userId: string; password: string }): CtpConfig {
    return {
      environment: preset.key as Environment,
      md_front_addr: preset.md_front_addr,
      trader_front_addr: preset.trader_front_addr,
      broker_id: preset.broker_id,
      investor_id: credentials?.userId || preset.defaultInvestorId || '',
      password: credentials?.password || preset.defaultPassword || '',
      app_id: preset.app_id,
      auth_code: preset.auth_code,
      flow_path: `./ctp_flow/${preset.key}/`,
      timeout_secs: 30,
      reconnect_interval_secs: 5,
      max_reconnect_attempts: 3,
    };
  }

  // ============================================================================
  // 连接管理方法
  // ============================================================================

  /**
   * 连接到 CTP 服务器
   */
  async connect(config: CtpConfig): Promise<void> {
    try {
      await this.ensureInitialized();
      await invoke<void>('ctp_connect', { config });
      this.currentConfig = config;
      console.log(`已连接到 CTP 服务器: ${config.environment}`);
    } catch (error) {
      throw this.handleError(error, 'CTP 连接');
    }
  }

  /**
   * 带重试的连接方法
   */
  async connectWithRetry(config: CtpConfig): Promise<void> {
    const maxRetries = this.serviceConfig.defaultRetry.maxRetries;
    let lastError: CtpError;
    
    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        await this.connect(config);
        return;
      } catch (error) {
        lastError = this.handleError(error, 'CTP 连接重试');
        
        if (attempt === maxRetries || !this.isRetryableError(lastError)) {
          throw lastError;
        }
        
        const delay = this.serviceConfig.defaultRetry.baseDelay * Math.pow(2, attempt);
        
        if (this.serviceConfig.defaultRetry.onRetry) {
          this.serviceConfig.defaultRetry.onRetry(lastError, attempt + 1);
        }
        
        console.log(`连接重试 ${attempt + 1}/${maxRetries}，${delay}ms 后重试`);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
    
    throw lastError!;
  }

  /**
   * 断开连接
   */
  async disconnect(): Promise<void> {
    try {
      await invoke<void>('ctp_disconnect');
      this.currentConfig = null;
      console.log('已断开 CTP 连接');
    } catch (error) {
      throw this.handleError(error, 'CTP 断开连接');
    }
  }

  /**
   * 用户登录
   */
  async login(credentials: LoginCredentials): Promise<LoginResponse> {
    try {
      await this.ensureInitialized();
      const response = await invoke<LoginResponse>('ctp_login', { credentials });
      console.log(`用户登录成功: ${credentials.userId}`);
      return response;
    } catch (error) {
      throw this.handleError(error, 'CTP 用户登录');
    }
  }

  /**
   * 带重试的登录方法
   */
  async loginWithRetry(credentials: LoginCredentials): Promise<LoginResponse> {
    const maxRetries = 2;
    let lastError: CtpError;
    
    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        return await this.login(credentials);
      } catch (error) {
        lastError = this.handleError(error, 'CTP 登录重试');
        
        if (attempt === maxRetries || !this.isRetryableError(lastError)) {
          throw lastError;
        }
        
        const delay = 1000 * Math.pow(2, attempt);
        console.log(`登录重试 ${attempt + 1}/${maxRetries}，${delay}ms 后重试`);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
    
    throw lastError!;
  }

  // ============================================================================
  // 状态查询方法
  // ============================================================================

  /**
   * 获取客户端状态
   */
  async getState(): Promise<ClientState> {
    try {
      const state = await invoke<ClientState>('ctp_get_state');
      return state;
    } catch (error) {
      throw this.handleError(error, '获取客户端状态');
    }
  }

  /**
   * 检查是否已连接
   */
  async isConnected(): Promise<boolean> {
    try {
      const connected = await invoke<boolean>('ctp_is_connected');
      return connected;
    } catch (error) {
      console.warn('检查连接状态失败:', error);
      return false;
    }
  }

  /**
   * 检查是否已登录
   */
  async isLoggedIn(): Promise<boolean> {
    try {
      const loggedIn = await invoke<boolean>('ctp_is_logged_in');
      return loggedIn;
    } catch (error) {
      console.warn('检查登录状态失败:', error);
      return false;
    }
  }

  /**
   * 获取连接统计信息
   */
  async getConnectionStats(): Promise<ConnectionStats> {
    try {
      const stats = await invoke<ConnectionStats>('ctp_get_connection_stats');
      return stats;
    } catch (error) {
      throw this.handleError(error, '获取连接统计');
    }
  }

  /**
   * 健康检查
   */
  async healthCheck(): Promise<HealthStatus> {
    try {
      const health = await invoke<HealthStatus>('ctp_health_check');
      return health;
    } catch (error) {
      throw this.handleError(error, '健康检查');
    }
  }

  /**
   * 重置客户端状态
   */
  async reset(): Promise<void> {
    try {
      await invoke<void>('ctp_reset');
      this.currentConfig = null;
      console.log('CTP 客户端状态已重置');
    } catch (error) {
      throw this.handleError(error, '重置客户端状态');
    }
  }

  /**
   * 获取配置信息（隐藏敏感信息）
   */
  async getConfigInfo(): Promise<ConfigInfo> {
    try {
      const configInfo = await invoke<ConfigInfo>('ctp_get_config_info');
      return configInfo;
    } catch (error) {
      throw this.handleError(error, '获取配置信息');
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
      await this.ensureLoggedIn();
      await invoke<void>('ctp_subscribe_market_data', { instruments });
      console.log(`已订阅行情: ${instruments.join(', ')}`);
    } catch (error) {
      throw this.handleError(error, '订阅行情数据');
    }
  }

  /**
   * 取消订阅行情数据
   */
  async unsubscribeMarketData(instruments: string[]): Promise<void> {
    try {
      await invoke<void>('ctp_unsubscribe_market_data', { instruments });
      console.log(`已取消订阅行情: ${instruments.join(', ')}`);
    } catch (error) {
      throw this.handleError(error, '取消订阅行情数据');
    }
  }

  /**
   * 检查合约是否已订阅
   */
  async isInstrumentSubscribed(instrumentId: string): Promise<boolean> {
    try {
      const subscribed = await invoke<boolean>('ctp_is_instrument_subscribed', { instrumentId });
      return subscribed;
    } catch (error) {
      console.warn(`检查合约订阅状态失败: ${instrumentId}`, error);
      return false;
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
      await this.ensureLoggedIn();
      const orderId = await invoke<string>('ctp_submit_order', { order });
      console.log(`订单已提交: ${orderId}`, order);
      return orderId;
    } catch (error) {
      throw this.handleError(error, '提交订单');
    }
  }

  /**
   * 撤销订单
   */
  async cancelOrder(orderId: string): Promise<void> {
    try {
      await this.ensureLoggedIn();
      await invoke<void>('ctp_cancel_order', { orderId });
      console.log(`订单已撤销: ${orderId}`);
    } catch (error) {
      throw this.handleError(error, '撤销订单');
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
      await this.ensureLoggedIn();
      await invoke<void>('ctp_query_account');
      console.log('账户信息查询已发送');
    } catch (error) {
      throw this.handleError(error, '查询账户信息');
    }
  }

  /**
   * 查询持仓信息
   */
  async queryPositions(): Promise<void> {
    try {
      await this.ensureLoggedIn();
      await invoke<void>('ctp_query_positions');
      console.log('持仓信息查询已发送');
    } catch (error) {
      throw this.handleError(error, '查询持仓信息');
    }
  }

  /**
   * 查询成交记录
   */
  async queryTrades(instrumentId?: string): Promise<void> {
    try {
      await this.ensureLoggedIn();
      await invoke<void>('ctp_query_trades', { instrumentId });
      console.log(`成交记录查询已发送${instrumentId ? `: ${instrumentId}` : ''}`);
    } catch (error) {
      throw this.handleError(error, '查询成交记录');
    }
  }

  /**
   * 查询订单状态
   */
  async queryOrders(instrumentId?: string): Promise<void> {
    try {
      await this.ensureLoggedIn();
      await invoke<void>('ctp_query_orders', { instrumentId });
      console.log(`订单状态查询已发送${instrumentId ? `: ${instrumentId}` : ''}`);
    } catch (error) {
      throw this.handleError(error, '查询订单状态');
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
      try {
        callback(event.payload);
      } catch (error) {
        console.error('CTP 事件处理错误:', error);
      }
    });

    this.eventListeners.set('ctp-event', unlisten);
    return unlisten;
  }

  /**
   * 监听行情数据更新
   */
  async listenToMarketData(callback: (data: MarketDataTick) => void): Promise<UnlistenFn> {
    const unlisten = await listen<MarketDataTick>('market-data', (event) => {
      try {
        callback(event.payload);
      } catch (error) {
        console.error('行情数据处理错误:', error);
      }
    });

    this.eventListeners.set('market-data', unlisten);
    return unlisten;
  }

  /**
   * 监听账户信息更新
   */
  async listenToAccountInfo(callback: (data: AccountInfo) => void): Promise<UnlistenFn> {
    const unlisten = await listen<AccountInfo>('account-info', (event) => {
      try {
        callback(event.payload);
      } catch (error) {
        console.error('账户信息处理错误:', error);
      }
    });

    this.eventListeners.set('account-info', unlisten);
    return unlisten;
  }

  /**
   * 监听持仓信息更新
   */
  async listenToPositionInfo(callback: (data: Position) => void): Promise<UnlistenFn> {
    const unlisten = await listen<Position>('position-info', (event) => {
      try {
        callback(event.payload);
      } catch (error) {
        console.error('持仓信息处理错误:', error);
      }
    });

    this.eventListeners.set('position-info', unlisten);
    return unlisten;
  }

  /**
   * 监听订单状态更新
   */
  async listenToOrderStatus(callback: (data: OrderStatus) => void): Promise<UnlistenFn> {
    const unlisten = await listen<OrderStatus>('order-status', (event) => {
      try {
        callback(event.payload);
      } catch (error) {
        console.error('订单状态处理错误:', error);
      }
    });

    this.eventListeners.set('order-status', unlisten);
    return unlisten;
  }

  /**
   * 监听成交记录更新
   */
  async listenToTradeInfo(callback: (data: TradeRecord) => void): Promise<UnlistenFn> {
    const unlisten = await listen<TradeRecord>('trade-info', (event) => {
      try {
        callback(event.payload);
      } catch (error) {
        console.error('成交记录处理错误:', error);
      }
    });

    this.eventListeners.set('trade-info', unlisten);
    return unlisten;
  }

  /**
   * 监听连接状态变化
   */
  async listenToConnectionStatus(callback: (state: ClientState) => void): Promise<UnlistenFn> {
    const unlisten = await listen<ClientState>('connection-status', (event) => {
      try {
        callback(event.payload);
      } catch (error) {
        console.error('连接状态处理错误:', error);
      }
    });

    this.eventListeners.set('connection-status', unlisten);
    return unlisten;
  }

  /**
   * 监听错误事件
   */
  async listenToErrors(callback: (error: CtpError) => void): Promise<UnlistenFn> {
    const unlisten = await listen<CtpError>('ctp-error', (event) => {
      try {
        callback(event.payload);
      } catch (error) {
        console.error('错误事件处理错误:', error);
      }
    });

    this.eventListeners.set('ctp-error', unlisten);
    return unlisten;
  }

  // ============================================================================
  // 工具和管理方法
  // ============================================================================

  /**
   * 移除所有事件监听器
   */
  async removeAllListeners(): Promise<void> {
    const promises = Array.from(this.eventListeners.entries()).map(async ([eventName, unlisten]) => {
      try {
        await unlisten();
        console.log(`已移除事件监听器: ${eventName}`);
      } catch (error) {
        console.error(`移除事件监听器失败: ${eventName}`, error);
      }
    });

    await Promise.allSettled(promises);
    this.eventListeners.clear();
    console.log('所有事件监听器已清理');
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
   * 获取当前配置
   */
  getCurrentConfig(): CtpConfig | null {
    return this.currentConfig;
  }

  /**
   * 获取服务配置
   */
  getServiceConfig(): ServiceConfig {
    return { ...this.serviceConfig };
  }

  /**
   * 更新服务配置
   */
  updateServiceConfig(config: Partial<ServiceConfig>): void {
    this.serviceConfig = { ...this.serviceConfig, ...config };
  }

  /**
   * 销毁服务实例
   */
  async destroy(): Promise<void> {
    try {
      await this.removeAllListeners();
      await this.disconnect();
      this.isInitialized = false;
      this.currentConfig = null;
      console.log('CTP 服务已销毁');
    } catch (error) {
      console.error('销毁 CTP 服务时出错:', error);
    }
  }

  // ============================================================================
  // 私有辅助方法
  // ============================================================================

  /**
   * 确保服务已初始化
   */
  private async ensureInitialized(): Promise<void> {
    if (!this.isInitialized) {
      await this.init();
    }
  }

  /**
   * 确保已连接
   */
  private async ensureConnected(): Promise<void> {
    const connected = await this.isConnected();
    if (!connected) {
      throw new Error('CTP 未连接，请先连接服务器');
    }
  }

  /**
   * 确保已登录
   */
  private async ensureLoggedIn(): Promise<void> {
    await this.ensureConnected();
    const loggedIn = await this.isLoggedIn();
    if (!loggedIn) {
      throw new Error('CTP 未登录，请先登录');
    }
  }

  /**
   * 处理错误
   */
  private handleError(error: any, context: string): CtpError {
    const ctpError = ErrorHandler.toCtpError(error);
    ErrorHandler.logError(ctpError, context);
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

  isAuthenticationError(error: CtpError): boolean {
    return ErrorHandler.isAuthenticationError(error);
  }

  isConfigError(error: CtpError): boolean {
    return ErrorHandler.isConfigError(error);
  }

  isTimeoutError(error: CtpError): boolean {
    return ErrorHandler.isTimeoutError(error);
  }

  isRetryableError(error: CtpError): boolean {
    return ErrorHandler.isRetryableError(error);
  }

  isFatalError(error: CtpError): boolean {
    return ErrorHandler.isFatalError(error);
  }
}

// 创建并导出单例实例
export const ctpServiceManager = CtpServiceManager.getInstance();

// 导出默认实例
export default ctpServiceManager;

// 导出便捷方法
export const ctpService = ctpServiceManager;
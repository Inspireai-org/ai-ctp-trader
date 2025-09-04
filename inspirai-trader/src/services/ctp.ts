import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { CtpConfig, MarketDataTick } from '@/types';

/**
 * CTP 服务层
 * 封装所有与 CTP 后端交互的 API 调用
 */
export class CtpService {
  private static instance: CtpService;
  private listeners: Map<string, () => void> = new Map();

  private constructor() {}

  /**
   * 获取单例实例
   */
  static getInstance(): CtpService {
    if (!CtpService.instance) {
      CtpService.instance = new CtpService();
    }
    return CtpService.instance;
  }

  /**
   * 初始化 CTP 组件
   */
  async init(): Promise<string> {
    return await invoke<string>('ctp_init');
  }

  /**
   * 创建默认配置
   */
  async createConfig(): Promise<CtpConfig> {
    return await invoke<CtpConfig>('ctp_create_config');
  }

  /**
   * 连接到 CTP 服务器
   */
  async connect(config: CtpConfig): Promise<string> {
    return await invoke<string>('ctp_connect', { config });
  }

  /**
   * 登录 CTP
   */
  async login(userId: string, password: string): Promise<string> {
    return await invoke<string>('ctp_login', { userId, password });
  }

  /**
   * 订阅行情
   */
  async subscribe(instrumentIds: string[]): Promise<string> {
    return await invoke<string>('ctp_subscribe', { instrumentIds });
  }

  /**
   * 取消订阅行情
   */
  async unsubscribe(instrumentIds: string[]): Promise<string> {
    return await invoke<string>('ctp_unsubscribe', { instrumentIds });
  }

  /**
   * 获取连接状态
   */
  async getStatus(): Promise<string> {
    return await invoke<string>('ctp_get_status');
  }

  /**
   * 断开连接
   */
  async disconnect(): Promise<string> {
    return await invoke<string>('ctp_disconnect');
  }

  /**
   * 监听市场数据更新事件
   */
  async onMarketData(callback: (tick: MarketDataTick) => void): Promise<() => void> {
    const unlisten = await listen<MarketDataTick>('ctp://market-data', (event) => {
      callback(event.payload);
    });
    
    const listenerId = Math.random().toString(36);
    this.listeners.set(listenerId, unlisten);
    
    return () => {
      unlisten();
      this.listeners.delete(listenerId);
    };
  }

  /**
   * 监听连接状态变化事件
   */
  async onConnectionStatus(callback: (status: string) => void): Promise<() => void> {
    const unlisten = await listen<string>('ctp://connection-status', (event) => {
      callback(event.payload);
    });
    
    const listenerId = Math.random().toString(36);
    this.listeners.set(listenerId, unlisten);
    
    return () => {
      unlisten();
      this.listeners.delete(listenerId);
    };
  }

  /**
   * 监听错误事件
   */
  async onError(callback: (error: string) => void): Promise<() => void> {
    const unlisten = await listen<string>('ctp://error', (event) => {
      callback(event.payload);
    });
    
    const listenerId = Math.random().toString(36);
    this.listeners.set(listenerId, unlisten);
    
    return () => {
      unlisten();
      this.listeners.delete(listenerId);
    };
  }

  /**
   * 清理所有监听器
   */
  cleanup(): void {
    this.listeners.forEach(unlisten => unlisten());
    this.listeners.clear();
  }
}

// 导出单例实例
export const ctpService = CtpService.getInstance();
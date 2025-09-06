/**
 * CTP 服务层 - 简化版本
 * 
 * 这个文件保留作为向后兼容，实际功能已迁移到 ctp.service.ts
 * @deprecated 请使用 ctp.service.ts 中的 CtpServiceManager
 */

import { CtpServiceManager } from './ctp.service';

/**
 * @deprecated 请使用 CtpServiceManager
 */
export class CtpService {
  private static instance: CtpService;
  private serviceManager = CtpServiceManager.getInstance();

  private constructor() {}

  static getInstance(): CtpService {
    if (!CtpService.instance) {
      CtpService.instance = new CtpService();
    }
    return CtpService.instance;
  }

  // 代理到新的服务管理器
  async init() {
    return this.serviceManager.init();
  }

  async createConfig() {
    return this.serviceManager.createConfig();
  }

  async connect(config: any) {
    return this.serviceManager.connect(config);
  }

  async login(userId: string, password: string) {
    return this.serviceManager.login({ userId, password } as any);
  }

  async subscribe(instrumentIds: string[]) {
    return this.serviceManager.subscribeMarketData(instrumentIds);
  }

  async unsubscribe(instrumentIds: string[]) {
    return this.serviceManager.unsubscribeMarketData(instrumentIds);
  }

  async getStatus() {
    return this.serviceManager.getState();
  }

  async disconnect() {
    return this.serviceManager.disconnect();
  }

  async onMarketData(callback: any) {
    return this.serviceManager.listenToMarketData(callback);
  }

  async onConnectionStatus(callback: any) {
    return this.serviceManager.listenToConnectionStatus(callback);
  }

  async onError(callback: any) {
    return this.serviceManager.listenToErrors(callback);
  }

  cleanup() {
    this.serviceManager.removeAllListeners();
  }
}

// 导出单例实例
export const ctpService = CtpService.getInstance();
/**
 * 环境管理器
 * 负责环境检测、验证、切换和状态管理
 */

import { CtpPreset, getPreset, isWeekend, getRecommendedPreset, getTtsPresets } from '@/config/ctp-presets';

export interface EnvironmentStatus {
  presetKey: string;
  isAvailable: boolean;
  responseTime?: number;
  lastChecked: Date;
  errorMessage?: string;
  features: string[];
  recommendedForWeekend: boolean;
}

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
  suggestions: string[];
}

export class EnvironmentManager {
  private static instance: EnvironmentManager;
  private environmentStatusCache = new Map<string, EnvironmentStatus>();
  private healthCheckInterval: NodeJS.Timeout | null = null;

  private constructor() {
    // 启动健康检查
    this.startHealthCheck();
  }

  static getInstance(): EnvironmentManager {
    if (!EnvironmentManager.instance) {
      EnvironmentManager.instance = new EnvironmentManager();
    }
    return EnvironmentManager.instance;
  }

  /**
   * 检测可用环境
   */
  async detectAvailableEnvironments(): Promise<EnvironmentStatus[]> {
    const presets = Object.values(await import('@/config/ctp-presets')).filter(
      item => typeof item === 'object' && item.key
    ) as CtpPreset[];

    const statusPromises = presets.map(preset => this.checkEnvironmentStatus(preset.key));
    const statuses = await Promise.allSettled(statusPromises);

    return statuses
      .filter((result): result is PromiseFulfilledResult<EnvironmentStatus> => 
        result.status === 'fulfilled'
      )
      .map(result => result.value);
  }

  /**
   * 验证环境配置
   */
  async validateEnvironment(presetKey: string): Promise<ValidationResult> {
    const preset = getPreset(presetKey);
    const result: ValidationResult = {
      isValid: true,
      errors: [],
      warnings: [],
      suggestions: []
    };

    if (!preset) {
      result.isValid = false;
      result.errors.push(`未找到预设配置: ${presetKey}`);
      return result;
    }

    // 验证必填字段
    if (!preset.md_front_addr) {
      result.errors.push('行情前置地址不能为空');
      result.isValid = false;
    }

    if (!preset.trader_front_addr) {
      result.errors.push('交易前置地址不能为空');
      result.isValid = false;
    }

    if (!preset.broker_id) {
      result.errors.push('经纪商代码不能为空');
      result.isValid = false;
    }

    // 验证地址格式
    const addressPattern = /^tcp:\/\/[\d.]+:\d+$/;
    if (preset.md_front_addr && !addressPattern.test(preset.md_front_addr)) {
      result.warnings.push('行情前置地址格式可能不正确');
    }

    if (preset.trader_front_addr && !addressPattern.test(preset.trader_front_addr)) {
      result.warnings.push('交易前置地址格式可能不正确');
    }

    // TTS 特定验证
    if (preset.category === 'tts') {
      if (!preset.isWeekendAvailable && isWeekend()) {
        result.warnings.push('当前为周末，此 TTS 环境可能不可用');
      }

      if (!preset.defaultInvestorId || !preset.defaultPassword) {
        result.suggestions.push('建议为 TTS 环境配置默认测试账号');
      }
    }

    // 生产环境警告
    if (preset.category === 'production') {
      result.warnings.push('⚠️ 这是生产环境，请谨慎操作');
      if (!preset.requireCredentials) {
        result.errors.push('生产环境必须要求用户凭证');
        result.isValid = false;
      }
    }

    return result;
  }

  /**
   * 检查环境状态
   */
  async checkEnvironmentStatus(presetKey: string): Promise<EnvironmentStatus> {
    const preset = getPreset(presetKey);
    if (!preset) {
      throw new Error(`未找到预设配置: ${presetKey}`);
    }

    const startTime = Date.now();
    let isAvailable = false;
    let errorMessage: string | undefined;

    try {
      // 简单的连通性检查（这里可以扩展为实际的网络检查）
      if (preset.md_front_addr && preset.trader_front_addr) {
        // 模拟网络检查
        await this.simulateNetworkCheck(preset.md_front_addr);
        isAvailable = true;
      }
    } catch (error) {
      errorMessage = (error as Error).message;
    }

    const responseTime = Date.now() - startTime;
    const status: EnvironmentStatus = {
      presetKey,
      isAvailable,
      responseTime,
      lastChecked: new Date(),
      errorMessage,
      features: preset.features || [],
      recommendedForWeekend: preset.isWeekendAvailable || false
    };

    // 缓存状态
    this.environmentStatusCache.set(presetKey, status);
    return status;
  }

  /**
   * 模拟网络检查
   */
  private async simulateNetworkCheck(address: string): Promise<void> {
    // 这里可以实现真实的网络连通性检查
    // 目前只是模拟延迟
    const delay = Math.random() * 1000 + 500; // 500-1500ms
    await new Promise(resolve => setTimeout(resolve, delay));

    // 模拟一些环境可能不可用的情况
    if (address.includes('127.0.0.1') && Math.random() > 0.7) {
      throw new Error('本地服务未启动');
    }
  }

  /**
   * 获取推荐环境
   */
  getRecommendedEnvironment(): string {
    const recommended = getRecommendedPreset();
    return recommended.key;
  }

  /**
   * 检查是否为周末模式
   */
  isWeekendMode(): boolean {
    return isWeekend();
  }

  /**
   * 获取周末推荐环境
   */
  getWeekendRecommendations(): CtpPreset[] {
    return getTtsPresets().filter(preset => preset.isWeekendAvailable);
  }

  /**
   * 获取缓存的环境状态
   */
  getCachedEnvironmentStatus(presetKey: string): EnvironmentStatus | null {
    return this.environmentStatusCache.get(presetKey) || null;
  }

  /**
   * 清除状态缓存
   */
  clearStatusCache(): void {
    this.environmentStatusCache.clear();
  }

  /**
   * 启动健康检查
   */
  private startHealthCheck(): void {
    // 每5分钟检查一次环境状态
    this.healthCheckInterval = setInterval(() => {
      this.detectAvailableEnvironments().catch(console.error);
    }, 5 * 60 * 1000);
  }

  /**
   * 停止健康检查
   */
  stopHealthCheck(): void {
    if (this.healthCheckInterval) {
      clearInterval(this.healthCheckInterval);
      this.healthCheckInterval = null;
    }
  }

  /**
   * 销毁实例
   */
  destroy(): void {
    this.stopHealthCheck();
    this.clearStatusCache();
  }
}

// 导出单例实例
export const environmentManager = EnvironmentManager.getInstance();
export default environmentManager;
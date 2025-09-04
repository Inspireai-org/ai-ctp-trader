/**
 * 错误处理工具
 * 
 * 提供统一的错误处理和转换功能
 */

import { CtpError, CtpErrorType } from '../types';

/**
 * 错误处理器类
 */
export class ErrorHandler {
  /**
   * 将任意错误转换为 CtpError 格式
   */
  static toCtpError(error: any): CtpError {
    // 如果已经是 CtpError 格式，直接返回
    if (error && typeof error === 'object' && error.type && error.message) {
      return error as CtpError;
    }

    // 如果是字符串错误
    if (typeof error === 'string') {
      return {
        type: CtpErrorType.UNKNOWN_ERROR,
        message: error,
        timestamp: Date.now(),
      };
    }

    // 如果是 Error 对象
    if (error instanceof Error) {
      return {
        type: this.classifyError(error.message),
        message: error.message,
        ...(error.stack && { details: error.stack }),
        timestamp: Date.now(),
      };
    }

    // 其他情况
    return {
      type: CtpErrorType.UNKNOWN_ERROR,
      message: error?.toString() || '未知错误',
      timestamp: Date.now(),
    };
  }

  /**
   * 根据错误消息分类错误类型
   */
  private static classifyError(message: string): CtpErrorType {
    const lowerMessage = message.toLowerCase();

    if (lowerMessage.includes('connection') || lowerMessage.includes('连接')) {
      return CtpErrorType.CONNECTION_ERROR;
    }
    if (lowerMessage.includes('timeout') || lowerMessage.includes('超时')) {
      return CtpErrorType.TIMEOUT_ERROR;
    }
    if (lowerMessage.includes('auth') || lowerMessage.includes('认证') || lowerMessage.includes('登录')) {
      return CtpErrorType.AUTHENTICATION_ERROR;
    }
    if (lowerMessage.includes('config') || lowerMessage.includes('配置')) {
      return CtpErrorType.CONFIG_ERROR;
    }
    if (lowerMessage.includes('network') || lowerMessage.includes('网络')) {
      return CtpErrorType.NETWORK_ERROR;
    }
    if (lowerMessage.includes('library') || lowerMessage.includes('库')) {
      return CtpErrorType.LIBRARY_LOAD_ERROR;
    }
    if (lowerMessage.includes('state') || lowerMessage.includes('状态')) {
      return CtpErrorType.STATE_ERROR;
    }
    if (lowerMessage.includes('conversion') || lowerMessage.includes('转换')) {
      return CtpErrorType.CONVERSION_ERROR;
    }

    return CtpErrorType.UNKNOWN_ERROR;
  }

  /**
   * 格式化错误消息
   */
  static formatError(error: CtpError): string {
    let message = `[${error.type}] ${error.message}`;
    
    if (error.code) {
      message += ` (代码: ${error.code})`;
    }
    
    if (error.details) {
      message += `\n详细信息: ${error.details}`;
    }
    
    return message;
  }

  /**
   * 获取用户友好的错误消息
   */
  static getUserFriendlyMessage(error: CtpError): string {
    switch (error.type) {
      case CtpErrorType.CONNECTION_ERROR:
        return '连接服务器失败，请检查网络连接或服务器地址';
      case CtpErrorType.AUTHENTICATION_ERROR:
        return '登录失败，请检查用户名、密码或授权信息';
      case CtpErrorType.CONFIG_ERROR:
        return '配置错误，请检查配置参数';
      case CtpErrorType.TIMEOUT_ERROR:
        return '操作超时，请稍后重试';
      case CtpErrorType.NETWORK_ERROR:
        return '网络错误，请检查网络连接';
      case CtpErrorType.LIBRARY_LOAD_ERROR:
        return 'CTP 库加载失败，请检查库文件';
      case CtpErrorType.STATE_ERROR:
        return '操作状态错误，请确认当前连接状态';
      case CtpErrorType.CONVERSION_ERROR:
        return '数据转换错误，请检查输入参数';
      case CtpErrorType.CTP_API_ERROR:
        return `CTP API 错误: ${error.message}`;
      default:
        return error.message || '未知错误';
    }
  }

  /**
   * 检查错误类型
   */
  static isConnectionError(error: CtpError): boolean {
    return error.type === CtpErrorType.CONNECTION_ERROR || 
           error.type === CtpErrorType.NETWORK_ERROR;
  }

  /**
   * 检查是否为认证错误
   */
  static isAuthenticationError(error: CtpError): boolean {
    return error.type === CtpErrorType.AUTHENTICATION_ERROR;
  }

  /**
   * 检查是否为配置错误
   */
  static isConfigError(error: CtpError): boolean {
    return error.type === CtpErrorType.CONFIG_ERROR;
  }

  /**
   * 检查是否为超时错误
   */
  static isTimeoutError(error: CtpError): boolean {
    return error.type === CtpErrorType.TIMEOUT_ERROR;
  }

  /**
   * 检查是否为可重试的错误
   */
  static isRetryableError(error: CtpError): boolean {
    return error.type === CtpErrorType.TIMEOUT_ERROR ||
           error.type === CtpErrorType.NETWORK_ERROR ||
           error.type === CtpErrorType.CONNECTION_ERROR;
  }

  /**
   * 检查是否为致命错误（需要重新配置）
   */
  static isFatalError(error: CtpError): boolean {
    return error.type === CtpErrorType.CONFIG_ERROR ||
           error.type === CtpErrorType.LIBRARY_LOAD_ERROR ||
           error.type === CtpErrorType.AUTHENTICATION_ERROR;
  }

  /**
   * 记录错误日志
   */
  static logError(error: CtpError, context?: string): void {
    const contextStr = context ? `[${context}] ` : '';
    const errorMsg = this.formatError(error);
    
    console.error(`${contextStr}${errorMsg}`);
    
    // 在开发环境下，也输出到控制台
    if (import.meta.env.DEV) {
      console.group(`${contextStr}CTP 错误详情`);
      console.error('错误类型:', error.type);
      console.error('错误消息:', error.message);
      if (error.code) console.error('错误代码:', error.code);
      if (error.details) console.error('详细信息:', error.details);
      console.error('时间戳:', new Date(error.timestamp).toLocaleString());
      console.groupEnd();
    }
  }

  /**
   * 创建重试策略
   */
  static createRetryStrategy(maxRetries: number = 3, baseDelay: number = 1000) {
    return {
      maxRetries,
      baseDelay,
      shouldRetry: (error: CtpError, attempt: number): boolean => {
        return attempt < maxRetries && this.isRetryableError(error);
      },
      getDelay: (attempt: number): number => {
        // 指数退避策略
        return baseDelay * Math.pow(2, attempt);
      },
    };
  }
}

/**
 * 重试装饰器
 */
export function withRetry<T extends any[], R>(
  fn: (...args: T) => Promise<R>,
  options: {
    maxRetries?: number;
    baseDelay?: number;
    onRetry?: (error: CtpError, attempt: number) => void;
  } = {}
) {
  const { maxRetries = 3, baseDelay = 1000, onRetry } = options;
  
  return async (...args: T): Promise<R> => {
    let lastError: CtpError;
    
    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        return await fn(...args);
      } catch (error) {
        lastError = ErrorHandler.toCtpError(error);
        
        if (attempt === maxRetries || !ErrorHandler.isRetryableError(lastError)) {
          throw lastError;
        }
        
        const delay = baseDelay * Math.pow(2, attempt);
        
        if (onRetry) {
          onRetry(lastError, attempt + 1);
        }
        
        ErrorHandler.logError(lastError, `重试 ${attempt + 1}/${maxRetries}`);
        
        // 等待后重试
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
    
    throw lastError!;
  };
}

// 导出默认实例
export const errorHandler = ErrorHandler;
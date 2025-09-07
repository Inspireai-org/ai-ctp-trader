/**
 * TTS 特定错误处理器
 */

import { CtpError, TtsErrorCode } from '@/types/ctp';

export class TtsErrorHandler {
  /**
   * 处理 TTS 特定错误
   */
  static handleTtsError(error: CtpError): string {
    switch (error.code) {
      case TtsErrorCode.TTS_SERVICE_UNAVAILABLE:
        return '⚠️ TTS 服务暂时不可用，请稍后重试或选择其他测试环境';
      
      case TtsErrorCode.TTS_INVALID_CONFIG:
        return '❌ TTS 配置无效，请检查服务器地址和端口设置';
      
      case TtsErrorCode.TTS_CONNECTION_TIMEOUT:
        return '⏰ TTS 连接超时，请检查网络连接或尝试其他 TTS 环境';
      
      case TtsErrorCode.TTS_AUTHENTICATION_FAILED:
        return '🔐 TTS 认证失败，请检查测试账号和密码';
      
      case TtsErrorCode.TTS_WEEKEND_ONLY:
        return '📅 此 TTS 环境仅在周末提供服务，工作日请使用其他环境';
      
      default:
        return `TTS 连接错误: ${error.message}`;
    }
  }

  /**
   * 获取错误解决建议
   */
  static getSuggestions(error: CtpError): string[] {
    const suggestions: string[] = [];

    switch (error.code) {
      case TtsErrorCode.TTS_SERVICE_UNAVAILABLE:
        suggestions.push('尝试连接其他 TTS 环境');
        suggestions.push('检查网络连接是否正常');
        suggestions.push('稍后重试或联系技术支持');
        break;

      case TtsErrorCode.TTS_INVALID_CONFIG:
        suggestions.push('检查服务器地址格式是否正确');
        suggestions.push('确认端口号是否可用');
        suggestions.push('尝试使用预设的 TTS 配置');
        break;

      case TtsErrorCode.TTS_CONNECTION_TIMEOUT:
        suggestions.push('检查网络连接稳定性');
        suggestions.push('尝试增加连接超时时间');
        suggestions.push('选择响应更快的 TTS 环境');
        break;

      case TtsErrorCode.TTS_AUTHENTICATION_FAILED:
        suggestions.push('使用默认的测试账号和密码');
        suggestions.push('检查账号格式是否正确');
        suggestions.push('尝试重新输入认证信息');
        break;

      case TtsErrorCode.TTS_WEEKEND_ONLY:
        suggestions.push('选择支持工作日的 TTS 环境');
        suggestions.push('使用 SimNow 或其他模拟环境');
        suggestions.push('等待周末时间再使用此环境');
        break;

      default:
        suggestions.push('检查网络连接');
        suggestions.push('尝试其他测试环境');
        suggestions.push('查看详细错误日志');
        break;
    }

    return suggestions;
  }

  /**
   * 检查是否为可重试的错误
   */
  static isRetryableError(error: CtpError): boolean {
    const retryableCodes = [
      TtsErrorCode.TTS_SERVICE_UNAVAILABLE,
      TtsErrorCode.TTS_CONNECTION_TIMEOUT,
    ];

    return retryableCodes.includes(error.code as TtsErrorCode);
  }

  /**
   * 检查是否为致命错误
   */
  static isFatalError(error: CtpError): boolean {
    const fatalCodes = [
      TtsErrorCode.TTS_INVALID_CONFIG,
      TtsErrorCode.TTS_WEEKEND_ONLY,
    ];

    return fatalCodes.includes(error.code as TtsErrorCode);
  }

  /**
   * 获取用户友好的错误消息
   */
  static getUserFriendlyMessage(error: CtpError): string {
    const message = this.handleTtsError(error);
    const suggestions = this.getSuggestions(error);

    if (suggestions.length > 0) {
      return `${message}\n\n建议解决方案：\n${suggestions.map(s => `• ${s}`).join('\n')}`;
    }

    return message;
  }

  /**
   * 创建 TTS 错误对象
   */
  static createTtsError(code: TtsErrorCode, message: string, details?: string): CtpError {
    return {
      code: code as any,
      message,
      details,
    };
  }
}

export default TtsErrorHandler;
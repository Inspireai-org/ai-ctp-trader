/**
 * 验证工具函数
 * 
 * 提供数据验证和格式检查功能
 */

import { OrderRequest, LoginCredentials, CtpConfig } from '../types';

/**
 * 验证结果接口
 */
export interface ValidationResult {
  /** 是否有效 */
  isValid: boolean;
  /** 错误消息 */
  errors: string[];
  /** 警告消息 */
  warnings?: string[] | undefined;
}

/**
 * 验证规则接口
 */
export interface ValidationRule<T> {
  /** 规则名称 */
  name: string;
  /** 验证函数 */
  validate: (value: T) => boolean;
  /** 错误消息 */
  message: string;
  /** 是否为警告（非阻塞） */
  isWarning?: boolean;
}

/**
 * 基础验证器类
 */
export class Validator<T> {
  private rules: ValidationRule<T>[] = [];

  /**
   * 添加验证规则
   * @param rule 验证规则
   * @returns 验证器实例
   */
  addRule(rule: ValidationRule<T>): this {
    this.rules.push(rule);
    return this;
  }

  /**
   * 验证数据
   * @param value 要验证的数据
   * @returns 验证结果
   */
  validate(value: T): ValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];

    for (const rule of this.rules) {
      if (!rule.validate(value)) {
        if (rule.isWarning) {
          warnings.push(rule.message);
        } else {
          errors.push(rule.message);
        }
      }
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings: warnings.length > 0 ? warnings : undefined,
    } as ValidationResult;
  }
}

/**
 * 通用验证函数
 */
export class ValidationUtils {
  /**
   * 验证是否为空
   * @param value 值
   * @returns 是否为空
   */
  static isEmpty(value: any): boolean {
    if (value === null || value === undefined) return true;
    if (typeof value === 'string') return value.trim().length === 0;
    if (Array.isArray(value)) return value.length === 0;
    if (typeof value === 'object') return Object.keys(value).length === 0;
    return false;
  }

  /**
   * 验证是否为有效数字
   * @param value 值
   * @returns 是否为有效数字
   */
  static isValidNumber(value: any): boolean {
    return typeof value === 'number' && !isNaN(value) && isFinite(value);
  }

  /**
   * 验证是否为正数
   * @param value 值
   * @returns 是否为正数
   */
  static isPositiveNumber(value: any): boolean {
    return this.isValidNumber(value) && value > 0;
  }

  /**
   * 验证是否为非负数
   * @param value 值
   * @returns 是否为非负数
   */
  static isNonNegativeNumber(value: any): boolean {
    return this.isValidNumber(value) && value >= 0;
  }

  /**
   * 验证数字是否在指定范围内
   * @param value 值
   * @param min 最小值
   * @param max 最大值
   * @returns 是否在范围内
   */
  static isInRange(value: number, min: number, max: number): boolean {
    return this.isValidNumber(value) && value >= min && value <= max;
  }

  /**
   * 验证字符串长度
   * @param value 字符串
   * @param minLength 最小长度
   * @param maxLength 最大长度
   * @returns 是否符合长度要求
   */
  static isValidLength(value: string, minLength: number = 0, maxLength: number = Infinity): boolean {
    if (typeof value !== 'string') return false;
    const length = value.trim().length;
    return length >= minLength && length <= maxLength;
  }

  /**
   * 验证正则表达式
   * @param value 字符串
   * @param pattern 正则表达式
   * @returns 是否匹配
   */
  static matchesPattern(value: string, pattern: RegExp): boolean {
    return typeof value === 'string' && pattern.test(value);
  }

  /**
   * 验证邮箱格式
   * @param email 邮箱地址
   * @returns 是否为有效邮箱
   */
  static isValidEmail(email: string): boolean {
    const emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return this.matchesPattern(email, emailPattern);
  }

  /**
   * 验证手机号格式（中国）
   * @param phone 手机号
   * @returns 是否为有效手机号
   */
  static isValidPhone(phone: string): boolean {
    const phonePattern = /^1[3-9]\d{9}$/;
    return this.matchesPattern(phone, phonePattern);
  }

  /**
   * 验证 URL 格式
   * @param url URL 地址
   * @returns 是否为有效 URL
   */
  static isValidUrl(url: string): boolean {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * 验证 IP 地址格式
   * @param ip IP 地址
   * @returns 是否为有效 IP
   */
  static isValidIP(ip: string): boolean {
    const ipPattern = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
    return this.matchesPattern(ip, ipPattern);
  }

  /**
   * 验证端口号
   * @param port 端口号
   * @returns 是否为有效端口
   */
  static isValidPort(port: number): boolean {
    return this.isValidNumber(port) && this.isInRange(port, 1, 65535);
  }
}

/**
 * 合约代码验证
 * @param instrumentId 合约代码
 * @returns 验证结果
 */
export const validateInstrumentId = (instrumentId: string): ValidationResult => {
  const validator = new Validator<string>()
    .addRule({
      name: 'required',
      validate: (value) => !ValidationUtils.isEmpty(value),
      message: '合约代码不能为空',
    })
    .addRule({
      name: 'length',
      validate: (value) => ValidationUtils.isValidLength(value, 1, 20),
      message: '合约代码长度应在1-20个字符之间',
    })
    .addRule({
      name: 'format',
      validate: (value) => ValidationUtils.matchesPattern(value, /^[A-Za-z0-9]+$/),
      message: '合约代码只能包含字母和数字',
    });

  return validator.validate(instrumentId);
};

/**
 * 价格验证
 * @param price 价格
 * @param priceTick 最小变动价位
 * @returns 验证结果
 */
export const validatePrice = (price: number, priceTick: number = 0.01): ValidationResult => {
  const validator = new Validator<number>()
    .addRule({
      name: 'required',
      validate: (value) => ValidationUtils.isValidNumber(value),
      message: '价格必须是有效数字',
    })
    .addRule({
      name: 'positive',
      validate: (value) => ValidationUtils.isPositiveNumber(value),
      message: '价格必须大于0',
    })
    .addRule({
      name: 'tick',
      validate: (value) => {
        if (!ValidationUtils.isValidNumber(value) || !ValidationUtils.isPositiveNumber(priceTick)) {
          return true; // 如果基础验证失败，跳过此规则
        }
        const remainder = (value % priceTick);
        return Math.abs(remainder) < 0.0001 || Math.abs(remainder - priceTick) < 0.0001;
      },
      message: `价格必须是最小变动价位(${priceTick})的整数倍`,
      isWarning: true,
    });

  return validator.validate(price);
};

/**
 * 数量验证
 * @param volume 数量
 * @returns 验证结果
 */
export const validateVolume = (volume: number): ValidationResult => {
  const validator = new Validator<number>()
    .addRule({
      name: 'required',
      validate: (value) => ValidationUtils.isValidNumber(value),
      message: '数量必须是有效数字',
    })
    .addRule({
      name: 'positive',
      validate: (value) => ValidationUtils.isPositiveNumber(value),
      message: '数量必须大于0',
    })
    .addRule({
      name: 'integer',
      validate: (value) => Number.isInteger(value),
      message: '数量必须是整数',
    })
    .addRule({
      name: 'reasonable',
      validate: (value) => ValidationUtils.isInRange(value, 1, 10000),
      message: '数量应在1-10000之间',
      isWarning: true,
    });

  return validator.validate(volume);
};

/**
 * 订单请求验证
 * @param order 订单请求
 * @returns 验证结果
 */
export const validateOrderRequest = (order: OrderRequest): ValidationResult => {
  const errors: string[] = [];
  const warnings: string[] = [];

  // 验证合约代码
  const instrumentResult = validateInstrumentId(order.instrumentId);
  if (!instrumentResult.isValid) {
    errors.push(...instrumentResult.errors);
  }
  if (instrumentResult.warnings) {
    warnings.push(...instrumentResult.warnings);
  }

  // 验证价格（限价单需要价格）
  if (order.orderType === 'Limit') {
    const priceResult = validatePrice(order.price);
    if (!priceResult.isValid) {
      errors.push(...priceResult.errors);
    }
    if (priceResult.warnings) {
      warnings.push(...priceResult.warnings);
    }
  }

  // 验证数量
  const volumeResult = validateVolume(order.volume);
  if (!volumeResult.isValid) {
    errors.push(...volumeResult.errors);
  }
  if (volumeResult.warnings) {
    warnings.push(...volumeResult.warnings);
  }

  // 验证买卖方向
  if (!['Buy', 'Sell'].includes(order.direction)) {
    errors.push('买卖方向无效');
  }

  // 验证开平仓标志
  if (!['Open', 'Close', 'CloseToday', 'CloseYesterday'].includes(order.offsetFlag)) {
    errors.push('开平仓标志无效');
  }

  // 验证订单类型
  if (!['Limit', 'Market', 'Conditional'].includes(order.orderType)) {
    errors.push('订单类型无效');
  }

  // 验证时间条件
  if (!['IOC', 'FOK', 'GFD'].includes(order.timeCondition)) {
    errors.push('时间条件无效');
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings: warnings.length > 0 ? warnings : undefined,
  };
};

/**
 * 登录凭据验证
 * @param credentials 登录凭据
 * @returns 验证结果
 */
export const validateLoginCredentials = (credentials: LoginCredentials): ValidationResult => {
  const errors: string[] = [];

  if (ValidationUtils.isEmpty(credentials.brokerId)) {
    errors.push('经纪商代码不能为空');
  }

  if (ValidationUtils.isEmpty(credentials.userId)) {
    errors.push('用户ID不能为空');
  }

  if (ValidationUtils.isEmpty(credentials.password)) {
    errors.push('密码不能为空');
  }

  if (ValidationUtils.isEmpty(credentials.appId)) {
    errors.push('应用ID不能为空');
  }

  if (ValidationUtils.isEmpty(credentials.authCode)) {
    errors.push('授权码不能为空');
  }

  // 验证用户ID格式
  if (!ValidationUtils.isEmpty(credentials.userId) && 
      !ValidationUtils.isValidLength(credentials.userId, 1, 20)) {
    errors.push('用户ID长度应在1-20个字符之间');
  }

  // 验证密码强度
  if (!ValidationUtils.isEmpty(credentials.password) && 
      !ValidationUtils.isValidLength(credentials.password, 6, 50)) {
    errors.push('密码长度应在6-50个字符之间');
  }

  return {
    isValid: errors.length === 0,
    errors,
  };
};

/**
 * CTP 配置验证
 * @param config CTP 配置
 * @returns 验证结果
 */
export const validateCtpConfig = (config: CtpConfig): ValidationResult => {
  const errors: string[] = [];
  const warnings: string[] = [];

  // 验证必填字段
  if (ValidationUtils.isEmpty(config.brokerId)) {
    errors.push('经纪商代码不能为空');
  }

  if (ValidationUtils.isEmpty(config.investorId)) {
    errors.push('投资者代码不能为空');
  }

  if (ValidationUtils.isEmpty(config.password)) {
    errors.push('密码不能为空');
  }

  if (ValidationUtils.isEmpty(config.mdFrontAddr)) {
    errors.push('行情前置地址不能为空');
  }

  if (ValidationUtils.isEmpty(config.traderFrontAddr)) {
    errors.push('交易前置地址不能为空');
  }

  // 验证前置地址格式
  if (!ValidationUtils.isEmpty(config.mdFrontAddr)) {
    if (!config.mdFrontAddr.startsWith('tcp://')) {
      errors.push('行情前置地址格式错误，应以tcp://开头');
    } else {
      const urlPart = config.mdFrontAddr.replace('tcp://', '');
      const [host, port] = urlPart.split(':');
      
      if (!host) {
        errors.push('行情前置地址缺少主机名');
      }
      
      if (!port || !ValidationUtils.isValidPort(parseInt(port))) {
        errors.push('行情前置地址端口号无效');
      }
    }
  }

  if (!ValidationUtils.isEmpty(config.traderFrontAddr)) {
    if (!config.traderFrontAddr.startsWith('tcp://')) {
      errors.push('交易前置地址格式错误，应以tcp://开头');
    } else {
      const urlPart = config.traderFrontAddr.replace('tcp://', '');
      const [host, port] = urlPart.split(':');
      
      if (!host) {
        errors.push('交易前置地址缺少主机名');
      }
      
      if (!port || !ValidationUtils.isValidPort(parseInt(port))) {
        errors.push('交易前置地址端口号无效');
      }
    }
  }

  // 验证超时设置
  if (!ValidationUtils.isPositiveNumber(config.timeoutSecs)) {
    errors.push('超时时间必须是正数');
  } else if (config.timeoutSecs < 5) {
    warnings.push('超时时间过短，建议至少5秒');
  } else if (config.timeoutSecs > 300) {
    warnings.push('超时时间过长，建议不超过300秒');
  }

  // 验证重连设置
  if (!ValidationUtils.isNonNegativeNumber(config.maxReconnectAttempts)) {
    errors.push('最大重连次数必须是非负数');
  } else if (config.maxReconnectAttempts > 10) {
    warnings.push('最大重连次数过多，建议不超过10次');
  }

  if (!ValidationUtils.isPositiveNumber(config.reconnectIntervalSecs)) {
    errors.push('重连间隔必须是正数');
  } else if (config.reconnectIntervalSecs < 1) {
    warnings.push('重连间隔过短，建议至少1秒');
  }

  // 验证环境设置
  if (!['simnow', 'tts', 'production'].includes(config.environment)) {
    errors.push('环境设置无效');
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings: warnings.length > 0 ? warnings : undefined,
  };
};

/**
 * 批量验证
 * @param items 要验证的项目数组
 * @param validator 验证函数
 * @returns 批量验证结果
 */
export const validateBatch = <T>(
  items: T[],
  validator: (item: T) => ValidationResult
): {
  isValid: boolean;
  results: ValidationResult[];
  totalErrors: number;
  totalWarnings: number;
} => {
  const results = items.map(validator);
  const totalErrors = results.reduce((sum, result) => sum + result.errors.length, 0);
  const totalWarnings = results.reduce((sum, result) => sum + (result.warnings?.length || 0), 0);

  return {
    isValid: totalErrors === 0,
    results,
    totalErrors,
    totalWarnings,
  };
};
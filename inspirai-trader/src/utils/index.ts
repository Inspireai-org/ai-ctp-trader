/**
 * 工具函数导出
 * 
 * 统一导出所有工具函数模块
 */

// 格式化工具
export * from './format';

// 颜色工具
export * from './color';

// 性能优化工具
export * from './performance';

// 数值计算工具
export * from './calculation';

// 验证工具
export * from './validation';

// 日期时间工具
export * from './date';

// 重新导出常用工具类和函数
export {
  // 颜色工具
  ColorUtils,
  darkTheme,
  lightTheme,
  getPriceColor,
  getPercentColor,
  getPnLColor,
  getPositionColor,
  getDirectionColor,
  getOrderStatusColor,
  getRiskRatioColor,
  getConnectionStatusColor,
} from './color';

export {
  // 性能工具
  LRUCache,
  PerformanceMonitor,
  performanceMonitor,
  debounce,
  throttle,
  rafThrottle,
  createMemoizedFunction,
  batchProcess,

  measurePerformance,
  createMeasuredFunction,
} from './performance';

export {
  // 计算工具
  MathUtils,
  calculateChangePercent,
  calculateChangeAmount,
  calculateMargin,
  calculateCommission,
  calculatePositionPnL,
  calculateRiskRatio,
  calculateMaxVolume,
  calculateTradeAmount,
  calculateAveragePrice,
  calculatePriceTicks,
  adjustPriceToTick,
  calculateStopLossPrice,
  calculateTakeProfitPrice,
  calculateCapitalUtilization,
  calculateReturnRate,
  calculateAnnualizedReturn,
  calculateMaxDrawdown,
  calculateSharpeRatio,
} from './calculation';

export {
  // 验证工具
  ValidationUtils,
  Validator,
  validateInstrumentId,
  validatePrice,
  validateVolume,
  validateOrderRequest,
  validateLoginCredentials,
  validateCtpConfig,
  validateBatch,
} from './validation';

export {
  // 日期工具
  DateUtils,
  TradingCalendar,
  DateFormat,
  TimePeriod,
} from './date';
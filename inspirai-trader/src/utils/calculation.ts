/**
 * 数值计算工具函数
 * 
 * 提供期货交易相关的计算功能
 */

/**
 * 计算涨跌幅
 * @param currentPrice 当前价格
 * @param previousPrice 前一价格
 * @returns 涨跌幅（小数形式，如 0.05 表示 5%）
 */
export const calculateChangePercent = (currentPrice: number, previousPrice: number): number => {
  if (previousPrice === 0 || isNaN(currentPrice) || isNaN(previousPrice)) {
    return 0;
  }
  
  return (currentPrice - previousPrice) / previousPrice;
};

/**
 * 计算涨跌额
 * @param currentPrice 当前价格
 * @param previousPrice 前一价格
 * @returns 涨跌额
 */
export const calculateChangeAmount = (currentPrice: number, previousPrice: number): number => {
  if (isNaN(currentPrice) || isNaN(previousPrice)) {
    return 0;
  }
  
  return currentPrice - previousPrice;
};

/**
 * 计算保证金
 * @param price 价格
 * @param volume 数量
 * @param marginRate 保证金率
 * @param volumeMultiple 交易单位
 * @returns 保证金金额
 */
export const calculateMargin = (
  price: number,
  volume: number,
  marginRate: number,
  volumeMultiple: number = 1
): number => {
  if (isNaN(price) || isNaN(volume) || isNaN(marginRate) || isNaN(volumeMultiple)) {
    return 0;
  }
  
  return price * volume * marginRate * volumeMultiple;
};

/**
 * 计算手续费
 * @param price 价格
 * @param volume 数量
 * @param commissionRate 手续费率
 * @param volumeMultiple 交易单位
 * @param isPercentage 是否为百分比费率
 * @returns 手续费金额
 */
export const calculateCommission = (
  price: number,
  volume: number,
  commissionRate: number,
  volumeMultiple: number = 1,
  isPercentage: boolean = true
): number => {
  if (isNaN(price) || isNaN(volume) || isNaN(commissionRate) || isNaN(volumeMultiple)) {
    return 0;
  }
  
  if (isPercentage) {
    // 按比例收费
    return price * volume * commissionRate * volumeMultiple;
  } else {
    // 按手数收费
    return volume * commissionRate;
  }
};

/**
 * 计算持仓盈亏
 * @param openPrice 开仓价格
 * @param currentPrice 当前价格
 * @param volume 持仓量
 * @param direction 持仓方向 ('Long' | 'Short')
 * @param volumeMultiple 交易单位
 * @returns 盈亏金额
 */
export const calculatePositionPnL = (
  openPrice: number,
  currentPrice: number,
  volume: number,
  direction: 'Long' | 'Short',
  volumeMultiple: number = 1
): number => {
  if (isNaN(openPrice) || isNaN(currentPrice) || isNaN(volume) || isNaN(volumeMultiple)) {
    return 0;
  }
  
  const priceDiff = currentPrice - openPrice;
  const multiplier = direction === 'Long' ? 1 : -1;
  
  return priceDiff * volume * multiplier * volumeMultiple;
};

/**
 * 计算风险度
 * @param usedMargin 已用保证金
 * @param availableBalance 可用资金
 * @returns 风险度（0-1之间的小数）
 */
export const calculateRiskRatio = (usedMargin: number, availableBalance: number): number => {
  if (isNaN(usedMargin) || isNaN(availableBalance) || availableBalance <= 0) {
    return 0;
  }
  
  const totalBalance = usedMargin + availableBalance;
  return totalBalance > 0 ? usedMargin / totalBalance : 0;
};

/**
 * 计算可开仓手数
 * @param availableBalance 可用资金
 * @param price 价格
 * @param marginRate 保证金率
 * @param volumeMultiple 交易单位
 * @returns 可开仓手数
 */
export const calculateMaxVolume = (
  availableBalance: number,
  price: number,
  marginRate: number,
  volumeMultiple: number = 1
): number => {
  if (isNaN(availableBalance) || isNaN(price) || isNaN(marginRate) || isNaN(volumeMultiple)) {
    return 0;
  }
  
  if (price <= 0 || marginRate <= 0 || volumeMultiple <= 0) {
    return 0;
  }
  
  const marginPerHand = price * marginRate * volumeMultiple;
  return Math.floor(availableBalance / marginPerHand);
};

/**
 * 计算成交金额
 * @param price 成交价格
 * @param volume 成交数量
 * @param volumeMultiple 交易单位
 * @returns 成交金额
 */
export const calculateTradeAmount = (
  price: number,
  volume: number,
  volumeMultiple: number = 1
): number => {
  if (isNaN(price) || isNaN(volume) || isNaN(volumeMultiple)) {
    return 0;
  }
  
  return price * volume * volumeMultiple;
};

/**
 * 计算平均价格
 * @param prices 价格数组
 * @param volumes 对应的数量数组
 * @returns 加权平均价格
 */
export const calculateAveragePrice = (prices: number[], volumes: number[]): number => {
  if (prices.length !== volumes.length || prices.length === 0) {
    return 0;
  }
  
  let totalAmount = 0;
  let totalVolume = 0;
  
  for (let i = 0; i < prices.length; i++) {
    const price = prices[i];
    const volume = volumes[i];
    if (price !== undefined && volume !== undefined && !isNaN(price) && !isNaN(volume)) {
      totalAmount += price * volume;
      totalVolume += volume;
    }
  }
  
  return totalVolume > 0 ? totalAmount / totalVolume : 0;
};

/**
 * 计算价格变动的最小单位数
 * @param price1 价格1
 * @param price2 价格2
 * @param priceTick 最小变动价位
 * @returns 最小单位数
 */
export const calculatePriceTicks = (price1: number, price2: number, priceTick: number): number => {
  if (isNaN(price1) || isNaN(price2) || isNaN(priceTick) || priceTick <= 0) {
    return 0;
  }
  
  return Math.round((price2 - price1) / priceTick);
};

/**
 * 调整价格到最小变动价位
 * @param price 原始价格
 * @param priceTick 最小变动价位
 * @param roundUp 是否向上取整
 * @returns 调整后的价格
 */
export const adjustPriceToTick = (price: number, priceTick: number, roundUp: boolean = false): number => {
  if (isNaN(price) || isNaN(priceTick) || priceTick <= 0) {
    return price;
  }
  
  const ticks = price / priceTick;
  const adjustedTicks = roundUp ? Math.ceil(ticks) : Math.round(ticks);
  
  return adjustedTicks * priceTick;
};

/**
 * 计算止损价格
 * @param entryPrice 入场价格
 * @param stopLossPercent 止损百分比
 * @param direction 方向 ('Long' | 'Short')
 * @param priceTick 最小变动价位
 * @returns 止损价格
 */
export const calculateStopLossPrice = (
  entryPrice: number,
  stopLossPercent: number,
  direction: 'Long' | 'Short',
  priceTick: number = 0.01
): number => {
  if (isNaN(entryPrice) || isNaN(stopLossPercent) || stopLossPercent <= 0) {
    return entryPrice;
  }
  
  const multiplier = direction === 'Long' ? (1 - stopLossPercent) : (1 + stopLossPercent);
  const stopPrice = entryPrice * multiplier;
  
  return adjustPriceToTick(stopPrice, priceTick);
};

/**
 * 计算止盈价格
 * @param entryPrice 入场价格
 * @param takeProfitPercent 止盈百分比
 * @param direction 方向 ('Long' | 'Short')
 * @param priceTick 最小变动价位
 * @returns 止盈价格
 */
export const calculateTakeProfitPrice = (
  entryPrice: number,
  takeProfitPercent: number,
  direction: 'Long' | 'Short',
  priceTick: number = 0.01
): number => {
  if (isNaN(entryPrice) || isNaN(takeProfitPercent) || takeProfitPercent <= 0) {
    return entryPrice;
  }
  
  const multiplier = direction === 'Long' ? (1 + takeProfitPercent) : (1 - takeProfitPercent);
  const profitPrice = entryPrice * multiplier;
  
  return adjustPriceToTick(profitPrice, priceTick);
};

/**
 * 计算资金使用率
 * @param usedMargin 已用保证金
 * @param totalBalance 总资金
 * @returns 资金使用率（0-1之间的小数）
 */
export const calculateCapitalUtilization = (usedMargin: number, totalBalance: number): number => {
  if (isNaN(usedMargin) || isNaN(totalBalance) || totalBalance <= 0) {
    return 0;
  }
  
  return Math.min(usedMargin / totalBalance, 1);
};

/**
 * 计算收益率
 * @param initialBalance 初始资金
 * @param currentBalance 当前资金
 * @returns 收益率（小数形式）
 */
export const calculateReturnRate = (initialBalance: number, currentBalance: number): number => {
  if (isNaN(initialBalance) || isNaN(currentBalance) || initialBalance <= 0) {
    return 0;
  }
  
  return (currentBalance - initialBalance) / initialBalance;
};

/**
 * 计算年化收益率
 * @param initialBalance 初始资金
 * @param currentBalance 当前资金
 * @param days 持有天数
 * @returns 年化收益率（小数形式）
 */
export const calculateAnnualizedReturn = (
  initialBalance: number,
  currentBalance: number,
  days: number
): number => {
  if (isNaN(initialBalance) || isNaN(currentBalance) || isNaN(days) || 
      initialBalance <= 0 || days <= 0) {
    return 0;
  }
  
  const returnRate = calculateReturnRate(initialBalance, currentBalance);
  return Math.pow(1 + returnRate, 365 / days) - 1;
};

/**
 * 计算最大回撤
 * @param balanceHistory 资金历史记录
 * @returns 最大回撤（小数形式）
 */
export const calculateMaxDrawdown = (balanceHistory: number[]): number => {
  if (balanceHistory.length < 2) {
    return 0;
  }
  
  let maxDrawdown = 0;
  let peak = balanceHistory[0];
  
  if (peak === undefined) {
    return 0;
  }
  
  for (let i = 1; i < balanceHistory.length; i++) {
    const current = balanceHistory[i];
    
    if (current === undefined) {
      continue;
    }
    
    if (current > peak) {
      peak = current;
    } else {
      const drawdown = (peak - current) / peak;
      maxDrawdown = Math.max(maxDrawdown, drawdown);
    }
  }
  
  return maxDrawdown;
};

/**
 * 计算夏普比率
 * @param returns 收益率数组
 * @param riskFreeRate 无风险收益率
 * @returns 夏普比率
 */
export const calculateSharpeRatio = (returns: number[], riskFreeRate: number = 0): number => {
  if (returns.length < 2) {
    return 0;
  }
  
  const avgReturn = returns.reduce((sum, ret) => sum + ret, 0) / returns.length;
  const excessReturn = avgReturn - riskFreeRate;
  
  const variance = returns.reduce((sum, ret) => {
    return sum + Math.pow(ret - avgReturn, 2);
  }, 0) / (returns.length - 1);
  
  const standardDeviation = Math.sqrt(variance);
  
  return standardDeviation > 0 ? excessReturn / standardDeviation : 0;
};

/**
 * 数学工具类
 */
export class MathUtils {
  /**
   * 安全除法（避免除零错误）
   * @param dividend 被除数
   * @param divisor 除数
   * @param defaultValue 默认值
   * @returns 除法结果
   */
  static safeDivide(dividend: number, divisor: number, defaultValue: number = 0): number {
    return divisor !== 0 ? dividend / divisor : defaultValue;
  }

  /**
   * 限制数值在指定范围内
   * @param value 数值
   * @param min 最小值
   * @param max 最大值
   * @returns 限制后的数值
   */
  static clamp(value: number, min: number, max: number): number {
    return Math.min(Math.max(value, min), max);
  }

  /**
   * 线性插值
   * @param start 起始值
   * @param end 结束值
   * @param factor 插值因子（0-1）
   * @returns 插值结果
   */
  static lerp(start: number, end: number, factor: number): number {
    return start + (end - start) * this.clamp(factor, 0, 1);
  }

  /**
   * 检查数值是否在指定范围内
   * @param value 数值
   * @param min 最小值
   * @param max 最大值
   * @returns 是否在范围内
   */
  static inRange(value: number, min: number, max: number): boolean {
    return value >= min && value <= max;
  }

  /**
   * 四舍五入到指定小数位
   * @param value 数值
   * @param decimals 小数位数
   * @returns 四舍五入后的数值
   */
  static round(value: number, decimals: number = 0): number {
    const factor = Math.pow(10, decimals);
    return Math.round(value * factor) / factor;
  }

  /**
   * 计算数组的统计信息
   * @param values 数值数组
   * @returns 统计信息
   */
  static getStatistics(values: number[]): {
    count: number;
    sum: number;
    mean: number;
    median: number;
    min: number;
    max: number;
    variance: number;
    standardDeviation: number;
  } {
    if (values.length === 0) {
      return {
        count: 0,
        sum: 0,
        mean: 0,
        median: 0,
        min: 0,
        max: 0,
        variance: 0,
        standardDeviation: 0,
      };
    }

    const count = values.length;
    const sum = values.reduce((acc, val) => acc + val, 0);
    const mean = sum / count;

    const sorted = [...values].sort((a, b) => a - b);
    const median = count % 2 === 0
      ? ((sorted[count / 2 - 1] ?? 0) + (sorted[count / 2] ?? 0)) / 2
      : (sorted[Math.floor(count / 2)] ?? 0);

    const min = Math.min(...values);
    const max = Math.max(...values);

    const variance = values.reduce((acc, val) => acc + Math.pow(val - mean, 2), 0) / count;
    const standardDeviation = Math.sqrt(variance);

    return {
      count,
      sum,
      mean,
      median,
      min,
      max,
      variance,
      standardDeviation,
    };
  }
}
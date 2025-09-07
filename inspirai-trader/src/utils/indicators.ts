/**
 * 技术指标计算工具
 */

import { KlineData } from '@/types';

/**
 * 计算简单移动平均线 (SMA)
 */
export function calculateSMA(data: number[], period: number): (number | null)[] {
  const result: (number | null)[] = [];
  
  for (let i = 0; i < data.length; i++) {
    if (i < period - 1) {
      result.push(null);
    } else {
      const sum = data.slice(i - period + 1, i + 1).reduce((a, b) => a + b, 0);
      result.push(sum / period);
    }
  }
  
  return result;
}

/**
 * 计算指数移动平均线 (EMA)
 */
export function calculateEMA(data: number[], period: number): (number | null)[] {
  const result: (number | null)[] = [];
  const multiplier = 2 / (period + 1);
  
  // 计算初始SMA作为第一个EMA值
  let ema = data.slice(0, period).reduce((a, b) => a + b, 0) / period;
  
  for (let i = 0; i < data.length; i++) {
    if (i < period - 1) {
      result.push(null);
    } else if (i === period - 1) {
      result.push(ema);
    } else {
      ema = (data[i] - ema) * multiplier + ema;
      result.push(ema);
    }
  }
  
  return result;
}

/**
 * 计算MACD指标
 */
export function calculateMACD(
  data: number[],
  fastPeriod = 12,
  slowPeriod = 26,
  signalPeriod = 9
): {
  macd: (number | null)[];
  signal: (number | null)[];
  histogram: (number | null)[];
} {
  const fastEMA = calculateEMA(data, fastPeriod);
  const slowEMA = calculateEMA(data, slowPeriod);
  
  const macd: (number | null)[] = [];
  const macdValues: number[] = [];
  
  for (let i = 0; i < data.length; i++) {
    if (fastEMA[i] !== null && slowEMA[i] !== null) {
      const value = fastEMA[i]! - slowEMA[i]!;
      macd.push(value);
      macdValues.push(value);
    } else {
      macd.push(null);
    }
  }
  
  const signal = calculateEMA(macdValues, signalPeriod);
  const histogram: (number | null)[] = [];
  
  for (let i = 0; i < macd.length; i++) {
    if (macd[i] !== null && signal[i] !== null) {
      histogram.push(macd[i]! - signal[i]!);
    } else {
      histogram.push(null);
    }
  }
  
  return { macd, signal, histogram };
}

/**
 * 计算RSI指标
 */
export function calculateRSI(data: number[], period = 14): (number | null)[] {
  const result: (number | null)[] = [];
  const gains: number[] = [];
  const losses: number[] = [];
  
  for (let i = 1; i < data.length; i++) {
    const change = data[i] - data[i - 1];
    gains.push(change > 0 ? change : 0);
    losses.push(change < 0 ? Math.abs(change) : 0);
  }
  
  result.push(null); // 第一个值为null
  
  for (let i = 0; i < gains.length; i++) {
    if (i < period - 1) {
      result.push(null);
    } else {
      const avgGain = gains.slice(i - period + 1, i + 1).reduce((a, b) => a + b, 0) / period;
      const avgLoss = losses.slice(i - period + 1, i + 1).reduce((a, b) => a + b, 0) / period;
      
      if (avgLoss === 0) {
        result.push(100);
      } else {
        const rs = avgGain / avgLoss;
        result.push(100 - (100 / (1 + rs)));
      }
    }
  }
  
  return result;
}

/**
 * 计算布林带 (Bollinger Bands)
 */
export function calculateBollingerBands(
  data: number[],
  period = 20,
  stdDev = 2
): {
  upper: (number | null)[];
  middle: (number | null)[];
  lower: (number | null)[];
} {
  const middle = calculateSMA(data, period);
  const upper: (number | null)[] = [];
  const lower: (number | null)[] = [];
  
  for (let i = 0; i < data.length; i++) {
    if (i < period - 1) {
      upper.push(null);
      lower.push(null);
    } else {
      const slice = data.slice(i - period + 1, i + 1);
      const mean = middle[i]!;
      const variance = slice.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / period;
      const std = Math.sqrt(variance);
      
      upper.push(mean + stdDev * std);
      lower.push(mean - stdDev * std);
    }
  }
  
  return { upper, middle, lower };
}

/**
 * 计算KDJ指标
 */
export function calculateKDJ(
  high: number[],
  low: number[],
  close: number[],
  period = 9,
  kPeriod = 3,
  dPeriod = 3
): {
  k: (number | null)[];
  d: (number | null)[];
  j: (number | null)[];
} {
  const rsv: (number | null)[] = [];
  
  for (let i = 0; i < close.length; i++) {
    if (i < period - 1) {
      rsv.push(null);
    } else {
      const highestHigh = Math.max(...high.slice(i - period + 1, i + 1));
      const lowestLow = Math.min(...low.slice(i - period + 1, i + 1));
      const range = highestHigh - lowestLow;
      
      if (range === 0) {
        rsv.push(50);
      } else {
        rsv.push(((close[i] - lowestLow) / range) * 100);
      }
    }
  }
  
  const k: (number | null)[] = [];
  const d: (number | null)[] = [];
  const j: (number | null)[] = [];
  
  let prevK = 50;
  let prevD = 50;
  
  for (let i = 0; i < rsv.length; i++) {
    if (rsv[i] === null) {
      k.push(null);
      d.push(null);
      j.push(null);
    } else {
      const currentK = (rsv[i]! * (1 / kPeriod)) + (prevK * ((kPeriod - 1) / kPeriod));
      const currentD = (currentK * (1 / dPeriod)) + (prevD * ((dPeriod - 1) / dPeriod));
      const currentJ = 3 * currentK - 2 * currentD;
      
      k.push(currentK);
      d.push(currentD);
      j.push(currentJ);
      
      prevK = currentK;
      prevD = currentD;
    }
  }
  
  return { k, d, j };
}

/**
 * 从K线数据中提取价格数组
 */
export function extractPrices(klines: KlineData[], priceType: 'open' | 'high' | 'low' | 'close'): number[] {
  return klines.map(k => k[priceType]);
}
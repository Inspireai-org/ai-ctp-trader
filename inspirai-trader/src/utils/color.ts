/**
 * 颜色工具函数
 * 
 * 提供涨跌颜色计算、主题色彩等功能
 */

/**
 * 涨跌颜色类型
 */
export interface PriceColor {
  /** 文字颜色 */
  color: string;
  /** 背景颜色（可选） */
  backgroundColor?: string;
  /** CSS 类名 */
  className: string;
}

/**
 * 颜色主题配置
 */
export interface ColorTheme {
  /** 涨色（红色） */
  up: string;
  /** 跌色（绿色） */
  down: string;
  /** 平色（黄色/灰色） */
  neutral: string;
  /** 主色调 */
  primary: string;
  /** 成功色 */
  success: string;
  /** 警告色 */
  warning: string;
  /** 错误色 */
  error: string;
  /** 信息色 */
  info: string;
}

/**
 * 默认颜色主题（深色主题）
 */
export const darkTheme: ColorTheme = {
  up: '#00d4aa',      // 涨 - 绿色
  down: '#ff4d4f',    // 跌 - 红色
  neutral: '#faad14', // 平 - 黄色
  primary: '#1890ff', // 主色调 - 蓝色
  success: '#52c41a', // 成功 - 绿色
  warning: '#faad14', // 警告 - 橙色
  error: '#f5222d',   // 错误 - 红色
  info: '#1890ff',    // 信息 - 蓝色
};

/**
 * 浅色主题
 */
export const lightTheme: ColorTheme = {
  up: '#52c41a',      // 涨 - 绿色
  down: '#f5222d',    // 跌 - 红色
  neutral: '#faad14', // 平 - 黄色
  primary: '#1890ff', // 主色调 - 蓝色
  success: '#52c41a', // 成功 - 绿色
  warning: '#faad14', // 警告 - 橙色
  error: '#f5222d',   // 错误 - 红色
  info: '#1890ff',    // 信息 - 蓝色
};

/**
 * 当前主题（默认为深色主题）
 */
let currentTheme: ColorTheme = darkTheme;

/**
 * 设置当前主题
 * @param theme 主题配置
 */
export const setTheme = (theme: ColorTheme): void => {
  currentTheme = theme;
};

/**
 * 获取当前主题
 * @returns 当前主题配置
 */
export const getTheme = (): ColorTheme => {
  return currentTheme;
};

/**
 * 根据涨跌情况获取颜色
 * @param change 涨跌额
 * @param theme 主题配置，默认使用当前主题
 * @returns 颜色信息
 */
export const getPriceColor = (change: number, theme: ColorTheme = currentTheme): PriceColor => {
  if (change > 0) {
    return {
      color: theme.up,
      className: 'text-up',
    };
  } else if (change < 0) {
    return {
      color: theme.down,
      className: 'text-down',
    };
  } else {
    return {
      color: theme.neutral,
      className: 'text-neutral',
    };
  }
};

/**
 * 根据涨跌幅获取颜色
 * @param changePercent 涨跌幅
 * @param theme 主题配置，默认使用当前主题
 * @returns 颜色信息
 */
export const getPercentColor = (changePercent: number, theme: ColorTheme = currentTheme): PriceColor => {
  return getPriceColor(changePercent, theme);
};

/**
 * 根据盈亏获取颜色
 * @param pnl 盈亏金额
 * @param theme 主题配置，默认使用当前主题
 * @returns 颜色信息
 */
export const getPnLColor = (pnl: number, theme: ColorTheme = currentTheme): PriceColor => {
  return getPriceColor(pnl, theme);
};

/**
 * 根据持仓方向获取颜色
 * @param direction 持仓方向 ('Long' | 'Short')
 * @param theme 主题配置，默认使用当前主题
 * @returns 颜色信息
 */
export const getPositionColor = (direction: string, theme: ColorTheme = currentTheme): PriceColor => {
  if (direction === 'Long') {
    return {
      color: theme.up,
      className: 'text-long',
    };
  } else if (direction === 'Short') {
    return {
      color: theme.down,
      className: 'text-short',
    };
  } else {
    return {
      color: theme.neutral,
      className: 'text-neutral',
    };
  }
};

/**
 * 根据买卖方向获取颜色
 * @param direction 买卖方向 ('Buy' | 'Sell')
 * @param theme 主题配置，默认使用当前主题
 * @returns 颜色信息
 */
export const getDirectionColor = (direction: string, theme: ColorTheme = currentTheme): PriceColor => {
  if (direction === 'Buy') {
    return {
      color: theme.up,
      className: 'text-buy',
    };
  } else if (direction === 'Sell') {
    return {
      color: theme.down,
      className: 'text-sell',
    };
  } else {
    return {
      color: theme.neutral,
      className: 'text-neutral',
    };
  }
};

/**
 * 根据订单状态获取颜色
 * @param status 订单状态
 * @param theme 主题配置，默认使用当前主题
 * @returns 颜色信息
 */
export const getOrderStatusColor = (status: string, theme: ColorTheme = currentTheme): PriceColor => {
  switch (status) {
    case 'AllTraded':
      return {
        color: theme.success,
        className: 'text-success',
      };
    case 'PartTradedQueueing':
    case 'NoTradeQueueing':
      return {
        color: theme.info,
        className: 'text-info',
      };
    case 'Canceled':
      return {
        color: theme.neutral,
        className: 'text-neutral',
      };
    case 'PartTradedNotQueueing':
    case 'NoTradeNotQueueing':
      return {
        color: theme.warning,
        className: 'text-warning',
      };
    default:
      return {
        color: theme.neutral,
        className: 'text-neutral',
      };
  }
};

/**
 * 根据风险度获取颜色
 * @param riskRatio 风险度（0-1之间）
 * @param theme 主题配置，默认使用当前主题
 * @returns 颜色信息
 */
export const getRiskRatioColor = (riskRatio: number, theme: ColorTheme = currentTheme): PriceColor => {
  if (riskRatio >= 0.8) {
    return {
      color: theme.error,
      className: 'text-error',
    };
  } else if (riskRatio >= 0.6) {
    return {
      color: theme.warning,
      className: 'text-warning',
    };
  } else {
    return {
      color: theme.success,
      className: 'text-success',
    };
  }
};

/**
 * 获取连接状态颜色
 * @param status 连接状态
 * @param theme 主题配置，默认使用当前主题
 * @returns 颜色信息
 */
export const getConnectionStatusColor = (status: string, theme: ColorTheme = currentTheme): PriceColor => {
  switch (status) {
    case 'CONNECTED':
    case 'LOGGED_IN':
      return {
        color: theme.success,
        className: 'text-success',
      };
    case 'CONNECTING':
    case 'LOGGING_IN':
      return {
        color: theme.info,
        className: 'text-info',
      };
    case 'DISCONNECTED':
      return {
        color: theme.neutral,
        className: 'text-neutral',
      };
    case 'ERROR':
      return {
        color: theme.error,
        className: 'text-error',
      };
    default:
      return {
        color: theme.neutral,
        className: 'text-neutral',
      };
  }
};

/**
 * 颜色工具类
 */
export class ColorUtils {
  /**
   * 将十六进制颜色转换为 RGB
   * @param hex 十六进制颜色值
   * @returns RGB 值对象
   */
  static hexToRgb(hex: string): { r: number; g: number; b: number } | null {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
      r: parseInt(result[1] ?? '0', 16),
      g: parseInt(result[2] ?? '0', 16),
      b: parseInt(result[3] ?? '0', 16),
    } : null;
  }

  /**
   * 将 RGB 转换为十六进制颜色
   * @param r 红色值 (0-255)
   * @param g 绿色值 (0-255)
   * @param b 蓝色值 (0-255)
   * @returns 十六进制颜色值
   */
  static rgbToHex(r: number, g: number, b: number): string {
    return `#${((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)}`;
  }

  /**
   * 调整颜色亮度
   * @param hex 十六进制颜色值
   * @param percent 亮度调整百分比 (-100 到 100)
   * @returns 调整后的十六进制颜色值
   */
  static adjustBrightness(hex: string, percent: number): string {
    const rgb = this.hexToRgb(hex);
    if (!rgb) return hex;

    const factor = percent / 100;
    const adjust = (value: number) => {
      if (factor > 0) {
        return Math.round(value + (255 - value) * factor);
      } else {
        return Math.round(value * (1 + factor));
      }
    };

    return this.rgbToHex(
      Math.max(0, Math.min(255, adjust(rgb.r))),
      Math.max(0, Math.min(255, adjust(rgb.g))),
      Math.max(0, Math.min(255, adjust(rgb.b)))
    );
  }

  /**
   * 获取颜色的对比色（黑色或白色）
   * @param hex 十六进制颜色值
   * @returns 对比色（#000000 或 #ffffff）
   */
  static getContrastColor(hex: string): string {
    const rgb = this.hexToRgb(hex);
    if (!rgb) return '#000000';

    // 计算亮度
    const brightness = (rgb.r * 299 + rgb.g * 587 + rgb.b * 114) / 1000;
    return brightness > 128 ? '#000000' : '#ffffff';
  }

  /**
   * 混合两种颜色
   * @param color1 第一种颜色
   * @param color2 第二种颜色
   * @param ratio 混合比例 (0-1)
   * @returns 混合后的颜色
   */
  static mixColors(color1: string, color2: string, ratio: number): string {
    const rgb1 = this.hexToRgb(color1);
    const rgb2 = this.hexToRgb(color2);
    
    if (!rgb1 || !rgb2) return color1;

    const r = Math.round(rgb1.r * (1 - ratio) + rgb2.r * ratio);
    const g = Math.round(rgb1.g * (1 - ratio) + rgb2.g * ratio);
    const b = Math.round(rgb1.b * (1 - ratio) + rgb2.b * ratio);

    return this.rgbToHex(r, g, b);
  }
}
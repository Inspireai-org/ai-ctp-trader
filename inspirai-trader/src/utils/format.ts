/**
 * 格式化工具函数
 * 
 * 提供数字、价格、时间等格式化功能
 */

/**
 * 格式化价格显示
 * @param price 价格
 * @param precision 小数位数，默认2位
 * @returns 格式化后的价格字符串
 */
export const formatPrice = (price: number, precision: number = 2): string => {
  if (isNaN(price) || price === null || price === undefined) {
    return '--';
  }
  
  return price.toFixed(precision);
};

/**
 * 格式化数量显示
 * @param volume 数量
 * @returns 格式化后的数量字符串
 */
export const formatVolume = (volume: number): string => {
  if (isNaN(volume) || volume === null || volume === undefined) {
    return '--';
  }
  
  if (volume >= 10000) {
    return `${(volume / 10000).toFixed(1)}万`;
  }
  
  return volume.toString();
};

/**
 * 格式化百分比显示
 * @param value 数值
 * @param precision 小数位数，默认2位
 * @returns 格式化后的百分比字符串
 */
export const formatPercent = (value: number, precision: number = 2): string => {
  if (isNaN(value) || value === null || value === undefined) {
    return '--';
  }
  
  const sign = value > 0 ? '+' : '';
  return `${sign}${(value * 100).toFixed(precision)}%`;
};

/**
 * 格式化涨跌额显示
 * @param change 涨跌额
 * @param precision 小数位数，默认2位
 * @returns 格式化后的涨跌额字符串
 */
export const formatChange = (change: number, precision: number = 2): string => {
  if (isNaN(change) || change === null || change === undefined) {
    return '--';
  }
  
  const sign = change > 0 ? '+' : '';
  return `${sign}${change.toFixed(precision)}`;
};

/**
 * 格式化时间显示
 * @param timestamp 时间戳或时间字符串
 * @param format 格式，默认 'HH:mm:ss'
 * @returns 格式化后的时间字符串
 */
export const formatTime = (timestamp: number | string, format: string = 'HH:mm:ss'): string => {
  try {
    const date = typeof timestamp === 'string' ? new Date(timestamp) : new Date(timestamp);
    
    if (isNaN(date.getTime())) {
      return '--';
    }
    
    const hours = date.getHours().toString().padStart(2, '0');
    const minutes = date.getMinutes().toString().padStart(2, '0');
    const seconds = date.getSeconds().toString().padStart(2, '0');
    
    return format
      .replace('HH', hours)
      .replace('mm', minutes)
      .replace('ss', seconds);
  } catch {
    return '--';
  }
};

/**
 * 格式化日期显示
 * @param timestamp 时间戳或时间字符串
 * @param format 格式，默认 'YYYY-MM-DD'
 * @returns 格式化后的日期字符串
 */
export const formatDate = (timestamp: number | string, format: string = 'YYYY-MM-DD'): string => {
  try {
    const date = typeof timestamp === 'string' ? new Date(timestamp) : new Date(timestamp);
    
    if (isNaN(date.getTime())) {
      return '--';
    }
    
    const year = date.getFullYear().toString();
    const month = (date.getMonth() + 1).toString().padStart(2, '0');
    const day = date.getDate().toString().padStart(2, '0');
    
    return format
      .replace('YYYY', year)
      .replace('MM', month)
      .replace('DD', day);
  } catch {
    return '--';
  }
};

/**
 * 格式化金额显示（带千分位分隔符）
 * @param amount 金额
 * @param precision 小数位数，默认2位
 * @returns 格式化后的金额字符串
 */
export const formatAmount = (amount: number, precision: number = 2): string => {
  if (isNaN(amount) || amount === null || amount === undefined) {
    return '--';
  }
  
  return amount.toLocaleString('zh-CN', {
    minimumFractionDigits: precision,
    maximumFractionDigits: precision,
  });
};

/**
 * 格式化持仓量显示
 * @param position 持仓量
 * @returns 格式化后的持仓量字符串
 */
export const formatPosition = (position: number): string => {
  if (isNaN(position) || position === null || position === undefined) {
    return '--';
  }
  
  if (position === 0) {
    return '0';
  }
  
  return position.toString();
};

/**
 * 格式化盈亏显示
 * @param pnl 盈亏金额
 * @param precision 小数位数，默认2位
 * @returns 格式化后的盈亏字符串
 */
export const formatPnL = (pnl: number, precision: number = 2): string => {
  if (isNaN(pnl) || pnl === null || pnl === undefined) {
    return '--';
  }
  
  const sign = pnl > 0 ? '+' : '';
  return `${sign}${formatAmount(pnl, precision)}`;
};

/**
 * 格式化风险度显示
 * @param riskRatio 风险度（0-1之间的小数）
 * @returns 格式化后的风险度字符串
 */
export const formatRiskRatio = (riskRatio: number): string => {
  if (isNaN(riskRatio) || riskRatio === null || riskRatio === undefined) {
    return '--';
  }
  
  const percentage = (riskRatio * 100).toFixed(1);
  return `${percentage}%`;
};

/**
 * 格式化数字（带千分位）
 * @param num 数字
 * @returns 格式化后的数字字符串
 */
export const formatNumber = (num: number | undefined | null): string => {
  if (num === undefined || num === null || isNaN(num)) {
    return '--';
  }
  
  return new Intl.NumberFormat('zh-CN').format(num);
};

/**
 * 格式化合约代码显示
 * @param instrumentId 合约代码
 * @returns 格式化后的合约代码
 */
export const formatInstrumentId = (instrumentId: string): string => {
  if (!instrumentId) {
    return '--';
  }
  
  return instrumentId.toUpperCase();
};

/**
 * 格式化订单状态显示
 * @param status 订单状态
 * @returns 中文状态描述
 */
export const formatOrderStatus = (status: string): string => {
  const statusMap: Record<string, string> = {
    'Unknown': '未知',
    'AllTraded': '全部成交',
    'PartTradedQueueing': '部分成交',
    'PartTradedNotQueueing': '部分成交不在队列',
    'NoTradeQueueing': '未成交',
    'NoTradeNotQueueing': '未成交不在队列',
    'Canceled': '已撤单',
    'Touched': '已触发',
  };
  
  return statusMap[status] || status;
};

/**
 * 格式化买卖方向显示
 * @param direction 买卖方向
 * @returns 中文方向描述
 */
export const formatDirection = (direction: string): string => {
  const directionMap: Record<string, string> = {
    'Buy': '买入',
    'Sell': '卖出',
    'Long': '多头',
    'Short': '空头',
  };
  
  return directionMap[direction] || direction;
};

/**
 * 格式化开平仓标志显示
 * @param offsetFlag 开平仓标志
 * @returns 中文开平仓描述
 */
export const formatOffsetFlag = (offsetFlag: string): string => {
  const offsetMap: Record<string, string> = {
    'Open': '开仓',
    'Close': '平仓',
    'CloseToday': '平今',
    'CloseYesterday': '平昨',
  };
  
  return offsetMap[offsetFlag] || offsetFlag;
};

/**
 * 格式化订单类型显示
 * @param orderType 订单类型
 * @returns 中文订单类型描述
 */
export const formatOrderType = (orderType: string): string => {
  const typeMap: Record<string, string> = {
    'Limit': '限价单',
    'Market': '市价单',
    'Conditional': '条件单',
  };
  
  return typeMap[orderType] || orderType;
};

/**
 * 格式化时间条件显示
 * @param timeCondition 时间条件
 * @returns 中文时间条件描述
 */
export const formatTimeCondition = (timeCondition: string): string => {
  const conditionMap: Record<string, string> = {
    'IOC': '立即成交否则撤销',
    'FOK': '全部成交否则撤销',
    'GFD': '当日有效',
  };
  
  return conditionMap[timeCondition] || timeCondition;
};
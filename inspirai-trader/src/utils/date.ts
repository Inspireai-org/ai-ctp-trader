/**
 * 日期时间工具函数
 * 
 * 提供日期时间处理和格式化功能
 */

/**
 * 日期格式枚举
 */
export enum DateFormat {
  /** YYYY-MM-DD */
  DATE = 'YYYY-MM-DD',
  /** HH:mm:ss */
  TIME = 'HH:mm:ss',
  /** YYYY-MM-DD HH:mm:ss */
  DATETIME = 'YYYY-MM-DD HH:mm:ss',
  /** MM-DD HH:mm */
  SHORT_DATETIME = 'MM-DD HH:mm',
  /** HH:mm */
  SHORT_TIME = 'HH:mm',
  /** YYYYMMDD */
  COMPACT_DATE = 'YYYYMMDD',
  /** HHmmss */
  COMPACT_TIME = 'HHmmss',
  /** YYYYMMDDHHmmss */
  COMPACT_DATETIME = 'YYYYMMDDHHmmss',
}

/**
 * 时间段枚举
 */
export enum TimePeriod {
  /** 今天 */
  TODAY = 'today',
  /** 昨天 */
  YESTERDAY = 'yesterday',
  /** 本周 */
  THIS_WEEK = 'thisWeek',
  /** 上周 */
  LAST_WEEK = 'lastWeek',
  /** 本月 */
  THIS_MONTH = 'thisMonth',
  /** 上月 */
  LAST_MONTH = 'lastMonth',
  /** 本年 */
  THIS_YEAR = 'thisYear',
  /** 去年 */
  LAST_YEAR = 'lastYear',
}

/**
 * 日期时间工具类
 */
export class DateUtils {
  /**
   * 格式化日期时间
   * @param date 日期对象、时间戳或字符串
   * @param format 格式字符串
   * @returns 格式化后的字符串
   */
  static format(date: Date | number | string, format: string = DateFormat.DATETIME): string {
    try {
      const dateObj = this.toDate(date);
      if (!dateObj) return '--';

      const year = dateObj.getFullYear();
      const month = dateObj.getMonth() + 1;
      const day = dateObj.getDate();
      const hours = dateObj.getHours();
      const minutes = dateObj.getMinutes();
      const seconds = dateObj.getSeconds();

      return format
        .replace('YYYY', year.toString())
        .replace('MM', month.toString().padStart(2, '0'))
        .replace('DD', day.toString().padStart(2, '0'))
        .replace('HH', hours.toString().padStart(2, '0'))
        .replace('mm', minutes.toString().padStart(2, '0'))
        .replace('ss', seconds.toString().padStart(2, '0'));
    } catch {
      return '--';
    }
  }

  /**
   * 将各种类型转换为 Date 对象
   * @param input 输入值
   * @returns Date 对象或 null
   */
  static toDate(input: Date | number | string): Date | null {
    if (input instanceof Date) {
      return isNaN(input.getTime()) ? null : input;
    }

    if (typeof input === 'number') {
      // 处理时间戳（毫秒或秒）
      const timestamp = input < 10000000000 ? input * 1000 : input;
      const date = new Date(timestamp);
      return isNaN(date.getTime()) ? null : date;
    }

    if (typeof input === 'string') {
      // 处理字符串格式
      if (input.trim() === '') return null;
      
      // 尝试解析各种格式
      const date = new Date(input);
      if (!isNaN(date.getTime())) return date;

      // 尝试解析 YYYYMMDD 格式
      if (/^\d{8}$/.test(input)) {
        const year = parseInt(input.substring(0, 4));
        const month = parseInt(input.substring(4, 6)) - 1;
        const day = parseInt(input.substring(6, 8));
        const parsedDate = new Date(year, month, day);
        return isNaN(parsedDate.getTime()) ? null : parsedDate;
      }

      // 尝试解析 HHmmss 格式
      if (/^\d{6}$/.test(input)) {
        const today = new Date();
        const hours = parseInt(input.substring(0, 2));
        const minutes = parseInt(input.substring(2, 4));
        const seconds = parseInt(input.substring(4, 6));
        const parsedDate = new Date(today.getFullYear(), today.getMonth(), today.getDate(), hours, minutes, seconds);
        return isNaN(parsedDate.getTime()) ? null : parsedDate;
      }
    }

    return null;
  }

  /**
   * 获取当前时间戳（毫秒）
   * @returns 时间戳
   */
  static now(): number {
    return Date.now();
  }

  /**
   * 获取今天的开始时间
   * @param date 基准日期，默认为当前日期
   * @returns 今天开始时间的 Date 对象
   */
  static startOfDay(date: Date = new Date()): Date {
    const result = new Date(date);
    result.setHours(0, 0, 0, 0);
    return result;
  }

  /**
   * 获取今天的结束时间
   * @param date 基准日期，默认为当前日期
   * @returns 今天结束时间的 Date 对象
   */
  static endOfDay(date: Date = new Date()): Date {
    const result = new Date(date);
    result.setHours(23, 59, 59, 999);
    return result;
  }

  /**
   * 获取本周的开始时间（周一）
   * @param date 基准日期，默认为当前日期
   * @returns 本周开始时间的 Date 对象
   */
  static startOfWeek(date: Date = new Date()): Date {
    const result = new Date(date);
    const day = result.getDay();
    const diff = result.getDate() - day + (day === 0 ? -6 : 1); // 调整为周一开始
    result.setDate(diff);
    return this.startOfDay(result);
  }

  /**
   * 获取本月的开始时间
   * @param date 基准日期，默认为当前日期
   * @returns 本月开始时间的 Date 对象
   */
  static startOfMonth(date: Date = new Date()): Date {
    const result = new Date(date);
    result.setDate(1);
    return this.startOfDay(result);
  }

  /**
   * 获取本年的开始时间
   * @param date 基准日期，默认为当前日期
   * @returns 本年开始时间的 Date 对象
   */
  static startOfYear(date: Date = new Date()): Date {
    const result = new Date(date);
    result.setMonth(0, 1);
    return this.startOfDay(result);
  }

  /**
   * 添加天数
   * @param date 基准日期
   * @param days 要添加的天数
   * @returns 新的 Date 对象
   */
  static addDays(date: Date, days: number): Date {
    const result = new Date(date);
    result.setDate(result.getDate() + days);
    return result;
  }

  /**
   * 添加小时
   * @param date 基准日期
   * @param hours 要添加的小时数
   * @returns 新的 Date 对象
   */
  static addHours(date: Date, hours: number): Date {
    const result = new Date(date);
    result.setHours(result.getHours() + hours);
    return result;
  }

  /**
   * 添加分钟
   * @param date 基准日期
   * @param minutes 要添加的分钟数
   * @returns 新的 Date 对象
   */
  static addMinutes(date: Date, minutes: number): Date {
    const result = new Date(date);
    result.setMinutes(result.getMinutes() + minutes);
    return result;
  }

  /**
   * 计算两个日期之间的天数差
   * @param date1 日期1
   * @param date2 日期2
   * @returns 天数差（date2 - date1）
   */
  static diffInDays(date1: Date, date2: Date): number {
    const timeDiff = date2.getTime() - date1.getTime();
    return Math.ceil(timeDiff / (1000 * 3600 * 24));
  }

  /**
   * 计算两个日期之间的小时差
   * @param date1 日期1
   * @param date2 日期2
   * @returns 小时差（date2 - date1）
   */
  static diffInHours(date1: Date, date2: Date): number {
    const timeDiff = date2.getTime() - date1.getTime();
    return Math.ceil(timeDiff / (1000 * 3600));
  }

  /**
   * 计算两个日期之间的分钟差
   * @param date1 日期1
   * @param date2 日期2
   * @returns 分钟差（date2 - date1）
   */
  static diffInMinutes(date1: Date, date2: Date): number {
    const timeDiff = date2.getTime() - date1.getTime();
    return Math.ceil(timeDiff / (1000 * 60));
  }

  /**
   * 检查是否为同一天
   * @param date1 日期1
   * @param date2 日期2
   * @returns 是否为同一天
   */
  static isSameDay(date1: Date, date2: Date): boolean {
    return date1.getFullYear() === date2.getFullYear() &&
           date1.getMonth() === date2.getMonth() &&
           date1.getDate() === date2.getDate();
  }

  /**
   * 检查是否为今天
   * @param date 日期
   * @returns 是否为今天
   */
  static isToday(date: Date): boolean {
    return this.isSameDay(date, new Date());
  }

  /**
   * 检查是否为昨天
   * @param date 日期
   * @returns 是否为昨天
   */
  static isYesterday(date: Date): boolean {
    const yesterday = this.addDays(new Date(), -1);
    return this.isSameDay(date, yesterday);
  }

  /**
   * 检查是否为工作日
   * @param date 日期
   * @returns 是否为工作日
   */
  static isWeekday(date: Date): boolean {
    const day = date.getDay();
    return day >= 1 && day <= 5;
  }

  /**
   * 检查是否为周末
   * @param date 日期
   * @returns 是否为周末
   */
  static isWeekend(date: Date): boolean {
    const day = date.getDay();
    return day === 0 || day === 6;
  }

  /**
   * 获取时间段的日期范围
   * @param period 时间段
   * @returns 日期范围 [开始日期, 结束日期]
   */
  static getDateRange(period: TimePeriod): [Date, Date] {
    const now = new Date();
    
    switch (period) {
      case TimePeriod.TODAY:
        return [this.startOfDay(now), this.endOfDay(now)];
      
      case TimePeriod.YESTERDAY:
        const yesterday = this.addDays(now, -1);
        return [this.startOfDay(yesterday), this.endOfDay(yesterday)];
      
      case TimePeriod.THIS_WEEK:
        const startOfWeek = this.startOfWeek(now);
        const endOfWeek = this.endOfDay(this.addDays(startOfWeek, 6));
        return [startOfWeek, endOfWeek];
      
      case TimePeriod.LAST_WEEK:
        const lastWeekStart = this.addDays(this.startOfWeek(now), -7);
        const lastWeekEnd = this.endOfDay(this.addDays(lastWeekStart, 6));
        return [lastWeekStart, lastWeekEnd];
      
      case TimePeriod.THIS_MONTH:
        const startOfMonth = this.startOfMonth(now);
        const endOfMonth = this.endOfDay(new Date(now.getFullYear(), now.getMonth() + 1, 0));
        return [startOfMonth, endOfMonth];
      
      case TimePeriod.LAST_MONTH:
        const lastMonthStart = this.startOfMonth(this.addDays(this.startOfMonth(now), -1));
        const lastMonthEnd = this.endOfDay(this.addDays(this.startOfMonth(now), -1));
        return [lastMonthStart, lastMonthEnd];
      
      case TimePeriod.THIS_YEAR:
        const startOfYear = this.startOfYear(now);
        const endOfYear = this.endOfDay(new Date(now.getFullYear(), 11, 31));
        return [startOfYear, endOfYear];
      
      case TimePeriod.LAST_YEAR:
        const lastYearStart = this.startOfYear(this.addDays(this.startOfYear(now), -1));
        const lastYearEnd = this.endOfDay(this.addDays(this.startOfYear(now), -1));
        return [lastYearStart, lastYearEnd];
      
      default:
        return [this.startOfDay(now), this.endOfDay(now)];
    }
  }

  /**
   * 格式化相对时间（如：刚刚、5分钟前、2小时前）
   * @param date 日期
   * @param now 当前时间，默认为系统当前时间
   * @returns 相对时间字符串
   */
  static formatRelativeTime(date: Date, now: Date = new Date()): string {
    const diffMs = now.getTime() - date.getTime();
    const diffSeconds = Math.floor(diffMs / 1000);
    const diffMinutes = Math.floor(diffSeconds / 60);
    const diffHours = Math.floor(diffMinutes / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffSeconds < 60) {
      return '刚刚';
    } else if (diffMinutes < 60) {
      return `${diffMinutes}分钟前`;
    } else if (diffHours < 24) {
      return `${diffHours}小时前`;
    } else if (diffDays < 7) {
      return `${diffDays}天前`;
    } else {
      return this.format(date, DateFormat.DATE);
    }
  }

  /**
   * 解析交易时间字符串（如：09:00-11:30,13:30-15:00）
   * @param tradingHours 交易时间字符串
   * @returns 交易时间段数组
   */
  static parseTradingHours(tradingHours: string): Array<{ start: string; end: string }> {
    if (!tradingHours) return [];

    try {
      return tradingHours.split(',').map(period => {
        const [start, end] = period.trim().split('-');
        return { start: (start ?? '').trim(), end: (end ?? '').trim() };
      });
    } catch {
      return [];
    }
  }

  /**
   * 检查当前时间是否在交易时间内
   * @param tradingHours 交易时间字符串
   * @param now 当前时间，默认为系统当前时间
   * @returns 是否在交易时间内
   */
  static isInTradingHours(tradingHours: string, now: Date = new Date()): boolean {
    const periods = this.parseTradingHours(tradingHours);
    if (periods.length === 0) return false;

    const currentTime = this.format(now, DateFormat.SHORT_TIME);

    return periods.some(period => {
      return currentTime >= period.start && currentTime <= period.end;
    });
  }

  /**
   * 获取下一个交易时间段
   * @param tradingHours 交易时间字符串
   * @param now 当前时间，默认为系统当前时间
   * @returns 下一个交易时间段或 null
   */
  static getNextTradingPeriod(
    tradingHours: string, 
    now: Date = new Date()
  ): { start: string; end: string } | null {
    const periods = this.parseTradingHours(tradingHours);
    if (periods.length === 0) return null;

    const currentTime = this.format(now, DateFormat.SHORT_TIME);

    // 查找当天的下一个交易时间段
    for (const period of periods) {
      if (currentTime < period.start) {
        return period;
      }
    }

    // 如果当天没有更多交易时间段，返回第一个时间段（表示明天）
    return periods[0] ?? null;
  }
}

/**
 * 交易日历工具类
 */
export class TradingCalendar {
  // 中国法定节假日（需要根据实际情况更新）
  private static holidays: string[] = [
    // 2024年节假日（示例）
    '2024-01-01', // 元旦
    '2024-02-10', '2024-02-11', '2024-02-12', '2024-02-13', '2024-02-14', '2024-02-15', '2024-02-16', '2024-02-17', // 春节
    '2024-04-04', '2024-04-05', '2024-04-06', // 清明节
    '2024-05-01', '2024-05-02', '2024-05-03', // 劳动节
    '2024-06-10', // 端午节
    '2024-09-15', '2024-09-16', '2024-09-17', // 中秋节
    '2024-10-01', '2024-10-02', '2024-10-03', '2024-10-04', '2024-10-05', '2024-10-06', '2024-10-07', // 国庆节
  ];

  /**
   * 检查是否为交易日
   * @param date 日期
   * @returns 是否为交易日
   */
  static isTradingDay(date: Date): boolean {
    // 检查是否为周末
    if (DateUtils.isWeekend(date)) {
      return false;
    }

    // 检查是否为节假日
    const dateStr = DateUtils.format(date, DateFormat.DATE);
    return !this.holidays.includes(dateStr);
  }

  /**
   * 获取下一个交易日
   * @param date 基准日期
   * @returns 下一个交易日
   */
  static getNextTradingDay(date: Date): Date {
    let nextDay = DateUtils.addDays(date, 1);
    
    while (!this.isTradingDay(nextDay)) {
      nextDay = DateUtils.addDays(nextDay, 1);
    }
    
    return nextDay;
  }

  /**
   * 获取上一个交易日
   * @param date 基准日期
   * @returns 上一个交易日
   */
  static getPreviousTradingDay(date: Date): Date {
    let prevDay = DateUtils.addDays(date, -1);
    
    while (!this.isTradingDay(prevDay)) {
      prevDay = DateUtils.addDays(prevDay, -1);
    }
    
    return prevDay;
  }

  /**
   * 计算两个日期之间的交易日数量
   * @param startDate 开始日期
   * @param endDate 结束日期
   * @returns 交易日数量
   */
  static getTradingDaysBetween(startDate: Date, endDate: Date): number {
    let count = 0;
    let currentDate = new Date(startDate);
    
    while (currentDate <= endDate) {
      if (this.isTradingDay(currentDate)) {
        count++;
      }
      currentDate = DateUtils.addDays(currentDate, 1);
    }
    
    return count;
  }

  /**
   * 添加节假日
   * @param holidays 节假日数组（YYYY-MM-DD 格式）
   */
  static addHolidays(holidays: string[]): void {
    this.holidays.push(...holidays);
    // 去重并排序
    this.holidays = [...new Set(this.holidays)].sort();
  }

  /**
   * 移除节假日
   * @param holidays 要移除的节假日数组
   */
  static removeHolidays(holidays: string[]): void {
    this.holidays = this.holidays.filter(holiday => !holidays.includes(holiday));
  }

  /**
   * 获取所有节假日
   * @returns 节假日数组
   */
  static getHolidays(): string[] {
    return [...this.holidays];
  }
}
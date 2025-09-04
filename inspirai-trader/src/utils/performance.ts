/**
 * 性能优化工具函数
 * 
 * 提供防抖、节流、缓存等性能优化功能
 */

/**
 * 防抖函数
 * @param func 要防抖的函数
 * @param delay 延迟时间（毫秒）
 * @param immediate 是否立即执行
 * @returns 防抖后的函数
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  delay: number,
  immediate: boolean = false
): (...args: Parameters<T>) => void {
  let timeoutId: NodeJS.Timeout | null = null;
  // let lastCallTime = 0;

  return function (this: any, ...args: Parameters<T>) {
    const callNow = immediate && !timeoutId;
    // const now = Date.now();

    if (timeoutId) {
      clearTimeout(timeoutId);
    }

    timeoutId = setTimeout(() => {
      timeoutId = null;
      // lastCallTime = now;
      if (!immediate) {
        func.apply(this, args);
      }
    }, delay);

    if (callNow) {
      func.apply(this, args);
    }
  };
}

/**
 * 节流函数
 * @param func 要节流的函数
 * @param delay 节流间隔（毫秒）
 * @param options 选项
 * @returns 节流后的函数
 */
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  delay: number,
  options: {
    leading?: boolean;
    trailing?: boolean;
  } = {}
): (...args: Parameters<T>) => void {
  const { leading = true, trailing = true } = options;
  let timeoutId: NodeJS.Timeout | null = null;
  let lastCallTime = 0;
  let lastArgs: Parameters<T> | null = null;

  return function (this: any, ...args: Parameters<T>) {
    const now = Date.now();
    const timeSinceLastCall = now - lastCallTime;

    lastArgs = args;

    if (lastCallTime === 0 && !leading) {
      lastCallTime = now;
      return;
    }

    if (timeSinceLastCall >= delay) {
      if (timeoutId) {
        clearTimeout(timeoutId);
        timeoutId = null;
      }
      lastCallTime = now;
      func.apply(this, args);
    } else if (!timeoutId && trailing) {
      timeoutId = setTimeout(() => {
        lastCallTime = leading ? Date.now() : 0;
        timeoutId = null;
        if (lastArgs) {
          func.apply(this, lastArgs);
        }
      }, delay - timeSinceLastCall);
    }
  };
}

/**
 * 请求动画帧节流
 * @param func 要节流的函数
 * @returns 节流后的函数
 */
export function rafThrottle<T extends (...args: any[]) => any>(
  func: T
): (...args: Parameters<T>) => void {
  let rafId: number | null = null;
  let lastArgs: Parameters<T> | null = null;

  return function (this: any, ...args: Parameters<T>) {
    lastArgs = args;

    if (rafId === null) {
      rafId = requestAnimationFrame(() => {
        rafId = null;
        if (lastArgs) {
          func.apply(this, lastArgs);
        }
      });
    }
  };
}

/**
 * LRU 缓存类
 */
export class LRUCache<K, V> {
  private capacity: number;
  private cache: Map<K, V>;

  constructor(capacity: number) {
    this.capacity = capacity;
    this.cache = new Map();
  }

  get(key: K): V | undefined {
    if (this.cache.has(key)) {
      // 移动到最后（最近使用）
      const value = this.cache.get(key)!;
      this.cache.delete(key);
      this.cache.set(key, value);
      return value;
    }
    return undefined;
  }

  set(key: K, value: V): void {
    if (this.cache.has(key)) {
      // 更新现有键
      this.cache.delete(key);
    } else if (this.cache.size >= this.capacity) {
      // 删除最久未使用的项（第一个）
      const firstKey = this.cache.keys().next().value;
      if (firstKey !== undefined) {
        this.cache.delete(firstKey);
      }
    }
    this.cache.set(key, value);
  }

  has(key: K): boolean {
    return this.cache.has(key);
  }

  delete(key: K): boolean {
    return this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }

  size(): number {
    return this.cache.size;
  }

  keys(): IterableIterator<K> {
    return this.cache.keys();
  }

  values(): IterableIterator<V> {
    return this.cache.values();
  }
}

/**
 * 内存化装饰器
 * @param maxSize 最大缓存大小
 * @returns 装饰器函数
 */
export function memoize<T extends (...args: any[]) => any>(
  maxSize: number = 100
) {
  return function (_target: any, _propertyKey: string, descriptor: PropertyDescriptor) {
    const originalMethod = descriptor.value;
    const cache = new LRUCache<string, any>(maxSize);

    descriptor.value = function (...args: Parameters<T>) {
      const key = JSON.stringify(args);
      
      if (cache.has(key)) {
        return cache.get(key);
      }

      const result = originalMethod.apply(this, args);
      cache.set(key, result);
      return result;
    };

    return descriptor;
  };
}

/**
 * 创建内存化函数
 * @param func 要内存化的函数
 * @param maxSize 最大缓存大小
 * @param keyGenerator 键生成器函数
 * @returns 内存化后的函数
 */
export function createMemoizedFunction<T extends (...args: any[]) => any>(
  func: T,
  maxSize: number = 100,
  keyGenerator?: (...args: Parameters<T>) => string
): T {
  const cache = new LRUCache<string, ReturnType<T>>(maxSize);
  const defaultKeyGenerator = (...args: Parameters<T>) => JSON.stringify(args);
  const getKey = keyGenerator || defaultKeyGenerator;

  return ((...args: Parameters<T>) => {
    const key = getKey(...args);
    
    if (cache.has(key)) {
      return cache.get(key);
    }

    const result = func(...args);
    cache.set(key, result);
    return result;
  }) as T;
}

/**
 * 批处理函数
 * @param func 要批处理的函数
 * @param delay 批处理延迟（毫秒）
 * @param maxBatchSize 最大批处理大小
 * @returns 批处理后的函数
 */
export function batchProcess<T, R>(
  func: (items: T[]) => Promise<R[]>,
  delay: number = 10,
  maxBatchSize: number = 100
): (item: T) => Promise<R> {
  let batch: T[] = [];
  let resolvers: Array<(value: R) => void> = [];
  let rejecters: Array<(reason: any) => void> = [];
  let timeoutId: NodeJS.Timeout | null = null;

  const processBatch = async () => {
    if (batch.length === 0) return;

    const currentBatch = batch.splice(0);
    const currentResolvers = resolvers.splice(0);
    const currentRejecters = rejecters.splice(0);

    try {
      const results = await func(currentBatch);
      results.forEach((result, index) => {
        currentResolvers[index]?.(result);
      });
    } catch (error) {
      currentRejecters.forEach(reject => reject(error));
    }
  };

  const scheduleBatch = () => {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    timeoutId = setTimeout(processBatch, delay);
  };

  return (item: T): Promise<R> => {
    return new Promise<R>((resolve, reject) => {
      batch.push(item);
      resolvers.push(resolve);
      rejecters.push(reject);

      if (batch.length >= maxBatchSize) {
        processBatch();
      } else {
        scheduleBatch();
      }
    });
  };
}

/**
 * 性能监控器
 */
export class PerformanceMonitor {
  private static instance: PerformanceMonitor;
  private metrics: Map<string, number[]> = new Map();

  static getInstance(): PerformanceMonitor {
    if (!PerformanceMonitor.instance) {
      PerformanceMonitor.instance = new PerformanceMonitor();
    }
    return PerformanceMonitor.instance;
  }

  /**
   * 开始性能测量
   * @param name 测量名称
   */
  start(name: string): void {
    performance.mark(`${name}-start`);
  }

  /**
   * 结束性能测量
   * @param name 测量名称
   */
  end(name: string): number {
    performance.mark(`${name}-end`);
    performance.measure(name, `${name}-start`, `${name}-end`);
    
    const measure = performance.getEntriesByName(name, 'measure')[0];
    const duration = measure?.duration ?? 0;

    if (!this.metrics.has(name)) {
      this.metrics.set(name, []);
    }
    this.metrics.get(name)!.push(duration);

    // 清理性能标记
    performance.clearMarks(`${name}-start`);
    performance.clearMarks(`${name}-end`);
    performance.clearMeasures(name);

    return duration;
  }

  /**
   * 获取性能统计
   * @param name 测量名称
   */
  getStats(name: string): {
    count: number;
    average: number;
    min: number;
    max: number;
    total: number;
  } | null {
    const durations = this.metrics.get(name);
    if (!durations || durations.length === 0) {
      return null;
    }

    const total = durations.reduce((sum, duration) => sum + duration, 0);
    const average = total / durations.length;
    const min = Math.min(...durations);
    const max = Math.max(...durations);

    return {
      count: durations.length,
      average,
      min,
      max,
      total,
    };
  }

  /**
   * 清除指定测量的统计数据
   * @param name 测量名称
   */
  clear(name: string): void {
    this.metrics.delete(name);
  }

  /**
   * 清除所有统计数据
   */
  clearAll(): void {
    this.metrics.clear();
  }

  /**
   * 获取所有测量名称
   */
  getAllNames(): string[] {
    return Array.from(this.metrics.keys());
  }
}

/**
 * 性能测量装饰器
 * @param name 测量名称
 */
export function measurePerformance(name?: string) {
  return function (target: any, propertyKey: string, descriptor: PropertyDescriptor) {
    const originalMethod = descriptor.value;
    const measureName = name || `${target.constructor.name}.${propertyKey}`;
    const monitor = PerformanceMonitor.getInstance();

    descriptor.value = function (...args: any[]) {
      monitor.start(measureName);
      
      try {
        const result = originalMethod.apply(this, args);
        
        if (result instanceof Promise) {
          return result.finally(() => {
            monitor.end(measureName);
          });
        } else {
          monitor.end(measureName);
          return result;
        }
      } catch (error) {
        monitor.end(measureName);
        throw error;
      }
    };

    return descriptor;
  };
}

/**
 * 创建性能测量函数
 * @param func 要测量的函数
 * @param name 测量名称
 * @returns 包装后的函数
 */
export function createMeasuredFunction<T extends (...args: any[]) => any>(
  func: T,
  name: string
): T {
  const monitor = PerformanceMonitor.getInstance();

  return ((...args: Parameters<T>) => {
    monitor.start(name);
    
    try {
      const result = func(...args);
      
      if (result instanceof Promise) {
        return result.finally(() => {
          monitor.end(name);
        });
      } else {
        monitor.end(name);
        return result;
      }
    } catch (error) {
      monitor.end(name);
      throw error;
    }
  }) as T;
}

/**
 * 获取性能监控器实例
 */
export const performanceMonitor = PerformanceMonitor.getInstance();
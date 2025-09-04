import { AppEvent, AppEventType } from '../types';

/**
 * 事件监听器类型
 */
type EventListener<T = any> = (event: AppEvent<T>) => void;

/**
 * 事件总线类
 */
class EventBus {
  private listeners: Map<AppEventType, Set<EventListener>> = new Map();
  private eventHistory: AppEvent[] = [];
  private maxHistorySize = 1000;

  /**
   * 订阅事件
   */
  on<T = any>(eventType: AppEventType, listener: EventListener<T>): () => void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, new Set());
    }
    
    const listenersSet = this.listeners.get(eventType)!;
    listenersSet.add(listener);
    
    // 返回取消订阅函数
    return () => {
      listenersSet.delete(listener);
      if (listenersSet.size === 0) {
        this.listeners.delete(eventType);
      }
    };
  }

  /**
   * 订阅一次性事件
   */
  once<T = any>(eventType: AppEventType, listener: EventListener<T>): () => void {
    const onceListener: EventListener<T> = (event) => {
      listener(event);
      unsubscribe();
    };
    
    const unsubscribe = this.on(eventType, onceListener);
    return unsubscribe;
  }

  /**
   * 发布事件
   */
  emit<T = any>(eventType: AppEventType, data: T): void {
    const event: AppEvent<T> = {
      type: eventType,
      data,
      timestamp: Date.now(),
      eventId: this.generateEventId(),
    };

    // 添加到历史记录
    this.addToHistory(event);

    // 通知所有监听器
    const listenersSet = this.listeners.get(eventType);
    if (listenersSet) {
      listenersSet.forEach(listener => {
        try {
          listener(event);
        } catch (error) {
          console.error(`事件监听器执行错误 [${eventType}]:`, error);
        }
      });
    }
  }

  /**
   * 取消所有订阅
   */
  off(eventType?: AppEventType): void {
    if (eventType) {
      this.listeners.delete(eventType);
    } else {
      this.listeners.clear();
    }
  }

  /**
   * 获取事件历史
   */
  getHistory(eventType?: AppEventType, limit?: number): AppEvent[] {
    let history = eventType 
      ? this.eventHistory.filter(event => event.type === eventType)
      : this.eventHistory;
    
    if (limit && limit > 0) {
      history = history.slice(-limit);
    }
    
    return history;
  }

  /**
   * 清空事件历史
   */
  clearHistory(): void {
    this.eventHistory = [];
  }

  /**
   * 获取监听器数量
   */
  getListenerCount(eventType?: AppEventType): number {
    if (eventType) {
      return this.listeners.get(eventType)?.size || 0;
    } else {
      return Array.from(this.listeners.values()).reduce((total, set) => total + set.size, 0);
    }
  }

  /**
   * 生成事件ID
   */
  private generateEventId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * 添加到历史记录
   */
  private addToHistory(event: AppEvent): void {
    this.eventHistory.push(event);
    
    // 限制历史记录大小
    if (this.eventHistory.length > this.maxHistorySize) {
      this.eventHistory = this.eventHistory.slice(-this.maxHistorySize);
    }
  }
}

/**
 * 全局事件总线实例
 */
export const eventBus = new EventBus();

/**
 * React Hook 用于订阅事件
 */
export const useEventListener = <T = any>(
  eventType: AppEventType,
  listener: EventListener<T>,
  deps: React.DependencyList = []
) => {
  React.useEffect(() => {
    const unsubscribe = eventBus.on(eventType, listener);
    return unsubscribe;
  }, deps);
};

/**
 * React Hook 用于发布事件
 */
export const useEventEmitter = () => {
  return React.useCallback(<T = any>(eventType: AppEventType, data: T) => {
    eventBus.emit(eventType, data);
  }, []);
};

/**
 * React Hook 用于获取事件历史
 */
export const useEventHistory = (eventType?: AppEventType, limit?: number) => {
  const [history, setHistory] = React.useState<AppEvent[]>([]);
  
  React.useEffect(() => {
    const updateHistory = () => {
      setHistory(eventBus.getHistory(eventType, limit));
    };
    
    // 初始化历史记录
    updateHistory();
    
    // 监听新事件来更新历史记录
    const unsubscribe = eventType 
      ? eventBus.on(eventType, updateHistory)
      : null;
    
    return unsubscribe || undefined;
  }, [eventType, limit]);
  
  return history;
};

// 导入 React（如果在 React 环境中使用）
import * as React from 'react';
import { useUIStore, showNotification, playSound } from '@/stores/uiStore';

interface NotificationOptions {
  title: string;
  body: string;
  type?: 'success' | 'error' | 'warning' | 'info';
  sound?: boolean;
}

export class NotificationService {
  private static instance: NotificationService;

  static getInstance(): NotificationService {
    if (!NotificationService.instance) {
      NotificationService.instance = new NotificationService();
    }
    return NotificationService.instance;
  }

  // 订单成交通知
  notifyOrderFilled(orderId: string, instrumentId: string, price: number, volume: number) {
    this.notify({
      title: '订单成交',
      body: `${instrumentId} 成交 ${volume} 手 @ ${price}`,
      type: 'success',
      sound: true
    });
  }

  // 订单撤销通知
  notifyOrderCancelled(orderId: string, instrumentId: string) {
    this.notify({
      title: '订单已撤销',
      body: `${instrumentId} 订单 ${orderId} 已撤销`,
      type: 'info'
    });
  }

  // 价格预警通知
  notifyPriceAlert(instrumentId: string, currentPrice: number, alertPrice: number) {
    this.notify({
      title: '价格预警',
      body: `${instrumentId} 当前价格 ${currentPrice} 触发预警 (预警价: ${alertPrice})`,
      type: 'warning',
      sound: true
    });
  }

  // 风险警告通知
  notifyRiskWarning(riskLevel: number, message: string) {
    this.notify({
      title: '风险警告',
      body: `风险度 ${riskLevel.toFixed(1)}%: ${message}`,
      type: 'error',
      sound: true
    });
  }

  // 连接状态通知
  notifyConnectionStatus(connected: boolean) {
    this.notify({
      title: connected ? '连接成功' : '连接断开',
      body: connected ? '已连接到交易服务器' : '与交易服务器的连接已断开',
      type: connected ? 'success' : 'error'
    });
  }

  // 通用通知方法
  private notify(options: NotificationOptions) {
    const store = useUIStore.getState();
    
    // 检查是否启用通知
    if (!store.notificationEnabled) return;
    
    // 播放声音
    if (options.sound && store.soundEnabled) {
      playSound(options.type || 'info');
    }
    
    // 显示系统通知
    showNotification(options.title, options.body, {
      icon: this.getIcon(options.type),
      badge: '/icon.png'
    });
  }

  private getIcon(type?: string): string {
    switch (type) {
      case 'success': return '✅';
      case 'error': return '❌';
      case 'warning': return '⚠️';
      default: return 'ℹ️';
    }
  }
}

export const notificationService = NotificationService.getInstance();
import { getCtpService, marketDataManager } from './ctpService';
import { useMarketStore } from '@/stores/marketStore';
import { useTradingStore } from '@/stores/tradingStore';
import { useConnectionStore } from '@/stores/connectionStore';
import { notificationService } from './notificationService';
import { listen } from '@tauri-apps/api/event';
import { CtpEvent } from '@/types/ctp';

export class RealtimeService {
  private static instance: RealtimeService;
  private updateInterval: NodeJS.Timeout | null = null;
  private eventListeners: Array<() => void> = [];

  static getInstance(): RealtimeService {
    if (!RealtimeService.instance) {
      RealtimeService.instance = new RealtimeService();
    }
    return RealtimeService.instance;
  }

  // 启动实时数据更新
  async start() {
    // 监听Tauri事件
    await this.setupEventListeners();
    
    // 启动定期更新
    this.startPeriodicUpdate();
  }

  // 停止实时数据更新
  stop() {
    // 清理定时器
    if (this.updateInterval) {
      clearInterval(this.updateInterval);
      this.updateInterval = null;
    }
    
    // 清理事件监听
    this.eventListeners.forEach(unlisten => unlisten());
    this.eventListeners = [];
  }

  // 设置事件监听
  private async setupEventListeners() {
    // 监听市场数据更新
    const unlistenMarketData = await listen<CtpEvent>('ctp://market-data', (event) => {
      if (event.payload.type === 'MarketData') {
        const marketStore = useMarketStore.getState();
        marketStore.updateMarketData(event.payload.data);
      }
    });
    this.eventListeners.push(unlistenMarketData as any);

    // 监听订单更新
    const unlistenOrder = await listen<CtpEvent>('ctp://order-update', (event) => {
      if (event.payload.type === 'OrderUpdate') {
        const tradingStore = useTradingStore.getState();
        tradingStore.updateOrderStatus(event.payload.order);
        
        // 订单成交通知
        if (event.payload.order.status === 'Filled') {
          notificationService.notifyOrderFilled(
            event.payload.order.order_ref,
            event.payload.order.instrument_id,
            event.payload.order.price,
            event.payload.order.volume
          );
        }
      }
    });
    this.eventListeners.push(unlistenOrder as any);

    // 监听成交更新
    const unlistenTrade = await listen<CtpEvent>('ctp://trade-update', (event) => {
      if (event.payload.type === 'TradeUpdate') {
        const tradingStore = useTradingStore.getState();
        tradingStore.addTrade(event.payload.trade);
      }
    });
    this.eventListeners.push(unlistenTrade as any);

    // 监听连接状态
    const unlistenConnection = await listen<CtpEvent>('ctp://connection', (event) => {
      const connectionStore = useConnectionStore.getState();
      
      switch (event.payload.type) {
        case 'Connected':
          connectionStore.updateConnectionState('Connected', '已连接');
          notificationService.notifyConnectionStatus(true);
          break;
        case 'Disconnected':
          connectionStore.updateConnectionState('Disconnected', event.payload.reason);
          notificationService.notifyConnectionStatus(false);
          break;
        case 'LoginSuccess':
          connectionStore.updateConnectionState('LoggedIn', '登录成功');
          break;
        case 'LoginFailed':
          connectionStore.updateConnectionState('Connected', event.payload.error);
          break;
      }
    });
    this.eventListeners.push(unlistenConnection as any);

    // 监听错误事件
    const unlistenError = await listen<CtpEvent>('ctp://error', (event) => {
      if (event.payload.type === 'Error') {
        console.error('CTP Error:', event.payload);
        // TODO: 使用错误处理服务
      }
    });
    this.eventListeners.push(unlistenError as any);
  }

  // 定期更新数据
  private startPeriodicUpdate() {
    // 每5秒更新一次账户和持仓
    this.updateInterval = setInterval(async () => {
      const connectionStore = useConnectionStore.getState();
      
      if (connectionStore.isLoggedIn) {
        try {
          const tradingStore = useTradingStore.getState();
          
          // 更新订单和成交
          await tradingStore.refreshOrders();
          await tradingStore.refreshTrades();
          
          // 更新市场数据
          const marketStore = useMarketStore.getState();
          const service = getCtpService();
          
          for (const instrumentId of marketStore.subscribedInstruments) {
            try {
              const data = await service.getMarketData(instrumentId);
              marketStore.updateMarketData(data);
            } catch (error) {
              console.error(`Failed to update ${instrumentId}:`, error);
            }
          }
        } catch (error) {
          console.error('Periodic update failed:', error);
        }
      }
    }, 5000);
  }

  // 手动刷新所有数据
  async refreshAll() {
    const connectionStore = useConnectionStore.getState();
    
    if (!connectionStore.isLoggedIn) {
      console.warn('Not logged in, skipping refresh');
      return;
    }
    
    try {
      const service = getCtpService();
      const tradingStore = useTradingStore.getState();
      
      // 并行更新所有数据
      await Promise.all([
        tradingStore.refreshOrders(),
        tradingStore.refreshTrades(),
        service.queryAccount(),
        service.queryPositions()
      ]);
      
      console.log('All data refreshed');
    } catch (error) {
      console.error('Refresh all failed:', error);
    }
  }
}

export const realtimeService = RealtimeService.getInstance();
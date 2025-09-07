import { invoke } from '@tauri-apps/api/core';
import { 
  MarketData, 
  OrderInput, 
  OrderRef,
  OrderStatus,
  Trade,
  Position,
  AccountInfo,
  InstrumentInfo,
  CommissionRate,
  MarginRate,
  RiskParams,
  LoginCredentials,
  CtpConfig,
  MarketDataSubscription
} from '@/types/ctp';

/**
 * CTP Trading Service
 * Provides comprehensive interface to CTP trading system
 */
export class CtpService {
  // Connection Management
  async init(): Promise<string> {
    return invoke('ctp_init');
  }

  async createConfig(): Promise<CtpConfig> {
    return invoke('ctp_create_config');
  }

  async connect(config: CtpConfig): Promise<string> {
    return invoke('ctp_connect', { config });
  }

  async login(credentials: LoginCredentials): Promise<string> {
    return invoke('ctp_login', { credentials });
  }

  async confirmSettlement(): Promise<string> {
    return invoke('ctp_confirm_settlement');
  }

  async getStatus(): Promise<string> {
    return invoke('ctp_get_status');
  }

  async disconnect(): Promise<string> {
    return invoke('ctp_disconnect');
  }

  // Market Data
  async subscribe(instrumentIds: string[]): Promise<string> {
    return invoke('ctp_subscribe', { instrumentIds });
  }

  async unsubscribe(instrumentIds: string[]): Promise<string> {
    return invoke('ctp_unsubscribe', { instrumentIds });
  }

  async batchSubscribe(subscriptions: MarketDataSubscription[]): Promise<string> {
    return invoke('ctp_batch_subscribe', { subscriptions });
  }

  async getMarketData(instrumentId: string): Promise<MarketData> {
    return invoke('ctp_get_market_data', { instrumentId });
  }

  async getAllMarketData(): Promise<MarketData[]> {
    return invoke('ctp_get_all_market_data');
  }

  // Trading Operations
  async placeOrder(order: OrderInput): Promise<OrderRef> {
    return invoke('ctp_place_order', { order });
  }

  async cancelOrder(orderRef: string, instrumentId: string): Promise<string> {
    return invoke('ctp_cancel_order', { orderRef, instrumentId });
  }

  // Query Operations
  async queryAccount(): Promise<AccountInfo> {
    return invoke('ctp_query_account');
  }

  async queryPositions(): Promise<Position[]> {
    return invoke('ctp_query_positions');
  }

  async queryOrders(): Promise<OrderStatus[]> {
    return invoke('ctp_query_orders');
  }

  async queryTrades(): Promise<Trade[]> {
    return invoke('ctp_query_trades');
  }

  async queryInstruments(): Promise<InstrumentInfo[]> {
    return invoke('ctp_query_instruments');
  }

  async queryCommissionRate(instrumentId: string): Promise<CommissionRate> {
    return invoke('ctp_query_commission_rate', { instrumentId });
  }

  async queryMarginRate(instrumentId: string): Promise<MarginRate> {
    return invoke('ctp_query_margin_rate', { instrumentId });
  }

  // Risk Management
  async setRiskParams(params: RiskParams): Promise<string> {
    return invoke('ctp_set_risk_params', { params });
  }
}

// Singleton instance
let ctpServiceInstance: CtpService | null = null;

export function getCtpService(): CtpService {
  if (!ctpServiceInstance) {
    ctpServiceInstance = new CtpService();
  }
  return ctpServiceInstance;
}

// Helper functions for common operations
export async function quickBuy(
  instrumentId: string, 
  price: number, 
  volume: number
): Promise<OrderRef> {
  const service = getCtpService();
  return service.placeOrder({
    instrument_id: instrumentId,
    direction: 'Buy',
    offset: 'Open',
    price,
    volume,
    order_type: 'Limit',
    time_condition: 'GFD',
    volume_condition: 'Any',
    min_volume: 1,
    contingent_condition: 'Immediately',
    stop_price: 0,
    force_close_reason: 'NotForceClose',
    is_auto_suspend: false
  });
}

export async function quickSell(
  instrumentId: string, 
  price: number, 
  volume: number
): Promise<OrderRef> {
  const service = getCtpService();
  return service.placeOrder({
    instrument_id: instrumentId,
    direction: 'Sell',
    offset: 'Open',
    price,
    volume,
    order_type: 'Limit',
    time_condition: 'GFD',
    volume_condition: 'Any',
    min_volume: 1,
    contingent_condition: 'Immediately',
    stop_price: 0,
    force_close_reason: 'NotForceClose',
    is_auto_suspend: false
  });
}

export async function closePosition(
  instrumentId: string,
  direction: 'Buy' | 'Sell',
  price: number,
  volume: number
): Promise<OrderRef> {
  const service = getCtpService();
  return service.placeOrder({
    instrument_id: instrumentId,
    direction: direction === 'Buy' ? 'Sell' : 'Buy',
    offset: 'Close',
    price,
    volume,
    order_type: 'Limit',
    time_condition: 'GFD',
    volume_condition: 'Any',
    min_volume: 1,
    contingent_condition: 'Immediately',
    stop_price: 0,
    force_close_reason: 'NotForceClose',
    is_auto_suspend: false
  });
}

export async function marketOrder(
  instrumentId: string,
  direction: 'Buy' | 'Sell',
  volume: number
): Promise<OrderRef> {
  const service = getCtpService();
  return service.placeOrder({
    instrument_id: instrumentId,
    direction,
    offset: 'Open',
    price: 0, // Market order uses 0 price
    volume,
    order_type: 'Market',
    time_condition: 'IOC',
    volume_condition: 'Any',
    min_volume: 1,
    contingent_condition: 'Immediately',
    stop_price: 0,
    force_close_reason: 'NotForceClose',
    is_auto_suspend: false
  });
}

// Real-time data subscription manager
export class MarketDataManager {
  private subscriptions: Set<string> = new Set();
  private dataCache: Map<string, MarketData> = new Map();
  private updateCallbacks: Map<string, Array<(data: MarketData) => void>> = new Map();
  private pollInterval: NodeJS.Timeout | null = null;

  async subscribe(instrumentId: string, callback?: (data: MarketData) => void): Promise<void> {
    if (!this.subscriptions.has(instrumentId)) {
      const service = getCtpService();
      await service.subscribe([instrumentId]);
      this.subscriptions.add(instrumentId);
    }

    if (callback) {
      if (!this.updateCallbacks.has(instrumentId)) {
        this.updateCallbacks.set(instrumentId, []);
      }
      this.updateCallbacks.get(instrumentId)!.push(callback);
    }

    // Start polling if not already started
    if (!this.pollInterval && this.subscriptions.size > 0) {
      this.startPolling();
    }
  }

  async unsubscribe(instrumentId: string): Promise<void> {
    if (this.subscriptions.has(instrumentId)) {
      const service = getCtpService();
      await service.unsubscribe([instrumentId]);
      this.subscriptions.delete(instrumentId);
      this.updateCallbacks.delete(instrumentId);
      this.dataCache.delete(instrumentId);
    }

    // Stop polling if no subscriptions
    if (this.subscriptions.size === 0 && this.pollInterval) {
      this.stopPolling();
    }
  }

  private startPolling(): void {
    this.pollInterval = setInterval(async () => {
      try {
        const service = getCtpService();
        const allData = await service.getAllMarketData();
        
        for (const data of allData) {
          const oldData = this.dataCache.get(data.instrument_id);
          this.dataCache.set(data.instrument_id, data);
          
          // Notify callbacks if data changed
          if (!oldData || oldData.last_price !== data.last_price) {
            const callbacks = this.updateCallbacks.get(data.instrument_id);
            if (callbacks) {
              callbacks.forEach(cb => cb(data));
            }
          }
        }
      } catch (error) {
        console.error('Failed to poll market data:', error);
      }
    }, 500); // Poll every 500ms
  }

  private stopPolling(): void {
    if (this.pollInterval) {
      clearInterval(this.pollInterval);
      this.pollInterval = null;
    }
  }

  getLatestData(instrumentId: string): MarketData | undefined {
    return this.dataCache.get(instrumentId);
  }

  getAllData(): MarketData[] {
    return Array.from(this.dataCache.values());
  }

  cleanup(): void {
    this.stopPolling();
    this.subscriptions.clear();
    this.dataCache.clear();
    this.updateCallbacks.clear();
  }
}

// Export singleton instance
export const marketDataManager = new MarketDataManager();
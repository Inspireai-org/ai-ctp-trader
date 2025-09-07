import { create } from 'zustand';
import { getCtpService, marketDataManager } from '@/services/ctpService';
import { 
  OrderStatus, 
  Trade, 
  OrderInput, 
  OrderRef,
  RiskParams 
} from '@/types/ctp';
import { message } from 'antd';

interface TradingStore {
  orders: OrderStatus[];
  activeOrders: OrderStatus[];
  trades: Trade[];
  riskParams: RiskParams | null;
  
  // Order Management
  placeOrder: (order: OrderInput) => Promise<OrderRef>;
  cancelOrder: (orderRef: string, instrumentId: string) => Promise<void>;
  updateOrderStatus: (order: OrderStatus) => void;
  
  // Trade Management
  addTrade: (trade: Trade) => void;
  
  // Query Operations
  refreshOrders: () => Promise<void>;
  refreshTrades: () => Promise<void>;
  
  // Risk Management
  setRiskParams: (params: RiskParams) => Promise<void>;
  checkRiskLimits: (order: OrderInput) => boolean;
  
  // Initialization
  initializeServices: () => Promise<void>;
  cleanup: () => void;
}

export const useTradingStore = create<TradingStore>((set, get) => ({
  orders: [],
  activeOrders: [],
  trades: [],
  riskParams: null,
  
  placeOrder: async (order) => {
    try {
      // Check risk limits before placing order
      if (!get().checkRiskLimits(order)) {
        throw new Error('Order exceeds risk limits');
      }
      
      const service = getCtpService();
      const orderRef = await service.placeOrder(order);
      
      // Create pending order status
      const pendingOrder: OrderStatus = {
        order_ref: orderRef.order_ref,
        instrument_id: order.instrument_id,
        direction: order.direction,
        offset: order.offset,
        price: order.price,
        volume: order.volume,
        volume_traded: 0,
        volume_left: order.volume,
        status: 'Submitted',
        status_msg: 'Order submitted',
        insert_time: new Date().toISOString(),
        update_time: new Date().toISOString(),
        front_id: orderRef.front_id,
        session_id: orderRef.session_id,
        exchange_id: '',
        order_sys_id: ''
      };
      
      set((state) => ({
        orders: [...state.orders, pendingOrder],
        activeOrders: [...state.activeOrders, pendingOrder]
      }));
      
      message.success(`Order placed: ${orderRef.order_ref}`);
      return orderRef;
    } catch (error: any) {
      message.error(`Failed to place order: ${error.message}`);
      throw error;
    }
  },
  
  cancelOrder: async (orderRef, instrumentId) => {
    try {
      const service = getCtpService();
      await service.cancelOrder(orderRef, instrumentId);
      
      set((state) => ({
        activeOrders: state.activeOrders.filter(o => o.order_ref !== orderRef),
        orders: state.orders.map(o => 
          o.order_ref === orderRef 
            ? { ...o, status: 'Cancelling' as const, update_time: new Date().toISOString() }
            : o
        )
      }));
      
      message.success(`Cancel request sent: ${orderRef}`);
    } catch (error: any) {
      message.error(`Failed to cancel order: ${error.message}`);
      throw error;
    }
  },
  
  updateOrderStatus: (order) => {
    set((state) => {
      const existingIndex = state.orders.findIndex(o => o.order_ref === order.order_ref);
      const newOrders = [...state.orders];
      
      if (existingIndex >= 0) {
        newOrders[existingIndex] = order;
      } else {
        newOrders.push(order);
      }
      
      const isActive = ['Submitted', 'Accepted', 'PartiallyFilled'].includes(order.status);
      const activeOrders = newOrders.filter(o => 
        ['Submitted', 'Accepted', 'PartiallyFilled'].includes(o.status)
      );
      
      return {
        orders: newOrders,
        activeOrders
      };
    });
  },
  
  addTrade: (trade) => {
    set((state) => ({
      trades: [...state.trades, trade]
    }));
  },
  
  refreshOrders: async () => {
    try {
      const service = getCtpService();
      const orders = await service.queryOrders();
      
      const activeOrders = orders.filter(o => 
        ['Submitted', 'Accepted', 'PartiallyFilled'].includes(o.status)
      );
      
      set({ orders, activeOrders });
    } catch (error: any) {
      console.error('Failed to refresh orders:', error);
    }
  },
  
  refreshTrades: async () => {
    try {
      const service = getCtpService();
      const trades = await service.queryTrades();
      set({ trades });
    } catch (error: any) {
      console.error('Failed to refresh trades:', error);
    }
  },
  
  setRiskParams: async (params) => {
    try {
      const service = getCtpService();
      await service.setRiskParams(params);
      set({ riskParams: params });
      message.success('Risk parameters updated');
    } catch (error: any) {
      message.error(`Failed to set risk parameters: ${error.message}`);
      throw error;
    }
  },
  
  checkRiskLimits: (order) => {
    const { riskParams } = get();
    if (!riskParams) return true;
    
    // Check max order volume
    if (order.volume > riskParams.max_order_volume) {
      message.warning(`Order volume exceeds limit: ${riskParams.max_order_volume}`);
      return false;
    }
    
    // Check forbidden instruments
    if (riskParams.forbidden_instruments.includes(order.instrument_id)) {
      message.warning(`Trading forbidden for instrument: ${order.instrument_id}`);
      return false;
    }
    
    // Check position limits
    const positionLimit = riskParams.position_limit.get(order.instrument_id);
    if (positionLimit && order.volume > positionLimit) {
      message.warning(`Order exceeds position limit for ${order.instrument_id}`);
      return false;
    }
    
    return true;
  },
  
  initializeServices: async () => {
    try {
      // Initialize CTP service
      const service = getCtpService();
      await service.init();
      
      // Set default risk parameters
      const defaultRiskParams: RiskParams = {
        max_position_ratio: 0.8,
        max_single_loss: 10000,
        max_daily_loss: 50000,
        max_order_volume: 100,
        position_limit: new Map(),
        forbidden_instruments: [],
        auto_stop_loss: true,
        stop_loss_ratio: 0.05,
        auto_take_profit: false,
        take_profit_ratio: 0.1
      };
      
      await get().setRiskParams(defaultRiskParams);
      
      // Start periodic refresh
      setInterval(() => {
        get().refreshOrders();
        get().refreshTrades();
      }, 5000);
      
      console.log('Trading services initialized');
    } catch (error: any) {
      console.error('Failed to initialize trading services:', error);
      throw error;
    }
  },
  
  cleanup: () => {
    marketDataManager.cleanup();
  }
}));
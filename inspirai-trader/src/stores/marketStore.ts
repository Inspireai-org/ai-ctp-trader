import { create } from 'zustand';
import { MarketData } from '@/types/ctp';
import { marketDataManager } from '@/services/ctpService';

interface MarketStore {
  // Market Data
  marketData: Map<string, MarketData>;
  selectedInstrument: string | null;
  subscribedInstruments: string[];
  
  // Actions
  selectInstrument: (instrumentId: string) => void;
  subscribeMarketData: (instrumentId: string) => Promise<void>;
  unsubscribeMarketData: (instrumentId: string) => Promise<void>;
  updateMarketData: (data: MarketData) => void;
  getMarketData: (instrumentId: string) => MarketData | undefined;
  
  // Batch operations
  subscribeMultiple: (instrumentIds: string[]) => Promise<void>;
  unsubscribeAll: () => Promise<void>;
}

export const useMarketStore = create<MarketStore>((set, get) => ({
  marketData: new Map(),
  selectedInstrument: null,
  subscribedInstruments: [],
  
  selectInstrument: (instrumentId) => {
    set({ selectedInstrument: instrumentId });
  },
  
  subscribeMarketData: async (instrumentId) => {
    try {
      await marketDataManager.subscribe(instrumentId, (data) => {
        get().updateMarketData(data);
      });
      
      set((state) => ({
        subscribedInstruments: [...new Set([...state.subscribedInstruments, instrumentId])]
      }));
    } catch (error) {
      console.error(`Failed to subscribe to ${instrumentId}:`, error);
    }
  },
  
  unsubscribeMarketData: async (instrumentId) => {
    try {
      await marketDataManager.unsubscribe(instrumentId);
      
      set((state) => ({
        subscribedInstruments: state.subscribedInstruments.filter(id => id !== instrumentId),
        marketData: new Map(Array.from(state.marketData).filter(([key]) => key !== instrumentId))
      }));
    } catch (error) {
      console.error(`Failed to unsubscribe from ${instrumentId}:`, error);
    }
  },
  
  updateMarketData: (data) => {
    set((state) => {
      const newData = new Map(state.marketData);
      newData.set(data.instrument_id, data);
      return { marketData: newData };
    });
  },
  
  getMarketData: (instrumentId) => {
    return get().marketData.get(instrumentId);
  },
  
  subscribeMultiple: async (instrumentIds) => {
    for (const id of instrumentIds) {
      await get().subscribeMarketData(id);
    }
  },
  
  unsubscribeAll: async () => {
    const { subscribedInstruments } = get();
    for (const id of subscribedInstruments) {
      await marketDataManager.unsubscribe(id);
    }
    
    set({
      subscribedInstruments: [],
      marketData: new Map()
    });
  }
}));
import { create } from 'zustand';
import { getCtpService } from '@/services/ctpService';
import { ConnectionState, CtpConfig, LoginCredentials } from '@/types/ctp';
import { message } from 'antd';

interface ConnectionStore {
  // Connection State
  connectionState: ConnectionState;
  connectionStatus: string;
  isConnected: boolean;
  isLoggedIn: boolean;
  userId: string | null;
  
  // Configuration
  config: CtpConfig | null;
  credentials: LoginCredentials | null;
  
  // Actions
  connect: () => Promise<void>;
  login: (credentials: LoginCredentials) => Promise<void>;
  disconnect: () => Promise<void>;
  updateConnectionState: (state: ConnectionState, status?: string) => void;
  setConfig: (config: CtpConfig) => void;
  
  // Auto-reconnect
  autoReconnect: boolean;
  reconnectAttempts: number;
  maxReconnectAttempts: number;
  toggleAutoReconnect: (enabled: boolean) => void;
}

export const useConnectionStore = create<ConnectionStore>((set, get) => ({
  connectionState: ConnectionState.Disconnected,
  connectionStatus: 'Disconnected',
  isConnected: false,
  isLoggedIn: false,
  userId: null,
  config: null,
  credentials: null,
  autoReconnect: true,
  reconnectAttempts: 0,
  maxReconnectAttempts: 5,
  
  connect: async () => {
    try {
      set({ 
        connectionState: ConnectionState.Connecting,
        connectionStatus: 'Connecting to CTP server...'
      });
      
      const service = getCtpService();
      
      // Get or create config
      let config = get().config;
      if (!config) {
        config = await service.createConfig();
        set({ config });
      }
      
      // Connect to CTP
      const result = await service.connect(config);
      
      set({ 
        connectionState: ConnectionState.Connected,
        connectionStatus: 'Connected',
        isConnected: true,
        reconnectAttempts: 0
      });
      
      message.success(result);
      
      // Auto-login if credentials are saved
      const { credentials } = get();
      if (credentials) {
        await get().login(credentials);
      }
    } catch (error: any) {
      set({ 
        connectionState: ConnectionState.Disconnected,
        connectionStatus: `Connection failed: ${error.message}`,
        isConnected: false
      });
      
      message.error(`Connection failed: ${error.message}`);
      
      // Auto-reconnect logic
      const { autoReconnect, reconnectAttempts, maxReconnectAttempts } = get();
      if (autoReconnect && reconnectAttempts < maxReconnectAttempts) {
        set((state) => ({ reconnectAttempts: state.reconnectAttempts + 1 }));
        
        const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 30000);
        message.info(`Reconnecting in ${delay / 1000} seconds...`);
        
        setTimeout(() => {
          get().connect();
        }, delay);
      }
      
      throw error;
    }
  },
  
  login: async (credentials) => {
    try {
      set({ 
        connectionState: ConnectionState.LoggingIn,
        connectionStatus: 'Logging in...',
        credentials
      });
      
      const service = getCtpService();
      const result = await service.login(credentials);
      
      // Auto-confirm settlement
      try {
        await service.confirmSettlement();
      } catch (error) {
        console.warn('Settlement confirmation failed:', error);
      }
      
      set({ 
        connectionState: ConnectionState.LoggedIn,
        connectionStatus: 'Logged in',
        isLoggedIn: true,
        userId: credentials.user_id
      });
      
      message.success(result);
    } catch (error: any) {
      set({ 
        connectionState: ConnectionState.Connected,
        connectionStatus: `Login failed: ${error.message}`,
        isLoggedIn: false
      });
      
      message.error(`Login failed: ${error.message}`);
      throw error;
    }
  },
  
  disconnect: async () => {
    try {
      set({ 
        connectionState: ConnectionState.Disconnecting,
        connectionStatus: 'Disconnecting...'
      });
      
      const service = getCtpService();
      const result = await service.disconnect();
      
      set({ 
        connectionState: ConnectionState.Disconnected,
        connectionStatus: 'Disconnected',
        isConnected: false,
        isLoggedIn: false,
        userId: null,
        reconnectAttempts: 0
      });
      
      message.info(result);
    } catch (error: any) {
      message.error(`Disconnect failed: ${error.message}`);
      throw error;
    }
  },
  
  updateConnectionState: (state, status) => {
    set({ 
      connectionState: state,
      connectionStatus: status || state,
      isConnected: [
        ConnectionState.Connected,
        ConnectionState.LoggingIn,
        ConnectionState.LoggedIn
      ].includes(state),
      isLoggedIn: state === ConnectionState.LoggedIn
    });
  },
  
  setConfig: (config) => {
    set({ config });
  },
  
  toggleAutoReconnect: (enabled) => {
    set({ autoReconnect: enabled });
    message.info(`Auto-reconnect ${enabled ? 'enabled' : 'disabled'}`);
  }
}));
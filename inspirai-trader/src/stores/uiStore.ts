import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface ChartConfig {
  theme: 'dark' | 'light';
  gridColor: string;
  upColor: string;
  downColor: string;
  volumeUpColor: string;
  volumeDownColor: string;
  indicators: string[];
  timeframe: string;
}

interface Shortcuts {
  buy: string;
  sell: string;
  closeAll: string;
  cancelAll: string;
  switchChart: string;
  fullscreen: string;
}

interface UIStore {
  // Theme
  theme: 'dark' | 'light' | 'auto';
  setTheme: (theme: 'dark' | 'light' | 'auto') => void;
  
  // Language
  language: 'zh-CN' | 'zh-TW' | 'en-US';
  setLanguage: (language: 'zh-CN' | 'zh-TW' | 'en-US') => void;
  
  // Font Size
  fontSize: number;
  setFontSize: (size: number) => void;
  
  // Sound & Notifications
  soundEnabled: boolean;
  setSoundEnabled: (enabled: boolean) => void;
  notificationEnabled: boolean;
  setNotificationEnabled: (enabled: boolean) => void;
  
  // Chart Configuration
  chartConfig: ChartConfig;
  updateChartConfig: (config: Partial<ChartConfig>) => void;
  
  // Shortcuts
  shortcuts: Shortcuts;
  updateShortcuts: (shortcuts: Partial<Shortcuts>) => void;
  
  // Settings Management
  saveSettings: (settings: any) => Promise<void>;
  loadSettings: (settings: any) => void;
  resetSettings: () => void;
}

const defaultChartConfig: ChartConfig = {
  theme: 'dark',
  gridColor: '#2b2b2b',
  upColor: '#ef5350',
  downColor: '#26a69a',
  volumeUpColor: 'rgba(239, 83, 80, 0.5)',
  volumeDownColor: 'rgba(38, 166, 154, 0.5)',
  indicators: ['MA', 'MACD', 'RSI'],
  timeframe: '1m'
};

const defaultShortcuts: Shortcuts = {
  buy: 'Ctrl+B',
  sell: 'Ctrl+S',
  closeAll: 'Ctrl+Shift+C',
  cancelAll: 'Ctrl+Shift+X',
  switchChart: 'Ctrl+Tab',
  fullscreen: 'F11'
};

export const useUIStore = create<UIStore>()(
  persist(
    (set, get) => ({
      // Initial values
      theme: 'dark',
      language: 'zh-CN',
      fontSize: 14,
      soundEnabled: true,
      notificationEnabled: true,
      chartConfig: defaultChartConfig,
      shortcuts: defaultShortcuts,
      
      // Theme
      setTheme: (theme) => {
        set({ theme });
        
        // Apply theme to document
        if (theme === 'auto') {
          const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
          document.documentElement.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
        } else {
          document.documentElement.setAttribute('data-theme', theme);
        }
      },
      
      // Language
      setLanguage: (language) => {
        set({ language });
        // TODO: Implement i18n language switching
      },
      
      // Font Size
      setFontSize: (fontSize) => {
        set({ fontSize });
        document.documentElement.style.fontSize = `${fontSize}px`;
      },
      
      // Sound & Notifications
      setSoundEnabled: (soundEnabled) => {
        set({ soundEnabled });
      },
      
      setNotificationEnabled: (notificationEnabled) => {
        set({ notificationEnabled });
        
        // Request notification permission if enabled
        if (notificationEnabled && 'Notification' in window) {
          Notification.requestPermission();
        }
      },
      
      // Chart Configuration
      updateChartConfig: (config) => {
        set((state) => ({
          chartConfig: { ...state.chartConfig, ...config }
        }));
      },
      
      // Shortcuts
      updateShortcuts: (shortcuts) => {
        set((state) => ({
          shortcuts: { ...state.shortcuts, ...shortcuts }
        }));
      },
      
      // Settings Management
      saveSettings: async (settings) => {
        // Save to localStorage
        localStorage.setItem('tradingSettings', JSON.stringify(settings));
        
        // TODO: Optionally save to backend
        return Promise.resolve();
      },
      
      loadSettings: (settings) => {
        if (settings.theme) get().setTheme(settings.theme);
        if (settings.language) get().setLanguage(settings.language);
        if (settings.fontSize) get().setFontSize(settings.fontSize);
        if (settings.soundEnabled !== undefined) get().setSoundEnabled(settings.soundEnabled);
        if (settings.notificationEnabled !== undefined) get().setNotificationEnabled(settings.notificationEnabled);
        if (settings.chartConfig) get().updateChartConfig(settings.chartConfig);
        if (settings.shortcuts) get().updateShortcuts(settings.shortcuts);
      },
      
      resetSettings: () => {
        set({
          theme: 'dark',
          language: 'zh-CN',
          fontSize: 14,
          soundEnabled: true,
          notificationEnabled: true,
          chartConfig: defaultChartConfig,
          shortcuts: defaultShortcuts
        });
        
        // Apply defaults
        get().setTheme('dark');
        get().setFontSize(14);
      }
    }),
    {
      name: 'ui-settings',
      partialize: (state) => ({
        theme: state.theme,
        language: state.language,
        fontSize: state.fontSize,
        soundEnabled: state.soundEnabled,
        notificationEnabled: state.notificationEnabled,
        chartConfig: state.chartConfig,
        shortcuts: state.shortcuts
      })
    }
  )
);

// Helper functions for notifications
export function showNotification(title: string, body: string, options?: NotificationOptions) {
  const store = useUIStore.getState();
  
  if (store.notificationEnabled && 'Notification' in window && Notification.permission === 'granted') {
    new Notification(title, { body, ...options });
  }
}

export function playSound(soundType: 'success' | 'error' | 'warning' | 'notification') {
  const store = useUIStore.getState();
  
  if (store.soundEnabled) {
    // TODO: Implement sound playing logic
    const audio = new Audio(`/sounds/${soundType}.mp3`);
    audio.play().catch(error => console.error('Failed to play sound:', error));
  }
}
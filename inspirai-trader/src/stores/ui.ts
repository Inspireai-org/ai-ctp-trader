import { create } from 'zustand';
import { persist } from 'zustand/middleware';

/**
 * 界面配置类型定义
 */
export interface LayoutConfig {
  panels: {
    market: PanelConfig;
    chart: PanelConfig;
    trading: PanelConfig;
    info: PanelConfig;
  };
  gridCols: number;
  gridRows: number;
}

export interface PanelConfig {
  visible: boolean;
  position: { x: number; y: number };
  size: { width: number; height: number };
  minSize: { width: number; height: number };
  resizable: boolean;
  draggable: boolean;
  collapsed: boolean;
}

export interface ThemeConfig {
  mode: 'dark' | 'light';
  primaryColor: string;
  upColor: string;
  downColor: string;
  neutralColor: string;
  fontSize: number;
  fontFamily: string;
}

export interface UserPreferences {
  language: 'zh-CN' | 'en-US';
  numberFormat: 'zh' | 'en';
  dateFormat: string;
  timeFormat: '12h' | '24h';
  shortcuts: Record<string, string>;
  autoSave: boolean;
  soundEnabled: boolean;
  notificationEnabled: boolean;
}

/**
 * UI 状态接口
 */
interface UIState {
  // 主题相关
  theme: 'dark' | 'light';
  themeConfig: ThemeConfig;
  
  // 布局相关
  layout: LayoutConfig;
  isLayoutLocked: boolean;
  
  // 用户偏好
  preferences: UserPreferences;
  
  // 界面状态
  sidebarCollapsed: boolean;
  fullscreenPanel: string | null;
  activeTab: string;
  
  // Actions
  switchTheme: (theme: 'dark' | 'light') => void;
  updateThemeConfig: (config: Partial<ThemeConfig>) => void;
  updateLayout: (layout: LayoutConfig) => void;
  toggleLayoutLock: () => void;
  updatePreferences: (prefs: Partial<UserPreferences>) => void;
  toggleSidebar: () => void;
  setFullscreenPanel: (panelId: string | null) => void;
  setActiveTab: (tabId: string) => void;
  resetToDefault: () => void;
  initializeTheme: () => Promise<void>;
}

/**
 * 默认配置
 */
const defaultLayout: LayoutConfig = {
  panels: {
    market: {
      visible: true,
      position: { x: 0, y: 0 },
      size: { width: 3, height: 6 },
      minSize: { width: 2, height: 4 },
      resizable: true,
      draggable: true,
      collapsed: false,
    },
    chart: {
      visible: true,
      position: { x: 3, y: 0 },
      size: { width: 6, height: 8 },
      minSize: { width: 4, height: 6 },
      resizable: true,
      draggable: true,
      collapsed: false,
    },
    trading: {
      visible: true,
      position: { x: 9, y: 0 },
      size: { width: 3, height: 6 },
      minSize: { width: 2, height: 4 },
      resizable: true,
      draggable: true,
      collapsed: false,
    },
    info: {
      visible: true,
      position: { x: 0, y: 8 },
      size: { width: 12, height: 4 },
      minSize: { width: 6, height: 3 },
      resizable: true,
      draggable: true,
      collapsed: false,
    },
  },
  gridCols: 12,
  gridRows: 12,
};

const defaultThemeConfig: ThemeConfig = {
  mode: 'dark',
  primaryColor: '#1890ff',
  upColor: '#00d4aa',
  downColor: '#ff4d4f',
  neutralColor: '#faad14',
  fontSize: 14,
  fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", "Helvetica Neue", Helvetica, Arial, sans-serif',
};

const defaultPreferences: UserPreferences = {
  language: 'zh-CN',
  numberFormat: 'zh',
  dateFormat: 'YYYY-MM-DD',
  timeFormat: '24h',
  shortcuts: {
    'buy': 'F1',
    'sell': 'F2',
    'cancel_all': 'F3',
    'close_all': 'F4',
    'refresh': 'F5',
  },
  autoSave: true,
  soundEnabled: true,
  notificationEnabled: true,
};

/**
 * UI 状态管理 Store
 */
export const useUIStore = create<UIState>()(
  persist(
    (set, get) => ({
      // 初始状态
      theme: 'dark',
      themeConfig: defaultThemeConfig,
      layout: defaultLayout,
      isLayoutLocked: false,
      preferences: defaultPreferences,
      sidebarCollapsed: false,
      fullscreenPanel: null,
      activeTab: 'market',

      // Actions
      switchTheme: (theme) => {
        set((state) => ({
          theme,
          themeConfig: { ...state.themeConfig, mode: theme },
        }));
        
        // 更新 DOM 属性（仅在浏览器环境中）
        if (typeof document !== 'undefined') {
          document.documentElement.setAttribute('data-theme', theme);
        }
      },

      updateThemeConfig: (config) => {
        set((state) => ({
          themeConfig: { ...state.themeConfig, ...config },
        }));
      },

      updateLayout: (layout) => {
        const { isLayoutLocked } = get();
        if (!isLayoutLocked) {
          set({ layout });
        }
      },

      toggleLayoutLock: () => {
        set((state) => ({ isLayoutLocked: !state.isLayoutLocked }));
      },

      updatePreferences: (prefs) => {
        set((state) => ({
          preferences: { ...state.preferences, ...prefs },
        }));
      },

      toggleSidebar: () => {
        set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed }));
      },

      setFullscreenPanel: (panelId) => {
        set({ fullscreenPanel: panelId });
      },

      setActiveTab: (tabId) => {
        set({ activeTab: tabId });
      },

      resetToDefault: () => {
        set({
          theme: 'dark',
          themeConfig: defaultThemeConfig,
          layout: defaultLayout,
          isLayoutLocked: false,
          preferences: defaultPreferences,
          sidebarCollapsed: false,
          fullscreenPanel: null,
          activeTab: 'market',
        });
        
        // 重置 DOM 属性（仅在浏览器环境中）
        if (typeof document !== 'undefined') {
          document.documentElement.setAttribute('data-theme', 'dark');
        }
      },

      initializeTheme: async () => {
        const { theme } = get();
        
        // 仅在浏览器环境中执行
        if (typeof window !== 'undefined') {
          // 检测系统主题偏好
          const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
          
          // 如果没有保存的主题设置，使用系统偏好
          const finalTheme = theme || (prefersDark ? 'dark' : 'light');
          
          // 设置主题
          set({ theme: finalTheme });
          if (typeof document !== 'undefined') {
            document.documentElement.setAttribute('data-theme', finalTheme);
          }
          
          // 监听系统主题变化
          window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
            const { preferences } = get();
            if (preferences.autoSave) {
              const newTheme = e.matches ? 'dark' : 'light';
              get().switchTheme(newTheme);
            }
          });
        }
      },
    }),
    {
      name: 'ui-store',
      partialize: (state) => ({
        theme: state.theme,
        themeConfig: state.themeConfig,
        layout: state.layout,
        preferences: state.preferences,
        sidebarCollapsed: state.sidebarCollapsed,
      }),
    }
  )
);

/**
 * 主题相关的 Hook
 */
export const useTheme = () => {
  const { theme, themeConfig, switchTheme, updateThemeConfig } = useUIStore();
  
  return {
    theme,
    themeConfig,
    switchTheme,
    updateThemeConfig,
    isDark: theme === 'dark',
    isLight: theme === 'light',
  };
};

/**
 * 布局相关的 Hook
 */
export const useLayout = () => {
  const { layout, isLayoutLocked, updateLayout, toggleLayoutLock } = useUIStore();
  
  return {
    layout,
    isLayoutLocked,
    updateLayout,
    toggleLayoutLock,
  };
};

/**
 * 用户偏好相关的 Hook
 */
export const usePreferences = () => {
  const { preferences, updatePreferences } = useUIStore();
  
  return {
    preferences,
    updatePreferences,
  };
};
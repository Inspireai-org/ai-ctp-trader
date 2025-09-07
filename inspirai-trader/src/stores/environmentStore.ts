/**
 * 环境状态管理 Store
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { EnvironmentStatus, ConnectionRecord } from '@/types/ctp';

export interface EnvironmentStore {
  // 状态
  currentEnvironment: string | null;
  availableEnvironments: EnvironmentStatus[];
  isWeekendMode: boolean;
  connectionHistory: ConnectionRecord[];
  environmentConfig: {
    defaultEnvironment: string;
    autoSwitchWeekend: boolean;
    connectionTimeout: number;
    autoReconnect: boolean;
  };

  // Actions
  setCurrentEnvironment: (env: string | null) => void;
  updateEnvironmentStatus: (status: EnvironmentStatus) => void;
  setAvailableEnvironments: (environments: EnvironmentStatus[]) => void;
  setWeekendMode: (isWeekend: boolean) => void;
  addConnectionRecord: (record: ConnectionRecord) => void;
  clearConnectionHistory: () => void;
  updateEnvironmentConfig: (config: Partial<EnvironmentStore['environmentConfig']>) => void;
  
  // Getters
  getEnvironmentStatus: (presetKey: string) => EnvironmentStatus | undefined;
  getRecentConnections: (limit?: number) => ConnectionRecord[];
  getSuccessfulConnections: () => ConnectionRecord[];
}

export const useEnvironmentStore = create<EnvironmentStore>()(
  persist(
    (set, get) => ({
      // 初始状态
      currentEnvironment: null,
      availableEnvironments: [],
      isWeekendMode: false,
      connectionHistory: [],
      environmentConfig: {
        defaultEnvironment: 'gzqh_test',
        autoSwitchWeekend: true,
        connectionTimeout: 30000,
        autoReconnect: true,
      },

      // Actions
      setCurrentEnvironment: (env) => {
        set({ currentEnvironment: env });
      },

      updateEnvironmentStatus: (status) => {
        set((state) => {
          const existingIndex = state.availableEnvironments.findIndex(
            env => env.presetKey === status.presetKey
          );
          
          const newEnvironments = [...state.availableEnvironments];
          if (existingIndex >= 0) {
            newEnvironments[existingIndex] = status;
          } else {
            newEnvironments.push(status);
          }
          
          return { availableEnvironments: newEnvironments };
        });
      },

      setAvailableEnvironments: (environments) => {
        set({ availableEnvironments: environments });
      },

      setWeekendMode: (isWeekend) => {
        set({ isWeekendMode: isWeekend });
      },

      addConnectionRecord: (record) => {
        set((state) => {
          const newHistory = [record, ...state.connectionHistory];
          // 只保留最近 50 条记录
          return { 
            connectionHistory: newHistory.slice(0, 50) 
          };
        });
      },

      clearConnectionHistory: () => {
        set({ connectionHistory: [] });
      },

      updateEnvironmentConfig: (config) => {
        set((state) => ({
          environmentConfig: { ...state.environmentConfig, ...config }
        }));
      },

      // Getters
      getEnvironmentStatus: (presetKey) => {
        const state = get();
        return state.availableEnvironments.find(env => env.presetKey === presetKey);
      },

      getRecentConnections: (limit = 10) => {
        const state = get();
        return state.connectionHistory.slice(0, limit);
      },

      getSuccessfulConnections: () => {
        const state = get();
        return state.connectionHistory.filter(record => record.success);
      },
    }),
    {
      name: 'environment-store',
      // 只持久化配置和历史记录，不持久化实时状态
      partialize: (state) => ({
        connectionHistory: state.connectionHistory,
        environmentConfig: state.environmentConfig,
      }),
    }
  )
);

// 便捷 hooks
export const useCurrentEnvironment = () => useEnvironmentStore(state => state.currentEnvironment);
export const useEnvironmentConfig = () => useEnvironmentStore(state => state.environmentConfig);
export const useConnectionHistory = () => useEnvironmentStore(state => state.connectionHistory);
export const useAvailableEnvironments = () => useEnvironmentStore(state => state.availableEnvironments);

export default useEnvironmentStore;
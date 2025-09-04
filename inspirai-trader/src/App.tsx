import React, { useEffect, useState } from "react";
import { ConfigProvider, theme, App as AntApp, Button, message } from "antd";
import { ApiOutlined, DisconnectOutlined } from "@ant-design/icons";
import zhCN from "antd/locale/zh_CN";
import { TradingLayout } from "@components/layout";
import { useUIStore } from "@stores/ui";
import { ErrorBoundary } from "@components/common";
import CtpConnectionDialog from "@components/connection/CtpConnectionDialog";
import { ctpService } from "@services/tauri";
import { useMarketDataStore } from "@stores/marketData";
import { UnlistenFn } from "@tauri-apps/api/event";
import type { MarketDataTick } from "@/types";
import { ConnectionStatus, ClientState } from "@/types";
import "@styles/globals.css";

/**
 * 期货交易应用主入口组件
 * 
 * 功能特性：
 * - 深色主题为主的专业交易界面
 * - 响应式布局适配不同屏幕尺寸
 * - 全局错误边界处理
 * - 主题切换支持
 * - 中文本地化配置
 */
const App: React.FC = () => {
  const { theme: currentTheme, initializeTheme } = useUIStore();
  const [isInitialized, setIsInitialized] = useState(false);
  const [connectionDialogVisible, setConnectionDialogVisible] = useState(false);
  const [isConnected, setIsConnected] = useState(false);
  const [marketDataListener, setMarketDataListener] = useState<UnlistenFn | null>(null);
  
  const { connectionStatus, setConnectionStatus, updateTick } = useMarketDataStore();

  // 初始化主题和应用配置
  useEffect(() => {
    const initApp = async () => {
      try {
        // 初始化主题设置
        await initializeTheme();
        
        // 设置文档主题属性
        document.documentElement.setAttribute('data-theme', currentTheme);
        
        // 设置应用标题
        document.title = 'Inspirai Trader - 专业期货交易平台';
        
        setIsInitialized(true);
      } catch (error) {
        console.error('应用初始化失败:', error);
        setIsInitialized(true); // 即使失败也要显示界面
      }
    };

    initApp();
  }, [currentTheme, initializeTheme]);

  // 设置市场数据监听器
  useEffect(() => {
    if (isConnected) {
      setupMarketDataListener();
    }

    return () => {
      if (marketDataListener) {
        marketDataListener();
      }
    };
  }, [isConnected]);

  const setupMarketDataListener = async () => {
    try {
      // 监听市场数据更新
      const unlisten = await ctpService.listenToMarketData((tick: MarketDataTick) => {
        // 更新 store 中的行情数据
        updateTick(tick);
      });
      
      setMarketDataListener(unlisten);
      
      // 监听连接状态变化
      await ctpService.listenToConnectionStatus((status) => {
        setConnectionStatus(status === ClientState.LOGGED_IN ? ConnectionStatus.CONNECTED : 
                          status === ClientState.DISCONNECTED ? ConnectionStatus.DISCONNECTED : ConnectionStatus.CONNECTING);
      });
      
      message.success('实时行情数据监听已启动');
    } catch (error) {
      console.error('设置市场数据监听器失败:', error);
      message.error('启动行情监听失败');
    }
  };

  const handleConnect = () => {
    setConnectionDialogVisible(true);
  };

  const handleDisconnect = async () => {
    try {
      await ctpService.disconnect();
      setIsConnected(false);
      setConnectionStatus(ConnectionStatus.DISCONNECTED);
      message.info('已断开 CTP 连接');
    } catch (error) {
      message.error('断开连接失败');
    }
  };

  const handleConnected = () => {
    setIsConnected(true);
    setConnectionDialogVisible(false);
  };

  // 主题配置
  const antdTheme = {
    algorithm: currentTheme === 'dark' ? theme.darkAlgorithm : theme.defaultAlgorithm,
    token: {
      // 基础色彩配置
      colorPrimary: '#1890ff',
      colorSuccess: currentTheme === 'dark' ? '#00d4aa' : '#52c41a',
      colorError: currentTheme === 'dark' ? '#ff4d4f' : '#f5222d',
      colorWarning: '#faad14',
      
      // 布局配置
      borderRadius: 4,
      fontSize: 14,
      fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", "Helvetica Neue", Helvetica, Arial, sans-serif',
      
      // 深色主题配置
      ...(currentTheme === 'dark' && {
        colorBgContainer: '#1a1a1a',
        colorBgElevated: '#2a2a2a',
        colorBgLayout: '#0a0a0a',
        colorBorder: '#3a3a3a',
        colorBorderSecondary: '#3a3a3a',
        colorText: '#ffffff',
        colorTextSecondary: '#cccccc',
        colorTextTertiary: '#888888',
        colorFill: '#4a4a4a',
        colorFillSecondary: '#3a3a3a',
        colorFillTertiary: '#2a2a2a',
      }),
    },
    components: {
      Layout: {
        headerBg: currentTheme === 'dark' ? '#1a1a1a' : '#ffffff',
        bodyBg: currentTheme === 'dark' ? '#0a0a0a' : '#f5f5f5',
        siderBg: currentTheme === 'dark' ? '#1a1a1a' : '#ffffff',
        headerHeight: 48, // 减小头部高度
        headerPadding: '0 16px',
      },
      Card: {
        colorBgContainer: currentTheme === 'dark' ? '#1a1a1a' : '#ffffff',
        colorBorderSecondary: currentTheme === 'dark' ? '#3a3a3a' : '#d9d9d9',
        paddingLG: 16,
      },
      Table: {
        colorBgContainer: currentTheme === 'dark' ? '#1a1a1a' : '#ffffff',
        headerBg: currentTheme === 'dark' ? '#2a2a2a' : '#fafafa',
        colorBorderSecondary: currentTheme === 'dark' ? '#3a3a3a' : '#f0f0f0',
        rowHoverBg: currentTheme === 'dark' ? '#4a4a4a' : '#f5f5f5',
        fontSize: 13, // 表格使用较小字体
        cellPaddingBlock: 8,
        cellPaddingInline: 12,
      },
      Button: {
        primaryColor: '#1890ff',
        borderRadius: 4,
        controlHeight: 32,
      },
      Input: {
        colorBgContainer: currentTheme === 'dark' ? '#2a2a2a' : '#ffffff',
        colorBorder: currentTheme === 'dark' ? '#3a3a3a' : '#d9d9d9',
        activeBorderColor: '#1890ff',
        hoverBorderColor: '#1890ff',
        controlHeight: 32,
      },
      Select: {
        colorBgContainer: currentTheme === 'dark' ? '#2a2a2a' : '#ffffff',
        colorBorder: currentTheme === 'dark' ? '#3a3a3a' : '#d9d9d9',
        optionSelectedBg: '#1890ff',
        controlHeight: 32,
      },
      Tabs: {
        cardBg: currentTheme === 'dark' ? '#1a1a1a' : '#ffffff',
        itemColor: currentTheme === 'dark' ? '#cccccc' : '#666666',
        itemSelectedColor: '#1890ff',
        itemHoverColor: '#1890ff',
        inkBarColor: '#1890ff',
      },
      Tooltip: {
        colorBgSpotlight: currentTheme === 'dark' ? '#2a2a2a' : '#ffffff',
        colorTextLightSolid: currentTheme === 'dark' ? '#ffffff' : '#000000',
      },
    },
  };

  // 加载状态
  if (!isInitialized) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-bg-primary">
        <div className="text-text-primary text-lg">正在初始化交易系统...</div>
      </div>
    );
  }

  return (
    <ErrorBoundary>
      <ConfigProvider
        locale={zhCN}
        theme={antdTheme}
        componentSize="middle"
      >
        <AntApp>
          <div className="app-container min-h-screen bg-bg-primary text-text-primary">
            {/* 连接状态栏 */}
            <div className="fixed top-0 right-0 z-50 p-4">
              {!isConnected ? (
                <Button 
                  type="primary" 
                  icon={<ApiOutlined />}
                  onClick={handleConnect}
                >
                  连接 CTP
                </Button>
              ) : (
                <Button 
                  danger
                  icon={<DisconnectOutlined />}
                  onClick={handleDisconnect}
                >
                  断开连接 ({connectionStatus})
                </Button>
              )}
            </div>
            
            {/* 主交易界面 */}
            <TradingLayout />
            
            {/* CTP 连接对话框 */}
            <CtpConnectionDialog
              visible={connectionDialogVisible}
              onClose={() => setConnectionDialogVisible(false)}
              onConnected={handleConnected}
            />
          </div>
        </AntApp>
      </ConfigProvider>
    </ErrorBoundary>
  );
};

export default App;

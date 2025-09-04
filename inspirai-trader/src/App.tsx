import React, { useEffect, useState } from "react";
import { ConfigProvider, theme, App as AntApp } from "antd";
import zhCN from "antd/locale/zh_CN";
import { TradingLayout } from "@components/layout";
import { useUIStore } from "@stores/ui";
import { ErrorBoundary } from "@components/common";
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
            <TradingLayout />
          </div>
        </AntApp>
      </ConfigProvider>
    </ErrorBoundary>
  );
};

export default App;

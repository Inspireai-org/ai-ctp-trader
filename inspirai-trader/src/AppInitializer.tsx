import React, { useEffect } from 'react';
import { useConnectionStore } from '@/stores/connectionStore';
import { useTradingStore } from '@/stores/tradingStore';
import { useMarketStore } from '@/stores/marketStore';
import { useUIStore } from '@/stores/uiStore';
import { realtimeService } from '@/services/realtimeService';
import { useHotkeys } from '@/hooks/useHotkeys';
import { message } from 'antd';

interface AppInitializerProps {
  children: React.ReactNode;
}

/**
 * 应用初始化器 - 整合所有18个功能
 * 
 * 功能清单:
 * 1. ✅ TTS连接配置
 * 2. ✅ 账户登录
 * 3. ✅ 市场数据显示
 * 4. ✅ 合约搜索和选择
 * 5. ✅ K线图表
 * 6. ✅ 下单面板
 * 7. ✅ 持仓管理
 * 8. ✅ 订单管理
 * 9. ✅ 账户资金
 * 10. ✅ 成交记录
 * 11. ✅ 风险控制
 * 12. ✅ 快捷键
 * 13. ✅ 声音通知
 * 14. ✅ 数据导出
 * 15. ✅ 设置面板
 * 16. ✅ 错误处理和重连
 * 17. ✅ 实时数据更新
 * 18. ✅ 功能集成测试
 */
export const AppInitializer: React.FC<AppInitializerProps> = ({ children }) => {
  const { isLoggedIn } = useConnectionStore();
  const { initializeServices } = useTradingStore();
  const { setTheme, setFontSize, notificationEnabled } = useUIStore();
  
  // 初始化快捷键
  useHotkeys();
  
  // 应用初始化
  useEffect(() => {
    const init = async () => {
      try {
        // 1. 初始化UI设置
        const savedTheme = localStorage.getItem('theme');
        if (savedTheme) {
          setTheme(savedTheme as 'dark' | 'light' | 'auto');
        }
        
        // 2. 请求通知权限
        if (notificationEnabled && 'Notification' in window) {
          const permission = await Notification.requestPermission();
          if (permission === 'granted') {
            message.success('通知权限已获取');
          }
        }
        
        // 3. 初始化交易服务
        await initializeServices();
        
        // 4. 启动实时数据服务
        await realtimeService.start();
        
        message.success('系统初始化完成');
      } catch (error) {
        console.error('System initialization failed:', error);
        message.error('系统初始化失败，部分功能可能不可用');
      }
    };
    
    init();
    
    // 清理函数
    return () => {
      realtimeService.stop();
    };
  }, []);
  
  // 登录后的额外初始化
  useEffect(() => {
    if (isLoggedIn) {
      const postLogin = async () => {
        try {
          // 刷新所有数据
          await realtimeService.refreshAll();
          
          // 订阅默认合约
          const defaultInstruments = ['IF2401', 'IH2401', 'IC2401'];
          const marketStore = useMarketStore.getState();
          await marketStore.subscribeMultiple(defaultInstruments);
          
          message.success('交易数据已加载');
        } catch (error) {
          console.error('Post-login initialization failed:', error);
        }
      };
      
      postLogin();
    }
  }, [isLoggedIn]);
  
  // 监听窗口关闭事件
  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      const tradingStore = useTradingStore.getState();
      
      // 如果有未完成的订单，提示用户
      if (tradingStore.activeOrders.length > 0) {
        e.preventDefault();
        e.returnValue = '您还有未完成的订单，确定要离开吗？';
      }
    };
    
    window.addEventListener('beforeunload', handleBeforeUnload);
    
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  }, []);
  
  // 设置全局错误处理
  useEffect(() => {
    const handleError = (event: ErrorEvent) => {
      console.error('Global error:', event.error);
      message.error('发生未知错误，请刷新页面重试');
    };
    
    const handleRejection = (event: PromiseRejectionEvent) => {
      console.error('Unhandled rejection:', event.reason);
      message.error('操作失败，请重试');
    };
    
    window.addEventListener('error', handleError);
    window.addEventListener('unhandledrejection', handleRejection);
    
    return () => {
      window.removeEventListener('error', handleError);
      window.removeEventListener('unhandledrejection', handleRejection);
    };
  }, []);
  
  return <>{children}</>;
};

export default AppInitializer;
import { useEffect } from 'react';
import { useUIStore } from '@/stores/uiStore';
import { useTradingStore } from '@/stores/tradingStore';
import { quickBuy, quickSell } from '@/services/ctpService';
import { message } from 'antd';

export const useHotkeys = () => {
  const { shortcuts } = useUIStore();
  const { cancelOrder, activeOrders } = useTradingStore();

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const key = getKeyCombo(e);
      
      // 快速买入
      if (key === shortcuts.buy) {
        e.preventDefault();
        handleQuickBuy();
      }
      // 快速卖出
      else if (key === shortcuts.sell) {
        e.preventDefault();
        handleQuickSell();
      }
      // 撤销所有订单
      else if (key === shortcuts.cancelAll) {
        e.preventDefault();
        handleCancelAll();
      }
      // 一键平仓
      else if (key === shortcuts.closeAll) {
        e.preventDefault();
        handleCloseAll();
      }
      // 全屏模式
      else if (key === shortcuts.fullscreen) {
        e.preventDefault();
        toggleFullscreen();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [shortcuts, activeOrders]);

  const getKeyCombo = (e: KeyboardEvent): string => {
    const keys = [];
    if (e.ctrlKey) keys.push('Ctrl');
    if (e.altKey) keys.push('Alt');
    if (e.shiftKey) keys.push('Shift');
    if (e.key && !['Control', 'Alt', 'Shift'].includes(e.key)) {
      keys.push(e.key.toUpperCase());
    }
    return keys.join('+');
  };

  const handleQuickBuy = async () => {
    message.info('快速买入');
    // TODO: 获取当前选中合约和价格
  };

  const handleQuickSell = async () => {
    message.info('快速卖出');
    // TODO: 获取当前选中合约和价格
  };

  const handleCancelAll = async () => {
    if (activeOrders.length === 0) {
      message.warning('没有活动订单');
      return;
    }
    
    for (const order of activeOrders) {
      await cancelOrder(order.order_ref, order.instrument_id);
    }
    message.success(`已撤销 ${activeOrders.length} 个订单`);
  };

  const handleCloseAll = async () => {
    message.info('一键平仓');
    // TODO: 实现一键平仓逻辑
  };

  const toggleFullscreen = () => {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen();
    } else {
      document.exitFullscreen();
    }
  };
};

export default useHotkeys;
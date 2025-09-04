import React from 'react';
import { Layout } from 'antd';
import { useUIStore } from '@stores/ui';

const { Header, Content } = Layout;

export interface TradingLayoutProps {
  children?: React.ReactNode;
}

/**
 * 主交易界面布局组件
 * 
 * 功能特性：
 * - 专业的交易软件布局
 * - 响应式设计
 * - 深色主题优化
 * - 可调整的面板布局
 */
const TradingLayout: React.FC<TradingLayoutProps> = ({ children }) => {
  const { theme } = useUIStore();

  return (
    <Layout className="min-h-screen bg-bg-primary">
      <Header className="trading-header h-12 px-4 flex items-center justify-between bg-bg-secondary border-b border-border-color">
        <div className="flex items-center space-x-4">
          <h1 className="text-lg font-semibold text-text-primary m-0">
            Inspirai Trader
          </h1>
          <div className="text-sm text-text-secondary">
            专业期货交易平台
          </div>
        </div>
        
        <div className="flex items-center space-x-2">
          <div className="text-xs text-text-muted">
            主题: {theme === 'dark' ? '深色' : '浅色'}
          </div>
        </div>
      </Header>
      
      <Content className="trading-content p-1 bg-bg-primary">
        {children || (
          <div className="grid grid-cols-12 grid-rows-12 gap-1 h-full min-h-[calc(100vh-48px)]">
            {/* 行情面板 - 左上 */}
            <div className="col-span-3 row-span-6 bg-bg-secondary border border-border-color rounded">
              <div className="p-2 border-b border-border-color">
                <h3 className="text-sm font-medium text-text-primary m-0">实时行情</h3>
              </div>
              <div className="p-2 text-text-secondary text-sm">
                行情数据将在这里显示...
              </div>
            </div>
            
            {/* 图表面板 - 中间 */}
            <div className="col-span-6 row-span-8 bg-bg-secondary border border-border-color rounded">
              <div className="p-2 border-b border-border-color">
                <h3 className="text-sm font-medium text-text-primary m-0">K线图表</h3>
              </div>
              <div className="p-2 text-text-secondary text-sm">
                K线图表将在这里显示...
              </div>
            </div>
            
            {/* 交易面板 - 右上 */}
            <div className="col-span-3 row-span-6 bg-bg-secondary border border-border-color rounded">
              <div className="p-2 border-b border-border-color">
                <h3 className="text-sm font-medium text-text-primary m-0">交易下单</h3>
              </div>
              <div className="p-2 text-text-secondary text-sm">
                交易下单界面将在这里显示...
              </div>
            </div>
            
            {/* 信息面板 - 底部 */}
            <div className="col-span-12 row-span-4 bg-bg-secondary border border-border-color rounded">
              <div className="p-2 border-b border-border-color">
                <h3 className="text-sm font-medium text-text-primary m-0">账户信息</h3>
              </div>
              <div className="p-2 text-text-secondary text-sm">
                持仓、委托、成交记录将在这里显示...
              </div>
            </div>
          </div>
        )}
      </Content>
    </Layout>
  );
};

export default TradingLayout;
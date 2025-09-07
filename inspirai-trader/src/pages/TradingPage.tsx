import React, { useState, useEffect } from 'react';
import { Layout, Tabs, Modal, Spin, message } from 'antd';
import { Responsive, WidthProvider } from 'react-grid-layout';
import MarketDataPanel from '@/components/market/MarketDataPanel';
import ChartPanel from '@/components/chart/ChartPanel';
import TradingPanel from '@/components/trading/TradingPanel';
import PositionPanel from '@/components/position/PositionPanel';
import OrderPanel from '@/components/order/OrderPanel';
import AccountPanel from '@/components/account/AccountPanel';
import TradeHistoryPanel from '@/components/trade/TradeHistoryPanel';
import SettingsPanel from '@/components/settings/SettingsPanel';
import { useMarketStore } from '@/stores/marketStore';
import { useTradingStore } from '@/stores/tradingStore';
import { useConnectionStore } from '@/stores/connectionStore';
import 'react-grid-layout/css/styles.css';
import 'react-resizable/css/styles.css';
import './TradingPage.css';

const { Content } = Layout;
const { TabPane } = Tabs;
const ResponsiveGridLayout = WidthProvider(Responsive);

export const TradingPage: React.FC = () => {
  const [selectedInstrument, setSelectedInstrument] = useState<string>('');
  const [fullscreenChart, setFullscreenChart] = useState(false);
  const [activeTab, setActiveTab] = useState('trading');
  const [layouts, setLayouts] = useState<any>({});
  
  const { connect, disconnect, isConnected, connectionStatus } = useConnectionStore();
  const { selectedInstrument: marketInstrument } = useMarketStore();
  const { initializeServices } = useTradingStore();

  // 初始化连接
  useEffect(() => {
    const initConnection = async () => {
      try {
        await connect();
        await initializeServices();
        message.success('系统初始化成功');
      } catch (error: any) {
        message.error(`系统初始化失败: ${error.message}`);
      }
    };
    
    initConnection();
    
    return () => {
      disconnect();
    };
  }, []);

  // 监听选中的合约
  useEffect(() => {
    if (marketInstrument) {
      setSelectedInstrument(marketInstrument);
    }
  }, [marketInstrument]);

  // 默认布局配置
  const defaultLayouts = {
    lg: [
      { i: 'market', x: 0, y: 0, w: 3, h: 6 },
      { i: 'chart', x: 3, y: 0, w: 6, h: 6 },
      { i: 'trading', x: 9, y: 0, w: 3, h: 6 },
      { i: 'position', x: 0, y: 6, w: 6, h: 4 },
      { i: 'order', x: 6, y: 6, w: 6, h: 4 },
      { i: 'account', x: 0, y: 10, w: 12, h: 3 },
    ],
    md: [
      { i: 'market', x: 0, y: 0, w: 4, h: 6 },
      { i: 'chart', x: 4, y: 0, w: 8, h: 6 },
      { i: 'trading', x: 0, y: 6, w: 4, h: 6 },
      { i: 'position', x: 4, y: 6, w: 8, h: 4 },
      { i: 'order', x: 0, y: 10, w: 12, h: 4 },
      { i: 'account', x: 0, y: 14, w: 12, h: 3 },
    ],
  };

  // 处理布局变化
  const handleLayoutChange = (layout: any, layouts: any) => {
    setLayouts(layouts);
    // 保存到本地存储
    localStorage.setItem('tradingLayouts', JSON.stringify(layouts));
  };

  // 加载保存的布局
  useEffect(() => {
    const savedLayouts = localStorage.getItem('tradingLayouts');
    if (savedLayouts) {
      setLayouts(JSON.parse(savedLayouts));
    } else {
      setLayouts(defaultLayouts);
    }
  }, []);

  // 处理风险预警
  const handleRiskWarning = (riskLevel: number) => {
    if (riskLevel > 80) {
      Modal.warning({
        title: '风险警告',
        content: `当前风险度已达到 ${riskLevel.toFixed(1)}%，请注意控制仓位！`,
      });
    }
  };

  // 交易界面布局
  const TradingLayout = () => (
    <ResponsiveGridLayout
      className="trading-grid-layout"
      layouts={layouts}
      onLayoutChange={handleLayoutChange}
      breakpoints={{ lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }}
      cols={{ lg: 12, md: 12, sm: 6, xs: 4, xxs: 2 }}
      rowHeight={60}
      isDraggable={true}
      isResizable={true}
      margin={[10, 10]}
      containerPadding={[10, 10]}
    >
      <div key="market">
        <MarketDataPanel onSelectInstrument={setSelectedInstrument} />
      </div>
      <div key="chart">
        <ChartPanel 
          instrumentId={selectedInstrument} 
          onFullscreen={() => setFullscreenChart(true)}
        />
      </div>
      <div key="trading">
        <TradingPanel instrumentId={selectedInstrument} />
      </div>
      <div key="position">
        <PositionPanel onSelectInstrument={setSelectedInstrument} />
      </div>
      <div key="order">
        <OrderPanel onSelectInstrument={setSelectedInstrument} />
      </div>
      <div key="account">
        <AccountPanel onRiskWarning={handleRiskWarning} />
      </div>
    </ResponsiveGridLayout>
  );

  // 如果未连接，显示加载状态
  if (!isConnected) {
    return (
      <div className="loading-container">
        <Spin size="large" tip={`正在连接... ${connectionStatus}`} />
      </div>
    );
  }

  return (
    <Layout className="trading-page">
      <Content>
        <Tabs 
          activeKey={activeTab} 
          onChange={setActiveTab}
          className="trading-tabs"
        >
          <TabPane tab="交易" key="trading">
            <TradingLayout />
          </TabPane>
          
          <TabPane tab="历史记录" key="history">
            <TradeHistoryPanel onSelectInstrument={setSelectedInstrument} />
          </TabPane>
          
          <TabPane tab="系统设置" key="settings">
            <SettingsPanel />
          </TabPane>
        </Tabs>
      </Content>

      {/* 全屏图表模态框 */}
      <Modal
        visible={fullscreenChart}
        onCancel={() => setFullscreenChart(false)}
        width="90%"
        style={{ top: 20 }}
        bodyStyle={{ height: 'calc(100vh - 100px)' }}
        footer={null}
      >
        <ChartPanel 
          instrumentId={selectedInstrument} 
          height={window.innerHeight - 150}
        />
      </Modal>
    </Layout>
  );
};

export default TradingPage;
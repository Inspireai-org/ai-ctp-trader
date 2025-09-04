import React, { useState, useCallback, useEffect, useRef } from 'react';
import { Layout, Button, Dropdown, Space, message } from 'antd';
import type { MenuProps } from 'antd';
import {
  MenuOutlined,
  LockOutlined,
  UnlockOutlined,
  LayoutOutlined,
  FullscreenOutlined,
  FullscreenExitOutlined,
  SettingOutlined,
  EyeOutlined,
} from '@ant-design/icons';
import GridLayout, { Layout as GridLayoutItem } from 'react-grid-layout';
import { useUIStore, useLayout } from '@stores/ui';
import { useResponsive, getResponsiveColumns, getResponsiveRowHeight } from '@hooks/useResponsive';
import PanelVisibilityManager from './PanelVisibilityManager';
import { MarketDataPanel } from '@components/market';
import 'react-grid-layout/css/styles.css';
import 'react-resizable/css/styles.css';

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
 * - 拖拽和调整大小支持
 * - 布局锁定功能
 * - 预设布局模板
 */
const TradingLayout: React.FC<TradingLayoutProps> = ({ children }) => {
  const { theme, setFullscreenPanel, fullscreenPanel } = useUIStore();
  const { layout, isLayoutLocked, updateLayout, toggleLayoutLock } = useLayout();
  const [currentLayout, setCurrentLayout] = useState<GridLayoutItem[]>([]);
  const [showPanelManager, setShowPanelManager] = useState(false);
  const responsive = useResponsive();
  const isUpdatingRef = useRef(false);

  // 将布局配置转换为 GridLayout 格式
  useEffect(() => {
    const gridItems: GridLayoutItem[] = Object.entries(layout.panels).map(([key, panel]) => ({
      i: key,
      x: panel.position.x,
      y: panel.position.y,
      w: panel.size.width,
      h: panel.size.height,
      minW: panel.minSize.width,
      minH: panel.minSize.height,
      isDraggable: !isLayoutLocked && panel.draggable,
      isResizable: !isLayoutLocked && panel.resizable,
      static: isLayoutLocked,
    }));
    setCurrentLayout(gridItems);
  }, [layout, isLayoutLocked]);


  // 布局变化处理
  const handleLayoutChange = useCallback((newLayout: GridLayoutItem[]) => {
    if (!isLayoutLocked && !isUpdatingRef.current) {
      // 检查布局是否真的改变了
      const hasChanged = newLayout.some(item => {
        const key = item.i as keyof typeof layout.panels;
        const panel = layout.panels[key];
        if (!panel) return false;
        
        return panel.position.x !== item.x || 
               panel.position.y !== item.y ||
               panel.size.width !== item.w ||
               panel.size.height !== item.h;
      });

      if (hasChanged) {
        isUpdatingRef.current = true;
        const updatedPanels = { ...layout.panels };
        
        newLayout.forEach(item => {
          const key = item.i as keyof typeof layout.panels;
          if (updatedPanels[key]) {
            updatedPanels[key] = {
              ...updatedPanels[key],
              position: { x: item.x, y: item.y },
              size: { width: item.w, height: item.h },
            };
          }
        });
        
        updateLayout({ ...layout, panels: updatedPanels });
        
        // 使用 setTimeout 重置标志，避免快速连续的更新
        setTimeout(() => {
          isUpdatingRef.current = false;
        }, 100);
      }
    }
  }, [isLayoutLocked, layout, updateLayout]);

  // 预设布局模板
  const layoutTemplates = [
    {
      key: 'default',
      label: '默认布局',
      layout: {
        market: { x: 0, y: 0, width: 3, height: 6 },
        chart: { x: 3, y: 0, width: 6, height: 8 },
        trading: { x: 9, y: 0, width: 3, height: 6 },
        info: { x: 0, y: 8, width: 12, height: 4 },
      },
    },
    {
      key: 'compact',
      label: '紧凑布局',
      layout: {
        market: { x: 0, y: 0, width: 2, height: 6 },
        chart: { x: 2, y: 0, width: 7, height: 7 },
        trading: { x: 9, y: 0, width: 3, height: 7 },
        info: { x: 0, y: 7, width: 12, height: 5 },
      },
    },
    {
      key: 'focus',
      label: '专注模式',
      layout: {
        market: { x: 0, y: 0, width: 3, height: 5 },
        chart: { x: 3, y: 0, width: 9, height: 9 },
        trading: { x: 0, y: 5, width: 3, height: 4 },
        info: { x: 0, y: 9, width: 12, height: 3 },
      },
    },
  ];

  // 应用预设布局
  const applyLayoutTemplate = useCallback((template: typeof layoutTemplates[0]) => {
    isUpdatingRef.current = true;
    const updatedPanels = { ...layout.panels };
    Object.entries(template.layout).forEach(([key, value]) => {
      const panelKey = key as keyof typeof layout.panels;
      if (updatedPanels[panelKey]) {
        updatedPanels[panelKey] = {
          ...updatedPanels[panelKey],
          position: { x: value.x, y: value.y },
          size: { width: value.width, height: value.height },
        };
      }
    });
    updateLayout({ ...layout, panels: updatedPanels });
    message.success(`已切换到${template.label}`);
    
    setTimeout(() => {
      isUpdatingRef.current = false;
    }, 200);
  }, [layout, updateLayout]);

  // 面板全屏切换
  const toggleFullscreen = (panelId: string) => {
    setFullscreenPanel(fullscreenPanel === panelId ? null : panelId);
  };


  // 布局菜单项
  const layoutMenuItems: MenuProps['items'] = [
    ...layoutTemplates.map(template => ({
      key: template.key,
      label: template.label,
      onClick: () => applyLayoutTemplate(template),
    })),
    { type: 'divider' as const },
    {
      key: 'panels',
      label: '管理面板',
      icon: <EyeOutlined />,
      onClick: () => setShowPanelManager(true),
    },
  ];

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
        
        <Space>
          <Dropdown menu={{ items: layoutMenuItems }}>
            <Button
              type="text"
              icon={<LayoutOutlined />}
              size="small"
            >
              布局
            </Button>
          </Dropdown>
          
          <Button
            type="text"
            icon={isLayoutLocked ? <LockOutlined /> : <UnlockOutlined />}
            onClick={toggleLayoutLock}
            size="small"
          >
            {isLayoutLocked ? '解锁' : '锁定'}
          </Button>
          
          <Button
            type="text"
            icon={<SettingOutlined />}
            size="small"
          >
            设置
          </Button>
          
          <div className="text-xs text-text-muted">
            主题: {theme === 'dark' ? '深色' : '浅色'}
          </div>
        </Space>
      </Header>
      
      <Content className="trading-content p-1 bg-bg-primary overflow-hidden">
        {children || (
          <GridLayout
            className="layout"
            layout={currentLayout}
            cols={getResponsiveColumns(responsive.deviceType)}
            rowHeight={getResponsiveRowHeight(responsive.deviceType)}
            width={responsive.width - 8}
            onDragStop={handleLayoutChange}
            onResizeStop={handleLayoutChange}
            isDraggable={!isLayoutLocked}
            isResizable={!isLayoutLocked}
            margin={[4, 4]}
            containerPadding={[0, 0]}
            compactType="vertical"
            preventCollision={false}
          >
            {/* 行情面板 */}
            <div key="market" className={`bg-bg-secondary border border-border-color rounded ${fullscreenPanel === 'market' ? 'fullscreen-panel' : ''}`}>
              <PanelHeader
                title="实时行情"
                panelId="market"
                onFullscreen={() => toggleFullscreen('market')}
                isFullscreen={fullscreenPanel === 'market'}
              />
              <div className="h-[calc(100%-36px)] overflow-hidden">
                <MarketDataPanel />
              </div>
            </div>
            
            {/* 图表面板 */}
            <div key="chart" className={`bg-bg-secondary border border-border-color rounded ${fullscreenPanel === 'chart' ? 'fullscreen-panel' : ''}`}>
              <PanelHeader
                title="K线图表"
                panelId="chart"
                onFullscreen={() => toggleFullscreen('chart')}
                isFullscreen={fullscreenPanel === 'chart'}
              />
              <div className="p-2 text-text-secondary text-sm overflow-auto h-[calc(100%-36px)]">
                K线图表将在这里显示...
              </div>
            </div>
            
            {/* 交易面板 */}
            <div key="trading" className={`bg-bg-secondary border border-border-color rounded ${fullscreenPanel === 'trading' ? 'fullscreen-panel' : ''}`}>
              <PanelHeader
                title="交易下单"
                panelId="trading"
                onFullscreen={() => toggleFullscreen('trading')}
                isFullscreen={fullscreenPanel === 'trading'}
              />
              <div className="p-2 text-text-secondary text-sm overflow-auto h-[calc(100%-36px)]">
                交易下单界面将在这里显示...
              </div>
            </div>
            
            {/* 信息面板 */}
            <div key="info" className={`bg-bg-secondary border border-border-color rounded ${fullscreenPanel === 'info' ? 'fullscreen-panel' : ''}`}>
              <PanelHeader
                title="账户信息"
                panelId="info"
                onFullscreen={() => toggleFullscreen('info')}
                isFullscreen={fullscreenPanel === 'info'}
              />
              <div className="p-2 text-text-secondary text-sm overflow-auto h-[calc(100%-36px)]">
                持仓、委托、成交记录将在这里显示...
              </div>
            </div>
          </GridLayout>
        )}
      </Content>
      
      {/* 面板管理器 */}
      <PanelVisibilityManager
        visible={showPanelManager}
        onClose={() => setShowPanelManager(false)}
      />
    </Layout>
  );
};

/**
 * 面板头部组件
 */
interface PanelHeaderProps {
  title: string;
  panelId: string;
  onFullscreen: () => void;
  isFullscreen: boolean;
}

const PanelHeader: React.FC<PanelHeaderProps> = ({ title, panelId, onFullscreen, isFullscreen }) => {
  const { layout, updateLayout } = useLayout();
  
  const handleClose = () => {
    const updatedPanels = { ...layout.panels };
    const key = panelId as keyof typeof layout.panels;
    if (updatedPanels[key]) {
      updatedPanels[key] = {
        ...updatedPanels[key],
        visible: false,
      };
      updateLayout({ ...layout, panels: updatedPanels });
    }
  };

  const panelMenuItems = [
    {
      key: 'fullscreen',
      label: isFullscreen ? '退出全屏' : '全屏',
      icon: isFullscreen ? <FullscreenExitOutlined /> : <FullscreenOutlined />,
      onClick: onFullscreen,
    },
    {
      key: 'close',
      label: '隐藏面板',
      onClick: handleClose,
    },
  ];

  return (
    <div className="flex items-center justify-between p-2 border-b border-border-color bg-bg-tertiary rounded-t">
      <h3 className="text-sm font-medium text-text-primary m-0">{title}</h3>
      <Dropdown menu={{ items: panelMenuItems }} trigger={['click']}>
        <Button
          type="text"
          icon={<MenuOutlined />}
          size="small"
          className="text-text-secondary hover:text-text-primary"
        />
      </Dropdown>
    </div>
  );
};

export default TradingLayout;
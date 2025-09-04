import React from 'react';
import { Card, Row, Col, Typography, message } from 'antd';
import {
  AppstoreOutlined,
  DesktopOutlined,
  MobileOutlined,
  TabletOutlined,
} from '@ant-design/icons';
import { useLayout } from '@stores/ui';
import { LayoutConfig } from '@stores/ui';

const { Title, Text } = Typography;

/**
 * 预设布局配置
 */
export const layoutPresets = {
  default: {
    name: '默认布局',
    description: '标准四象限布局，适合大屏幕',
    icon: <DesktopOutlined />,
    config: {
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
    },
  },
  compact: {
    name: '紧凑布局',
    description: '节省空间的布局，适合中等屏幕',
    icon: <TabletOutlined />,
    config: {
      panels: {
        market: { 
          visible: true, 
          position: { x: 0, y: 0 }, 
          size: { width: 2, height: 6 },
          minSize: { width: 2, height: 4 },
          resizable: true,
          draggable: true,
          collapsed: false,
        },
        chart: { 
          visible: true, 
          position: { x: 2, y: 0 }, 
          size: { width: 7, height: 7 },
          minSize: { width: 4, height: 5 },
          resizable: true,
          draggable: true,
          collapsed: false,
        },
        trading: { 
          visible: true, 
          position: { x: 9, y: 0 }, 
          size: { width: 3, height: 7 },
          minSize: { width: 2, height: 4 },
          resizable: true,
          draggable: true,
          collapsed: false,
        },
        info: { 
          visible: true, 
          position: { x: 0, y: 7 }, 
          size: { width: 12, height: 5 },
          minSize: { width: 6, height: 3 },
          resizable: true,
          draggable: true,
          collapsed: false,
        },
      },
      gridCols: 12,
      gridRows: 12,
    },
  },
  focus: {
    name: '专注模式',
    description: '突出图表区域，适合技术分析',
    icon: <AppstoreOutlined />,
    config: {
      panels: {
        market: { 
          visible: true, 
          position: { x: 0, y: 0 }, 
          size: { width: 3, height: 5 },
          minSize: { width: 2, height: 4 },
          resizable: true,
          draggable: true,
          collapsed: false,
        },
        chart: { 
          visible: true, 
          position: { x: 3, y: 0 }, 
          size: { width: 9, height: 9 },
          minSize: { width: 6, height: 6 },
          resizable: true,
          draggable: true,
          collapsed: false,
        },
        trading: { 
          visible: true, 
          position: { x: 0, y: 5 }, 
          size: { width: 3, height: 4 },
          minSize: { width: 2, height: 3 },
          resizable: true,
          draggable: true,
          collapsed: false,
        },
        info: { 
          visible: true, 
          position: { x: 0, y: 9 }, 
          size: { width: 12, height: 3 },
          minSize: { width: 6, height: 2 },
          resizable: true,
          draggable: true,
          collapsed: false,
        },
      },
      gridCols: 12,
      gridRows: 12,
    },
  },
  mobile: {
    name: '移动布局',
    description: '适配小屏幕的垂直布局',
    icon: <MobileOutlined />,
    config: {
      panels: {
        market: { 
          visible: true, 
          position: { x: 0, y: 0 }, 
          size: { width: 6, height: 3 },
          minSize: { width: 6, height: 3 },
          resizable: false,
          draggable: false,
          collapsed: false,
        },
        chart: { 
          visible: true, 
          position: { x: 0, y: 3 }, 
          size: { width: 6, height: 4 },
          minSize: { width: 6, height: 4 },
          resizable: false,
          draggable: false,
          collapsed: false,
        },
        trading: { 
          visible: true, 
          position: { x: 0, y: 7 }, 
          size: { width: 6, height: 3 },
          minSize: { width: 6, height: 3 },
          resizable: false,
          draggable: false,
          collapsed: false,
        },
        info: { 
          visible: true, 
          position: { x: 0, y: 10 }, 
          size: { width: 6, height: 2 },
          minSize: { width: 6, height: 2 },
          resizable: false,
          draggable: false,
          collapsed: false,
        },
      },
      gridCols: 6,
      gridRows: 12,
    },
  },
};

interface LayoutPresetsProps {
  onSelect?: (preset: LayoutConfig) => void;
}

/**
 * 布局预设选择器组件
 */
const LayoutPresets: React.FC<LayoutPresetsProps> = ({ onSelect }) => {
  const { updateLayout } = useLayout();

  const handleSelectPreset = (presetKey: keyof typeof layoutPresets) => {
    const preset = layoutPresets[presetKey];
    updateLayout(preset.config as LayoutConfig);
    if (onSelect) {
      onSelect(preset.config as LayoutConfig);
    }
    message.success(`已切换到${preset.name}`);
  };

  return (
    <div className="p-4">
      <Title level={4}>选择布局模板</Title>
      <Row gutter={[16, 16]}>
        {Object.entries(layoutPresets).map(([key, preset]) => (
          <Col key={key} xs={24} sm={12} md={6}>
            <Card
              hoverable
              className="h-full cursor-pointer"
              onClick={() => handleSelectPreset(key as keyof typeof layoutPresets)}
            >
              <div className="text-center">
                <div className="text-3xl mb-2 text-active-color">
                  {preset.icon}
                </div>
                <Title level={5} className="mb-1">
                  {preset.name}
                </Title>
                <Text type="secondary" className="text-xs">
                  {preset.description}
                </Text>
              </div>
            </Card>
          </Col>
        ))}
      </Row>
    </div>
  );
};

export default LayoutPresets;
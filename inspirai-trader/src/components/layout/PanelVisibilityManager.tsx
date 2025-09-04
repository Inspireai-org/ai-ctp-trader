import React from 'react';
import { Modal, Switch, Space, Typography } from 'antd';
import { useLayout } from '@stores/ui';

const { Text } = Typography;

interface PanelVisibilityManagerProps {
  visible: boolean;
  onClose: () => void;
}

/**
 * 面板可见性管理器
 */
const PanelVisibilityManager: React.FC<PanelVisibilityManagerProps> = ({ visible, onClose }) => {
  const { layout, updateLayout } = useLayout();

  const panelInfo = {
    market: { title: '实时行情', description: '显示合约的实时价格和行情数据' },
    chart: { title: 'K线图表', description: '显示价格走势图表和技术指标' },
    trading: { title: '交易下单', description: '快速下单和交易操作面板' },
    info: { title: '账户信息', description: '持仓、委托、成交记录等信息' },
  };

  const handleToggle = (panelKey: keyof typeof layout.panels) => {
    const updatedPanels = { ...layout.panels };
    updatedPanels[panelKey] = {
      ...updatedPanels[panelKey],
      visible: !updatedPanels[panelKey].visible,
    };
    updateLayout({ ...layout, panels: updatedPanels });
  };

  return (
    <Modal
      title="面板显示管理"
      open={visible}
      onCancel={onClose}
      footer={null}
      width={500}
    >
      <Space direction="vertical" className="w-full">
        {Object.entries(layout.panels).map(([key, panel]) => {
          const info = panelInfo[key as keyof typeof panelInfo];
          return (
            <div
              key={key}
              className="flex items-center justify-between p-3 bg-bg-secondary rounded border border-border-color"
            >
              <div className="flex-1">
                <Text strong className="text-text-primary">
                  {info.title}
                </Text>
                <div className="text-xs text-text-muted mt-1">
                  {info.description}
                </div>
              </div>
              <Switch
                checked={panel.visible}
                onChange={() => handleToggle(key as keyof typeof layout.panels)}
              />
            </div>
          );
        })}
      </Space>
    </Modal>
  );
};

export default PanelVisibilityManager;
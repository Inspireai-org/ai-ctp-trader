import React from 'react';
import { Card, Button, Alert, Space, Typography, Tag } from 'antd';
import { ThunderboltOutlined, EnvironmentOutlined, ClockCircleOutlined } from '@ant-design/icons';
import { isWeekend, getRecommendedPreset, getTtsPresets } from '@/config/ctp-presets';

const { Text, Paragraph } = Typography;

interface QuickConnectPanelProps {
  onQuickConnect: (presetKey: string) => void;
  loading?: boolean;
}

const QuickConnectPanel: React.FC<QuickConnectPanelProps> = ({
  onQuickConnect,
  loading = false
}) => {
  const weekendMode = isWeekend();
  const recommendedPreset = getRecommendedPreset();
  const ttsPresets = getTtsPresets();

  const handleQuickConnect = () => {
    onQuickConnect(recommendedPreset.key);
  };

  return (
    <Card 
      size="small" 
      title={
        <Space>
          <ThunderboltOutlined />
          <span>快速连接</span>
        </Space>
      }
      style={{ marginBottom: 16 }}
    >
      {weekendMode && (
        <Alert
          message="周末开发模式"
          description="当前为周末时间，已自动推荐 TTS 测试环境，支持完整的交易功能测试。"
          type="info"
          showIcon
          icon={<ClockCircleOutlined />}
          style={{ marginBottom: 16 }}
        />
      )}

      <div style={{ textAlign: 'center' }}>
        <Button
          type="primary"
          size="large"
          block
          loading={loading}
          onClick={handleQuickConnect}
          icon={<EnvironmentOutlined />}
        >
          连接到 {recommendedPreset.label}
        </Button>

        <div style={{ marginTop: 12 }}>
          <Space direction="vertical" size="small" style={{ width: '100%' }}>
            <div>
              <Text type="secondary">推荐环境特性：</Text>
              <div style={{ marginTop: 4 }}>
                <Space wrap>
                  {recommendedPreset.features?.map(feature => (
                    <Tag key={feature} color="blue" size="small">
                      {feature}
                    </Tag>
                  ))}
                </Space>
              </div>
            </div>

            {recommendedPreset.defaultInvestorId && recommendedPreset.defaultPassword && (
              <Text type="secondary" style={{ fontSize: '12px' }}>
                将使用默认测试账号自动登录
              </Text>
            )}
          </Space>
        </div>
      </div>

      {weekendMode && ttsPresets.length > 1 && (
        <div style={{ marginTop: 16, paddingTop: 16, borderTop: '1px solid #f0f0f0' }}>
          <Text strong style={{ fontSize: '12px' }}>其他 TTS 环境：</Text>
          <div style={{ marginTop: 8 }}>
            <Space wrap>
              {ttsPresets
                .filter(preset => preset.key !== recommendedPreset.key)
                .map(preset => (
                  <Button
                    key={preset.key}
                    size="small"
                    onClick={() => onQuickConnect(preset.key)}
                    loading={loading}
                  >
                    {preset.label}
                  </Button>
                ))}
            </Space>
          </div>
        </div>
      )}
    </Card>
  );
};

export default QuickConnectPanel;
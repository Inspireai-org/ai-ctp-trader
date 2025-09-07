import React, { useEffect, useState } from 'react';
import {
  Card,
  Row,
  Col,
  Statistic,
  Progress,
  Alert,
  Table,
  Tag,
  Space,
  Button,
  Tooltip,
  Divider
} from 'antd';
import {
  WalletOutlined,
  ArrowUpOutlined,
  ArrowDownOutlined,
  WarningOutlined,
  SafetyOutlined,
  FundOutlined,
  LineChartOutlined,
  ReloadOutlined
} from '@ant-design/icons';
import { Line } from '@ant-design/charts';
import { AccountInfo } from '@/types';
import { useTradingStore } from '@/stores/tradingStore';
import { formatPrice, formatPercent } from '@/utils/format';
import './AccountPanel.css';

interface AccountPanelProps {
  onRiskWarning?: (riskLevel: number) => void;
}

export const AccountPanel: React.FC<AccountPanelProps> = ({ onRiskWarning }) => {
  const { account, loading, refreshAccount, getAccountHistory } = useTradingStore();
  const [historyData, setHistoryData] = useState<any[]>([]);
  const [riskLevel, setRiskLevel] = useState<number>(0);
  const [showRiskAlert, setShowRiskAlert] = useState(false);

  // 定期刷新账户数据
  useEffect(() => {
    refreshAccount();
    const interval = setInterval(refreshAccount, 5000); // 每5秒刷新一次
    return () => clearInterval(interval);
  }, [refreshAccount]);

  // 获取历史数据
  useEffect(() => {
    const history = getAccountHistory();
    const chartData = history.map((item, index) => ({
      time: new Date(Date.now() - (history.length - index) * 60000).toLocaleTimeString(),
      value: item.balance,
      type: '账户余额',
    }));
    setHistoryData(chartData);
  }, [account, getAccountHistory]);

  // 计算风险度
  useEffect(() => {
    if (!account) return;
    
    const risk = calculateRiskLevel(account);
    setRiskLevel(risk);
    
    // 风险预警
    if (risk > 80) {
      setShowRiskAlert(true);
      onRiskWarning?.(risk);
    } else {
      setShowRiskAlert(false);
    }
  }, [account, onRiskWarning]);

  // 计算风险度
  const calculateRiskLevel = (acc: AccountInfo): number => {
    if (acc.balance === 0) return 0;
    const usedMargin = acc.margin;
    const totalAssets = acc.balance;
    return Math.min((usedMargin / totalAssets) * 100, 100);
  };

  // 计算可用资金比例
  const getAvailableRatio = (): number => {
    if (!account || account.balance === 0) return 0;
    return (account.available / account.balance) * 100;
  };

  // 获取风险等级
  const getRiskStatus = (): { level: string; color: string; icon: React.ReactNode } => {
    if (riskLevel < 30) {
      return { level: '安全', color: 'success', icon: <SafetyOutlined /> };
    } else if (riskLevel < 60) {
      return { level: '正常', color: 'warning', icon: <SafetyOutlined /> };
    } else if (riskLevel < 80) {
      return { level: '警戒', color: 'orange', icon: <WarningOutlined /> };
    } else {
      return { level: '危险', color: 'error', icon: <WarningOutlined /> };
    }
  };

  const riskStatus = getRiskStatus();

  // 资金变动图表配置
  const chartConfig = {
    data: historyData,
    xField: 'time',
    yField: 'value',
    seriesField: 'type',
    smooth: true,
    animation: {
      appear: {
        animation: 'path-in',
        duration: 1000,
      },
    },
    xAxis: {
      label: {
        style: {
          fill: '#999',
        },
      },
      line: {
        style: {
          stroke: '#2b2b2b',
        },
      },
    },
    yAxis: {
      label: {
        style: {
          fill: '#999',
        },
        formatter: (v: string) => formatPrice(Number(v)),
      },
      grid: {
        line: {
          style: {
            stroke: '#2b2b2b',
            lineDash: [2, 2],
          },
        },
      },
    },
    theme: {
      background: 'transparent',
    },
    color: ['#1890ff'],
    point: {
      size: 3,
      shape: 'circle',
    },
    tooltip: {
      formatter: (datum: any) => {
        return {
          name: datum.type,
          value: formatPrice(datum.value),
        };
      },
    },
  };

  // 资金明细表格
  const fundDetails = [
    { label: '账户余额', value: account?.balance || 0, key: 'balance' },
    { label: '可用资金', value: account?.available || 0, key: 'available' },
    { label: '占用保证金', value: account?.margin || 0, key: 'margin' },
    { label: '冻结保证金', value: account?.frozenMargin || 0, key: 'frozenMargin' },
    { label: '手续费', value: account?.commission || 0, key: 'commission' },
    { label: '平仓盈亏', value: account?.closeProfit || 0, key: 'closeProfit' },
    { label: '持仓盈亏', value: account?.positionProfit || 0, key: 'positionProfit' },
  ];

  if (!account) {
    return (
      <Card className="account-panel" loading={loading}>
        <div style={{ textAlign: 'center', padding: '40px' }}>
          <WalletOutlined style={{ fontSize: 48, color: '#999' }} />
          <p style={{ marginTop: 16, color: '#999' }}>暂无账户数据</p>
        </div>
      </Card>
    );
  }

  return (
    <div className="account-panel">
      {/* 风险警告 */}
      {showRiskAlert && (
        <Alert
          message="风险警告"
          description={`当前风险度 ${riskLevel.toFixed(1)}%，请注意控制仓位！`}
          type="error"
          showIcon
          closable
          style={{ marginBottom: 16 }}
        />
      )}

      <Row gutter={16}>
        {/* 账户概览 */}
        <Col span={12}>
          <Card 
            title={
              <Space>
                <WalletOutlined />
                <span>账户资金</span>
              </Space>
            }
            extra={
              <Button 
                icon={<ReloadOutlined />} 
                size="small"
                onClick={refreshAccount}
                loading={loading}
              >
                刷新
              </Button>
            }
          >
            <Row gutter={[16, 16]}>
              <Col span={12}>
                <Statistic
                  title="账户余额"
                  value={account.balance}
                  precision={2}
                  prefix="¥"
                  valueStyle={{ color: '#1890ff' }}
                />
              </Col>
              <Col span={12}>
                <Statistic
                  title="可用资金"
                  value={account.available}
                  precision={2}
                  prefix="¥"
                  valueStyle={{ color: '#52c41a' }}
                />
              </Col>
              <Col span={12}>
                <Statistic
                  title="占用保证金"
                  value={account.margin}
                  precision={2}
                  prefix="¥"
                />
              </Col>
              <Col span={12}>
                <Statistic
                  title="冻结保证金"
                  value={account.frozenMargin}
                  precision={2}
                  prefix="¥"
                />
              </Col>
            </Row>

            <Divider />

            {/* 盈亏统计 */}
            <Row gutter={[16, 16]}>
              <Col span={12}>
                <Statistic
                  title="平仓盈亏"
                  value={account.closeProfit}
                  precision={2}
                  prefix="¥"
                  valueStyle={{ 
                    color: account.closeProfit >= 0 ? '#f5222d' : '#52c41a' 
                  }}
                  prefix={account.closeProfit >= 0 ? <ArrowUpOutlined /> : <ArrowDownOutlined />}
                />
              </Col>
              <Col span={12}>
                <Statistic
                  title="持仓盈亏"
                  value={account.positionProfit}
                  precision={2}
                  prefix="¥"
                  valueStyle={{ 
                    color: account.positionProfit >= 0 ? '#f5222d' : '#52c41a' 
                  }}
                  prefix={account.positionProfit >= 0 ? <ArrowUpOutlined /> : <ArrowDownOutlined />}
                />
              </Col>
            </Row>
          </Card>
        </Col>

        {/* 风险指标 */}
        <Col span={12}>
          <Card 
            title={
              <Space>
                <FundOutlined />
                <span>风险监控</span>
              </Space>
            }
          >
            {/* 风险度 */}
            <div className="risk-indicator">
              <div className="risk-header">
                <span>风险度</span>
                <Tag color={riskStatus.color} icon={riskStatus.icon}>
                  {riskStatus.level}
                </Tag>
              </div>
              <Progress 
                percent={riskLevel} 
                strokeColor={
                  riskLevel < 30 ? '#52c41a' :
                  riskLevel < 60 ? '#faad14' :
                  riskLevel < 80 ? '#ff9800' : '#f5222d'
                }
                format={percent => `${percent?.toFixed(1)}%`}
              />
            </div>

            <Divider />

            {/* 资金使用率 */}
            <div className="risk-indicator">
              <div className="risk-header">
                <span>资金使用率</span>
                <Tooltip title="已用保证金 / 账户余额">
                  <WarningOutlined style={{ color: '#999', marginLeft: 8 }} />
                </Tooltip>
              </div>
              <Progress 
                percent={100 - getAvailableRatio()} 
                strokeColor="#1890ff"
                format={percent => `${percent?.toFixed(1)}%`}
              />
            </div>

            <Divider />

            {/* 可用资金比例 */}
            <div className="risk-indicator">
              <div className="risk-header">
                <span>可用资金比例</span>
              </div>
              <Progress 
                percent={getAvailableRatio()} 
                strokeColor="#52c41a"
                format={percent => `${percent?.toFixed(1)}%`}
              />
            </div>

            <Divider />

            {/* 风险提示 */}
            <div className="risk-tips">
              <Space direction="vertical" size="small">
                <span className="tip-item">
                  <SafetyOutlined style={{ marginRight: 8, color: '#52c41a' }} />
                  风险度 &lt; 30%：仓位安全
                </span>
                <span className="tip-item">
                  <WarningOutlined style={{ marginRight: 8, color: '#faad14' }} />
                  风险度 30%-60%：正常水平
                </span>
                <span className="tip-item">
                  <WarningOutlined style={{ marginRight: 8, color: '#ff9800' }} />
                  风险度 60%-80%：注意控制
                </span>
                <span className="tip-item">
                  <WarningOutlined style={{ marginRight: 8, color: '#f5222d' }} />
                  风险度 &gt; 80%：高度警戒
                </span>
              </Space>
            </div>
          </Card>
        </Col>
      </Row>

      {/* 资金变动图表 */}
      <Card 
        title={
          <Space>
            <LineChartOutlined />
            <span>资金变动趋势</span>
          </Space>
        }
        style={{ marginTop: 16 }}
      >
        <div style={{ height: 300 }}>
          {historyData.length > 0 ? (
            <Line {...chartConfig} />
          ) : (
            <div style={{ textAlign: 'center', padding: '50px', color: '#999' }}>
              暂无历史数据
            </div>
          )}
        </div>
      </Card>

      {/* 资金明细 */}
      <Card 
        title={
          <Space>
            <FundOutlined />
            <span>资金明细</span>
          </Space>
        }
        style={{ marginTop: 16 }}
      >
        <Row gutter={[16, 16]}>
          {fundDetails.map(item => (
            <Col span={6} key={item.key}>
              <div className="fund-detail-item">
                <span className="label">{item.label}</span>
                <span className={`value ${item.value < 0 ? 'negative' : ''}`}>
                  {formatPrice(item.value)}
                </span>
              </div>
            </Col>
          ))}
        </Row>
      </Card>
    </div>
  );
};

export default AccountPanel;
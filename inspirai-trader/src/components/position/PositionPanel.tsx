import React, { useEffect, useState } from 'react';
import { 
  Card, 
  Table, 
  Tag, 
  Button, 
  Space, 
  Popconfirm,
  Modal,
  InputNumber,
  message,
  Tooltip,
  Progress
} from 'antd';
import { 
  CloseOutlined, 
  PlusOutlined, 
  WarningOutlined,
  LineChartOutlined,
  DollarOutlined
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { Position, OrderDirection, OffsetFlag } from '@/types';
import { useTradingStore } from '@/stores/tradingStore';
import { useMarketStore } from '@/stores/marketStore';
import { formatNumber, formatPrice, formatPercent } from '@/utils/format';
import './PositionPanel.css';

interface PositionPanelProps {
  onSelectInstrument?: (instrumentId: string) => void;
}

export const PositionPanel: React.FC<PositionPanelProps> = ({ onSelectInstrument }) => {
  const { positions, closePosition, loading, refreshPositions } = useTradingStore();
  const { getMarketData } = useMarketStore();
  const [selectedPosition, setSelectedPosition] = useState<Position | null>(null);
  const [closeModalVisible, setCloseModalVisible] = useState(false);
  const [closeVolume, setCloseVolume] = useState<number>(0);
  const [stopLossModalVisible, setStopLossModalVisible] = useState(false);
  const [stopLossPrice, setStopLossPrice] = useState<number>(0);
  const [takeProfitPrice, setTakeProfitPrice] = useState<number>(0);

  // 定期刷新持仓数据
  useEffect(() => {
    refreshPositions();
    const interval = setInterval(refreshPositions, 5000); // 每5秒刷新一次
    return () => clearInterval(interval);
  }, [refreshPositions]);

  // 计算浮动盈亏
  const calculateFloatingPnL = (position: Position): number => {
    const marketData = getMarketData(position.instrumentId);
    if (!marketData) return 0;
    
    const currentPrice = position.direction === 'Long' 
      ? marketData.bidPrice1 
      : marketData.askPrice1;
    
    const pnl = position.direction === 'Long'
      ? (currentPrice - position.avgPrice) * position.volume
      : (position.avgPrice - currentPrice) * position.volume;
    
    return pnl;
  };

  // 计算盈亏百分比
  const calculatePnLPercent = (position: Position): number => {
    const pnl = calculateFloatingPnL(position);
    const cost = position.avgPrice * position.volume;
    return cost > 0 ? (pnl / cost) * 100 : 0;
  };

  // 计算风险度
  const calculateRiskLevel = (position: Position): number => {
    const pnlPercent = calculatePnLPercent(position);
    // 简单的风险度计算：亏损超过10%为高风险
    if (pnlPercent < -10) return 90;
    if (pnlPercent < -5) return 60;
    if (pnlPercent < 0) return 30;
    return 10;
  };

  // 处理平仓
  const handleClosePosition = async () => {
    if (!selectedPosition || closeVolume <= 0) return;
    
    try {
      await closePosition(selectedPosition.instrumentId, closeVolume);
      message.success('平仓指令已发送');
      setCloseModalVisible(false);
      setSelectedPosition(null);
    } catch (error: any) {
      message.error(`平仓失败: ${error.message}`);
    }
  };

  // 处理设置止损止盈
  const handleSetStopLoss = async () => {
    if (!selectedPosition) return;
    
    // TODO: 实现止损止盈设置
    message.info('止损止盈功能开发中');
    setStopLossModalVisible(false);
  };

  const columns: ColumnsType<Position> = [
    {
      title: '合约',
      dataIndex: 'instrumentId',
      key: 'instrumentId',
      fixed: 'left',
      render: (text: string) => (
        <Button 
          type="link" 
          size="small"
          onClick={() => onSelectInstrument?.(text)}
        >
          {text}
        </Button>
      ),
    },
    {
      title: '方向',
      dataIndex: 'direction',
      key: 'direction',
      width: 80,
      render: (direction: string) => (
        <Tag color={direction === 'Long' ? 'red' : 'green'}>
          {direction === 'Long' ? '多' : '空'}
        </Tag>
      ),
    },
    {
      title: '持仓量',
      dataIndex: 'volume',
      key: 'volume',
      width: 100,
      align: 'right',
      render: (volume: number) => formatNumber(volume),
    },
    {
      title: '可平量',
      dataIndex: 'availableVolume',
      key: 'availableVolume',
      width: 100,
      align: 'right',
      render: (volume: number) => formatNumber(volume),
    },
    {
      title: '开仓均价',
      dataIndex: 'avgPrice',
      key: 'avgPrice',
      width: 120,
      align: 'right',
      render: (price: number) => formatPrice(price),
    },
    {
      title: '当前价',
      key: 'currentPrice',
      width: 120,
      align: 'right',
      render: (_, record) => {
        const marketData = getMarketData(record.instrumentId);
        return marketData ? formatPrice(marketData.lastPrice) : '-';
      },
    },
    {
      title: '浮动盈亏',
      key: 'floatingPnL',
      width: 150,
      align: 'right',
      render: (_, record) => {
        const pnl = calculateFloatingPnL(record);
        const pnlPercent = calculatePnLPercent(record);
        return (
          <Space direction="vertical" size={0}>
            <span className={pnl >= 0 ? 'profit' : 'loss'}>
              {formatPrice(pnl)}
            </span>
            <span className={pnl >= 0 ? 'profit' : 'loss'}>
              {formatPercent(pnlPercent)}
            </span>
          </Space>
        );
      },
    },
    {
      title: '占用保证金',
      dataIndex: 'margin',
      key: 'margin',
      width: 120,
      align: 'right',
      render: (margin: number) => formatPrice(margin),
    },
    {
      title: '风险度',
      key: 'risk',
      width: 100,
      render: (_, record) => {
        const risk = calculateRiskLevel(record);
        let status: 'success' | 'normal' | 'exception' = 'success';
        if (risk > 60) status = 'exception';
        else if (risk > 30) status = 'normal';
        
        return (
          <Tooltip title={`风险度: ${risk}%`}>
            <Progress 
              percent={risk} 
              size="small" 
              status={status}
              showInfo={false}
            />
          </Tooltip>
        );
      },
    },
    {
      title: '操作',
      key: 'action',
      fixed: 'right',
      width: 200,
      render: (_, record) => (
        <Space size="small">
          <Button
            type="primary"
            size="small"
            icon={<CloseOutlined />}
            onClick={() => {
              setSelectedPosition(record);
              setCloseVolume(record.availableVolume);
              setCloseModalVisible(true);
            }}
          >
            平仓
          </Button>
          <Button
            size="small"
            icon={<PlusOutlined />}
            onClick={() => {
              onSelectInstrument?.(record.instrumentId);
              message.info('请在交易面板进行加仓操作');
            }}
          >
            加仓
          </Button>
          <Button
            size="small"
            icon={<WarningOutlined />}
            onClick={() => {
              setSelectedPosition(record);
              setStopLossModalVisible(true);
            }}
          >
            止损
          </Button>
        </Space>
      ),
    },
  ];

  // 计算汇总数据
  const summary = {
    totalPositions: positions.length,
    totalPnL: positions.reduce((sum, pos) => sum + calculateFloatingPnL(pos), 0),
    totalMargin: positions.reduce((sum, pos) => sum + pos.margin, 0),
    profitCount: positions.filter(pos => calculateFloatingPnL(pos) > 0).length,
    lossCount: positions.filter(pos => calculateFloatingPnL(pos) < 0).length,
  };

  return (
    <>
      <Card 
        className="position-panel"
        title="持仓管理"
        extra={
          <Space>
            <Button 
              onClick={refreshPositions}
              loading={loading}
              size="small"
            >
              刷新
            </Button>
          </Space>
        }
      >
        {/* 汇总信息 */}
        <div className="position-summary">
          <div className="summary-item">
            <span className="label">持仓数：</span>
            <span className="value">{summary.totalPositions}</span>
          </div>
          <div className="summary-item">
            <span className="label">浮动盈亏：</span>
            <span className={`value ${summary.totalPnL >= 0 ? 'profit' : 'loss'}`}>
              {formatPrice(summary.totalPnL)}
            </span>
          </div>
          <div className="summary-item">
            <span className="label">占用保证金：</span>
            <span className="value">{formatPrice(summary.totalMargin)}</span>
          </div>
          <div className="summary-item">
            <span className="label">盈/亏：</span>
            <span className="value">
              <span className="profit">{summary.profitCount}</span> / 
              <span className="loss"> {summary.lossCount}</span>
            </span>
          </div>
        </div>

        {/* 持仓列表 */}
        <Table
          columns={columns}
          dataSource={positions}
          rowKey="id"
          loading={loading}
          size="small"
          scroll={{ x: 1300 }}
          pagination={false}
          rowClassName={(record) => {
            const risk = calculateRiskLevel(record);
            if (risk > 60) return 'high-risk-row';
            return '';
          }}
        />
      </Card>

      {/* 平仓确认对话框 */}
      <Modal
        title="平仓确认"
        visible={closeModalVisible}
        onOk={handleClosePosition}
        onCancel={() => setCloseModalVisible(false)}
        confirmLoading={loading}
      >
        {selectedPosition && (
          <Space direction="vertical" style={{ width: '100%' }}>
            <div>
              <span>合约：{selectedPosition.instrumentId}</span>
            </div>
            <div>
              <span>方向：</span>
              <Tag color={selectedPosition.direction === 'Long' ? 'red' : 'green'}>
                {selectedPosition.direction === 'Long' ? '多' : '空'}
              </Tag>
            </div>
            <div>
              <span>可平量：{selectedPosition.availableVolume}</span>
            </div>
            <div>
              <span>平仓数量：</span>
              <InputNumber
                min={1}
                max={selectedPosition.availableVolume}
                value={closeVolume}
                onChange={(value) => setCloseVolume(value || 0)}
              />
            </div>
          </Space>
        )}
      </Modal>

      {/* 止损止盈设置对话框 */}
      <Modal
        title="止损止盈设置"
        visible={stopLossModalVisible}
        onOk={handleSetStopLoss}
        onCancel={() => setStopLossModalVisible(false)}
      >
        {selectedPosition && (
          <Space direction="vertical" style={{ width: '100%' }}>
            <div>
              <span>合约：{selectedPosition.instrumentId}</span>
            </div>
            <div>
              <span>开仓均价：{formatPrice(selectedPosition.avgPrice)}</span>
            </div>
            <div>
              <span>止损价：</span>
              <InputNumber
                style={{ width: 150 }}
                value={stopLossPrice}
                onChange={(value) => setStopLossPrice(value || 0)}
                step={0.01}
              />
            </div>
            <div>
              <span>止盈价：</span>
              <InputNumber
                style={{ width: 150 }}
                value={takeProfitPrice}
                onChange={(value) => setTakeProfitPrice(value || 0)}
                step={0.01}
              />
            </div>
          </Space>
        )}
      </Modal>
    </>
  );
};

export default PositionPanel;
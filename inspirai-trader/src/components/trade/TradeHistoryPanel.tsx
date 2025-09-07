import React, { useState, useEffect } from 'react';
import {
  Card,
  Table,
  DatePicker,
  Select,
  Input,
  Button,
  Space,
  Tag,
  Row,
  Col,
  Statistic,
  Divider,
  Modal,
  Descriptions
} from 'antd';
import {
  SearchOutlined,
  ExportOutlined,
  PrinterOutlined,
  FileExcelOutlined,
  BarChartOutlined,
  PieChartOutlined
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { Pie, Column } from '@ant-design/charts';
import { TradeRecord, OrderDirection, OffsetFlag } from '@/types';
import { useTradingStore } from '@/stores/tradingStore';
import { formatPrice, formatNumber, formatTime, formatPercent } from '@/utils/format';
import dayjs from 'dayjs';
import './TradeHistoryPanel.css';

const { RangePicker } = DatePicker;
const { Option } = Select;
const { Search } = Input;

interface TradeHistoryPanelProps {
  onSelectInstrument?: (instrumentId: string) => void;
}

export const TradeHistoryPanel: React.FC<TradeHistoryPanelProps> = ({ onSelectInstrument }) => {
  const { trades, loading, fetchTradeHistory } = useTradingStore();
  const [filteredTrades, setFilteredTrades] = useState<TradeRecord[]>([]);
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs, dayjs.Dayjs] | null>(null);
  const [selectedInstrument, setSelectedInstrument] = useState<string>('');
  const [selectedDirection, setSelectedDirection] = useState<string>('all');
  const [searchText, setSearchText] = useState('');
  const [statisticsVisible, setStatisticsVisible] = useState(false);
  const [selectedTrade, setSelectedTrade] = useState<TradeRecord | null>(null);
  const [detailVisible, setDetailVisible] = useState(false);

  // 加载成交记录
  useEffect(() => {
    fetchTradeHistory();
  }, [fetchTradeHistory]);

  // 过滤数据
  useEffect(() => {
    let filtered = [...trades];

    // 日期范围过滤
    if (dateRange) {
      filtered = filtered.filter(trade => {
        const tradeDate = dayjs(trade.tradeTime);
        return tradeDate.isAfter(dateRange[0]) && tradeDate.isBefore(dateRange[1]);
      });
    }

    // 合约过滤
    if (selectedInstrument) {
      filtered = filtered.filter(trade => 
        trade.instrumentId === selectedInstrument
      );
    }

    // 方向过滤
    if (selectedDirection !== 'all') {
      filtered = filtered.filter(trade => 
        trade.direction === selectedDirection
      );
    }

    // 搜索过滤
    if (searchText) {
      filtered = filtered.filter(trade =>
        trade.instrumentId.includes(searchText) ||
        trade.orderId.includes(searchText) ||
        trade.tradeId.includes(searchText)
      );
    }

    setFilteredTrades(filtered);
  }, [trades, dateRange, selectedInstrument, selectedDirection, searchText]);

  // 计算统计数据
  const calculateStatistics = () => {
    const stats = {
      totalTrades: filteredTrades.length,
      totalVolume: 0,
      totalAmount: 0,
      totalProfit: 0,
      totalLoss: 0,
      totalCommission: 0,
      winRate: 0,
      avgProfit: 0,
      avgLoss: 0,
      profitFactor: 0,
      instrumentStats: {} as Record<string, { count: number; profit: number }>,
    };

    let winCount = 0;
    let lossCount = 0;

    filteredTrades.forEach(trade => {
      stats.totalVolume += trade.volume;
      stats.totalAmount += trade.price * trade.volume;
      stats.totalCommission += trade.commission || 0;

      const profit = (trade.closeProfit || 0);
      if (profit > 0) {
        stats.totalProfit += profit;
        winCount++;
      } else {
        stats.totalLoss += Math.abs(profit);
        lossCount++;
      }

      // 按合约统计
      if (!stats.instrumentStats[trade.instrumentId]) {
        stats.instrumentStats[trade.instrumentId] = { count: 0, profit: 0 };
      }
      stats.instrumentStats[trade.instrumentId].count++;
      stats.instrumentStats[trade.instrumentId].profit += profit;
    });

    if (filteredTrades.length > 0) {
      stats.winRate = (winCount / filteredTrades.length) * 100;
      stats.avgProfit = winCount > 0 ? stats.totalProfit / winCount : 0;
      stats.avgLoss = lossCount > 0 ? stats.totalLoss / lossCount : 0;
      stats.profitFactor = stats.totalLoss > 0 ? stats.totalProfit / stats.totalLoss : 0;
    }

    return stats;
  };

  const stats = calculateStatistics();

  // 导出数据
  const handleExport = () => {
    // TODO: 实现导出功能
    const csvContent = [
      ['成交时间', '合约', '方向', '开平', '价格', '数量', '手续费', '平仓盈亏'],
      ...filteredTrades.map(trade => [
        formatTime(trade.tradeTime),
        trade.instrumentId,
        trade.direction === OrderDirection.BUY ? '买入' : '卖出',
        trade.offsetFlag,
        trade.price,
        trade.volume,
        trade.commission || 0,
        trade.closeProfit || 0,
      ])
    ].map(row => row.join(',')).join('\n');

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = `trades_${dayjs().format('YYYYMMDD_HHmmss')}.csv`;
    link.click();
  };

  const columns: ColumnsType<TradeRecord> = [
    {
      title: '成交时间',
      dataIndex: 'tradeTime',
      key: 'tradeTime',
      width: 160,
      render: (time: any) => formatTime(time),
      sorter: (a, b) => new Date(a.tradeTime).getTime() - new Date(b.tradeTime).getTime(),
    },
    {
      title: '合约',
      dataIndex: 'instrumentId',
      key: 'instrumentId',
      width: 120,
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
      render: (direction: OrderDirection) => (
        <Tag color={direction === OrderDirection.BUY ? 'red' : 'green'}>
          {direction === OrderDirection.BUY ? '买入' : '卖出'}
        </Tag>
      ),
      filters: [
        { text: '买入', value: OrderDirection.BUY },
        { text: '卖出', value: OrderDirection.SELL },
      ],
      onFilter: (value, record) => record.direction === value,
    },
    {
      title: '开平',
      dataIndex: 'offsetFlag',
      key: 'offsetFlag',
      width: 80,
      render: (flag: OffsetFlag) => {
        const flagText = {
          [OffsetFlag.OPEN]: '开仓',
          [OffsetFlag.CLOSE]: '平仓',
          [OffsetFlag.CLOSE_TODAY]: '平今',
          [OffsetFlag.CLOSE_YESTERDAY]: '平昨',
        };
        return flagText[flag] || flag;
      },
    },
    {
      title: '成交价',
      dataIndex: 'price',
      key: 'price',
      width: 100,
      align: 'right',
      render: (price: number) => formatPrice(price),
      sorter: (a, b) => a.price - b.price,
    },
    {
      title: '成交量',
      dataIndex: 'volume',
      key: 'volume',
      width: 80,
      align: 'right',
      render: (volume: number) => formatNumber(volume),
      sorter: (a, b) => a.volume - b.volume,
    },
    {
      title: '成交额',
      key: 'amount',
      width: 120,
      align: 'right',
      render: (_, record) => formatPrice(record.price * record.volume),
      sorter: (a, b) => (a.price * a.volume) - (b.price * b.volume),
    },
    {
      title: '手续费',
      dataIndex: 'commission',
      key: 'commission',
      width: 100,
      align: 'right',
      render: (commission: number) => formatPrice(commission || 0),
    },
    {
      title: '平仓盈亏',
      dataIndex: 'closeProfit',
      key: 'closeProfit',
      width: 120,
      align: 'right',
      render: (profit: number) => (
        <span className={profit >= 0 ? 'profit' : 'loss'}>
          {formatPrice(profit || 0)}
        </span>
      ),
      sorter: (a, b) => (a.closeProfit || 0) - (b.closeProfit || 0),
    },
    {
      title: '操作',
      key: 'action',
      fixed: 'right',
      width: 80,
      render: (_, record) => (
        <Button
          type="link"
          size="small"
          onClick={() => {
            setSelectedTrade(record);
            setDetailVisible(true);
          }}
        >
          详情
        </Button>
      ),
    },
  ];

  // 饼图数据
  const pieData = Object.entries(stats.instrumentStats).map(([key, value]) => ({
    type: key,
    value: value.count,
  }));

  // 柱状图数据
  const columnData = Object.entries(stats.instrumentStats).map(([key, value]) => ({
    instrument: key,
    profit: value.profit,
  }));

  return (
    <>
      <Card 
        className="trade-history-panel"
        title="成交记录"
        extra={
          <Space>
            <Button
              icon={<BarChartOutlined />}
              onClick={() => setStatisticsVisible(true)}
            >
              统计分析
            </Button>
            <Button
              icon={<ExportOutlined />}
              onClick={handleExport}
            >
              导出
            </Button>
          </Space>
        }
      >
        {/* 搜索栏 */}
        <Row gutter={16} style={{ marginBottom: 16 }}>
          <Col span={6}>
            <RangePicker
              style={{ width: '100%' }}
              onChange={(dates) => setDateRange(dates as [dayjs.Dayjs, dayjs.Dayjs])}
              placeholder={['开始日期', '结束日期']}
            />
          </Col>
          <Col span={4}>
            <Select
              style={{ width: '100%' }}
              placeholder="选择合约"
              allowClear
              onChange={setSelectedInstrument}
            >
              {Array.from(new Set(trades.map(t => t.instrumentId))).map(id => (
                <Option key={id} value={id}>{id}</Option>
              ))}
            </Select>
          </Col>
          <Col span={4}>
            <Select
              style={{ width: '100%' }}
              placeholder="买卖方向"
              value={selectedDirection}
              onChange={setSelectedDirection}
            >
              <Option value="all">全部</Option>
              <Option value={OrderDirection.BUY}>买入</Option>
              <Option value={OrderDirection.SELL}>卖出</Option>
            </Select>
          </Col>
          <Col span={6}>
            <Search
              placeholder="搜索合约/订单号"
              onSearch={setSearchText}
              enterButton={<SearchOutlined />}
            />
          </Col>
        </Row>

        {/* 统计信息 */}
        <Row gutter={16} style={{ marginBottom: 16 }}>
          <Col span={4}>
            <Statistic title="成交笔数" value={stats.totalTrades} />
          </Col>
          <Col span={4}>
            <Statistic title="总成交量" value={stats.totalVolume} />
          </Col>
          <Col span={4}>
            <Statistic 
              title="总盈亏" 
              value={stats.totalProfit - stats.totalLoss}
              precision={2}
              valueStyle={{ 
                color: stats.totalProfit - stats.totalLoss >= 0 ? '#f5222d' : '#52c41a' 
              }}
            />
          </Col>
          <Col span={4}>
            <Statistic 
              title="胜率" 
              value={stats.winRate}
              precision={1}
              suffix="%"
            />
          </Col>
          <Col span={4}>
            <Statistic 
              title="盈亏比" 
              value={stats.profitFactor}
              precision={2}
            />
          </Col>
          <Col span={4}>
            <Statistic 
              title="总手续费" 
              value={stats.totalCommission}
              precision={2}
            />
          </Col>
        </Row>

        {/* 成交记录表格 */}
        <Table
          columns={columns}
          dataSource={filteredTrades}
          rowKey="tradeId"
          loading={loading}
          size="small"
          scroll={{ x: 1300 }}
          pagination={{
            pageSize: 20,
            showSizeChanger: true,
            showTotal: (total) => `共 ${total} 条记录`,
          }}
        />
      </Card>

      {/* 统计分析弹窗 */}
      <Modal
        title="交易统计分析"
        visible={statisticsVisible}
        onCancel={() => setStatisticsVisible(false)}
        width={1000}
        footer={null}
      >
        <Row gutter={16}>
          <Col span={12}>
            <Card title="成交分布" size="small">
              <Pie
                data={pieData}
                angleField="value"
                colorField="type"
                radius={0.8}
                label={{
                  type: 'outer',
                  content: '{name} {percentage}',
                }}
                interactions={[{ type: 'element-active' }]}
              />
            </Card>
          </Col>
          <Col span={12}>
            <Card title="盈亏分布" size="small">
              <Column
                data={columnData}
                xField="instrument"
                yField="profit"
                label={{
                  position: 'middle',
                  style: {
                    fill: '#FFFFFF',
                    opacity: 0.6,
                  },
                }}
                color={(datum) => {
                  return datum.profit >= 0 ? '#f5222d' : '#52c41a';
                }}
              />
            </Card>
          </Col>
        </Row>
      </Modal>

      {/* 成交详情弹窗 */}
      <Modal
        title="成交详情"
        visible={detailVisible}
        onCancel={() => setDetailVisible(false)}
        footer={null}
      >
        {selectedTrade && (
          <Descriptions bordered column={2}>
            <Descriptions.Item label="成交编号">{selectedTrade.tradeId}</Descriptions.Item>
            <Descriptions.Item label="订单编号">{selectedTrade.orderId}</Descriptions.Item>
            <Descriptions.Item label="合约代码">{selectedTrade.instrumentId}</Descriptions.Item>
            <Descriptions.Item label="成交时间">{formatTime(selectedTrade.tradeTime)}</Descriptions.Item>
            <Descriptions.Item label="买卖方向">
              <Tag color={selectedTrade.direction === OrderDirection.BUY ? 'red' : 'green'}>
                {selectedTrade.direction === OrderDirection.BUY ? '买入' : '卖出'}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label="开平标志">{selectedTrade.offsetFlag}</Descriptions.Item>
            <Descriptions.Item label="成交价格">{formatPrice(selectedTrade.price)}</Descriptions.Item>
            <Descriptions.Item label="成交数量">{selectedTrade.volume}</Descriptions.Item>
            <Descriptions.Item label="成交金额">{formatPrice(selectedTrade.price * selectedTrade.volume)}</Descriptions.Item>
            <Descriptions.Item label="手续费">{formatPrice(selectedTrade.commission || 0)}</Descriptions.Item>
            <Descriptions.Item label="平仓盈亏" span={2}>
              <span className={selectedTrade.closeProfit >= 0 ? 'profit' : 'loss'}>
                {formatPrice(selectedTrade.closeProfit || 0)}
              </span>
            </Descriptions.Item>
          </Descriptions>
        )}
      </Modal>
    </>
  );
};

export default TradeHistoryPanel;
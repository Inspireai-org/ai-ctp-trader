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
  Select,
  DatePicker,
  Row,
  Col
} from 'antd';
import {
  DeleteOutlined,
  EditOutlined,
  ReloadOutlined,
  FilterOutlined,
  CheckCircleOutlined,
  ClockCircleOutlined,
  CloseCircleOutlined,
  SyncOutlined
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { OrderStatus, OrderStatusType, OrderDirection, OffsetFlag } from '@/types';
import { useTradingStore } from '@/stores/tradingStore';
import { formatNumber, formatPrice, formatTime } from '@/utils/format';
import './OrderPanel.css';

const { Option } = Select;
const { RangePicker } = DatePicker;

interface OrderPanelProps {
  onSelectInstrument?: (instrumentId: string) => void;
}

export const OrderPanel: React.FC<OrderPanelProps> = ({ onSelectInstrument }) => {
  const { orders, cancelOrder, modifyOrder, loading, refreshOrders } = useTradingStore();
  const [selectedOrder, setSelectedOrder] = useState<OrderStatus | null>(null);
  const [modifyModalVisible, setModifyModalVisible] = useState(false);
  const [newPrice, setNewPrice] = useState<number>(0);
  const [newVolume, setNewVolume] = useState<number>(0);
  const [filterStatus, setFilterStatus] = useState<OrderStatusType | 'all'>('all');
  const [filterInstrument, setFilterInstrument] = useState<string>('');
  const [selectedRowKeys, setSelectedRowKeys] = useState<string[]>([]);

  // 定期刷新订单数据
  useEffect(() => {
    refreshOrders();
    const interval = setInterval(refreshOrders, 2000); // 每2秒刷新一次
    return () => clearInterval(interval);
  }, [refreshOrders]);

  // 获取状态标签
  const getStatusTag = (status: OrderStatusType) => {
    const statusConfig = {
      [OrderStatusType.PENDING]: { color: 'processing', icon: <ClockCircleOutlined />, text: '待报' },
      [OrderStatusType.SUBMITTED]: { color: 'processing', icon: <SyncOutlined spin />, text: '已报' },
      [OrderStatusType.PARTIAL_FILLED]: { color: 'warning', icon: <SyncOutlined spin />, text: '部分成交' },
      [OrderStatusType.FILLED]: { color: 'success', icon: <CheckCircleOutlined />, text: '全部成交' },
      [OrderStatusType.CANCELLED]: { color: 'default', icon: <CloseCircleOutlined />, text: '已撤' },
      [OrderStatusType.REJECTED]: { color: 'error', icon: <CloseCircleOutlined />, text: '拒绝' },
    };

    const config = statusConfig[status] || { color: 'default', icon: null, text: '未知' };
    
    return (
      <Tag color={config.color} icon={config.icon}>
        {config.text}
      </Tag>
    );
  };

  // 处理撤单
  const handleCancelOrder = async (orderId: string) => {
    try {
      await cancelOrder(orderId);
      message.success('撤单指令已发送');
    } catch (error: any) {
      message.error(`撤单失败: ${error.message}`);
    }
  };

  // 批量撤单
  const handleBatchCancel = async () => {
    if (selectedRowKeys.length === 0) {
      message.warning('请选择要撤销的订单');
      return;
    }

    try {
      await Promise.all(selectedRowKeys.map(id => cancelOrder(id)));
      message.success(`已撤销 ${selectedRowKeys.length} 个订单`);
      setSelectedRowKeys([]);
    } catch (error: any) {
      message.error(`批量撤单失败: ${error.message}`);
    }
  };

  // 处理改单
  const handleModifyOrder = async () => {
    if (!selectedOrder) return;

    try {
      await modifyOrder(selectedOrder.orderId, newPrice, newVolume);
      message.success('改单指令已发送');
      setModifyModalVisible(false);
    } catch (error: any) {
      message.error(`改单失败: ${error.message}`);
    }
  };

  // 过滤订单
  const filteredOrders = orders.filter(order => {
    if (filterStatus !== 'all' && order.status !== filterStatus) return false;
    if (filterInstrument && !order.instrumentId.includes(filterInstrument)) return false;
    return true;
  });

  const columns: ColumnsType<OrderStatus> = [
    {
      title: '合约',
      dataIndex: 'instrumentId',
      key: 'instrumentId',
      width: 120,
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
      title: '委托时间',
      dataIndex: 'submitTime',
      key: 'submitTime',
      width: 160,
      render: (time: any) => formatTime(time),
    },
    {
      title: '方向',
      key: 'direction',
      width: 80,
      render: (_, record) => (
        <Tag color={record.direction === OrderDirection.BUY ? 'red' : 'green'}>
          {record.direction === OrderDirection.BUY ? '买入' : '卖出'}
        </Tag>
      ),
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
      title: '委托价',
      dataIndex: 'price',
      key: 'price',
      width: 100,
      align: 'right',
      render: (price: number) => formatPrice(price),
    },
    {
      title: '委托量',
      dataIndex: 'volume',
      key: 'volume',
      width: 80,
      align: 'right',
      render: (volume: number) => formatNumber(volume),
    },
    {
      title: '成交量',
      dataIndex: 'volumeTraded',
      key: 'volumeTraded',
      width: 80,
      align: 'right',
      render: (volume: number) => formatNumber(volume),
    },
    {
      title: '剩余量',
      dataIndex: 'volumeLeft',
      key: 'volumeLeft',
      width: 80,
      align: 'right',
      render: (volume: number) => formatNumber(volume),
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 120,
      render: (status: OrderStatusType) => getStatusTag(status),
    },
    {
      title: '成交进度',
      key: 'progress',
      width: 150,
      render: (_, record) => {
        const percent = record.volume > 0 
          ? Math.round((record.volumeTraded / record.volume) * 100)
          : 0;
        return (
          <div className="order-progress">
            <div className="progress-bar" style={{ width: `${percent}%` }} />
            <span className="progress-text">{percent}%</span>
          </div>
        );
      },
    },
    {
      title: '操作',
      key: 'action',
      fixed: 'right',
      width: 150,
      render: (_, record) => {
        const canCancel = [
          OrderStatusType.PENDING,
          OrderStatusType.SUBMITTED,
          OrderStatusType.PARTIAL_FILLED
        ].includes(record.status);

        const canModify = [
          OrderStatusType.PENDING,
          OrderStatusType.SUBMITTED
        ].includes(record.status);

        return (
          <Space size="small">
            {canModify && (
              <Button
                size="small"
                icon={<EditOutlined />}
                onClick={() => {
                  setSelectedOrder(record);
                  setNewPrice(record.price);
                  setNewVolume(record.volumeLeft);
                  setModifyModalVisible(true);
                }}
              >
                改单
              </Button>
            )}
            {canCancel && (
              <Popconfirm
                title="确定要撤销此订单吗？"
                onConfirm={() => handleCancelOrder(record.orderId)}
                okText="确定"
                cancelText="取消"
              >
                <Button
                  size="small"
                  danger
                  icon={<DeleteOutlined />}
                >
                  撤单
                </Button>
              </Popconfirm>
            )}
          </Space>
        );
      },
    },
  ];

  const rowSelection = {
    selectedRowKeys,
    onChange: (keys: React.Key[]) => {
      setSelectedRowKeys(keys as string[]);
    },
    getCheckboxProps: (record: OrderStatus) => ({
      disabled: ![
        OrderStatusType.PENDING,
        OrderStatusType.SUBMITTED,
        OrderStatusType.PARTIAL_FILLED
      ].includes(record.status),
    }),
  };

  return (
    <>
      <Card 
        className="order-panel"
        title="委托管理"
        extra={
          <Space>
            <Select
              value={filterStatus}
              onChange={setFilterStatus}
              style={{ width: 120 }}
              size="small"
              placeholder="状态筛选"
            >
              <Option value="all">全部状态</Option>
              <Option value={OrderStatusType.PENDING}>待报</Option>
              <Option value={OrderStatusType.SUBMITTED}>已报</Option>
              <Option value={OrderStatusType.PARTIAL_FILLED}>部分成交</Option>
              <Option value={OrderStatusType.FILLED}>全部成交</Option>
              <Option value={OrderStatusType.CANCELLED}>已撤</Option>
            </Select>
            
            <Button 
              onClick={refreshOrders}
              loading={loading}
              icon={<ReloadOutlined />}
              size="small"
            >
              刷新
            </Button>
            
            {selectedRowKeys.length > 0 && (
              <Popconfirm
                title={`确定要撤销选中的 ${selectedRowKeys.length} 个订单吗？`}
                onConfirm={handleBatchCancel}
                okText="确定"
                cancelText="取消"
              >
                <Button 
                  danger
                  size="small"
                >
                  批量撤单({selectedRowKeys.length})
                </Button>
              </Popconfirm>
            )}
          </Space>
        }
      >
        <Table
          rowSelection={rowSelection}
          columns={columns}
          dataSource={filteredOrders}
          rowKey="orderId"
          loading={loading}
          size="small"
          scroll={{ x: 1500 }}
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
            showTotal: (total) => `共 ${total} 条`,
          }}
        />
      </Card>

      {/* 改单对话框 */}
      <Modal
        title="修改订单"
        visible={modifyModalVisible}
        onOk={handleModifyOrder}
        onCancel={() => setModifyModalVisible(false)}
        confirmLoading={loading}
      >
        {selectedOrder && (
          <Space direction="vertical" style={{ width: '100%' }}>
            <div>
              <span>合约：{selectedOrder.instrumentId}</span>
            </div>
            <div>
              <span>方向：</span>
              <Tag color={selectedOrder.direction === OrderDirection.BUY ? 'red' : 'green'}>
                {selectedOrder.direction === OrderDirection.BUY ? '买入' : '卖出'}
              </Tag>
            </div>
            <div>
              <span>原价格：{formatPrice(selectedOrder.price)}</span>
            </div>
            <div>
              <span>新价格：</span>
              <InputNumber
                style={{ width: 150 }}
                value={newPrice}
                onChange={(value) => setNewPrice(value || 0)}
                step={0.01}
                min={0}
              />
            </div>
            <div>
              <span>原数量：{selectedOrder.volumeLeft}</span>
            </div>
            <div>
              <span>新数量：</span>
              <InputNumber
                style={{ width: 150 }}
                value={newVolume}
                onChange={(value) => setNewVolume(value || 0)}
                min={1}
                max={selectedOrder.volumeLeft}
              />
            </div>
          </Space>
        )}
      </Modal>
    </>
  );
};

export default OrderPanel;
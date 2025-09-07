import React, { useState, useEffect, useMemo } from 'react';
import { 
  Card, 
  Form, 
  Input, 
  InputNumber, 
  Button, 
  Select, 
  Space, 
  Radio, 
  Tabs, 
  Typography, 
  Divider,
  Tooltip,
  Modal,
  message
} from 'antd';
import { 
  ArrowUpOutlined, 
  ArrowDownOutlined, 
  ThunderboltOutlined,
  SwapOutlined,
  StopOutlined
} from '@ant-design/icons';
import { OrderDirection, OffsetFlag, OrderType, OrderRequest } from '@/types';
import { useTradingStore } from '@/stores/tradingStore';
import { useMarketStore } from '@/stores/marketStore';
import { formatNumber, formatPrice } from '@/utils/format';
import './TradingPanel.css';

const { Title, Text } = Typography;
const { TabPane } = Tabs;
const { Option } = Select;

interface TradingPanelProps {
  instrumentId?: string;
}

export const TradingPanel: React.FC<TradingPanelProps> = ({ instrumentId }) => {
  const [form] = Form.useForm();
  const [direction, setDirection] = useState<OrderDirection>(OrderDirection.BUY);
  const [orderType, setOrderType] = useState<OrderType>(OrderType.LIMIT);
  const [offsetFlag, setOffsetFlag] = useState<OffsetFlag>(OffsetFlag.OPEN);
  const [confirmModalVisible, setConfirmModalVisible] = useState(false);
  const [pendingOrder, setPendingOrder] = useState<OrderRequest | null>(null);
  
  const { placeOrder, loading } = useTradingStore();
  const { getMarketData, selectedInstrument } = useMarketStore();
  
  const currentInstrument = instrumentId || selectedInstrument;
  const marketData = currentInstrument ? getMarketData(currentInstrument) : null;

  // 计算保证金和手续费
  const calculateMargin = (price: number, volume: number): number => {
    // TODO: 根据合约信息计算实际保证金
    return price * volume * 0.1; // 假设10%保证金率
  };

  const calculateCommission = (price: number, volume: number): number => {
    // TODO: 根据合约信息计算实际手续费
    return price * volume * 0.0001; // 假设万分之一手续费
  };

  // 监听市场价格变化，更新表单价格
  useEffect(() => {
    if (marketData && orderType === OrderType.LIMIT) {
      form.setFieldsValue({
        price: direction === OrderDirection.BUY ? marketData.bidPrice1 : marketData.askPrice1
      });
    }
  }, [marketData, direction, orderType, form]);

  // 处理下单
  const handleSubmit = async (values: any) => {
    if (!currentInstrument) {
      message.error('请选择合约');
      return;
    }

    const order: OrderRequest = {
      instrumentId: currentInstrument,
      direction,
      offsetFlag,
      orderType,
      price: orderType === OrderType.MARKET ? 0 : values.price,
      volume: values.volume,
      timeCondition: values.timeCondition || 'GFD',
      volumeCondition: values.volumeCondition || 'ANY',
      minVolume: 1,
      contingentCondition: 'IMMEDIATELY',
      stopPrice: 0,
    };

    setPendingOrder(order);
    setConfirmModalVisible(true);
  };

  // 确认下单
  const handleConfirmOrder = async () => {
    if (!pendingOrder) return;
    
    try {
      await placeOrder(pendingOrder);
      message.success('订单提交成功');
      form.resetFields();
      setConfirmModalVisible(false);
    } catch (error: any) {
      message.error(`订单提交失败: ${error.message}`);
    }
  };

  // 快速交易按钮
  const QuickTradeButtons = () => (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Button 
        type="primary" 
        danger 
        icon={<ThunderboltOutlined />}
        block
        onClick={() => {
          // TODO: 实现一键平仓
          message.info('一键平仓功能开发中');
        }}
      >
        一键平仓
      </Button>
      <Button 
        icon={<SwapOutlined />}
        block
        onClick={() => {
          // TODO: 实现反手操作
          message.info('反手功能开发中');
        }}
      >
        反手
      </Button>
      <Button 
        icon={<StopOutlined />}
        block
        onClick={() => {
          // TODO: 实现止损止盈设置
          message.info('止损止盈功能开发中');
        }}
      >
        止损止盈
      </Button>
    </Space>
  );

  return (
    <Card className="trading-panel" title="交易下单">
      <Tabs defaultActiveKey="normal">
        <TabPane tab="普通下单" key="normal">
          <Form
            form={form}
            layout="vertical"
            onFinish={handleSubmit}
            initialValues={{
              volume: 1,
              timeCondition: 'GFD',
              volumeCondition: 'ANY'
            }}
          >
            {/* 买卖方向选择 */}
            <Form.Item>
              <Radio.Group 
                value={direction} 
                onChange={e => setDirection(e.target.value)}
                buttonStyle="solid"
                style={{ width: '100%' }}
              >
                <Radio.Button 
                  value={OrderDirection.BUY} 
                  style={{ width: '50%', textAlign: 'center' }}
                  className="buy-button"
                >
                  <ArrowUpOutlined /> 买入
                </Radio.Button>
                <Radio.Button 
                  value={OrderDirection.SELL} 
                  style={{ width: '50%', textAlign: 'center' }}
                  className="sell-button"
                >
                  <ArrowDownOutlined /> 卖出
                </Radio.Button>
              </Radio.Group>
            </Form.Item>

            {/* 开平仓选择 */}
            <Form.Item label="开平">
              <Select value={offsetFlag} onChange={setOffsetFlag}>
                <Option value={OffsetFlag.OPEN}>开仓</Option>
                <Option value={OffsetFlag.CLOSE}>平仓</Option>
                <Option value={OffsetFlag.CLOSE_TODAY}>平今</Option>
                <Option value={OffsetFlag.CLOSE_YESTERDAY}>平昨</Option>
              </Select>
            </Form.Item>

            {/* 价格类型 */}
            <Form.Item label="价格类型">
              <Select value={orderType} onChange={setOrderType}>
                <Option value={OrderType.LIMIT}>限价</Option>
                <Option value={OrderType.MARKET}>市价</Option>
                <Option value={OrderType.BEST}>最优价</Option>
                <Option value={OrderType.LAST_PRICE}>最新价</Option>
              </Select>
            </Form.Item>

            {/* 价格输入 */}
            {orderType === OrderType.LIMIT && (
              <Form.Item
                label="价格"
                name="price"
                rules={[{ required: true, message: '请输入价格' }]}
              >
                <InputNumber
                  style={{ width: '100%' }}
                  step={0.01}
                  min={0}
                  placeholder="输入价格"
                />
              </Form.Item>
            )}

            {/* 数量输入 */}
            <Form.Item
              label="数量"
              name="volume"
              rules={[{ required: true, message: '请输入数量' }]}
            >
              <InputNumber
                style={{ width: '100%' }}
                min={1}
                placeholder="输入数量"
              />
            </Form.Item>

            {/* 费用预估 */}
            <div className="fee-estimate">
              <div className="fee-item">
                <Text type="secondary">保证金：</Text>
                <Text>{formatPrice(calculateMargin(form.getFieldValue('price') || 0, form.getFieldValue('volume') || 0))}</Text>
              </div>
              <div className="fee-item">
                <Text type="secondary">手续费：</Text>
                <Text>{formatPrice(calculateCommission(form.getFieldValue('price') || 0, form.getFieldValue('volume') || 0))}</Text>
              </div>
            </div>

            <Divider />

            {/* 提交按钮 */}
            <Form.Item>
              <Button
                type="primary"
                htmlType="submit"
                block
                size="large"
                loading={loading}
                className={direction === OrderDirection.BUY ? 'submit-buy' : 'submit-sell'}
              >
                {direction === OrderDirection.BUY ? '买入' : '卖出'}
              </Button>
            </Form.Item>
          </Form>
        </TabPane>
        
        <TabPane tab="快速交易" key="quick">
          <QuickTradeButtons />
        </TabPane>
        
        <TabPane tab="条件单" key="condition">
          <div style={{ padding: '20px', textAlign: 'center' }}>
            <Text type="secondary">条件单功能开发中...</Text>
          </div>
        </TabPane>
      </Tabs>

      {/* 确认对话框 */}
      <Modal
        title="订单确认"
        visible={confirmModalVisible}
        onOk={handleConfirmOrder}
        onCancel={() => setConfirmModalVisible(false)}
        confirmLoading={loading}
      >
        {pendingOrder && (
          <div className="order-confirm-content">
            <div className="confirm-item">
              <Text strong>合约：</Text> {pendingOrder.instrumentId}
            </div>
            <div className="confirm-item">
              <Text strong>方向：</Text> 
              <Text type={pendingOrder.direction === OrderDirection.BUY ? 'danger' : 'success'}>
                {pendingOrder.direction === OrderDirection.BUY ? '买入' : '卖出'}
              </Text>
            </div>
            <div className="confirm-item">
              <Text strong>开平：</Text> {pendingOrder.offsetFlag}
            </div>
            <div className="confirm-item">
              <Text strong>价格：</Text> 
              {pendingOrder.orderType === OrderType.MARKET ? '市价' : formatPrice(pendingOrder.price)}
            </div>
            <div className="confirm-item">
              <Text strong>数量：</Text> {pendingOrder.volume}
            </div>
            <Divider />
            <div className="confirm-item">
              <Text strong>预估保证金：</Text> 
              {formatPrice(calculateMargin(pendingOrder.price, pendingOrder.volume))}
            </div>
            <div className="confirm-item">
              <Text strong>预估手续费：</Text> 
              {formatPrice(calculateCommission(pendingOrder.price, pendingOrder.volume))}
            </div>
          </div>
        )}
      </Modal>
    </Card>
  );
};

export default TradingPanel;
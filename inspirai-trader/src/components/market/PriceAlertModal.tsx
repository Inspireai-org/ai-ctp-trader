import React from 'react';
import { Modal, Form, Select, InputNumber, Switch, message } from 'antd';
import { BellOutlined } from '@ant-design/icons';

const { Option } = Select;

interface PriceAlertModalProps {
  visible: boolean;
  instrumentId: string;
  currentPrice?: number;
  onClose: () => void;
  onConfirm: (alert: PriceAlert) => void;
}

export interface PriceAlert {
  instrumentId: string;
  condition: 'above' | 'below' | 'cross';
  price: number;
  enabled: boolean;
  sound: boolean;
  notification: boolean;
}

/**
 * 价格预警设置弹窗
 */
const PriceAlertModal: React.FC<PriceAlertModalProps> = ({
  visible,
  instrumentId,
  currentPrice,
  onClose,
  onConfirm,
}) => {
  const [form] = Form.useForm();

  const handleOk = async () => {
    try {
      const values = await form.validateFields();
      const alert: PriceAlert = {
        instrumentId,
        ...values,
      };
      
      onConfirm(alert);
      form.resetFields();
      onClose();
      message.success('价格预警设置成功');
    } catch (error) {
      message.error('请填写完整的预警信息');
    }
  };

  return (
    <Modal
      title={
        <>
          <BellOutlined /> 设置价格预警 - {instrumentId}
        </>
      }
      open={visible}
      onOk={handleOk}
      onCancel={onClose}
      okText="确定"
      cancelText="取消"
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          condition: 'above',
          price: currentPrice,
          enabled: true,
          sound: true,
          notification: true,
        }}
      >
        <Form.Item
          label="预警条件"
          name="condition"
          rules={[{ required: true, message: '请选择预警条件' }]}
        >
          <Select>
            <Option value="above">高于</Option>
            <Option value="below">低于</Option>
            <Option value="cross">穿越</Option>
          </Select>
        </Form.Item>

        <Form.Item
          label="预警价格"
          name="price"
          rules={[
            { required: true, message: '请输入预警价格' },
            { type: 'number', min: 0, message: '价格必须大于0' },
          ]}
        >
          <InputNumber
            style={{ width: '100%' }}
            precision={2}
            placeholder="请输入预警价格"
            addonAfter={currentPrice ? `当前价: ${currentPrice}` : undefined}
          />
        </Form.Item>

        <Form.Item
          label="立即启用"
          name="enabled"
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>

        <Form.Item
          label="声音提醒"
          name="sound"
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>

        <Form.Item
          label="系统通知"
          name="notification"
          valuePropName="checked"
        >
          <Switch />
        </Form.Item>
      </Form>
    </Modal>
  );
};

export default PriceAlertModal;
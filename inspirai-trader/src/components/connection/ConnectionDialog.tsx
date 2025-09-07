import React, { useState, useEffect } from 'react';
import { Modal, Form, Input, Select, Button, Alert, Tabs, Space, Divider } from 'antd';
import { LinkOutlined, UserOutlined, KeyOutlined, SettingOutlined } from '@ant-design/icons';
import { useConnectionStore } from '@/stores/connectionStore';
import { CtpConfig, LoginCredentials } from '@/types/ctp';
import './ConnectionDialog.css';

const { Option } = Select;
const { TabPane } = Tabs;

interface ConnectionDialogProps {
  visible: boolean;
  onClose: () => void;
}

export const ConnectionDialog: React.FC<ConnectionDialogProps> = ({ visible, onClose }) => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState('quick');
  
  const { 
    connect, 
    login, 
    connectionState, 
    connectionStatus,
    setConfig 
  } = useConnectionStore();

  // 预设配置
  const presetConfigs = {
    simnow: {
      environment: 'SimNow',
      broker_id: '9999',
      market_front: ['tcp://180.168.146.187:10131'],
      trade_front: ['tcp://180.168.146.187:10130'],
      app_id: 'simnow_client',
      auth_code: '0000000000000000'
    },
    tts: {
      environment: 'TTS',
      broker_id: '9999',
      market_front: ['tcp://180.168.146.187:10211'],
      trade_front: ['tcp://180.168.146.187:10201'],
      app_id: 'tts_client',
      auth_code: '0000000000000000'
    },
    production: {
      environment: 'Production',
      broker_id: '',
      market_front: [''],
      trade_front: [''],
      app_id: '',
      auth_code: ''
    }
  };

  const handleQuickConnect = async () => {
    try {
      const values = await form.validateFields();
      setLoading(true);
      
      // 构建配置
      const config: CtpConfig = {
        ...presetConfigs[values.preset as keyof typeof presetConfigs],
        user_product_info: 'InspirAI_Trader',
        protocol_info: 'InspirAI_Protocol',
        flow_path: './flow',
        public_resume_type: 2,
        private_resume_type: 2,
        using_udp: false,
        using_multicast: false
      } as CtpConfig;
      
      setConfig(config);
      await connect();
      
      // 登录
      const credentials: LoginCredentials = {
        user_id: values.user_id,
        password: values.password,
        broker_id: config.broker_id
      };
      
      await login(credentials);
      onClose();
    } catch (error) {
      console.error('Connection failed:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCustomConnect = async () => {
    try {
      const values = await form.validateFields();
      setLoading(true);
      
      const config: CtpConfig = {
        environment: 'Production' as const,
        broker_id: values.broker_id,
        market_front: values.market_front.split(',').map((s: string) => s.trim()),
        trade_front: values.trade_front.split(',').map((s: string) => s.trim()),
        app_id: values.app_id,
        auth_code: values.auth_code,
        user_product_info: values.user_product_info || 'InspirAI_Trader',
        protocol_info: values.protocol_info || 'InspirAI_Protocol',
        flow_path: values.flow_path || './flow',
        public_resume_type: 2,
        private_resume_type: 2,
        using_udp: false,
        using_multicast: false
      };
      
      setConfig(config);
      await connect();
      
      const credentials: LoginCredentials = {
        user_id: values.user_id,
        password: values.password,
        broker_id: values.broker_id
      };
      
      await login(credentials);
      onClose();
    } catch (error) {
      console.error('Connection failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal
      title="连接到交易系统"
      visible={visible}
      onCancel={onClose}
      footer={null}
      width={600}
      className="connection-dialog"
    >
      <Tabs activeKey={activeTab} onChange={setActiveTab}>
        <TabPane tab="快速连接" key="quick">
          <Form form={form} layout="vertical">
            <Form.Item
              name="preset"
              label="选择环境"
              rules={[{ required: true, message: '请选择连接环境' }]}
              initialValue="simnow"
            >
              <Select size="large">
                <Option value="simnow">SimNow 模拟环境</Option>
                <Option value="tts">TTS 测试环境</Option>
                <Option value="production">生产环境</Option>
              </Select>
            </Form.Item>
            
            <Divider />
            
            <Form.Item
              name="user_id"
              label="用户名"
              rules={[{ required: true, message: '请输入用户名' }]}
            >
              <Input 
                prefix={<UserOutlined />} 
                placeholder="请输入交易账号"
                size="large"
              />
            </Form.Item>
            
            <Form.Item
              name="password"
              label="密码"
              rules={[{ required: true, message: '请输入密码' }]}
            >
              <Input.Password 
                prefix={<KeyOutlined />} 
                placeholder="请输入交易密码"
                size="large"
              />
            </Form.Item>
            
            {connectionStatus && (
              <Alert 
                message={connectionStatus} 
                type={connectionState === 'LoggedIn' ? 'success' : 'info'} 
                showIcon 
                style={{ marginBottom: 16 }}
              />
            )}
            
            <Space style={{ width: '100%', justifyContent: 'flex-end' }}>
              <Button onClick={onClose}>取消</Button>
              <Button 
                type="primary" 
                icon={<LinkOutlined />}
                onClick={handleQuickConnect}
                loading={loading}
              >
                连接
              </Button>
            </Space>
          </Form>
        </TabPane>
        
        <TabPane tab="自定义连接" key="custom">
          <Form form={form} layout="vertical">
            <Form.Item
              name="broker_id"
              label="经纪商ID"
              rules={[{ required: true, message: '请输入经纪商ID' }]}
            >
              <Input placeholder="例如: 9999" />
            </Form.Item>
            
            <Form.Item
              name="market_front"
              label="行情前置地址"
              rules={[{ required: true, message: '请输入行情前置地址' }]}
            >
              <Input placeholder="例如: tcp://180.168.146.187:10131" />
            </Form.Item>
            
            <Form.Item
              name="trade_front"
              label="交易前置地址"
              rules={[{ required: true, message: '请输入交易前置地址' }]}
            >
              <Input placeholder="例如: tcp://180.168.146.187:10130" />
            </Form.Item>
            
            <Form.Item
              name="app_id"
              label="应用ID"
              rules={[{ required: true }]}
            >
              <Input placeholder="应用标识" />
            </Form.Item>
            
            <Form.Item
              name="auth_code"
              label="认证码"
              rules={[{ required: true }]}
            >
              <Input placeholder="认证码" />
            </Form.Item>
            
            <Divider />
            
            <Form.Item
              name="user_id"
              label="用户名"
              rules={[{ required: true, message: '请输入用户名' }]}
            >
              <Input prefix={<UserOutlined />} placeholder="交易账号" />
            </Form.Item>
            
            <Form.Item
              name="password"
              label="密码"
              rules={[{ required: true, message: '请输入密码' }]}
            >
              <Input.Password prefix={<KeyOutlined />} placeholder="交易密码" />
            </Form.Item>
            
            <Space style={{ width: '100%', justifyContent: 'flex-end' }}>
              <Button onClick={onClose}>取消</Button>
              <Button 
                type="primary" 
                icon={<SettingOutlined />}
                onClick={handleCustomConnect}
                loading={loading}
              >
                连接
              </Button>
            </Space>
          </Form>
        </TabPane>
      </Tabs>
    </Modal>
  );
};

export default ConnectionDialog;
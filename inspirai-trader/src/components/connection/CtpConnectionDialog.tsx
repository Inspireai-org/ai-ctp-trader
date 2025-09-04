import React, { useState, useEffect } from 'react';
import { Modal, Form, Input, Select, Button, Steps, message, Alert, Spin, Space } from 'antd';
import { 
  ApiOutlined, 
  LoginOutlined, 
  CheckCircleOutlined, 
  LoadingOutlined 
} from '@ant-design/icons';
import { ctpService } from '@/services/tauri';
import { useMarketDataStore } from '@/stores/marketData';
import type { CtpConfig, Environment, LoginCredentials } from '@/types';
import { ConnectionStatus } from '@/types';

interface CtpConnectionDialogProps {
  visible: boolean;
  onClose: () => void;
  onConnected: () => void;
}

const CtpConnectionDialog: React.FC<CtpConnectionDialogProps> = ({
  visible,
  onClose,
  onConnected
}) => {
  const [form] = Form.useForm();
  const [currentStep, setCurrentStep] = useState(0);
  const [loading, setLoading] = useState(false);
  const [config, setConfig] = useState<CtpConfig | null>(null);
  const [connectionError, setConnectionError] = useState<string>('');
  
  const { setConnectionStatus } = useMarketDataStore();

  useEffect(() => {
    if (visible) {
      loadDefaultConfig();
    }
  }, [visible]);

  const loadDefaultConfig = async () => {
    try {
      setLoading(true);
      await ctpService.init();
      const defaultConfig = await ctpService.createConfig();
      setConfig(defaultConfig);
      
      // 设置表单默认值
      form.setFieldsValue({
        environment: defaultConfig.environment || 'SimNow',
        brokerId: defaultConfig.brokerId || '9999',
        appId: defaultConfig.appId || 'simnow_client',
        authCode: defaultConfig.authCode || '0000000000000000',
      });
    } catch (error) {
      message.error('加载配置失败: ' + (error as Error).message);
      setConnectionError((error as Error).message);
    } finally {
      setLoading(false);
    }
  };

  const handleConnect = async () => {
    try {
      const values = await form.validateFields(['environment', 'brokerId', 'appId', 'authCode']);
      setLoading(true);
      setConnectionError('');
      setConnectionStatus(ConnectionStatus.CONNECTING);

      // 构建连接配置
      const connectionConfig: CtpConfig = {
        ...config!,
        environment: values.environment as Environment,
        brokerId: values.brokerId,
        appId: values.appId,
        authCode: values.authCode,
      };

      // 连接到 CTP 服务器
      await ctpService.connect(connectionConfig);
      
      message.success('连接成功！');
      setCurrentStep(1);
      setConnectionStatus(ConnectionStatus.CONNECTED);
    } catch (error) {
      const errorMsg = (error as Error).message;
      message.error('连接失败: ' + errorMsg);
      setConnectionError(errorMsg);
      setConnectionStatus(ConnectionStatus.DISCONNECTED);
    } finally {
      setLoading(false);
    }
  };

  const handleLogin = async () => {
    try {
      const values = await form.validateFields(['userId', 'password']);
      setLoading(true);
      setConnectionError('');

      // 登录
      await ctpService.login({
        brokerId: config!.brokerId,
        userId: values.userId,
        password: values.password,
        appId: config!.appId,
        authCode: config!.authCode,
      });

      message.success('登录成功！');
      setCurrentStep(2);

      // 订阅默认合约
      await subscribeDefaultInstruments();
      
      // 通知父组件连接成功
      onConnected();
      
      // 延迟关闭对话框
      setTimeout(() => {
        onClose();
        setCurrentStep(0);
        form.resetFields();
      }, 1500);
    } catch (error) {
      const errorMsg = (error as Error).message;
      message.error('登录失败: ' + errorMsg);
      setConnectionError(errorMsg);
    } finally {
      setLoading(false);
    }
  };

  const subscribeDefaultInstruments = async () => {
    try {
      // 订阅一些默认的热门合约
      const defaultInstruments = ['rb2501', 'i2501', 'ag2502', 'au2502', 'cu2501'];
      const { batchSubscribe, addToWatchlist } = useMarketDataStore.getState();
      
      // 添加到自选列表
      defaultInstruments.forEach(instrumentId => {
        addToWatchlist(instrumentId);
      });
      
      // 批量订阅
      await batchSubscribe(defaultInstruments);
      message.success(`已订阅 ${defaultInstruments.length} 个默认合约`);
    } catch (error) {
      console.error('订阅默认合约失败:', error);
    }
  };

  const handleCancel = () => {
    if (loading) {
      message.warning('正在连接中，请稍候...');
      return;
    }
    setCurrentStep(0);
    form.resetFields();
    setConnectionError('');
    onClose();
  };

  return (
    <Modal
      title="连接 CTP 交易系统"
      open={visible}
      onCancel={handleCancel}
      width={600}
      footer={null}
      maskClosable={false}
    >
      <Steps
        current={currentStep}
        items={[
          {
            title: '服务器配置',
            icon: currentStep === 0 && loading ? <LoadingOutlined /> : <ApiOutlined />,
          },
          {
            title: '账户登录',
            icon: currentStep === 1 && loading ? <LoadingOutlined /> : <LoginOutlined />,
          },
          {
            title: '完成',
            icon: <CheckCircleOutlined />,
          },
        ]}
        style={{ marginBottom: 24 }}
      />

      <Spin spinning={loading}>
        <Form
          form={form}
          layout="vertical"
          onFinish={currentStep === 0 ? handleConnect : handleLogin}
        >
          {currentStep === 0 && (
            <>
              <Form.Item
                label="交易环境"
                name="environment"
                rules={[{ required: true, message: '请选择交易环境' }]}
              >
                <Select>
                  <Select.Option value="SimNow">SimNow 模拟环境</Select.Option>
                  <Select.Option value="TTS">TTS 测试环境</Select.Option>
                  <Select.Option value="Production">生产环境</Select.Option>
                </Select>
              </Form.Item>

              <Form.Item
                label="经纪商代码"
                name="brokerId"
                rules={[{ required: true, message: '请输入经纪商代码' }]}
              >
                <Input placeholder="例如: 9999" />
              </Form.Item>

              <Form.Item
                label="应用标识"
                name="appId"
                rules={[{ required: true, message: '请输入应用标识' }]}
              >
                <Input placeholder="例如: simnow_client" />
              </Form.Item>

              <Form.Item
                label="认证码"
                name="authCode"
                rules={[{ required: true, message: '请输入认证码' }]}
              >
                <Input placeholder="例如: 0000000000000000" />
              </Form.Item>
            </>
          )}

          {currentStep === 1 && (
            <>
              <Form.Item
                label="用户账号"
                name="userId"
                rules={[{ required: true, message: '请输入用户账号' }]}
              >
                <Input placeholder="请输入您的交易账号" />
              </Form.Item>

              <Form.Item
                label="登录密码"
                name="password"
                rules={[{ required: true, message: '请输入登录密码' }]}
              >
                <Input.Password placeholder="请输入您的登录密码" />
              </Form.Item>
            </>
          )}

          {currentStep === 2 && (
            <div style={{ textAlign: 'center', padding: '20px 0' }}>
              <CheckCircleOutlined style={{ fontSize: 48, color: '#52c41a' }} />
              <h3 style={{ marginTop: 16 }}>连接成功！</h3>
              <p>正在初始化交易界面...</p>
            </div>
          )}

          {connectionError && (
            <Alert
              message="连接错误"
              description={connectionError}
              type="error"
              showIcon
              closable
              onClose={() => setConnectionError('')}
              style={{ marginBottom: 16 }}
            />
          )}

          {currentStep < 2 && (
            <Form.Item>
              <Space style={{ width: '100%', justifyContent: 'flex-end' }}>
                {currentStep > 0 && (
                  <Button onClick={() => setCurrentStep(currentStep - 1)} disabled={loading}>
                    上一步
                  </Button>
                )}
                <Button type="primary" htmlType="submit" loading={loading}>
                  {currentStep === 0 ? '连接服务器' : '登录账户'}
                </Button>
              </Space>
            </Form.Item>
          )}
        </Form>
      </Spin>
    </Modal>
  );
};

export default CtpConnectionDialog;
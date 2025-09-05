import React, { useState, useEffect } from 'react';
import { 
  Modal, 
  Form, 
  Input, 
  Select, 
  Button, 
  Steps, 
  message, 
  Alert, 
  Spin, 
  Space, 
  Card,
  Typography,
  Divider,
  Row,
  Col
} from 'antd';
import { 
  ApiOutlined, 
  LoginOutlined, 
  CheckCircleOutlined, 
  LoadingOutlined,
  EnvironmentOutlined,
  UserOutlined,
  LockOutlined,
  InfoCircleOutlined
} from '@ant-design/icons';
import { ctpService } from '@/services/tauri';
import { useMarketDataStore } from '@/stores/marketData';
import type { CtpConfig } from '@/types';
import { ConnectionStatus } from '@/types';
import { CTP_PRESETS, type CtpPreset } from '@/config/ctp-presets';

const { Title, Text, Paragraph } = Typography;

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
  const [connectionError, setConnectionError] = useState<string>('');
  const [selectedPreset, setSelectedPreset] = useState<CtpPreset | null>(null);
  const [isCustomConfig, setIsCustomConfig] = useState(false);
  
  const { setConnectionStatus } = useMarketDataStore();

  useEffect(() => {
    if (visible) {
      // 重置状态
      setCurrentStep(0);
      setConnectionError('');
      setSelectedPreset(null);
      setIsCustomConfig(false);
      form.resetFields();
      
      // 开发环境默认选择广州期货评测环境
      const defaultPreset = CTP_PRESETS.gzqh_test;
      handlePresetSelect(defaultPreset.key);
    }
  }, [visible]);

  const handlePresetSelect = (presetKey: string) => {
    const preset = CTP_PRESETS[presetKey];
    if (!preset) return;

    setSelectedPreset(preset);
    setIsCustomConfig(presetKey === 'production_template');

    // 自动填充表单
    const formValues: any = {
      environment: preset.key,
      md_front_addr: preset.md_front_addr,
      trader_front_addr: preset.trader_front_addr,
      broker_id: preset.broker_id,
      app_id: preset.app_id,
      auth_code: preset.auth_code,
    };

    // 开发环境自动填充默认账号密码
    if (preset.defaultInvestorId && preset.defaultPassword) {
      formValues.investor_id = preset.defaultInvestorId;
      formValues.password = preset.defaultPassword;
    }

    form.setFieldsValue(formValues);
  };

  const handleConnect = async () => {
    try {
      const values = await form.validateFields();
      setLoading(true);
      setConnectionError('');
      setConnectionStatus(ConnectionStatus.CONNECTING);

      // 初始化 CTP
      await ctpService.init();

      // 构建完整的连接配置
      // 对于广州期货评测环境，使用 production 环境配置
      const actualEnvironment = values.environment === 'gzqh_test' ? 'production' : 
                               values.environment === 'production_template' ? 'production' : 
                               values.environment;
      
      const connectionConfig: CtpConfig = {
        environment: actualEnvironment,
        md_front_addr: values.md_front_addr,
        trader_front_addr: values.trader_front_addr,
        broker_id: values.broker_id,
        investor_id: values.investor_id || '', // 用户ID即投资者代码
        password: values.password || '',
        app_id: values.app_id,
        auth_code: values.auth_code || '0000000000000000',
        flow_path: `./ctp_flow/${actualEnvironment}/`,
        timeout_secs: 30,
        reconnect_interval_secs: 5,
        max_reconnect_attempts: 3,
      };

      // 连接到 CTP 服务器
      await ctpService.connect(connectionConfig);
      
      message.success('连接成功！');
      
      // 如果在第一步就有用户名密码，直接登录
      if (values.investor_id && values.password) {
        await handleDirectLogin(values);
      } else {
        setCurrentStep(1);
      }
      
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

  const handleDirectLogin = async (values: any) => {
    try {
      setLoading(true);
      
      // 直接使用表单中的值进行登录
      await ctpService.login({
        brokerId: values.broker_id,
        userId: values.investor_id,
        password: values.password,
        appId: values.app_id,
        authCode: values.auth_code || '0000000000000000',
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
      throw new Error('登录失败: ' + errorMsg);
    }
  };

  const handleLogin = async () => {
    try {
      const values = await form.validateFields(['investor_id_step2', 'password_step2']);
      setLoading(true);
      setConnectionError('');
      
      const formValues = form.getFieldsValue();

      // 登录
      await ctpService.login({
        brokerId: formValues.broker_id,
        userId: values.investor_id_step2,
        password: values.password_step2,
        appId: formValues.app_id,
        authCode: formValues.auth_code || '0000000000000000',
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
    setSelectedPreset(null);
    onClose();
  };

  return (
    <Modal
      title="连接 CTP 交易系统"
      open={visible}
      onCancel={handleCancel}
      width={720}
      footer={null}
      maskClosable={false}
    >
      <Steps
        current={currentStep}
        items={[
          {
            title: '选择环境',
            icon: currentStep === 0 && loading ? <LoadingOutlined /> : <EnvironmentOutlined />,
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
                label="选择交易环境"
                name="environment"
                rules={[{ required: true, message: '请选择交易环境' }]}
              >
                <Select 
                  size="large"
                  placeholder="请选择预置的交易环境"
                  onChange={handlePresetSelect}
                >
                  {Object.values(CTP_PRESETS).map(preset => (
                    <Select.Option key={preset.key} value={preset.key}>
                      <Space>
                        <EnvironmentOutlined />
                        <span>{preset.label}</span>
                      </Space>
                    </Select.Option>
                  ))}
                </Select>
              </Form.Item>

              {selectedPreset && (
                <>
                  <Alert
                    message={selectedPreset.description}
                    description={selectedPreset.tips}
                    type="info"
                    showIcon
                    icon={<InfoCircleOutlined />}
                    style={{ marginBottom: 16 }}
                  />

                  <Card size="small" title="服务器配置" style={{ marginBottom: 16 }}>
                    <Row gutter={16}>
                      <Col span={12}>
                        <Form.Item
                          label="行情前置地址"
                          name="md_front_addr"
                          rules={[{ required: true, message: '请输入行情前置地址' }]}
                        >
                          <Input 
                            placeholder="tcp://x.x.x.x:port" 
                            disabled={!isCustomConfig}
                          />
                        </Form.Item>
                      </Col>
                      <Col span={12}>
                        <Form.Item
                          label="交易前置地址"
                          name="trader_front_addr"
                          rules={[{ required: true, message: '请输入交易前置地址' }]}
                        >
                          <Input 
                            placeholder="tcp://x.x.x.x:port" 
                            disabled={!isCustomConfig}
                          />
                        </Form.Item>
                      </Col>
                    </Row>

                    <Row gutter={16}>
                      <Col span={8}>
                        <Form.Item
                          label="经纪商代码"
                          name="broker_id"
                          rules={[{ required: true, message: '请输入经纪商代码' }]}
                        >
                          <Input 
                            placeholder="例如: 9999" 
                            disabled={!isCustomConfig}
                          />
                        </Form.Item>
                      </Col>
                      <Col span={8}>
                        <Form.Item
                          label="应用标识"
                          name="app_id"
                        >
                          <Input 
                            placeholder="例如: client_test" 
                            disabled={!isCustomConfig}
                          />
                        </Form.Item>
                      </Col>
                      <Col span={8}>
                        <Form.Item
                          label="认证码"
                          name="auth_code"
                        >
                          <Input 
                            placeholder="默认: 0000000000000000" 
                            disabled={!isCustomConfig}
                          />
                        </Form.Item>
                      </Col>
                    </Row>
                  </Card>

                  <Divider>账户信息（可选，稍后填写）</Divider>

                  <Row gutter={16}>
                    <Col span={12}>
                      <Form.Item
                        label="投资者账号"
                        name="investor_id"
                        tooltip="您的交易账号，也称为投资者代码"
                      >
                        <Input 
                          prefix={<UserOutlined />}
                          placeholder="请输入您的交易账号" 
                        />
                      </Form.Item>
                    </Col>
                    <Col span={12}>
                      <Form.Item
                        label="登录密码"
                        name="password"
                      >
                        <Input.Password 
                          prefix={<LockOutlined />}
                          placeholder="请输入您的登录密码" 
                        />
                      </Form.Item>
                    </Col>
                  </Row>

                  <Alert
                    message="提示"
                    description="如果现在输入账号密码，连接成功后将自动登录；否则将在下一步要求输入。"
                    type="info"
                    showIcon
                    style={{ marginTop: 16 }}
                  />
                </>
              )}
            </>
          )}

          {currentStep === 1 && (
            <>
              <Alert
                message="服务器连接成功"
                description="请输入您的交易账号和密码进行登录"
                type="success"
                showIcon
                style={{ marginBottom: 24 }}
              />

              <Row gutter={16}>
                <Col span={12}>
                  <Form.Item
                    label="投资者账号"
                    name="investor_id_step2"
                    rules={[{ required: true, message: '请输入投资者账号' }]}
                    tooltip="您的交易账号，也称为投资者代码"
                  >
                    <Input 
                      size="large"
                      prefix={<UserOutlined />}
                      placeholder="请输入您的交易账号" 
                    />
                  </Form.Item>
                </Col>
                <Col span={12}>
                  <Form.Item
                    label="登录密码"
                    name="password_step2"
                    rules={[{ required: true, message: '请输入登录密码' }]}
                  >
                    <Input.Password 
                      size="large"
                      prefix={<LockOutlined />}
                      placeholder="请输入您的登录密码" 
                    />
                  </Form.Item>
                </Col>
              </Row>
            </>
          )}

          {currentStep === 2 && (
            <div style={{ textAlign: 'center', padding: '40px 0' }}>
              <CheckCircleOutlined style={{ fontSize: 64, color: '#52c41a' }} />
              <Title level={3} style={{ marginTop: 24 }}>连接成功！</Title>
              <Paragraph type="secondary">
                正在初始化交易界面，即将自动关闭...
              </Paragraph>
            </div>
          )}

          {connectionError && (
            <Alert
              message="错误"
              description={connectionError}
              type="error"
              showIcon
              closable
              onClose={() => setConnectionError('')}
              style={{ marginBottom: 16 }}
            />
          )}

          {currentStep < 2 && (
            <Form.Item style={{ marginTop: 24, marginBottom: 0 }}>
              <Space style={{ width: '100%', justifyContent: 'flex-end' }}>
                <Button onClick={handleCancel} disabled={loading}>
                  取消
                </Button>
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
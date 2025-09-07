import React, { useState } from 'react';
import {
  Card,
  Form,
  Switch,
  Select,
  InputNumber,
  Button,
  Divider,
  Space,
  Radio,
  Slider,
  ColorPicker,
  Tabs,
  Alert,
  message,
  Upload,
  Input
} from 'antd';
import {
  BgColorsOutlined,
  FontSizeOutlined,
  GlobalOutlined,
  KeyOutlined,
  SaveOutlined,
  UploadOutlined,
  DownloadOutlined,
  ReloadOutlined,
  SettingOutlined,
  SoundOutlined,
  BellOutlined
} from '@ant-design/icons';
import { useUIStore } from '@/stores/uiStore';
import './SettingsPanel.css';

const { TabPane } = Tabs;
const { Option } = Select;

export const SettingsPanel: React.FC = () => {
  const {
    theme,
    setTheme,
    language,
    setLanguage,
    fontSize,
    setFontSize,
    soundEnabled,
    setSoundEnabled,
    notificationEnabled,
    setNotificationEnabled,
    chartConfig,
    updateChartConfig,
    shortcuts,
    updateShortcuts,
    saveSettings,
    loadSettings,
    resetSettings
  } = useUIStore();

  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);

  // 保存设置
  const handleSaveSettings = async () => {
    setLoading(true);
    try {
      const values = form.getFieldsValue();
      await saveSettings(values);
      message.success('设置已保存');
    } catch (error) {
      message.error('保存设置失败');
    } finally {
      setLoading(false);
    }
  };

  // 导出设置
  const handleExportSettings = () => {
    const settings = {
      theme,
      language,
      fontSize,
      soundEnabled,
      notificationEnabled,
      chartConfig,
      shortcuts,
      ...form.getFieldsValue()
    };
    
    const blob = new Blob([JSON.stringify(settings, null, 2)], { 
      type: 'application/json' 
    });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = 'trading-settings.json';
    link.click();
    message.success('设置已导出');
  };

  // 导入设置
  const handleImportSettings = (file: any) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const settings = JSON.parse(e.target?.result as string);
        loadSettings(settings);
        form.setFieldsValue(settings);
        message.success('设置已导入');
      } catch (error) {
        message.error('导入设置失败');
      }
    };
    reader.readAsText(file);
    return false;
  };

  // 重置设置
  const handleResetSettings = () => {
    resetSettings();
    form.resetFields();
    message.success('设置已重置');
  };

  return (
    <Card className="settings-panel" title="系统设置">
      <Tabs defaultActiveKey="appearance">
        {/* 外观设置 */}
        <TabPane 
          tab={
            <span>
              <BgColorsOutlined />
              外观设置
            </span>
          } 
          key="appearance"
        >
          <Form form={form} layout="vertical">
            <Form.Item label="主题模式" name="theme">
              <Radio.Group 
                value={theme} 
                onChange={e => setTheme(e.target.value)}
                buttonStyle="solid"
              >
                <Radio.Button value="dark">深色模式</Radio.Button>
                <Radio.Button value="light">浅色模式</Radio.Button>
                <Radio.Button value="auto">跟随系统</Radio.Button>
              </Radio.Group>
            </Form.Item>

            <Form.Item label="主题颜色" name="primaryColor">
              <Space>
                <div 
                  className="color-picker"
                  style={{ backgroundColor: '#1890ff' }}
                  onClick={() => {}}
                />
                <div 
                  className="color-picker"
                  style={{ backgroundColor: '#52c41a' }}
                  onClick={() => {}}
                />
                <div 
                  className="color-picker"
                  style={{ backgroundColor: '#f5222d' }}
                  onClick={() => {}}
                />
                <div 
                  className="color-picker"
                  style={{ backgroundColor: '#fa8c16' }}
                  onClick={() => {}}
                />
                <div 
                  className="color-picker"
                  style={{ backgroundColor: '#722ed1' }}
                  onClick={() => {}}
                />
              </Space>
            </Form.Item>

            <Form.Item label="字体大小" name="fontSize">
              <Slider
                min={12}
                max={20}
                value={fontSize}
                onChange={setFontSize}
                marks={{
                  12: '小',
                  14: '标准',
                  16: '中',
                  18: '大',
                  20: '特大'
                }}
              />
            </Form.Item>

            <Form.Item label="界面缩放" name="zoom">
              <Slider
                min={80}
                max={120}
                defaultValue={100}
                marks={{
                  80: '80%',
                  90: '90%',
                  100: '100%',
                  110: '110%',
                  120: '120%'
                }}
                tooltip={{ formatter: (value) => `${value}%` }}
              />
            </Form.Item>

            <Form.Item label="紧凑模式" name="compactMode">
              <Switch defaultChecked={false} />
            </Form.Item>
          </Form>
        </TabPane>

        {/* 语言设置 */}
        <TabPane 
          tab={
            <span>
              <GlobalOutlined />
              语言设置
            </span>
          } 
          key="language"
        >
          <Form layout="vertical">
            <Form.Item label="界面语言" name="language">
              <Select value={language} onChange={setLanguage}>
                <Option value="zh-CN">简体中文</Option>
                <Option value="zh-TW">繁體中文</Option>
                <Option value="en-US">English</Option>
              </Select>
            </Form.Item>

            <Form.Item label="数字格式" name="numberFormat">
              <Select defaultValue="cn">
                <Option value="cn">中式 (1,234.56)</Option>
                <Option value="us">美式 (1,234.56)</Option>
                <Option value="eu">欧式 (1.234,56)</Option>
              </Select>
            </Form.Item>

            <Form.Item label="日期格式" name="dateFormat">
              <Select defaultValue="YYYY-MM-DD">
                <Option value="YYYY-MM-DD">2024-01-01</Option>
                <Option value="MM/DD/YYYY">01/01/2024</Option>
                <Option value="DD/MM/YYYY">01/01/2024</Option>
              </Select>
            </Form.Item>

            <Form.Item label="时间格式" name="timeFormat">
              <Radio.Group defaultValue="24">
                <Radio value="24">24小时制</Radio>
                <Radio value="12">12小时制</Radio>
              </Radio.Group>
            </Form.Item>
          </Form>
        </TabPane>

        {/* 交易设置 */}
        <TabPane 
          tab={
            <span>
              <SettingOutlined />
              交易设置
            </span>
          } 
          key="trading"
        >
          <Form layout="vertical">
            <Form.Item label="默认下单数量" name="defaultVolume">
              <InputNumber min={1} defaultValue={1} style={{ width: '100%' }} />
            </Form.Item>

            <Form.Item label="价格跳动单位" name="priceTick">
              <InputNumber min={0.01} step={0.01} defaultValue={0.01} style={{ width: '100%' }} />
            </Form.Item>

            <Form.Item label="下单确认" name="orderConfirm">
              <Switch defaultChecked />
            </Form.Item>

            <Form.Item label="一键平仓确认" name="closeAllConfirm">
              <Switch defaultChecked />
            </Form.Item>

            <Form.Item label="止损止盈默认值" name="stopLossDefault">
              <Space>
                <InputNumber 
                  placeholder="止损点数" 
                  min={0} 
                  style={{ width: 120 }} 
                />
                <InputNumber 
                  placeholder="止盈点数" 
                  min={0} 
                  style={{ width: 120 }} 
                />
              </Space>
            </Form.Item>

            <Form.Item label="滑点设置" name="slippage">
              <InputNumber 
                min={0} 
                max={10} 
                defaultValue={2} 
                suffix="跳"
                style={{ width: '100%' }} 
              />
            </Form.Item>
          </Form>
        </TabPane>

        {/* 快捷键设置 */}
        <TabPane 
          tab={
            <span>
              <KeyOutlined />
              快捷键
            </span>
          } 
          key="shortcuts"
        >
          <Alert 
            message="提示" 
            description="点击输入框后按下想要设置的快捷键组合" 
            type="info" 
            showIcon 
            style={{ marginBottom: 16 }}
          />
          
          <Form layout="vertical">
            <Form.Item label="快速买入" name="shortcutBuy">
              <Input 
                placeholder="例如: Ctrl+B" 
                defaultValue="Ctrl+B"
                onKeyDown={(e) => {
                  e.preventDefault();
                  const keys = [];
                  if (e.ctrlKey) keys.push('Ctrl');
                  if (e.altKey) keys.push('Alt');
                  if (e.shiftKey) keys.push('Shift');
                  if (e.key && e.key !== 'Control' && e.key !== 'Alt' && e.key !== 'Shift') {
                    keys.push(e.key.toUpperCase());
                  }
                  if (keys.length > 1) {
                    e.currentTarget.value = keys.join('+');
                  }
                }}
              />
            </Form.Item>

            <Form.Item label="快速卖出" name="shortcutSell">
              <Input placeholder="例如: Ctrl+S" defaultValue="Ctrl+S" />
            </Form.Item>

            <Form.Item label="一键平仓" name="shortcutCloseAll">
              <Input placeholder="例如: Ctrl+Shift+C" defaultValue="Ctrl+Shift+C" />
            </Form.Item>

            <Form.Item label="撤销所有订单" name="shortcutCancelAll">
              <Input placeholder="例如: Ctrl+Shift+X" defaultValue="Ctrl+Shift+X" />
            </Form.Item>

            <Form.Item label="切换图表" name="shortcutSwitchChart">
              <Input placeholder="例如: Ctrl+Tab" defaultValue="Ctrl+Tab" />
            </Form.Item>

            <Form.Item label="全屏模式" name="shortcutFullscreen">
              <Input placeholder="例如: F11" defaultValue="F11" />
            </Form.Item>
          </Form>
        </TabPane>

        {/* 声音通知 */}
        <TabPane 
          tab={
            <span>
              <SoundOutlined />
              声音通知
            </span>
          } 
          key="notification"
        >
          <Form layout="vertical">
            <Form.Item label="启用声音提醒" name="soundEnabled">
              <Switch checked={soundEnabled} onChange={setSoundEnabled} />
            </Form.Item>

            <Form.Item label="音量" name="volume">
              <Slider 
                min={0} 
                max={100} 
                defaultValue={50}
                disabled={!soundEnabled}
              />
            </Form.Item>

            <Divider />

            <Form.Item label="启用系统通知" name="notificationEnabled">
              <Switch checked={notificationEnabled} onChange={setNotificationEnabled} />
            </Form.Item>

            <Form.Item label="通知设置">
              <Space direction="vertical" style={{ width: '100%' }}>
                <Switch checkedChildren="成交通知" unCheckedChildren="成交通知" defaultChecked />
                <Switch checkedChildren="撤单通知" unCheckedChildren="撤单通知" defaultChecked />
                <Switch checkedChildren="价格预警" unCheckedChildren="价格预警" defaultChecked />
                <Switch checkedChildren="风险提醒" unCheckedChildren="风险提醒" defaultChecked />
                <Switch checkedChildren="系统消息" unCheckedChildren="系统消息" defaultChecked />
              </Space>
            </Form.Item>
          </Form>
        </TabPane>
      </Tabs>

      <Divider />

      {/* 操作按钮 */}
      <Space style={{ width: '100%', justifyContent: 'space-between' }}>
        <Space>
          <Upload
            accept=".json"
            showUploadList={false}
            beforeUpload={handleImportSettings}
          >
            <Button icon={<UploadOutlined />}>导入设置</Button>
          </Upload>
          <Button icon={<DownloadOutlined />} onClick={handleExportSettings}>
            导出设置
          </Button>
          <Button icon={<ReloadOutlined />} onClick={handleResetSettings}>
            重置默认
          </Button>
        </Space>
        
        <Button 
          type="primary" 
          icon={<SaveOutlined />}
          loading={loading}
          onClick={handleSaveSettings}
        >
          保存设置
        </Button>
      </Space>
    </Card>
  );
};

export default SettingsPanel;
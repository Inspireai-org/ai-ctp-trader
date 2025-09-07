# TTS 交易通道 UI 集成设计文档

## 概述

本设计文档描述了如何在现有的 CTP 交易系统 UI 中集成 TTS（Test Trading System）交易通道配置，以便开发者在周末或非交易时间进行开发测试。设计将基于现有的预设配置系统，扩展支持 TTS 环境，并提供便捷的开发测试功能。

## 架构

### 整体架构
```
┌─────────────────────────────────────────────────────────────┐
│                    前端 UI 层                                │
├─────────────────────────────────────────────────────────────┤
│  CtpConnectionDialog  │  SettingsPanel  │  EnvironmentSwitch │
├─────────────────────────────────────────────────────────────┤
│                   配置管理层                                 │
├─────────────────────────────────────────────────────────────┤
│  CTP_PRESETS  │  EnvironmentManager  │  ConfigValidator     │
├─────────────────────────────────────────────────────────────┤
│                   服务层                                     │
├─────────────────────────────────────────────────────────────┤
│  CtpServiceManager  │  ConnectionManager  │  StateManager   │
├─────────────────────────────────────────────────────────────┤
│                   后端 Tauri 层                              │
├─────────────────────────────────────────────────────────────┤
│  TTS Config  │  CTP Client  │  Event System  │  Logging     │
└─────────────────────────────────────────────────────────────┘
```

### 数据流
```
用户选择环境 → 预设配置加载 → 参数验证 → 连接建立 → 状态同步 → UI 更新
```

## 组件和接口

### 1. 配置预设扩展

#### 1.1 TTS 预设配置
在 `ctp-presets.ts` 中添加 TTS 相关的预设配置：

```typescript
// 新增 TTS 预设配置
export const TTS_PRESETS: Record<string, CtpPreset> = {
  tts_openctp: {
    key: 'tts_openctp',
    label: 'TTS - OpenCTP 测试',
    description: 'OpenCTP 提供的 TTS 测试环境，支持 7x24 小时交易测试',
    md_front_addr: 'tcp://121.37.80.177:20004',
    trader_front_addr: 'tcp://121.37.80.177:20002',
    broker_id: '9999',
    app_id: 'simnow_client_test',
    auth_code: '0000000000000000',
    requireCredentials: true,
    tips: '🔧 TTS 测试环境 - 适合周末和非交易时间开发测试',
    defaultInvestorId: 'test001',
    defaultPassword: 'test123',
    category: 'tts',
    features: ['7x24小时', '模拟交易', '开发测试'],
    connectionTimeout: 10000,
    isWeekendAvailable: true
  },
  tts_local: {
    key: 'tts_local',
    label: 'TTS - 本地测试',
    description: '本地 TTS 测试环境，用于离线开发和调试',
    md_front_addr: 'tcp://127.0.0.1:20004',
    trader_front_addr: 'tcp://127.0.0.1:20002',
    broker_id: '9999',
    app_id: 'local_test',
    auth_code: '0000000000000000',
    requireCredentials: false,
    tips: '💻 本地测试环境 - 需要先启动本地 TTS 服务',
    defaultInvestorId: 'local',
    defaultPassword: 'local',
    category: 'tts',
    features: ['本地部署', '离线测试', '快速调试'],
    connectionTimeout: 5000,
    isWeekendAvailable: true
  }
};
```

#### 1.2 预设配置接口扩展
```typescript
export interface CtpPreset {
  // 现有字段...
  
  // 新增字段
  category?: 'production' | 'simulation' | 'tts' | 'development';
  features?: string[];
  connectionTimeout?: number;
  isWeekendAvailable?: boolean;
  priority?: number;
  healthCheckUrl?: string;
  documentationUrl?: string;
}
```

### 2. 环境管理器

#### 2.1 EnvironmentManager 类
```typescript
export class EnvironmentManager {
  private static instance: EnvironmentManager;
  private currentEnvironment: string | null = null;
  private environmentHistory: string[] = [];
  
  // 环境检测和验证
  async detectAvailableEnvironments(): Promise<EnvironmentStatus[]>;
  async validateEnvironment(presetKey: string): Promise<ValidationResult>;
  
  // 环境切换
  async switchEnvironment(presetKey: string): Promise<void>;
  async getRecommendedEnvironment(): Promise<string>;
  
  // 周末模式检测
  isWeekendMode(): boolean;
  getWeekendRecommendations(): CtpPreset[];
  
  // 环境状态管理
  getEnvironmentStatus(presetKey: string): Promise<EnvironmentStatus>;
  monitorEnvironmentHealth(): void;
}
```

#### 2.2 环境状态接口
```typescript
export interface EnvironmentStatus {
  presetKey: string;
  isAvailable: boolean;
  responseTime?: number;
  lastChecked: Date;
  errorMessage?: string;
  features: string[];
  recommendedForWeekend: boolean;
}

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
  suggestions: string[];
}
```

### 3. UI 组件增强

#### 3.1 连接对话框增强
在 `CtpConnectionDialog.tsx` 中添加以下功能：

1. **环境分类显示**
```typescript
// 环境分组显示
const groupedPresets = useMemo(() => {
  const groups = {
    tts: [],
    simulation: [],
    production: []
  };
  
  Object.values(CTP_PRESETS).forEach(preset => {
    const category = preset.category || 'simulation';
    groups[category].push(preset);
  });
  
  return groups;
}, []);
```

2. **周末模式检测**
```typescript
// 周末模式提示
const isWeekend = useMemo(() => {
  const now = new Date();
  const day = now.getDay();
  return day === 0 || day === 6; // 周日或周六
}, []);

// 推荐环境显示
const recommendedPresets = useMemo(() => {
  if (isWeekend) {
    return Object.values(CTP_PRESETS).filter(preset => preset.isWeekendAvailable);
  }
  return Object.values(CTP_PRESETS);
}, [isWeekend]);
```

3. **环境状态指示器**
```typescript
// 环境状态组件
const EnvironmentStatusIndicator: React.FC<{ preset: CtpPreset }> = ({ preset }) => {
  const [status, setStatus] = useState<EnvironmentStatus | null>(null);
  
  useEffect(() => {
    checkEnvironmentStatus(preset.key).then(setStatus);
  }, [preset.key]);
  
  return (
    <Space>
      <Badge 
        status={status?.isAvailable ? 'success' : 'error'} 
        text={status?.isAvailable ? '可用' : '不可用'}
      />
      {status?.responseTime && (
        <Text type="secondary">{status.responseTime}ms</Text>
      )}
    </Space>
  );
};
```

#### 3.2 快速连接组件
```typescript
// 快速连接组件
const QuickConnectPanel: React.FC = () => {
  const isWeekend = useIsWeekend();
  const recommendedPreset = useRecommendedPreset();
  
  return (
    <Card size="small" title="快速连接">
      {isWeekend && (
        <Alert
          message="周末模式"
          description="检测到当前为周末，推荐使用 TTS 测试环境进行开发"
          type="info"
          showIcon
          style={{ marginBottom: 16 }}
        />
      )}
      
      <Button
        type="primary"
        size="large"
        block
        onClick={() => handleQuickConnect(recommendedPreset)}
      >
        连接到 {recommendedPreset?.label}
      </Button>
      
      <Text type="secondary" style={{ display: 'block', textAlign: 'center', marginTop: 8 }}>
        使用默认测试账号快速连接
      </Text>
    </Card>
  );
};
```

### 4. 设置面板集成

#### 4.1 环境管理设置
在 `SettingsPanel.tsx` 中添加环境管理选项：

```typescript
// 环境偏好设置
const EnvironmentPreferences: React.FC = () => {
  return (
    <Card title="环境偏好设置">
      <Form.Item label="默认环境">
        <Select placeholder="选择默认连接环境">
          {Object.values(CTP_PRESETS).map(preset => (
            <Select.Option key={preset.key} value={preset.key}>
              {preset.label}
            </Select.Option>
          ))}
        </Select>
      </Form.Item>
      
      <Form.Item label="周末自动切换">
        <Switch 
          checkedChildren="启用" 
          unCheckedChildren="禁用"
          defaultChecked={true}
        />
        <Text type="secondary" style={{ marginLeft: 8 }}>
          周末自动推荐 TTS 环境
        </Text>
      </Form.Item>
      
      <Form.Item label="连接超时">
        <InputNumber
          min={5}
          max={60}
          defaultValue={30}
          addonAfter="秒"
        />
      </Form.Item>
      
      <Form.Item label="自动重连">
        <Switch defaultChecked={true} />
      </Form.Item>
    </Card>
  );
};
```

## 数据模型

### 1. 配置数据模型
```typescript
// TTS 特定配置
export interface TtsConfig extends CtpConfig {
  ttsMode: 'openctp' | 'local' | 'custom';
  simulationFeatures: {
    enableLatencySimulation: boolean;
    latencyRange: [number, number];
    enableSlippage: boolean;
    slippageRate: number;
    enablePartialFill: boolean;
  };
  testDataSets: string[];
  mockMarketData: boolean;
}

// 环境配置存储
export interface EnvironmentConfig {
  defaultEnvironment: string;
  autoSwitchWeekend: boolean;
  connectionTimeout: number;
  autoReconnect: boolean;
  environmentHistory: string[];
  customPresets: CtpPreset[];
}
```

### 2. 状态管理模型
```typescript
// 环境状态 Store
export interface EnvironmentStore {
  currentEnvironment: string | null;
  availableEnvironments: EnvironmentStatus[];
  isWeekendMode: boolean;
  connectionHistory: ConnectionRecord[];
  
  // Actions
  setCurrentEnvironment: (env: string) => void;
  updateEnvironmentStatus: (status: EnvironmentStatus) => void;
  addConnectionRecord: (record: ConnectionRecord) => void;
  clearHistory: () => void;
}

// 连接记录
export interface ConnectionRecord {
  environment: string;
  timestamp: Date;
  success: boolean;
  duration: number;
  errorMessage?: string;
}
```

## 错误处理

### 1. TTS 特定错误处理
```typescript
export enum TtsErrorCode {
  TTS_SERVICE_UNAVAILABLE = 'TTS_SERVICE_UNAVAILABLE',
  TTS_INVALID_CONFIG = 'TTS_INVALID_CONFIG',
  TTS_CONNECTION_TIMEOUT = 'TTS_CONNECTION_TIMEOUT',
  TTS_AUTHENTICATION_FAILED = 'TTS_AUTHENTICATION_FAILED',
  TTS_WEEKEND_ONLY = 'TTS_WEEKEND_ONLY'
}

export class TtsErrorHandler {
  static handleTtsError(error: CtpError): string {
    switch (error.code) {
      case TtsErrorCode.TTS_SERVICE_UNAVAILABLE:
        return '⚠️ TTS 服务暂时不可用，请稍后重试或选择其他测试环境';
      case TtsErrorCode.TTS_WEEKEND_ONLY:
        return '📅 此 TTS 环境仅在周末提供服务，工作日请使用其他环境';
      default:
        return `TTS 连接错误: ${error.message}`;
    }
  }
  
  static getSuggestions(error: CtpError): string[] {
    const suggestions = [];
    
    if (error.code === TtsErrorCode.TTS_SERVICE_UNAVAILABLE) {
      suggestions.push('尝试连接其他 TTS 环境');
      suggestions.push('检查网络连接');
      suggestions.push('联系技术支持');
    }
    
    return suggestions;
  }
}
```

### 2. 连接重试策略
```typescript
export class TtsConnectionManager {
  private retryConfig = {
    maxRetries: 3,
    baseDelay: 2000,
    maxDelay: 10000,
    backoffFactor: 2
  };
  
  async connectWithRetry(config: TtsConfig): Promise<void> {
    let lastError: CtpError;
    
    for (let attempt = 0; attempt <= this.retryConfig.maxRetries; attempt++) {
      try {
        await this.connect(config);
        return;
      } catch (error) {
        lastError = error as CtpError;
        
        if (attempt === this.retryConfig.maxRetries) {
          throw lastError;
        }
        
        const delay = Math.min(
          this.retryConfig.baseDelay * Math.pow(this.retryConfig.backoffFactor, attempt),
          this.retryConfig.maxDelay
        );
        
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }
}
```

## 测试策略

### 1. 单元测试
- 预设配置验证测试
- 环境管理器功能测试
- 错误处理逻辑测试
- 状态管理测试

### 2. 集成测试
- TTS 连接流程测试
- 环境切换测试
- 周末模式检测测试
- UI 组件交互测试

### 3. 端到端测试
- 完整连接流程测试
- 多环境切换测试
- 错误恢复测试
- 性能测试

### 4. 测试数据
```typescript
// 测试预设配置
export const TEST_PRESETS = {
  tts_mock: {
    key: 'tts_mock',
    label: 'TTS Mock Environment',
    // ... 其他配置
  }
};

// 测试工具
export class TtsTestUtils {
  static createMockEnvironmentStatus(): EnvironmentStatus;
  static simulateWeekendMode(): void;
  static mockTtsConnection(): Promise<void>;
}
```

## 性能考虑

### 1. 连接优化
- 连接池管理
- 预连接机制
- 智能重连策略
- 超时控制

### 2. UI 性能
- 环境状态缓存
- 延迟加载
- 虚拟滚动（如果环境列表很长）
- 防抖处理

### 3. 内存管理
- 连接状态清理
- 事件监听器管理
- 缓存策略
- 垃圾回收优化

## 安全考虑

### 1. 配置安全
- 敏感信息加密存储
- 配置文件权限控制
- 默认密码提醒
- 生产环境隔离

### 2. 连接安全
- SSL/TLS 支持
- 证书验证
- 连接加密
- 访问控制

### 3. 数据安全
- 本地数据加密
- 日志脱敏
- 敏感信息过滤
- 安全审计

## 部署和维护

### 1. 配置管理
- 环境配置版本控制
- 配置热更新
- 配置验证
- 配置备份恢复

### 2. 监控和日志
- 连接状态监控
- 性能指标收集
- 错误日志记录
- 用户行为分析

### 3. 更新和维护
- 预设配置更新机制
- 向后兼容性
- 平滑升级
- 回滚策略
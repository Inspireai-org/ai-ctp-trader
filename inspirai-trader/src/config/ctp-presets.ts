/**
 * CTP 预置环境配置
 * 包含各个交易环境的预设参数
 */

export interface CtpPreset {
  /** 环境标识 */
  key: string;
  /** 显示名称 */
  label: string;
  /** 环境描述 */
  description: string;
  /** 行情前置地址 */
  md_front_addr: string;
  /** 交易前置地址 */
  trader_front_addr: string;
  /** 经纪商代码 */
  broker_id: string;
  /** 应用ID */
  app_id: string;
  /** 认证码 */
  auth_code: string;
  /** 是否需要用户名密码 */
  requireCredentials: boolean;
  /** 提示信息 */
  tips?: string;
  /** 默认投资者代码（仅开发环境使用） */
  defaultInvestorId?: string;
  /** 默认密码（仅开发环境使用） */
  defaultPassword?: string;
  /** 环境分类 */
  category?: 'production' | 'simulation' | 'tts' | 'development';
  /** 环境特性 */
  features?: string[];
  /** 连接超时时间 */
  connectionTimeout?: number;
  /** 是否支持周末使用 */
  isWeekendAvailable?: boolean;
  /** 优先级 */
  priority?: number;
}

/**
 * 预置的CTP环境配置列表
 */
export const CTP_PRESETS: Record<string, CtpPreset> = {
  simnow: {
    key: 'simnow',
    label: 'SimNow 模拟环境',
    description: 'SimNow 7x24小时测试环境，适用于开发测试',
    md_front_addr: 'tcp://180.168.146.187:10131',
    trader_front_addr: 'tcp://180.168.146.187:10130',
    broker_id: '9999',
    app_id: 'simnow_client_test',
    auth_code: '0000000000000000',
    requireCredentials: true,
    tips: '请使用您的 SimNow 账号登录，如未注册请访问 www.simnow.com.cn'
  },
  simnow_7x24: {
    key: 'simnow_7x24',
    label: 'SimNow 7x24',
    description: 'SimNow 7x24小时测试环境',
    md_front_addr: 'tcp://180.168.146.187:10212',
    trader_front_addr: 'tcp://180.168.146.187:10202',
    broker_id: '9999',
    app_id: 'simnow_client_test',
    auth_code: '0000000000000000',
    requireCredentials: true,
    tips: '全天候测试环境，适合自动化测试'
  },
  openctp_sim: {
    key: 'openctp_sim',
    label: 'OpenCTP 仿真环境',
    description: 'OpenCTP 提供的仿真测试环境',
    md_front_addr: 'tcp://121.37.80.177:20004',
    trader_front_addr: 'tcp://121.37.80.177:20002',
    broker_id: '9999',
    app_id: 'test',
    auth_code: '0000000000000000',
    requireCredentials: true,
    tips: '开源社区提供的测试环境'
  },
  gzqh_test: {
    key: 'gzqh_test',
    label: '广州期货-评测环境',
    description: '广州期货评测版本地址',
    md_front_addr: 'tcp://58.62.16.148:41214',
    trader_front_addr: 'tcp://58.62.16.148:41206',
    broker_id: '5071',
    app_id: 'inspirai_strategy_1.0.0',
    auth_code: 'QHFK5E2GLEUB9XHV',
    requireCredentials: true,
    tips: '广州期货评测环境，用于正式交易前的系统测试',
    // 开发阶段默认账号密码（生产环境请删除）
    defaultInvestorId: '00001',
    defaultPassword: 'abc123456'
  },
  production_template: {
    key: 'production_template',
    label: '生产环境（需配置）',
    description: '生产环境模板，需要手动配置服务器地址',
    md_front_addr: '',
    trader_front_addr: '',
    broker_id: '',
    app_id: '',
    auth_code: '',
    requireCredentials: true,
    tips: '⚠️ 生产环境请谨慎操作！请向期货公司获取正确的服务器地址和认证信息',
    category: 'production'
  },
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
    tips: '🔧 TTS 测试环境 - 适合周末和非交易时间开发测试，支持完整的交易功能',
    defaultInvestorId: 'test001',
    defaultPassword: 'test123',
    category: 'tts',
    features: ['7x24小时', '模拟交易', '开发测试', '周末可用'],
    connectionTimeout: 10000,
    isWeekendAvailable: true,
    priority: 1
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
    isWeekendAvailable: true,
    priority: 2
  }
};

/**
 * 获取预置配置
 */
export function getPreset(key: string): CtpPreset | undefined {
  return CTP_PRESETS[key];
}

/**
 * 获取所有预置配置列表
 */
export function getAllPresets(): CtpPreset[] {
  return Object.values(CTP_PRESETS);
}

/**
 * 获取默认预置配置
 * 开发环境使用广州期货评测环境
 */
export function getDefaultPreset(): CtpPreset {
  const preset = CTP_PRESETS.gzqh_test;
  if (!preset) {
    throw new Error('默认预设配置不存在');
  }
  return preset;
}

/**
 * 检查是否为周末
 */
export function isWeekend(): boolean {
  const now = new Date();
  const day = now.getDay();
  return day === 0 || day === 6; // 周日或周六
}

/**
 * 获取推荐的预设配置
 * 周末推荐 TTS 环境，工作日推荐常规环境
 */
export function getRecommendedPreset(): CtpPreset {
  if (isWeekend()) {
    // 周末推荐 TTS 环境
    const ttsPresets = Object.values(CTP_PRESETS).filter(
      preset => preset.category === 'tts' && preset.isWeekendAvailable
    );
    if (ttsPresets.length > 0) {
      // 按优先级排序，返回优先级最高的
      ttsPresets.sort((a, b) => (a.priority || 0) - (b.priority || 0));
      return ttsPresets[0];
    }
  }
  
  // 工作日或没有 TTS 环境时，返回默认环境
  return getDefaultPreset();
}

/**
 * 获取周末可用的预设配置
 */
export function getWeekendPresets(): CtpPreset[] {
  return Object.values(CTP_PRESETS).filter(preset => preset.isWeekendAvailable);
}

/**
 * 按分类获取预设配置
 */
export function getPresetsByCategory(category: string): CtpPreset[] {
  return Object.values(CTP_PRESETS).filter(preset => preset.category === category);
}

/**
 * 获取 TTS 预设配置
 */
export function getTtsPresets(): CtpPreset[] {
  return getPresetsByCategory('tts');
}
/**
 * 全局类型定义文件
 * 
 * 包含应用程序中使用的所有通用类型定义
 * 基于后端 Rust 数据模型创建对应的 TypeScript 接口
 */

// ============================================================================
// 基础类型定义
// ============================================================================

/**
 * 通用响应类型
 */
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  message?: string;
  code?: number;
  timestamp?: number;
}

/**
 * 分页参数
 */
export interface PaginationParams {
  page: number;
  pageSize: number;
  total?: number;
}

/**
 * 排序参数
 */
export interface SortParams {
  field: string;
  order: 'asc' | 'desc';
}

/**
 * 筛选参数
 */
export interface FilterParams {
  [key: string]: any;
}

// ============================================================================
// CTP 相关类型定义（基于后端 Rust 模型）
// ============================================================================

/**
 * 登录凭据
 */
export interface LoginCredentials {
  /** 经纪商代码 */
  brokerId: string;
  /** 用户ID */
  userId: string;
  /** 密码 */
  password: string;
  /** 应用ID */
  appId: string;
  /** 授权码 */
  authCode: string;
}

/**
 * 登录响应
 */
export interface LoginResponse {
  /** 交易日 */
  tradingDay: string;
  /** 登录时间 */
  loginTime: string;
  /** 经纪商代码 */
  brokerId: string;
  /** 用户ID */
  userId: string;
  /** 系统名称 */
  systemName: string;
  /** 前置编号 */
  frontId: number;
  /** 会话编号 */
  sessionId: number;
  /** 最大报单引用 */
  maxOrderRef: string;
}

/**
 * 客户端状态
 */
export enum ClientState {
  /** 未连接 */
  DISCONNECTED = 'DISCONNECTED',
  /** 连接中 */
  CONNECTING = 'CONNECTING',
  /** 已连接 */
  CONNECTED = 'CONNECTED',
  /** 登录中 */
  LOGGING_IN = 'LOGGING_IN',
  /** 已登录 */
  LOGGED_IN = 'LOGGED_IN',
  /** 错误状态 */
  ERROR = 'ERROR',
}

/**
 * 环境类型
 */
export enum Environment {
  /** SimNow 模拟环境 */
  SIMNOW = 'simnow',
  /** TTS 7x24 测试环境 */
  TTS = 'tts',
  /** 生产环境 */
  PRODUCTION = 'production',
}

/**
 * CTP 配置
 * 注意：字段名使用 snake_case 以匹配 Rust 后端结构
 */
export interface CtpConfig {
  /** 环境类型 */
  environment: Environment | string;
  /** 行情前置地址 */
  md_front_addr: string;
  /** 交易前置地址 */
  trader_front_addr: string;
  /** 经纪商代码 */
  broker_id: string;
  /** 投资者代码 */
  investor_id: string;
  /** 密码 */
  password: string;
  /** 应用标识 */
  app_id: string;
  /** 授权编码 */
  auth_code: string;
  /** 流文件路径 */
  flow_path: string;
  /** 行情动态库路径 */
  md_dynlib_path?: string;
  /** 交易动态库路径 */
  td_dynlib_path?: string;
  /** 超时时间（秒） */
  timeout_secs: number;
  /** 重连间隔（秒） */
  reconnect_interval_secs: number;
  /** 最大重连次数 */
  max_reconnect_attempts: number;
}

// ============================================================================
// 市场数据类型
// ============================================================================

/**
 * 行情数据（基于后端 MarketDataTick）
 */
export interface MarketDataTick {
  /** 合约代码 */
  instrumentId: string;
  /** 最新价 */
  lastPrice: number;
  /** 成交量 */
  volume: number;
  /** 成交额 */
  turnover: number;
  /** 持仓量 */
  openInterest: number;
  /** 买一价 */
  bidPrice1: number;
  /** 买一量 */
  bidVolume1: number;
  /** 卖一价 */
  askPrice1: number;
  /** 卖一量 */
  askVolume1: number;
  /** 更新时间 */
  updateTime: string;
  /** 更新毫秒 */
  updateMillisec: number;
  /** 涨跌幅 */
  changePercent: number;
  /** 涨跌额 */
  changeAmount: number;
  /** 今开盘 */
  openPrice: number;
  /** 最高价 */
  highestPrice: number;
  /** 最低价 */
  lowestPrice: number;
  /** 昨收盘 */
  preClosePrice: number;
}

/**
 * K线数据
 */
export interface KlineData {
  /** 时间戳 */
  timestamp: number;
  /** 开盘价 */
  open: number;
  /** 最高价 */
  high: number;
  /** 最低价 */
  low: number;
  /** 收盘价 */
  close: number;
  /** 成交量 */
  volume: number;
  /** 成交额 */
  turnover?: number;
  /** 持仓量 */
  openInterest?: number;
}

/**
 * 时间周期枚举
 */
export enum TimeFrame {
  M1 = '1m',
  M5 = '5m',
  M15 = '15m',
  M30 = '30m',
  H1 = '1h',
  H4 = '4h',
  D1 = '1d',
  W1 = '1w',
  MN1 = '1M',
}

// ============================================================================
// 交易相关类型（基于后端 Rust 模型）
// ============================================================================

/**
 * 买卖方向
 */
export enum OrderDirection {
  /** 买入 */
  BUY = 'Buy',
  /** 卖出 */
  SELL = 'Sell',
}

/**
 * 开平仓标志
 */
export enum OffsetFlag {
  /** 开仓 */
  OPEN = 'Open',
  /** 平仓 */
  CLOSE = 'Close',
  /** 平今 */
  CLOSE_TODAY = 'CloseToday',
  /** 平昨 */
  CLOSE_YESTERDAY = 'CloseYesterday',
}

/**
 * 订单类型
 */
export enum OrderType {
  /** 限价单 */
  LIMIT = 'Limit',
  /** 市价单 */
  MARKET = 'Market',
  /** 条件单 */
  CONDITIONAL = 'Conditional',
}

/**
 * 时间条件
 */
export enum TimeCondition {
  /** 立即完成，否则撤销 */
  IOC = 'IOC',
  /** 全部成交或撤销 */
  FOK = 'FOK',
  /** 当日有效 */
  GFD = 'GFD',
}

/**
 * 订单状态类型
 */
export enum OrderStatusType {
  /** 未知 */
  UNKNOWN = 'Unknown',
  /** 全部成交 */
  ALL_TRADED = 'AllTraded',
  /** 部分成交还在队列中 */
  PART_TRADED_QUEUEING = 'PartTradedQueueing',
  /** 部分成交不在队列中 */
  PART_TRADED_NOT_QUEUEING = 'PartTradedNotQueueing',
  /** 未成交还在队列中 */
  NO_TRADE_QUEUEING = 'NoTradeQueueing',
  /** 未成交不在队列中 */
  NO_TRADE_NOT_QUEUEING = 'NoTradeNotQueueing',
  /** 撤单 */
  CANCELED = 'Canceled',
  /** 触发 */
  TOUCHED = 'Touched',
}

/**
 * 操作标志
 */
export enum ActionFlag {
  /** 删除（撤单） */
  DELETE = 'Delete',
}

/**
 * 订单请求（基于后端 OrderRequest）
 */
export interface OrderRequest {
  /** 合约代码 */
  instrumentId: string;
  /** 买卖方向 */
  direction: OrderDirection;
  /** 开平仓标志 */
  offsetFlag: OffsetFlag;
  /** 价格 */
  price: number;
  /** 数量 */
  volume: number;
  /** 订单类型 */
  orderType: OrderType;
  /** 时间条件 */
  timeCondition: TimeCondition;
}

/**
 * 撤单请求（基于后端 OrderAction）
 */
export interface OrderAction {
  /** 订单号 */
  orderId: string;
  /** 合约代码 */
  instrumentId: string;
  /** 操作类型（撤单） */
  actionFlag: ActionFlag;
}

/**
 * 订单状态（基于后端 OrderStatus）
 */
export interface OrderStatus {
  /** 订单号 */
  orderId: string;
  /** 合约代码 */
  instrumentId: string;
  /** 买卖方向 */
  direction: OrderDirection;
  /** 开平仓标志 */
  offsetFlag: OffsetFlag;
  /** 委托价格 */
  limitPrice: number;
  /** 委托数量 */
  volumeTotalOriginal: number;
  /** 成交数量 */
  volumeTraded: number;
  /** 剩余数量 */
  volumeTotal: number;
  /** 订单状态 */
  status: OrderStatusType;
  /** 委托时间 */
  insertTime: string;
  /** 更新时间 */
  updateTime: string;
  /** 状态信息 */
  statusMsg?: string;
}

/**
 * 成交记录（基于后端 TradeRecord）
 */
export interface TradeRecord {
  /** 成交编号 */
  tradeId: string;
  /** 订单号 */
  orderId: string;
  /** 合约代码 */
  instrumentId: string;
  /** 买卖方向 */
  direction: OrderDirection;
  /** 开平仓标志 */
  offsetFlag: OffsetFlag;
  /** 成交价格 */
  price: number;
  /** 成交数量 */
  volume: number;
  /** 成交时间 */
  tradeTime: string;
}

/**
 * 持仓方向
 */
export enum PositionDirection {
  /** 多头 */
  LONG = 'Long',
  /** 空头 */
  SHORT = 'Short',
}

/**
 * 持仓信息（基于后端 Position）
 */
export interface Position {
  /** 合约代码 */
  instrumentId: string;
  /** 持仓方向 */
  direction: PositionDirection;
  /** 总持仓 */
  totalPosition: number;
  /** 昨持仓 */
  yesterdayPosition: number;
  /** 今持仓 */
  todayPosition: number;
  /** 开仓成本 */
  openCost: number;
  /** 持仓成本 */
  positionCost: number;
  /** 占用保证金 */
  margin: number;
  /** 浮动盈亏 */
  unrealizedPnl: number;
  /** 平仓盈亏 */
  realizedPnl: number;
}

// ============================================================================
// 账户相关类型（基于后端 Rust 模型）
// ============================================================================

/**
 * 账户信息（基于后端 AccountInfo）
 */
export interface AccountInfo {
  /** 账户代码 */
  accountId: string;
  /** 可用资金 */
  available: number;
  /** 账户余额 */
  balance: number;
  /** 冻结资金 */
  frozenMargin: number;
  /** 冻结手续费 */
  frozenCommission: number;
  /** 当前保证金总额 */
  currMargin: number;
  /** 手续费 */
  commission: number;
  /** 平仓盈亏 */
  closeProfit: number;
  /** 持仓盈亏 */
  positionProfit: number;
  /** 风险度 */
  riskRatio: number;
}

/**
 * 资金流水
 */
export interface FundFlow {
  /** 流水ID */
  flowId: string;
  /** 账户ID */
  accountId: string;
  /** 流水类型 */
  flowType: 'DEPOSIT' | 'WITHDRAW' | 'TRADE' | 'COMMISSION' | 'INTEREST';
  /** 金额 */
  amount: number;
  /** 余额 */
  balance: number;
  /** 描述 */
  description: string;
  /** 时间 */
  createTime: string;
}

// ============================================================================
// 合约信息类型
// ============================================================================

/**
 * 合约信息
 */
export interface ContractInfo {
  /** 合约代码 */
  instrumentId: string;
  /** 合约名称 */
  instrumentName: string;
  /** 交易所代码 */
  exchangeId: string;
  /** 产品类型 */
  productType: string;
  /** 交易单位 */
  volumeMultiple: number;
  /** 最小变动价位 */
  priceTick: number;
  /** 保证金率 */
  marginRate: number;
  /** 手续费率 */
  commissionRate: number;
  /** 是否活跃 */
  isActive: boolean;
  /** 到期日 */
  expireDate?: string;
  /** 交易时间段 */
  tradingHours?: string[];
}

// ============================================================================
// 连接状态和统计信息类型
// ============================================================================

/**
 * 连接状态
 */
export enum ConnectionStatus {
  DISCONNECTED = 'DISCONNECTED',
  CONNECTING = 'CONNECTING',
  CONNECTED = 'CONNECTED',
  RECONNECTING = 'RECONNECTING',
  ERROR = 'ERROR',
}

/**
 * 连接信息
 */
export interface ConnectionInfo {
  /** 连接状态 */
  status: ConnectionStatus;
  /** 连接时间 */
  connectedTime?: string;
  /** 最后心跳时间 */
  lastHeartbeat?: string;
  /** 错误信息 */
  errorMessage?: string;
  /** 重连次数 */
  reconnectCount: number;
}

/**
 * 连接统计信息
 */
export interface ConnectionStats {
  /** 客户端状态 */
  state: ClientState;
  /** 重连次数 */
  reconnectCount: number;
  /** 连接持续时间（毫秒） */
  connectDuration?: number;
  /** 配置环境 */
  configEnvironment: Environment;
}

/**
 * 健康状态
 */
export interface HealthStatus {
  /** 是否健康 */
  isHealthy: boolean;
  /** 客户端状态 */
  state: ClientState;
  /** 最后检查时间 */
  lastCheckTime: string;
  /** 错误信息 */
  errorMessage?: string;
}

/**
 * 配置信息（隐藏敏感信息）
 */
export interface ConfigInfo {
  /** 环境类型 */
  environment: Environment;
  /** 经纪商代码 */
  brokerId: string;
  /** 用户ID */
  userId: string;
  /** 行情前置地址 */
  mdFrontAddr: string;
  /** 交易前置地址 */
  traderFrontAddr: string;
  /** 流文件路径 */
  flowPath: string;
  /** 超时时间（秒） */
  timeoutSecs: number;
  /** 最大重连次数 */
  maxReconnectAttempts: number;
}

// ============================================================================
// CTP 事件类型（基于后端事件系统）
// ============================================================================

/**
 * CTP 事件类型
 */
export enum CtpEventType {
  // 连接事件
  CONNECTED = 'Connected',
  DISCONNECTED = 'Disconnected',
  LOGIN_SUCCESS = 'LoginSuccess',
  LOGIN_FAILED = 'LoginFailed',
  
  // 市场数据事件
  MARKET_DATA = 'MarketData',
  SUBSCRIPTION_SUCCESS = 'SubscriptionSuccess',
  SUBSCRIPTION_FAILED = 'SubscriptionFailed',
  
  // 交易事件
  ORDER_SUBMITTED = 'OrderSubmitted',
  ORDER_FILLED = 'OrderFilled',
  ORDER_CANCELLED = 'OrderCancelled',
  ORDER_REJECTED = 'OrderRejected',
  
  // 查询响应事件
  ACCOUNT_INFO = 'AccountInfo',
  POSITION_INFO = 'PositionInfo',
  TRADE_INFO = 'TradeInfo',
  
  // 错误事件
  ERROR = 'Error',
}

/**
 * CTP 事件
 */
export interface CtpEvent<T = any> {
  /** 事件类型 */
  type: CtpEventType;
  /** 事件数据 */
  data: T;
  /** 时间戳 */
  timestamp: number;
  /** 事件ID */
  eventId?: string;
}

/**
 * 应用事件类型
 */
export enum AppEventType {
  // 连接事件
  CONNECTION_CHANGED = 'CONNECTION_CHANGED',
  
  // 市场数据事件
  MARKET_DATA_UPDATED = 'MARKET_DATA_UPDATED',
  KLINE_DATA_UPDATED = 'KLINE_DATA_UPDATED',
  
  // 交易事件
  ORDER_UPDATED = 'ORDER_UPDATED',
  POSITION_UPDATED = 'POSITION_UPDATED',
  TRADE_UPDATED = 'TRADE_UPDATED',
  ACCOUNT_UPDATED = 'ACCOUNT_UPDATED',
  
  // 系统事件
  ERROR_OCCURRED = 'ERROR_OCCURRED',
  NOTIFICATION = 'NOTIFICATION',
}

/**
 * 应用事件
 */
export interface AppEvent<T = any> {
  /** 事件类型 */
  type: AppEventType;
  /** 事件数据 */
  data: T;
  /** 时间戳 */
  timestamp: number;
  /** 事件ID */
  eventId?: string;
}

// ============================================================================
// 错误类型定义（基于后端 CtpError）
// ============================================================================

/**
 * CTP 错误类型
 */
export enum CtpErrorType {
  /** 连接错误 */
  CONNECTION_ERROR = 'ConnectionError',
  /** 认证错误 */
  AUTHENTICATION_ERROR = 'AuthenticationError',
  /** 配置错误 */
  CONFIG_ERROR = 'ConfigError',
  /** 库加载错误 */
  LIBRARY_LOAD_ERROR = 'LibraryLoadError',
  /** 状态错误 */
  STATE_ERROR = 'StateError',
  /** 超时错误 */
  TIMEOUT_ERROR = 'TimeoutError',
  /** 转换错误 */
  CONVERSION_ERROR = 'ConversionError',
  /** CTP API 错误 */
  CTP_API_ERROR = 'CtpApiError',
  /** 网络错误 */
  NETWORK_ERROR = 'NetworkError',
  /** 未知错误 */
  UNKNOWN_ERROR = 'UnknownError',
}

/**
 * CTP 错误信息
 */
export interface CtpError {
  /** 错误类型 */
  type: CtpErrorType;
  /** 错误消息 */
  message: string;
  /** 错误代码 */
  code?: number;
  /** 详细信息 */
  details?: string;
  /** 时间戳 */
  timestamp: number;
}

// ============================================================================
// 工具类型
// ============================================================================

/**
 * 深度可选类型
 */
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

/**
 * 深度只读类型
 */
export type DeepReadonly<T> = {
  readonly [P in keyof T]: T[P] extends object ? DeepReadonly<T[P]> : T[P];
};

/**
 * 提取数组元素类型
 */
export type ArrayElement<T> = T extends (infer U)[] ? U : never;

/**
 * 函数参数类型
 */
export type FunctionParams<T> = T extends (...args: infer P) => any ? P : never;

/**
 * 函数返回类型
 */
export type FunctionReturn<T> = T extends (...args: any[]) => infer R ? R : never;
/**
 * CTP Trading System Type Definitions
 */

// Market Data Types
export interface MarketData {
  instrument_id: string;
  exchange_id: string;
  last_price: number;
  pre_settlement_price: number;
  pre_close_price: number;
  pre_open_interest: number;
  open_price: number;
  highest_price: number;
  lowest_price: number;
  volume: number;
  turnover: number;
  open_interest: number;
  close_price: number;
  settlement_price: number;
  upper_limit_price: number;
  lower_limit_price: number;
  bid_price: number;
  bid_volume: number;
  ask_price: number;
  ask_volume: number;
  average_price: number;
  update_time: string;
  update_millisec: number;
  trading_day: string;
}

// Order Types
export interface OrderInput {
  instrument_id: string;
  direction: 'Buy' | 'Sell';
  offset: 'Open' | 'Close' | 'CloseToday' | 'CloseYesterday';
  price: number;
  volume: number;
  order_type: 'Limit' | 'Market' | 'Stop' | 'StopLimit';
  time_condition: 'IOC' | 'GFS' | 'GFD' | 'GTD' | 'GTC' | 'GFA';
  volume_condition: 'Any' | 'Min' | 'All';
  min_volume: number;
  contingent_condition: 'Immediately' | 'Touch' | 'TouchProfit';
  stop_price: number;
  force_close_reason: 'NotForceClose' | 'LackDeposit' | 'ClientOverPositionLimit';
  is_auto_suspend: boolean;
}

export interface OrderRef {
  order_ref: string;
  front_id: number;
  session_id: number;
}

export interface OrderStatus {
  order_ref: string;
  instrument_id: string;
  direction: 'Buy' | 'Sell';
  offset: 'Open' | 'Close' | 'CloseToday' | 'CloseYesterday';
  price: number;
  volume: number;
  volume_traded: number;
  volume_left: number;
  status: 'Submitted' | 'Accepted' | 'Rejected' | 'PartiallyFilled' | 'Filled' | 'Cancelled' | 'Cancelling';
  status_msg: string;
  insert_time: string;
  update_time: string;
  cancel_time?: string;
  front_id: number;
  session_id: number;
  exchange_id: string;
  order_sys_id: string;
}

// Trade Types
export interface Trade {
  trade_id: string;
  order_ref: string;
  instrument_id: string;
  direction: 'Buy' | 'Sell';
  offset: 'Open' | 'Close' | 'CloseToday' | 'CloseYesterday';
  price: number;
  volume: number;
  trade_time: string;
  trade_type: 'Common' | 'OptionsExecution' | 'OTC' | 'EFPDerived';
  exchange_id: string;
  commission: number;
}

// Position Types
export interface Position {
  instrument_id: string;
  direction: 'Buy' | 'Sell';
  position: number;
  position_today: number;
  position_yesterday: number;
  frozen: number;
  frozen_today: number;
  frozen_yesterday: number;
  average_price: number;
  position_cost: number;
  margin: number;
  close_profit: number;
  position_profit: number;
  open_volume: number;
  close_volume: number;
  settlement_price: number;
  exchange_id: string;
}

// Account Types
export interface AccountInfo {
  account_id: string;
  available: number;
  balance: number;
  margin: number;
  frozen_margin: number;
  frozen_commission: number;
  curr_margin: number;
  commission: number;
  close_profit: number;
  position_profit: number;
  risk_ratio: number;
}

// Instrument Types
export interface InstrumentInfo {
  instrument_id: string;
  exchange_id: string;
  instrument_name: string;
  product_id: string;
  product_class: 'Futures' | 'Options' | 'Combination' | 'Spot' | 'EFP' | 'SpotOption';
  delivery_year: number;
  delivery_month: number;
  max_market_order_volume: number;
  min_market_order_volume: number;
  max_limit_order_volume: number;
  min_limit_order_volume: number;
  volume_multiple: number;
  price_tick: number;
  create_date: string;
  open_date: string;
  expire_date: string;
  start_delivery_date: string;
  end_delivery_date: string;
  is_trading: boolean;
  underlying_instrument: string;
  strike_price: number;
  options_type?: 'Call' | 'Put';
  underlying_multiple: number;
  combination_type?: string;
  long_margin_ratio: number;
  short_margin_ratio: number;
}

// Commission and Margin Types
export interface CommissionRate {
  instrument_id: string;
  open_ratio_by_money: number;
  open_ratio_by_volume: number;
  close_ratio_by_money: number;
  close_ratio_by_volume: number;
  close_today_ratio_by_money: number;
  close_today_ratio_by_volume: number;
}

export interface MarginRate {
  instrument_id: string;
  long_margin_ratio_by_money: number;
  long_margin_ratio_by_volume: number;
  short_margin_ratio_by_money: number;
  short_margin_ratio_by_volume: number;
}

// Configuration Types
export interface LoginCredentials {
  user_id: string;
  password: string;
  broker_id: string;
}

export interface CtpConfig {
  environment: 'SimNow' | 'TTS' | 'Production';
  broker_id: string;
  app_id: string;
  auth_code: string;
  market_front: string[];
  trade_front: string[];
  user_product_info: string;
  protocol_info: string;
  flow_path: string;
  public_resume_type: number;
  private_resume_type: number;
  using_udp: boolean;
  using_multicast: boolean;
  md_dynlib_path?: string;
  td_dynlib_path?: string;
}

// Risk Management Types
export interface RiskParams {
  max_position_ratio: number;
  max_single_loss: number;
  max_daily_loss: number;
  max_order_volume: number;
  position_limit: Map<string, number>;
  forbidden_instruments: string[];
  auto_stop_loss: boolean;
  stop_loss_ratio: number;
  auto_take_profit: boolean;
  take_profit_ratio: number;
}

// Subscription Types
export interface MarketDataSubscription {
  instruments: string[];
  priority: 'Low' | 'Normal' | 'High' | 'Urgent';
  filter?: MarketDataFilter;
}

export interface MarketDataFilter {
  min_volume?: number;
  min_turnover?: number;
  exchanges?: string[];
  product_classes?: string[];
}

// Event Types
export type CtpEvent = 
  | { type: 'Connected' }
  | { type: 'Disconnected'; reason: string }
  | { type: 'LoginSuccess'; user_id: string }
  | { type: 'LoginFailed'; error: string }
  | { type: 'MarketData'; data: MarketData }
  | { type: 'OrderUpdate'; order: OrderStatus }
  | { type: 'TradeUpdate'; trade: Trade }
  | { type: 'PositionUpdate'; position: Position }
  | { type: 'AccountUpdate'; account: AccountInfo }
  | { type: 'Error'; code: number; message: string };

// Connection States
export enum ConnectionState {
  Disconnected = 'Disconnected',
  Connecting = 'Connecting',
  Connected = 'Connected',
  LoggingIn = 'LoggingIn',
  LoggedIn = 'LoggedIn',
  Disconnecting = 'Disconnecting'
}

// Error Types
export interface CtpError {
  code: number;
  message: string;
  details?: string;
}

// TTS 特定配置
export interface TtsConfig extends CtpConfig {
  ttsMode: 'openctp' | 'local' | 'custom';
  simulationFeatures?: {
    enableLatencySimulation: boolean;
    latencyRange: [number, number];
    enableSlippage: boolean;
    slippageRate: number;
    enablePartialFill: boolean;
  };
  testDataSets?: string[];
  mockMarketData?: boolean;
}

// 环境配置存储
export interface EnvironmentConfig {
  defaultEnvironment: string;
  autoSwitchWeekend: boolean;
  connectionTimeout: number;
  autoReconnect: boolean;
  environmentHistory: string[];
  customPresets: any[];
}

// 连接记录
export interface ConnectionRecord {
  environment: string;
  timestamp: Date;
  success: boolean;
  duration: number;
  errorMessage?: string;
}

// TTS 错误码
export enum TtsErrorCode {
  TTS_SERVICE_UNAVAILABLE = 'TTS_SERVICE_UNAVAILABLE',
  TTS_INVALID_CONFIG = 'TTS_INVALID_CONFIG',
  TTS_CONNECTION_TIMEOUT = 'TTS_CONNECTION_TIMEOUT',
  TTS_AUTHENTICATION_FAILED = 'TTS_AUTHENTICATION_FAILED',
  TTS_WEEKEND_ONLY = 'TTS_WEEKEND_ONLY'
}
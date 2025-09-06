/**
 * 服务层入口文件
 * 
 * 导出所有服务模块，提供统一的 API 接口
 */

// 主要 CTP 服务层
export { CtpServiceManager, ctpServiceManager, ctpService } from './ctp.service';

// Tauri 服务层（底层实现）
export { CtpService as TauriCtpService, ctpService as tauriCtpService } from './tauri';

// 向后兼容的 CTP 服务
export { CtpService as LegacyCtpService, ctpService as legacyCtpService } from './ctp';

// 错误处理工具
export { ErrorHandler, errorHandler, withRetry } from './errorHandler';

// 重新导出类型定义
export type {
  TauriResult,
} from './tauri';

// 其他服务模块（待实现）
// export * from './marketData';
// export * from './trading';
// export * from './account';
// export * from './config';
// export * from './websocket';

// 默认导出主要服务
export default ctpServiceManager;

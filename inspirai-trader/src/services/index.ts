/**
 * 服务层入口文件
 * 
 * 导出所有服务模块，提供统一的 API 接口
 */

// Tauri 服务层
export { CtpService, ctpService } from './tauri';

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

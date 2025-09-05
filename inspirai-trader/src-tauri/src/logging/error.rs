use std::path::PathBuf;
use thiserror::Error;

/// 日志系统错误类型
#[derive(Debug, Error)]
pub enum LogError {
    /// 日志文件写入失败
    #[error("日志文件写入失败: {0}")]
    WriteError(#[from] std::io::Error),
    
    /// 日志目录创建失败
    #[error("日志目录创建失败: {path:?}")]
    DirectoryCreationError { path: PathBuf },
    
    /// 日志轮转失败
    #[error("日志轮转失败: {reason}")]
    RotationError { reason: String },
    
    /// 日志压缩失败
    #[error("日志压缩失败: {file:?}")]
    CompressionError { file: PathBuf },
    
    /// 磁盘空间不足
    #[error("磁盘空间不足: 可用空间 {available_mb}MB")]
    InsufficientDiskSpace { available_mb: u64 },
    
    /// 日志配置无效
    #[error("日志配置无效: {field}")]
    InvalidConfig { field: String },
    
    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    /// 序列化失败
    #[error("序列化失败: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// 日志系统初始化失败
    #[error("日志系统初始化失败: {0}")]
    InitError(String),
    
    /// 异步操作失败
    #[error("异步操作失败: {0}")]
    AsyncError(String),
    
    /// 文件锁定失败
    #[error("文件锁定失败: {file:?}")]
    FileLockError { file: PathBuf },
    
    /// 解压缩失败
    #[error("解压缩失败: {file:?}")]
    DecompressionError { file: PathBuf },
    
    /// 索引操作失败
    #[error("索引操作失败: {operation}")]
    IndexError { operation: String },
    
    /// 查询操作失败
    #[error("查询操作失败: {query}")]
    QueryError { query: String },
    
    /// 权限不足
    #[error("权限不足: {operation}")]
    PermissionDenied { operation: String },
    
    /// 文件格式错误
    #[error("文件格式错误: {file:?}")]
    FileFormatError { file: PathBuf },
    
    /// 校验和不匹配
    #[error("校验和不匹配: 预期 {expected}, 实际 {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    
    /// 超时错误
    #[error("操作超时: {operation}")]
    TimeoutError { operation: String },
    
    /// 缓冲区溢出
    #[error("缓冲区溢出: 大小 {size}")]
    BufferOverflow { size: usize },
    
    /// 内存不足
    #[error("内存不足: 需要 {required_mb}MB")]
    OutOfMemory { required_mb: u64 },
}

impl LogError {
    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            LogError::WriteError(_) => "WRITE_ERROR",
            LogError::DirectoryCreationError { .. } => "DIR_CREATE_ERROR",
            LogError::RotationError { .. } => "ROTATION_ERROR",
            LogError::CompressionError { .. } => "COMPRESSION_ERROR",
            LogError::InsufficientDiskSpace { .. } => "DISK_SPACE_ERROR",
            LogError::InvalidConfig { .. } => "INVALID_CONFIG_ERROR",
            LogError::ConfigError(_) => "CONFIG_ERROR",
            LogError::SerializationError(_) => "SERIALIZATION_ERROR",
            LogError::InitError(_) => "INIT_ERROR",
            LogError::AsyncError(_) => "ASYNC_ERROR",
            LogError::FileLockError { .. } => "FILE_LOCK_ERROR",
            LogError::DecompressionError { .. } => "DECOMPRESSION_ERROR",
            LogError::IndexError { .. } => "INDEX_ERROR",
            LogError::QueryError { .. } => "QUERY_ERROR",
            LogError::PermissionDenied { .. } => "PERMISSION_DENIED",
            LogError::FileFormatError { .. } => "FILE_FORMAT_ERROR",
            LogError::ChecksumMismatch { .. } => "CHECKSUM_MISMATCH",
            LogError::TimeoutError { .. } => "TIMEOUT_ERROR",
            LogError::BufferOverflow { .. } => "BUFFER_OVERFLOW",
            LogError::OutOfMemory { .. } => "OUT_OF_MEMORY",
        }
    }
    
    /// 判断是否为可恢复的错误
    pub fn is_recoverable(&self) -> bool {
        match self {
            // 可恢复的错误 - 可以通过重试或降级策略处理
            LogError::WriteError(_) => true,
            LogError::InsufficientDiskSpace { .. } => true,
            LogError::AsyncError(_) => true,
            LogError::FileLockError { .. } => true,
            LogError::TimeoutError { .. } => true,
            LogError::BufferOverflow { .. } => true,
            
            // 不可恢复的错误 - 需要人工干预
            LogError::DirectoryCreationError { .. } => false,
            LogError::InvalidConfig { .. } => false,
            LogError::ConfigError(_) => false,
            LogError::InitError(_) => false,
            LogError::PermissionDenied { .. } => false,
            LogError::FileFormatError { .. } => false,
            LogError::ChecksumMismatch { .. } => false,
            LogError::OutOfMemory { .. } => false,
            
            // 部分可恢复的错误
            LogError::RotationError { .. } => true,
            LogError::CompressionError { .. } => true,
            LogError::DecompressionError { .. } => false,
            LogError::SerializationError(_) => false,
            LogError::IndexError { .. } => true,
            LogError::QueryError { .. } => true,
        }
    }
    
    /// 获取建议的重试次数
    pub fn suggested_retry_count(&self) -> u32 {
        match self {
            LogError::WriteError(_) => 3,
            LogError::InsufficientDiskSpace { .. } => 1,
            LogError::AsyncError(_) => 2,
            LogError::FileLockError { .. } => 5,
            LogError::TimeoutError { .. } => 2,
            LogError::BufferOverflow { .. } => 1,
            LogError::RotationError { .. } => 2,
            LogError::CompressionError { .. } => 1,
            LogError::IndexError { .. } => 3,
            LogError::QueryError { .. } => 2,
            _ => 0, // 不可恢复的错误不重试
        }
    }
    
    /// 获取建议的重试延迟（毫秒）
    pub fn suggested_retry_delay_ms(&self) -> u64 {
        match self {
            LogError::WriteError(_) => 100,
            LogError::InsufficientDiskSpace { .. } => 5000,
            LogError::AsyncError(_) => 200,
            LogError::FileLockError { .. } => 50,
            LogError::TimeoutError { .. } => 1000,
            LogError::BufferOverflow { .. } => 500,
            LogError::RotationError { .. } => 1000,
            LogError::CompressionError { .. } => 2000,
            LogError::IndexError { .. } => 300,
            LogError::QueryError { .. } => 500,
            _ => 0,
        }
    }
    
    /// 获取错误的严重程度级别
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // 致命错误 - 系统无法继续运行
            LogError::InitError(_) => ErrorSeverity::Fatal,
            LogError::OutOfMemory { .. } => ErrorSeverity::Fatal,
            LogError::PermissionDenied { .. } => ErrorSeverity::Fatal,
            
            // 高危错误 - 影响核心功能
            LogError::DirectoryCreationError { .. } => ErrorSeverity::Critical,
            LogError::InvalidConfig { .. } => ErrorSeverity::Critical,
            LogError::ConfigError(_) => ErrorSeverity::Critical,
            LogError::ChecksumMismatch { .. } => ErrorSeverity::Critical,
            LogError::FileFormatError { .. } => ErrorSeverity::Critical,
            
            // 中等错误 - 影响部分功能
            LogError::WriteError(_) => ErrorSeverity::High,
            LogError::InsufficientDiskSpace { .. } => ErrorSeverity::High,
            LogError::RotationError { .. } => ErrorSeverity::High,
            LogError::SerializationError(_) => ErrorSeverity::High,
            
            // 低危错误 - 不影响核心功能
            LogError::CompressionError { .. } => ErrorSeverity::Medium,
            LogError::DecompressionError { .. } => ErrorSeverity::Medium,
            LogError::IndexError { .. } => ErrorSeverity::Medium,
            LogError::QueryError { .. } => ErrorSeverity::Medium,
            
            // 轻微错误 - 临时性问题
            LogError::AsyncError(_) => ErrorSeverity::Low,
            LogError::FileLockError { .. } => ErrorSeverity::Low,
            LogError::TimeoutError { .. } => ErrorSeverity::Low,
            LogError::BufferOverflow { .. } => ErrorSeverity::Low,
        }
    }
    
    /// 创建磁盘空间不足错误
    pub fn insufficient_disk_space(available_bytes: u64) -> Self {
        Self::InsufficientDiskSpace {
            available_mb: available_bytes / (1024 * 1024),
        }
    }
    
    /// 创建缓冲区溢出错误
    pub fn buffer_overflow(size: usize) -> Self {
        Self::BufferOverflow { size }
    }
    
    /// 创建超时错误
    pub fn timeout(operation: &str) -> Self {
        Self::TimeoutError {
            operation: operation.to_string(),
        }
    }
}

/// 错误严重程度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
    Fatal = 5,
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Low => "LOW",
            ErrorSeverity::Medium => "MEDIUM",
            ErrorSeverity::High => "HIGH",
            ErrorSeverity::Critical => "CRITICAL",
            ErrorSeverity::Fatal => "FATAL",
        }
    }
    
    /// 是否需要立即处理
    pub fn requires_immediate_action(&self) -> bool {
        matches!(self, ErrorSeverity::Critical | ErrorSeverity::Fatal)
    }
    
    /// 是否需要告警
    pub fn requires_alert(&self) -> bool {
        *self >= ErrorSeverity::High
    }
}

/// 错误恢复策略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 重试操作
    Retry {
        max_attempts: u32,
        delay_ms: u64,
        backoff: bool,
    },
    /// 降级运行
    Degrade {
        fallback_action: String,
    },
    /// 忽略错误
    Ignore,
    /// 停止操作
    Stop,
    /// 切换到备用路径
    Fallback {
        backup_path: std::path::PathBuf,
    },
}

impl RecoveryStrategy {
    /// 为错误类型创建默认恢复策略
    pub fn for_error(error: &LogError) -> Self {
        match error {
            // 临时性错误使用重试策略
            e if e.is_recoverable() => RecoveryStrategy::Retry {
                max_attempts: e.suggested_retry_count(),
                delay_ms: e.suggested_retry_delay_ms(),
                backoff: true,
            },
            
            // 磁盘空间不足使用降级策略
            LogError::InsufficientDiskSpace { .. } => RecoveryStrategy::Degrade {
                fallback_action: "启用紧急清理模式".to_string(),
            },
            
            // 目录创建失败尝试备用路径
            LogError::DirectoryCreationError { .. } => RecoveryStrategy::Fallback {
                backup_path: std::env::temp_dir().join("inspirai_trader_logs"),
            },
            
            // 致命错误停止操作
            e if e.severity() == ErrorSeverity::Fatal => RecoveryStrategy::Stop,
            
            // 其他错误忽略
            _ => RecoveryStrategy::Ignore,
        }
    }
}

/// 错误处理器
pub struct ErrorHandler;

impl ErrorHandler {
    /// 处理日志错误
    pub async fn handle_error(error: LogError, context: &str) -> Result<(), LogError> {
        let strategy = RecoveryStrategy::for_error(&error);
        
        // 记录错误（使用 eprintln! 避免递归）
        eprintln!(
            "[{}] 日志错误 {}: {} - 上下文: {} - 策略: {:?}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            error.error_code(),
            error,
            context,
            strategy
        );
        
        // 执行恢复策略
        match strategy {
            RecoveryStrategy::Retry { max_attempts: _, delay_ms: _, backoff: _ } => {
                // 重试逻辑由调用者实现
                Err(error)
            }
            
            RecoveryStrategy::Degrade { fallback_action } => {
                eprintln!("启用降级模式: {}", fallback_action);
                Ok(())
            }
            
            RecoveryStrategy::Ignore => {
                eprintln!("忽略错误并继续");
                Ok(())
            }
            
            RecoveryStrategy::Stop => {
                eprintln!("停止操作");
                Err(error)
            }
            
            RecoveryStrategy::Fallback { backup_path } => {
                eprintln!("使用备用路径: {:?}", backup_path);
                // 创建备用目录
                if let Err(e) = std::fs::create_dir_all(&backup_path) {
                    eprintln!("创建备用目录失败: {}", e);
                    return Err(error);
                }
                Ok(())
            }
        }
    }
    
    /// 批量处理错误
    pub async fn handle_errors(errors: Vec<LogError>, context: &str) -> Vec<LogError> {
        let mut remaining_errors = Vec::new();
        
        for error in errors {
            if let Err(e) = Self::handle_error(error, context).await {
                remaining_errors.push(e);
            }
        }
        
        remaining_errors
    }
    
    /// 检查是否需要立即停止
    pub fn should_stop(errors: &[LogError]) -> bool {
        errors.iter().any(|e| e.severity() == ErrorSeverity::Fatal)
    }
}

/// 重试助手
pub struct RetryHelper;

impl RetryHelper {
    /// 带指数退避的重试
    pub async fn retry_with_backoff<F, T, E>(
        mut operation: F,
        max_attempts: u32,
        initial_delay_ms: u64,
        max_delay_ms: u64,
    ) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        let mut attempts = 0;
        let mut delay_ms = initial_delay_ms;
        
        loop {
            attempts += 1;
            
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempts >= max_attempts {
                        return Err(error);
                    }
                    
                    // 等待后重试
                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    
                    // 指数退避
                    delay_ms = std::cmp::min(delay_ms * 2, max_delay_ms);
                    
                    eprintln!(
                        "重试操作 (第 {} 次，总共 {} 次) - 错误: {:?}",
                        attempts, max_attempts, error
                    );
                }
            }
        }
    }
    
    /// 简单重试
    pub async fn retry<F, T, E>(
        mut operation: F,
        max_attempts: u32,
        delay_ms: u64,
    ) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        for attempt in 1..=max_attempts {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempt >= max_attempts {
                        return Err(error);
                    }
                    
                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    eprintln!(
                        "重试操作 (第 {} 次，总共 {} 次) - 错误: {:?}",
                        attempt, max_attempts, error
                    );
                }
            }
        }
        
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_error_properties() {
        let error = LogError::WriteError(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "权限不足"
        ));
        
        assert_eq!(error.error_code(), "WRITE_ERROR");
        assert!(error.is_recoverable());
        assert_eq!(error.suggested_retry_count(), 3);
        assert_eq!(error.suggested_retry_delay_ms(), 100);
        assert_eq!(error.severity(), ErrorSeverity::High);
    }
    
    #[test]
    fn test_error_severity() {
        assert!(ErrorSeverity::Fatal.requires_immediate_action());
        assert!(ErrorSeverity::Critical.requires_immediate_action());
        assert!(ErrorSeverity::High.requires_alert());
        assert!(!ErrorSeverity::Low.requires_alert());
        
        assert!(ErrorSeverity::Fatal > ErrorSeverity::High);
        assert!(ErrorSeverity::High > ErrorSeverity::Low);
    }
    
    #[test]
    fn test_recovery_strategy() {
        let recoverable_error = LogError::WriteError(std::io::Error::new(
            std::io::ErrorKind::Interrupted,
            "中断"
        ));
        
        let strategy = RecoveryStrategy::for_error(&recoverable_error);
        
        match strategy {
            RecoveryStrategy::Retry { max_attempts, .. } => {
                assert_eq!(max_attempts, 3);
            }
            _ => panic!("应该是重试策略"),
        }
    }
    
    #[tokio::test]
    async fn test_retry_helper() {
        let mut counter = 0;
        let operation = || {
            counter += 1;
            if counter < 3 {
                Err("失败")
            } else {
                Ok("成功")
            }
        };
        
        let result = RetryHelper::retry(operation, 5, 10).await;
        assert_eq!(result.unwrap(), "成功");
        assert_eq!(counter, 3);
    }
    
    #[tokio::test]
    async fn test_error_handler() {
        let error = LogError::AsyncError("测试错误".to_string());
        let result = ErrorHandler::handle_error(error, "测试上下文").await;
        
        // 对于可恢复的错误，应该返回错误以便重试
        assert!(result.is_err());
    }
}
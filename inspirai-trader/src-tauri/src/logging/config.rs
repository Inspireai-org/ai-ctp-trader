use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use crate::ctp::config::Environment;
use super::error::LogError;

/// 日志级别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG", 
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
    
    pub fn from_str(s: &str) -> Result<Self, LogError> {
        match s.to_uppercase().as_str() {
            "TRACE" => Ok(LogLevel::Trace),
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARN" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            _ => Err(LogError::InvalidConfig { 
                field: format!("不支持的日志级别: {}", s) 
            }),
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<tracing::Level> for LogLevel {
    fn from(level: tracing::Level) -> Self {
        match level {
            tracing::Level::TRACE => LogLevel::Trace,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::ERROR => LogLevel::Error,
        }
    }
}

impl Into<tracing::Level> for LogLevel {
    fn into(self) -> tracing::Level {
        match self {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

/// 日志类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogType {
    App,
    Ctp,
    Trading,
    MarketData,
    Error,
    Performance,
}

impl LogType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogType::App => "app",
            LogType::Ctp => "ctp", 
            LogType::Trading => "trading",
            LogType::MarketData => "market_data",
            LogType::Error => "error",
            LogType::Performance => "performance",
        }
    }
    
    pub fn file_name(&self) -> &'static str {
        match self {
            LogType::App => "app.log",
            LogType::Ctp => "ctp.log",
            LogType::Trading => "trading.log", 
            LogType::MarketData => "market_data.log",
            LogType::Error => "error.log",
            LogType::Performance => "performance.log",
        }
    }
    
    /// 获取所有日志类型
    pub fn all() -> Vec<LogType> {
        vec![
            LogType::App,
            LogType::Ctp,
            LogType::Trading,
            LogType::MarketData,
            LogType::Error,
            LogType::Performance,
        ]
    }
}

impl std::fmt::Display for LogType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 日志配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// 日志级别
    pub level: LogLevel,
    /// 输出目录
    pub output_dir: PathBuf,
    /// 是否输出到控制台
    pub console_output: bool,
    /// 是否输出到文件
    pub file_output: bool,
    /// 最大文件大小 (字节)
    pub max_file_size: u64,
    /// 最大文件数量
    pub max_files: usize,
    /// 是否启用压缩
    pub compression_enabled: bool,
    /// 保留天数
    pub retention_days: u32,
    /// 异步缓冲区大小
    pub async_buffer_size: usize,
    /// 批量写入大小
    pub batch_size: usize,
    /// 刷新间隔
    pub flush_interval: Duration,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            output_dir: PathBuf::from("./logs"),
            console_output: true,
            file_output: true,
            max_file_size: 50 * 1024 * 1024, // 50MB
            max_files: 30,
            compression_enabled: true,
            retention_days: 90,
            async_buffer_size: 64 * 1024, // 64KB
            batch_size: 1000,
            flush_interval: Duration::from_millis(100),
        }
    }
}

impl LogConfig {
    /// 为开发环境创建配置
    pub fn development() -> Self {
        Self {
            level: LogLevel::Debug,
            output_dir: PathBuf::from("./logs"),
            console_output: true,
            file_output: true,
            max_file_size: 10 * 1024 * 1024, // 10MB 用于开发
            max_files: 10,
            compression_enabled: false, // 开发环境不压缩便于调试
            retention_days: 7, // 开发环境保留7天
            async_buffer_size: 32 * 1024, // 32KB
            batch_size: 500,
            flush_interval: Duration::from_millis(50), // 更快刷新用于调试
        }
    }
    
    /// 为生产环境创建配置
    pub fn production() -> Result<Self, LogError> {
        let output_dir = Self::get_user_data_dir()?;
        
        Ok(Self {
            level: LogLevel::Info,
            output_dir,
            console_output: false, // 生产环境不输出到控制台
            file_output: true,
            max_file_size: 50 * 1024 * 1024, // 50MB
            max_files: 30,
            compression_enabled: true,
            retention_days: 90,
            async_buffer_size: 64 * 1024, // 64KB
            batch_size: 1000,
            flush_interval: Duration::from_millis(100),
        })
    }
    
    /// 根据环境创建配置
    pub fn for_environment(env: Environment) -> Result<Self, LogError> {
        match env {
            Environment::SimNow | Environment::Tts => Ok(Self::development()),
            Environment::Production => Self::production(),
        }
    }
    
    /// 获取用户数据目录下的日志路径
    fn get_user_data_dir() -> Result<PathBuf, LogError> {
        let user_dir = dirs::data_dir()
            .ok_or_else(|| LogError::ConfigError("无法获取用户数据目录".to_string()))?;
        
        let logs_dir = user_dir.join("InspirAI Trader").join("logs");
        
        if !logs_dir.exists() {
            std::fs::create_dir_all(&logs_dir)
                .map_err(|_| LogError::DirectoryCreationError { 
                    path: logs_dir.clone() 
                })?;
        }
        
        Ok(logs_dir)
    }
    
    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), LogError> {
        // 验证输出目录
        if self.file_output {
            if let Some(parent) = self.output_dir.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|_| LogError::DirectoryCreationError { 
                            path: parent.to_path_buf() 
                        })?;
                }
            }
        }
        
        // 验证文件大小限制
        if self.max_file_size < 1024 * 1024 { // 最小1MB
            return Err(LogError::InvalidConfig {
                field: "max_file_size 不能小于 1MB".to_string(),
            });
        }
        
        // 验证文件数量
        if self.max_files == 0 {
            return Err(LogError::InvalidConfig {
                field: "max_files 必须大于 0".to_string(),
            });
        }
        
        // 验证保留天数
        if self.retention_days == 0 {
            return Err(LogError::InvalidConfig {
                field: "retention_days 必须大于 0".to_string(),
            });
        }
        
        // 验证缓冲区大小
        if self.async_buffer_size < 1024 { // 最小1KB
            return Err(LogError::InvalidConfig {
                field: "async_buffer_size 不能小于 1KB".to_string(),
            });
        }
        
        // 验证批量大小
        if self.batch_size == 0 {
            return Err(LogError::InvalidConfig {
                field: "batch_size 必须大于 0".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// 从环境变量覆盖配置
    pub fn apply_env_overrides(&mut self) {
        // 日志级别
        if let Ok(level_str) = std::env::var("LOG_LEVEL") {
            if let Ok(level) = LogLevel::from_str(&level_str) {
                self.level = level;
            }
        }
        
        // 输出目录
        if let Ok(output_dir) = std::env::var("LOG_OUTPUT_DIR") {
            self.output_dir = PathBuf::from(output_dir);
        }
        
        // 控制台输出
        if let Ok(console_output) = std::env::var("LOG_CONSOLE_OUTPUT") {
            self.console_output = console_output.to_lowercase() == "true";
        }
        
        // 文件输出
        if let Ok(file_output) = std::env::var("LOG_FILE_OUTPUT") {
            self.file_output = file_output.to_lowercase() == "true";
        }
        
        // 最大文件大小
        if let Ok(max_size) = std::env::var("LOG_MAX_FILE_SIZE") {
            if let Ok(size) = max_size.parse::<u64>() {
                self.max_file_size = size;
            }
        }
        
        // 压缩启用
        if let Ok(compression) = std::env::var("LOG_COMPRESSION_ENABLED") {
            self.compression_enabled = compression.to_lowercase() == "true";
        }
    }
    
    /// 获取特定日志类型的文件路径
    pub fn get_log_file_path(&self, log_type: LogType) -> PathBuf {
        self.output_dir.join(log_type.as_str()).join(log_type.file_name())
    }
    
    /// 获取存档目录路径
    pub fn get_archive_dir(&self) -> PathBuf {
        self.output_dir.join("archive")
    }
    
    /// 创建所有必要的目录
    pub fn ensure_directories(&self) -> Result<(), LogError> {
        // 创建主输出目录
        if !self.output_dir.exists() {
            std::fs::create_dir_all(&self.output_dir)
                .map_err(|_| LogError::DirectoryCreationError { 
                    path: self.output_dir.clone() 
                })?;
        }
        
        // 为每个日志类型创建子目录
        for log_type in LogType::all() {
            let log_dir = self.output_dir.join(log_type.as_str());
            if !log_dir.exists() {
                std::fs::create_dir_all(&log_dir)
                    .map_err(|_| LogError::DirectoryCreationError { 
                        path: log_dir 
                    })?;
            }
        }
        
        // 创建存档目录
        let archive_dir = self.get_archive_dir();
        if !archive_dir.exists() {
            std::fs::create_dir_all(&archive_dir)
                .map_err(|_| LogError::DirectoryCreationError { 
                    path: archive_dir 
                })?;
        }
        
        Ok(())
    }
    
    /// 获取当前配置的摘要信息
    pub fn summary(&self) -> String {
        format!(
            "LogConfig {{ level: {}, output_dir: {:?}, console: {}, file: {}, max_size: {}MB, max_files: {}, compression: {}, retention: {}days }}",
            self.level,
            self.output_dir,
            self.console_output,
            self.file_output,
            self.max_file_size / (1024 * 1024),
            self.max_files,
            self.compression_enabled,
            self.retention_days
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_level() {
        assert_eq!(LogLevel::from_str("DEBUG").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert!(LogLevel::Debug < LogLevel::Info);
    }
    
    #[test]
    fn test_log_type() {
        assert_eq!(LogType::Trading.as_str(), "trading");
        assert_eq!(LogType::Trading.file_name(), "trading.log");
        assert_eq!(LogType::all().len(), 6);
    }
    
    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert!(config.console_output);
        assert!(config.file_output);
        assert_eq!(config.max_file_size, 50 * 1024 * 1024);
    }
    
    #[test]
    fn test_log_config_development() {
        let config = LogConfig::development();
        assert_eq!(config.level, LogLevel::Debug);
        assert_eq!(config.output_dir, PathBuf::from("./logs"));
        assert!(!config.compression_enabled);
        assert_eq!(config.retention_days, 7);
    }
    
    #[test]
    fn test_log_config_validation() {
        let mut config = LogConfig::default();
        assert!(config.validate().is_ok());
        
        config.max_file_size = 100; // 太小
        assert!(config.validate().is_err());
        
        config.max_file_size = 10 * 1024 * 1024; // 恢复正常
        config.max_files = 0; // 无效值
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_log_config_env_overrides() {
        std::env::set_var("LOG_LEVEL", "ERROR");
        std::env::set_var("LOG_CONSOLE_OUTPUT", "false");
        
        let mut config = LogConfig::default();
        config.apply_env_overrides();
        
        assert_eq!(config.level, LogLevel::Error);
        assert!(!config.console_output);
        
        // 清理环境变量
        std::env::remove_var("LOG_LEVEL");
        std::env::remove_var("LOG_CONSOLE_OUTPUT");
    }
    
    #[test]
    fn test_log_config_paths() {
        let temp_dir = TempDir::new().unwrap();
        let config = LogConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..LogConfig::default()
        };
        
        let trading_path = config.get_log_file_path(LogType::Trading);
        assert!(trading_path.to_string_lossy().contains("trading/trading.log"));
        
        let archive_path = config.get_archive_dir();
        assert!(archive_path.to_string_lossy().ends_with("archive"));
    }
    
    #[test]
    fn test_ensure_directories() {
        let temp_dir = TempDir::new().unwrap();
        let config = LogConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..LogConfig::default()
        };
        
        assert!(config.ensure_directories().is_ok());
        
        // 验证目录是否创建
        assert!(config.output_dir.exists());
        assert!(config.output_dir.join("app").exists());
        assert!(config.output_dir.join("trading").exists());
        assert!(config.get_archive_dir().exists());
    }
}
use thiserror::Error;

/// CTP 组件错误类型
#[derive(Debug, Error)]
pub enum CtpError {
    #[error("连接错误: {0}")]
    ConnectionError(String),
    
    #[error("认证失败: {0}")]
    AuthenticationError(String),
    
    #[error("网络错误: {0}")]
    NetworkError(String),
    
    #[error("CTP API 错误: {code} - {message}")]
    CtpApiError { code: i32, message: String },
    
    #[error("数据转换错误: {0}")]
    ConversionError(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("超时错误")]
    TimeoutError,
    
    #[error("库加载错误: {0}")]
    LibraryLoadError(String),
    
    #[error("状态错误: {0}")]
    StateError(String),
    
    #[error("验证错误: {0}")]
    ValidationError(String),
    
    #[error("未找到: {0}")]
    NotFound(String),
    
    #[error("未实现: {0}")]
    NotImplemented(String),
    
    #[error("未知错误: {0}")]
    Unknown(String),
}

impl CtpError {
    /// 从 CTP 错误码创建错误
    /// 严格按照 CTP 官方错误码进行处理，提供中文错误信息
    pub fn from_ctp_error(error_code: i32, error_msg: &str) -> Self {
        match error_code {
            0 => panic!("成功状态不应该创建错误，这表明调用逻辑有问题"),
            -1 => CtpError::NetworkError("网络连接失败".to_string()),
            -2 => CtpError::AuthenticationError("用户名或密码错误".to_string()),
            -3 => CtpError::AuthenticationError("用户已登录".to_string()),
            -4 => CtpError::AuthenticationError("用户不存在".to_string()),
            -5 => CtpError::AuthenticationError("密码错误".to_string()),
            -6 => CtpError::AuthenticationError("用户被锁定".to_string()),
            -7 => CtpError::NetworkError("连接超时".to_string()),
            -8 => CtpError::AuthenticationError("认证失败".to_string()),
            -9 => CtpError::NetworkError("前置不活跃".to_string()),
            -10 => CtpError::AuthenticationError("重复登录".to_string()),
            -11 => CtpError::ConfigError("经纪商代码错误".to_string()),
            -12 => CtpError::ConfigError("投资者代码错误".to_string()),
            -13 => CtpError::AuthenticationError("认证码错误".to_string()),
            -14 => CtpError::AuthenticationError("应用标识错误".to_string()),
            -15 => CtpError::NetworkError("会话超时".to_string()),
            _ => {
                // 对于未知错误码，记录详细信息以便后续分析
                tracing::warn!("遇到未知的 CTP 错误码: {}, 错误信息: {}", error_code, error_msg);
                CtpError::CtpApiError {
                    code: error_code,
                    message: format!("CTP 错误 ({}): {}", error_code, error_msg),
                }
            }
        }
    }

    /// 检查是否为可重试的错误
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            CtpError::NetworkError(_) | CtpError::TimeoutError | CtpError::ConnectionError(_)
        )
    }

    /// 获取错误代码（用于日志和监控）
    pub fn error_code(&self) -> &'static str {
        match self {
            CtpError::ConnectionError(_) => "CONNECTION_ERROR",
            CtpError::AuthenticationError(_) => "AUTH_ERROR",
            CtpError::NetworkError(_) => "NETWORK_ERROR",
            CtpError::CtpApiError { .. } => "CTP_API_ERROR",
            CtpError::ConversionError(_) => "CONVERSION_ERROR",
            CtpError::ConfigError(_) => "CONFIG_ERROR",
            CtpError::IoError(_) => "IO_ERROR",
            CtpError::TimeoutError => "TIMEOUT_ERROR",
            CtpError::LibraryLoadError(_) => "LIBRARY_LOAD_ERROR",
            CtpError::StateError(_) => "STATE_ERROR",
            CtpError::ValidationError(_) => "VALIDATION_ERROR",
            CtpError::NotFound(_) => "NOT_FOUND",
            CtpError::NotImplemented(_) => "NOT_IMPLEMENTED",
            CtpError::Unknown(_) => "UNKNOWN_ERROR",
        }
    }
}
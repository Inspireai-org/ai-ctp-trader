use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use clap::ValueEnum;

/// 环境类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum Environment {
    /// SimNow 模拟环境
    #[serde(rename = "simnow")]
    SimNow,
    /// TTS 7x24 测试环境
    #[serde(rename = "tts")]
    Tts,
    /// 生产环境
    #[serde(rename = "production")]
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::SimNow
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::SimNow => write!(f, "simnow"),
            Environment::Tts => write!(f, "tts"),
            Environment::Production => write!(f, "production"),
        }
    }
}

/// CTP 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtpConfig {
    /// 环境类型
    #[serde(default)]
    pub environment: Environment,
    /// 行情前置地址
    pub md_front_addr: String,
    /// 交易前置地址  
    pub trader_front_addr: String,
    /// 经纪商代码
    pub broker_id: String,
    /// 投资者代码
    pub investor_id: String,
    /// 密码
    pub password: String,
    /// 应用标识
    pub app_id: String,
    /// 授权编码
    pub auth_code: String,
    /// 流文件路径
    pub flow_path: String,
    /// 行情动态库路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md_dynlib_path: Option<PathBuf>,
    /// 交易动态库路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub td_dynlib_path: Option<PathBuf>,
    /// 超时时间（秒）
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// 重连间隔（秒）
    #[serde(default = "default_reconnect_interval")]
    pub reconnect_interval_secs: u64,
    /// 最大重连次数
    #[serde(default = "default_max_reconnect_attempts")]
    pub max_reconnect_attempts: u32,
}

impl CtpConfig {
    /// 创建默认配置（SimNow 环境）
    pub fn default() -> Self {
        Self::for_environment(Environment::SimNow, String::new(), String::new())
    }

    /// 为指定环境创建配置
    pub fn for_environment(env: Environment, investor_id: String, password: String) -> Self {
        match env {
            Environment::SimNow => Self::simnow_config(investor_id, password),
            Environment::Tts => Self::tts_config(investor_id, password),
            Environment::Production => Self::production_config(investor_id, password),
        }
    }

    /// SimNow 模拟环境配置
    pub fn simnow_config(investor_id: String, password: String) -> Self {
        Self {
            environment: Environment::SimNow,
            md_front_addr: "tcp://180.168.146.187:10131".to_string(),
            trader_front_addr: "tcp://180.168.146.187:10130".to_string(),
            broker_id: "9999".to_string(),
            investor_id,
            password,
            app_id: "simnow_client_test".to_string(),
            auth_code: "0000000000000000".to_string(),
            flow_path: "./ctp_flow/simnow/".to_string(),
            md_dynlib_path: None,
            td_dynlib_path: None,
            timeout_secs: 30,
            reconnect_interval_secs: 5,
            max_reconnect_attempts: 3,
        }
    }

    /// TTS 7x24 测试环境配置
    pub fn tts_config(investor_id: String, password: String) -> Self {
        Self {
            environment: Environment::Tts,
            md_front_addr: "tcp://121.37.80.177:20004".to_string(),
            trader_front_addr: "tcp://121.37.80.177:20002".to_string(),
            broker_id: "9999".to_string(),
            investor_id,
            password,
            app_id: "simnow_client_test".to_string(),
            auth_code: "0000000000000000".to_string(),
            flow_path: "./ctp_flow/tts/".to_string(),
            md_dynlib_path: None,
            td_dynlib_path: None,
            timeout_secs: 30,
            reconnect_interval_secs: 5,
            max_reconnect_attempts: 3,
        }
    }

    /// 生产环境配置（需要用户提供具体参数）
    pub fn production_config(investor_id: String, password: String) -> Self {
        Self {
            environment: Environment::Production,
            md_front_addr: "tcp://180.168.146.187:10131".to_string(), // 需要替换为实际地址
            trader_front_addr: "tcp://180.168.146.187:10130".to_string(), // 需要替换为实际地址
            broker_id: "".to_string(), // 需要用户配置
            investor_id,
            password,
            app_id: "".to_string(), // 需要用户配置
            auth_code: "".to_string(), // 需要用户配置
            flow_path: "./ctp_flow/production/".to_string(),
            md_dynlib_path: None,
            td_dynlib_path: None,
            timeout_secs: 30,
            reconnect_interval_secs: 5,
            max_reconnect_attempts: 3,
        }
    }

    /// 自动检测并设置动态库路径
    pub fn auto_detect_dynlib_paths(&mut self) -> Result<(), crate::ctp::CtpError> {
        let (md_path, td_path) = Self::detect_dynlib_paths()?;
        self.md_dynlib_path = Some(md_path);
        self.td_dynlib_path = Some(td_path);
        Ok(())
    }

    /// 检测动态库路径
    pub fn detect_dynlib_paths() -> Result<(PathBuf, PathBuf), crate::ctp::CtpError> {
        #[cfg(target_os = "macos")]
        {
            Self::detect_macos_dynlib_paths()
        }
        #[cfg(target_os = "linux")]
        {
            Self::detect_linux_dynlib_paths()
        }
        #[cfg(target_os = "windows")]
        {
            Self::detect_windows_dynlib_paths()
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_macos_dynlib_paths() -> Result<(PathBuf, PathBuf), crate::ctp::CtpError> {
        // 检查项目中的 CTP 库文件
        let base_paths = vec![
            PathBuf::from("./lib/macos/6.7.7/cepin"),
            PathBuf::from("./lib/macos/6.7.7/product"),
            PathBuf::from("../lib/macos/6.7.7/cepin"),
            PathBuf::from("../lib/macos/6.7.7/product"),
        ];

        for base_path in base_paths {
            let md_path = base_path.join("thostmduserapi_se.framework/thostmduserapi_se");
            let td_path = base_path.join("thosttraderapi_se.framework/thosttraderapi_se");

            if md_path.exists() && td_path.exists() {
                tracing::info!("检测到 CTP 动态库: {:?}", base_path);
                return Ok((md_path, td_path));
            }
        }

        Err(crate::ctp::CtpError::LibraryLoadError(
            "未找到 macOS CTP 动态库文件".to_string()
        ))
    }

    #[cfg(target_os = "linux")]
    fn detect_linux_dynlib_paths() -> Result<(PathBuf, PathBuf), crate::ctp::CtpError> {
        let base_paths = vec![
            PathBuf::from("./lib/linux"),
            PathBuf::from("../lib/linux"),
            PathBuf::from("/usr/local/lib/ctp"),
        ];

        for base_path in base_paths {
            let md_path = base_path.join("thostmduserapi_se.so");
            let td_path = base_path.join("thosttraderapi_se.so");

            if md_path.exists() && td_path.exists() {
                tracing::info!("检测到 CTP 动态库: {:?}", base_path);
                return Ok((md_path, td_path));
            }
        }

        Err(crate::ctp::CtpError::LibraryLoadError(
            "未找到 Linux CTP 动态库文件".to_string()
        ))
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_dynlib_paths() -> Result<(PathBuf, PathBuf), crate::ctp::CtpError> {
        let base_paths = vec![
            PathBuf::from("./lib/windows"),
            PathBuf::from("../lib/windows"),
            PathBuf::from("C:/Program Files/CTP"),
        ];

        for base_path in base_paths {
            let md_path = base_path.join("thostmduserapi_se.dll");
            let td_path = base_path.join("thosttraderapi_se.dll");

            if md_path.exists() && td_path.exists() {
                tracing::info!("检测到 CTP 动态库: {:?}", base_path);
                return Ok((md_path, td_path));
            }
        }

        Err(crate::ctp::CtpError::LibraryLoadError(
            "未找到 Windows CTP 动态库文件".to_string()
        ))
    }

    /// 获取超时时间
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// 获取重连间隔
    pub fn reconnect_interval(&self) -> Duration {
        Duration::from_secs(self.reconnect_interval_secs)
    }

    /// 获取行情动态库路径
    pub fn get_md_dynlib_path(&self) -> Result<&PathBuf, crate::ctp::CtpError> {
        self.md_dynlib_path.as_ref().ok_or_else(|| {
            crate::ctp::CtpError::ConfigError("行情动态库路径未设置".to_string())
        })
    }

    /// 获取交易动态库路径
    pub fn get_td_dynlib_path(&self) -> Result<&PathBuf, crate::ctp::CtpError> {
        self.td_dynlib_path.as_ref().ok_or_else(|| {
            crate::ctp::CtpError::ConfigError("交易动态库路径未设置".to_string())
        })
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), crate::ctp::CtpError> {
        if self.broker_id.is_empty() {
            return Err(crate::ctp::CtpError::ConfigError("经纪商代码不能为空".to_string()));
        }
        if self.investor_id.is_empty() {
            return Err(crate::ctp::CtpError::ConfigError("投资者代码不能为空".to_string()));
        }
        if self.password.is_empty() {
            return Err(crate::ctp::CtpError::ConfigError("密码不能为空".to_string()));
        }
        if self.md_front_addr.is_empty() {
            return Err(crate::ctp::CtpError::ConfigError("行情前置地址不能为空".to_string()));
        }
        if self.trader_front_addr.is_empty() {
            return Err(crate::ctp::CtpError::ConfigError("交易前置地址不能为空".to_string()));
        }

        // 验证动态库路径
        if let Some(md_path) = &self.md_dynlib_path {
            if !md_path.exists() {
                return Err(crate::ctp::CtpError::LibraryLoadError(
                    format!("行情动态库文件不存在: {:?}", md_path)
                ));
            }
        }
        if let Some(td_path) = &self.td_dynlib_path {
            if !td_path.exists() {
                return Err(crate::ctp::CtpError::LibraryLoadError(
                    format!("交易动态库文件不存在: {:?}", td_path)
                ));
            }
        }

        Ok(())
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_reconnect_interval() -> u64 {
    5
}

fn default_max_reconnect_attempts() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_enum() {
        assert_eq!(Environment::SimNow.to_string(), "simnow");
        assert_eq!(Environment::Tts.to_string(), "tts");
        assert_eq!(Environment::Production.to_string(), "production");
    }

    #[test]
    fn test_config_for_different_environments() {
        let simnow = CtpConfig::for_environment(
            Environment::SimNow,
            "test".to_string(),
            "pass".to_string(),
        );
        assert_eq!(simnow.environment, Environment::SimNow);
        assert_eq!(simnow.broker_id, "9999");

        let tts = CtpConfig::for_environment(
            Environment::Tts,
            "test".to_string(),
            "pass".to_string(),
        );
        assert_eq!(tts.environment, Environment::Tts);
        assert!(tts.md_front_addr.contains("121.37.80.177"));

        let prod = CtpConfig::for_environment(
            Environment::Production,
            "test".to_string(),
            "pass".to_string(),
        );
        assert_eq!(prod.environment, Environment::Production);
    }

    #[test]
    fn test_config_validation() {
        let mut config = CtpConfig::default();
        
        // 默认配置应该验证失败（缺少用户信息）
        assert!(config.validate().is_err());

        // 填写必要信息
        config.investor_id = "test_user".to_string();
        config.password = "test_pass".to_string();
        config.broker_id = "9999".to_string();
        
        // 现在应该验证成功
        assert!(config.validate().is_ok());
    }
}
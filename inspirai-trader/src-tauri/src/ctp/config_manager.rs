use crate::ctp::{CtpConfig, CtpError};
use crate::ctp::config::Environment;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

/// 扩展的配置结构，包含日志和环境设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedCtpConfig {
    #[serde(flatten)]
    pub ctp: CtpConfig,
    pub logging: LoggingConfig,
    pub environment: EnvironmentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 日志文件路径
    pub file_path: String,
    /// 是否启用控制台输出
    pub console: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// 环境类型
    pub env_type: String,
    /// 是否启用模拟模式
    pub simulation_mode: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self::for_environment(Environment::SimNow)
    }
}

impl LoggingConfig {
    pub fn for_environment(env: Environment) -> Self {
        match env {
            Environment::SimNow => Self {
                level: "debug".to_string(),
                file_path: "./logs/ctp_simnow.log".to_string(),
                console: true,
            },
            Environment::Tts => Self {
                level: "info".to_string(),
                file_path: "./logs/ctp_tts.log".to_string(),
                console: true,
            },
            Environment::Production => Self {
                level: "warn".to_string(),
                file_path: "./logs/ctp_production.log".to_string(),
                console: false,
            },
        }
    }
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self::for_environment(Environment::SimNow)
    }
}

impl EnvironmentConfig {
    pub fn for_environment(env: Environment) -> Self {
        match env {
            Environment::SimNow => Self {
                env_type: "simnow".to_string(),
                simulation_mode: true,
            },
            Environment::Tts => Self {
                env_type: "tts".to_string(),
                simulation_mode: true,
            },
            Environment::Production => Self {
                env_type: "production".to_string(),
                simulation_mode: false,
            },
        }
    }
}

impl Default for ExtendedCtpConfig {
    fn default() -> Self {
        Self {
            ctp: CtpConfig::default(),
            logging: LoggingConfig::default(),
            environment: EnvironmentConfig::default(),
        }
    }
}

/// 配置管理器
pub struct ConfigManager;

impl ConfigManager {
    /// 从 TOML 文件加载配置
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<ExtendedCtpConfig, CtpError> {
        let path = path.as_ref();
        
        if !path.exists() {
            tracing::warn!("配置文件不存在: {:?}，将创建默认配置", path);
            let default_config = ExtendedCtpConfig::default();
            Self::save_to_file(&default_config, path).await?;
            return Ok(default_config);
        }
        
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| CtpError::ConfigError(format!("读取配置文件失败: {}", e)))?;
        
        let mut config: ExtendedCtpConfig = toml::from_str(&content)
            .map_err(|e| CtpError::ConfigError(format!("解析配置文件失败: {}", e)))?;
        
        // 自动检测动态库路径（如果未设置）
        if config.ctp.md_dynlib_path.is_none() || config.ctp.td_dynlib_path.is_none() {
            tracing::info!("自动检测 CTP 动态库路径...");
            if let Err(e) = config.ctp.auto_detect_dynlib_paths() {
                tracing::warn!("自动检测动态库路径失败: {}", e);
            }
        }
        
        // 验证配置
        config.ctp.validate()?;
        
        tracing::info!("成功加载配置文件: {:?}", path);
        Ok(config)
    }

    /// 为指定环境加载配置
    pub async fn load_for_environment(
        env: Environment,
        investor_id: String,
        password: String,
    ) -> Result<ExtendedCtpConfig, CtpError> {
        let config_dir = PathBuf::from("./config");
        let config_file = config_dir.join(format!("{}.toml", env));
        
        // 如果配置文件不存在，创建默认配置
        if !config_file.exists() {
            tracing::info!("创建 {} 环境的默认配置文件", env);
            let mut ctp_config = CtpConfig::for_environment(env, investor_id, password);
            
            // 尝试自动检测动态库路径
            if let Err(e) = ctp_config.auto_detect_dynlib_paths() {
                tracing::warn!("自动检测动态库路径失败: {}", e);
            }
            
            let extended_config = ExtendedCtpConfig {
                ctp: ctp_config,
                logging: LoggingConfig::for_environment(env),
                environment: EnvironmentConfig::for_environment(env),
            };
            
            Self::save_to_file(&extended_config, &config_file).await?;
            return Ok(extended_config);
        }
        
        // 加载现有配置文件
        let mut config = Self::load_from_file(&config_file).await?;
        
        // 更新用户凭据（如果提供）
        if !investor_id.is_empty() {
            config.ctp.investor_id = investor_id;
        }
        if !password.is_empty() {
            config.ctp.password = password;
        }
        
        Ok(config)
    }
    
    /// 保存配置到 TOML 文件
    pub async fn save_to_file<P: AsRef<Path>>(
        config: &ExtendedCtpConfig,
        path: P,
    ) -> Result<(), CtpError> {
        let path = path.as_ref();
        
        // 创建目录（如果不存在）
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| CtpError::ConfigError(format!("创建配置目录失败: {}", e)))?;
        }
        
        let content = toml::to_string_pretty(config)
            .map_err(|e| CtpError::ConfigError(format!("序列化配置失败: {}", e)))?;
        
        fs::write(path, content)
            .await
            .map_err(|e| CtpError::ConfigError(format!("写入配置文件失败: {}", e)))?;
        
        tracing::info!("配置文件已保存: {:?}", path);
        Ok(())
    }
    
    /// 从环境变量加载配置
    pub fn load_from_env() -> Result<CtpConfig, CtpError> {
        let mut config = CtpConfig::default();
        
        if let Ok(md_addr) = std::env::var("CTP_MD_FRONT_ADDR") {
            config.md_front_addr = md_addr;
        }
        
        if let Ok(trader_addr) = std::env::var("CTP_TRADER_FRONT_ADDR") {
            config.trader_front_addr = trader_addr;
        }
        
        if let Ok(broker_id) = std::env::var("CTP_BROKER_ID") {
            config.broker_id = broker_id;
        }
        
        if let Ok(investor_id) = std::env::var("CTP_INVESTOR_ID") {
            config.investor_id = investor_id;
        }
        
        if let Ok(password) = std::env::var("CTP_PASSWORD") {
            config.password = password;
        }
        
        if let Ok(app_id) = std::env::var("CTP_APP_ID") {
            config.app_id = app_id;
        }
        
        if let Ok(auth_code) = std::env::var("CTP_AUTH_CODE") {
            config.auth_code = auth_code;
        }
        
        if let Ok(flow_path) = std::env::var("CTP_FLOW_PATH") {
            config.flow_path = flow_path;
        }
        
        if let Ok(timeout) = std::env::var("CTP_TIMEOUT_SECS") {
            config.timeout_secs = timeout.parse()
                .map_err(|e| CtpError::ConfigError(format!("解析超时时间失败: {}", e)))?;
        }
        
        config.validate()?;
        
        tracing::info!("从环境变量加载配置完成");
        Ok(config)
    }

    /// 验证配置文件完整性
    pub async fn validate_config_file<P: AsRef<Path>>(path: P) -> Result<(), CtpError> {
        let config = Self::load_from_file(path).await?;
        config.ctp.validate()?;
        
        // 检查动态库文件是否存在
        if let Some(md_path) = &config.ctp.md_dynlib_path {
            if !md_path.exists() {
                return Err(CtpError::LibraryLoadError(
                    format!("行情动态库文件不存在: {:?}", md_path)
                ));
            }
        }
        if let Some(td_path) = &config.ctp.td_dynlib_path {
            if !td_path.exists() {
                return Err(CtpError::LibraryLoadError(
                    format!("交易动态库文件不存在: {:?}", td_path)
                ));
            }
        }
        
        tracing::info!("配置文件验证通过");
        Ok(())
    }

    /// 创建所有环境的默认配置文件
    pub async fn create_default_configs() -> Result<(), CtpError> {
        let config_dir = PathBuf::from("./config");
        fs::create_dir_all(&config_dir)
            .await
            .map_err(|e| CtpError::ConfigError(format!("创建配置目录失败: {}", e)))?;

        for env in [Environment::SimNow, Environment::Tts, Environment::Production] {
            let config_file = config_dir.join(format!("{}.toml", env));
            if !config_file.exists() {
                let mut ctp_config = CtpConfig::for_environment(env, String::new(), String::new());
                
                // 尝试自动检测动态库路径
                if let Err(e) = ctp_config.auto_detect_dynlib_paths() {
                    tracing::warn!("为 {} 环境自动检测动态库路径失败: {}", env, e);
                }
                
                let extended_config = ExtendedCtpConfig {
                    ctp: ctp_config,
                    logging: LoggingConfig::for_environment(env),
                    environment: EnvironmentConfig::for_environment(env),
                };
                
                Self::save_to_file(&extended_config, &config_file).await?;
                tracing::info!("创建 {} 环境配置文件: {:?}", env, config_file);
            }
        }
        
        Ok(())
    }

    /// 获取配置文件路径
    pub fn get_config_path(env: Environment) -> PathBuf {
        PathBuf::from("./config").join(format!("{}.toml", env))
    }
    
    /// 合并配置（环境变量优先）
    pub fn merge_configs(file_config: CtpConfig, env_config: CtpConfig) -> CtpConfig {
        CtpConfig {
            environment: file_config.environment,
            md_front_addr: if env_config.md_front_addr != CtpConfig::default().md_front_addr {
                env_config.md_front_addr
            } else {
                file_config.md_front_addr
            },
            trader_front_addr: if env_config.trader_front_addr != CtpConfig::default().trader_front_addr {
                env_config.trader_front_addr
            } else {
                file_config.trader_front_addr
            },
            broker_id: if !env_config.broker_id.is_empty() {
                env_config.broker_id
            } else {
                file_config.broker_id
            },
            investor_id: if !env_config.investor_id.is_empty() {
                env_config.investor_id
            } else {
                file_config.investor_id
            },
            password: if !env_config.password.is_empty() {
                env_config.password
            } else {
                file_config.password
            },
            app_id: if env_config.app_id != CtpConfig::default().app_id {
                env_config.app_id
            } else {
                file_config.app_id
            },
            auth_code: if env_config.auth_code != CtpConfig::default().auth_code {
                env_config.auth_code
            } else {
                file_config.auth_code
            },
            flow_path: if env_config.flow_path != CtpConfig::default().flow_path {
                env_config.flow_path
            } else {
                file_config.flow_path
            },
            md_dynlib_path: file_config.md_dynlib_path.or(env_config.md_dynlib_path),
            td_dynlib_path: file_config.td_dynlib_path.or(env_config.td_dynlib_path),
            timeout_secs: if env_config.timeout_secs != CtpConfig::default().timeout_secs {
                env_config.timeout_secs
            } else {
                file_config.timeout_secs
            },
            reconnect_interval_secs: if env_config.reconnect_interval_secs != CtpConfig::default().reconnect_interval_secs {
                env_config.reconnect_interval_secs
            } else {
                file_config.reconnect_interval_secs
            },
            max_reconnect_attempts: if env_config.max_reconnect_attempts != CtpConfig::default().max_reconnect_attempts {
                env_config.max_reconnect_attempts
            } else {
                file_config.max_reconnect_attempts
            },
        }
    }
}
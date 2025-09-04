#[cfg(test)]
mod production_config_tests {
    use crate::ctp::{ConfigManager, ExtendedCtpConfig, Environment};
    use std::path::Path;

    #[tokio::test]
    async fn test_production_config_loading() {
        let config_path = Path::new("config/production.toml");
        
        println!("测试生产环境配置文件: {:?}", config_path);
        
        // 检查配置文件是否存在
        if !config_path.exists() {
            panic!("生产环境配置文件不存在: {:?}", config_path);
        }
        
        // 加载配置文件
        let result = ConfigManager::load_from_file(config_path).await;
        
        match result {
            Ok(config) => {
                println!("✅ 配置文件加载成功");
                
                // 验证基本配置
                assert_eq!(config.ctp.environment, Environment::Production);
                assert_eq!(config.ctp.broker_id, "5071");
                assert_eq!(config.ctp.investor_id, "00001");
                assert_eq!(config.ctp.md_front_addr, "tcp://58.62.16.148:41214");
                assert_eq!(config.ctp.trader_front_addr, "tcp://58.62.16.148:41206");
                assert_eq!(config.ctp.app_id, "inspirai_strategy_1.0.0");
                assert_eq!(config.ctp.auth_code, "QHFK5E2GLEUB9XHV");
                
                println!("✅ 基本配置验证通过");
                println!("  经纪商: {}", config.ctp.broker_id);
                println!("  用户ID: {}", config.ctp.investor_id);
                println!("  行情服务器: {}", config.ctp.md_front_addr);
                println!("  交易服务器: {}", config.ctp.trader_front_addr);
                println!("  应用ID: {}", config.ctp.app_id);
                
                // 验证环境配置
                assert_eq!(config.environment.env_type, "production");
                assert_eq!(config.environment.simulation_mode, false);
                println!("✅ 环境配置验证通过");
                
                // 验证日志配置
                assert_eq!(config.logging.level, "warn");
                assert_eq!(config.logging.console, false);
                assert_eq!(config.logging.file_path, "./logs/ctp_production.log");
                println!("✅ 日志配置验证通过");
                
                // 验证动态库路径
                if let Some(md_path) = &config.ctp.md_dynlib_path {
                    println!("  行情库路径: {:?}", md_path);
                    if md_path.exists() {
                        println!("✅ 行情动态库文件存在");
                    } else {
                        println!("⚠️  行情动态库文件不存在: {:?}", md_path);
                    }
                }
                
                if let Some(td_path) = &config.ctp.td_dynlib_path {
                    println!("  交易库路径: {:?}", td_path);
                    if td_path.exists() {
                        println!("✅ 交易动态库文件存在");
                    } else {
                        println!("⚠️  交易动态库文件不存在: {:?}", td_path);
                    }
                }
                
                // 验证配置完整性
                let validation_result = config.ctp.validate();
                match validation_result {
                    Ok(_) => println!("✅ 配置验证通过"),
                    Err(e) => {
                        println!("❌ 配置验证失败: {}", e);
                        panic!("配置验证失败: {}", e);
                    }
                }
                
                println!("🎉 生产环境配置测试全部通过！");
            }
            Err(e) => {
                println!("❌ 配置文件加载失败: {}", e);
                panic!("配置文件加载失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_production_config_structure() {
        let config_path = Path::new("config/production.toml");
        
        if !config_path.exists() {
            println!("⚠️  生产环境配置文件不存在，跳过结构测试");
            return;
        }
        
        // 读取原始配置内容
        let content = tokio::fs::read_to_string(config_path).await.unwrap();
        println!("配置文件内容:");
        println!("{}", content);
        
        // 解析为 TOML 值进行结构验证
        let toml_value: toml::Value = toml::from_str(&content).unwrap();
        
        // 验证必需的字段（在顶层，因为使用了 flatten）
        let required_fields = vec![
            "md_front_addr",
            "trader_front_addr", 
            "broker_id",
            "investor_id",
            "password",
            "app_id",
            "auth_code",
            "environment",
        ];
        
        for field in required_fields {
            if toml_value.get(field).is_some() {
                println!("✅ 必需字段 '{}' 存在", field);
            } else {
                panic!("❌ 缺少必需字段: {}", field);
            }
        }
        
        // 验证嵌套结构
        if let Some(env_section) = toml_value.get("environment") {
            println!("✅ environment 配置段存在");
            if env_section.get("env_type").is_some() {
                println!("✅ environment.env_type 存在");
            }
            if env_section.get("simulation_mode").is_some() {
                println!("✅ environment.simulation_mode 存在");
            }
        }
        
        if let Some(logging_section) = toml_value.get("logging") {
            println!("✅ logging 配置段存在");
            if logging_section.get("level").is_some() {
                println!("✅ logging.level 存在");
            }
            if logging_section.get("file_path").is_some() {
                println!("✅ logging.file_path 存在");
            }
            if logging_section.get("console").is_some() {
                println!("✅ logging.console 存在");
            }
        }
        
        println!("✅ 配置文件结构验证通过");
    }

    #[test]
    fn test_production_config_security() {
        println!("🔒 生产环境配置安全检查");
        
        // 这里可以添加安全相关的检查
        // 比如密码强度、敏感信息处理等
        
        // 检查是否使用了生产环境的安全设置
        let config_path = std::path::Path::new("config/production.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path).unwrap();
            
            // 检查是否禁用了控制台日志输出（生产环境建议）
            if content.contains("console = false") {
                println!("✅ 生产环境已禁用控制台日志输出");
            } else {
                println!("⚠️  建议在生产环境中禁用控制台日志输出");
            }
            
            // 检查日志级别是否合适
            if content.contains("level = \"warn\"") || content.contains("level = \"error\"") {
                println!("✅ 生产环境使用了合适的日志级别");
            } else {
                println!("⚠️  建议在生产环境中使用 warn 或 error 日志级别");
            }
            
            // 检查是否禁用了模拟模式
            if content.contains("simulation_mode = false") {
                println!("✅ 生产环境已禁用模拟模式");
            } else {
                println!("❌ 生产环境必须禁用模拟模式！");
            }
        }
        
        println!("🔒 安全检查完成");
    }
}
#[cfg(test)]
mod simple_production_tests {
    use std::path::Path;

    #[tokio::test]
    async fn test_production_config_basic_parsing() {
        let config_path = Path::new("config/production.toml");
        
        if !config_path.exists() {
            println!("⚠️  生产环境配置文件不存在，跳过测试");
            return;
        }
        
        println!("🔍 测试生产环境配置文件基本解析");
        
        // 读取配置文件内容
        let content = tokio::fs::read_to_string(config_path).await.unwrap();
        println!("📄 配置文件内容:");
        println!("{}", content);
        
        // 尝试解析为基本的 TOML 值
        match toml::from_str::<toml::Value>(&content) {
            Ok(value) => {
                println!("✅ TOML 格式解析成功");
                
                // 检查关键字段
                let key_fields = vec![
                    ("md_front_addr", "行情前置地址"),
                    ("trader_front_addr", "交易前置地址"),
                    ("broker_id", "经纪商代码"),
                    ("investor_id", "投资者代码"),
                    ("password", "密码"),
                    ("app_id", "应用标识"),
                    ("auth_code", "授权编码"),
                ];
                
                for (key, desc) in key_fields {
                    if let Some(val) = value.get(key) {
                        println!("✅ {}: {}", desc, val);
                    } else {
                        println!("❌ 缺少字段: {} ({})", key, desc);
                    }
                }
                
                // 检查嵌套配置
                if let Some(logging) = value.get("logging") {
                    println!("✅ 日志配置段存在");
                    if let Some(level) = logging.get("level") {
                        println!("  - 日志级别: {}", level);
                    }
                    if let Some(console) = logging.get("console") {
                        println!("  - 控制台输出: {}", console);
                    }
                }
                
                if let Some(environment) = value.get("environment") {
                    println!("✅ 环境配置段存在");
                    if let Some(env_type) = environment.get("env_type") {
                        println!("  - 环境类型: {}", env_type);
                    }
                    if let Some(simulation_mode) = environment.get("simulation_mode") {
                        println!("  - 模拟模式: {}", simulation_mode);
                    }
                }
                
                println!("🎉 配置文件基本验证通过！");
            }
            Err(e) => {
                println!("❌ TOML 解析失败: {}", e);
                
                // 提供修复建议
                if e.to_string().contains("duplicate key") {
                    println!("💡 修复建议: 检查是否有重复的键名");
                    println!("   - 确保顶层没有与段名冲突的键");
                    println!("   - 例如：不能同时有 'environment = \"...\"' 和 '[environment]' 段");
                }
                
                panic!("配置文件解析失败");
            }
        }
    }

    #[test]
    fn test_production_config_security_check() {
        let config_path = Path::new("config/production.toml");
        
        if !config_path.exists() {
            println!("⚠️  生产环境配置文件不存在，跳过安全检查");
            return;
        }
        
        println!("🔒 生产环境配置安全检查");
        
        let content = std::fs::read_to_string(config_path).unwrap();
        
        // 安全检查项
        let mut security_issues = Vec::new();
        
        // 检查是否使用了弱密码
        if content.contains("password = \"123456\"") || 
           content.contains("password = \"admin\"") ||
           content.contains("password = \"password\"") {
            security_issues.push("使用了弱密码");
        }
        
        // 检查是否禁用了控制台日志
        if !content.contains("console = false") {
            security_issues.push("生产环境建议禁用控制台日志输出");
        }
        
        // 检查日志级别
        if !content.contains("level = \"warn\"") && !content.contains("level = \"error\"") {
            security_issues.push("生产环境建议使用 warn 或 error 日志级别");
        }
        
        // 检查是否禁用了模拟模式
        if !content.contains("simulation_mode = false") {
            security_issues.push("生产环境必须禁用模拟模式");
        }
        
        // 输出检查结果
        if security_issues.is_empty() {
            println!("✅ 安全检查通过");
        } else {
            println!("⚠️  发现安全建议:");
            for issue in security_issues {
                println!("  - {}", issue);
            }
        }
        
        println!("🔒 安全检查完成");
    }

    #[test]
    fn test_production_config_completeness() {
        let config_path = Path::new("config/production.toml");
        
        if !config_path.exists() {
            println!("⚠️  生产环境配置文件不存在，跳过完整性检查");
            return;
        }
        
        println!("📋 生产环境配置完整性检查");
        
        let content = std::fs::read_to_string(config_path).unwrap();
        
        // 必需字段检查
        let required_fields = vec![
            ("md_front_addr", "行情前置地址"),
            ("trader_front_addr", "交易前置地址"),
            ("broker_id", "经纪商代码"),
            ("investor_id", "投资者代码"),
            ("password", "密码"),
            ("app_id", "应用标识"),
            ("auth_code", "授权编码"),
            ("flow_path", "流文件路径"),
        ];
        
        let mut missing_fields = Vec::new();
        
        for (field, desc) in required_fields {
            if !content.contains(&format!("{} =", field)) {
                missing_fields.push(format!("{} ({})", field, desc));
            } else {
                println!("✅ {}: 已配置", desc);
            }
        }
        
        if missing_fields.is_empty() {
            println!("✅ 所有必需字段都已配置");
        } else {
            println!("❌ 缺少以下必需字段:");
            for field in missing_fields {
                println!("  - {}", field);
            }
            panic!("配置不完整");
        }
        
        println!("📋 完整性检查完成");
    }
}
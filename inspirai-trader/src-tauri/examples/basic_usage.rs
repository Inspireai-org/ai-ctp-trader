use inspirai_trader_lib::ctp::{
    ConfigManager, Environment, CtpClient, CtpConfig, 
    init_with_config, LoggerManager, PerformanceMonitor
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CTP 交易组件基础使用示例");
    
    // 1. 创建配置
    let mut config = CtpConfig::for_environment(
        Environment::SimNow,
        "your_user_id".to_string(),
        "your_password".to_string(),
    );
    
    // 尝试自动检测动态库路径
    if let Err(e) = config.auto_detect_dynlib_paths() {
        println!("警告: 自动检测动态库路径失败: {}", e);
        println!("请确保 CTP 库文件已正确放置");
    }
    
    // 2. 创建扩展配置
    let extended_config = inspirai_trader_lib::ctp::ExtendedCtpConfig {
        ctp: config.clone(),
        logging: inspirai_trader_lib::ctp::config_manager::LoggingConfig::for_environment(Environment::SimNow),
        environment: inspirai_trader_lib::ctp::config_manager::EnvironmentConfig::for_environment(Environment::SimNow),
    };
    
    // 3. 初始化组件（包括日志系统）
    if let Err(e) = init_with_config(&extended_config) {
        println!("初始化失败: {}", e);
        return Ok(());
    }
    
    println!("✓ CTP 组件初始化成功");
    
    // 4. 创建客户端
    let monitor = PerformanceMonitor::start("client_creation");
    let mut client = match CtpClient::new(config).await {
        Ok(client) => {
            monitor.finish();
            println!("✓ CTP 客户端创建成功");
            client
        }
        Err(e) => {
            println!("✗ 创建客户端失败: {}", e);
            return Ok(());
        }
    };
    
    // 5. 显示客户端信息
    let config_info = client.get_config_info();
    println!("客户端配置信息:");
    println!("  环境: {:?}", config_info.environment);
    println!("  经纪商: {}", config_info.broker_id);
    println!("  用户ID: {}", config_info.user_id);
    println!("  行情服务器: {}", config_info.md_front_addr);
    println!("  交易服务器: {}", config_info.trader_front_addr);
    
    // 6. 健康检查
    let health = client.health_check().await?;
    println!("健康状态: {:?}", health.is_healthy);
    println!("当前状态: {:?}", health.state);
    
    // 7. 尝试连接（这会失败，因为没有实际的 CTP 库）
    println!("\n尝试连接到 CTP 服务器...");
    match client.connect_with_retry().await {
        Ok(_) => {
            println!("✓ 连接成功");
            
            // 显示连接统计
            let stats = client.get_connection_stats();
            println!("连接统计:");
            println!("  状态: {:?}", stats.state);
            println!("  重连次数: {}", stats.reconnect_count);
            if let Some(duration) = stats.connect_duration {
                println!("  连接耗时: {:?}", duration);
            }
        }
        Err(e) => {
            println!("✗ 连接失败: {}", e);
            println!("这是预期的，因为没有实际的 CTP 库文件");
        }
    }
    
    // 8. 演示配置文件操作
    println!("\n演示配置文件操作...");
    
    // 创建所有环境的默认配置文件
    if let Err(e) = ConfigManager::create_default_configs().await {
        println!("创建默认配置文件失败: {}", e);
    } else {
        println!("✓ 默认配置文件创建成功");
    }
    
    // 验证配置文件
    for env in [Environment::SimNow, Environment::Tts, Environment::Production] {
        let config_path = ConfigManager::get_config_path(env);
        match ConfigManager::validate_config_file(&config_path).await {
            Ok(_) => println!("✓ {} 环境配置验证通过", env),
            Err(e) => println!("✗ {} 环境配置验证失败: {}", env, e),
        }
    }
    
    // 9. 清理
    client.disconnect();
    println!("\n✓ 客户端已断开连接");
    
    println!("\n示例程序执行完成！");
    Ok(())
}
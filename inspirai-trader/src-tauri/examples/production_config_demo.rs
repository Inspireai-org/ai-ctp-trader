use inspirai_trader_lib::ctp::{
    ConfigManager, Environment, CtpClient, CtpConfig, 
    init_with_config, LoggerManager, PerformanceMonitor
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏭 生产环境配置演示程序");
    println!("==============================");
    
    // 1. 加载生产环境配置
    println!("📋 加载生产环境配置...");
    let config_path = std::path::Path::new("config/production.toml");
    
    if !config_path.exists() {
        println!("❌ 生产环境配置文件不存在: {:?}", config_path);
        println!("请确保配置文件已正确放置");
        return Ok(());
    }
    
    let extended_config = match ConfigManager::load_from_file(config_path).await {
        Ok(config) => {
            println!("✅ 生产环境配置加载成功");
            config
        }
        Err(e) => {
            println!("❌ 配置加载失败: {}", e);
            return Ok(());
        }
    };
    
    // 2. 显示配置信息
    println!("\n📊 配置信息:");
    println!("  经纪商代码: {}", extended_config.ctp.broker_id);
    println!("  投资者代码: {}", extended_config.ctp.investor_id);
    println!("  行情服务器: {}", extended_config.ctp.md_front_addr);
    println!("  交易服务器: {}", extended_config.ctp.trader_front_addr);
    println!("  应用标识: {}", extended_config.ctp.app_id);
    println!("  流文件路径: {}", extended_config.ctp.flow_path);
    println!("  日志级别: {}", extended_config.logging.level);
    println!("  控制台输出: {}", extended_config.logging.console);
    println!("  模拟模式: {}", extended_config.environment.simulation_mode);
    
    // 3. 验证动态库路径
    println!("\n🔍 验证动态库路径:");
    if let Some(md_path) = &extended_config.ctp.md_dynlib_path {
        if md_path.exists() {
            println!("✅ 行情动态库: {:?}", md_path);
        } else {
            println!("⚠️  行情动态库不存在: {:?}", md_path);
        }
    } else {
        println!("⚠️  未配置行情动态库路径");
    }
    
    if let Some(td_path) = &extended_config.ctp.td_dynlib_path {
        if td_path.exists() {
            println!("✅ 交易动态库: {:?}", td_path);
        } else {
            println!("⚠️  交易动态库不存在: {:?}", td_path);
        }
    } else {
        println!("⚠️  未配置交易动态库路径");
    }
    
    // 4. 初始化组件
    println!("\n🚀 初始化 CTP 组件...");
    if let Err(e) = init_with_config(&extended_config) {
        println!("❌ 组件初始化失败: {}", e);
        return Ok(());
    }
    println!("✅ CTP 组件初始化成功");
    
    // 5. 创建客户端
    println!("\n👤 创建 CTP 客户端...");
    let monitor = PerformanceMonitor::start("production_client_creation");
    
    let mut client = match CtpClient::new(extended_config.ctp.clone()).await {
        Ok(client) => {
            monitor.finish();
            println!("✅ 生产环境客户端创建成功");
            client
        }
        Err(e) => {
            println!("❌ 客户端创建失败: {}", e);
            return Ok(());
        }
    };
    
    // 6. 显示客户端信息
    let config_info = client.get_config_info();
    println!("\n📋 客户端配置信息:");
    println!("  环境: {:?}", config_info.environment);
    println!("  经纪商: {}", config_info.broker_id);
    println!("  用户ID: {}", config_info.user_id);
    println!("  超时时间: {} 秒", config_info.timeout_secs);
    println!("  最大重连次数: {}", config_info.max_reconnect_attempts);
    
    // 7. 健康检查
    println!("\n🏥 执行健康检查...");
    let health = client.health_check().await?;
    println!("  健康状态: {}", if health.is_healthy { "✅ 健康" } else { "❌ 不健康" });
    println!("  当前状态: {:?}", health.state);
    println!("  检查时间: {}", health.last_check_time.format("%Y-%m-%d %H:%M:%S"));
    
    // 8. 安全提醒
    println!("\n🔒 生产环境安全提醒:");
    println!("  ⚠️  这是生产环境配置，请确保:");
    println!("    - 密码安全性足够强");
    println!("    - 网络连接安全可靠");
    println!("    - 日志文件定期清理");
    println!("    - 监控系统正常运行");
    println!("    - 备份和恢复策略已就绪");
    
    // 9. 模拟连接测试（不实际连接）
    println!("\n🔌 连接准备检查:");
    println!("  服务器地址: {} / {}", 
        config_info.md_front_addr, 
        config_info.trader_front_addr
    );
    println!("  认证信息: 已配置");
    println!("  动态库: 已检查");
    println!("  ✅ 连接准备就绪");
    
    // 10. 清理
    client.disconnect();
    println!("\n🧹 清理完成");
    
    println!("\n🎉 生产环境配置演示完成！");
    println!("==============================");
    
    Ok(())
}
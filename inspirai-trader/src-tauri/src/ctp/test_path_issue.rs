#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctp::{CtpConfig, ExtendedCtpConfig};
    use std::path::PathBuf;

    #[test]
    fn test_config_path_loading() {
        println!("\n=== 测试配置路径加载 ===\n");
        
        // 读取实际的production.toml文件
        let config_path = PathBuf::from("config/production.toml");
        println!("配置文件路径: {:?}", config_path);
        println!("配置文件存在: {}", config_path.exists());
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap();
            let preview_len: usize = content.chars().take(100).map(|c| c.len_utf8()).sum();
            println!("配置文件内容前100字符: {}", &content[..preview_len.min(content.len())]);
            
            // 尝试解析配置
            match toml::from_str::<ExtendedCtpConfig>(&content) {
                Ok(config) => {
                    println!("\n✅ 配置解析成功!");
                    println!("MD动态库路径: {:?}", config.ctp.md_dynlib_path);
                    println!("TD动态库路径: {:?}", config.ctp.td_dynlib_path);
                    
                    if let Some(md_path) = &config.ctp.md_dynlib_path {
                        println!("\nMD库路径检查:");
                        println!("  路径: {:?}", md_path);
                        println!("  存在: {}", md_path.exists());
                        if md_path.exists() {
                            println!("  规范化路径: {:?}", std::fs::canonicalize(md_path).unwrap());
                        }
                    } else {
                        println!("❌ MD动态库路径为None!");
                    }
                    
                    if let Some(td_path) = &config.ctp.td_dynlib_path {
                        println!("\nTD库路径检查:");
                        println!("  路径: {:?}", td_path);
                        println!("  存在: {}", td_path.exists());
                        if td_path.exists() {
                            println!("  规范化路径: {:?}", std::fs::canonicalize(td_path).unwrap());
                        }
                    } else {
                        println!("❌ TD动态库路径为None!");
                    }
                    
                    // 测试get方法
                    println!("\n测试get方法:");
                    match config.ctp.get_md_dynlib_path() {
                        Ok(path) => println!("  get_md_dynlib_path成功: {:?}", path),
                        Err(e) => println!("  ❌ get_md_dynlib_path失败: {}", e),
                    }
                    
                    match config.ctp.get_td_dynlib_path() {
                        Ok(path) => println!("  get_td_dynlib_path成功: {:?}", path),
                        Err(e) => println!("  ❌ get_td_dynlib_path失败: {}", e),
                    }
                    
                } 
                Err(e) => {
                    println!("❌ 配置解析失败: {}", e);
                }
            }
        } else {
            println!("❌ 配置文件不存在!");
        }
    }
}
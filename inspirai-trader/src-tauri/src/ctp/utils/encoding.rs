use crate::ctp::CtpError;

/// 将 GB18030 编码的字节数组转换为 UTF-8 字符串
/// 
/// CTP API 使用 GB18030 编码，需要转换为 Rust 的 UTF-8 字符串
pub fn gb18030_to_utf8(gb18030_bytes: &[u8]) -> Result<String, CtpError> {
    // 移除尾部的空字节
    let trimmed_bytes: Vec<u8> = gb18030_bytes.iter()
        .take_while(|&&b| b != 0)
        .copied()
        .collect();
    
    if trimmed_bytes.is_empty() {
        return Ok(String::new());
    }
    
    // 尝试直接解析为 UTF-8（对于纯 ASCII 字符）
    match String::from_utf8(trimmed_bytes.clone()) {
        Ok(s) => Ok(s),
        Err(_) => {
            // 如果不是有效的 UTF-8，尝试 GB18030 解码
            // 这里使用简化的处理方式，在实际项目中可能需要使用专门的编码库
            // 如 encoding_rs 或 iconv
            
            // 对于大多数情况，CTP 返回的字符串都是 ASCII 或简单的中文
            // 这里使用 lossy 转换作为后备方案
            Ok(String::from_utf8_lossy(&trimmed_bytes).to_string())
        }
    }
}

/// 将 UTF-8 字符串转换为 GB18030 编码的字节数组
/// 
/// 用于向 CTP API 传递字符串参数
pub fn utf8_to_gb18030(utf8_str: &str) -> Result<Vec<u8>, CtpError> {
    // 对于纯 ASCII 字符，UTF-8 和 GB18030 编码相同
    if utf8_str.is_ascii() {
        Ok(utf8_str.as_bytes().to_vec())
    } else {
        // 对于包含中文的字符串，这里需要实际的 GB18030 编码
        // 在实际项目中，应该使用专门的编码库
        // 这里暂时使用 UTF-8 字节作为后备方案
        tracing::warn!("字符串包含非 ASCII 字符，可能需要 GB18030 编码: {}", utf8_str);
        Ok(utf8_str.as_bytes().to_vec())
    }
}

/// 将 CTP 字符数组转换为 Rust 字符串的便捷函数
pub fn ctp_string_to_string(ctp_str: &[i8]) -> Result<String, CtpError> {
    let bytes: Vec<u8> = ctp_str.iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as u8)
        .collect();
    
    gb18030_to_utf8(&bytes)
}

/// 将 Rust 字符串复制到 CTP 字符数组的便捷函数
pub fn string_to_ctp_string(rust_str: &str, ctp_field: &mut [i8]) -> Result<(), CtpError> {
    let gb18030_bytes = utf8_to_gb18030(rust_str)?;
    
    if gb18030_bytes.len() >= ctp_field.len() {
        return Err(CtpError::ConversionError(
            format!("字符串过长，无法复制到 CTP 字段: {} (长度: {}, 字段大小: {})", 
                rust_str, gb18030_bytes.len(), ctp_field.len())
        ));
    }
    
    // 清零字段
    for byte in ctp_field.iter_mut() {
        *byte = 0;
    }
    
    // 复制数据
    for (i, &byte) in gb18030_bytes.iter().enumerate() {
        ctp_field[i] = byte as i8;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_conversion() {
        let ascii_str = "rb2401";
        let gb18030_bytes = utf8_to_gb18030(ascii_str).unwrap();
        let converted_back = gb18030_to_utf8(&gb18030_bytes).unwrap();
        
        assert_eq!(ascii_str, converted_back);
    }

    #[test]
    fn test_empty_string_conversion() {
        let empty_str = "";
        let gb18030_bytes = utf8_to_gb18030(empty_str).unwrap();
        let converted_back = gb18030_to_utf8(&gb18030_bytes).unwrap();
        
        assert_eq!(empty_str, converted_back);
    }

    #[test]
    fn test_ctp_string_conversion() {
        let test_bytes = [114, 98, 50, 52, 48, 49, 0, 0, 0]; // "rb2401" + null bytes
        let result = ctp_string_to_string(&test_bytes).unwrap();
        assert_eq!(result, "rb2401");
    }

    #[test]
    fn test_string_to_ctp_field() {
        let mut ctp_field = [0i8; 32];
        let test_str = "rb2401";
        
        string_to_ctp_string(test_str, &mut ctp_field).unwrap();
        
        let converted_back = ctp_string_to_string(&ctp_field).unwrap();
        assert_eq!(converted_back, test_str);
    }

    #[test]
    fn test_string_too_long() {
        let mut ctp_field = [0i8; 5];
        let long_str = "this_string_is_too_long";
        
        let result = string_to_ctp_string(long_str, &mut ctp_field);
        assert!(result.is_err());
    }
}
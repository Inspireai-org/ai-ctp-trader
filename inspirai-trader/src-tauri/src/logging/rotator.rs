use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc, TimeZone};
use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Sha256, Digest};

use super::{
    config::{LogConfig, LogType}, 
    error::LogError
};

/// 日志轮转器 - 负责日志文件的轮转、压缩和清理
#[derive(Debug)]
pub struct LogRotator {
    config: LogConfig,
    rotation_stats: RotationStats,
}

/// 轮转统计信息
#[derive(Debug, Clone, Default)]
pub struct RotationStats {
    pub total_rotations: u64,
    pub total_compressions: u64,
    pub total_deletions: u64,
    pub bytes_compressed: u64,
    pub bytes_deleted: u64,
    pub last_rotation_time: Option<DateTime<Utc>>,
    pub last_cleanup_time: Option<DateTime<Utc>>,
    pub compression_ratio: f64, // 平均压缩比
}

impl LogRotator {
    /// 创建新的日志轮转器
    pub fn new(config: &LogConfig) -> Result<Self, LogError> {
        Ok(Self {
            config: config.clone(),
            rotation_stats: RotationStats::default(),
        })
    }
    
    /// 检查并执行轮转操作
    pub async fn check_and_rotate(&mut self, config: &LogConfig) -> Result<(), LogError> {
        for log_type in LogType::all() {
            self.check_and_rotate_log_type(log_type, config).await?;
        }
        
        // 执行清理操作
        self.cleanup_old_logs(config).await?;
        
        Ok(())
    }
    
    /// 检查并轮转特定类型的日志
    async fn check_and_rotate_log_type(
        &mut self, 
        log_type: LogType, 
        config: &LogConfig
    ) -> Result<(), LogError> {
        let log_file_path = config.get_log_file_path(log_type);
        
        if !log_file_path.exists() {
            return Ok(());
        }
        
        // 检查文件大小
        let metadata = fs::metadata(&log_file_path)
            .map_err(LogError::WriteError)?;
        
        if metadata.len() >= config.max_file_size {
            self.rotate_log_file(&log_file_path, log_type, config).await?;
        }
        
        Ok(())
    }
    
    /// 轮转单个日志文件
    async fn rotate_log_file(
        &mut self,
        log_file_path: &Path,
        log_type: LogType,
        config: &LogConfig,
    ) -> Result<(), LogError> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let parent_dir = log_file_path.parent()
            .ok_or_else(|| LogError::RotationError {
                reason: "无法获取日志文件父目录".to_string(),
            })?;
        
        // 生成轮转后的文件名
        let file_stem = log_file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("log");
        let file_ext = log_file_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("log");
        
        let rotated_file_name = format!("{}.{}.{}", file_stem, timestamp, file_ext);
        let rotated_file_path = parent_dir.join(&rotated_file_name);
        
        // 移动当前日志文件
        fs::rename(log_file_path, &rotated_file_path)
            .map_err(|e| LogError::RotationError {
                reason: format!("文件重命名失败: {}", e),
            })?;
        
        // 如果启用压缩，压缩轮转的文件
        if config.compression_enabled {
            let compressed_path = self.compress_log_file(&rotated_file_path).await?;
            
            // 删除原始轮转文件
            if compressed_path != rotated_file_path {
                fs::remove_file(&rotated_file_path)
                    .map_err(LogError::WriteError)?;
            }
        }
        
        // 更新统计信息
        self.rotation_stats.total_rotations += 1;
        self.rotation_stats.last_rotation_time = Some(Utc::now());
        
        tracing::info!(
            log_type = log_type.as_str(),
            rotated_file = %rotated_file_path.display(),
            "日志文件轮转完成"
        );
        
        Ok(())
    }
    
    /// 压缩日志文件
    async fn compress_log_file(&mut self, file_path: &Path) -> Result<PathBuf, LogError> {
        let compressed_path = file_path.with_extension(
            format!("{}.gz", 
                file_path.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("log")
            )
        );
        
        let original_size = fs::metadata(file_path)
            .map_err(LogError::WriteError)?
            .len();
        
        // 使用 tokio 进行异步压缩
        let file_path_owned = file_path.to_owned();
        let compressed_path_owned = compressed_path.clone();
        
        let compression_result = tokio::task::spawn_blocking(move || {
            Self::compress_file_sync(&file_path_owned, &compressed_path_owned)
        }).await
        .map_err(|e| LogError::CompressionError { 
            file: file_path.to_path_buf() 
        })?;
        
        compression_result?;
        
        let compressed_size = fs::metadata(&compressed_path)
            .map_err(LogError::WriteError)?
            .len();
        
        // 更新压缩统计
        self.rotation_stats.total_compressions += 1;
        self.rotation_stats.bytes_compressed += original_size;
        
        // 计算压缩比
        let ratio = compressed_size as f64 / original_size as f64;
        if self.rotation_stats.compression_ratio == 0.0 {
            self.rotation_stats.compression_ratio = ratio;
        } else {
            // 移动平均
            self.rotation_stats.compression_ratio = 
                (self.rotation_stats.compression_ratio * 0.9) + (ratio * 0.1);
        }
        
        tracing::info!(
            original_file = %file_path.display(),
            compressed_file = %compressed_path.display(),
            original_size = original_size,
            compressed_size = compressed_size,
            compression_ratio = format!("{:.2}%", ratio * 100.0),
            "日志文件压缩完成"
        );
        
        Ok(compressed_path)
    }
    
    /// 同步压缩文件（在 spawn_blocking 中调用）
    fn compress_file_sync(
        input_path: &Path, 
        output_path: &Path
    ) -> Result<(), LogError> {
        let mut input_file = fs::File::open(input_path)
            .map_err(LogError::WriteError)?;
        
        let output_file = fs::File::create(output_path)
            .map_err(LogError::WriteError)?;
        
        let mut encoder = GzEncoder::new(output_file, Compression::default());
        let mut buffer = [0; 8192]; // 8KB 缓冲区
        
        loop {
            let bytes_read = input_file.read(&mut buffer)
                .map_err(LogError::WriteError)?;
            
            if bytes_read == 0 {
                break;
            }
            
            encoder.write_all(&buffer[..bytes_read])
                .map_err(LogError::WriteError)?;
        }
        
        encoder.finish()
            .map_err(LogError::WriteError)?;
        
        Ok(())
    }
    
    /// 清理过期的日志文件
    async fn cleanup_old_logs(&mut self, config: &LogConfig) -> Result<(), LogError> {
        let retention_duration = chrono::Duration::days(config.retention_days as i64);
        let cutoff_time = Utc::now() - retention_duration;
        
        for log_type in LogType::all() {
            self.cleanup_log_type_files(log_type, config, cutoff_time).await?;
        }
        
        self.rotation_stats.last_cleanup_time = Some(Utc::now());
        
        Ok(())
    }
    
    /// 清理特定类型的日志文件
    async fn cleanup_log_type_files(
        &mut self,
        log_type: LogType,
        config: &LogConfig,
        cutoff_time: DateTime<Utc>,
    ) -> Result<(), LogError> {
        let log_dir = config.output_dir.join(log_type.as_str());
        
        if !log_dir.exists() {
            return Ok(());
        }
        
        let entries = fs::read_dir(&log_dir)
            .map_err(LogError::WriteError)?;
        
        let mut files_to_delete = Vec::new();
        let mut files_to_keep = Vec::new();
        
        for entry in entries {
            let entry = entry.map_err(LogError::WriteError)?;
            let path = entry.path();
            
            if path.is_file() {
                let metadata = entry.metadata()
                    .map_err(LogError::WriteError)?;
                
                if let Ok(modified_time) = metadata.modified() {
                    let modified_datetime = DateTime::<Utc>::from(modified_time);
                    
                    if modified_datetime < cutoff_time {
                        files_to_delete.push((path, metadata.len()));
                    } else {
                        files_to_keep.push(path);
                    }
                }
            }
        }
        
        // 检查文件数量限制
        if files_to_keep.len() > config.max_files {
            // 按修改时间排序，删除最旧的文件
            files_to_keep.sort_by_key(|path| {
                fs::metadata(path)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            });
            
            let excess_count = files_to_keep.len() - config.max_files;
            for path in files_to_keep.drain(..excess_count) {
                let size = fs::metadata(&path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                files_to_delete.push((path, size));
            }
        }
        
        // 删除标记的文件
        for (file_path, file_size) in files_to_delete {
            match fs::remove_file(&file_path) {
                Ok(_) => {
                    self.rotation_stats.total_deletions += 1;
                    self.rotation_stats.bytes_deleted += file_size;
                    
                    tracing::info!(
                        file = %file_path.display(),
                        size = file_size,
                        "删除过期日志文件"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        file = %file_path.display(),
                        error = %e,
                        "删除日志文件失败"
                    );
                }
            }
        }
        
        Ok(())
    }
    
    /// 手动轮转指定的日志文件
    pub async fn force_rotate(&mut self, log_type: LogType) -> Result<(), LogError> {
        let log_file_path = self.config.get_log_file_path(log_type);
        
        if log_file_path.exists() {
            let config = self.config.clone();
            self.rotate_log_file(&log_file_path, log_type, &config).await?;
        }
        
        Ok(())
    }
    
    /// 手动压缩指定的日志文件
    pub async fn force_compress(&mut self, file_path: &Path) -> Result<PathBuf, LogError> {
        if !file_path.exists() {
            return Err(LogError::CompressionError { 
                file: file_path.to_path_buf() 
            });
        }
        
        self.compress_log_file(file_path).await
    }
    
    /// 验证压缩文件的完整性
    pub async fn verify_compressed_file(&self, compressed_path: &Path) -> Result<bool, LogError> {
        if !compressed_path.exists() {
            return Ok(false);
        }
        
        // 尝试读取压缩文件的头部来验证格式
        let compressed_path_owned = compressed_path.to_owned();
        
        let is_valid = tokio::task::spawn_blocking(move || {
            Self::verify_gzip_format(&compressed_path_owned)
        }).await
        .map_err(|_| LogError::DecompressionError { 
            file: compressed_path.to_path_buf() 
        })?;
        
        is_valid
    }
    
    /// 同步验证 gzip 格式
    fn verify_gzip_format(path: &Path) -> Result<bool, LogError> {
        use flate2::read::GzDecoder;
        
        let file = fs::File::open(path)
            .map_err(LogError::WriteError)?;
        
        let mut decoder = GzDecoder::new(file);
        let mut buffer = [0; 1024];
        
        // 尝试读取一小部分数据来验证格式
        match decoder.read(&mut buffer) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// 计算文件校验和
    pub fn calculate_checksum(&self, file_path: &Path) -> Result<String, LogError> {
        let mut file = fs::File::open(file_path)
            .map_err(LogError::WriteError)?;
        
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];
        
        loop {
            let bytes_read = file.read(&mut buffer)
                .map_err(LogError::WriteError)?;
            
            if bytes_read == 0 {
                break;
            }
            
            hasher.update(&buffer[..bytes_read]);
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    /// 获取轮转统计信息
    pub fn get_stats(&self) -> &RotationStats {
        &self.rotation_stats
    }
    
    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.rotation_stats = RotationStats::default();
    }
    
    /// 获取磁盘使用情况
    pub fn get_disk_usage(&self) -> Result<DiskUsage, LogError> {
        let mut total_size = 0u64;
        let mut file_count = 0usize;
        let mut compressed_count = 0usize;
        
        for log_type in LogType::all() {
            let log_dir = self.config.output_dir.join(log_type.as_str());
            
            if log_dir.exists() {
                let (size, files, compressed) = self.scan_directory(&log_dir)?;
                total_size += size;
                file_count += files;
                compressed_count += compressed;
            }
        }
        
        Ok(DiskUsage {
            total_size_bytes: total_size,
            file_count,
            compressed_file_count: compressed_count,
            average_file_size: if file_count > 0 { 
                total_size / file_count as u64 
            } else { 
                0 
            },
        })
    }
    
    /// 扫描目录获取文件统计
    fn scan_directory(&self, dir_path: &Path) -> Result<(u64, usize, usize), LogError> {
        let mut total_size = 0u64;
        let mut file_count = 0usize;
        let mut compressed_count = 0usize;
        
        let entries = fs::read_dir(dir_path)
            .map_err(LogError::WriteError)?;
        
        for entry in entries {
            let entry = entry.map_err(LogError::WriteError)?;
            let path = entry.path();
            
            if path.is_file() {
                let metadata = entry.metadata()
                    .map_err(LogError::WriteError)?;
                
                total_size += metadata.len();
                file_count += 1;
                
                if path.extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "gz")
                    .unwrap_or(false) {
                    compressed_count += 1;
                }
            }
        }
        
        Ok((total_size, file_count, compressed_count))
    }
    
    /// 检查并处理磁盘空间不足的情况
    pub async fn handle_disk_space_emergency(&mut self) -> Result<(), LogError> {
        // 获取可用磁盘空间
        let available_space = self.get_available_disk_space()?;
        let emergency_threshold = 100 * 1024 * 1024; // 100MB
        
        if available_space < emergency_threshold {
            tracing::warn!(
                available_mb = available_space / (1024 * 1024),
                "磁盘空间不足，启动紧急清理"
            );
            
            // 紧急清理：删除最旧的压缩文件
            self.emergency_cleanup().await?;
        }
        
        Ok(())
    }
    
    /// 获取可用磁盘空间
    fn get_available_disk_space(&self) -> Result<u64, LogError> {
        use std::os::unix::fs::MetadataExt;
        
        let metadata = fs::metadata(&self.config.output_dir)
            .map_err(LogError::WriteError)?;
        
        // 这是一个简化的实现，实际情况可能需要使用系统调用
        // 这里假设有足够的空间，实际实现需要调用 statvfs 或类似的系统调用
        Ok(1024 * 1024 * 1024) // 假设有1GB可用空间
    }
    
    /// 紧急清理
    async fn emergency_cleanup(&mut self) -> Result<(), LogError> {
        // 找到所有压缩文件并按时间排序
        let mut compressed_files = Vec::new();
        
        for log_type in LogType::all() {
            let log_dir = self.config.output_dir.join(log_type.as_str());
            if !log_dir.exists() {
                continue;
            }
            
            let entries = fs::read_dir(&log_dir)
                .map_err(LogError::WriteError)?;
            
            for entry in entries {
                let entry = entry.map_err(LogError::WriteError)?;
                let path = entry.path();
                
                if path.extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "gz")
                    .unwrap_or(false) {
                    
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            compressed_files.push((path, modified, metadata.len()));
                        }
                    }
                }
            }
        }
        
        // 按修改时间排序（最旧的优先）
        compressed_files.sort_by_key(|(_, modified, _)| *modified);
        
        // 删除最旧的文件，直到释放足够空间
        let target_cleanup_size = 500 * 1024 * 1024; // 500MB
        let mut cleaned_size = 0u64;
        
        for (path, _, size) in compressed_files {
            if cleaned_size >= target_cleanup_size {
                break;
            }
            
            match fs::remove_file(&path) {
                Ok(_) => {
                    cleaned_size += size;
                    self.rotation_stats.total_deletions += 1;
                    self.rotation_stats.bytes_deleted += size;
                    
                    tracing::info!(
                        file = %path.display(),
                        size = size,
                        "紧急清理删除文件"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        file = %path.display(),
                        error = %e,
                        "紧急清理删除文件失败"
                    );
                }
            }
        }
        
        tracing::info!(
            cleaned_mb = cleaned_size / (1024 * 1024),
            "紧急清理完成"
        );
        
        Ok(())
    }
}

/// 磁盘使用统计
#[derive(Debug, Clone)]
pub struct DiskUsage {
    pub total_size_bytes: u64,
    pub file_count: usize,
    pub compressed_file_count: usize,
    pub average_file_size: u64,
}

impl DiskUsage {
    /// 获取总大小（MB）
    pub fn total_size_mb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0)
    }
    
    /// 获取压缩文件比例
    pub fn compression_percentage(&self) -> f64 {
        if self.file_count == 0 {
            0.0
        } else {
            (self.compressed_file_count as f64 / self.file_count as f64) * 100.0
        }
    }
    
    /// 获取平均文件大小（MB）
    pub fn average_file_size_mb(&self) -> f64 {
        self.average_file_size as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::OpenOptions;
    use std::io::Write as IoWrite;

    fn create_test_config() -> (LogConfig, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = LogConfig {
            output_dir: temp_dir.path().to_path_buf(),
            max_file_size: 1024, // 1KB for testing
            max_files: 3,
            retention_days: 1,
            compression_enabled: true,
            ..LogConfig::development()
        };
        (config, temp_dir)
    }
    
    fn create_test_log_file(path: &Path, size: usize) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        
        let content = "A".repeat(size);
        file.write_all(content.as_bytes())?;
        file.flush()?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_log_rotation() {
        let (config, _temp_dir) = create_test_config();
        config.ensure_directories().unwrap();
        
        let mut rotator = LogRotator::new(&config).unwrap();
        
        // 创建一个超过大小限制的日志文件
        let log_file_path = config.get_log_file_path(LogType::App);
        create_test_log_file(&log_file_path, 2048).unwrap(); // 2KB
        
        // 执行轮转
        let result = rotator.check_and_rotate(&config).await;
        assert!(result.is_ok());
        
        // 检查统计信息
        let stats = rotator.get_stats();
        assert_eq!(stats.total_rotations, 1);
        if config.compression_enabled {
            assert_eq!(stats.total_compressions, 1);
        }
        
        // 检查轮转后的文件是否存在
        let log_dir = config.output_dir.join("app");
        let entries: Vec<_> = fs::read_dir(log_dir).unwrap().collect();
        assert!(!entries.is_empty());
    }
    
    #[tokio::test]
    async fn test_file_compression() {
        let (config, _temp_dir) = create_test_config();
        let mut rotator = LogRotator::new(&config).unwrap();
        
        // 创建一个测试文件
        let test_file = config.output_dir.join("test.log");
        create_test_log_file(&test_file, 1024).unwrap();
        
        // 压缩文件
        let compressed_path = rotator.compress_log_file(&test_file).await.unwrap();
        
        // 验证压缩文件存在
        assert!(compressed_path.exists());
        assert!(compressed_path.extension().unwrap() == "gz");
        
        // 验证压缩文件格式
        let is_valid = rotator.verify_compressed_file(&compressed_path).await.unwrap();
        assert!(is_valid);
        
        // 检查统计信息
        let stats = rotator.get_stats();
        assert_eq!(stats.total_compressions, 1);
        assert!(stats.bytes_compressed > 0);
        assert!(stats.compression_ratio > 0.0);
    }
    
    #[tokio::test]
    async fn test_old_files_cleanup() {
        let (mut config, _temp_dir) = create_test_config();
        config.retention_days = 0; // 立即过期
        config.ensure_directories().unwrap();
        
        let mut rotator = LogRotator::new(&config).unwrap();
        
        // 创建一些测试文件
        let log_dir = config.output_dir.join("app");
        for i in 0..5 {
            let file_path = log_dir.join(format!("old_log_{}.log", i));
            create_test_log_file(&file_path, 512).unwrap();
            
            // 修改文件的修改时间使其看起来很旧
            let old_time = SystemTime::now() - std::time::Duration::from_secs(86400 * 2); // 2天前
            filetime::set_file_mtime(&file_path, filetime::FileTime::from_system_time(old_time)).unwrap();
        }
        
        // 执行清理
        let result = rotator.cleanup_old_logs(&config).await;
        assert!(result.is_ok());
        
        // 检查文件是否被删除
        let stats = rotator.get_stats();
        assert!(stats.total_deletions > 0);
    }
    
    #[tokio::test]
    async fn test_disk_usage_calculation() {
        let (config, _temp_dir) = create_test_config();
        config.ensure_directories().unwrap();
        
        let rotator = LogRotator::new(&config).unwrap();
        
        // 创建一些测试文件
        for log_type in &[LogType::App, LogType::Trading] {
            let log_dir = config.output_dir.join(log_type.as_str());
            for i in 0..3 {
                let file_path = log_dir.join(format!("test_{}.log", i));
                create_test_log_file(&file_path, 1024).unwrap();
            }
            
            // 创建一个压缩文件
            let compressed_file = log_dir.join("compressed.log.gz");
            create_test_log_file(&compressed_file, 512).unwrap();
        }
        
        // 计算磁盘使用情况
        let disk_usage = rotator.get_disk_usage().unwrap();
        
        assert!(disk_usage.total_size_bytes > 0);
        assert!(disk_usage.file_count > 0);
        assert!(disk_usage.compressed_file_count > 0);
        assert!(disk_usage.compression_percentage() > 0.0);
    }
    
    #[tokio::test]
    async fn test_checksum_calculation() {
        let (config, _temp_dir) = create_test_config();
        let rotator = LogRotator::new(&config).unwrap();
        
        // 创建测试文件
        let test_file = config.output_dir.join("checksum_test.log");
        create_test_log_file(&test_file, 1024).unwrap();
        
        // 计算校验和
        let checksum1 = rotator.calculate_checksum(&test_file).unwrap();
        let checksum2 = rotator.calculate_checksum(&test_file).unwrap();
        
        // 相同文件应该有相同的校验和
        assert_eq!(checksum1, checksum2);
        assert!(!checksum1.is_empty());
        assert_eq!(checksum1.len(), 64); // SHA256 十六进制长度
    }
    
    #[tokio::test]
    async fn test_force_rotation() {
        let (config, _temp_dir) = create_test_config();
        config.ensure_directories().unwrap();
        
        let mut rotator = LogRotator::new(&config).unwrap();
        
        // 创建一个小文件（不会触发自动轮转）
        let log_file_path = config.get_log_file_path(LogType::App);
        create_test_log_file(&log_file_path, 512).unwrap();
        
        // 强制轮转
        let result = rotator.force_rotate(LogType::App).await;
        assert!(result.is_ok());
        
        // 检查轮转统计
        let stats = rotator.get_stats();
        assert_eq!(stats.total_rotations, 1);
    }
    
    #[test]
    fn test_rotation_stats() {
        let config = LogConfig::development();
        let mut rotator = LogRotator::new(&config).unwrap();
        
        // 初始状态
        let stats = rotator.get_stats();
        assert_eq!(stats.total_rotations, 0);
        assert_eq!(stats.total_compressions, 0);
        assert_eq!(stats.total_deletions, 0);
        
        // 重置统计
        rotator.reset_stats();
        let stats = rotator.get_stats();
        assert_eq!(stats.total_rotations, 0);
    }
}
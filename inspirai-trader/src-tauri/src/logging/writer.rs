use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot, Mutex as AsyncMutex};
use tokio::time::{Duration, Instant};
use std::io::{Write as StdWrite, BufWriter};
use std::fs::OpenOptions;

use super::{
    config::{LogConfig, LogType},
    error::LogError,
    formatter::{LogFormatter, JsonFormatter, HumanReadableFormatter},
    LogEntry,
};

/// 异步日志写入器
#[derive(Debug)]
pub struct AsyncWriter {
    sender: mpsc::UnboundedSender<WriteCommand>,
    handle: tokio::task::JoinHandle<()>,
    metrics: Arc<AsyncMutex<WriterMetrics>>,
}

/// 写入命令
#[derive(Debug)]
enum WriteCommand {
    Write {
        log_type: LogType,
        entry: LogEntry,
    },
    Flush {
        response: oneshot::Sender<Result<(), LogError>>,
    },
    Shutdown,
}

/// 写入器指标
#[derive(Debug, Clone, Default)]
pub struct WriterMetrics {
    pub total_writes: u64,
    pub successful_writes: u64,
    pub failed_writes: u64,
    pub bytes_written: u64,
    pub average_write_time_ms: f64,
    pub queue_size: usize,
    pub last_write_time: Option<Instant>,
    pub flush_count: u64,
}

impl AsyncWriter {
    /// 创建新的异步写入器
    pub async fn new(config: &LogConfig) -> Result<Self, LogError> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let metrics = Arc::new(AsyncMutex::new(WriterMetrics::default()));
        
        // 确保输出目录存在
        config.ensure_directories()?;
        
        // 启动后台写入任务
        let worker_config = config.clone();
        let worker_metrics = metrics.clone();
        let handle = tokio::spawn(async move {
            let mut worker = WriterWorker::new(worker_config, worker_metrics).await;
            worker.run(receiver).await;
        });
        
        Ok(Self {
            sender,
            handle,
            metrics,
        })
    }
    
    /// 异步写入日志条目
    pub fn write_async(&self, log_type: LogType, entry: LogEntry) -> Result<(), LogError> {
        self.sender
            .send(WriteCommand::Write { log_type, entry })
            .map_err(|_| LogError::AsyncError("写入命令发送失败".to_string()))
    }
    
    /// 刷新所有缓冲的日志
    pub async fn flush(&self) -> Result<(), LogError> {
        let (tx, rx) = oneshot::channel();
        
        self.sender
            .send(WriteCommand::Flush { response: tx })
            .map_err(|_| LogError::AsyncError("刷新命令发送失败".to_string()))?;
        
        rx.await
            .map_err(|_| LogError::AsyncError("刷新响应接收失败".to_string()))?
    }
    
    /// 关闭写入器
    pub async fn shutdown(self) -> Result<(), LogError> {
        // 发送关闭命令
        self.sender
            .send(WriteCommand::Shutdown)
            .map_err(|_| LogError::AsyncError("关闭命令发送失败".to_string()))?;
        
        // 等待工作线程完成
        self.handle.await
            .map_err(|e| LogError::AsyncError(format!("等待工作线程关闭失败: {}", e)))?;
        
        Ok(())
    }
    
    /// 获取写入器指标
    pub async fn get_metrics(&self) -> WriterMetrics {
        self.metrics.lock().await.clone()
    }
}

/// 写入器工作线程
struct WriterWorker {
    config: LogConfig,
    formatters: HashMap<LogType, Box<dyn LogFormatter + Send>>,
    file_handles: HashMap<LogType, BufWriter<std::fs::File>>,
    buffer: HashMap<LogType, VecDeque<LogEntry>>,
    last_flush: Instant,
    metrics: Arc<AsyncMutex<WriterMetrics>>,
}

impl WriterWorker {
    async fn new(config: LogConfig, metrics: Arc<AsyncMutex<WriterMetrics>>) -> Self {
        let mut formatters: HashMap<LogType, Box<dyn LogFormatter + Send>> = HashMap::new();
        
        // 为每个日志类型创建格式化器
        for log_type in LogType::all() {
            let formatter: Box<dyn LogFormatter + Send> = match log_type {
                LogType::Performance => Box::new(JsonFormatter::new()),
                LogType::Error => Box::new(JsonFormatter::new()),
                _ => Box::new(HumanReadableFormatter::new()),
            };
            formatters.insert(log_type, formatter);
        }
        
        Self {
            config,
            formatters,
            file_handles: HashMap::new(),
            buffer: HashMap::new(),
            last_flush: Instant::now(),
            metrics,
        }
    }
    
    async fn run(&mut self, mut receiver: mpsc::UnboundedReceiver<WriteCommand>) {
        // 定时刷新任务
        let mut flush_interval = tokio::time::interval(self.config.flush_interval);
        
        loop {
            tokio::select! {
                // 处理写入命令
                cmd = receiver.recv() => {
                    match cmd {
                        Some(WriteCommand::Write { log_type, entry }) => {
                            self.handle_write(log_type, entry).await;
                        }
                        Some(WriteCommand::Flush { response }) => {
                            let result = self.flush_all().await;
                            let _ = response.send(result);
                        }
                        Some(WriteCommand::Shutdown) => {
                            let _ = self.flush_all().await;
                            self.close_all_files().await;
                            break;
                        }
                        None => {
                            // 发送端已关闭，退出
                            break;
                        }
                    }
                }
                
                // 定时刷新
                _ = flush_interval.tick() => {
                    if self.should_flush() {
                        let _ = self.flush_all().await;
                    }
                }
            }
        }
    }
    
    async fn handle_write(&mut self, log_type: LogType, entry: LogEntry) {
        let start_time = Instant::now();
        
        // 更新队列大小指标
        {
            let mut metrics = self.metrics.lock().await;
            metrics.queue_size = self.buffer.values().map(|buf| buf.len()).sum();
        }
        
        // 添加到缓冲区
        self.buffer
            .entry(log_type)
            .or_insert_with(VecDeque::new)
            .push_back(entry);
        
        // 检查是否需要立即刷新
        if self.should_immediate_flush(log_type) {
            let _ = self.flush_log_type(log_type).await;
        }
        
        // 更新指标
        let write_time = start_time.elapsed();
        let mut metrics = self.metrics.lock().await;
        metrics.total_writes += 1;
        
        // 更新平均写入时间（简单移动平均）
        if metrics.average_write_time_ms == 0.0 {
            metrics.average_write_time_ms = write_time.as_secs_f64() * 1000.0;
        } else {
            metrics.average_write_time_ms = 
                (metrics.average_write_time_ms * 0.9) + (write_time.as_secs_f64() * 1000.0 * 0.1);
        }
        
        metrics.last_write_time = Some(Instant::now());
    }
    
    fn should_flush(&self) -> bool {
        // 检查时间间隔
        if self.last_flush.elapsed() >= self.config.flush_interval {
            return true;
        }
        
        // 检查缓冲区大小
        let total_buffered: usize = self.buffer.values().map(|buf| buf.len()).sum();
        if total_buffered >= self.config.batch_size {
            return true;
        }
        
        false
    }
    
    fn should_immediate_flush(&self, log_type: LogType) -> bool {
        // 错误日志立即刷新
        if matches!(log_type, LogType::Error) {
            return true;
        }
        
        // 检查特定类型的缓冲区大小
        if let Some(buffer) = self.buffer.get(&log_type) {
            if buffer.len() >= self.config.batch_size / 2 {
                return true;
            }
        }
        
        false
    }
    
    async fn flush_all(&mut self) -> Result<(), LogError> {
        let mut errors = Vec::new();
        
        for log_type in LogType::all() {
            if let Err(e) = self.flush_log_type(log_type).await {
                errors.push(e);
            }
        }
        
        self.last_flush = Instant::now();
        
        // 更新刷新指标
        {
            let mut metrics = self.metrics.lock().await;
            metrics.flush_count += 1;
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            // 返回第一个错误，其他错误记录到日志
            for error in errors.iter().skip(1) {
                eprintln!("刷新日志时出错: {}", error);
            }
            Err(errors.into_iter().next().unwrap())
        }
    }
    
    async fn flush_log_type(&mut self, log_type: LogType) -> Result<(), LogError> {
        // 先取出缓冲区的内容
        let entries: Vec<LogEntry> = if let Some(buffer) = self.buffer.get_mut(&log_type) {
            if buffer.is_empty() {
                return Ok(());
            }
            buffer.drain(..).collect()
        } else {
            return Ok(());
        };
        
        // 确保文件句柄存在
        if !self.file_handles.contains_key(&log_type) {
            self.create_file_handle(log_type).await?;
        }
        
        // 现在可以安全地获取格式化器和文件句柄
        let formatter = self.formatters.get(&log_type).unwrap();
        let file_handle = self.file_handles.get_mut(&log_type).unwrap();
        
        let mut bytes_written = 0u64;
        let mut successful_writes = 0u64;
        let mut failed_writes = 0u64;
        let mut failed_entries = Vec::new();
        
        // 批量写入条目
        for entry in entries {
            match formatter.format(&entry) {
                Ok(formatted) => {
                    match file_handle.write_all(formatted.as_bytes()) {
                        Ok(_) => {
                            bytes_written += formatted.len() as u64;
                            successful_writes += 1;
                        }
                        Err(e) => {
                            failed_writes += 1;
                            eprintln!("写入日志文件失败: {}", e);
                            // 将失败的条目保存起来
                            failed_entries.push(entry);
                            break;
                        }
                    }
                }
                Err(e) => {
                    failed_writes += 1;
                    eprintln!("格式化日志条目失败: {}", e);
                }
            }
        }
        
        // 将失败的条目放回缓冲区
        if !failed_entries.is_empty() {
            if let Some(buffer) = self.buffer.get_mut(&log_type) {
                for entry in failed_entries.into_iter().rev() {
                    buffer.push_front(entry);
                }
            }
        }
        
        // 刷新文件缓冲区
        if let Err(e) = file_handle.flush() {
            return Err(LogError::WriteError(e));
        }
        
        // 更新指标
        {
            let mut metrics = self.metrics.lock().await;
            metrics.successful_writes += successful_writes;
            metrics.failed_writes += failed_writes;
            metrics.bytes_written += bytes_written;
        }
        
        Ok(())
    }
    
    async fn create_file_handle(&mut self, log_type: LogType) -> Result<(), LogError> {
        if !self.file_handles.contains_key(&log_type) {
            let file_path = self.config.get_log_file_path(log_type);
            
            // 确保父目录存在
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|_| LogError::DirectoryCreationError { 
                            path: parent.to_path_buf() 
                        })?;
                }
            }
            
            // 打开或创建文件
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
                .map_err(LogError::WriteError)?;
            
            let buf_writer = BufWriter::with_capacity(
                self.config.async_buffer_size, 
                file
            );
            
            self.file_handles.insert(log_type, buf_writer);
        }
        
        Ok(())
    }
    
    async fn close_all_files(&mut self) {
        for (log_type, mut file_handle) in self.file_handles.drain() {
            if let Err(e) = file_handle.flush() {
                eprintln!("关闭日志文件 {} 时刷新失败: {}", log_type, e);
            }
        }
    }
}

/// 同步日志写入器（用于简单场景）
#[derive(Debug)]
pub struct SyncWriter {
    config: LogConfig,
    formatters: HashMap<LogType, Box<dyn LogFormatter + Send>>,
    file_handles: Mutex<HashMap<LogType, BufWriter<std::fs::File>>>,
    metrics: Mutex<WriterMetrics>,
}

impl SyncWriter {
    /// 创建新的同步写入器
    pub fn new(config: LogConfig) -> Result<Self, LogError> {
        config.ensure_directories()?;
        
        let mut formatters: HashMap<LogType, Box<dyn LogFormatter + Send>> = HashMap::new();
        
        // 为每个日志类型创建格式化器
        for log_type in LogType::all() {
            let formatter: Box<dyn LogFormatter + Send> = match log_type {
                LogType::Performance => Box::new(JsonFormatter::new()),
                LogType::Error => Box::new(JsonFormatter::new()),
                _ => Box::new(HumanReadableFormatter::new()),
            };
            formatters.insert(log_type, formatter);
        }
        
        Ok(Self {
            config,
            formatters,
            file_handles: Mutex::new(HashMap::new()),
            metrics: Mutex::new(WriterMetrics::default()),
        })
    }
    
    /// 同步写入日志条目
    pub fn write_sync(&self, log_type: LogType, entry: LogEntry) -> Result<(), LogError> {
        let start_time = Instant::now();
        
        let formatter = self.formatters.get(&log_type).unwrap();
        let formatted = formatter.format(&entry)?;
        
        // 获取或创建文件句柄
        {
            let mut handles = self.file_handles.lock().unwrap();
            if !handles.contains_key(&log_type) {
                let file_handle = self.create_file_handle(log_type)?;
                handles.insert(log_type, file_handle);
            }
            
            let file_handle = handles.get_mut(&log_type).unwrap();
            file_handle.write_all(formatted.as_bytes())
                .map_err(LogError::WriteError)?;
            
            file_handle.flush().map_err(LogError::WriteError)?;
        }
        
        // 更新指标
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.total_writes += 1;
            metrics.successful_writes += 1;
            metrics.bytes_written += formatted.len() as u64;
            
            let write_time = start_time.elapsed().as_secs_f64() * 1000.0;
            if metrics.average_write_time_ms == 0.0 {
                metrics.average_write_time_ms = write_time;
            } else {
                metrics.average_write_time_ms = 
                    (metrics.average_write_time_ms * 0.9) + (write_time * 0.1);
            }
            
            metrics.last_write_time = Some(Instant::now());
        }
        
        Ok(())
    }
    
    fn create_file_handle(&self, log_type: LogType) -> Result<BufWriter<std::fs::File>, LogError> {
        let file_path = self.config.get_log_file_path(log_type);
        
        // 确保父目录存在
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|_| LogError::DirectoryCreationError { 
                        path: parent.to_path_buf() 
                    })?;
            }
        }
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(LogError::WriteError)?;
        
        Ok(BufWriter::with_capacity(self.config.async_buffer_size, file))
    }
    
    /// 获取写入器指标
    pub fn get_metrics(&self) -> WriterMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    /// 刷新所有文件
    pub fn flush_all(&self) -> Result<(), LogError> {
        let mut handles = self.file_handles.lock().unwrap();
        
        for (_, file_handle) in handles.iter_mut() {
            file_handle.flush().map_err(LogError::WriteError)?;
        }
        
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.flush_count += 1;
        }
        
        Ok(())
    }
}

/// 内存写入器（用于测试）
#[derive(Debug, Default)]
pub struct MemoryWriter {
    entries: Arc<Mutex<HashMap<LogType, Vec<LogEntry>>>>,
    formatters: HashMap<LogType, Box<dyn LogFormatter + Send>>,
}

impl MemoryWriter {
    /// 创建新的内存写入器
    pub fn new() -> Self {
        let mut formatters: HashMap<LogType, Box<dyn LogFormatter + Send>> = HashMap::new();
        
        for log_type in LogType::all() {
            formatters.insert(log_type, Box::new(JsonFormatter::new()));
        }
        
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            formatters,
        }
    }
    
    /// 写入日志条目到内存
    pub fn write(&self, log_type: LogType, entry: LogEntry) -> Result<(), LogError> {
        let mut entries = self.entries.lock().unwrap();
        entries.entry(log_type).or_insert_with(Vec::new).push(entry);
        Ok(())
    }
    
    /// 获取指定类型的所有日志条目
    pub fn get_entries(&self, log_type: LogType) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries.get(&log_type).cloned().unwrap_or_default()
    }
    
    /// 获取所有日志条目
    pub fn get_all_entries(&self) -> HashMap<LogType, Vec<LogEntry>> {
        self.entries.lock().unwrap().clone()
    }
    
    /// 清空所有日志条目
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
    }
    
    /// 获取条目数量
    pub fn count(&self) -> usize {
        let entries = self.entries.lock().unwrap();
        entries.values().map(|v| v.len()).sum()
    }
    
    /// 获取指定类型的条目数量
    pub fn count_by_type(&self, log_type: LogType) -> usize {
        let entries = self.entries.lock().unwrap();
        entries.get(&log_type).map_or(0, |v| v.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::context::LogContext;
    use tempfile::TempDir;

    fn create_test_config() -> LogConfig {
        let temp_dir = TempDir::new().unwrap();
        LogConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..LogConfig::development()
        }
    }
    
    fn create_test_entry() -> LogEntry {
        let context = LogContext::new(super::super::config::LogLevel::Info, "test_module");
        LogEntry {
            timestamp: chrono::Utc::now(),
            level: super::super::config::LogLevel::Info,
            module: "test_module".to_string(),
            thread_id: "test_thread".to_string(),
            message: "test message".to_string(),
            context,
            request_id: None,
            session_id: None,
            fields: HashMap::new(),
        }
    }
    
    #[tokio::test]
    async fn test_async_writer() {
        let config = create_test_config();
        let writer = AsyncWriter::new(&config).await.unwrap();
        
        let entry = create_test_entry();
        assert!(writer.write_async(LogType::App, entry).is_ok());
        
        // 刷新并检查指标
        assert!(writer.flush().await.is_ok());
        
        let metrics = writer.get_metrics().await;
        assert_eq!(metrics.total_writes, 1);
        assert_eq!(metrics.successful_writes, 1);
        assert!(metrics.bytes_written > 0);
        
        // 关闭写入器
        assert!(writer.shutdown().await.is_ok());
    }
    
    #[test]
    fn test_sync_writer() {
        let config = create_test_config();
        let writer = SyncWriter::new(config).unwrap();
        
        let entry = create_test_entry();
        assert!(writer.write_sync(LogType::App, entry).is_ok());
        
        assert!(writer.flush_all().is_ok());
        
        let metrics = writer.get_metrics();
        assert_eq!(metrics.total_writes, 1);
        assert_eq!(metrics.successful_writes, 1);
    }
    
    #[test]
    fn test_memory_writer() {
        let writer = MemoryWriter::new();
        
        let entry1 = create_test_entry();
        let entry2 = create_test_entry();
        
        assert!(writer.write(LogType::App, entry1).is_ok());
        assert!(writer.write(LogType::Trading, entry2).is_ok());
        
        assert_eq!(writer.count(), 2);
        assert_eq!(writer.count_by_type(LogType::App), 1);
        assert_eq!(writer.count_by_type(LogType::Trading), 1);
        
        let app_entries = writer.get_entries(LogType::App);
        assert_eq!(app_entries.len(), 1);
        
        writer.clear();
        assert_eq!(writer.count(), 0);
    }
    
    #[tokio::test]
    async fn test_writer_metrics() {
        let config = create_test_config();
        let writer = AsyncWriter::new(&config).await.unwrap();
        
        // 写入多个条目
        for i in 0..10 {
            let mut entry = create_test_entry();
            entry.message = format!("test message {}", i);
            assert!(writer.write_async(LogType::App, entry).is_ok());
        }
        
        assert!(writer.flush().await.is_ok());
        
        let metrics = writer.get_metrics().await;
        assert_eq!(metrics.total_writes, 10);
        assert_eq!(metrics.successful_writes, 10);
        assert_eq!(metrics.failed_writes, 0);
        assert!(metrics.bytes_written > 0);
        assert!(metrics.average_write_time_ms >= 0.0);
        assert_eq!(metrics.flush_count, 1);
        
        assert!(writer.shutdown().await.is_ok());
    }
}
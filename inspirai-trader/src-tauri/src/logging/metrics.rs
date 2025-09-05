use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::Instant;
use serde::{Serialize, Deserialize};

use super::config::LogLevel;

/// 日志系统指标收集器
#[derive(Debug)]
pub struct LogMetrics {
    /// 总写入日志数
    pub logs_written_total: u64,
    /// 丢弃的日志数
    pub logs_dropped_total: u64,
    /// 写入延迟直方图（毫秒）
    pub write_latency_ms: Histogram,
    /// 当前队列大小
    pub queue_size: usize,
    /// 磁盘使用量（字节）
    pub disk_usage_bytes: u64,
    /// 错误计数器
    pub error_count: u64,
    /// 按日志级别分组的计数器
    pub level_counters: HashMap<LogLevel, u64>,
    /// 按模块分组的计数器
    pub module_counters: HashMap<String, u64>,
    /// 系统资源指标
    pub system_metrics: SystemMetrics,
}

impl LogMetrics {
    /// 创建新的指标实例
    pub fn new() -> Self {
        Self {
            logs_written_total: 0,
            logs_dropped_total: 0,
            write_latency_ms: Histogram::new(),
            queue_size: 0,
            disk_usage_bytes: 0,
            error_count: 0,
            level_counters: HashMap::new(),
            module_counters: HashMap::new(),
            system_metrics: SystemMetrics::new(),
        }
    }
    
    /// 记录成功写入的日志
    pub fn record_log_written(&mut self, level: LogLevel, module: &str, latency_ms: f64) {
        self.logs_written_total += 1;
        self.write_latency_ms.record(latency_ms);
        
        *self.level_counters.entry(level).or_insert(0) += 1;
        *self.module_counters.entry(module.to_string()).or_insert(0) += 1;
    }
    
    /// 记录丢弃的日志
    pub fn record_log_dropped(&mut self) {
        self.logs_dropped_total += 1;
    }
    
    /// 记录错误
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }
    
    /// 更新队列大小
    pub fn update_queue_size(&mut self, size: usize) {
        self.queue_size = size;
    }
    
    /// 更新磁盘使用量
    pub fn update_disk_usage(&mut self, bytes: u64) {
        self.disk_usage_bytes = bytes;
    }
    
    /// 收集系统指标
    pub fn collect_system_metrics(&mut self) {
        self.system_metrics.update();
    }
    
    /// 获取写入成功率
    pub fn get_success_rate(&self) -> f64 {
        let total = self.logs_written_total + self.logs_dropped_total;
        if total == 0 {
            1.0
        } else {
            self.logs_written_total as f64 / total as f64
        }
    }
    
    /// 获取平均写入延迟
    pub fn get_average_latency_ms(&self) -> f64 {
        self.write_latency_ms.mean()
    }
    
    /// 获取95百分位延迟
    pub fn get_p95_latency_ms(&self) -> f64 {
        self.write_latency_ms.percentile(0.95)
    }
    
    /// 获取99百分位延迟
    pub fn get_p99_latency_ms(&self) -> f64 {
        self.write_latency_ms.percentile(0.99)
    }
    
    /// 重置计数器
    pub fn reset_counters(&mut self) {
        self.logs_written_total = 0;
        self.logs_dropped_total = 0;
        self.error_count = 0;
        self.level_counters.clear();
        self.module_counters.clear();
        self.write_latency_ms.reset();
    }
    
    /// 生成指标快照
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            timestamp: chrono::Utc::now(),
            logs_written_total: self.logs_written_total,
            logs_dropped_total: self.logs_dropped_total,
            success_rate: self.get_success_rate(),
            average_latency_ms: self.get_average_latency_ms(),
            p95_latency_ms: self.get_p95_latency_ms(),
            p99_latency_ms: self.get_p99_latency_ms(),
            queue_size: self.queue_size,
            disk_usage_bytes: self.disk_usage_bytes,
            error_count: self.error_count,
            level_distribution: self.level_counters.clone(),
            top_modules: self.get_top_modules(10),
            system_metrics: self.system_metrics.clone(),
        }
    }
    
    /// 获取活跃度最高的模块
    fn get_top_modules(&self, limit: usize) -> Vec<(String, u64)> {
        let mut modules: Vec<_> = self.module_counters.iter().collect();
        modules.sort_by(|a, b| b.1.cmp(a.1));
        modules.into_iter()
            .take(limit)
            .map(|(k, &v)| (k.clone(), v))
            .collect()
    }
}

impl Default for LogMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// LogMetrics doesn't implement Clone because Histogram contains atomics
// Use snapshot() method instead to get a point-in-time copy

/// 指标快照 - 某个时间点的指标状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub logs_written_total: u64,
    pub logs_dropped_total: u64,
    pub success_rate: f64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub queue_size: usize,
    pub disk_usage_bytes: u64,
    pub error_count: u64,
    pub level_distribution: HashMap<LogLevel, u64>,
    pub top_modules: Vec<(String, u64)>,
    pub system_metrics: SystemMetrics,
}

/// 系统资源指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub thread_count: usize,
    pub disk_io_read_bytes: u64,
    pub disk_io_write_bytes: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub uptime_seconds: u64,
}

impl SystemMetrics {
    pub fn new() -> Self {
        Self {
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            thread_count: 0,
            disk_io_read_bytes: 0,
            disk_io_write_bytes: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
            uptime_seconds: 0,
        }
    }
    
    /// 更新系统指标（简化实现）
    pub fn update(&mut self) {
        // 这里是简化的实现，实际应该调用系统API获取真实数据
        self.memory_usage_mb = self.get_memory_usage();
        self.cpu_usage_percent = self.get_cpu_usage();
        self.thread_count = self.get_thread_count();
        self.uptime_seconds = self.get_uptime();
    }
    
    fn get_memory_usage(&self) -> f64 {
        // 简化实现：估算内存使用
        // 实际实现应该读取 /proc/meminfo 或使用系统调用
        100.0 // 假设使用100MB内存
    }
    
    fn get_cpu_usage(&self) -> f64 {
        // 简化实现：随机CPU使用率
        // 实际实现应该读取 /proc/stat 或使用系统调用
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..20.0) // 假设0-20%的CPU使用率
    }
    
    fn get_thread_count(&self) -> usize {
        // 获取当前线程数
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
    
    fn get_uptime(&self) -> u64 {
        // 简化实现：从程序启动时间计算
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// 直方图 - 用于统计延迟分布
#[derive(Debug)]
pub struct Histogram {
    buckets: Vec<AtomicU64>,
    bucket_bounds: Vec<f64>,
    count: AtomicU64,
    sum: std::sync::Mutex<f64>,
}

impl Histogram {
    /// 创建新的直方图
    pub fn new() -> Self {
        // 延迟桶边界（毫秒）
        let bucket_bounds = vec![
            0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0
        ];
        let bucket_count = bucket_bounds.len() + 1; // +1 for overflow bucket
        
        let buckets = (0..bucket_count)
            .map(|_| AtomicU64::new(0))
            .collect();
        
        Self {
            buckets,
            bucket_bounds,
            count: AtomicU64::new(0),
            sum: std::sync::Mutex::new(0.0),
        }
    }
    
    /// 记录一个值
    pub fn record(&self, value: f64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        
        // 更新总和
        {
            let mut sum = self.sum.lock().unwrap();
            *sum += value;
        }
        
        // 找到对应的桶并增加计数
        let bucket_index = self.find_bucket_index(value);
        self.buckets[bucket_index].fetch_add(1, Ordering::Relaxed);
    }
    
    fn find_bucket_index(&self, value: f64) -> usize {
        for (i, &bound) in self.bucket_bounds.iter().enumerate() {
            if value <= bound {
                return i;
            }
        }
        // 超出最大边界，使用溢出桶
        self.bucket_bounds.len()
    }
    
    /// 获取样本数量
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
    
    /// 获取平均值
    pub fn mean(&self) -> f64 {
        let count = self.count();
        if count == 0 {
            return 0.0;
        }
        
        let sum = *self.sum.lock().unwrap();
        sum / count as f64
    }
    
    /// 获取百分位数
    pub fn percentile(&self, p: f64) -> f64 {
        let count = self.count();
        if count == 0 {
            return 0.0;
        }
        
        let target_count = (count as f64 * p) as u64;
        let mut cumulative_count = 0u64;
        
        for (i, bucket) in self.buckets.iter().enumerate() {
            cumulative_count += bucket.load(Ordering::Relaxed);
            if cumulative_count >= target_count {
                return if i < self.bucket_bounds.len() {
                    self.bucket_bounds[i]
                } else {
                    self.bucket_bounds.last().copied().unwrap_or(0.0) * 2.0
                };
            }
        }
        
        0.0
    }
    
    /// 重置直方图
    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
        *self.sum.lock().unwrap() = 0.0;
        
        for bucket in &self.buckets {
            bucket.store(0, Ordering::Relaxed);
        }
    }
    
    /// 获取桶数据
    pub fn get_buckets(&self) -> Vec<(f64, u64)> {
        let mut result = Vec::new();
        
        for (i, bucket) in self.buckets.iter().enumerate() {
            let bound = if i < self.bucket_bounds.len() {
                self.bucket_bounds[i]
            } else {
                f64::INFINITY
            };
            let count = bucket.load(Ordering::Relaxed);
            result.push((bound, count));
        }
        
        result
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能监控器
pub struct PerformanceMonitor {
    start_time: Instant,
    operation_name: String,
    metrics: Option<Arc<tokio::sync::Mutex<LogMetrics>>>,
}

impl PerformanceMonitor {
    /// 开始监控操作
    pub fn start(operation_name: &str) -> Self {
        Self {
            start_time: Instant::now(),
            operation_name: operation_name.to_string(),
            metrics: None,
        }
    }
    
    /// 开始监控操作（带指标收集）
    pub fn start_with_metrics(
        operation_name: &str,
        metrics: Arc<tokio::sync::Mutex<LogMetrics>>,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            operation_name: operation_name.to_string(),
            metrics: Some(metrics),
        }
    }
    
    /// 结束监控并记录耗时
    pub async fn finish(self) -> std::time::Duration {
        let duration = self.start_time.elapsed();
        
        if let Some(metrics) = &self.metrics {
            let mut m = metrics.lock().await;
            m.record_log_written(
                LogLevel::Info, 
                "performance_monitor", 
                duration.as_secs_f64() * 1000.0
            );
        }
        
        tracing::debug!(
            operation = self.operation_name,
            duration_ms = duration.as_secs_f64() * 1000.0,
            "操作完成"
        );
        
        duration
    }
    
    /// 记录中间检查点
    pub fn checkpoint(&self, step_name: &str) -> std::time::Duration {
        let elapsed = self.start_time.elapsed();
        
        tracing::debug!(
            operation = self.operation_name,
            step = step_name,
            elapsed_ms = elapsed.as_secs_f64() * 1000.0,
            "检查点"
        );
        
        elapsed
    }
    
    /// 获取已耗时
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// 指标导出器
pub struct MetricsExporter {
    format: ExportFormat,
}

/// 导出格式
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Prometheus,
    Csv,
}

impl MetricsExporter {
    /// 创建新的指标导出器
    pub fn new(format: ExportFormat) -> Self {
        Self { format }
    }
    
    /// 导出指标
    pub fn export(&self, snapshot: &MetricsSnapshot) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self.format {
            ExportFormat::Json => self.export_json(snapshot),
            ExportFormat::Prometheus => self.export_prometheus(snapshot),
            ExportFormat::Csv => self.export_csv(snapshot),
        }
    }
    
    fn export_json(&self, snapshot: &MetricsSnapshot) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::to_string_pretty(snapshot)?)
    }
    
    fn export_prometheus(&self, snapshot: &MetricsSnapshot) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut output = String::new();
        
        // 基础指标
        output.push_str(&format!(
            "# HELP logging_logs_written_total Total number of logs written\n"
        ));
        output.push_str(&format!(
            "# TYPE logging_logs_written_total counter\n"
        ));
        output.push_str(&format!(
            "logging_logs_written_total {}\n",
            snapshot.logs_written_total
        ));
        
        output.push_str(&format!(
            "# HELP logging_logs_dropped_total Total number of logs dropped\n"
        ));
        output.push_str(&format!(
            "# TYPE logging_logs_dropped_total counter\n"
        ));
        output.push_str(&format!(
            "logging_logs_dropped_total {}\n",
            snapshot.logs_dropped_total
        ));
        
        output.push_str(&format!(
            "# HELP logging_write_latency_seconds Write latency in seconds\n"
        ));
        output.push_str(&format!(
            "# TYPE logging_write_latency_seconds histogram\n"
        ));
        output.push_str(&format!(
            "logging_write_latency_seconds_sum {}\n",
            snapshot.average_latency_ms * snapshot.logs_written_total as f64 / 1000.0
        ));
        output.push_str(&format!(
            "logging_write_latency_seconds_count {}\n",
            snapshot.logs_written_total
        ));
        
        output.push_str(&format!(
            "# HELP logging_queue_size Current queue size\n"
        ));
        output.push_str(&format!(
            "# TYPE logging_queue_size gauge\n"
        ));
        output.push_str(&format!(
            "logging_queue_size {}\n",
            snapshot.queue_size
        ));
        
        // 按级别分组的指标
        for (level, count) in &snapshot.level_distribution {
            output.push_str(&format!(
                "logging_logs_by_level{{level=\"{}\"}} {}\n",
                level.as_str().to_lowercase(),
                count
            ));
        }
        
        Ok(output)
    }
    
    fn export_csv(&self, snapshot: &MetricsSnapshot) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut output = String::new();
        
        // CSV 标题行
        output.push_str("timestamp,logs_written_total,logs_dropped_total,success_rate,average_latency_ms,p95_latency_ms,p99_latency_ms,queue_size,disk_usage_bytes,error_count\n");
        
        // 数据行
        output.push_str(&format!(
            "{},{},{},{:.4},{:.2},{:.2},{:.2},{},{},{}\n",
            snapshot.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            snapshot.logs_written_total,
            snapshot.logs_dropped_total,
            snapshot.success_rate,
            snapshot.average_latency_ms,
            snapshot.p95_latency_ms,
            snapshot.p99_latency_ms,
            snapshot.queue_size,
            snapshot.disk_usage_bytes,
            snapshot.error_count
        ));
        
        Ok(output)
    }
}

/// 指标收集任务
pub struct MetricsCollector {
    metrics: Arc<tokio::sync::Mutex<LogMetrics>>,
    collection_interval: std::time::Duration,
    export_interval: std::time::Duration,
    exporter: Option<MetricsExporter>,
    export_path: Option<std::path::PathBuf>,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new(
        metrics: Arc<tokio::sync::Mutex<LogMetrics>>,
        collection_interval: std::time::Duration,
    ) -> Self {
        Self {
            metrics,
            collection_interval,
            export_interval: std::time::Duration::from_secs(60),
            exporter: None,
            export_path: None,
        }
    }
    
    /// 设置导出器
    pub fn with_exporter(
        mut self,
        exporter: MetricsExporter,
        export_path: std::path::PathBuf,
        export_interval: std::time::Duration,
    ) -> Self {
        self.exporter = Some(exporter);
        self.export_path = Some(export_path);
        self.export_interval = export_interval;
        self
    }
    
    /// 启动收集任务
    pub async fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut collection_interval = tokio::time::interval(self.collection_interval);
            let mut export_interval = tokio::time::interval(self.export_interval);
            
            loop {
                tokio::select! {
                    _ = collection_interval.tick() => {
                        // 收集系统指标
                        let mut metrics = self.metrics.lock().await;
                        metrics.collect_system_metrics();
                    }
                    
                    _ = export_interval.tick() => {
                        // 导出指标
                        if let (Some(exporter), Some(export_path)) = (&self.exporter, &self.export_path) {
                            let snapshot = {
                                let metrics = self.metrics.lock().await;
                                metrics.snapshot()
                            };
                            
                            if let Ok(exported) = exporter.export(&snapshot) {
                                if let Err(e) = tokio::fs::write(&export_path, exported).await {
                                    tracing::error!(
                                        export_path = %export_path.display(),
                                        error = %e,
                                        "导出指标失败"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_metrics() {
        let mut metrics = LogMetrics::new();
        
        // 记录一些日志
        metrics.record_log_written(LogLevel::Info, "test_module", 10.5);
        metrics.record_log_written(LogLevel::Error, "test_module", 25.2);
        metrics.record_log_dropped();
        
        // 检查统计
        assert_eq!(metrics.logs_written_total, 2);
        assert_eq!(metrics.logs_dropped_total, 1);
        assert_eq!(metrics.get_success_rate(), 2.0 / 3.0);
        assert!(metrics.get_average_latency_ms() > 0.0);
        
        // 检查级别分布
        assert_eq!(metrics.level_counters.get(&LogLevel::Info), Some(&1));
        assert_eq!(metrics.level_counters.get(&LogLevel::Error), Some(&1));
        
        // 检查模块统计
        assert_eq!(metrics.module_counters.get("test_module"), Some(&2));
    }
    
    #[test]
    fn test_histogram() {
        let histogram = Histogram::new();
        
        // 记录一些值
        histogram.record(1.0);
        histogram.record(5.0);
        histogram.record(10.0);
        histogram.record(50.0);
        histogram.record(100.0);
        
        assert_eq!(histogram.count(), 5);
        assert!(histogram.mean() > 0.0);
        assert!(histogram.percentile(0.5) > 0.0);
        assert!(histogram.percentile(0.95) >= histogram.percentile(0.5));
    }
    
    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::start("test_operation");
        
        // 模拟一些工作
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        let checkpoint_duration = monitor.checkpoint("middle");
        assert!(checkpoint_duration.as_millis() >= 10);
        
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        let total_duration = monitor.finish().await;
        assert!(total_duration.as_millis() >= 20);
    }
    
    #[test]
    fn test_metrics_snapshot() {
        let mut metrics = LogMetrics::new();
        metrics.record_log_written(LogLevel::Info, "test", 15.0);
        metrics.update_queue_size(42);
        metrics.update_disk_usage(1024 * 1024);
        
        let snapshot = metrics.snapshot();
        
        assert_eq!(snapshot.logs_written_total, 1);
        assert_eq!(snapshot.queue_size, 42);
        assert_eq!(snapshot.disk_usage_bytes, 1024 * 1024);
        assert!(!snapshot.level_distribution.is_empty());
        assert!(!snapshot.top_modules.is_empty());
    }
    
    #[test]
    fn test_metrics_export() {
        let mut metrics = LogMetrics::new();
        metrics.record_log_written(LogLevel::Info, "test", 10.0);
        let snapshot = metrics.snapshot();
        
        // 测试 JSON 导出
        let json_exporter = MetricsExporter::new(ExportFormat::Json);
        let json_result = json_exporter.export(&snapshot);
        assert!(json_result.is_ok());
        assert!(json_result.unwrap().contains("logs_written_total"));
        
        // 测试 Prometheus 导出
        let prometheus_exporter = MetricsExporter::new(ExportFormat::Prometheus);
        let prometheus_result = prometheus_exporter.export(&snapshot);
        assert!(prometheus_result.is_ok());
        assert!(prometheus_result.unwrap().contains("logging_logs_written_total"));
        
        // 测试 CSV 导出
        let csv_exporter = MetricsExporter::new(ExportFormat::Csv);
        let csv_result = csv_exporter.export(&snapshot);
        assert!(csv_result.is_ok());
        assert!(csv_result.unwrap().contains("timestamp,logs_written_total"));
    }
    
    #[test]
    fn test_system_metrics() {
        let mut system_metrics = SystemMetrics::new();
        system_metrics.update();
        
        assert!(system_metrics.memory_usage_mb >= 0.0);
        assert!(system_metrics.cpu_usage_percent >= 0.0);
        assert!(system_metrics.thread_count > 0);
        assert!(system_metrics.uptime_seconds > 0);
    }
}
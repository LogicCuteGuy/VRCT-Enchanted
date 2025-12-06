// Performance profiling utilities for VRCT
// Provides memory, CPU, and latency profiling capabilities

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Memory usage statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Current heap allocation in bytes (estimated)
    pub heap_allocated: u64,
    /// Peak heap allocation in bytes
    pub peak_heap_allocated: u64,
    /// Number of active allocations tracked
    pub active_allocations: u64,
}

/// CPU usage statistics for a specific operation
#[derive(Debug, Clone)]
pub struct CpuStats {
    /// Total time spent in this operation
    pub total_time: Duration,
    /// Number of times this operation was called
    pub call_count: u64,
    /// Average time per call
    pub avg_time: Duration,
    /// Minimum time observed
    pub min_time: Duration,
    /// Maximum time observed
    pub max_time: Duration,
}

impl Default for CpuStats {
    fn default() -> Self {
        Self {
            total_time: Duration::ZERO,
            call_count: 0,
            avg_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
        }
    }
}

/// Latency measurement for end-to-end operations
#[derive(Debug, Clone)]
pub struct LatencyStats {
    /// Operation name
    pub operation: String,
    /// Total samples collected
    pub sample_count: u64,
    /// Average latency
    pub avg_latency: Duration,
    /// P50 latency (median)
    pub p50_latency: Duration,
    /// P95 latency
    pub p95_latency: Duration,
    /// P99 latency
    pub p99_latency: Duration,
    /// Minimum latency
    pub min_latency: Duration,
    /// Maximum latency
    pub max_latency: Duration,
}

impl Default for LatencyStats {
    fn default() -> Self {
        Self {
            operation: String::new(),
            sample_count: 0,
            avg_latency: Duration::ZERO,
            p50_latency: Duration::ZERO,
            p95_latency: Duration::ZERO,
            p99_latency: Duration::ZERO,
            min_latency: Duration::MAX,
            max_latency: Duration::ZERO,
        }
    }
}

/// Latency sample collector for percentile calculations
#[derive(Debug)]
struct LatencySamples {
    samples: Vec<Duration>,
    max_samples: usize,
}

impl LatencySamples {
    fn new(max_samples: usize) -> Self {
        Self {
            samples: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    fn add(&mut self, duration: Duration) {
        if self.samples.len() >= self.max_samples {
            // Remove oldest sample (simple FIFO)
            self.samples.remove(0);
        }
        self.samples.push(duration);
    }

    fn calculate_stats(&self, operation: &str) -> LatencyStats {
        if self.samples.is_empty() {
            return LatencyStats {
                operation: operation.to_string(),
                ..Default::default()
            };
        }

        let mut sorted = self.samples.clone();
        sorted.sort();

        let count = sorted.len();
        let total: Duration = sorted.iter().sum();
        let avg = total / count as u32;

        let p50_idx = (count as f64 * 0.50) as usize;
        let p95_idx = (count as f64 * 0.95) as usize;
        let p99_idx = (count as f64 * 0.99) as usize;

        LatencyStats {
            operation: operation.to_string(),
            sample_count: count as u64,
            avg_latency: avg,
            p50_latency: sorted.get(p50_idx.min(count - 1)).copied().unwrap_or_default(),
            p95_latency: sorted.get(p95_idx.min(count - 1)).copied().unwrap_or_default(),
            p99_latency: sorted.get(p99_idx.min(count - 1)).copied().unwrap_or_default(),
            min_latency: sorted.first().copied().unwrap_or_default(),
            max_latency: sorted.last().copied().unwrap_or_default(),
        }
    }
}

/// Performance profiler for tracking memory, CPU, and latency
pub struct Profiler {
    /// Whether profiling is enabled
    enabled: Arc<AtomicU64>,
    /// CPU timing stats by operation name
    cpu_stats: Arc<RwLock<HashMap<String, CpuStats>>>,
    /// Latency samples by operation name
    latency_samples: Arc<RwLock<HashMap<String, LatencySamples>>>,
    /// Memory tracking (estimated)
    memory_allocated: Arc<AtomicU64>,
    /// Peak memory usage
    peak_memory: Arc<AtomicU64>,
    /// Allocation count
    allocation_count: Arc<AtomicU64>,
    /// Maximum latency samples to keep per operation
    max_latency_samples: usize,
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Profiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicU64::new(1)), // Enabled by default
            cpu_stats: Arc::new(RwLock::new(HashMap::new())),
            latency_samples: Arc::new(RwLock::new(HashMap::new())),
            memory_allocated: Arc::new(AtomicU64::new(0)),
            peak_memory: Arc::new(AtomicU64::new(0)),
            allocation_count: Arc::new(AtomicU64::new(0)),
            max_latency_samples: 1000,
        }
    }

    /// Enable or disable profiling
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(if enabled { 1 } else { 0 }, Ordering::Relaxed);
        info!("Profiling {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) != 0
    }

    /// Start timing an operation
    pub fn start_timing(&self, operation: &str) -> Option<TimingGuard> {
        if !self.is_enabled() {
            return None;
        }
        Some(TimingGuard {
            operation: operation.to_string(),
            start: Instant::now(),
            profiler: self.clone_refs(),
        })
    }

    /// Record a CPU timing measurement
    pub fn record_cpu_timing(&self, operation: &str, duration: Duration) {
        if !self.is_enabled() {
            return;
        }

        if let Ok(mut stats) = self.cpu_stats.write() {
            let entry = stats.entry(operation.to_string()).or_default();
            entry.total_time += duration;
            entry.call_count += 1;
            entry.avg_time = entry.total_time / entry.call_count as u32;
            entry.min_time = entry.min_time.min(duration);
            entry.max_time = entry.max_time.max(duration);
        }
    }

    /// Record a latency measurement
    pub fn record_latency(&self, operation: &str, duration: Duration) {
        if !self.is_enabled() {
            return;
        }

        if let Ok(mut samples) = self.latency_samples.write() {
            let entry = samples
                .entry(operation.to_string())
                .or_insert_with(|| LatencySamples::new(self.max_latency_samples));
            entry.add(duration);
        }
    }

    /// Track a memory allocation (estimated)
    pub fn track_allocation(&self, size: u64) {
        if !self.is_enabled() {
            return;
        }

        let current = self.memory_allocated.fetch_add(size, Ordering::Relaxed) + size;
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        // Update peak if necessary
        let mut peak = self.peak_memory.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_memory.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }
    }

    /// Track a memory deallocation (estimated)
    pub fn track_deallocation(&self, size: u64) {
        if !self.is_enabled() {
            return;
        }

        self.memory_allocated.fetch_sub(size.min(self.memory_allocated.load(Ordering::Relaxed)), Ordering::Relaxed);
    }

    /// Get memory statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            heap_allocated: self.memory_allocated.load(Ordering::Relaxed),
            peak_heap_allocated: self.peak_memory.load(Ordering::Relaxed),
            active_allocations: self.allocation_count.load(Ordering::Relaxed),
        }
    }

    /// Get CPU statistics for all operations
    pub fn get_cpu_stats(&self) -> HashMap<String, CpuStats> {
        self.cpu_stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// Get CPU statistics for a specific operation
    pub fn get_cpu_stats_for(&self, operation: &str) -> Option<CpuStats> {
        self.cpu_stats.read().ok()?.get(operation).cloned()
    }

    /// Get latency statistics for all operations
    pub fn get_latency_stats(&self) -> HashMap<String, LatencyStats> {
        if let Ok(samples) = self.latency_samples.read() {
            samples
                .iter()
                .map(|(k, v)| (k.clone(), v.calculate_stats(k)))
                .collect()
        } else {
            HashMap::new()
        }
    }

    /// Get latency statistics for a specific operation
    pub fn get_latency_stats_for(&self, operation: &str) -> Option<LatencyStats> {
        self.latency_samples
            .read()
            .ok()?
            .get(operation)
            .map(|s| s.calculate_stats(operation))
    }

    /// Reset all statistics
    pub fn reset(&self) {
        if let Ok(mut stats) = self.cpu_stats.write() {
            stats.clear();
        }
        if let Ok(mut samples) = self.latency_samples.write() {
            samples.clear();
        }
        self.memory_allocated.store(0, Ordering::Relaxed);
        self.peak_memory.store(0, Ordering::Relaxed);
        self.allocation_count.store(0, Ordering::Relaxed);
        info!("Profiler statistics reset");
    }

    /// Generate a profiling report
    pub fn generate_report(&self) -> ProfilingReport {
        ProfilingReport {
            memory: self.get_memory_stats(),
            cpu: self.get_cpu_stats(),
            latency: self.get_latency_stats(),
        }
    }

    /// Clone references for use in guards
    fn clone_refs(&self) -> ProfilerRefs {
        ProfilerRefs {
            cpu_stats: Arc::clone(&self.cpu_stats),
            latency_samples: Arc::clone(&self.latency_samples),
            enabled: Arc::clone(&self.enabled),
            max_latency_samples: self.max_latency_samples,
        }
    }
}

/// Internal struct for sharing profiler references
struct ProfilerRefs {
    cpu_stats: Arc<RwLock<HashMap<String, CpuStats>>>,
    latency_samples: Arc<RwLock<HashMap<String, LatencySamples>>>,
    enabled: Arc<AtomicU64>,
    max_latency_samples: usize,
}

impl ProfilerRefs {
    fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) != 0
    }

    fn record_cpu_timing(&self, operation: &str, duration: Duration) {
        if !self.is_enabled() {
            return;
        }

        if let Ok(mut stats) = self.cpu_stats.write() {
            let entry = stats.entry(operation.to_string()).or_default();
            entry.total_time += duration;
            entry.call_count += 1;
            entry.avg_time = entry.total_time / entry.call_count as u32;
            entry.min_time = entry.min_time.min(duration);
            entry.max_time = entry.max_time.max(duration);
        }
    }

    fn record_latency(&self, operation: &str, duration: Duration) {
        if !self.is_enabled() {
            return;
        }

        if let Ok(mut samples) = self.latency_samples.write() {
            let entry = samples
                .entry(operation.to_string())
                .or_insert_with(|| LatencySamples::new(self.max_latency_samples));
            entry.add(duration);
        }
    }
}

/// RAII guard for timing operations
pub struct TimingGuard {
    operation: String,
    start: Instant,
    profiler: ProfilerRefs,
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.profiler.record_cpu_timing(&self.operation, duration);
        self.profiler.record_latency(&self.operation, duration);
        debug!("{} took {:?}", self.operation, duration);
    }
}

/// Complete profiling report
#[derive(Debug, Clone)]
pub struct ProfilingReport {
    pub memory: MemoryStats,
    pub cpu: HashMap<String, CpuStats>,
    pub latency: HashMap<String, LatencyStats>,
}

impl ProfilingReport {
    /// Format the report as a human-readable string
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str("=== VRCT Performance Profiling Report ===\n\n");

        // Memory section
        output.push_str("--- Memory Usage ---\n");
        output.push_str(&format!(
            "Current Heap: {} KB\n",
            self.memory.heap_allocated / 1024
        ));
        output.push_str(&format!(
            "Peak Heap: {} KB\n",
            self.memory.peak_heap_allocated / 1024
        ));
        output.push_str(&format!(
            "Active Allocations: {}\n\n",
            self.memory.active_allocations
        ));

        // CPU section
        output.push_str("--- CPU Usage by Operation ---\n");
        let mut cpu_entries: Vec<_> = self.cpu.iter().collect();
        cpu_entries.sort_by(|a, b| b.1.total_time.cmp(&a.1.total_time));

        for (op, stats) in cpu_entries {
            output.push_str(&format!(
                "{}: total={:?}, calls={}, avg={:?}, min={:?}, max={:?}\n",
                op,
                stats.total_time,
                stats.call_count,
                stats.avg_time,
                if stats.min_time == Duration::MAX {
                    Duration::ZERO
                } else {
                    stats.min_time
                },
                stats.max_time
            ));
        }
        output.push('\n');

        // Latency section
        output.push_str("--- Latency Statistics ---\n");
        let mut latency_entries: Vec<_> = self.latency.iter().collect();
        latency_entries.sort_by(|a, b| b.1.avg_latency.cmp(&a.1.avg_latency));

        for (op, stats) in latency_entries {
            output.push_str(&format!(
                "{}: samples={}, avg={:?}, p50={:?}, p95={:?}, p99={:?}, min={:?}, max={:?}\n",
                op,
                stats.sample_count,
                stats.avg_latency,
                stats.p50_latency,
                stats.p95_latency,
                stats.p99_latency,
                if stats.min_latency == Duration::MAX {
                    Duration::ZERO
                } else {
                    stats.min_latency
                },
                stats.max_latency
            ));
        }

        output
    }

    /// Log the report using tracing
    pub fn log(&self) {
        info!("Performance Report:\n{}", self.format());
    }
}

/// Global profiler instance
static GLOBAL_PROFILER: std::sync::OnceLock<Profiler> = std::sync::OnceLock::new();

/// Get the global profiler instance
pub fn global_profiler() -> &'static Profiler {
    GLOBAL_PROFILER.get_or_init(Profiler::new)
}

/// Convenience macro for timing a block of code
#[macro_export]
macro_rules! profile_block {
    ($name:expr, $block:expr) => {{
        let _guard = $crate::utils::profiling::global_profiler().start_timing($name);
        $block
    }};
}

/// Convenience macro for tracking memory allocation
#[macro_export]
macro_rules! track_alloc {
    ($size:expr) => {
        $crate::utils::profiling::global_profiler().track_allocation($size as u64)
    };
}

/// Convenience macro for tracking memory deallocation
#[macro_export]
macro_rules! track_dealloc {
    ($size:expr) => {
        $crate::utils::profiling::global_profiler().track_deallocation($size as u64)
    };
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profiler_creation() {
        let profiler = Profiler::new();
        assert!(profiler.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let profiler = Profiler::new();
        
        assert!(profiler.is_enabled());
        
        profiler.set_enabled(false);
        assert!(!profiler.is_enabled());
        
        profiler.set_enabled(true);
        assert!(profiler.is_enabled());
    }

    #[test]
    fn test_cpu_timing() {
        let profiler = Profiler::new();
        
        // Record some timings
        profiler.record_cpu_timing("test_op", Duration::from_millis(10));
        profiler.record_cpu_timing("test_op", Duration::from_millis(20));
        profiler.record_cpu_timing("test_op", Duration::from_millis(30));
        
        let stats = profiler.get_cpu_stats_for("test_op").unwrap();
        
        assert_eq!(stats.call_count, 3);
        assert_eq!(stats.total_time, Duration::from_millis(60));
        assert_eq!(stats.avg_time, Duration::from_millis(20));
        assert_eq!(stats.min_time, Duration::from_millis(10));
        assert_eq!(stats.max_time, Duration::from_millis(30));
    }

    #[test]
    fn test_timing_guard() {
        let profiler = Profiler::new();
        
        {
            let _guard = profiler.start_timing("guarded_op");
            thread::sleep(Duration::from_millis(10));
        }
        
        let stats = profiler.get_cpu_stats_for("guarded_op").unwrap();
        assert_eq!(stats.call_count, 1);
        assert!(stats.total_time >= Duration::from_millis(10));
    }

    #[test]
    fn test_latency_stats() {
        let profiler = Profiler::new();
        
        // Record latency samples
        for i in 1..=100 {
            profiler.record_latency("latency_test", Duration::from_millis(i));
        }
        
        let stats = profiler.get_latency_stats_for("latency_test").unwrap();
        
        assert_eq!(stats.sample_count, 100);
        assert_eq!(stats.min_latency, Duration::from_millis(1));
        assert_eq!(stats.max_latency, Duration::from_millis(100));
        // P50 should be around 50ms
        assert!(stats.p50_latency >= Duration::from_millis(45));
        assert!(stats.p50_latency <= Duration::from_millis(55));
    }

    #[test]
    fn test_memory_tracking() {
        let profiler = Profiler::new();
        
        profiler.track_allocation(1000);
        profiler.track_allocation(2000);
        
        let stats = profiler.get_memory_stats();
        assert_eq!(stats.heap_allocated, 3000);
        assert_eq!(stats.peak_heap_allocated, 3000);
        
        profiler.track_deallocation(1000);
        
        let stats = profiler.get_memory_stats();
        assert_eq!(stats.heap_allocated, 2000);
        assert_eq!(stats.peak_heap_allocated, 3000); // Peak unchanged
    }

    #[test]
    fn test_reset() {
        let profiler = Profiler::new();
        
        profiler.record_cpu_timing("op1", Duration::from_millis(10));
        profiler.record_latency("op1", Duration::from_millis(10));
        profiler.track_allocation(1000);
        
        profiler.reset();
        
        assert!(profiler.get_cpu_stats().is_empty());
        assert!(profiler.get_latency_stats().is_empty());
        assert_eq!(profiler.get_memory_stats().heap_allocated, 0);
    }

    #[test]
    fn test_report_generation() {
        let profiler = Profiler::new();
        
        profiler.record_cpu_timing("audio_processing", Duration::from_millis(5));
        profiler.record_cpu_timing("transcription", Duration::from_millis(100));
        profiler.record_latency("transcription", Duration::from_millis(100));
        profiler.track_allocation(1024 * 1024); // 1 MB
        
        let report = profiler.generate_report();
        let formatted = report.format();
        
        assert!(formatted.contains("audio_processing"));
        assert!(formatted.contains("transcription"));
        assert!(formatted.contains("Memory Usage"));
        assert!(formatted.contains("CPU Usage"));
        assert!(formatted.contains("Latency Statistics"));
    }

    #[test]
    fn test_disabled_profiler() {
        let profiler = Profiler::new();
        profiler.set_enabled(false);
        
        // These should be no-ops when disabled
        profiler.record_cpu_timing("disabled_op", Duration::from_millis(10));
        profiler.record_latency("disabled_op", Duration::from_millis(10));
        profiler.track_allocation(1000);
        
        assert!(profiler.get_cpu_stats().is_empty());
        assert!(profiler.get_latency_stats().is_empty());
        assert_eq!(profiler.get_memory_stats().heap_allocated, 0);
    }

    #[test]
    fn test_global_profiler() {
        let profiler = global_profiler();
        assert!(profiler.is_enabled());
        
        profiler.record_cpu_timing("global_test", Duration::from_millis(5));
        let stats = profiler.get_cpu_stats_for("global_test");
        assert!(stats.is_some());
    }

    #[test]
    fn test_multiple_operations() {
        let profiler = Profiler::new();
        
        profiler.record_cpu_timing("op_a", Duration::from_millis(10));
        profiler.record_cpu_timing("op_b", Duration::from_millis(20));
        profiler.record_cpu_timing("op_c", Duration::from_millis(30));
        
        let all_stats = profiler.get_cpu_stats();
        assert_eq!(all_stats.len(), 3);
        assert!(all_stats.contains_key("op_a"));
        assert!(all_stats.contains_key("op_b"));
        assert!(all_stats.contains_key("op_c"));
    }

    #[test]
    fn test_latency_percentiles() {
        let profiler = Profiler::new();
        
        // Add samples with known distribution
        // 1-90: 90 samples of 10ms
        // 91-95: 5 samples of 100ms
        // 96-99: 4 samples of 500ms
        // 100: 1 sample of 1000ms
        
        for _ in 0..90 {
            profiler.record_latency("percentile_test", Duration::from_millis(10));
        }
        for _ in 0..5 {
            profiler.record_latency("percentile_test", Duration::from_millis(100));
        }
        for _ in 0..4 {
            profiler.record_latency("percentile_test", Duration::from_millis(500));
        }
        profiler.record_latency("percentile_test", Duration::from_millis(1000));
        
        let stats = profiler.get_latency_stats_for("percentile_test").unwrap();
        
        assert_eq!(stats.sample_count, 100);
        // P50 should be 10ms (median is in the first 90 samples)
        assert_eq!(stats.p50_latency, Duration::from_millis(10));
        // P95 should be around 100ms
        assert!(stats.p95_latency >= Duration::from_millis(10));
        assert!(stats.p95_latency <= Duration::from_millis(500));
        // P99 should be around 500ms or higher
        assert!(stats.p99_latency >= Duration::from_millis(100));
    }
}

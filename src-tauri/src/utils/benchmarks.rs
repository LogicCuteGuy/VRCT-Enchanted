// Benchmarks for performance profiling
// Provides utilities to measure and compare performance of key operations

use std::time::{Duration, Instant};
use tracing::info;

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations to run
    pub iterations: usize,
    /// Warmup iterations (not counted in results)
    pub warmup_iterations: usize,
    /// Whether to log progress
    pub verbose: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            warmup_iterations: 10,
            verbose: false,
        }
    }
}

/// Result of a benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub throughput: f64, // operations per second
}

impl BenchmarkResult {
    /// Format the result as a human-readable string
    pub fn format(&self) -> String {
        format!(
            "{}: {} iterations, total={:?}, avg={:?}, min={:?}, max={:?}, throughput={:.2} ops/sec",
            self.name,
            self.iterations,
            self.total_time,
            self.avg_time,
            self.min_time,
            self.max_time,
            self.throughput
        )
    }
}

/// Run a benchmark on a closure
pub fn run_benchmark<F>(name: &str, config: &BenchmarkConfig, mut f: F) -> BenchmarkResult
where
    F: FnMut(),
{
    // Warmup
    if config.verbose {
        info!("Running {} warmup iterations for {}", config.warmup_iterations, name);
    }
    for _ in 0..config.warmup_iterations {
        f();
    }

    // Actual benchmark
    if config.verbose {
        info!("Running {} benchmark iterations for {}", config.iterations, name);
    }

    let mut times = Vec::with_capacity(config.iterations);
    let start = Instant::now();

    for _ in 0..config.iterations {
        let iter_start = Instant::now();
        f();
        times.push(iter_start.elapsed());
    }

    let total_time = start.elapsed();
    let min_time = times.iter().min().copied().unwrap_or_default();
    let max_time = times.iter().max().copied().unwrap_or_default();
    let avg_time = total_time / config.iterations as u32;
    let throughput = config.iterations as f64 / total_time.as_secs_f64();

    BenchmarkResult {
        name: name.to_string(),
        iterations: config.iterations,
        total_time,
        avg_time,
        min_time,
        max_time,
        throughput,
    }
}

/// Benchmark audio energy calculation
pub fn benchmark_energy_calculation(sample_sizes: &[usize], config: &BenchmarkConfig) -> Vec<BenchmarkResult> {
    use crate::audio::energy::EnergyDetector;

    let mut results = Vec::new();

    for &size in sample_sizes {
        // Generate test samples
        let samples: Vec<f32> = (0..size)
            .map(|i| (i as f32 * 0.01).sin() * 0.5)
            .collect();

        let result = run_benchmark(
            &format!("energy_calculation_{}_samples", size),
            config,
            || {
                let _ = EnergyDetector::calculate_rms_energy(&samples);
            },
        );

        results.push(result);
    }

    results
}

/// Benchmark audio buffer operations
pub fn benchmark_audio_buffer(buffer_sizes: &[usize], config: &BenchmarkConfig) -> Vec<BenchmarkResult> {
    use ringbuf::traits::{Consumer, Producer, Split};
    use ringbuf::HeapRb;

    let mut results = Vec::new();

    for &size in buffer_sizes {
        let buffer = HeapRb::<f32>::new(size);
        let (mut producer, mut consumer) = buffer.split();

        // Generate test samples
        let samples: Vec<f32> = (0..size / 2)
            .map(|i| (i as f32 * 0.01).sin() * 0.5)
            .collect();

        let result = run_benchmark(
            &format!("audio_buffer_write_{}_samples", size / 2),
            config,
            || {
                for &sample in &samples {
                    let _ = producer.try_push(sample);
                }
                // Clear buffer for next iteration
                while consumer.try_pop().is_some() {}
            },
        );

        results.push(result);
    }

    results
}

/// Benchmark transliteration operations
pub fn benchmark_transliteration(text_lengths: &[usize], config: &BenchmarkConfig) -> Vec<BenchmarkResult> {
    use crate::transliteration::Transliterator;

    let transliterator = Transliterator::new();
    let mut results = Vec::new();

    for &length in text_lengths {
        // Generate test text (mix of hiragana)
        let text: String = "こんにちは世界".chars().cycle().take(length).collect();

        let result = run_benchmark(
            &format!("transliteration_{}_chars", length),
            config,
            || {
                let _ = transliterator.analyze(&text);
            },
        );

        results.push(result);
    }

    results
}

/// Benchmark keyword filtering
pub fn benchmark_keyword_filter(text_lengths: &[usize], filter_sizes: &[usize], config: &BenchmarkConfig) -> Vec<BenchmarkResult> {
    use crate::utils::keyword_filter::KeywordFilter;

    let mut results = Vec::new();

    for &filter_size in filter_sizes {
        // Generate filter words
        let filter_words: Vec<String> = (0..filter_size)
            .map(|i| format!("badword{}", i))
            .collect();

        let filter = KeywordFilter::new();
        filter.load_filters(filter_words);

        for &text_length in text_lengths {
            // Generate test text
            let text: String = "hello world this is a test message "
                .chars()
                .cycle()
                .take(text_length)
                .collect();

            let result = run_benchmark(
                &format!("keyword_filter_{}_chars_{}_filters", text_length, filter_size),
                config,
                || {
                    let _ = filter.contains_filtered_word(&text);
                },
            );

            results.push(result);
        }
    }

    results
}

/// Benchmark message formatting
pub fn benchmark_message_formatting(config: &BenchmarkConfig) -> Vec<BenchmarkResult> {
    use crate::utils::message_formatter::MessageFormatter;
    use crate::config::types::{MessageFormatParts, MessagePart, TranslationPart};

    let format = MessageFormatParts {
        message: MessagePart {
            prefix: "[".to_string(),
            suffix: "]".to_string(),
        },
        separator: " | ".to_string(),
        translation: TranslationPart {
            prefix: "(".to_string(),
            separator: ", ".to_string(),
            suffix: ")".to_string(),
        },
        translation_first: false,
    };

    let formatter = MessageFormatter::new(format.clone(), format, false);
    let mut results = Vec::new();

    // Test with different message lengths
    for length in [10, 100, 500, 1000] {
        let message: String = "Hello world! ".chars().cycle().take(length).collect();
        let translation = "こんにちは世界！";

        let result = run_benchmark(
            &format!("message_format_{}_chars", length),
            config,
            || {
                let _ = formatter.format_sent_message(&message, Some(translation));
            },
        );

        results.push(result);
    }

    results
}

/// Run all benchmarks and generate a comprehensive report
pub fn run_all_benchmarks(config: &BenchmarkConfig) -> Vec<BenchmarkResult> {
    let mut all_results = Vec::new();

    info!("Starting comprehensive benchmark suite...");

    // Energy calculation benchmarks
    info!("Running energy calculation benchmarks...");
    let energy_results = benchmark_energy_calculation(
        &[1000, 4800, 16000, 48000],
        config,
    );
    all_results.extend(energy_results);

    // Audio buffer benchmarks
    info!("Running audio buffer benchmarks...");
    let buffer_results = benchmark_audio_buffer(
        &[4800, 16000, 48000, 96000],
        config,
    );
    all_results.extend(buffer_results);

    // Transliteration benchmarks
    info!("Running transliteration benchmarks...");
    let translit_results = benchmark_transliteration(
        &[10, 50, 100, 500],
        config,
    );
    all_results.extend(translit_results);

    // Keyword filter benchmarks
    info!("Running keyword filter benchmarks...");
    let filter_results = benchmark_keyword_filter(
        &[100, 500, 1000],
        &[10, 100, 1000],
        config,
    );
    all_results.extend(filter_results);

    // Message formatting benchmarks
    info!("Running message formatting benchmarks...");
    let format_results = benchmark_message_formatting(config);
    all_results.extend(format_results);

    info!("Benchmark suite complete. {} benchmarks run.", all_results.len());

    all_results
}

/// Format benchmark results as a report
pub fn format_benchmark_report(results: &[BenchmarkResult]) -> String {
    let mut output = String::new();
    output.push_str("=== VRCT Benchmark Report ===\n\n");

    // Group by category
    let mut energy_results = Vec::new();
    let mut buffer_results = Vec::new();
    let mut translit_results = Vec::new();
    let mut filter_results = Vec::new();
    let mut format_results = Vec::new();
    let mut other_results = Vec::new();

    for result in results {
        if result.name.starts_with("energy_") {
            energy_results.push(result);
        } else if result.name.starts_with("audio_buffer_") {
            buffer_results.push(result);
        } else if result.name.starts_with("transliteration_") {
            translit_results.push(result);
        } else if result.name.starts_with("keyword_filter_") {
            filter_results.push(result);
        } else if result.name.starts_with("message_format_") {
            format_results.push(result);
        } else {
            other_results.push(result);
        }
    }

    if !energy_results.is_empty() {
        output.push_str("--- Energy Calculation ---\n");
        for result in energy_results {
            output.push_str(&format!("  {}\n", result.format()));
        }
        output.push('\n');
    }

    if !buffer_results.is_empty() {
        output.push_str("--- Audio Buffer Operations ---\n");
        for result in buffer_results {
            output.push_str(&format!("  {}\n", result.format()));
        }
        output.push('\n');
    }

    if !translit_results.is_empty() {
        output.push_str("--- Transliteration ---\n");
        for result in translit_results {
            output.push_str(&format!("  {}\n", result.format()));
        }
        output.push('\n');
    }

    if !filter_results.is_empty() {
        output.push_str("--- Keyword Filtering ---\n");
        for result in filter_results {
            output.push_str(&format!("  {}\n", result.format()));
        }
        output.push('\n');
    }

    if !format_results.is_empty() {
        output.push_str("--- Message Formatting ---\n");
        for result in format_results {
            output.push_str(&format!("  {}\n", result.format()));
        }
        output.push('\n');
    }

    if !other_results.is_empty() {
        output.push_str("--- Other ---\n");
        for result in other_results {
            output.push_str(&format!("  {}\n", result.format()));
        }
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.iterations, 100);
        assert_eq!(config.warmup_iterations, 10);
        assert!(!config.verbose);
    }

    #[test]
    fn test_run_benchmark() {
        let config = BenchmarkConfig {
            iterations: 10,
            warmup_iterations: 2,
            verbose: false,
        };

        let mut counter = 0;
        let result = run_benchmark("test_benchmark", &config, || {
            counter += 1;
        });

        assert_eq!(result.name, "test_benchmark");
        assert_eq!(result.iterations, 10);
        // Counter should be warmup + iterations
        assert_eq!(counter, 12);
        assert!(result.throughput > 0.0);
    }

    #[test]
    fn test_benchmark_energy_calculation() {
        let config = BenchmarkConfig {
            iterations: 10,
            warmup_iterations: 2,
            verbose: false,
        };

        let results = benchmark_energy_calculation(&[1000, 4800], &config);
        assert_eq!(results.len(), 2);
        
        for result in &results {
            assert!(result.throughput > 0.0);
            assert!(result.avg_time > Duration::ZERO);
        }
    }

    #[test]
    fn test_benchmark_transliteration() {
        let config = BenchmarkConfig {
            iterations: 10,
            warmup_iterations: 2,
            verbose: false,
        };

        let results = benchmark_transliteration(&[10, 50], &config);
        assert_eq!(results.len(), 2);
        
        for result in &results {
            assert!(result.throughput > 0.0);
        }
    }

    #[test]
    fn test_benchmark_keyword_filter() {
        let config = BenchmarkConfig {
            iterations: 10,
            warmup_iterations: 2,
            verbose: false,
        };

        let results = benchmark_keyword_filter(&[100], &[10], &config);
        assert_eq!(results.len(), 1);
        assert!(results[0].throughput > 0.0);
    }

    #[test]
    fn test_benchmark_message_formatting() {
        let config = BenchmarkConfig {
            iterations: 10,
            warmup_iterations: 2,
            verbose: false,
        };

        let results = benchmark_message_formatting(&config);
        assert!(!results.is_empty());
        
        for result in &results {
            assert!(result.throughput > 0.0);
        }
    }

    #[test]
    fn test_format_benchmark_report() {
        let results = vec![
            BenchmarkResult {
                name: "energy_test".to_string(),
                iterations: 100,
                total_time: Duration::from_millis(100),
                avg_time: Duration::from_millis(1),
                min_time: Duration::from_micros(500),
                max_time: Duration::from_millis(2),
                throughput: 1000.0,
            },
            BenchmarkResult {
                name: "transliteration_test".to_string(),
                iterations: 100,
                total_time: Duration::from_millis(200),
                avg_time: Duration::from_millis(2),
                min_time: Duration::from_millis(1),
                max_time: Duration::from_millis(5),
                throughput: 500.0,
            },
        ];

        let report = format_benchmark_report(&results);
        assert!(report.contains("Energy Calculation"));
        assert!(report.contains("Transliteration"));
        assert!(report.contains("energy_test"));
        assert!(report.contains("transliteration_test"));
    }
}

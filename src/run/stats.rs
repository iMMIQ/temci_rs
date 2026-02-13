//! Statistical analysis for benchmark results
//!
//! Provides statistical functions for analyzing benchmark data including
//! descriptive statistics, percentiles, outlier detection, and confidence intervals.
//!
//! # Implementation
//!
//! This module uses the following crates for statistical computations:
//! - [`statrs`](https://docs.rs/statrs) for mean, median, and percentile calculations
//! - [`stats-ci`](https://docs.rs/stats-ci) for confidence interval calculations
//!
//! Custom implementations are retained for:
//! - Outlier detection using IQR method
//! - Sample management utilities

use std::time::Duration;
use statrs::statistics::{Data, OrderStatistics, Statistics as StatrsStatistics};
use stats_ci::Confidence;

/// A collection of data samples for statistical analysis
#[derive(Debug, Clone)]
pub struct Sample {
    data: Vec<f64>,
}

impl Sample {
    /// Create a new sample from a vector of values
    pub fn new(data: Vec<f64>) -> Self {
        Self { data }
    }

    /// Create a sample from a vector of Durations, converting to milliseconds
    pub fn from_durations(durations: Vec<Duration>) -> Self {
        let data = durations
            .iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();
        Self { data }
    }

    /// Return the number of samples
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the sample is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get a reference to the underlying data (sorted)
    pub fn data(&self) -> &[f64] {
        &self.data
    }

    /// Add a single value to the sample
    pub fn push(&mut self, value: f64) {
        self.data.push(value);
    }

    /// Extend the sample with multiple values
    pub fn extend(&mut self, values: Vec<f64>) {
        self.data.extend(values);
    }

    /// Get sorted data
    fn sorted_data(&self) -> Vec<f64> {
        let mut sorted = self.data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        sorted
    }

    /// Detect outliers using the IQR (Interquartile Range) method
    /// multiplier is typically 1.5 for mild outliers, 3.0 for extreme outliers
    pub fn detect_outliers(&self, multiplier: f64) -> Vec<f64> {
        if self.len() < 4 {
            return Vec::new();
        }

        let sorted = self.sorted_data();
        let q1 = Self::percentile(&sorted, 25.0);
        let q3 = Self::percentile(&sorted, 75.0);
        let iqr = q3 - q1;
        let lower_bound = q1 - (multiplier * iqr);
        let upper_bound = q3 + (multiplier * iqr);

        self.data
            .iter()
            .filter(|&&x| x < lower_bound || x > upper_bound)
            .copied()
            .collect()
    }

    /// Calculate confidence interval using t-distribution via stats-ci
    pub fn confidence_interval(&self, confidence: f64) -> ConfidenceInterval {
        if self.len() < 2 {
            return ConfidenceInterval {
                confidence,
                lower: 0.0,
                upper: 0.0,
            };
        }

        // Use stats-ci for accurate confidence interval calculation
        // stats-ci uses Student's t-distribution up to ~100,000 samples, then normal distribution
        let confidence_level = Confidence::new(confidence);
        let result = stats_ci::mean::Arithmetic::ci(confidence_level, &self.data);

        match result {
            Ok(interval) => {
                let lower = interval.low_f();
                let upper = interval.high_f();
                ConfidenceInterval {
                    confidence,
                    lower,
                    upper,
                }
            }
            Err(_) => {
                // Fallback to simple calculation if stats-ci fails
                let stats = Statistics::from_sample(self);
                let mean = stats.mean.unwrap_or(0.0);
                let std_dev = stats.std_dev.unwrap_or(0.0);
                let n = self.len() as f64;
                let t_value = match confidence {
                    c if (c - 0.90).abs() < 0.01 => 1.645,
                    c if (c - 0.95).abs() < 0.01 => 1.960,
                    c if (c - 0.99).abs() < 0.01 => 2.576,
                    _ => 1.96,
                };
                let margin = t_value * std_dev / n.sqrt();
                ConfidenceInterval {
                    confidence,
                    lower: mean - margin,
                    upper: mean + margin,
                }
            }
        }
    }

    /// Calculate percentile using statrs (R-7 algorithm, industry standard)
    fn percentile(sorted_data: &[f64], percentile: f64) -> f64 {
        if sorted_data.is_empty() {
            return 0.0;
        }
        // statrs quantile requires mutable data (it modifies in place)
        // We clone since we want to keep sorted_data unchanged
        let mut data = Data::new(sorted_data.to_vec());
        data.quantile(percentile / 100.0)
    }
}

/// Confidence interval for a sample mean
#[derive(Debug, Clone)]
pub struct ConfidenceInterval {
    /// The confidence level (e.g., 0.95 for 95% confidence)
    pub confidence: f64,
    /// Lower bound of the interval
    pub lower: f64,
    /// Upper bound of the interval
    pub upper: f64,
}

/// Statistical summary of a sample
#[derive(Debug, Clone)]
pub struct Statistics {
    /// Arithmetic mean
    pub mean: Option<f64>,
    /// Median (50th percentile)
    pub median: Option<f64>,
    /// Minimum value
    pub min: Option<f64>,
    /// Maximum value
    pub max: Option<f64>,
    /// Variance (sample variance)
    pub variance: Option<f64>,
    /// Standard deviation
    pub std_dev: Option<f64>,
    /// 25th percentile
    pub p25: Option<f64>,
    /// 50th percentile (same as median)
    pub p50: Option<f64>,
    /// 75th percentile
    pub p75: Option<f64>,
    /// 90th percentile
    pub p90: Option<f64>,
    /// 95th percentile
    pub p95: Option<f64>,
    /// 99th percentile
    pub p99: Option<f64>,
}

impl Statistics {
    /// Calculate statistics from a sample
    pub fn from_sample(sample: &Sample) -> Self {
        if sample.is_empty() {
            return Self {
                mean: None,
                median: None,
                min: None,
                max: None,
                variance: None,
                std_dev: None,
                p25: None,
                p50: None,
                p75: None,
                p90: None,
                p95: None,
                p99: None,
            };
        }

        let sorted = sample.sorted_data();
        let n = sample.len();

        // Compute mean from unsorted data (no clone needed)
        let mean = sample.data().mean();
        let min = sorted.first().copied();
        let max = sorted.last().copied();

        // Calculate variance (sample variance)
        let variance = if n > 1 {
            let sum_squared_diff: f64 = sorted
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum();
            Some(sum_squared_diff / (n - 1) as f64)
        } else {
            Some(0.0)
        };

        let std_dev = variance.map(|v| v.sqrt());

        let p25 = Some(Sample::percentile(&sorted, 25.0));
        let p50 = Some(Sample::percentile(&sorted, 50.0));
        let p75 = Some(Sample::percentile(&sorted, 75.0));
        let p90 = Some(Sample::percentile(&sorted, 90.0));
        let p95 = Some(Sample::percentile(&sorted, 95.0));
        let p99 = Some(Sample::percentile(&sorted, 99.0));

        Self {
            mean: Some(mean),
            median: p50,
            min,
            max,
            variance,
            std_dev,
            p25,
            p50,
            p75,
            p90,
            p95,
            p99,
        }
    }

    /// Calculate the range (max - min)
    pub fn range(&self) -> Option<f64> {
        match (self.min, self.max) {
            (Some(min), Some(max)) => Some(max - min),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        // statrs uses R-7 algorithm (industry standard)
        assert!((Sample::percentile(&data, 25.0) - 2.9167).abs() < 0.01);
        assert!((Sample::percentile(&data, 50.0) - 5.5).abs() < 0.01);
        assert!((Sample::percentile(&data, 75.0) - 8.0833).abs() < 0.01);
    }
}

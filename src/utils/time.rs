//! Time conversion utilities
//!
//! Provides helper functions for converting time durations to commonly used formats.

use std::time::Duration;

/// Convert a Duration to milliseconds (f64)
///
/// # Example
/// ```
/// use std::time::Duration;
/// use temci::utils::time::duration_to_ms;
///
/// let dur = Duration::from_secs(1);
/// assert_eq!(duration_to_ms(dur), 1000.0);
/// ```
pub fn duration_to_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

/// Convert a Duration reference to milliseconds (f64)
///
/// Useful when working with references to Durations, avoiding unnecessary clones.
///
/// # Example
/// ```
/// use std::time::Duration;
/// use temci::utils::time::duration_as_ms;
///
/// let dur = Duration::from_millis(500);
/// assert_eq!(duration_as_ms(&dur), 500.0);
/// ```
pub fn duration_as_ms(duration: &Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_to_ms() {
        assert_eq!(duration_to_ms(Duration::from_secs(1)), 1000.0);
        assert_eq!(duration_to_ms(Duration::from_millis(500)), 500.0);
        assert_eq!(duration_to_ms(Duration::from_nanos(1_000_000)), 1.0);
        assert_eq!(duration_to_ms(Duration::ZERO), 0.0);
    }

    #[test]
    fn test_duration_as_ms() {
        assert_eq!(duration_as_ms(&Duration::from_secs(2)), 2000.0);
        assert_eq!(duration_as_ms(&Duration::from_millis(250)), 250.0);
    }
}

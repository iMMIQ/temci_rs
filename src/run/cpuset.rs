//! CPU set and CPU affinity management
//!
//! Provides functionality for managing CPU assignments and
//! setting CPU affinity for processes.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fmt;

/// Errors that can occur when working with CPU sets
#[derive(Debug, Clone)]
pub enum CpuSetError {
    /// Invalid CPU specification
    InvalidSpec(String),
    /// CPU not available
    CpuNotAvailable(usize),
    /// System error getting CPU count
    SystemError(String),
}

impl fmt::Display for CpuSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuSetError::InvalidSpec(s) => write!(f, "Invalid CPU specification: {}", s),
            CpuSetError::CpuNotAvailable(cpu) => write!(f, "CPU {} not available", cpu),
            CpuSetError::SystemError(s) => write!(f, "System error: {}", s),
        }
    }
}

impl std::error::Error for CpuSetError {}

/// A set of CPU cores
#[derive(Debug, Clone)]
pub struct CpuSet {
    cpus: BTreeSet<usize>,
}

impl CpuSet {
    /// Create an empty CPU set
    pub fn empty() -> Self {
        Self {
            cpus: BTreeSet::new(),
        }
    }

    /// Create a CPU set from a single CPU
    pub fn from_cpu(cpu: usize) -> Self {
        let mut cpuset = Self::empty();
        cpuset.cpus.insert(cpu);
        cpuset
    }

    /// Create a CPU set from a slice of CPUs
    pub fn from_cpus(cpus: &[usize]) -> Self {
        let mut cpuset = Self::empty();
        for &cpu in cpus {
            cpuset.cpus.insert(cpu);
        }
        cpuset
    }

    /// Create a CPU set from a range of CPUs (inclusive)
    pub fn from_range(start: usize, end: usize) -> Self {
        let mut cpuset = Self::empty();
        for cpu in start..end {
            cpuset.cpus.insert(cpu);
        }
        cpuset
    }

    /// Create a CPU set containing all available CPUs
    pub fn all() -> Self {
        let count = num_cpus::get();
        if count == 0 {
            return Self::empty();
        }
        (0..count).collect()
    }

    /// Parse a CPU specification string
    ///
    /// Supported formats:
    /// - Single CPU: "0"
    /// - List: "0,2,4"
    /// - Range: "0-3"
    /// - Mixed: "0,2-4,6"
    pub fn parse(spec: &str) -> Result<Self, CpuSetError> {
        if spec.is_empty() {
            return Err(CpuSetError::InvalidSpec("Empty specification".to_string()));
        }

        let mut cpuset = Self::empty();

        for part in spec.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some(range_idx) = part.find('-') {
                // Range: "0-3"
                let start_str = &part[..range_idx];
                let end_str = &part[range_idx + 1..];

                let start = start_str.parse().map_err(|_| {
                    CpuSetError::InvalidSpec(format!("Invalid range start: {}", start_str))
                })?;

                let end = end_str.parse().map_err(|_| {
                    CpuSetError::InvalidSpec(format!("Invalid range end: {}", end_str))
                })?;

                if start > end {
                    return Err(CpuSetError::InvalidSpec(format!(
                        "Range start {} greater than end {}",
                        start, end
                    )));
                }

                cpuset.add_range(start, end + 1);
            } else {
                // Single CPU
                let cpu = part.parse().map_err(|_| {
                    CpuSetError::InvalidSpec(format!("Invalid CPU: {}", part))
                })?;
                cpuset.add(cpu);
            }
        }

        Ok(cpuset)
    }

    /// Add a CPU to the set
    pub fn add(&mut self, cpu: usize) {
        self.cpus.insert(cpu);
    }

    /// Add a range of CPUs to the set (end is exclusive)
    pub fn add_range(&mut self, start: usize, end: usize) {
        for cpu in start..end {
            self.cpus.insert(cpu);
        }
    }

    /// Remove a CPU from the set
    pub fn remove(&mut self, cpu: usize) {
        self.cpus.remove(&cpu);
    }

    /// Clear all CPUs from the set
    pub fn clear(&mut self) {
        self.cpus.clear();
    }

    /// Check if the set contains a CPU
    pub fn contains(&self, cpu: usize) -> bool {
        self.cpus.contains(&cpu)
    }

    /// Get the number of CPUs in the set
    pub fn len(&self) -> usize {
        self.cpus.len()
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.cpus.is_empty()
    }

    /// Get the first (lowest) CPU in the set
    pub fn first(&self) -> Option<usize> {
        self.cpus.iter().next().copied()
    }

    /// Get the total number of CPUs available on the system
    pub fn count(&self) -> usize {
        num_cpus::get()
    }

    /// Iterate over CPUs in the set (in ascending order)
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.cpus.iter().copied()
    }

    /// Convert to a vector of CPUs
    pub fn to_vec(&self) -> Vec<usize> {
        self.cpus.iter().copied().collect()
    }

    /// Create an iterator that collects into a CpuSet
    #[allow(clippy::should_implement_trait)]
    pub fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        let mut cpuset = Self::empty();
        for cpu in iter {
            cpuset.cpus.insert(cpu);
        }
        cpuset
    }
}

impl FromIterator<usize> for CpuSet {
    fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        Self::from_iter(iter)
    }
}

impl fmt::Display for CpuSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cpus: Vec<String> = self.cpus.iter().map(|c| c.to_string()).collect();
        write!(f, "{{{}}}", cpus.join(", "))
    }
}

impl PartialEq for CpuSet {
    fn eq(&self, other: &Self) -> bool {
        self.cpus == other.cpus
    }
}

impl Eq for CpuSet {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_iter() {
        let cpuset: CpuSet = [1, 3, 5].into_iter().collect();
        assert_eq!(cpuset.len(), 3);
        assert!(cpuset.contains(1));
        assert!(cpuset.contains(3));
        assert!(cpuset.contains(5));
    }
}

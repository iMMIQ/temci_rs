//! Worker pool for parallel execution
//!
//! Provides a semaphore-based worker pool for limiting concurrent operations.

use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::debug;

/// A worker pool that limits concurrent execution using a semaphore.
#[derive(Debug, Clone)]
pub struct WorkerPool {
    /// The inner semaphore for actual concurrency control
    inner: Arc<Semaphore>,

    /// Number of total permits (for tracking).
    permits: usize,
}

impl WorkerPool {
    /// Create a new worker pool with a specified number of permits.
    pub fn new(permits: usize) -> Self {
        debug!("Creating worker pool with {} permits", permits);
        Self {
            inner: Arc::new(Semaphore::new(permits)),
            permits,
        }
    }

    /// Acquire a permit from the pool asynchronously.
    /// Returns Some(Permit) when acquired, None if closed.
    pub async fn acquire(&self) -> Option<Permit> {
        match self.inner.clone().acquire_owned().await {
            Ok(permit) => Some(Permit {
                _permit: Some(permit),
                acquired: true,
                has: true,
            }),
            Err(_) => None,
        }
    }

    /// Try to acquire a permit without blocking.
    /// Returns Some(Permit) if a permit is available, None otherwise.
    pub fn try_acquire(&self) -> Option<Permit> {
        self.inner.clone().try_acquire_owned().ok().map(|permit| Permit {
            _permit: Some(permit),
            acquired: true,
            has: true,
        })
    }

    /// Get the number of available permits.
    pub fn available_permits(&self) -> usize {
        self.inner.available_permits()
    }

    /// Close the semaphore, preventing new acquisitions.
    pub fn close(&self) {
        self.inner.close();
    }

    /// Check if the semaphore is closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Get a reference to the inner semaphore.
    pub fn inner(&self) -> &Arc<Semaphore> {
        &self.inner
    }

    /// Get the total number of permits.
    pub fn total_permits(&self) -> usize {
        self.permits
    }
}

/// A permit acquired from a WorkerPool.
///
/// When dropped, the permit is returned to the pool.
///
/// This holds an OwnedSemaphorePermit which automatically
/// returns the permit to the semaphore when dropped.
#[derive(Debug)]
pub struct Permit {
    /// The actual owned permit from the semaphore.
    /// When dropped, this returns the permit to the semaphore.
    _permit: Option<OwnedSemaphorePermit>,

    /// Whether the permit is valid/has ownership.
    pub has: bool,

    /// Whether this permit holds a live semaphore permit.
    pub acquired: bool,
}

impl Clone for Permit {
    fn clone(&self) -> Self {
        Self {
            _permit: None,  // Cloned permits don't hold the semaphore permit
            has: false,
            acquired: false,
        }
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        if self._permit.is_some() {
            debug!("Permit dropped, returned to worker pool");
        }
        // OwnedSemaphorePermit handles returning the permit to the semaphore
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_pool_creation() {
        let pool = WorkerPool::new(5);
        assert_eq!(pool.available_permits(), 5);
        assert_eq!(pool.total_permits(), 5);
        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn test_worker_pool_acquire() {
        let pool = WorkerPool::new(2);

        // Acquire two permits
        let permit1 = pool.acquire().await;
        assert!(permit1.is_some());
        assert!(permit1.as_ref().unwrap().acquired);
        assert_eq!(pool.available_permits(), 1);
        assert_eq!(pool.total_permits(), 2);

        let permit2 = pool.acquire().await;
        assert!(permit2.is_some());
        assert!(permit2.as_ref().unwrap().acquired);
        assert_eq!(pool.available_permits(), 0);

        // Give time for the drop to complete
        drop(permit1);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        assert_eq!(pool.available_permits(), 1);
        assert_eq!(pool.total_permits(), 2);
    }

    #[test]
    fn test_worker_pool_try_acquire() {
        let pool = WorkerPool::new(1);

        let permit1 = pool.try_acquire();
        assert!(permit1.is_some());
        assert!(permit1.as_ref().unwrap().acquired);

        let permit2 = pool.try_acquire();
        assert!(permit2.is_none());

        drop(permit1);

        let permit2 = pool.try_acquire();
        assert!(permit2.is_some());
    }
}

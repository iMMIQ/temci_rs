use temci::run::worker_pool::{WorkerPool};

#[test]
fn test_basic() {
    let pool = WorkerPool::new(2);
    let permit1 = pool.try_acquire();
    assert!(permit1.is_some());
}

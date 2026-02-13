use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use temci::run::worker_pool::WorkerPool;

#[tokio::test]
async fn test_worker_pool_basic() {
    let pool = WorkerPool::new(2);

    let task1 = pool.acquire().await;
    let task2 = pool.acquire().await;

    assert!(task1.is_some());
    assert!(task2.is_some());
    assert!(task1.unwrap().acquired);
    assert!(task2.unwrap().acquired);
}

#[tokio::test]
async fn test_worker_pool_concurrent() {
    let pool = Arc::new(WorkerPool::new(2));

    let handle1 = pool.acquire().await;
    let handle2 = pool.acquire().await;

    assert!(handle1.is_some());
    assert!(handle2.is_some());
    assert!(handle1.as_ref().unwrap().has);
    assert!(handle2.as_ref().unwrap().acquired);

    // Spawn a task that tries to acquire
    let pool_clone = Arc::clone(&pool);
    let handle3_task = tokio::spawn(async move {
        pool_clone.acquire().await
    });

    // Give it a moment to try to acquire
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Drop first handle
    drop(handle1);

    // Now the task should be able to acquire
    let handle3 = handle3_task.await.unwrap();
    assert!(handle3.is_some());
    assert!(handle3.unwrap().acquired);
}

#[tokio::test]
async fn test_worker_pool_single_threaded() {
    let pool = WorkerPool::new(1);

    let handle1 = pool.acquire().await;
    assert!(handle1.is_some());
    assert!(handle1.as_ref().unwrap().has);

    // Drop first handle
    drop(handle1);

    // Now it should work
    let handle2 = pool.acquire().await;
    assert!(handle2.is_some());
    assert!(handle2.unwrap().acquired);
}

#[tokio::test]
async fn test_worker_pool_max_permits() {
    let pool = WorkerPool::new(10);

    let mut handles = Vec::new();
    for _ in 0..10 {
        let handle = pool.acquire().await;
        assert!(handle.is_some());
        assert!(handle.as_ref().unwrap().acquired);
        handles.push(handle);
    }

    // Release all permits
    handles.clear();

    // Now we should be able to acquire again
    let handle = pool.acquire().await;
    assert!(handle.is_some());
    assert!(handle.unwrap().acquired);
}

#[tokio::test]
async fn test_worker_pool_try_acquire() {
    let pool = WorkerPool::new(1);

    let handle1 = pool.try_acquire();
    assert!(handle1.is_some());
    assert!(handle1.as_ref().unwrap().has);
    assert!(handle1.as_ref().unwrap().acquired);

    let handle2 = pool.try_acquire();
    assert!(handle2.is_none());

    drop(handle1);

    let handle2 = pool.try_acquire();
    assert!(handle2.is_some());
    assert!(handle2.as_ref().unwrap().has);
    assert!(handle2.as_ref().unwrap().acquired);
}

#[tokio::test]
async fn test_worker_pool_with_timeout() {
    let pool = Arc::new(WorkerPool::new(1));

    let handle1 = pool.acquire().await;
    assert!(handle1.is_some());
    assert!(handle1.as_ref().unwrap().has);

    // Try to acquire with timeout - should timeout since we don't release
    let pool_clone = Arc::clone(&pool);
    let handle2_result = timeout(Duration::from_millis(100), async move {
        pool_clone.acquire().await
    }).await;
    assert!(handle2_result.is_err());

    // Release first handle
    drop(handle1);

    // Now it should succeed
    let handle2 = timeout(Duration::from_millis(100), pool.acquire()).await;
    assert!(handle2.is_ok());
    let handle2 = handle2.unwrap();
    assert!(handle2.is_some());
    assert!(handle2.unwrap().acquired);
}

#[tokio::test]
async fn test_worker_pool_semaphore_size() {
    let pool = WorkerPool::new(5);

    assert_eq!(pool.available_permits(), 5);

    let _h1 = pool.acquire().await;
    assert_eq!(pool.available_permits(), 4);

    let _h2 = pool.acquire().await;
    assert_eq!(pool.available_permits(), 3);
}

#[tokio::test]
async fn test_worker_pool_scoped() {
    let pool = Arc::new(WorkerPool::new(2));

    let handle1 = pool.acquire().await;
    assert!(handle1.is_some());
    assert!(handle1.as_ref().unwrap().has);

    let handle2 = pool.acquire().await;
    assert!(handle2.is_some());
    assert!(handle2.as_ref().unwrap().acquired);

    // Spawn a task that tries to acquire
    let pool_clone = Arc::clone(&pool);
    let handle3_task = tokio::spawn(async move {
        pool_clone.acquire().await
    });

    // Give it a moment to try to acquire
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Release handle2
    drop(handle2);

    // Now the task should be able to acquire
    let handle3 = handle3_task.await.unwrap();
    assert!(handle3.is_some());
    assert!(handle3.unwrap().acquired);
}

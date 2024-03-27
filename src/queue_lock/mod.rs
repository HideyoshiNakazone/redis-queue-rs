pub mod async_queue_lock;
pub mod queue_lock;
pub mod queue_lock_builder;


// Write tests
#[cfg(test)]
mod tests {
    use std::sync::{Arc};
    use crate::queue_lock::queue_lock_builder::QueueLockBuilder;
    use crate::test_utils::init_redis::{initialize_async_redis, initialize_redis, initialize_redis_client};

    #[tokio::test]
    async fn initialize_async_queue_lock() {
        let queue_lock = super::async_queue_lock::AsyncQueueLock::new(
            "test".to_string(),
            initialize_async_redis().await,
            None
        );
        assert_eq!(queue_lock.get_lock_name(), "redis-queue:test:lock".to_string());
    }
    
    #[tokio::test]
    async fn test_async_lock() {
        let mut queue_lock = super::async_queue_lock::AsyncQueueLock::new(
            "test".to_string(),
            initialize_async_redis().await,
            None
        );
        queue_lock.lock(
            || async {
                println!("Locked");
            }
        ).await;
    }
    
    #[tokio::test]
    async fn test_async_concurrent_lock() {
        let increment_mutex = Arc::new(async_std::sync::Mutex::new(0));
        
        let increment_mutex_1 = increment_mutex.clone();
        let h1 = tokio::spawn(async move {
            let mut queue_lock = super::async_queue_lock::AsyncQueueLock::new(
                "test".to_string(),
                initialize_async_redis().await,
                None
            );
            queue_lock.lock(
                || async {
                    let mut increment = increment_mutex_1.lock().await;
                    if *increment == 0 {
                        *increment = 1;
                    } else {
                        panic!("Incremented second");
                    }
                }
            ).await;
        });
        
        let increment_mutex_2 = increment_mutex.clone();
        let h2 = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            let mut queue_lock2 = super::async_queue_lock::AsyncQueueLock::new(
                "test".to_string(),
                initialize_async_redis().await,
                None
            );
            queue_lock2.lock(
                || async {
                    let mut increment = increment_mutex_2.lock().await;
                    if *increment == 1 {
                        *increment = 2;
                    } else {
                        panic!("Incremented first");
                    }
                }
            ).await;

        });
        
        h1.await.unwrap();
        h2.await.unwrap();
    }
    
    #[test]
    fn test_initialize_queue_lock() {
        let queue_lock = super::queue_lock::QueueLock::new(
            "test".to_string(),
            initialize_redis(),
            None
        );
        assert_eq!(queue_lock.get_lock_name(), "redis-queue:test:lock".to_string());
    }
    
    #[test]
    fn test_lock() {
        let mut queue_lock = super::queue_lock::QueueLock::new(
            "test".to_string(),
            initialize_redis(),
            None
        );
        let result: u8 = queue_lock.lock(
            || {
                return 0;
            }
        );
        assert_eq!(result, 0);
    }
    
    #[test]
    fn test_concurrent_lock() {
        let increment_mutex = Arc::new(std::sync::Mutex::new(0));

        let increment_mutex_1 = increment_mutex.clone();
        let h1 = std::thread::spawn(move || {
            let mut queue_lock = super::queue_lock::QueueLock::new(
                "test".to_string(),
                initialize_redis(),
                None
            );

            queue_lock.lock(
                || {
                    let mut increment = increment_mutex_1.lock().unwrap();
                    if *increment == 0 {
                        *increment = 1;
                    } else {
                        panic!("Incremented second");
                    }
                }
            );
        });

        let increment_mutex_2 = increment_mutex.clone();
        let h2 = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let mut queue_lock2 = super::queue_lock::QueueLock::new(
                "test".to_string(),
                initialize_redis(),
                None
            );
            queue_lock2.lock(
                || {
                    let mut increment = increment_mutex_2.lock().unwrap();
                    if *increment == 1 {
                        *increment = 2;
                    } else {
                        panic!("Incremented first");
                    }
                }
            );

        });

        h1.join().unwrap();
        h2.join().unwrap();
    }

    #[test]
    fn test_queue_builder() {
        let queue_lock = QueueLockBuilder::default()
            .with_queue_name("test".to_string())
            .with_redis_client(initialize_redis_client())
            .with_retry_interval(100)
            .build();
        assert_eq!(queue_lock.get_lock_name(), "redis-queue:test:lock".to_string());
    }

    #[tokio::test]
    async fn test_async_queue_builder() {
        let queue_lock = QueueLockBuilder::default()
            .with_queue_name("test".to_string())
            .with_redis_client(initialize_redis_client())
            .with_retry_interval(100)
            .async_build().await;
        assert_eq!(queue_lock.get_lock_name(), "redis-queue:test:lock".to_string());
    }
}
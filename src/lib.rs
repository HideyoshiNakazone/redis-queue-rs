pub mod async_redis_queue;
mod queue_lock;
mod queue_state;
pub mod redis_queue;
mod test_utils;

#[cfg(test)]
mod tests {
    use crate::async_redis_queue::AsyncRedisQueue;
    use crate::redis_queue::RedisQueue;
    use crate::test_utils::init_redis::initialize_redis_client;

    #[test]
    fn initialize_redis_queue() {
        let _: RedisQueue<String> = RedisQueue::new(
            "initialize_redis_queue".to_string(),
            initialize_redis_client(),
        );
    }

    #[test]
    fn test_push_pop_to_redis_queue() {
        let mut redis_queue = RedisQueue::new(
            "test_push_pop_to_redis_queue".to_string(),
            initialize_redis_client(),
        );

        let item = "test".to_string();

        redis_queue.push(item.clone());
        let result = redis_queue.pop().unwrap();
        assert_eq!(result, item);
    }

    #[test]
    fn test_redis_queue_with_concurrent_push_pop() {
        let redis_queue = RedisQueue::new(
            "test_redis_queue_with_concurrent_push_pop".to_string(),
            initialize_redis_client(),
        );
        let item = "test".to_string();
        
        let num_tasks = 100;

        let mut handles = vec![];
        for _ in 0..num_tasks {
            let mut queue = redis_queue.clone();
            let local_item = item.clone();

            let handle = std::thread::spawn(move || {
                queue.push(local_item.clone());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut handles = vec![];
        for _ in 0..num_tasks {
            let result = item.clone();
            let mut queue = redis_queue.clone();

            let handle = std::thread::spawn(move || {
                let value = queue.pop();
                assert_eq!(value.unwrap(), result);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[tokio::test]
    async fn initialize_async_redis_queue() {
        let _: AsyncRedisQueue<String> = AsyncRedisQueue::new(
            "initialize_async_redis_queue".to_string(),
            initialize_redis_client(),
        )
        .await;
    }

    #[tokio::test]
    async fn test_async_push_pop_to_redis_queue() {
        let mut redis_queue = AsyncRedisQueue::new(
            "test_async_push_pop_to_redis_queue".to_string(),
            initialize_redis_client(),
        )
        .await;

        let item = "test".to_string();

        redis_queue.push(item.clone()).await;
        let result = redis_queue.pop().await.unwrap();
        assert_eq!(result, item);
    }

    #[tokio::test]
    async fn test_async_redis_queue_with_concurrent_push_pop() {
        let redis_queue = AsyncRedisQueue::new(
            "test_async_redis_queue_with_concurrent_push_pop".to_string(),
            initialize_redis_client(),
        )
        .await;
        let item = "test".to_string();
        
        let num_tasks = 100;

        let mut handles = vec![];
        for _ in 0..num_tasks {
            let mut queue = redis_queue.clone();
            let local_item = item.clone();

            let handle = tokio::spawn(async move {
                queue.push(local_item.clone()).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let mut handles = vec![];
        for _ in 0..num_tasks   {
            let result = item.clone();
            let mut queue = redis_queue.clone();

            let handle = tokio::spawn(async move {
                let value = queue.pop().await;
                assert_eq!(value.unwrap(), result);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }
}

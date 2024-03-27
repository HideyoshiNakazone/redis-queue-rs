use crate::queue_lock::async_queue_lock::AsyncQueueLock;
use crate::queue_lock::queue_lock::QueueLock;

#[derive(Clone)]
pub struct QueueLockBuilder {
    queue_name: Option<String>,
    redis_client: Option<redis::Client>,
    retry_interval: Option<u64>
}

impl QueueLockBuilder {
    pub fn default() -> Self {
        QueueLockBuilder {
            queue_name: None,
            redis_client: None,
            retry_interval: None
        }
    }
    
    pub fn with_queue_name(mut self, queue_name: String) -> Self {
        self.queue_name = Some(queue_name);
        self
    }
    
    pub fn with_redis_client(mut self, redis_client: redis::Client) -> Self {
        self.redis_client = Some(redis_client);
        self
    }
    
    pub fn with_retry_interval(mut self, retry_interval: u64) -> Self {
        self.retry_interval = Some(retry_interval);
        self
    }
    
    pub fn build(self) -> QueueLock {
        if self.redis_client.is_none() {
            panic!("Redis Client is required to build QueueLock");
        }
        
        return QueueLock::new(
            self.queue_name.unwrap(),
            self.redis_client.unwrap().get_connection().unwrap(),
            self.retry_interval
        );
    }
    
    pub async fn async_build(self) -> AsyncQueueLock {
        if self.redis_client.is_none() {
            panic!("Redis Client is required to build AsyncQueueLock");
        }
        
        return AsyncQueueLock::new(
            self.queue_name.unwrap(),
            self.redis_client.unwrap().get_multiplexed_async_connection()
                .await.unwrap(),
            self.retry_interval
        );
    }
}
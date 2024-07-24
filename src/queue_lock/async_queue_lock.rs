use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, ExistenceCheck, SetOptions};
use uuid::Uuid;

#[derive(Clone)]
pub struct AsyncQueueLock {
    redis_connection: MultiplexedConnection,
    retry_interval: u64,

    queue_name: String,
}

impl AsyncQueueLock {
    pub fn new(
        queue_name: String,
        redis_connection: MultiplexedConnection,
        retry_interval: Option<u64>,
    ) -> Self {
        AsyncQueueLock {
            redis_connection,
            retry_interval: retry_interval.unwrap_or(100),
            queue_name: queue_name.to_string(),
        }
    }

    pub async fn lock<F, R>(&mut self, f: F) -> <R as std::future::Future>::Output
    where
        F: FnOnce() -> R,
        R: std::future::Future,
    {
        let lock_identifier = Uuid::new_v4().to_string();
        while !self.try_lock(lock_identifier.clone()).await {
            async_std::task::sleep(std::time::Duration::from_millis(self.retry_interval)).await;
        }

        let result = f().await;

        self.unlock().await;

        result
    }

    pub fn get_lock_name(&self) -> String {
        format!("redis-queue:{}:lock", self.queue_name)
    }

    async fn try_lock(&mut self, lock_identifier: String) -> bool {
        let set_options = SetOptions::default()
            .conditional_set(ExistenceCheck::NX)
            .get(true);
        
        let active_lock_identifier = self.redis_connection
            .set_options::<String, String, String>(self.get_lock_name(), lock_identifier.clone(), set_options)
            .await.unwrap_or("".to_string());
        
        return active_lock_identifier == lock_identifier;
    }

    async fn unlock(&mut self) {
        self.redis_connection
            .del::<String, u8>(self.get_lock_name())
            .await
            .unwrap();
    }
}

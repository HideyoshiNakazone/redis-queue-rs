use redis::Commands;
use uuid::Uuid;
use crate::add;

pub struct QueueState {
    pub head: Option<String>,
    pub tail: Option<String>,
    pub locked: bool
}


pub struct QueueLock {
    redis_connection: redis::Connection,
    retry_interval: u64,

    queue_name: String,
    lock_identifier: Option<String>
}


impl QueueLock {
    pub fn new(queue_name: String, redis_connection: redis::Connection, retry_interval: Option<u64>) -> Self {
        QueueLock {
            redis_connection,
            retry_interval: retry_interval.unwrap_or(100),
            queue_name: queue_name.to_string(),
            lock_identifier: None
        }
    }

    pub fn lock(&mut self) {
        let lock_identifier = Uuid::new_v4().to_string();
        while !self.try_lock(lock_identifier.clone()) {
            // async_std::task::sleep(std::time::Duration::from_millis(self.retry_interval)).await;
            std::thread::sleep(std::time::Duration::from_millis(self.retry_interval));
        }
    }

    fn try_lock(&mut self, lock_identifier: String) -> bool {
        self.redis_connection.set(self.get_lock_name(), lock_identifier.clone()).unwrap();

        self.redis_connection.get(self.get_lock_name()).unwrap() == Some(lock_identifier)
    }

    fn unlock(&mut self) {
        self.redis_connection.del(self.get_lock_name()).unwrap();
    }

    fn get_lock_name(&self) -> String {
        format!("redis-queue-lock:{}", self.queue_name)
    }
}

impl Drop for QueueLock {
    fn drop(&mut self) {
        unsafe {
            self.unlock();
        }
    }
}
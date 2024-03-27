use redis::{Commands};
use redis::Connection;
use uuid::Uuid;


pub struct QueueLock {
    redis_connection: Connection,
    retry_interval: u64,

    lock_identifier: Option<String>,
    queue_name: String
}


impl QueueLock {
    pub fn new(queue_name: String, redis_connection: Connection, retry_interval: Option<u64>) -> Self {
        QueueLock {
            redis_connection,
            retry_interval: retry_interval.unwrap_or(100),
            queue_name: queue_name.to_string(),
            lock_identifier: None
        }
    }

    pub fn lock<F>(&mut self, f: F) where F: FnOnce() {
        let lock_identifier = Uuid::new_v4().to_string();
        while !self.try_lock(lock_identifier.clone()) {
            std::thread::sleep(std::time::Duration::from_millis(self.retry_interval));
        }
        
        f();
        
        self.unlock();
    }

    pub fn get_lock_name(&self) -> String {
        format!("redis-queue-lock:{}", self.queue_name)
    }

    fn try_lock(&mut self, lock_identifier: String) -> bool {
        self.redis_connection.set::<String, String, String>(
            self.get_lock_name(), lock_identifier.clone()
        ).unwrap();

        let active_lock_identifier: String = self.redis_connection.get(self.get_lock_name()).unwrap();
        return if active_lock_identifier == lock_identifier {
            self.lock_identifier = Some(lock_identifier);
            true
        } else {
            false
        }
    }
    
    fn unlock(&mut self) {
        if self.lock_identifier.is_none() {
            return;
        }
        self.redis_connection.del::<String, u8>(self.get_lock_name()).unwrap();
    }
}
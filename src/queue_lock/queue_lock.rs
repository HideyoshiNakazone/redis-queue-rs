use redis::{Commands, ExistenceCheck, SetOptions};
use redis::Connection;
use uuid::Uuid;


pub struct QueueLock {
    redis_connection: Connection,
    retry_interval: u64,

    queue_name: String
}


impl QueueLock {
    pub fn new(queue_name: String, redis_connection: Connection, retry_interval: Option<u64>) -> Self {
        QueueLock {
            redis_connection,
            retry_interval: retry_interval.unwrap_or(100),
            queue_name: queue_name.to_string(),
        }
    }
    pub fn lock<F, R>(&mut self, f: F) -> R 
    where F: FnOnce() -> R {
        let lock_identifier = Uuid::new_v4().to_string();
        while !self.try_lock(lock_identifier.clone()) {
            std::thread::sleep(std::time::Duration::from_millis(self.retry_interval));
        }
        
        let result = f();
        
        self.unlock();

        result
    }

    pub fn get_lock_name(&self) -> String {
        format!("redis-queue:{}:lock", self.queue_name)
    }

    fn try_lock(&mut self, lock_identifier: String) -> bool {
        let set_options = SetOptions::default()
            .conditional_set(ExistenceCheck::NX)
            .get(true);
        
        let active_lock_identifier: String = self.redis_connection.set_options(
            self.get_lock_name(), lock_identifier.clone(), set_options
        ).unwrap_or("".to_string());

        return active_lock_identifier == lock_identifier;
    }
    
    fn unlock(&mut self) {
        self.redis_connection.del::<String, u8>(self.get_lock_name()).unwrap();
    }
}
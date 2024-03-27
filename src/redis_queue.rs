use redis::Commands;
use serde::{Deserialize, Serialize};

use crate::queue_lock::queue_lock::QueueLock;
use crate::queue_lock::queue_lock_builder::QueueLockBuilder;
use crate::queue_state::queue_element::QueueElement;

#[derive(Clone)]
pub struct RedisQueue {
    queue_name: String,
    redis_client: redis::Client,

    queue_lock_builder: QueueLockBuilder,
}

impl RedisQueue {
    pub fn new(queue_name: String, redis_client: redis::Client) -> RedisQueue {
        let queue_lock_builder = QueueLockBuilder::default()
            .with_queue_name(queue_name.clone())
            .with_redis_client(redis_client.clone());
        RedisQueue {
            queue_name,
            queue_lock_builder,
            redis_client,
        }
    }

    pub fn push<T>(&mut self, item: T)
    where
        T: Clone + Serialize + for<'de> Deserialize<'de>,
    {
        self.get_lock().lock(move || {
            let element = QueueElement::new(item);
            self.push_element(element.clone());

            if self.get_first_element_id().is_none() {
                self.set_first_element_id(element.get_id());
            }

            self.set_last_element_id::<T>(element.get_id());
        });
    }

    pub fn pop<T>(&mut self) -> Option<T>
    where
        T: Clone + Serialize + for<'de> Deserialize<'de>,
    {
        self.get_lock().lock(|| {
            let option_element_id = self.get_first_element_id();
            if option_element_id.is_none() {
                return None;
            }

            let element_id = option_element_id.unwrap();

            let option_element = self.get_element::<T>(element_id.clone());
            if option_element.is_none() {
                return None;
            }
            let first_element = option_element.unwrap();

            if first_element.get_next().is_none() {
                self.unset_first_element_id();
                self.unset_last_element_id();
            } else {
                let next_element_id = first_element.get_next().unwrap();
                self.set_first_element_id(next_element_id);
            }

            self.delete_element(element_id.clone());

            Some(first_element.get_data())
        })
    }

    fn get_element<T>(&mut self, element_id: String) -> Option<QueueElement<T>>
    where
        T: Clone + Serialize + for<'de> Deserialize<'de>,
    {
        let element_key = format!("redis-queue:{}:element:{}", self.queue_name, element_id);
        self.redis_connection()
            .get(element_key)
            .map_or(None, |data: String| serde_json::from_str(&data).ok())
    }

    fn push_element<T>(&mut self, element: QueueElement<T>)
    where
        T: Clone + Serialize,
    {
        let element_key = format!(
            "redis-queue:{}:element:{}",
            self.queue_name,
            element.get_id()
        );
        let element_data = serde_json::to_string(&element).unwrap();

        self.redis_connection()
            .set::<String, String, String>(element_key, element_data)
            .unwrap();
    }

    fn get_first_element_id(&mut self) -> Option<String> {
        let first_element_key = format!("redis-queue:{}:state:first", self.queue_name);
        self.redis_connection().get(first_element_key).ok()
    }

    fn set_first_element_id(&mut self, element_id: String) {
        let first_element_key = format!("redis-queue:{}:state:first", self.queue_name);
        self.redis_connection()
            .set::<String, String, String>(first_element_key, element_id)
            .unwrap();
    }

    fn unset_first_element_id(&mut self) {
        let first_element_key = format!("redis-queue:{}:state:first", self.queue_name);
        self.redis_connection()
            .del::<String, u8>(first_element_key)
            .unwrap();
    }

    fn get_last_element_id(&mut self) -> Option<String> {
        let last_element_key = format!("redis-queue:{}:state:last", self.queue_name);
        self.redis_connection().get(last_element_key).ok()
    }

    fn set_last_element_id<T>(&mut self, element_id: String)
    where
        T: Clone + Serialize + for<'de> Deserialize<'de>,
    {
        let current_last_element_id = self.get_last_element_id();
        if current_last_element_id.is_some() {
            let current_last_element_id = current_last_element_id.unwrap();
            let mut current_last_element = self.get_element::<T>(current_last_element_id).unwrap();
            current_last_element.set_next(Some(element_id.clone()));
            self.update_element::<T>(current_last_element);
        }

        let last_element_key = format!("redis-queue:{}:state:last", self.queue_name);
        self.redis_connection()
            .set::<String, String, String>(last_element_key, element_id)
            .unwrap();
    }

    fn unset_last_element_id(&mut self) {
        let last_element_key = format!("redis-queue:{}:state:last", self.queue_name);
        self.redis_connection()
            .del::<String, u8>(last_element_key)
            .unwrap();
    }

    fn delete_element(&mut self, element_id: String) {
        let element_key = format!("redis-queue:{}:element:{}", self.queue_name, element_id);
        self.redis_connection()
            .del::<String, u8>(element_key)
            .unwrap();
    }

    fn update_element<T>(&mut self, element: QueueElement<T>)
    where
        T: Clone + Serialize + for<'de> Deserialize<'de>,
    {
        let element_key = format!(
            "redis-queue:{}:element:{}",
            self.queue_name,
            element.get_id()
        );
        let element_data = serde_json::to_string(&element).unwrap();

        self.redis_connection()
            .set::<String, String, String>(element_key, element_data)
            .unwrap();
    }

    fn get_lock(&self) -> QueueLock {
        self.queue_lock_builder.clone().build()
    }

    fn redis_connection(&self) -> redis::Connection {
        self.redis_client.get_connection().unwrap()
    }
}

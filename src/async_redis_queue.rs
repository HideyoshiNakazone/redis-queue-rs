use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::queue_lock::async_queue_lock::AsyncQueueLock;
use crate::queue_lock::queue_lock_builder::QueueLockBuilder;
use crate::queue_state::queue_element::QueueElement;

#[derive(Clone)]
pub struct AsyncRedisQueue<T> {
    queue_data_type: std::marker::PhantomData<T>,
    
    queue_name: String,
    redis_connection: MultiplexedConnection,

    queue_lock_builder: QueueLockBuilder,
}

impl<T> AsyncRedisQueue<T>
where T: Clone + Serialize + for<'de> Deserialize<'de> {
    pub async fn new(queue_name: String, redis_client: redis::Client) -> AsyncRedisQueue<T> {
        let queue_lock_builder = QueueLockBuilder::default()
            .with_queue_name(queue_name.clone())
            .with_redis_client(redis_client.clone());
        AsyncRedisQueue {
            queue_data_type: std::marker::PhantomData,
            queue_name,
            queue_lock_builder,
            redis_connection: redis_client
                .get_multiplexed_async_connection()
                .await
                .unwrap(),
        }
    }

    pub async fn push(&mut self, item: T) {
        self.get_lock()
            .await
            .lock(|| async move {
                let element = QueueElement::new(item);

                self.push_element(element.clone()).await;

                if self.get_first_element_id().await.is_none() {
                    self.set_first_element_id(element.get_id()).await;
                }

                self.set_last_element_id(element.get_id()).await;
            })
            .await;
    }

    pub async fn pop(&mut self) -> Option<T> {
        self.get_lock()
            .await
            .lock(|| async {
                let option_element_id = self.get_first_element_id().await;
                if option_element_id.is_none() {
                    return None;
                }

                let element_id = option_element_id.unwrap();

                let option_element = self.get_element(element_id.clone()).await;
                if option_element.is_none() {
                    return None;
                }
                let first_element = option_element.unwrap();

                if first_element.get_next().is_none() {
                    self.unset_first_element_id().await;
                    self.unset_last_element_id().await;
                } else {
                    let next_element_id = first_element.get_next().unwrap();
                    self.set_first_element_id(next_element_id).await;
                }

                self.delete_element(element_id.clone()).await;

                Some(first_element.get_data())
            })
            .await
    }

    async fn get_element(&mut self, element_id: String) -> Option<QueueElement<T>> {
        let element_key = format!("redis-queue:{}:element:{}", self.queue_name, element_id);
        self.redis_connection
            .get(element_key)
            .await
            .map_or(None, |data: String| serde_json::from_str(&data).ok())
    }

    async fn push_element(&mut self, element: QueueElement<T>) {
        let element_key = format!(
            "redis-queue:{}:element:{}",
            self.queue_name,
            element.get_id()
        );
        let element_data = serde_json::to_string(&element).unwrap();

        self.redis_connection
            .set::<String, String, String>(element_key, element_data)
            .await
            .unwrap();
    }

    async fn get_first_element_id(&mut self) -> Option<String> {
        let first_element_key = format!("redis-queue:{}:state:first", self.queue_name);
        self.redis_connection.get(first_element_key).await.ok()
    }

    async fn set_first_element_id(&mut self, element_id: String) {
        let first_element_key = format!("redis-queue:{}:state:first", self.queue_name);
        self.redis_connection
            .set::<String, String, String>(first_element_key, element_id)
            .await
            .unwrap();
    }

    async fn unset_first_element_id(&mut self) {
        let first_element_key = format!("redis-queue:{}:state:first", self.queue_name);
        self.redis_connection
            .del::<String, u8>(first_element_key)
            .await
            .unwrap();
    }

    async fn get_last_element_id(&mut self) -> Option<String> {
        let last_element_key = format!("redis-queue:{}:state:last", self.queue_name);
        self.redis_connection.get(last_element_key).await.ok()
    }

    async fn set_last_element_id(&mut self, element_id: String) {
        let current_last_element_id = self.get_last_element_id().await;
        if current_last_element_id.is_some() {
            let current_last_element_id = current_last_element_id.unwrap();
            let mut current_last_element = self
                .get_element(current_last_element_id)
                .await
                .unwrap();
            current_last_element.set_next(Some(element_id.clone()));
            self.update_element(current_last_element).await;
        }

        let last_element_key = format!("redis-queue:{}:state:last", self.queue_name);
        self.redis_connection
            .set::<String, String, String>(last_element_key, element_id)
            .await
            .unwrap();
    }

    async fn unset_last_element_id(&mut self) {
        let last_element_key = format!("redis-queue:{}:state:last", self.queue_name);
        self.redis_connection
            .del::<String, u8>(last_element_key)
            .await
            .unwrap();
    }

    async fn delete_element(&mut self, element_id: String) {
        let element_key = format!("redis-queue:{}:element:{}", self.queue_name, element_id);
        self.redis_connection
            .del::<String, u8>(element_key)
            .await
            .unwrap();
    }

    async fn update_element(&mut self, element: QueueElement<T>) {
        let element_key = format!(
            "redis-queue:{}:element:{}",
            self.queue_name,
            element.get_id()
        );
        let element_data = serde_json::to_string(&element).unwrap();

        self.redis_connection
            .set::<String, String, String>(element_key, element_data)
            .await
            .unwrap();
    }

    async fn get_lock(&self) -> AsyncQueueLock {
        self.queue_lock_builder.clone().async_build().await
    }
}

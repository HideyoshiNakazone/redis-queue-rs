# RedisQueueRS

A simple Redis queue implementation in Rust.

This project ofers two implementations: `RedisQueue` and `AsyncRedisQueue`. 
Both of them are based on the same RedisQueue struct and are thread safe due to the use of a Redis key based lock, 
but the AsyncRedisQueue uses async/await to handle the Redis connection.

This project is considered a work in progress and is not yet ready for production use.

# Advantages of a Redis Queue

- **Decoupling**: The producer and the consumer are decoupled. 
The producer can produce messages without worrying about the consumer.
- **Scalability**: The producer and the consumer can be scaled independently.
- **Reliability**: The queue can be used to store messages in case of failure.
- **Asynchronous Communication**: The producer and the consumer can be asynchronous.
- **Load Balancing**: The queue can be used to balance the load between multiple consumers.

## Installation

Add the following to your Cargo.toml:

```toml
[dependencies]
redis-queue-rs = "*"
```

## Usage

### RedisQueue - Synchronous Implementation

```rust
use redis_queue_rs::redis_queue::RedisQueue;
use redis::Client;

fn main() {
    let redis_client = Client::open("redis://127.0.0.1:6379").unwrap();
    
    let mut redis_queue = RedisQueue::new(
        "name_of_queue".to_string(),
        redis_client,
    );
    
    let item = "test".to_string();
    
    redis_queue.push(item.clone());
    let result = redis_queue.pop::<String>().unwrap();
}
```

### AsyncRedisQueue - Asynchronous Implementation

```rust
use redis_queue_rs::async_redis_queue::AsyncRedisQueue;
use redis::Client;

#[tokio::main]
async fn main() {
    let redis_client = Client::open("redis://127.0.0.1:6379").unwrap();
    
    let mut redis_queue = AsyncRedisQueue::new(
        "name_of_queue".to_string(),
        redis_client,
    ).await;
    
    let item = "test".to_string();

    redis_queue.push(item.clone()).await;
    let result = redis_queue.pop::<String>().await.unwrap();
}
```

## License

This project is licensed under the MIT License, feel free to use it in your projects :)
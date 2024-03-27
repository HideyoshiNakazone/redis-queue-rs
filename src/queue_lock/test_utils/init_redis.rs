pub fn initialize_redis_client() -> redis::Client {
    redis::Client::open("redis://127.0.0.1:6379").unwrap()
}

pub fn initialize_redis() -> redis::Connection {
    initialize_redis_client().get_connection().unwrap()
}

pub async fn initialize_async_redis() -> redis::aio::MultiplexedConnection {
    initialize_redis_client().get_multiplexed_async_connection().await.unwrap() 
}
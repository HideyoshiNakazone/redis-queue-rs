use std::env;

pub fn initialize_redis_client() -> redis::Client {
    let host = env::var("REDIS_HOST").unwrap_or("127.0.0.1".to_string());
    let port = env::var("REDIS_PORT").unwrap_or("6379".to_string());
    
    redis::Client::open(format!("redis://{}:{}", host, port)).unwrap()
}

pub fn initialize_redis() -> redis::Connection {
    initialize_redis_client().get_connection().unwrap()
}

pub async fn initialize_async_redis() -> redis::aio::MultiplexedConnection {
    initialize_redis_client()
        .get_multiplexed_async_connection()
        .await
        .unwrap()
}

use redis::aio::MultiplexedConnection;

pub async fn initialize_async_redis() -> MultiplexedConnection {
    let client = redis::Client::open("redis://127.0.0.1:6379" ).unwrap();
    
    client.get_multiplexed_async_connection().await.unwrap()
}

pub fn initialize_redis() -> redis::Connection {
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    client.get_connection().unwrap()
}
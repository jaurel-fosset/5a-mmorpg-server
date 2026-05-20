mod heartbeat;
mod scaler;

use redis::aio::MultiplexedConnection;
use std::env;
use std::sync::Arc;

struct AppState {
    redis_connexion: MultiplexedConnection,
}

#[dotenvy::load(path = ".env")]
#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let client = redis::Client::open(env::var("REDIS_URL").unwrap()).unwrap();
    let con = client.get_multiplexed_async_connection().await.unwrap();

    let shared_state = Arc::new(AppState {
        redis_connexion: con,
    });

    let value = shared_state.clone();
    let _t1 = tokio::spawn(async move { heartbeat::listen(value.clone()).await });
    let _t2 = tokio::spawn(async move { scaler::run(shared_state).await });

    tokio::join!(_t1, _t2);
}

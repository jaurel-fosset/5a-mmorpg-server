mod heartbeat;
mod scaler;
mod listener;

use redis::aio::MultiplexedConnection;
use std::env;
use std::sync::Arc;

struct AppState {
    redis_connexion: MultiplexedConnection,
    peer: tokio::sync::Mutex<game_sockets::GamePeer>,
}

#[dotenvy::load(path = ".env", required = false)]
#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let client = redis::Client::open(env::var("REDIS_URL").unwrap()).unwrap();
    let con = client.get_multiplexed_async_connection().await.unwrap();

    let backend = game_sockets::protocols::QuicBackend::new();
    let mut peer = game_sockets::GamePeer::new(backend);

    let port : u16 = env::var("ORCH_PORT").unwrap().parse::<u16>().unwrap();

    peer.listen("0.0.0.0", port).unwrap();

    let shared_state = Arc::new(AppState {
        redis_connexion: con,
        peer: tokio::sync::Mutex::new(peer),
    });

    let value = shared_state.clone();
    //let _t1 = tokio::spawn(async move { heartbeat::listen(value.clone()).await });
    //let _t2 = tokio::spawn(async move { scaler::run(shared_state).await });
    let _t3 = tokio::spawn(async move { listener::listen(shared_state).await });

    //tokio::join!(_t1, _t2);
    tokio::join!(_t3);
}

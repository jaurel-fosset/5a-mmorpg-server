mod routes;

use std::{env, sync::Arc};

use axum::{
    Router,
    response::Html,
    routing::{get, post},
};

use axum::extract::State;
use redis::aio::MultiplexedConnection;
use tokio::sync::Mutex;

struct AppState {
    redis_connexion: MultiplexedConnection,
}

#[dotenvy::load(path = ".env")]
#[tokio::main]
async fn main() {
    println!("REDIS_URL={}", env::var("REDIS_URL").unwrap());

    let client = redis::Client::open(env::var("REDIS_URL").unwrap()).unwrap();
    let con = client.get_multiplexed_async_connection().await.unwrap();

    let shared_state = Arc::new(Mutex::new(AppState {
        redis_connexion: con,
    }));

    let app = Router::new()
        .route("/", get(handler))
        .route("/login", post(routes::login::login))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(State(_state): State<Arc<Mutex<AppState>>>) -> Html<String> {
    Html(format!("<h1>Hello, {}!</h1>", "world"))
}
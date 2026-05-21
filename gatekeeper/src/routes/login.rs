use crate::AppState;
use axum::extract::State;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

// Requête pour récupérer l'ip du GameServer (hardcodé)
//
// curl -s \
//     -w '\n' \
//     -H 'Content-Type: application/json' \
//     -d '{"username":"foo"}' \
//     http://localhost:3000/login

// Commande à réaliser sur Redis pour réaliser le premier test en attendant l'orchestrator
// > HSET gameserver:gs-01 ip "127.0.0.1" port "7001" cpu "51" ram "20" players "64" max_players "120" zone "America/Toronto"
// > HSET gameserver:gs-02 ip "127.0.0.1" port "7002" cpu "80" ram "70" players "110" max_players "120" zone "Europe/Paris"
// > HSET gameserver:gs-03 ip "127.0.0.1" port "7003" cpu "1" ram "12" players "3" max_players "100" zone "America/New_York"
// > ZADD servers:available 10 "gameserver:gs-01"
// > ZADD servers:available 80 "gameserver:gs-02"
// > ZADD servers:available 50 "gameserver:gs-03"

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>, AuthError> {
    if payload.username.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    let mut con = state.redis_connexion.clone();

    let mut found_server = false;
    let mut info_server: Vec<String> = Vec::new();

    while !found_server {
        let game_server: Vec<String>  = redis::cmd("ZRANGE")
            .arg(&["servers:available", "0", "0"])
            .query_async(&mut con)
            .await
            .unwrap();

        if game_server.is_empty() {
            return Err(AuthError::NoServerAvailable);
        }

        info_server = redis::cmd("HGETALL")
            .arg(game_server.clone())
            .query_async(&mut con)
            .await
            .unwrap();

        if info_server.is_empty() {
            redis::cmd("ZREM")
                .arg(&["servers:available", game_server.get(0).unwrap().as_str()])
                .exec_async(&mut con)
                .await
                .unwrap();
        } else {
            found_server = true;
        }
    }

    let mut ip: String = "".to_owned();
    let mut port: u16 = 0;
    let mut zone: String = "".to_owned();

    let number_data = info_server.len() / 2;
    for n in 0..number_data {
        if info_server[n * 2] == "ip" {
            ip = info_server[n * 2 + 1].clone();
        }
        if info_server[n * 2] == "port" {
            port = info_server[n * 2 + 1].parse::<u16>().unwrap();
        }
        if info_server[n * 2] == "zone" {
            zone = info_server[n * 2 + 1].clone();
        }
    }

    Ok(Json(AuthBody::new(ip, port, zone)))
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    username: String,
}
#[derive(Debug, Serialize)]
pub struct AuthBody {
    ip: String,
    port: u16,
    zone: String,
}

impl AuthBody {
    fn new(ip: String, port: u16, zone: String) -> Self {
        Self { ip, port, zone }
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingCredentials,
    NoServerAvailable,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::NoServerAvailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "No server available")
            }
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

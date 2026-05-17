use crate::AppState;
use axum::extract::State;
use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

// Requête pour récupérer l'ip du GameServer (hardcodé)
//
// curl -s \
//     -w '\n' \
//     -H 'Content-Type: application/json' \
//     -d '{"username":"foo"}' \
//     http://localhost:3000/login

pub async fn login(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>, AuthError> {
    if payload.username.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    let mut state = state.lock().await;
    redis::cmd("SET")
        .arg(&["key", "foo"])
        .exec_async(&mut state.redis_connexion)
        .await
        .unwrap();

    let token = "127.0.0.1:7777".to_owned();

    Ok(Json(AuthBody::new(token)))
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    username: String,
}
#[derive(Debug, Serialize)]
pub struct AuthBody {
    server_ip: String,
}

impl AuthBody {
    fn new(server_ip: String) -> Self {
        Self { server_ip }
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingCredentials,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

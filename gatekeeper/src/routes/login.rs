use axum::{
    response::{Response},
    http::StatusCode,
    Json,
    response::IntoResponse
};
use serde::{Deserialize, Serialize};
use serde_json::json;

// Requête pour récupérer l'ip du GameServer (hardcodé)
//
// curl -s \
//     -w '\n' \
//     -H 'Content-Type: application/json' \
//     -d '{"username":"foo"}' \
//     http://localhost:3000/login

pub async fn login(Json(payload): Json<AuthPayload>) -> Result<Json<AuthBody>,AuthError> {
    if payload.username.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

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
        Self {
            server_ip,
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    //WrongCredentials,
    MissingCredentials,
    //TokenCreation,
    //InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            //AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            //AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            //AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
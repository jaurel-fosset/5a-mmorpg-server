use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;
use crate::AppState;

pub async fn health(
    State(state): State<Arc<AppState>>
) -> Result<Json<HealthBody>, HealthError> {
    let mut con = state.redis_connexion.clone();

    let result  = redis::cmd("PING")
        .query_async(&mut con)
        .await;

    assert_eq!(result,Ok("PONG".to_string()));

    println!("{:?}", result);

    Ok(Json(HealthBody::new("ok".to_owned())))
}

#[derive(Debug,Serialize)]
pub struct HealthBody{
    status: String,
}
impl HealthBody{
    fn new(status: String) -> Self { Self { status } }
}

#[derive(Debug)]
pub struct HealthError {}
impl IntoResponse for HealthError {
    fn into_response(self) -> Response {
        (StatusCode::NOT_IMPLEMENTED, Json(json!({"error": "Not implemented yet"}))).into_response()
    }
}

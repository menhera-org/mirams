
use super::build_json_response;
use crate::server::Server;
use crate::store::DbConnection;
use super::ApiResponse;
use super::ApiResponseVariant;

use axum::response::IntoResponse;
use axum::extract::State as StateExtractor;
use axum::extract::Json as JsonExtractor;

use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn api_v1_login<T: DbConnection + Clone + Send + Sync>(StateExtractor(s): StateExtractor<Server<T>>, JsonExtractor(req): JsonExtractor<LoginRequest>) -> impl IntoResponse {
    let store = s.store().clone();

    let token = tokio::task::spawn_blocking(move || {
        if let Ok(true) = store.users().check_password(&req.username, &req.password) {
            store.users().generate_api_key(&req.username).ok()
        } else {
            None
        }
    }).await.unwrap();

    let (status, response) = match token {
        Some(token) => (200, ApiResponse {
            error: None,
            result: Some(ApiResponseVariant::ApiToken(token)),
        }),
        None => (401, ApiResponse {
            error: Some("Unauthorized".to_string()),
            result: None,
        }),
    };

    let response = build_json_response(
        response,
        status,
    );

    response
}

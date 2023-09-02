use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::extract::Extension;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use tower_http::request_id::RequestId;
use super::state::State;

#[derive(Serialize)]
struct PingResponse {
    message: String,
    request_id: String
}


pub async fn ping(
    Extension(_state): Extension<Arc<State>>,
    Extension(_request_id): Extension<RequestId>
) -> impl IntoResponse {
    let response = PingResponse {
        message: String::from("pong!"),
        request_id: _request_id.header_value().to_str().unwrap().to_string(),
    };

    (StatusCode::OK, Json(response))
}


use std::sync::Arc;
use axum::{
    http,
    http::{Request, StatusCode},
    response::Response,
    middleware::Next,
};
use redis::AsyncCommands;
use serde_json;

use super::state::CurrentUser;
use super::state::State;

pub async fn auth<B>(
    mut req: Request<B>,
    next: Next<B>
) -> Result<Response, StatusCode> {
    let _state = req.extensions().get::<Arc<State>>().unwrap();
    let auth_header = req.headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Some(current_user) = get_user_by_token(auth_header, _state).await {
        req.extensions_mut().insert(current_user);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn get_user_by_token(token: &str, _state: &Arc<State>) -> Option<CurrentUser> {
    let mut con = _state.redis_client.get_async_connection().await.unwrap();
    let data: Option<String> = con.get(format!("auth:{}", token))
        .await
        .unwrap_or(None);

    if data.is_none() {
        return None;
    }

    let current_user: Option<CurrentUser> = serde_json::from_str(&data.unwrap()).unwrap_or(None);

    current_user
}

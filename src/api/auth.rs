use std::sync::Arc;
use serde::Serialize;
use serde::Deserialize;
use axum::extract::Extension;
use axum::response::IntoResponse;
use axum::Json;
use axum::http::StatusCode;
use tower_http::request_id::RequestId;
use bson::oid::ObjectId;

use bson::doc;
use super::code;
use super::state::State;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde_json;
use redis::AsyncCommands;


pub fn token_generate() -> String {
    rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(32)
    .map(char::from)
    .collect()
}

#[derive(Debug, Serialize, Deserialize)]
struct Account {
    #[serde(rename(deserialize = "_id", serialize = "id"))]
    id: ObjectId,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignInRequest {
    username: String,
    password: String
}

#[derive(Debug, Serialize, Deserialize)]
struct SignInResponseData {
    token: String
}

#[derive(Debug, Serialize, Deserialize)]
struct SignInResponse {
    request_id: String,
    code: String,
    data: Option<SignInResponseData>
}

pub async fn sign_in(
    Json(payload): Json<SignInRequest>,
    Extension(_state): Extension<Arc<State>>,
    Extension(_request_id): Extension<RequestId>
) -> impl IntoResponse {
    let request_id = _request_id.header_value().to_str().unwrap().to_string();

    let collection = _state.db.collection("account");
    
    let filter = doc! {
        "username": payload.username.trim()
    };

    let result: Option<Account> = collection.find_one(filter, None).await;
    
    if result.is_none() {
        let response = SignInResponse {
            request_id, 
            code: code::INVALID_CREDENTIALS.to_owned(),
            data: None
        };
    
        return (StatusCode::BAD_REQUEST, Json(response))
    }

    let account = result.unwrap();
    if account.password != payload.password {
        let response = SignInResponse {
            request_id, 
            code: code::INVALID_CREDENTIALS.to_owned(),
            data: None
        };

        return (StatusCode::BAD_REQUEST, Json(response))
    }

    let token = token_generate();
    let key = format!("auth:{}", token);
    let value = serde_json::to_string(&account).unwrap();

    let mut con = _state.redis_client.get_async_connection().await.unwrap();
    let _: () = con.set(key, value).await.unwrap();

    let response = SignInResponse {
        request_id, 
        code: code::SUCCESSED.to_owned(),
        data: Some(SignInResponseData {
            token,
        })
    };

    (StatusCode::OK, Json(response))
}

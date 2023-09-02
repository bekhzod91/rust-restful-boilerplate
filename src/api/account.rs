use std::str::FromStr;
use std::sync::Arc;
use serde::Serialize;
use serde::Deserialize;
use axum::extract::{Path, Extension};
use axum::response::IntoResponse;
use axum::Json;
use axum::http::StatusCode;
use tower_http::request_id::RequestId;
use futures::TryStreamExt;
use bson::oid::ObjectId;
use bson::doc;
use super::code;
use super::state::State;


#[derive(Debug, Serialize, Deserialize)]
struct Account {
    #[serde(rename(deserialize = "_id"))]
    id: ObjectId,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountListResult {
    id: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountListResponseData {
    count: usize,
    results: Vec<AccountListResult>
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountListResponse {
    request_id: String,
    code: String,
    data: AccountListResponseData
}


pub async fn account_list(
    Extension(_state): Extension<Arc<State>>,
    Extension(_request_id): Extension<RequestId>
) -> impl IntoResponse {
    let request_id = _request_id.header_value().to_str().unwrap().to_string();
    let mut results = vec![];

    let collection = _state.db.collection::<Account>("account");

    let count = collection.count_documents(None, None).await.unwrap();
    let mut cursor = collection.find(None, None).await.unwrap();

    while let Some(account) = cursor.try_next().await.unwrap() {
       results.push(AccountListResult {
            id: account.id.to_string(),
            username: account.username,
            password: account.password,
       })
    }

    let response = AccountListResponse {
        request_id, 
        code: code::SUCCESSED.to_owned(),
        data: AccountListResponseData {
            count: count.try_into().unwrap(),
            results
        }
    };

    (StatusCode::OK, Json(response))
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountDetailData {
    id: String,
    username: String,
    password: String
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountDetailResponse {
    request_id: String,
    code: String,
    data: Option<AccountDetailData>
}

pub async fn account_detail(
    Path(_id): Path<String>,
    Extension(_state): Extension<Arc<State>>,
    Extension(_request_id): Extension<RequestId>
) -> impl IntoResponse {
    let request_id = _request_id.header_value().to_str().unwrap().to_string();
    let collection = _state.db.collection("account");
    
    let filter = doc! {
        "_id": ObjectId::from_str(&_id).unwrap_or(ObjectId::default())
    };

    let result: Option<Account> = collection.find_one(filter, None).await.unwrap();

    match result {
        Some(account) => {
            let response = AccountDetailResponse {
                request_id,
                code: code::SUCCESSED.to_owned(),
                data: Some(AccountDetailData {
                    id: account.id.to_string(),
                    username: account.username,
                    password: account.password
                })
            };

            (StatusCode::OK, Json(response))
        },
        None => {
            let response = AccountDetailResponse {
                request_id,
                code: code::NOT_FOUND.to_owned(),
                data: None
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AccountCreateRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountCreateDataResponse {
    id: String
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountCreateResponse {
    request_id: String,
    code: String,
    data: AccountCreateDataResponse
}

pub async fn account_create(
    Json(payload): Json<AccountCreateRequest>,
    Extension(_state): Extension<Arc<State>>,
    Extension(_request_id): Extension<RequestId>
) -> impl IntoResponse {
    let request_id = _request_id.header_value().to_str().unwrap().to_string();
    let collection = _state.db.collection("account");
    let doc = doc! {
        "username": payload.username,
        "password": payload.password
    };
    let account = collection.insert_one(doc, None).await.unwrap();

    let response = AccountCreateResponse {
        request_id,
        code: code::SUCCESSED.to_owned(),
        data: AccountCreateDataResponse {
            id: account.inserted_id.as_object_id().unwrap().to_string()
        }
    };

    (StatusCode::CREATED, Json(response))
}


#[derive(Debug, Serialize, Deserialize)]
struct AccountRemoveResponse {
    request_id: String,
    code: String,
}

pub async fn account_remove(
    Path(_id): Path<String>,
    Extension(_state): Extension<Arc<State>>,
    Extension(_request_id): Extension<RequestId>
) -> impl IntoResponse {
    let request_id = _request_id.header_value().to_str().unwrap().to_string();

    let filter = doc! {
        "_id": ObjectId::from_str(&_id).unwrap_or(ObjectId::default())
    };
    let collection = _state.db.collection::<Account>("account");
    let result = collection.delete_one(filter, None).await.unwrap();

    if result.deleted_count != 0 {
        let response = AccountRemoveResponse {
            request_id,
            code: code:: NOT_FOUND.to_owned()
        };
        
        return (StatusCode::NOT_FOUND, Json(response))
    }

    let response = AccountRemoveResponse {
        request_id,
        code: code::SUCCESSED.to_owned()
    };

    (StatusCode::OK, Json(response))
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AccountUpdateRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountUpdateDataResponse {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountUpdateResponse {
    request_id: String,
    code: String,
    data: Option<AccountUpdateDataResponse>
}

pub async fn account_update(
    Path(_id): Path<String>,
    Json(payload): Json<AccountUpdateRequest>,
    Extension(_state): Extension<Arc<State>>,
    Extension(_request_id): Extension<RequestId>
) -> impl IntoResponse {
    let request_id = _request_id.header_value().to_str().unwrap().to_string();
    let collection = _state.db.collection::<Account>("account");

    let filter = doc! {
        "_id": ObjectId::from_str(&_id).unwrap_or(ObjectId::default())
    };
    let doc = doc! {
        "$set": {
            "username": payload.username,
            "password": payload.password
        }
    };
    let result = collection.update_one(filter, doc, None).await.unwrap();

    if result.matched_count == 0 {
        let response = AccountUpdateResponse {
            request_id,
            code: code::NOT_FOUND.to_owned(),
            data: None
        };
        return (StatusCode::NOT_FOUND, Json(response))
    }

    let response = AccountUpdateResponse {
        request_id,
        code: code::SUCCESSED.to_owned(),
        data: Some(AccountUpdateDataResponse {
            id: _id,
        })
    };

    (StatusCode::OK, Json(response))
}
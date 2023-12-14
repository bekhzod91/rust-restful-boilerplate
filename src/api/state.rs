use serde::Serialize;
use serde::Deserialize;
use mongodb::Database;
use bson::oid::ObjectId;
extern crate redis;

#[derive(Debug)]
pub struct State {
    pub db: Database,
    pub redis_client: redis::Client
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentUser {
    #[serde(rename(deserialize = "id"))]
    pub id: ObjectId,
    pub username: String, 
    pub password: String
}
use serde::{Deserialize, Serialize};
use async_graphql::SimpleObject;
use mongodb::bson::oid::ObjectId;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Player {
    pub username: String,
    pub character: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Lobby {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub code: String,
    pub players: Vec<Player>,
    pub state: String, // "waiting", "playing"
    pub created_at: DateTime<Utc>,
}

use mongodb::{Client, Database, Collection};
use mongodb::bson::doc;
use std::env;
use crate::model::{User, Lobby};

#[derive(Clone)]
pub struct DB {
    pub client: Client,
    pub db: Database,
}

impl DB {
    pub async fn init() -> Self {
        let uri = env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let client = Client::with_uri_str(&uri).await.expect("Failed to create MongoDB client");
        let db = client.database("monopoly_world");
        DB { client, db }
    }

    pub fn users(&self) -> Collection<User> {
        self.db.collection("users")
    }

    pub fn lobbies(&self) -> Collection<Lobby> {
        self.db.collection("lobbies")
    }

    pub async fn find_user_by_username(&self, username: &str) -> mongodb::error::Result<Option<User>> {
        self.users().find_one(doc! { "username": username }, None).await
    }
}

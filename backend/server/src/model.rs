use serde::{Deserialize, Serialize};
use async_graphql::SimpleObject;
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub password_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Player {
    pub username: String,
    pub character: Option<String>,
    // Game state fields
    #[serde(default)]
    pub position: u8,
    #[serde(default)]
    pub in_jail: bool,
    #[serde(default)]
    pub consecutive_doubles: u8,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            username: String::new(),
            character: None,
            position: 0,
            in_jail: false,
            consecutive_doubles: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct GameState {
    #[serde(default)]
    pub current_turn_index: u8,
    #[serde(default)]
    pub last_die1: u8,
    #[serde(default)]
    pub last_die2: u8,
    #[serde(default)]
    pub awaiting_action: bool, // true if player needs to make a choice (e.g., forced deal)
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            current_turn_index: 0,
            last_die1: 1,
            last_die2: 1,
            awaiting_action: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Lobby {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub code: String,
    pub players: Vec<Player>,
    #[serde(default)]
    pub host: String,
    pub state: String, // "waiting", "playing"
    pub created_at: String,
    #[serde(default)]
    pub game_state: Option<GameState>,
}


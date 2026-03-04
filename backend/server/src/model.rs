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
pub struct PropertyInfo {
    pub name: String,
    pub color: String, // "gray" for First Class, or hex/color name for destinations
    pub diameter: f32,
    pub column: String, // "left" or "right"
    pub destination_id: Option<u8>,
    pub x: f32,   // horizontal position in pixels (within column)
    pub y: f32,   // vertical position in pixels (from bottom of column)
    pub size: u32, // diameter in pixels
    pub price: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct GqlHereAndNowCard {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct GqlChanceCard {
    pub id: String,
    pub description: String,
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
    #[serde(default)]
    pub money: i32,
    #[serde(default)]
    pub properties: Vec<PropertyInfo>,
    #[serde(default)]
    pub here_and_now_cards: Vec<GqlHereAndNowCard>,
    #[serde(default)]
    pub chance_cards: Vec<GqlChanceCard>,
    #[serde(default)]
    pub can_use_say_no: bool,
    #[serde(default)]
    pub can_use_discount: bool,
    #[serde(default)]
    pub can_use_intercept: bool,
    #[serde(default)]
    pub can_use_collect_tax: bool,
    #[serde(default)]
    pub can_use_steal_first_class: bool,
    #[serde(default)]
    pub skip_next_turn: bool,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            username: String::new(),
            character: None,
            position: 0,
            in_jail: false,
            consecutive_doubles: 0,
            money: 1500,
            properties: Vec::new(),
            here_and_now_cards: Vec::new(),
            chance_cards: Vec::new(),
            can_use_say_no: false,
            can_use_discount: false,
            can_use_intercept: false,
            can_use_collect_tax: false,
            can_use_steal_first_class: false,
            skip_next_turn: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct Lobby {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub code: String,
    pub players: Vec<Player>,
    #[serde(default)]
    pub host: String,
    pub state: String, // "waiting", "playing"
    pub created_at: String,
    #[serde(default, rename = "game")]
    #[graphql(skip)]
    pub game: Option<game_engine::game::Game>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: String,
    pub lobby_code: String,
}

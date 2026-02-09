use async_graphql::{ComplexObject, Context, Object, Schema, EmptySubscription, Result, Error, SimpleObject};
use crate::database::DB;
use crate::model::{User, Lobby, Player};
use mongodb::bson::doc;
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use game_engine::game::{Game, GameStep};

// --- GraphQL Objects for Display ---

#[derive(SimpleObject)]
pub struct PendingPurchase {
    pub dest_id: u8,
    pub dest_name: String,
    pub price: u32,
}

#[derive(SimpleObject)]
pub struct GameStateDisplay {
    pub current_turn_index: u8,
    pub last_die1: u8,
    pub last_die2: u8,
    pub awaiting_action: bool,
    pub pending_purchase: Option<PendingPurchase>,
    pub pending_first_class: bool,
}

#[ComplexObject]
impl Lobby {
    async fn game_state(&self) -> Option<GameStateDisplay> {
        if let Some(game) = &self.game {
            let (d1, d2) = game.last_dice.unwrap_or((1, 1));
            
            // Check for pending purchase
            let pending_purchase = match &game.step {
                GameStep::WaitingForPurchaseDecision { dest_id, price } => {
                    // Find destination name from board
                    let name = game.board.find_destination_by_id(*dest_id)
                        .map(|d| d.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    Some(PendingPurchase {
                        dest_id: *dest_id,
                        dest_name: name,
                        price: *price,
                    })
                },
                _ => None,
            };
            
            let pending_first_class = game.step == GameStep::WaitingForFirstClassDecision;
            
            Some(GameStateDisplay {
                current_turn_index: game.current_player_idx as u8,
                last_die1: d1,
                last_die2: d2,
                awaiting_action: game.step == GameStep::WaitingForForcedDeal,
                pending_purchase,
                pending_first_class,
            })
        } else {
            None
        }
    }
}

fn sync_lobby_state(players: &mut Vec<Player>, game: &Game) {
    for server_player in players {
        if let Some(engine_player) = game.players.iter().find(|p| p.name == server_player.username) {
            server_player.position = engine_player.position as u8;
            server_player.in_jail = engine_player.in_jail;
            server_player.money = engine_player.money;
            server_player.properties = engine_player.passport.all_stamps().iter()
                .map(|s| {
                    let color = if let Some(id) = s.destination_id {
                        match game.board.find_destination_by_id(id).map(|d| d.color) {
                            Some(game_engine::board::Color::Brown) => "brown",
                            Some(game_engine::board::Color::LightBlue) => "lightblue",
                            Some(game_engine::board::Color::Pink) => "pink",
                            Some(game_engine::board::Color::Orange) => "orange",
                            Some(game_engine::board::Color::Red) => "red",
                            Some(game_engine::board::Color::Yellow) => "yellow",
                            Some(game_engine::board::Color::Green) => "green",
                            Some(game_engine::board::Color::DarkBlue) => "darkblue",
                            None => "gray",
                        }.to_string()
                    } else {
                        "gray".to_string()
                    };
                    crate::model::PropertyInfo {
                        name: s.name.clone(),
                        color,
                    }
                })
                .collect();
        }
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> String {
        "Hello from Monopoly backend (Game Engine Integrated)!".to_string()
    }

    async fn get_lobby(&self, ctx: &Context<'_>, code: String) -> Result<Option<Lobby>> {
        let db = ctx.data::<DB>()?;
        let lobby = db.lobbies().find_one(doc! { "code": code }, None).await?;
        Ok(lobby)
    }
    
    async fn me(&self, ctx: &Context<'_>, username: String) -> Result<Option<User>> {
         let db = ctx.data::<DB>()?;
         Ok(db.find_user_by_username(&username).await?)
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn register(&self, ctx: &Context<'_>, username: String, password: String) -> Result<User> {
        let db = ctx.data::<DB>()?;
        
        if let Some(_) = db.find_user_by_username(&username).await? {
            return Err(Error::new("Username already exists"));
        }

        let new_user = User {
            id: None,
            username: username.clone(),
            password_hash: password, // TODO: Hash this
            created_at: Utc::now().to_rfc3339(),
        };

        // Note: MongoDB driver 2.8.2 uses insert_one(doc, options)
        let result = db.users().insert_one(new_user.clone(), None).await?;
        
        let mut user = new_user;
        user.id = Some(result.inserted_id.as_object_id().ok_or(Error::new("Failed to get ID"))?);
        
        Ok(user)
    }

    async fn login(&self, ctx: &Context<'_>, username: String, password: String) -> Result<User> {
        let db = ctx.data::<DB>()?;
        let user_opt = db.find_user_by_username(&username).await?;

        match user_opt {
            Some(mut user) => {
                if user.password_hash != password {
                    return Err(Error::new("Invalid password"));
                }
                
                Ok(user)
            },
            None => Err(Error::new("User not found"))
        }
    }

    async fn create_lobby(&self, ctx: &Context<'_>, username: String) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        
        let code = generate_code();

        let new_lobby = Lobby {
            id: None,
            code: code.clone(),
            players: vec![Player { 
                username: username.clone(), 
                character: None,
                position: 0,
                in_jail: false,
                consecutive_doubles: 0,
                money: 1500,
                properties: Vec::new(),
            }],
            host: username,
            state: "waiting".to_string(),
            created_at: Utc::now().to_rfc3339(),
            game: None,
        };

        let result = db.lobbies().insert_one(new_lobby.clone(), None).await?;
        let mut lobby = new_lobby;
        lobby.id = Some(result.inserted_id.as_object_id().ok_or(Error::new("Failed to get ID"))?);

        Ok(lobby)
    }
    
    async fn join_lobby(&self, ctx: &Context<'_>, code: String, username: String) -> Result<Lobby> {
         let db = ctx.data::<DB>()?;
         let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;
         
         if let Some(mut lobby) = lobby_opt {
             if lobby.players.len() >= 4 {
                 return Err(Error::new("Lobby is full"));
             }
             if lobby.players.iter().any(|p| p.username == username) {
                 return Ok(lobby);
             }
             if lobby.state != "waiting" {
                 return Err(Error::new("Game already started"));
             }

             let new_player = Player { 
                 username: username.clone(), 
                 character: None,
                 position: 0,
                 in_jail: false,
                 consecutive_doubles: 0,
                 money: 1500,
                 properties: Vec::new(),
             };
             lobby.players.push(new_player.clone());
             
             db.lobbies().update_one(
                 doc! { "_id": lobby.id },
                 doc! { "$push": { "players": mongodb::bson::to_bson(&new_player).unwrap() } },
                 None
             ).await?;
             
             Ok(lobby)
         } else {
             Err(Error::new("Lobby not found"))
         }
    }
    
    async fn select_character(&self, ctx: &Context<'_>, code: String, username: String, character: String) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
             if lobby.players.iter().any(|p| p.character.as_ref() == Some(&character) && p.username != username) {
                return Err(Error::new("Character already taken"));
            }

            let mut found = false;
            for p in &mut lobby.players {
                if p.username == username {
                    p.character = Some(character.clone());
                    found = true;
                    break;
                }
            }
            
            if !found {
                 return Err(Error::new("Player not in lobby"));
            }

            db.lobbies().update_one(
                doc! { "_id": lobby.id, "players.username": username },
                doc! { "$set": { "players.$.character": character } },
                None
            ).await?;

            Ok(lobby)
        } else {
            Err(Error::new("Lobby not found"))
        }
    }

    async fn start_game(&self, ctx: &Context<'_>, code: String, username: String) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            if lobby.host != username {
                return Err(Error::new("Only host can start the game"));
            }

            if lobby.players.len() < 2 {
                return Err(Error::new("Need at least 2 players to start"));
            }
            
            lobby.state = "playing".to_string();
            
            // Initialize Game Engine
            let player_names: Vec<String> = lobby.players.iter().map(|p| p.username.clone()).collect();
            let mut game = Game::new(player_names);
            
            // Randomize starting player
            let starting_player = rand::thread_rng().gen_range(0..lobby.players.len());
            game.current_player_idx = starting_player;
            
            lobby.game = Some(game);

            // Save
            db.lobbies().update_one(
                doc! { "_id": lobby.id },
                doc! { 
                    "$set": { 
                        "state": "playing",
                        "game": mongodb::bson::to_bson(&lobby.game).unwrap()
                    } 
                },
                None
            ).await?;

            Ok(lobby)
        } else {
             Err(Error::new("Lobby not found"))
        }
    }

    /// Roll dice using Game Engine
    async fn roll_dice(&self, ctx: &Context<'_>, code: String, username: String) -> Result<RollResult> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            if lobby.state != "playing" {
                return Err(Error::new("Game not started"));
            }

            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;
            
            // Validate turn
            let current_idx = game.current_player_idx;
            if current_idx >= game.players.len() {
                return Err(Error::new("Invalid game state index"));
            }
            if game.players[current_idx].name != username {
                return Err(Error::new("Not your turn"));
            }

            // Execute Roll
            let result = game.roll_dice()
                .map_err(|e| Error::new(e))?;

            // Sync State
            sync_lobby_state(&mut lobby.players, game);

            // Save to DB
            db.lobbies().update_one(
                doc! { "_id": lobby.id },
                doc! { 
                    "$set": { 
                        "players": mongodb::bson::to_bson(&lobby.players).unwrap(),
                        "game": mongodb::bson::to_bson(&lobby.game).unwrap()
                    } 
                },
                None
            ).await?;

            // Map Result
            Ok(RollResult {
                die1: result.die1,
                die2: result.die2,
                is_double: result.is_double,
                is_forced_deal: result.is_forced_deal,
                new_position: result.new_position,
                went_to_jail: result.went_to_jail,
                turn_ends: result.turn_ends,
                current_turn_index: result.current_player_index,
            })
        } else {
            Err(Error::new("Lobby not found"))
        }
    }

    /// Resolve forced deal using Game Engine
    async fn resolve_forced_deal(&self, ctx: &Context<'_>, code: String, username: String, action: String) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;

            // Validate turn
            let current_idx = game.current_player_idx;
             if game.players[current_idx].name != username {
                return Err(Error::new("Not your turn"));
            }

            // Execute Action
            game.resolve_forced_deal(&action)
                .map_err(|e| Error::new(e))?;

            // Sync State
            sync_lobby_state(&mut lobby.players, game);

            // Save
            db.lobbies().update_one(
                doc! { "_id": lobby.id },
                doc! { 
                    "$set": { 
                        "players": mongodb::bson::to_bson(&lobby.players).unwrap(),
                        "game": mongodb::bson::to_bson(&lobby.game).unwrap()
                    } 
                },
                None
            ).await?;

            Ok(lobby)
        } else {
            Err(Error::new("Lobby not found"))
        }
    }

    /// Resolve property purchase decision
    async fn resolve_purchase(&self, ctx: &Context<'_>, code: String, username: String, buy: bool) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;

            // Validate turn
            let current_idx = game.current_player_idx;
            if game.players[current_idx].name != username {
                return Err(Error::new("Not your turn"));
            }

            // Execute Action
            game.resolve_purchase(buy)
                .map_err(|e| Error::new(e))?;

            // Sync State
            sync_lobby_state(&mut lobby.players, game);

            // Save
            db.lobbies().update_one(
                doc! { "_id": lobby.id },
                doc! { 
                    "$set": { 
                        "players": mongodb::bson::to_bson(&lobby.players).unwrap(),
                        "game": mongodb::bson::to_bson(&lobby.game).unwrap()
                    } 
                },
                None
            ).await?;

            Ok(lobby)
        } else {
            Err(Error::new("Lobby not found"))
        }
    }

    /// Resolve First Class stamp purchase decision
    async fn resolve_first_class(&self, ctx: &Context<'_>, code: String, username: String, buy: bool) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;

            // Validate turn
            let current_idx = game.current_player_idx;
            if game.players[current_idx].name != username {
                return Err(Error::new("Not your turn"));
            }

            // Execute Action
            game.resolve_first_class(buy)
                .map_err(|e| Error::new(e))?;

            // Sync State
            sync_lobby_state(&mut lobby.players, game);

            // Save
            db.lobbies().update_one(
                doc! { "_id": lobby.id },
                doc! { 
                    "$set": { 
                        "players": mongodb::bson::to_bson(&lobby.players).unwrap(),
                        "game": mongodb::bson::to_bson(&lobby.game).unwrap()
                    } 
                },
                None
            ).await?;

            Ok(lobby)
        } else {
            Err(Error::new("Lobby not found"))
        }
    }
}

/// Result of a dice roll
#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct RollResult {
    pub die1: u8,
    pub die2: u8,
    pub is_double: bool,
    pub is_forced_deal: bool,
    pub new_position: u8,
    pub went_to_jail: bool,
    pub turn_ends: bool,
    pub current_turn_index: u8,
}

pub type MonopolySchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

fn generate_code() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>()
        .to_uppercase()
}


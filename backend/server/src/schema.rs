use async_graphql::{Context, Object, Schema, EmptySubscription, Result, Error};
use crate::database::DB;
use crate::model::{User, Lobby, Player, GameState};
use mongodb::bson::doc;
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> String {
        "Hello from Monopoly backend!".to_string()
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
            }],
            host: username,
            state: "waiting".to_string(),
            created_at: Utc::now().to_rfc3339(),
            game_state: None,
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
            
            // Initialize game state with random starting player
            let starting_player = rand::thread_rng().gen_range(0..lobby.players.len()) as u8;
            let game_state = GameState {
                current_turn_index: starting_player,
                last_die1: 1,
                last_die2: 1,
                awaiting_action: false,
            };
            lobby.game_state = Some(game_state.clone());

            db.lobbies().update_one(
                doc! { "_id": lobby.id },
                doc! { 
                    "$set": { 
                        "state": "playing",
                        "game_state": mongodb::bson::to_bson(&game_state).unwrap()
                    } 
                },
                None
            ).await?;

            Ok(lobby)
        } else {
             Err(Error::new("Lobby not found"))
        }
    }

    /// Roll dice for the current player
    /// Returns the dice values, new position, and whether it's a double/forced deal
    async fn roll_dice(&self, ctx: &Context<'_>, code: String, username: String) -> Result<RollResult> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            if lobby.state != "playing" {
                return Err(Error::new("Game not started"));
            }

            let game_state = lobby.game_state.as_mut().ok_or(Error::new("No game state"))?;
            
            // Check if it's this player's turn
            let current_player_idx = game_state.current_turn_index as usize;
            if current_player_idx >= lobby.players.len() {
                return Err(Error::new("Invalid turn index"));
            }
            
            let current_player = &lobby.players[current_player_idx];
            if current_player.username != username {
                return Err(Error::new("Not your turn"));
            }

            // Check if awaiting action (forced deal choice)
            if game_state.awaiting_action {
                return Err(Error::new("You must resolve the forced deal first"));
            }

            // Roll the dice
            let die1: u8 = rand::thread_rng().gen_range(1..=6);
            let die2: u8 = rand::thread_rng().gen_range(1..=6);
            let is_double = die1 == die2;
            let is_forced_deal = die1 == 1; // Forced deal when die1 shows 1
            let total = die1 + die2;

            // Get mutable reference to player
            let player = &mut lobby.players[current_player_idx];
            
            let mut went_to_jail = false;
            let old_position = player.position;
            let mut new_position = old_position;
            let mut turn_ends = true;

            if is_double {
                player.consecutive_doubles += 1;
                
                // 3rd consecutive double = go to jail
                if player.consecutive_doubles >= 3 {
                    player.position = 10;
                    player.in_jail = true;
                    player.consecutive_doubles = 0;
                    went_to_jail = true;
                    new_position = 10;
                } else if !is_forced_deal {
                    // Move normally, but player gets another turn
                    new_position = (old_position + total) % 40;
                    player.position = new_position;
                    turn_ends = false; // Player rolls again
                }
            } else {
                // Reset consecutive doubles on non-double
                player.consecutive_doubles = 0;
                
                if !is_forced_deal {
                    new_position = (old_position + total) % 40;
                    player.position = new_position;
                }
            }

            // If forced deal, set awaiting_action (player must choose)
            if is_forced_deal && !went_to_jail {
                game_state.awaiting_action = true;
                turn_ends = false; // Wait for player's choice
            }

            // Update last dice
            game_state.last_die1 = die1;
            game_state.last_die2 = die2;

            // If turn ends and not awaiting action, advance to next player
            if turn_ends && !game_state.awaiting_action {
                game_state.current_turn_index = ((current_player_idx + 1) % lobby.players.len()) as u8;
                // Reset doubles counter for next player
                lobby.players[game_state.current_turn_index as usize].consecutive_doubles = 0;
            }

            // Save to database
            db.lobbies().update_one(
                doc! { "_id": lobby.id },
                doc! { 
                    "$set": { 
                        "players": mongodb::bson::to_bson(&lobby.players).unwrap(),
                        "game_state": mongodb::bson::to_bson(&game_state).unwrap()
                    } 
                },
                None
            ).await?;

            Ok(RollResult {
                die1,
                die2,
                is_double,
                is_forced_deal,
                new_position,
                went_to_jail,
                turn_ends: turn_ends && !game_state.awaiting_action,
                current_turn_index: game_state.current_turn_index,
            })
        } else {
            Err(Error::new("Lobby not found"))
        }
    }

    /// Resolve forced deal choice: "sneaky_swap" or "move"
    async fn resolve_forced_deal(&self, ctx: &Context<'_>, code: String, username: String, action: String) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game_state = lobby.game_state.as_mut().ok_or(Error::new("No game state"))?;
            
            // Check if awaiting action
            if !game_state.awaiting_action {
                return Err(Error::new("No pending forced deal"));
            }

            let current_player_idx = game_state.current_turn_index as usize;
            let current_player = &lobby.players[current_player_idx];
            if current_player.username != username {
                return Err(Error::new("Not your turn"));
            }

            let die2 = game_state.last_die2;
            let player = &mut lobby.players[current_player_idx];
            
            match action.as_str() {
                "sneaky_swap" => {
                    // TODO: Implement sneaky swap logic
                    // For now, just skip turn
                }
                "move" => {
                    // Move by die2 spaces
                    player.position = (player.position + die2) % 40;
                }
                _ => return Err(Error::new("Invalid action. Use 'sneaky_swap' or 'move'")),
            }

            // Clear awaiting action and advance turn
            game_state.awaiting_action = false;
            game_state.current_turn_index = ((current_player_idx + 1) % lobby.players.len()) as u8;

            // Save to database
            db.lobbies().update_one(
                doc! { "_id": lobby.id },
                doc! { 
                    "$set": { 
                        "players": mongodb::bson::to_bson(&lobby.players).unwrap(),
                        "game_state": mongodb::bson::to_bson(&game_state).unwrap()
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


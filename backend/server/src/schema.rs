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
    pub price: i32,
    pub buyer_idx: usize,
}

#[derive(SimpleObject)]
pub struct PendingDecision {
    pub buyer_idx: usize,
}

#[derive(SimpleObject)]
pub struct TargetSelectionData {
    pub action: String,
    pub card_id: Option<String>,
    pub selector_idx: usize,
}

#[derive(SimpleObject)]
pub struct DiceDuelData {
    pub challenger_idx: usize,
    pub target_idx: usize,
    pub challenger_die1: Option<u8>,
    pub challenger_die2: Option<u8>,
    pub target_die1: Option<u8>,
    pub target_die2: Option<u8>,
}

#[derive(SimpleObject)]
pub struct AuctionData {
    pub dest_id: u8,
    pub dest_name: String,
    pub current_bid: i32,
    pub highest_bidder_idx: Option<usize>,
}

#[derive(SimpleObject)]
pub struct ActivityLogEntryDisplay {
    pub player_idx: Option<u8>,
    pub message: String,
}

#[derive(SimpleObject)]
pub struct GameStateDisplay {
    pub current_turn_index: u8,
    pub last_die1: u8,
    pub last_die2: u8,
    pub awaiting_action: bool,
    pub pending_purchase: Option<PendingPurchase>,
    pub pending_first_class: Option<PendingDecision>,
    pub pending_airport_decision: Option<PendingDecision>,
    pub pending_airport_destination: Option<PendingDecision>,
    pub is_forced_deal: bool,
    pub is_game_over: bool,
    pub winner_name: Option<String>,
    pub target_selection: Option<TargetSelectionData>,
    pub dice_duel: Option<DiceDuelData>,
    pub pending_auction: Option<AuctionData>,
    pub is_jail_decision: bool,
    pub activity_log: Vec<ActivityLogEntryDisplay>,
}

#[ComplexObject]
impl Lobby {
    async fn game_state(&self) -> Option<GameStateDisplay> {
        if let Some(game) = &self.game {
            let (d1, d2) = game.last_dice.unwrap_or((1, 1));
            
            // Check for pending purchase
            let pending_purchase = match &game.step {
                GameStep::WaitingForPurchaseDecision { dest_id, price, buyer_idx } => {
                    // Find destination name from board
                    let name = game.board.find_destination_by_id(*dest_id)
                        .map(|d| d.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    Some(PendingPurchase {
                        dest_id: *dest_id,
                        dest_name: name,
                        price: *price,
                        buyer_idx: *buyer_idx,
                    })
                },
                _ => None,
            };

            let target_selection = match &game.step {
                GameStep::WaitingForTargetSelection { action, card_id, selector_idx } => {
                    Some(TargetSelectionData {
                        action: action.clone(),
                        card_id: card_id.clone(),
                        selector_idx: *selector_idx,
                    })
                },
                _ => None,
            };

            let dice_duel = match &game.step {
                GameStep::WaitingForDiceDuel { challenger_idx, target_idx, challenger_roll, target_roll } => {
                    Some(DiceDuelData {
                        challenger_idx: *challenger_idx,
                        target_idx: *target_idx,
                        challenger_die1: challenger_roll.map(|r| r.0),
                        challenger_die2: challenger_roll.map(|r| r.1),
                        target_die1: target_roll.map(|r| r.0),
                        target_die2: target_roll.map(|r| r.1),
                    })
                },
                _ => None,
            };
            
            let pending_first_class = match &game.step {
                GameStep::WaitingForFirstClassDecision { buyer_idx } => Some(PendingDecision { buyer_idx: *buyer_idx }),
                _ => None,
            };
            let pending_airport_decision = match &game.step {
                GameStep::WaitingForAirportDecision { buyer_idx } => Some(PendingDecision { buyer_idx: *buyer_idx }),
                _ => None,
            };
            let pending_airport_destination = match &game.step {
                GameStep::WaitingForAirportDestination { buyer_idx } => Some(PendingDecision { buyer_idx: *buyer_idx }),
                _ => None,
            };
            
            let mut winner_name = None;
            if game.game_over {
                // Find winner (player with a full passport)
                winner_name = game.players.iter()
                    .find(|p| p.passport.is_full())
                    .map(|p| p.name.clone());
            }

            let pending_auction = match &game.step {
                GameStep::WaitingForAuction { dest_id, current_bid, highest_bidder } => {
                    let name = game.board.find_destination_by_id(*dest_id)
                        .map(|d| d.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    Some(AuctionData {
                        dest_id: *dest_id,
                        dest_name: name,
                        current_bid: *current_bid,
                        highest_bidder_idx: *highest_bidder,
                    })
                },
                _ => None,
            };

            Some(GameStateDisplay {
                current_turn_index: game.current_player_idx as u8,
                last_die1: d1,
                last_die2: d2,
                awaiting_action: match &game.step {
                    GameStep::WaitingForRoll => false,
                    _ => true,
                },
                pending_purchase,
                pending_first_class,
                pending_airport_decision,
                pending_airport_destination,
                is_forced_deal: game.step == GameStep::WaitingForForcedDeal,
                is_game_over: game.game_over,
                winner_name,
                target_selection,
                dice_duel,
                pending_auction,
                is_jail_decision: game.step == GameStep::WaitingForJailDecision,
                activity_log: game.activity_log.iter().rev().take(5).map(|e| ActivityLogEntryDisplay {
                    player_idx: e.player_idx.map(|i| i as u8),
                    message: e.message.clone(),
                }).collect(),
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
            let mut properties = Vec::new();

            // Populate from left column
            for s in &engine_player.passport.left_column {
                properties.push(map_stamp_to_info(s, game, "left"));
            }

            // Populate from right column
            for s in &engine_player.passport.right_column {
                properties.push(map_stamp_to_info(s, game, "right"));
            }

            server_player.properties = properties;

            // Sync Here & Now cards
            server_player.here_and_now_cards = engine_player.here_and_now_cards.iter().map(|c| crate::model::GqlHereAndNowCard {
                id: c.id.clone(),
                description: c.description.clone(),
            }).collect();

            // Sync Chance cards
            server_player.chance_cards = engine_player.chance_cards.iter().map(|c| crate::model::GqlChanceCard {
                id: c.id.clone(),
                description: c.description.clone(),
            }).collect();

            // Calculate reactive cards availability
            let engine_player_idx = game.players.iter().position(|p| p.name == engine_player.name).unwrap_or(0);
            server_player.can_use_say_no = game.check_can_player_say_no(engine_player_idx);
            server_player.can_use_discount = game.check_can_player_use_discount(engine_player_idx);
            server_player.can_use_intercept = game.check_can_player_use_intercept(engine_player_idx);
            server_player.can_use_collect_tax = game.check_can_player_use_collect_tax(engine_player_idx);
            server_player.can_use_steal_first_class = game.check_can_player_use_steal_first_class(engine_player_idx);
        }
    }
}

fn map_stamp_to_info(s: &game_engine::passport::Stamp, game: &game_engine::game::Game, column: &str) -> crate::model::PropertyInfo {
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
            None => "grey",
        }.to_string()
    } else {
        "grey".to_string()
    };

    crate::model::PropertyInfo {
        name: s.name.clone(),
        color,
        diameter: s.diameter,
        column: column.to_string(),
        destination_id: s.destination_id,
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
            Some(user) => {
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
                here_and_now_cards: Vec::new(),
                chance_cards: Vec::new(),
                can_use_say_no: false,
                can_use_discount: false,
                can_use_intercept: false,
                can_use_collect_tax: false,
                can_use_steal_first_class: false,
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
                 here_and_now_cards: Vec::new(),
                 chance_cards: Vec::new(),
                  can_use_say_no: false,
                  can_use_discount: false,
                  can_use_intercept: false,
                  can_use_collect_tax: false,
                  can_use_steal_first_class: false,
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
                return Err(Error::new("Nu este randul tau!"));
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

    async fn resolve_jail_decision(&self, ctx: &Context<'_>, code: String, username: String, action: String) -> Result<RollResult> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;

            // Validate turn
            let current_idx = game.current_player_idx;
            if game.players[current_idx].name != username {
                return Err(Error::new("Nu este rândul tău!"));
            }

            let jail_action = match action.as_str() {
                "PayFine" => game_engine::game::JailAction::PayFine,
                "UseCard" => game_engine::game::JailAction::UseCard,
                "Roll" => game_engine::game::JailAction::Roll,
                _ => return Err(Error::new("Acțiune invalidă pentru închisoare")),
            };

            // Execute Action
            let result = game.resolve_jail_decision(jail_action)
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
    async fn resolve_forced_deal(&self, ctx: &Context<'_>, code: String, username: String, action: String, target: Option<String>) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;

            // Validate turn
            let current_idx = game.current_player_idx;
             if game.players[current_idx].name != username {
                return Err(Error::new("Nu este randul tau!"));
            }

            // Execute Action
            game.resolve_forced_deal(&action, target)
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

            // Validate turn - support intercepted purchase
            let buyer_idx = if let GameStep::WaitingForPurchaseDecision { buyer_idx, .. } = &game.step {
                *buyer_idx
            } else {
                game.current_player_idx
            };

            if game.players[buyer_idx].name != username {
                return Err(Error::new("Nu este rândul tău să cumperi!"));
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

            // Validate turn - support intercepted first class
            let buyer_idx = if let GameStep::WaitingForFirstClassDecision { buyer_idx, .. } = &game.step {
                *buyer_idx
            } else {
                game.current_player_idx
            };

            if game.players[buyer_idx].name != username {
                return Err(Error::new("Nu este rândul tău să cumperi First Class!"));
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

    /// Resolve Airport flight decision
    async fn resolve_airport_decision(&self, ctx: &Context<'_>, code: String, username: String, buy_flight: bool) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;

            // Validate turn - support out-of-turn movement
            let buyer_idx = if let GameStep::WaitingForAirportDecision { buyer_idx } = game.step {
                buyer_idx
            } else {
                game.current_player_idx
            };

            if game.players[buyer_idx].name != username {
                return Err(Error::new("Nu este rândul tău să alegi zborul!"));
            }

            // Execute Action
            game.resolve_airport_decision(buy_flight)
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

    /// Resolve Airport destination selection
    async fn resolve_airport_destination(&self, ctx: &Context<'_>, code: String, username: String, target_position: u8) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;

            // Validate turn - support out-of-turn movement
            let buyer_idx = if let GameStep::WaitingForAirportDestination { buyer_idx } = game.step {
                buyer_idx
            } else {
                game.current_player_idx
            };

            if game.players[buyer_idx].name != username {
                return Err(Error::new("Nu este rândul tău să alegi destinația!"));
            }

            // Execute Action
            game.resolve_airport_destination(target_position)
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

    /// Use a card from hand
    async fn use_card(&self, ctx: &Context<'_>, code: String, username: String, card_id: String) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;

            // Find player index
            let player_idx = game.players.iter().position(|p| p.name == username)
                .ok_or_else(|| Error::new("Player not found in game"))?;

            // Try to use as Here & Now card first, then Chance card
            let res = game.use_here_and_now_card(player_idx, card_id.clone());
            
            if res.is_err() {
                // Try Chance card
                game.use_chance_card(player_idx, card_id)
                    .map_err(|e| Error::new(e))?;
            }

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

    /// Resolve target selection (Dice Duel or Stamp Swap)
    async fn resolve_target_selection(&self, ctx: &Context<'_>, code: String, username: String, target_username: String) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;

            // Validate turn - check if the user is the selector
            if let GameStep::WaitingForTargetSelection { selector_idx, .. } = &game.step {
                if game.players[*selector_idx].name != username {
                    return Err(Error::new("Nu este rândul tău să alegi jucătorul!"));
                }
            } else {
                return Err(Error::new("Nu ești în etapa de a alege un jucător!"));
            }

            // Execute Action
            game.resolve_target_selection(target_username)
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

    /// Roll dice in a duel
    async fn roll_duel_die(&self, ctx: &Context<'_>, code: String, username: String) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;
            
            // Validate turn - check if the user is the one who should roll
            let roller_idx = match &game.step {
                GameStep::WaitingForDiceDuel { challenger_idx, target_idx, challenger_roll, .. } => {
                    if challenger_roll.is_none() { *challenger_idx } else { *target_idx }
                },
                _ => return Err(Error::new("Nu ești într-un duel de zaruri!")),
            };

            if game.players[roller_idx].name != username {
                return Err(Error::new("Nu este rândul tău să dai cu zarul!"));
            }

            // Execute Action
            game.roll_duel_die()
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

    /// Place a bid during an auction (any player can bid)
    async fn place_bid(&self, ctx: &Context<'_>, code: String, username: String, amount: u32) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;

            // Find player index - any player can bid, not just current turn player
            let bidder_idx = game.players.iter().position(|p| p.name == username)
                .ok_or_else(|| Error::new("Player not found in game"))?;

            // Execute bid
            game.place_bid(bidder_idx, amount)
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

    /// Resolve auction when timer expires
    async fn resolve_auction(&self, ctx: &Context<'_>, code: String) -> Result<Lobby> {
        let db = ctx.data::<DB>()?;
        let lobby_opt = db.lobbies().find_one(doc! { "code": &code }, None).await?;

        if let Some(mut lobby) = lobby_opt {
            let game = lobby.game.as_mut().ok_or(Error::new("No game engine state"))?;

            // Execute auction resolution
            game.resolve_auction()
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


use async_graphql::{Context, Object, Schema, EmptySubscription, Result, Error};
use crate::db::DB;
use crate::model::{User, Lobby, Player};
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
         db.find_user_by_username(&username).await.map_err(|e| e.into())
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
            created_at: Utc::now(),
            last_login: None,
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
                
                user.last_login = Some(Utc::now());
                db.users().update_one(
                    doc! { "_id": user.id },
                    doc! { "$set": { "last_login": user.last_login } },
                    None
                ).await?;
                
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
            players: vec![Player { username, character: None }],
            state: "waiting".to_string(),
            created_at: Utc::now(),
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

             let new_player = Player { username: username.clone(), character: None };
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

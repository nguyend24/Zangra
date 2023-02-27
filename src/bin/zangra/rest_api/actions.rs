use std::{str::FromStr, sync::Arc};

use axum::{http::response::Response, http::StatusCode, response::IntoResponse};
use serde_json::{json, Value};
use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::{
        channel::{Embed, Message},
        id::{ChannelId, GuildId, MessageId},
        ModelError,
    },
    Error,
};
use sqlx::{Pool, Sqlite};
use tracing::error;

pub enum DiscordAction {
    EmbedCreate,
    EmbedDelete,
    EmbedEdit,
    EmbedGet,
    RoleSelectorCreate,
    RoleSelectorDelete,
    RoleSelectorEdit,
}

//TODO Add logging for errors

impl FromStr for DiscordAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EmbedCreate" => Ok(Self::EmbedCreate),
            "EmbedDelete" => Ok(Self::EmbedDelete),
            "EmbedEdit" => Ok(Self::EmbedEdit),
            "EmbedGet" => Ok(Self::EmbedGet),
            "RoleSelectorCreate" => Ok(Self::RoleSelectorCreate),
            "RoleSelectorDelete" => Ok(Self::RoleSelectorDelete),
            "RoleSelectorEdit" => Ok(Self::RoleSelectorEdit),
            _ => Err(()),
        }
    }
}

pub struct ActionRunner {
    pub ctx: Arc<Context>,
    pub db_pool: Pool<Sqlite>,
    pub discord_action: DiscordAction,
    pub data: Value,
}

impl ActionRunner {
    pub async fn run(self) -> impl IntoResponse {
        match self.discord_action {
            DiscordAction::EmbedCreate => self.embed_create().await,
            DiscordAction::EmbedDelete => self.embed_delete().await,
            DiscordAction::EmbedEdit => self.embed_edit().await,
            DiscordAction::EmbedGet => self.embed_get().await,
            DiscordAction::RoleSelectorCreate => self.role_selector_create().await,
            DiscordAction::RoleSelectorDelete => self.role_selector_delete().await,
            DiscordAction::RoleSelectorEdit => self.role_selector_edit().await,
        }
    }

    /// Send a new embed in a channel
    /// 
    /// RECEIVE
    /// {
    ///     "action": "EmbedCreate"
    ///     "data":
    ///     {
    ///         "channel_id": ChannelId,
    ///         "guild_id": GuildId
    ///         "embed": Embed
    ///     }
    /// }
    async fn embed_create(self) -> Result<Response<String>, StatusCode> {
        let channel_id: ChannelId = match serde_json::from_value(self.data["channel_id"].clone()) {
            Ok(cid) => cid,
            Err(_why) => {
                error!("EmbedCreate: Missing ChannelId");
                return Err(StatusCode::BAD_REQUEST)},
        };

        let guild_id: GuildId = match serde_json::from_value(self.data["guild_id"].clone()) {
            Ok(gid) => gid,
            Err(_why) => {
                error!("EmbedCreate: Missing GuildId");
                return Err(StatusCode::BAD_REQUEST)},
        };

        let embed: Embed = match serde_json::from_value(self.data["embed"].clone()) {
            Ok(e) => e,
            Err(_why) => {
                error!("EmbedCreate: Missing embed data");
                return Err(StatusCode::BAD_REQUEST);},
        };

        let msg = match channel_id
            .send_message(&self.ctx, |m| {
                m.set_embed(CreateEmbed::from(embed));
                m
            })
            .await
        {
            Ok(msg) => msg,
            Err(Error::Model(ModelError::MessageTooLong(mtl))) => {
                let response = match Response::builder().status(StatusCode::BAD_REQUEST).body(
                    json!({
                        "status": "error",
                        "error": "MessageTooLong",
                        "length_over": mtl
                    })
                    .to_string(),
                ) {
                    Ok(r) => r,
                    Err(why) => {
                        error!("EmbedCreate: Error creating response: {}", why);
                        return Err(StatusCode::BAD_GATEWAY)}
                };

                return Ok(response);
            }
            Err(Error::Http(_http_error)) => {
                //missing permissions to send in this channel
                let response = match Response::builder().status(StatusCode::BAD_REQUEST).body(
                    json!({
                        "status": "error",
                        "error": "MissingSendPermission"
                    })
                    .to_string(),
                ) {
                    Ok(r) => r,
                    Err(why) => {
                        error!("EmbedCreate: Error creating response: {}", why);
                        return Err(StatusCode::BAD_GATEWAY)},
                };

                return Ok(response);
            }
            Err(why) => {
                //unknown error
                error!("EmbedCreate: Unknown message send error: {}", why);
                return Err(StatusCode::BAD_GATEWAY);
            }
        };

        //save embed id into database
        let msg_id_i64 = msg.id.as_u64().to_owned() as i64;
        let guild_id_i64 = guild_id.as_u64().to_owned() as i64;
        let channel_id_i64 = channel_id.as_u64().to_owned() as i64;

        match sqlx::query!(
            "INSERT INTO Embed (EmbedId, GuildId, ChannelId) VALUES (?, ?, ?)",
            msg_id_i64,
            guild_id_i64,
            channel_id_i64
        )
        .execute(&self.db_pool)
        .await
        {
            Ok(_sqr) => {}
            Err(why) => {
                error!("EmbedCreate: Sqlite write error: {}", why);
                let _ = msg.delete(&self.ctx).await;
                return Err(StatusCode::BAD_GATEWAY);
            }
        }

        //return a success code along with the message that was sent
        let response = match Response::builder().status(StatusCode::OK).body(
            json!({
                "status": "success",
                "message": msg
            })
            .to_string(),
        ) {
            Ok(r) => r,
            Err(why) => {
                error!("EmbedCreate: Error creating response: {}", why);
                return Err(StatusCode::BAD_GATEWAY)},
        };

        Ok(response)
    }

    /// Delete an existing embed in a channel
    /// 
    /// {
    ///     "action": "EmbedDelete"
    ///     "data":
    ///     {
    ///         "channel_id": ChannelId
    ///         "message": MessageId
    ///     }
    /// }
    async fn embed_delete(self) -> Result<Response<String>, StatusCode> {
        let channel_id: ChannelId = match serde_json::from_value(self.data["channel_id"].clone()) {
            Ok(cid) => cid,
            Err(_why) => {
                error!("EmbedDelete: Missing ChannelId data");
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        let msg_id: MessageId = match serde_json::from_value(self.data["message_id"].clone()) {
            Ok(m) => m,
            Err(_why) => {
                error!("EmbedDelete: Missing message data");
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        let msg: Message = match self.ctx.http.get_message(channel_id.as_u64().to_owned(), msg_id.as_u64().to_owned()).await {
            Ok(m) => m,
            Err(why) => {
                error!("EmbedDelete: Unable to retrieve message: {}", why);
                return Err(StatusCode::BAD_GATEWAY);
            }
        };

        match msg.delete(&self.ctx).await {
            Ok(()) => {},
            Err(Error::Model(ModelError::InvalidPermissions(ip))) => {
                let response = match Response::builder().status(StatusCode::BAD_REQUEST).body(
                    json!({
                        "status": "error",
                        "error": "InvalidPermissions",
                        "permissions": ip
                    })
                    .to_string(),
                ) {
                    Ok(r) => r,
                    Err(why) => {
                        error!("EmbedDelete: Error creating response: {}", why);
                        return Err(StatusCode::BAD_GATEWAY)},
                };

                return Ok(response);
            }
            Err(why) => {
                //unknown error
                error!("EmbedDelete: Unknown error: {}", why);
                return Err(StatusCode::BAD_GATEWAY);
            }
        }
        
        let response = match Response::builder().status(StatusCode::OK).body(
            json!({
                "status": "success",
            }).to_string()
        ) {
            Ok(r) => r,
            Err(why) => {
                error!("EmbedDelete: Error creating response: {}", why);
                return Err(StatusCode::BAD_GATEWAY)},
        };

        Ok(response)
    }

    /// Edit an existing embed in a channel
    /// 
    /// {
    ///     "action": "EmbedEdit"
    ///     "data":
    ///     {
    ///         "channel_id": ChannelId
    ///         "message": MessageId
    ///         "new_embed": Embed
    ///     }
    /// }
    async fn embed_edit(self) -> Result<Response<String>, StatusCode> {
        let channel_id: ChannelId = match serde_json::from_value(self.data["channel_id"].clone()) {
            Ok(cid) => cid,
            Err(_why) => {
                error!("EmbedEdit: Missing ChannelId data");
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        let msg_id: MessageId = match serde_json::from_value(self.data["message_id"].clone()) {
            Ok(m) => m,
            Err(_why) => {
                error!("EmbedEdit: Missing message data");
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        let mut msg: Message = match self.ctx.http.get_message(channel_id.as_u64().to_owned(), msg_id.as_u64().to_owned()).await {
            Ok(m) => m,
            Err(why) => {
                error!("EmbedEdit: Unable to retrieve message: {}", why);
                return Err(StatusCode::BAD_GATEWAY);
            }
        };

        let new_embed: Embed = match serde_json::from_value(self.data["new_embed"].clone()) {
            Ok(e) => e,
            Err(_why) => {
                error!("EmbedEdit: Missing embed data");
                return Err(StatusCode::BAD_REQUEST);
            }
        }; 

        match msg.edit(&self.ctx, |m| {
            m.set_embed(CreateEmbed::from(new_embed));
            m
        }).await {
            Ok(()) => {},
            Err(Error::Model(ModelError::MessageTooLong(mtl))) => {
                let response = match Response::builder().status(StatusCode::BAD_REQUEST).body(
                    json!({
                        "status": "error",
                        "error": "MessageTooLong",
                        "length_over": mtl
                    })
                    .to_string(),
                ) {
                    Ok(r) => r,
                    Err(why) => {
                        error!("EmbedCreate: Error creating response: {}", why);
                        return Err(StatusCode::BAD_REQUEST)},
                };

                return Ok(response);
            }
            Err(Error::Model(ModelError::InvalidUser)) => {
                //This shouldn't be happening
                return Err(StatusCode::BAD_GATEWAY)
            }
            _ => {}
        };

        //return a success code along with the message that was sent
        let response = match Response::builder().status(StatusCode::OK).body(
            json!({
                "status": "success",
                "message": msg
            })
            .to_string(),
        ) {
            Ok(r) => r,
            Err(why) => {
                error!("EmbedCreate: Error creating response: {}", why);
                return Err(StatusCode::BAD_GATEWAY)},
        };

        Ok(response)
    }

    ///Get a list of embeds for server
    /// 
    /// RECEIVE 
    /// {
    ///     "action": "EmbedGet"
    ///     "data": {
    ///         "guild_id": GuildId
    ///     } 
    /// }
    /// 
    /// SEND
    /// {
    ///     "status": "success"
    ///     "data": {
    ///         "messages": [Message]
    ///     }
    /// }
    async fn embed_get(self) -> Result<Response<String>, StatusCode> {
        let guild_id: GuildId = match serde_json::from_value(self.data["guild_id"].clone()) {
            Ok(gid) => gid,
            Err(_why) => {
                error!("EmbedGet: Missing GuildId");
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        let guild_id_i64 = guild_id.as_u64().to_owned() as i64;
        let embeds = match sqlx::query!("SELECT EmbedId, GuildId FROM Embed WHERE GuildId = ?", guild_id_i64).fetch_all(&self.db_pool).await {
            Ok(r) => r,
            Err(why) => {
                error!("Can't read from database: {}", why);
                return Err(StatusCode::BAD_GATEWAY);
            }
        };

        for embed_id in embeds {
            let e = embed_id.EmbedId;
        }

        todo!()
    }

    /// Create a new role selector interaction in a channel
    async fn role_selector_create(self) -> Result<Response<String>, StatusCode> {
        todo!()
    }

    /// Delete a new role selector interaction in a channel
    async fn role_selector_delete(self) -> Result<Response<String>, StatusCode> {
        todo!()
    }

    /// Edit an existing role selector interaction in a channel
    async fn role_selector_edit(self) -> Result<Response<String>, StatusCode> {
        todo!()
    }
}

/*
EmbedCreate:
data - serenity:Embed
Return Success message

EmbedEdit:
data - serenity:Embed
Return success message

RoleSelectorCreate:
data - serenity:Embed
Return success message

RoleSelectorEdit:
data - serenity:Embed
Return success message
*/

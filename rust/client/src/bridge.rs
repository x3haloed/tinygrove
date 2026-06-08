#[path = "../generated/mod.rs"]
mod generated;

use std::collections::HashMap;

use generated::{
    ChatMessage, ChatMessageTableAccess, DbConnection, Player, PlayerPosition,
    PlayerPositionTableAccess, PlayerTableAccess, join_game, move_player, send_chat,
};
use godot::prelude::*;
use spacetimedb_sdk::{DbContext, Table};

const CLIENT_PROTOCOL_VERSION: u32 = 1;
const DEFAULT_DB_URI: &str = "http://127.0.0.1:3000";
const DEFAULT_DB_NAME: &str = "tinygrove-dev";

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct TinyGroveClient {
    connection: Option<DbConnection>,
    connected: bool,
    subscribed: bool,
    status: String,
    last_error: String,
}

#[godot_api]
impl IRefCounted for TinyGroveClient {
    fn init(_base: Base<RefCounted>) -> Self {
        Self {
            connection: None,
            connected: false,
            subscribed: false,
            status: "disconnected".to_string(),
            last_error: String::new(),
        }
    }
}

#[godot_api]
impl TinyGroveClient {
    #[func]
    pub fn connect_local(&mut self) -> bool {
        self.connect_to(DEFAULT_DB_URI.into(), DEFAULT_DB_NAME.into())
    }

    #[func]
    pub fn connect_to(&mut self, uri: GString, database_name: GString) -> bool {
        self.connection = None;
        self.connected = false;
        self.subscribed = false;
        self.last_error.clear();
        self.status = "connecting".to_string();

        let uri = uri.to_string();
        let database_name = database_name.to_string();
        let build_result = DbConnection::builder()
            .with_uri(uri)
            .with_database_name(database_name)
            .on_connect(|ctx, _identity, _token| {
                ctx.subscription_builder().subscribe([
                    "SELECT * FROM server_config",
                    "SELECT * FROM player",
                    "SELECT * FROM player_position",
                    "SELECT * FROM chat_message",
                ]);
            })
            .build();

        match build_result {
            Ok(connection) => {
                self.connection = Some(connection);
                self.connected = true;
                self.status = "connected".to_string();
                true
            }
            Err(error) => {
                self.status = "connect failed".to_string();
                self.last_error = error.to_string();
                false
            }
        }
    }

    #[func]
    pub fn poll(&mut self) {
        let Some(connection) = self.connection.as_ref() else {
            return;
        };

        match connection.frame_tick() {
            Ok(()) => {
                self.connected = connection.is_active();
                self.subscribed = true;
                if self.connected {
                    self.status = "connected".to_string();
                }
            }
            Err(error) => {
                self.connected = false;
                self.status = "disconnected".to_string();
                self.last_error = error.to_string();
            }
        }
    }

    #[func]
    pub fn join_game(&mut self, display_name: GString, avatar_color: i64) -> bool {
        let Some(connection) = self.connection.as_ref() else {
            self.last_error = "Not connected".to_string();
            return false;
        };

        let avatar_color = avatar_color.clamp(0, u32::MAX as i64) as u32;
        match connection.reducers.join_game(
            display_name.to_string(),
            avatar_color,
            CLIENT_PROTOCOL_VERSION,
        ) {
            Ok(()) => true,
            Err(error) => {
                self.last_error = error.to_string();
                false
            }
        }
    }

    #[func]
    pub fn move_player(&mut self, dx: i64, dy: i64) -> bool {
        let Some(connection) = self.connection.as_ref() else {
            self.last_error = "Not connected".to_string();
            return false;
        };

        match connection
            .reducers
            .move_player(clamp_axis(dx), clamp_axis(dy))
        {
            Ok(()) => true,
            Err(error) => {
                self.last_error = error.to_string();
                false
            }
        }
    }

    #[func]
    pub fn send_chat(&mut self, body: GString) -> bool {
        let Some(connection) = self.connection.as_ref() else {
            self.last_error = "Not connected".to_string();
            return false;
        };

        match connection.reducers.send_chat(body.to_string()) {
            Ok(()) => true,
            Err(error) => {
                self.last_error = error.to_string();
                false
            }
        }
    }

    #[func]
    pub fn status(&self) -> GString {
        GString::from(&self.status)
    }

    #[func]
    pub fn last_error(&self) -> GString {
        GString::from(&self.last_error)
    }

    #[func]
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    #[func]
    pub fn players(&self) -> Array<Dictionary<Variant, Variant>> {
        let mut rows = Array::new();
        let Some(connection) = self.connection.as_ref() else {
            return rows;
        };

        let players = connection
            .db
            .player()
            .iter()
            .map(|player| (identity_key(&player.identity), player))
            .collect::<HashMap<_, _>>();

        for position in connection.db.player_position().iter() {
            rows.push(&player_dictionary(
                players.get(&identity_key(&position.identity)),
                &position,
            ));
        }

        rows
    }

    #[func]
    pub fn chat_messages(&self) -> Array<Dictionary<Variant, Variant>> {
        let mut rows = Array::new();
        let Some(connection) = self.connection.as_ref() else {
            return rows;
        };

        let players = connection
            .db
            .player()
            .iter()
            .map(|player| (identity_key(&player.identity), player))
            .collect::<HashMap<_, _>>();

        let mut messages = connection.db.chat_message().iter().collect::<Vec<_>>();
        messages.sort_by_key(|message| message.id);

        for message in messages.into_iter().rev().take(24).rev() {
            rows.push(&chat_dictionary(
                &message,
                players.get(&identity_key(&message.sender)),
            ));
        }

        rows
    }
}

fn clamp_axis(value: i64) -> i32 {
    value.clamp(-1, 1) as i32
}

fn identity_key(identity: &spacetimedb_sdk::Identity) -> String {
    identity.to_hex().to_string()
}

fn player_dictionary(
    player: Option<&Player>,
    position: &PlayerPosition,
) -> Dictionary<Variant, Variant> {
    let mut dict = Dictionary::new();
    let identity = identity_key(&position.identity);
    let display_name = player
        .map(|player| player.display_name.clone())
        .unwrap_or_else(|| short_identity(&identity));
    let avatar_color = player.map(|player| player.avatar_color).unwrap_or(0x66CCAA);
    let online = player.map(|player| player.online).unwrap_or(false);

    dict.set("identity", identity);
    dict.set("display_name", display_name);
    dict.set("avatar_color", avatar_color as i64);
    dict.set("online", online);
    dict.set("x", position.x);
    dict.set("y", position.y);
    dict
}

fn chat_dictionary(message: &ChatMessage, player: Option<&Player>) -> Dictionary<Variant, Variant> {
    let mut dict = Dictionary::new();
    let sender = identity_key(&message.sender);
    let display_name = player
        .map(|player| player.display_name.clone())
        .unwrap_or_else(|| short_identity(&sender));

    dict.set("id", message.id as i64);
    dict.set("sender", sender);
    dict.set("display_name", display_name);
    dict.set("body", message.body.clone());
    dict
}

fn short_identity(identity: &str) -> String {
    identity.chars().take(8).collect()
}

#[path = "../generated/mod.rs"]
mod generated;

use std::collections::HashMap;

use generated::{
    ChatMessage, ChatMessageTableAccess, DbConnection, Player, PlayerPlot, PlayerPlotTableAccess,
    PlayerPosition, PlayerPositionTableAccess, PlayerTableAccess, WorldObject,
    WorldObjectTableAccess, interact_near, join_game, move_player, place_object, send_chat,
};
use godot::prelude::*;
use spacetimedb_sdk::{DbContext, Table, credentials};

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
        self.connect_local_with_profile("human".into())
    }

    #[func]
    pub fn connect_local_with_profile(&mut self, profile: GString) -> bool {
        self.connect_to_with_profile(DEFAULT_DB_URI.into(), DEFAULT_DB_NAME.into(), profile)
    }

    #[func]
    pub fn connect_to(&mut self, uri: GString, database_name: GString) -> bool {
        self.connect_to_with_profile(uri, database_name, "human".into())
    }

    #[func]
    pub fn connect_to_with_profile(
        &mut self,
        uri: GString,
        database_name: GString,
        profile: GString,
    ) -> bool {
        self.connection = None;
        self.connected = false;
        self.subscribed = false;
        self.last_error.clear();
        self.status = "connecting".to_string();

        let uri = uri.to_string();
        let database_name = database_name.to_string();
        let credential_key = credential_key(&uri, &database_name, &profile.to_string());
        let token = match credentials::File::new(&credential_key).load() {
            Ok(token) => token,
            Err(error) => {
                self.last_error = format!("Could not load credentials for {credential_key}: {error}");
                None
            }
        };
        let save_credential_key = credential_key.clone();
        let build_result = DbConnection::builder()
            .with_uri(uri)
            .with_database_name(database_name)
            .with_token(token)
            .on_connect(move |ctx, _identity, token| {
                if let Err(error) = credentials::File::new(&save_credential_key).save(token.to_string()) {
                    godot_warn!(
                        "Could not save SpacetimeDB credentials for {}: {}",
                        save_credential_key,
                        error
                    );
                }
                ctx.subscription_builder().subscribe([
                    "SELECT * FROM server_config",
                    "SELECT * FROM player",
                    "SELECT * FROM player_plot",
                    "SELECT * FROM player_position",
                    "SELECT * FROM chat_message",
                    "SELECT * FROM world_object",
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
    pub fn place_object(&mut self, kind: GString) -> bool {
        let Some(connection) = self.connection.as_ref() else {
            self.last_error = "Not connected".to_string();
            return false;
        };

        match connection.reducers.place_object(kind.to_string()) {
            Ok(()) => true,
            Err(error) => {
                self.last_error = error.to_string();
                false
            }
        }
    }

    #[func]
    pub fn interact_near(&mut self) -> bool {
        let Some(connection) = self.connection.as_ref() else {
            self.last_error = "Not connected".to_string();
            return false;
        };

        match connection.reducers.interact_near() {
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
    pub fn local_identity(&self) -> GString {
        let Some(connection) = self.connection.as_ref() else {
            return GString::new();
        };

        connection
            .try_identity()
            .map(|identity| GString::from(&identity_key(&identity)))
            .unwrap_or_else(GString::new)
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

    #[func]
    pub fn world_objects(&self) -> Array<Dictionary<Variant, Variant>> {
        let mut rows = Array::new();
        let Some(connection) = self.connection.as_ref() else {
            return rows;
        };

        let mut objects = connection.db.world_object().iter().collect::<Vec<_>>();
        objects.sort_by_key(|object| object.id);

        for object in objects {
            rows.push(&world_object_dictionary(&object));
        }

        rows
    }

    #[func]
    pub fn player_plots(&self) -> Array<Dictionary<Variant, Variant>> {
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

        let mut plots = connection.db.player_plot().iter().collect::<Vec<_>>();
        plots.sort_by_key(|plot| (plot.origin_y, plot.origin_x));

        for plot in plots {
            rows.push(&player_plot_dictionary(
                &plot,
                players.get(&identity_key(&plot.owner)),
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

fn credential_key(uri: &str, database_name: &str, profile: &str) -> String {
    let profile = if profile.trim().is_empty() {
        "human"
    } else {
        profile.trim()
    };
    let raw = format!("tinygrove-{profile}-{uri}-{database_name}");
    raw.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
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
    dict.set("last_dx", position.last_dx);
    dict.set("last_dy", position.last_dy);
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

fn world_object_dictionary(object: &WorldObject) -> Dictionary<Variant, Variant> {
    let mut dict = Dictionary::new();
    dict.set("id", object.id as i64);
    dict.set("kind", object.kind.clone());
    dict.set("x", object.x);
    dict.set("y", object.y);
    dict.set("state", object.state);
    dict.set("created_by", identity_key(&object.created_by));
    dict
}

fn player_plot_dictionary(
    plot: &PlayerPlot,
    player: Option<&Player>,
) -> Dictionary<Variant, Variant> {
    let mut dict = Dictionary::new();
    let owner = identity_key(&plot.owner);
    let display_name = player
        .map(|player| player.display_name.clone())
        .unwrap_or_else(|| short_identity(&owner));

    dict.set("owner", owner);
    dict.set("display_name", display_name);
    dict.set("origin_x", plot.origin_x);
    dict.set("origin_y", plot.origin_y);
    dict.set("width", plot.width);
    dict.set("height", plot.height);
    dict
}

fn short_identity(identity: &str) -> String {
    identity.chars().take(8).collect()
}

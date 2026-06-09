#[path = "../generated/mod.rs"]
mod generated;

use std::collections::HashMap;

use generated::{
    ChatMessage, ChatMessageTableAccess, ContentAsset, ContentAssetTableAccess, DbConnection,
    Player, PlayerPlot, PlayerPlotTableAccess, PlayerPosition, PlayerPositionTableAccess,
    PlayerTableAccess, WorldObject, WorldObjectTableAccess, WorldTile, WorldTileTableAccess,
    interact_near, join_game, move_player, place_object, place_tile, send_chat,
};
use generated::create_content_asset_reducer::create_content_asset;
use generated::update_content_asset_reducer::update_content_asset;
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
    content_asset_cache: HashMap<u64, ContentAssetCacheEntry>,
}

#[derive(Clone, Debug)]
struct ContentAssetCacheEntry {
    id: u64,
    asset_kind: String,
    name: String,
    slug: String,
    status: String,
    grid_divisor: i32,
    placement_w: i32,
    placement_h: i32,
    anchor_x: i32,
    anchor_y: i32,
    collidable: bool,
    transparent_allowed: bool,
    render_format: String,
    render_bytes: String,
    collision_format: String,
    collision_bytes: String,
    preview_format: String,
    preview_bytes: String,
    updated_at_micros: i64,
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
            content_asset_cache: HashMap::new(),
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
        self.content_asset_cache.clear();
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
                    "SELECT * FROM content_asset",
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
    pub fn place_object(&mut self, kind: GString, target_x: i64, target_y: i64) -> bool {
        let Some(connection) = self.connection.as_ref() else {
            self.last_error = "Not connected".to_string();
            return false;
        };

        match connection
            .reducers
            .place_object(kind.to_string(), target_x as i32, target_y as i32)
        {
            Ok(()) => true,
            Err(error) => {
                self.last_error = error.to_string();
                false
            }
        }
    }

    #[func]
    pub fn place_tile(&mut self, kind: GString, target_x: i64, target_y: i64) -> bool {
        let Some(connection) = self.connection.as_ref() else {
            self.last_error = "Not connected".to_string();
            return false;
        };

        match connection
            .reducers
            .place_tile(kind.to_string(), target_x as i32, target_y as i32)
        {
            Ok(()) => true,
            Err(error) => {
                self.last_error = error.to_string();
                false
            }
        }
    }

    #[func]
    pub fn create_content_asset(&mut self, data: Dictionary<Variant, Variant>) -> bool {
        let Some(connection) = self.connection.as_ref() else {
            self.last_error = "Not connected".to_string();
            return false;
        };

        let asset_kind = data.get("kind").map(|v: Variant| v.to_string()).unwrap_or_default();
        let name = data.get("name").map(|v: Variant| v.to_string()).unwrap_or_default();
        let slug = data.get("slug").map(|v: Variant| v.to_string()).unwrap_or_default();
        let status = data.get("status").map(|v: Variant| v.to_string()).unwrap_or_default();
        let grid_divisor = data.get("grid_divisor").map(|v: Variant| v.to::<i64>()).unwrap_or(4) as i32;
        let placement_w = data.get("placement_w").map(|v: Variant| v.to::<i64>()).unwrap_or(1) as i32;
        let placement_h = data.get("placement_h").map(|v: Variant| v.to::<i64>()).unwrap_or(1) as i32;
        let anchor_x = data.get("anchor_x").map(|v: Variant| v.to::<i64>()).unwrap_or(0) as i32;
        let anchor_y = data.get("anchor_y").map(|v: Variant| v.to::<i64>()).unwrap_or(0) as i32;
        let collidable = data.get("collidable").map(|v: Variant| v.to::<bool>()).unwrap_or(false);
        let transparent_allowed = data.get("transparent_allowed").map(|v: Variant| v.to::<bool>()).unwrap_or(false);
        let render_format = data.get("render_format").map(|v: Variant| v.to_string()).unwrap_or_default();
        let render_bytes = data.get("render_bytes").map(|v: Variant| v.to_string()).unwrap_or_default();
        let collision_format = data.get("collision_format").map(|v: Variant| v.to_string()).unwrap_or_default();
        let collision_bytes = data.get("collision_bytes").map(|v: Variant| v.to_string()).unwrap_or_default();
        let preview_format = data.get("preview_format").map(|v: Variant| v.to_string()).unwrap_or_default();
        let preview_bytes = data.get("preview_bytes").map(|v: Variant| v.to_string()).unwrap_or_default();

        match connection.reducers.create_content_asset(
            asset_kind,
            name,
            slug,
            status,
            grid_divisor,
            placement_w,
            placement_h,
            anchor_x,
            anchor_y,
            collidable,
            transparent_allowed,
            render_format,
            render_bytes,
            collision_format,
            collision_bytes,
            preview_format,
            preview_bytes,
        ) {
            Ok(()) => true,
            Err(error) => {
                self.last_error = error.to_string();
                false
            }
        }
    }

    #[func]
    pub fn update_content_asset(&mut self, asset_id: i64, data: Dictionary<Variant, Variant>) -> bool {
        let Some(connection) = self.connection.as_ref() else {
            self.last_error = "Not connected".to_string();
            return false;
        };

        let asset_kind = data.get("kind").map(|v: Variant| v.to_string()).unwrap_or_default();
        let name = data.get("name").map(|v: Variant| v.to_string()).unwrap_or_default();
        let slug = data.get("slug").map(|v: Variant| v.to_string()).unwrap_or_default();
        let status = data.get("status").map(|v: Variant| v.to_string()).unwrap_or_default();
        let grid_divisor = data.get("grid_divisor").map(|v: Variant| v.to::<i64>()).unwrap_or(4) as i32;
        let placement_w = data.get("placement_w").map(|v: Variant| v.to::<i64>()).unwrap_or(1) as i32;
        let placement_h = data.get("placement_h").map(|v: Variant| v.to::<i64>()).unwrap_or(1) as i32;
        let anchor_x = data.get("anchor_x").map(|v: Variant| v.to::<i64>()).unwrap_or(0) as i32;
        let anchor_y = data.get("anchor_y").map(|v: Variant| v.to::<i64>()).unwrap_or(0) as i32;
        let collidable = data.get("collidable").map(|v: Variant| v.to::<bool>()).unwrap_or(false);
        let transparent_allowed = data.get("transparent_allowed").map(|v: Variant| v.to::<bool>()).unwrap_or(false);
        let render_format = data.get("render_format").map(|v: Variant| v.to_string()).unwrap_or_default();
        let render_bytes = data.get("render_bytes").map(|v: Variant| v.to_string()).unwrap_or_default();
        let collision_format = data.get("collision_format").map(|v: Variant| v.to_string()).unwrap_or_default();
        let collision_bytes = data.get("collision_bytes").map(|v: Variant| v.to_string()).unwrap_or_default();
        let preview_format = data.get("preview_format").map(|v: Variant| v.to_string()).unwrap_or_default();
        let preview_bytes = data.get("preview_bytes").map(|v: Variant| v.to_string()).unwrap_or_default();

        match connection.reducers.update_content_asset(
            asset_id as u64,
            name,
            slug,
            status,
            grid_divisor,
            placement_w,
            placement_h,
            anchor_x,
            anchor_y,
            collidable,
            transparent_allowed,
            render_format,
            render_bytes,
            collision_format,
            collision_bytes,
            preview_format,
            preview_bytes,
        ) {
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
    pub fn world_tiles(&self) -> Array<Dictionary<Variant, Variant>> {
        let mut rows = Array::new();
        let Some(connection) = self.connection.as_ref() else {
            return rows;
        };

        let mut tiles = connection.db.world_tile().iter().collect::<Vec<_>>();
        tiles.sort_by_key(|tile| (tile.y, tile.x));

        for tile in tiles {
            rows.push(&world_tile_dictionary(&tile));
        }

        rows
    }

    #[func]
    pub fn content_assets(&mut self) -> Array<Dictionary<Variant, Variant>> {
        self.refresh_content_asset_cache();
        let mut rows = Array::new();
        let Some(connection) = self.connection.as_ref() else {
            return rows;
        };

        let mut assets = connection.db.content_asset().iter().collect::<Vec<_>>();
        assets.sort_by_key(|asset| asset.id);

        for asset in assets {
            rows.push(&content_asset_dictionary(&asset));
        }

        rows
    }

    #[func]
    pub fn cached_content_asset(&mut self, asset_id: i64) -> Dictionary<Variant, Variant> {
        self.refresh_content_asset_cache();

        let mut dict = Dictionary::new();
        if let Some(entry) = self.content_asset_cache.get(&(asset_id as u64)) {
            dict.set("id", entry.id as i64);
            dict.set("asset_kind", entry.asset_kind.clone());
            dict.set("name", entry.name.clone());
            dict.set("slug", entry.slug.clone());
            dict.set("status", entry.status.clone());
            dict.set("grid_divisor", entry.grid_divisor);
            dict.set("placement_w", entry.placement_w);
            dict.set("placement_h", entry.placement_h);
            dict.set("anchor_x", entry.anchor_x);
            dict.set("anchor_y", entry.anchor_y);
            dict.set("collidable", entry.collidable);
            dict.set("transparent_allowed", entry.transparent_allowed);
            dict.set("render_format", entry.render_format.clone());
            dict.set("render_bytes", entry.render_bytes.clone());
            dict.set("collision_format", entry.collision_format.clone());
            dict.set("collision_bytes", entry.collision_bytes.clone());
            dict.set("preview_format", entry.preview_format.clone());
            dict.set("preview_bytes", entry.preview_bytes.clone());
            dict.set("updated_at_micros", entry.updated_at_micros);
        }
        dict
    }

    fn refresh_content_asset_cache(&mut self) {
        let Some(connection) = self.connection.as_ref() else {
            return;
        };

        self.content_asset_cache.clear();
        for asset in connection.db.content_asset().iter() {
            self.content_asset_cache
                .insert(asset.id, content_asset_cache_entry(&asset));
        }
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
    dict.set(
        "updated_at_micros",
        position.updated_at.to_micros_since_unix_epoch(),
    );
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
    dict.set(
        "sent_at_micros",
        message.sent_at.to_micros_since_unix_epoch(),
    );
    dict
}

fn world_object_dictionary(object: &WorldObject) -> Dictionary<Variant, Variant> {
    let mut dict = Dictionary::new();
    dict.set("id", object.id as i64);
    dict.set("kind", object.kind.clone());
    dict.set("text", object.text.clone());
    dict.set("x", object.x);
    dict.set("y", object.y);
    dict.set("state", object.state);
    dict.set("created_by", identity_key(&object.created_by));
    dict.set(
        "created_at_micros",
        object.created_at.to_micros_since_unix_epoch(),
    );
    dict.set(
        "updated_at_micros",
        object.updated_at.to_micros_since_unix_epoch(),
    );
    dict
}

fn world_tile_dictionary(tile: &WorldTile) -> Dictionary<Variant, Variant> {
    let mut dict = Dictionary::new();
    dict.set("id", tile.id as i64);
    dict.set("kind", tile.kind.clone());
    dict.set("x", tile.x);
    dict.set("y", tile.y);
    dict.set("created_by", identity_key(&tile.created_by));
    dict.set(
        "created_at_micros",
        tile.created_at.to_micros_since_unix_epoch(),
    );
    dict.set(
        "updated_at_micros",
        tile.updated_at.to_micros_since_unix_epoch(),
    );
    dict
}

fn content_asset_dictionary(asset: &ContentAsset) -> Dictionary<Variant, Variant> {
    let mut dict = Dictionary::new();
    dict.set("id", asset.id as i64);
    dict.set("owner", identity_key(&asset.owner));
    dict.set("asset_kind", asset.asset_kind.clone());
    dict.set("name", asset.name.clone());
    dict.set("slug", asset.slug.clone());
    dict.set("status", asset.status.clone());
    dict.set("grid_divisor", asset.grid_divisor);
    dict.set("placement_w", asset.placement_w);
    dict.set("placement_h", asset.placement_h);
    dict.set("anchor_x", asset.anchor_x);
    dict.set("anchor_y", asset.anchor_y);
    dict.set("collidable", asset.collidable);
    dict.set("transparent_allowed", asset.transparent_allowed);
    dict.set("render_format", asset.render_format.clone());
    dict.set("render_bytes", asset.render_bytes.clone());
    dict.set("collision_format", asset.collision_format.clone());
    dict.set("collision_bytes", asset.collision_bytes.clone());
    dict.set("preview_format", asset.preview_format.clone());
    dict.set("preview_bytes", asset.preview_bytes.clone());
    dict.set(
        "created_at_micros",
        asset.created_at.to_micros_since_unix_epoch(),
    );
    dict.set(
        "updated_at_micros",
        asset.updated_at.to_micros_since_unix_epoch(),
    );
    dict
}

fn content_asset_cache_entry(asset: &ContentAsset) -> ContentAssetCacheEntry {
    ContentAssetCacheEntry {
        id: asset.id,
        asset_kind: asset.asset_kind.clone(),
        name: asset.name.clone(),
        slug: asset.slug.clone(),
        status: asset.status.clone(),
        grid_divisor: asset.grid_divisor,
        placement_w: asset.placement_w,
        placement_h: asset.placement_h,
        anchor_x: asset.anchor_x,
        anchor_y: asset.anchor_y,
        collidable: asset.collidable,
        transparent_allowed: asset.transparent_allowed,
        render_format: asset.render_format.clone(),
        render_bytes: asset.render_bytes.clone(),
        collision_format: asset.collision_format.clone(),
        collision_bytes: asset.collision_bytes.clone(),
        preview_format: asset.preview_format.clone(),
        preview_bytes: asset.preview_bytes.clone(),
        updated_at_micros: asset.updated_at.to_micros_since_unix_epoch(),
    }
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
    dict.set(
        "assigned_at_micros",
        plot.assigned_at.to_micros_since_unix_epoch(),
    );
    dict
}

fn short_identity(identity: &str) -> String {
    identity.chars().take(8).collect()
}

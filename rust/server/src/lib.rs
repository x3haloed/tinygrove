use spacetimedb::{Identity, ReducerContext, Table, Timestamp, reducer, table};

const SUPPORTED_CLIENT_PROTOCOL: u32 = 1;
const DEFAULT_SPAWN_X: i32 = 0;
const DEFAULT_SPAWN_Y: i32 = 0;
const MOVE_STEP: i32 = 16;
const MAX_DISPLAY_NAME_CHARS: usize = 24;
const MAX_CHAT_BODY_CHARS: usize = 240;

#[table(accessor = server_config, public)]
pub struct ServerConfig {
    #[primary_key]
    pub key: String,
    pub value: String,
    pub updated_at: Timestamp,
}

#[table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub display_name: String,
    pub avatar_color: u32,
    pub client_protocol: u32,
    pub online: bool,
    pub joined_at: Timestamp,
    pub updated_at: Timestamp,
}

#[table(accessor = player_position, public)]
pub struct PlayerPosition {
    #[primary_key]
    pub identity: Identity,
    pub x: i32,
    pub y: i32,
    pub last_dx: i32,
    pub last_dy: i32,
    pub updated_at: Timestamp,
}

#[table(accessor = chat_message, public)]
pub struct ChatMessage {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub sender: Identity,
    pub body: String,
    pub sent_at: Timestamp,
}

#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    upsert_server_config(
        ctx,
        "supported_client_protocol",
        SUPPORTED_CLIENT_PROTOCOL.to_string(),
    );
}

#[reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    if let Some(player) = ctx.db.player().identity().find(ctx.sender()) {
        ctx.db.player().identity().update(Player {
            online: true,
            updated_at: ctx.timestamp,
            ..player
        });
    }
}

#[reducer(client_disconnected)]
pub fn client_disconnected(ctx: &ReducerContext) {
    if let Some(player) = ctx.db.player().identity().find(ctx.sender()) {
        ctx.db.player().identity().update(Player {
            online: false,
            updated_at: ctx.timestamp,
            ..player
        });
    }
}

#[reducer]
pub fn join_game(
    ctx: &ReducerContext,
    display_name: String,
    avatar_color: u32,
    client_protocol: u32,
) -> Result<(), String> {
    if client_protocol != SUPPORTED_CLIENT_PROTOCOL {
        return Err(format!(
            "Unsupported client protocol {client_protocol}; server requires {SUPPORTED_CLIENT_PROTOCOL}"
        ));
    }

    let display_name = clean_display_name(display_name)?;

    if let Some(player) = ctx.db.player().identity().find(ctx.sender()) {
        ctx.db.player().identity().update(Player {
            display_name,
            avatar_color,
            client_protocol,
            online: true,
            updated_at: ctx.timestamp,
            ..player
        });
    } else {
        ctx.db.player().insert(Player {
            identity: ctx.sender(),
            display_name,
            avatar_color,
            client_protocol,
            online: true,
            joined_at: ctx.timestamp,
            updated_at: ctx.timestamp,
        });
    }

    if ctx
        .db
        .player_position()
        .identity()
        .find(ctx.sender())
        .is_none()
    {
        ctx.db.player_position().insert(PlayerPosition {
            identity: ctx.sender(),
            x: DEFAULT_SPAWN_X,
            y: DEFAULT_SPAWN_Y,
            last_dx: 0,
            last_dy: 0,
            updated_at: ctx.timestamp,
        });
    }

    Ok(())
}

#[reducer]
pub fn move_player(ctx: &ReducerContext, dx: i32, dy: i32) -> Result<(), String> {
    require_joined(ctx)?;

    let dx = dx.clamp(-1, 1);
    let dy = dy.clamp(-1, 1);
    let position = ctx
        .db
        .player_position()
        .identity()
        .find(ctx.sender())
        .ok_or_else(|| "Player position not found".to_string())?;

    ctx.db.player_position().identity().update(PlayerPosition {
        x: position.x + dx * MOVE_STEP,
        y: position.y + dy * MOVE_STEP,
        last_dx: dx,
        last_dy: dy,
        updated_at: ctx.timestamp,
        ..position
    });

    Ok(())
}

#[reducer]
pub fn send_chat(ctx: &ReducerContext, body: String) -> Result<(), String> {
    require_joined(ctx)?;

    let body = clean_chat_body(body)?;
    ctx.db.chat_message().insert(ChatMessage {
        id: 0,
        sender: ctx.sender(),
        body,
        sent_at: ctx.timestamp,
    });

    Ok(())
}

fn require_joined(ctx: &ReducerContext) -> Result<Player, String> {
    ctx.db
        .player()
        .identity()
        .find(ctx.sender())
        .ok_or_else(|| "Join the game before sending actions".to_string())
}

fn upsert_server_config(ctx: &ReducerContext, key: &str, value: String) {
    let key = key.to_string();
    if let Some(config) = ctx.db.server_config().key().find(&key) {
        ctx.db.server_config().key().update(ServerConfig {
            value,
            updated_at: ctx.timestamp,
            ..config
        });
    } else {
        ctx.db.server_config().insert(ServerConfig {
            key,
            value,
            updated_at: ctx.timestamp,
        });
    }
}

fn clean_display_name(display_name: String) -> Result<String, String> {
    let name = display_name.trim();
    if name.is_empty() {
        return Err("Display name cannot be empty".to_string());
    }
    if name.chars().count() > MAX_DISPLAY_NAME_CHARS {
        return Err(format!(
            "Display name must be {MAX_DISPLAY_NAME_CHARS} characters or fewer"
        ));
    }
    Ok(name.to_string())
}

fn clean_chat_body(body: String) -> Result<String, String> {
    let body = body.trim();
    if body.is_empty() {
        return Err("Chat message cannot be empty".to_string());
    }
    if body.chars().count() > MAX_CHAT_BODY_CHARS {
        return Err(format!(
            "Chat message must be {MAX_CHAT_BODY_CHARS} characters or fewer"
        ));
    }
    Ok(body.to_string())
}

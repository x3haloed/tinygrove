use spacetimedb::{Identity, ReducerContext, Table, Timestamp, reducer, table};

const SUPPORTED_CLIENT_PROTOCOL: u32 = 1;
const MOVE_STEP: i32 = 16;
const INTERACTION_REACH: i32 = MOVE_STEP;
const PLOT_SIZE: i32 = MOVE_STEP * 8;
const PLOT_GAP: i32 = MOVE_STEP * 4;
const PLOT_COLUMNS: i32 = 4;
const MAX_DISPLAY_NAME_CHARS: usize = 24;
const MAX_CHAT_BODY_CHARS: usize = 240;
const MAX_OBJECT_KIND_CHARS: usize = 24;
const MAX_TILE_KIND_CHARS: usize = 24;
const MAX_PLACE_RADIUS: i32 = MOVE_STEP * 8;

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

#[table(accessor = player_plot, public)]
pub struct PlayerPlot {
    #[primary_key]
    pub owner: Identity,
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: i32,
    pub height: i32,
    pub assigned_at: Timestamp,
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

#[table(accessor = world_object, public)]
pub struct WorldObject {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub kind: String,
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub state: i32,
    pub created_by: Identity,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[table(accessor = world_tile, public)]
pub struct WorldTile {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub kind: String,
    pub x: i32,
    pub y: i32,
    pub created_by: Identity,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    upsert_server_config(
        ctx,
        "supported_client_protocol",
        SUPPORTED_CLIENT_PROTOCOL.to_string(),
    );
    seed_world_objects(ctx);
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

    let plot = ensure_player_plot(ctx);

    if ctx
        .db
        .player_position()
        .identity()
        .find(ctx.sender())
        .is_none()
    {
        ctx.db.player_position().insert(PlayerPosition {
            identity: ctx.sender(),
            x: plot.origin_x + plot.width / 2,
            y: plot.origin_y + plot.height / 2,
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

#[reducer]
pub fn place_object(
    ctx: &ReducerContext,
    kind: String,
    target_x: i32,
    target_y: i32,
) -> Result<(), String> {
    require_joined(ctx)?;

    let kind = clean_object_kind(kind)?;
    let position = require_position(ctx)?;
    let target = (target_x, target_y);
    let plot = require_plot(ctx)?;

    if !within_place_radius(&position, target.0, target.1) {
        return Err(format!(
            "You can only place within {MAX_PLACE_RADIUS} world units of your current position"
        ));
    }

    if !plot_contains(&plot, target.0, target.1) {
        return Err("You can only place objects inside your plot".to_string());
    }

    if ctx
        .db
        .world_object()
        .iter()
        .any(|object| object.x == target.0 && object.y == target.1)
    {
        return Err("There is already something there".to_string());
    }

    ctx.db.world_object().insert(WorldObject {
        id: 0,
        kind,
        text: String::new(),
        x: target.0,
        y: target.1,
        state: 0,
        created_by: ctx.sender(),
        created_at: ctx.timestamp,
        updated_at: ctx.timestamp,
    });

    Ok(())
}

#[reducer]
pub fn place_tile(
    ctx: &ReducerContext,
    kind: String,
    target_x: i32,
    target_y: i32,
) -> Result<(), String> {
    require_joined(ctx)?;

    let kind = clean_tile_kind(kind)?;
    let position = require_position(ctx)?;
    let target = (target_x, target_y);
    let plot = require_plot(ctx)?;

    if !within_place_radius(&position, target.0, target.1) {
        return Err(format!(
            "You can only place within {MAX_PLACE_RADIUS} world units of your current position"
        ));
    }

    if !plot_contains(&plot, target.0, target.1) {
        return Err("You can only place tiles inside your plot".to_string());
    }

    if ctx
        .db
        .world_tile()
        .iter()
        .any(|tile| tile.x == target.0 && tile.y == target.1)
    {
        return Err("There is already a tile there".to_string());
    }

    ctx.db.world_tile().insert(WorldTile {
        id: 0,
        kind,
        x: target.0,
        y: target.1,
        created_by: ctx.sender(),
        created_at: ctx.timestamp,
        updated_at: ctx.timestamp,
    });

    Ok(())
}

#[reducer]
pub fn interact_near(ctx: &ReducerContext) -> Result<(), String> {
    let player = require_joined(ctx)?;
    let position = require_position(ctx)?;
    let target = interaction_target(&position);
    let object = ctx
        .db
        .world_object()
        .iter()
        .filter(|object| {
            (object.x - target.0).abs() <= INTERACTION_REACH
                && (object.y - target.1).abs() <= INTERACTION_REACH
        })
        .min_by_key(|object| (object.x - target.0).abs() + (object.y - target.1).abs())
        .ok_or_else(|| "There is nothing nearby to interact with".to_string())?;

    match object.kind.as_str() {
        "button" => {
            ctx.db.world_object().id().update(WorldObject {
                state: if object.state == 0 { 1 } else { 0 },
                updated_at: ctx.timestamp,
                ..object
            });
        }
        "sign" => {
            let text = if object.text.is_empty() {
                "Welcome to Tiny Grove.".to_string()
            } else {
                object.text.clone()
            };
            ctx.db.chat_message().insert(ChatMessage {
                id: 0,
                sender: ctx.sender(),
                body: format!("{} reads the sign: {}", player.display_name, text),
                sent_at: ctx.timestamp,
            });
        }
        _ => {
            ctx.db.chat_message().insert(ChatMessage {
                id: 0,
                sender: ctx.sender(),
                body: format!("{} inspects the {}.", player.display_name, object.kind),
                sent_at: ctx.timestamp,
            });
        }
    }

    Ok(())
}

fn require_joined(ctx: &ReducerContext) -> Result<Player, String> {
    ctx.db
        .player()
        .identity()
        .find(ctx.sender())
        .ok_or_else(|| "Join the game before sending actions".to_string())
}

fn require_position(ctx: &ReducerContext) -> Result<PlayerPosition, String> {
    ctx.db
        .player_position()
        .identity()
        .find(ctx.sender())
        .ok_or_else(|| "Player position not found".to_string())
}

fn require_plot(ctx: &ReducerContext) -> Result<PlayerPlot, String> {
    ctx.db
        .player_plot()
        .owner()
        .find(ctx.sender())
        .ok_or_else(|| "Player plot not found".to_string())
}

fn ensure_player_plot(ctx: &ReducerContext) -> PlayerPlot {
    if let Some(plot) = ctx.db.player_plot().owner().find(ctx.sender()) {
        return plot;
    }

    let index = ctx.db.player_plot().iter().count() as i32;
    let stride = PLOT_SIZE + PLOT_GAP;
    let column = index % PLOT_COLUMNS;
    let row = index / PLOT_COLUMNS;
    let origin_x = column * stride - stride;
    let origin_y = row * stride - stride;
    let owner = ctx.sender();
    ctx.db.player_plot().insert(PlayerPlot {
        owner,
        origin_x,
        origin_y,
        width: PLOT_SIZE,
        height: PLOT_SIZE,
        assigned_at: ctx.timestamp,
    });

    PlayerPlot {
        owner,
        origin_x,
        origin_y,
        width: PLOT_SIZE,
        height: PLOT_SIZE,
        assigned_at: ctx.timestamp,
    }
}

fn plot_contains(plot: &PlayerPlot, x: i32, y: i32) -> bool {
    x >= plot.origin_x
        && x < plot.origin_x + plot.width
        && y >= plot.origin_y
        && y < plot.origin_y + plot.height
}

fn within_place_radius(position: &PlayerPosition, x: i32, y: i32) -> bool {
    let dx = x - position.x;
    let dy = y - position.y;
    dx * dx + dy * dy <= MAX_PLACE_RADIUS * MAX_PLACE_RADIUS
}

fn interaction_target(position: &PlayerPosition) -> (i32, i32) {
    let (dx, dy) = if position.last_dx == 0 && position.last_dy == 0 {
        (0, 1)
    } else {
        (position.last_dx, position.last_dy)
    };
    (position.x + dx * MOVE_STEP, position.y + dy * MOVE_STEP)
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

fn seed_world_objects(ctx: &ReducerContext) {
    if ctx.db.world_object().iter().next().is_some() {
        return;
    }

    for (kind, x, y, state) in [
        ("sign", 0, -32, 0),
        ("button", 48, 0, 0),
        ("flower", -48, 32, 0),
        ("rock", 96, -32, 0),
    ] {
        ctx.db.world_object().insert(WorldObject {
            id: 0,
            kind: kind.to_string(),
            text: String::new(),
            x,
            y,
            state,
            created_by: ctx.sender(),
            created_at: ctx.timestamp,
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

fn clean_object_kind(kind: String) -> Result<String, String> {
    let kind = kind.trim().to_lowercase();
    if kind.is_empty() {
        return Err("Object kind cannot be empty".to_string());
    }
    if kind.chars().count() > MAX_OBJECT_KIND_CHARS {
        return Err(format!(
            "Object kind must be {MAX_OBJECT_KIND_CHARS} characters or fewer"
        ));
    }
    match kind.as_str() {
        "button" | "flower" | "rock" | "sign" => Ok(kind),
        _ => Err("Object kind must be button, flower, rock, or sign".to_string()),
    }
}

fn clean_tile_kind(kind: String) -> Result<String, String> {
    let kind = kind.trim().to_lowercase();
    if kind.is_empty() {
        return Err("Tile kind cannot be empty".to_string());
    }
    if kind.chars().count() > MAX_TILE_KIND_CHARS {
        return Err(format!(
            "Tile kind must be {MAX_TILE_KIND_CHARS} characters or fewer"
        ));
    }
    match kind.as_str() {
        "grass" | "path" | "water" | "dirt" => Ok(kind),
        _ => Err("Tile kind must be grass, path, water, or dirt".to_string()),
    }
}

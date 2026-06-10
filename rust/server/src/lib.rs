use base64::Engine;
use spacetimedb::{Identity, ReducerContext, Table, Timestamp, reducer, table};

const SUPPORTED_CLIENT_PROTOCOL: u32 = 1;
const MOVE_STEP: i32 = 16;
const INTERACTION_REACH: i32 = MOVE_STEP;
const PLOT_SIZE: i32 = MOVE_STEP * 8;
const PLOT_GAP: i32 = MOVE_STEP * 4;
const PLOT_COLUMNS: i32 = 4;
const MAX_DISPLAY_NAME_CHARS: usize = 24;
const MAX_CHAT_BODY_CHARS: usize = 240;
const MAX_ASSET_NAME_CHARS: usize = 32;
const MAX_ASSET_SLUG_CHARS: usize = 32;
const MAX_ASSET_STATUS_CHARS: usize = 16;
const MAX_ASSET_FORMAT_CHARS: usize = 16;
const MAX_ASSET_DATA_CHARS: usize = 262_144;
const MAX_PLACE_RADIUS: i32 = MOVE_STEP * 8;
const TILE_GRID_DIVISOR: i32 = 4;
const TILE_PLACEMENT_W: i32 = 4;
const TILE_PLACEMENT_H: i32 = 4;
const DECORATION_PLACEMENT_FULL: &str = "Full";
const DECORATION_PLACEMENT_HALF: &str = "Half Width";
const DECORATION_PLACEMENT_QUARTER: &str = "Quarter";

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

#[table(accessor = world_tile, public)]
pub struct WorldTile {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub kind: String,
    pub text: String,
    pub state: i32,
    pub x: i32,
    pub y: i32,
    pub created_by: Identity,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[table(accessor = content_asset, public)]
pub struct ContentAsset {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub owner: Identity,
    pub asset_kind: String,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub grid_divisor: i32,
    pub placement_w: i32,
    pub placement_h: i32,
    pub anchor_x: i32,
    pub anchor_y: i32,
    pub collidable: bool,
    pub transparent_allowed: bool,
    pub render_format: String,
    pub render_bytes: String,
    pub collision_format: String,
    pub collision_bytes: String,
    pub preview_format: String,
    pub preview_bytes: String,
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
    seed_content_assets(ctx);
}

#[reducer]
pub fn repair_content_assets(ctx: &ReducerContext) -> Result<(), String> {
    seed_content_assets(ctx);
    Ok(())
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
    seed_content_assets(ctx);
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
pub fn place_tile(
    ctx: &ReducerContext,
    kind: String,
    target_x: i32,
    target_y: i32,
) -> Result<(), String> {
    require_joined(ctx)?;

    let kind = clean_asset_slug(kind)?;
    let asset = ctx
        .db
        .content_asset()
        .iter()
        .find(|asset| asset.slug == kind)
        .ok_or_else(|| format!("Content asset with slug '{kind}' not found"))?;
    if asset.status != "published" {
        return Err(format!(
            "Content asset with slug '{kind}' is not published (status: {})",
            asset.status
        ));
    }
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

    if ctx.db.world_tile().iter().any(|tile| tile.x == target.0 && tile.y == target.1) {
        return Err("There is already a tile there".to_string());
    }

    ctx.db.world_tile().insert(WorldTile {
        id: 0,
        kind,
        text: String::new(),
        state: 0,
        x: target.0,
        y: target.1,
        created_by: ctx.sender(),
        created_at: ctx.timestamp,
        updated_at: ctx.timestamp,
    });

    Ok(())
}

#[reducer]
pub fn create_content_asset(
    ctx: &ReducerContext,
    asset_kind: String,
    name: String,
    slug: String,
    status: String,
    grid_divisor: i32,
    placement_variant: String,
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
) -> Result<(), String> {
    require_joined(ctx)?;

    let asset_kind = clean_asset_kind(asset_kind)?;
    let name = clean_asset_name(name)?;
    let slug = clean_asset_slug(slug)?;
    if ctx.db.content_asset().iter().any(|asset| asset.slug == slug) {
        return Err(format!("Content asset with slug '{slug}' already exists"));
    }
    let status = clean_asset_status(status)?;
    let render_format = clean_asset_format(render_format)?;
    let collision_format = clean_asset_format(collision_format)?;
    let preview_format = clean_asset_format(preview_format)?;
    let render_bytes = clean_asset_data(render_bytes)?;
    let collision_bytes = clean_asset_data(collision_bytes)?;
    let preview_bytes = clean_asset_data(preview_bytes)?;

    if grid_divisor <= 0 {
        return Err("Grid divisor must be positive".to_string());
    }
    let (placement_w, placement_h, collidable, transparent_allowed) =
        normalize_asset_shape(&asset_kind, placement_variant, collidable, transparent_allowed)?;

    ctx.db.content_asset().insert(ContentAsset {
        id: 0,
        owner: ctx.sender(),
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
        created_at: ctx.timestamp,
        updated_at: ctx.timestamp,
    });

    Ok(())
}

#[reducer]
pub fn update_content_asset(
    ctx: &ReducerContext,
    asset_id: u64,
    name: String,
    slug: String,
    status: String,
    grid_divisor: i32,
    placement_variant: String,
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
) -> Result<(), String> {
    require_joined(ctx)?;

    let existing = ctx
        .db
        .content_asset()
        .id()
        .find(&asset_id)
        .ok_or_else(|| "Content asset not found".to_string())?;
    if existing.owner != ctx.sender() {
        return Err("You can only edit your own content assets".to_string());
    }

    let name = clean_asset_name(name)?;
    let slug = clean_asset_slug(slug)?;
    if ctx
        .db
        .content_asset()
        .iter()
        .any(|asset| asset.slug == slug && asset.id != asset_id)
    {
        return Err(format!("Content asset with slug '{slug}' already exists"));
    }
    let status = clean_asset_status(status)?;
    let render_format = clean_asset_format(render_format)?;
    let collision_format = clean_asset_format(collision_format)?;
    let preview_format = clean_asset_format(preview_format)?;
    let render_bytes = clean_asset_data(render_bytes)?;
    let collision_bytes = clean_asset_data(collision_bytes)?;
    let preview_bytes = clean_asset_data(preview_bytes)?;
    let (placement_w, placement_h, collidable, transparent_allowed) = normalize_asset_shape(
        &existing.asset_kind,
        placement_variant,
        collidable,
        transparent_allowed,
    )?;

    ctx.db.content_asset().id().update(ContentAsset {
        id: asset_id,
        owner: existing.owner,
        name,
        slug,
        status,
        asset_kind: existing.asset_kind,
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
        created_at: existing.created_at,
        updated_at: ctx.timestamp,
    });

    Ok(())
}

fn normalize_asset_shape(
    asset_kind: &str,
    placement_variant: String,
    collidable: bool,
    transparent_allowed: bool,
) -> Result<(i32, i32, bool, bool), String> {
    match asset_kind {
        "tile" => {
            if placement_variant != DECORATION_PLACEMENT_FULL {
                return Err("Tiles must use the full tile footprint".to_string());
            }
            if !collidable {
                return Err("Tiles must be collidable".to_string());
            }
            if transparent_allowed {
                return Err("Tiles cannot allow transparent pixels".to_string());
            }
            Ok((TILE_PLACEMENT_W, TILE_PLACEMENT_H, true, false))
        }
        "decoration" => {
            let (placement_w, placement_h) = match placement_variant.as_str() {
                DECORATION_PLACEMENT_FULL => (TILE_PLACEMENT_W, TILE_PLACEMENT_H),
                DECORATION_PLACEMENT_HALF => (2, 4),
                DECORATION_PLACEMENT_QUARTER => (2, 2),
                _ => {
                    return Err(format!(
                        "Unsupported decoration placement variant {placement_variant}"
                    ));
                }
            };
            Ok((placement_w, placement_h, collidable, transparent_allowed))
        }
        _ => Err(format!("Unsupported asset kind {asset_kind}")),
    }
}

#[reducer]
pub fn interact_near(ctx: &ReducerContext) -> Result<(), String> {
    let player = require_joined(ctx)?;
    let position = require_position(ctx)?;
    let target = interaction_target(&position);
    let object = ctx
        .db
        .world_tile()
        .iter()
        .filter(|object| {
            (object.x - target.0).abs() <= INTERACTION_REACH
                && (object.y - target.1).abs() <= INTERACTION_REACH
        })
        .min_by_key(|object| (object.x - target.0).abs() + (object.y - target.1).abs())
        .ok_or_else(|| "There is nothing nearby to interact with".to_string())?;

    match object.kind.as_str() {
        "button" => {
            ctx.db.world_tile().id().update(WorldTile {
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

fn seed_content_assets(ctx: &ReducerContext) {
    if ctx.db.content_asset().iter().next().is_some() {
        return;
    }

    for (asset_kind, name, slug, status, grid_divisor, placement_variant, collidable, transparent_allowed, render_bytes) in [
        ("tile", "Grass", "grass", "published", TILE_GRID_DIVISOR, DECORATION_PLACEMENT_FULL, true, false, asset_png_base64("grass")),
        ("tile", "Path", "path", "published", TILE_GRID_DIVISOR, DECORATION_PLACEMENT_FULL, true, false, asset_png_base64("path")),
        ("tile", "Water", "water", "published", TILE_GRID_DIVISOR, DECORATION_PLACEMENT_FULL, true, false, asset_png_base64("water")),
        ("tile", "Dirt", "dirt", "published", TILE_GRID_DIVISOR, DECORATION_PLACEMENT_FULL, true, false, asset_png_base64("dirt")),
        ("decoration", "Flower", "flower", "published", TILE_GRID_DIVISOR, DECORATION_PLACEMENT_QUARTER, false, true, asset_png_base64("flower")),
        ("decoration", "Button", "button", "published", TILE_GRID_DIVISOR, DECORATION_PLACEMENT_HALF, true, true, asset_png_base64("button")),
        ("decoration", "Sign", "sign", "published", TILE_GRID_DIVISOR, DECORATION_PLACEMENT_FULL, true, true, asset_png_base64("sign")),
        ("decoration", "Rock", "rock", "published", TILE_GRID_DIVISOR, DECORATION_PLACEMENT_FULL, true, true, asset_png_base64("rock")),
    ] {
        let (placement_w, placement_h, collidable, transparent_allowed) = normalize_asset_shape(
            asset_kind,
            placement_variant.to_string(),
            collidable,
            transparent_allowed,
        )
        .expect("seed content asset shape should be valid");
        ctx.db.content_asset().insert(ContentAsset {
            id: 0,
            owner: ctx.sender(),
            asset_kind: asset_kind.to_string(),
            name: name.to_string(),
            slug: slug.to_string(),
            status: status.to_string(),
            grid_divisor,
            placement_w,
            placement_h,
            anchor_x: 0,
            anchor_y: 0,
            collidable,
            transparent_allowed,
            render_format: "png".to_string(),
            render_bytes,
            collision_format: "mask1".to_string(),
            collision_bytes: String::new(),
            preview_format: "png".to_string(),
            preview_bytes: asset_png_base64(slug),
            created_at: ctx.timestamp,
            updated_at: ctx.timestamp,
        });
    }
}

fn asset_png_base64(name: &str) -> String {
    let bytes = match name {
        "grass" => include_bytes!("../assets/content_assets/grass.png").as_slice(),
        "path" => include_bytes!("../assets/content_assets/path.png").as_slice(),
        "water" => include_bytes!("../assets/content_assets/water.png").as_slice(),
        "dirt" => include_bytes!("../assets/content_assets/dirt.png").as_slice(),
        "flower" => include_bytes!("../assets/content_assets/flower.png").as_slice(),
        "button" => include_bytes!("../assets/content_assets/button.png").as_slice(),
        "sign" => include_bytes!("../assets/content_assets/sign.png").as_slice(),
        "rock" => include_bytes!("../assets/content_assets/rock.png").as_slice(),
        _ => &[],
    };
    base64::engine::general_purpose::STANDARD.encode(bytes)
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

fn clean_asset_kind(kind: String) -> Result<String, String> {
    let kind = kind.trim().to_lowercase();
    match kind.as_str() {
        "tile" | "decoration" => Ok(kind),
        _ => Err("Asset kind must be tile or decoration".to_string()),
    }
}

fn clean_asset_name(name: String) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Asset name cannot be empty".to_string());
    }
    if name.chars().count() > MAX_ASSET_NAME_CHARS {
        return Err(format!(
            "Asset name must be {MAX_ASSET_NAME_CHARS} characters or fewer"
        ));
    }
    Ok(name.to_string())
}

fn clean_asset_slug(slug: String) -> Result<String, String> {
    let slug = slug.trim().to_lowercase();
    if slug.is_empty() {
        return Err("Asset slug cannot be empty".to_string());
    }
    if slug.chars().count() > MAX_ASSET_SLUG_CHARS {
        return Err(format!(
            "Asset slug must be {MAX_ASSET_SLUG_CHARS} characters or fewer"
        ));
    }
    Ok(slug)
}

fn clean_asset_status(status: String) -> Result<String, String> {
    let status = status.trim().to_lowercase();
    if status.chars().count() > MAX_ASSET_STATUS_CHARS {
        return Err(format!(
            "Asset status must be {MAX_ASSET_STATUS_CHARS} characters or fewer"
        ));
    }
    match status.as_str() {
        "draft" | "published" | "archived" => Ok(status),
        _ => Err("Asset status must be draft, published, or archived".to_string()),
    }
}

fn clean_asset_format(format: String) -> Result<String, String> {
    let format = format.trim().to_lowercase();
    if format.is_empty() {
        return Err("Asset format cannot be empty".to_string());
    }
    if format.chars().count() > MAX_ASSET_FORMAT_CHARS {
        return Err(format!(
            "Asset format must be {MAX_ASSET_FORMAT_CHARS} characters or fewer"
        ));
    }
    Ok(format)
}

fn clean_asset_data(data: String) -> Result<String, String> {
    if data.chars().count() > MAX_ASSET_DATA_CHARS {
        return Err(format!(
            "Asset data must be {MAX_ASSET_DATA_CHARS} characters or fewer"
        ));
    }
    Ok(data)
}

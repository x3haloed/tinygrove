extends Control

const TILE_SIZE := 32
const AVATAR_HALF_WIDTH := 8.0
const AVATAR_HEIGHT := 22.0
const MOVE_REPEAT_SECONDS := 0.10
const AVATAR_SMOOTH_SPEED := 18.0
const CAMERA_SMOOTH_SPEED := 10.0
const CAMERA_DEADZONE := Vector2(220.0, 120.0)
const SMOKE_JOIN_FRAME := 6
const SMOKE_MOVE_FRAME := 18
const SMOKE_CHAT_FRAME := 30
const SMOKE_PLACE_FRAME := 42
const SMOKE_INTERACT_FRAME := 54
const STATUS_MAX_CHARS := 92
const BUBBLE_VISIBLE_SECONDS := 3.5
const BUBBLE_FADE_SECONDS := 1.1
const BUBBLE_MAX_CHARS := 28
const AGENT_HTTP_HOST := "127.0.0.1"
const AGENT_HTTP_PORT := 37373
const AGENT_HTTP_PORT_SCAN_COUNT := 32
const AGENT_REGISTRY_RELATIVE_DIR := "../.tinygrove/agents"
const AGENT_REGISTRY_WRITE_SECONDS := 1.0
const AGENT_MAX_INDIVIDUAL_OBJECTS := 40
const AGENT_MAX_DELTA_EVENTS := 24
const AGENT_MAX_STORED_EVENTS := 240
const AGENT_MAX_MOVE_STEPS := 12
const AGENT_ACTION_SETTLE_MSEC := 180
const AGENT_ACTION_POLL_MSEC := 15
const AGENT_DEFAULT_SCREENSHOT_MAX := Vector2i(1024, 768)
const AGENT_BIGGER_SCREENSHOT_MAX := Vector2i(1280, 720)
const AGENT_MAX_SCREENSHOT_MAX := Vector2i(1920, 1080)
const PLACE_RADIUS_TILES := 8
const PLACE_RADIUS_PIXELS := TILE_SIZE * PLACE_RADIUS_TILES
const PLACE_PREVIEW_SNAP := TILE_SIZE

@onready var world: Node2D = $World
@onready var status_label: Label = $Hud/VBox/Status
@onready var name_edit: LineEdit = $Hud/VBox/NameRow/NameEdit
@onready var join_button: Button = $Hud/VBox/NameRow/JoinButton
@onready var recent_label: Label = $Hud/VBox/Recent
@onready var chat_edit: LineEdit = $ChatInput/ChatRow/ChatEdit
@onready var send_button: Button = $ChatInput/ChatRow/SendButton
@onready var library_overlay: PanelContainer = $LibraryOverlay
@onready var library_close_button: Button = $LibraryOverlay/LibraryRoot/LibraryHeader/LibraryClose
@onready var library_tile_tab: Button = $LibraryOverlay/LibraryRoot/LibraryTabs/TileTab
@onready var library_object_tab: Button = $LibraryOverlay/LibraryRoot/LibraryTabs/ObjectTab
@onready var library_grid: GridContainer = $LibraryOverlay/LibraryRoot/LibraryBody/LibraryGridPanel/LibraryGridScroll/LibraryGrid
@onready var library_preview_title: Label = $LibraryOverlay/LibraryRoot/LibraryBody/LibraryPreviewPanel/LibraryPreviewRoot/LibraryPreviewTitle
@onready var library_preview_desc: Label = $LibraryOverlay/LibraryRoot/LibraryBody/LibraryPreviewPanel/LibraryPreviewRoot/LibraryPreviewDesc
@onready var library_preview_canvas: Control = $LibraryOverlay/LibraryRoot/LibraryBody/LibraryPreviewPanel/LibraryPreviewRoot/LibraryPreviewBox/LibraryPreviewCanvas
@onready var library_place_button: Button = $LibraryOverlay/LibraryRoot/LibraryBody/LibraryPreviewPanel/LibraryPreviewRoot/LibraryActions/LibraryPlaceButton
@onready var library_cancel_button: Button = $LibraryOverlay/LibraryRoot/LibraryBody/LibraryPreviewPanel/LibraryPreviewRoot/LibraryActions/LibraryCancelButton

var client: RefCounted
var avatars: Dictionary = {}
var world_tiles: Dictionary = {}
var player_plots: Dictionary = {}
var latest_chat_by_sender: Dictionary = {}
var chat_messages_seen: Dictionary = {}
var chat_bubbles_by_sender: Dictionary = {}
var local_identity := ""
var camera_position := Vector2.ZERO
var camera_initialized := false
var place_mode := false
var place_layer := "tile"
var place_kind := "flower"
var library_open := false
var library_category := "tile"
var library_selected_kind := "grass"
var place_target := Vector2.ZERO
var place_target_clamped := Vector2.ZERO
var place_target_valid := false
var pixel_editor: Control
var content_assets_by_kind: Dictionary = {}
var content_asset_textures: Dictionary = {}
var content_asset_preview_textures: Dictionary = {}
var move_elapsed := 0.0
var frame := 0
var smoke_enabled := false
var smoke_name := ""
var smoke_message := ""
var smoke_dx := 0
var smoke_dy := 0
var smoke_object_kind := ""
var agent_server := TCPServer.new()
var agent_connections: Array[Dictionary] = []
var agent_http_status := ""
var agent_http_port := 0
var agent_profile := "human"
var agent_registry_path := ""
var agent_registry_elapsed := 0.0
var agent_event_cursor := 0
var agent_last_seen_cursor := 0
var agent_events: Array[Dictionary] = []
var agent_player_state: Dictionary = {}
var agent_object_state: Dictionary = {}
var agent_plot_state: Dictionary = {}
var agent_seen_chat_ids: Dictionary = {}
var agent_baseline_ready := false

func _ready() -> void:
	world.y_sort_enabled = true
	var ground := GroundNode.new()
	ground.z_index = -1000
	world.add_child(ground)
	var plots := PlotLayerNode.new()
	plots.z_index = -900
	world.add_child(plots)
	_style_ui()
	join_button.pressed.connect(_join_game)
	send_button.pressed.connect(_send_chat)
	library_close_button.pressed.connect(_close_library)
	library_place_button.pressed.connect(_library_activate_selection)
	library_cancel_button.pressed.connect(_close_library)
	library_tile_tab.pressed.connect(func() -> void:
		library_category = "tile"
		library_selected_kind = "grass"
		_update_library_tabs()
	)
	library_object_tab.pressed.connect(func() -> void:
		library_category = "object"
		library_selected_kind = "flower"
		_update_library_tabs()
	)
	chat_edit.text_submitted.connect(func(_text: String) -> void: _send_chat())
	_load_smoke_config()
	_load_agent_config()
	if not ClassDB.class_exists("TinyGroveClient"):
		GDExtensionManager.load_extension("res://tinygrove_client.gdextension")
	client = ClassDB.instantiate("TinyGroveClient")
	if client != null:
		client.connect_local_with_profile(agent_profile)
	_start_agent_http()
	
	var editor_scene := preload("res://scenes/pixel_editor.tscn")
	pixel_editor = editor_scene.instantiate()
	pixel_editor.visible = false
	add_child(pixel_editor)
	pixel_editor.save_requested.connect(_on_pixel_editor_save)

func _process(delta: float) -> void:
	if client == null:
		_poll_agent_http()
		return
	frame += 1
	client.poll()
	_refresh_content_asset_cache()
	_update_agent_registry(delta)
	_run_smoke_actions()
	_update_status()
	_update_chat()
	_update_world()
	_update_plots()
	_update_objects()
	_update_camera(delta)
	_update_place_preview()
	_poll_agent_http()
	_handle_interaction()
	_handle_movement(delta)

func _input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		_update_place_preview()
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_TAB:
		_toggle_library()
		get_viewport().set_input_as_handled()
		return
	if event is InputEventMouseButton and event.pressed:
		if place_mode and event.button_index == MOUSE_BUTTON_LEFT:
			_attempt_place_selected()
		elif place_mode and event.button_index == MOUSE_BUTTON_RIGHT:
			_cancel_place_mode()
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_F:
			_toggle_place_mode("tile", "flower")
		elif event.keycode == KEY_T:
			_toggle_place_mode("tile", "grass")
		elif event.keycode == KEY_B:
			_toggle_place_mode("tile", "button")
		elif event.keycode == KEY_P:
			pixel_editor.visible = not pixel_editor.visible
			if pixel_editor.visible:
				_cancel_place_mode()
				_close_library()
			get_viewport().set_input_as_handled()
		elif event.keycode == KEY_ESCAPE and place_mode:
			_cancel_place_mode()
		elif event.keycode == KEY_ESCAPE and library_open:
			_close_library()
		elif event.keycode == KEY_ESCAPE and pixel_editor.visible:
			pixel_editor.visible = false

func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST or what == NOTIFICATION_PREDELETE:
		_remove_agent_registry()

func _join_game() -> void:
	var color := Color.from_hsv(randf(), 0.58, 0.9).to_rgba32()
	client.join_game(name_edit.text, color)

func _send_chat() -> void:
	var body := chat_edit.text.strip_edges()
	if body.is_empty():
		return
	if client.send_chat(body):
		chat_edit.clear()

func _handle_movement(delta: float) -> void:
	move_elapsed += delta
	if move_elapsed < MOVE_REPEAT_SECONDS:
		return

	var dx := Input.get_axis("move_left", "move_right")
	var dy := Input.get_axis("move_up", "move_down")
	var ix := int(sign(dx))
	var iy := int(sign(dy))
	if ix == 0 and iy == 0:
		return

	move_elapsed = 0.0
	client.move_player(ix, iy)

func _handle_interaction() -> void:
	if Input.is_action_just_pressed("interact"):
		client.interact_near()

func _toggle_place_mode(layer: String, kind: String) -> void:
	if place_mode and place_layer == layer and place_kind == kind:
		_cancel_place_mode()
		return
	place_mode = true
	place_layer = layer
	place_kind = kind
	_update_place_preview()
	queue_redraw()

func _cancel_place_mode() -> void:
	place_mode = false
	place_target_valid = false
	queue_redraw()

func _toggle_library() -> void:
	library_open = not library_open
	library_overlay.visible = library_open
	if library_open:
		_update_library_tabs()
		_update_library_selection(library_selected_kind)
	else:
		library_grid.mouse_filter = Control.MOUSE_FILTER_IGNORE

func _close_library() -> void:
	library_open = false
	library_overlay.visible = false

func _update_library_overlay() -> void:
	library_overlay.visible = library_open
	if not library_open:
		return
	_update_library_tabs()
	_update_library_preview()

func _update_library_tabs() -> void:
	library_tile_tab.button_pressed = library_category == "tile"
	library_object_tab.button_pressed = library_category == "object"
	var entries := _library_entries_for(library_category)
	if entries.is_empty():
		library_selected_kind = ""
	elif _library_entry_for(library_category, library_selected_kind).is_empty():
		library_selected_kind = str(entries[0]["kind"])
	_refresh_library_grid()

func _refresh_library_grid() -> void:
	for child in library_grid.get_children():
		child.queue_free()
	var entries := _library_entries_for(library_category)
	for entry in entries:
		var button := Button.new()
		button.text = str(entry["name"])
		button.toggle_mode = true
		button.button_pressed = str(entry["kind"]) == library_selected_kind
		button.focus_mode = Control.FOCUS_ALL
		button.custom_minimum_size = Vector2(120, 72)
		button.pressed.connect(func() -> void:
			_update_library_selection(str(entry["kind"]))
		)
		library_grid.add_child(button)

func _update_library_selection(kind: String) -> void:
	library_selected_kind = kind
	_update_library_preview()
	_refresh_library_grid()

func _update_library_preview() -> void:
	var entry := _library_entry_for(library_category, library_selected_kind)
	if entry.is_empty():
		library_preview_title.text = "Select an item"
		library_preview_desc.text = ""
		library_preview_canvas.queue_redraw()
		return
	library_preview_title.text = str(entry["name"])
	library_preview_desc.text = str(entry["description"])
	library_preview_canvas.queue_redraw()

func _draw_library_preview() -> void:
	var entry := _library_entry_for(library_category, library_selected_kind)
	if entry.is_empty():
		return
	var kind := str(entry["kind"])
	var preview_rect := library_preview_canvas.get_rect().grow(-24.0)
	var center := preview_rect.get_center()
	draw_rect(Rect2(center - Vector2(112, 112), Vector2(224, 224)), Color(0.09, 0.10, 0.12, 0.90))
	var texture := _content_asset_preview_texture(kind)
	if texture != null:
		var tex_size := Vector2(texture.get_width(), texture.get_height())
		var scale := minf(192.0 / maxf(tex_size.x, 1.0), 192.0 / maxf(tex_size.y, 1.0))
		var draw_size := tex_size * scale
		draw_texture_rect(texture, Rect2(center - draw_size * 0.5, draw_size), false)
	if library_category == "object" and kind == "button":
		draw_rect(Rect2(center + Vector2(-34, 34), Vector2(68, 8)), Color(0.12, 0.10, 0.08, 0.40))
		draw_rect(Rect2(center + Vector2(-18, -18), Vector2(36, 16)), Color(0.15, 0.70, 0.35, 0.22))
	draw_rect(Rect2(center - Vector2(64, 64), Vector2(128, 128)), Color(0.95, 0.92, 0.80), false, 4.0)
	draw_string(ThemeDB.fallback_font, center + Vector2(-54, 96), str(entry["name"]), HORIZONTAL_ALIGNMENT_LEFT, 180, 18, Color(0.96, 0.93, 0.81))

func _library_entry_for(category: String, kind: String) -> Dictionary:
	var entries := _library_entries_for(category)
	for entry in entries:
		if str(entry["kind"]) == kind:
			return entry
	return {}

func _library_entries_for(category: String) -> Array:
	var assets: Array = content_assets_by_kind.get(category, [])
	var entries: Array = []
	for asset in assets:
		var key := str(asset.get("slug", ""))
		if key.is_empty():
			key = str(asset.get("name", "")).to_lower()
		entries.append({
			"kind": key,
			"name": str(asset.get("name", "")),
			"description": _content_asset_description(asset),
			"button_pressed": key == "button",
		})
	return entries

func _content_asset_description(asset: Dictionary) -> String:
	var asset_kind := str(asset.get("asset_kind", ""))
	var placement := str(asset.get("placement_variant", ""))
	if asset_kind == "tile":
		return "Tile asset, %s placement." % placement.to_lower()
	if asset_kind == "decoration":
		return "Decoration asset, %s placement." % placement.to_lower()
	return "Content asset."

func _refresh_content_asset_cache() -> void:
	if client == null:
		return
	var rows: Array = client.content_assets()
	var next_by_kind := {"tile": [], "object": []}
	var next_textures := {}
	var next_preview_textures := {}
	for row in rows:
		var asset_kind := str(row.get("asset_kind", ""))
		var group := "tile" if asset_kind == "tile" else "object" if asset_kind == "decoration" else ""
		if group.is_empty():
			continue
		next_by_kind[group].append(row)
		var kind := str(row.get("slug", ""))
		if kind.is_empty():
			kind = str(row.get("name", "")).to_lower()
		var render_texture := _texture_from_base64(str(row.get("render_bytes", "")))
		if render_texture != null:
			next_textures[kind] = render_texture
		var preview_texture := _texture_from_base64(str(row.get("preview_bytes", "")))
		if preview_texture == null:
			preview_texture = render_texture
		if preview_texture != null:
			next_preview_textures[kind] = preview_texture
	content_assets_by_kind = next_by_kind
	content_asset_textures = next_textures
	content_asset_preview_textures = next_preview_textures

func _content_asset_texture(kind: String) -> Texture2D:
	if content_asset_textures.has(kind):
		return content_asset_textures[kind]
	return null

func _texture_from_base64(encoded: String) -> Texture2D:
	if encoded.is_empty():
		return null
	var bytes := Marshalls.base64_to_raw(encoded)
	if bytes.is_empty():
		return null
	var image := Image.new()
	var error := image.load_png_from_buffer(bytes)
	if error != OK:
		return null
	return ImageTexture.create_from_image(image)

func _content_asset_preview_texture(kind: String) -> Texture2D:
	if content_asset_preview_textures.has(kind):
		return content_asset_preview_textures[kind]
	return null

func _tile_preview_color(kind: String) -> Color:
	match kind:
		"path":
			return Color(0.69, 0.59, 0.39)
		"water":
			return Color(0.22, 0.48, 0.74)
		"dirt":
			return Color(0.47, 0.34, 0.20)
		_:
			return Color(0.35, 0.56, 0.32)

func _library_activate_selection() -> void:
	if library_open:
		if library_category == "tile":
			_toggle_place_mode("tile", library_selected_kind)
		else:
			_toggle_place_mode("tile", library_selected_kind)
		_close_library()

func _on_pixel_editor_save(data: Dictionary) -> void:
	if client == null: return
	
	# Add the extra defaults expected by the dictionary parsing
	data["status"] = "published"
	data["anchor_x"] = 0
	data["anchor_y"] = 0
	
	client.create_content_asset(data)
	pixel_editor.visible = false

func _update_place_preview() -> void:
	if not place_mode or local_identity.is_empty() or not avatars.has(local_identity):
		place_target_valid = false
		queue_redraw()
		return

	var local_avatar: AvatarNode = avatars[local_identity]
	var cursor_world := _screen_to_world(get_viewport().get_mouse_position())
	var clamped := _clamp_to_place_radius(local_avatar.world_target, cursor_world)
	place_target = cursor_world
	place_target_clamped = _snap_to_tile_center(clamped)
	place_target_valid = _is_place_target_valid(local_avatar.world_target, place_target_clamped)
	queue_redraw()

func _attempt_place_selected() -> void:
	if not place_mode:
		return
	if not place_target_valid:
		return
	var sent: bool = false
	sent = client.place_tile(place_kind, int(place_target_clamped.x), int(place_target_clamped.y))
	if sent:
		_cancel_place_mode()

func _screen_to_world(screen_point: Vector2) -> Vector2:
	return screen_point - world.position

func _world_to_screen(world_point: Vector2) -> Vector2:
	return world.position + world_point

func _snap_to_tile_center(world_point: Vector2) -> Vector2:
	return Vector2(
		floor(world_point.x / TILE_SIZE) * TILE_SIZE + TILE_SIZE * 0.5,
		floor(world_point.y / TILE_SIZE) * TILE_SIZE + TILE_SIZE * 0.5,
	)

func _clamp_to_place_radius(origin: Vector2, target: Vector2) -> Vector2:
	var delta := target - origin
	if delta.length() <= PLACE_RADIUS_PIXELS:
		return target
	return origin + delta.normalized() * PLACE_RADIUS_PIXELS

func _is_place_target_valid(origin: Vector2, target: Vector2) -> bool:
	return origin.distance_to(target) <= PLACE_RADIUS_PIXELS + 0.01

func _update_status() -> void:
	var text: String = client.status()
	var error: String = client.last_error()
	if not error.is_empty():
		text += " | " + _short_error(error)
	if place_mode:
		text += " | placing %s %s within %d tiles" % [place_layer, place_kind, PLACE_RADIUS_TILES]
	status_label.text = text

func _update_world() -> void:
	var seen := {}
	local_identity = str(client.local_identity())
	var record_events := agent_baseline_ready and _agent_is_logged_in()
	for row in client.players():
		var identity := str(row["identity"])
		seen[identity] = true
		var target := Vector2(float(row["x"]), float(row["y"]))
		var display_name := str(row["display_name"])
		var last_dx := int(row.get("last_dx", 0))
		var last_dy := int(row.get("last_dy", 0))
		var previous: Dictionary = agent_player_state.get(identity, {})
		var state := {
			"display_name": display_name,
			"position": target,
			"last_dx": last_dx,
			"last_dy": last_dy,
			"online": bool(row.get("online", true)),
			"updated_at_micros": int(row.get("updated_at_micros", 0)),
		}
		if record_events:
			if previous.is_empty():
				_agent_record_event("player_joined", "%s appeared at %s." % [display_name, _agent_point_text(target)], target, {
					"identity": identity,
					"display_name": display_name,
					"position": _agent_point(target),
				})
			else:
				var old_position: Vector2 = previous["position"]
				var old_name := str(previous["display_name"])
				if old_position != target:
					_agent_record_event("player_moved", "%s moved from %s to %s, now facing %s." % [display_name, _agent_point_text(old_position), _agent_point_text(target), _agent_facing_phrase(last_dx, last_dy)], target, {
						"identity": identity,
						"display_name": display_name,
						"from": _agent_point(old_position),
						"to": _agent_point(target),
						"facing": _agent_facing_phrase(last_dx, last_dy),
					})
				if old_name != display_name:
					_agent_record_event("player_renamed", "%s is now named %s." % [old_name, display_name], target, {
						"identity": identity,
						"old_display_name": old_name,
						"display_name": display_name,
						"position": _agent_point(target),
					})
		agent_player_state[identity] = state
		var avatar := _avatar_for(identity)
		avatar.set_world_target(target)
		avatar.set_meta("display_name", display_name)
		avatar.set_meta("avatar_color", int(row["avatar_color"]))
		avatar.set_meta("identity", identity)
		avatar.set_meta("is_local", identity == local_identity)
		avatar.set_meta("last_dx", last_dx)
		avatar.set_meta("last_dy", last_dy)
		var bubble_info: Dictionary = chat_bubbles_by_sender.get(identity, {})
		avatar.set_meta("bubble_body", str(bubble_info.get("body", "")))
		avatar.set_meta("bubble_started_at", float(bubble_info.get("started_at", 0.0)))
		avatar.queue_redraw()

	for identity in avatars.keys():
		if not seen.has(identity):
			if record_events and agent_player_state.has(identity):
				var previous: Dictionary = agent_player_state[identity]
				var previous_position: Vector2 = previous["position"]
				var previous_name := str(previous["display_name"])
				_agent_record_event("player_left", "%s left the visible world state." % previous_name, previous_position, {
					"identity": identity,
					"display_name": previous_name,
					"last_position": _agent_point(previous_position),
				})
			agent_player_state.erase(identity)
			avatars[identity].queue_free()
			avatars.erase(identity)

func _update_objects() -> void:
	var seen_tiles := {}
	var record_events := agent_baseline_ready and _agent_is_logged_in()
	for row in client.world_tiles():
		var id := int(row["id"])
		seen_tiles[id] = true
		var tile := _tile_for(id)
		var tile_position := Vector2(float(row["x"]), float(row["y"]))
		var kind := str(row["kind"])
		var state := int(row.get("state", 0))
		tile.position = tile_position
		tile.set_meta("kind", kind)
		tile.set_meta("created_by", str(row.get("created_by", "")))
		tile.set_meta("text", str(row.get("text", "")))
		tile.set_meta("state", state)
		tile.set_meta("asset_texture", _content_asset_texture(kind))
		tile.z_index = int(tile.position.y) - 2
		tile.queue_redraw()

	for id in world_tiles.keys():
		if not seen_tiles.has(id):
			world_tiles[id].queue_free()
			world_tiles.erase(id)
	if _agent_is_logged_in() and not agent_baseline_ready:
		agent_baseline_ready = true
	elif not _agent_is_logged_in():
		agent_baseline_ready = false

func _update_plots() -> void:
	var seen := {}
	var record_events := agent_baseline_ready and _agent_is_logged_in()
	player_plots.clear()
	for row in client.player_plots():
		var owner := str(row["owner"])
		seen[owner] = true
		var origin := Vector2(float(row["origin_x"]), float(row["origin_y"]))
		var plot_size := Vector2(float(row["width"]), float(row["height"]))
		var display_name := str(row["display_name"])
		var previous: Dictionary = agent_plot_state.get(owner, {})
		var state := {
			"display_name": display_name,
			"origin": origin,
			"size": plot_size,
			"assigned_at_micros": int(row.get("assigned_at_micros", 0)),
		}
		if record_events and previous.is_empty():
			_agent_record_event("plot_assigned", "%s has a plot at %s." % [display_name, _agent_point_text(origin)], origin, {
				"owner": owner,
				"display_name": display_name,
				"rect": _agent_rect(Rect2(origin, plot_size)),
			})
		agent_plot_state[owner] = state
		player_plots[owner] = {
			"display_name": str(row["display_name"]),
			"origin": origin,
			"size": plot_size,
			"is_local": owner == local_identity,
		}
	for owner in agent_plot_state.keys():
		if not seen.has(owner):
			agent_plot_state.erase(owner)

func _ingest_chat_messages() -> String:
	var recent := ""
	for row in client.chat_messages():
		var message_id := int(row["id"])
		if chat_messages_seen.has(message_id):
			continue

		chat_messages_seen[message_id] = true
		var sender := str(row["sender"])
		var body := str(row["body"])
		var display_name := str(row["display_name"])
		var now := Time.get_ticks_msec() / 1000.0
		chat_bubbles_by_sender[sender] = {
			"body": body,
			"display_name": display_name,
			"started_at": now,
		}
		recent = "%s: %s" % [display_name, body]
		agent_seen_chat_ids[message_id] = true
		if agent_baseline_ready and _agent_is_logged_in():
			var sender_position := _agent_player_position(sender)
			_agent_record_event("chat", "%s said: %s" % [display_name, body], sender_position, {
				"id": message_id,
				"sender": sender,
				"display_name": display_name,
				"body": body,
				"position": _agent_point(sender_position),
				"sent_at_micros": int(row.get("sent_at_micros", 0)),
			})

	return recent

func _update_camera(delta: float) -> void:
	if local_identity.is_empty() or not avatars.has(local_identity):
		_apply_camera()
		return

	var avatar: AvatarNode = avatars[local_identity]
	var focus := avatar.world_target
	if not camera_initialized:
		camera_position = focus
		camera_initialized = true
		_apply_camera()
		return

	var half_view := size * 0.5
	var half_deadzone := CAMERA_DEADZONE * 0.5
	var screen_focus := focus - camera_position + half_view
	var left := half_view.x - half_deadzone.x
	var right := half_view.x + half_deadzone.x
	var top := half_view.y - half_deadzone.y
	var bottom := half_view.y + half_deadzone.y
	var desired := camera_position

	if screen_focus.x < left:
		desired.x -= left - screen_focus.x
	elif screen_focus.x > right:
		desired.x += screen_focus.x - right

	if screen_focus.y < top:
		desired.y -= top - screen_focus.y
	elif screen_focus.y > bottom:
		desired.y += screen_focus.y - bottom

	var t := 1.0 - exp(-CAMERA_SMOOTH_SPEED * delta)
	camera_position = camera_position.lerp(desired, t)
	_apply_camera()

func _apply_camera() -> void:
	world.position = (size * 0.5 - camera_position).round()

func _draw() -> void:
	if not place_mode:
		pass
	else:
		var local_avatar: AvatarNode = avatars.get(local_identity, null)
		if local_avatar != null:
			var origin_screen := _world_to_screen(local_avatar.world_target)
			var target_screen := _world_to_screen(place_target_clamped)
			var valid_color := Color(0.98, 0.88, 0.52, 0.95) if place_layer == "object" else Color(0.52, 0.82, 0.96, 0.95)
			var invalid_color := Color(0.94, 0.30, 0.24, 0.85)
			var preview_color := valid_color if place_target_valid else invalid_color
			draw_arc(origin_screen, float(PLACE_RADIUS_PIXELS), 0.0, TAU, 48, Color(1.0, 1.0, 1.0, 0.10), 1.0, true)
			draw_arc(origin_screen, float(PLACE_RADIUS_PIXELS), 0.0, TAU, 48, preview_color, 2.0, false)
			if place_layer == "tile":
				draw_rect(Rect2(target_screen - Vector2(TILE_SIZE * 0.5, TILE_SIZE * 0.5), Vector2(TILE_SIZE, TILE_SIZE)), preview_color, true)
				draw_rect(Rect2(target_screen - Vector2(TILE_SIZE * 0.5, TILE_SIZE * 0.5), Vector2(TILE_SIZE, TILE_SIZE)), preview_color, false, 2.0)
			else:
				draw_rect(Rect2(target_screen - Vector2(TILE_SIZE * 0.5, TILE_SIZE * 0.5), Vector2(TILE_SIZE, TILE_SIZE)), preview_color, false, 2.0)
			draw_line(origin_screen, target_screen, preview_color, 1.0)
			draw_string(ThemeDB.fallback_font, origin_screen + Vector2(16, -12), "place %s %s" % [place_layer, place_kind], HORIZONTAL_ALIGNMENT_LEFT, 160, 12, preview_color)

	if not library_open:
		return

	var pane_rect := library_preview_canvas.get_global_rect()
	var entry := _library_entry_for(library_category, library_selected_kind)
	if entry.is_empty():
		return
	var kind := str(entry["kind"])
	var center := pane_rect.get_center()
	var bg := Color(0.08, 0.09, 0.11, 0.95)
	draw_rect(pane_rect, bg, true)
	draw_rect(pane_rect, Color(0.95, 0.92, 0.80, 0.35), false, 2.0)
	var texture := _content_asset_preview_texture(kind)
	if texture != null:
		var tex_size := Vector2(texture.get_width(), texture.get_height())
		var scale := minf(192.0 / maxf(tex_size.x, 1.0), 192.0 / maxf(tex_size.y, 1.0))
		var draw_size := tex_size * scale
		draw_texture_rect(texture, Rect2(center - draw_size * 0.5, draw_size), false)
	if library_category == "object" and kind == "button":
		var overlay_color := Color(0.22, 0.78, 0.36, 0.25) if bool(entry.get("button_pressed", false)) else Color(0.92, 0.28, 0.20, 0.12)
		draw_rect(Rect2(center - Vector2(64, 64), Vector2(128, 128)), overlay_color, true)
	draw_rect(Rect2(center - Vector2(64, 64), Vector2(128, 128)), Color(0.95, 0.92, 0.80), false, 4.0)
	draw_string(ThemeDB.fallback_font, center + Vector2(-54, 96), str(entry["name"]), HORIZONTAL_ALIGNMENT_LEFT, 180, 18, Color(0.96, 0.93, 0.81))

func _update_chat() -> void:
	latest_chat_by_sender.clear()
	var recent := _ingest_chat_messages()
	for row in client.chat_messages():
		var sender := str(row["sender"])
		var body := str(row["body"])
		latest_chat_by_sender[sender] = body
	if recent.is_empty():
		recent_label.text = "WASD / arrows to move"
	else:
		recent_label.text = recent

func _avatar_for(identity: String) -> Node2D:
	if avatars.has(identity):
		return avatars[identity]

	var avatar := AvatarNode.new()
	world.add_child(avatar)
	avatars[identity] = avatar
	return avatar

func _tile_for(id: int) -> Node2D:
	if world_tiles.has(id):
		return world_tiles[id]

	var tile := WorldTileNode.new()
	world.add_child(tile)
	world_tiles[id] = tile
	return tile

func _style_ui() -> void:
	var panel_style := StyleBoxFlat.new()
	panel_style.bg_color = Color(0.10, 0.12, 0.13, 0.86)
	panel_style.border_color = Color(0.88, 0.80, 0.58)
	panel_style.set_border_width_all(2)
	panel_style.set_corner_radius_all(0)
	$Hud.add_theme_stylebox_override("panel", panel_style)
	$ChatInput.add_theme_stylebox_override("panel", panel_style)
	status_label.add_theme_color_override("font_color", Color(0.94, 0.91, 0.78))
	recent_label.add_theme_color_override("font_color", Color(0.78, 0.94, 0.82))

func _short_error(error: String) -> String:
	var compact := error.replace("Connection with ws://127.0.0.1:3000/v1/database/tinygrove-dev/subscribe?compression=Brotli IO error: ", "")
	if compact.length() > STATUS_MAX_CHARS:
		return compact.substr(0, STATUS_MAX_CHARS - 3) + "..."
	return compact

func _start_agent_http() -> void:
	var preferred_port := int(OS.get_environment("TINYGROVE_AGENT_PORT"))
	if preferred_port <= 0:
		preferred_port = AGENT_HTTP_PORT

	var exact_port := not OS.get_environment("TINYGROVE_AGENT_PORT").strip_edges().is_empty()
	var ports_to_try := 1 if exact_port else AGENT_HTTP_PORT_SCAN_COUNT
	for offset in range(ports_to_try):
		var candidate_port := preferred_port + offset
		var error := agent_server.listen(candidate_port, AGENT_HTTP_HOST)
		if error == OK:
			agent_http_port = candidate_port
			agent_http_status = "Agent loopback listening at http://%s:%d (%s)" % [AGENT_HTTP_HOST, agent_http_port, agent_profile]
			_write_agent_registry()
			return

	agent_http_status = "Agent loopback failed to listen on %s starting at %d for profile %s" % [AGENT_HTTP_HOST, preferred_port, agent_profile]
	push_warning(agent_http_status)

func _load_agent_config() -> void:
	agent_profile = OS.get_environment("TINYGROVE_AGENT_PROFILE").strip_edges()
	if agent_profile.is_empty():
		agent_profile = "human"

func _poll_agent_http() -> void:
	if agent_server.is_listening():
		while agent_server.is_connection_available():
			var peer := agent_server.take_connection()
			agent_connections.append({"peer": peer, "buffer": ""})

	var finished: Array[Dictionary] = []
	for connection in agent_connections:
		var peer: StreamPeerTCP = connection["peer"]
		if peer.get_status() != StreamPeerTCP.STATUS_CONNECTED:
			finished.append(connection)
			continue

		var available := peer.get_available_bytes()
		if available > 0:
			connection["buffer"] = str(connection["buffer"]) + peer.get_utf8_string(available)

		var request_text := str(connection["buffer"])
		if _agent_http_request_complete(request_text):
			_handle_agent_http_request(peer, request_text)
			peer.disconnect_from_host()
			finished.append(connection)

	for connection in finished:
		agent_connections.erase(connection)

func _agent_http_request_complete(request_text: String) -> bool:
	var header_end := request_text.find("\r\n\r\n")
	if header_end == -1:
		return false

	var content_length := 0
	var headers := request_text.substr(0, header_end).split("\r\n")
	for header in headers:
		var separator := header.find(":")
		if separator == -1:
			continue
		if header.substr(0, separator).strip_edges().to_lower() == "content-length":
			content_length = int(header.substr(separator + 1).strip_edges())
			break

	var received_body_length := request_text.length() - header_end - 4
	return received_body_length >= content_length

func _handle_agent_http_request(peer: StreamPeerTCP, request_text: String) -> void:
	var request := _parse_agent_http_request(request_text)
	var method: String = request["method"]
	var path: String = request["path"]
	var query: Dictionary = request["query"]

	if method == "OPTIONS":
		_send_agent_http_bytes(peer, 204, "text/plain; charset=utf-8", PackedByteArray())
		return

	match path:
		"/", "/help":
			_send_agent_http_json(peer, 200, _agent_help())
		"/login":
			if method != "POST" and method != "GET":
				_send_agent_http_json(peer, 405, {"ok": false, "message": "Use POST /login with optional JSON like {\"display_name\":\"Agent\"}."})
			else:
				_send_agent_http_json(peer, 200, _agent_login(request["body"], query))
		"/snapshot":
			if method != "GET":
				_send_agent_http_json(peer, 405, {"ok": false, "message": "Use GET /snapshot."})
			else:
				_send_agent_http_json(peer, 200, _agent_snapshot(true))
		"/delta":
			if method != "GET":
				_send_agent_http_json(peer, 405, {"ok": false, "message": "Use GET /delta, optionally with ?since=<cursor>."})
			else:
				_send_agent_http_json(peer, 200, _agent_delta_response(_agent_since_from_query(query), true))
		"/move":
			if method != "POST" and method != "GET":
				_send_agent_http_json(peer, 405, {"ok": false, "message": "Use POST /move with JSON like {\"direction\":\"east\",\"steps\":3}."})
			else:
				_send_agent_http_json(peer, 200, _agent_move(request["body"], query))
		"/chat":
			if method != "POST" and method != "GET":
				_send_agent_http_json(peer, 405, {"ok": false, "message": "Use POST /chat with JSON like {\"body\":\"Hello\"}."})
			else:
				_send_agent_http_json(peer, 200, _agent_chat(request["body"], query))
		"/place":
			if method != "POST" and method != "GET":
				_send_agent_http_json(peer, 405, {"ok": false, "message": "Use POST /place with JSON like {\"kind\":\"flower\"}."})
			else:
				_send_agent_http_json(peer, 200, _agent_place(request["body"], query))
		"/interact":
			if method != "POST" and method != "GET":
				_send_agent_http_json(peer, 405, {"ok": false, "message": "Use POST /interact."})
			else:
				_send_agent_http_json(peer, 200, _agent_interact(query))
		"/screenshot":
			if method != "GET":
				_send_agent_http_json(peer, 405, {"ok": false, "message": "Use GET /screenshot, optionally with ?size=bigger or ?size=max."})
			else:
				var screenshot := _agent_screenshot(str(query.get("size", "default")))
				_send_agent_http_bytes(peer, 200, screenshot["content_type"], screenshot["bytes"])
		_:
			_send_agent_http_json(peer, 404, {"ok": false, "message": "Unknown Tiny Grove agent endpoint. Try GET /help."})

func _parse_agent_http_request(request_text: String) -> Dictionary:
	var header_end := request_text.find("\r\n\r\n")
	var head := request_text.substr(0, header_end)
	var body := request_text.substr(header_end + 4)
	var lines := head.split("\r\n")
	var request_line := str(lines[0]).split(" ")
	var method := "GET"
	var raw_path := "/"
	if request_line.size() >= 2:
		method = str(request_line[0]).to_upper()
		raw_path = str(request_line[1])

	var path := raw_path
	var query := {}
	var query_start := raw_path.find("?")
	if query_start != -1:
		path = raw_path.substr(0, query_start)
		query = _parse_agent_query(raw_path.substr(query_start + 1))

	return {
		"method": method,
		"path": path,
		"query": query,
		"body": body,
	}

func _parse_agent_query(query_text: String) -> Dictionary:
	var query := {}
	if query_text.is_empty():
		return query

	for part in query_text.split("&", false):
		var separator := str(part).find("=")
		if separator == -1:
			query[str(part).uri_decode()] = ""
		else:
			var key := str(part).substr(0, separator).uri_decode()
			var value := str(part).substr(separator + 1).uri_decode()
			query[key] = value
	return query

func _send_agent_http_json(peer: StreamPeerTCP, status_code: int, data: Dictionary) -> void:
	var body := JSON.stringify(data, "\t").to_utf8_buffer()
	_send_agent_http_bytes(peer, status_code, "application/json; charset=utf-8", body)

func _send_agent_http_bytes(peer: StreamPeerTCP, status_code: int, content_type: String, body: PackedByteArray) -> void:
	var reason := "OK"
	match status_code:
		204:
			reason = "No Content"
		404:
			reason = "Not Found"
		405:
			reason = "Method Not Allowed"
		_:
			reason = "OK"
	var header_text := "\r\n".join([
		"HTTP/1.1 %d %s" % [status_code, reason],
		"Content-Type: %s" % content_type,
		"Content-Length: %d" % body.size(),
		"Access-Control-Allow-Origin: *",
		"Access-Control-Allow-Methods: GET, POST, OPTIONS",
		"Access-Control-Allow-Headers: Content-Type",
		"Connection: close",
		"",
		"",
	])
	var headers := header_text.to_utf8_buffer()
	var response := PackedByteArray()
	response.append_array(headers)
	response.append_array(body)
	peer.put_data(response)

func _agent_help() -> Dictionary:
	return {
		"ok": true,
		"name": "Tiny Grove agent loopback",
		"base_url": _agent_base_url(),
		"profile": agent_profile,
		"registry_path": agent_registry_path,
		"endpoints": [
			"GET /snapshot - camera-scoped text/JSON snapshot of the current view",
			"GET /delta - camera-scoped text events since your last look, or ?since=<cursor>",
			"POST /login - join gently; JSON body may include display_name",
			"POST /move - move by direction and optional steps",
			"POST /chat - send a chat message",
			"POST /place - place an explicit object or tile target within the placement radius",
			"POST /interact - interact with the nearby object you face",
			"GET /screenshot - JPEG, downsampled to fit 1024x768",
			"GET /screenshot?size=bigger - PNG, downsampled to fit 1280x720",
			"GET /screenshot?size=max - PNG, downsampled to fit 1920x1080",
		],
		"notes": [
			"No stream endpoint exists yet.",
			"The snapshot describes only what is inside the current camera view.",
			"Placement is only valid within a fixed radius of your current position; the server rejects targets outside that circle.",
			"Use layer=tile with tile_kind=grass|path|water|dirt for ground placement, or layer=object with kind=flower|button|sign|rock for top-of-tile placement.",
			"Action endpoints include a delta and advance your cursor.",
			"If a response says you are not logged in, call POST /login and retry after a moment.",
		],
	}

func _agent_login(body: String, query: Dictionary) -> Dictionary:
	var requested_name := str(query.get("display_name", "")).strip_edges()
	var parsed: Variant = JSON.parse_string(body) if not body.strip_edges().is_empty() else null
	if requested_name.is_empty() and typeof(parsed) == TYPE_DICTIONARY:
		requested_name = str(parsed.get("display_name", "")).strip_edges()
	if requested_name.is_empty():
		requested_name = OS.get_environment("TINYGROVE_AGENT_NAME").strip_edges()
	if requested_name.is_empty():
		requested_name = "Agent"

	if _agent_is_logged_in():
		return {
			"ok": true,
			"already_logged_in": true,
			"display_name": _agent_local_display_name(),
			"identity": local_identity,
			"message": "Successfully logged in as %s." % _agent_local_display_name(),
			"delta": _agent_delta_payload(agent_last_seen_cursor, true),
		}

	if client == null:
		return {
			"ok": false,
			"message": "The Tiny Grove client is not ready yet. Wait a moment, then call POST /login again.",
			"delta": _agent_delta_payload(agent_last_seen_cursor, true),
		}

	if not bool(client.call("is_connected")):
		return {
			"ok": false,
			"message": "The Godot client is not connected to SpacetimeDB yet. Start the Tiny Grove dev server, wait for the client to connect, then call POST /login again.",
			"status": client.status(),
			"last_error": client.last_error(),
			"delta": _agent_delta_payload(agent_last_seen_cursor, true),
		}

	name_edit.text = requested_name
	var color := Color.from_hsv(randf(), 0.58, 0.9).to_rgba32()
	var sent: bool = client.join_game(requested_name, color)
	_agent_settle_after_action()
	if sent:
		return {
			"ok": true,
			"already_logged_in": false,
			"display_name": requested_name,
			"message": "Login requested as %s. Call GET /snapshot after a moment; if it still says not logged in, retry once." % requested_name,
			"delta": _agent_delta_payload(agent_last_seen_cursor, true),
		}
	return {
		"ok": false,
		"message": "The login request could not be sent. Check GET /snapshot for connection status, then retry POST /login.",
		"status": client.status(),
		"last_error": client.last_error(),
		"delta": _agent_delta_payload(agent_last_seen_cursor, true),
	}

func _agent_snapshot(advance_cursor := false) -> Dictionary:
	var view_rect := _agent_visible_world_rect()
	var logged_in := _agent_is_logged_in()
	var local_avatar: AvatarNode = avatars.get(local_identity, null)
	var local_position := local_avatar.world_target if local_avatar != null else view_rect.get_center()
	var visible_players := _agent_visible_players(view_rect, local_position)
	var visible_plots := _agent_visible_plots(view_rect)
	var visible_objects := _agent_visible_objects(view_rect, local_position)
	var visible_tiles := _agent_visible_tiles(view_rect, local_position)
	var visible_bubbles := _agent_visible_bubbles(view_rect)
	var placement := {
		"radius_tiles": PLACE_RADIUS_TILES,
		"radius_pixels": PLACE_RADIUS_PIXELS,
		"note": "Targets outside this radius are invalid and will be rejected by the server.",
		"layers": ["object", "tile"],
	}
	var hints := []
	if not logged_in:
		hints.append("You are not logged in. Call POST /login with JSON like {\"display_name\":\"Agent\"}, then retry GET /snapshot.")

	return {
		"ok": true,
		"logged_in": logged_in,
		"connection": {
			"connected": client != null and bool(client.call("is_connected")),
			"status": client.status() if client != null else "client not ready",
			"last_error": client.last_error() if client != null else "",
			"agent_http": agent_http_status,
			"agent_profile": agent_profile,
			"agent_base_url": _agent_base_url(),
			"agent_registry_path": agent_registry_path,
		},
		"you": _agent_you_dictionary(local_avatar),
		"placement": placement,
		"camera": {
			"center": _agent_point(camera_position),
			"view_rect": _agent_rect(view_rect),
			"note": "All players, objects, plots, and chat bubbles below are constrained to this camera view.",
		},
		"summary": _agent_snapshot_summary(logged_in, local_avatar, view_rect, visible_players, visible_objects),
		"visible_players": visible_players,
		"visible_chat_bubbles": visible_bubbles,
		"visible_plots": visible_plots,
		"visible_objects": visible_objects,
		"visible_tiles": visible_tiles,
		"delta": _agent_delta_payload(agent_last_seen_cursor, advance_cursor),
		"hints": hints,
	}

func _agent_move(body: String, query: Dictionary) -> Dictionary:
	var payload := _agent_json_payload(body)
	var direction := str(query.get("direction", payload.get("direction", ""))).strip_edges().to_lower()
	var dx := int(query.get("dx", payload.get("dx", 0)))
	var dy := int(query.get("dy", payload.get("dy", 0)))
	if not direction.is_empty():
		var direction_axis := _agent_direction_to_axis(direction)
		dx = int(direction_axis.x)
		dy = int(direction_axis.y)
	var steps := int(query.get("steps", payload.get("steps", 1)))
	steps = clampi(steps, 1, AGENT_MAX_MOVE_STEPS)

	var result := _agent_action_base("move")
	if not result["ok"]:
		result["delta"] = _agent_delta_payload(agent_last_seen_cursor, true)
		return result
	if dx == 0 and dy == 0:
		result["ok"] = false
		result["message"] = "Choose a direction: north, south, east, west, up, down, left, right, or provide dx/dy."
		result["delta"] = _agent_delta_payload(agent_last_seen_cursor, true)
		return result

	var sent := 0
	for _index in range(steps):
		if client.move_player(dx, dy):
			sent += 1
		else:
			break
	_agent_settle_after_action()
	result["ok"] = sent > 0
	var delta := _agent_delta_payload(agent_last_seen_cursor, true)
	result["message"] = "Moved %d step(s) %s." % [sent, _agent_facing_phrase(dx, dy)] if sent > 0 and int(delta["visible_event_count"]) > 0 else "Move request was sent, but no visible movement delta arrived yet." if sent > 0 else "Move request could not be sent."
	result["requested_steps"] = steps
	result["sent_steps"] = sent
	result["delta"] = delta
	result["you"] = _agent_you_dictionary(avatars.get(local_identity, null))
	return result

func _agent_chat(body: String, query: Dictionary) -> Dictionary:
	var payload := _agent_json_payload(body)
	var chat_body := str(query.get("body", payload.get("body", ""))).strip_edges()
	var result := _agent_action_base("chat")
	if not result["ok"]:
		result["delta"] = _agent_delta_payload(agent_last_seen_cursor, true)
		return result
	if chat_body.is_empty():
		result["ok"] = false
		result["message"] = "Provide a chat body, for example POST /chat with {\"body\":\"Hello\"}."
		result["delta"] = _agent_delta_payload(agent_last_seen_cursor, true)
		return result

	var sent: bool = client.send_chat(chat_body)
	_agent_settle_after_action()
	var delta := _agent_delta_payload(agent_last_seen_cursor, true)
	result["ok"] = sent
	result["message"] = "Sent chat: %s" % chat_body if sent and int(delta["visible_event_count"]) > 0 else "Chat request was sent, but no visible chat delta arrived yet." if sent else "Chat request could not be sent."
	result["delta"] = delta
	return result

func _agent_place(body: String, query: Dictionary) -> Dictionary:
	var payload := _agent_json_payload(body)
	var layer := str(query.get("layer", payload.get("layer", "object"))).strip_edges().to_lower()
	var kind_key := "tile_kind" if layer == "tile" else "kind"
	var kind := str(query.get(kind_key, payload.get(kind_key, "flower" if layer != "tile" else "grass"))).strip_edges().to_lower()
	var result := _agent_action_base("place")
	if not result["ok"]:
		result["delta"] = _agent_delta_payload(agent_last_seen_cursor, true)
		return result
	if kind.is_empty():
		kind = "grass" if layer == "tile" else "flower"
	var target := _agent_place_target(query, payload)
	if target.is_empty():
		result["ok"] = false
		result["message"] = "Provide layer=tile and tile_kind, or layer=object and kind, with a target within %d tiles. Example: POST /place with {\"layer\":\"tile\",\"tile_kind\":\"grass\",\"tile_x\":12,\"tile_y\":7}." % PLACE_RADIUS_TILES
		result["delta"] = _agent_delta_payload(agent_last_seen_cursor, true)
		return result

	var sent: bool = false
	if layer == "tile":
		sent = client.place_tile(kind, int(target["x"]), int(target["y"]))
	_agent_settle_after_action()
	var delta := _agent_delta_payload(agent_last_seen_cursor, true)
	result["ok"] = sent
	result["message"] = "Placed %s %s at %s." % [layer, kind, _agent_point_text(Vector2(float(target["x"]), float(target["y"])))] if sent and int(delta["visible_event_count"]) > 0 else "Place request was sent, but no visible delta arrived. The server may have rejected it, the target may be outside your plot, or the target may be outside the placement radius." if sent else "Place request could not be sent. Check the placement radius and plot boundaries."
	result["layer"] = layer
	if layer == "tile":
		result["tile_kind"] = kind
	else:
		result["kind"] = kind
	result["target"] = target
	result["delta"] = delta
	return result

func _agent_place_target(query: Dictionary, payload: Dictionary) -> Dictionary:
	var raw_x: Variant = query.get("x", payload.get("x", null))
	var raw_y: Variant = query.get("y", payload.get("y", null))
	if raw_x != null and raw_y != null:
		return {"x": int(raw_x), "y": int(raw_y)}

	var raw_tile_x: Variant = query.get("tile_x", payload.get("tile_x", null))
	var raw_tile_y: Variant = query.get("tile_y", payload.get("tile_y", null))
	if raw_tile_x != null and raw_tile_y != null:
		return {
			"x": int(raw_tile_x) * TILE_SIZE + int(TILE_SIZE * 0.5),
			"y": int(raw_tile_y) * TILE_SIZE + int(TILE_SIZE * 0.5),
		}

	return {}

func _agent_interact(query: Dictionary) -> Dictionary:
	var result := _agent_action_base("interact")
	if not result["ok"]:
		result["delta"] = _agent_delta_payload(_agent_since_from_query(query), true)
		return result

	var sent: bool = client.interact_near()
	_agent_settle_after_action()
	var delta := _agent_delta_payload(_agent_since_from_query(query), true)
	result["ok"] = sent
	result["message"] = "Interacted with the nearby object you are facing." if sent and int(delta["visible_event_count"]) > 0 else "Interact request was sent, but no visible delta arrived. There may be nothing nearby or the interaction may have no visible effect." if sent else "Interact request could not be sent. There may be nothing nearby."
	result["delta"] = delta
	return result

func _agent_action_base(action_name: String) -> Dictionary:
	if not _agent_is_logged_in():
		return {
			"ok": false,
			"action": action_name,
			"message": "You are not logged in. Call POST /login with {\"display_name\":\"Agent\"}, wait a moment, then retry.",
		}
	if client == null or not bool(client.call("is_connected")):
		return {
			"ok": false,
			"action": action_name,
			"message": "The Godot client is not connected to SpacetimeDB. Start the dev server, wait for connection, then retry.",
		}
	return {
		"ok": true,
		"action": action_name,
	}

func _agent_visible_world_rect() -> Rect2:
	var viewport_size := size
	if viewport_size.x <= 0.0 or viewport_size.y <= 0.0:
		viewport_size = get_viewport_rect().size
	return Rect2(camera_position - viewport_size * 0.5, viewport_size)

func _agent_is_logged_in() -> bool:
	return not local_identity.is_empty() and avatars.has(local_identity)

func _agent_local_display_name() -> String:
	if _agent_is_logged_in():
		var avatar: AvatarNode = avatars[local_identity]
		return str(avatar.get_meta("display_name", name_edit.text))
	if not name_edit.text.strip_edges().is_empty():
		return name_edit.text.strip_edges()
	return "Agent"

func _agent_you_dictionary(local_avatar: AvatarNode) -> Dictionary:
	if local_avatar == null:
		return {
			"logged_in": false,
			"instruction": "Call POST /login with optional JSON like {\"display_name\":\"Agent\"}.",
		}

	return {
		"logged_in": true,
		"identity": local_identity,
		"display_name": str(local_avatar.get_meta("display_name", "Player")),
		"position": _agent_point(local_avatar.world_target),
		"facing": _agent_facing_phrase(int(local_avatar.get_meta("last_dx", 0)), int(local_avatar.get_meta("last_dy", 0))),
	}

func _agent_visible_players(view_rect: Rect2, local_position: Vector2) -> Array:
	var players := []
	for identity in avatars.keys():
		var avatar: AvatarNode = avatars[identity]
		if not view_rect.has_point(avatar.world_target):
			continue
		players.append({
			"identity": identity,
			"display_name": str(avatar.get_meta("display_name", "Player")),
			"position": _agent_point(avatar.world_target),
			"relative_to_you": _agent_relative_phrase(local_position, avatar.world_target),
			"facing": _agent_facing_phrase(int(avatar.get_meta("last_dx", 0)), int(avatar.get_meta("last_dy", 0))),
			"is_you": identity == local_identity,
			"online": true,
		})
	return players

func _agent_visible_bubbles(view_rect: Rect2) -> Array:
	var bubbles := []
	var now := Time.get_ticks_msec() / 1000.0
	for identity in avatars.keys():
		var avatar: AvatarNode = avatars[identity]
		if not view_rect.has_point(avatar.world_target):
			continue
		var body := str(avatar.get_meta("bubble_body", ""))
		if body.is_empty():
			continue
		var started_at := float(avatar.get_meta("bubble_started_at", 0.0))
		var age := now - started_at
		if age > BUBBLE_VISIBLE_SECONDS + BUBBLE_FADE_SECONDS:
			continue
		bubbles.append({
			"speaker": str(avatar.get_meta("display_name", "Player")),
			"position": _agent_point(avatar.world_target),
			"preview": body.substr(0, BUBBLE_MAX_CHARS) + ("..." if body.length() > BUBBLE_MAX_CHARS else ""),
			"age_seconds": snappedf(age, 0.1),
		})
	return bubbles

func _agent_visible_plots(view_rect: Rect2) -> Array:
	var plots := []
	for owner in player_plots.keys():
		var plot: Dictionary = player_plots[owner]
		var rect := Rect2(plot["origin"], plot["size"])
		if not rect.intersects(view_rect, true):
			continue
		plots.append({
			"owner": owner,
			"display_name": str(plot["display_name"]),
			"is_yours": bool(plot["is_local"]),
			"rect": _agent_rect(rect),
		})
	return plots

func _agent_visible_objects(view_rect: Rect2, local_position: Vector2) -> Dictionary:
	var objects := []
	for id in world_tiles.keys():
		var object: Node2D = world_tiles[id]
		if not view_rect.has_point(object.position):
			continue
		var kind := str(object.get_meta("kind", "object"))
		var priority := 0 if kind == "button" or kind == "sign" else 1
		objects.append({
			"id": id,
			"kind": kind,
			"text": str(object.get_meta("text", "")),
			"state": int(object.get_meta("state", 0)),
			"position": _agent_point(object.position),
			"relative_to_you": _agent_relative_phrase(local_position, object.position),
			"view_area": _agent_area_phrase(object.position, view_rect),
			"_priority": priority,
			"_distance": local_position.distance_squared_to(object.position),
		})

	objects.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if int(a["_priority"]) != int(b["_priority"]):
			return int(a["_priority"]) < int(b["_priority"])
		return float(a["_distance"]) < float(b["_distance"])
	)

	var included := []
	var overflow := []
	for index in range(objects.size()):
		var object: Dictionary = objects[index]
		object.erase("_priority")
		object.erase("_distance")
		if index < AGENT_MAX_INDIVIDUAL_OBJECTS:
			included.append(object)
		else:
			overflow.append(object)

	return {
		"total_count": objects.size(),
		"listed_count": included.size(),
		"listed": included,
		"grouped_overflow": _agent_group_objects(overflow),
		"overflow_note": "Repetitive lower-priority objects are grouped after %d individual objects to save tokens." % AGENT_MAX_INDIVIDUAL_OBJECTS if overflow.size() > 0 else "",
	}

func _agent_visible_tiles(view_rect: Rect2, local_position: Vector2) -> Dictionary:
	var tiles := []
	for id in world_tiles.keys():
		var tile: Node2D = world_tiles[id]
		if not view_rect.has_point(tile.position):
			continue
		tiles.append({
			"id": id,
			"kind": str(tile.get_meta("kind", "tile")),
			"position": _agent_point(tile.position),
			"relative_to_you": _agent_relative_phrase(local_position, tile.position),
			"view_area": _agent_area_phrase(tile.position, view_rect),
			"_distance": local_position.distance_squared_to(tile.position),
		})
	tiles.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return float(a["_distance"]) < float(b["_distance"])
	)
	for tile in tiles:
		tile.erase("_distance")
	return {
		"total_count": tiles.size(),
		"listed": tiles,
	}

func _agent_group_objects(objects: Array) -> Array:
	var groups := {}
	for object in objects:
		var key := "%s:%s" % [str(object["kind"]), str(object["view_area"])]
		if not groups.has(key):
			groups[key] = {
				"kind": str(object["kind"]),
				"view_area": str(object["view_area"]),
				"count": 0,
				"sample_positions": [],
			}
		var group: Dictionary = groups[key]
		group["count"] = int(group["count"]) + 1
		var samples: Array = group["sample_positions"]
		if samples.size() < 3:
			samples.append(object["position"])

	var grouped := []
	for key in groups.keys():
		grouped.append(groups[key])
	grouped.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if str(a["kind"]) != str(b["kind"]):
			return str(a["kind"]) < str(b["kind"])
		return int(a["count"]) > int(b["count"])
	)
	return grouped

func _agent_snapshot_summary(logged_in: bool, local_avatar: AvatarNode, view_rect: Rect2, players: Array, objects: Dictionary) -> String:
	if not logged_in:
		return "You are viewing Tiny Grove but are not logged in. The camera shows world x %.1f..%.1f and y %.1f..%.1f. Call POST /login, then ask for another snapshot." % [view_rect.position.x, view_rect.end.x, view_rect.position.y, view_rect.end.y]

	var player_names := []
	for player in players:
		if not bool(player["is_you"]):
			player_names.append(str(player["display_name"]))
	var nearby_text := "No other visible players."
	if player_names.size() > 0:
		nearby_text = "Visible players: %s." % ", ".join(player_names)

	var object_text := "No placed objects are visible."
	if int(objects["total_count"]) > 0:
		object_text = "%d placed object(s) are visible; %d are listed individually." % [int(objects["total_count"]), int(objects["listed_count"])]

	return "You are %s at %s, facing %s. Camera view is x %.1f..%.1f, y %.1f..%.1f. %s %s" % [
		str(local_avatar.get_meta("display_name", "Player")),
		_agent_point_text(local_avatar.world_target),
		_agent_facing_phrase(int(local_avatar.get_meta("last_dx", 0)), int(local_avatar.get_meta("last_dy", 0))),
		view_rect.position.x,
		view_rect.end.x,
		view_rect.position.y,
		view_rect.end.y,
		nearby_text,
		object_text,
	]

func _agent_screenshot(size_mode: String) -> Dictionary:
	var mode := size_mode.to_lower()
	var max_size := AGENT_DEFAULT_SCREENSHOT_MAX
	var content_type := "image/jpeg"
	if mode == "bigger":
		max_size = AGENT_BIGGER_SCREENSHOT_MAX
		content_type = "image/png"
	elif mode == "max":
		max_size = AGENT_MAX_SCREENSHOT_MAX
		content_type = "image/png"

	var viewport_texture := get_viewport().get_texture() if DisplayServer.get_name() != "headless" else null
	var image: Image = null
	if viewport_texture != null:
		image = viewport_texture.get_image()
	if image == null:
		image = Image.create(max_size.x, max_size.y, false, Image.FORMAT_RGB8)
		image.fill(Color(0.08, 0.10, 0.09))
	var target_size := _agent_fit_size(Vector2i(image.get_width(), image.get_height()), max_size)
	if target_size.x != image.get_width() or target_size.y != image.get_height():
		image.resize(target_size.x, target_size.y, Image.INTERPOLATE_LANCZOS)

	var bytes := PackedByteArray()
	if content_type == "image/jpeg" and image.has_method("save_jpg_to_buffer"):
		bytes = image.save_jpg_to_buffer(0.72)
	if bytes.is_empty():
		content_type = "image/png"
		bytes = image.save_png_to_buffer()

	return {
		"content_type": content_type,
		"bytes": bytes,
	}

func _agent_base_url() -> String:
	if agent_http_port <= 0:
		return ""
	return "http://%s:%d" % [AGENT_HTTP_HOST, agent_http_port]

func _agent_registry_dir() -> String:
	return ProjectSettings.globalize_path("res://%s" % AGENT_REGISTRY_RELATIVE_DIR).simplify_path()

func _update_agent_registry(delta: float) -> void:
	if agent_http_port <= 0:
		return
	agent_registry_elapsed += delta
	if agent_registry_elapsed < AGENT_REGISTRY_WRITE_SECONDS:
		return
	agent_registry_elapsed = 0.0
	_write_agent_registry()

func _write_agent_registry() -> void:
	if agent_http_port <= 0:
		return

	var registry_dir := _agent_registry_dir()
	DirAccess.make_dir_recursive_absolute(registry_dir)
	agent_registry_path = registry_dir.path_join("%d.json" % agent_http_port)
	var data := {
		"game": "tinygrove",
		"profile": agent_profile,
		"pid": OS.get_process_id(),
		"host": AGENT_HTTP_HOST,
		"port": agent_http_port,
		"base_url": _agent_base_url(),
		"display_name": _agent_local_display_name(),
		"identity": local_identity,
		"logged_in": _agent_is_logged_in(),
		"connected": client != null and bool(client.call("is_connected")),
		"status": client.status() if client != null else "client not ready",
		"updated_unix": Time.get_unix_time_from_system(),
	}
	var file := FileAccess.open(agent_registry_path, FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(data, "\t"))

func _remove_agent_registry() -> void:
	if not agent_registry_path.is_empty() and FileAccess.file_exists(agent_registry_path):
		DirAccess.remove_absolute(agent_registry_path)

func _agent_json_payload(body: String) -> Dictionary:
	if body.strip_edges().is_empty():
		return {}
	var parsed: Variant = JSON.parse_string(body)
	if typeof(parsed) == TYPE_DICTIONARY:
		return parsed
	return {}

func _agent_direction_to_axis(direction: String) -> Vector2i:
	match direction:
		"north", "up":
			return Vector2i(0, -1)
		"south", "down":
			return Vector2i(0, 1)
		"west", "left":
			return Vector2i(-1, 0)
		"east", "right":
			return Vector2i(1, 0)
		"northwest", "north-west", "up-left":
			return Vector2i(-1, -1)
		"northeast", "north-east", "up-right":
			return Vector2i(1, -1)
		"southwest", "south-west", "down-left":
			return Vector2i(-1, 1)
		"southeast", "south-east", "down-right":
			return Vector2i(1, 1)
		_:
			return Vector2i.ZERO

func _agent_settle_after_action() -> void:
	var started := Time.get_ticks_msec()
	while Time.get_ticks_msec() - started < AGENT_ACTION_SETTLE_MSEC:
		if client != null:
			client.poll()
			_update_chat()
			_update_world()
			_update_plots()
			_update_objects()
			_update_camera(0.0)
		OS.delay_msec(AGENT_ACTION_POLL_MSEC)

func _agent_since_from_query(query: Dictionary) -> int:
	if query.has("since"):
		return int(query["since"])
	if query.has("cursor"):
		return int(query["cursor"])
	return agent_last_seen_cursor

func _agent_record_event(kind: String, text: String, position: Vector2, details: Dictionary) -> void:
	agent_event_cursor += 1
	var event := {
		"cursor": agent_event_cursor,
		"frame": frame,
		"observed_unix": Time.get_unix_time_from_system(),
		"kind": kind,
		"text": text,
		"position": _agent_point(position),
		"details": details,
	}
	agent_events.append(event)
	while agent_events.size() > AGENT_MAX_STORED_EVENTS:
		agent_events.pop_front()

func _agent_delta_response(since: int, advance_cursor := true) -> Dictionary:
	return {
		"ok": true,
		"logged_in": _agent_is_logged_in(),
		"connection": {
			"connected": client != null and bool(client.call("is_connected")),
			"status": client.status() if client != null else "client not ready",
			"last_error": client.last_error() if client != null else "",
			"agent_base_url": _agent_base_url(),
			"agent_profile": agent_profile,
		},
		"delta": _agent_delta_payload(since, advance_cursor),
	}

func _agent_delta_payload(since: int, advance_cursor := true) -> Dictionary:
	var view_rect := _agent_visible_world_rect()
	var visible_events := []
	var hidden_count := 0
	var omitted_count := 0
	for event in agent_events:
		var cursor := int(event["cursor"])
		if cursor <= since:
			continue
		if not _agent_event_in_view(event, view_rect):
			hidden_count += 1
			continue
		if visible_events.size() >= AGENT_MAX_DELTA_EVENTS:
			omitted_count += 1
			continue
		visible_events.append(event)

	var from_cursor := since
	var to_cursor := agent_event_cursor
	if advance_cursor:
		agent_last_seen_cursor = to_cursor

	var summary := "No visible changes since cursor %d." % from_cursor
	if visible_events.size() > 0:
		var pieces := []
		for event in visible_events:
			pieces.append(str(event["text"]))
		summary = " ".join(pieces)
	if omitted_count > 0:
		summary += " %d additional visible event(s) omitted; ask with since=%d if needed." % [omitted_count, from_cursor]

	return {
		"from_cursor": from_cursor,
		"to_cursor": to_cursor,
		"next_since": to_cursor,
		"advanced_cursor": advance_cursor,
		"view_rect": _agent_rect(view_rect),
		"max_events": AGENT_MAX_DELTA_EVENTS,
		"events": visible_events,
		"visible_event_count": visible_events.size(),
		"hidden_out_of_view_count": hidden_count,
		"omitted_visible_count": omitted_count,
		"summary": summary,
	}

func _agent_event_in_view(event: Dictionary, view_rect: Rect2) -> bool:
	var position: Dictionary = event.get("position", {})
	if position.is_empty():
		return true
	return view_rect.has_point(Vector2(float(position.get("x", 0.0)), float(position.get("y", 0.0))))

func _agent_player_position(identity: String) -> Vector2:
	if avatars.has(identity):
		var avatar: AvatarNode = avatars[identity]
		return avatar.world_target
	if agent_player_state.has(identity):
		var state: Dictionary = agent_player_state[identity]
		return state["position"]
	return camera_position

func _agent_fit_size(current: Vector2i, maximum: Vector2i) -> Vector2i:
	if current.x <= maximum.x and current.y <= maximum.y:
		return current
	var scale: float = min(float(maximum.x) / float(current.x), float(maximum.y) / float(current.y))
	return Vector2i(max(1, int(floor(current.x * scale))), max(1, int(floor(current.y * scale))))

func _agent_point(point: Vector2) -> Dictionary:
	return {
		"x": snappedf(point.x, 0.1),
		"y": snappedf(point.y, 0.1),
		"tile_x": int(floor(point.x / TILE_SIZE)),
		"tile_y": int(floor(point.y / TILE_SIZE)),
	}

func _agent_rect(rect: Rect2) -> Dictionary:
	return {
		"x": snappedf(rect.position.x, 0.1),
		"y": snappedf(rect.position.y, 0.1),
		"width": snappedf(rect.size.x, 0.1),
		"height": snappedf(rect.size.y, 0.1),
		"max_x": snappedf(rect.end.x, 0.1),
		"max_y": snappedf(rect.end.y, 0.1),
	}

func _agent_point_text(point: Vector2) -> String:
	return "(%.1f, %.1f)" % [point.x, point.y]

func _agent_facing_phrase(dx: int, dy: int) -> String:
	if dx < 0:
		return "west"
	if dx > 0:
		return "east"
	if dy < 0:
		return "north"
	if dy > 0:
		return "south"
	return "south"

func _agent_relative_phrase(origin: Vector2, target: Vector2) -> String:
	var delta := target - origin
	var tiles_x := snappedf(delta.x / TILE_SIZE, 0.1)
	var tiles_y := snappedf(delta.y / TILE_SIZE, 0.1)
	var horizontal := ""
	var vertical := ""
	if absf(tiles_x) >= 0.5:
		horizontal = "%.1f tile(s) %s" % [absf(tiles_x), "east" if tiles_x > 0.0 else "west"]
	if absf(tiles_y) >= 0.5:
		vertical = "%.1f tile(s) %s" % [absf(tiles_y), "south" if tiles_y > 0.0 else "north"]
	if horizontal.is_empty() and vertical.is_empty():
		return "at your position"
	if horizontal.is_empty():
		return vertical
	if vertical.is_empty():
		return horizontal
	return "%s, %s" % [vertical, horizontal]

func _agent_area_phrase(point: Vector2, view_rect: Rect2) -> String:
	var rx := clampf((point.x - view_rect.position.x) / maxf(view_rect.size.x, 1.0), 0.0, 1.0)
	var ry := clampf((point.y - view_rect.position.y) / maxf(view_rect.size.y, 1.0), 0.0, 1.0)
	var horizontal := "west" if rx < 0.33 else "east" if rx > 0.66 else "center"
	var vertical := "north" if ry < 0.33 else "south" if ry > 0.66 else "middle"
	if horizontal == "center" and vertical == "middle":
		return "center"
	if horizontal == "center":
		return vertical
	if vertical == "middle":
		return horizontal
	return "%s-%s" % [vertical, horizontal]

func _load_smoke_config() -> void:
	smoke_enabled = OS.get_environment("TINYGROVE_SMOKE") == "1"
	if not smoke_enabled:
		return

	smoke_name = OS.get_environment("TINYGROVE_SMOKE_NAME")
	if smoke_name.is_empty():
		smoke_name = "Smoke"
	smoke_message = OS.get_environment("TINYGROVE_SMOKE_MESSAGE")
	if smoke_message.is_empty():
		smoke_message = "smoke message"
	smoke_dx = int(OS.get_environment("TINYGROVE_SMOKE_DX"))
	smoke_dy = int(OS.get_environment("TINYGROVE_SMOKE_DY"))
	smoke_object_kind = OS.get_environment("TINYGROVE_SMOKE_OBJECT")
	if smoke_object_kind.is_empty():
		smoke_object_kind = "flower"
	name_edit.text = smoke_name
	chat_edit.text = smoke_message

func _run_smoke_actions() -> void:
	if not smoke_enabled:
		return

	if frame == SMOKE_JOIN_FRAME:
		client.join_game(smoke_name, Color.from_hsv(randf(), 0.58, 0.9).to_rgba32())
	elif frame == SMOKE_MOVE_FRAME:
		client.move_player(smoke_dx, smoke_dy)
	elif frame == SMOKE_CHAT_FRAME:
		client.send_chat(smoke_message)
	elif frame == SMOKE_PLACE_FRAME:
		var smoke_target := camera_position
		if avatars.has(local_identity):
			var avatar: AvatarNode = avatars[local_identity]
			smoke_target = avatar.world_target
		client.place_tile(smoke_object_kind, int(smoke_target.x), int(smoke_target.y))
	elif frame == SMOKE_INTERACT_FRAME:
		client.interact_near()

class AvatarNode:
	extends Node2D

	var world_target := Vector2.ZERO
	var initialized := false

	func _ready() -> void:
		set_process(true)

	func _process(delta: float) -> void:
		if not initialized:
			return

		var t := 1.0 - exp(-AVATAR_SMOOTH_SPEED * delta)
		position = position.lerp(world_target, t)
		z_index = int(position.y)
		queue_redraw()

	func set_world_target(target: Vector2) -> void:
		world_target = target
		if not initialized:
			position = target
			initialized = true
			z_index = int(position.y)

	func _draw() -> void:
		var color_int: int = int(get_meta("avatar_color", 0x66CCAA))
		var color: Color = Color.hex(color_int)
		var shadow: Color = Color(0.08, 0.12, 0.10, 0.30)
		draw_rect(Rect2(Vector2(-10, 8), Vector2(20, 4)), shadow)
		draw_rect(Rect2(Vector2(-6, -16), Vector2(12, 10)), Color(0.96, 0.78, 0.58))
		draw_rect(Rect2(Vector2(-7, -5), Vector2(14, 12)), color)
		draw_rect(Rect2(Vector2(-7, 7), Vector2(5, 7)), Color(0.18, 0.21, 0.28))
		draw_rect(Rect2(Vector2(2, 7), Vector2(5, 7)), Color(0.18, 0.21, 0.28))
		draw_rect(Rect2(Vector2(-5, -13), Vector2(3, 3)), Color(0.08, 0.09, 0.11))
		draw_rect(Rect2(Vector2(2, -13), Vector2(3, 3)), Color(0.08, 0.09, 0.11))
		draw_rect(Rect2(Vector2(-8, -18), Vector2(16, 4)), Color(0.24, 0.13, 0.09))
		var label: String = str(get_meta("display_name", "Player"))
		draw_string(ThemeDB.fallback_font, Vector2(-36, -24), label, HORIZONTAL_ALIGNMENT_CENTER, 72, 12, Color(0.97, 0.95, 0.82))
		var bubble_body: String = str(get_meta("bubble_body", ""))
		if not bubble_body.is_empty():
			var preview: String = bubble_body.substr(0, BUBBLE_MAX_CHARS)
			if bubble_body.length() > BUBBLE_MAX_CHARS:
				preview += "..."
			var started_at: float = float(get_meta("bubble_started_at", 0.0))
			var age := Time.get_ticks_msec() / 1000.0 - started_at
			var alpha := 0.0
			if age < 0.15:
				alpha = clampf(age / 0.15, 0.0, 1.0)
			elif age <= BUBBLE_VISIBLE_SECONDS:
				alpha = 1.0
			elif age <= BUBBLE_VISIBLE_SECONDS + BUBBLE_FADE_SECONDS:
				alpha = clampf(1.0 - ((age - BUBBLE_VISIBLE_SECONDS) / BUBBLE_FADE_SECONDS), 0.0, 1.0)
			if alpha > 0.0:
				var width: float = clampf(ThemeDB.fallback_font.get_string_size(preview, HORIZONTAL_ALIGNMENT_LEFT, -1, 11).x + 12.0, 36.0, 150.0)
				var rect: Rect2 = Rect2(Vector2(-width * 0.5, -48), Vector2(width, 18))
				var fill := Color(0.98, 0.95, 0.82, alpha)
				var trim := Color(0.25, 0.20, 0.15, alpha)
				var text := Color(0.12, 0.10, 0.08, alpha)
				draw_rect(rect, fill)
				draw_rect(Rect2(rect.position, Vector2(rect.size.x, 2)), trim)
				draw_rect(Rect2(rect.position + Vector2(0, rect.size.y - 2), Vector2(rect.size.x, 2)), trim)
				draw_rect(Rect2(rect.position, Vector2(2, rect.size.y)), trim)
				draw_rect(Rect2(rect.position + Vector2(rect.size.x - 2, 0), Vector2(2, rect.size.y)), trim)
				draw_string(ThemeDB.fallback_font, rect.position + Vector2(6, 13), preview, HORIZONTAL_ALIGNMENT_LEFT, width - 12, 11, text)

class WorldTileNode:
	extends Node2D

	func _draw() -> void:
		var texture: Texture2D = get_meta("asset_texture", null)
		if texture != null:
			var size := Vector2(texture.get_width(), texture.get_height())
			draw_texture_rect(texture, Rect2(-size * 0.5, size), false)
			return
		var kind: String = str(get_meta("kind", "grass"))
		match kind:
			"path":
				_draw_path()
			"water":
				_draw_water()
			"dirt":
				_draw_dirt()
			_:
				_draw_grass()

	func _draw_grass() -> void:
		draw_rect(Rect2(Vector2(-16, -16), Vector2(32, 32)), Color(0.35, 0.56, 0.32))
		draw_rect(Rect2(Vector2(-14, -14), Vector2(28, 28)), Color(0.40, 0.64, 0.36, 0.35))

	func _draw_path() -> void:
		draw_rect(Rect2(Vector2(-16, -16), Vector2(32, 32)), Color(0.69, 0.59, 0.39))
		draw_rect(Rect2(Vector2(-14, -14), Vector2(28, 28)), Color(0.76, 0.66, 0.45, 0.30))

	func _draw_water() -> void:
		draw_rect(Rect2(Vector2(-16, -16), Vector2(32, 32)), Color(0.22, 0.48, 0.74))
		draw_rect(Rect2(Vector2(-14, -14), Vector2(28, 28)), Color(0.34, 0.64, 0.90, 0.35))

	func _draw_dirt() -> void:
		draw_rect(Rect2(Vector2(-16, -16), Vector2(32, 32)), Color(0.47, 0.34, 0.20))
		draw_rect(Rect2(Vector2(-14, -14), Vector2(28, 28)), Color(0.58, 0.42, 0.24, 0.25))

class PlotLayerNode:
	extends Node2D

	func _process(_delta: float) -> void:
		queue_redraw()

	func _draw() -> void:
		var main := get_parent().get_parent()
		if main == null:
			return

		for owner in main.player_plots.keys():
			var plot: Dictionary = main.player_plots[owner]
			var origin: Vector2 = plot["origin"]
			var plot_size: Vector2 = plot["size"]
			var is_local: bool = bool(plot["is_local"])
			var rect := Rect2(origin, plot_size)
			var fill := Color(0.74, 0.92, 0.55, 0.10) if is_local else Color(0.80, 0.86, 0.66, 0.06)
			var border := Color(1.0, 0.95, 0.55, 0.95) if is_local else Color(0.65, 0.78, 0.55, 0.70)
			draw_rect(rect, fill)
			draw_rect(rect, border, false, 2.0)
			draw_string(ThemeDB.fallback_font, origin + Vector2(6, 14), str(plot["display_name"]), HORIZONTAL_ALIGNMENT_LEFT, plot_size.x - 12, 11, border)

class GroundNode:
	extends Node2D

	func _process(_delta: float) -> void:
		queue_redraw()

	func _draw() -> void:
		var origin: Vector2 = -get_parent().position
		var viewport_size: Vector2 = get_viewport_rect().size
		var min_tile: int = int(floor((origin.x - TILE_SIZE) / TILE_SIZE))
		var max_tile_x: int = int(ceil((origin.x + viewport_size.x + TILE_SIZE) / TILE_SIZE))
		var min_tile_y: int = int(floor((origin.y - TILE_SIZE) / TILE_SIZE))
		var max_tile_y: int = int(ceil((origin.y + viewport_size.y + TILE_SIZE) / TILE_SIZE))
		for y in range(min_tile_y, max_tile_y):
			for x in range(min_tile, max_tile_x):
				var base: Color = Color(0.41, 0.66, 0.38) if (x + y) % 2 == 0 else Color(0.36, 0.61, 0.34)
				var pos: Vector2 = Vector2(x * TILE_SIZE, y * TILE_SIZE)
				draw_rect(Rect2(pos, Vector2(TILE_SIZE, TILE_SIZE)), base)
				draw_rect(Rect2(pos + Vector2(2, 2), Vector2(4, 4)), Color(0.56, 0.76, 0.42, 0.45))
		_draw_path(min_tile, max_tile_x)
		_draw_trees(origin, viewport_size)

	func _draw_path(min_tile_x: int, max_tile_x: int) -> void:
		for tile_x in range(min_tile_x, max_tile_x):
			var x := tile_x * TILE_SIZE
			draw_rect(Rect2(Vector2(x, 0), Vector2(TILE_SIZE, TILE_SIZE)), Color(0.72, 0.62, 0.42))
			draw_rect(Rect2(Vector2(x, TILE_SIZE - 4), Vector2(TILE_SIZE, 4)), Color(0.56, 0.47, 0.31))

	func _draw_trees(origin: Vector2, viewport_size: Vector2) -> void:
		var min_x: int = int(floor((origin.x - 160.0) / 128.0)) * 128
		var max_x: int = int(ceil((origin.x + viewport_size.x + 160.0) / 128.0)) * 128
		var min_y: int = int(floor((origin.y - 160.0) / 160.0)) * 160
		var max_y: int = int(ceil((origin.y + viewport_size.y + 160.0) / 160.0)) * 160
		for x in range(min_x + 32, max_x, 128):
			_draw_tree(Vector2(x, -168 + int(abs(x) / 128) % 2 * 24))
		for x in range(min_x + 80, max_x, 128):
			_draw_tree(Vector2(x, 176 + int(abs(x) / 128) % 2 * 24))
		for y in range(min_y + 96, max_y, 160):
			_draw_tree(Vector2(-360 + int(abs(y) / 160) % 2 * 32, y))
			_draw_tree(Vector2(360 - int(abs(y) / 160) % 2 * 32, y))

	func _draw_tree(pos: Vector2) -> void:
		draw_rect(Rect2(pos + Vector2(-5, 16), Vector2(10, 16)), Color(0.36, 0.20, 0.10))
		draw_rect(Rect2(pos + Vector2(-18, -2), Vector2(36, 22)), Color(0.16, 0.43, 0.22))
		draw_rect(Rect2(pos + Vector2(-12, -12), Vector2(24, 18)), Color(0.20, 0.55, 0.26))
		draw_rect(Rect2(pos + Vector2(-6, -22), Vector2(12, 14)), Color(0.27, 0.65, 0.30))

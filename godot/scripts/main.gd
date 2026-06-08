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
const STATUS_MAX_CHARS := 92

@onready var world: Node2D = $World
@onready var status_label: Label = $Hud/VBox/Status
@onready var name_edit: LineEdit = $Hud/VBox/NameRow/NameEdit
@onready var join_button: Button = $Hud/VBox/NameRow/JoinButton
@onready var recent_label: Label = $Hud/VBox/Recent
@onready var chat_edit: LineEdit = $ChatInput/ChatRow/ChatEdit
@onready var send_button: Button = $ChatInput/ChatRow/SendButton

var client: RefCounted
var avatars: Dictionary = {}
var latest_chat_by_sender: Dictionary = {}
var local_identity := ""
var camera_position := Vector2.ZERO
var camera_initialized := false
var move_elapsed := 0.0
var frame := 0
var smoke_enabled := false
var smoke_name := ""
var smoke_message := ""
var smoke_dx := 0
var smoke_dy := 0

func _ready() -> void:
	world.y_sort_enabled = true
	var ground := GroundNode.new()
	ground.z_index = -1000
	world.add_child(ground)
	_style_ui()
	join_button.pressed.connect(_join_game)
	send_button.pressed.connect(_send_chat)
	chat_edit.text_submitted.connect(func(_text: String) -> void: _send_chat())
	_load_smoke_config()
	if not ClassDB.class_exists("TinyGroveClient"):
		GDExtensionManager.load_extension("res://tinygrove_client.gdextension")
	client = ClassDB.instantiate("TinyGroveClient")
	if client != null:
		client.connect_local()

func _process(delta: float) -> void:
	if client == null:
		return
	frame += 1
	client.poll()
	_run_smoke_actions()
	_update_status()
	_update_chat()
	_update_world()
	_update_camera(delta)
	_handle_movement(delta)

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

func _update_status() -> void:
	var text: String = client.status()
	var error: String = client.last_error()
	if not error.is_empty():
		text += " | " + _short_error(error)
	status_label.text = text

func _update_world() -> void:
	var seen := {}
	local_identity = str(client.local_identity())
	for row in client.players():
		var identity := str(row["identity"])
		seen[identity] = true
		var avatar := _avatar_for(identity)
		avatar.set_world_target(Vector2(float(row["x"]), float(row["y"])))
		avatar.set_meta("display_name", str(row["display_name"]))
		avatar.set_meta("avatar_color", int(row["avatar_color"]))
		avatar.set_meta("bubble", latest_chat_by_sender.get(identity, ""))
		avatar.set_meta("is_local", identity == local_identity)
		avatar.queue_redraw()

	for identity in avatars.keys():
		if not seen.has(identity):
			avatars[identity].queue_free()
			avatars.erase(identity)

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

func _update_chat() -> void:
	latest_chat_by_sender.clear()
	var recent := ""
	for row in client.chat_messages():
		var sender := str(row["sender"])
		var body := str(row["body"])
		latest_chat_by_sender[sender] = body
		recent = "%s: %s" % [str(row["display_name"]), body]
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

class AvatarNode:
	extends Node2D

	var world_target := Vector2.ZERO
	var initialized := false

	func _process(delta: float) -> void:
		if not initialized:
			return

		var t := 1.0 - exp(-AVATAR_SMOOTH_SPEED * delta)
		position = position.lerp(world_target, t)
		z_index = int(position.y)

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
		var bubble: String = str(get_meta("bubble", ""))
		if not bubble.is_empty():
			var preview: String = bubble.substr(0, 28)
			if bubble.length() > 28:
				preview += "..."
			var width: float = clampf(ThemeDB.fallback_font.get_string_size(preview, HORIZONTAL_ALIGNMENT_LEFT, -1, 11).x + 12.0, 36.0, 150.0)
			var rect: Rect2 = Rect2(Vector2(-width * 0.5, -48), Vector2(width, 18))
			draw_rect(rect, Color(0.98, 0.95, 0.82))
			draw_rect(Rect2(rect.position, Vector2(rect.size.x, 2)), Color(0.25, 0.20, 0.15))
			draw_rect(Rect2(rect.position + Vector2(0, rect.size.y - 2), Vector2(rect.size.x, 2)), Color(0.25, 0.20, 0.15))
			draw_rect(Rect2(rect.position, Vector2(2, rect.size.y)), Color(0.25, 0.20, 0.15))
			draw_rect(Rect2(rect.position + Vector2(rect.size.x - 2, 0), Vector2(2, rect.size.y)), Color(0.25, 0.20, 0.15))
			draw_string(ThemeDB.fallback_font, rect.position + Vector2(6, 13), preview, HORIZONTAL_ALIGNMENT_LEFT, width - 12, 11, Color(0.12, 0.10, 0.08))

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

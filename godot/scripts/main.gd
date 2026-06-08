extends Control

const AVATAR_RADIUS := 14.0
const MOVE_REPEAT_SECONDS := 0.10

@onready var world: Node2D = $World
@onready var status_label: Label = $Hud/VBox/Status
@onready var name_edit: LineEdit = $Hud/VBox/NameRow/NameEdit
@onready var join_button: Button = $Hud/VBox/NameRow/JoinButton
@onready var chat_log: RichTextLabel = $Hud/VBox/ChatLog
@onready var chat_edit: LineEdit = $Hud/VBox/ChatRow/ChatEdit
@onready var send_button: Button = $Hud/VBox/ChatRow/SendButton

var client: RefCounted
var avatars: Dictionary = {}
var move_elapsed := 0.0

func _ready() -> void:
	join_button.pressed.connect(_join_game)
	send_button.pressed.connect(_send_chat)
	chat_edit.text_submitted.connect(func(_text: String) -> void: _send_chat())
	if not ClassDB.class_exists("TinyGroveClient"):
		GDExtensionManager.load_extension("res://tinygrove_client.gdextension")
	client = ClassDB.instantiate("TinyGroveClient")
	if client != null:
		client.connect_local()

func _process(delta: float) -> void:
	client.poll()
	_update_status()
	_update_world()
	_update_chat()
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
		text += " | " + error
	status_label.text = text

func _update_world() -> void:
	var seen := {}
	for row in client.players():
		var identity := str(row["identity"])
		seen[identity] = true
		var avatar := _avatar_for(identity)
		avatar.position = Vector2(float(row["x"]), float(row["y"])) + size * 0.5
		avatar.set_meta("display_name", str(row["display_name"]))
		avatar.set_meta("avatar_color", int(row["avatar_color"]))
		avatar.queue_redraw()

	for identity in avatars.keys():
		if not seen.has(identity):
			avatars[identity].queue_free()
			avatars.erase(identity)

func _update_chat() -> void:
	var lines: Array[String] = []
	for row in client.chat_messages():
		lines.append("[b]%s:[/b] %s" % [str(row["display_name"]), str(row["body"])])
	chat_log.text = "\n".join(lines)

func _avatar_for(identity: String) -> Node2D:
	if avatars.has(identity):
		return avatars[identity]

	var avatar := AvatarNode.new()
	world.add_child(avatar)
	avatars[identity] = avatar
	return avatar

class AvatarNode:
	extends Node2D

	func _draw() -> void:
		var color_int := int(get_meta("avatar_color", 0x66CCAA))
		var color := Color.hex(color_int)
		draw_circle(Vector2.ZERO, AVATAR_RADIUS, color)
		draw_arc(Vector2.ZERO, AVATAR_RADIUS, 0.0, TAU, 48, Color(0.08, 0.10, 0.12), 2.0)
		var label := str(get_meta("display_name", "Player"))
		draw_string(ThemeDB.fallback_font, Vector2(-36, -24), label, HORIZONTAL_ALIGNMENT_CENTER, 72, 12, Color.WHITE)

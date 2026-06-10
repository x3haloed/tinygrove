extends PanelContainer

signal closed
signal save_requested(asset_data: Dictionary)

const CANVAS_SIZE = 32
const PIXEL_SCALE = 8

@onready var close_button: Button = $Root/CenterPanel/Header/CloseButton
@onready var canvas: Control = $Root/CenterPanel/CanvasContainer/DrawingCanvas
@onready var pen_button: Button = $Root/ToolsPanel/PenButton
@onready var fill_button: Button = $Root/ToolsPanel/FillButton
@onready var color_grid: GridContainer = $Root/ToolsPanel/ColorGrid
@onready var save_button: Button = $Root/PropsPanel/SaveButton

@onready var name_edit: LineEdit = $Root/PropsPanel/NameEdit
@onready var slug_edit: LineEdit = $Root/PropsPanel/SlugEdit
@onready var kind_button: OptionButton = $Root/PropsPanel/KindButton
@onready var placement_variant_button: OptionButton = $Root/PropsPanel/PlacementVariantButton
@onready var grid_divisor: SpinBox = $Root/PropsPanel/GridDivisor
@onready var collidable_check: CheckButton = $Root/PropsPanel/CollidableCheck
@onready var transparent_check: CheckButton = $Root/PropsPanel/TransparentCheck

var current_color: Color = Color.WHITE
var current_tool: String = "pen"
var image: Image
var texture: ImageTexture

const PALETTE: Array[Color] = [
	Color.BLACK, Color.WHITE, Color.RED, Color.GREEN,
	Color.BLUE, Color.YELLOW, Color.CYAN, Color.MAGENTA,
	Color(0.5, 0.25, 0.0), Color.TRANSPARENT
]

const TILE_KIND_INDEX := 0
const DECORATION_KIND_INDEX := 1
const TILE_PLACEMENT_FULL := "Full"
const DECORATION_PLACEMENT_FULL := "Full"
const DECORATION_PLACEMENT_HALF := "Half Width"
const DECORATION_PLACEMENT_QUARTER := "Quarter"
const TILE_FOOTPRINT := Vector2i(4, 4)
const DECORATION_FULL_FOOTPRINT := Vector2i(4, 4)
const DECORATION_HALF_WIDTH_FOOTPRINT := Vector2i(2, 4)
const DECORATION_QUARTER_FOOTPRINT := Vector2i(2, 2)

func _ready() -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	close_button.pressed.connect(func() -> void:
		visible = false
		closed.emit()
	)
	pen_button.pressed.connect(func() -> void:
		current_tool = "pen"
		fill_button.button_pressed = false
	)
	fill_button.pressed.connect(func() -> void:
		current_tool = "fill"
		pen_button.button_pressed = false
	)
	save_button.pressed.connect(_on_save_pressed)
	
	kind_button.add_item("Tile", TILE_KIND_INDEX)
	kind_button.add_item("Decoration", DECORATION_KIND_INDEX)
	kind_button.item_selected.connect(_on_kind_changed)
	placement_variant_button.add_item(DECORATION_PLACEMENT_FULL, 0)
	placement_variant_button.add_item(DECORATION_PLACEMENT_HALF, 1)
	placement_variant_button.add_item(DECORATION_PLACEMENT_QUARTER, 2)
	
	image = Image.create(CANVAS_SIZE, CANVAS_SIZE, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)
	texture = ImageTexture.create_from_image(image)
	
	canvas.gui_input.connect(_on_canvas_gui_input)
	canvas.draw.connect(_on_canvas_draw)
	
	for c: Color in PALETTE:
		var btn := ColorRect.new()
		btn.color = c
		if c == Color.TRANSPARENT:
			# Visual cue for transparent
			var bg := ColorRect.new()
			bg.color = Color(0.2, 0.2, 0.2)
			bg.set_anchors_preset(PRESET_FULL_RECT)
			btn.add_child(bg)
			var fg := ColorRect.new()
			fg.color = Color(0.8, 0.2, 0.2)
			fg.size = Vector2(16, 16)
			fg.position = Vector2(8, 8)
			btn.add_child(fg)
			
		btn.custom_minimum_size = Vector2(32, 32)
		btn.gui_input.connect(func(event: InputEvent) -> void:
			if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
				current_color = c
		)
		color_grid.add_child(btn)

	_on_kind_changed(kind_button.selected)

func _on_canvas_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion or event is InputEventMouseButton:
		var is_drawing := Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
		var is_erasing := Input.is_mouse_button_pressed(MOUSE_BUTTON_RIGHT)
		
		if not (is_drawing or is_erasing):
			return
			
		var local_pos := canvas.get_local_mouse_position()
		var px := int(local_pos.x / PIXEL_SCALE)
		var py := int(local_pos.y / PIXEL_SCALE)
		
		if px >= 0 and py >= 0 and px < CANVAS_SIZE and py < CANVAS_SIZE:
			var c := Color.TRANSPARENT if is_erasing else current_color
			if current_tool == "pen" or is_erasing:
				image.set_pixel(px, py, c)
				texture.update(image)
				canvas.queue_redraw()
			elif current_tool == "fill" and is_drawing:
				var target_color := image.get_pixel(px, py)
				if target_color != c:
					_flood_fill(px, py, target_color, c)
					texture.update(image)
					canvas.queue_redraw()

func _flood_fill(start_x: int, start_y: int, target_color: Color, replacement_color: Color) -> void:
	var q: Array[Vector2i] = [Vector2i(start_x, start_y)]
	while q.size() > 0:
		var p: Vector2i = q.pop_back()
		if p.x < 0 or p.y < 0 or p.x >= CANVAS_SIZE or p.y >= CANVAS_SIZE:
			continue
		if image.get_pixel(p.x, p.y) == target_color:
			image.set_pixel(p.x, p.y, replacement_color)
			q.push_back(Vector2i(p.x + 1, p.y))
			q.push_back(Vector2i(p.x - 1, p.y))
			q.push_back(Vector2i(p.x, p.y + 1))
			q.push_back(Vector2i(p.x, p.y - 1))

func _on_canvas_draw() -> void:
	canvas.draw_texture_rect(texture, Rect2(0, 0, CANVAS_SIZE * PIXEL_SCALE, CANVAS_SIZE * PIXEL_SCALE), false)
	# Draw grid
	var grid_color := Color(1, 1, 1, 0.1)
	for i in range(CANVAS_SIZE + 1):
		var x := float(i * PIXEL_SCALE) + 0.5
		var y := float(i * PIXEL_SCALE) + 0.5
		canvas.draw_line(Vector2(x, 0.0), Vector2(x, CANVAS_SIZE * PIXEL_SCALE), grid_color)
		canvas.draw_line(Vector2(0.0, y), Vector2(CANVAS_SIZE * PIXEL_SCALE, y), grid_color)

func _on_kind_changed(index: int) -> void:
	var is_tile := index == TILE_KIND_INDEX
	collidable_check.visible = not is_tile
	transparent_check.visible = not is_tile
	placement_variant_button.visible = not is_tile
	_update_kind_defaults(is_tile)
	_update_size_controls(is_tile)
	_refresh_kind_help_text(is_tile)

func _update_kind_defaults(is_tile: bool) -> void:
	if is_tile:
		collidable_check.button_pressed = true
		transparent_check.button_pressed = false
	else:
		if transparent_check.button_pressed == false:
			transparent_check.button_pressed = true

func _update_size_controls(is_tile: bool) -> void:
	if is_tile:
		grid_divisor.value = 4
		grid_divisor.editable = false
		placement_variant_button.selected = 0
	else:
		grid_divisor.editable = true
		if placement_variant_button.selected < 0:
			placement_variant_button.selected = 0

func _refresh_kind_help_text(is_tile: bool) -> void:
	collidable_check.text = "Collidable"
	transparent_check.text = "Transparent"

func _on_save_pressed() -> void:
	var png_bytes := image.save_png_to_buffer()
	var base64_data := Marshalls.raw_to_base64(png_bytes)
	var is_tile := kind_button.selected == TILE_KIND_INDEX
	var kind_name := "tile" if is_tile else "decoration"
	var placement_variant := TILE_PLACEMENT_FULL
	if not is_tile:
		match placement_variant_button.selected:
			1:
				placement_variant = DECORATION_PLACEMENT_HALF
			2:
				placement_variant = DECORATION_PLACEMENT_QUARTER
			_:
				placement_variant = DECORATION_PLACEMENT_FULL
	
	var data := {
		"name": name_edit.text,
		"slug": slug_edit.text,
		"kind": kind_name,
		"grid_divisor": int(grid_divisor.value),
		"placement_variant": placement_variant,
		"collidable": true if is_tile else collidable_check.button_pressed,
		"transparent_allowed": false if is_tile else transparent_check.button_pressed,
		"render_format": "png",
		"render_bytes": base64_data,
		"collision_format": "mask1",
		"collision_bytes": "", # For MVP
		"preview_format": "png",
		"preview_bytes": base64_data
	}
	save_requested.emit(data)

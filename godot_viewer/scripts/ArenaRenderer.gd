extends Node3D
class_name ArenaRenderer

# MockData будет доступен глобально

@export var agent_scene: PackedScene
@export var arena_size := Vector2(800, 600)

var agents_nodes := {}
var websocket_client: WebSocketClient
var camera_controller: Node3D

# UI элементы
@onready var agent_count_label: Label
@onready var match_time_label: Label
@onready var fps_label: Label

func _ready():
	# Находим WebSocket клиент
	websocket_client = get_node("/root/Main/WebSocketClient")
	websocket_client.arena_state_updated.connect(_on_arena_state_updated)
	
	# Настраиваем камеру
	setup_camera()
	
	# Создаем арену
	create_arena_floor()
	
	# Находим UI элементы
	agent_count_label = get_node("/root/Main/UI/BottomPanel/HBoxContainer/AgentCount")
	match_time_label = get_node("/root/Main/UI/BottomPanel/HBoxContainer/MatchTime")
	fps_label = get_node("/root/Main/UI/BottomPanel/HBoxContainer/FPS")
	
	# UI обновления
	var timer = Timer.new()
	timer.wait_time = 0.1
	timer.timeout.connect(_update_ui)
	add_child(timer)
	timer.start()

func setup_camera():
	camera_controller = Node3D.new()
	add_child(camera_controller)
	
	var camera = Camera3D.new()
	camera_controller.add_child(camera)
	
	# Позиционируем камеру для обзора арены
	camera_controller.position = Vector3(0, 15, 20)
	camera_controller.rotation_degrees = Vector3(-25, 0, 0)

func create_arena_floor():
	# Создаем пол арены
	var floor_mesh = MeshInstance3D.new()
	var plane_mesh = PlaneMesh.new()
	plane_mesh.size = Vector2(arena_size.x / 100.0, arena_size.y / 100.0)
	floor_mesh.mesh = plane_mesh
	
	# Материал пола
	var material = StandardMaterial3D.new()
	material.albedo_color = Color(0.1, 0.1, 0.2, 1.0)
	material.metallic = 0.1
	material.roughness = 0.8
	floor_mesh.material_override = material
	
	add_child(floor_mesh)
	
	# Создаем стены арены
	create_arena_walls()

func create_arena_walls():
	var wall_height = 2.0
	var wall_thickness = 0.1
	var half_x = arena_size.x / 200.0
	var half_y = arena_size.y / 200.0
	
	# Позиции стен: север, юг, восток, запад
	var wall_positions = [
		Vector3(0, wall_height/2, half_y),      # Север
		Vector3(0, wall_height/2, -half_y),     # Юг  
		Vector3(half_x, wall_height/2, 0),      # Восток
		Vector3(-half_x, wall_height/2, 0)      # Запад
	]
	
	var wall_scales = [
		Vector3(arena_size.x/100, wall_height, wall_thickness),  # Север/Юг
		Vector3(arena_size.x/100, wall_height, wall_thickness),  # Север/Юг
		Vector3(wall_thickness, wall_height, arena_size.y/100),  # Восток/Запад
		Vector3(wall_thickness, wall_height, arena_size.y/100)   # Восток/Запад
	]
	
	for i in range(4):
		var wall = MeshInstance3D.new()
		var box_mesh = BoxMesh.new()
		box_mesh.size = wall_scales[i]
		wall.mesh = box_mesh
		
		var material = StandardMaterial3D.new()
		material.albedo_color = Color(0.3, 0.3, 0.4, 1.0)
		wall.material_override = material
		
		wall.position = wall_positions[i]
		add_child(wall)

func _on_arena_state_updated(state: Dictionary):
	print("📡 Received arena state with ", state.get("agents", []).size(), " agents")
	
	# Обновляем состояние агентов
	if state.has("agents"):
		update_agents(state.agents)
	
	# Обновляем статистику
	if state.has("statistics"):
		update_statistics(state.statistics)

func update_agents(agents_data: Array):
	print("🤖 Updating ", agents_data.size(), " agents")
	
	# Удаляем агентов, которых больше нет
	for agent_id in agents_nodes.keys():
		var found = false
		for agent_data in agents_data:
			if agent_data.get("id") == agent_id:
				found = true
				break
		if not found:
			print("❌ Removing agent: ", agent_id)
			agents_nodes[agent_id].queue_free()
			agents_nodes.erase(agent_id)
	
	# Обновляем существующих и создаем новых агентов
	for agent_data in agents_data:
		var agent_id = agent_data.get("id")
		
		if agent_id in agents_nodes:
			# Обновляем существующего агента
			update_agent_node(agents_nodes[agent_id], agent_data)
		else:
			# Создаем нового агента
			print("✨ Creating new agent: ", agent_id)
			var agent_node = create_agent_node(agent_data)
			agents_nodes[agent_id] = agent_node
			add_child(agent_node)

func create_agent_node(agent_data: Dictionary) -> Node3D:
	var agent_node = Node3D.new()
	
	# Устанавливаем позицию агента
	var pos_data = agent_data.get("position", {"x": 0, "z": 0})
	var agent_pos = Vector3(pos_data.x / 100.0, 0.5, pos_data.z / 100.0)
	agent_node.position = agent_pos
	
	print("🤖 Creating agent at position: ", agent_pos)
	
	# Визуальное представление агента
	var mesh_instance = MeshInstance3D.new()
	var sphere_mesh = SphereMesh.new()
	sphere_mesh.radius = 0.5
	mesh_instance.mesh = sphere_mesh
	
	# Материал зависит от команды
	var material = StandardMaterial3D.new()
	var team = agent_data.get("team", "")
	match team:
		"red":
			material.albedo_color = Color.RED
		"blue": 
			material.albedo_color = Color.BLUE
		_:
			material.albedo_color = Color.WHITE
	
	material.metallic = 0.3
	material.roughness = 0.7
	mesh_instance.material_override = material
	agent_node.add_child(mesh_instance)
	
	# Метка с именем агента
	var label_3d = Label3D.new()
	label_3d.text = agent_data.get("name", "Agent")
	label_3d.position = Vector3(0, 1, 0)
	label_3d.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	agent_node.add_child(label_3d)
	
	# Полоска здоровья
	var health_bar = create_health_bar(agent_data.get("health", 100))
	agent_node.add_child(health_bar)
	
	return agent_node

func update_agent_node(agent_node: Node3D, agent_data: Dictionary):
	# Обновляем позицию
	var pos_data = agent_data.get("position", {"x": 0, "z": 0})
	var target_pos = Vector3(
		pos_data.x / 100.0, 
		0.3, 
		pos_data.z / 100.0
	)
	
	# Плавное движение
	var tween = create_tween()
	tween.tween_property(agent_node, "position", target_pos, 0.1)
	
	# Обновляем здоровье
	var health_value = agent_data.get("health", 100)
	update_health_bar(agent_node, health_value)

func create_health_bar(health: float) -> Node3D:
	var health_container = Node3D.new()
	health_container.position = Vector3(0, 0.8, 0)
	
	# Фон полоски здоровья
	var bg_mesh = MeshInstance3D.new()
	var bg_box = BoxMesh.new()
	bg_box.size = Vector3(0.6, 0.1, 0.05)
	bg_mesh.mesh = bg_box
	
	var bg_material = StandardMaterial3D.new()
	bg_material.albedo_color = Color(0.2, 0.2, 0.2, 0.8)
	bg_material.flags_transparent = true
	bg_mesh.material_override = bg_material
	health_container.add_child(bg_mesh)
	
	# Полоска здоровья
	var health_mesh = MeshInstance3D.new()
	var health_box = BoxMesh.new()
	health_box.size = Vector3(0.5 * (health / 100.0), 0.08, 0.03)
	health_mesh.mesh = health_box
	
	var health_material = StandardMaterial3D.new()
	if health > 60:
		health_material.albedo_color = Color.GREEN
	elif health > 30:
		health_material.albedo_color = Color.YELLOW
	else:
		health_material.albedo_color = Color.RED
	
	health_mesh.material_override = health_material
	health_mesh.position = Vector3(-(0.5 - 0.5 * (health / 100.0)) / 2, 0.01, 0)
	health_container.add_child(health_mesh)
	
	return health_container

func update_health_bar(agent_node: Node3D, health: float):
	# Находим полоску здоровья и обновляем ее
	var health_container = agent_node.get_child(2)  # Третий ребенок
	if health_container:
		var health_mesh = health_container.get_child(1)
		if health_mesh and health_mesh is MeshInstance3D:
			var health_box = health_mesh.mesh as BoxMesh
			health_box.size.x = 0.5 * (health / 100.0)
			
			# Обновляем цвет
			var material = health_mesh.material_override as StandardMaterial3D
			if health > 60:
				material.albedo_color = Color.GREEN
			elif health > 30:
				material.albedo_color = Color.YELLOW
			else:
				material.albedo_color = Color.RED

func update_statistics(_stats: Dictionary):
	# Обновляем статистику в UI
	pass  # Реализуем позже

func _update_ui():
	# Обновляем UI элементы
	if agent_count_label:
		agent_count_label.text = "Agents: " + str(agents_nodes.size())
	
	if fps_label:
		fps_label.text = "FPS: " + str(Engine.get_frames_per_second())

func _input(event):
	# Управление камерой
	if event is InputEventMouseMotion and Input.is_action_pressed("camera_rotate"):
		camera_controller.rotation_degrees.y -= event.relative.x * 0.5
		camera_controller.rotation_degrees.x -= event.relative.y * 0.5
		camera_controller.rotation_degrees.x = clamp(camera_controller.rotation_degrees.x, -80, 80)
	
	# Зум камерой
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			camera_controller.position = camera_controller.position * 0.9
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			camera_controller.position = camera_controller.position * 1.1

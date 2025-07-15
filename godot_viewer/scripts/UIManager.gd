extends Control
class_name UIManager

@onready var status_label: Label
@onready var agent_count_label: Label
@onready var match_time_label: Label
@onready var fps_label: Label

var websocket_client: WebSocketClient
var start_time: float = 0.0

func _ready():
	# Находим UI элементы
	status_label = get_node("../UI/TopPanel/HBoxContainer/Status")
	agent_count_label = get_node("../UI/BottomPanel/HBoxContainer/AgentCount")
	match_time_label = get_node("../UI/BottomPanel/HBoxContainer/MatchTime")
	fps_label = get_node("../UI/BottomPanel/HBoxContainer/FPS")
	
	# Находим WebSocket клиент
	websocket_client = get_node("../WebSocketClient")
	websocket_client.connection_established.connect(_on_connected)
	websocket_client.connection_lost.connect(_on_disconnected)
	
	start_time = Time.get_time_dict_from_system().hour * 3600 + Time.get_time_dict_from_system().minute * 60 + Time.get_time_dict_from_system().second

func _process(_delta):
	# Обновляем FPS
	if fps_label:
		fps_label.text = "FPS: " + str(Engine.get_frames_per_second())
	
	# Обновляем время матча
	if match_time_label:
		var current_time = Time.get_time_dict_from_system().hour * 3600 + Time.get_time_dict_from_system().minute * 60 + Time.get_time_dict_from_system().second
		var elapsed = current_time - start_time
		var minutes = int(elapsed / 60)
		var seconds = int(elapsed % 60)
		match_time_label.text = "Time: %02d:%02d" % [minutes, seconds]

func _on_connected():
	if status_label:
		status_label.text = "● Connected"
		status_label.modulate = Color.GREEN

func _on_disconnected():
	if status_label:
		status_label.text = "● Disconnected"
		status_label.modulate = Color.RED

func update_agent_count(count: int):
	if agent_count_label:
		agent_count_label.text = "Agents: " + str(count)

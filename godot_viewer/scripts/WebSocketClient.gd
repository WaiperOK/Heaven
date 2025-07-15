extends Node
class_name WebSocketClient

signal arena_state_updated(state: Dictionary)
signal connection_established()
signal connection_lost()

var socket := WebSocketPeer.new()
var connection_url := "ws://localhost:8082"
var reconnect_timer := Timer.new()
var ws_connected := false
var demo_mode := true  # Для демонстрации без сервера
var demo_timer := Timer.new()

func _ready():
	# Настраиваем таймер переподключения
	add_child(reconnect_timer)
	reconnect_timer.wait_time = 3.0
	reconnect_timer.timeout.connect(_attempt_reconnect)
	
	# Настраиваем демо режим
	add_child(demo_timer)
	demo_timer.wait_time = 1.0  # Обновляем каждую секунду
	demo_timer.timeout.connect(_send_demo_data)
	
	if demo_mode:
		print("🎮 Starting in DEMO mode")
		demo_timer.start()
		# Эмулируем подключение
		ws_connected = true
		connection_established.emit()
	else:
		# Пытаемся подключиться к реальному серверу
		connect_to_server()

func connect_to_server():
	print("🚀 Connecting to Arena Core at: ", connection_url)
	var error = socket.connect_to_url(connection_url)
	
	if error != OK:
		print("❌ Failed to connect: ", error)
		reconnect_timer.start()
	else:
		print("🔄 Connection initiated...")

func _process(_delta):
	# В демо режиме не обрабатываем socket
	if demo_mode:
		return
		
	socket.poll()
	var state = socket.get_ready_state()
	
	match state:
		WebSocketPeer.STATE_OPEN:
			if not ws_connected:
				ws_connected = true
				reconnect_timer.stop()
				print("✅ Connected to Arena Core!")
				connection_established.emit()
			
			# Получаем сообщения
			while socket.get_available_packet_count():
				var packet = socket.get_packet()
				var json_string = packet.get_string_from_utf8()
				var json = JSON.new()
				var parse_result = json.parse(json_string)
				
				if parse_result == OK:
					var data = json.data
					arena_state_updated.emit(data)
				else:
					print("⚠️ Failed to parse JSON: ", json_string)
		
		WebSocketPeer.STATE_CLOSING:
			print("🔄 Connection closing...")
		
		WebSocketPeer.STATE_CLOSED:
			if ws_connected:
				ws_connected = false
				print("❌ Connection lost!")
				connection_lost.emit()
				reconnect_timer.start()

func _attempt_reconnect():
	if not ws_connected:
		print("🔄 Attempting to reconnect...")
		connect_to_server()

func send_command(command: Dictionary):
	if ws_connected:
		var json_string = JSON.stringify(command)
		socket.send_text(json_string)
	else:
		print("⚠️ Cannot send command: not connected")

func _send_demo_data():
	# Отправляем mock данные для демонстрации
	var mock_data = MockData.get_mock_arena_state()
	
	# Добавляем небольшую анимацию агентов
	for i in range(mock_data.agents.size()):
		var agent = mock_data.agents[i]
		var time_offset = Time.get_time_dict_from_system().second + i * 2
		agent.position.x += sin(time_offset * 0.5) * 20
		agent.position.z += cos(time_offset * 0.3) * 15
		
		# Случайные изменения здоровья
		agent.health += randf_range(-2, 2)
		agent.health = clamp(agent.health, 10, 100)
	
	arena_state_updated.emit(mock_data)

func _exit_tree():
	if socket:
		socket.close()

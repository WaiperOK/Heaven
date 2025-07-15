extends Node
class_name MockData

# Создаем тестовые данные для демонстрации
static func get_mock_arena_state() -> Dictionary:
	return {
		"agents": [
			{
				"id": "agent_1",
				"name": "Gladiator Alpha",
				"position": {"x": 100, "z": 50},
				"health": 85,
				"energy": 90,
				"team": "red",
				"status": "fighting"
			},
			{
				"id": "agent_2", 
				"name": "Warrior Beta",
				"position": {"x": -80, "z": -30},
				"health": 65,
				"energy": 75,
				"team": "blue", 
				"status": "defending"
			},
			{
				"id": "agent_3",
				"name": "Scout Gamma", 
				"position": {"x": 200, "z": 150},
				"health": 45,
				"energy": 95,
				"team": "red",
				"status": "moving"
			}
		],
		"match_id": "demo_match_001",
		"current_tick": 1247,
		"match_time": 187.5,
		"arena_bounds": {"x": 800, "y": 0, "z": 600},
		"statistics": {
			"total_agents": 3,
			"active_agents": 3,
			"eliminated_agents": 0,
			"average_health": 65.0,
			"match_duration": 187.5
		}
	}

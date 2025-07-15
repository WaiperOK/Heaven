use bevy::prelude::*;
use bevy::render::mesh::shape;
use bevy::input::Input;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_plot::{Line, Plot};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::Rng;
use reqwest::Client;
use tokio::runtime::Runtime;
use std::process::{Command, Child, Stdio};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::net::TcpListener;

// Arena Theme Resource
#[derive(Resource, Debug, Clone)]
pub struct ArenaTheme {
    pub name: String,
    pub floor_color: Color,
    pub floor_metallic: f32,
    pub floor_roughness: f32,
    pub wall_color: Color,
    pub wall_metallic: f32,
    pub wall_roughness: f32,
    pub sky_color: Color,
    pub light_intensity: f32,
    pub light_color: Color,
}

impl Default for ArenaTheme {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            floor_color: Color::rgb(0.1, 0.1, 0.2),
            floor_metallic: 0.1,
            floor_roughness: 0.8,
            wall_color: Color::rgb(0.3, 0.3, 0.4),
            wall_metallic: 0.1,
            wall_roughness: 0.8,
            sky_color: Color::rgb(0.2, 0.2, 0.3),
            light_intensity: 1.0,
            light_color: Color::WHITE,
        }
    }
}

impl ArenaTheme {
    pub fn forest() -> Self {
        Self {
            name: "Forest".to_string(),
            floor_color: Color::rgb(0.2, 0.4, 0.1),
            floor_metallic: 0.0,
            floor_roughness: 0.9,
            wall_color: Color::rgb(0.4, 0.2, 0.1),
            wall_metallic: 0.0,
            wall_roughness: 0.8,
            sky_color: Color::rgb(0.4, 0.6, 0.8),
            light_intensity: 0.8,
            light_color: Color::rgb(1.0, 0.9, 0.7),
        }
    }

    pub fn desert() -> Self {
        Self {
            name: "Desert".to_string(),
            floor_color: Color::rgb(0.8, 0.6, 0.3),
            floor_metallic: 0.0,
            floor_roughness: 0.9,
            wall_color: Color::rgb(0.6, 0.4, 0.2),
            wall_metallic: 0.0,
            wall_roughness: 0.8,
            sky_color: Color::rgb(0.9, 0.7, 0.5),
            light_intensity: 1.2,
            light_color: Color::rgb(1.0, 0.8, 0.6),
        }
    }

    pub fn ice() -> Self {
        Self {
            name: "Ice".to_string(),
            floor_color: Color::rgb(0.7, 0.8, 0.9),
            floor_metallic: 0.3,
            floor_roughness: 0.1,
            wall_color: Color::rgb(0.5, 0.7, 0.8),
            wall_metallic: 0.2,
            wall_roughness: 0.2,
            sky_color: Color::rgb(0.8, 0.9, 1.0),
            light_intensity: 0.9,
            light_color: Color::rgb(0.9, 0.9, 1.0),
        }
    }

    pub fn volcano() -> Self {
        Self {
            name: "Volcano".to_string(),
            floor_color: Color::rgb(0.3, 0.1, 0.1),
            floor_metallic: 0.0,
            floor_roughness: 0.8,
            wall_color: Color::rgb(0.2, 0.05, 0.05),
            wall_metallic: 0.0,
            wall_roughness: 0.9,
            sky_color: Color::rgb(0.5, 0.2, 0.1),
            light_intensity: 1.3,
            light_color: Color::rgb(1.0, 0.5, 0.3),
        }
    }

    pub fn cyberpunk() -> Self {
        Self {
            name: "Cyberpunk".to_string(),
            floor_color: Color::rgb(0.1, 0.1, 0.1),
            floor_metallic: 0.9,
            floor_roughness: 0.1,
            wall_color: Color::rgb(0.05, 0.05, 0.05),
            wall_metallic: 0.8,
            wall_roughness: 0.2,
            sky_color: Color::rgb(0.0, 0.0, 0.0),
            light_intensity: 1.5,
            light_color: Color::rgb(0.2, 0.8, 1.0),
        }
    }

    pub fn get_available_themes() -> Vec<ArenaTheme> {
        vec![
            ArenaTheme::default(),
            ArenaTheme::forest(),
            ArenaTheme::desert(),
            ArenaTheme::ice(),
            ArenaTheme::volcano(),
            ArenaTheme::cyberpunk(),
        ]
    }
}

// Agent Creation Resource
#[derive(Resource, Default)]
pub struct AgentCreator {
    pub window_open: bool,
    pub agent_name: String,
    pub selected_team: String,
    pub selected_role: String,
    pub spawn_position: Vec3,
    pub health: f32,
    pub energy: f32,
    pub ai_enabled: bool,
    pub custom_prompt: String,
}

impl AgentCreator {
    pub fn new() -> Self {
        Self {
            window_open: false,
            agent_name: "New Agent".to_string(),
            selected_team: "red".to_string(),
            selected_role: "warrior".to_string(),
            spawn_position: Vec3::new(0.0, 0.5, 0.0),
            health: 100.0,
            energy: 100.0,
            ai_enabled: true,
            custom_prompt: "You are a skilled warrior ready for battle.".to_string(),
        }
    }

    pub fn get_available_teams() -> Vec<&'static str> {
        vec!["red", "blue", "green", "yellow", "purple"]
    }

    pub fn get_available_roles() -> Vec<&'static str> {
        vec!["warrior", "scout", "mage", "archer", "tank"]
    }
}

// Structures for Ollama API
#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaResponse {
    pub response: String,
    pub done: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModel>,
}

// Компоненты для анимации агентов
#[derive(Component)]
pub struct AgentAnimation {
    pub animation_type: String,
    pub duration: f32,
    pub start_time: f32,
}

#[derive(Component)]
pub struct AgentEffect {
    pub effect_type: String,
    pub intensity: f32,
    pub duration: f32,
    pub start_time: f32,
}

// Компоненты для AI агентов
#[derive(Component, Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub health: f32,
    pub energy: f32,
    pub team: String,
    pub status: String,
    pub ai_enabled: bool,
    pub decision_cooldown: Timer,
}

#[derive(Component)]
pub struct AgentVisual;

#[derive(Component)]
pub struct SelectionOutline {
    pub selected: bool,
    pub hovered: bool,
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct AIBrain {
    pub model: String,
    pub context: String,
    pub last_action: String,
    pub thinking: bool,
}

#[derive(Component)]
pub struct Movement {
    pub velocity: Vec3,
    pub target_position: Option<Vec3>,
    pub speed: f32,
}

#[derive(Component)]
pub struct Combat {
    pub attack_damage: f32,
    pub attack_range: f32,
    pub defense: f32,
    pub last_attack_time: f32,
    pub attack_cooldown: f32,
}

// Сцены и уровни
#[derive(Resource)]
pub struct SceneManager {
    pub current_scene: String,
    pub available_scenes: Vec<String>,
    pub scene_creator_open: bool,
    pub new_scene_name: String,
}

impl Default for SceneManager {
    fn default() -> Self {
        Self {
            current_scene: "Arena Basic".to_string(),
            available_scenes: vec![
                "Arena Basic".to_string(),
                "Maze Challenge".to_string(),
                "Battle Royale".to_string(),
                "Capture the Flag".to_string(),
            ],
            scene_creator_open: false,
            new_scene_name: String::new(),
        }
    }
}

// Система промптов для агентов
#[derive(Resource)]
pub struct AgentPrompts {
    pub prompts: HashMap<String, String>,
    pub custom_prompt_window: bool,
    pub selected_agent: String,
    pub temp_prompt: String,
}

impl Default for AgentPrompts {
    fn default() -> Self {
        let mut prompts = HashMap::new();
        prompts.insert("red_gladiator".to_string(), "Найди ключ в арене".to_string());
        prompts.insert("blue_warrior".to_string(), "Атакуй красных агентов".to_string());
        prompts.insert("red_scout".to_string(), "Защищай свою команду".to_string());
        
        Self {
            prompts,
            custom_prompt_window: false,
            selected_agent: String::new(),
            temp_prompt: String::new(),
        }
    }
}

// Система обучения и метрик
#[derive(Resource)]
pub struct TrainingSystem {
    pub is_training: bool,
    pub current_epoch: u32,
    pub total_epochs: u32,
    pub steps_in_epoch: u32,
    pub current_step: u32,
    pub learning_rate: f32,
    pub loss_history: Vec<f32>,
    pub reward_history: Vec<f32>,
    pub accuracy_history: Vec<f32>,
    pub training_window_open: bool,
}

// Система для настройки внешнего вида агентов
#[derive(Resource)]
pub struct AgentAppearance {
    pub agent_shapes: HashMap<String, String>, // agent_id -> shape_type
    pub agent_colors: HashMap<String, [f32; 3]>, // agent_id -> RGB color
    pub available_shapes: Vec<String>,
    pub appearance_window_open: bool,
    pub selected_agent_for_appearance: String,
}

impl Default for AgentAppearance {
    fn default() -> Self {
        let mut shapes = HashMap::new();
        shapes.insert("red_gladiator".to_string(), "humanoid".to_string());
        shapes.insert("blue_warrior".to_string(), "humanoid".to_string());
        shapes.insert("red_scout".to_string(), "humanoid".to_string());
        
        let mut colors = HashMap::new();
        colors.insert("red_gladiator".to_string(), [0.8, 0.2, 0.2]);
        colors.insert("blue_warrior".to_string(), [0.2, 0.2, 0.8]);
        colors.insert("red_scout".to_string(), [0.8, 0.2, 0.2]);
        
        Self {
            agent_shapes: shapes,
            agent_colors: colors,
            available_shapes: vec![
                "humanoid".to_string(),
                "robot".to_string(),
                "sphere".to_string(),
                "cube".to_string(),
                "cylinder".to_string(),
            ],
            appearance_window_open: false,
            selected_agent_for_appearance: "red_gladiator".to_string(),
        }
    }
}

// Система симуляции времени жизни агентов
#[derive(Resource)]
pub struct TimeSimulation {
    pub current_time: f32,
    pub time_scale: f32, // 1.0 = реальное время, 60.0 = 1 минута = 1 секунда
    pub simulated_days: u32,
    pub simulated_hours: u32,
    pub simulated_minutes: u32,
    pub paused: bool,
    pub window_open: bool,
    pub agent_lifespans: HashMap<String, f32>, // agent_id -> жизненный цикл в днях
    pub agent_activities: HashMap<String, Vec<String>>, // agent_id -> список активностей
}

impl Default for TimeSimulation {
    fn default() -> Self {
        let mut lifespans = HashMap::new();
        lifespans.insert("red_gladiator".to_string(), 7.0); // 7 дней по умолчанию
        lifespans.insert("blue_warrior".to_string(), 7.0);
        lifespans.insert("red_scout".to_string(), 7.0);
        
        let mut activities = HashMap::new();
        activities.insert("red_gladiator".to_string(), vec!["Born".to_string()]);
        activities.insert("blue_warrior".to_string(), vec!["Born".to_string()]);
        activities.insert("red_scout".to_string(), vec!["Born".to_string()]);
        
        Self {
            current_time: 0.0,
            time_scale: 60.0, // 1 минута = 1 секунда
            simulated_days: 0,
            simulated_hours: 0,
            simulated_minutes: 0,
            paused: false,
            window_open: false,
            agent_lifespans: lifespans,
            agent_activities: activities,
        }
    }
}

// Система чата между агентами
#[derive(Resource)]
pub struct AgentChat {
    pub chat_history: Vec<ChatMessage>,
    pub window_open: bool,
    pub auto_chat_enabled: bool,
    pub chat_frequency: f32, // секунды между автоматическими сообщениями
    pub last_chat_time: f32,
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub sender_id: String,
    pub sender_name: String,
    pub receiver_id: String,
    pub receiver_name: String,
    pub message: String,
    pub timestamp: f32,
    pub message_type: String, // "greeting", "question", "response", "action"
}

impl Default for AgentChat {
    fn default() -> Self {
        Self {
            chat_history: Vec::new(),
            window_open: false,
            auto_chat_enabled: true,
            chat_frequency: 10.0, // каждые 10 секунд
            last_chat_time: 0.0,
        }
    }
}

// Система Drag & Drop для настройки арены
#[derive(Resource)]
pub struct ArenaDragDrop {
    pub arena_editor_open: bool,
    pub dragging: bool,
    pub selected_object: String,
    pub drag_start_pos: egui::Vec2,
    pub available_objects: Vec<String>,
    pub placed_objects: Vec<PlacedObject>,
}

#[derive(Clone)]
pub struct PlacedObject {
    pub object_type: String,
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec3,
}

impl Default for ArenaDragDrop {
    fn default() -> Self {
        Self {
            arena_editor_open: false,
            dragging: false,
            selected_object: "Box".to_string(),
            drag_start_pos: egui::Vec2::ZERO,
            available_objects: vec![
                "Box".to_string(),
                "Sphere".to_string(),
                "Cylinder".to_string(),
                "Wall".to_string(),
                "Pillar".to_string(),
            ],
            placed_objects: Vec::new(),
        }
    }
}

// Настройки движения агентов
#[derive(Resource)]
pub struct MovementSettings {
    pub movement_speed: f32,
    pub movement_smoothness: f32,
    pub show_movement_lines: bool,
    pub show_attack_range: bool,
    pub settings_window_open: bool,
    pub agent_selection_enabled: bool,
}

// Система выделения и перемещения агентов
#[derive(Resource)]
pub struct AgentSelection {
    pub selected_agents: Vec<String>,
    pub dragging_agent: Option<String>,
    pub selection_mode: bool,
    pub gizmo_enabled: bool,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            movement_speed: 3.0,
            movement_smoothness: 5.0,
            show_movement_lines: true,
            show_attack_range: false,
            settings_window_open: false,
            agent_selection_enabled: true,
        }
    }
}

impl Default for AgentSelection {
    fn default() -> Self {
        Self {
            selected_agents: Vec::new(),
            dragging_agent: None,
            selection_mode: false,
            gizmo_enabled: true,
        }
    }
}

// Настройки темы интерфейса
#[derive(Resource)]
pub struct ThemeSettings {
    pub dark_mode: bool,
    pub theme_window_open: bool,
    pub accent_color: [f32; 3],
    pub background_alpha: f32,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            dark_mode: true,
            theme_window_open: false,
            accent_color: [0.2, 0.6, 1.0], // Blue
            background_alpha: 0.9,
        }
    }
}

// Настройки горячих клавиш
#[derive(Resource)]
pub struct HotkeySettings {
    pub hotkey_window_open: bool,
    pub custom_hotkeys: std::collections::HashMap<String, String>,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        let mut hotkeys = std::collections::HashMap::new();
        hotkeys.insert("Camera Forward".to_string(), "W".to_string());
        hotkeys.insert("Camera Backward".to_string(), "S".to_string());
        hotkeys.insert("Camera Left".to_string(), "A".to_string());
        hotkeys.insert("Camera Right".to_string(), "D".to_string());
        hotkeys.insert("Camera Up".to_string(), "Space".to_string());
        hotkeys.insert("Camera Down".to_string(), "Ctrl".to_string());
        hotkeys.insert("Toggle Inspector".to_string(), "F12".to_string());
        
        Self {
            hotkey_window_open: false,
            custom_hotkeys: hotkeys,
        }
    }
}

impl Default for TrainingSystem {
    fn default() -> Self {
        Self {
            is_training: false,
            current_epoch: 0,
            total_epochs: 10,
            steps_in_epoch: 100,
            current_step: 0,
            learning_rate: 0.001,
            loss_history: Vec::new(),
            reward_history: Vec::new(),
            accuracy_history: Vec::new(),
            training_window_open: false,
        }
    }
}

// Соединение с Ollama
#[derive(Resource)]
pub struct OllamaConnection {
    pub connected: bool,
    pub url: String,
    pub model: String,
    pub status: String,
    pub client: Client,
    pub runtime: Runtime,
    pub available_models: Vec<String>,
    pub downloading_model: bool,
    pub download_progress: f32,
    pub model_to_download: String,
}

// Система управления процессом Ollama
#[derive(Resource)]
pub struct OllamaProcess {
    pub process: Option<Child>,
    pub status: String,
    pub auto_start: bool,
    pub logs: Arc<Mutex<Vec<String>>>,
    pub window_open: bool,
    pub process_id: Option<u32>,
    pub start_time: Option<std::time::Instant>,
    pub restart_attempts: u32,
}

impl Default for OllamaProcess {
    fn default() -> Self {
        Self {
            process: None,
            status: "Not started".to_string(),
            auto_start: false, // Отключаем автозапуск по умолчанию
            logs: Arc::new(Mutex::new(Vec::new())),
            window_open: false,
            process_id: None,
            start_time: None,
            restart_attempts: 0,
        }
    }
}

impl Default for OllamaConnection {
    fn default() -> Self {
        Self {
            connected: false,
            url: "http://localhost:11434".to_string(),
            model: "llama3.2:1b".to_string(),
            status: "Disconnected".to_string(),
            client: Client::new(),
            runtime: Runtime::new().unwrap(),
            available_models: Vec::new(),
            downloading_model: false,
            download_progress: 0.0,
            model_to_download: "llama3.2:1b".to_string(),
        }
    }
}

// Система логов и уведомлений
#[derive(Resource)]
pub struct LogSystem {
    pub logs: Vec<String>,
    pub show_logs: bool,
    pub max_logs: usize,
}

impl Default for LogSystem {
    fn default() -> Self {
        Self {
            logs: Vec::new(),
            show_logs: true,
            max_logs: 100,
        }
    }
}

impl LogSystem {
    pub fn add_log(&mut self, message: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let log_entry = format!("[{}] {}", timestamp % 10000, message);
        
        self.logs.push(log_entry);
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }
        println!("📝 {}", message);
    }
}

// Состояние выполнения промптов
#[derive(Resource)]
pub struct PromptExecution {
    pub running: bool,
    pub current_agent: String,
    pub results: HashMap<String, String>,
}

impl Default for PromptExecution {
    fn default() -> Self {
        Self {
            running: false,
            current_agent: String::new(),
            results: HashMap::new(),
        }
    }
}

// Ресурсы для состояния арены
#[derive(Resource, Default)]
pub struct ArenaState {
    pub agents: HashMap<String, Agent>,
    pub match_time: f32,
    pub total_agents: u32,
    pub connection_status: String,
}

#[derive(Resource)]
pub struct DemoMode {
    pub enabled: bool,
    pub timer: Timer,
}

impl Default for DemoMode {
    fn default() -> Self {
        Self {
            enabled: true,
            timer: Timer::from_seconds(2.0, TimerMode::Repeating), // Увеличили интервал для снижения нагрузки
        }
    }
}

/// Main entry point for the Heaven AI Arena application
/// Sets up the Bevy app with all necessary plugins, resources, and systems
fn main() {
    App::new()
        // Configure default plugins with custom window settings
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "🌌 Heaven AI Arena - Bevy Viewer".into(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Add UI and debugging plugins
        .add_plugins(EguiPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .init_resource::<ArenaState>()
        // .init_resource::<DemoMode>() // УБРАЛИ ДЕМО-РЕЖИМ
        .init_resource::<SceneManager>()
        .init_resource::<AgentPrompts>()
        .init_resource::<TrainingSystem>()
        .init_resource::<OllamaConnection>()
        .init_resource::<LogSystem>()
        .init_resource::<PromptExecution>()
        .init_resource::<ArenaTheme>()
        .init_resource::<AgentCreator>()
        .init_resource::<LastSceneState>()
        .init_resource::<LastGeneratedScene>()
        .init_resource::<AgentAppearance>()
        .init_resource::<MovementSettings>()
        .init_resource::<AgentSelection>()
        .init_resource::<ArenaDragDrop>()
        .init_resource::<ThemeSettings>()
        .init_resource::<HotkeySettings>()
        .init_resource::<TimeSimulation>()
        .init_resource::<AgentChat>()
        .init_resource::<AgentIdGenerator>()
        .init_resource::<OllamaProcess>()
        .add_systems(Startup, (setup_arena, setup_real_agents))
        .add_systems(Update, update_agents_system)
        .add_systems(Update, (arena_gui_system, arena_theme_system))
        .add_systems(Update, camera_controls)
        .add_systems(Update, (
            movement_system,
            combat_system,
            ai_decision_system,
            agent_respawn_system,
        ))
        .add_systems(Update, (
            prompt_execution_system,
            scene_generation_system,
            ollama_connection_system,
            scene_transition_system,
            agent_animation_system,
            walking_animation_system,
            agent_effects_system,
            agent_selection_system,
        ))
        .add_systems(Update, (
            time_simulation_system,
            agent_chat_system,
            training_simulation_system,
            ollama_process_system,
            additional_windows_system,
        ))
        .run();
}

// Система управления процессом Ollama
fn ollama_process_system(
    mut ollama_process: ResMut<OllamaProcess>,
    mut log_system: ResMut<LogSystem>,
    time: Res<Time>,
) {
    // Выполняем проверку только раз в секунду
    static mut LAST_CHECK_TIME: f32 = 0.0;
    let current_time = time.elapsed_seconds();
    
    unsafe {
        if current_time - LAST_CHECK_TIME < 1.0 {
            return; // Пропускаем выполнение
        }
        LAST_CHECK_TIME = current_time;
    }
    
    // Автоматический запуск при старте
    if ollama_process.auto_start && ollama_process.process.is_none() {
        start_ollama_server(&mut ollama_process, &mut log_system);
    }
    
    // Проверяем статус процесса
    if let Some(ref mut process) = ollama_process.process {
        match process.try_wait() {
            Ok(Some(status)) => {
                // Процесс завершился
                ollama_process.status = format!("Process exited with: {}", status);
                ollama_process.process = None;
                ollama_process.process_id = None;
                log_system.add_log(format!("❌ Ollama process exited: {}", status));
                
                // Попытка перезапуска только если включен автозапуск
                if ollama_process.auto_start && ollama_process.restart_attempts < 3 {
                    ollama_process.restart_attempts += 1;
                    log_system.add_log(format!("🔄 Attempting to restart Ollama (attempt {})", ollama_process.restart_attempts));
                    // Увеличиваем задержку между попытками
                    thread::sleep(Duration::from_secs(5));
                    start_ollama_server(&mut ollama_process, &mut log_system);
                } else if ollama_process.restart_attempts >= 3 {
                    ollama_process.auto_start = false;
                    log_system.add_log("❌ Maximum restart attempts reached. Auto-start disabled.".to_string());
                }
            }
            Ok(None) => {
                // Процесс все еще работает
                if let Some(start_time) = ollama_process.start_time {
                    let uptime = start_time.elapsed().as_secs();
                    ollama_process.status = format!("Running (uptime: {}s)", uptime);
                }
            }
            Err(e) => {
                ollama_process.status = format!("Error checking process: {}", e);
                log_system.add_log(format!("❌ Error checking Ollama process: {}", e));
            }
        }
    }
}

/// Check if a specific port is available for binding
/// Returns true if the port can be bound to, false otherwise
fn is_port_available(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

/// Find an available port starting from a base port
/// Searches through the next 100 ports from the base port
/// Returns Some(port) if found, None if no available ports
fn find_available_port(base_port: u16) -> Option<u16> {
    for port in base_port..base_port + 100 {
        if is_port_available(port) {
            return Some(port);
        }
    }
    None
}

/// Start Ollama server with automatic port conflict resolution
/// If the default port 11434 is taken, it will find an alternative port
/// and set the OLLAMA_HOST environment variable accordingly
fn start_ollama_server(ollama_process: &mut OllamaProcess, log_system: &mut LogSystem) {
    log_system.add_log("🚀 Starting Ollama server...".to_string());
    
    // Check if default port 11434 is available
    if !is_port_available(11434) {
        log_system.add_log("⚠️  Default Ollama port 11434 is already in use.".to_string());
        if let Some(available_port) = find_available_port(11435) {
            log_system.add_log(format!("🔄 Using alternative port: {}", available_port));
            // Set environment variable for Ollama to use different port
            std::env::set_var("OLLAMA_HOST", format!("127.0.0.1:{}", available_port));
        } else {
            log_system.add_log("❌ No available ports found. Please close other Ollama instances.".to_string());
            return;
        }
    }
    
    // Try to start ollama serve
    match Command::new("ollama")
        .arg("serve")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            ollama_process.process_id = child.id().into();
            ollama_process.start_time = Some(std::time::Instant::now());
            ollama_process.status = "Starting...".to_string();
            ollama_process.restart_attempts = 0;
            
            log_system.add_log(format!("✅ Ollama server started with PID: {}", child.id()));
            
            // Запускаем поток для чтения логов
            if let Some(stdout) = child.stdout.take() {
                let logs = Arc::clone(&ollama_process.logs);
                thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if let Ok(mut logs) = logs.lock() {
                                logs.push(format!("[STDOUT] {}", line));
                                if logs.len() > 1000 {
                                    logs.remove(0);
                                }
                            }
                        }
                    }
                });
            }
            
            if let Some(stderr) = child.stderr.take() {
                let logs = Arc::clone(&ollama_process.logs);
                thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if let Ok(mut logs) = logs.lock() {
                                logs.push(format!("[STDERR] {}", line));
                                if logs.len() > 1000 {
                                    logs.remove(0);
                                }
                            }
                        }
                    }
                });
            }
            
            ollama_process.process = Some(child);
        }
        Err(e) => {
            ollama_process.status = format!("Failed to start: {}", e);
            log_system.add_log(format!("❌ Failed to start Ollama server: {}", e));
            log_system.add_log("📝 Make sure Ollama is installed and in PATH".to_string());
        }
    }
}

// Система для управления дополнительными окнами
fn additional_windows_system(
    mut contexts: EguiContexts,
    mut time_simulation: ResMut<TimeSimulation>,
    mut agent_chat: ResMut<AgentChat>,
    mut ollama_process: ResMut<OllamaProcess>,
    mut agent_id_generator: ResMut<AgentIdGenerator>,
    mut arena_state: ResMut<ArenaState>,
    mut agent_creator: ResMut<AgentCreator>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // Обработка горячих клавиш
    if keyboard_input.just_pressed(KeyCode::F1) {
        time_simulation.window_open = !time_simulation.window_open;
    }
    if keyboard_input.just_pressed(KeyCode::F2) {
        agent_chat.window_open = !agent_chat.window_open;
    }
    if keyboard_input.just_pressed(KeyCode::F3) {
        ollama_process.window_open = !ollama_process.window_open;
    }
    
    // Отображаем окна
    show_time_simulation_window(&mut contexts, &mut time_simulation);
    show_agent_chat_window(&mut contexts, &mut agent_chat);
    show_ollama_process_window(&mut contexts, &mut ollama_process);
    show_agent_creator_window(&mut contexts, &mut agent_creator, &mut commands, &mut meshes, &mut materials, &mut arena_state, &mut agent_id_generator);
    
    // Плавающая панель с кнопками быстрого доступа
    egui::Window::new("🎀 Quick Actions")
        .default_size([200.0, 150.0])
        .resizable(true)
        .collapsible(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                if ui.button("⏰ Time Simulation (F1)").clicked() {
                    time_simulation.window_open = !time_simulation.window_open;
                }
                
                if ui.button("💬 Agent Chat (F2)").clicked() {
                    agent_chat.window_open = !agent_chat.window_open;
                }
                
                if ui.button("🤖 Ollama Server (F3)").clicked() {
                    ollama_process.window_open = !ollama_process.window_open;
                }
                
                ui.separator();
                
                if ui.button("➕ Add Random Agent").clicked() {
                    spawn_random_agent(&mut commands, &mut meshes, &mut materials, &mut arena_state, &mut agent_id_generator);
                }
                
                if ui.button("🎛️ Create Custom Agent").clicked() {
                    agent_creator.window_open = true;
                }
            });
        });
}

// Arena Theme Management System
fn arena_theme_system(
    mut contexts: EguiContexts,
    mut arena_theme: ResMut<ArenaTheme>,
    mut log_system: ResMut<LogSystem>,
) {
    egui::SidePanel::left("theme_panel").resizable(true).show(contexts.ctx_mut(), |ui| {
        ui.heading("🎨 Arena Theme");
        ui.label(format!("Current: {}", arena_theme.name));
        let available_themes = ArenaTheme::get_available_themes();
        egui::ComboBox::from_label("Select Theme")
            .selected_text(&arena_theme.name)
            .show_ui(ui, |ui| {
                for theme in &available_themes {
                    if ui.selectable_value(&mut arena_theme.name, theme.name.clone(), &theme.name).clicked() {
                        // Update the arena theme when selection changes
                        *arena_theme = theme.clone();
                        log_system.add_log(format!("🎨 Changed arena theme to: {}", theme.name));
                    }
                }
            });
    });
}

// Agent Creator Window
fn show_agent_creator_window(
    contexts: &mut EguiContexts,
    agent_creator: &mut AgentCreator,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    arena_state: &mut ResMut<ArenaState>,
    agent_id_generator: &mut ResMut<AgentIdGenerator>,
) {
    if agent_creator.window_open {
        egui::Window::new("🎛️ Create Custom Agent")
            .default_size([400.0, 600.0])
            .resizable(true)
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Agent Configuration");
                ui.separator();
                
                // Basic Info
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut agent_creator.agent_name);
                });
                
                // Team Selection
                ui.horizontal(|ui| {
                    ui.label("Team:");
                    egui::ComboBox::from_label("")
                        .selected_text(&agent_creator.selected_team)
                        .show_ui(ui, |ui| {
                            for team in AgentCreator::get_available_teams() {
                                ui.selectable_value(&mut agent_creator.selected_team, team.to_string(), team);
                            }
                        });
                });
                
                // Role Selection
                ui.horizontal(|ui| {
                    ui.label("Role:");
                    egui::ComboBox::from_label("")
                        .selected_text(&agent_creator.selected_role)
                        .show_ui(ui, |ui| {
                            for role in AgentCreator::get_available_roles() {
                                ui.selectable_value(&mut agent_creator.selected_role, role.to_string(), role);
                            }
                        });
                });
                
                ui.separator();
                
                // Stats
                ui.label("Stats:");
                ui.horizontal(|ui| {
                    ui.label("Health:");
                    ui.add(egui::Slider::new(&mut agent_creator.health, 1.0..=200.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Energy:");
                    ui.add(egui::Slider::new(&mut agent_creator.energy, 1.0..=200.0));
                });
                
                ui.separator();
                
                // Position
                ui.label("Spawn Position:");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(egui::DragValue::new(&mut agent_creator.spawn_position.x).speed(0.1));
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut agent_creator.spawn_position.y).speed(0.1));
                    ui.label("Z:");
                    ui.add(egui::DragValue::new(&mut agent_creator.spawn_position.z).speed(0.1));
                });
                
                ui.separator();
                
                // AI Settings
                ui.checkbox(&mut agent_creator.ai_enabled, "AI Enabled");
                
                ui.label("Custom AI Prompt:");
                ui.text_edit_multiline(&mut agent_creator.custom_prompt);
                
                ui.separator();
                
                // Buttons
                ui.horizontal(|ui| {
                    if ui.button("✨ Create Agent").clicked() {
                        create_custom_agent(commands, meshes, materials, arena_state, agent_id_generator, agent_creator);
                        agent_creator.window_open = false;
                    }
                    
                    if ui.button("🔄 Reset").clicked() {
                        *agent_creator = AgentCreator::new();
                        agent_creator.window_open = true;
                    }
                    
                    if ui.button("❌ Cancel").clicked() {
                        agent_creator.window_open = false;
                    }
                });
            });
    }
}

// Function to create a custom agent
fn create_custom_agent(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    arena_state: &mut ResMut<ArenaState>,
    agent_id_generator: &mut ResMut<AgentIdGenerator>,
    agent_creator: &AgentCreator,
) {
    let id = agent_id_generator.generate_id(&agent_creator.selected_team, &agent_creator.selected_role);
    
    let agent = Agent {
        id: id.clone(),
        name: agent_creator.agent_name.clone(),
        health: agent_creator.health,
        energy: agent_creator.energy,
        team: agent_creator.selected_team.clone(),
        status: "waiting_for_command".to_string(),
        ai_enabled: agent_creator.ai_enabled,
        decision_cooldown: Timer::from_seconds(2.0, TimerMode::Repeating),
    };
    
    // Enhanced color system based on team and role
    let base_color = match agent_creator.selected_team.as_str() {
        "red" => Color::rgb(0.8, 0.2, 0.2),
        "blue" => Color::rgb(0.2, 0.2, 0.8),
        "green" => Color::rgb(0.2, 0.8, 0.2),
        "yellow" => Color::rgb(0.8, 0.8, 0.2),
        "purple" => Color::rgb(0.8, 0.2, 0.8),
        _ => Color::rgb(0.5, 0.5, 0.5),
    };
    
    // Role-specific color modifications
    let color = match agent_creator.selected_role.as_str() {
        "warrior" => Color::rgb(base_color.r() * 0.9, base_color.g() * 0.9, base_color.b() * 0.9),
        "mage" => Color::rgb(base_color.r() * 1.2, base_color.g() * 1.2, base_color.b() * 1.2),
        "archer" => Color::rgb(base_color.r() * 0.7, base_color.g() * 1.1, base_color.b() * 0.7),
        "tank" => Color::rgb(base_color.r() * 0.6, base_color.g() * 0.6, base_color.b() * 0.6),
        "scout" => Color::rgb(base_color.r() * 1.1, base_color.g() * 1.1, base_color.b() * 1.1),
        _ => base_color,
    };
    
    // Create 3D model of agent with role-specific appearance
    let entity = create_diverse_agent(commands, meshes, materials, color, agent_creator.spawn_position, &agent_creator.selected_role);
    
    // Add components to main entity
    commands.entity(entity).insert((
        agent.clone(),
        AgentVisual,
        SelectionOutline {
            selected: false,
            hovered: false,
        },
        AIBrain {
            model: "llama3.2:latest".to_string(),
            context: agent_creator.custom_prompt.clone(),
            last_action: "none".to_string(),
            thinking: false,
        },
        Movement {
            velocity: Vec3::ZERO,
            target_position: None,
            speed: 3.0,
        },
        Combat {
            attack_damage: 25.0,
            attack_range: 2.0,
            defense: 10.0,
            last_attack_time: 0.0,
            attack_cooldown: 1.0,
        },
    ));
    
    // Add to arena state
    arena_state.agents.insert(id.clone(), agent);
    arena_state.total_agents += 1;
    
    println!("✨ Created custom agent: {} at {:?}", agent_creator.agent_name, agent_creator.spawn_position);
}

// Ollama Server Monitor Window
fn show_ollama_process_window(contexts: &mut EguiContexts, ollama_process: &mut OllamaProcess) {
    if ollama_process.window_open {
        egui::Window::new("🤖 Ollama Server Monitor")
            .default_size([600.0, 400.0])
            .resizable(true)
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Ollama Server Status");
                ui.separator();
                
                // Статус сервера
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    let status_color = if ollama_process.process.is_some() {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::RED
                    };
                    ui.colored_label(status_color, &ollama_process.status);
                });
                
                if let Some(pid) = ollama_process.process_id {
                    ui.horizontal(|ui| {
                        ui.label("Process ID:");
                        ui.label(pid.to_string());
                    });
                }
                
                if let Some(start_time) = ollama_process.start_time {
                    ui.horizontal(|ui| {
                        ui.label("Uptime:");
                        ui.label(format!("{}s", start_time.elapsed().as_secs()));
                    });
                }
                
                ui.separator();
                
                // Управление сервером
                ui.horizontal(|ui| {
                    if ollama_process.process.is_none() {
                        if ui.button("🚀 Start Server").clicked() {
                            // Запуск будет обработан в системе
                            ollama_process.auto_start = true;
                        }
                    } else {
                        if ui.button("⏹️ Stop Server").clicked() {
                            if let Some(ref mut process) = ollama_process.process {
                                let _ = process.kill();
                                ollama_process.process = None;
                                ollama_process.process_id = None;
                                ollama_process.status = "Stopped by user".to_string();
                            }
                        }
                    }
                    
                    if ui.button("🔄 Restart").clicked() {
                        if let Some(ref mut process) = ollama_process.process {
                            let _ = process.kill();
                            ollama_process.process = None;
                            ollama_process.process_id = None;
                        }
                        ollama_process.restart_attempts = 0;
                        ollama_process.auto_start = true;
                    }
                    
                    ui.checkbox(&mut ollama_process.auto_start, "Auto-start");
                });
                
                ui.separator();
                
                // Логи сервера
                ui.heading("Server Logs");
                
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if let Ok(logs) = ollama_process.logs.lock() {
                            for log in logs.iter().rev().take(100) {
                                ui.horizontal(|ui| {
                                    if log.contains("[STDERR]") {
                                        ui.colored_label(egui::Color32::RED, log);
                                    } else if log.contains("[STDOUT]") {
                                        ui.colored_label(egui::Color32::GREEN, log);
                                    } else {
                                        ui.label(log);
                                    }
                                });
                            }
                            
                            if logs.is_empty() {
                                ui.label("No logs yet...");
                            }
                        }
                    });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("🗑️ Clear Logs").clicked() {
                        if let Ok(mut logs) = ollama_process.logs.lock() {
                            logs.clear();
                        }
                    }
                    
                    if ui.button("❌ Close").clicked() {
                        ollama_process.window_open = false;
                    }
                });
            });
    }
}

// Система симуляции времени
fn time_simulation_system(
    mut time_simulation: ResMut<TimeSimulation>,
    time: Res<Time>,
    mut log_system: ResMut<LogSystem>,
) {
    if !time_simulation.paused {
        time_simulation.current_time += time.delta_seconds() * time_simulation.time_scale;
        
        // Обновляем симулируемое время
        let total_minutes = time_simulation.current_time / 60.0;
        time_simulation.simulated_minutes = (total_minutes % 60.0) as u32;
        time_simulation.simulated_hours = ((total_minutes / 60.0) % 24.0) as u32;
        time_simulation.simulated_days = (total_minutes / (60.0 * 24.0)) as u32;
        
        // Логируем важные события времени
        let current_minute = time_simulation.simulated_minutes;
        let current_hour = time_simulation.simulated_hours;
        let current_day = time_simulation.simulated_days;
        
        // Каждые 10 минут симулируемого времени (но не чаще чем раз в 10 секунд)
        static mut LAST_LOG_TIME: f32 = 0.0;
        let current_real_time = time.elapsed_seconds();
        
        if current_minute % 10 == 0 && current_minute != 0 {
            unsafe {
                if current_real_time - LAST_LOG_TIME > 10.0 {
                    log_system.add_log(format!("⏰ Simulated time: Day {}, {:02}:{:02}", current_day, current_hour, current_minute));
                    LAST_LOG_TIME = current_real_time;
                }
            }
        }
        
        // Обновляем активности агентов
        let lifespans = time_simulation.agent_lifespans.clone();
        for (agent_id, activities) in time_simulation.agent_activities.iter_mut() {
            let lifespan = lifespans.get(agent_id).unwrap_or(&7.0);
            let life_progress = (current_day as f32) / lifespan;
            
            // Добавляем события жизни агента
            if life_progress > 0.1 && !activities.contains(&"Learning".to_string()) {
                activities.push("Learning".to_string());
            }
            if life_progress > 0.3 && !activities.contains(&"Experienced".to_string()) {
                activities.push("Experienced".to_string());
            }
            if life_progress > 0.7 && !activities.contains(&"Veteran".to_string()) {
                activities.push("Veteran".to_string());
            }
            if life_progress >= 1.0 && !activities.contains(&"Lifecycle Complete".to_string()) {
                activities.push("Lifecycle Complete".to_string());
                log_system.add_log(format!("🎓 Agent {} completed lifecycle!", agent_id));
            }
        }
    }
}

// Система чата между агентами
fn agent_chat_system(
    mut agent_chat: ResMut<AgentChat>,
    time: Res<Time>,
    agents_query: Query<&Agent, With<AgentVisual>>,
    mut log_system: ResMut<LogSystem>,
    ollama_connection: Res<OllamaConnection>,
) {
    if !agent_chat.auto_chat_enabled {
        return;
    }
    
    let current_time = time.elapsed_seconds();
    if current_time - agent_chat.last_chat_time < agent_chat.chat_frequency.max(5.0) { // Минимум 5 секунд между сообщениями
        return;
    }
    
    agent_chat.last_chat_time = current_time;
    
    // Получаем всех агентов
    let agents: Vec<&Agent> = agents_query.iter().collect();
    if agents.len() < 2 {
        return;
    }
    
    // Выбираем случайного отправителя и получателя
    let sender_idx = rand::random::<usize>() % agents.len();
    let mut receiver_idx = rand::random::<usize>() % agents.len();
    while receiver_idx == sender_idx {
        receiver_idx = rand::random::<usize>() % agents.len();
    }
    
    let sender = agents[sender_idx];
    let receiver = agents[receiver_idx];
    
    // Generate message based on context and AI if available
    let message = if ollama_connection.connected {
        // Use Ollama to generate contextual messages
        let context = match (sender.team.as_str(), receiver.team.as_str()) {
            (sender_team, receiver_team) if sender_team == receiver_team => {
                format!("You are {} from team {}. You're talking to your ally {} from the same team. Generate a short (max 8 words) tactical message about coordinating in battle.", 
                    sender.name, sender.team, receiver.name)
            }
            (sender_team, receiver_team) => {
                format!("You are {} from team {}. You're talking to enemy {} from team {}. Generate a short (max 8 words) battle taunt or challenge.", 
                    sender.name, sender.team, receiver.name, receiver.team)
            }
        };
        
        // Enhanced templates when Ollama is connected
        let message_templates = if sender.team == receiver.team {
            // Ally messages - more tactical and coordinated
            vec![
                "Ready for battle, ally?",
                "Let's coordinate our attack!",
                "Watch my back!",
                "Need backup here!",
                "Enemy spotted nearby!",
                "Cover me, I'm moving!",
                "Let's flank them together!",
                "Group up for assault!",
                "Form defensive position!",
                "Execute pincer movement!",
            ]
        } else {
            // Enemy messages - more aggressive and taunting
            vec![
                "You're going down!",
                "Prepare for defeat!",
                "This ends now!",
                "You won't escape!",
                "Face me in combat!",
                "Your time is up!",
                "I'll crush you!",
                "Victory will be mine!",
                "Surrender now!",
                "Meet your match!",
            ]
        };
        message_templates[rand::random::<usize>() % message_templates.len()].to_string()
    } else {
        // Use predefined templates when Ollama is not connected
        let message_templates = if sender.team == receiver.team {
            // Messages between allies
            vec![
                "Ready for battle, ally?",
                "Let's coordinate our attack!",
                "Watch my back!",
                "Need backup here!",
                "Enemy spotted nearby!",
                "Cover me, I'm moving!",
                "Let's flank them together!",
                "Group up for assault!",
            ]
        } else {
            // Messages between enemies
            vec![
                "You're going down!",
                "Prepare for defeat!",
                "This ends now!",
                "You won't escape!",
                "Face me in combat!",
                "Your time is up!",
                "I'll crush you!",
                "Victory will be mine!",
            ]
        };
        message_templates[rand::random::<usize>() % message_templates.len()].to_string()
    };
    
    let chat_message = ChatMessage {
        sender_id: sender.id.clone(),
        sender_name: sender.name.clone(),
        receiver_id: receiver.id.clone(),
        receiver_name: receiver.name.clone(),
        message: message.clone(),
        timestamp: current_time,
        message_type: "greeting".to_string(),
    };
    
    agent_chat.chat_history.push(chat_message);
    
    // Ограничиваем историю чата
    if agent_chat.chat_history.len() > 100 {
        agent_chat.chat_history.remove(0);
    }
    
    log_system.add_log(format!("💬 {}: {}", sender.name, message));
}

// Окно симуляции времени
fn show_time_simulation_window(contexts: &mut EguiContexts, time_simulation: &mut TimeSimulation) {
    if time_simulation.window_open {
        egui::Window::new("⏰ Time Simulation")
            .default_size([400.0, 350.0])
            .resizable(true)
            .collapsible(true)
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Life Simulation System");
                ui.separator();
                
                // Текущее время
                ui.label(format!("Current Simulated Time: Day {}, {:02}:{:02}", 
                    time_simulation.simulated_days, 
                    time_simulation.simulated_hours, 
                    time_simulation.simulated_minutes));
                
                ui.separator();
                
                // Управление временем
                ui.horizontal(|ui| {
                    if ui.button(if time_simulation.paused { "▶️ Resume" } else { "⏸️ Pause" }).clicked() {
                        time_simulation.paused = !time_simulation.paused;
                    }
                    
                    if ui.button("🔄 Reset").clicked() {
                        time_simulation.current_time = 0.0;
                        time_simulation.simulated_days = 0;
                        time_simulation.simulated_hours = 0;
                        time_simulation.simulated_minutes = 0;
                    }
                });
                
                ui.separator();
                
                // Скорость времени
                ui.horizontal(|ui| {
                    ui.label("Time Scale:");
                    let scale_val = time_simulation.time_scale;
                    ui.add(egui::Slider::new(&mut time_simulation.time_scale, 1.0..=3600.0)
                        .logarithmic(true)
                        .text(format!("{:.1}x", scale_val)));
                });
                
                ui.label("Quick presets:");
                ui.horizontal(|ui| {
                    if ui.button("⏰ Real time").clicked() {
                        time_simulation.time_scale = 1.0;
                    }
                    if ui.button("⚡ 1 min = 1 sec").clicked() {
                        time_simulation.time_scale = 60.0;
                    }
                    if ui.button("🚀 1 hour = 1 sec").clicked() {
                        time_simulation.time_scale = 3600.0;
                    }
                });
                
                ui.separator();
                
                // Жизненные циклы агентов
                ui.label("Agent Lifespans:");
                for (agent_id, lifespan) in time_simulation.agent_lifespans.iter_mut() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: ", agent_id));
                        let lifespan_val = *lifespan;
                        ui.add(egui::Slider::new(lifespan, 1.0..=30.0)
                            .text(format!("{:.1} days", lifespan_val)));
                    });
                }
                
                ui.separator();
                
                // Активности агентов
                ui.label("Agent Activities:");
                for (agent_id, activities) in time_simulation.agent_activities.iter() {
                    ui.collapsing(agent_id, |ui| {
                        for activity in activities.iter() {
                            ui.label(format!("• {}", activity));
                        }
                    });
                }
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("❌ Close").clicked() {
                        time_simulation.window_open = false;
                    }
                });
            });
    }
}

// Окно чата между агентами
fn show_agent_chat_window(contexts: &mut EguiContexts, agent_chat: &mut AgentChat) {
    if agent_chat.window_open {
        egui::Window::new("💬 Agent Chat")
            .default_size([500.0, 400.0])
            .resizable(true)
            .collapsible(true)
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Agent Communication System");
                ui.separator();
                
                // Настройки чата
                ui.horizontal(|ui| {
                    ui.checkbox(&mut agent_chat.auto_chat_enabled, "Auto Chat");
                    ui.label("Frequency:");
                    let freq_val = agent_chat.chat_frequency;
                    ui.add(egui::Slider::new(&mut agent_chat.chat_frequency, 1.0..=60.0)
                        .text(format!("{:.1}s", freq_val)));
                });
                
                ui.separator();
                
                // История чата
                ui.label("Chat History:");
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .show(ui, |ui| {
                        for message in agent_chat.chat_history.iter().rev() {
                            ui.horizontal(|ui| {
                                ui.label(format!("[{:.1}s]", message.timestamp));
                                ui.label(format!("{}:", message.sender_name));
                                ui.label(&message.message);
                                ui.label(format!("→ {}", message.receiver_name));
                            });
                        }
                        
                        if agent_chat.chat_history.is_empty() {
                            ui.label("No messages yet...");
                        }
                    });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("🗑️ Clear History").clicked() {
                        agent_chat.chat_history.clear();
                    }
                    
                    if ui.button("❌ Close").clicked() {
                        agent_chat.window_open = false;
                    }
                });
            });
    }
}

/// Set up the 3D arena environment with themed floor, walls, lighting, and camera
/// Uses the current ArenaTheme resource to determine visual appearance
/// Creates a 20x20 unit arena with 2-unit high walls and dynamic lighting
fn setup_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    arena_theme: Res<ArenaTheme>,
) {
    // Create arena floor with theme
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(20.0).into()),
        material: materials.add(StandardMaterial {
            base_color: arena_theme.floor_color,
            metallic: arena_theme.floor_metallic,
            perceptual_roughness: arena_theme.floor_roughness,
            ..default()
        }),
        ..default()
    });

    // Create arena walls with theme
    for (pos, scale) in [
        (Vec3::new(0.0, 1.0, 10.0), Vec3::new(20.0, 2.0, 0.2)),   // North
        (Vec3::new(0.0, 1.0, -10.0), Vec3::new(20.0, 2.0, 0.2)),  // South
        (Vec3::new(10.0, 1.0, 0.0), Vec3::new(0.2, 2.0, 20.0)),   // East
        (Vec3::new(-10.0, 1.0, 0.0), Vec3::new(0.2, 2.0, 20.0)),  // West
    ] {
        commands.spawn(PbrBundle {
            mesh: meshes.add(shape::Box::new(scale.x, scale.y, scale.z).into()),
            material: materials.add(StandardMaterial {
                base_color: arena_theme.wall_color,
                metallic: arena_theme.wall_metallic,
                perceptual_roughness: arena_theme.wall_roughness,
                ..default()
            }),
            transform: Transform::from_translation(pos),
            ..default()
        });
    }

    // Setup lighting with theme
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: arena_theme.light_intensity * 10000.0,
            color: arena_theme.light_color,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 8.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });

    // Create camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 15.0, 20.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    println!("🏟️ Arena setup complete with {} theme!", arena_theme.name);
}

// Компонент для частей тела агента
#[derive(Component)]
pub struct AgentBodyPart {
    pub part_type: String,
    pub relative_position: Vec3,
    pub animation_offset: Vec3,
}

// Функция для создания 3D человечка из примитивов (исправлена)
fn create_humanoid_agent(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    color: Color,
    position: Vec3,
) -> Entity {
    create_diverse_agent(commands, meshes, materials, color, position, "warrior")
}

/// Create a diverse 3D humanoid agent with role-specific visual characteristics
/// Each role (warrior, mage, archer, tank, scout) has unique dimensions, materials, and appearance
/// Returns the main entity ID for the created agent
fn create_diverse_agent(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    color: Color,
    position: Vec3,
    role: &str,
) -> Entity {
    // Role-specific material properties
    let (metallic, roughness, emissive) = match role {
        "warrior" => (0.8, 0.2, Color::BLACK), // Metallic armor
        "mage" => (0.0, 0.9, Color::rgb(0.1, 0.1, 0.3)), // Magical glow
        "archer" => (0.2, 0.7, Color::BLACK), // Leather-like
        "tank" => (0.9, 0.1, Color::BLACK), // Heavy metal
        "scout" => (0.1, 0.9, Color::BLACK), // Matte finish
        _ => (0.1, 0.8, Color::BLACK),
    };
    
    let material = materials.add(StandardMaterial {
        base_color: color,
        metallic,
        perceptual_roughness: roughness,
        emissive,
        ..default()
    });
    
    // Role-specific torso dimensions
    let (torso_width, torso_height, torso_depth) = match role {
        "warrior" => (0.5, 0.7, 0.3), // Broader, taller
        "mage" => (0.3, 0.8, 0.2), // Thinner, taller
        "archer" => (0.35, 0.6, 0.2), // Lean build
        "tank" => (0.6, 0.8, 0.4), // Bulky
        "scout" => (0.3, 0.5, 0.2), // Small and agile
        _ => (0.4, 0.6, 0.2),
    };
    
    // Create main entity (torso) - this will be the root node
    let main_entity = commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Box::new(torso_width, torso_height, torso_depth).into()),
            material: material.clone(),
            transform: Transform::from_translation(position),
            ..default()
        },
    )).id();
    
    // Role-specific head properties
    let (head_shape, head_size, head_y_offset) = match role {
        "warrior" => ("helmet", 0.18, torso_height * 0.6), // Helmet-like
        "mage" => ("pointed", 0.16, torso_height * 0.7), // Pointed hat effect
        "archer" => ("hood", 0.15, torso_height * 0.6), // Hood-like
        "tank" => ("heavy", 0.20, torso_height * 0.5), // Heavy helmet
        "scout" => ("light", 0.12, torso_height * 0.6), // Light helmet
        _ => ("normal", 0.15, torso_height * 0.6),
    };
    
    // Create head with role-specific appearance
    let head_mesh = match head_shape {
        "helmet" => meshes.add(shape::Box::new(head_size * 1.2, head_size, head_size * 1.2).into()),
        "pointed" => meshes.add(shape::Box::new(head_size, head_size * 1.4, head_size).into()),
        "hood" => meshes.add(shape::UVSphere { radius: head_size * 0.9, sectors: 12, stacks: 12 }.into()),
        "heavy" => meshes.add(shape::Box::new(head_size * 1.5, head_size * 1.2, head_size * 1.5).into()),
        "light" => meshes.add(shape::UVSphere { radius: head_size * 0.8, sectors: 8, stacks: 8 }.into()),
        _ => meshes.add(shape::UVSphere { radius: head_size, sectors: 16, stacks: 16 }.into()),
    };
    
    commands.spawn((
        PbrBundle {
            mesh: head_mesh,
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, head_y_offset, 0.0)),
            ..default()
        },
        AgentBodyPart {
            part_type: "head".to_string(),
            relative_position: Vec3::new(0.0, head_y_offset, 0.0),
            animation_offset: Vec3::ZERO,
        },
    )).set_parent(main_entity);
    
    // Левая рука
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Box::new(0.1, 0.5, 0.1).into()),
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(-0.3, 0.1, 0.0)),
            ..default()
        },
        AgentBodyPart {
            part_type: "left_arm".to_string(),
            relative_position: Vec3::new(-0.3, 0.1, 0.0),
            animation_offset: Vec3::ZERO,
        },
    )).set_parent(main_entity);
    
    // Правая рука
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Box::new(0.1, 0.5, 0.1).into()),
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(0.3, 0.1, 0.0)),
            ..default()
        },
        AgentBodyPart {
            part_type: "right_arm".to_string(),
            relative_position: Vec3::new(0.3, 0.1, 0.0),
            animation_offset: Vec3::ZERO,
        },
    )).set_parent(main_entity);
    
    // Левая нога
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Box::new(0.15, 0.6, 0.15).into()),
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(-0.1, -0.6, 0.0)),
            ..default()
        },
        AgentBodyPart {
            part_type: "left_leg".to_string(),
            relative_position: Vec3::new(-0.1, -0.6, 0.0),
            animation_offset: Vec3::ZERO,
        },
    )).set_parent(main_entity);
    
    // Правая нога
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Box::new(0.15, 0.6, 0.15).into()),
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(0.1, -0.6, 0.0)),
            ..default()
        },
        AgentBodyPart {
            part_type: "right_leg".to_string(),
            relative_position: Vec3::new(0.1, -0.6, 0.0),
            animation_offset: Vec3::ZERO,
        },
    )).set_parent(main_entity);
    
    main_entity
}

/// Spawn a random agent with automatically generated team, role, and position
/// Uses the AgentIdGenerator to create unique IDs and select random attributes
/// Adds the agent to both the ECS world and the ArenaState resource
fn spawn_random_agent(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    arena_state: &mut ResMut<ArenaState>,
    agent_id_generator: &mut ResMut<AgentIdGenerator>,
) {
    let (id, name, team, position, role) = agent_id_generator.generate_random_agent();
    
    let agent = Agent {
        id: id.clone(),
        name: name.clone(),
        health: 100.0,
        energy: 100.0,
        team: team.clone(),
        status: "waiting_for_command".to_string(),
        ai_enabled: true,
        decision_cooldown: Timer::from_seconds(2.0, TimerMode::Repeating),
    };
    
    // Enhanced color system based on team and role
    let base_color = match team.as_str() {
        "red" => Color::rgb(0.8, 0.2, 0.2),
        "blue" => Color::rgb(0.2, 0.2, 0.8),
        "green" => Color::rgb(0.2, 0.8, 0.2),
        "yellow" => Color::rgb(0.8, 0.8, 0.2),
        "purple" => Color::rgb(0.8, 0.2, 0.8),
        _ => Color::rgb(0.5, 0.5, 0.5),
    };
    
    // Role-specific color modifications
    let color = match role.as_str() {
        "warrior" => Color::rgb(base_color.r() * 0.9, base_color.g() * 0.9, base_color.b() * 0.9), // Darker
        "mage" => Color::rgb(base_color.r() * 1.2, base_color.g() * 1.2, base_color.b() * 1.2), // Brighter
        "archer" => Color::rgb(base_color.r() * 0.7, base_color.g() * 1.1, base_color.b() * 0.7), // Greenish tint
        "tank" => Color::rgb(base_color.r() * 0.6, base_color.g() * 0.6, base_color.b() * 0.6), // Much darker
        "scout" => Color::rgb(base_color.r() * 1.1, base_color.g() * 1.1, base_color.b() * 1.1), // Slightly brighter
        _ => base_color,
    };
    
    // Create 3D model of agent with role-specific appearance
    let entity = create_diverse_agent(commands, meshes, materials, color, position, &role);
    
    // Добавляем компоненты к главному entity
    commands.entity(entity).insert((
        agent.clone(),
        AgentVisual,
        SelectionOutline {
            selected: false,
            hovered: false,
        },
        AIBrain {
            model: "llama3.2:latest".to_string(),
            context: "You are an AI agent in a 3D arena".to_string(),
            last_action: "none".to_string(),
            thinking: false,
        },
        Movement {
            velocity: Vec3::ZERO,
            target_position: None,
            speed: 3.0,
        },
        Combat {
            attack_damage: 25.0,
            attack_range: 2.0,
            defense: 10.0,
            last_attack_time: 0.0,
            attack_cooldown: 1.0,
        },
    ));
    
    arena_state.agents.insert(id.clone(), agent);
    arena_state.total_agents = arena_state.agents.len() as u32;
    
    println!("✨ Spawned new agent: {} at {:?}", name, position);
}

// Система автоматического генератора ID и создания агентов
#[derive(Resource)]
pub struct AgentIdGenerator {
    next_id: u32,
    used_ids: std::collections::HashSet<String>,
}

impl Default for AgentIdGenerator {
    fn default() -> Self {
        let mut used_ids = std::collections::HashSet::new();
        used_ids.insert("red_gladiator".to_string());
        used_ids.insert("blue_warrior".to_string());
        used_ids.insert("red_scout".to_string());
        
        Self {
            next_id: 1,
            used_ids,
        }
    }
}

impl AgentIdGenerator {
    pub fn generate_id(&mut self, team: &str, role: &str) -> String {
        loop {
            let id = format!("{}_{}_{}", team, role, self.next_id);
            self.next_id += 1;
            
            if !self.used_ids.contains(&id) {
                self.used_ids.insert(id.clone());
                return id;
            }
        }
    }
    
    pub fn generate_random_agent(&mut self) -> (String, String, String, Vec3, String) {
        // Ограничиваем команды только red и blue для боевых действий
        let teams = vec!["red", "blue"];
        let roles = vec!["warrior", "scout", "mage", "archer", "tank"];
        
        let team = teams[rand::random::<usize>() % teams.len()];
        let role = roles[rand::random::<usize>() % roles.len()];
        let id = self.generate_id(team, role);
        
        let emoji = match team {
            "red" => "🔴",
            "blue" => "🔵",
            "green" => "🟢",
            "yellow" => "🟡",
            "purple" => "🟣",
            _ => "⚪",
        };
        
        let role_name = match role {
            "warrior" => "Warrior",
            "scout" => "Scout",
            "mage" => "Mage",
            "archer" => "Archer",
            "tank" => "Tank",
            _ => "Fighter",
        };
        
        let name = format!("{} {} {}", emoji, role_name, self.next_id - 1);
        
        // Случайная позиция в арене
        let position = Vec3::new(
            (rand::random::<f32>() - 0.5) * 16.0,
            0.5,
            (rand::random::<f32>() - 0.5) * 12.0,
        );
        
        (id, name, team.to_string(), position, role.to_string())
    }
}

/// System to create the initial set of real agents when the arena starts
/// Creates predefined agents with specific roles and positions
/// Only runs once when the arena is empty
fn setup_real_agents(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut arena_state: ResMut<ArenaState>,
) {
    // Create real agents only once when the arena is empty
    if arena_state.agents.is_empty() {
        println!("🤖 Creating real agents...");
        
        let agent_data = vec![
            ("red_gladiator", "🔴 Gladiator Alpha", "red", Vec3::new(-5.0, 0.5, 0.0), "warrior"),
            ("blue_warrior", "🔵 Warrior Beta", "blue", Vec3::new(5.0, 0.5, 0.0), "warrior"),
            ("red_scout", "🔴 Scout Gamma", "red", Vec3::new(0.0, 0.5, 5.0), "scout"),
        ];
        
        for (id, name, team, position, role) in agent_data {
            let agent = Agent {
                id: id.to_string(),
                name: name.to_string(),
                health: 100.0,
                energy: 100.0,
                team: team.to_string(),
                status: "waiting_for_command".to_string(),
                ai_enabled: true,
                decision_cooldown: Timer::from_seconds(2.0, TimerMode::Repeating),
            };
            
            // Enhanced color system based on team and role
            let base_color = match team {
                "red" => Color::rgb(0.8, 0.2, 0.2),
                "blue" => Color::rgb(0.2, 0.2, 0.8),
                _ => Color::rgb(0.5, 0.5, 0.5),
            };
            
            // Role-specific color modifications
            let color = match role {
                "warrior" => Color::rgb(base_color.r() * 0.9, base_color.g() * 0.9, base_color.b() * 0.9),
                "scout" => Color::rgb(base_color.r() * 1.1, base_color.g() * 1.1, base_color.b() * 1.1),
                _ => base_color,
            };
            
            // Create 3D model of agent with role-specific appearance
            let entity = create_diverse_agent(&mut commands, &mut meshes, &mut materials, color, position, role);
            
            // Добавляем компоненты к главному entity
            commands.entity(entity).insert((
                agent.clone(),
                AgentVisual,
                SelectionOutline {
                    selected: false,
                    hovered: false,
                },
                AIBrain {
                    model: "llama3.2:latest".to_string(),
                    context: "You are an AI agent in a 3D arena".to_string(),
                    last_action: "none".to_string(),
                    thinking: false,
                },
                Movement {
                    velocity: Vec3::ZERO,
                    target_position: None,
                    speed: 3.0,
                },
                Combat {
                    attack_damage: 25.0,
                    attack_range: 2.0,
                    defense: 10.0,
                    last_attack_time: 0.0,
                    attack_cooldown: 1.0, // 1 секунда кулдаун
                },
            ));
            
            arena_state.agents.insert(id.to_string(), agent);
            println!("✨ Created real agent: {} at {:?}", name, position);
        }
        
        arena_state.total_agents = arena_state.agents.len() as u32;
        println!("🎯 Total agents created: {}", arena_state.total_agents);
    }
}

fn demo_mode_system(
    time: Res<Time>,
    mut demo_mode: ResMut<DemoMode>,
    mut arena_state: ResMut<ArenaState>,
) {
    if !demo_mode.enabled {
        return;
    }

    demo_mode.timer.tick(time.delta());
    
    if demo_mode.timer.just_finished() {
        arena_state.connection_status = "🎮 Demo Mode".to_string();
        arena_state.match_time += 1.0;
        
        // Создаем mock агентов с AI компонентами
        let mock_agents = vec![
            Agent {
                id: "agent_1".to_string(),
                name: "Gladiator Alpha".to_string(),
                health: 85.0,
                energy: 90.0,
                team: "red".to_string(),
                status: "fighting".to_string(),
                ai_enabled: true,
                decision_cooldown: Timer::from_seconds(2.0, TimerMode::Repeating),
            },
            Agent {
                id: "agent_2".to_string(),
                name: "Warrior Beta".to_string(),
                health: 65.0,
                energy: 75.0,
                team: "blue".to_string(),
                status: "defending".to_string(),
                ai_enabled: true,
                decision_cooldown: Timer::from_seconds(2.5, TimerMode::Repeating),
            },
            Agent {
                id: "agent_3".to_string(),
                name: "Scout Gamma".to_string(),
                health: 45.0,
                energy: 95.0,
                team: "red".to_string(),
                status: "moving".to_string(),
                ai_enabled: true,
                decision_cooldown: Timer::from_seconds(1.5, TimerMode::Repeating),
            },
        ];

        arena_state.agents.clear();
        for mut agent in mock_agents {
            // Добавляем случайные изменения здоровья для анимации
            agent.health += (time.elapsed_seconds().sin() * 5.0).clamp(-2.0, 2.0);
            agent.health = agent.health.clamp(10.0, 100.0);
            
            arena_state.agents.insert(agent.id.clone(), agent);
        }
        
        arena_state.total_agents = arena_state.agents.len() as u32;
    }
}

fn update_agents_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    arena_state: Res<ArenaState>,
    time: Res<Time>,
    query: Query<(Entity, &Agent), With<AgentVisual>>,
) {
    // Удаляем старых агентов
    for (entity, agent) in query.iter() {
        if !arena_state.agents.contains_key(&agent.id) {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Создаем/обновляем агентов
    for (agent_id, agent) in &arena_state.agents {
        let existing = query.iter().find(|(_, a)| &a.id == agent_id);
        
        if existing.is_none() {
            // Создаем нового агента
            let color = match agent.team.as_str() {
                "red" => Color::RED,
                "blue" => Color::BLUE,
                _ => Color::WHITE,
            };

            // Позиция с анимацией
            let time_offset = agent_id.len() as f32;
            let x = (time.elapsed_seconds() + time_offset).sin() * 8.0;
            let z = (time.elapsed_seconds() * 0.7 + time_offset).cos() * 6.0;
            
            // Создаем полноценного агента с AI компонентами
            let _agent_entity = commands.spawn((
                SpatialBundle {
                    transform: Transform::from_xyz(x, 0.0, z),
                    ..default()
                },
                agent.clone(),
                AgentVisual,
                AIBrain {
                    model: "llama3.2:1b".to_string(),
                    context: format!("You are {} from team {}. Your current status: {}. Make strategic decisions.", 
                                   agent.name, agent.team, agent.status),
                    last_action: "spawned".to_string(),
                    thinking: false,
                },
                Movement {
                    velocity: Vec3::ZERO,
                    target_position: None,
                    speed: 3.0,
                },
                Combat {
                    attack_damage: 25.0,
                    attack_range: 2.0,
                    defense: 10.0,
                    last_attack_time: 0.0,
                    attack_cooldown: 1.0, // 1 секунда кулдаун
                },
                Name::new(format!("Agent: {}", agent.name)),
            )).with_children(|parent| {
                // Тело (торс)
                parent.spawn(PbrBundle {
                    mesh: meshes.add(shape::Box::new(0.4, 0.8, 0.2).into()),
                    material: materials.add(StandardMaterial {
                        base_color: color,
                        metallic: 0.1,
                        perceptual_roughness: 0.8,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..default()
                });
                
                // Голова
                parent.spawn(PbrBundle {
                    mesh: meshes.add(shape::UVSphere { radius: 0.2, sectors: 16, stacks: 8 }.into()),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.9, 0.7, 0.6), // Цвет кожи
                        metallic: 0.0,
                        perceptual_roughness: 0.9,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 1.6, 0.0),
                    ..default()
                });
                
                // Левая рука
                parent.spawn(PbrBundle {
                    mesh: meshes.add(shape::Box::new(0.1, 0.6, 0.1).into()),
                    material: materials.add(StandardMaterial {
                        base_color: color * 0.8,
                        metallic: 0.1,
                        perceptual_roughness: 0.8,
                        ..default()
                    }),
                    transform: Transform::from_xyz(-0.3, 1.0, 0.0),
                    ..default()
                });
                
                // Правая рука
                parent.spawn(PbrBundle {
                    mesh: meshes.add(shape::Box::new(0.1, 0.6, 0.1).into()),
                    material: materials.add(StandardMaterial {
                        base_color: color * 0.8,
                        metallic: 0.1,
                        perceptual_roughness: 0.8,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.3, 1.0, 0.0),
                    ..default()
                });
                
                // Левая нога
                parent.spawn(PbrBundle {
                    mesh: meshes.add(shape::Box::new(0.15, 0.8, 0.15).into()),
                    material: materials.add(StandardMaterial {
                        base_color: color * 0.6,
                        metallic: 0.1,
                        perceptual_roughness: 0.8,
                        ..default()
                    }),
                    transform: Transform::from_xyz(-0.1, 0.4, 0.0),
                    ..default()
                });
                
                // Правая нога
                parent.spawn(PbrBundle {
                    mesh: meshes.add(shape::Box::new(0.15, 0.8, 0.15).into()),
                    material: materials.add(StandardMaterial {
                        base_color: color * 0.6,
                        metallic: 0.1,
                        perceptual_roughness: 0.8,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.1, 0.4, 0.0),
                    ..default()
                });
            }).id();

            // Добавляем полоску здоровья
            let health_ratio = agent.health / 100.0;
            let health_color = if health_ratio > 0.6 {
                Color::GREEN
            } else if health_ratio > 0.3 {
                Color::YELLOW
            } else {
                Color::RED
            };

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Box::new(health_ratio * 1.0, 0.1, 0.05).into()),
                    material: materials.add(StandardMaterial {
                        base_color: health_color,
                        ..default()
                    }),
                    transform: Transform::from_xyz(x, 1.2, z),
                    ..default()
                },
                HealthBar,
                Name::new(format!("HealthBar: {}", agent.name)),
            ));

            println!("✨ Created agent: {} at ({:.1}, {:.1})", agent.name, x, z);
        }
    }
}

fn arena_gui_system(
    mut contexts: EguiContexts,
    mut arena_state: ResMut<ArenaState>,
    mut scene_manager: ResMut<SceneManager>,
    mut agent_prompts: ResMut<AgentPrompts>,
    mut training_system: ResMut<TrainingSystem>,
    mut ollama_connection: ResMut<OllamaConnection>,
    mut log_system: ResMut<LogSystem>,
    mut prompt_execution: ResMut<PromptExecution>,
    mut agent_appearance: ResMut<AgentAppearance>,
    mut movement_settings: ResMut<MovementSettings>,
    agent_selection: ResMut<AgentSelection>,
    mut arena_drag_drop: ResMut<ArenaDragDrop>,
    mut theme_settings: ResMut<ThemeSettings>,
    mut hotkey_settings: ResMut<HotkeySettings>,
    time: Res<Time>,
    agents_query: Query<(&Agent, &AIBrain), With<AgentVisual>>,
) {
    // Применяем тему
    apply_theme(&mut contexts, &theme_settings);
    
    // Верхняя панель
    egui::TopBottomPanel::top("top_panel").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.heading("🌌 Heaven AI Arena Viewer");
            ui.separator();
            ui.label(format!("Status: {}", arena_state.connection_status));
            ui.separator();
            ui.label(format!("FPS: {:.0}", 1.0 / time.delta_seconds()));
        });
    });

    // Боковая панель с информацией об агентах
    egui::SidePanel::left("agents_panel").show(contexts.ctx_mut(), |ui| {
        ui.heading("🤖 Agents");
        ui.separator();
        
        ui.label(format!("Total: {}", arena_state.total_agents));
        ui.label(format!("Match Time: {:.1}s", arena_state.match_time));
        
        ui.separator();
        
        ui.label("💡 Tip: Click ✏️ to edit agent prompts, then press 🏃 Run to execute them!");
        
        ui.separator();
        
        // Менеджер сцен
        ui.heading("🏟️ Scene Manager");
        ui.label(format!("Current: {}", scene_manager.current_scene));
        let current_scene = scene_manager.current_scene.clone();
        let available_scenes = scene_manager.available_scenes.clone();
        egui::ComboBox::from_label("Select Scene")
            .selected_text(&current_scene)
            .show_ui(ui, |ui| {
                for scene in &available_scenes {
                    ui.selectable_value(&mut scene_manager.current_scene, scene.clone(), scene);
                }
            });
        
        if ui.button("🎬 Create New Scene").clicked() {
            scene_manager.scene_creator_open = true;
        }
        
        ui.separator();
        
        // Детальная информация об агентах с промптами
        for agent in arena_state.agents.values() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}", agent.name));
                    if ui.button("✏️").clicked() {
                        agent_prompts.selected_agent = agent.id.clone();
                        agent_prompts.temp_prompt = agent_prompts.prompts.get(&agent.id).unwrap_or(&String::new()).clone();
                        agent_prompts.custom_prompt_window = true;
                    }
                });
                
                ui.label(format!("ID: {}", agent.id));
                ui.label(format!("Team: {}", agent.team));
                ui.label(format!("Health: {:.0}%", agent.health));
                ui.label(format!("Energy: {:.0}%", agent.energy));
                let status_emoji = match agent.status.as_str() {
                    "waiting_for_command" => "⏳ Waiting for command",
                    "attacking" => "⚔️ Attacking",
                    "defending" => "🛡️ Defending", 
                    "searching_key" => "🔍 Searching key",
                    "moving" => "🏃 Moving",
                    "idle" => "💤 Idle",
                    _ => &format!("🤖 {}", agent.status),
                };
                ui.label(format!("Status: {}", status_emoji));
                ui.label(format!("AI: {}", if agent.ai_enabled { "🧠 Enabled" } else { "❌ Disabled" }));
                
                // Показываем текущий промпт
                if let Some(prompt) = agent_prompts.prompts.get(&agent.id) {
                    ui.label(format!("Prompt: {}", if prompt.chars().count() > 30 { 
                        format!("{}...", prompt.chars().take(30).collect::<String>()) 
                    } else { 
                        prompt.clone() 
                    }));
                }
                
                // Показываем AI состояние если доступно
                if let Some((_, brain)) = agents_query.iter().find(|(a, _)| a.id == agent.id) {
                    ui.label(format!("Model: {}", brain.model));
                    ui.label(format!("Last Action: {}", brain.last_action));
                    if brain.thinking {
                        ui.label("🤔 Thinking...");
                    }
                }
            });
        }
    });

    // Нижняя панель с контролами
    egui::TopBottomPanel::bottom("controls_panel").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("🎮 Controls:");
            ui.separator();
            ui.label("WASD: Move camera");
            ui.separator();
            ui.label("Mouse: Look around");
            ui.separator();
            ui.label("Scroll: Zoom");
            ui.separator();
            
            // Ollama Connection
            let connect_text = if ollama_connection.connected { 
                "🟢 Connected" 
            } else { 
                "🧠 Connect Ollama" 
            };
            if ui.button(connect_text).clicked() {
                if !ollama_connection.connected {
                    log_system.add_log(format!("🧠 Attempting to connect to Ollama at {}...", ollama_connection.url));
                    ollama_connection.status = "Connecting...".to_string();
                    // Реальная система подключения запустится через ollama_connection_system
                } else {
                    log_system.add_log("🔌 Disconnecting from Ollama...".to_string());
                    ollama_connection.connected = false;
                    ollama_connection.status = "Disconnected".to_string();
                    ollama_connection.available_models.clear();
                }
            }
            
            // Model Management
            ui.separator();
            ui.label("📦 Model Management:");
            
            ui.horizontal(|ui| {
                ui.label("Model:");
                ui.text_edit_singleline(&mut ollama_connection.model_to_download);
                if ui.button("📥 Download").clicked() {
                    log_system.add_log(format!("📥 Starting {} download...", ollama_connection.model_to_download));
                    ollama_connection.downloading_model = true;
                    ollama_connection.download_progress = 0.0;
                }
            });
            
            ui.label("Popular models:");
            ui.horizontal(|ui| {
                if ui.button("llama3.2:1b").clicked() {
                    ollama_connection.model_to_download = "llama3.2:1b".to_string();
                }
                if ui.button("llama3.2:3b").clicked() {
                    ollama_connection.model_to_download = "llama3.2:3b".to_string();
                }
                if ui.button("llama3.1:8b").clicked() {
                    ollama_connection.model_to_download = "llama3.1:8b".to_string();
                }
            });
            
            if ollama_connection.downloading_model {
                ui.label("⏳ Downloading model...");
                ui.add(egui::ProgressBar::new(ollama_connection.download_progress));
            }
            
            // Run Prompts
            let run_text = if prompt_execution.running { "⏸️ Stop" } else { "🏃 Run" };
            if ui.button(run_text).clicked() {
                if !prompt_execution.running {
                    log_system.add_log("🏃 Starting prompt execution for all agents...".to_string());
                    prompt_execution.running = true;
                    prompt_execution.results.clear();
                    
                    for (agent_id, prompt) in &agent_prompts.prompts {
                        log_system.add_log(format!("📝 Agent {}: '{}'", agent_id, prompt));
                    }
                } else {
                    log_system.add_log("⏸️ Stopping prompt execution...".to_string());
                    prompt_execution.running = false;
                    
                    // Останавливаем всех агентов
                    for agent in arena_state.agents.values_mut() {
                        agent.status = "waiting_for_command".to_string();
                    }
                    log_system.add_log("⏹️ All agents stopped and waiting for commands".to_string());
                }
            }
            
            // Training
            let training_text = if training_system.is_training { 
                "⏹️ Stop Training" 
            } else { 
                "🚀 Train Agents" 
            };
            if ui.button(training_text).clicked() {
                if !training_system.is_training {
                    log_system.add_log("🚀 Starting AI training session...".to_string());
                    log_system.add_log(format!("⚙️ Learning Rate: {}", training_system.learning_rate));
                    log_system.add_log(format!("📊 Total Epochs: {}", training_system.total_epochs));
                    log_system.add_log(format!("🔢 Steps per Epoch: {}", training_system.steps_in_epoch));
                    
                    training_system.is_training = true;
                    training_system.current_epoch = 0;
                    training_system.current_step = 0;
                    training_system.training_window_open = true;
                } else {
                    log_system.add_log("⏹️ Stopping training session...".to_string());
                    training_system.is_training = false;
                }
            }
            
            // Agent Appearance
            if ui.button("🎨 Agent Appearance").clicked() {
                agent_appearance.appearance_window_open = true;
            }
            
            // Movement Settings
            if ui.button("⚙️ Movement Settings").clicked() {
                movement_settings.settings_window_open = true;
            }
            
            // Arena Editor
            if ui.button("🏗️ Arena Editor").clicked() {
                arena_drag_drop.arena_editor_open = true;
            }
            
            // Ollama Server Monitor
            if ui.button("🤖 Ollama Server").clicked() {
                // Это будет обработано в отдельной системе
            }
            
            // Note: Time Simulation, Agent Chat and Add New Agent moved to separate systems
            
            // Theme Settings
            if ui.button("🎨 Theme Settings").clicked() {
                theme_settings.theme_window_open = true;
            }
            
            // Hotkey Settings
            if ui.button("⌨️ Hotkey Settings").clicked() {
                hotkey_settings.hotkey_window_open = true;
            }
        });
    });
    
    // Правая панель с логами
    egui::SidePanel::right("logs_panel").show(contexts.ctx_mut(), |ui| {
        ui.heading("📝 System Logs");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut log_system.show_logs, "Show Logs");
            if ui.button("🗑️ Clear").clicked() {
                log_system.logs.clear();
                log_system.add_log("Logs cleared".to_string());
            }
        });
        
        ui.separator();
        
        if log_system.show_logs {
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for log in log_system.logs.iter().rev().take(20) {
                        ui.label(log);
                    }
                });
        }
        
        ui.separator();
        
        ui.heading("🔄 System Status");
        ui.label(format!("Ollama: {}", ollama_connection.status));
        ui.label(format!("Training: {}", if training_system.is_training { "🟢 Active" } else { "🔴 Stopped" }));
        ui.label(format!("Prompt Execution: {}", if prompt_execution.running { "🟢 Running" } else { "⚪ Idle" }));
        
        if movement_settings.agent_selection_enabled {
            ui.separator();
            if agent_selection.selected_agents.is_empty() {
                ui.label("👆 Click agents to select them");
            } else {
                ui.label(format!("Selected: {} agents", agent_selection.selected_agents.len()));
                for agent_id in &agent_selection.selected_agents {
                    ui.label(format!("  • {}", agent_id));
                }
            }
        }
        // ui.label(format!("Demo Mode: {}", if demo_mode.enabled { "🟢 On" } else { "🔴 Off" })); // УБРАЛИ ДЕМО-РЕЖИМ
    });
    
    // Окна редактирования
    show_prompt_editor(&mut contexts, &mut agent_prompts, &mut log_system);
    show_scene_creator(&mut contexts, &mut scene_manager, &mut log_system);
    show_training_window(&mut contexts, &mut training_system);
    show_appearance_window(&mut contexts, &mut agent_appearance);
    show_movement_settings(&mut contexts, &mut movement_settings);
    show_arena_editor(&mut contexts, &mut arena_drag_drop);
    show_theme_settings(&mut contexts, &mut theme_settings);
    show_hotkey_settings(&mut contexts, &mut hotkey_settings);
    // show_time_simulation_window(&mut contexts, &mut time_simulation);
    // show_agent_chat_window(&mut contexts, &mut agent_chat);
}

fn camera_controls(
    keyboard_input: Res<Input<KeyCode>>,
    mut contexts: EguiContexts,
    mut query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    // Если UI имеет фокус, не обрабатываем управление камерой
    if contexts.ctx_mut().wants_keyboard_input() || contexts.ctx_mut().wants_pointer_input() {
        return;
    }
    if let Ok(mut transform) = query.get_single_mut() {
        let mut velocity = Vec3::ZERO;
        
        if keyboard_input.pressed(KeyCode::W) {
            velocity += transform.forward();
        }
        if keyboard_input.pressed(KeyCode::S) {
            velocity += transform.back();
        }
        if keyboard_input.pressed(KeyCode::A) {
            velocity += transform.left();
        }
        if keyboard_input.pressed(KeyCode::D) {
            velocity += transform.right();
        }
        if keyboard_input.pressed(KeyCode::Space) {
            velocity += Vec3::Y;
        }
        if keyboard_input.pressed(KeyCode::ControlLeft) {
            velocity -= Vec3::Y;
        }
        
        transform.translation += velocity * 10.0 * time.delta_seconds();
    }
}

// AI система принятия решений
/// AI decision system that makes agents interact intelligently
/// Agents will find enemies, move towards them, and engage in combat
fn ai_decision_system(
    time: Res<Time>,
    mut agents: Query<(&mut Agent, &mut AIBrain, &mut Movement, &Transform), With<AgentVisual>>,
    arena_state: Res<ArenaState>,
) {
    for (mut agent, mut brain, mut movement, transform) in agents.iter_mut() {
        if !agent.ai_enabled {
            continue;
        }
        
        agent.decision_cooldown.tick(time.delta());
        
        if agent.decision_cooldown.just_finished() && !brain.thinking {
            // Enhanced AI logic with team-based interactions using arena_state
            let decision = simple_ai_decision_from_arena(&agent, transform, &arena_state);
            
            match decision.as_str() {
                "move_random" => {
                    let target = Vec3::new(
                        (rand::random::<f32>() - 0.5) * 16.0,
                        0.5,
                        (rand::random::<f32>() - 0.5) * 12.0,
                    );
                    movement.target_position = Some(target);
                    agent.status = "moving".to_string();
                    brain.last_action = "patrolling area".to_string();
                }
                "move_to_enemy" => {
                    // Find nearest enemy using arena state
                    let mut nearest_enemy_pos = None;
                    let mut nearest_distance = f32::MAX;
                    
                    for (other_id, other_agent) in &arena_state.agents {
                        if other_agent.team != agent.team {
                            let other_position = match other_id.as_str() {
                                "red_gladiator" => Vec3::new(-5.0, 0.5, 0.0),
                                "blue_warrior" => Vec3::new(5.0, 0.5, 0.0),
                                "red_scout" => Vec3::new(0.0, 0.5, 5.0),
                                _ => Vec3::new(0.0, 0.5, 0.0),
                            };
                            let distance = transform.translation.distance(other_position);
                            if distance < nearest_distance {
                                nearest_distance = distance;
                                nearest_enemy_pos = Some(other_position);
                            }
                        }
                    }
                    
                    if let Some(enemy_pos) = nearest_enemy_pos {
                        movement.target_position = Some(enemy_pos);
                        agent.status = "hunting".to_string();
                        brain.last_action = "moving towards enemy".to_string();
                    } else {
                        // No enemy found, patrol
                        let target = Vec3::new(
                            (rand::random::<f32>() - 0.5) * 16.0,
                            0.5,
                            (rand::random::<f32>() - 0.5) * 12.0,
                        );
                        movement.target_position = Some(target);
                        agent.status = "searching".to_string();
                        brain.last_action = "searching for enemies".to_string();
                    }
                }
                "move_to_ally" => {
                    // Find nearest ally using arena state
                    let mut nearest_ally_pos = None;
                    let mut nearest_distance = f32::MAX;
                    
                    for (other_id, other_agent) in &arena_state.agents {
                        if other_agent.team == agent.team && other_agent.id != agent.id {
                            let other_position = match other_id.as_str() {
                                "red_gladiator" => Vec3::new(-5.0, 0.5, 0.0),
                                "blue_warrior" => Vec3::new(5.0, 0.5, 0.0),
                                "red_scout" => Vec3::new(0.0, 0.5, 5.0),
                                _ => Vec3::new(0.0, 0.5, 0.0),
                            };
                            let distance = transform.translation.distance(other_position);
                            if distance < nearest_distance {
                                nearest_distance = distance;
                                nearest_ally_pos = Some(other_position);
                            }
                        }
                    }
                    
                    if let Some(ally_pos) = nearest_ally_pos {
                        movement.target_position = Some(ally_pos);
                        agent.status = "regrouping".to_string();
                        brain.last_action = "moving towards ally".to_string();
                    } else {
                        // No ally found, defend in place
                        movement.target_position = None;
                        agent.status = "defending".to_string();
                        brain.last_action = "defending position".to_string();
                    }
                }
                "attack" => {
                    movement.target_position = None;
                    agent.status = "attacking".to_string();
                    brain.last_action = "engaging enemy".to_string();
                }
                "defend" => {
                    movement.target_position = None;
                    agent.status = "defending".to_string();
                    brain.last_action = "taking defensive stance".to_string();
                }
                _ => {
                    agent.status = "idle".to_string();
                    brain.last_action = "waiting".to_string();
                }
            }
        }
    }
}

/// Simple AI logic that makes agents interact with each other using arena state
/// Agents will find enemies, move towards them, and engage in combat
fn simple_ai_decision_from_arena(
    agent: &Agent,
    transform: &Transform,
    arena_state: &ArenaState,
) -> String {
    // Find the nearest enemy using arena state
    let mut nearest_enemy: Option<(f32, Vec3)> = None;
    let mut nearest_ally: Option<(f32, Vec3)> = None;
    let mut enemies_found = 0;
    let mut allies_found = 0;
    
    for (other_id, other_agent) in &arena_state.agents {
        if other_agent.id == agent.id {
            continue; // Skip self
        }
        
        // For now, we'll use fixed positions since we don't have transform data in arena_state
        // This is a simplified approach - in practice, you'd want to store positions in arena_state
        let other_position = match other_id.as_str() {
            "red_gladiator" => Vec3::new(-5.0, 0.5, 0.0),
            "blue_warrior" => Vec3::new(5.0, 0.5, 0.0),
            "red_scout" => Vec3::new(0.0, 0.5, 5.0),
            _ => Vec3::new(0.0, 0.5, 0.0), // Default position
        };
        
        let distance = transform.translation.distance(other_position);
        
        if other_agent.team != agent.team {
            // Enemy found
            enemies_found += 1;
            if let Some((current_dist, _)) = nearest_enemy {
                if distance < current_dist {
                    nearest_enemy = Some((distance, other_position));
                }
            } else {
                nearest_enemy = Some((distance, other_position));
            }
        } else {
            // Ally found
            allies_found += 1;
            if let Some((current_dist, _)) = nearest_ally {
                if distance < current_dist {
                    nearest_ally = Some((distance, other_position));
                }
            } else {
                nearest_ally = Some((distance, other_position));
            }
        }
    }
    
    // Decision making based on agent's health and nearby entities
    match agent.health {
        h if h > 70.0 => {
            // High health - be aggressive
            if let Some((distance, _)) = nearest_enemy {
                if distance < 5.0 {
                    println!("🔥 {} ({}): Attacking nearby enemy at distance {:.1} (found {} enemies)", agent.name, agent.team, distance, enemies_found);
                    "attack".to_string()
                } else if distance < 10.0 {
                    println!("🏃 {} ({}): Moving towards enemy at distance {:.1} (found {} enemies)", agent.name, agent.team, distance, enemies_found);
                    "move_to_enemy".to_string()
                } else {
                    println!("🔍 {} ({}): Patrolling for enemies (found {} enemies, closest at {:.1})", agent.name, agent.team, enemies_found, distance);
                    "move_random".to_string()
                }
            } else {
                println!("🚶 {} ({}): No enemies found, patrolling (scanned {} agents)", agent.name, agent.team, enemies_found + allies_found);
                "move_random".to_string()
            }
        }
        h if h > 30.0 => {
            // Medium health - be cautious
            if let Some((distance, _)) = nearest_enemy {
                if distance < 3.0 {
                    println!("🔥 {} ({}): Defending against close enemy at distance {:.1}", agent.name, agent.team, distance);
                    "attack".to_string()
                } else if distance < 8.0 {
                    if let Some((ally_distance, _)) = nearest_ally {
                        if ally_distance < 5.0 {
                            println!("🏃 {} ({}): Moving towards enemy with ally support", agent.name, agent.team);
                            "move_to_enemy".to_string()
                        } else {
                            println!("🤝 {} ({}): Seeking ally support", agent.name, agent.team);
                            "move_to_ally".to_string()
                        }
                    } else {
                        println!("🛡️ {} ({}): Taking defensive position", agent.name, agent.team);
                        "defend".to_string()
                    }
                } else {
                    println!("🔍 {} ({}): Cautiously patrolling", agent.name, agent.team);
                    "move_random".to_string()
                }
            } else {
                println!("🚶 {} ({}): No enemies found, patrolling cautiously", agent.name, agent.team);
                "move_random".to_string()
            }
        }
        _ => {
            // Low health - be defensive
            if let Some((ally_distance, _)) = nearest_ally {
                if ally_distance > 3.0 {
                    println!("🚑 {} ({}): Low health, seeking ally support", agent.name, agent.team);
                    "move_to_ally".to_string()
                } else {
                    println!("🛡️ {} ({}): Low health, defending with ally", agent.name, agent.team);
                    "defend".to_string()
                }
            } else {
                println!("🛡️ {} ({}): Low health, taking defensive stance", agent.name, agent.team);
                "defend".to_string()
            }
        }
    }
}

/// Simple AI logic that makes agents interact with each other
/// Agents will find enemies, move towards them, and engage in combat
fn simple_ai_decision(
    agent: &Agent,
    transform: &Transform,
    other_agents: &Query<(&Agent, &Transform), With<AgentVisual>>,
) -> String {
    // Find the nearest enemy
    let mut nearest_enemy: Option<(f32, Vec3)> = None;
    let mut nearest_ally: Option<(f32, Vec3)> = None;
    let mut enemies_found = 0;
    let mut allies_found = 0;
    
    for (other_agent, other_transform) in other_agents.iter() {
        if other_agent.id == agent.id {
            continue; // Skip self
        }
        
        let distance = transform.translation.distance(other_transform.translation);
        
        if other_agent.team != agent.team {
            // Enemy found
            enemies_found += 1;
            if let Some((current_dist, _)) = nearest_enemy {
                if distance < current_dist {
                    nearest_enemy = Some((distance, other_transform.translation));
                }
            } else {
                nearest_enemy = Some((distance, other_transform.translation));
            }
        } else {
            // Ally found
            allies_found += 1;
            if let Some((current_dist, _)) = nearest_ally {
                if distance < current_dist {
                    nearest_ally = Some((distance, other_transform.translation));
                }
            } else {
                nearest_ally = Some((distance, other_transform.translation));
            }
        }
    }
    
    // Decision making based on agent's health and nearby entities
    match agent.health {
        h if h > 70.0 => {
            // High health - be aggressive
            if let Some((distance, _)) = nearest_enemy {
                if distance < 5.0 {
                    println!("🔥 {} ({}): Attacking nearby enemy at distance {:.1} (found {} enemies)", agent.name, agent.team, distance, enemies_found);
                    "attack".to_string()
                } else if distance < 10.0 {
                    println!("🏃 {} ({}): Moving towards enemy at distance {:.1} (found {} enemies)", agent.name, agent.team, distance, enemies_found);
                    "move_to_enemy".to_string()
                } else {
                    println!("🔍 {} ({}): Patrolling for enemies (found {} enemies, closest at {:.1})", agent.name, agent.team, enemies_found, distance);
                    "move_random".to_string()
                }
            } else {
                println!("🚶 {} ({}): No enemies found, patrolling (scanned {} agents)", agent.name, agent.team, enemies_found + allies_found);
                "move_random".to_string()
            }
        }
        h if h > 30.0 => {
            // Medium health - be cautious
            if let Some((distance, _)) = nearest_enemy {
                if distance < 3.0 {
                    println!("⚔️ {} ({}): Low health, engaging in close combat", agent.name, agent.team);
                    "attack".to_string()
                } else if distance < 8.0 {
                    println!("🛡️ {} ({}): Medium health, defending position", agent.name, agent.team);
                    "defend".to_string()
                } else {
                    println!("🚶 {} ({}): Medium health, moving carefully", agent.name, agent.team);
                    "move_random".to_string()
                }
            } else {
                "move_random".to_string()
            }
        }
        _ => {
            // Low health - be defensive
            if let Some((ally_distance, _)) = nearest_ally {
                if ally_distance > 5.0 {
                    println!("🏥 {} ({}): Low health, retreating to ally", agent.name, agent.team);
                    "move_to_ally".to_string()
                } else {
                    println!("🛡️ {} ({}): Low health, defending near ally", agent.name, agent.team);
                    "defend".to_string()
                }
            } else {
                println!("🛡️ {} ({}): Low health, no allies found, defending", agent.name, agent.team);
                "defend".to_string()
            }
        }
    }
}

// Система движения
fn movement_system(
    time: Res<Time>,
    movement_settings: Res<MovementSettings>,
    mut agents: Query<(&mut Transform, &mut Movement), With<AgentVisual>>,
) {
    for (mut transform, mut movement) in agents.iter_mut() {
        // Применяем скорость
        transform.translation += movement.velocity * time.delta_seconds();
        
        // Движение к цели
        if let Some(target) = movement.target_position {
            let direction = (target - transform.translation).normalize_or_zero();
            let distance = transform.translation.distance(target);
            
            if distance > 0.5 {
                // Плавно интерполируем к целевой скорости
                let target_velocity = direction * (movement.speed * movement_settings.movement_speed);
                movement.velocity = movement.velocity.lerp(target_velocity, time.delta_seconds() * movement_settings.movement_smoothness);
                
                // Поворот к цели с настраиваемой плавностью
                if direction.length() > 0.1 {
                    let target_rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
                    transform.rotation = transform.rotation.slerp(target_rotation, time.delta_seconds() * movement_settings.movement_smoothness);
                }
            } else {
                // Плавно останавливаемся
                movement.velocity = movement.velocity.lerp(Vec3::ZERO, time.delta_seconds() * movement_settings.movement_smoothness * 2.0);
                if movement.velocity.length() < 0.1 {
                    movement.velocity = Vec3::ZERO;
                    movement.target_position = None;
                }
            }
        } else {
            // Постепенно останавливаемся
            movement.velocity *= 0.9;
        }
        
        // Ограничиваем движение в пределах арены
        transform.translation.x = transform.translation.x.clamp(-9.0, 9.0);
        transform.translation.z = transform.translation.z.clamp(-7.0, 7.0);
        transform.translation.y = 0.0;
    }
}

// Система боя
fn combat_system(
    time: Res<Time>,
    mut agents: Query<(&mut Agent, &Transform, &mut Combat), With<AgentVisual>>,
) {
    let current_time = time.elapsed_seconds();
    let mut attacks_to_perform = Vec::new();
    
    // Сначала собираем информацию о возможных атаках
    {
        let agent_data: Vec<_> = agents.iter().collect();
        
        for i in 0..agent_data.len() {
            for j in (i + 1)..agent_data.len() {
                let (agent1, transform1, combat1) = &agent_data[i];
                let (agent2, transform2, combat2) = &agent_data[j];
                
                // Проверяем разные команды
                if agent1.team == agent2.team {
                    continue;
                }
                
                let distance = transform1.translation.distance(transform2.translation);
                
                // Бой в ближнем радиусе
                if distance < combat1.attack_range.max(combat2.attack_range) {
                    if agent1.status == "attacking" && current_time - combat1.last_attack_time > combat1.attack_cooldown {
                        let damage = combat1.attack_damage - combat2.defense;
                        if damage > 0.0 {
                            attacks_to_perform.push((i, j, damage, agent1.name.clone(), agent2.name.clone()));
                        }
                    }
                    
                    if agent2.status == "attacking" && current_time - combat2.last_attack_time > combat2.attack_cooldown {
                        let damage = combat2.attack_damage - combat1.defense;
                        if damage > 0.0 {
                            attacks_to_perform.push((j, i, damage, agent2.name.clone(), agent1.name.clone()));
                        }
                    }
                }
            }
        }
    }
    
    // Выполняем атаки и обновляем кулдауны
    for (attacker_idx, target_idx, damage, attacker_name, target_name) in attacks_to_perform {
        let mut agent_data: Vec<_> = agents.iter_mut().collect();
        
        // Update attacker's cooldown
        if let Some((_, _, combat)) = agent_data.get_mut(attacker_idx) {
            combat.last_attack_time = current_time;
        }
        
        // Damage target
        if let Some((ref mut target_agent, _, _)) = agent_data.get_mut(target_idx) {
            target_agent.health -= damage;
            println!("⚔️ {} attacks {} for {} damage! {} health: {:.1}", 
                attacker_name, target_name, damage, target_name, target_agent.health);
            
            // Check if target is dead
            if target_agent.health <= 0.0 {
                println!("💀 {} has been defeated by {}!", target_name, attacker_name);
                target_agent.health = 0.0;
                target_agent.status = "dead".to_string();
                target_agent.ai_enabled = false;
            }
        }
    }
}

/// System to respawn dead agents after a delay
fn agent_respawn_system(
    time: Res<Time>,
    mut agents: Query<(&mut Agent, &mut Transform), With<AgentVisual>>,
) {
    for (mut agent, mut transform) in agents.iter_mut() {
        if agent.status == "dead" && agent.health <= 0.0 {
            // Respawn after 10 seconds
            if time.elapsed_seconds() as u32 % 10 == 0 && rand::random::<f32>() < 0.1 {
                agent.health = 100.0;
                agent.status = "respawned".to_string();
                agent.ai_enabled = true;
                
                // Respawn at a random position
                let spawn_pos = Vec3::new(
                    (rand::random::<f32>() - 0.5) * 14.0,
                    0.5,
                    (rand::random::<f32>() - 0.5) * 10.0,
                );
                transform.translation = spawn_pos;
                
                println!("✨ {} respawned at {:?}", agent.name, spawn_pos);
            }
        }
    }
}

// Training simulation system
fn training_simulation_system(
    time: Res<Time>,
    mut training_system: ResMut<TrainingSystem>,
) {
    if training_system.is_training {
        // Симулируем шаги тренировки (каждые 0.1 секунды)
        if time.elapsed_seconds() as u32 % 10 == 0 {
            training_system.current_step += 1;
            
            // Показываем прогресс каждые 10 шагов
            if training_system.current_step % 10 == 0 {
                println!("📈 Epoch {}/{}, Step {}/{} ({:.1}%)", 
                        training_system.current_epoch + 1, 
                        training_system.total_epochs,
                        training_system.current_step,
                        training_system.steps_in_epoch,
                        (training_system.current_step as f32 / training_system.steps_in_epoch as f32) * 100.0);
            }
        }
        
        if training_system.current_step >= training_system.steps_in_epoch {
            training_system.current_step = 0;
            training_system.current_epoch += 1;
            
            // Генерируем fake метрики для демонстрации
            let fake_loss = 1.0 - (training_system.current_epoch as f32 / training_system.total_epochs as f32) * 0.8 + 
                           (rand::random::<f32>() - 0.5) * 0.1;
            let fake_reward = (training_system.current_epoch as f32 / training_system.total_epochs as f32) * 100.0 + 
                             (rand::random::<f32>() - 0.5) * 10.0;
            let fake_accuracy = (training_system.current_epoch as f32 / training_system.total_epochs as f32) * 0.9 + 
                               (rand::random::<f32>() - 0.5) * 0.1;
            
            training_system.loss_history.push(fake_loss.max(0.1));
            training_system.reward_history.push(fake_reward.max(0.0));
            training_system.accuracy_history.push(fake_accuracy.clamp(0.0, 1.0));
            
            println!("📊 Epoch {}: Loss={:.4}, Reward={:.2}, Accuracy={:.1}%", 
                    training_system.current_epoch, fake_loss, fake_reward, fake_accuracy * 100.0);
            
            if training_system.current_epoch >= training_system.total_epochs {
                training_system.is_training = false;
                println!("✅ Training completed!");
            }
        }
    }
}

// Окно редактирования промптов
fn show_prompt_editor(contexts: &mut EguiContexts, agent_prompts: &mut AgentPrompts, log_system: &mut LogSystem) {
    if agent_prompts.custom_prompt_window {
        egui::Window::new("✏️ Edit Agent Prompt")
            .default_size([400.0, 300.0])
            .resizable(true)
            .collapsible(true)
            .show(contexts.ctx_mut(), |ui| {
                ui.heading(format!("Agent: {}", agent_prompts.selected_agent));
                ui.separator();
                
                ui.label("Enter custom prompt:");
                ui.text_edit_multiline(&mut agent_prompts.temp_prompt);
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("💾 Save").clicked() {
                        agent_prompts.prompts.insert(
                            agent_prompts.selected_agent.clone(), 
                            agent_prompts.temp_prompt.clone()
                        );
                        log_system.add_log(format!("💾 Saved prompt for {}: '{}'", agent_prompts.selected_agent, agent_prompts.temp_prompt));
                        agent_prompts.custom_prompt_window = false;
                    }
                    
                    if ui.button("❌ Cancel").clicked() {
                        agent_prompts.custom_prompt_window = false;
                    }
                });
                
                ui.separator();
                ui.label("🎯 Prompt Examples:");
                if ui.button("Find the key").clicked() {
                    agent_prompts.temp_prompt = "Find the key in the arena and secure it".to_string();
                }
                if ui.button("Attack enemies").clicked() {
                    agent_prompts.temp_prompt = "Attack enemy agents and protect your team".to_string();
                }
                if ui.button("Defend position").clicked() {
                    agent_prompts.temp_prompt = "Defend your position and support allies".to_string();
                }
            });
    }
}

// Окно создания сцен
fn show_scene_creator(contexts: &mut EguiContexts, scene_manager: &mut SceneManager, log_system: &mut LogSystem) {
    if scene_manager.scene_creator_open {
        egui::Window::new("🎬 Scene Creator")
            .default_size([500.0, 400.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Create New Scene");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Scene Name:");
                    ui.text_edit_singleline(&mut scene_manager.new_scene_name);
                });
                
                ui.separator();
                
                ui.label("🏗️ Scene Templates:");
                ui.horizontal(|ui| {
                    if ui.button("🏟️ Basic Arena").clicked() {
                        scene_manager.new_scene_name = "Custom Arena".to_string();
                    }
                    if ui.button("🌀 Maze").clicked() {
                        scene_manager.new_scene_name = "Custom Maze".to_string();
                    }
                    if ui.button("⚔️ Battle Royal").clicked() {
                        scene_manager.new_scene_name = "Custom Battle".to_string();
                    }
                });
                
                ui.separator();
                
                ui.label("🎯 Objectives:");
                ui.checkbox(&mut true, "Capture the Flag");
                ui.checkbox(&mut false, "Eliminate All Enemies");
                ui.checkbox(&mut false, "Find Hidden Items");
                ui.checkbox(&mut false, "Survive X Minutes");
                
                ui.separator();
                
                ui.label("🌍 Environment:");
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.add(egui::Slider::new(&mut 20, 10..=50).text("x"));
                    ui.add(egui::Slider::new(&mut 20, 10..=50).text("z"));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Obstacles:");
                    ui.add(egui::Slider::new(&mut 5, 0..=20).text("count"));
                });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("✅ Create Scene").clicked() {
                        if !scene_manager.new_scene_name.is_empty() {
                            log_system.add_log(format!("🎬 Creating new scene: '{}'", scene_manager.new_scene_name));
                            log_system.add_log("🏗️ Generating arena geometry...".to_string());
                            log_system.add_log("🎯 Setting up objectives...".to_string());
                            log_system.add_log("🌍 Placing environment objects...".to_string());
                            
                            scene_manager.available_scenes.push(scene_manager.new_scene_name.clone());
                            scene_manager.current_scene = scene_manager.new_scene_name.clone();
                            
                            log_system.add_log(format!("✅ Scene '{}' created successfully!", scene_manager.new_scene_name));
                            scene_manager.scene_creator_open = false;
                            scene_manager.new_scene_name.clear();
                        } else {
                            log_system.add_log("❌ Scene name cannot be empty!".to_string());
                        }
                    }
                    
                    if ui.button("❌ Cancel").clicked() {
                        scene_manager.scene_creator_open = false;
                        scene_manager.new_scene_name.clear();
                    }
                });
            });
    }
}

// Окно тренировки с графиками
fn show_training_window(contexts: &mut EguiContexts, training_system: &mut TrainingSystem) {
    if training_system.training_window_open {
        egui::Window::new("🚀 AI Training Dashboard")
            .default_size([600.0, 500.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Training Progress");
                ui.separator();
                
                // Статус тренировки
                let status = if training_system.is_training { "🟢 Training" } else { "🔴 Stopped" };
                ui.label(format!("Status: {}", status));
                
                ui.horizontal(|ui| {
                    ui.label(format!("Epoch: {}/{}", training_system.current_epoch, training_system.total_epochs));
                    ui.separator();
                    ui.label(format!("Step: {}/{}", training_system.current_step, training_system.steps_in_epoch));
                });
                
                // Прогресс бары
                let epoch_progress = training_system.current_epoch as f32 / training_system.total_epochs as f32;
                let step_progress = training_system.current_step as f32 / training_system.steps_in_epoch as f32;
                
                ui.add(egui::ProgressBar::new(epoch_progress).text("Epoch Progress"));
                ui.add(egui::ProgressBar::new(step_progress).text("Step Progress"));
                
                ui.separator();
                
                // Настройки тренировки
                ui.heading("⚙️ Training Settings");
                ui.horizontal(|ui| {
                    ui.label("Learning Rate:");
                    ui.add(egui::DragValue::new(&mut training_system.learning_rate).speed(0.0001).clamp_range(0.0001..=0.1));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Total Epochs:");
                    ui.add(egui::DragValue::new(&mut training_system.total_epochs).speed(1).clamp_range(1..=100));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Steps per Epoch:");
                    ui.add(egui::DragValue::new(&mut training_system.steps_in_epoch).speed(10).clamp_range(10..=1000));
                });
                
                ui.separator();
                
                // Метрики (имитация)
                ui.heading("📊 Metrics");
                ui.label(format!("Current Loss: {:.4}", if training_system.loss_history.is_empty() { 0.0 } else { *training_system.loss_history.last().unwrap() }));
                ui.label(format!("Average Reward: {:.2}", if training_system.reward_history.is_empty() { 0.0 } else { *training_system.reward_history.last().unwrap() }));
                ui.label(format!("Accuracy: {:.1}%", if training_system.accuracy_history.is_empty() { 0.0 } else { *training_system.accuracy_history.last().unwrap() * 100.0 }));
                
                // Простой график (линии)
                if !training_system.loss_history.is_empty() {
                    ui.separator();
                    ui.label("📈 Loss History");
                    let points: Vec<[f64; 2]> = training_system.loss_history.iter()
                        .enumerate()
                        .map(|(i, &loss)| [i as f64, loss as f64])
                        .collect();
                    
                    if points.len() > 1 {
                        Plot::new("loss_plot")
                            .height(150.0)
                            .show(ui, |plot_ui| {
                                plot_ui.line(Line::new(points).name("Loss"));
                            });
                    }
                }
                
                ui.separator();
                
                // Кнопки управления
                ui.horizontal(|ui| {
                    if ui.button("🔄 Reset Training").clicked() {
                        training_system.current_epoch = 0;
                        training_system.current_step = 0;
                        training_system.loss_history.clear();
                        training_system.reward_history.clear();
                        training_system.accuracy_history.clear();
                        println!("🔄 Training reset");
                    }
                    
                    if ui.button("💾 Save Model").clicked() {
                        println!("💾 Saving trained model...");
                    }
                    
                    if ui.button("❌ Close").clicked() {
                        training_system.training_window_open = false;
                    }
                });
            });
    }
}

// Система выполнения промптов
fn prompt_execution_system(
    mut commands: Commands,
    mut prompt_execution: ResMut<PromptExecution>,
    mut log_system: ResMut<LogSystem>,
    agent_prompts: Res<AgentPrompts>,
    mut query: Query<(Entity, &mut Agent, &Transform, &mut Movement)>,
    time: Res<Time>,
    ollama: Res<OllamaConnection>,
    mut arena_state: ResMut<ArenaState>,
) {
    if prompt_execution.running {
        // Анализируем промпты и изменяем поведение агентов
        for (entity, mut agent, transform, mut movement) in query.iter_mut() {
            if let Some(prompt) = agent_prompts.prompts.get(&agent.id) {
                // Анализируем содержимое промпта (поддержка русского и английского)
                let prompt_lower = prompt.to_lowercase();
                
                if prompt_lower.contains("ключ") || prompt_lower.contains("key") || 
                   prompt_lower.contains("найд") || prompt_lower.contains("find") ||
                   prompt_lower.contains("поиск") || prompt_lower.contains("search") {
                    agent.status = "searching_key".to_string();
                    log_system.add_log(format!("🔍 Agent {} executing key search prompt", agent.name));
                    
                    // Движение для поиска ключа - в углы арены
                    let search_positions = vec![
                        Vec3::new(-8.0, 0.5, -6.0),
                        Vec3::new(8.0, 0.5, -6.0),
                        Vec3::new(-8.0, 0.5, 6.0),
                        Vec3::new(8.0, 0.5, 6.0),
                    ];
                    let target_idx = (agent.id.len() % search_positions.len()) as usize;
                    movement.target_position = Some(search_positions[target_idx]);
                    movement.speed = 3.0;
                    log_system.add_log(format!("🔍 Agent {} searching for key at {:?}", agent.name, search_positions[target_idx]));
                    
                    // Добавляем анимацию поиска
                    commands.entity(entity).insert(AgentAnimation {
                        animation_type: "search".to_string(),
                        duration: 3.0,
                        start_time: time.elapsed_seconds(),
                    });
                    
                } else if prompt_lower.contains("атак") || prompt_lower.contains("attack") || 
                          prompt_lower.contains("убей") || prompt_lower.contains("kill") ||
                          prompt_lower.contains("воюй") || prompt_lower.contains("fight") ||
                          prompt_lower.contains("бей") || prompt_lower.contains("hit") {
                    agent.status = "attacking".to_string();
                    log_system.add_log(format!("⚔️ Agent {} executing attack prompt", agent.name));
                    
                    // Находим ближайшего врага для атаки
                    let current_position = transform.translation;
                    let mut closest_enemy_pos = None;
                    let mut closest_distance = f32::MAX;
                    
                    // Ищем врагов в arena_state
                    for (enemy_id, enemy_agent) in &arena_state.agents {
                        if enemy_agent.team != agent.team {
                            // Для упрощения, используем позицию из создания агентов
                            let enemy_pos = match enemy_id.as_str() {
                                "red_gladiator" => Vec3::new(-5.0, 0.5, 0.0),
                                "blue_warrior" => Vec3::new(5.0, 0.5, 0.0),
                                "red_scout" => Vec3::new(0.0, 0.5, 5.0),
                                _ => Vec3::ZERO,
                            };
                            
                            let distance = current_position.distance(enemy_pos);
                            if distance < closest_distance {
                                closest_distance = distance;
                                closest_enemy_pos = Some(enemy_pos);
                            }
                        }
                    }
                    
                    // Устанавливаем цель движения к ближайшему врагу
                    if let Some(target) = closest_enemy_pos {
                        movement.target_position = Some(target);
                        movement.speed = 5.0; // Увеличиваем скорость для атаки
                        log_system.add_log(format!("🎯 Agent {} moving to attack at {:?}", agent.name, target));
                    }
                    
                    // Добавляем анимацию атаки
                    commands.entity(entity).insert(AgentAnimation {
                        animation_type: "spin".to_string(),
                        duration: 2.0,
                        start_time: time.elapsed_seconds(),
                    });
                    
                } else if prompt_lower.contains("защит") || prompt_lower.contains("defend") || 
                          prompt_lower.contains("оборон") || prompt_lower.contains("guard") ||
                          prompt_lower.contains("прикрой") || prompt_lower.contains("cover") ||
                          prompt_lower.contains("блокир") || prompt_lower.contains("block") {
                    agent.status = "defending".to_string();
                    log_system.add_log(format!("🛡️ Agent {} executing defense prompt", agent.name));
                    
                    // Для защиты двигаемся к центру арены
                    movement.target_position = Some(Vec3::new(0.0, 0.5, 0.0));
                    movement.speed = 2.0;
                    log_system.add_log(format!("🏛️ Agent {} moving to defensive position", agent.name));
                    
                    // Добавляем анимацию защиты
                    commands.entity(entity).insert(AgentAnimation {
                        animation_type: "pulse".to_string(),
                        duration: 2.5,
                        start_time: time.elapsed_seconds(),
                    });
                    
                } else if prompt_lower.contains("двиг") || prompt_lower.contains("move") || 
                          prompt_lower.contains("иди") || prompt_lower.contains("go") ||
                          prompt_lower.contains("беги") || prompt_lower.contains("run") ||
                          prompt_lower.contains("ходи") || prompt_lower.contains("walk") {
                    agent.status = "moving".to_string();
                    log_system.add_log(format!("🏃 Agent {} executing movement prompt", agent.name));
                    
                    // Случайное движение в арене
                    let random_x = (rand::random::<f32>() - 0.5) * 16.0;
                    let random_z = (rand::random::<f32>() - 0.5) * 12.0;
                    movement.target_position = Some(Vec3::new(random_x, 0.5, random_z));
                    movement.speed = 4.0;
                    log_system.add_log(format!("🎯 Agent {} moving to random position", agent.name));
                    
                    // Добавляем анимацию движения
                    commands.entity(entity).insert(AgentAnimation {
                        animation_type: "bounce".to_string(),
                        duration: 1.5,
                        start_time: time.elapsed_seconds(),
                    });
                    
                } else {
                    agent.status = "idle".to_string();
                    log_system.add_log(format!("💭 Agent {} analyzing prompt: '{}'", agent.name, prompt));
                }
                
                // Если подключены к Ollama, отправляем реальный запрос
                if ollama.connected {
                    log_system.add_log(format!("🤖 Sending prompt to Ollama: '{}'", prompt));
                    
                    let request = OllamaRequest {
                        model: ollama.model.clone(),
                        prompt: format!("You are {} in a 3D arena. Task: {}. Respond with ONE action: move, attack, defend, or search.", agent.name, prompt),
                        stream: false,
                    };
                    
                    // Отправляем запрос к Ollama
                    let generate_url = format!("{}/api/generate", ollama.url);
                    log_system.add_log(format!("🌐 Sending request to: {}", generate_url));
                    
                    match ollama.runtime.block_on(async {
                        ollama.client.post(&generate_url)
                            .json(&request)
                            .timeout(std::time::Duration::from_secs(10))
                            .send()
                            .await
                    }) {
                        Ok(response) => {
                            log_system.add_log(format!("📡 HTTP Status: {}", response.status()));
                            
                            if response.status().is_success() {
                                match ollama.runtime.block_on(async {
                                    response.text().await
                                }) {
                                    Ok(response_text) => {
                                        log_system.add_log(format!("📝 Raw response: {}", response_text.chars().take(200).collect::<String>()));
                                        
                                        // Парсим как один JSON объект
                                        match serde_json::from_str::<OllamaResponse>(&response_text) {
                                            Ok(ollama_response) => {
                                                let final_response = ollama_response.response;
                                                log_system.add_log(format!("🧠 Ollama response for {}: '{}'", agent.name, final_response));
                                                
                                                // Анализируем ответ от Ollama
                                                let response_lower = final_response.to_lowercase();
                                                if response_lower.contains("attack") || response_lower.contains("атак") {
                                                    agent.status = "attacking".to_string();
                                                } else if response_lower.contains("defend") || response_lower.contains("защит") {
                                                    agent.status = "defending".to_string();
                                                } else if response_lower.contains("search") || response_lower.contains("ключ") {
                                                    agent.status = "searching_key".to_string();
                                                } else if response_lower.contains("move") || response_lower.contains("двиг") {
                                                    agent.status = "moving".to_string();
                                                } else {
                                                    agent.status = "idle".to_string();
                                                }
                                                
                                                log_system.add_log(format!("✅ Agent {} new status: {}", agent.name, agent.status));
                                                
                                                // Обновляем статус в ArenaState для GUI
                                                if let Some(arena_agent) = arena_state.agents.get_mut(&agent.id) {
                                                    arena_agent.status = agent.status.clone();
                                                }
                                            }
                                            Err(e) => {
                                                log_system.add_log(format!("❌ JSON parse error: {}", e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log_system.add_log(format!("❌ Error reading response text: {}", e));
                                    }
                                }
                            } else {
                                log_system.add_log(format!("❌ HTTP Error: {}", response.status()));
                            }
                        }
                        Err(e) => {
                            log_system.add_log(format!("❌ Request failed: {}", e));
                        }
                    }
                }
                
                // Записываем результат выполнения
                prompt_execution.results.insert(
                    agent.id.clone(), 
                    format!("Status changed to: {}", agent.status)
                );
            }
        }
        
        log_system.add_log("✅ Prompt execution completed for all agents".to_string());
        prompt_execution.running = false;
    }
}

// Ресурс для отслеживания последней сгенерированной сцены
#[derive(Resource)]
pub struct LastGeneratedScene {
    pub scene_name: String,
}

impl Default for LastGeneratedScene {
    fn default() -> Self {
        Self {
            scene_name: "".to_string(),
        }
    }
}

// Система генерации сцен (исправлена)
fn scene_generation_system(
    mut last_generated: ResMut<LastGeneratedScene>,
    scene_manager: Res<SceneManager>,
    mut log_system: ResMut<LogSystem>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Проверяем, нужно ли генерировать новую сцену
    if scene_manager.current_scene != last_generated.scene_name {
        log_system.add_log(format!("🎬 Generating scene: '{}'", scene_manager.current_scene));
        
        // Обновляем последнюю сгенерированную сцену
        last_generated.scene_name = scene_manager.current_scene.clone();
        
        // Создаем новую сцену в зависимости от выбранной
        match scene_manager.current_scene.as_str() {
            "Arena Basic" => {
                log_system.add_log("🏟️ Creating basic arena...".to_string());
                // Код создания базовой арены уже есть в setup_arena
            }
            "Maze Challenge" => {
                log_system.add_log("🌀 Creating maze challenge...".to_string());
                // Создаем дополнительные препятствия для лабиринта
                for x in (-5..=5).step_by(2) {
                    for z in (-5..=5).step_by(2) {
                        if x != 0 || z != 0 {
                            commands.spawn(PbrBundle {
                                mesh: meshes.add(shape::Box::new(1.0, 2.0, 1.0).into()),
                                material: materials.add(StandardMaterial {
                                    base_color: Color::rgb(0.6, 0.4, 0.2),
                                    ..default()
                                }),
                                transform: Transform::from_translation(Vec3::new(x as f32, 1.0, z as f32)),
                                ..default()
                            });
                        }
                    }
                }
            }
            "Battle Royale" => {
                log_system.add_log("⚔️ Creating battle royale arena...".to_string());
                // Создаем случайные препятствия
                for _ in 0..10 {
                    let mut rng = rand::thread_rng();
                    let x = (rng.gen::<f32>() - 0.5) * 16.0;
                    let z = (rng.gen::<f32>() - 0.5) * 16.0;
                    commands.spawn(PbrBundle {
                        mesh: meshes.add(shape::Cylinder { radius: 1.0, height: 3.0, resolution: 12, segments: 1 }.into()),
                        material: materials.add(StandardMaterial {
                            base_color: Color::rgb(0.8, 0.6, 0.4),
                            ..default()
                        }),
                        transform: Transform::from_translation(Vec3::new(x, 1.5, z)),
                        ..default()
                    });
                }
            }
            "Capture the Flag" => {
                log_system.add_log("🏁 Creating capture the flag arena...".to_string());
                // Создаем базы команд
                for (pos, color) in [
                    (Vec3::new(-8.0, 1.0, 0.0), Color::RED),
                    (Vec3::new(8.0, 1.0, 0.0), Color::BLUE),
                ] {
                    commands.spawn(PbrBundle {
                        mesh: meshes.add(shape::Box::new(2.0, 4.0, 2.0).into()),
                        material: materials.add(StandardMaterial {
                            base_color: color,
                            ..default()
                        }),
                        transform: Transform::from_translation(pos),
                        ..default()
                    });
                }
            }
            _ => {
                log_system.add_log(format!("🎨 Creating custom scene: '{}'", scene_manager.current_scene));
                // Создаем пользовательскую сцену
                for _ in 0..5 {
                    let mut rng = rand::thread_rng();
                    let x = (rng.gen::<f32>() - 0.5) * 14.0;
                    let z = (rng.gen::<f32>() - 0.5) * 14.0;
                    commands.spawn(PbrBundle {
                        mesh: meshes.add(shape::Box::new(1.0, 1.0, 1.0).into()),
                        material: materials.add(StandardMaterial {
                            base_color: Color::rgb(0.5, 0.5, 0.8),
                            ..default()
                        }),
                        transform: Transform::from_translation(Vec3::new(x, 0.5, z)),
                        ..default()
                    });
                }
            }
        }
        
        log_system.add_log(format!("✅ Scene '{}' generated successfully!", scene_manager.current_scene));
    }
}

// Система подключения к Ollama
fn ollama_connection_system(
    mut ollama: ResMut<OllamaConnection>,
    mut log_system: ResMut<LogSystem>,
) {
    // Проверяем подключение только если не подключены
    if !ollama.connected && ollama.status == "Connecting..." {
        log_system.add_log("🔄 Trying to connect to Ollama...".to_string());
        
        // Пытаемся получить список моделей
        let models_url = format!("{}/api/tags", ollama.url);
        
        match ollama.runtime.block_on(async {
            ollama.client.get(&models_url).send().await
        }) {
            Ok(response) => {
                if response.status().is_success() {
                    match ollama.runtime.block_on(async {
                        response.json::<OllamaModelsResponse>().await
                    }) {
                        Ok(models_response) => {
                            ollama.available_models = models_response.models
                                .into_iter()
                                .map(|m| m.name)
                                .collect();
                            
                            ollama.connected = true;
                            ollama.status = "Connected".to_string();
                            
                            log_system.add_log(format!("✅ Connected to Ollama! Found {} models", ollama.available_models.len()));
                            for model in &ollama.available_models {
                                log_system.add_log(format!("  📦 Model: {}", model));
                            }
                        }
                        Err(e) => {
                            ollama.status = "Error parsing models".to_string();
                            log_system.add_log(format!("❌ Error parsing models: {}", e));
                        }
                    }
                } else {
                    ollama.status = format!("HTTP {}", response.status());
                    log_system.add_log(format!("❌ HTTP error: {}", response.status()));
                }
            }
            Err(e) => {
                ollama.status = "Connection failed".to_string();
                log_system.add_log(format!("❌ Connection failed: {}", e));
            }
        }
    }
}

// Система анимации агентов с частями тела
fn agent_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut AgentAnimation), With<Agent>>,
    mut body_parts_query: Query<(&mut Transform, &mut AgentBodyPart), (With<AgentBodyPart>, Without<Agent>)>,
    children_query: Query<&Children>,
    mut log_system: ResMut<LogSystem>,
) {
    for (entity, mut transform, animation) in query.iter_mut() {
        let elapsed = time.elapsed_seconds() - animation.start_time;
        
        if elapsed >= animation.duration {
            // Анимация завершена
            commands.entity(entity).remove::<AgentAnimation>();
            log_system.add_log(format!("✅ Animation '{}' completed", animation.animation_type));
            continue;
        }
        
        let progress = elapsed / animation.duration;
        
        match animation.animation_type.as_str() {
            "bounce" => {
                let bounce_height = (progress * std::f32::consts::PI * 2.0).sin() * 0.5;
                transform.translation.y = 0.5 + bounce_height;
            }
            "spin" => {
                let rotation_speed = progress * std::f32::consts::PI * 4.0;
                transform.rotation = Quat::from_rotation_y(rotation_speed);
            }
            "pulse" => {
                let scale = 1.0 + (progress * std::f32::consts::PI * 2.0).sin() * 0.2;
                transform.scale = Vec3::splat(scale);
            }
            "search" => {
                // Движение по кругу при поиске
                let radius = 2.0;
                let angle = progress * std::f32::consts::PI * 2.0;
                transform.translation.x = angle.cos() * radius;
                transform.translation.z = angle.sin() * radius;
            }
            "walking" => {
                // Анимация ходьбы - анимируем части тела
                if let Ok(children) = children_query.get(entity) {
                    for child in children.iter() {
                        if let Ok((mut body_transform, mut body_part)) = body_parts_query.get_mut(*child) {
                            let walk_speed = 8.0; // скорость анимации ходьбы
                            let walk_time = time.elapsed_seconds() * walk_speed;
                            
                            match body_part.part_type.as_str() {
                                "left_arm" => {
                                    body_part.animation_offset.z = walk_time.sin() * 0.15;
                                    body_part.animation_offset.x = walk_time.cos() * 0.05;
                                }
                                "right_arm" => {
                                    body_part.animation_offset.z = -walk_time.sin() * 0.15;
                                    body_part.animation_offset.x = -walk_time.cos() * 0.05;
                                }
                                "left_leg" => {
                                    body_part.animation_offset.z = -walk_time.sin() * 0.2;
                                    body_part.animation_offset.y = (walk_time.sin() * 0.1).max(0.0);
                                }
                                "right_leg" => {
                                    body_part.animation_offset.z = walk_time.sin() * 0.2;
                                    body_part.animation_offset.y = (-walk_time.sin() * 0.1).max(0.0);
                                }
                                "head" => {
                                    body_part.animation_offset.y = (walk_time * 2.0).sin() * 0.02;
                                }
                                _ => {}
                            }
                            
                            // Применяем анимационный оффсет
                            body_transform.translation = body_part.relative_position + body_part.animation_offset;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// Система анимации ходьбы при движении
fn walking_animation_system(
    time: Res<Time>,
    mut agent_query: Query<(Entity, &Movement, &mut Transform), (With<Agent>, With<AgentVisual>)>,
    mut body_parts_query: Query<(&mut Transform, &mut AgentBodyPart), (With<AgentBodyPart>, Without<Agent>)>,
    children_query: Query<&Children>,
    _commands: Commands,
) {
    for (entity, movement, _agent_transform) in agent_query.iter_mut() {
        let is_moving = movement.velocity.length() > 0.1;
        
        if is_moving {
            // Добавляем анимацию ходьбы если её нет
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    if let Ok((mut body_transform, mut body_part)) = body_parts_query.get_mut(*child) {
                        let walk_speed = 8.0 * movement.velocity.length(); // скорость анимации зависит от скорости движения
                        let walk_time = time.elapsed_seconds() * walk_speed;
                        
                        match body_part.part_type.as_str() {
                            "left_arm" => {
                                body_part.animation_offset.z = walk_time.sin() * 0.15;
                                body_part.animation_offset.x = walk_time.cos() * 0.05;
                            }
                            "right_arm" => {
                                body_part.animation_offset.z = -walk_time.sin() * 0.15;
                                body_part.animation_offset.x = -walk_time.cos() * 0.05;
                            }
                            "left_leg" => {
                                body_part.animation_offset.z = -walk_time.sin() * 0.2;
                                body_part.animation_offset.y = (walk_time.sin() * 0.1).max(0.0);
                            }
                            "right_leg" => {
                                body_part.animation_offset.z = walk_time.sin() * 0.2;
                                body_part.animation_offset.y = (-walk_time.sin() * 0.1).max(0.0);
                            }
                            "head" => {
                                body_part.animation_offset.y = (walk_time * 2.0).sin() * 0.02;
                            }
                            _ => {}
                        }
                        
                        // Применяем анимационный оффсет
                        body_transform.translation = body_part.relative_position + body_part.animation_offset;
                    }
                }
            }
        } else {
            // Обнуляем анимационные оффсеты когда агент стоит
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    if let Ok((mut body_transform, mut body_part)) = body_parts_query.get_mut(*child) {
                        // Плавно возвращаем в исходное положение
                        body_part.animation_offset = body_part.animation_offset.lerp(Vec3::ZERO, time.delta_seconds() * 5.0);
                        body_transform.translation = body_part.relative_position + body_part.animation_offset;
                    }
                }
            }
        }
    }
}

// Система визуальных эффектов
fn agent_effects_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AgentEffect), With<Agent>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, effect) in query.iter_mut() {
        let elapsed = time.elapsed_seconds() - effect.start_time;
        
        if elapsed >= effect.duration {
            commands.entity(entity).remove::<AgentEffect>();
            continue;
        }
        
        let progress = elapsed / effect.duration;
        
        match effect.effect_type.as_str() {
            "glow" => {
                // Эффект свечения
                let _glow_intensity = (progress * std::f32::consts::PI).sin() * effect.intensity;
                // Здесь можно добавить изменение материала для свечения
            }
            "damage" => {
                // Красный эффект при получении урона
                if progress < 0.5 {
                    // Мигание красным
                }
            }
            _ => {}
        }
    }
}

// Ресурс для отслеживания последней сцены
#[derive(Resource)]
pub struct LastSceneState {
    pub last_scene: String,
}

impl Default for LastSceneState {
    fn default() -> Self {
        Self {
            last_scene: "".to_string(),
        }
    }
}

// Система переходов между сценами (исправлена)
fn scene_transition_system(
    mut last_scene_state: ResMut<LastSceneState>,
    scene_manager: Res<SceneManager>,
    mut log_system: ResMut<LogSystem>,
) {
    // Проверяем, действительно ли изменилась сцена
    if scene_manager.current_scene != last_scene_state.last_scene {
        log_system.add_log(format!("🎬 Scene transition: '{}' -> '{}'", 
            last_scene_state.last_scene, scene_manager.current_scene));
        
        // Обновляем состояние последней сцены
        last_scene_state.last_scene = scene_manager.current_scene.clone();
        
        log_system.add_log("✨ Scene transition completed".to_string());
    }
}

// Окно настройки внешнего вида агентов (расширенное)
fn show_appearance_window(contexts: &mut EguiContexts, agent_appearance: &mut AgentAppearance) {
    if agent_appearance.appearance_window_open {
        egui::Window::new("🎨 Agent Appearance Editor")
            .default_size([450.0, 400.0])
            .resizable(true)
            .collapsible(true)
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Customize Agent Appearance");
                ui.separator();
                
                // Выбор агента
                ui.label("Select Agent:");
                egui::ComboBox::from_label("Agent")
                    .selected_text(&agent_appearance.selected_agent_for_appearance)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut agent_appearance.selected_agent_for_appearance, "red_gladiator".to_string(), "🔴 Red Gladiator");
                        ui.selectable_value(&mut agent_appearance.selected_agent_for_appearance, "blue_warrior".to_string(), "🔵 Blue Warrior");
                        ui.selectable_value(&mut agent_appearance.selected_agent_for_appearance, "red_scout".to_string(), "🔴 Red Scout");
                    });
                
                ui.separator();
                
                // Выбор формы
                ui.label("Shape:");
                let current_shape = agent_appearance.agent_shapes.get(&agent_appearance.selected_agent_for_appearance).unwrap_or(&"humanoid".to_string()).clone();
                
                for shape in &agent_appearance.available_shapes {
                    let shape_name = match shape.as_str() {
                        "humanoid" => "🤖 Humanoid",
                        "robot" => "🦾 Robot",
                        "sphere" => "⚽ Sphere",
                        "cube" => "🎲 Cube",
                        "cylinder" => "🥫 Cylinder",
                        _ => shape,
                    };
                    
                    if ui.selectable_label(current_shape == *shape, shape_name).clicked() {
                        agent_appearance.agent_shapes.insert(agent_appearance.selected_agent_for_appearance.clone(), shape.clone());
                    }
                }
                
                ui.separator();
                
                // Выбор цвета
                ui.label("Color:");
                let current_color = agent_appearance.agent_colors.get(&agent_appearance.selected_agent_for_appearance).unwrap_or(&[0.8, 0.2, 0.2]).clone();
                let mut color_array = current_color;
                
                ui.horizontal(|ui| {
                    ui.label("Red:");
                    let red_val = color_array[0];
                    ui.add(egui::Slider::new(&mut color_array[0], 0.0..=1.0).text(format!("{:.2}", red_val)));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Green:");
                    let green_val = color_array[1];
                    ui.add(egui::Slider::new(&mut color_array[1], 0.0..=1.0).text(format!("{:.2}", green_val)));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Blue:");
                    let blue_val = color_array[2];
                    ui.add(egui::Slider::new(&mut color_array[2], 0.0..=1.0).text(format!("{:.2}", blue_val)));
                });
                
                agent_appearance.agent_colors.insert(agent_appearance.selected_agent_for_appearance.clone(), color_array);
                
                // Быстрые цвета
                ui.label("Quick Colors:");
                ui.horizontal(|ui| {
                    if ui.button("🔴 Red").clicked() {
                        agent_appearance.agent_colors.insert(agent_appearance.selected_agent_for_appearance.clone(), [0.8, 0.2, 0.2]);
                    }
                    if ui.button("🔵 Blue").clicked() {
                        agent_appearance.agent_colors.insert(agent_appearance.selected_agent_for_appearance.clone(), [0.2, 0.2, 0.8]);
                    }
                    if ui.button("🟢 Green").clicked() {
                        agent_appearance.agent_colors.insert(agent_appearance.selected_agent_for_appearance.clone(), [0.2, 0.8, 0.2]);
                    }
                    if ui.button("🟡 Yellow").clicked() {
                        agent_appearance.agent_colors.insert(agent_appearance.selected_agent_for_appearance.clone(), [0.8, 0.8, 0.2]);
                    }
                    if ui.button("🟣 Purple").clicked() {
                        agent_appearance.agent_colors.insert(agent_appearance.selected_agent_for_appearance.clone(), [0.8, 0.2, 0.8]);
                    }
                });
                
                ui.separator();
                
                // Кнопки
                ui.horizontal(|ui| {
                    if ui.button("✅ Apply").clicked() {
                        // Применить изменения внешнего вида
                        println!("🎨 Applied appearance changes for {}", agent_appearance.selected_agent_for_appearance);
                    }
                    
                    if ui.button("🔄 Reset").clicked() {
                        // Сбросить к значениям по умолчанию
                        agent_appearance.agent_shapes.insert(agent_appearance.selected_agent_for_appearance.clone(), "humanoid".to_string());
                        agent_appearance.agent_colors.insert(agent_appearance.selected_agent_for_appearance.clone(), [0.8, 0.2, 0.2]);
                    }
                    
                    if ui.button("❌ Close").clicked() {
                        agent_appearance.appearance_window_open = false;
                    }
                });
            });
    }
}

// Окно настроек движения
fn show_movement_settings(contexts: &mut EguiContexts, movement_settings: &mut MovementSettings) {
    if movement_settings.settings_window_open {
        egui::Window::new("⚙️ Movement Settings")
            .default_size([350.0, 250.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Agent Movement Settings");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Speed Multiplier:");
                    let speed_val = movement_settings.movement_speed;
                    ui.add(egui::Slider::new(&mut movement_settings.movement_speed, 0.1..=5.0)
                        .text(format!("{:.1}x", speed_val)));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Smoothness:");
                    let smooth_val = movement_settings.movement_smoothness;
                    ui.add(egui::Slider::new(&mut movement_settings.movement_smoothness, 1.0..=10.0)
                        .text(format!("{:.1}", smooth_val)));
                });
                
                ui.separator();
                
                ui.label("Quick Presets:");
                ui.horizontal(|ui| {
                    if ui.button("🐌 Slow").clicked() {
                        movement_settings.movement_speed = 0.5;
                        movement_settings.movement_smoothness = 2.0;
                    }
                    if ui.button("⚡ Normal").clicked() {
                        movement_settings.movement_speed = 1.0;
                        movement_settings.movement_smoothness = 5.0;
                    }
                    if ui.button("🚀 Fast").clicked() {
                        movement_settings.movement_speed = 2.0;
                        movement_settings.movement_smoothness = 8.0;
                    }
                });
                
                ui.separator();
                
                ui.checkbox(&mut movement_settings.show_movement_lines, "Show Movement Lines");
                ui.checkbox(&mut movement_settings.show_attack_range, "Show Attack Range");
                ui.checkbox(&mut movement_settings.agent_selection_enabled, "Enable Agent Selection");
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("✅ Apply").clicked() {
                        // Настройки применяются автоматически
                        println!("⚙️ Movement settings applied");
                    }
                    
                    if ui.button("🔄 Reset").clicked() {
                        movement_settings.movement_speed = 3.0;
                        movement_settings.movement_smoothness = 5.0;
                        movement_settings.show_movement_lines = true;
                        movement_settings.show_attack_range = false;
                        println!("🔄 Movement settings reset");
                    }
                    
                    if ui.button("❌ Close").clicked() {
                        movement_settings.settings_window_open = false;
                    }
                });
            });
    }
}

// Функция применения темы
fn apply_theme(contexts: &mut EguiContexts, theme_settings: &ThemeSettings) {
    let ctx = contexts.ctx_mut();
    
    if theme_settings.dark_mode {
        ctx.set_visuals(egui::Visuals::dark());
    } else {
        ctx.set_visuals(egui::Visuals::light());
    }
    
    // Применяем пользовательские цвета
    let mut visuals = ctx.style().visuals.clone();
    visuals.selection.bg_fill = egui::Color32::from_rgb(
        (theme_settings.accent_color[0] * 255.0) as u8,
        (theme_settings.accent_color[1] * 255.0) as u8,
        (theme_settings.accent_color[2] * 255.0) as u8,
    );
    visuals.panel_fill = visuals.panel_fill.gamma_multiply(theme_settings.background_alpha);
    
    ctx.set_visuals(visuals);
}

// Окно редактора арены
fn show_arena_editor(contexts: &mut EguiContexts, arena_drag_drop: &mut ArenaDragDrop) {
    if arena_drag_drop.arena_editor_open {
        egui::Window::new("🏗️ Arena Editor")
            .default_size([400.0, 500.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Drag & Drop Arena Editor");
                ui.separator();
                
                // Панель объектов
                ui.label("Available Objects:");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for object in &arena_drag_drop.available_objects {
                        let selected = arena_drag_drop.selected_object == *object;
                        if ui.selectable_label(selected, object).clicked() {
                            arena_drag_drop.selected_object = object.clone();
                        }
                    }
                });
                
                ui.separator();
                
                // Drag & Drop область
                ui.label("Drag & Drop Area:");
                let (response, painter) = ui.allocate_painter(egui::Vec2::new(350.0, 200.0), egui::Sense::drag());
                
                // Рисуем фон
                painter.rect_filled(
                    response.rect,
                    egui::Rounding::same(5.0),
                    egui::Color32::from_gray(40),
                );
                
                // Drag & Drop логика
                if response.drag_started() {
                    arena_drag_drop.dragging = true;
                    if let Some(pos) = response.interact_pointer_pos() {
                        arena_drag_drop.drag_start_pos = pos.to_vec2();
                    }
                }
                
                if response.drag_released() && arena_drag_drop.dragging {
                    if let Some(pos) = response.interact_pointer_pos() {
                        // Конвертируем 2D позицию в 3D
                        let rel_pos = pos - response.rect.min;
                        let world_pos = Vec3::new(
                            (rel_pos.x / 350.0 - 0.5) * 20.0, // Масштаб арены
                            0.5,
                            (rel_pos.y / 200.0 - 0.5) * 20.0,
                        );
                        
                        // Добавляем объект
                        arena_drag_drop.placed_objects.push(PlacedObject {
                            object_type: arena_drag_drop.selected_object.clone(),
                            position: world_pos,
                            rotation: 0.0,
                            scale: Vec3::ONE,
                        });
                        
                        println!("🏗️ Placed {} at {:?}", arena_drag_drop.selected_object, world_pos);
                    }
                    arena_drag_drop.dragging = false;
                }
                
                // Отображаем размещенные объекты
                for (_i, obj) in arena_drag_drop.placed_objects.iter().enumerate() {
                    let screen_pos = egui::pos2(
                        response.rect.min.x + (obj.position.x / 20.0 + 0.5) * 350.0,
                        response.rect.min.y + (obj.position.z / 20.0 + 0.5) * 200.0,
                    );
                    
                    painter.circle_filled(screen_pos, 5.0, egui::Color32::YELLOW);
                    painter.text(
                        screen_pos + egui::vec2(8.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        format!("{}", obj.object_type),
                        egui::FontId::default(),
                        egui::Color32::WHITE,
                    );
                }
                
                ui.separator();
                
                // Управление
                ui.horizontal(|ui| {
                    if ui.button("🗑️ Clear All").clicked() {
                        arena_drag_drop.placed_objects.clear();
                    }
                    
                    if ui.button("💾 Save Layout").clicked() {
                        println!("💾 Arena layout saved!");
                    }
                    
                    if ui.button("📂 Load Layout").clicked() {
                        println!("📂 Arena layout loaded!");
                    }
                });
                
                ui.separator();
                
                if ui.button("❌ Close").clicked() {
                    arena_drag_drop.arena_editor_open = false;
                }
            });
    }
}

// Окно настроек темы
fn show_theme_settings(contexts: &mut EguiContexts, theme_settings: &mut ThemeSettings) {
    if theme_settings.theme_window_open {
        egui::Window::new("🎨 Theme Settings")
            .default_size([300.0, 250.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("UI Theme Settings");
                ui.separator();
                
                ui.checkbox(&mut theme_settings.dark_mode, "Dark Mode");
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Accent Color:");
                    ui.color_edit_button_rgb(&mut theme_settings.accent_color);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Background Alpha:");
                    ui.add(egui::Slider::new(&mut theme_settings.background_alpha, 0.1..=1.0));
                });
                
                ui.separator();
                
                ui.label("Theme Presets:");
                ui.horizontal(|ui| {
                    if ui.button("🌙 Dark Blue").clicked() {
                        theme_settings.dark_mode = true;
                        theme_settings.accent_color = [0.2, 0.6, 1.0];
                        theme_settings.background_alpha = 0.9;
                    }
                    
                    if ui.button("☀️ Light Blue").clicked() {
                        theme_settings.dark_mode = false;
                        theme_settings.accent_color = [0.1, 0.5, 0.9];
                        theme_settings.background_alpha = 0.8;
                    }
                });
                
                ui.horizontal(|ui| {
                    if ui.button("🟢 Green").clicked() {
                        theme_settings.dark_mode = true;
                        theme_settings.accent_color = [0.2, 0.8, 0.3];
                        theme_settings.background_alpha = 0.9;
                    }
                    
                    if ui.button("🔴 Red").clicked() {
                        theme_settings.dark_mode = true;
                        theme_settings.accent_color = [0.9, 0.2, 0.2];
                        theme_settings.background_alpha = 0.9;
                    }
                });
                
                ui.separator();
                
                if ui.button("❌ Close").clicked() {
                    theme_settings.theme_window_open = false;
                }
            });
    }
}

// Окно настроек горячих клавиш
fn show_hotkey_settings(contexts: &mut EguiContexts, hotkey_settings: &mut HotkeySettings) {
    if hotkey_settings.hotkey_window_open {
        egui::Window::new("⌨️ Hotkey Settings")
            .default_size([400.0, 300.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Keyboard Shortcuts");
                ui.separator();
                
                ui.label("Camera Controls:");
                egui::Grid::new("hotkey_grid").show(ui, |ui| {
                    for (action, key) in hotkey_settings.custom_hotkeys.iter_mut() {
                        ui.label(action);
                        ui.text_edit_singleline(key);
                        ui.end_row();
                    }
                });
                
                ui.separator();
                
                ui.label("Instructions:");
                ui.label("• Click on a key field to change the hotkey");
                ui.label("• Use standard key names (W, A, S, D, Space, Ctrl, etc.)");
                ui.label("• Changes apply immediately");
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("🔄 Reset to Defaults").clicked() {
                        hotkey_settings.custom_hotkeys.clear();
                        hotkey_settings.custom_hotkeys.insert("Camera Forward".to_string(), "W".to_string());
                        hotkey_settings.custom_hotkeys.insert("Camera Backward".to_string(), "S".to_string());
                        hotkey_settings.custom_hotkeys.insert("Camera Left".to_string(), "A".to_string());
                        hotkey_settings.custom_hotkeys.insert("Camera Right".to_string(), "D".to_string());
                        hotkey_settings.custom_hotkeys.insert("Camera Up".to_string(), "Space".to_string());
                        hotkey_settings.custom_hotkeys.insert("Camera Down".to_string(), "Ctrl".to_string());
                        hotkey_settings.custom_hotkeys.insert("Toggle Inspector".to_string(), "F12".to_string());
                    }
                    
                    if ui.button("❌ Close").clicked() {
                        hotkey_settings.hotkey_window_open = false;
                    }
                });
            });
    }
}

// Система выделения и перемещения агентов
fn agent_selection_system(
    mut agent_selection: ResMut<AgentSelection>,
    movement_settings: Res<MovementSettings>,
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut contexts: EguiContexts,
    mut agents_query: Query<(Entity, &mut Agent, &mut SelectionOutline, &mut Transform, &mut Movement), With<AgentVisual>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
    windows: Query<&Window>,
) {
    if !movement_settings.agent_selection_enabled {
        return;
    }
    
    // Если UI захватил мышь, не обрабатываем
    if contexts.ctx_mut().wants_pointer_input() {
        return;
    }
    
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();
    
    // Левый клик для выделения
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            let mut closest_agent = None;
            let mut closest_distance = f32::MAX;
            
            // Проверяем каждого агента на клик
            for (_entity, agent, _outline, transform, _movement) in agents_query.iter_mut() {
                // Проектируем 3D позицию агента в экранные координаты
                if let Some(screen_pos) = camera.world_to_viewport(camera_transform, transform.translation) {
                    let distance = screen_pos.distance(cursor_pos);
                    
                    if distance < 50.0 && distance < closest_distance {
                        closest_distance = distance;
                        closest_agent = Some(agent.id.clone());
                    }
                }
            }
            
            // Обновляем выделение
            if let Some(agent_id) = closest_agent {
                if keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight) {
                    // Ctrl+Click - добавляем/убираем из выделения
                    if agent_selection.selected_agents.contains(&agent_id) {
                        agent_selection.selected_agents.retain(|id| id != &agent_id);
                    } else {
                        agent_selection.selected_agents.push(agent_id);
                    }
                } else {
                    // Простой клик - выделяем только этого агента
                    agent_selection.selected_agents.clear();
                    agent_selection.selected_agents.push(agent_id);
                }
            } else {
                // Клик по пустому месту - снимаем выделение
                agent_selection.selected_agents.clear();
            }
        }
    }
    
    // Правый клик для перемещения выделенных агентов
    if mouse_input.just_pressed(MouseButton::Right) && !agent_selection.selected_agents.is_empty() {
        if let Some(cursor_pos) = window.cursor_position() {
            // Преобразуем экранные координаты в мировые
            let ndc = (cursor_pos / Vec2::new(window.width(), window.height())) * 2.0 - Vec2::ONE;
            let ndc = Vec3::new(ndc.x, -ndc.y, 0.0);
            
            // Простое приближение - проецируем на плоскость Y=0.5
            let world_pos = Vec3::new(ndc.x * 10.0, 0.5, ndc.y * 10.0);
            
            // Перемещаем всех выделенных агентов
            for (_entity, agent, _outline, _transform, mut movement) in agents_query.iter_mut() {
                if agent_selection.selected_agents.contains(&agent.id) {
                    movement.target_position = Some(world_pos);
                    println!("🎯 Moving agent {} to {:?}", agent.name, world_pos);
                }
            }
        }
    }
    
    // Обновляем визуальные индикаторы выделения
    for (_entity, agent, mut outline, _transform, _movement) in agents_query.iter_mut() {
        outline.selected = agent_selection.selected_agents.contains(&agent.id);
    }
}

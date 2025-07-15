use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::time::{Duration, SystemTime};
use anyhow::Result;

mod arena;
mod agents;
mod game_state;
mod ui;
mod network;
mod dataset;
mod websocket;

use arena::ArenaPlugin;
use agents::AgentsPlugin;
use game_state::GameStatePlugin;
use ui::UIPlugin;
use network::NetworkPlugin;
use dataset::DatasetPlugin;
use websocket::{WebSocketServer, ArenaState, AgentData, Position, ArenaStatistics};

// Ресурсы для игрового состояния
#[derive(Resource)]
pub struct GameConfig {
    pub arena_size: Vec2,
    pub max_agents: usize,
    pub match_duration: Duration,
    pub tick_rate: f64,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            arena_size: Vec2::new(800.0, 600.0),
            max_agents: 10,
            match_duration: Duration::from_secs(300), // 5 минут
            tick_rate: 60.0,
        }
    }
}

#[derive(Resource)]
pub struct MatchState {
    pub is_running: bool,
    pub start_time: Option<f64>,
    pub current_tick: u64,
    pub match_id: uuid::Uuid,
}

#[derive(Resource)]
pub struct WebSocketResource {
    pub server: WebSocketServer,
}

impl Default for MatchState {
    fn default() -> Self {
        Self {
            is_running: false,
            start_time: None,
            current_tick: 0,
            match_id: uuid::Uuid::new_v4(),
        }
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Menu,
    InGame,
    Paused,
    GameOver,
}

fn main() -> Result<()> {
    env_logger::init();
    
    info!("Запуск Heaven AI Arena Core v0.1.0");
    
    let mut app = App::new();
    
    if std::env::var("HEADLESS").is_ok() {
        // Headless режим - только минимальные плагины
        app.add_plugins(MinimalPlugins);
    } else {
        // Полный режим с GUI
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Heaven AI Arena".to_string(),
                resolution: (1280.0, 720.0).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin);
    }
    
    app
        .add_plugins(ArenaPlugin)
        .add_plugins(AgentsPlugin)
        .add_plugins(GameStatePlugin)
        .add_plugins(UIPlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins(DatasetPlugin)
        .init_resource::<GameConfig>()
        .init_resource::<MatchState>()
        .add_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_input,
            game_loop.run_if(in_state(AppState::InGame)),
            ui_update,
        ))
        .run();
    
    Ok(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameConfig>,
) {
    info!("Инициализация игрового мира");
    
    // Камера
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Освещение
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Пол арены
    let arena_size = config.arena_size;
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane::from_size(arena_size.x))),
        material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // Границы арены
    let wall_height = 2.0;
    let wall_thickness = 0.1;
    
    // Стены
    for (pos, size) in [
        (Vec3::new(0.0, wall_height / 2.0, arena_size.y / 2.0), Vec3::new(arena_size.x, wall_height, wall_thickness)),
        (Vec3::new(0.0, wall_height / 2.0, -arena_size.y / 2.0), Vec3::new(arena_size.x, wall_height, wall_thickness)),
        (Vec3::new(arena_size.x / 2.0, wall_height / 2.0, 0.0), Vec3::new(wall_thickness, wall_height, arena_size.y)),
        (Vec3::new(-arena_size.x / 2.0, wall_height / 2.0, 0.0), Vec3::new(wall_thickness, wall_height, arena_size.y)),
    ] {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(size.x, size.y, size.z))),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()),
            transform: Transform::from_translation(pos),
            ..default()
        });
    }

    info!("Игровой мир инициализирован");
}

fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut match_state: ResMut<MatchState>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        match_state.is_running = !match_state.is_running;
        if match_state.is_running {
            match_state.start_time = Some(SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64());
            match_state.match_id = uuid::Uuid::new_v4();
            info!("Матч начат: {}", match_state.match_id);
        } else {
            info!("Матч приостановлен");
        }
    }
    
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_state.set(AppState::Menu);
    }
    
    if keyboard_input.just_pressed(KeyCode::R) {
        match_state.is_running = false;
        match_state.start_time = None;
        match_state.current_tick = 0;
        match_state.match_id = uuid::Uuid::new_v4();
        info!("Матч сброшен");
    }
}

fn game_loop(
    time: Res<Time>,
    mut match_state: ResMut<MatchState>,
    config: Res<GameConfig>,
) {
    if !match_state.is_running {
        return;
    }
    
    // Обновляем тик
    match_state.current_tick += 1;
    
    // Проверяем время матча
    if let Some(start_time) = match_state.start_time {
        let elapsed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64() - start_time;
            
        if elapsed >= config.match_duration.as_secs_f64() {
            match_state.is_running = false;
            info!("Матч завершен по времени: {}", match_state.match_id);
        }
    }
    
    // Здесь будет логика обновления агентов
    // Вызов agent.decide(state) для каждого агента
    // Обновление физики и коллизий
    // Логирование состояний
}

fn ui_update(
    mut contexts: EguiContexts,
    match_state: Res<MatchState>,
    config: Res<GameConfig>,
) {
    egui::Window::new("Heaven AI Arena")
        .default_size([300.0, 200.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Состояние матча");
            
            ui.horizontal(|ui| {
                ui.label("Статус:");
                if match_state.is_running {
                    ui.colored_label(egui::Color32::GREEN, "Активен");
                } else {
                    ui.colored_label(egui::Color32::RED, "Остановлен");
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Тик:");
                ui.label(match_state.current_tick.to_string());
            });
            
            ui.horizontal(|ui| {
                ui.label("ID матча:");
                ui.label(match_state.match_id.to_string());
            });
            
            ui.separator();
            
            ui.label("Управление:");
            ui.label("SPACE - Старт/Пауза");
            ui.label("R - Сброс");
            ui.label("ESC - Меню");
            
            ui.separator();
            
            ui.label(format!("Размер арены: {}x{}", config.arena_size.x, config.arena_size.y));
            ui.label(format!("Макс. агентов: {}", config.max_agents));
            ui.label(format!("Длительность: {}с", config.match_duration.as_secs()));
        });
} 
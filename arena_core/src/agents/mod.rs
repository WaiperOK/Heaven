use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub mod scripted_agent;
pub mod llm_agent;
pub mod agent_manager;

pub use scripted_agent::ScriptedAgent;
pub use llm_agent::LLMAgent;
pub use agent_manager::AgentManager;

// Состояние игрового мира для агента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub agent_id: Uuid,
    pub position: Vec3,
    pub health: f32,
    pub energy: f32,
    pub nearby_agents: Vec<AgentInfo>,
    pub nearby_objects: Vec<ObjectInfo>,
    pub arena_bounds: Vec2,
    pub current_tick: u64,
    pub time_remaining: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: Uuid,
    pub position: Vec3,
    pub health: f32,
    pub team: Option<String>,
    pub distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    pub id: Uuid,
    pub position: Vec3,
    pub object_type: String,
    pub distance: f32,
}

// Действия агента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentAction {
    Move(Vec3),           // Движение в направлении
    Attack(Uuid),         // Атака другого агента
    UseItem(String),      // Использование предмета
    Communicate(String),  // Коммуникация
    Wait,                 // Ожидание
    Defend,               // Защита
}

// Интерфейс агента
#[async_trait]
pub trait AgentTrait: Send + Sync {
    // Основной метод принятия решения
    async fn decide(&mut self, state: &GameState) -> Result<AgentAction>;
    
    // Метаданные агента
    fn get_id(&self) -> Uuid;
    fn get_name(&self) -> &str;
    fn get_agent_type(&self) -> &str;
    fn get_team(&self) -> Option<&str>;
    
    // Инициализация и завершение
    async fn initialize(&mut self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    
    // Обработка событий
    async fn on_damage_received(&mut self, damage: f32, attacker_id: Uuid) -> Result<()>;
    async fn on_kill(&mut self, victim_id: Uuid) -> Result<()>;
    async fn on_death(&mut self) -> Result<()>;
    async fn on_message(&mut self, sender_id: Uuid, message: &str) -> Result<()>;
}

// Компоненты для ECS
#[derive(Component)]
pub struct Agent {
    pub id: Uuid,
    pub agent_type: String,
    pub name: String,
    pub team: Option<String>,
    pub health: f32,
    pub max_health: f32,
    pub energy: f32,
    pub max_energy: f32,
    pub speed: f32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub vision_range: f32,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_type: "Unknown".to_string(),
            name: "Agent".to_string(),
            team: None,
            health: 100.0,
            max_health: 100.0,
            energy: 100.0,
            max_energy: 100.0,
            speed: 5.0,
            attack_damage: 10.0,
            attack_range: 2.0,
            vision_range: 10.0,
        }
    }
}

#[derive(Component)]
pub struct AgentDecision {
    pub action: AgentAction,
    pub timestamp: f64,
    pub decision_time: f64, // Время принятия решения в мс
}

// Упрощенная архитектура без динамических агентов для начала
// #[derive(Component)]
// pub struct AgentController {
//     pub agent: Arc<dyn AgentTrait>,
// }

// События
#[derive(Event)]
pub struct AgentSpawnEvent {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub agent_type: String,
    pub team: Option<String>,
    pub position: Vec3,
}

#[derive(Event)]
pub struct AgentActionEvent {
    pub agent_id: Uuid,
    pub action: AgentAction,
    pub timestamp: f64,
}

#[derive(Event)]
pub struct AgentDamageEvent {
    pub agent_id: Uuid,
    pub damage: f32,
    pub attacker_id: Uuid,
}

#[derive(Event)]
pub struct AgentDeathEvent {
    pub agent_id: Uuid,
    pub killer_id: Option<Uuid>,
}

// Ресурсы
#[derive(Resource)]
pub struct AgentRegistry {
    pub agents: HashMap<Uuid, Entity>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }
}

// Плагин для системы агентов
pub struct AgentsPlugin;

impl Plugin for AgentsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AgentRegistry>()
            .add_event::<AgentSpawnEvent>()
            .add_event::<AgentActionEvent>()
            .add_event::<AgentDamageEvent>()
            .add_event::<AgentDeathEvent>()
            .add_systems(Update, (
                spawn_agents,
                update_agent_decisions,
                process_agent_actions,
                handle_agent_damage,
                handle_agent_death,
                regenerate_energy,
            ));
    }
}

// Системы
fn spawn_agents(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut agent_registry: ResMut<AgentRegistry>,
    mut spawn_events: EventReader<AgentSpawnEvent>,
) {
    // Обрабатываем события напрямую
    for event in spawn_events.read() {
        // Создаем визуальное представление агента
        let entity = commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: 0.5,
                    depth: 1.0,
                    ..default()
                })),
                material: materials.add(Color::rgb(0.8, 0.3, 0.3).into()),
                transform: Transform::from_translation(event.position),
                ..default()
            },
            Agent {
                id: event.agent_id,
                agent_type: event.agent_type.clone(),
                name: event.agent_name.clone(),
                team: event.team.clone(),
                ..default()
            },
        )).id();
        
        agent_registry.agents.insert(event.agent_id, entity);
        
        info!("Агент создан: {} ({})", event.agent_name, event.agent_id);
    }
}

fn update_agent_decisions(
    mut query: Query<(&Agent, &Transform)>,
    _time: Res<Time>,
    match_state: Res<crate::MatchState>,
    config: Res<crate::GameConfig>,
) {
    if !match_state.is_running {
        return;
    }
    
    for (agent_comp, transform) in query.iter() {
        let _game_state = GameState {
            agent_id: agent_comp.id,
            position: transform.translation,
            health: agent_comp.health,
            energy: agent_comp.energy,
            nearby_agents: vec![], // TODO: Реализовать поиск ближайших агентов
            nearby_objects: vec![], // TODO: Реализовать поиск объектов
            arena_bounds: config.arena_size,
            current_tick: match_state.current_tick,
            time_remaining: 0.0, // TODO: Вычислить оставшееся время
        };
        
        // TODO: Упрощенная логика принятия решений для MVP
        // В будущем здесь будет логика для различных типов агентов
    }
}

fn process_agent_actions(
    mut action_events: EventReader<AgentActionEvent>,
    mut query: Query<(&Agent, &mut Transform)>,
    time: Res<Time>,
) {
    for event in action_events.read() {
        if let Ok((_agent, mut transform)) = query
            .iter_mut()
            .find(|(a, _)| a.id == event.agent_id)
            .ok_or("Agent not found")
        {
            match &event.action {
                AgentAction::Move(direction) => {
                    let movement = *direction * time.delta_seconds();
                    transform.translation += movement;
                }
                AgentAction::Attack(target_id) => {
                    // TODO: Реализовать атаку
                    info!("Агент {} атакует {}", event.agent_id, target_id);
                }
                AgentAction::Wait => {
                    // Ничего не делаем
                }
                _ => {
                    // TODO: Реализовать другие действия
                }
            }
        }
    }
}

fn handle_agent_damage(
    mut damage_events: EventReader<AgentDamageEvent>,
    mut query: Query<&mut Agent>,
    mut death_events: EventWriter<AgentDeathEvent>,
) {
    for event in damage_events.read() {
        if let Ok(mut agent) = query
            .iter_mut()
            .find(|a| a.id == event.agent_id)
            .ok_or("Agent not found")
        {
            agent.health -= event.damage;
            
            if agent.health <= 0.0 {
                death_events.send(AgentDeathEvent {
                    agent_id: event.agent_id,
                    killer_id: Some(event.attacker_id),
                });
            }
        }
    }
}

fn handle_agent_death(
    mut commands: Commands,
    mut death_events: EventReader<AgentDeathEvent>,
    mut agent_registry: ResMut<AgentRegistry>,
) {
    for event in death_events.read() {
        if let Some(entity) = agent_registry.agents.remove(&event.agent_id) {
            commands.entity(entity).despawn();
            info!("Агент {} погиб", event.agent_id);
        }
    }
}

fn regenerate_energy(
    mut query: Query<&mut Agent>,
    time: Res<Time>,
    match_state: Res<crate::MatchState>,
) {
    if !match_state.is_running {
        return;
    }
    
    let energy_regen_rate = 5.0; // Энергия в секунду
    
    for mut agent in query.iter_mut() {
        if agent.energy < agent.max_energy {
            agent.energy = (agent.energy + energy_regen_rate * time.delta_seconds())
                .min(agent.max_energy);
        }
    }
} 
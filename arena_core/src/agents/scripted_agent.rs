use super::{AgentTrait, GameState, AgentAction, AgentInfo};
use async_trait::async_trait;
use anyhow::Result;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::time::Instant;

// Состояния FSM для ScriptedAgent
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Exploring,
    Chasing(Uuid),
    Attacking(Uuid),
    Fleeing,
    Defending,
    Dead,
}

// Параметры поведения агента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub aggression: f32,        // 0.0 - 1.0 (пассивный - агрессивный)
    pub caution: f32,           // 0.0 - 1.0 (безрассудный - осторожный)
    pub exploration: f32,       // 0.0 - 1.0 (статичный - исследователь)
    pub cooperation: f32,       // 0.0 - 1.0 (индивидуалист - командный)
    pub health_threshold: f32,  // Порог здоровья для бегства
    pub attack_range: f32,      // Дальность атаки
    pub vision_range: f32,      // Дальность обзора
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            aggression: 0.7,
            caution: 0.5,
            exploration: 0.6,
            cooperation: 0.4,
            health_threshold: 30.0,
            attack_range: 3.0,
            vision_range: 10.0,
        }
    }
}

// Планировщик целей (GOAP-like)
#[derive(Debug, Clone)]
pub struct Goal {
    pub name: String,
    pub priority: f32,
    pub conditions: HashMap<String, f32>,
    pub actions: Vec<String>,
}

// ScriptedAgent с FSM и GOAP элементами
pub struct ScriptedAgent {
    pub id: Uuid,
    pub name: String,
    pub agent_type: String,
    pub team: Option<String>,
    pub current_state: AgentState,
    pub behavior: BehaviorConfig,
    pub memory: AgentMemory,
    pub goals: Vec<Goal>,
    pub last_decision_time: Instant,
    pub state_duration: f32,
    pub target_position: Option<Vec3>,
}

#[derive(Debug, Clone)]
pub struct AgentMemory {
    pub known_enemies: HashMap<Uuid, AgentInfo>,
    pub known_allies: HashMap<Uuid, AgentInfo>,
    pub visited_positions: Vec<Vec3>,
    pub last_damage_time: Option<Instant>,
    pub last_damage_source: Option<Uuid>,
    pub kill_count: u32,
    pub death_count: u32,
}

impl Default for AgentMemory {
    fn default() -> Self {
        Self {
            known_enemies: HashMap::new(),
            known_allies: HashMap::new(),
            visited_positions: Vec::new(),
            last_damage_time: None,
            last_damage_source: None,
            kill_count: 0,
            death_count: 0,
        }
    }
}

impl ScriptedAgent {
    pub fn new(name: String, team: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            agent_type: "ScriptedAgent".to_string(),
            team,
            current_state: AgentState::Idle,
            behavior: BehaviorConfig::default(),
            memory: AgentMemory::default(),
            goals: Self::create_default_goals(),
            last_decision_time: Instant::now(),
            state_duration: 0.0,
            target_position: None,
        }
    }

    pub fn with_behavior(mut self, behavior: BehaviorConfig) -> Self {
        self.behavior = behavior;
        self
    }

    fn create_default_goals() -> Vec<Goal> {
        vec![
            Goal {
                name: "Survive".to_string(),
                priority: 1.0,
                conditions: HashMap::from([("health".to_string(), 1.0)]),
                actions: vec!["flee".to_string(), "defend".to_string()],
            },
            Goal {
                name: "Eliminate_Enemies".to_string(),
                priority: 0.8,
                conditions: HashMap::from([("enemy_nearby".to_string(), 1.0)]),
                actions: vec!["chase".to_string(), "attack".to_string()],
            },
            Goal {
                name: "Explore".to_string(),
                priority: 0.3,
                conditions: HashMap::new(),
                actions: vec!["explore".to_string()],
            },
        ]
    }

    // FSM переход состояний
    fn transition_state(&mut self, state: &GameState) -> AgentState {
        let current_health_ratio = state.health / 100.0; // Предполагаем макс. здоровье 100
        
        match self.current_state {
            AgentState::Idle => {
                if current_health_ratio < self.behavior.health_threshold / 100.0 {
                    AgentState::Fleeing
                } else if let Some(enemy) = self.find_nearest_enemy(&state.nearby_agents) {
                    if enemy.distance <= self.behavior.attack_range {
                        AgentState::Attacking(enemy.id)
                    } else if enemy.distance <= self.behavior.vision_range {
                        AgentState::Chasing(enemy.id)
                    } else {
                        AgentState::Exploring
                    }
                } else {
                    AgentState::Exploring
                }
            },
            
            AgentState::Exploring => {
                if current_health_ratio < self.behavior.health_threshold / 100.0 {
                    AgentState::Fleeing
                } else if let Some(enemy) = self.find_nearest_enemy(&state.nearby_agents) {
                    if enemy.distance <= self.behavior.attack_range {
                        AgentState::Attacking(enemy.id)
                    } else {
                        AgentState::Chasing(enemy.id)
                    }
                } else if self.state_duration > 5.0 {
                    // Исследуем достаточно долго, возвращаемся в idle
                    AgentState::Idle
                } else {
                    AgentState::Exploring
                }
            },
            
            AgentState::Chasing(target_id) => {
                if current_health_ratio < self.behavior.health_threshold / 100.0 {
                    AgentState::Fleeing
                } else if let Some(enemy) = state.nearby_agents.iter()
                    .find(|a| a.id == target_id && a.team != self.team) {
                    if enemy.distance <= self.behavior.attack_range {
                        AgentState::Attacking(target_id)
                    } else if enemy.distance > self.behavior.vision_range {
                        AgentState::Exploring
                    } else {
                        AgentState::Chasing(target_id)
                    }
                } else {
                    AgentState::Exploring
                }
            },
            
            AgentState::Attacking(target_id) => {
                if current_health_ratio < self.behavior.health_threshold / 100.0 {
                    AgentState::Fleeing
                } else if let Some(enemy) = state.nearby_agents.iter()
                    .find(|a| a.id == target_id && a.team != self.team) {
                    if enemy.distance <= self.behavior.attack_range {
                        AgentState::Attacking(target_id)
                    } else {
                        AgentState::Chasing(target_id)
                    }
                } else {
                    AgentState::Exploring
                }
            },
            
            AgentState::Fleeing => {
                if current_health_ratio > (self.behavior.health_threshold + 20.0) / 100.0 {
                    AgentState::Idle
                } else if self.is_safe_distance(&state.nearby_agents) {
                    AgentState::Defending
                } else {
                    AgentState::Fleeing
                }
            },
            
            AgentState::Defending => {
                if current_health_ratio > (self.behavior.health_threshold + 30.0) / 100.0 {
                    AgentState::Idle
                } else if !self.is_safe_distance(&state.nearby_agents) {
                    AgentState::Fleeing
                } else {
                    AgentState::Defending
                }
            },
            
            AgentState::Dead => AgentState::Dead,
        }
    }

    fn find_nearest_enemy<'a>(&self, nearby_agents: &'a [AgentInfo]) -> Option<&'a AgentInfo> {
        nearby_agents.iter()
            .filter(|agent| agent.team != self.team)
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
    }

    fn is_safe_distance(&self, nearby_agents: &[AgentInfo]) -> bool {
        let min_safe_distance = 5.0;
        nearby_agents.iter()
            .filter(|agent| agent.team != self.team)
            .all(|agent| agent.distance > min_safe_distance)
    }

    fn generate_explore_position(&mut self, state: &GameState) -> Vec3 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Избегаем уже посещенных позиций
        let mut attempts = 0;
        let max_attempts = 10;
        
        while attempts < max_attempts {
            let x = rng.gen_range(-state.arena_bounds.x / 2.0..state.arena_bounds.x / 2.0);
            let z = rng.gen_range(-state.arena_bounds.y / 2.0..state.arena_bounds.y / 2.0);
            let pos = Vec3::new(x, 0.0, z);
            
            // Проверяем, не слишком ли близко к уже посещенным местам
            if !self.memory.visited_positions.iter()
                .any(|visited| visited.distance(pos) < 5.0) {
                self.memory.visited_positions.push(pos);
                return pos;
            }
            
            attempts += 1;
        }
        
        // Если не нашли уникальное место, идем случайно
        Vec3::new(
            rng.gen_range(-state.arena_bounds.x / 2.0..state.arena_bounds.x / 2.0),
            0.0,
            rng.gen_range(-state.arena_bounds.y / 2.0..state.arena_bounds.y / 2.0),
        )
    }

    fn calculate_flee_direction(&self, state: &GameState) -> Vec3 {
        let mut flee_direction = Vec3::ZERO;
        
        // Бежим от всех врагов
        for enemy in state.nearby_agents.iter()
            .filter(|agent| agent.team != self.team) {
            let direction = (state.position - enemy.position).normalize();
            let weight = 1.0 / (enemy.distance + 0.1); // Ближе = больше вес
            flee_direction += direction * weight;
        }
        
        // Нормализуем и добавляем скорость
        if flee_direction.length() > 0.0 {
            flee_direction.normalize() * 8.0 // Быстрое бегство
        } else {
            // Если нет врагов поблизости, идем к углу арены
            let corner = Vec3::new(
                -state.arena_bounds.x / 2.0,
                0.0,
                -state.arena_bounds.y / 2.0,
            );
            (corner - state.position).normalize() * 6.0
        }
    }

    fn update_memory(&mut self, state: &GameState) {
        // Обновляем информацию о агентах
        for agent in &state.nearby_agents {
            if agent.team == self.team {
                self.memory.known_allies.insert(agent.id, agent.clone());
            } else {
                self.memory.known_enemies.insert(agent.id, agent.clone());
            }
        }
        
        // Очищаем старую информацию (агенты, которых давно не видели)
        // В реальной реализации можно добавить timestamp для каждого агента
    }
}

#[async_trait]
impl AgentTrait for ScriptedAgent {
    async fn decide(&mut self, state: &GameState) -> Result<AgentAction> {
        let decision_start = Instant::now();
        
        // Обновляем длительность текущего состояния
        self.state_duration += decision_start.duration_since(self.last_decision_time).as_secs_f32();
        self.last_decision_time = decision_start;
        
        // Обновляем память
        self.update_memory(state);
        
        // Переходим в новое состояние если нужно
        let new_state = self.transition_state(state);
        if new_state != self.current_state {
            self.current_state = new_state;
            self.state_duration = 0.0;
        }
        
        // Принимаем решение на основе текущего состояния
        let action = match self.current_state {
            AgentState::Idle => {
                AgentAction::Wait
            },
            
            AgentState::Exploring => {
                if self.target_position.is_none() || 
                   self.target_position.unwrap().distance(state.position) < 2.0 {
                    self.target_position = Some(self.generate_explore_position(state));
                }
                
                if let Some(target) = self.target_position {
                    let direction = (target - state.position).normalize() * 3.0;
                    AgentAction::Move(direction)
                } else {
                    AgentAction::Wait
                }
            },
            
            AgentState::Chasing(target_id) => {
                if let Some(enemy) = state.nearby_agents.iter()
                    .find(|a| a.id == target_id && a.team != self.team) {
                    let direction = (enemy.position - state.position).normalize() * 5.0;
                    AgentAction::Move(direction)
                } else {
                    AgentAction::Wait
                }
            },
            
            AgentState::Attacking(target_id) => {
                if state.nearby_agents.iter()
                                         .any(|a| a.id == target_id && a.team != self.team) {
                    AgentAction::Attack(target_id)
                } else {
                    AgentAction::Wait
                }
            },
            
            AgentState::Fleeing => {
                let flee_direction = self.calculate_flee_direction(state);
                AgentAction::Move(flee_direction)
            },
            
            AgentState::Defending => {
                AgentAction::Defend
            },
            
            AgentState::Dead => {
                AgentAction::Wait
            },
        };
        
        Ok(action)
    }

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_agent_type(&self) -> &str {
        &self.agent_type
    }

    fn get_team(&self) -> Option<&str> {
        self.team.as_deref()
    }

    async fn initialize(&mut self) -> Result<()> {
        self.current_state = AgentState::Idle;
        self.last_decision_time = Instant::now();
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.current_state = AgentState::Dead;
        Ok(())
    }

    async fn on_damage_received(&mut self, damage: f32, attacker_id: Uuid) -> Result<()> {
        self.memory.last_damage_time = Some(Instant::now());
        self.memory.last_damage_source = Some(attacker_id);
        
        // Повышаем приоритет бегства если здоровье низкое
        if self.behavior.caution > 0.5 {
            self.behavior.health_threshold = (self.behavior.health_threshold * 1.2).min(80.0);
        }
        
        Ok(())
    }

    async fn on_kill(&mut self, victim_id: Uuid) -> Result<()> {
        self.memory.kill_count += 1;
        self.memory.known_enemies.remove(&victim_id);
        
        // Увеличиваем агрессивность после убийства
        self.behavior.aggression = (self.behavior.aggression * 1.1).min(1.0);
        
        Ok(())
    }

    async fn on_death(&mut self) -> Result<()> {
        self.memory.death_count += 1;
        self.current_state = AgentState::Dead;
        
        // Повышаем осторожность после смерти
        self.behavior.caution = (self.behavior.caution * 1.2).min(1.0);
        
        Ok(())
    }

    async fn on_message(&mut self, sender_id: Uuid, message: &str) -> Result<()> {
        // Простая обработка сообщений от союзников
        if self.memory.known_allies.contains_key(&sender_id) {
            if message.contains("enemy") {
                // Союзник сообщает о враге - повышаем алертность
                self.behavior.aggression = (self.behavior.aggression * 1.1).min(1.0);
            } else if message.contains("help") {
                // Союзник просит помощи - меняем цель
                self.behavior.cooperation = (self.behavior.cooperation * 1.1).min(1.0);
            }
        }
        
        Ok(())
    }
} 
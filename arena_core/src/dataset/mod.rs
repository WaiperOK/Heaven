use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use uuid::Uuid;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::{DateTime, Utc};
use super::agents::{GameState, AgentAction};

// Структура для логирования одного действия агента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDecisionLog {
    pub timestamp: DateTime<Utc>,
    pub match_id: Uuid,
    pub agent_id: Uuid,
    pub agent_name: String,
    pub agent_type: String,
    pub team: Option<String>,
    pub tick: u64,
    pub state: GameState,
    pub action: AgentAction,
    pub decision_time_ms: u64,
    pub reasoning: Option<String>, // Для LLM агентов
}

// Структура для метаданных матча
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchMetadata {
    pub match_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<f64>,
    pub arena_config: ArenaConfig,
    pub participating_agents: Vec<AgentMetadata>,
    pub winner: Option<String>, // Команда или agent_id
    pub final_scores: Vec<AgentScore>,
    pub total_actions: u64,
    pub total_damage_dealt: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaConfig {
    pub arena_size: (f32, f32),
    pub match_duration_seconds: f64,
    pub max_agents: usize,
    pub tick_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub agent_type: String,
    pub team: Option<String>,
    pub config: serde_json::Value, // Конфигурация агента в JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentScore {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub kills: u32,
    pub deaths: u32,
    pub damage_dealt: f32,
    pub damage_received: f32,
    pub survival_time_seconds: f64,
    pub actions_taken: u64,
    pub avg_decision_time_ms: f64,
}

// Конфигурация системы логирования
#[derive(Resource)]
pub struct DatasetConfig {
    pub output_directory: String,
    pub enable_logging: bool,
    pub log_level: LogLevel,
    pub max_file_size_mb: usize,
    pub compress_old_files: bool,
    pub retention_days: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    All,        // Все действия
    Important,  // Только важные действия (атака, смерть)
    Minimal,    // Только результаты матчей
}

impl Default for DatasetConfig {
    fn default() -> Self {
        Self {
            output_directory: "data/logs".to_string(),
            enable_logging: true,
            log_level: LogLevel::All,
            max_file_size_mb: 100,
            compress_old_files: true,
            retention_days: 30,
        }
    }
}

// Основная структура для управления логированием
#[derive(Resource)]
pub struct DatasetLogger {
    pub config: DatasetConfig,
    pub current_match: Option<MatchMetadata>,
    pub agent_scores: std::collections::HashMap<Uuid, AgentScore>,
    pub actions_buffer: Vec<AgentDecisionLog>,
    pub buffer_size: usize,
    pub last_flush: std::time::Instant,
    pub flush_interval_seconds: u64,
}

impl DatasetLogger {
    pub fn new(config: DatasetConfig) -> Self {
        Self {
            config,
            current_match: None,
            agent_scores: std::collections::HashMap::new(),
            actions_buffer: Vec::new(),
            buffer_size: 1000,
            last_flush: std::time::Instant::now(),
            flush_interval_seconds: 10,
        }
    }

    // Начинаем новый матч
    pub fn start_match(&mut self, match_id: Uuid, arena_config: ArenaConfig, agents: Vec<AgentMetadata>) -> Result<()> {
        if !self.config.enable_logging {
            return Ok(());
        }

        // Завершаем предыдущий матч если есть
        if let Some(current_match) = &self.current_match {
            self.end_match(None)?;
        }

        // Создаем новый матч
        self.current_match = Some(MatchMetadata {
            match_id,
            start_time: Utc::now(),
            end_time: None,
            duration_seconds: None,
            arena_config,
            participating_agents: agents.clone(),
            winner: None,
            final_scores: Vec::new(),
            total_actions: 0,
            total_damage_dealt: 0.0,
        });

        // Инициализируем счетчики агентов
        self.agent_scores.clear();
        for agent in &agents {
            self.agent_scores.insert(agent.agent_id, AgentScore {
                agent_id: agent.agent_id,
                agent_name: agent.agent_name.clone(),
                kills: 0,
                deaths: 0,
                damage_dealt: 0.0,
                damage_received: 0.0,
                survival_time_seconds: 0.0,
                actions_taken: 0,
                avg_decision_time_ms: 0.0,
            });
        }

        // Создаем директорию если не существует
        std::fs::create_dir_all(&self.config.output_directory)?;

        info!("Started logging match: {}", match_id);
        Ok(())
    }

    // Завершаем текущий матч
    pub fn end_match(&mut self, winner: Option<String>) -> Result<()> {
        if !self.config.enable_logging {
            return Ok(());
        }

        let Some(mut current_match) = self.current_match.take() else {
            return Ok(());
        };

        current_match.end_time = Some(Utc::now());
        current_match.duration_seconds = Some(
            current_match.end_time.unwrap()
                .signed_duration_since(current_match.start_time)
                .num_milliseconds() as f64 / 1000.0
        );
        current_match.winner = winner;
        current_match.final_scores = self.agent_scores.values().cloned().collect();

        // Сохраняем буфер действий
        self.flush_actions_buffer()?;

        // Сохраняем метаданные матча
        self.save_match_metadata(&current_match)?;

        info!("Ended logging match: {} (duration: {:.1}s)", 
              current_match.match_id, 
              current_match.duration_seconds.unwrap_or(0.0));

        Ok(())
    }

    // Логируем действие агента
    pub fn log_agent_action(
        &mut self,
        agent_id: Uuid,
        agent_name: String,
        agent_type: String,
        team: Option<String>,
        tick: u64,
        state: GameState,
        action: AgentAction,
        decision_time_ms: u64,
        reasoning: Option<String>,
    ) -> Result<()> {
        if !self.config.enable_logging {
            return Ok(());
        }

        let Some(current_match) = &mut self.current_match else {
            return Ok(());
        };

        // Проверяем уровень логирования
        let should_log = match self.config.log_level {
            LogLevel::All => true,
            LogLevel::Important => matches!(action, AgentAction::Attack(_) | AgentAction::Defend),
            LogLevel::Minimal => false,
        };

        if !should_log {
            return Ok(());
        }

        let log_entry = AgentDecisionLog {
            timestamp: Utc::now(),
            match_id: current_match.match_id,
            agent_id,
            agent_name,
            agent_type,
            team,
            tick,
            state,
            action,
            decision_time_ms,
            reasoning,
        };

        self.actions_buffer.push(log_entry);
        current_match.total_actions += 1;

        // Обновляем статистику агента
        if let Some(score) = self.agent_scores.get_mut(&agent_id) {
            score.actions_taken += 1;
            
            // Обновляем среднее время принятия решения
            let total_time = score.avg_decision_time_ms * (score.actions_taken - 1) as f64;
            score.avg_decision_time_ms = (total_time + decision_time_ms as f64) / score.actions_taken as f64;
        }

        // Периодически сохраняем буфер
        if self.actions_buffer.len() >= self.buffer_size || 
           self.last_flush.elapsed().as_secs() >= self.flush_interval_seconds {
            self.flush_actions_buffer()?;
        }

        Ok(())
    }

    // Логируем урон
    pub fn log_damage(&mut self, attacker_id: Uuid, victim_id: Uuid, damage: f32) -> Result<()> {
        if !self.config.enable_logging {
            return Ok(());
        }

        if let Some(current_match) = &mut self.current_match {
            current_match.total_damage_dealt += damage;
        }

        // Обновляем статистику
        if let Some(attacker_score) = self.agent_scores.get_mut(&attacker_id) {
            attacker_score.damage_dealt += damage;
        }
        
        if let Some(victim_score) = self.agent_scores.get_mut(&victim_id) {
            victim_score.damage_received += damage;
        }

        Ok(())
    }

    // Логируем смерть агента
    pub fn log_agent_death(&mut self, agent_id: Uuid, killer_id: Option<Uuid>) -> Result<()> {
        if !self.config.enable_logging {
            return Ok(());
        }

        // Обновляем статистику
        if let Some(victim_score) = self.agent_scores.get_mut(&agent_id) {
            victim_score.deaths += 1;
        }

        if let Some(killer_id) = killer_id {
            if let Some(killer_score) = self.agent_scores.get_mut(&killer_id) {
                killer_score.kills += 1;
            }
        }

        Ok(())
    }

    // Сохраняем буфер действий в файл
    fn flush_actions_buffer(&mut self) -> Result<()> {
        if self.actions_buffer.is_empty() {
            return Ok(());
        }

        let Some(current_match) = &self.current_match else {
            return Ok(());
        };

        let filename = format!("{}/actions_{}.jsonl", 
                              self.config.output_directory, 
                              current_match.match_id);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)?;

        for action in &self.actions_buffer {
            let json_line = serde_json::to_string(action)?;
            writeln!(file, "{}", json_line)?;
        }

        file.flush()?;
        self.actions_buffer.clear();
        self.last_flush = std::time::Instant::now();

        Ok(())
    }

    // Сохраняем метаданные матча
    fn save_match_metadata(&self, match_metadata: &MatchMetadata) -> Result<()> {
        let filename = format!("{}/match_{}.json", 
                              self.config.output_directory, 
                              match_metadata.match_id);

        let json = serde_json::to_string_pretty(match_metadata)?;
        std::fs::write(&filename, json)?;

        Ok(())
    }

    // Экспорт данных в формат для обучения
    pub fn export_training_data(&self, match_ids: Vec<Uuid>, output_path: &str) -> Result<()> {
        let mut training_data = Vec::new();

        for match_id in match_ids {
            let actions_file = format!("{}/actions_{}.jsonl", 
                                      self.config.output_directory, 
                                      match_id);

            if Path::new(&actions_file).exists() {
                let content = std::fs::read_to_string(&actions_file)?;
                
                for line in content.lines() {
                    if let Ok(action_log) = serde_json::from_str::<AgentDecisionLog>(line) {
                        // Конвертируем в формат для обучения
                        let training_entry = serde_json::json!({
                            "instruction": format!("You are an AI agent in a combat arena. Analyze the game state and choose the best action."),
                            "input": format!("{:?}", action_log.state),
                            "output": format!("{:?}", action_log.action),
                            "metadata": {
                                "agent_type": action_log.agent_type,
                                "decision_time_ms": action_log.decision_time_ms,
                                "reasoning": action_log.reasoning
                            }
                        });
                        
                        training_data.push(training_entry);
                    }
                }
            }
        }

        // Сохраняем в файл
        let json = serde_json::to_string_pretty(&training_data)?;
        std::fs::write(output_path, json)?;

        info!("Exported {} training examples to {}", training_data.len(), output_path);
        Ok(())
    }

    // Получение статистики
    pub fn get_match_stats(&self) -> Option<&MatchMetadata> {
        self.current_match.as_ref()
    }

    pub fn get_agent_stats(&self, agent_id: &Uuid) -> Option<&AgentScore> {
        self.agent_scores.get(agent_id)
    }

    // Очистка старых файлов
    pub fn cleanup_old_files(&self) -> Result<()> {
        let retention_duration = chrono::Duration::days(self.config.retention_days as i64);
        let cutoff_time = Utc::now() - retention_duration;

        let dir = std::fs::read_dir(&self.config.output_directory)?;
        
        for entry in dir {
            let entry = entry?;
            let path = entry.path();
            
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified_time = DateTime::<Utc>::from(modified);
                    
                    if modified_time < cutoff_time {
                        if path.is_file() {
                            std::fs::remove_file(&path)?;
                            info!("Removed old log file: {:?}", path);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// События для системы логирования
#[derive(Event)]
pub struct LogAgentActionEvent {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub agent_type: String,
    pub team: Option<String>,
    pub tick: u64,
    pub state: GameState,
    pub action: AgentAction,
    pub decision_time_ms: u64,
    pub reasoning: Option<String>,
}

#[derive(Event)]
pub struct LogDamageEvent {
    pub attacker_id: Uuid,
    pub victim_id: Uuid,
    pub damage: f32,
}

#[derive(Event)]
pub struct LogAgentDeathEvent {
    pub agent_id: Uuid,
    pub killer_id: Option<Uuid>,
}

#[derive(Event)]
pub struct ExportTrainingDataEvent {
    pub match_ids: Vec<Uuid>,
    pub output_path: String,
}

// Плагин для системы логирования
pub struct DatasetPlugin;

impl Plugin for DatasetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DatasetConfig>()
            .insert_resource(DatasetLogger::new(DatasetConfig::default()))
            .add_event::<LogAgentActionEvent>()
            .add_event::<LogDamageEvent>()
            .add_event::<LogAgentDeathEvent>()
            .add_event::<ExportTrainingDataEvent>()
            .add_systems(Update, (
                handle_log_agent_action,
                handle_log_damage,
                handle_log_agent_death,
                handle_export_training_data,
            ))
            .add_systems(Update, periodic_cleanup);
    }
}

// Системы обработки событий
fn handle_log_agent_action(
    mut logger: ResMut<DatasetLogger>,
    mut events: EventReader<LogAgentActionEvent>,
) {
    for event in events.read() {
        if let Err(e) = logger.log_agent_action(
            event.agent_id,
            event.agent_name.clone(),
            event.agent_type.clone(),
            event.team.clone(),
            event.tick,
            event.state.clone(),
            event.action.clone(),
            event.decision_time_ms,
            event.reasoning.clone(),
        ) {
            error!("Failed to log agent action: {}", e);
        }
    }
}

fn handle_log_damage(
    mut logger: ResMut<DatasetLogger>,
    mut events: EventReader<LogDamageEvent>,
) {
    for event in events.read() {
        if let Err(e) = logger.log_damage(event.attacker_id, event.victim_id, event.damage) {
            error!("Failed to log damage: {}", e);
        }
    }
}

fn handle_log_agent_death(
    mut logger: ResMut<DatasetLogger>,
    mut events: EventReader<LogAgentDeathEvent>,
) {
    for event in events.read() {
        if let Err(e) = logger.log_agent_death(event.agent_id, event.killer_id) {
            error!("Failed to log agent death: {}", e);
        }
    }
}

fn handle_export_training_data(
    logger: Res<DatasetLogger>,
    mut events: EventReader<ExportTrainingDataEvent>,
) {
    for event in events.read() {
        if let Err(e) = logger.export_training_data(event.match_ids.clone(), &event.output_path) {
            error!("Failed to export training data: {}", e);
        }
    }
}

fn periodic_cleanup(
    logger: Res<DatasetLogger>,
    mut last_cleanup: Local<Option<std::time::Instant>>,
) {
    let cleanup_interval = std::time::Duration::from_secs(3600); // Каждый час
    
    if let Some(last_time) = *last_cleanup {
        if last_time.elapsed() >= cleanup_interval {
            if let Err(e) = logger.cleanup_old_files() {
                error!("Failed to cleanup old files: {}", e);
            }
            *last_cleanup = Some(std::time::Instant::now());
        }
    } else {
        *last_cleanup = Some(std::time::Instant::now());
    }
} 
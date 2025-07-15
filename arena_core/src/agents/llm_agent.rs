use super::{AgentTrait, GameState, AgentAction, AgentInfo, ObjectInfo};
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::Instant;
use reqwest::Client;
use std::collections::HashMap;

// Запрос к LLM сервису
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub stop_sequences: Option<Vec<String>>,
    pub system_prompt: Option<String>,
}

// Ответ от LLM сервиса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub text: String,
    pub tokens_used: u32,
    pub processing_time_ms: u64,
    pub model_name: String,
}

// Конфигурация LLM агента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub model_name: String,
    pub llm_service_url: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub timeout_seconds: u64,
    pub system_prompt: String,
    pub action_prompt_template: String,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            model_name: "llama2:7b".to_string(),
            llm_service_url: "http://localhost:8000".to_string(),
            max_tokens: 150,
            temperature: 0.7,
            top_p: 0.9,
            timeout_seconds: 5,
            system_prompt: "You are an AI agent in a combat arena. You must make tactical decisions to survive and eliminate enemies. Be strategic and decisive.".to_string(),
            action_prompt_template: "Game State: {state}\n\nAvailable Actions: Move(direction), Attack(target_id), UseItem(item), Communicate(message), Wait, Defend\n\nChoose your action:".to_string(),
        }
    }
}

// Статистика производительности LLM
#[derive(Debug, Clone, Default)]
pub struct LLMStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_tokens_used: u64,
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
}

// Кеш для повторяющихся состояний
#[derive(Debug, Clone)]
pub struct DecisionCache {
    pub entries: HashMap<String, CachedDecision>,
    pub max_size: usize,
    pub hit_count: u64,
    pub miss_count: u64,
}

#[derive(Debug, Clone)]
pub struct CachedDecision {
    pub action: AgentAction,
    pub timestamp: Instant,
    pub ttl_seconds: u64,
}

impl DecisionCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn get(&mut self, state_hash: &str) -> Option<AgentAction> {
        if let Some(cached) = self.entries.get(state_hash) {
            if cached.timestamp.elapsed().as_secs() < cached.ttl_seconds {
                self.hit_count += 1;
                return Some(cached.action.clone());
            } else {
                self.entries.remove(state_hash);
            }
        }
        self.miss_count += 1;
        None
    }

    pub fn insert(&mut self, state_hash: String, action: AgentAction, ttl_seconds: u64) {
        if self.entries.len() >= self.max_size {
            // Удаляем самую старую запись
            if let Some(oldest_key) = self.entries.iter()
                .min_by_key(|(_, v)| v.timestamp)
                .map(|(k, _)| k.clone()) {
                self.entries.remove(&oldest_key);
            }
        }

        self.entries.insert(state_hash, CachedDecision {
            action,
            timestamp: Instant::now(),
            ttl_seconds,
        });
    }

    pub fn clear_expired(&mut self) {
        self.entries.retain(|_, cached| {
            cached.timestamp.elapsed().as_secs() < cached.ttl_seconds
        });
    }
}

// LLM Agent
pub struct LLMAgent {
    pub id: Uuid,
    pub name: String,
    pub agent_type: String,
    pub team: Option<String>,
    pub config: LLMConfig,
    pub client: Client,
    pub stats: LLMStats,
    pub decision_cache: DecisionCache,
    pub conversation_history: Vec<String>,
    pub max_history_size: usize,
    pub last_state_hash: Option<String>,
    pub consecutive_failures: u32,
    pub max_consecutive_failures: u32,
    pub fallback_actions: Vec<AgentAction>,
    pub fallback_index: usize,
}

impl LLMAgent {
    pub fn new(name: String, team: Option<String>, config: LLMConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            id: Uuid::new_v4(),
            name,
            agent_type: "LLMAgent".to_string(),
            team,
            config,
            client,
            stats: LLMStats::default(),
            decision_cache: DecisionCache::new(100),
            conversation_history: Vec::new(),
            max_history_size: 10,
            last_state_hash: None,
            consecutive_failures: 0,
            max_consecutive_failures: 3,
            fallback_actions: vec![
                AgentAction::Wait,
                AgentAction::Defend,
                AgentAction::Move(Vec3::new(1.0, 0.0, 0.0)),
                AgentAction::Move(Vec3::new(-1.0, 0.0, 0.0)),
                AgentAction::Move(Vec3::new(0.0, 0.0, 1.0)),
                AgentAction::Move(Vec3::new(0.0, 0.0, -1.0)),
            ],
            fallback_index: 0,
        }
    }

    pub fn with_config(mut self, config: LLMConfig) -> Self {
        self.config = config;
        self
    }

    // Создаем хеш состояния для кеширования
    fn create_state_hash(&self, state: &GameState) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        // Хешируем ключевые параметры состояния
        state.position.x.to_bits().hash(&mut hasher);
        state.position.z.to_bits().hash(&mut hasher);
        (state.health as i32).hash(&mut hasher);
        (state.energy as i32).hash(&mut hasher);
        state.nearby_agents.len().hash(&mut hasher);
        state.current_tick.hash(&mut hasher);
        
        // Хешируем позиции ближайших агентов
        for agent in &state.nearby_agents {
            agent.position.x.to_bits().hash(&mut hasher);
            agent.position.z.to_bits().hash(&mut hasher);
            (agent.health as i32).hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    // Конвертируем игровое состояние в текстовый промпт
    fn state_to_prompt(&self, state: &GameState) -> String {
        let mut prompt = format!(
            "=== AGENT STATUS ===\n\
            Position: ({:.1}, {:.1})\n\
            Health: {:.1}/100\n\
            Energy: {:.1}/100\n\
            Tick: {}\n\
            Time Remaining: {:.1}s\n\n",
            state.position.x, state.position.z,
            state.health, state.energy,
            state.current_tick, state.time_remaining
        );

        // Информация о ближайших агентах
        if !state.nearby_agents.is_empty() {
            prompt.push_str("=== NEARBY AGENTS ===\n");
            for agent in &state.nearby_agents {
                let relation = if agent.team.as_ref() == self.team.as_ref() {
                    "ALLY"
                } else {
                    "ENEMY"
                };
                prompt.push_str(&format!(
                    "- {} at ({:.1}, {:.1}), Health: {:.1}, Distance: {:.1}\n",
                    relation, agent.position.x, agent.position.z,
                    agent.health, agent.distance
                ));
            }
            prompt.push('\n');
        }

        // Информация об объектах
        if !state.nearby_objects.is_empty() {
            prompt.push_str("=== NEARBY OBJECTS ===\n");
            for obj in &state.nearby_objects {
                prompt.push_str(&format!(
                    "- {} at ({:.1}, {:.1}), Distance: {:.1}\n",
                    obj.object_type, obj.position.x, obj.position.z, obj.distance
                ));
            }
            prompt.push('\n');
        }

        // Границы арены
        prompt.push_str(&format!(
            "=== ARENA INFO ===\n\
            Bounds: {:.1}x{:.1}\n\
            Center: (0, 0)\n\n",
            state.arena_bounds.x, state.arena_bounds.y
        ));

        // Контекст из истории
        if !self.conversation_history.is_empty() {
            prompt.push_str("=== RECENT ACTIONS ===\n");
            for (i, action) in self.conversation_history.iter().rev().take(3).enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, action));
            }
            prompt.push('\n');
        }

        // Шаблон для действий
        prompt.push_str(&self.config.action_prompt_template.replace("{state}", &prompt));
        prompt.push_str("\n\nRespond with ONLY the action in format: ACTION_NAME(parameters)\n");
        prompt.push_str("Examples:\n");
        prompt.push_str("- Move(2.0, 0.0, 1.5)\n");
        prompt.push_str("- Attack(enemy_id)\n");
        prompt.push_str("- Wait\n");
        prompt.push_str("- Defend\n");
        prompt.push_str("- Communicate(message)\n");

        prompt
    }

    // Отправляем запрос к LLM сервису
    async fn query_llm(&mut self, prompt: String) -> Result<LLMResponse> {
        let request = LLMRequest {
            model: self.config.model_name.clone(),
            prompt,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            stop_sequences: Some(vec!["\n".to_string()]),
            system_prompt: Some(self.config.system_prompt.clone()),
        };

        let start_time = Instant::now();
        
        let response = self.client
            .post(&format!("{}/generate", self.config.llm_service_url))
            .json(&request)
            .send()
            .await?;

        let response_time = start_time.elapsed().as_millis() as u64;

        if response.status().is_success() {
            let llm_response: LLMResponse = response.json().await?;
            
            // Обновляем статистику
            self.stats.total_requests += 1;
            self.stats.successful_requests += 1;
            self.stats.total_tokens_used += llm_response.tokens_used as u64;
            
            // Обновляем статистику времени ответа
            if self.stats.min_response_time_ms == 0 || response_time < self.stats.min_response_time_ms {
                self.stats.min_response_time_ms = response_time;
            }
            if response_time > self.stats.max_response_time_ms {
                self.stats.max_response_time_ms = response_time;
            }
            
            let total_time = self.stats.avg_response_time_ms * (self.stats.successful_requests - 1) as f64;
            self.stats.avg_response_time_ms = (total_time + response_time as f64) / self.stats.successful_requests as f64;
            
            self.consecutive_failures = 0;
            Ok(llm_response)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow!("LLM service error: {}", error_text))
        }
    }

    // Парсим ответ LLM в действие
    fn parse_llm_response(&self, response: &str) -> Result<AgentAction> {
        let trimmed = response.trim();
        
        // Простой парсер для действий
        if trimmed.starts_with("Move(") && trimmed.ends_with(")") {
            let coords = &trimmed[5..trimmed.len()-1];
            let parts: Vec<&str> = coords.split(',').collect();
            if parts.len() == 3 {
                let x: f32 = parts[0].trim().parse().unwrap_or(0.0);
                let y: f32 = parts[1].trim().parse().unwrap_or(0.0);
                let z: f32 = parts[2].trim().parse().unwrap_or(0.0);
                return Ok(AgentAction::Move(Vec3::new(x, y, z)));
            }
        }
        
        if trimmed.starts_with("Attack(") && trimmed.ends_with(")") {
            let target_str = &trimmed[7..trimmed.len()-1];
            if let Ok(target_id) = Uuid::parse_str(target_str) {
                return Ok(AgentAction::Attack(target_id));
            }
        }
        
        if trimmed.starts_with("Communicate(") && trimmed.ends_with(")") {
            let message = &trimmed[12..trimmed.len()-1];
            return Ok(AgentAction::Communicate(message.to_string()));
        }
        
        if trimmed.starts_with("UseItem(") && trimmed.ends_with(")") {
            let item = &trimmed[8..trimmed.len()-1];
            return Ok(AgentAction::UseItem(item.to_string()));
        }
        
        match trimmed.to_lowercase().as_str() {
            "wait" => Ok(AgentAction::Wait),
            "defend" => Ok(AgentAction::Defend),
            _ => {
                // Если не удалось распарсить, возвращаем случайное действие
                warn!("Failed to parse LLM response: {}", trimmed);
                Ok(AgentAction::Wait)
            }
        }
    }

    // Получаем резервное действие при сбое LLM
    fn get_fallback_action(&mut self) -> AgentAction {
        let action = self.fallback_actions[self.fallback_index].clone();
        self.fallback_index = (self.fallback_index + 1) % self.fallback_actions.len();
        action
    }

    // Обновляем историю разговора
    fn update_conversation_history(&mut self, state: &GameState, action: &AgentAction) {
        let entry = format!(
            "Tick {}: Health={:.1}, Pos=({:.1},{:.1}) -> {:?}",
            state.current_tick, state.health, 
            state.position.x, state.position.z, action
        );
        
        self.conversation_history.push(entry);
        
        // Ограничиваем размер истории
        if self.conversation_history.len() > self.max_history_size {
            self.conversation_history.remove(0);
        }
    }

    // Получаем статистику производительности
    pub fn get_stats(&self) -> &LLMStats {
        &self.stats
    }

    // Очищаем кеш и статистику
    pub fn reset_stats(&mut self) {
        self.stats = LLMStats::default();
        self.decision_cache = DecisionCache::new(100);
        self.conversation_history.clear();
        self.consecutive_failures = 0;
    }
}

#[async_trait]
impl AgentTrait for LLMAgent {
    async fn decide(&mut self, state: &GameState) -> Result<AgentAction> {
        // Создаем хеш состояния для кеширования
        let state_hash = self.create_state_hash(state);
        
        // Проверяем кеш
        if let Some(cached_action) = self.decision_cache.get(&state_hash) {
            return Ok(cached_action);
        }

        // Очищаем устаревшие записи кеша
        self.decision_cache.clear_expired();

        // Если слишком много неудачных попыток, используем резервное действие
        if self.consecutive_failures >= self.max_consecutive_failures {
            warn!("Too many consecutive failures, using fallback action");
            return Ok(self.get_fallback_action());
        }

        // Создаем промпт из состояния
        let prompt = self.state_to_prompt(state);
        
        // Запрашиваем LLM
        match self.query_llm(prompt).await {
            Ok(response) => {
                // Парсим ответ
                match self.parse_llm_response(&response.text) {
                    Ok(action) => {
                        // Сохраняем в кеш
                        self.decision_cache.insert(state_hash, action.clone(), 2); // TTL 2 секунды
                        
                        // Обновляем историю
                        self.update_conversation_history(state, &action);
                        
                        info!("LLM decision: {:?} ({}ms, {} tokens)", 
                              action, response.processing_time_ms, response.tokens_used);
                        
                        Ok(action)
                    }
                    Err(e) => {
                        self.consecutive_failures += 1;
                        self.stats.failed_requests += 1;
                        warn!("Failed to parse LLM response: {}", e);
                        Ok(self.get_fallback_action())
                    }
                }
            }
            Err(e) => {
                self.consecutive_failures += 1;
                self.stats.total_requests += 1;
                self.stats.failed_requests += 1;
                warn!("LLM query failed: {}", e);
                Ok(self.get_fallback_action())
            }
        }
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
        // Проверяем доступность LLM сервиса
        let health_check = self.client
            .get(&format!("{}/health", self.config.llm_service_url))
            .send()
            .await;
            
        match health_check {
            Ok(response) if response.status().is_success() => {
                info!("LLM service is available at {}", self.config.llm_service_url);
                Ok(())
            }
            _ => {
                warn!("LLM service is not available, agent will use fallback actions");
                Ok(())
            }
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        // Логируем финальную статистику
        info!("LLM Agent {} shutting down. Stats: {}/{} requests successful, {} tokens used, {:.1}ms avg response time",
              self.name, self.stats.successful_requests, self.stats.total_requests,
              self.stats.total_tokens_used, self.stats.avg_response_time_ms);
        Ok(())
    }

    async fn on_damage_received(&mut self, damage: f32, attacker_id: Uuid) -> Result<()> {
        // Добавляем контекст о получении урона в историю
        self.conversation_history.push(format!(
            "Received {} damage from {}", damage, attacker_id
        ));
        Ok(())
    }

    async fn on_kill(&mut self, victim_id: Uuid) -> Result<()> {
        self.conversation_history.push(format!(
            "Eliminated enemy {}", victim_id
        ));
        Ok(())
    }

    async fn on_death(&mut self) -> Result<()> {
        self.conversation_history.push("Agent died".to_string());
        Ok(())
    }

    async fn on_message(&mut self, sender_id: Uuid, message: &str) -> Result<()> {
        self.conversation_history.push(format!(
            "Message from {}: {}", sender_id, message
        ));
        Ok(())
    }
} 
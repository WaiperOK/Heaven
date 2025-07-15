use super::*;
use bevy::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;

// Менеджер агентов для управления всеми агентами в арене
pub struct AgentManager {
    pub agents: HashMap<Uuid, Box<dyn AgentTrait>>,
    pub active_agents: Vec<Uuid>,
    pub respawn_queue: Vec<(Uuid, f32)>, // agent_id, respawn_time
}

impl Default for AgentManager {
    fn default() -> Self {
        Self {
            agents: HashMap::new(),
            active_agents: Vec::new(),
            respawn_queue: Vec::new(),
        }
    }
}

impl AgentManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_agent(&mut self, agent: Box<dyn AgentTrait>) -> Result<()> {
        let agent_id = agent.get_id();
        self.agents.insert(agent_id, agent);
        self.active_agents.push(agent_id);
        Ok(())
    }

    pub fn remove_agent(&mut self, agent_id: &Uuid) -> Option<Box<dyn AgentTrait>> {
        self.active_agents.retain(|id| id != agent_id);
        self.agents.remove(agent_id)
    }

    pub fn get_agent(&self, agent_id: &Uuid) -> Option<&Box<dyn AgentTrait>> {
        self.agents.get(agent_id)
    }

    pub fn get_agent_mut(&mut self, agent_id: &Uuid) -> Option<&mut Box<dyn AgentTrait>> {
        self.agents.get_mut(agent_id)
    }

    pub fn get_active_agents(&self) -> &Vec<Uuid> {
        &self.active_agents
    }

    pub fn get_agent_count(&self) -> usize {
        self.active_agents.len()
    }

    pub fn is_agent_active(&self, agent_id: &Uuid) -> bool {
        self.active_agents.contains(agent_id)
    }

    pub fn deactivate_agent(&mut self, agent_id: &Uuid) {
        self.active_agents.retain(|id| id != agent_id);
    }

    pub fn activate_agent(&mut self, agent_id: &Uuid) {
        if self.agents.contains_key(agent_id) && !self.active_agents.contains(agent_id) {
            self.active_agents.push(*agent_id);
        }
    }

    pub fn clear_all(&mut self) {
        self.agents.clear();
        self.active_agents.clear();
        self.respawn_queue.clear();
    }

    pub async fn initialize_all_agents(&mut self) -> Result<()> {
        for agent in self.agents.values_mut() {
            agent.initialize().await?;
        }
        Ok(())
    }

    pub async fn shutdown_all_agents(&mut self) -> Result<()> {
        for agent in self.agents.values_mut() {
            agent.shutdown().await?;
        }
        Ok(())
    }
} 
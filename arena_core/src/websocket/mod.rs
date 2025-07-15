use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use serde_json::json;
use log::{info, warn, error};
use anyhow::Result;
use uuid::Uuid;

use crate::agents::AgentTrait;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentData {
    pub id: String,
    pub name: String,
    pub position: Position,
    pub health: f32,
    pub energy: f32,
    pub team: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArenaState {
    pub agents: Vec<AgentData>,
    pub match_id: String,
    pub current_tick: u64,
    pub match_time: f64,
    pub arena_bounds: Position,
    pub statistics: ArenaStatistics,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArenaStatistics {
    pub total_agents: usize,
    pub active_agents: usize,
    pub eliminated_agents: usize,
    pub average_health: f32,
    pub match_duration: f64,
}

pub struct WebSocketServer {
    sender: Arc<broadcast::Sender<String>>,
    listener: Option<TcpListener>,
}

impl WebSocketServer {
    pub async fn new(port: u16) -> Result<Self> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        info!("🚀 WebSocket server listening on {}", addr);
        
        let (sender, _) = broadcast::channel(100);
        
        Ok(Self {
            sender: Arc::new(sender),
            listener: Some(listener),
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        if let Some(listener) = self.listener.take() {
            let sender = Arc::clone(&self.sender);
            
            tokio::spawn(async move {
                while let Ok((stream, addr)) = listener.accept().await {
                    info!("🔗 New WebSocket connection from {}", addr);
                    let sender_clone = Arc::clone(&sender);
                    tokio::spawn(handle_connection(stream, sender_clone));
                }
            });
        }
        
        Ok(())
    }
    
    pub fn broadcast_arena_state(&self, state: &ArenaState) -> Result<()> {
        let json_data = serde_json::to_string(state)?;
        
        match self.sender.send(json_data) {
            Ok(receiver_count) => {
                if receiver_count > 0 {
                    info!("📡 Broadcasted arena state to {} viewers", receiver_count);
                }
            }
            Err(e) => warn!("Failed to broadcast arena state: {}", e),
        }
        
        Ok(())
    }
    
    pub fn get_viewer_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

async fn handle_connection(
    stream: TcpStream,
    sender: Arc<broadcast::Sender<String>>,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws_stream) => ws_stream,
        Err(e) => {
            error!("❌ WebSocket handshake failed: {}", e);
            return;
        }
    };
    
    info!("✅ WebSocket connection established");
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut receiver = sender.subscribe();
    
    // Отправляем приветственное сообщение
    let welcome_msg = json!({
        "type": "welcome",
        "message": "Connected to Heaven AI Arena",
        "version": "0.1.0"
    });
    
    if let Err(e) = ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(
        welcome_msg.to_string()
    )).await {
        error!("Failed to send welcome message: {}", e);
        return;
    }
    
    // Задача для отправки обновлений состояния арены
    let send_task = tokio::spawn(async move {
        while let Ok(message) = receiver.recv().await {
            if let Err(e) = ws_sender.send(
                tokio_tungstenite::tungstenite::Message::Text(message)
            ).await {
                warn!("📡 Failed to send message to viewer: {}", e);
                break;
            }
        }
    });
    
    // Задача для обработки входящих сообщений от клиента
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                    // Обрабатываем команды от Godot viewer
                    if let Ok(command) = serde_json::from_str::<serde_json::Value>(&text) {
                        handle_viewer_command(command).await;
                    }
                }
                Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                    info!("👋 WebSocket connection closed by client");
                    break;
                }
                Err(e) => {
                    warn!("❌ WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });
    
    // Ждем завершения любой из задач
    tokio::select! {
        _ = send_task => info!("📤 Send task completed"),
        _ = receive_task => info!("📥 Receive task completed"),
    }
    
    info!("🔌 WebSocket connection closed");
}

async fn handle_viewer_command(command: serde_json::Value) {
    info!("📨 Received command from viewer: {}", command);
    
    // Здесь можно обрабатывать команды от Godot viewer
    // Например: pause/resume, camera positions, agent selection, etc.
    match command.get("type").and_then(|t| t.as_str()) {
        Some("pause_simulation") => {
            info!("⏸️ Pause simulation requested");
            // TODO: Implement pause logic
        }
        Some("resume_simulation") => {
            info!("▶️ Resume simulation requested");
            // TODO: Implement resume logic  
        }
        Some("reset_simulation") => {
            info!("🔄 Reset simulation requested");
            // TODO: Implement reset logic
        }
        Some("select_agent") => {
            if let Some(agent_id) = command.get("agent_id").and_then(|id| id.as_str()) {
                info!("👆 Agent selected: {}", agent_id);
                // TODO: Implement agent selection logic
            }
        }
        _ => {
            warn!("❓ Unknown command type: {:?}", command.get("type"));
        }
    }
}
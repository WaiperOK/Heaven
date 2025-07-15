use bevy::prelude::*;

// Базовый плагин для игрового состояния
pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, game_state_system);
    }
}

fn game_state_system() {
    // Placeholder для системы игрового состояния
} 
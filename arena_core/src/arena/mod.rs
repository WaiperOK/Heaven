use bevy::prelude::*;

// Базовый плагин для арены
pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, arena_system);
    }
}

fn arena_system() {
    // Placeholder для системы арены
} 
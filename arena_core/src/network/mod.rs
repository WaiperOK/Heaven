use bevy::prelude::*;

// Базовый плагин для сетевого взаимодействия
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, network_system);
    }
}

fn network_system() {
    // Placeholder для системы сетевого взаимодействия
} 
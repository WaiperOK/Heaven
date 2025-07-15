use bevy::prelude::*;

// Базовый плагин для пользовательского интерфейса
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_system);
    }
}

fn ui_system() {
    // Placeholder для системы пользовательского интерфейса
} 
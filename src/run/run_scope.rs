use bevy::prelude::*;

use crate::game_state::GameState;

#[derive(Component)]
pub struct RunScoped;

pub fn register(app: &mut App) {
    app.add_systems(OnExit(GameState::Playing), despawn_run_scoped);
}

fn despawn_run_scoped(mut commands: Commands, query: Query<Entity, With<RunScoped>>) {
    for entity in &query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}

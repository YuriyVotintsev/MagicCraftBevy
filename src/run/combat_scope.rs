use bevy::prelude::*;

use crate::wave::WavePhase;

#[derive(Component)]
pub struct CombatScoped;

#[derive(Component)]
pub struct SkipDeathShrink;

pub fn register(app: &mut App) {
    app.add_systems(OnExit(WavePhase::Combat), despawn_combat_scoped);
}

fn despawn_combat_scoped(
    mut commands: Commands,
    query: Query<Entity, With<CombatScoped>>,
) {
    for entity in &query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}

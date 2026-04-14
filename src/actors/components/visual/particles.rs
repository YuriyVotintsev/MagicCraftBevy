use bevy::prelude::*;

use crate::actors::SpawnSource;
use crate::GameState;

#[derive(Component)]
pub struct Particles {
    pub config: String,
    pub position: Vec2,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        spawn_particles.run_if(in_state(GameState::Playing)),
    );
}

fn spawn_particles(
    mut commands: Commands,
    query: Query<(Entity, &SpawnSource, &Particles), Added<Particles>>,
) {
    for (entity, _source, particles) in &query {
        crate::particles::start_particles(&mut commands, &particles.config, particles.position);
        commands.entity(entity).despawn();
    }
}

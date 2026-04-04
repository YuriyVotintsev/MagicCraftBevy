use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::SpawnSource;
use crate::GameState;

#[blueprint_component]
pub struct Particles {
    pub config: String,
    #[default_expr("target.position")]
    pub position: VecExpr,
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

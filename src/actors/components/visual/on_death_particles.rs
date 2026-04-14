use bevy::prelude::*;

use super::super::combat::Dead;
use crate::particles;

#[derive(Component, Debug, Clone)]
pub struct OnDeathParticles {
    pub config: &'static str,
}

pub fn register_systems(app: &mut App) {
    app.add_observer(on_death_particles_observer);
}

fn on_death_particles_observer(
    on: On<Add, Dead>,
    mut commands: Commands,
    query: Query<(&OnDeathParticles, &Transform)>,
) {
    let entity = on.event_target();
    let Ok((effect, transform)) = query.get(entity) else { return };
    let pos = crate::coord::to_2d(transform.translation);
    particles::start_particles(&mut commands, effect.config, pos);
}

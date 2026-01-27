use avian2d::prelude::*;
use bevy::prelude::*;

use crate::wave::remove_invulnerability;
use crate::MovementLocked;

use super::effects::{Dashing, PreDashLayers};

pub fn update_dashing(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Dashing, &mut LinearVelocity, &PreDashLayers)>,
) {
    for (entity, mut dashing, mut velocity, pre_dash_layers) in &mut query {
        velocity.0 = dashing.direction * dashing.speed;

        if dashing.timer.tick(time.delta()).just_finished() {
            let restored_layers = pre_dash_layers.0;
            commands
                .entity(entity)
                .remove::<(Dashing, MovementLocked, PreDashLayers)>()
                .insert(restored_layers);
            remove_invulnerability(&mut commands, entity);
        }
    }
}

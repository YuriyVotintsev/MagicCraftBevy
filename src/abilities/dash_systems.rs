use avian2d::prelude::*;
use bevy::prelude::*;

use crate::wave::remove_invulnerability;
use crate::MovementLocked;

use super::effects::Dashing;

pub fn update_dashing(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Dashing, &mut LinearVelocity)>,
) {
    for (entity, mut dashing, mut velocity) in &mut query {
        velocity.0 = dashing.direction * dashing.speed;

        if dashing.timer.tick(time.delta()).just_finished() {
            commands.entity(entity).remove::<(Dashing, MovementLocked)>();
            remove_invulnerability(&mut commands, entity);
        }
    }
}

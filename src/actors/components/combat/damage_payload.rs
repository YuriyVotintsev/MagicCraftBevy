use bevy::prelude::*;

use super::Caster;
use crate::schedule::GameSet;
use super::PendingDamage;
use crate::GameState;

#[derive(Component)]
pub struct DamagePayload {
    pub amount: f32,
    pub target: Entity,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        process_damage_payloads
            .in_set(GameSet::Damage)
            .run_if(in_state(GameState::Playing)),
    );
}

fn process_damage_payloads(
    mut commands: Commands,
    mut pending: MessageWriter<PendingDamage>,
    query: Query<(Entity, &DamagePayload, &Caster)>,
) {
    for (entity, payload, caster) in &query {
        pending.write(PendingDamage {
            target: payload.target,
            amount: payload.amount,
            source: Some(caster.0),
            on_hit: Default::default(),
        });
        commands.entity(entity).despawn();
    }
}

use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::SpawnSource;
use crate::schedule::GameSet;
use crate::stats::PendingDamage;
use crate::GameState;

#[blueprint_component]
pub struct DamagePayload {
    pub amount: ScalarExpr,
    #[default_expr("target.entity")]
    pub target: EntityExpr,
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
    query: Query<(Entity, &DamagePayload, &SpawnSource)>,
) {
    for (entity, payload, source) in &query {
        pending.write(PendingDamage {
            target: payload.target,
            amount: payload.amount,
            source: source.caster.entity,
        });
        commands.entity(entity).despawn();
    }
}

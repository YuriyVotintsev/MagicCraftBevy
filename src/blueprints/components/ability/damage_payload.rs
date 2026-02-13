use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

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
    query: Query<(Entity, &DamagePayload)>,
) {
    let mut damage_map = bevy::platform::collections::HashMap::<Entity, f32>::default();
    for (entity, payload) in &query {
        *damage_map.entry(payload.target).or_default() += payload.amount;
        commands.entity(entity).despawn();
    }
    for (target, total) in damage_map {
        if let Ok(mut target_commands) = commands.get_entity(target) {
            target_commands.insert(PendingDamage(total));
        }
    }
}

use bevy::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_activator;
use crate::abilities::expr::ScalarExpr;
use crate::abilities::eval_context::EvalContext;
use crate::abilities::{ActivateAbilityEvent, AbilityContext, TargetInfo, ProvidedFields, AbilityInputs, AbilityInstance};
use crate::stats::ComputedStats;
use crate::schedule::GameSet;
use crate::{Faction, GameState};

#[derive(Debug, Clone, Default, GenerateRaw)]
#[activator]
pub struct WhileHeldParams {
    pub interval: ScalarExpr,
}

#[derive(Component)]
pub struct WhileHeldActivator {
    pub interval: ScalarExpr,
    pub timer: f32,
}

impl WhileHeldActivator {
    pub fn from_params(params: &WhileHeldParams) -> Self {
        Self {
            interval: params.interval.clone(),
            timer: 0.0,
        }
    }
}

pub fn provided_fields() -> ProvidedFields {
    ProvidedFields::SOURCE_ENTITY
        .union(ProvidedFields::SOURCE_POSITION)
        .union(ProvidedFields::TARGET_DIRECTION)
}

fn while_held_system(
    time: Res<Time>,
    mut trigger_events: MessageWriter<ActivateAbilityEvent>,
    mut ability_query: Query<(&AbilityInstance, &mut WhileHeldActivator)>,
    owner_query: Query<(&AbilityInputs, &Transform, &Faction, &ComputedStats)>,
) {
    let delta = time.delta_secs();

    for (instance, mut activator) in &mut ability_query {
        let Ok((inputs, transform, faction, stats)) = owner_query.get(instance.owner) else {
            continue;
        };

        let Some(input) = inputs.get(instance.ability_id) else { continue };

        if !input.pressed {
            activator.timer = 0.0;
            continue;
        }

        activator.timer -= delta;
        if activator.timer > 0.0 { continue }

        let source = TargetInfo::from_entity_and_position(instance.owner, transform.translation.truncate());
        let target = TargetInfo::from_direction(input.direction.truncate());

        let ctx = AbilityContext::new(
            instance.owner,
            *faction,
            source,
            target,
        );

        trigger_events.write(ActivateAbilityEvent {
            ability_id: instance.ability_id,
            context: ctx,
        });

        activator.timer = activator.interval.eval(&EvalContext::stats_only(stats));
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        while_held_system
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

register_activator!(WhileHeldParams, WhileHeldActivator);

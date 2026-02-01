use bevy::prelude::*;
use std::collections::HashMap;
use crate::register_activator;
use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::{TriggerAbilityEvent, AbilityContext, Target, AbilityInputs, AbilityInstance};
use crate::stats::{ComputedStats, StatRegistry};
use crate::schedule::GameSet;
use crate::{Faction, GameState};

#[derive(Debug, Clone)]
pub struct WhileHeldParams {
    pub interval: ParamValue,
}

impl WhileHeldParams {
    pub fn parse(raw: &HashMap<String, ParamValueRaw>, stat_registry: &StatRegistry) -> Self {
        Self {
            interval: raw.get("interval")
                .map(|v| resolve_param_value(v, stat_registry))
                .expect("while_held requires 'interval' parameter"),
        }
    }
}

#[derive(Component)]
pub struct WhileHeldActivator {
    pub interval: ParamValue,
    pub timer: f32,
}

impl WhileHeldActivator {
    pub fn from_params_impl(params: &WhileHeldParams) -> Self {
        Self {
            interval: params.interval.clone(),
            timer: 0.0,
        }
    }
}

fn while_held_system(
    time: Res<Time>,
    mut trigger_events: MessageWriter<TriggerAbilityEvent>,
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

        let ctx = AbilityContext::new(
            instance.owner,
            *faction,
            Target::Point(transform.translation),
            Some(Target::Direction(input.direction)),
        );

        trigger_events.write(TriggerAbilityEvent {
            ability_id: instance.ability_id,
            context: ctx,
        });

        activator.timer = activator.interval.evaluate_f32(stats);
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

register_activator!(WhileHeldActivator, params: WhileHeldParams, name: "while_held");

use bevy::prelude::*;
use std::collections::HashMap;
use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::{TriggerAbilityEvent, AbilityContext, Target};
use crate::stats::{ComputedStats, StatRegistry};
use crate::schedule::GameSet;
use crate::{Faction, GameState};

use super::AbilityInstance;

#[derive(Debug, Clone)]
pub struct IntervalParams {
    pub interval: ParamValue,
    pub skip_first: bool,
}

impl IntervalParams {
    pub fn parse(raw: &HashMap<String, ParamValueRaw>, stat_registry: &StatRegistry) -> Self {
        Self {
            interval: raw.get("interval")
                .map(|v| resolve_param_value(v, stat_registry))
                .expect("interval requires 'interval' parameter"),
            skip_first: raw.get("skip_first")
                .and_then(|v| match v {
                    ParamValueRaw::Bool(b) => Some(*b),
                    _ => None,
                })
                .unwrap_or(false),
        }
    }
}

#[derive(Component)]
pub struct IntervalActivator {
    pub interval: ParamValue,
    pub timer: f32,
    pub skip_first: bool,
    pub activated: bool,
}

impl IntervalActivator {
    pub fn from_params_impl(params: &IntervalParams) -> Self {
        Self {
            interval: params.interval.clone(),
            timer: 0.0,
            skip_first: params.skip_first,
            activated: false,
        }
    }
}

fn interval_system(
    time: Res<Time>,
    mut trigger_events: MessageWriter<TriggerAbilityEvent>,
    mut ability_query: Query<(&AbilityInstance, &mut IntervalActivator)>,
    owner_query: Query<(&Transform, &Faction, &ComputedStats)>,
) {
    let delta = time.delta_secs();

    for (instance, mut activator) in &mut ability_query {
        if activator.skip_first && !activator.activated {
            let Ok((_, _, stats)) = owner_query.get(instance.owner) else { continue };
            activator.timer = activator.interval.evaluate_f32(stats);
            activator.activated = true;
            continue;
        }

        activator.timer -= delta;
        if activator.timer > 0.0 { continue }

        let Ok((transform, faction, stats)) = owner_query.get(instance.owner) else { continue };

        let ctx = AbilityContext::new(
            instance.owner,
            *faction,
            Target::Point(transform.translation),
            None,
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
        interval_system
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

register_activator!(IntervalActivator, params: IntervalParams, name: "interval");

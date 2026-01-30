use std::collections::HashMap;
use bevy::prelude::*;

use crate::abilities::ids::ParamId;
use crate::abilities::effect_def::ParamValue;
use crate::abilities::registry::TriggerHandler;
use crate::abilities::{AbilityId, AbilityRegistry, TriggerRegistry, AbilityContext, TriggerAbilityEvent};
use crate::schedule::GameSet;
use crate::Faction;
use crate::GameState;

#[derive(Component, Default)]
pub struct IntervalTriggers {
    pub entries: Vec<IntervalEntry>,
}

pub struct IntervalEntry {
    pub ability_id: AbilityId,
    pub interval: ParamValue,
    pub timer: f32,
}

impl IntervalTriggers {
    pub fn add(&mut self, ability_id: AbilityId, interval: ParamValue) {
        let initial_timer = interval.as_float().unwrap_or(1.0);
        self.entries.push(IntervalEntry {
            ability_id,
            interval,
            timer: initial_timer,
        });
    }
}

pub fn interval_system(
    mut trigger_events: MessageWriter<TriggerAbilityEvent>,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut IntervalTriggers,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
) {
    let delta = time.delta_secs();

    for (entity, mut triggers, transform, faction) in &mut query {
        for entry in &mut triggers.entries {
            entry.timer -= delta;

            if entry.timer > 0.0 {
                continue;
            }

            let Some(_ability_def) = ability_registry.get(entry.ability_id) else {
                continue;
            };

            let ctx = AbilityContext::new(
                entity,
                *faction,
                transform.translation,
            );

            trigger_events.write(TriggerAbilityEvent {
                ability_id: entry.ability_id,
                context: ctx,
            });

            entry.timer = entry.interval.as_float().unwrap_or(1.0);
        }
    }
}

#[derive(Default)]
pub struct IntervalHandler;

impl TriggerHandler for IntervalHandler {
    fn name(&self) -> &'static str {
        "interval"
    }

    fn add_to_entity(
        &self,
        commands: &mut Commands,
        entity: Entity,
        ability_id: AbilityId,
        params: &HashMap<ParamId, ParamValue>,
        registry: &TriggerRegistry,
    ) {
        let interval = registry
            .get_param_id("interval")
            .and_then(|id| params.get(&id).cloned())
            .unwrap_or(ParamValue::Float(1.0));
        commands
            .entity(entity)
            .entry::<IntervalTriggers>()
            .or_default()
            .and_modify(move |mut a| a.add(ability_id, interval));
    }

    fn register_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            interval_system
                .in_set(GameSet::AbilityActivation)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

register_trigger!(IntervalHandler);

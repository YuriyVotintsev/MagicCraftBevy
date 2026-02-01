use std::collections::HashMap;
use bevy::prelude::*;
use crate::register_node;

use crate::abilities::ids::AbilityId;
use crate::abilities::{ParamValue, ParamValueRaw, ParseNodeParams, resolve_param_value, NodeParams};
use crate::abilities::node::{NodeHandler, NodeKind, NodeRegistry};
use crate::abilities::{AbilityRegistry, AbilityContext, TriggerAbilityEvent, Target};
use crate::stats::StatRegistry;
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
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
    pub skip_first: bool,
}

impl IntervalTriggers {
    pub fn add(&mut self, ability_id: AbilityId, interval: ParamValue, skip_first: bool) {
        self.entries.push(IntervalEntry {
            ability_id,
            interval,
            timer: 0.0,
            skip_first,
        });
    }
}

#[derive(Debug, Clone)]
pub struct IntervalParams {
    pub interval: ParamValue,
    pub skip_first: bool,
}

impl ParseNodeParams for IntervalParams {
    fn parse(raw: &HashMap<String, ParamValueRaw>, stat_registry: &StatRegistry) -> Self {
        Self {
            interval: raw.get("interval")
                .map(|v| resolve_param_value(v, stat_registry))
                .unwrap_or(ParamValue::Float(1.0)),
            skip_first: raw.get("skip_first")
                .and_then(|v| match v {
                    ParamValueRaw::Bool(b) => Some(*b),
                    _ => None,
                })
                .unwrap_or(true),
        }
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
        &ComputedStats,
    )>,
    ability_registry: Res<AbilityRegistry>,
) {
    let delta = time.delta_secs();

    for (entity, mut triggers, transform, faction, stats) in &mut query {
        for entry in &mut triggers.entries {
            if entry.skip_first {
                entry.timer = entry.interval.evaluate_f32(stats);
                entry.skip_first = false;
                continue;
            }

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
                Target::Point(transform.translation),
                None,
            );

            trigger_events.write(TriggerAbilityEvent {
                ability_id: entry.ability_id,
                context: ctx,
            });

            entry.timer = entry.interval.evaluate_f32(stats);
        }
    }
}

#[derive(Default)]
pub struct IntervalHandler;


impl NodeHandler for IntervalHandler {
    fn name(&self) -> &'static str {
        "interval"
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Trigger
    }

    fn add_to_entity(
        &self,
        commands: &mut Commands,
        entity: Entity,
        ability_id: AbilityId,
        params: &NodeParams,
        _registry: &NodeRegistry,
    ) {
        let p = params.expect_trigger().expect_interval();
        let interval = p.interval.clone();
        let skip_first = p.skip_first;

        commands
            .entity(entity)
            .entry::<IntervalTriggers>()
            .or_default()
            .and_modify(move |mut a| a.add(ability_id, interval, skip_first));
    }

    fn register_input_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            interval_system
                .in_set(GameSet::AbilityActivation)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

register_node!(IntervalHandler, params: IntervalParams, name: "interval");

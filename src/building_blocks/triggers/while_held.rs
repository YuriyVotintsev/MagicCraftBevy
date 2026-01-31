use std::collections::HashMap;
use bevy::prelude::*;
use crate::register_node;

use crate::abilities::ids::{ParamId, AbilityId};
use crate::abilities::param::ParamValue;
use crate::abilities::node::{NodeHandler, NodeKind, NodeRegistry, AbilityRegistry};
use crate::abilities::{TriggerAbilityEvent, AbilityInputs, AbilityContext};
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::Faction;
use crate::GameState;

#[derive(Component, Default)]
pub struct WhileHeldTriggers {
    pub entries: Vec<WhileHeldEntry>,
}

pub struct WhileHeldEntry {
    pub ability_id: AbilityId,
    pub cooldown: ParamValue,
    pub timer: f32,
}

impl WhileHeldTriggers {
    pub fn add(&mut self, ability_id: AbilityId, cooldown: ParamValue) {
        self.entries.push(WhileHeldEntry {
            ability_id,
            cooldown,
            timer: 0.0,
        });
    }
}

pub fn while_held_system(
    mut trigger_events: MessageWriter<TriggerAbilityEvent>,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut WhileHeldTriggers,
        &AbilityInputs,
        &Transform,
        &Faction,
        &ComputedStats,
    )>,
    ability_registry: Res<AbilityRegistry>,
) {
    let delta = time.delta_secs();

    for (entity, mut triggers, inputs, transform, faction, stats) in &mut query {
        for entry in &mut triggers.entries {
            entry.timer = (entry.timer - delta).max(0.0);

            let Some(input) = inputs.get(entry.ability_id) else {
                continue;
            };

            if !input.pressed {
                continue;
            }

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
            )
            .with_target_direction(input.direction)
            .with_target_point(input.point);

            trigger_events.write(TriggerAbilityEvent {
                ability_id: entry.ability_id,
                context: ctx,
            });

            entry.timer = entry.cooldown.evaluate_f32(stats);
        }
    }
}

#[derive(Default)]
pub struct WhileHeldHandler;


impl NodeHandler for WhileHeldHandler {
    fn name(&self) -> &'static str {
        "while_held"
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Trigger
    }

    fn add_to_entity(
        &self,
        commands: &mut Commands,
        entity: Entity,
        ability_id: AbilityId,
        params: &HashMap<ParamId, ParamValue>,
        registry: &NodeRegistry,
    ) {
        let cooldown = registry
            .get_param_id("cooldown")
            .and_then(|id| params.get(&id).cloned())
            .unwrap_or(ParamValue::Float(0.05));
        commands
            .entity(entity)
            .entry::<WhileHeldTriggers>()
            .or_default()
            .and_modify(move |mut a| a.add(ability_id, cooldown));
    }

    fn register_input_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            while_held_system
                .in_set(GameSet::AbilityActivation)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

register_node!(WhileHeldHandler);

use std::collections::HashMap;
use bevy::prelude::*;
use crate::register_node;

use crate::abilities::ids::{ParamId, AbilityId};
use crate::abilities::param::ParamValue;
use crate::abilities::node::{NodeHandler, NodeKind, NodeRegistry, AbilityRegistry};
use crate::abilities::{TriggerAbilityEvent, AbilityInputs, AbilityContext, Target};
use crate::schedule::GameSet;
use crate::Faction;
use crate::GameState;

#[derive(Component, Default)]
pub struct OnInputTriggers {
    pub entries: Vec<OnInputEntry>,
}

pub struct OnInputEntry {
    pub ability_id: AbilityId,
}

impl OnInputTriggers {
    pub fn add(&mut self, ability_id: AbilityId) {
        self.entries.push(OnInputEntry { ability_id });
    }
}

pub fn on_input_system(
    mut trigger_events: MessageWriter<TriggerAbilityEvent>,
    query: Query<(
        Entity,
        &OnInputTriggers,
        &AbilityInputs,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
) {
    for (entity, triggers, inputs, transform, faction) in &query {
        for entry in &triggers.entries {
            let Some(input) = inputs.get(entry.ability_id) else {
                continue;
            };

            if !input.just_pressed {
                continue;
            }

            let Some(_ability_def) = ability_registry.get(entry.ability_id) else {
                continue;
            };

            let ctx = AbilityContext::new(
                entity,
                *faction,
                Target::Point(transform.translation),
                Some(Target::Direction(input.direction)),
            );

            trigger_events.write(TriggerAbilityEvent {
                ability_id: entry.ability_id,
                context: ctx,
            });
        }
    }
}

#[derive(Default)]
pub struct OnInputHandler;


impl NodeHandler for OnInputHandler {
    fn name(&self) -> &'static str {
        "on_input"
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Trigger
    }

    fn add_to_entity(
        &self,
        commands: &mut Commands,
        entity: Entity,
        ability_id: AbilityId,
        _params: &HashMap<ParamId, ParamValue>,
        _registry: &NodeRegistry,
    ) {
        commands
            .entity(entity)
            .entry::<OnInputTriggers>()
            .or_default()
            .and_modify(move |mut a| a.add(ability_id));
    }

    fn register_input_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            on_input_system
                .in_set(GameSet::AbilityActivation)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

register_node!(OnInputHandler);

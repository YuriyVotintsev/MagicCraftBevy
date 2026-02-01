use bevy::prelude::*;
use crate::register_node;

use crate::abilities::ids::AbilityId;
use crate::abilities::{NodeParams, NoParams};
use crate::abilities::node::{NodeHandler, NodeKind, NodeRegistry, AbilityRegistry};
use crate::abilities::{TriggerAbilityEvent, AbilityContext, Target};
use crate::schedule::GameSet;
use crate::Faction;
use crate::GameState;

#[derive(Component, Default)]
pub struct EveryFrameTriggers {
    pub entries: Vec<EveryFrameEntry>,
}

pub struct EveryFrameEntry {
    pub ability_id: AbilityId,
    pub activated: bool,
}

impl EveryFrameTriggers {
    pub fn add(&mut self, ability_id: AbilityId) {
        self.entries.push(EveryFrameEntry {
            ability_id,
            activated: false,
        });
    }
}

pub fn every_frame_system(
    mut trigger_events: MessageWriter<TriggerAbilityEvent>,
    mut query: Query<(
        Entity,
        &mut EveryFrameTriggers,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
) {
    for (entity, mut triggers, transform, faction) in &mut query {
        for entry in &mut triggers.entries {
            if entry.activated {
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

            entry.activated = true;
        }
    }
}

#[derive(Default)]
pub struct EveryFrameHandler;

impl NodeHandler for EveryFrameHandler {
    fn name(&self) -> &'static str {
        "every_frame"
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Trigger
    }

    fn add_to_entity(
        &self,
        commands: &mut Commands,
        entity: Entity,
        ability_id: AbilityId,
        _params: &NodeParams,
        _registry: &NodeRegistry,
    ) {
        commands
            .entity(entity)
            .entry::<EveryFrameTriggers>()
            .or_default()
            .and_modify(move |mut a| a.add(ability_id));
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        every_frame_system
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

register_node!(EveryFrameHandler, params: NoParams, name: "every_frame", systems: register_systems);

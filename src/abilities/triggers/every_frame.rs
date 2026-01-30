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
                transform.translation,
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

impl TriggerHandler for EveryFrameHandler {
    fn name(&self) -> &'static str {
        "every_frame"
    }

    fn add_to_entity(
        &self,
        commands: &mut Commands,
        entity: Entity,
        ability_id: AbilityId,
        _params: &HashMap<ParamId, ParamValue>,
        _registry: &TriggerRegistry,
    ) {
        commands
            .entity(entity)
            .entry::<EveryFrameTriggers>()
            .or_default()
            .and_modify(move |mut a| a.add(ability_id));
    }

    fn register_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            every_frame_system
                .in_set(GameSet::AbilityActivation)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

register_trigger!(EveryFrameHandler);

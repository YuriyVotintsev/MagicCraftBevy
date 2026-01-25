use bevy::prelude::*;

use super::behaviour::{MoveTowardPlayer, UseAbilities};
use super::components::{CurrentState, MobType};
use super::events::StateTransition;
use super::registry::MobRegistry;
use super::transitions::{AfterTime, WhenNear};
use super::types::{BehaviourDef, MobDef, TransitionDef};

pub fn fsm_transition_system(
    mut commands: Commands,
    mut events: MessageReader<StateTransition>,
    mob_registry: Res<MobRegistry>,
    mut query: Query<(&MobType, &mut CurrentState)>,
) {
    for event in events.read() {
        let Ok((mob_type, mut current_state)) = query.get_mut(event.entity) else {
            continue;
        };

        let Some(mob_def) = mob_registry.get(&mob_type.0) else {
            continue;
        };

        let old_state_name = current_state.0.clone();
        let new_state_name = &event.to;

        if old_state_name == *new_state_name {
            continue;
        }

        if let Some(old_state) = mob_def.states.get(&old_state_name) {
            remove_state_components(&mut commands, event.entity, mob_def, old_state);
        }

        if let Some(new_state) = mob_def.states.get(new_state_name) {
            add_state_components(&mut commands, event.entity, mob_def, new_state);
        }

        current_state.0 = new_state_name.clone();

        println!(
            "FSM: {} transitioned from '{}' to '{}'",
            mob_def.name, old_state_name, new_state_name
        );
    }
}

fn remove_state_components(
    commands: &mut Commands,
    entity: Entity,
    _mob_def: &MobDef,
    state: &super::types::StateDef,
) {
    let mut entity_commands = commands.entity(entity);

    for behaviour in &state.behaviour {
        match behaviour {
            BehaviourDef::MoveTowardPlayer => {
                entity_commands.remove::<MoveTowardPlayer>();
            }
            BehaviourDef::UseAbilities(_) => {
                entity_commands.remove::<UseAbilities>();
            }
        }
    }

    for transition in &state.transitions {
        match transition {
            TransitionDef::WhenNear(_) => {
                entity_commands.remove::<WhenNear>();
            }
            TransitionDef::AfterTime(_, _) => {
                entity_commands.remove::<AfterTime>();
            }
        }
    }
}

pub fn add_state_components(
    commands: &mut Commands,
    entity: Entity,
    _mob_def: &MobDef,
    state: &super::types::StateDef,
) {
    let mut entity_commands = commands.entity(entity);

    for behaviour in &state.behaviour {
        match behaviour {
            BehaviourDef::MoveTowardPlayer => {
                entity_commands.insert(MoveTowardPlayer);
            }
            BehaviourDef::UseAbilities(abilities) => {
                entity_commands.insert(UseAbilities::new(abilities.clone()));
            }
        }
    }

    for transition in &state.transitions {
        match transition {
            TransitionDef::WhenNear(targets) => {
                entity_commands.insert(WhenNear::new(targets.clone()));
            }
            TransitionDef::AfterTime(target, duration) => {
                entity_commands.insert(AfterTime::new(target.clone(), *duration));
            }
        }
    }
}

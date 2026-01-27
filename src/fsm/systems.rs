use bevy::prelude::*;

use super::behaviour_registry::BehaviourRegistry;
use super::components::{CurrentState, MobType};
use super::events::StateTransition;
use super::registry::MobRegistry;
use super::transition_registry::TransitionRegistry;
use super::types::{MobDef, StateDef};

pub fn fsm_transition_system(
    mut commands: Commands,
    mut events: MessageReader<StateTransition>,
    mob_registry: Res<MobRegistry>,
    behaviour_registry: Res<BehaviourRegistry>,
    transition_registry: Res<TransitionRegistry>,
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
            remove_state_components(
                &mut commands,
                event.entity,
                mob_def,
                old_state,
                &behaviour_registry,
                &transition_registry,
            );
        }

        if let Some(new_state) = mob_def.states.get(new_state_name) {
            add_state_components(
                &mut commands,
                event.entity,
                mob_def,
                new_state,
                &behaviour_registry,
                &transition_registry,
            );
        }

        current_state.0 = new_state_name.clone();

        info!(
            "FSM: {} transitioned from '{}' to '{}'",
            mob_def.name, old_state_name, new_state_name
        );
    }
}

fn remove_state_components(
    commands: &mut Commands,
    entity: Entity,
    _mob_def: &MobDef,
    state: &StateDef,
    behaviour_registry: &BehaviourRegistry,
    transition_registry: &TransitionRegistry,
) {
    for behaviour in &state.behaviour {
        behaviour_registry.remove(commands, entity, behaviour);
    }

    for transition in &state.transitions {
        transition_registry.remove(commands, entity, transition);
    }
}

pub fn add_state_components(
    commands: &mut Commands,
    entity: Entity,
    _mob_def: &MobDef,
    state: &StateDef,
    behaviour_registry: &BehaviourRegistry,
    transition_registry: &TransitionRegistry,
) {
    for behaviour in &state.behaviour {
        behaviour_registry.add(commands, entity, behaviour);
    }

    for transition in &state.transitions {
        transition_registry.add(commands, entity, transition);
    }
}

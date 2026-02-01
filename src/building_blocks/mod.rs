pub mod actions;
pub mod activators;
pub mod behaviours;
mod node_params;
pub mod transitions;
pub mod triggers;

pub use node_params::{NodeParams, NodeParamsRaw};

pub use behaviours::{keep_distance_system, move_toward_player_system, use_abilities_system, KeepDistance, MoveTowardPlayer, UseAbilities};
pub use transitions::{after_time_system, when_near_system, AfterTime, WhenNear};

use bevy::prelude::*;

use crate::fsm::{BehaviourDef, BehaviourRegistry, TransitionDef, TransitionRegistry};
use crate::schedule::GameSet;

pub struct MobAiPlugin;

impl Plugin for MobAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_behaviours)
            .add_systems(
                Update,
                (
                    move_toward_player_system,
                    keep_distance_system,
                    use_abilities_system,
                    when_near_system,
                    after_time_system,
                )
                    .in_set(GameSet::MobAI),
            );
    }
}

fn register_behaviours(
    mut behaviour_registry: ResMut<BehaviourRegistry>,
    mut transition_registry: ResMut<TransitionRegistry>,
) {
    behaviour_registry.register(
        "MoveTowardPlayer",
        |commands, entity, _| {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.insert(MoveTowardPlayer);
            }
        },
        |commands, entity| {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<MoveTowardPlayer>();
            }
        },
    );

    behaviour_registry.register(
        "UseAbilities",
        |commands, entity, behaviour| {
            if let BehaviourDef::UseAbilities { abilities, cooldown } = behaviour {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.insert(UseAbilities::new(abilities.clone()).with_cooldown(*cooldown));
                }
            }
        },
        |commands, entity| {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<UseAbilities>();
            }
        },
    );

    behaviour_registry.register(
        "KeepDistance",
        |commands, entity, behaviour| {
            if let BehaviourDef::KeepDistance { min, max } = behaviour {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.insert(KeepDistance::new(*min, *max));
                }
            }
        },
        |commands, entity| {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<KeepDistance>();
            }
        },
    );

    transition_registry.register(
        "WhenNear",
        |commands, entity, transition| {
            if let TransitionDef::WhenNear(targets) = transition {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.insert(WhenNear::new(targets.clone()));
                }
            }
        },
        |commands, entity| {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<WhenNear>();
            }
        },
    );

    transition_registry.register(
        "AfterTime",
        |commands, entity, transition| {
            if let TransitionDef::AfterTime(target, duration) = transition {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.insert(AfterTime::new(target.clone(), *duration));
                }
            }
        },
        |commands, entity| {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<AfterTime>();
            }
        },
    );
}

use bevy::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_activator;
use crate::abilities::{ActivateAbilityEvent, AbilityContext, TargetInfo, ProvidedFields, AbilityInstance};
use crate::schedule::GameSet;
use crate::{Faction, GameState};

#[derive(Debug, Clone, Default, GenerateRaw)]
#[activator]
pub struct OnceParams;

#[derive(Component, Default)]
pub struct OnceActivator {
    pub triggered: bool,
}

impl OnceActivator {
    pub fn from_params(_params: &OnceParams) -> Self {
        Self { triggered: false }
    }
}

pub fn provided_fields() -> ProvidedFields {
    ProvidedFields::SOURCE_ENTITY
        .union(ProvidedFields::SOURCE_POSITION)
}

fn once_system(
    mut trigger_events: MessageWriter<ActivateAbilityEvent>,
    mut ability_query: Query<(&AbilityInstance, &mut OnceActivator)>,
    owner_query: Query<(&Transform, &Faction)>,
) {
    for (instance, mut activator) in &mut ability_query {
        if activator.triggered { continue }

        let Ok((transform, faction)) = owner_query.get(instance.owner) else {
            continue;
        };

        let source = TargetInfo::from_entity_and_position(instance.owner, transform.translation.truncate());
        let target = TargetInfo::EMPTY;

        let ctx = AbilityContext::new(
            instance.owner,
            *faction,
            source,
            target,
        );

        trigger_events.write(ActivateAbilityEvent {
            ability_id: instance.ability_id,
            context: ctx,
        });

        activator.triggered = true;
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        once_system
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

register_activator!(OnceParams, OnceActivator);

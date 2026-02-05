use bevy::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_activator;
use crate::abilities::{ActivateAbilityEvent, AbilityContext, TargetInfo, ProvidedFields, AbilityInputs, AbilityInstance};
use crate::schedule::GameSet;
use crate::{Faction, GameState};

#[derive(Debug, Clone, Default, GenerateRaw)]
#[activator]
pub struct OnInputParams;

#[derive(Component, Default)]
pub struct OnInputActivator;

impl OnInputActivator {
    pub fn from_params(_params: &OnInputParams) -> Self {
        Self
    }
}

pub fn provided_fields() -> ProvidedFields {
    ProvidedFields::SOURCE_ENTITY
        .union(ProvidedFields::SOURCE_POSITION)
        .union(ProvidedFields::TARGET_DIRECTION)
}

fn on_input_system(
    mut trigger_events: MessageWriter<ActivateAbilityEvent>,
    ability_query: Query<(&AbilityInstance, &OnInputActivator)>,
    owner_query: Query<(&AbilityInputs, &Transform, &Faction)>,
) {
    for (instance, _activator) in &ability_query {
        let Ok((inputs, transform, faction)) = owner_query.get(instance.owner) else {
            continue;
        };

        let Some(input) = inputs.get(instance.ability_id) else { continue };
        if !input.just_pressed { continue }

        let source = TargetInfo::from_entity_and_position(instance.owner, transform.translation.truncate());
        let target = TargetInfo::from_direction(input.direction.truncate());

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
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        on_input_system
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

register_activator!(OnInputParams, OnInputActivator);

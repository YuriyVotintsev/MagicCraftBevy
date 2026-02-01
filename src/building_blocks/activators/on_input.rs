use bevy::prelude::*;
use crate::register_activator;
use crate::abilities::{TriggerAbilityEvent, AbilityContext, Target, AbilityInputs, AbilityInstance};
use crate::schedule::GameSet;
use crate::{Faction, GameState};

#[derive(Component, Default)]
pub struct OnInputActivator;

fn on_input_system(
    mut trigger_events: MessageWriter<TriggerAbilityEvent>,
    ability_query: Query<(&AbilityInstance, &OnInputActivator)>,
    owner_query: Query<(&AbilityInputs, &Transform, &Faction)>,
) {
    for (instance, _activator) in &ability_query {
        let Ok((inputs, transform, faction)) = owner_query.get(instance.owner) else {
            continue;
        };

        let Some(input) = inputs.get(instance.ability_id) else { continue };
        if !input.just_pressed { continue }

        let ctx = AbilityContext::new(
            instance.owner,
            *faction,
            Target::Point(transform.translation),
            Some(Target::Direction(input.direction)),
        );

        trigger_events.write(TriggerAbilityEvent {
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

register_activator!(OnInputActivator, params: (), name: "on_input");

use bevy::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_activator;
use crate::abilities::{TriggerAbilityEvent, AbilityContext, Target, AbilityInstance};
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

fn once_system(
    mut trigger_events: MessageWriter<TriggerAbilityEvent>,
    mut ability_query: Query<(&AbilityInstance, &mut OnceActivator)>,
    owner_query: Query<(&Transform, &Faction)>,
) {
    for (instance, mut activator) in &mut ability_query {
        if activator.triggered { continue }

        let Ok((transform, faction)) = owner_query.get(instance.owner) else {
            continue;
        };

        let ctx = AbilityContext::new(
            instance.owner,
            *faction,
            Target::Point(transform.translation),
            None,
        );

        trigger_events.write(TriggerAbilityEvent {
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

pub mod ids;
mod context;
mod owner;
mod effect_def;
mod activator_def;
mod ability_def;
mod components;
mod registry;
mod dispatcher;

pub mod activators;
pub mod effects;

#[allow(unused_imports)]
pub use context::{AbilityContext, ContextValue};
#[allow(unused_imports)]
pub use owner::OwnedBy;
#[allow(unused_imports)]
pub use effect_def::{EffectDef, EffectDefRaw, ParamValue, ParamValueRaw};
#[allow(unused_imports)]
pub use activator_def::{ActivatorDef, ActivatorDefRaw, ActivatorState, ActivationResult};
#[allow(unused_imports)]
pub use ability_def::{AbilityDef, AbilityDefRaw};
#[allow(unused_imports)]
pub use components::{Abilities, AbilityInstance, AbilityInput};
#[allow(unused_imports)]
pub use registry::{Activator, EffectExecutor, ActivatorRegistry, EffectRegistry, AbilityRegistry};

use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::Faction;

mod projectile_systems;
mod orbiting_systems;
mod dash_systems;

#[allow(unused_imports)]
pub use effects::Projectile;

#[derive(Component)]
pub struct PassiveAbilitiesActivated;

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        let mut activator_registry = ActivatorRegistry::new();
        activators::register_activators(&mut activator_registry);

        let mut effect_registry = EffectRegistry::new();
        effects::register_effects(&mut effect_registry);

        app.insert_resource(activator_registry)
            .insert_resource(effect_registry)
            .init_resource::<AbilityRegistry>()
            .add_systems(
                Update,
                (
                    dispatcher::ability_dispatcher,
                    passive_ability_activator,
                )
                    .in_set(GameSet::AbilityActivation),
            )
            .add_systems(
                Update,
                (
                    projectile_systems::projectile_collision,
                    orbiting_systems::update_orbiting_positions,
                    dash_systems::update_dashing,
                )
                    .in_set(GameSet::AbilityExecution),
            )
            .add_systems(
                PostUpdate,
                orbiting_systems::cleanup_orbiting_on_owner_despawn,
            );
    }
}

fn passive_ability_activator(
    mut commands: Commands,
    query: Query<
        (Entity, &Abilities, &ComputedStats, &Transform, &Faction),
        (Added<Abilities>, Without<PassiveAbilitiesActivated>),
    >,
    ability_registry: Res<AbilityRegistry>,
    activator_registry: Res<ActivatorRegistry>,
    effect_registry: Res<EffectRegistry>,
) {
    for (entity, abilities, stats, transform, faction) in &query {
        let mut activated_any = false;

        for (ability_id, _instance) in abilities.iter() {
            let Some(ability_def) = ability_registry.get(ability_id) else {
                continue;
            };

            if activator_registry.get_name(ability_def.activator.activator_type) != Some("passive") {
                continue;
            }

            let ctx = AbilityContext::new(entity, *faction, stats, transform.translation, ability_id);

            for effect_def in &ability_def.effects {
                effect_registry.execute(effect_def, &ctx, &mut commands);
            }

            activated_any = true;
        }

        if activated_any {
            commands.entity(entity).insert(PassiveAbilitiesActivated);
        }
    }
}

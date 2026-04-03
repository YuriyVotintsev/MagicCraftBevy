use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::{BlueprintActivationInput, BlueprintRegistry, SpawnSource};
use crate::blueprints::context::TargetInfo;

#[blueprint_component]
pub struct UseAbilities {
    #[default_expr("target.entity")]
    pub target: EntityExpr,
    pub abilities: Vec<String>,
    #[raw(default = 1.0)]
    pub cooldown: ScalarExpr,
    #[raw(default = false)]
    pub immediate: bool,
}

#[derive(Component)]
pub struct UseAbilitiesTimer {
    pub elapsed: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_use_abilities_timer, use_abilities_system)
            .chain()
            .in_set(crate::schedule::GameSet::MobAI),
    );
}

fn init_use_abilities_timer(
    mut commands: Commands,
    query: Query<(Entity, &UseAbilities), Added<UseAbilities>>,
) {
    for (entity, use_abilities) in &query {
        let elapsed = if use_abilities.immediate { use_abilities.cooldown } else { 0.0 };
        commands.entity(entity).insert(UseAbilitiesTimer { elapsed });
    }
}

fn use_abilities_system(
    time: Res<Time>,
    blueprint_registry: Res<BlueprintRegistry>,
    transforms: Query<&Transform, Without<UseAbilities>>,
    mut query: Query<(&Transform, &UseAbilities, &mut UseAbilitiesTimer, &SpawnSource)>,
    mut activation_input_query: Query<(&SpawnSource, &mut BlueprintActivationInput)>,
) {
    for (transform, use_abilities, mut timer, owner_source) in &mut query {
        let Ok(target_transform) = transforms.get(use_abilities.target) else {
            continue;
        };

        timer.elapsed += time.delta_secs();

        if timer.elapsed < use_abilities.cooldown {
            continue;
        }

        timer.elapsed = 0.0;

        let direction = (target_transform.translation - transform.translation).normalize_or_zero();
        let Some(caster_entity) = owner_source.caster.entity else { continue };

        for blueprint_name in &use_abilities.abilities {
            if let Some(bid) = blueprint_registry.get_id(blueprint_name) {
                for (source, mut input) in &mut activation_input_query {
                    if source.blueprint_id == bid && source.caster.entity == Some(caster_entity) {
                        input.pressed = true;
                        input.target = TargetInfo::from_direction(crate::coord::to_2d(direction));
                    }
                }
            }
        }
    }
}

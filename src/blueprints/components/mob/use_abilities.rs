use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::{BlueprintActivationInput, BlueprintRegistry, SpawnSource};
use crate::blueprints::context::TargetInfo;

#[derive(Component)]
pub struct ShotFired;

#[blueprint_component]
pub struct UseAbilities {
    pub abilities: Vec<String>,
    #[raw(default = 1.0)]
    pub cooldown: ScalarExpr,
    #[raw(default = false)]
    pub immediate: bool,
    pub max_range: Option<ScalarExpr>,
}

#[derive(Component)]
pub struct UseAbilitiesTimer {
    pub elapsed: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (cleanup_shot_fired, init_use_abilities_timer, use_abilities_system)
            .chain()
            .in_set(crate::schedule::GameSet::MobAI),
    );
}

fn cleanup_shot_fired(
    mut commands: Commands,
    query: Query<Entity, With<ShotFired>>,
) {
    for entity in &query {
        commands.entity(entity).remove::<ShotFired>();
    }
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
    mut commands: Commands,
    time: Res<Time>,
    blueprint_registry: Res<BlueprintRegistry>,
    transforms: Query<&Transform, Without<UseAbilities>>,
    mut query: Query<(&Transform, &UseAbilities, &mut UseAbilitiesTimer, &SpawnSource)>,
    mut activation_input_query: Query<(&SpawnSource, &mut BlueprintActivationInput), Without<UseAbilities>>,
) {
    for (transform, use_abilities, mut timer, owner_source) in &mut query {
        let Some(target_entity) = owner_source.target.entity else { continue };
        let Ok(target_transform) = transforms.get(target_entity) else { continue };

        timer.elapsed += time.delta_secs();

        if timer.elapsed < use_abilities.cooldown {
            continue;
        }

        if let Some(max_range) = use_abilities.max_range {
            let dist = transform.translation.distance(target_transform.translation);
            if dist > max_range {
                continue;
            }
        }

        timer.elapsed = 0.0;

        let direction = (target_transform.translation - transform.translation).normalize_or_zero();
        let target_pos = crate::coord::to_2d(target_transform.translation);
        let Some(caster_entity) = owner_source.caster.entity else { continue };

        for blueprint_name in &use_abilities.abilities {
            if let Some(bid) = blueprint_registry.get_id(blueprint_name) {
                for (source, mut input) in &mut activation_input_query {
                    if source.blueprint_id == bid && source.caster.entity == Some(caster_entity) {
                        input.pressed = true;
                        input.target = TargetInfo {
                            entity: Some(target_entity),
                            position: Some(target_pos),
                            direction: Some(crate::coord::to_2d(direction)),
                        };
                    }
                }
            }
        }

        commands.entity(caster_entity).insert(ShotFired);
    }
}

use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::{AbilityInput, AbilityRegistry, AbilitySource};
use crate::abilities::context::TargetInfo;

#[ability_component]
pub struct UseAbilities {
    #[default_expr("target.entity")]
    pub target: EntityExpr,
    pub abilities: Vec<String>,
    #[raw(default = 1.0)]
    pub cooldown: ScalarExpr,
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
    query: Query<Entity, Added<UseAbilities>>,
) {
    for entity in &query {
        commands.entity(entity).insert(UseAbilitiesTimer { elapsed: 0.0 });
    }
}

fn use_abilities_system(
    time: Res<Time>,
    ability_registry: Res<AbilityRegistry>,
    transforms: Query<&Transform, Without<UseAbilities>>,
    mut query: Query<(&Transform, &UseAbilities, &mut UseAbilitiesTimer, &AbilitySource)>,
    mut ability_input_query: Query<(&AbilitySource, &mut AbilityInput)>,
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
        let caster_entity = owner_source.caster.entity.unwrap();

        for ability_name in &use_abilities.abilities {
            if let Some(ability_id) = ability_registry.get_id(ability_name) {
                for (source, mut input) in &mut ability_input_query {
                    if source.ability_id == ability_id && source.caster.entity == Some(caster_entity) {
                        input.pressed = true;
                        input.target = TargetInfo::from_direction(direction.truncate());
                    }
                }
            }
        }
    }
}

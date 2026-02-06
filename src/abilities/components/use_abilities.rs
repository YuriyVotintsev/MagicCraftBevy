use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::{AbilityInputs, AbilityRegistry, InputState};
use crate::player::Player;

#[ability_component]
pub struct UseAbilities {
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
    player_query: Query<&Transform, With<Player>>,
    mut query: Query<(&Transform, &UseAbilities, &mut UseAbilitiesTimer, &mut AbilityInputs)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (transform, use_abilities, mut timer, mut inputs) in &mut query {
        timer.elapsed += time.delta_secs();

        if timer.elapsed < use_abilities.cooldown {
            continue;
        }

        timer.elapsed = 0.0;

        let direction = (player_pos - transform.translation).normalize_or_zero();

        for ability_name in &use_abilities.abilities {
            if let Some(ability_id) = ability_registry.get_id(ability_name) {
                inputs.set(ability_id, InputState {
                    pressed: true,
                    just_pressed: true,
                    direction,
                });
            }
        }
    }
}

use bevy::prelude::*;

use crate::abilities::{AbilityInputs, AbilityRegistry, InputState};
use crate::player::Player;

#[derive(Component)]
pub struct UseAbilities {
    pub ability_names: Vec<String>,
    pub cooldown: f32,
    pub timer: f32,
}

impl UseAbilities {
    pub fn new(abilities: Vec<String>) -> Self {
        Self {
            ability_names: abilities,
            cooldown: 1.0,
            timer: 0.0,
        }
    }

    #[allow(dead_code)]
    pub fn with_cooldown(mut self, cooldown: f32) -> Self {
        self.cooldown = cooldown;
        self
    }
}

pub fn use_abilities_system(
    time: Res<Time>,
    ability_registry: Res<AbilityRegistry>,
    player_query: Query<&Transform, With<Player>>,
    mut query: Query<(&Transform, &mut UseAbilities, &mut AbilityInputs)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (transform, mut use_abilities, mut inputs) in &mut query {
        use_abilities.timer += time.delta_secs();

        if use_abilities.timer < use_abilities.cooldown {
            continue;
        }

        use_abilities.timer = 0.0;

        let direction = (player_pos - transform.translation).normalize_or_zero();

        for ability_name in &use_abilities.ability_names {
            if let Some(ability_id) = ability_registry.get_id(ability_name) {
                inputs.set(ability_id, InputState {
                    pressed: true,
                    just_pressed: true,
                    direction,
                    point: player_pos,
                });
            }
        }
    }
}

use bevy::prelude::*;
use magic_craft_macros::ability_component;
use rand::Rng;

use crate::abilities::{AbilityRegistry, AbilitySource, attach_ability};
use crate::schedule::GameSet;

#[ability_component]
pub struct Spawn {
    pub ability: String,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        init_spawn.in_set(GameSet::Spawning),
    );
}

fn init_spawn(
    mut commands: Commands,
    query: Query<(Entity, &Spawn, &AbilitySource), Added<Spawn>>,
    ability_registry: Res<AbilityRegistry>,
) {
    for (entity, spawn, source) in &query {
        let pos = source.source.position.unwrap_or(Vec2::ZERO);
        let mut rng = rand::rng();
        let offset = Vec2::new(
            rng.random_range(-20.0..20.0),
            rng.random_range(-20.0..20.0),
        );
        let spawn_pos = pos + offset;

        commands.entity(entity).insert(Transform::from_translation(spawn_pos.extend(0.0)));

        if let Some(ability_id) = ability_registry.get_id(&spawn.ability) {
            attach_ability(&mut commands, entity, source.caster_faction, ability_id, &ability_registry);
        }

        commands.entity(entity).insert(super::lifetime::Lifetime { remaining: 5.0 });
    }
}

use bevy::prelude::*;
use magic_craft_macros::blueprint_component;
use rand::Rng;

use crate::blueprints::{BlueprintRegistry, SpawnSource, spawn_blueprint_entity};
use crate::schedule::GameSet;

#[blueprint_component]
pub struct Spawn {
    pub blueprint: String,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        init_spawn.in_set(GameSet::Spawning),
    );
}

fn init_spawn(
    mut commands: Commands,
    query: Query<(Entity, &Spawn, &SpawnSource), Added<Spawn>>,
    blueprint_registry: Res<BlueprintRegistry>,
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

        if let Some(bid) = blueprint_registry.get_id(&spawn.blueprint) {
            spawn_blueprint_entity(&mut commands, entity, source.caster_faction, bid, true);
        }

        commands.entity(entity).insert(crate::blueprints::components::ability::lifetime::Lifetime { remaining: 5.0 });
    }
}

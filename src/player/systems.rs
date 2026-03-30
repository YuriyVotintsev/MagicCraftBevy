use avian3d::prelude::*;
use bevy::prelude::*;

use crate::Faction;
use crate::blueprints::{BlueprintRegistry, spawn_blueprint_entity};
use crate::blueprints::spawn::EntitySpawner;
use crate::player::selected_spells::SpellSlot;
use crate::skill_tree::graph::SkillGraph;
use crate::wave::WavePhase;

use super::{AvailableHeroes, SelectedSpells};

#[derive(Component)]
pub struct Player;

pub fn reset_player_velocity(mut query: Query<&mut LinearVelocity, With<Player>>) {
    for mut velocity in &mut query {
        velocity.0 = Vec3::ZERO;
    }
}

pub fn spawn_player(
    mut spawner: EntitySpawner,
    available_heroes: Res<AvailableHeroes>,
    blueprint_registry: Res<BlueprintRegistry>,
    skill_graph: Option<Res<SkillGraph>>,
    mut selected_spells: ResMut<SelectedSpells>,
) {
    let Some(blueprint_def) = blueprint_registry.get(available_heroes.base_blueprint) else {
        warn!("Base hero blueprint not found");
        return;
    };
    let Some(base_entity_def) = blueprint_def.entities.first() else {
        warn!("Base hero blueprint has no entities");
        return;
    };

    let entity_def = base_entity_def.clone();

    let mut modifier_tuples: Vec<_> = Vec::new();
    if let Some(graph) = &skill_graph {
        for node in &graph.nodes {
            for _ in 0..node.level {
                modifier_tuples.extend(node.rolled_values.iter().copied());
            }
        }
    }

    let entity = spawner.spawn_root(&entity_def, Faction::Player, &modifier_tuples);
    spawner.commands.entity(entity).insert((
        Name::new("Player"),
        Player,
        DespawnOnExit(WavePhase::Combat),
    ));

    if let Some(fireball_id) = blueprint_registry.get_id("fireball") {
        selected_spells.set(SpellSlot::Active, fireball_id);
        spawn_blueprint_entity(&mut spawner.commands, entity, Faction::Player, fireball_id, false);
    }
}


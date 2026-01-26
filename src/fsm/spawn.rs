use avian2d::prelude::*;
use bevy::prelude::*;

use crate::Faction;
use crate::abilities::{Abilities, AbilityInput, AbilityRegistry};
use crate::wave::WavePhase;
use crate::stats::{
    ComputedStats, DirtyStats, Health, Modifiers, StatCalculators, StatId, StatRegistry,
};

use super::components::{CurrentState, MobType};
use super::registry::MobRegistry;
use super::systems::add_state_components;
use super::types::{ColliderShape, Shape};

pub fn spawn_mob(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    mob_registry: &MobRegistry,
    stat_registry: &StatRegistry,
    calculators: &StatCalculators,
    ability_registry: &AbilityRegistry,
    mob_name: &str,
    position: Vec3,
) -> Option<Entity> {
    let mob_def = mob_registry.get(mob_name)?;

    let mesh = match mob_def.visual.shape {
        Shape::Circle => meshes.add(Circle::new(mob_def.visual.size)),
        Shape::Rectangle => {
            meshes.add(Rectangle::new(mob_def.visual.size, mob_def.visual.size))
        }
        Shape::Triangle => meshes.add(RegularPolygon::new(mob_def.visual.size, 3)),
    };
    let color = Color::srgb(
        mob_def.visual.color[0],
        mob_def.visual.color[1],
        mob_def.visual.color[2],
    );

    let mut modifiers = Modifiers::new();
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::new(stat_registry.len());

    dirty.mark_all((0..stat_registry.len() as u32).map(StatId));

    for (stat_name, value) in &mob_def.base_stats {
        if let Some(id) = stat_registry.get(stat_name) {
            modifiers.add(id, *value, None);
        }
    }

    calculators.recalculate(&modifiers, &mut computed, &mut dirty);

    let max_life = stat_registry
        .get("max_life")
        .map(|id| computed.get(id))
        .unwrap_or(100.0);

    let collider = match mob_def.collider.shape {
        ColliderShape::Circle => Collider::circle(mob_def.collider.size / 2.0),
        ColliderShape::Rectangle => {
            Collider::rectangle(mob_def.collider.size, mob_def.collider.size)
        }
    };

    let mut abilities = Abilities::new();
    for ability_name in &mob_def.abilities {
        if let Some(ability_id) = ability_registry.get_id(ability_name) {
            abilities.add(ability_id);
        }
    }

    let entity = commands
        .spawn((
            (
                DespawnOnExit(WavePhase::Combat),
                Mesh2d(mesh),
                MeshMaterial2d(materials.add(color)),
                Transform::from_translation(position),
                MobType(mob_name.to_string()),
                CurrentState(mob_def.initial_state.clone()),
                Faction::Enemy,
                collider,
                RigidBody::Dynamic,
            ),
            (
                LockedAxes::ROTATION_LOCKED,
                LinearVelocity::ZERO,
                modifiers,
                computed,
                dirty,
                Health::new(max_life),
                abilities,
                AbilityInput::new(),
            ),
        ))
        .id();

    let initial_state = mob_def.states.get(&mob_def.initial_state)?;
    add_state_components(commands, entity, mob_def, initial_state);

    info!("Spawned mob '{}' at {:?}", mob_name, position);

    Some(entity)
}

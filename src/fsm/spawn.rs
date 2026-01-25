use bevy::prelude::*;

use crate::stats::{ComputedStats, DirtyStats, Modifiers, StatId, StatRegistry};

use super::components::{CurrentState, MobType};
use super::registry::MobRegistry;
use super::systems::add_state_components;
use super::types::Shape;

pub fn spawn_mob(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    mob_registry: &MobRegistry,
    stat_registry: &StatRegistry,
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

    for i in 0..stat_registry.len() {
        dirty.mark(StatId(i as u32));
    }

    if let Some(id) = stat_registry.get("movement_speed_base") {
        modifiers.add(id, 100.0, None);
    }
    if let Some(id) = stat_registry.get("max_life_base") {
        modifiers.add(id, 50.0, None);
    }

    let entity = commands
        .spawn((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(color)),
            Transform::from_translation(position),
            MobType(mob_name.to_string()),
            CurrentState(mob_def.initial_state.clone()),
            modifiers,
            ComputedStats::new(stat_registry.len()),
            dirty,
        ))
        .id();

    let initial_state = mob_def.states.get(&mob_def.initial_state)?;
    add_state_components(commands, entity, mob_def, initial_state);

    println!("Spawned mob '{}' at {:?}", mob_name, position);

    Some(entity)
}

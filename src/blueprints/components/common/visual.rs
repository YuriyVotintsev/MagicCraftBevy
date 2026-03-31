use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::SpawnSource;
use crate::stats::{ComputedStats, DEFAULT_STATS};

#[blueprint_component]
pub struct Visual {
    pub children: Vec<EntityDef>,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_visual);
}

fn init_visual(
    mut commands: Commands,
    query: Query<(Entity, &Visual, &SpawnSource, Option<&ComputedStats>), Added<Visual>>,
) {
    for (parent, visual, source, stats) in &query {
        let stats = stats.unwrap_or(&DEFAULT_STATS);
        commands.entity(parent).insert(Visibility::default());
        for child_def in &visual.children {
            let child = commands.spawn_empty().id();
            for comp in &child_def.components {
                comp.insert_component(&mut commands.entity(child), source, stats);
            }
            commands.entity(parent).add_child(child);
        }
    }
}

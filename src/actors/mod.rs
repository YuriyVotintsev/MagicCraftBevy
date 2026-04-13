pub mod mobs;
pub mod combat;
pub mod components;
pub mod effects;
pub mod player;
pub mod spawn_source;
pub mod target_info;

pub use spawn_source::SpawnSource;
pub use target_info::TargetInfo;

use bevy::prelude::*;

pub struct ActorsPlugin;

impl Plugin for ActorsPlugin {
    fn build(&self, app: &mut App) {
        combat::register_systems(app);
        effects::register_systems(app);
        mobs::register_systems(app);
        player::register_systems(app);
        components::register_component_systems(app);
    }
}

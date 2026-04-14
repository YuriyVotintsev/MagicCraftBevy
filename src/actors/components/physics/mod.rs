use bevy::prelude::*;

mod collider;
mod dynamic_body;
mod size;
mod static_body;

pub use collider::{Collider, GameLayer, Shape};
pub use dynamic_body::DynamicBody;
pub use size::{Size, SizeScaleLayer};
pub use static_body::StaticBody;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        collider::register_systems(app);
        dynamic_body::register_systems(app);
        static_body::register_systems(app);
        size::register_systems(app);
    }
}

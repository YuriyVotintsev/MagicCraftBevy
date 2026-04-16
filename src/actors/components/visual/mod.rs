use bevy::prelude::*;

mod bobbing_animation;
mod fade_out;
mod growing;
mod jump_walk_animation;
mod on_collision_particles;
mod on_death_particles;
mod shadow;
mod shape;
mod shoot_squish;

pub use bobbing_animation::BobbingAnimation;
pub use fade_out::FadeOut;
pub use growing::Growing;
pub use jump_walk_animation::{JumpWalkAnimation, JumpWalkAnimationState, SelfMoving};
pub use on_collision_particles::OnCollisionParticles;
pub use on_death_particles::OnDeathParticles;
pub use shadow::Shadow;
pub use shape::{CapsuleShape, CircleShape, Shape, ShapeColor, ShapeKind};
pub use shoot_squish::ShootSquish;

pub struct VisualPlugin;

impl Plugin for VisualPlugin {
    fn build(&self, app: &mut App) {
        shape::register_systems(app);
        shadow::register_systems(app);
        bobbing_animation::register_systems(app);
        jump_walk_animation::register_systems(app);
        shoot_squish::register_systems(app);
        fade_out::register_systems(app);
        growing::register_systems(app);
        on_death_particles::register_systems(app);
        on_collision_particles::register_systems(app);
    }
}

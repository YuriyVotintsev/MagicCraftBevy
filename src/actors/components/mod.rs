use bevy::prelude::*;

mod combat;
mod lifetime;
mod physics;
mod player;
mod visual;

pub use combat::{
    death_system, Caster, DeathEvent, FindNearestEnemy, Health, MeleeAttacker, OnCollisionDamage,
    PendingDamage, Projectile, ShotFired, SkipCleanup, Target,
};
pub use lifetime::Lifetime;
pub use physics::{Collider, ColliderShape, DynamicBody, GameLayer, Size, StaticBody};
pub use player::{
    InputTrigger, KeyboardMovement, MouseButtonKind, MovementLocked, PlayerAbilityCooldowns,
    PlayerInput, TargetingMode,
};
pub use visual::{
    BobbingAnimation, CapsuleShape, CircleShape, FadeOut, Growing, JumpWalkAnimation,
    JumpWalkAnimationState, OnCollisionParticles, OnDeathParticles, SelfMoving, Shadow,
    ShootSquish, Shape, ShapeColor, ShapeKind,
};

pub struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            visual::VisualPlugin,
            physics::PhysicsPlugin,
            combat::CombatPlugin,
            player::PlayerComponentsPlugin,
        ));
        lifetime::register_systems(app);
    }
}

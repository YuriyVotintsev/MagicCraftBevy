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
pub use physics::{Collider, DynamicBody, GameLayer, Shape, Size, StaticBody};
pub use player::{
    InputTrigger, KeyboardMovement, MouseButtonKind, MovementLocked, PlayerAbilityCooldowns,
    PlayerInput, TargetingMode,
};
pub use visual::{
    BobbingAnimation, CapsuleSprite, CircleSprite, FadeOut, Growing, JumpWalkAnimation,
    JumpWalkAnimationState, OnCollisionParticles, OnDeathParticles, SelfMoving, Shadow,
    ShootSquish, Sprite, SpriteColor, SpriteShape,
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

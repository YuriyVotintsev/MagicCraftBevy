use bevy::prelude::*;

mod components;
mod mobs;
mod player;

pub use components::{
    death_system, CapsuleSprite, Caster, CircleSprite, DeathEvent, GameLayer, Health,
    JumpWalkAnimationState, MovementLocked, Shadow, SkipCleanup, Sprite,
};
pub use mobs::{spawn_mob, GhostAlpha, GhostTransparency, MobKind, MobsBalance};
pub use player::Player;

pub struct ActorsPlugin;

impl Plugin for ActorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            mobs::MobsPlugin,
            components::ComponentsPlugin,
            player::PlayerPlugin,
        ));
    }
}

use bevy::prelude::*;

pub mod components;
pub mod mobs;
pub mod player;

pub use components::{
    death_system, CapsuleShape, CircleShape, DeathEvent, Fade, GameLayer,
    Health, JumpWalkAnimationState, MovementLocked, Shape, SkipCleanup,
};
pub use mobs::{spawn_mob, GhostTransparency, MobKind, WaveModifiers};
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

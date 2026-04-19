use bevy::prelude::*;

mod components;
mod mobs;
mod player;

pub use components::{
    death_system, CapsuleShape, CircleShape, DeathEvent, Fade, GameLayer,
    Health, JumpWalkAnimationState, MovementLocked, Shadow, Shape, SkipCleanup,
};
pub use mobs::{spawn_mob, GhostTransparency, MobKind, MobsBalance, WaveModifiers};
pub use player::{compute_player_stats, Player};

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

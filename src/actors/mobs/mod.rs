use bevy::prelude::*;

mod ghost;
mod jumper;
mod slime;
mod spawn;
mod spinner;
mod tower;

pub use ghost::{GhostAlpha, GhostTransparency};
pub use spawn::{spawn_mob, MobKind, MobsBalance};

pub struct MobsPlugin;

impl Plugin for MobsPlugin {
    fn build(&self, app: &mut App) {
        ghost::register_systems(app);
        slime::register_systems(app);
        tower::register_systems(app);
        jumper::register_systems(app);
        spinner::register_systems(app);
    }
}

use bevy::prelude::*;

mod balance;
mod ghost;
mod helpers;
mod jumper;
mod kind;
mod slime;
mod spawn;
mod spinner;
mod tower;

pub use balance::MobsBalance;
pub use ghost::{GhostAlpha, GhostTransparency};
pub use kind::MobKind;
pub use spawn::spawn_mob;

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

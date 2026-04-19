use bevy::prelude::*;

pub mod loader;
pub mod parser;
pub mod types;

pub use types::{Globals, MobCommonStats, MobsBalance, RuneCosts, WavesConfig};

pub struct BalancePlugin;

impl Plugin for BalancePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, loader::setup_balance);
        #[cfg(feature = "dev")]
        app.add_systems(Update, loader::reload_balance);
    }
}

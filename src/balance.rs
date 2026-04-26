use bevy::prelude::*;

pub mod loader;
pub mod parser;
pub mod types;

pub use types::{Globals, MobCommonStats, MobsBalance, WavesConfig};

pub struct BalancePlugin;

impl Plugin for BalancePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, loader::setup_balance);
        #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
        app.add_systems(Update, loader::reload_balance);
    }
}

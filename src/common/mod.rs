mod lifecycle;

pub use lifecycle::AttachedTo;

use bevy::prelude::*;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        lifecycle::register_lifecycle_systems(app);
    }
}

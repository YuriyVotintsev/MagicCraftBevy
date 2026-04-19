use bevy::prelude::*;

mod camera;
mod floor;
mod shop_arena;
mod size;
mod spawn;
mod walls;
mod window;

pub use camera::{CameraAngle, CameraZoom};
pub use size::CurrentArenaSize;
pub use walls::Wall;
pub use window::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        camera::register(app);
        walls::register(app);
        floor::register(app);
        spawn::register(app);
        shop_arena::register(app);
    }
}

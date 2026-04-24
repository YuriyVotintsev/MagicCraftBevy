use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod twin_stick;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PlayerIntent {
    pub move_dir: Vec2,
    pub aim_dir: Vec2,
    pub fire: bool,
}

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerIntent>();
        #[cfg(not(target_arch = "wasm32"))]
        native::build(app);
        #[cfg(target_arch = "wasm32")]
        twin_stick::build(app);
    }
}

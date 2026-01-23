mod arena;
mod bullet;
mod enemy;
mod player;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use arena::{ArenaPlugin, WINDOW_HEIGHT, WINDOW_WIDTH};
use bullet::BulletPlugin;
use enemy::EnemyPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                title: "Twin Stick Shooter".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((ArenaPlugin, PlayerPlugin, EnemyPlugin, BulletPlugin))
        .run();
}

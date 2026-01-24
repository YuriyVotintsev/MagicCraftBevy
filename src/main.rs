mod arena;
mod bullet;
mod enemy;
mod player;
mod stats;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use arena::{ArenaPlugin, WINDOW_HEIGHT, WINDOW_WIDTH};
use bullet::BulletPlugin;
use enemy::EnemyPlugin;
use player::PlayerPlugin;
use stats::StatsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                title: "Magic Craft".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            ArenaPlugin,
            PlayerPlugin,
            EnemyPlugin,
            BulletPlugin,
            StatsPlugin,
        ))
        .run();
}

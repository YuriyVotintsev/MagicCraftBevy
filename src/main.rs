mod abilities;
mod arena;
mod expression;
mod fsm;
mod mob_ai;
mod player;
mod stats;

use bevy::prelude::*;

use abilities::AbilityPlugin;
use arena::ArenaPlugin;
use fsm::FsmPlugin;
use player::PlayerPlugin;
use stats::StatsPlugin;

#[cfg(not(feature = "headless"))]
use arena::{WINDOW_HEIGHT, WINDOW_WIDTH};
#[cfg(not(feature = "headless"))]
use bevy::window::WindowResolution;

#[cfg(feature = "headless")]
use bevy::app::ScheduleRunnerPlugin;
#[cfg(feature = "headless")]
use std::time::Duration;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "headless")]
    {
        app.add_plugins(
            DefaultPlugins
                .build()
                .disable::<bevy::winit::WinitPlugin>()
                .set(WindowPlugin {
                    primary_window: None,
                    ..default()
                }),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
    }

    #[cfg(not(feature = "headless"))]
    {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                title: "Magic Craft".to_string(),
                ..default()
            }),
            ..default()
        }));
    }

    app.add_plugins((
        ArenaPlugin,
        PlayerPlugin,
        StatsPlugin,
        AbilityPlugin,
        FsmPlugin,
    ))
    .run();
}

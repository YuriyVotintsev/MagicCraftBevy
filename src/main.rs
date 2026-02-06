mod abilities;
mod arena;
mod common;
mod faction;
mod fsm;
mod game_state;
mod loading;
mod money;
mod movement;
mod physics;
mod player;
mod schedule;
mod stats;
mod ui;
mod wave;

pub use faction::Faction;
pub use game_state::GameState;
pub use movement::MovementLocked;

use avian2d::prelude::*;
use bevy::prelude::*;

use abilities::AbilityPlugin;
use arena::ArenaPlugin;
use common::CommonPlugin;
use fsm::FsmPlugin;
use loading::LoadingPlugin;
use player::PlayerPlugin;
use schedule::{GameSet, PostGameSet};
use stats::StatsPlugin;
use ui::UiPlugin;
use wave::{WavePhase, WavePlugin};

#[cfg(not(feature = "headless"))]
use arena::{WINDOW_HEIGHT, WINDOW_WIDTH};
#[cfg(not(feature = "headless"))]
use bevy::window::WindowResolution;
#[cfg(not(feature = "headless"))]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
#[cfg(feature = "headless")]
use bevy::window::ExitCondition;

#[cfg(feature = "headless")]
use bevy::app::ScheduleRunnerPlugin;
#[cfg(feature = "headless")]
use std::time::Duration;

#[cfg(feature = "headless")]
#[derive(Resource)]
struct HeadlessTimeout {
    timer: Timer,
}

#[cfg(feature = "headless")]
fn parse_timeout_arg() -> f32 {
    let args: Vec<String> = std::env::args().collect();

    for i in 0..args.len() {
        if args[i] == "--timeout" || args[i] == "-t" {
            if let Some(value) = args.get(i + 1) {
                return value.parse().unwrap_or_else(|_| {
                    eprintln!("Error: Invalid timeout value '{}'", value);
                    std::process::exit(1);
                });
            } else {
                eprintln!("Error: --timeout requires a value in seconds");
                std::process::exit(1);
            }
        }
    }

    eprintln!("Error: Headless mode requires --timeout <seconds> argument");
    eprintln!("Usage: cargo run --features headless -- --timeout 10");
    std::process::exit(1);
}

#[cfg(feature = "headless")]
fn headless_timeout_system(
    time: Res<Time>,
    mut timeout: ResMut<HeadlessTimeout>,
    mut exit: MessageWriter<AppExit>,
) {
    if timeout.timer.tick(time.delta()).just_finished() {
        info!("Headless timeout reached, exiting");
        exit.write(AppExit::Success);
    }
}

fn main() {
    let mut app = App::new();

    #[cfg(feature = "headless")]
    {
        let timeout_secs = parse_timeout_arg();
        info!("Running in headless mode with {}s timeout", timeout_secs);

        app.add_plugins(
            DefaultPlugins
                .build()
                .disable::<bevy::winit::WinitPlugin>()
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                }),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .insert_resource(HeadlessTimeout {
            timer: Timer::from_seconds(timeout_secs, TimerMode::Once),
        })
        .add_systems(Update, headless_timeout_system);
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
        }))
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()));
    }

    app.init_state::<GameState>()
        .add_sub_state::<WavePhase>()
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::MobAI,
                GameSet::AbilityActivation,
                GameSet::AbilityExecution,
                GameSet::Damage,
                GameSet::WaveManagement,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .configure_sets(PostUpdate, PostGameSet.run_if(in_state(GameState::Playing)))
        .add_systems(
            Update,
            ApplyDeferred
                .after(GameSet::AbilityExecution)
                .before(GameSet::Damage),
        )
        .add_plugins((
            PhysicsPlugins::default().with_length_unit(100.0),
            LoadingPlugin,
            CommonPlugin,
            ArenaPlugin,
            PlayerPlugin,
            StatsPlugin,
            AbilityPlugin,
            FsmPlugin,
            UiPlugin,
            WavePlugin,
        ))
        .run();
}

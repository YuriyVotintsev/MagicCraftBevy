mod affixes;
mod artifacts;
mod blueprints;
mod arena;
mod common;
mod faction;
mod game_state;
mod hit_flash;
mod loading;
mod money;
mod movement;
mod physics;
mod player;
mod schedule;
mod stats;
mod ui;
mod wave;

#[cfg(test)]
mod validation_tests;

pub use faction::Faction;
pub use game_state::GameState;
pub use movement::MovementLocked;

fn disable_physics_debug(mut store: ResMut<GizmoConfigStore>) {
    store.config_mut::<PhysicsGizmos>().0.enabled = false;
}

fn toggle_physics_debug(key: Res<ButtonInput<KeyCode>>, mut store: ResMut<GizmoConfigStore>) {
    if key.just_pressed(KeyCode::Backquote) {
        let config = &mut store.config_mut::<PhysicsGizmos>().0;
        config.enabled = !config.enabled;
    }
}

use avian2d::prelude::*;
use bevy::prelude::*;

use affixes::AffixPlugin;
use artifacts::ArtifactPlugin;
use blueprints::BlueprintPlugin;
use arena::ArenaPlugin;
use common::CommonPlugin;
use hit_flash::HitFlashPlugin;
use loading::LoadingPlugin;
use player::PlayerPlugin;
use schedule::{GameSet, PostGameSet};
use stats::StatsPlugin;
use ui::UiPlugin;
use bevy_tweening::TweeningPlugin;
use wave::{CombatPhase, WavePhase, WavePlugin};

#[cfg(not(feature = "headless"))]
use arena::{WINDOW_HEIGHT, WINDOW_WIDTH};
#[cfg(not(feature = "headless"))]
use bevy::window::WindowResolution;
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
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
                    .with_scale_factor_override(1.0),
                title: "Magic Craft".to_string(),
                ..default()
            }),
            ..default()
        }));
    }

    app.init_state::<GameState>()
        .add_sub_state::<WavePhase>()
        .add_sub_state::<CombatPhase>()
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::MobAI,
                GameSet::Spawning,
                GameSet::BlueprintActivation,
                GameSet::BlueprintExecution,
                GameSet::Damage,
                GameSet::DamageApply,
                GameSet::WaveManagement,
                GameSet::Cleanup,
            )
                .chain()
                .run_if(in_state(CombatPhase::Running)),
        )
        .configure_sets(PostUpdate, PostGameSet.run_if(in_state(CombatPhase::Running)))
        .add_systems(
            Update,
            ApplyDeferred
                .after(GameSet::Spawning)
                .before(GameSet::BlueprintActivation),
        )
        .add_systems(
            Update,
            ApplyDeferred
                .after(GameSet::BlueprintExecution)
                .before(GameSet::Damage),
        )
        .add_plugins((
            PhysicsPlugins::default().with_length_unit(100.0),
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, disable_physics_debug)
        .add_systems(Update, toggle_physics_debug)
        .add_plugins((
            LoadingPlugin,
            CommonPlugin,
            ArenaPlugin,
            PlayerPlugin,
            StatsPlugin,
            BlueprintPlugin,
            ArtifactPlugin,
            AffixPlugin,
            HitFlashPlugin,
            TweeningPlugin,
            UiPlugin,
            WavePlugin,
        ))
        .run();
}

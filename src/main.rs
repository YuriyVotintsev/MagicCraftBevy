mod actors;
mod balance;
mod composite_scale;
mod arena;
mod faction;
mod game_state;
mod health_material;
mod hit_flash;
mod loading;
mod coord;
pub mod palette;
mod particles;
mod run;
mod rune;
mod schedule;
mod stats;
mod transition;
mod ui;
mod wave;

pub use faction::Faction;
pub use game_state::GameState;
pub use transition::{Transition, TransitionAction};

fn disable_physics_debug(mut store: ResMut<GizmoConfigStore>) {
    store.config_mut::<PhysicsGizmos>().0.enabled = false;
}

fn toggle_physics_debug(key: Res<ButtonInput<KeyCode>>, mut store: ResMut<GizmoConfigStore>) {
    if key.just_pressed(KeyCode::F3) {
        let config = &mut store.config_mut::<PhysicsGizmos>().0;
        config.enabled = !config.enabled;
    }
}

use avian3d::prelude::*;
use bevy::prelude::*;

use actors::ActorsPlugin;
use arena::ArenaPlugin;
use health_material::HealthMaterialPlugin;
use hit_flash::HitFlashPlugin;
use loading::LoadingPlugin;
use schedule::{GameSet, PostGameSet, ShopSet};
use stats::StatsPlugin;
use transition::TransitionPlugin;
use ui::UiPlugin;
use bevy_tweening::TweeningPlugin;
use run::RunPlugin;
use rune::RunePlugin;
use wave::{CombatPhase, WavePhase, WavePlugin};

use arena::{WINDOW_HEIGHT, WINDOW_WIDTH};
use bevy::window::WindowResolution;

#[cfg(feature = "dev")]
use bevy::window::ExitCondition;
#[cfg(feature = "dev")]
use bevy::app::ScheduleRunnerPlugin;
#[cfg(feature = "dev")]
use std::time::Duration;

#[cfg(feature = "dev")]
#[derive(Resource)]
struct HeadlessTimeout {
    timer: Timer,
}

#[cfg(feature = "dev")]
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
    eprintln!("Usage: HEADLESS=1 cargo run --features dev -- --timeout 10");
    std::process::exit(1);
}

#[cfg(feature = "dev")]
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

    #[cfg(feature = "dev")]
    if std::env::var("HEADLESS").is_ok() {
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
    } else {
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

    #[cfg(not(feature = "dev"))]
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
                .with_scale_factor_override(1.0),
            title: "Magic Craft".to_string(),
            ..default()
        }),
        ..default()
    }));

    app.init_state::<GameState>()
        .add_sub_state::<WavePhase>()
        .add_sub_state::<CombatPhase>()
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::MobAI,
                GameSet::Spawning,
                GameSet::AbilityActivation,
                GameSet::AbilityExecution,
                GameSet::AbilityLifecycle,
                GameSet::Damage,
                GameSet::DamageApply,
                GameSet::WaveManagement,
                GameSet::Cleanup,
            )
                .chain()
                .run_if(in_state(CombatPhase::Running)),
        )
        .configure_sets(
            Update,
            (ShopSet::Input, ShopSet::Process, ShopSet::Display)
                .chain()
                .run_if(in_state(WavePhase::Shop)),
        )
        .configure_sets(PostUpdate, PostGameSet.run_if(in_state(CombatPhase::Running)))
        .add_systems(
            Update,
            ApplyDeferred
                .after(GameSet::Spawning)
                .before(GameSet::AbilityActivation),
        )
        .add_systems(
            Update,
            ApplyDeferred
                .after(GameSet::AbilityExecution)
                .before(GameSet::AbilityLifecycle),
        )
        .add_plugins((
            PhysicsPlugins::default().with_length_unit(100.0),
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, disable_physics_debug)
        .add_systems(Update, toggle_physics_debug)
        .add_plugins((
            LoadingPlugin,
            ArenaPlugin,
            StatsPlugin,
            ActorsPlugin,
            RunPlugin,
            RunePlugin,
            HealthMaterialPlugin,
            HitFlashPlugin,
            TweeningPlugin,
            TransitionPlugin,
            UiPlugin,
            WavePlugin,
        ))
        .add_plugins(particles::ParticlesPlugin)
        .add_plugins(composite_scale::CompositeScalePlugin);

    app.run();
}

mod actors;
mod balance;
mod composite_scale;
mod arena;
mod dissolve_material;
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
mod rune_ball_material;
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
use balance::BalancePlugin;
use dissolve_material::DissolveMaterialPlugin;
use health_material::HealthMaterialPlugin;
use rune_ball_material::RuneBallMaterialPlugin;
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

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use bevy::window::ExitCondition;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use bevy::app::ScheduleRunnerPlugin;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use std::time::Duration;

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
#[derive(Resource)]
struct HeadlessTimeout {
    timer: Timer,
}

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
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

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
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

#[cfg(not(target_arch = "wasm32"))]
fn windowed_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
                .with_scale_factor_override(1.0),
            title: "Magic Craft".to_string(),
            position: WindowPosition::Centered(MonitorSelection::Primary),
            ..default()
        }),
        ..default()
    }
}

#[cfg(target_arch = "wasm32")]
fn windowed_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#bevy-canvas".to_string()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            title: "Magic Craft".to_string(),
            ..default()
        }),
        ..default()
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    let headless = std::env::var("HEADLESS").is_ok();
    #[cfg(any(not(feature = "dev"), target_arch = "wasm32"))]
    let headless = false;

    if headless {
        #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
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
    } else {
        #[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut))]
        let mut plugins = DefaultPlugins.set(windowed_plugin());
        // On WebGL2 a handful of Bevy sub-plugins (SSAO, atmosphere, OIT
        // resolve) log a warning every startup because their required GPU
        // features don't exist in the browser. They're nested inside
        // `PbrPlugin`/`CorePipelinePlugin` so `.disable::<T>()` can't reach
        // them — raise their log level via `LogPlugin::filter` instead.
        #[cfg(target_arch = "wasm32")]
        {
            plugins = plugins.set(bevy::log::LogPlugin {
                filter: format!(
                    "{}bevy_pbr::ssao=error,bevy_pbr::atmosphere=error,\
                     bevy_core_pipeline::oit::resolve=error",
                    bevy::log::DEFAULT_FILTER,
                ),
                ..default()
            });
            plugins = plugins.set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            });
        }
        app.add_plugins(plugins);
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
        .add_plugins((
            PhysicsPlugins::default().with_length_unit(100.0),
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, disable_physics_debug)
        .add_systems(Update, toggle_physics_debug)
        .add_plugins((
            BalancePlugin,
            LoadingPlugin,
            ArenaPlugin,
            StatsPlugin,
            ActorsPlugin,
            RunPlugin,
            RunePlugin,
            DissolveMaterialPlugin,
            HealthMaterialPlugin,
            RuneBallMaterialPlugin,
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

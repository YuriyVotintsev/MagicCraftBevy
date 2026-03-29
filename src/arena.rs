use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::camera::ScalingMode;
use rand::Rng;

use crate::balance::{ArenaBalance, GameBalance};
use crate::blueprints::components::ability::projectile::Projectile;
use crate::blueprints::components::common::health::Health;
use crate::blueprints::components::common::visual::Visual;
use crate::blueprints::{BlueprintRegistry, spawn_blueprint_entity};
use crate::coin::Coin;
use crate::physics::{GameLayer, Wall};
use crate::run::RunState;
use crate::schedule::GameSet;
use crate::stats::{DirtyStats, Modifiers, StatRegistry};
use crate::wave::{WaveEnemy, WavePhase, WaveState};
use crate::Faction;
use crate::GameState;

#[cfg(not(feature = "headless"))]
pub const WINDOW_WIDTH: f32 = 1920.0;
#[cfg(not(feature = "headless"))]
pub const WINDOW_HEIGHT: f32 = 1080.0;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(
                OnEnter(GameState::MainMenu),
                spawn_arena.run_if(not(any_with_component::<Wall>)),
            )
            .add_systems(
                Update,
                (update_target_count, spawn_enemies, tag_wave_enemies)
                    .chain()
                    .in_set(GameSet::Spawning)
                    .run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                Update,
                apply_enemy_scaling
                    .in_set(GameSet::BlueprintActivation)
                    .run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                PostUpdate,
                (camera_follow, y_sort).run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.insert_resource(ClearColor(crate::palette::color("background")));
    commands.spawn((
        Name::new("MainCamera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 1080.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn spawn_arena(mut commands: Commands, balance: Res<GameBalance>) {
    let arena = &balance.arena;

    let half_w = arena.half_w;
    let half_h = arena.half_h;

    let vertices = vec![
        Vec2::new(half_w, half_h),
        Vec2::new(-half_w, half_h),
        Vec2::new(-half_w, -half_h),
        Vec2::new(half_w, -half_h),
    ];
    let indices: Vec<[u32; 2]> = vec![[0, 1], [1, 2], [2, 3], [3, 0]];

    let wall_layers = CollisionLayers::new(GameLayer::Wall, LayerMask::ALL);

    commands.spawn((
        Name::new("ArenaWall"),
        Wall,
        Transform::default(),
        Collider::polyline(vertices, Some(indices)),
        CollisionMargin(5.0),
        RigidBody::Static,
        wall_layers,
    ));
}

fn camera_follow(
    player_query: Query<&Transform, With<crate::player::Player>>,
    mut camera_query: Query<
        (&mut Transform, &Projection),
        (With<Camera2d>, Without<crate::player::Player>),
    >,
    balance: Res<GameBalance>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok((mut camera_transform, projection)) = camera_query.single_mut() else {
        return;
    };

    let arena = &balance.arena;
    let (vp_hw, vp_hh) = match projection {
        Projection::Orthographic(ortho) => (ortho.area.max.x, ortho.area.max.y),
        _ => return,
    };

    let max_x = (arena.half_w - vp_hw).max(0.0);
    let max_y = (arena.half_h - vp_hh).max(0.0);

    camera_transform.translation.x = player_transform.translation.x.clamp(-max_x, max_x);
    camera_transform.translation.y = player_transform.translation.y.clamp(-max_y, max_y);
}

fn y_sort(
    mut query: Query<
        &mut Transform,
        Or<(With<Visual>, With<Coin>, With<Projectile>)>,
    >,
) {
    for mut transform in &mut query {
        transform.translation.z = -transform.translation.y * 0.001;
    }
}

fn update_target_count(
    run_state: Res<RunState>,
    mut wave_state: ResMut<WaveState>,
    balance: Res<GameBalance>,
) {
    let wb = &balance.wave;
    let t = (run_state.elapsed / wb.ramp_duration_secs).clamp(0.0, 1.0);
    let target = wb.start_enemies as f32 + t * (wb.max_enemies - wb.start_enemies) as f32;
    wave_state.max_concurrent = target.round() as u32;
}

fn spawn_enemies(
    mut commands: Commands,
    mut wave_state: ResMut<WaveState>,
    enemies_query: Query<&Faction, With<Health>>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    blueprint_registry: Res<BlueprintRegistry>,
    balance: Res<GameBalance>,
) {
    let alive_enemies = enemies_query.iter().filter(|f| **f == Faction::Enemy).count() as u32;
    let deficit = wave_state.max_concurrent.saturating_sub(alive_enemies);
    if deficit == 0 {
        return;
    }

    let player_pos = player_query
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    let arena = &balance.arena;
    let safe_radius_sq = balance.wave.safe_spawn_radius * balance.wave.safe_spawn_radius;
    let margin = 30.0;
    let hw = arena.half_w - margin;
    let hh = arena.half_h - margin;
    let mut rng = rand::rng();

    for _ in 0..deficit {
        let (x, y) = {
            let mut attempts = 0;
            loop {
                let x = rng.random_range(-hw..hw);
                let y = rng.random_range(-hh..hh);
                let pos = Vec2::new(x, y);
                attempts += 1;
                if attempts > 100
                    || (is_inside_arena(pos, margin, arena)
                        && pos.distance_squared(player_pos) > safe_radius_sq)
                {
                    break (x, y);
                }
            }
        };

        let blueprint_name = "hopper";

        if let Some(blueprint_id) = blueprint_registry.get_id(blueprint_name) {
            let entity = commands
                .spawn((
                    Name::new("Enemy"),
                    Transform::from_xyz(x, y, 0.0),
                    WaveEnemy,
                    DespawnOnExit(WavePhase::Combat),
                ))
                .id();

            spawn_blueprint_entity(&mut commands, entity, Faction::Enemy, blueprint_id, true);
            wave_state.spawned_count += 1;
        }
    }
}

fn apply_enemy_scaling(
    mut query: Query<(&mut Modifiers, &mut DirtyStats), Added<WaveEnemy>>,
    run_state: Res<RunState>,
    stat_registry: Res<StatRegistry>,
    balance: Res<GameBalance>,
) {
    let elapsed = run_state.elapsed;
    if elapsed <= 0.0 {
        return;
    }
    let hp_bonus = elapsed * balance.run.hp_scale_per_sec;
    let dmg_bonus = elapsed * balance.run.dmg_scale_per_sec;
    let hp_stat = stat_registry.get("max_life_increased");
    let dmg_stat = stat_registry.get("physical_damage_increased");

    for (mut modifiers, mut dirty) in &mut query {
        if let Some(stat) = hp_stat {
            modifiers.add(stat, hp_bonus);
            dirty.mark(stat);
        }
        if let Some(stat) = dmg_stat {
            modifiers.add(stat, dmg_bonus);
            dirty.mark(stat);
        }
    }
}

fn is_inside_arena(pos: Vec2, margin: f32, arena: &ArenaBalance) -> bool {
    let hw = arena.half_w - margin;
    let hh = arena.half_h - margin;
    pos.x.abs() <= hw && pos.y.abs() <= hh
}

fn tag_wave_enemies(
    mut commands: Commands,
    query: Query<Entity, (Added<Health>, With<Faction>, Without<WaveEnemy>)>,
    faction_query: Query<&Faction>,
) {
    for entity in &query {
        let Ok(faction) = faction_query.get(entity) else { continue };
        if *faction == Faction::Enemy {
            commands.entity(entity).insert(DespawnOnExit(WavePhase::Combat));
        }
    }
}

use bevy::prelude::*;
use rand::Rng;

use crate::actors::Health;
use crate::actors::{MobKind, MobsBalance};
use crate::arena::CurrentArenaSize;
use crate::balance::GameBalance;
use crate::run::{PlayerDying, RunState};
use crate::schedule::GameSet;
use crate::stats::Stat;
use super::phase::WavePhase;
use super::state::{WaveEnemy, WaveState};
use super::summoning::{SummoningCircle, SummoningCircleMaterial, SummoningCircleMesh};
use crate::Faction;

const ALL_ENEMY_TYPES: &[MobKind] = &[
    MobKind::SlimeSmall,
    MobKind::Jumper,
    MobKind::Tower,
    MobKind::Ghost,
    MobKind::Spinner,
];

#[derive(Resource)]
pub struct EnemySpawnPool {
    pub enabled: Vec<(MobKind, bool)>,
}

impl Default for EnemySpawnPool {
    fn default() -> Self {
        Self {
            enabled: ALL_ENEMY_TYPES.iter().map(|&k| (k, true)).collect(),
        }
    }
}

impl EnemySpawnPool {
    pub fn active_kinds(&self) -> Vec<MobKind> {
        self.enabled.iter().filter(|(_, on)| *on).map(|(k, _)| *k).collect()
    }
}

pub fn register(app: &mut App) {
    app.init_resource::<EnemySpawnPool>()
        .add_systems(OnEnter(WavePhase::Combat), reset_arena_size)
        .add_systems(
            Update,
            (update_arena_size, update_target_count, spawn_enemies, tag_wave_enemies)
                .chain()
                .in_set(GameSet::Spawning)
                .run_if(in_state(WavePhase::Combat))
                .run_if(not(resource_exists::<PlayerDying>)),
        );
}

fn reset_arena_size(
    mut arena_size: ResMut<CurrentArenaSize>,
    balance: Res<GameBalance>,
) {
    arena_size.width = balance.arena.start_width;
    arena_size.height = balance.arena.start_height;
}

fn update_arena_size(
    run_state: Res<RunState>,
    balance: Res<GameBalance>,
    mut arena_size: ResMut<CurrentArenaSize>,
) {
    let arena = &balance.arena;
    let t = (run_state.elapsed / balance.wave.ramp_duration_secs).clamp(0.0, 1.0);
    let w = arena.start_width + t * (arena.width - arena.start_width);
    let h = arena.start_height + t * (arena.height - arena.start_height);
    if (arena_size.width - w).abs() > 0.1 || (arena_size.height - h).abs() > 0.1 {
        arena_size.width = w;
        arena_size.height = h;
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
    player_query: Query<&Transform, With<crate::actors::Player>>,
    mobs_balance: Res<MobsBalance>,
    balance: Res<GameBalance>,
    run_state: Res<RunState>,
    circle_mesh: Res<SummoningCircleMesh>,
    circle_material: Res<SummoningCircleMaterial>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spawn_pool: Res<EnemySpawnPool>,
    arena_size: Res<CurrentArenaSize>,
) {
    let active_enemies = wave_state.spawned_count - wave_state.killed_count;
    let deficit = wave_state.max_concurrent.saturating_sub(active_enemies);
    if deficit == 0 {
        return;
    }

    let player_pos = player_query
        .single()
        .map(|t| crate::coord::to_2d(t.translation))
        .unwrap_or(Vec2::ZERO);

    let safe_radius_sq = balance.wave.safe_spawn_radius * balance.wave.safe_spawn_radius;
    let margin = 30.0;
    let hw = arena_size.half_w() - margin;
    let hh = arena_size.half_h() - margin;
    let mut rng = rand::rng();

    let mut extra_modifiers: Vec<(Stat, f32)> = Vec::new();
    let elapsed = run_state.elapsed;
    if elapsed > 0.0 {
        let hp_bonus = elapsed * balance.run.hp_scale_per_sec;
        let dmg_bonus = elapsed * balance.run.dmg_scale_per_sec;
        extra_modifiers.push((Stat::MaxLifeIncreased, hp_bonus));
        extra_modifiers.push((Stat::PhysicalDamageIncreased, dmg_bonus));
    }

    for _ in 0..deficit {
        let (x, y) = {
            let mut attempts = 0;
            loop {
                let x = rng.random_range(-hw..hw);
                let y = rng.random_range(-hh..hh);
                let pos = Vec2::new(x, y);
                attempts += 1;
                if attempts > 100
                    || (is_inside_arena(pos, margin, &arena_size)
                        && pos.distance_squared(player_pos) > safe_radius_sq)
                {
                    break (x, y);
                }
            }
        };

        let active = spawn_pool.active_kinds();
        if active.is_empty() {
            break;
        }
        let kind = active[rng.random_range(0..active.len())];
        let circle_size = kind.size(&mobs_balance);
        let ground = crate::coord::ground_pos(Vec2::new(x, y));

        let is_ghost = matches!(kind, MobKind::Ghost);
        let mat_handle = if is_ghost {
            let cloned = materials.get(&circle_material.0).cloned();
            if let Some(base_mat) = cloned {
                MeshMaterial3d(materials.add(base_mat))
            } else {
                MeshMaterial3d(circle_material.0.clone())
            }
        } else {
            MeshMaterial3d(circle_material.0.clone())
        };

        let mut entity_commands = commands.spawn((
            Name::new("SummoningCircle"),
            Mesh3d(circle_mesh.0.clone()),
            mat_handle,
            Transform::from_translation(ground + Vec3::Y * 0.02)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::ZERO),
            SummoningCircle::new(kind, circle_size, extra_modifiers.clone()),
            DespawnOnExit(WavePhase::Combat),
        ));

        if is_ghost {
            use crate::actors::{GhostAlpha, GhostTransparency};
            entity_commands.insert((
                GhostTransparency {
                    visible_distance: mobs_balance.ghost.visible_distance,
                    invisible_distance: mobs_balance.ghost.invisible_distance,
                },
                GhostAlpha { value: 0.0 },
            ));
        }
        wave_state.spawned_count += 1;
        wave_state.summoning_count += 1;
    }
}

fn is_inside_arena(pos: Vec2, margin: f32, arena: &CurrentArenaSize) -> bool {
    let hw = arena.half_w() - margin;
    let hh = arena.half_h() - margin;
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

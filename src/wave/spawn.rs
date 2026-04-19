use bevy::prelude::*;
use rand::Rng;

use crate::actors::Health;
use crate::actors::MobKind;
use crate::arena::CurrentArenaSize;
use crate::balance::{Globals, MobsBalance, WavesConfig};
use crate::dissolve_material::DissolveMaterial;
use crate::run::{CombatScoped, PlayerDying, RunState};
use crate::schedule::GameSet;
use super::phase::WavePhase;
use super::state::{WaveEnemy, WaveState};
use super::summoning::{SummoningCircle, SummoningCircleMaterial, SummoningCircleMesh};
use crate::Faction;

#[derive(Resource)]
pub struct EnemySpawnPool {
    pub enabled: Vec<(MobKind, bool)>,
}

impl Default for EnemySpawnPool {
    fn default() -> Self {
        Self {
            enabled: MobKind::iter().map(|k| (k, true)).collect(),
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
        .add_systems(
            OnEnter(WavePhase::Combat),
            apply_wave_config.after(crate::run::init_run),
        )
        .add_systems(
            Update,
            (spawn_enemies, tag_wave_enemies)
                .chain()
                .in_set(GameSet::Spawning)
                .run_if(in_state(WavePhase::Combat))
                .run_if(not(resource_exists::<PlayerDying>)),
        );
}

fn apply_wave_config(
    run_state: Res<RunState>,
    mut wave_state: ResMut<WaveState>,
    mut pool: ResMut<EnemySpawnPool>,
    waves: Res<WavesConfig>,
) {
    let def = waves.for_wave(run_state.wave);
    wave_state.max_concurrent = def.max_concurrent;

    let mut rng = rand::rng();
    let active = waves.resolve_pool(run_state.wave, &mut rng);
    pool.enabled = MobKind::iter()
        .map(|k| (k, active.contains(&k)))
        .collect();
}

fn spawn_enemies(
    mut commands: Commands,
    mut wave_state: ResMut<WaveState>,
    player_query: Query<&Transform, With<crate::actors::Player>>,
    mobs_balance: Res<MobsBalance>,
    globals: Res<Globals>,
    circle_mesh: Res<SummoningCircleMesh>,
    circle_material: Res<SummoningCircleMaterial>,
    mut materials: ResMut<Assets<DissolveMaterial>>,
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

    let safe_radius_sq = globals.safe_spawn_radius * globals.safe_spawn_radius;
    let margin = 30.0;
    let inner_radius = (arena_size.radius - margin).max(0.0);
    let mut rng = rand::rng();

    for _ in 0..deficit {
        let (x, y) = {
            let mut attempts = 0;
            loop {
                let r = inner_radius * rng.random_range(0.0_f32..1.0).sqrt();
                let theta = rng.random_range(0.0..std::f32::consts::TAU);
                let x = r * theta.cos();
                let y = r * theta.sin();
                let pos = Vec2::new(x, y);
                attempts += 1;
                if attempts > 100 || pos.distance_squared(player_pos) > safe_radius_sq {
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
            SummoningCircle::new(kind, circle_size),
            CombatScoped,
        ));

        if is_ghost {
            use crate::actors::GhostTransparency;
            use crate::actors::mobs::ghost::{GHOST_INVISIBLE_DISTANCE, GHOST_VISIBLE_DISTANCE};
            entity_commands.insert(GhostTransparency {
                visible_distance: GHOST_VISIBLE_DISTANCE,
                invisible_distance: GHOST_INVISIBLE_DISTANCE,
            });
        }
        wave_state.spawned_count += 1;
        wave_state.summoning_count += 1;
    }
}

fn tag_wave_enemies(
    mut commands: Commands,
    query: Query<Entity, (Added<Health>, With<Faction>, Without<WaveEnemy>)>,
    faction_query: Query<&Faction>,
) {
    for entity in &query {
        let Ok(faction) = faction_query.get(entity) else { continue };
        if *faction == Faction::Enemy {
            commands.entity(entity).insert(CombatScoped);
        }
    }
}

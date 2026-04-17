use avian3d::prelude::*;
use bevy::prelude::*;

use super::components::{
    Caster, Collider, DynamicBody, Health, InputTrigger, JumpWalkAnimation, KeyboardMovement,
    MouseButtonKind, OnCollisionDamage, OnCollisionParticles, PlayerAbilityCooldowns, PlayerInput,
    Projectile, Shadow, ColliderShape, Size, Shape, ShapeColor, ShapeKind,
    TargetingMode,
};
use crate::palette;
use crate::run::CombatScoped;
use crate::rune::{add_grid_modifiers, RuneGrid};
use crate::stats::{ComputedStats, DirtyStats, ModifierKind, Modifiers, Stat, StatCalculators};
use crate::wave::WavePhase;
use crate::Faction;

pub const PLAYER_BASE_STATS: &[(Stat, ModifierKind, f32)] = &[
    (Stat::MaxLife, ModifierKind::Flat, 20.0),
    (Stat::MovementSpeed, ModifierKind::Flat, 550.0),
    (Stat::PhysicalDamage, ModifierKind::Flat, 1.0),
    (Stat::CritChance, ModifierKind::Flat, 0.05),
    (Stat::CritMultiplier, ModifierKind::Flat, 1.5),
    (Stat::AttackSpeed, ModifierKind::Flat, 1.0),
    (Stat::PickupRadius, ModifierKind::Flat, 200.0),
];

pub fn compute_player_stats(
    grid: &RuneGrid,
    calculators: &StatCalculators,
) -> (Modifiers, ComputedStats) {
    let mut modifiers = Modifiers::new();
    for &(stat, kind, value) in PLAYER_BASE_STATS {
        modifiers.add(stat, kind, value);
    }
    add_grid_modifiers(grid, &mut modifiers);

    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::default();
    dirty.mark_all(Stat::iter());
    calculators.recalculate(&modifiers, &mut computed, &mut dirty);
    (modifiers, computed)
}

pub const FIREBALL_DAMAGE_PCT: f32 = 1.0;
pub const FIREBALL_BASE_SPEED: f32 = 800.0;
pub const FIREBALL_COOLDOWN: f32 = 0.5;
pub const FIREBALL_SIZE: f32 = 60.0;
pub const FIREBALL_GAP: f32 = 60.0;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WavePhase::Combat), spawn_player);
    }
}

fn player_shape_color() -> ShapeColor {
    let (r, g, b) = palette::lookup("player").unwrap_or((0.5, 0.8, 1.0));
    let flash = palette::flash_lookup("player");
    ShapeColor { r, g, b, a: 1.0, flash }
}

fn player_ability_shape_color() -> ShapeColor {
    let (r, g, b) = palette::lookup("player_ability").unwrap_or((0.5, 0.5, 1.0));
    let flash = palette::flash_lookup("player_ability");
    ShapeColor { r, g, b, a: 1.0, flash }
}

pub fn spawn_player(
    mut commands: Commands,
    calculators: Res<StatCalculators>,
    grid: Res<RuneGrid>,
) {
    let (modifiers, computed) = compute_player_stats(&grid, &calculators);
    let mut dirty = DirtyStats::default();
    dirty.mark_all(Stat::iter());
    let hp = computed.final_of(Stat::MaxLife);

    let entity = commands.spawn((
        Name::new("Player"),
        Player,
        Transform::from_translation(Vec3::ZERO),
        Visibility::default(),
        Faction::Player,
        modifiers, dirty, computed,
        Size { value: 120.0 },
        Collider { shape: ColliderShape::Rectangle, sensor: false },
        DynamicBody { mass: 3.0 },
        Health { current: hp },
        KeyboardMovement {},
        PlayerAbilityCooldowns::default(),
        CombatScoped,
    )).id();
    commands.entity(entity).insert(PlayerInput {
        trigger: InputTrigger::MouseHold(MouseButtonKind::Left),
        targeting: TargetingMode::Cursor,
    });

    commands.entity(entity).with_children(|p| {
        p.spawn(Shadow);
        p.spawn((
            Shape {
                color: player_shape_color(), kind: ShapeKind::Circle,
                position: Vec2::ZERO, elevation: 0.5, half_length: 0.5,
            },
            JumpWalkAnimation { bounce_height: 0.6, bounce_duration: 0.45, land_squish: 0.3, land_duration: 0.125 },
        ));
    });
}

fn calc_physical_damage(stats: Option<&ComputedStats>, pct: f32) -> f32 {
    stats.map(|s| s.final_of(Stat::PhysicalDamage) * pct).unwrap_or(0.0)
}

fn calc_projectile_speed(stats: Option<&ComputedStats>, base: f32) -> f32 {
    stats.map(|s| s.apply(Stat::ProjectileSpeed, base)).unwrap_or(base)
}

fn projectile_count(stats: Option<&ComputedStats>, base: u32) -> u32 {
    let added = stats
        .map(|s| s.final_of(Stat::ProjectileCount))
        .unwrap_or(0.0)
        .max(0.0) as u32;
    base + added
}

pub fn fire_fireball(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: Vec2,
    caster_faction: Faction,
    direction: Vec2,
    caster_stats: Option<&ComputedStats>,
) {
    let count = projectile_count(caster_stats, 1).max(1);
    let speed = calc_projectile_speed(caster_stats, FIREBALL_BASE_SPEED);
    let damage = calc_physical_damage(caster_stats, FIREBALL_DAMAGE_PCT);
    let base_dir = direction.normalize_or_zero();
    let perpendicular = Vec2::new(-base_dir.y, base_dir.x);

    for i in 0..count {
        let offset = FIREBALL_GAP * (i as f32 - (count as f32 - 1.0) / 2.0);
        let spawn_pos_2d = caster_pos + perpendicular * offset;
        let ground = crate::coord::ground_pos(spawn_pos_2d);
        let velocity = base_dir * speed;

        let proj = commands.spawn((
            Transform::from_translation(ground),
            Visibility::default(),
            caster_faction,
            Caster(caster),
            Projectile,
            Size { value: FIREBALL_SIZE },
            Collider { shape: ColliderShape::Circle, sensor: true },
            RigidBody::Kinematic,
            LockedAxes::ROTATION_LOCKED.lock_translation_y(),
            LinearVelocity(crate::coord::ground_vel(velocity)),
            OnCollisionDamage { amount: damage },
            OnCollisionParticles { config: "hit_burst" },
            CombatScoped,
        )).id();

        commands.entity(proj).with_children(|p| {
            p.spawn(Shadow);
            p.spawn(Shape {
                color: player_ability_shape_color(), kind: ShapeKind::Circle,
                position: Vec2::ZERO, elevation: 2.0, half_length: 0.5,
            });
        });
    }
}

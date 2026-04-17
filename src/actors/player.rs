use avian3d::prelude::*;
use bevy::prelude::*;

use super::components::{
    Caster, Collider, DynamicBody, Health, InputTrigger, JumpWalkAnimation, KeyboardMovement,
    MouseButtonKind, OnCollisionDamage, OnCollisionParticles, PlayerAbilityCooldowns, PlayerInput,
    Projectile, Shadow, ColliderShape, Size, Shape, ShapeColor, ShapeKind,
    TargetingMode,
};
use crate::palette;
use crate::rune::{write_pattern_contains, RuneGrid, WriteEffect};
use crate::stats::{ComputedStats, DirtyStats, ModifierKind, Modifiers, Stat, StatCalculators};
use crate::wave::WavePhase;
use crate::Faction;

pub const FIREBALL_BASE_DAMAGE: f32 = 1.0;
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
    let base_stats: &[(Stat, ModifierKind, f32)] = &[
        (Stat::MaxLife, ModifierKind::Flat, 20.0),
        (Stat::MovementSpeed, ModifierKind::Flat, 550.0),
        (Stat::CritChance, ModifierKind::Flat, 0.05),
        (Stat::CritMultiplier, ModifierKind::Flat, 1.5),
        (Stat::AttackSpeed, ModifierKind::Flat, 1.0),
        (Stat::PickupRadius, ModifierKind::Flat, 200.0),
    ];

    let mut modifiers = Modifiers::new();
    for &(stat, kind, value) in base_stats {
        modifiers.add(stat, kind, value);
    }
    for (coord, rune) in grid.cells.iter() {
        let Some(kind) = rune.kind else { continue };
        let (stat, mod_kind, base_value) = kind.def().base_effect;

        let mut factor = 1.0_f32;
        for (src_coord, src_rune) in grid.cells.iter() {
            if src_coord == coord { continue }
            let Some(src_kind) = src_rune.kind else { continue };
            let Some(write) = src_kind.def().write else { continue };
            if !write_pattern_contains(&write, *src_coord, *coord) { continue }
            match write.effect {
                WriteEffect::More { factor: f } => factor *= f,
            }
        }

        modifiers.add(stat, mod_kind, base_value * factor);
    }
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::default();
    dirty.mark_all(Stat::iter());
    calculators.recalculate(&modifiers, &mut computed, &mut dirty);
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
        DespawnOnExit(WavePhase::Combat),
    )).id();
    commands.entity(entity).insert(PlayerInput {
        trigger: InputTrigger::MouseHold(MouseButtonKind::Left),
        targeting: TargetingMode::Cursor,
    });

    commands.entity(entity).with_children(|p| {
        p.spawn(Shadow { opacity: 0.45 });
        p.spawn((
            Shape {
                color: player_shape_color(), kind: ShapeKind::Circle,
                position: Vec2::ZERO, elevation: 0.5, half_length: 0.5,
            },
            JumpWalkAnimation { bounce_height: 0.6, bounce_duration: 0.45, land_squish: 0.3, land_duration: 0.125 },
        ));
    });
}

fn calc_physical_damage(stats: Option<&ComputedStats>, base: f32) -> f32 {
    stats.map(|s| s.apply(Stat::PhysicalDamage, base)).unwrap_or(base)
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
    let damage = calc_physical_damage(caster_stats, FIREBALL_BASE_DAMAGE);
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
        )).id();

        commands.entity(proj).with_children(|p| {
            p.spawn(Shadow { opacity: 0.45 });
            p.spawn(Shape {
                color: player_ability_shape_color(), kind: ShapeKind::Circle,
                position: Vec2::ZERO, elevation: 2.0, half_length: 0.5,
            });
        });
    }
}

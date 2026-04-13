use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::combat::Health;
use crate::actors::components::ability::projectile::Projectile;
use crate::actors::components::common::collider::{Collider, Shape as ColliderShape};
use crate::actors::components::common::dynamic_body::DynamicBody;
use crate::actors::components::common::jump_walk_animation::JumpWalkAnimation;
use crate::actors::components::common::shadow::Shadow;
use crate::actors::components::common::size::Size;
use crate::actors::components::common::sprite::{Sprite, SpriteColor, SpriteShape};
use crate::actors::components::player::keyboard_movement::KeyboardMovement;
use crate::actors::components::player::player_input::{
    InputTrigger, MouseButtonKind, PlayerAbilityCooldowns, PlayerInput, TargetingMode,
};
use crate::actors::effects::{OnCollisionDamage, OnCollisionParticles};
use crate::actors::{SpawnSource, TargetInfo};
use crate::palette;
use crate::stats::{ComputedStats, DirtyStats, Modifiers, Stat, StatCalculators};
use crate::wave::WavePhase;
use crate::Faction;

pub const FIREBALL_BASE_DAMAGE: f32 = 1.0;
pub const FIREBALL_BASE_SPEED: f32 = 800.0;
pub const FIREBALL_COOLDOWN: f32 = 0.5;
pub const FIREBALL_SIZE: f32 = 60.0;
pub const FIREBALL_GAP: f32 = 60.0;

#[derive(Component)]
pub struct Player;

pub fn register_systems(app: &mut App) {
    app.add_systems(OnEnter(WavePhase::Combat), spawn_player);
}

fn player_sprite_color() -> SpriteColor {
    let (r, g, b) = palette::lookup("player").unwrap_or((0.5, 0.8, 1.0));
    let flash = palette::flash_lookup("player");
    SpriteColor { r, g, b, a: 1.0, flash }
}

fn player_ability_sprite_color() -> SpriteColor {
    let (r, g, b) = palette::lookup("player_ability").unwrap_or((0.5, 0.5, 1.0));
    let flash = palette::flash_lookup("player_ability");
    SpriteColor { r, g, b, a: 1.0, flash }
}

pub fn spawn_player(
    mut commands: Commands,
    calculators: Res<StatCalculators>,
) {
    let base_stats: &[(Stat, f32)] = &[
        (Stat::MaxLifeFlat, 20.0),
        (Stat::MovementSpeedFlat, 550.0),
        (Stat::CritChanceFlat, 0.05),
        (Stat::CritMultiplier, 1.5),
        (Stat::PickupRadiusFlat, 200.0),
    ];

    let mut modifiers = Modifiers::new();
    for &(stat, value) in base_stats {
        modifiers.add(stat, value);
    }
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::default();
    dirty.mark_all(Stat::ALL.iter().copied());
    calculators.recalculate(&modifiers, &mut computed, &mut dirty);
    let hp = computed.get(Stat::MaxLife);

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
    commands.entity(entity).insert((
        SpawnSource::from_caster(entity, Vec2::ZERO),
        PlayerInput {
            trigger: InputTrigger::MouseHold(MouseButtonKind::Left),
            targeting: TargetingMode::Cursor,
        },
    ));

    commands.entity(entity).with_children(|p| {
        p.spawn(Shadow { opacity: 0.45 });
        p.spawn((
            Sprite {
                color: player_sprite_color(), shape: SpriteShape::Circle,
                position: Vec2::ZERO, scale: 1.0, elevation: 0.5, half_length: 0.5,
            },
            JumpWalkAnimation { bounce_height: 0.6, bounce_duration: 0.45, land_squish: 0.3, land_duration: 0.125 },
        ));
    });
}

fn stat_value(stats: Option<&ComputedStats>, stat: Stat) -> f32 {
    stats.map(|s| s.get(stat)).unwrap_or(0.0)
}

fn calc_physical_damage(stats: Option<&ComputedStats>, base: f32, scale: f32) -> f32 {
    let flat = stat_value(stats, Stat::PhysicalDamageFlat);
    let inc = stat_value(stats, Stat::PhysicalDamageIncreased);
    let more = stat_value(stats, Stat::PhysicalDamageMore).max(0.0001);
    (base + flat * scale) * (1.0 + inc) * more
}

fn calc_projectile_speed(stats: Option<&ComputedStats>, base: f32) -> f32 {
    let flat = stat_value(stats, Stat::ProjectileSpeedFlat);
    let inc = stat_value(stats, Stat::ProjectileSpeedIncreased);
    (base + flat) * (1.0 + inc)
}

fn projectile_count(stats: Option<&ComputedStats>, base: u32) -> u32 {
    base + stat_value(stats, Stat::ProjectileCount).max(0.0) as u32
}

pub fn fire_fireball(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: Vec2,
    caster_faction: Faction,
    target: TargetInfo,
    caster_stats: Option<&ComputedStats>,
) {
    let count = projectile_count(caster_stats, 1).max(1);
    let speed = calc_projectile_speed(caster_stats, FIREBALL_BASE_SPEED);
    let damage = calc_physical_damage(caster_stats, FIREBALL_BASE_DAMAGE, 1.0);
    let base_dir = target.direction.unwrap_or(Vec2::X).normalize_or_zero();
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
            SpawnSource::with_target(caster, caster_pos, target),
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
            p.spawn(Sprite {
                color: player_ability_sprite_color(), shape: SpriteShape::Circle,
                position: Vec2::ZERO, scale: 1.0, elevation: 2.0, half_length: 0.5,
            });
        });
    }
}

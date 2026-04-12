use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::actors::effects::{OnCollisionDamage, OnCollisionParticles};
use crate::actors::components::ability::lifetime::Lifetime;
use crate::actors::components::ability::melee_strike::MeleeStrike;
use crate::actors::components::ability::projectile::Projectile;
use crate::actors::components::common::collider::{Collider, Shape as ColliderShape};
use crate::actors::components::common::shadow::Shadow;
use crate::actors::components::common::size::Size;
use crate::actors::components::common::sprite::{Sprite, SpriteColor, SpriteShape};
use crate::actors::TargetInfo;
use crate::actors::SpawnSource;
use crate::faction::Faction;
use crate::palette;
use crate::stats::{ComputedStats, StatRegistry};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum AbilityKind {
    MeleeAttack,
    JumperShot,
    TowerShot,
    Fireball,
}

impl AbilityKind {
    #[allow(dead_code)]
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "melee_attack" => Some(AbilityKind::MeleeAttack),
            "jumper_shot" => Some(AbilityKind::JumperShot),
            "tower_shot" => Some(AbilityKind::TowerShot),
            "fireball" => Some(AbilityKind::Fireball),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn id(&self) -> &'static str {
        match self {
            AbilityKind::MeleeAttack => "melee_attack",
            AbilityKind::JumperShot => "jumper_shot",
            AbilityKind::TowerShot => "tower_shot",
            AbilityKind::Fireball => "fireball",
        }
    }
}

#[derive(Resource, Clone, Deserialize, Debug)]
pub struct AbilitiesBalance {
    pub melee_attack: MeleeAttackParams,
    pub jumper_shot: JumperShotParams,
    pub tower_shot: TowerShotParams,
    pub fireball: FireballParams,
}

#[derive(Clone, Deserialize, Debug)]
pub struct FireballParams {
    pub base_damage: f32,
    pub base_speed: f32,
    pub cooldown: f32,
    pub size: f32,
    pub gap: f32,
}

#[derive(Clone, Deserialize, Debug)]
pub struct MeleeAttackParams {
    pub range: f32,
}

#[derive(Clone, Deserialize, Debug)]
pub struct JumperShotParams {
    pub projectile_count: u32,
    pub projectile_speed: f32,
    pub projectile_size: f32,
    pub projectile_lifetime: f32,
    pub spread_degrees: f32,
}

#[derive(Clone, Deserialize, Debug)]
pub struct TowerShotParams {
    pub flight_duration: f32,
    pub arc_height: f32,
    pub start_elevation: f32,
    pub spread: f32,
    pub projectile_size: f32,
    pub explosion_radius: f32,
    pub explosion_duration: f32,
    pub indicator_duration: f32,
}

pub fn stat_flat(stats: Option<&ComputedStats>, registry: &StatRegistry, name: &str) -> f32 {
    match (stats, registry.get(name)) {
        (Some(s), Some(id)) => s.get(id),
        _ => 0.0,
    }
}

fn calc_physical_damage(stats: Option<&ComputedStats>, registry: &StatRegistry, base: f32, scale: f32) -> f32 {
    let flat = stat_flat(stats, registry, "physical_damage_flat");
    let inc = stat_flat(stats, registry, "physical_damage_increased");
    let more = stat_flat(stats, registry, "physical_damage_more").max(0.0001);
    (base + flat * scale) * (1.0 + inc) * more
}

fn calc_projectile_speed(stats: Option<&ComputedStats>, registry: &StatRegistry, base: f32) -> f32 {
    let flat = stat_flat(stats, registry, "projectile_speed_flat");
    let inc = stat_flat(stats, registry, "projectile_speed_increased");
    (base + flat) * (1.0 + inc)
}

fn projectile_count(stats: Option<&ComputedStats>, registry: &StatRegistry, base: u32) -> u32 {
    base + stat_flat(stats, registry, "projectile_count").max(0.0) as u32
}

pub fn fire_ability(
    commands: &mut Commands,
    kind: AbilityKind,
    caster: Entity,
    caster_pos: Vec2,
    caster_faction: Faction,
    target: TargetInfo,
    abilities: &AbilitiesBalance,
    caster_stats: Option<&ComputedStats>,
    stat_registry: &StatRegistry,
) {
    match kind {
        AbilityKind::MeleeAttack => fire_melee_attack(commands, caster, caster_pos, caster_faction, target, &abilities.melee_attack, caster_stats, stat_registry),
        AbilityKind::JumperShot => fire_jumper_shot(commands, caster, caster_pos, caster_faction, target, &abilities.jumper_shot, caster_stats, stat_registry),
        AbilityKind::TowerShot => crate::actors::tower_shot::fire_tower_shot_impl(commands, caster, caster_pos, caster_faction, target, &abilities.tower_shot, caster_stats, stat_registry),
        AbilityKind::Fireball => fire_fireball(commands, caster, caster_pos, caster_faction, target, &abilities.fireball, caster_stats, stat_registry),
    }
}

fn enemy_ability_sprite_color() -> SpriteColor {
    let (r, g, b) = palette::lookup("enemy_ability").unwrap_or((1.0, 0.5, 0.5));
    let flash = palette::flash_lookup("enemy_ability");
    SpriteColor { r, g, b, a: 1.0, flash }
}

fn make_projectile_source(caster: Entity, caster_pos: Vec2, caster_faction: Faction, target: TargetInfo) -> SpawnSource {
    SpawnSource {
        caster: TargetInfo::from_entity_and_position(caster, caster_pos),
        caster_faction,
        target,
    }
}

fn rotate_vec2(v: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

fn fire_jumper_shot(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: Vec2,
    caster_faction: Faction,
    target: TargetInfo,
    params: &JumperShotParams,
    caster_stats: Option<&ComputedStats>,
    stat_registry: &StatRegistry,
) {
    let damage = stat_flat(caster_stats, stat_registry, "physical_damage_flat");
    let count = params.projectile_count as usize;
    let base_dir = target.direction.unwrap_or(Vec2::X).normalize_or_zero();
    let spread_rad = params.spread_degrees.to_radians();
    let mut rng = rand::rng();

    for i in 0..count {
        let radial_angle = std::f32::consts::TAU * i as f32 / count as f32;
        let spread = rng.random_range(-spread_rad..spread_rad);
        let direction = rotate_vec2(base_dir, radial_angle + spread);
        let velocity = direction * params.projectile_speed;

        let ground = crate::coord::ground_pos(caster_pos);
        let proj = commands.spawn((
            Transform::from_translation(ground),
            Visibility::default(),
            caster_faction,
            make_projectile_source(caster, caster_pos, caster_faction, target),
            Projectile,
            Size { value: params.projectile_size },
            Collider { shape: ColliderShape::Circle, sensor: true },
            Lifetime { remaining: params.projectile_lifetime },
            RigidBody::Kinematic,
            LockedAxes::ROTATION_LOCKED.lock_translation_y(),
            LinearVelocity(crate::coord::ground_vel(velocity)),
            OnCollisionDamage { amount: damage },
        )).id();

        commands.entity(proj).with_children(|p| {
            p.spawn(Shadow { opacity: 0.45 });
            p.spawn(Sprite {
                color: enemy_ability_sprite_color(), shape: SpriteShape::Circle,
                position: Vec2::ZERO, scale: 1.0, image: None, elevation: 0.7, half_length: 0.5,
            });
        });
    }
    let _ = OnCollisionParticles { config: "enemy_ability_death" };
}

fn player_ability_sprite_color() -> SpriteColor {
    let (r, g, b) = palette::lookup("player_ability").unwrap_or((0.5, 0.5, 1.0));
    let flash = palette::flash_lookup("player_ability");
    SpriteColor { r, g, b, a: 1.0, flash }
}

fn fire_fireball(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: Vec2,
    caster_faction: Faction,
    target: TargetInfo,
    params: &FireballParams,
    caster_stats: Option<&ComputedStats>,
    stat_registry: &StatRegistry,
) {
    let count = projectile_count(caster_stats, stat_registry, 1).max(1);
    let speed = calc_projectile_speed(caster_stats, stat_registry, params.base_speed);
    let damage = calc_physical_damage(caster_stats, stat_registry, params.base_damage, 1.0);
    let base_dir = target.direction.unwrap_or(Vec2::X).normalize_or_zero();
    let perpendicular = Vec2::new(-base_dir.y, base_dir.x);

    for i in 0..count {
        let offset = params.gap * (i as f32 - (count as f32 - 1.0) / 2.0);
        let spawn_pos_2d = caster_pos + perpendicular * offset;
        let ground = crate::coord::ground_pos(spawn_pos_2d);
        let velocity = base_dir * speed;

        let proj = commands.spawn((
            Transform::from_translation(ground),
            Visibility::default(),
            caster_faction,
            make_projectile_source(caster, caster_pos, caster_faction, target),
            Projectile,
            Size { value: params.size },
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
                position: Vec2::ZERO, scale: 1.0, image: None, elevation: 2.0, half_length: 0.5,
            });
        });
    }
}

fn fire_melee_attack(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: Vec2,
    caster_faction: Faction,
    target: TargetInfo,
    params: &MeleeAttackParams,
    caster_stats: Option<&ComputedStats>,
    stat_registry: &StatRegistry,
) {
    let damage = stat_flat(caster_stats, stat_registry, "physical_damage_flat");

    let source = SpawnSource {
        caster: TargetInfo::from_entity_and_position(caster, caster_pos),
        caster_faction,
        target,
    };

    commands.spawn((
        MeleeStrike { range: params.range, damage },
        source,
        caster_faction,
    ));
}

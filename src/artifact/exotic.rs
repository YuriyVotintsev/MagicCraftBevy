use avian3d::prelude::*;
use bevy::prelude::*;

use super::effect::ExoticKind;
use crate::actors::components::combat::{
    Caster, OnCollisionDamage, PendingDamage,
};
use crate::actors::components::physics::{Collider, ColliderShape, Size};
use crate::actors::components::visual::{Shadow, Shape, ShapeColor, ShapeKind};
use crate::actors::Player;
use crate::palette;
use crate::run::CombatScoped;
use crate::schedule::GameSet;
use crate::wave::{CombatPhase, WaveEnemy};
use crate::Faction;

#[derive(Component)]
pub struct ExoticHelper;

#[derive(Component)]
pub struct OrbitOrb {
    pub angle_offset: f32,
    pub radius: f32,
    pub speed: f32,
}

#[derive(Component)]
pub struct Turret {
    pub fire_interval: f32,
    pub damage_pct: f32,
    pub cooldown: f32,
}

#[derive(Component)]
pub struct PeriodicAoe {
    pub interval: f32,
    pub radius: f32,
    pub damage_pct: f32,
    pub cooldown: f32,
}

pub fn register(app: &mut App) {
    app.add_systems(
        Update,
        (update_orbiting_orbs, tick_turrets, tick_periodic_aoe)
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(CombatPhase::Running)),
    );
}

fn ability_color() -> ShapeColor {
    let (r, g, b) = palette::lookup("player_ability").unwrap_or((0.5, 0.5, 1.0));
    let flash = palette::flash_lookup("player_ability");
    ShapeColor { r, g, b, a: 1.0, flash }
}

pub fn attach_exotic(commands: &mut Commands, player: Entity, kind: ExoticKind) {
    match kind {
        ExoticKind::OrbitingOrbs {
            count,
            radius,
            damage,
        } => {
            for i in 0..count {
                let angle_offset = (i as f32 / count.max(1) as f32) * std::f32::consts::TAU;
                let orb = commands
                    .spawn((
                        Name::new("OrbitOrb"),
                        ExoticHelper,
                        Faction::Player,
                        Caster(player),
                        Transform::from_translation(Vec3::ZERO),
                        Visibility::default(),
                        Size { value: 32.0 },
                        Collider {
                            shape: ColliderShape::Circle,
                            sensor: true,
                        },
                        RigidBody::Kinematic,
                        LockedAxes::ROTATION_LOCKED.lock_translation_y(),
                        LinearVelocity(Vec3::ZERO),
                        OnCollisionDamage { amount: damage },
                        OrbitOrb {
                            angle_offset,
                            radius,
                            speed: 1.5,
                        },
                        CombatScoped,
                    ))
                    .id();
                commands.entity(orb).with_children(|p| {
                    p.spawn(Shadow);
                    p.spawn(Shape {
                        color: ability_color(),
                        kind: ShapeKind::Circle,
                        position: Vec2::ZERO,
                        elevation: 1.5,
                        half_length: 0.5,
                    });
                });
            }
        }
        ExoticKind::Turret {
            fire_interval,
            damage_pct,
        } => {
            let turret = commands
                .spawn((
                    Name::new("ExoticTurret"),
                    ExoticHelper,
                    Faction::Player,
                    Transform::from_translation(Vec3::new(150.0, 0.0, 0.0)),
                    Visibility::default(),
                    Size { value: 60.0 },
                    Turret {
                        fire_interval,
                        damage_pct,
                        cooldown: 0.0,
                    },
                    CombatScoped,
                ))
                .id();
            commands.entity(turret).insert(Caster(turret));
            commands.entity(turret).with_children(|p| {
                p.spawn(Shadow);
                p.spawn(Shape {
                    color: ability_color(),
                    kind: ShapeKind::Circle,
                    position: Vec2::ZERO,
                    elevation: 0.6,
                    half_length: 0.7,
                });
            });
        }
        ExoticKind::PeriodicAoe { .. } => {
            // PeriodicAoe is attached as component on player by apply.rs, not here.
        }
    }
}

fn update_orbiting_orbs(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut orbs: Query<(&OrbitOrb, &mut Transform), Without<Player>>,
) {
    let Ok(player_transform) = player_q.single() else { return };
    let player_pos = crate::coord::to_2d(player_transform.translation);
    let now = time.elapsed_secs();
    for (orb, mut transform) in &mut orbs {
        let angle = orb.angle_offset + now * orb.speed;
        let pos_2d = player_pos + Vec2::new(angle.cos(), angle.sin()) * orb.radius;
        transform.translation = crate::coord::ground_pos(pos_2d) + Vec3::Y * 0.5;
    }
}

fn tick_turrets(
    mut commands: Commands,
    time: Res<Time>,
    mut turrets: Query<(Entity, &Transform, &mut Turret, &Faction)>,
    enemies: Query<&Transform, (With<WaveEnemy>, Without<Turret>)>,
    player_stats: Query<&crate::stats::ComputedStats, With<Player>>,
) {
    let dt = time.delta_secs();
    for (turret_entity, transform, mut turret, faction) in &mut turrets {
        if turret.cooldown > 0.0 {
            turret.cooldown -= dt;
            continue;
        }
        let pos = crate::coord::to_2d(transform.translation);
        let mut nearest: Option<(f32, Vec2)> = None;
        for et in &enemies {
            let ep = crate::coord::to_2d(et.translation);
            let d = (ep - pos).length_squared();
            if nearest.map(|(b, _)| d < b).unwrap_or(true) {
                nearest = Some((d, ep));
            }
        }
        let Some((_, target)) = nearest else { continue };
        let dir = (target - pos).normalize_or_zero();

        let mut scaled_stats = None;
        let stats_owned;
        if let Ok(real_stats) = player_stats.single() {
            stats_owned = scale_stats_for_turret(real_stats, turret.damage_pct);
            scaled_stats = Some(stats_owned);
        }

        crate::actors::player::fire_fireball(
            &mut commands,
            turret_entity,
            pos,
            *faction,
            dir,
            scaled_stats.as_ref(),
        );
        turret.cooldown = turret.fire_interval;
    }
}

fn scale_stats_for_turret(
    base: &crate::stats::ComputedStats,
    damage_pct: f32,
) -> crate::stats::ComputedStats {
    let mut copy = base.clone();
    let dmg = copy.final_of(crate::stats::Stat::PhysicalDamage) * damage_pct;
    copy.set_final(crate::stats::Stat::PhysicalDamage, dmg);
    copy
}

fn tick_periodic_aoe(
    mut commands: Commands,
    time: Res<Time>,
    mut player_q: Query<(&Transform, &mut PeriodicAoe, &crate::stats::ComputedStats), With<Player>>,
    enemies: Query<(Entity, &Transform), With<WaveEnemy>>,
    mut pending: MessageWriter<PendingDamage>,
) {
    let Ok((transform, mut aoe, stats)) = player_q.single_mut() else { return };
    let dt = time.delta_secs();
    if aoe.cooldown > 0.0 {
        aoe.cooldown -= dt;
        return;
    }
    let pos = crate::coord::to_2d(transform.translation);
    let radius_sq = aoe.radius * aoe.radius;
    let damage = stats.final_of(crate::stats::Stat::PhysicalDamage) * aoe.damage_pct;
    for (e, et) in &enemies {
        let ep = crate::coord::to_2d(et.translation);
        if (ep - pos).length_squared() <= radius_sq {
            pending.write(PendingDamage {
                target: e,
                amount: damage,
                source: None,
                on_hit: Default::default(),
            });
        }
    }
    crate::particles::start_particles(&mut commands, "hit_burst", pos);
    aoe.cooldown = aoe.interval;
}

use avian3d::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use super::on_collision_damage::OnCollisionDamage;
use super::Caster;
use super::PendingDamage;
use crate::arena::Wall;
use crate::artifact::OnHitEffectStack;
use crate::schedule::GameSet;
use crate::wave::WaveEnemy;
use crate::Faction;

#[derive(Component)]
pub struct Projectile;

#[derive(Component)]
pub struct PierceCount(pub u32);

#[derive(Component)]
pub struct Ricochet {
    pub remaining: u32,
}

#[derive(Component)]
pub struct Homing(pub f32);

#[derive(Component)]
pub struct Splash {
    pub radius: f32,
    pub frac_damage: f32,
}

const HOMING_TURN_RATE: f32 = 6.0;
const HOMING_RANGE: f32 = 700.0;
const RICOCHET_RANGE: f32 = 600.0;

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_projectile);
    app.add_systems(
        Update,
        (update_homing, projectile_collision_physics)
            .chain()
            .in_set(GameSet::AbilityExecution),
    );
}

fn init_projectile(mut commands: Commands, query: Query<Entity, Added<Projectile>>) {
    for entity in &query {
        commands.entity(entity).insert(Name::new("Projectile"));
    }
}

fn update_homing(
    time: Res<Time>,
    mut q_proj: Query<(&Homing, &Transform, &mut LinearVelocity), With<Projectile>>,
    enemy_q: Query<&Transform, (With<WaveEnemy>, Without<Projectile>)>,
) {
    let dt = time.delta_secs();
    for (homing, transform, mut velocity) in &mut q_proj {
        let pos = crate::coord::to_2d(transform.translation);
        let mut nearest: Option<(f32, Vec2)> = None;
        for et in &enemy_q {
            let ep = crate::coord::to_2d(et.translation);
            let d = (ep - pos).length_squared();
            if nearest.map(|(best, _)| d < best).unwrap_or(true) {
                nearest = Some((d, ep));
            }
        }
        let Some((d_sq, ep)) = nearest else { continue };
        if d_sq > HOMING_RANGE * HOMING_RANGE {
            continue;
        }
        let current_2d = crate::coord::to_2d(velocity.0);
        let speed = current_2d.length().max(1.0);
        let desired = (ep - pos).normalize_or_zero() * speed;
        let blend = (homing.0 * HOMING_TURN_RATE * dt).clamp(0.0, 1.0);
        let new_2d = current_2d.lerp(desired, blend);
        velocity.0 = crate::coord::ground_vel(new_2d.normalize_or_zero() * speed);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn projectile_collision_physics(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionStart>,
    mut proj_q: Query<
        (
            &Faction,
            Option<&mut PierceCount>,
            Option<&mut Ricochet>,
            Option<&Splash>,
            Option<&OnCollisionDamage>,
            Option<&OnHitEffectStack>,
            &Caster,
            &Transform,
            &mut LinearVelocity,
        ),
        (With<Projectile>, Without<Wall>),
    >,
    target_faction_q: Query<&Faction, Without<Projectile>>,
    enemy_q: Query<(Entity, &Transform), With<WaveEnemy>>,
    wall_q: Query<(), With<Wall>>,
    mut pending: MessageWriter<PendingDamage>,
    mut despawned: Local<HashSet<Entity>>,
) {
    despawned.clear();
    for event in collision_events.read() {
        let (proj_entity, other_entity) = if proj_q.contains(event.collider1) {
            (event.collider1, event.collider2)
        } else if proj_q.contains(event.collider2) {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if despawned.contains(&proj_entity) {
            continue;
        }

        if wall_q.contains(other_entity) {
            if let Ok(mut ec) = commands.get_entity(proj_entity) {
                ec.despawn();
                despawned.insert(proj_entity);
            }
            continue;
        }

        if proj_q.contains(other_entity) {
            continue;
        }

        let Ok((
            proj_faction,
            mut pierce_opt,
            mut ricochet_opt,
            splash_opt,
            damage_opt,
            on_hit_opt,
            _caster,
            transform,
            mut velocity,
        )) = proj_q.get_mut(proj_entity)
        else {
            continue;
        };
        let Ok(target_faction) = target_faction_q.get(other_entity) else {
            continue;
        };
        if proj_faction == target_faction {
            continue;
        }

        if let Some(splash) = splash_opt {
            if let Some(damage) = damage_opt {
                let pos = crate::coord::to_2d(transform.translation);
                let radius_sq = splash.radius * splash.radius;
                let splash_amount = damage.amount * splash.frac_damage;
                let on_hit_payload = on_hit_opt.copied().unwrap_or_default();
                for (enemy_entity, et) in &enemy_q {
                    if enemy_entity == other_entity {
                        continue;
                    }
                    let ep = crate::coord::to_2d(et.translation);
                    if (ep - pos).length_squared() <= radius_sq {
                        pending.write(PendingDamage {
                            target: enemy_entity,
                            amount: splash_amount,
                            source: None,
                            on_hit: on_hit_payload,
                        });
                    }
                }
            }
        }

        if let Some(ricochet) = ricochet_opt.as_deref_mut() {
            if ricochet.remaining > 0 {
                let pos = crate::coord::to_2d(transform.translation);
                let mut best: Option<(f32, Vec2)> = None;
                for (enemy_entity, et) in &enemy_q {
                    if enemy_entity == other_entity {
                        continue;
                    }
                    let ep = crate::coord::to_2d(et.translation);
                    let d_sq = (ep - pos).length_squared();
                    if d_sq < RICOCHET_RANGE * RICOCHET_RANGE
                        && best.map(|(b, _)| d_sq < b).unwrap_or(true)
                    {
                        best = Some((d_sq, ep));
                    }
                }
                if let Some((_, target_pos)) = best {
                    let speed = crate::coord::to_2d(velocity.0).length().max(1.0);
                    let dir = (target_pos - pos).normalize_or_zero();
                    velocity.0 = crate::coord::ground_vel(dir * speed);
                    ricochet.remaining -= 1;
                    continue;
                }
            }
        }

        if let Some(pierce) = pierce_opt.as_deref_mut() {
            if pierce.0 > 0 {
                pierce.0 -= 1;
                continue;
            }
        }

        if let Ok(mut ec) = commands.get_entity(proj_entity) {
            ec.despawn();
            despawned.insert(proj_entity);
        }
    }
}

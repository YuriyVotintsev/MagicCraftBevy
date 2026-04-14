use avian3d::prelude::{Collider as AvianCollider, *};
use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::GameState;
use super::super::components::{
    Caster, CircleSprite, Collider, FadeOut, FindNearestEnemy, Growing, Health, Lifetime,
    OnDeathParticles, PendingDamage, Shadow, Shape as ColliderShape, ShootSquish, ShotFired, Size,
    Sprite, SpriteColor, SpriteShape, StaticBody, Target,
};
use crate::faction::Faction;
use crate::palette;
use crate::particles;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat, StatCalculators};

use super::spawn::{compute_stats, current_max_life, enemy_sprite_color};

#[derive(Clone, Deserialize, Debug)]
pub struct TowerStats {
    pub hp: f32,
    pub damage: f32,
    pub size: f32,
    pub shot_cooldown: f32,
    pub flight_duration: f32,
    pub arc_height: f32,
    pub start_elevation: f32,
    pub spread: f32,
    pub projectile_size: f32,
    pub explosion_radius: f32,
    pub explosion_duration: f32,
    pub indicator_duration: f32,
}

#[derive(Component)]
pub struct TowerShooter {
    pub cooldown: f32,
    pub elapsed: f32,
    pub flight_duration: f32,
    pub arc_height: f32,
    pub start_elevation: f32,
    pub spread: f32,
    pub projectile_size: f32,
    pub explosion_radius: f32,
    pub explosion_duration: f32,
    pub indicator_duration: f32,
}

#[derive(Component)]
pub struct ArcTowerShot {
    pub start: Vec2,
    pub target: Vec2,
    pub duration: f32,
    pub arc_height: f32,
    pub start_elevation: f32,
    pub elapsed: f32,
    pub explosion_radius: f32,
    pub explosion_duration: f32,
    pub indicator_duration: f32,
    pub damage: f32,
    pub caster: Entity,
    pub caster_faction: Faction,
    pub sprite_entity: Option<Entity>,
    pub spawned_indicator: bool,
}

const TOWER_CYLINDER_RADIUS: f32 = 0.2;
const TOWER_CYLINDER_HEIGHT: f32 = 0.6;

#[derive(Component)]
pub struct TowerVisual {}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        tower_shooter_system.in_set(GameSet::MobAI),
    );
    app.add_systems(
        Update,
        update_arc_tower_shot
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
    app.add_systems(PostUpdate, init_tower_visual);
}

fn init_tower_visual(
    mut commands: Commands,
    query: Query<Entity, Added<TowerVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &query {
        let color = palette::color("enemy_ability");
        let material = materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        });
        let mesh = meshes.add(Cylinder::new(TOWER_CYLINDER_RADIUS, TOWER_CYLINDER_HEIGHT));
        let cylinder = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(0.0, TOWER_CYLINDER_HEIGHT / 2.0, 0.0)),
            ))
            .id();
        commands.entity(entity).add_child(cylinder);
    }
}

pub fn spawn_tower(
    commands: &mut Commands,
    pos: Vec2,
    s: &TowerStats,
    calculators: &StatCalculators,
    extra_modifiers: &[(Stat, f32)],
) -> Entity {
    let (modifiers, dirty, computed) = compute_stats(
        calculators,
        &[(Stat::MaxLifeFlat, s.hp), (Stat::PhysicalDamageFlat, s.damage)],
        extra_modifiers,
    );
    let hp = current_max_life(&computed);
    let ground = crate::coord::ground_pos(pos);

    let id = commands.spawn((
        Transform::from_translation(ground),
        Visibility::default(),
        Faction::Enemy,
        modifiers, dirty, computed,
        Size { value: s.size },
        Collider { shape: ColliderShape::Circle, sensor: false },
        StaticBody,
        Health { current: hp },
        TowerVisual {},
        ShootSquish { amplitude: 0.3, duration: 0.25 },
        FindNearestEnemy { size: 4000.0, center: Entity::PLACEHOLDER },
        TowerShooter {
            cooldown: s.shot_cooldown,
            elapsed: 0.0,
            flight_duration: s.flight_duration,
            arc_height: s.arc_height,
            start_elevation: s.start_elevation,
            spread: s.spread,
            projectile_size: s.projectile_size,
            explosion_radius: s.explosion_radius,
            explosion_duration: s.explosion_duration,
            indicator_duration: s.indicator_duration,
        },
    )).id();

    commands.entity(id).insert((
        Caster(id),
        FindNearestEnemy { size: 4000.0, center: id },
        OnDeathParticles { config: "enemy_death_large" },
    ));

    commands.entity(id).with_children(|p| {
        p.spawn(Shadow { opacity: 0.45 });
        p.spawn(Sprite {
            color: enemy_sprite_color(), shape: SpriteShape::Circle,
            position: Vec2::ZERO, scale: 1.0, elevation: 1.2, half_length: 0.5,
        });
    });

    id
}

fn tower_shooter_system(
    mut commands: Commands,
    time: Res<Time>,
    transforms: Query<&Transform, Without<TowerShooter>>,
    stats_query: Query<&ComputedStats>,
    mut query: Query<(Entity, &Transform, &mut TowerShooter, Option<&Target>, &Faction), Without<crate::wave::RiseFromGround>>,
) {
    for (caster, transform, mut shooter, target, faction) in &mut query {
        shooter.elapsed += time.delta_secs();
        if shooter.elapsed < shooter.cooldown { continue }

        let Some(target) = target else { continue };
        let Ok(target_transform) = transforms.get(target.0) else { continue };

        shooter.elapsed = 0.0;

        let caster_pos = crate::coord::to_2d(transform.translation);
        let target_pos = crate::coord::to_2d(target_transform.translation);
        let caster_stats = stats_query.get(caster).ok();

        commands.entity(caster).insert(ShotFired);
        fire_tower_shot(&mut commands, caster, caster_pos, *faction, target_pos, &shooter, caster_stats);
    }
}

fn fire_tower_shot(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: Vec2,
    caster_faction: Faction,
    mut target_pos: Vec2,
    shooter: &TowerShooter,
    caster_stats: Option<&ComputedStats>,
) {
    let damage = caster_stats.map(|s| s.get(Stat::PhysicalDamageFlat)).unwrap_or(0.0);
    if shooter.spread > 0.0 {
        let mut rng = rand::rng();
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let dist = rng.random_range(0.0..shooter.spread);
        target_pos += Vec2::new(angle.cos(), angle.sin()) * dist;
    }
    let ground = crate::coord::ground_pos(caster_pos);
    let proj = commands.spawn((
        Transform::from_translation(ground),
        Visibility::default(),
        caster_faction,
        ArcTowerShot {
            start: caster_pos, target: target_pos,
            duration: shooter.flight_duration, arc_height: shooter.arc_height,
            start_elevation: shooter.start_elevation, elapsed: 0.0,
            explosion_radius: shooter.explosion_radius,
            explosion_duration: shooter.explosion_duration,
            indicator_duration: shooter.indicator_duration,
            damage, caster, caster_faction,
            sprite_entity: None, spawned_indicator: false,
        },
        Size { value: shooter.projectile_size },
    )).id();
    commands.entity(proj).with_children(|p| {
        p.spawn(Shadow { opacity: 0.45 });
        p.spawn(Sprite {
            color: {
                let (r, g, b) = palette::lookup("enemy_ability").unwrap_or((1.0, 0.5, 0.5));
                SpriteColor { r, g, b, a: 1.0, flash: palette::flash_lookup("enemy_ability") }
            },
            shape: SpriteShape::Circle,
            position: Vec2::ZERO, scale: 1.0, elevation: 0.5, half_length: 0.5,
        });
    });
}

fn enemy_ability_color_alpha(alpha: f32) -> SpriteColor {
    let (r, g, b) = palette::lookup("enemy_ability").unwrap_or((1.0, 0.5, 0.5));
    SpriteColor { r, g, b, a: alpha, flash: None }
}

#[allow(clippy::too_many_arguments)]
fn update_arc_tower_shot(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ArcTowerShot, &mut Transform)>,
    mut child_transforms: Query<&mut Transform, Without<ArcTowerShot>>,
    children_query: Query<&Children>,
    sprite_marker: Query<Entity, With<CircleSprite>>,
    mut pending: MessageWriter<PendingDamage>,
    spatial: SpatialQuery,
    faction_query: Query<&Faction>,
) {
    let dt = time.delta_secs();
    for (entity, mut arc, mut transform) in &mut query {
        arc.elapsed += dt;
        let t = (arc.elapsed / arc.duration).clamp(0.0, 1.0);

        let start3 = crate::coord::ground_pos(arc.start);
        let end3 = crate::coord::ground_pos(arc.target);
        let ground = start3.lerp(end3, t);
        transform.translation.x = ground.x;
        transform.translation.z = ground.z;

        let arc_h = arc.arc_height * 4.0 * t * (1.0 - t);
        let elev = arc.start_elevation * (1.0 - t);
        let height = arc_h + elev;

        if !arc.spawned_indicator {
            arc.spawned_indicator = true;
            let ground_ind = crate::coord::ground_pos(arc.target);
            commands.spawn((
                Transform::from_translation(ground_ind),
                Visibility::default(),
                Size { value: arc.explosion_radius },
                Sprite {
                    color: enemy_ability_color_alpha(0.2), shape: SpriteShape::Disc,
                    position: Vec2::ZERO, scale: 1.0, elevation: 0.02, half_length: 0.5,
                },
                Growing { start_size: 0.0, end_size: arc.explosion_radius },
                Lifetime { remaining: arc.indicator_duration },
            ));
        }

        if arc.sprite_entity.is_none() {
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    if sprite_marker.contains(child) {
                        arc.sprite_entity = Some(child);
                    } else if let Ok(grand) = children_query.get(child) {
                        for gc in grand.iter() {
                            if sprite_marker.contains(gc) {
                                arc.sprite_entity = Some(gc);
                            }
                        }
                    }
                }
            }
        }
        if let Some(se) = arc.sprite_entity {
            if let Ok(mut tf) = child_transforms.get_mut(se) {
                tf.translation.y = height;
            }
        }

        if t >= 1.0 {
            let target_ground = crate::coord::ground_pos(arc.target);
            particles::start_particles(&mut commands, "tower_explosion", arc.target);

            commands.spawn((
                Transform::from_translation(target_ground),
                Visibility::default(),
                Size { value: arc.explosion_radius },
                Sprite {
                    color: enemy_ability_color_alpha(0.4), shape: SpriteShape::Disc,
                    position: Vec2::ZERO, scale: 1.0, elevation: 0.02, half_length: 0.5,
                },
                Lifetime { remaining: arc.explosion_duration },
                FadeOut {},
            ));

            let shape = AvianCollider::sphere(arc.explosion_radius / 2.0);
            let filter = SpatialQueryFilter::from_mask(arc.caster_faction.enemy_layer());
            let hits = spatial.shape_intersections(&shape, target_ground, Quat::IDENTITY, &filter);
            for hit in hits {
                if faction_query.get(hit).map(|f| *f != arc.caster_faction).unwrap_or(false) {
                    pending.write(PendingDamage { target: hit, amount: arc.damage, source: Some(arc.caster) });
                }
            }

            commands.entity(entity).despawn();
        }
    }
}

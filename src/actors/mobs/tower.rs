use avian3d::prelude::{Collider as AvianCollider, *};
use bevy::prelude::*;
use rand::Rng;

use crate::GameState;
use crate::balance::MobCommonStats;
use super::super::components::{
    CircleShape, Growing, Lifetime, PendingDamage, ScaleOut, Shadow, ShootSquish, ShotFired, Size,
    Shape, ShapeColor, ShapeKind,
};
use crate::faction::Faction;
use crate::palette;
use crate::particles;
use crate::run::CombatScoped;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, ModifierKind, Stat, StatCalculators};

use super::spawn::{enemy_shape_color, spawn_enemy_core, EnemyBody, WaveModifiers};

const TOWER_FLIGHT_DURATION: f32 = 0.8;
const TOWER_ARC_HEIGHT: f32 = 8.0;
const TOWER_START_ELEVATION: f32 = 1.6;
const TOWER_SPREAD: f32 = 450.0;
const TOWER_PROJECTILE_SIZE: f32 = 60.0;
const TOWER_EXPLOSION_RADIUS: f32 = 400.0;
const TOWER_EXPLOSION_DURATION: f32 = 0.5;
const TOWER_INDICATOR_DURATION: f32 = 0.8;

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
    pub shape_entity: Option<Entity>,
    pub spawned_indicator: bool,
}

const TOWER_CYLINDER_RADIUS: f32 = 0.2;
const TOWER_CYLINDER_HEIGHT: f32 = 0.6;
const TOWER_SHOT_DAMAGE_PCT: f32 = 1.0;

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
    s: &MobCommonStats,
    calculators: &StatCalculators,
    wave_mods: WaveModifiers,
) -> Entity {
    let id = spawn_enemy_core(
        commands,
        pos,
        calculators,
        &[
            (Stat::MaxLife, ModifierKind::Flat, s.hp),
            (Stat::PhysicalDamage, ModifierKind::Flat, s.damage),
        ],
        s.size,
        EnemyBody::Static,
        "enemy_death_large",
        wave_mods,
    );

    commands.entity(id).insert((
        TowerVisual {},
        ShootSquish { amplitude: 0.3, duration: 0.25 },
        TowerShooter {
            cooldown: s.attack_speed.unwrap_or(2.5),
            elapsed: 0.0,
            flight_duration: TOWER_FLIGHT_DURATION,
            arc_height: TOWER_ARC_HEIGHT,
            start_elevation: TOWER_START_ELEVATION,
            spread: TOWER_SPREAD,
            projectile_size: TOWER_PROJECTILE_SIZE,
            explosion_radius: TOWER_EXPLOSION_RADIUS,
            explosion_duration: TOWER_EXPLOSION_DURATION,
            indicator_duration: TOWER_INDICATOR_DURATION,
        },
    ));

    commands.entity(id).with_children(|p| {
        p.spawn(Shape {
            color: enemy_shape_color(), kind: ShapeKind::Circle,
            position: Vec2::ZERO, elevation: 1.2, half_length: 0.5,
        });
    });

    id
}

fn tower_shooter_system(
    mut commands: Commands,
    time: Res<Time>,
    stats_query: Query<&ComputedStats>,
    mut query: Query<(Entity, &Transform, &mut TowerShooter, &Faction), Without<crate::wave::RiseFromGround>>,
    player: Option<Single<&Transform, (With<crate::actors::Player>, Without<TowerShooter>)>>,
) {
    let Some(player) = player else { return };
    for (caster, transform, mut shooter, faction) in &mut query {
        shooter.elapsed += time.delta_secs();
        if shooter.elapsed < shooter.cooldown { continue }

        shooter.elapsed = 0.0;

        let caster_pos = crate::coord::to_2d(transform.translation);
        let target_pos = crate::coord::to_2d(player.translation);
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
    let damage = caster_stats
        .map(|s| s.final_of(Stat::PhysicalDamage) * TOWER_SHOT_DAMAGE_PCT)
        .unwrap_or(0.0);
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
            shape_entity: None, spawned_indicator: false,
        },
        Size { value: shooter.projectile_size },
        CombatScoped,
    )).id();
    commands.entity(proj).with_children(|p| {
        p.spawn(Shadow);
        p.spawn(Shape {
            color: {
                let (r, g, b) = palette::lookup("enemy_ability").unwrap_or((1.0, 0.5, 0.5));
                ShapeColor { r, g, b, a: 1.0, flash: palette::flash_lookup("enemy_ability") }
            },
            kind: ShapeKind::Circle,
            position: Vec2::ZERO, elevation: 0.5, half_length: 0.5,
        });
    });
}

fn enemy_ability_color_alpha(alpha: f32) -> ShapeColor {
    let (r, g, b) = palette::lookup("enemy_ability").unwrap_or((1.0, 0.5, 0.5));
    ShapeColor { r, g, b, a: alpha, flash: None }
}

fn coral_light_color() -> ShapeColor {
    let (r, g, b) = palette::lookup("coral_light").unwrap_or((1.0, 0.7, 0.6));
    ShapeColor { r, g, b, a: 1.0, flash: None }
}

#[allow(clippy::too_many_arguments)]
fn update_arc_tower_shot(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ArcTowerShot, &mut Transform)>,
    mut child_transforms: Query<&mut Transform, Without<ArcTowerShot>>,
    children_query: Query<&Children>,
    shape_marker: Query<Entity, With<CircleShape>>,
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
                Shape {
                    color: enemy_ability_color_alpha(0.2), kind: ShapeKind::Disc,
                    position: Vec2::ZERO, elevation: 0.02, half_length: 0.5,
                },
                Growing { start_size: 0.0, end_size: arc.explosion_radius },
                Lifetime { remaining: arc.indicator_duration },
                CombatScoped,
            ));
        }

        if arc.shape_entity.is_none() {
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    if shape_marker.contains(child) {
                        arc.shape_entity = Some(child);
                    } else if let Ok(grand) = children_query.get(child) {
                        for gc in grand.iter() {
                            if shape_marker.contains(gc) {
                                arc.shape_entity = Some(gc);
                            }
                        }
                    }
                }
            }
        }
        if let Some(se) = arc.shape_entity {
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
                Shape {
                    color: coral_light_color(), kind: ShapeKind::Disc,
                    position: Vec2::ZERO, elevation: 0.02, half_length: 0.5,
                },
                Lifetime { remaining: arc.explosion_duration },
                ScaleOut {},
                CombatScoped,
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

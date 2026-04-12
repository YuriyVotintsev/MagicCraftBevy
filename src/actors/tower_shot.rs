use avian3d::prelude::{Collider as AvianCollider, *};
use bevy::prelude::*;
use rand::Rng;
use crate::actors::components::common::fade_out::FadeOut;
use crate::actors::components::common::shadow::Shadow;
use crate::actors::components::common::size::Size;
use crate::actors::components::common::sprite::{Sprite, SpriteColor, SpriteShape};
use crate::faction::Faction;
use crate::palette;
use crate::particles;
use crate::schedule::GameSet;
use crate::stats::PendingDamage;
use crate::GameState;

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

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        update_arc_tower_shot
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
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
    sprite_marker: Query<Entity, With<crate::actors::components::common::sprite::CircleSprite>>,
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
            use crate::actors::components::ability::growing::Growing;
            use crate::actors::components::ability::lifetime::Lifetime;
            commands.spawn((
                Transform::from_translation(ground_ind),
                Visibility::default(),
                Size { value: arc.explosion_radius },
                Sprite {
                    color: enemy_ability_color_alpha(0.2), shape: SpriteShape::Disc,
                    position: Vec2::ZERO, scale: 1.0, image: None, elevation: 0.02, half_length: 0.5,
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

            use crate::actors::components::ability::lifetime::Lifetime;
            commands.spawn((
                Transform::from_translation(target_ground),
                Visibility::default(),
                Size { value: arc.explosion_radius },
                Sprite {
                    color: enemy_ability_color_alpha(0.4), shape: SpriteShape::Disc,
                    position: Vec2::ZERO, scale: 1.0, image: None, elevation: 0.02, half_length: 0.5,
                },
                Lifetime { remaining: arc.explosion_duration },
                FadeOut {},
            ));

            let shape = AvianCollider::sphere(arc.explosion_radius / 2.0);
            let target_layer = match arc.caster_faction {
                Faction::Player => crate::physics::GameLayer::Enemy,
                Faction::Enemy => crate::physics::GameLayer::Player,
            };
            let filter = SpatialQueryFilter::from_mask(target_layer);
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

pub fn fire_tower_shot_impl(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: Vec2,
    caster_faction: Faction,
    target: crate::actors::TargetInfo,
    params: &crate::actors::abilities::TowerShotParams,
    caster_stats: Option<&crate::stats::ComputedStats>,
    stat_registry: &crate::stats::StatRegistry,
) {
    let damage = crate::actors::abilities::stat_flat(caster_stats, stat_registry, "physical_damage_flat");
    let mut target_pos = target.position.unwrap_or(caster_pos);
    if params.spread > 0.0 {
        let mut rng = rand::rng();
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let dist = rng.random_range(0.0..params.spread);
        target_pos += Vec2::new(angle.cos(), angle.sin()) * dist;
    }
    let ground = crate::coord::ground_pos(caster_pos);
    let proj = commands.spawn((
        Transform::from_translation(ground),
        Visibility::default(),
        caster_faction,
        ArcTowerShot {
            start: caster_pos, target: target_pos,
            duration: params.flight_duration, arc_height: params.arc_height,
            start_elevation: params.start_elevation, elapsed: 0.0,
            explosion_radius: params.explosion_radius,
            explosion_duration: params.explosion_duration,
            indicator_duration: params.indicator_duration,
            damage, caster, caster_faction,
            sprite_entity: None, spawned_indicator: false,
        },
        Size { value: params.projectile_size },
    )).id();
    commands.entity(proj).with_children(|p| {
        p.spawn(Shadow { y_offset: -0.5, opacity: 0.45 });
        p.spawn(Sprite {
            color: {
                let (r, g, b) = palette::lookup("enemy_ability").unwrap_or((1.0, 0.5, 0.5));
                SpriteColor { r, g, b, a: 1.0, flash: palette::flash_lookup("enemy_ability") }
            },
            shape: SpriteShape::Circle,
            position: Vec2::ZERO, scale: 1.0, image: None, elevation: 0.5, half_length: 0.5,
        });
    });
}

use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::components::common::sprite::CircleSprite;
use crate::blueprints::context::TargetInfo;
use crate::blueprints::spawn::EntitySpawner;
use crate::blueprints::SpawnSource;
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::GameState;

#[blueprint_component(SOURCE_POSITION)]
pub struct ArcProjectile {
    pub duration: ScalarExpr,
    pub arc_height: ScalarExpr,
    #[raw(default = 0.0)]
    pub start_elevation: ScalarExpr,
    #[raw(default = 0.0)]
    pub spread: ScalarExpr,
    #[default_expr("source.position")]
    pub start_position: VecExpr,
    #[default_expr("target.position")]
    pub target_position: VecExpr,
    pub entities: Vec<EntityDef>,
}

#[derive(Component)]
pub struct ArcProjectileProgress {
    pub elapsed: f32,
    pub sprite_entity: Option<Entity>,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_arc_progress, update_arc_projectiles)
            .chain()
            .in_set(GameSet::BlueprintExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_arc_progress(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ArcProjectile), Without<ArcProjectileProgress>>,
) {
    for (entity, mut arc) in &mut query {
        if arc.spread > 0.0 {
            let angle = rand::random_range(0.0..std::f32::consts::TAU);
            let dist = rand::random_range(0.0..arc.spread);
            arc.target_position += Vec2::new(angle.cos(), angle.sin()) * dist;
        }
        commands.entity(entity).insert(ArcProjectileProgress {
            elapsed: 0.0,
            sprite_entity: None,
        });
    }
}

fn update_arc_projectiles(
    mut spawner: EntitySpawner,
    time: Res<Time>,
    mut query: Query<(Entity, &ArcProjectile, &mut ArcProjectileProgress, &SpawnSource, &mut Transform)>,
    stats_query: Query<&ComputedStats>,
    mut other_transforms: Query<&mut Transform, Without<ArcProjectile>>,
    children_query: Query<&Children>,
    circle_query: Query<Entity, With<CircleSprite>>,
) {
    let dt = time.delta_secs();

    for (entity, arc, mut progress, source, mut transform) in &mut query {
        progress.elapsed += dt;
        let t = (progress.elapsed / arc.duration).clamp(0.0, 1.0);

        let start = crate::coord::ground_pos(arc.start_position);
        let end = crate::coord::ground_pos(arc.target_position);
        let ground = start.lerp(end, t);
        transform.translation.x = ground.x;
        transform.translation.z = ground.z;

        let arc_h = arc.arc_height * 4.0 * t * (1.0 - t);
        let elev = arc.start_elevation * (1.0 - t);
        let height = arc_h + elev;

        if progress.sprite_entity.is_none() {
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    if circle_query.contains(child) {
                        progress.sprite_entity = Some(child);
                    } else if let Ok(grandchildren) = children_query.get(child) {
                        for grandchild in grandchildren.iter() {
                            if circle_query.contains(grandchild) {
                                progress.sprite_entity = Some(grandchild);
                            }
                        }
                    }
                }
            }
        }

        if let Some(sprite_entity) = progress.sprite_entity {
            if let Ok(mut sprite_tf) = other_transforms.get_mut(sprite_entity) {
                sprite_tf.translation.y = height;
            }
        }

        if t >= 1.0 {
            let target_pos = arc.target_position;
            let entities = arc.entities.clone();
            let readonly = other_transforms.reborrow().into_readonly();
            spawner.spawn_triggered(
                entity,
                source,
                TargetInfo::from_position(target_pos),
                TargetInfo::EMPTY,
                &entities,
                &stats_query,
                &readonly,
            );

            spawner.commands.entity(entity).despawn();
        }
    }
}

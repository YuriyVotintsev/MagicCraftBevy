use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_node;
use crate::abilities::NodeRegistry;
use crate::abilities::ParamValue;
use crate::abilities::ids::NodeTypeId;
use crate::abilities::AbilitySource;
use crate::building_blocks::actions::ExecuteMeteorEvent;
use crate::building_blocks::triggers::on_area::OnAreaTrigger;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::Faction;
use crate::Lifetime;
use crate::GameState;

#[derive(Debug, Clone, Default, GenerateRaw)]
#[node(kind = Action)]
pub struct MeteorParams {
    #[raw(default = 500.0)]
    pub search_radius: ParamValue,
    #[raw(default = 80.0)]
    pub damage_radius: ParamValue,
    #[raw(default = 0.5)]
    pub fall_duration: ParamValue,
}

const METEOR_START_HEIGHT: f32 = 400.0;
const METEOR_SIZE: f32 = 40.0;
const EXPLOSION_DURATION: f32 = 0.3;

#[derive(Component)]
pub struct MeteorRequest {
    pub search_radius: f32,
    pub damage_radius: f32,
    pub fall_duration: f32,
    pub has_area_trigger: bool,
}

#[derive(Component)]
pub struct MeteorFalling {
    pub target_position: Vec3,
    pub damage_radius: f32,
    pub fall_duration: f32,
    pub elapsed: f32,
    pub has_area_trigger: bool,
}

#[derive(Component)]
pub struct MeteorIndicator;

#[derive(Component)]
pub struct MeteorExplosion;

fn execute_meteor_action(
    mut commands: Commands,
    mut action_events: MessageReader<ExecuteMeteorEvent>,
    node_registry: Res<NodeRegistry>,
    stats_query: Query<&ComputedStats>,
    mut cached_area_id: Local<Option<NodeTypeId>>,
) {
    let area_id = *cached_area_id.get_or_insert_with(|| {
        node_registry.get_id("OnAreaParams")
            .expect("OnAreaParams not registered")
    });

    for event in action_events.read() {
        let caster_stats = stats_query
            .get(event.base.context.caster)
            .unwrap_or(&DEFAULT_STATS);

        let search_radius = event.params.search_radius.evaluate_f32(&caster_stats);
        let damage_radius = event.params.damage_radius.evaluate_f32(&caster_stats);
        let fall_duration = event.params.fall_duration.evaluate_f32(&caster_stats);

        let has_area_trigger = event.base.child_triggers.contains(&area_id);

        commands.spawn((
            Name::new("MeteorRequest"),
            MeteorRequest {
                search_radius,
                damage_radius,
                fall_duration,
                has_area_trigger,
            },
            AbilitySource::new(
                event.base.ability_id,
                event.base.node_id,
                event.base.context.caster,
                event.base.context.caster_faction,
            ),
            Transform::from_translation(event.base.context.source.as_point().unwrap_or(Vec3::ZERO)),
        ));
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        execute_meteor_action
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
    app.add_systems(
        Update,
        (meteor_target_finder, meteor_falling_update).in_set(GameSet::AbilityExecution),
    );
}

fn meteor_target_finder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &MeteorRequest, &AbilitySource, &Transform)>,
    spatial_query: SpatialQuery,
    transforms: Query<&Transform, Without<MeteorRequest>>,
) {
    for (request_entity, request, source, caster_transform) in &query {
        let caster_pos = caster_transform.translation.truncate();

        let target_layer = match source.caster_faction {
            Faction::Player => GameLayer::Enemy,
            Faction::Enemy => GameLayer::Player,
        };

        let filter = SpatialQueryFilter::from_mask(target_layer);
        let shape = Collider::circle(request.search_radius);
        let hits = spatial_query.shape_intersections(&shape, caster_pos, 0.0, &filter);

        commands.entity(request_entity).despawn();

        let Some(target_pos) = hits
            .iter()
            .filter_map(|&entity| {
                let pos = transforms.get(entity).ok()?.translation.truncate();
                Some((caster_pos.distance_squared(pos), pos))
            })
            .min_by(|(dist_a, _), (dist_b, _)| dist_a.total_cmp(dist_b))
            .map(|(_, pos)| pos)
        else {
            continue;
        };

        let target_position = target_pos.extend(0.0);

        let indicator_mesh = meshes.add(Circle::new(request.damage_radius));
        let indicator_material = materials.add(ColorMaterial::from_color(Color::srgba(1.0, 0.3, 0.0, 0.3)));

        commands.spawn((
            Name::new("MeteorIndicator"),
            MeteorIndicator,
            Mesh2d(indicator_mesh),
            MeshMaterial2d(indicator_material),
            Transform::from_translation(target_position.with_z(-1.0)),
            Lifetime { remaining: request.fall_duration + EXPLOSION_DURATION },
        ));

        let meteor_mesh = meshes.add(Circle::new(METEOR_SIZE / 2.0));
        let meteor_material = materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.5, 0.0)));

        let start_position = target_position + Vec3::new(0.0, METEOR_START_HEIGHT, 0.0);
        commands.spawn((
            Name::new("MeteorFalling"),
            MeteorFalling {
                target_position,
                damage_radius: request.damage_radius,
                fall_duration: request.fall_duration,
                elapsed: 0.0,
                has_area_trigger: request.has_area_trigger,
            },
            source.clone(),
            Mesh2d(meteor_mesh),
            MeshMaterial2d(meteor_material),
            Transform::from_translation(start_position),
        ));
    }
}

fn meteor_falling_update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut MeteorFalling, &AbilitySource, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut meteor, source, mut transform) in &mut query {
        meteor.elapsed += dt;
        let t = (meteor.elapsed / meteor.fall_duration).clamp(0.0, 1.0);
        let eased_t = t * t;

        let start_y = meteor.target_position.y + METEOR_START_HEIGHT;
        let current_y = start_y - (METEOR_START_HEIGHT * eased_t);
        transform.translation.y = current_y;

        if t >= 1.0 {
            commands.entity(entity).despawn();

            let explosion_mesh = meshes.add(Circle::new(meteor.damage_radius));
            let explosion_material = materials.add(ColorMaterial::from_color(Color::srgba(1.0, 0.5, 0.0, 0.8)));

            let mut explosion_entity = commands.spawn((
                Name::new("MeteorExplosion"),
                MeteorExplosion,
                source.clone(),
                Mesh2d(explosion_mesh),
                MeshMaterial2d(explosion_material),
                Transform::from_translation(meteor.target_position),
                Lifetime { remaining: EXPLOSION_DURATION },
            ));

            if meteor.has_area_trigger {
                explosion_entity.insert(OnAreaTrigger::new(meteor.damage_radius));
            }
        }
    }
}

register_node!(MeteorParams);

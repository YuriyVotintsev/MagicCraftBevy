use std::collections::HashMap;
use avian2d::prelude::*;
use bevy::prelude::*;
use crate::register_node;

use crate::abilities::{AbilityRegistry, NodeRegistry};
use crate::abilities::{ParamValue, ParamValueRaw, ParseNodeParams, resolve_param_value};
use crate::abilities::node::{NodeHandler, NodeKind};
use crate::abilities::events::ExecuteNodeEvent;
use crate::abilities::AbilitySource;
use crate::building_blocks::triggers::on_area::OnAreaTrigger;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS, StatRegistry};
use crate::Faction;
use crate::Lifetime;
use crate::GameState;

#[derive(Debug, Clone)]
pub struct MeteorParams {
    pub search_radius: ParamValue,
    pub damage_radius: ParamValue,
    pub fall_duration: ParamValue,
}

impl ParseNodeParams for MeteorParams {
    fn parse(raw: &HashMap<String, ParamValueRaw>, stat_registry: &StatRegistry) -> Self {
        Self {
            search_radius: raw.get("search_radius")
                .map(|v| resolve_param_value(v, stat_registry))
                .unwrap_or(ParamValue::Float(500.0)),
            damage_radius: raw.get("damage_radius")
                .map(|v| resolve_param_value(v, stat_registry))
                .unwrap_or(ParamValue::Float(80.0)),
            fall_duration: raw.get("fall_duration")
                .map(|v| resolve_param_value(v, stat_registry))
                .unwrap_or(ParamValue::Float(0.5)),
        }
    }
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
    mut action_events: MessageReader<ExecuteNodeEvent>,
    node_registry: Res<NodeRegistry>,
    ability_registry: Res<AbilityRegistry>,
    stats_query: Query<&ComputedStats>,
) {
    let Some(handler_id) = node_registry.get_id("spawn_meteor") else {
        return;
    };

    for event in action_events.read() {
        let Some(ability_def) = ability_registry.get(event.ability_id) else {
            continue;
        };
        let Some(node_def) = ability_def.get_node(event.node_id) else {
            continue;
        };

        if node_def.node_type != handler_id {
            continue;
        }

        let params = node_def.params.unwrap_action().unwrap_meteor();

        let caster_stats = stats_query
            .get(event.context.caster)
            .unwrap_or(&DEFAULT_STATS);

        let search_radius = params.search_radius.evaluate_f32(&caster_stats);
        let damage_radius = params.damage_radius.evaluate_f32(&caster_stats);
        let fall_duration = params.fall_duration.evaluate_f32(&caster_stats);

        let has_area_trigger = OnAreaTrigger::if_configured(ability_def, event.node_id, &node_registry, damage_radius).is_some();

        commands.spawn((
            Name::new("MeteorRequest"),
            MeteorRequest {
                search_radius,
                damage_radius,
                fall_duration,
                has_area_trigger,
            },
            AbilitySource::new(
                event.ability_id,
                event.node_id,
                event.context.caster,
                event.context.caster_faction,
            ),
            Transform::from_translation(event.context.source.as_point().unwrap_or(Vec3::ZERO)),
        ));
    }
}

#[derive(Default)]
pub struct SpawnMeteorHandler;

impl NodeHandler for SpawnMeteorHandler {
    fn name(&self) -> &'static str {
        "spawn_meteor"
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Action
    }

    fn register_execution_system(&self, app: &mut App) {
        app.add_systems(
            Update,
            execute_meteor_action
                .in_set(GameSet::AbilityExecution)
                .run_if(in_state(GameState::Playing)),
        );
    }

    fn register_behavior_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            (meteor_target_finder, meteor_falling_update).in_set(GameSet::AbilityExecution),
        );
    }
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

register_node!(SpawnMeteorHandler, params: MeteorParams, name: "spawn_meteor");

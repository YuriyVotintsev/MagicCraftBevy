use bevy::prelude::*;
use avian2d::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_node;
use crate::abilities::node::NodeRegistry;
use crate::abilities::ids::NodeTypeId;
use crate::abilities::context::{AbilityContext, Target};
use crate::abilities::events::NodeTriggerEvent;
use crate::abilities::AbilitySource;
use crate::abilities::ParamValue;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::Faction;

#[derive(Debug, Clone, Default, GenerateRaw)]
#[node(kind = Trigger)]
pub struct OnAreaParams {
    #[raw(default = 100.0)]
    pub radius: ParamValue,
    pub interval: Option<ParamValue>,
}

#[derive(Component)]
pub struct OnAreaTrigger {
    pub radius: f32,
    pub interval: Option<f32>,
    pub timer: f32,
}

impl OnAreaTrigger {
    pub fn new(radius: f32) -> Self {
        Self { radius, interval: None, timer: 0.0 }
    }

    pub fn with_interval(radius: f32, interval: Option<f32>) -> Self {
        Self { radius, interval, timer: 0.0 }
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Update, on_area_trigger_system.in_set(GameSet::AbilityExecution));
}

fn on_area_trigger_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut OnAreaTrigger, &AbilitySource, &Transform)>,
    mut trigger_events: MessageWriter<NodeTriggerEvent>,
    spatial_query: SpatialQuery,
    node_registry: Res<NodeRegistry>,
    mut cached_id: Local<Option<NodeTypeId>>,
) {
    let trigger_id = *cached_id.get_or_insert_with(|| {
        node_registry.get_id("OnAreaParams")
            .expect("OnAreaParams not registered")
    });

    let dt = time.delta_secs();

    for (entity, mut trigger, source, transform) in &mut query {
        trigger.timer += dt;

        if let Some(interval) = trigger.interval {
            if trigger.timer < interval {
                continue;
            }
            trigger.timer = 0.0;
        }

        let position = transform.translation.truncate();

        let target_layer = match source.caster_faction {
            Faction::Player => GameLayer::Enemy,
            Faction::Enemy => GameLayer::Player,
        };

        let filter = SpatialQueryFilter::from_mask(target_layer);
        let shape = Collider::circle(trigger.radius);
        let hits = spatial_query.shape_intersections(&shape, position, 0.0, &filter);

        for target_entity in hits {
            let ctx = AbilityContext::new(
                source.caster,
                source.caster_faction,
                Target::Point(transform.translation),
                Some(Target::Entity(target_entity)),
            );

            trigger_events.write(NodeTriggerEvent {
                ability_id: source.ability_id,
                action_node_id: source.node_id,
                trigger_type: trigger_id,
                context: ctx,
            });
        }

        if trigger.interval.is_none() {
            commands.entity(entity).remove::<OnAreaTrigger>();
        }
    }
}

register_node!(OnAreaParams);

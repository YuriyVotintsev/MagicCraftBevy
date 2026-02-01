use bevy::prelude::*;
use avian2d::prelude::*;
use crate::register_node;

use crate::abilities::node::{NodeHandler, NodeKind, NodeRegistry};
use crate::abilities::ids::{AbilityId, NodeTypeId};
use crate::abilities::{NodeParams, NoParams};
use crate::abilities::context::{AbilityContext, Target};
use crate::abilities::events::NodeTriggerEvent;
use crate::abilities::{AbilityDef, AbilitySource};
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::Faction;

pub const ON_AREA: &str = "on_area";

#[derive(Component)]
pub struct OnAreaTrigger {
    pub radius: f32,
}

impl OnAreaTrigger {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }

    pub fn if_configured(
        ability_def: &AbilityDef,
        node_id: crate::abilities::ids::NodeDefId,
        registry: &NodeRegistry,
        radius: f32,
    ) -> Option<Self> {
        let trigger_id = registry.get_id(ON_AREA)?;
        ability_def.has_trigger(node_id, trigger_id).then_some(Self { radius })
    }
}

#[derive(Default)]
pub struct OnAreaTriggerHandler;

impl NodeHandler for OnAreaTriggerHandler {
    fn name(&self) -> &'static str {
        ON_AREA
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Trigger
    }

    fn add_to_entity(
        &self,
        _commands: &mut Commands,
        _entity: Entity,
        _ability_id: AbilityId,
        _params: &NodeParams,
        _registry: &NodeRegistry,
    ) {
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Update, on_area_trigger_system.in_set(GameSet::AbilityExecution));
}

fn on_area_trigger_system(
    mut commands: Commands,
    query: Query<(Entity, &OnAreaTrigger, &AbilitySource, &Transform)>,
    mut trigger_events: MessageWriter<NodeTriggerEvent>,
    spatial_query: SpatialQuery,
    node_registry: Res<NodeRegistry>,
    mut cached_id: Local<Option<NodeTypeId>>,
) {
    let trigger_id = *cached_id.get_or_insert_with(|| {
        node_registry.get_id(ON_AREA)
            .expect("on_area trigger not registered")
    });

    for (entity, trigger, source, transform) in &query {
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

        commands.entity(entity).remove::<OnAreaTrigger>();
    }
}

register_node!(OnAreaTriggerHandler, params: NoParams, name: ON_AREA, systems: register_systems);

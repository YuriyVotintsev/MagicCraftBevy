use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use avian2d::prelude::*;
use crate::register_node;

use crate::abilities::node::{NodeHandler, NodeKind, NodeRegistry};
use crate::abilities::ids::{AbilityId, ParamId};
use crate::abilities::param::ParamValue;
use crate::abilities::context::{AbilityContext, Target};
use crate::abilities::events::NodeTriggerEvent;
use crate::abilities::{AbilityDef, AbilitySource};
use crate::physics::Wall;
use crate::schedule::GameSet;
use crate::Faction;
use std::collections::HashMap;

#[derive(Component)]
pub struct OnCollisionTrigger;

impl OnCollisionTrigger {
    pub fn if_configured(
        ability_def: &AbilityDef,
        node_id: crate::abilities::ids::NodeDefId,
        registry: &NodeRegistry,
    ) -> Option<Self> {
        let trigger_id = registry.get_id("on_collision")?;
        ability_def.has_trigger(node_id, trigger_id).then_some(Self)
    }
}

#[derive(Default)]
pub struct OnCollisionTriggerHandler;

impl NodeHandler for OnCollisionTriggerHandler {
    fn name(&self) -> &'static str {
        "on_collision"
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Trigger
    }

    fn add_to_entity(
        &self,
        _commands: &mut Commands,
        _entity: Entity,
        _ability_id: AbilityId,
        _params: &HashMap<ParamId, ParamValue>,
        _registry: &NodeRegistry,
    ) {
    }

    fn register_input_systems(&self, _app: &mut App) {
    }

    fn register_behavior_systems(&self, app: &mut App) {
        app.add_systems(Update, on_collision_trigger_system.in_set(GameSet::AbilityExecution));
    }
}

fn on_collision_trigger_system(
    mut collision_events: MessageReader<CollisionStart>,
    mut trigger_events: MessageWriter<NodeTriggerEvent>,
    hittable_query: Query<(&AbilitySource, &Faction, &Transform), With<OnCollisionTrigger>>,
    target_query: Query<&Faction, Without<OnCollisionTrigger>>,
    wall_query: Query<(), With<Wall>>,
    node_registry: Res<NodeRegistry>,
) {
    let Some(trigger_id) = node_registry.get_id("on_collision") else {
        return;
    };

    let mut processed: HashSet<(Entity, Entity)> = HashSet::default();

    for event in collision_events.read() {
        let (hittable_entity, other_entity) = if hittable_query.contains(event.collider1) {
            (event.collider1, event.collider2)
        } else if hittable_query.contains(event.collider2) {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if processed.contains(&(hittable_entity, other_entity)) {
            continue;
        }
        processed.insert((hittable_entity, other_entity));

        if wall_query.contains(other_entity) {
            continue;
        }

        if hittable_query.contains(other_entity) {
            continue;
        }

        let Ok((source, hittable_faction, transform)) = hittable_query.get(hittable_entity) else {
            continue;
        };
        let Ok(target_faction) = target_query.get(other_entity) else {
            continue;
        };

        if hittable_faction == target_faction {
            continue;
        }

        let ctx = AbilityContext::new(
            source.caster,
            source.caster_faction,
            Target::Point(transform.translation),
            Some(Target::Entity(other_entity)),
        );

        trigger_events.write(NodeTriggerEvent {
            ability_id: source.ability_id,
            action_node_id: source.node_id,
            trigger_type: trigger_id,
            context: ctx,
        });
    }
}

register_node!(OnCollisionTriggerHandler);

use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::Faction;
use crate::stats::ComputedStats;
use super::ids::AbilityId;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ContextValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Entity(Entity),
    Vec3(Vec3),
}

#[derive(Clone)]
pub struct AbilityContext {
    pub caster: Entity,
    pub caster_faction: Faction,
    pub stats_snapshot: Arc<ComputedStats>,
    pub caster_position: Vec3,
    pub ability_id: AbilityId,
    pub target_point: Option<Vec3>,
    pub target_direction: Option<Vec3>,
    pub params: HashMap<String, ContextValue>,
}

impl AbilityContext {
    pub fn new(
        caster: Entity,
        caster_faction: Faction,
        stats: &ComputedStats,
        position: Vec3,
        ability_id: AbilityId,
    ) -> Self {
        Self {
            caster,
            caster_faction,
            stats_snapshot: Arc::new(stats.clone()),
            caster_position: position,
            ability_id,
            target_point: None,
            target_direction: None,
            params: HashMap::new(),
        }
    }

    pub fn with_target_direction(mut self, direction: Vec3) -> Self {
        self.target_direction = Some(direction);
        self
    }

    pub fn with_target_point(mut self, point: Vec3) -> Self {
        self.target_point = Some(point);
        self
    }

    pub fn set_param(&mut self, key: &str, value: ContextValue) {
        self.params.insert(key.to_string(), value);
    }

    #[allow(dead_code)]
    pub fn get_param(&self, key: &str) -> Option<&ContextValue> {
        self.params.get(key)
    }

    #[allow(dead_code)]
    pub fn get_param_float(&self, key: &str) -> Option<f32> {
        match self.params.get(key) {
            Some(ContextValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_param_entity(&self, key: &str) -> Option<Entity> {
        match self.params.get(key) {
            Some(ContextValue::Entity(e)) => Some(*e),
            _ => None,
        }
    }
}

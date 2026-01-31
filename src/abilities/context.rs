use bevy::prelude::*;

use crate::Faction;

#[derive(Debug, Clone, Copy)]
pub enum Target {
    Point(Vec3),
    Direction(Vec3),
    Entity(Entity),
}

impl Target {
    pub fn as_point(&self) -> Option<Vec3> {
        match self {
            Target::Point(p) => Some(*p),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct AbilityContext {
    pub caster: Entity,
    pub caster_faction: Faction,
    pub source: Target,
    pub target: Option<Target>,
}

impl AbilityContext {
    pub fn new(caster: Entity, caster_faction: Faction, source: Target, target: Option<Target>) -> Self {
        Self {
            caster,
            caster_faction,
            source,
            target,
        }
    }
}

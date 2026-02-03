use avian2d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::spawn::SpawnContext;
use crate::physics::GameLayer;
use crate::Faction;

#[derive(Debug, Clone, Deserialize)]
pub enum ShapeRaw {
    Circle(ParamValueRaw),
}

#[derive(Debug, Clone)]
pub enum Shape {
    Circle(ParamValue),
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub shape: ShapeRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub shape: Shape,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            shape: match &self.shape {
                ShapeRaw::Circle(r) => Shape::Circle(resolve_param_value(r, stat_registry)),
            },
        }
    }
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let collider = match &def.shape {
        Shape::Circle(r) => {
            let size = r.evaluate_f32(ctx.stats);
            Collider::circle(size / 2.0)
        }
    };

    let layers = match ctx.caster_faction {
        Faction::Player => CollisionLayers::new(
            GameLayer::PlayerProjectile,
            [GameLayer::Enemy, GameLayer::Wall],
        ),
        Faction::Enemy => CollisionLayers::new(
            GameLayer::EnemyProjectile,
            [GameLayer::Player, GameLayer::Wall],
        ),
    };

    commands.insert((collider, Sensor, CollisionEventsEnabled, layers));
}

pub fn register_systems(_app: &mut App) {}

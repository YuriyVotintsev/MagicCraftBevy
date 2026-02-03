use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::spawn::SpawnContext;
use crate::abilities::Target;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub speed: ParamValueRaw,
    #[serde(default)]
    pub spread: Option<ParamValueRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub speed: ParamValue,
    pub spread: Option<ParamValue>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            speed: resolve_param_value(&self.speed, stat_registry),
            spread: self.spread.as_ref().map(|s| resolve_param_value(s, stat_registry)),
        }
    }
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let speed = def.speed.evaluate_f32(ctx.stats);
    let spread = def.spread.as_ref().map(|s| s.evaluate_f32(ctx.stats)).unwrap_or(0.0);

    let base_direction = match ctx.target {
        Some(Target::Direction(d)) => d.truncate().normalize_or_zero(),
        _ => Vec2::X,
    };

    let direction = if spread > 0.0 {
        let spread_rad = spread.to_radians();
        let angle_offset = rand::rng().random_range(-spread_rad..spread_rad);
        let cos = angle_offset.cos();
        let sin = angle_offset.sin();
        Vec2::new(
            base_direction.x * cos - base_direction.y * sin,
            base_direction.x * sin + base_direction.y * cos,
        )
    } else {
        base_direction
    };

    let velocity = direction * speed;
    commands.insert((RigidBody::Kinematic, LinearVelocity(velocity)));
}

pub fn register_systems(_app: &mut App) {}

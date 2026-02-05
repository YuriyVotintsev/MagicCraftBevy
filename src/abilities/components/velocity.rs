use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw, VecExpr, VecExprRaw};
use crate::abilities::spawn::SpawnContext;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub speed: ScalarExprRaw,
    #[serde(default)]
    pub spread: Option<ScalarExprRaw>,
    #[serde(default)]
    pub direction: Option<VecExprRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub speed: ScalarExpr,
    pub spread: Option<ScalarExpr>,
    pub direction: Option<VecExpr>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            speed: self.speed.resolve(stat_registry),
            spread: self.spread.as_ref().map(|s| s.resolve(stat_registry)),
            direction: self.direction.as_ref().map(|d| d.resolve(stat_registry)),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let mut fields = raw.speed.required_fields();
    if let Some(ref spread) = raw.spread {
        fields = fields.union(spread.required_fields());
    }
    if let Some(ref direction) = raw.direction {
        fields = fields.union(direction.required_fields());
    }
    (fields, None)
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let eval_ctx = ctx.eval_context();
    let speed = def.speed.eval(&eval_ctx);
    let spread = def.spread.as_ref().map(|s| s.eval(&eval_ctx)).unwrap_or(0.0);

    let base_direction = match &def.direction {
        Some(dir_expr) => dir_expr.eval(&eval_ctx).normalize_or_zero(),
        None => ctx.target.direction.unwrap_or(Vec2::X),
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

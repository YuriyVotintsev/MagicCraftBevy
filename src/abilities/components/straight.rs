use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw, VecExpr, VecExprRaw};
use crate::abilities::spawn::SpawnContext;
use crate::schedule::GameSet;
use crate::GameState;

use super::speed::Speed;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    #[serde(default)]
    pub spread: Option<ScalarExprRaw>,
    #[serde(default)]
    pub direction: Option<VecExprRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub spread: Option<ScalarExpr>,
    pub direction: Option<VecExpr>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            spread: self.spread.as_ref().map(|s| s.resolve(stat_registry)),
            direction: self.direction.as_ref().map(|d| d.resolve(stat_registry)),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let mut fields = ProvidedFields::NONE;
    if let Some(ref spread) = raw.spread {
        fields = fields.union(spread.required_fields());
    }
    match &raw.direction {
        Some(dir) => fields = fields.union(dir.required_fields()),
        None => fields = fields.union(ProvidedFields::TARGET_DIRECTION),
    }
    (fields, None)
}

#[derive(Component)]
pub struct Straight {
    pub direction: Vec2,
}

pub fn insert_component(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let eval_ctx = ctx.eval_context();
    let spread = def.spread.as_ref().map(|s| s.eval(&eval_ctx)).unwrap_or(0.0);

    let base_direction = match &def.direction {
        Some(dir_expr) => dir_expr.eval(&eval_ctx).normalize_or_zero(),
        None => ctx.target.direction.expect("Straight requires target.direction when direction field is not specified"),
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

    commands.insert(Straight { direction });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        init_straight
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_straight(
    mut commands: Commands,
    query: Query<(Entity, &Speed, &Straight), Added<Straight>>,
) {
    for (entity, speed, straight) in &query {
        commands.entity(entity).insert((
            RigidBody::Kinematic,
            LinearVelocity(straight.direction * speed.0),
        ));
    }
}

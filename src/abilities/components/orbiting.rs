use std::f32::consts::PI;
use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::abilities::spawn::SpawnContext;
use crate::common::AttachedTo;
use crate::schedule::GameSet;
use crate::GameState;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub radius: ScalarExprRaw,
    pub angular_speed: ScalarExprRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub radius: ScalarExpr,
    pub angular_speed: ScalarExpr,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            radius: self.radius.resolve(stat_registry),
            angular_speed: self.angular_speed.resolve(stat_registry),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let fields = raw.radius.required_fields().union(raw.angular_speed.required_fields());
    (fields, None)
}

#[derive(Component)]
pub struct Orbiting {
    pub radius: f32,
    pub angular_speed: f32,
    pub current_angle: f32,
}

pub fn insert_component(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let eval_ctx = ctx.eval_context();
    let radius = def.radius.eval(&eval_ctx);
    let angular_speed = def.angular_speed.eval(&eval_ctx);

    let initial_angle = if ctx.count > 1 {
        2.0 * PI * (ctx.index as f32) / (ctx.count as f32)
    } else {
        0.0
    };

    commands.insert(Orbiting {
        radius,
        angular_speed,
        current_angle: initial_angle,
    });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_orbiting, update_orbiting_positions)
            .chain()
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_orbiting(
    mut commands: Commands,
    query: Query<(Entity, &Orbiting, &crate::abilities::AbilitySource), Added<Orbiting>>,
    transforms: Query<&Transform>,
) {
    for (entity, orbiting, source) in &query {
        let owner_pos = transforms.get(source.caster).map(|t| t.translation).unwrap_or_default();
        let offset = Vec2::new(orbiting.current_angle.cos(), orbiting.current_angle.sin()) * orbiting.radius;
        let position = owner_pos + offset.extend(0.0);
        commands.entity(entity).insert((
            AttachedTo { owner: source.caster },
            Transform::from_translation(position),
        ));
    }
}

fn update_orbiting_positions(
    time: Res<Time>,
    owner_query: Query<&Transform, Without<Orbiting>>,
    mut orb_query: Query<(&AttachedTo, &mut Orbiting, &mut Transform)>,
) {
    for (attached, mut orbiting, mut transform) in &mut orb_query {
        orbiting.current_angle += orbiting.angular_speed * time.delta_secs();

        if let Ok(owner_transform) = owner_query.get(attached.owner) {
            let offset = Vec2::new(
                orbiting.current_angle.cos() * orbiting.radius,
                orbiting.current_angle.sin() * orbiting.radius,
            );
            transform.translation = owner_transform.translation + offset.extend(0.0);
        }
    }
}

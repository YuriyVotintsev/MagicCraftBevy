use bevy::prelude::*;
use serde::{Deserialize, Deserializer};

use crate::stats::{StatId, StatRegistry};
use super::context::ProvidedFields;
use super::eval_context::EvalContext;
use super::expr_parser::{TypedExpr, parse_expr_string};

#[derive(Debug, Clone)]
pub enum ScalarExprRaw {
    Literal(f32),
    Stat(String),
    Index,
    Count,
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Neg(Box<Self>),
    Min(Box<Self>, Box<Self>),
    Max(Box<Self>, Box<Self>),
    Length(Box<VecExprRaw>),
    Distance(Box<VecExprRaw>, Box<VecExprRaw>),
    Dot(Box<VecExprRaw>, Box<VecExprRaw>),
    X(Box<VecExprRaw>),
    Y(Box<VecExprRaw>),
    Angle(Box<VecExprRaw>),
}

#[derive(Debug, Clone)]
pub enum ScalarExpr {
    Literal(f32),
    Stat(StatId),
    Index,
    Count,
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Neg(Box<Self>),
    Min(Box<Self>, Box<Self>),
    Max(Box<Self>, Box<Self>),
    Length(Box<VecExpr>),
    Distance(Box<VecExpr>, Box<VecExpr>),
    Dot(Box<VecExpr>, Box<VecExpr>),
    X(Box<VecExpr>),
    Y(Box<VecExpr>),
    Angle(Box<VecExpr>),
}

impl Default for ScalarExpr {
    fn default() -> Self {
        Self::Literal(0.0)
    }
}

impl Default for ScalarExprRaw {
    fn default() -> Self {
        Self::Literal(0.0)
    }
}

#[derive(Debug, Clone)]
pub enum VecExprRaw {
    CasterPos,
    SourcePos,
    SourceDir,
    TargetPos,
    TargetDir,
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Scale(Box<Self>, Box<ScalarExprRaw>),
    Normalize(Box<Self>),
    Rotate(Box<Self>, Box<ScalarExprRaw>),
    Lerp(Box<Self>, Box<Self>, Box<ScalarExprRaw>),
    Vec2Expr(Box<ScalarExprRaw>, Box<ScalarExprRaw>),
    FromAngle(Box<ScalarExprRaw>),
}

#[derive(Debug, Clone)]
pub enum VecExpr {
    CasterPos,
    SourcePos,
    SourceDir,
    TargetPos,
    TargetDir,
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Scale(Box<Self>, Box<ScalarExpr>),
    Normalize(Box<Self>),
    Rotate(Box<Self>, Box<ScalarExpr>),
    Lerp(Box<Self>, Box<Self>, Box<ScalarExpr>),
    Vec2Expr(Box<ScalarExpr>, Box<ScalarExpr>),
    FromAngle(Box<ScalarExpr>),
}

#[derive(Debug, Clone)]
pub enum EntityExprRaw {
    CasterEntity,
    SourceEntity,
    TargetEntity,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum EntityExpr {
    CasterEntity,
    SourceEntity,
    TargetEntity,
}

impl ScalarExprRaw {
    pub fn resolve(&self, reg: &StatRegistry) -> ScalarExpr {
        match self {
            Self::Literal(v) => ScalarExpr::Literal(*v),
            Self::Stat(name) => {
                let id = reg
                    .get(name)
                    .unwrap_or_else(|| panic!("Unknown stat '{}'", name));
                ScalarExpr::Stat(id)
            }
            Self::Index => ScalarExpr::Index,
            Self::Count => ScalarExpr::Count,
            Self::Add(a, b) => ScalarExpr::Add(
                Box::new(a.resolve(reg)),
                Box::new(b.resolve(reg)),
            ),
            Self::Sub(a, b) => ScalarExpr::Sub(
                Box::new(a.resolve(reg)),
                Box::new(b.resolve(reg)),
            ),
            Self::Mul(a, b) => ScalarExpr::Mul(
                Box::new(a.resolve(reg)),
                Box::new(b.resolve(reg)),
            ),
            Self::Div(a, b) => ScalarExpr::Div(
                Box::new(a.resolve(reg)),
                Box::new(b.resolve(reg)),
            ),
            Self::Neg(a) => ScalarExpr::Neg(Box::new(a.resolve(reg))),
            Self::Min(a, b) => ScalarExpr::Min(
                Box::new(a.resolve(reg)),
                Box::new(b.resolve(reg)),
            ),
            Self::Max(a, b) => ScalarExpr::Max(
                Box::new(a.resolve(reg)),
                Box::new(b.resolve(reg)),
            ),
            Self::Length(v) => ScalarExpr::Length(Box::new(v.resolve(reg))),
            Self::Distance(a, b) => ScalarExpr::Distance(
                Box::new(a.resolve(reg)),
                Box::new(b.resolve(reg)),
            ),
            Self::Dot(a, b) => ScalarExpr::Dot(
                Box::new(a.resolve(reg)),
                Box::new(b.resolve(reg)),
            ),
            Self::X(v) => ScalarExpr::X(Box::new(v.resolve(reg))),
            Self::Y(v) => ScalarExpr::Y(Box::new(v.resolve(reg))),
            Self::Angle(v) => ScalarExpr::Angle(Box::new(v.resolve(reg))),
        }
    }

    pub fn required_fields(&self) -> ProvidedFields {
        match self {
            Self::Literal(_) | Self::Stat(_) | Self::Index | Self::Count => ProvidedFields::NONE,
            Self::Add(a, b)
            | Self::Sub(a, b)
            | Self::Mul(a, b)
            | Self::Div(a, b)
            | Self::Min(a, b)
            | Self::Max(a, b) => a.required_fields().union(b.required_fields()),
            Self::Neg(a) => a.required_fields(),
            Self::Length(v) | Self::X(v) | Self::Y(v) | Self::Angle(v) => v.required_fields(),
            Self::Distance(a, b) | Self::Dot(a, b) => {
                a.required_fields().union(b.required_fields())
            }
        }
    }
}

impl VecExprRaw {
    pub fn resolve(&self, reg: &StatRegistry) -> VecExpr {
        match self {
            Self::CasterPos => VecExpr::CasterPos,
            Self::SourcePos => VecExpr::SourcePos,
            Self::SourceDir => VecExpr::SourceDir,
            Self::TargetPos => VecExpr::TargetPos,
            Self::TargetDir => VecExpr::TargetDir,
            Self::Add(a, b) => VecExpr::Add(
                Box::new(a.resolve(reg)),
                Box::new(b.resolve(reg)),
            ),
            Self::Sub(a, b) => VecExpr::Sub(
                Box::new(a.resolve(reg)),
                Box::new(b.resolve(reg)),
            ),
            Self::Scale(v, s) => VecExpr::Scale(
                Box::new(v.resolve(reg)),
                Box::new(s.resolve(reg)),
            ),
            Self::Normalize(v) => VecExpr::Normalize(Box::new(v.resolve(reg))),
            Self::Rotate(v, a) => VecExpr::Rotate(
                Box::new(v.resolve(reg)),
                Box::new(a.resolve(reg)),
            ),
            Self::Lerp(a, b, t) => VecExpr::Lerp(
                Box::new(a.resolve(reg)),
                Box::new(b.resolve(reg)),
                Box::new(t.resolve(reg)),
            ),
            Self::Vec2Expr(x, y) => VecExpr::Vec2Expr(
                Box::new(x.resolve(reg)),
                Box::new(y.resolve(reg)),
            ),
            Self::FromAngle(a) => VecExpr::FromAngle(Box::new(a.resolve(reg))),
        }
    }

    pub fn required_fields(&self) -> ProvidedFields {
        match self {
            Self::CasterPos => ProvidedFields::NONE,
            Self::SourcePos => ProvidedFields::SOURCE_POSITION,
            Self::SourceDir => ProvidedFields::SOURCE_DIRECTION,
            Self::TargetPos => ProvidedFields::TARGET_POSITION,
            Self::TargetDir => ProvidedFields::TARGET_DIRECTION,
            Self::Add(a, b) | Self::Sub(a, b) => {
                a.required_fields().union(b.required_fields())
            }
            Self::Scale(v, s) => v.required_fields().union(s.required_fields()),
            Self::Normalize(v) => v.required_fields(),
            Self::Rotate(v, s) => v.required_fields().union(s.required_fields()),
            Self::Lerp(a, b, t) => a
                .required_fields()
                .union(b.required_fields())
                .union(t.required_fields()),
            Self::Vec2Expr(x, y) => x.required_fields().union(y.required_fields()),
            Self::FromAngle(a) => a.required_fields(),
        }
    }
}

#[allow(dead_code)]
impl EntityExprRaw {
    pub fn resolve(&self, _reg: &StatRegistry) -> EntityExpr {
        match self {
            Self::CasterEntity => EntityExpr::CasterEntity,
            Self::SourceEntity => EntityExpr::SourceEntity,
            Self::TargetEntity => EntityExpr::TargetEntity,
        }
    }

    pub fn required_fields(&self) -> ProvidedFields {
        match self {
            Self::CasterEntity => ProvidedFields::NONE,
            Self::SourceEntity => ProvidedFields::SOURCE_ENTITY,
            Self::TargetEntity => ProvidedFields::TARGET_ENTITY,
        }
    }
}

impl ScalarExpr {
    pub fn eval(&self, ctx: &EvalContext) -> f32 {
        match self {
            Self::Literal(v) => *v,
            Self::Stat(id) => ctx.stats.get(*id),
            Self::Index => ctx.index as f32,
            Self::Count => ctx.count as f32,
            Self::Add(a, b) => a.eval(ctx) + b.eval(ctx),
            Self::Sub(a, b) => a.eval(ctx) - b.eval(ctx),
            Self::Mul(a, b) => a.eval(ctx) * b.eval(ctx),
            Self::Div(a, b) => {
                let d = b.eval(ctx);
                if d.abs() < f32::EPSILON {
                    0.0
                } else {
                    a.eval(ctx) / d
                }
            }
            Self::Neg(a) => -a.eval(ctx),
            Self::Min(a, b) => a.eval(ctx).min(b.eval(ctx)),
            Self::Max(a, b) => a.eval(ctx).max(b.eval(ctx)),
            Self::Length(v) => v.eval(ctx).length(),
            Self::Distance(a, b) => a.eval(ctx).distance(b.eval(ctx)),
            Self::Dot(a, b) => a.eval(ctx).dot(b.eval(ctx)),
            Self::X(v) => v.eval(ctx).x,
            Self::Y(v) => v.eval(ctx).y,
            Self::Angle(v) => {
                let v = v.eval(ctx);
                v.y.atan2(v.x)
            }
        }
    }
}

impl VecExpr {
    pub fn eval(&self, ctx: &EvalContext) -> Vec2 {
        match self {
            Self::CasterPos => ctx.caster.position.unwrap_or(Vec2::ZERO),
            Self::SourcePos => ctx.source.position.unwrap_or(Vec2::ZERO),
            Self::SourceDir => ctx.source.direction.unwrap_or(Vec2::ZERO),
            Self::TargetPos => ctx.target.position.unwrap_or(Vec2::ZERO),
            Self::TargetDir => ctx.target.direction.unwrap_or(Vec2::ZERO),
            Self::Add(a, b) => a.eval(ctx) + b.eval(ctx),
            Self::Sub(a, b) => a.eval(ctx) - b.eval(ctx),
            Self::Scale(v, s) => v.eval(ctx) * s.eval(ctx),
            Self::Normalize(v) => v.eval(ctx).normalize_or_zero(),
            Self::Rotate(v, angle) => {
                let v = v.eval(ctx);
                let a = angle.eval(ctx);
                Vec2::new(
                    v.x * a.cos() - v.y * a.sin(),
                    v.x * a.sin() + v.y * a.cos(),
                )
            }
            Self::Lerp(a, b, t) => a.eval(ctx).lerp(b.eval(ctx), t.eval(ctx)),
            Self::Vec2Expr(x, y) => Vec2::new(x.eval(ctx), y.eval(ctx)),
            Self::FromAngle(a) => Vec2::from_angle(a.eval(ctx)),
        }
    }
}

#[allow(dead_code)]
impl EntityExpr {
    pub fn eval(&self, ctx: &EvalContext) -> Option<Entity> {
        match self {
            Self::CasterEntity => ctx.caster.entity,
            Self::SourceEntity => ctx.source.entity,
            Self::TargetEntity => ctx.target.entity,
        }
    }
}

impl<'de> Deserialize<'de> for ScalarExprRaw {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match parse_expr_string(&s) {
            Ok(TypedExpr::Scalar(e)) => Ok(e),
            Ok(TypedExpr::Vec2(_)) => Err(serde::de::Error::custom(format!(
                "Expected scalar expression, got vec2: '{}'",
                s
            ))),
            Ok(TypedExpr::Entity(_)) => Err(serde::de::Error::custom(format!(
                "Expected scalar expression, got entity: '{}'",
                s
            ))),
            Err(e) => Err(serde::de::Error::custom(format!(
                "Failed to parse scalar expression '{}': {}",
                s, e
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for VecExprRaw {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match parse_expr_string(&s) {
            Ok(TypedExpr::Vec2(e)) => Ok(e),
            Ok(TypedExpr::Scalar(_)) => Err(serde::de::Error::custom(format!(
                "Expected vec2 expression, got scalar: '{}'",
                s
            ))),
            Ok(TypedExpr::Entity(_)) => Err(serde::de::Error::custom(format!(
                "Expected vec2 expression, got entity: '{}'",
                s
            ))),
            Err(e) => Err(serde::de::Error::custom(format!(
                "Failed to parse vec2 expression '{}': {}",
                s, e
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for EntityExprRaw {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match parse_expr_string(&s) {
            Ok(TypedExpr::Entity(e)) => Ok(e),
            Ok(TypedExpr::Scalar(_)) => Err(serde::de::Error::custom(format!(
                "Expected entity expression, got scalar: '{}'",
                s
            ))),
            Ok(TypedExpr::Vec2(_)) => Err(serde::de::Error::custom(format!(
                "Expected entity expression, got vec2: '{}'",
                s
            ))),
            Err(e) => Err(serde::de::Error::custom(format!(
                "Failed to parse entity expression '{}': {}",
                s, e
            ))),
        }
    }
}

pub fn parse_required_fields(expr_str: &str) -> super::context::ProvidedFields {
    match parse_expr_string(expr_str) {
        Ok(TypedExpr::Scalar(e)) => e.required_fields(),
        Ok(TypedExpr::Vec2(e)) => e.required_fields(),
        Ok(TypedExpr::Entity(e)) => e.required_fields(),
        Err(e) => panic!("Failed to parse default expression '{}': {}", expr_str, e),
    }
}

pub fn parse_and_resolve_scalar(expr_str: &str, reg: &StatRegistry) -> ScalarExpr {
    match parse_expr_string(expr_str) {
        Ok(TypedExpr::Scalar(e)) => e.resolve(reg),
        Ok(_) => panic!("Expected scalar expression, got different type: '{}'", expr_str),
        Err(e) => panic!("Failed to parse scalar expression '{}': {}", expr_str, e),
    }
}

pub fn parse_and_resolve_vec(expr_str: &str, reg: &StatRegistry) -> VecExpr {
    match parse_expr_string(expr_str) {
        Ok(TypedExpr::Vec2(e)) => e.resolve(reg),
        Ok(_) => panic!("Expected vec2 expression, got different type: '{}'", expr_str),
        Err(e) => panic!("Failed to parse vec2 expression '{}': {}", expr_str, e),
    }
}

pub fn parse_and_resolve_entity(expr_str: &str, reg: &StatRegistry) -> EntityExpr {
    match parse_expr_string(expr_str) {
        Ok(TypedExpr::Entity(e)) => e.resolve(reg),
        Ok(_) => panic!("Expected entity expression, got different type: '{}'", expr_str),
        Err(e) => panic!("Failed to parse entity expression '{}': {}", expr_str, e),
    }
}


use bevy::prelude::*;
use serde::{Deserialize, Deserializer};
use std::fmt::Debug;
use std::hash::Hash;

use crate::stats::{ComputedStats, StatId, StatRegistry};
#[cfg(test)]
use super::context::ProvidedFields;
use super::core_components::AbilitySource;
use super::expr_parser::{TypedExpr, parse_expr_string};

pub trait ExprFamily: Clone + Debug {
    type StatRef: Clone + Debug + Eq + Hash;
    type ComponentDef: Clone + Debug;
    type StatesBlock: Clone + Debug;
}

#[derive(Clone, Debug)]
pub struct Raw;
impl ExprFamily for Raw {
    type StatRef = String;
    type ComponentDef = super::components::ComponentDefRaw;
    type StatesBlock = super::entity_def::StatesBlockRaw;
}

#[derive(Clone, Debug)]
pub struct Resolved;
impl ExprFamily for Resolved {
    type StatRef = StatId;
    type ComponentDef = super::components::ComponentDef;
    type StatesBlock = super::entity_def::StatesBlock;
}

pub type ScalarExprRaw = ScalarExpr<Raw>;
pub type VecExprRaw = VecExpr<Raw>;
pub type EntityExprRaw = EntityExpr;

#[derive(Debug, Clone)]
pub enum ScalarExpr<F: ExprFamily = Resolved> {
    Literal(f32),
    Stat(F::StatRef),
    Index,
    Count,
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Neg(Box<Self>),
    Min(Box<Self>, Box<Self>),
    Max(Box<Self>, Box<Self>),
    Length(Box<VecExpr<F>>),
    Distance(Box<VecExpr<F>>, Box<VecExpr<F>>),
    Dot(Box<VecExpr<F>>, Box<VecExpr<F>>),
    X(Box<VecExpr<F>>),
    Y(Box<VecExpr<F>>),
    Angle(Box<VecExpr<F>>),
    Recalc(Box<Self>),
}

impl<F: ExprFamily> Default for ScalarExpr<F> {
    fn default() -> Self {
        Self::Literal(0.0)
    }
}

#[derive(Debug, Clone)]
pub enum VecExpr<F: ExprFamily = Resolved> {
    CasterPos,
    SourcePos,
    SourceDir,
    TargetPos,
    TargetDir,
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Scale(Box<Self>, Box<ScalarExpr<F>>),
    Normalize(Box<Self>),
    Rotate(Box<Self>, Box<ScalarExpr<F>>),
    Lerp(Box<Self>, Box<Self>, Box<ScalarExpr<F>>),
    Vec2Expr(Box<ScalarExpr<F>>, Box<ScalarExpr<F>>),
    FromAngle(Box<ScalarExpr<F>>),
    Recalc(Box<Self>),
}

#[derive(Debug, Clone)]
pub enum EntityExpr {
    CasterEntity,
    SourceEntity,
    TargetEntity,
    Recalc(Box<Self>),
}

// --- Shared impls (both Raw and Resolved) ---

impl<F: ExprFamily> ScalarExpr<F> {
    pub fn uses_recalc(&self) -> bool {
        match self {
            Self::Literal(_) | Self::Index | Self::Count | Self::Stat(_) => false,
            Self::Recalc(_) => true,
            Self::Add(a, b)
            | Self::Sub(a, b)
            | Self::Mul(a, b)
            | Self::Div(a, b)
            | Self::Min(a, b)
            | Self::Max(a, b) => a.uses_recalc() || b.uses_recalc(),
            Self::Neg(a) => a.uses_recalc(),
            Self::Length(v) | Self::X(v) | Self::Y(v) | Self::Angle(v) => v.uses_recalc(),
            Self::Distance(a, b) | Self::Dot(a, b) => a.uses_recalc() || b.uses_recalc(),
        }
    }

    #[cfg(test)]
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
            Self::Recalc(e) => e.required_fields(),
        }
    }
}

impl<F: ExprFamily> VecExpr<F> {
    pub fn uses_recalc(&self) -> bool {
        match self {
            Self::CasterPos | Self::SourcePos | Self::SourceDir
            | Self::TargetPos | Self::TargetDir => false,
            Self::Recalc(_) => true,
            Self::Add(a, b) | Self::Sub(a, b) => a.uses_recalc() || b.uses_recalc(),
            Self::Scale(v, s) => v.uses_recalc() || s.uses_recalc(),
            Self::Normalize(v) => v.uses_recalc(),
            Self::Rotate(v, a) => v.uses_recalc() || a.uses_recalc(),
            Self::Lerp(a, b, t) => a.uses_recalc() || b.uses_recalc() || t.uses_recalc(),
            Self::Vec2Expr(x, y) => x.uses_recalc() || y.uses_recalc(),
            Self::FromAngle(a) => a.uses_recalc(),
        }
    }

    #[cfg(test)]
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
            Self::Recalc(e) => e.required_fields(),
        }
    }
}

impl EntityExpr {
    pub fn resolve(&self, _reg: &StatRegistry) -> EntityExpr {
        self.clone()
    }

    #[cfg(test)]
    pub fn required_fields(&self) -> ProvidedFields {
        match self {
            Self::CasterEntity => ProvidedFields::NONE,
            Self::SourceEntity => ProvidedFields::SOURCE_ENTITY,
            Self::TargetEntity => ProvidedFields::TARGET_ENTITY,
            Self::Recalc(e) => e.required_fields(),
        }
    }

    pub fn eval(&self, source: &AbilitySource) -> Option<Entity> {
        match self {
            Self::CasterEntity => source.caster.entity,
            Self::SourceEntity => source.source.entity,
            Self::TargetEntity => source.target.entity,
            Self::Recalc(e) => e.eval(source),
        }
    }
}

// --- Raw-only impls (resolve) ---

impl ScalarExpr<Raw> {
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
            Self::Recalc(e) => ScalarExpr::Recalc(Box::new(e.resolve(reg))),
        }
    }
}

impl VecExpr<Raw> {
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
            Self::Recalc(e) => VecExpr::Recalc(Box::new(e.resolve(reg))),
        }
    }
}

// --- Resolved-only impls (eval) ---

impl ScalarExpr {
    pub fn eval(&self, source: &AbilitySource, stats: &ComputedStats) -> f32 {
        match self {
            Self::Literal(v) => *v,
            Self::Stat(id) => stats.get(*id),
            Self::Index => source.index as f32,
            Self::Count => source.count as f32,
            Self::Add(a, b) => a.eval(source, stats) + b.eval(source, stats),
            Self::Sub(a, b) => a.eval(source, stats) - b.eval(source, stats),
            Self::Mul(a, b) => a.eval(source, stats) * b.eval(source, stats),
            Self::Div(a, b) => {
                let d = b.eval(source, stats);
                if d.abs() < f32::EPSILON {
                    0.0
                } else {
                    a.eval(source, stats) / d
                }
            }
            Self::Neg(a) => -a.eval(source, stats),
            Self::Min(a, b) => a.eval(source, stats).min(b.eval(source, stats)),
            Self::Max(a, b) => a.eval(source, stats).max(b.eval(source, stats)),
            Self::Length(v) => v.eval(source, stats).length(),
            Self::Distance(a, b) => a.eval(source, stats).distance(b.eval(source, stats)),
            Self::Dot(a, b) => a.eval(source, stats).dot(b.eval(source, stats)),
            Self::X(v) => v.eval(source, stats).x,
            Self::Y(v) => v.eval(source, stats).y,
            Self::Angle(v) => {
                let v = v.eval(source, stats);
                v.y.atan2(v.x)
            }
            Self::Recalc(e) => e.eval(source, stats),
        }
    }
}

impl VecExpr {
    pub fn eval(&self, source: &AbilitySource, stats: &ComputedStats) -> Vec2 {
        match self {
            Self::CasterPos => source.caster.position.unwrap_or(Vec2::ZERO),
            Self::SourcePos => source.source.position.unwrap_or(Vec2::ZERO),
            Self::SourceDir => source.source.direction.unwrap_or(Vec2::ZERO),
            Self::TargetPos => source.target.position.unwrap_or(Vec2::ZERO),
            Self::TargetDir => source.target.direction.unwrap_or(Vec2::ZERO),
            Self::Add(a, b) => a.eval(source, stats) + b.eval(source, stats),
            Self::Sub(a, b) => a.eval(source, stats) - b.eval(source, stats),
            Self::Scale(v, s) => v.eval(source, stats) * s.eval(source, stats),
            Self::Normalize(v) => v.eval(source, stats).normalize_or_zero(),
            Self::Rotate(v, angle) => {
                let v = v.eval(source, stats);
                let a = angle.eval(source, stats);
                Vec2::new(
                    v.x * a.cos() - v.y * a.sin(),
                    v.x * a.sin() + v.y * a.cos(),
                )
            }
            Self::Lerp(a, b, t) => a.eval(source, stats).lerp(b.eval(source, stats), t.eval(source, stats)),
            Self::Vec2Expr(x, y) => Vec2::new(x.eval(source, stats), y.eval(source, stats)),
            Self::FromAngle(a) => Vec2::from_angle(a.eval(source, stats)),
            Self::Recalc(e) => e.eval(source, stats),
        }
    }
}

// --- Deserialization (Raw only) ---

impl<'de> Deserialize<'de> for ScalarExpr<Raw> {
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

impl<'de> Deserialize<'de> for VecExpr<Raw> {
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

impl<'de> Deserialize<'de> for EntityExpr {
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

// --- Utility functions ---

#[cfg(test)]
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

pub fn parse_and_resolve_entity(expr_str: &str, _reg: &StatRegistry) -> EntityExpr {
    match parse_expr_string(expr_str) {
        Ok(TypedExpr::Entity(e)) => e,
        Ok(_) => panic!("Expected entity expression, got different type: '{}'", expr_str),
        Err(e) => panic!("Failed to parse entity expression '{}': {}", expr_str, e),
    }
}

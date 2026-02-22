pub mod calc;
pub mod parser;

use bevy::prelude::*;

use parser::{TypedExpr, parse_expr_string};

// --- StatId ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct StatId(pub u32);

// --- StatProvider ---

pub trait StatProvider {
    fn get_stat(&self, id: StatId) -> f32;
}

// --- EvalCtx ---

pub struct EvalCtx<'a> {
    pub stats: &'a dyn StatProvider,
    pub index: usize,
    pub count: usize,
    pub caster_pos: Vec2,
    pub source_pos: Vec2,
    pub source_dir: Vec2,
    pub target_pos: Vec2,
    pub target_dir: Vec2,
    pub caster_entity: Option<Entity>,
    pub source_entity: Option<Entity>,
    pub target_entity: Option<Entity>,
}

impl<'a> EvalCtx<'a> {
    pub fn stat_only(stats: &'a dyn StatProvider) -> Self {
        Self {
            stats,
            index: 0,
            count: 1,
            caster_pos: Vec2::ZERO,
            source_pos: Vec2::ZERO,
            source_dir: Vec2::ZERO,
            target_pos: Vec2::ZERO,
            target_dir: Vec2::ZERO,
            caster_entity: None,
            source_entity: None,
            target_entity: None,
        }
    }
}

// --- Raw expression types (Stat uses String, for deserialization) ---

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
    Clamp(Box<Self>, Box<Self>, Box<Self>),
    Length(Box<VecExprRaw>),
    Distance(Box<VecExprRaw>, Box<VecExprRaw>),
    Dot(Box<VecExprRaw>, Box<VecExprRaw>),
    X(Box<VecExprRaw>),
    Y(Box<VecExprRaw>),
    Angle(Box<VecExprRaw>),
    Recalc(Box<Self>),
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
    Recalc(Box<Self>),
}

pub type EntityExprRaw = EntityExpr;

// --- Resolved expression types (Stat uses StatId) ---

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
    Clamp(Box<Self>, Box<Self>, Box<Self>),
    Length(Box<VecExpr>),
    Distance(Box<VecExpr>, Box<VecExpr>),
    Dot(Box<VecExpr>, Box<VecExpr>),
    X(Box<VecExpr>),
    Y(Box<VecExpr>),
    Angle(Box<VecExpr>),
    Recalc(Box<Self>),
}

impl Default for ScalarExpr {
    fn default() -> Self {
        Self::Literal(0.0)
    }
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
    Recalc(Box<Self>),
}

#[derive(Debug, Clone)]
pub enum EntityExpr {
    CasterEntity,
    SourceEntity,
    TargetEntity,
    Recalc(Box<Self>),
}

// --- ScalarExprRaw impls ---

impl ScalarExprRaw {
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
            Self::Clamp(v, lo, hi) => v.uses_recalc() || lo.uses_recalc() || hi.uses_recalc(),
            Self::Neg(a) => a.uses_recalc(),
            Self::Length(v) | Self::X(v) | Self::Y(v) | Self::Angle(v) => v.uses_recalc(),
            Self::Distance(a, b) | Self::Dot(a, b) => a.uses_recalc() || b.uses_recalc(),
        }
    }

    pub fn resolve(&self, lookup: &dyn Fn(&str) -> Option<StatId>) -> ScalarExpr {
        match self {
            Self::Literal(v) => ScalarExpr::Literal(*v),
            Self::Stat(name) => {
                let id = lookup(name)
                    .unwrap_or_else(|| panic!("Unknown stat '{}'", name));
                ScalarExpr::Stat(id)
            }
            Self::Index => ScalarExpr::Index,
            Self::Count => ScalarExpr::Count,
            Self::Add(a, b) => ScalarExpr::Add(Box::new(a.resolve(lookup)), Box::new(b.resolve(lookup))),
            Self::Sub(a, b) => ScalarExpr::Sub(Box::new(a.resolve(lookup)), Box::new(b.resolve(lookup))),
            Self::Mul(a, b) => ScalarExpr::Mul(Box::new(a.resolve(lookup)), Box::new(b.resolve(lookup))),
            Self::Div(a, b) => ScalarExpr::Div(Box::new(a.resolve(lookup)), Box::new(b.resolve(lookup))),
            Self::Neg(a) => ScalarExpr::Neg(Box::new(a.resolve(lookup))),
            Self::Min(a, b) => ScalarExpr::Min(Box::new(a.resolve(lookup)), Box::new(b.resolve(lookup))),
            Self::Max(a, b) => ScalarExpr::Max(Box::new(a.resolve(lookup)), Box::new(b.resolve(lookup))),
            Self::Clamp(v, lo, hi) => ScalarExpr::Clamp(
                Box::new(v.resolve(lookup)),
                Box::new(lo.resolve(lookup)),
                Box::new(hi.resolve(lookup)),
            ),
            Self::Length(v) => ScalarExpr::Length(Box::new(v.resolve(lookup))),
            Self::Distance(a, b) => ScalarExpr::Distance(Box::new(a.resolve(lookup)), Box::new(b.resolve(lookup))),
            Self::Dot(a, b) => ScalarExpr::Dot(Box::new(a.resolve(lookup)), Box::new(b.resolve(lookup))),
            Self::X(v) => ScalarExpr::X(Box::new(v.resolve(lookup))),
            Self::Y(v) => ScalarExpr::Y(Box::new(v.resolve(lookup))),
            Self::Angle(v) => ScalarExpr::Angle(Box::new(v.resolve(lookup))),
            Self::Recalc(e) => ScalarExpr::Recalc(Box::new(e.resolve(lookup))),
        }
    }
}

// --- VecExprRaw impls ---

impl VecExprRaw {
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

    pub fn resolve(&self, lookup: &dyn Fn(&str) -> Option<StatId>) -> VecExpr {
        match self {
            Self::CasterPos => VecExpr::CasterPos,
            Self::SourcePos => VecExpr::SourcePos,
            Self::SourceDir => VecExpr::SourceDir,
            Self::TargetPos => VecExpr::TargetPos,
            Self::TargetDir => VecExpr::TargetDir,
            Self::Add(a, b) => VecExpr::Add(Box::new(a.resolve(lookup)), Box::new(b.resolve(lookup))),
            Self::Sub(a, b) => VecExpr::Sub(Box::new(a.resolve(lookup)), Box::new(b.resolve(lookup))),
            Self::Scale(v, s) => VecExpr::Scale(Box::new(v.resolve(lookup)), Box::new(s.resolve(lookup))),
            Self::Normalize(v) => VecExpr::Normalize(Box::new(v.resolve(lookup))),
            Self::Rotate(v, a) => VecExpr::Rotate(Box::new(v.resolve(lookup)), Box::new(a.resolve(lookup))),
            Self::Lerp(a, b, t) => VecExpr::Lerp(
                Box::new(a.resolve(lookup)),
                Box::new(b.resolve(lookup)),
                Box::new(t.resolve(lookup)),
            ),
            Self::Vec2Expr(x, y) => VecExpr::Vec2Expr(Box::new(x.resolve(lookup)), Box::new(y.resolve(lookup))),
            Self::FromAngle(a) => VecExpr::FromAngle(Box::new(a.resolve(lookup))),
            Self::Recalc(e) => VecExpr::Recalc(Box::new(e.resolve(lookup))),
        }
    }
}

// --- ScalarExpr (resolved) impls ---

impl ScalarExpr {
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
            Self::Clamp(v, lo, hi) => v.uses_recalc() || lo.uses_recalc() || hi.uses_recalc(),
            Self::Neg(a) => a.uses_recalc(),
            Self::Length(v) | Self::X(v) | Self::Y(v) | Self::Angle(v) => v.uses_recalc(),
            Self::Distance(a, b) | Self::Dot(a, b) => a.uses_recalc() || b.uses_recalc(),
        }
    }

    pub fn eval(&self, ctx: &EvalCtx) -> f32 {
        match self {
            Self::Literal(v) => *v,
            Self::Stat(id) => ctx.stats.get_stat(*id),
            Self::Index => ctx.index as f32,
            Self::Count => ctx.count as f32,
            Self::Add(a, b) => a.eval(ctx) + b.eval(ctx),
            Self::Sub(a, b) => a.eval(ctx) - b.eval(ctx),
            Self::Mul(a, b) => a.eval(ctx) * b.eval(ctx),
            Self::Div(a, b) => {
                let d = b.eval(ctx);
                if d.abs() < f32::EPSILON { 0.0 } else { a.eval(ctx) / d }
            }
            Self::Neg(a) => -a.eval(ctx),
            Self::Min(a, b) => a.eval(ctx).min(b.eval(ctx)),
            Self::Max(a, b) => a.eval(ctx).max(b.eval(ctx)),
            Self::Clamp(v, lo, hi) => v.eval(ctx).clamp(lo.eval(ctx), hi.eval(ctx)),
            Self::Length(v) => v.eval(ctx).length(),
            Self::Distance(a, b) => a.eval(ctx).distance(b.eval(ctx)),
            Self::Dot(a, b) => a.eval(ctx).dot(b.eval(ctx)),
            Self::X(v) => v.eval(ctx).x,
            Self::Y(v) => v.eval(ctx).y,
            Self::Angle(v) => {
                let v = v.eval(ctx);
                v.y.atan2(v.x)
            }
            Self::Recalc(e) => e.eval(ctx),
        }
    }

    pub fn uses_stats(&self) -> bool {
        match self {
            Self::Stat(_) => true,
            Self::Literal(_) | Self::Index | Self::Count => false,
            Self::Add(a, b) | Self::Sub(a, b) | Self::Mul(a, b) | Self::Div(a, b)
            | Self::Min(a, b) | Self::Max(a, b) => a.uses_stats() || b.uses_stats(),
            Self::Clamp(v, lo, hi) => v.uses_stats() || lo.uses_stats() || hi.uses_stats(),
            Self::Neg(a) => a.uses_stats(),
            Self::Length(v) | Self::X(v) | Self::Y(v) | Self::Angle(v) => v.uses_stats(),
            Self::Distance(a, b) | Self::Dot(a, b) => a.uses_stats() || b.uses_stats(),
            Self::Recalc(e) => e.uses_stats(),
        }
    }

    pub fn collect_stat_deps(&self, deps: &mut Vec<StatId>) {
        match self {
            Self::Stat(id) => deps.push(*id),
            Self::Literal(_) | Self::Index | Self::Count => {}
            Self::Add(a, b) | Self::Sub(a, b) | Self::Mul(a, b) | Self::Div(a, b)
            | Self::Min(a, b) | Self::Max(a, b) => {
                a.collect_stat_deps(deps);
                b.collect_stat_deps(deps);
            }
            Self::Clamp(v, lo, hi) => {
                v.collect_stat_deps(deps);
                lo.collect_stat_deps(deps);
                hi.collect_stat_deps(deps);
            }
            Self::Neg(a) | Self::Recalc(a) => a.collect_stat_deps(deps),
            Self::Length(v) | Self::X(v) | Self::Y(v) | Self::Angle(v) => v.collect_stat_deps(deps),
            Self::Distance(a, b) | Self::Dot(a, b) => {
                a.collect_stat_deps(deps);
                b.collect_stat_deps(deps);
            }
        }
    }
}

// --- VecExpr (resolved) impls ---

impl VecExpr {
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

    pub fn eval(&self, ctx: &EvalCtx) -> Vec2 {
        match self {
            Self::CasterPos => ctx.caster_pos,
            Self::SourcePos => ctx.source_pos,
            Self::SourceDir => ctx.source_dir,
            Self::TargetPos => ctx.target_pos,
            Self::TargetDir => ctx.target_dir,
            Self::Add(a, b) => a.eval(ctx) + b.eval(ctx),
            Self::Sub(a, b) => a.eval(ctx) - b.eval(ctx),
            Self::Scale(v, s) => v.eval(ctx) * s.eval(ctx),
            Self::Normalize(v) => v.eval(ctx).normalize_or_zero(),
            Self::Rotate(v, angle) => {
                let v = v.eval(ctx);
                let a = angle.eval(ctx);
                Vec2::new(v.x * a.cos() - v.y * a.sin(), v.x * a.sin() + v.y * a.cos())
            }
            Self::Lerp(a, b, t) => a.eval(ctx).lerp(b.eval(ctx), t.eval(ctx)),
            Self::Vec2Expr(x, y) => Vec2::new(x.eval(ctx), y.eval(ctx)),
            Self::FromAngle(a) => Vec2::from_angle(a.eval(ctx)),
            Self::Recalc(e) => e.eval(ctx),
        }
    }

    pub fn uses_stats(&self) -> bool {
        match self {
            Self::CasterPos | Self::SourcePos | Self::SourceDir
            | Self::TargetPos | Self::TargetDir => false,
            Self::Add(a, b) | Self::Sub(a, b) => a.uses_stats() || b.uses_stats(),
            Self::Scale(v, s) => v.uses_stats() || s.uses_stats(),
            Self::Normalize(v) => v.uses_stats(),
            Self::Rotate(v, a) => v.uses_stats() || a.uses_stats(),
            Self::Lerp(a, b, t) => a.uses_stats() || b.uses_stats() || t.uses_stats(),
            Self::Vec2Expr(x, y) => x.uses_stats() || y.uses_stats(),
            Self::FromAngle(a) => a.uses_stats(),
            Self::Recalc(e) => e.uses_stats(),
        }
    }

    pub fn collect_stat_deps(&self, deps: &mut Vec<StatId>) {
        match self {
            Self::CasterPos | Self::SourcePos | Self::SourceDir
            | Self::TargetPos | Self::TargetDir => {}
            Self::Add(a, b) | Self::Sub(a, b) => {
                a.collect_stat_deps(deps);
                b.collect_stat_deps(deps);
            }
            Self::Scale(v, s) => { v.collect_stat_deps(deps); s.collect_stat_deps(deps); }
            Self::Normalize(v) => v.collect_stat_deps(deps),
            Self::Rotate(v, a) => { v.collect_stat_deps(deps); a.collect_stat_deps(deps); }
            Self::Lerp(a, b, t) => {
                a.collect_stat_deps(deps);
                b.collect_stat_deps(deps);
                t.collect_stat_deps(deps);
            }
            Self::Vec2Expr(x, y) => { x.collect_stat_deps(deps); y.collect_stat_deps(deps); }
            Self::FromAngle(a) => a.collect_stat_deps(deps),
            Self::Recalc(e) => e.collect_stat_deps(deps),
        }
    }
}

// --- EntityExpr impls ---

impl EntityExpr {
    pub fn resolve(&self, _lookup: &dyn Fn(&str) -> Option<StatId>) -> EntityExpr {
        self.clone()
    }

    pub fn eval(&self, ctx: &EvalCtx) -> Option<Entity> {
        match self {
            Self::CasterEntity => ctx.caster_entity,
            Self::SourceEntity => ctx.source_entity,
            Self::TargetEntity => ctx.target_entity,
            Self::Recalc(e) => e.eval(ctx),
        }
    }
}

// --- Utility functions ---

pub fn expand_parse_resolve_scalar(expr_str: &str, lookup: &dyn Fn(&str) -> Option<StatId>, calc_reg: &calc::CalcRegistry) -> ScalarExpr {
    let expanded = calc_reg.expand(expr_str)
        .unwrap_or_else(|e| panic!("Failed to expand calc in '{}': {}", expr_str, e));
    parse_and_resolve_scalar(&expanded, lookup)
}

pub fn expand_parse_resolve_vec(expr_str: &str, lookup: &dyn Fn(&str) -> Option<StatId>, calc_reg: &calc::CalcRegistry) -> VecExpr {
    let expanded = calc_reg.expand(expr_str)
        .unwrap_or_else(|e| panic!("Failed to expand calc in '{}': {}", expr_str, e));
    parse_and_resolve_vec(&expanded, lookup)
}

pub fn expand_parse_resolve_entity(expr_str: &str, lookup: &dyn Fn(&str) -> Option<StatId>, calc_reg: &calc::CalcRegistry) -> EntityExpr {
    let expanded = calc_reg.expand(expr_str)
        .unwrap_or_else(|e| panic!("Failed to expand calc in '{}': {}", expr_str, e));
    parse_and_resolve_entity(&expanded, lookup)
}

pub fn parse_and_resolve_scalar(expr_str: &str, lookup: &dyn Fn(&str) -> Option<StatId>) -> ScalarExpr {
    match parse_expr_string(expr_str) {
        Ok(TypedExpr::Scalar(e)) => e.resolve(lookup),
        Ok(_) => panic!("Expected scalar expression, got different type: '{}'", expr_str),
        Err(e) => panic!("Failed to parse scalar expression '{}': {}", expr_str, e),
    }
}

pub fn parse_and_resolve_vec(expr_str: &str, lookup: &dyn Fn(&str) -> Option<StatId>) -> VecExpr {
    match parse_expr_string(expr_str) {
        Ok(TypedExpr::Vec2(e)) => e.resolve(lookup),
        Ok(_) => panic!("Expected vec2 expression, got different type: '{}'", expr_str),
        Err(e) => panic!("Failed to parse vec2 expression '{}': {}", expr_str, e),
    }
}

pub fn parse_and_resolve_entity(expr_str: &str, _lookup: &dyn Fn(&str) -> Option<StatId>) -> EntityExpr {
    match parse_expr_string(expr_str) {
        Ok(TypedExpr::Entity(e)) => e,
        Ok(_) => panic!("Expected entity expression, got different type: '{}'", expr_str),
        Err(e) => panic!("Failed to parse entity expression '{}': {}", expr_str, e),
    }
}

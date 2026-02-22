pub use crate::expr::*;

use bevy::prelude::*;

use super::core_components::SpawnSource;

impl<'a> EvalCtx<'a> {
    pub fn from_source(source: &SpawnSource, stats: &'a dyn StatProvider) -> Self {
        Self {
            stats,
            index: source.index,
            count: source.count,
            caster_pos: source.caster.position.unwrap_or(Vec2::ZERO),
            source_pos: source.source.position.unwrap_or(Vec2::ZERO),
            source_dir: source.source.direction.unwrap_or(Vec2::ZERO),
            target_pos: source.target.position.unwrap_or(Vec2::ZERO),
            target_dir: source.target.direction.unwrap_or(Vec2::ZERO),
            caster_entity: source.caster.entity,
            source_entity: source.source.entity,
            target_entity: source.target.entity,
        }
    }
}

#[cfg(test)]
use super::context::ProvidedFields;

#[cfg(test)]
impl ScalarExprRaw {
    pub fn required_fields(&self) -> ProvidedFields {
        match self {
            Self::Literal(_) | Self::Stat(_) | Self::Index | Self::Count => ProvidedFields::NONE,
            Self::Add(a, b)
            | Self::Sub(a, b)
            | Self::Mul(a, b)
            | Self::Div(a, b)
            | Self::Min(a, b)
            | Self::Max(a, b) => a.required_fields().union(b.required_fields()),
            Self::Clamp(v, lo, hi) => v.required_fields().union(lo.required_fields()).union(hi.required_fields()),
            Self::Neg(a) => a.required_fields(),
            Self::Length(v) | Self::X(v) | Self::Y(v) | Self::Angle(v) => v.required_fields(),
            Self::Distance(a, b) | Self::Dot(a, b) => a.required_fields().union(b.required_fields()),
            Self::Recalc(e) => e.required_fields(),
        }
    }
}

#[cfg(test)]
impl VecExprRaw {
    pub fn required_fields(&self) -> ProvidedFields {
        match self {
            Self::CasterPos => ProvidedFields::NONE,
            Self::SourcePos => ProvidedFields::SOURCE_POSITION,
            Self::SourceDir => ProvidedFields::SOURCE_DIRECTION,
            Self::TargetPos => ProvidedFields::TARGET_POSITION,
            Self::TargetDir => ProvidedFields::TARGET_DIRECTION,
            Self::Add(a, b) | Self::Sub(a, b) => a.required_fields().union(b.required_fields()),
            Self::Scale(v, s) => v.required_fields().union(s.required_fields()),
            Self::Normalize(v) => v.required_fields(),
            Self::Rotate(v, s) => v.required_fields().union(s.required_fields()),
            Self::Lerp(a, b, t) => a.required_fields().union(b.required_fields()).union(t.required_fields()),
            Self::Vec2Expr(x, y) => x.required_fields().union(y.required_fields()),
            Self::FromAngle(a) => a.required_fields(),
            Self::Recalc(e) => e.required_fields(),
        }
    }
}

#[cfg(test)]
impl EntityExpr {
    pub fn required_fields(&self) -> ProvidedFields {
        match self {
            Self::CasterEntity => ProvidedFields::NONE,
            Self::SourceEntity => ProvidedFields::SOURCE_ENTITY,
            Self::TargetEntity => ProvidedFields::TARGET_ENTITY,
            Self::Recalc(e) => e.required_fields(),
        }
    }
}

#[cfg(test)]
pub fn parse_required_fields(expr_str: &str) -> ProvidedFields {
    use crate::expr::parser::{TypedExpr, parse_expr_string};
    use crate::expr::calc::CalcRegistry;

    let expanded = if expr_str.contains("calc(") {
        use std::sync::OnceLock;
        static CALC_REG: OnceLock<CalcRegistry> = OnceLock::new();
        let reg = CALC_REG.get_or_init(|| {
            #[derive(serde::Deserialize)]
            struct Cfg {
                #[serde(default)]
                #[allow(dead_code)]
                stat_ids: Vec<crate::stats::loader::StatDefRaw>,
                #[serde(default)]
                calcs: Vec<crate::expr::calc::CalcTemplateRaw>,
            }
            let content = std::fs::read_to_string("assets/stats/config.stats.ron").unwrap();
            let cfg: Cfg = ron::from_str(&content).unwrap();
            CalcRegistry::from_raw(&cfg.calcs)
        });
        reg.expand(expr_str).unwrap_or_else(|e| panic!("Failed to expand calc in '{}': {}", expr_str, e))
    } else {
        expr_str.to_string()
    };

    match parse_expr_string(&expanded) {
        Ok(TypedExpr::Scalar(e)) => e.required_fields(),
        Ok(TypedExpr::Vec2(e)) => e.required_fields(),
        Ok(TypedExpr::Entity(e)) => e.required_fields(),
        Err(e) => panic!("Failed to parse default expression '{}': {}", expr_str, e),
    }
}

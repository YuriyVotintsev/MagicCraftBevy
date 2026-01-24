use crate::stats::{ComputedStats, StatId};

#[derive(Debug, Clone)]
pub enum StatExpression {
    Constant(f32),
    Stat(StatId),
    Add(Box<StatExpression>, Box<StatExpression>),
    Sub(Box<StatExpression>, Box<StatExpression>),
    Mul(Box<StatExpression>, Box<StatExpression>),
    Div(Box<StatExpression>, Box<StatExpression>),
    Min(Box<StatExpression>, Box<StatExpression>),
    Max(Box<StatExpression>, Box<StatExpression>),
    Clamp {
        value: Box<StatExpression>,
        min: f32,
        max: f32,
    },
    PercentOf {
        stat: StatId,
        percent: f32,
    },
}

impl StatExpression {
    pub fn evaluate(&self, stats: &ComputedStats) -> f32 {
        match self {
            Self::Constant(v) => *v,
            Self::Stat(id) => stats.get(*id),
            Self::Add(a, b) => a.evaluate(stats) + b.evaluate(stats),
            Self::Sub(a, b) => a.evaluate(stats) - b.evaluate(stats),
            Self::Mul(a, b) => a.evaluate(stats) * b.evaluate(stats),
            Self::Div(a, b) => {
                let divisor = b.evaluate(stats);
                if divisor.abs() < f32::EPSILON {
                    0.0
                } else {
                    a.evaluate(stats) / divisor
                }
            }
            Self::Min(a, b) => a.evaluate(stats).min(b.evaluate(stats)),
            Self::Max(a, b) => a.evaluate(stats).max(b.evaluate(stats)),
            Self::Clamp { value, min, max } => value.evaluate(stats).clamp(*min, *max),
            Self::PercentOf { stat, percent } => stats.get(*stat) * percent,
        }
    }

    pub fn constant(value: f32) -> Self {
        Self::Constant(value)
    }

    pub fn stat(id: StatId) -> Self {
        Self::Stat(id)
    }

    pub fn add(a: Self, b: Self) -> Self {
        Self::Add(Box::new(a), Box::new(b))
    }

    pub fn mul(a: Self, b: Self) -> Self {
        Self::Mul(Box::new(a), Box::new(b))
    }
}

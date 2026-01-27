use serde::{Deserialize, Serialize};

use super::{ComputedStats, Modifiers, StatId, StatRegistry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpressionRaw {
    Constant(f32),
    Stat(String),
    ModifierSum(String),
    ModifierProduct(String),

    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Min(Box<Self>, Box<Self>),
    Max(Box<Self>, Box<Self>),
    Clamp {
        value: Box<Self>,
        min: f32,
        max: f32,
    },
}

impl ExpressionRaw {
    pub fn resolve(&self, registry: &StatRegistry) -> Expression {
        match self {
            Self::Constant(v) => Expression::Constant(*v),
            Self::Stat(name) => {
                let id = registry.get(name).expect(&format!("Unknown stat: {}", name));
                Expression::Stat(id)
            }
            Self::ModifierSum(name) => {
                let id = registry.get(name).expect(&format!("Unknown stat: {}", name));
                Expression::ModifierSum(id)
            }
            Self::ModifierProduct(name) => {
                let id = registry.get(name).expect(&format!("Unknown stat: {}", name));
                Expression::ModifierProduct(id)
            }
            Self::Add(a, b) => {
                Expression::Add(Box::new(a.resolve(registry)), Box::new(b.resolve(registry)))
            }
            Self::Sub(a, b) => {
                Expression::Sub(Box::new(a.resolve(registry)), Box::new(b.resolve(registry)))
            }
            Self::Mul(a, b) => {
                Expression::Mul(Box::new(a.resolve(registry)), Box::new(b.resolve(registry)))
            }
            Self::Div(a, b) => {
                Expression::Div(Box::new(a.resolve(registry)), Box::new(b.resolve(registry)))
            }
            Self::Min(a, b) => {
                Expression::Min(Box::new(a.resolve(registry)), Box::new(b.resolve(registry)))
            }
            Self::Max(a, b) => {
                Expression::Max(Box::new(a.resolve(registry)), Box::new(b.resolve(registry)))
            }
            Self::Clamp { value, min, max } => Expression::Clamp {
                value: Box::new(value.resolve(registry)),
                min: *min,
                max: *max,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Constant(f32),
    Stat(StatId),
    ModifierSum(StatId),
    ModifierProduct(StatId),

    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Min(Box<Self>, Box<Self>),
    Max(Box<Self>, Box<Self>),
    Clamp { value: Box<Self>, min: f32, max: f32 },
    #[allow(dead_code)]
    PercentOf { stat: StatId, percent: f32 },
}

impl Expression {
    pub fn evaluate(&self, modifiers: &Modifiers, computed: &ComputedStats) -> f32 {
        match self {
            Self::Constant(v) => *v,
            Self::Stat(id) => computed.get(*id),
            Self::ModifierSum(id) => modifiers.sum(*id),
            Self::ModifierProduct(id) => modifiers.product(*id),
            Self::Add(a, b) => a.evaluate(modifiers, computed) + b.evaluate(modifiers, computed),
            Self::Sub(a, b) => a.evaluate(modifiers, computed) - b.evaluate(modifiers, computed),
            Self::Mul(a, b) => a.evaluate(modifiers, computed) * b.evaluate(modifiers, computed),
            Self::Div(a, b) => {
                let divisor = b.evaluate(modifiers, computed);
                if divisor.abs() < f32::EPSILON {
                    0.0
                } else {
                    a.evaluate(modifiers, computed) / divisor
                }
            }
            Self::Min(a, b) => {
                a.evaluate(modifiers, computed)
                    .min(b.evaluate(modifiers, computed))
            }
            Self::Max(a, b) => {
                a.evaluate(modifiers, computed)
                    .max(b.evaluate(modifiers, computed))
            }
            Self::Clamp { value, min, max } => value.evaluate(modifiers, computed).clamp(*min, *max),
            Self::PercentOf { stat, percent } => computed.get(*stat) * percent,
        }
    }

    pub fn evaluate_computed(&self, computed: &ComputedStats) -> f32 {
        let empty_modifiers = Modifiers::new();
        self.evaluate(&empty_modifiers, computed)
    }
}

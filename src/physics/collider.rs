use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub enum ColliderShape {
    #[default]
    Circle,
    Rectangle,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColliderDef {
    pub shape: ColliderShape,
    pub size: f32,
}

impl Default for ColliderDef {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Circle,
            size: 30.0,
        }
    }
}

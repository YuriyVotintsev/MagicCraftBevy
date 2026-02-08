use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

#[blueprint_component]
pub struct Health {
    #[default_expr("stat(max_life)")]
    pub current: ScalarExpr,
}

pub fn register_systems(_app: &mut App) {
}

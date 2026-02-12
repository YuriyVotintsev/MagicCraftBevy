use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

#[blueprint_component]
pub struct Fan {
    pub angle: ScalarExpr,
}

pub fn register_systems(_app: &mut App) {}

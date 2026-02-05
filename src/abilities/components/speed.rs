use bevy::prelude::*;
use magic_craft_macros::ability_component;


#[ability_component]
pub struct Speed {
    pub value: ScalarExpr,
}

pub fn register_systems(_app: &mut App) {}

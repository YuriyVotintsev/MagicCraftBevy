use bevy::prelude::*;
use magic_craft_macros::ability_component;


#[ability_component]
pub struct Pierce {
    pub count: Option<ScalarExpr>,
}

pub fn register_systems(_app: &mut App) {}

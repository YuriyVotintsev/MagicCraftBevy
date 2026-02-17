use bevy::prelude::*;
use serde::Deserialize;

use crate::blueprints::BlueprintId;
use crate::stats::{ModifierDef, ModifierDefRaw};

#[derive(Deserialize)]
pub struct HeroClassRaw {
    pub id: String,
    pub display_name: String,
    pub color: (f32, f32, f32, f32),
    #[serde(default)]
    pub sprite: Option<String>,
    pub modifiers: Vec<ModifierDefRaw>,
}

pub struct HeroClass {
    pub display_name: String,
    pub color: (f32, f32, f32, f32),
    pub sprite: Option<String>,
    pub modifiers: Vec<ModifierDef>,
}

#[derive(Resource)]
pub struct AvailableHeroes {
    pub base_blueprint: BlueprintId,
    pub classes: Vec<HeroClass>,
}

#[derive(Resource, Default)]
pub struct SelectedHero(pub Option<usize>);

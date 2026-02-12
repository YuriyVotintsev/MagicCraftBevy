use std::collections::HashMap;
use bevy::prelude::*;
use serde::Deserialize;

use crate::blueprints::BlueprintId;
use crate::stats::StatId;

#[derive(Deserialize)]
pub struct HeroClassRaw {
    pub id: String,
    pub display_name: String,
    pub color: (f32, f32, f32, f32),
    pub modifiers: HashMap<String, f32>,
}

pub struct HeroClassModifier {
    pub stat: StatId,
    pub value: f32,
    pub name: String,
}

pub struct HeroClass {
    pub display_name: String,
    pub color: (f32, f32, f32, f32),
    pub modifiers: Vec<HeroClassModifier>,
}

#[derive(Resource)]
pub struct AvailableHeroes {
    pub base_blueprint: BlueprintId,
    pub classes: Vec<HeroClass>,
}

#[derive(Resource, Default)]
pub struct SelectedHero(pub Option<usize>);

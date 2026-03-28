use bevy::prelude::*;

use crate::blueprints::BlueprintId;

#[derive(Resource)]
pub struct AvailableHeroes {
    pub base_blueprint: BlueprintId,
}

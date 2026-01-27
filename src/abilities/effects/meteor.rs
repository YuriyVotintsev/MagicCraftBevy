use bevy::prelude::*;

use crate::abilities::registry::{EffectExecutor, EffectRegistry};
use crate::abilities::effect_def::{EffectDef, ParamValue};
use crate::abilities::context::AbilityContext;

#[derive(Component)]
pub struct MeteorRequest {
    pub search_radius: f32,
    pub damage_radius: f32,
    pub fall_duration: f32,
    pub on_hit_effects: Vec<EffectDef>,
    pub context: AbilityContext,
}

#[derive(Component)]
pub struct MeteorFalling {
    pub target_position: Vec3,
    pub damage_radius: f32,
    pub fall_duration: f32,
    pub elapsed: f32,
    pub on_hit_effects: Vec<EffectDef>,
    pub context: AbilityContext,
}

#[derive(Component)]
pub struct MeteorIndicator;

#[derive(Component)]
pub struct MeteorExplosion {
    pub damage_radius: f32,
    pub on_hit_effects: Vec<EffectDef>,
    pub context: AbilityContext,
    pub damaged: bool,
}

pub struct SpawnMeteorEffect;

impl EffectExecutor for SpawnMeteorEffect {
    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        commands: &mut Commands,
        registry: &EffectRegistry,
    ) {
        let search_radius = match def.get_param("search_radius", registry) {
            Some(ParamValue::Float(v)) => *v,
            _ => 500.0,
        };

        let damage_radius = match def.get_param("damage_radius", registry) {
            Some(ParamValue::Float(v)) => *v,
            _ => 80.0,
        };

        let fall_duration = match def.get_param("fall_duration", registry) {
            Some(ParamValue::Float(v)) => *v,
            _ => 0.5,
        };

        let on_hit_effects = match def.get_param("on_hit", registry) {
            Some(ParamValue::EffectList(effects)) => effects.clone(),
            _ => Vec::new(),
        };

        commands.spawn((
            Name::new("MeteorRequest"),
            MeteorRequest {
                search_radius,
                damage_radius,
                fall_duration,
                on_hit_effects,
                context: ctx.clone(),
            },
        ));
    }
}

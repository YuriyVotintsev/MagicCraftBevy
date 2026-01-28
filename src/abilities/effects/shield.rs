use bevy::prelude::*;

use crate::abilities::context::AbilityContext;
use crate::abilities::effect_def::{EffectDef, ParamValue};
use crate::abilities::registry::{EffectExecutor, EffectRegistry};
use crate::wave::add_invulnerability;
use crate::Faction;

const DEFAULT_SHIELD_DURATION: f32 = 0.5;
const DEFAULT_SHIELD_RADIUS: f32 = 100.0;

#[derive(Component)]
pub struct ShieldActive {
    pub timer: Timer,
    pub radius: f32,
    pub owner_faction: Faction,
}

#[derive(Component)]
pub struct ShieldVisual {
    pub owner: Entity,
}

pub struct ShieldEffect;

impl EffectExecutor for ShieldEffect {
    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        commands: &mut Commands,
        registry: &EffectRegistry,
    ) {
        let duration = match def.get_param("duration", registry) {
            Some(ParamValue::Float(v)) => *v,
            _ => DEFAULT_SHIELD_DURATION,
        };

        let radius = match def.get_param("radius", registry) {
            Some(ParamValue::Float(v)) => *v,
            _ => DEFAULT_SHIELD_RADIUS,
        };

        let caster = ctx.caster;
        let caster_position = ctx.caster_position;

        commands.entity(caster).insert(ShieldActive {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            radius,
            owner_faction: ctx.caster_faction,
        });

        add_invulnerability(commands, caster);

        commands.spawn((
            Name::new("ShieldVisual"),
            ShieldVisual { owner: caster },
            Sprite {
                color: Color::srgba(0.3, 0.7, 1.0, 0.4),
                custom_size: Some(Vec2::splat(radius * 2.0)),
                ..default()
            },
            Transform::from_translation(caster_position.with_z(0.5)),
        ));
    }
}

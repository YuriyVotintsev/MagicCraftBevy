use bevy::prelude::*;

use crate::abilities::context::AbilityContext;
use crate::abilities::effect_def::{EffectDef, ParamValue};
use crate::abilities::registry::{EffectExecutor, EffectRegistry};
use crate::wave::add_invulnerability;
use crate::MovementLocked;

const DEFAULT_DASH_SPEED: f32 = 1500.0;
const DEFAULT_DASH_DURATION: f32 = 0.2;

#[derive(Component)]
pub struct Dashing {
    pub timer: Timer,
    pub direction: Vec2,
    pub speed: f32,
}

pub struct DashEffect;

impl EffectExecutor for DashEffect {
    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        commands: &mut Commands,
        registry: &EffectRegistry,
    ) {
        let speed = match def.get_param("speed", registry) {
            Some(ParamValue::Float(v)) => *v,
            Some(ParamValue::Stat(stat_id)) => ctx.stats_snapshot.get(*stat_id),
            _ => DEFAULT_DASH_SPEED,
        };

        let duration = match def.get_param("duration", registry) {
            Some(ParamValue::Float(v)) => *v,
            _ => DEFAULT_DASH_DURATION,
        };

        let direction = ctx
            .target_direction
            .map(|d| d.truncate().normalize_or_zero())
            .unwrap_or(Vec2::ZERO);

        if direction == Vec2::ZERO {
            return;
        }

        let caster = ctx.caster;
        commands.entity(caster).insert((
            Dashing {
                timer: Timer::from_seconds(duration, TimerMode::Once),
                direction,
                speed,
            },
            MovementLocked,
        ));

        add_invulnerability(commands, caster);
    }
}

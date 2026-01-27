use crate::abilities::activator_def::{ActivatorDef, ActivatorState, ActivationResult};
use crate::abilities::context::AbilityContext;
use crate::abilities::components::AbilityInput;
use crate::abilities::effect_def::ParamValue;
use crate::abilities::registry::Activator;
use crate::abilities::ids::ParamId;

const COOLDOWN_TIMER_ID: ParamId = ParamId(0);

pub struct WhileHeldActivator;

impl Activator for WhileHeldActivator {
    fn check(
        &self,
        def: &ActivatorDef,
        state: &mut ActivatorState,
        ctx: &mut AbilityContext,
        input: &AbilityInput,
        delta_time: f32,
    ) -> ActivationResult {
        let timer = state.get(COOLDOWN_TIMER_ID);
        let new_timer = (timer - delta_time).max(0.0);
        state.set(COOLDOWN_TIMER_ID, new_timer);

        if input.want_to_cast != Some(ctx.ability_id) {
            return ActivationResult::NotReady;
        }

        if new_timer <= 0.0 {
            let cooldown = def.params.values()
                .find_map(|v| match v {
                    ParamValue::Float(f) => Some(*f),
                    _ => None,
                })
                .unwrap_or(0.05);
            state.set(COOLDOWN_TIMER_ID, cooldown);
            ActivationResult::Ready
        } else {
            ActivationResult::NotReady
        }
    }
}

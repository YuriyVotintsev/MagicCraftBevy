use crate::abilities::activator_def::{ActivatorDef, ActivatorState, ActivationResult};
use crate::abilities::context::AbilityContext;
use crate::abilities::components::AbilityInput;
use crate::abilities::effect_def::ParamValue;
use crate::abilities::registry::Activator;
use crate::abilities::ids::ParamId;

const INTERVAL_TIMER_ID: ParamId = ParamId(0);

pub struct IntervalActivator;

impl Activator for IntervalActivator {
    fn check(
        &self,
        def: &ActivatorDef,
        state: &mut ActivatorState,
        _ctx: &mut AbilityContext,
        _input: &AbilityInput,
        delta_time: f32,
    ) -> ActivationResult {
        let timer = state.get(INTERVAL_TIMER_ID);
        let new_timer = timer - delta_time;

        if new_timer <= 0.0 {
            let interval = def.params.values()
                .find_map(|v| match v {
                    ParamValue::Float(f) => Some(*f),
                    _ => None,
                })
                .unwrap_or(1.0);
            state.set(INTERVAL_TIMER_ID, interval);
            ActivationResult::Ready
        } else {
            state.set(INTERVAL_TIMER_ID, new_timer);
            ActivationResult::NotReady
        }
    }
}

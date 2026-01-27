use crate::abilities::registry::Activator;
use crate::abilities::activator_def::{ActivatorDef, ActivatorState, ActivationResult};
use crate::abilities::context::AbilityContext;
use crate::abilities::components::AbilityInput;

pub struct OnInputActivator;

impl Activator for OnInputActivator {
    fn check(
        &self,
        _def: &ActivatorDef,
        _state: &mut ActivatorState,
        ctx: &mut AbilityContext,
        input: &AbilityInput,
        _delta_time: f32,
    ) -> ActivationResult {
        if input.want_to_cast == Some(ctx.ability_id) {
            ActivationResult::Ready
        } else {
            ActivationResult::NotReady
        }
    }
}

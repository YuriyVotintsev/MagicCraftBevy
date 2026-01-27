use crate::abilities::activator_def::{ActivatorDef, ActivatorState, ActivationResult};
use crate::abilities::context::AbilityContext;
use crate::abilities::components::AbilityInput;
use crate::abilities::registry::Activator;

pub struct PassiveActivator;

impl Activator for PassiveActivator {
    fn check(
        &self,
        _def: &ActivatorDef,
        _state: &mut ActivatorState,
        _ctx: &mut AbilityContext,
        _input: &AbilityInput,
    ) -> ActivationResult {
        ActivationResult::Ready
    }
}

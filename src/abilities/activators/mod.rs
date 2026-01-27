mod on_input;
mod passive;

pub use on_input::OnInputActivator;
pub use passive::PassiveActivator;

use super::registry::ActivatorRegistry;

pub fn register_activators(registry: &mut ActivatorRegistry) {
    registry.register("on_input", OnInputActivator);
    registry.register("passive", PassiveActivator);
}

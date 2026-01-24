mod on_input;

pub use on_input::OnInputActivator;

use super::registry::ActivatorRegistry;

pub fn register_activators(registry: &mut ActivatorRegistry) {
    registry.register("on_input", OnInputActivator);
}

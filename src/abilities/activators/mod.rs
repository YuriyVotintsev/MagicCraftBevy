mod on_input;
mod passive;
mod while_held;

pub use on_input::OnInputActivator;
pub use passive::PassiveActivator;
pub use while_held::WhileHeldActivator;

use super::registry::ActivatorRegistry;

pub fn register_activators(registry: &mut ActivatorRegistry) {
    registry.register("on_input", OnInputActivator);
    registry.register("passive", PassiveActivator);
    registry.register("while_held", WhileHeldActivator);
}

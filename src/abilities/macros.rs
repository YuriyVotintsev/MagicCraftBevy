#[macro_export]
macro_rules! register_activator {
    ($handler:ty) => {
        pub fn __register_activator(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::ActivatorRegistry,
        ) {
            use $crate::abilities::registry::ActivatorHandler;
            let handler = <$handler>::default();
            handler.register_systems(app);
            registry.register(Box::new(handler));
        }
    };
}

#[macro_export]
macro_rules! collect_activators {
    ($($module:ident),* $(,)?) => {
        $(mod $module;)*
        $(pub use $module::*;)*

        pub fn register_all(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::ActivatorRegistry,
        ) {
            $($module::__register_activator(app, registry);)*
        }
    };
}

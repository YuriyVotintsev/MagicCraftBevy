#[macro_export]
macro_rules! register_trigger {
    ($handler:ty) => {
        pub fn __register_trigger(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::TriggerRegistry,
        ) {
            use $crate::abilities::registry::TriggerHandler;
            let handler = <$handler>::default();
            handler.register_systems(app);
            registry.register(Box::new(handler));
        }
    };
}

#[macro_export]
macro_rules! collect_triggers {
    ($($module:ident),* $(,)?) => {
        $(pub mod $module;)*

        pub fn register_all(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::TriggerRegistry,
        ) {
            $($module::__register_trigger(app, registry);)*
        }
    };
}

#[macro_export]
macro_rules! register_effect {
    ($handler:ty) => {
        pub fn __register_effect(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::EffectRegistry,
        ) {
            use $crate::abilities::registry::EffectHandler;
            let handler = <$handler>::default();
            handler.register_execution_system(app);
            handler.register_behavior_systems(app);
            registry.register(Box::new(handler));
        }
    };
}

#[macro_export]
macro_rules! collect_effects {
    ($($module:ident),* $(,)?) => {
        $(pub mod $module;)*

        pub fn register_all(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::EffectRegistry,
        ) {
            $($module::__register_effect(app, registry);)*
        }
    };
}

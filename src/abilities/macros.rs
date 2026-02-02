#[macro_export]
macro_rules! register_node {
    ($params:ty) => {
        pub type __NodeParams = $params;
        paste::paste! {
            pub type __NodeParamsRaw = [<$params Raw>];
        }

        pub fn __register_node(app: &mut ::bevy::prelude::App) {
            register_systems(app);
        }
    };
}

#[macro_export]
macro_rules! collect_action_nodes {
    ([$($module:ident),* $(,)?]) => {
        $(pub mod $module;)*

        paste::paste! {
            #[derive(Debug, Clone, serde::Deserialize)]
            pub enum ActionParamsRaw {
                $(
                    [<$module:camel Params>]($module::__NodeParamsRaw),
                )*
            }
        }

        paste::paste! {
            #[derive(Debug, Clone)]
            pub enum ActionParams {
                $(
                    [<$module:camel Params>]($module::__NodeParams),
                )*
            }
        }

        impl ActionParamsRaw {
            pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> ActionParams {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel Params>](p) => ActionParams::[<$module:camel Params>](p.resolve(stat_registry)),
                        )*
                    }
                }
            }

            pub fn children(&self) -> &[crate::building_blocks::NodeParamsRaw] {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel Params>](p) => p.children(),
                        )*
                    }
                }
            }

            pub fn name(&self) -> &'static str {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel Params>](_) => stringify!([<$module:camel Params>]),
                        )*
                    }
                }
            }
        }

        paste::paste! {
            impl ActionParams {
                $(
                    #[inline]
                    #[allow(dead_code)]
                    pub fn [<unwrap_ $module>](&self) -> &$module::__NodeParams {
                        match self {
                            Self::[<$module:camel Params>](p) => p,
                            _ => unreachable!("node_type check guarantees {} params", stringify!($module)),
                        }
                    }
                )*
            }
        }

        paste::paste! {
            $(
                #[derive(::bevy::prelude::Message, Clone)]
                pub struct [<Execute $module:camel Event>] {
                    pub base: $crate::abilities::events::ActionEventBase,
                    pub params: $module::__NodeParams,
                }
            )*

            #[derive(::bevy::ecs::system::SystemParam)]
            pub struct ActionEventWriters<'w> {
                $(
                    $module: ::bevy::prelude::MessageWriter<'w, [<Execute $module:camel Event>]>,
                )*
            }

            impl ActionEventWriters<'_> {
                pub fn dispatch(&mut self, base: $crate::abilities::events::ActionEventBase, params: &ActionParams) {
                    match params {
                        $(
                            ActionParams::[<$module:camel Params>](p) => {
                                self.$module.write([<Execute $module:camel Event>] {
                                    base,
                                    params: p.clone(),
                                });
                            }
                        )*
                    }
                }
            }

            pub fn init_action_messages(app: &mut ::bevy::prelude::App) {
                $(
                    app.init_resource::<::bevy::prelude::Messages<[<Execute $module:camel Event>]>>();
                )*
            }
        }

        pub fn register_all(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::NodeRegistry,
        ) {
            paste::paste! {
                $(
                    $module::__register_node(app);
                    let handler = $module::__NodeParams::default();
                    registry.register(Box::new(handler));
                )*
            }
        }
    };
}

#[macro_export]
macro_rules! collect_trigger_nodes {
    ([$($module:ident),* $(,)?]) => {
        $(pub mod $module;)*

        paste::paste! {
            #[derive(Debug, Clone, serde::Deserialize)]
            pub enum TriggerParamsRaw {
                $(
                    [<$module:camel Params>]($module::__NodeParamsRaw),
                )*
            }
        }

        paste::paste! {
            #[derive(Debug, Clone)]
            pub enum TriggerParams {
                $(
                    [<$module:camel Params>]($module::__NodeParams),
                )*
            }
        }

        impl TriggerParamsRaw {
            pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> TriggerParams {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel Params>](p) => TriggerParams::[<$module:camel Params>](p.resolve(stat_registry)),
                        )*
                    }
                }
            }

            pub fn children(&self) -> &[crate::building_blocks::NodeParamsRaw] {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel Params>](p) => p.children(),
                        )*
                    }
                }
            }

            pub fn name(&self) -> &'static str {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel Params>](_) => stringify!([<$module:camel Params>]),
                        )*
                    }
                }
            }
        }

        pub fn register_all(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::NodeRegistry,
        ) {
            paste::paste! {
                $(
                    $module::__register_node(app);
                    let handler = $module::__NodeParams::default();
                    registry.register(Box::new(handler));
                )*
            }
        }
    };
}

#[macro_export]
macro_rules! register_activator {
    ($params:ty, $component:ty) => {
        pub type __ActivatorParams = $params;
        paste::paste! {
            pub type __ActivatorParamsRaw = [<$params Raw>];
        }
        pub type __ActivatorComponent = $component;

        pub fn __register_activator(app: &mut ::bevy::prelude::App) {
            register_systems(app);
        }
    };
}

#[macro_export]
macro_rules! collect_activators {
    ([$($module:ident),* $(,)?]) => {
        $(pub mod $module;)*

        paste::paste! {
            #[derive(Debug, Clone, serde::Deserialize)]
            pub enum ActivatorParamsRaw {
                $(
                    [<$module:camel Params>]($module::__ActivatorParamsRaw),
                )*
            }
        }

        paste::paste! {
            #[derive(Debug, Clone)]
            pub enum ActivatorParams {
                $(
                    [<$module:camel Params>]($module::__ActivatorParams),
                )*
            }
        }

        impl ActivatorParamsRaw {
            pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> ActivatorParams {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel Params>](p) => ActivatorParams::[<$module:camel Params>](p.resolve(stat_registry)),
                        )*
                    }
                }
            }

            #[allow(dead_code)]
            pub fn name(&self) -> &'static str {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel Params>](_) => stringify!([<$module:camel Params>]),
                        )*
                    }
                }
            }

            pub fn children(&self) -> &[crate::building_blocks::NodeParamsRaw] {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel Params>](p) => p.children(),
                        )*
                    }
                }
            }
        }

        paste::paste! {
            impl ActivatorParams {
                $(
                    #[allow(dead_code)]
                    pub fn [<as_ $module>](&self) -> &$module::__ActivatorParams {
                        match self {
                            Self::[<$module:camel Params>](p) => p,
                            _ => panic!("Expected {} params", stringify!($module)),
                        }
                    }
                )*
            }
        }

        pub fn spawn_activator(
            commands: &mut ::bevy::prelude::EntityCommands,
            params: &ActivatorParams,
        ) {
            paste::paste! {
                match params {
                    $(
                        ActivatorParams::[<$module:camel Params>](p) => {
                            commands.insert($module::__ActivatorComponent::from_params(p));
                        }
                    )*
                }
            }
        }

        pub fn register_all(app: &mut ::bevy::prelude::App) {
            $(
                $module::__register_activator(app);
            )*
        }
    };
}

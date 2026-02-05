#[macro_export]
macro_rules! collect_components {
    ([$($module:ident),* $(,)?]) => {
        $(pub mod $module;)*

        paste::paste! {
            #[derive(Debug, Clone, serde::Deserialize)]
            pub enum ComponentDefRaw {
                $(
                    [<$module:camel>]($module::DefRaw),
                )*
            }
        }

        paste::paste! {
            #[derive(Debug, Clone)]
            pub enum ComponentDef {
                $(
                    [<$module:camel>]($module::Def),
                )*
            }
        }

        impl ComponentDefRaw {
            pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> ComponentDef {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel>](raw) => ComponentDef::[<$module:camel>](raw.resolve(stat_registry)),
                        )*
                    }
                }
            }

            pub fn required_fields_and_nested(&self) -> (
                crate::abilities::context::ProvidedFields,
                Option<(crate::abilities::context::ProvidedFields, &[crate::abilities::entity_def::EntityDefRaw])>,
            ) {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel>](raw) => $module::required_fields_and_nested(raw),
                        )*
                    }
                }
            }
        }

        impl ComponentDef {
            pub fn spawn(&self, commands: &mut ::bevy::prelude::EntityCommands, ctx: &super::spawn::SpawnContext) {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel>](def) => $module::spawn(commands, def, ctx),
                        )*
                    }
                }
            }
        }

        pub fn register_component_systems(app: &mut ::bevy::prelude::App) {
            $(
                $module::register_systems(app);
            )*
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

        pub fn __provided_fields() -> crate::abilities::context::ProvidedFields {
            provided_fields()
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
                    [<$module:camel>]($module::__ActivatorParamsRaw),
                )*
            }
        }

        paste::paste! {
            #[derive(Debug, Clone)]
            pub enum ActivatorParams {
                $(
                    [<$module:camel>]($module::__ActivatorParams),
                )*
            }
        }

        impl ActivatorParamsRaw {
            pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> ActivatorParams {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel>](p) => ActivatorParams::[<$module:camel>](p.resolve(stat_registry)),
                        )*
                    }
                }
            }

            #[allow(dead_code)]
            pub fn name(&self) -> &'static str {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel>](_) => stringify!([<$module:camel>]),
                        )*
                    }
                }
            }

            pub fn provided_fields(&self) -> crate::abilities::context::ProvidedFields {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$module:camel>](_) => $module::__provided_fields(),
                        )*
                    }
                }
            }
        }

        pub fn spawn_activator(
            commands: &mut ::bevy::prelude::EntityCommands,
            params: &ActivatorParams,
        ) {
            paste::paste! {
                match params {
                    $(
                        ActivatorParams::[<$module:camel>](p) => {
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

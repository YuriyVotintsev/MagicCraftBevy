#[macro_export]
macro_rules! collect_components {
    (
        components: [$($component:ident),* $(,)?]
    ) => {
        $(pub mod $component;)*

        paste::paste! {
            #[derive(Debug, Clone, serde::Deserialize)]
            pub enum ComponentDefRaw {
                $(
                    [<$component:camel>]($component::DefRaw),
                )*
            }
        }

        paste::paste! {
            #[derive(Debug, Clone)]
            pub enum ComponentDef {
                $(
                    [<$component:camel>]($component::Def),
                )*
            }
        }

        impl ComponentDefRaw {
            pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry, state_indices: Option<&std::collections::HashMap<String, usize>>) -> ComponentDef {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$component:camel>](raw) => ComponentDef::[<$component:camel>](raw.resolve(stat_registry, state_indices)),
                        )*
                    }
                }
            }

            #[cfg(test)]
            pub fn required_fields_and_nested(&self) -> (
                crate::abilities::context::ProvidedFields,
                Option<(crate::abilities::context::ProvidedFields, &[crate::abilities::entity_def::EntityDefRaw])>,
            ) {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$component:camel>](raw) => $component::required_fields_and_nested(raw),
                        )*
                    }
                }
            }
        }

        impl ComponentDef {
            pub fn insert_component(&self, commands: &mut ::bevy::prelude::EntityCommands, source: &super::AbilitySource, stats: &crate::stats::ComputedStats) {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$component:camel>](def) => $component::insert_component(commands, def, source, stats),
                        )*
                    }
                }
            }

            pub fn update_component(&self, commands: &mut ::bevy::prelude::EntityCommands, source: &super::AbilitySource, stats: &crate::stats::ComputedStats) {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$component:camel>](def) => $component::update_component(commands, def, source, stats),
                        )*
                    }
                }
            }

            pub fn remove_component(&self, commands: &mut ::bevy::prelude::EntityCommands) {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$component:camel>](_) => $component::remove_component(commands),
                        )*
                    }
                }
            }

            pub fn has_recalc(&self) -> bool {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$component:camel>](def) => $component::has_recalc(def),
                        )*
                    }
                }
            }
        }

        pub fn register_component_systems(app: &mut ::bevy::prelude::App) {
            $(
                $component::register_systems(app);
            )*
        }
    };
}

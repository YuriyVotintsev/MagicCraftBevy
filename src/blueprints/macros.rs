#[macro_export]
macro_rules! collect_components {
    (
        groups: {
            $($group:ident: [$($component:ident),* $(,)?]),* $(,)?
        }
    ) => {
        $(
            pub mod $group {
                $(pub mod $component;)*
            }
        )*

        paste::paste! {
            #[derive(Debug, Clone, serde::Deserialize)]
            pub enum ComponentDefRaw {
                $($(
                    [<$component:camel>]($group::$component::DefRaw),
                )*)*
            }
        }

        paste::paste! {
            #[derive(Debug, Clone)]
            pub enum ComponentDef {
                $($(
                    [<$component:camel>]($group::$component::Def),
                )*)*
            }
        }

        impl ComponentDefRaw {
            pub fn resolve(&self, lookup: &dyn Fn(&str) -> Option<crate::expr::StatId>, state_indices: Option<&std::collections::HashMap<String, usize>>, calc_reg: &crate::expr::calc::CalcRegistry) -> ComponentDef {
                paste::paste! {
                    match self {
                        $($(
                            Self::[<$component:camel>](raw) => ComponentDef::[<$component:camel>](raw.resolve(lookup, state_indices, calc_reg)),
                        )*)*
                    }
                }
            }

            #[cfg(test)]
            pub fn required_fields_and_nested(&self) -> (
                crate::blueprints::context::ProvidedFields,
                Option<(crate::blueprints::context::ProvidedFields, &[crate::blueprints::entity_def::EntityDefRaw])>,
            ) {
                paste::paste! {
                    match self {
                        $($(
                            Self::[<$component:camel>](raw) => $group::$component::required_fields_and_nested(raw),
                        )*)*
                    }
                }
            }
        }

        impl ComponentDef {
            pub fn insert_component(&self, commands: &mut ::bevy::prelude::EntityCommands, source: &super::SpawnSource, stats: &crate::stats::ComputedStats) {
                paste::paste! {
                    match self {
                        $($(
                            Self::[<$component:camel>](def) => $group::$component::insert_component(commands, def, source, stats),
                        )*)*
                    }
                }
            }

            pub fn update_component(&self, commands: &mut ::bevy::prelude::EntityCommands, source: &super::SpawnSource, stats: &crate::stats::ComputedStats) {
                paste::paste! {
                    match self {
                        $($(
                            Self::[<$component:camel>](def) => $group::$component::update_component(commands, def, source, stats),
                        )*)*
                    }
                }
            }

            pub fn remove_component(&self, commands: &mut ::bevy::prelude::EntityCommands) {
                paste::paste! {
                    match self {
                        $($(
                            Self::[<$component:camel>](_) => $group::$component::remove_component(commands),
                        )*)*
                    }
                }
            }

            pub fn has_recalc(&self) -> bool {
                paste::paste! {
                    match self {
                        $($(
                            Self::[<$component:camel>](def) => $group::$component::has_recalc(def),
                        )*)*
                    }
                }
            }
        }

        pub fn register_component_systems(app: &mut ::bevy::prelude::App) {
            $($(
                $group::$component::register_systems(app);
            )*)*
        }
    };
}

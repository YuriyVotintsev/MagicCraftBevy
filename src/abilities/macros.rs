#[macro_export]
macro_rules! collect_components {
    (
        activators: [$($activator:ident),* $(,)?],
        components: [$($component:ident),* $(,)?]
    ) => {
        $(pub mod $activator;)*
        $(pub mod $component;)*

        paste::paste! {
            #[derive(Debug, Clone, serde::Deserialize)]
            pub enum ComponentDefRaw {
                $(
                    [<$activator:camel>]($activator::DefRaw),
                )*
                $(
                    [<$component:camel>]($component::DefRaw),
                )*
            }
        }

        paste::paste! {
            #[derive(Debug, Clone)]
            pub enum ComponentDef {
                $(
                    [<$activator:camel>]($activator::Def),
                )*
                $(
                    [<$component:camel>]($component::Def),
                )*
            }
        }

        impl ComponentDefRaw {
            pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> ComponentDef {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$activator:camel>](raw) => ComponentDef::[<$activator:camel>](raw.resolve(stat_registry)),
                        )*
                        $(
                            Self::[<$component:camel>](raw) => ComponentDef::[<$component:camel>](raw.resolve(stat_registry)),
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
                            Self::[<$activator:camel>](raw) => $activator::required_fields_and_nested(raw),
                        )*
                        $(
                            Self::[<$component:camel>](raw) => $component::required_fields_and_nested(raw),
                        )*
                    }
                }
            }

            pub fn is_activator(&self) -> bool {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$activator:camel>](_) => true,
                        )*
                        $(
                            Self::[<$component:camel>](_) => false,
                        )*
                    }
                }
            }

            pub fn provided_fields(&self) -> crate::abilities::context::ProvidedFields {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$activator:camel>](_) => $activator::provided_fields(),
                        )*
                        $(
                            Self::[<$component:camel>](_) => crate::abilities::context::ProvidedFields::NONE,
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
                            Self::[<$activator:camel>](def) => $activator::insert_component(commands, def, source, stats),
                        )*
                        $(
                            Self::[<$component:camel>](def) => $component::insert_component(commands, def, source, stats),
                        )*
                    }
                }
            }

            pub fn update_component(&self, entity: ::bevy::prelude::Entity, source: &super::AbilitySource, stats: &crate::stats::ComputedStats, world: &mut ::bevy::prelude::World) {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$activator:camel>](def) => $activator::update_component(entity, def, source, stats, world),
                        )*
                        $(
                            Self::[<$component:camel>](def) => $component::update_component(entity, def, source, stats, world),
                        )*
                    }
                }
            }

            #[allow(dead_code)]
            pub fn is_activator(&self) -> bool {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$activator:camel>](_) => true,
                        )*
                        $(
                            Self::[<$component:camel>](_) => false,
                        )*
                    }
                }
            }

            pub fn remove_component(&self, commands: &mut ::bevy::prelude::EntityCommands) {
                paste::paste! {
                    match self {
                        $(
                            Self::[<$activator:camel>](_) => $activator::remove_component(commands),
                        )*
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
                            Self::[<$activator:camel>](def) => $activator::has_recalc(def),
                        )*
                        $(
                            Self::[<$component:camel>](def) => $component::has_recalc(def),
                        )*
                    }
                }
            }
        }

        pub fn register_component_systems(app: &mut ::bevy::prelude::App) {
            $(
                $activator::register_systems(app);
            )*
            $(
                $component::register_systems(app);
            )*
        }
    };
}

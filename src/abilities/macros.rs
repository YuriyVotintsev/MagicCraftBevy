#[macro_export]
macro_rules! register_node {
    ($handler:ty, params: $params:ty, name: $name:expr, systems: $systems:path) => {
        pub type __NodeParams = $params;
        pub const __NODE_NAME: &str = $name;

        pub fn __register_node(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::NodeRegistry,
        ) {
            $systems(app);

            let handler = <$handler>::default();
            registry.register(Box::new(handler));
        }
    };
}

#[macro_export]
macro_rules! collect_nodes {
    (
        with_params: [$($with_params:ident),* $(,)?],
        no_params: [$($no_params:ident),* $(,)?]
    ) => {
        $(pub mod $with_params;)*
        $(pub mod $no_params;)*

        #[derive(Debug, Clone)]
        #[allow(non_camel_case_types, dead_code)]
        pub enum NodeParams {
            $(
                $with_params($with_params::__NodeParams),
            )*
            $(
                $no_params($no_params::__NodeParams),
            )*
        }

        #[allow(dead_code)]
        impl NodeParams {
            pub fn parse(
                name: &str,
                raw: &::std::collections::HashMap<String, $crate::abilities::ParamValueRaw>,
                stat_registry: &$crate::stats::StatRegistry,
            ) -> Self {
                use $crate::abilities::ParseNodeParams;
                match name {
                    $(
                        $with_params::__NODE_NAME => {
                            Self::$with_params($with_params::__NodeParams::parse(raw, stat_registry))
                        }
                    )*
                    $(
                        $no_params::__NODE_NAME => {
                            Self::$no_params($no_params::__NodeParams::parse(raw, stat_registry))
                        }
                    )*
                    _ => panic!("Unknown node type: {}", name),
                }
            }

            // Only generate unwrap methods for handlers with params
            $(
                paste::paste! {
                    #[inline]
                    pub fn [<unwrap_ $with_params>](&self) -> &$with_params::__NodeParams {
                        match self {
                            Self::$with_params(p) => p,
                            _ => unreachable!("node_type check guarantees {} params", stringify!($with_params)),
                        }
                    }
                }
            )*
        }

        pub fn register_all(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::NodeRegistry,
        ) {
            $($with_params::__register_node(app, registry);)*
            $($no_params::__register_node(app, registry);)*
        }
    };
}

#[macro_export]
macro_rules! register_activator {
    ($component:ty, params: (), name: $name:expr) => {
        pub type __ActivatorComponent = $component;
        pub type __ActivatorParams = ();
        pub const __ACTIVATOR_NAME: &str = $name;
    };
    ($component:ty, params: $params:ty, name: $name:expr) => {
        pub type __ActivatorComponent = $component;
        pub type __ActivatorParams = $params;
        pub const __ACTIVATOR_NAME: &str = $name;

        impl __ActivatorComponent {
            pub fn from_params(params: &__ActivatorParams) -> Self {
                Self::from_params_impl(params)
            }
        }
    };
}

#[macro_export]
macro_rules! collect_activators {
    (
        with_params: [$($with_params:ident),* $(,)?],
        no_params: [$($no_params:ident),* $(,)?]
    ) => {
        $(pub mod $with_params;)*
        $(pub mod $no_params;)*

        paste::paste! {
            #[derive(Debug, Clone)]
            pub enum ActivatorParams {
                $(
                    [<$with_params:camel>]($with_params::__ActivatorParams),
                )*
                $(
                    [<$no_params:camel>],
                )*
            }
        }

        paste::paste! {
            impl ActivatorParams {
                $(
                    pub fn [<as_ $with_params>](&self) -> &$with_params::__ActivatorParams {
                        match self {
                            Self::[<$with_params:camel>](p) => p,
                            _ => panic!("Expected {} params", stringify!($with_params)),
                        }
                    }
                )*
            }
        }

        pub fn spawn_activator(
            commands: &mut ::bevy::prelude::EntityCommands,
            activator_type: &str,
            params: &ActivatorParams,
        ) {
            match activator_type {
                $(
                    $with_params::__ACTIVATOR_NAME => {
                        paste::paste! {
                            let p = params.[<as_ $with_params>]();
                            commands.insert($with_params::__ActivatorComponent::from_params(p));
                        }
                    }
                )*
                $(
                    $no_params::__ACTIVATOR_NAME => {
                        commands.insert($no_params::__ActivatorComponent::default());
                    }
                )*
                unknown => {
                    ::bevy::prelude::warn!("Unknown activator type: {}", unknown);
                }
            }
        }

        pub fn register_all(app: &mut ::bevy::prelude::App) {
            $(
                $with_params::register_systems(app);
            )*
            $(
                $no_params::register_systems(app);
            )*
        }
    };
}

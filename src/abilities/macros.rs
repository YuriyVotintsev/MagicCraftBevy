#[macro_export]
macro_rules! register_node {
    ($handler:ty, params: $params:ty, name: $name:expr) => {
        pub type __NodeParams = $params;
        pub const __NODE_NAME: &str = $name;

        pub fn __register_node(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::NodeRegistry,
        ) {
            use $crate::abilities::node::NodeHandler;
            let handler = <$handler>::default();
            let kind = <$handler as NodeHandler>::kind(&handler);

            match kind {
                $crate::abilities::NodeKind::Trigger => {
                    <$handler as NodeHandler>::register_input_systems(&handler, app);
                }
                $crate::abilities::NodeKind::Action => {
                    <$handler as NodeHandler>::register_execution_system(&handler, app);
                    <$handler as NodeHandler>::register_behavior_systems(&handler, app);
                }
            }

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
        #[allow(non_camel_case_types)]
        pub enum NodeParams {
            $(
                $with_params($with_params::__NodeParams),
            )*
            $(
                $no_params($no_params::__NodeParams),
            )*
        }

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

#[macro_export]
macro_rules! register_node {
    ($handler:ty, params: $params:ty, name: $name:literal) => {
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
    ($($module:ident),* $(,)?) => {
        $(pub mod $module;)*

        #[derive(Debug, Clone)]
        #[allow(non_camel_case_types)]
        pub enum NodeParams {
            $(
                $module($module::__NodeParams),
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
                        $module::__NODE_NAME => {
                            Self::$module($module::__NodeParams::parse(raw, stat_registry))
                        }
                    )*
                    _ => panic!("Unknown node type: {}", name),
                }
            }

            $(
                paste::paste! {
                    pub fn [<expect_ $module>](&self) -> &$module::__NodeParams {
                        match self {
                            Self::$module(p) => p,
                            _ => panic!("Expected {} params", stringify!($module)),
                        }
                    }
                }
            )*
        }

        pub fn register_all(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::NodeRegistry,
        ) {
            $($module::__register_node(app, registry);)*
        }
    };
}

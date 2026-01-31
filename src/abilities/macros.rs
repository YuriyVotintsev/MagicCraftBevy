#[macro_export]
macro_rules! register_node {
    ($handler:ty) => {
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

        pub fn register_all(
            app: &mut ::bevy::prelude::App,
            registry: &mut $crate::abilities::NodeRegistry,
        ) {
            $($module::__register_node(app, registry);)*
        }
    };
}

use bevy::prelude::*;
use crate::abilities::ids::AbilityId;

#[derive(Component)]
pub struct AbilityInstance {
    pub ability_id: AbilityId,
    pub owner: Entity,
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

collect_activators! {
    with_params: [interval, while_held],
    no_params: [on_input, every_frame]
}

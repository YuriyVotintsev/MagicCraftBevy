use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

#[blueprint_component]
pub struct DynamicBody {
    #[raw(default = 1.0)]
    pub mass: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_dynamic_body);
}

fn init_dynamic_body(mut commands: Commands, query: Query<(Entity, &DynamicBody), Added<DynamicBody>>) {
    for (entity, body) in &query {
        commands.entity(entity).insert((
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED.lock_translation_y(),
            LinearVelocity::ZERO,
            Mass(body.mass),
            SleepingDisabled,
        ));
    }
}

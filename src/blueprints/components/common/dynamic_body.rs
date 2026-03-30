use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

#[blueprint_component]
pub struct DynamicBody;

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_dynamic_body);
}

fn init_dynamic_body(mut commands: Commands, query: Query<Entity, Added<DynamicBody>>) {
    for entity in &query {
        commands.entity(entity).insert((
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED.lock_translation_y(),
            LinearVelocity::ZERO,
        ));
    }
}

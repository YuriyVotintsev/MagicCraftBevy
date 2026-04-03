use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

#[blueprint_component]
pub struct StaticBody;

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_static_body);
}

fn init_static_body(mut commands: Commands, query: Query<Entity, Added<StaticBody>>) {
    for entity in &query {
        commands.entity(entity).insert(RigidBody::Static);
    }
}

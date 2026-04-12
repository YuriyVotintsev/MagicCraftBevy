use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct DynamicBody {
    pub mass: f32,
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

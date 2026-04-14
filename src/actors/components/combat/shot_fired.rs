use bevy::prelude::*;

#[derive(Component)]
pub struct ShotFired;

pub fn register_systems(app: &mut App) {
    app.add_systems(Update, cleanup_shot_fired);
}

fn cleanup_shot_fired(mut commands: Commands, q: Query<Entity, With<ShotFired>>) {
    for e in &q { commands.entity(e).remove::<ShotFired>(); }
}

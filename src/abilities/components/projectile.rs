use bevy::prelude::*;
use magic_craft_macros::ability_component;

#[ability_component]
pub struct Projectile;

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_projectile);
}

fn init_projectile(mut commands: Commands, query: Query<Entity, Added<Projectile>>) {
    for entity in &query {
        commands.entity(entity).insert(Name::new("Projectile"));
    }
}

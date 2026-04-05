use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::palette;

const CYLINDER_RADIUS: f32 = 0.2;
const CYLINDER_HEIGHT: f32 = 0.6;

#[blueprint_component]
pub struct TowerVisual {}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_tower_visual);
}

fn init_tower_visual(
    mut commands: Commands,
    query: Query<Entity, Added<TowerVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &query {
        let color = palette::color("enemy_ability");
        let material = materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        });
        let mesh = meshes.add(Cylinder::new(CYLINDER_RADIUS, CYLINDER_HEIGHT));
        let cylinder = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(0.0, CYLINDER_HEIGHT / 2.0, 0.0)),
            ))
            .id();
        commands.entity(entity).add_child(cylinder);
    }
}

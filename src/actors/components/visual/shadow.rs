use bevy::prelude::*;

use crate::dissolve_material::DissolveMaterial;
use crate::palette;

#[derive(Component, Default)]
pub struct Shadow;

#[derive(Resource)]
struct ShadowMeshHandle(Handle<Mesh>);

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, setup_shadow_mesh);
    app.add_systems(PostUpdate, init_shadow);
}

fn setup_shadow_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Circle::new(0.5));
    commands.insert_resource(ShadowMeshHandle(mesh));
}

fn init_shadow(
    mut commands: Commands,
    query: Query<Entity, Added<Shadow>>,
    shadow_mesh: Option<Res<ShadowMeshHandle>>,
    mut materials: ResMut<Assets<DissolveMaterial>>,
) {
    let Some(shadow_mesh) = shadow_mesh else { return };

    for entity in &query {
        let material = materials.add(DissolveMaterial::new(palette::color("cream_dark")));
        commands.entity(entity).insert((
            Mesh3d(shadow_mesh.0.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::new(0.0, 0.01, 0.0))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ));
    }
}

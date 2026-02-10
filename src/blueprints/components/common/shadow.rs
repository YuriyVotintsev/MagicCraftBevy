use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

#[blueprint_component]
pub struct Shadow {
    #[raw(default = -0.42)]
    pub y_offset: ScalarExpr,
    #[raw(default = 0.45)]
    pub opacity: ScalarExpr,
}

#[derive(Resource)]
struct ShadowMeshHandle(Handle<Mesh>);

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, setup_shadow_mesh);
    app.add_systems(PostUpdate, init_shadow);
}

fn setup_shadow_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Ellipse::new(0.4, 0.1));
    commands.insert_resource(ShadowMeshHandle(mesh));
}

fn init_shadow(
    mut commands: Commands,
    query: Query<(Entity, &Shadow), Added<Shadow>>,
    shadow_mesh: Option<Res<ShadowMeshHandle>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(shadow_mesh) = shadow_mesh else { return };

    for (entity, shadow) in &query {
        let material = materials.add(ColorMaterial::from_color(
            Color::srgba(0.0, 0.0, 0.0, shadow.opacity),
        ));
        commands.entity(entity).insert((
            Mesh2d(shadow_mesh.0.clone()),
            MeshMaterial2d(material),
            Transform::from_translation(Vec3::new(0.0, shadow.y_offset, -0.001)),
        ));
    }
}

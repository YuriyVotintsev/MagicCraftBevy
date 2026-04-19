use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

#[derive(ShaderType, Clone)]
pub struct RuneBallMaterialData {
    pub base_color: LinearRgba,
    pub icon_dir: Vec4,
    pub icon_radius: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct RuneBallMaterial {
    #[uniform(0)]
    pub data: RuneBallMaterialData,
    #[texture(1)]
    #[sampler(2)]
    pub icon: Handle<Image>,
}

impl Material for RuneBallMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/rune_ball_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

pub struct RuneBallMaterialPlugin;

impl Plugin for RuneBallMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<RuneBallMaterial>::default());
    }
}

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

#[derive(ShaderType, Clone)]
pub struct DissolveMaterialData {
    pub color: LinearRgba,
    pub alpha: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct DissolveMaterial {
    #[uniform(0)]
    pub data: DissolveMaterialData,
}

impl DissolveMaterial {
    pub fn new(color: Color) -> Self {
        Self {
            data: DissolveMaterialData {
                color: color.to_linear(),
                alpha: 1.0,
            },
        }
    }
}

impl Material for DissolveMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/dissolve_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        if self.data.alpha < 1.0 {
            AlphaMode::Mask(0.5)
        } else {
            AlphaMode::Opaque
        }
    }
}

pub struct DissolveMaterialPlugin;

impl Plugin for DissolveMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<DissolveMaterial>::default());
    }
}

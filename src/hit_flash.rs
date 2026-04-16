use bevy::prelude::*;

use crate::actors::{CapsuleShape, CircleShape};
use crate::composite_scale::{ScaleLayerId, ScaleLayerRegistry, ScaleModifiers};
use crate::health_material::HealthMaterial;

#[derive(Component)]
pub struct HitFlash {
    elapsed: f32,
    duration: f32,
}

impl HitFlash {
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            duration: 0.3,
        }
    }
}

#[derive(Resource)]
struct HitFlashScaleLayer(ScaleLayerId);

pub struct HitFlashPlugin;

impl Plugin for HitFlashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_layer)
            .add_systems(PostUpdate, tick_hit_flash);
    }
}

fn register_layer(mut registry: ResMut<ScaleLayerRegistry>, mut commands: Commands) {
    commands.insert_resource(HitFlashScaleLayer(registry.register()));
}

fn get_shape_colors(
    entity: Entity,
    color_query: &Query<(Option<&CircleShape>, Option<&CapsuleShape>)>,
) -> Option<(Color, Option<Color>)> {
    color_query.get(entity).ok().and_then(|(c, cap)| {
        c.map(|c| (c.color, c.flash_color))
            .or(cap.map(|cap| (cap.color, cap.flash_color)))
    })
}

fn tick_hit_flash(
    layer: Res<HitFlashScaleLayer>,
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &mut HitFlash, &Children)>,
    material_query: Query<
        &MeshMaterial3d<StandardMaterial>,
        Without<MeshMaterial3d<HealthMaterial>>,
    >,
    health_mat_query: Query<
        &MeshMaterial3d<HealthMaterial>,
        Without<MeshMaterial3d<StandardMaterial>>,
    >,
    mut modifiers_query: Query<&mut ScaleModifiers>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut health_materials: ResMut<Assets<HealthMaterial>>,
    color_query: Query<(Option<&CircleShape>, Option<&CapsuleShape>)>,
) {
    for (entity, mut flash, children) in &mut flash_query {
        flash.elapsed += time.delta_secs();
        let t = (flash.elapsed / flash.duration).clamp(0.0, 1.0);

        let done = t >= 1.0;

        let (scale_x, scale_y) = if done {
            (1.0, 1.0)
        } else {
            (0.7_f32.lerp(1.0, t), 1.3_f32.lerp(1.0, t))
        };

        for child in children.iter() {
            if let Ok(mut modifiers) = modifiers_query.get_mut(child) {
                modifiers.set(layer.0, Vec3::new(scale_x, scale_y, scale_x));
            }

            let Some((original_color, flash_color)) = get_shape_colors(child, &color_query) else {
                continue;
            };

            if let Ok(mat_handle) = material_query.get(child) {
                if let Some(material) = materials.get_mut(&mat_handle.0) {
                    if done {
                        material.base_color = original_color;
                    } else {
                        let flash_amount = 1.0 - t;
                        if let Some(target) = flash_color {
                            let orig = original_color.to_srgba();
                            let tgt = target.to_srgba();
                            material.base_color = Color::srgb(
                                orig.red.lerp(tgt.red, flash_amount),
                                orig.green.lerp(tgt.green, flash_amount),
                                orig.blue.lerp(tgt.blue, flash_amount),
                            );
                        }
                    }
                }
            } else if let Ok(mat_handle) = health_mat_query.get(child) {
                if let Some(material) = health_materials.get_mut(&mat_handle.0) {
                    if done {
                        material.data.base_color = original_color.to_linear();
                    } else {
                        let flash_amount = 1.0 - t;
                        if let Some(target) = flash_color {
                            let orig = original_color.to_srgba();
                            let tgt = target.to_srgba();
                            material.data.base_color = Color::srgb(
                                orig.red.lerp(tgt.red, flash_amount),
                                orig.green.lerp(tgt.green, flash_amount),
                                orig.blue.lerp(tgt.blue, flash_amount),
                            )
                            .to_linear();
                        }
                    }
                }
            }
        }

        if done {
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

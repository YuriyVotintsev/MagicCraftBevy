use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use crate::blueprints::components::common::health::Health;
use crate::blueprints::components::common::sprite::{CapsuleSprite, CircleSprite, Sprite};
use crate::palette;
use crate::stats::{ComputedStats, StatRegistry};
use crate::Faction;

#[derive(ShaderType, Clone)]
pub struct HealthMaterialData {
    pub base_color: LinearRgba,
    pub damage_color: LinearRgba,
    pub hp_fraction: f32,
    pub uv_top: f32,
    pub uv_bottom: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct HealthMaterial {
    #[uniform(0)]
    pub data: HealthMaterialData,
}

impl Material for HealthMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/health_material.wgsl".into()
    }
}

#[derive(Component)]
pub struct HealthMaterialLink {
    pub handle: Handle<HealthMaterial>,
}

pub struct HealthMaterialPlugin;

impl Plugin for HealthMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<HealthMaterial>::default());
        app.add_systems(PostUpdate, (apply_health_material, update_health_material));
    }
}

fn apply_health_material(
    mut commands: Commands,
    stat_registry: Res<StatRegistry>,
    mut health_materials: ResMut<Assets<HealthMaterial>>,
    enemy_query: Query<
        (Entity, &Faction, &Health, &ComputedStats, &Children),
        Without<HealthMaterialLink>,
    >,
    circle_query: Query<(Entity, &CircleSprite), With<MeshMaterial3d<StandardMaterial>>>,
    capsule_query: Query<(Entity, &CapsuleSprite, &Sprite), With<MeshMaterial3d<StandardMaterial>>>,
) {
    let max_life_id = stat_registry.get("max_life");

    for (enemy_entity, faction, health, stats, children) in &enemy_query {
        if *faction != Faction::Enemy {
            continue;
        }

        for child in children.iter() {
            let sprite_info: Option<(Entity, Color, f32, f32)> = circle_query
                .get(child)
                .ok()
                .map(|(e, c)| (e, c.color, 0.0, 1.0))
                .or_else(|| {
                    capsule_query.get(child).ok().map(|(e, c, sprite)| {
                        let mesh_half = c.half_length + 0.5;
                        let total = 2.0 * mesh_half;
                        let ground_uv = ((mesh_half - sprite.elevation) / total).clamp(0.0, 1.0);
                        (e, c.color, 1.0, ground_uv)
                    })
                });

            let Some((sprite_entity, color, uv_top, uv_bottom)) = sprite_info else {
                continue;
            };

            let max = max_life_id.map(|id| stats.get(id)).unwrap_or(1.0).max(1.0);
            let hp_frac = (health.current / max).clamp(0.0, 1.0);

            let handle = health_materials.add(HealthMaterial {
                data: HealthMaterialData {
                    base_color: color.to_linear(),
                    damage_color: palette::color("enemy_injured").to_linear(),
                    hp_fraction: hp_frac,
                    uv_top,
                    uv_bottom,
                },
            });

            commands
                .entity(sprite_entity)
                .remove::<MeshMaterial3d<StandardMaterial>>()
                .insert(MeshMaterial3d(handle.clone()));

            commands
                .entity(enemy_entity)
                .insert(HealthMaterialLink { handle });

            break;
        }
    }
}

fn update_health_material(
    stat_registry: Res<StatRegistry>,
    mut health_materials: ResMut<Assets<HealthMaterial>>,
    query: Query<
        (&Health, &ComputedStats, &HealthMaterialLink),
        Or<(Changed<Health>, Changed<ComputedStats>)>,
    >,
) {
    let max_life_id = stat_registry.get("max_life");

    for (health, stats, link) in &query {
        let max = max_life_id.map(|id| stats.get(id)).unwrap_or(1.0).max(1.0);
        let hp_frac = (health.current / max).clamp(0.0, 1.0);

        if let Some(material) = health_materials.get_mut(&link.handle) {
            material.data.hp_fraction = hp_frac;
        }
    }
}

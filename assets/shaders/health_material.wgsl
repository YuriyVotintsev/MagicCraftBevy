#import bevy_pbr::forward_io::VertexOutput

struct HealthMaterialData {
    base_color: vec4<f32>,
    damage_color: vec4<f32>,
    hp_fraction: f32,
    uv_top: f32,
    uv_bottom: f32,
    alpha: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: HealthMaterialData;

const EDGE_SOFTNESS: f32 = 0.02;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    if (material.hp_fraction >= 1.0) {
        return vec4(material.base_color.rgb, material.base_color.a * material.alpha);
    }

    let uv_min = min(material.uv_top, material.uv_bottom);
    let uv_max = max(material.uv_top, material.uv_bottom);
    let raw = clamp(in.uv.y, uv_min, uv_max);
    let normalized = (raw - material.uv_top) / (material.uv_bottom - material.uv_top);

    let damage_level = 1.0 - material.hp_fraction;
    let blend = 1.0 - smoothstep(damage_level - EDGE_SOFTNESS, damage_level + EDGE_SOFTNESS, normalized);

    let color = mix(material.base_color.rgb, material.damage_color.rgb, blend);
    return vec4(color, material.base_color.a * material.alpha);
}

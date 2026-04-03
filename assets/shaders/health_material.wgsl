#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct HealthMaterialData {
    base_color: vec4<f32>,
    hp_fraction: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: HealthMaterialData;

const GRAY: vec3<f32> = vec3(0.35, 0.35, 0.35);
const EDGE_SOFTNESS: f32 = 0.02;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // For a sphere, the normal direction equals the position direction from center.
    // Transforming to view space gives us screen-relative vertical position.
    let view_normal = normalize((view.view_from_world * vec4(in.world_normal, 0.0)).xyz);

    // Map view_normal.y from [-1, 1] to [0, 1]: 0 = screen bottom, 1 = screen top
    let t = (view_normal.y + 1.0) * 0.5;

    // Smooth blend at the health boundary: below = base color, above = gray
    let blend = smoothstep(
        material.hp_fraction - EDGE_SOFTNESS,
        material.hp_fraction + EDGE_SOFTNESS,
        t
    );

    let color = mix(material.base_color.rgb, GRAY, blend);
    return vec4(color, material.base_color.a);
}

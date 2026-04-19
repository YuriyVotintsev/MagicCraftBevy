#import bevy_pbr::forward_io::VertexOutput

struct RuneBallData {
    base_color: vec4<f32>,
    icon_dir: vec4<f32>,
    icon_radius: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: RuneBallData;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var icon_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var icon_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let dir = normalize(material.icon_dir.xyz);
    let cos_a = dot(n, dir);

    var color = material.base_color.rgb;

    if (cos_a > 0.0) {
        let world_up = vec3<f32>(0.0, 1.0, 0.0);
        var right = cross(world_up, dir);
        if (length(right) < 0.001) {
            right = vec3<f32>(1.0, 0.0, 0.0);
        }
        right = normalize(right);
        let up = normalize(cross(dir, right));

        let u = dot(n, right);
        let v = dot(n, up);

        let r = material.icon_radius;
        let uv = vec2<f32>(0.5 + u / (2.0 * r), 0.5 - v / (2.0 * r));

        if (uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0) {
            let icon = textureSample(icon_tex, icon_sampler, uv);
            color = mix(color, icon.rgb, icon.a);
        }
    }

    return vec4(color, material.base_color.a);
}

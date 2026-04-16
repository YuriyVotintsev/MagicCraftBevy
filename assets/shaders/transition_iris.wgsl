#import bevy_ui::ui_vertex_output::UiVertexOutput

struct IrisData {
    color: vec4<f32>,
    radius: f32,
    softness: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(1) @binding(0) var<uniform> material: IrisData;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let centered = in.uv - vec2<f32>(0.5, 0.5);
    let aspect = in.size.x / max(in.size.y, 1.0);
    let corrected = vec2<f32>(centered.x * aspect, centered.y);
    let dist = length(corrected);
    let alpha = smoothstep(material.radius, material.radius + material.softness, dist);
    return vec4<f32>(material.color.rgb, material.color.a * alpha);
}

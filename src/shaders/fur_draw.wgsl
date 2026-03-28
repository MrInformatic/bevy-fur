#import bevy_render::view::View

struct OutputVertex {
    position: vec4<f32>,
    color:    vec4<f32>,
}

@group(0) @binding(0) var<storage, read> verts: array<OutputVertex>;
@group(1) @binding(0) var<uniform>       view:  View;

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)       color:    vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    let v = verts[vi];
    return VsOut(view.clip_from_world * v.position, v.color);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}

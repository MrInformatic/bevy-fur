// Fur geometry — Approach 1 (gs_1 port)
// 25 uniform layers, each layer extrudes all 3 triangle vertices along
// their per-vertex normals by LAYER_STEP * n.
// One workgroup per input triangle, 25 threads (one per layer).
// Each thread writes 3 output vertices.

const LAYERS:        u32 = 25u;
const LAYER_STEP:    f32 = 0.04;
const VERTS_PER_TRI: u32 = 75u;  // 25 * 3

// ---- types ----

struct InputVertex {
    position: vec3<f32>,
    _pad0:    f32,
    normal:   vec3<f32>,
    _pad1:    f32,
}

struct InputTri {
    v: array<InputVertex, 3>,
}

struct OutputVertex {
    position: vec4<f32>,
    color:    vec4<f32>,
}

struct FurParams {
    time:  f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

// ---- bindings ----

@group(0) @binding(0) var<storage, read>       input_tris:   array<InputTri>;
@group(0) @binding(1) var<storage, read_write> output_verts: array<OutputVertex>;
@group(0) @binding(2) var<uniform>             params:       FurParams;

// ---- main ----

@compute @workgroup_size(25, 1, 1)
fn main(
    @builtin(workgroup_id)        wg:  vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let tri_idx = wg.x;
    let n       = lid.x;   // layer index 0..24

    let tri      = input_tris[tri_idx];
    let out_base = tri_idx * VERTS_PER_TRI + n * 3u;
    let bright   = LAYER_STEP * f32(n + 1u);
    let offset   = LAYER_STEP * f32(n);

    for (var i = 0u; i < 3u; i += 1u) {
        let pos = tri.v[i].position + tri.v[i].normal * offset;
        let col = vec4<f32>(tri.v[i].normal * bright, 1.0);
        output_verts[out_base + i] = OutputVertex(vec4<f32>(pos, 1.0), col);
    }
}

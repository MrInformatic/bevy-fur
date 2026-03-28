// Fur geometry — Approach 3 (gs_3 port)
// Cone to averaged centre point: each edge of the triangle connects to
// a tip at avg_pos + avg_normal * 0.5.
// One workgroup per input triangle, 3 threads (one per edge).
// Each thread writes exactly 3 output vertices.
//
// Thread 0 : (v2, v0, mid_tip)
// Thread 1 : (v0, v1, mid_tip)
// Thread 2 : (v1, v2, mid_tip)

const VERTS_PER_TRI: u32 = 9u;  // 3 threads * 3 verts

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

// ---- helpers ----

fn make_vert(pos: vec3<f32>, nor: vec3<f32>, bright: f32) -> OutputVertex {
    return OutputVertex(vec4<f32>(pos, 1.0), vec4<f32>(nor * bright, 1.0));
}

// ---- main ----

@compute @workgroup_size(3, 1, 1)
fn main(
    @builtin(workgroup_id)        wg:  vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let tri_idx  = wg.x;
    let thread   = lid.x;   // 0, 1, 2
    let tri      = input_tris[tri_idx];
    let out_base = tri_idx * VERTS_PER_TRI + thread * 3u;

    // Averaged centre tip
    let mid_nor = (tri.v[0].normal   + tri.v[1].normal   + tri.v[2].normal)   / 3.0;
    let mid_pos = (tri.v[0].position + tri.v[1].position + tri.v[2].position) / 3.0
                + mid_nor * 0.5;

    // Edge table: (a_idx, b_idx) for edges (2→0), (0→1), (1→2)
    let edge_pairs = array<vec2<u32>, 3>(
        vec2<u32>(2u, 0u),
        vec2<u32>(0u, 1u),
        vec2<u32>(1u, 2u),
    );
    let ai = edge_pairs[thread].x;
    let bi = edge_pairs[thread].y;

    output_verts[out_base + 0u] = make_vert(tri.v[ai].position, tri.v[ai].normal, 0.5);
    output_verts[out_base + 1u] = make_vert(tri.v[bi].position, tri.v[bi].normal, 0.5);
    output_verts[out_base + 2u] = make_vert(mid_pos, mid_nor, 1.0);
}

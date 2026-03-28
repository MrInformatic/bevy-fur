// Fur geometry — Approach 2 (gs_2 port)
// Solid shell: original triangle + double-sided ridge on each edge.
// One workgroup per input triangle, 7 threads.
// Each thread writes exactly 3 output vertices.
//
// Thread 0 : original triangle (v0, v1, v2)
// Threads 1-2 : edge(v2 → v0) forward + reversed
// Threads 3-4 : edge(v0 → v1) forward + reversed
// Threads 5-6 : edge(v1 → v2) forward + reversed
//
// Ridge tip: c = mix(va.pos, vb.pos, 0.5) + mix(va.nor, vb.nor, 0.5) * 0.5

const VERTS_PER_TRI: u32 = 21u;  // 7 threads * 3 verts

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

@compute @workgroup_size(7, 1, 1)
fn main(
    @builtin(workgroup_id)        wg:  vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let tri_idx  = wg.x;
    let thread   = lid.x;
    let tri      = input_tris[tri_idx];
    let out_base = tri_idx * VERTS_PER_TRI + thread * 3u;

    if thread == 0u {
        // Original triangle, flat brightness
        output_verts[out_base + 0u] = make_vert(tri.v[0].position, tri.v[0].normal, 0.5);
        output_verts[out_base + 1u] = make_vert(tri.v[1].position, tri.v[1].normal, 0.5);
        output_verts[out_base + 2u] = make_vert(tri.v[2].position, tri.v[2].normal, 0.5);
        return;
    }

    // Edge table: (a_idx, b_idx) for edges (2→0), (0→1), (1→2)
    let edge_pairs = array<vec2<u32>, 3>(
        vec2<u32>(2u, 0u),
        vec2<u32>(0u, 1u),
        vec2<u32>(1u, 2u),
    );
    let edge = (thread - 1u) / 2u;   // 0, 1, 2
    let flip = ((thread - 1u) & 1u) != 0u;

    let ai = edge_pairs[edge].x;
    let bi = edge_pairs[edge].y;

    let a_pos = tri.v[ai].position;
    let b_pos = tri.v[bi].position;
    let a_nor = tri.v[ai].normal;
    let b_nor = tri.v[bi].normal;

    // Ridge tip: midpoint of edge extruded by midpoint normal
    let c_pos = mix(a_pos, b_pos, 0.5) + mix(a_nor, b_nor, 0.5) * 0.5;
    let c_nor = normalize(mix(a_nor, b_nor, 0.5));

    if !flip {
        output_verts[out_base + 0u] = make_vert(a_pos, a_nor, 0.5);
        output_verts[out_base + 1u] = make_vert(b_pos, b_nor, 0.5);
        output_verts[out_base + 2u] = make_vert(c_pos, c_nor, 1.0);
    } else {
        output_verts[out_base + 0u] = make_vert(b_pos, b_nor, 0.5);
        output_verts[out_base + 1u] = make_vert(a_pos, a_nor, 0.5);
        output_verts[out_base + 2u] = make_vert(c_pos, c_nor, 1.0);
    }
}

// Fur geometry expansion — GPU port of Fur_gs_4.glsl
// One workgroup per input triangle, 93 threads per workgroup.
// 93 = 3 sides × 31 strip-triangles
// Each thread writes 3 output vertices → 279 output verts per input triangle.

const SEGMENTS:      u32 = 16u;
const TRIS_PER_SIDE: u32 = 31u;   // = SEGMENTS*2 + 1 - 2 strip triangles
const VERTS_PER_TRI: u32 = 279u;  // = 3 sides * TRIS_PER_SIDE * 3
const LENGTH:        f32 = 0.5;
const BRIGHTMAX:     f32 = 1.0;
const BRIGHTMIN:     f32 = 0.5;
const TAU:           f32 = 6.28318530718;
const PI_OVER_20:    f32 = 0.15707963268;

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

// Y-axis rotation matrix for segment level n at time t
// Matches CPU: angle = cos((t - phase) * 2π) * (π/20) * i/SEGMENTS
fn rotation_y(n: u32, t: f32) -> mat4x4<f32> {
    let phase = f32(n) / f32(SEGMENTS) * 0.5;
    let angle = cos((t - phase) * TAU) * PI_OVER_20 * f32(n) / f32(SEGMENTS);
    let c = cos(angle);
    let s = sin(angle);
    // column-major
    return mat4x4<f32>(
        vec4<f32>( c,   0.0, s,   0.0),
        vec4<f32>( 0.0, 1.0, 0.0, 0.0),
        vec4<f32>(-s,   0.0, c,   0.0),
        vec4<f32>( 0.0, 0.0, 0.0, 1.0),
    );
}

// Port of EmitSegmetVertex() in Fur_gs_4.glsl
fn segment_vertex(
    base_pos: vec3<f32>,
    base_nor: vec3<f32>,
    mid_pos:  vec3<f32>,
    mid_nor:  vec3<f32>,
    rot:      mat4x4<f32>,
    len_h:    f32,   // = LENGTH * seg / SEGMENTS
    sharp_h:  f32,   // = seg / SEGMENTS
    bright_h: f32,   // brightness for this level
    sintime:  f32,
) -> OutputVertex {
    let extrusion = mid_nor * len_h;
    // GLSL mix(a,b,t) = lerp; CPU: v.position.lerp(mid_pos, sharp_h) = mix(v.pos, mid_pos, sharp_h)
    let sharpness = mix(base_pos, mid_pos, sharp_h);
    let gravity   = vec3<f32>(0.0, sintime * 0.3 * sqrt(sharp_h), 0.0);
    // CPU: mid_normal.lerp(v.normal, sharp_h) = mix(mid_nor, base_nor, sharp_h)
    let n_blend   = mix(mid_nor, base_nor, sharp_h);
    let pos       = (rot * vec4<f32>(sharpness + extrusion - gravity, 1.0)).xyz;
    let col       = vec4<f32>(n_blend * bright_h, 1.0);
    return OutputVertex(vec4<f32>(pos, 1.0), col);
}

// Port of EmitLastSegmetVertex() in Fur_gs_4.glsl
fn tip_vertex(
    mid_pos: vec3<f32>,
    mid_nor: vec3<f32>,
    rot:     mat4x4<f32>,
    sintime: f32,
) -> OutputVertex {
    let extrusion = mid_nor * LENGTH;
    let gravity   = vec3<f32>(0.0, sintime * 0.3, 0.0);
    let pos       = (rot * vec4<f32>(mid_pos + extrusion - gravity, 1.0)).xyz;
    let col       = vec4<f32>(mid_nor * BRIGHTMAX, 1.0);
    return OutputVertex(vec4<f32>(pos, 1.0), col);
}

// Returns strip vertex at index sv (0..=32) for one side.
// Strip has SEGMENTS*2 = 32 alternating va/vb verts + 1 tip.
fn strip_vert(
    sv:       u32,
    va_pos:   vec3<f32>,
    va_nor:   vec3<f32>,
    vb_pos:   vec3<f32>,
    vb_nor:   vec3<f32>,
    mid_pos:  vec3<f32>,
    mid_nor:  vec3<f32>,
    sintime:  f32,
    t:        f32,
) -> OutputVertex {
    if sv == 32u {
        return tip_vertex(mid_pos, mid_nor, rotation_y(SEGMENTS, t), sintime);
    }
    let seg      = sv / 2u;
    let is_b     = (sv & 1u) != 0u;
    let base_pos = select(va_pos, vb_pos, is_b);
    let base_nor = select(va_nor, vb_nor, is_b);
    let len_h    = LENGTH * f32(seg) / f32(SEGMENTS);
    let sharp_h  = f32(seg) / f32(SEGMENTS);
    let bright_h = (BRIGHTMAX - BRIGHTMIN) / f32(SEGMENTS) * f32(seg) + BRIGHTMIN;
    return segment_vertex(
        base_pos, base_nor, mid_pos, mid_nor,
        rotation_y(seg, t), len_h, sharp_h, bright_h, sintime,
    );
}

// ---- main ----

@compute @workgroup_size(93, 1, 1)
fn main(
    @builtin(workgroup_id)        wg:  vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let tri_idx  = wg.x;
    let local_id = lid.x;   // 0..92

    let t       = params.time;
    let sintime = sin(t * TAU);

    let tri     = input_tris[tri_idx];
    let mid_pos = (tri.v[0].position + tri.v[1].position + tri.v[2].position) / 3.0;
    let mid_nor = (tri.v[0].normal   + tri.v[1].normal   + tri.v[2].normal)   / 3.0;

    // Decode which side and which strip-triangle this thread handles
    let side         = local_id / TRIS_PER_SIDE;   // 0, 1, or 2
    let tri_in_strip = local_id % TRIS_PER_SIDE;   // 0..30

    // Side vertex pairs: (0,1), (1,2), (2,0)
    let a_idx  = side;
    let b_idx  = (side + 1u) % 3u;
    let va_pos = tri.v[a_idx].position;
    let va_nor = tri.v[a_idx].normal;
    let vb_pos = tri.v[b_idx].position;
    let vb_nor = tri.v[b_idx].normal;

    // Triangle strip → triangle list winding:
    // even:  strip[i], strip[i+1], strip[i+2]
    // odd:   strip[i+1], strip[i], strip[i+2]
    let i = tri_in_strip;
    var sv0: u32; var sv1: u32; var sv2: u32;
    if (i & 1u) == 0u {
        sv0 = i; sv1 = i + 1u; sv2 = i + 2u;
    } else {
        sv0 = i + 1u; sv1 = i; sv2 = i + 2u;
    }

    let out_base = tri_idx * VERTS_PER_TRI + local_id * 3u;
    output_verts[out_base + 0u] = strip_vert(sv0, va_pos, va_nor, vb_pos, vb_nor, mid_pos, mid_nor, sintime, t);
    output_verts[out_base + 1u] = strip_vert(sv1, va_pos, va_nor, vb_pos, vb_nor, mid_pos, mid_nor, sintime, t);
    output_verts[out_base + 2u] = strip_vert(sv2, va_pos, va_nor, vb_pos, vb_nor, mid_pos, mid_nor, sintime, t);
}

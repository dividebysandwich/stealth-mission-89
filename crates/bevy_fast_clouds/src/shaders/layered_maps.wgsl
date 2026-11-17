// The layered clouds' maps around the eye (render.rs `build_maps`):
// - weather map: the convective clouds drawn as squares, the depth test keeping the nearest
//   cloud by distance over radius: (distance / radius, fill + 2 for a storm cell, base, top);
// - the deck's coverage field (`deck_field`), the application's value noise.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct MapUniform {
    // The map's centre (world x, z), half span (m).
    anchor: vec4<f32>,
    // Field seed, unit (metres).
    field: vec4<f32>,
    cells: vec4<f32>,
    weights: vec4<f32>,
}

struct Instances {
    // Two per cloud: (x, z from the centre, radius, fill + 2 if a storm), (base y, top y).
    c: array<vec4<f32>, 512>,
}

@group(0) @binding(0) var<uniform> map: MapUniform;
@group(0) @binding(1) var<uniform> instances: Instances;

struct QuadOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) rel: vec2<f32>,
    @location(1) @interpolate(flat) instance: u32,
}

const CORNERS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
    vec2(-1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0),
);

// How far a cloud reaches, over its radius: a storm cell's anvil spreads out, fragments drift
// around the cumulus.
fn reach(a: vec4<f32>) -> f32 {
    // Cumulus: room for the ragged fragments around them.
    return select(3.0, 2.2, a.w >= 1.5);
}

@vertex
fn quad(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> QuadOut {
    let a = instances.c[2u * i];
    let p = a.xy + CORNERS[v] * a.z * reach(a);
    var out: QuadOut;
    out.clip = vec4<f32>(p / map.anchor.z, 0.5, 1.0);
    out.rel = p;
    out.instance = i;
    return out;
}

struct QuadFrag {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn quad_fragment(in: QuadOut) -> QuadFrag {
    let a = instances.c[2u * in.instance];
    let b = instances.c[2u * in.instance + 1u];
    let d = length(in.rel - a.xy) / max(a.z, 1.0);
    if (d > reach(a)) { discard; }
    var out: QuadFrag;
    out.color = vec4<f32>(d, a.w, b.x, b.y);
    out.depth = d * 0.2;
    return out;
}

// --- The deck's coverage field (palette.wgsl `stratus_field`, stealth_mission's) -----------

fn value_hash(ix: i32, iy: i32, seed: u32) -> f32 {
    var h = (u32(ix) * 0x8DA6B343u) ^ (u32(iy) * 0xD8163841u) ^ (seed * 0xCB1AB31Fu);
    h ^= h >> 16u;
    h *= 0x7FEB352Du;
    h ^= h >> 15u;
    h *= 0x846CA68Bu;
    h ^= h >> 16u;
    return f32(h >> 8u) / 16777216.0;
}

fn value_noise(p: vec2<f32>, seed: u32) -> f32 {
    let f = floor(p);
    let i = vec2<i32>(f);
    let t = p - f;
    let u = t * t * (3.0 - 2.0 * t);
    let a = value_hash(i.x, i.y, seed);
    let b = value_hash(i.x + 1, i.y, seed);
    let c = value_hash(i.x, i.y + 1, seed);
    let d = value_hash(i.x + 1, i.y + 1, seed);
    let top = a + (b - a) * u.x;
    let bottom = c + (d - c) * u.x;
    return top + (bottom - top) * u.y;
}

@fragment
fn deck_field(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let half = map.anchor.z;
    let world = map.anchor.xy + vec2<f32>((in.uv.x - 0.5) * 2.0 * half, (0.5 - in.uv.y) * 2.0 * half);
    let unit = map.field.y;
    let m = vec2<f32>(world.x / unit, 32768.0 + world.y / unit);
    let seed = u32(map.field.x);
    let f = map.weights.x * value_noise(m / map.cells.x, seed)
        + map.weights.y * value_noise(m / map.cells.y, seed + 1u)
        + map.weights.z * value_noise(m / map.cells.z, seed + 2u);
    return vec4<f32>(f, 0.0, 0.0, 1.0);
}

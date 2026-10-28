// A landing light's beam (landing_lights.rs) where the local lights are off: a closed cone from
// the lamp, additive, drawn where the view ray leaves it with the haze it crossed in it, which
// fades towards the beam's edge and far end; the lamp glows. A searchlight's beam (barrage.rs,
// negative intensity) is too long to march: it is lit by how much of the view ray lies in the
// cone where the ray passes closest to its axis.

#import bevy_pbr::{
    mesh_functions,
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
}

struct BeamMaterial {
    // x: intensity 0..1 (negative: a searchlight's, -x), y: tangent of the half angle, z: the
    // lamp's radius, w: length (m).
    params: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: BeamMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    // Lamp frame, the beam along -z.
    @location(0) position: vec3<f32>,
    // x: 0 (lamp) .. 1 (far end).
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world: vec3<f32>,
    // The lamp and the beam's direction (world).
    @location(1) @interpolate(flat) apex: vec3<f32>,
    @location(2) @interpolate(flat) axis: vec3<f32>,
};

@vertex
fn vertex(v: Vertex) -> VertexOutput {
    let world_from_local = mesh_functions::get_world_from_local(v.instance_index);
    let world = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(v.position, 1.0)).xyz;
    var out: VertexOutput;
    out.clip_position = position_world_to_clip(world);
    out.world = world;
    out.apex = world_from_local[3].xyz;
    out.axis = -normalize(world_from_local[2].xyz);
    return out;
}

const COLOUR: vec3<f32> = vec3<f32>(1.0, 0.95, 0.85);
// Samples along the view ray through the beam.
const STEPS: i32 = 16;

// Where world point q is along the beam (m) and off its axis (share of its radius there).
fn beam_coords(q: vec3<f32>, apex: vec3<f32>, axis: vec3<f32>, p: vec4<f32>) -> vec2<f32> {
    let along = dot(q - apex, axis);
    return vec2<f32>(along, length(q - apex - axis * along) / (p.z + p.y * max(along, 0.0)));
}

// The beam's density at q: fading towards its edge and its far end.
fn density(q: vec3<f32>, apex: vec3<f32>, axis: vec3<f32>, p: vec4<f32>) -> f32 {
    let c = beam_coords(q, apex, axis, p);
    if (c.x < 0.0 || c.x > p.w || c.y > 1.0) {
        return 0.0;
    }
    return pow(1.0 - c.x / p.w, 1.5) * (1.0 - c.y * c.y);
}

// A searchlight's light along the view ray (eye, d) up to `exit`: the ray's length inside the
// cone where it passes closest to the axis, fading towards the beam's edge and far end; and the
// lamp's glow.
fn searchlight(eye: vec3<f32>, d: vec3<f32>, exit: f32, apex: vec3<f32>, axis: vec3<f32>, p: vec4<f32>) -> f32 {
    let w = eye - apex;
    let b = dot(d, axis);
    let across = max(1.0 - b * b, 1e-4);
    let t = clamp((dot(axis, w) - b * dot(d, w)) / across, 0.0, p.w);
    let s = clamp(dot(apex + axis * t - eye, d), 0.0, exit);
    let r = length(eye + d * s - apex - axis * t);
    let radius = p.z + p.y * t;
    let inside = max(1.0 - r * r / (radius * radius), 0.0);
    let path = min(2.0 * radius * sqrt(inside) / max(sqrt(across), 0.05), p.w);
    let lamp = length(eye + d * clamp(dot(apex - eye, d), 0.0, exit) - apex) / (2.0 * p.z);
    return 0.012 * pow(1.0 - t / p.w, 1.5) * inside * path + 0.8 * exp(-lamp * lamp);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let p = material.params;
    let eye = view.world_position;
    let d = normalize(in.world - eye);
    // Only where the view ray leaves the beam (its back faces), so each ray adds it up once.
    if (front) {
        discard;
    }
    let exit = length(in.world - eye);
    if (p.x < 0.0) {
        let a = -p.x * searchlight(eye, d, exit, in.apex, in.axis, p);
        if (a <= 0.003) {
            discard;
        }
        return vec4<f32>(COLOUR, min(a, 1.0));
    }
    // March back from there across the longest chord the beam has (from the eye when in it).
    let chord = length(vec2<f32>(p.w, 2.0 * (p.z + p.y * p.w)));
    let start = max(exit - chord, 0.0);
    let ds = (exit - start) / f32(STEPS);
    var sum = 0.0;
    for (var i = 0; i < STEPS; i++) {
        sum += density(eye + d * (start + (f32(i) + 0.5) * ds), in.apex, in.axis, p);
    }
    // The lamp's glow where the ray passes it.
    let s = clamp(dot(in.apex - eye, d), start, exit);
    let r = length(eye + d * s - in.apex);
    let a = p.x * (0.03 * sum * ds + 0.8 * exp(-r * r / 0.5));
    if (a <= 0.003) {
        discard;
    }
    return vec4<f32>(COLOUR, min(a, 1.0));
}

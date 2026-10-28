// A sunbeam (sunbeams.rs): a shaft of sunlight through a gap of the deck, a closed cylinder from
// the gap along the sunlight to the ground (its local -z; radius and length are the transform's
// scale). Additive: the light it scatters along the view ray's chord through it, most across
// its axis, fading out at the deck, towards the ground and into the haze, brightest looking
// towards the sun.

#import bevy_pbr::{
    mesh_functions,
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
}

struct SunbeamMaterial {
    // x: brightness 0..1, y: haze distance (m).
    params: vec4<f32>,
    // Linear colour of the sunlight.
    colour: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: SunbeamMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world: vec3<f32>,
    // Where it hangs from the deck, its direction (world), radius and length (m).
    @location(1) @interpolate(flat) apex: vec3<f32>,
    @location(2) @interpolate(flat) axis: vec3<f32>,
    @location(3) @interpolate(flat) size: vec2<f32>,
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
    out.size = vec2<f32>(length(world_from_local[0].xyz), length(world_from_local[2].xyz));
    return out;
}

// Light scattered per metre of the beam crossed.
const DENSITY: f32 = 1.0 / 600.0;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let eye = view.world_position;
    let to = in.world - eye;
    let dist = length(to);
    let d = to / max(dist, 1e-3);
    let r = in.size.x;
    let len = in.size.y;
    // The eye in it: the faces where the view ray leaves it; else where it enters (so the
    // ground in front hides what is behind it). Each ray adds the beam up once.
    let rel = eye - in.apex;
    let along_eye = dot(rel, in.axis);
    let w = rel - in.axis * along_eye;
    let inside = dot(w, w) < r * r && along_eye > 0.0 && along_eye < len;
    if (front == inside) {
        discard;
    }
    // The view ray's chord through the cylinder, between its ends.
    let da = dot(d, in.axis);
    let dp = d - in.axis * da;
    let a = dot(dp, dp);
    let b = dot(w, dp);
    let c = dot(w, w) - r * r;
    var t = vec2<f32>(0.0, 1e9);
    if (a > 1e-6) {
        let disc = b * b - a * c;
        if (disc <= 0.0) {
            discard;
        }
        let s = sqrt(disc);
        t = vec2<f32>((-b - s) / a, (-b + s) / a);
    } else if (c > 0.0) {
        discard;
    }
    if (abs(da) > 1e-5) {
        let e = vec2<f32>(-along_eye, len - along_eye) / da;
        t = vec2<f32>(max(t.x, min(e.x, e.y)), min(t.y, max(e.x, e.y)));
    }
    t.x = max(t.x, 0.0);
    if (inside) {
        t.y = min(t.y, dist);
    }
    if (t.y <= t.x) {
        discard;
    }
    // Thickest across the axis: the ray's distance from it where it passes closest.
    let closest = clamp(-b / max(a, 1e-6), t.x, t.y);
    let off = length(w + dp * closest) / r;
    let across = max(1.0 - off * off, 0.0);
    let mid = eye + d * (0.5 * (t.x + t.y));
    let along = dot(mid - in.apex, in.axis);
    let fall = smoothstep(0.0, 0.06 * len, along) * exp(-1.2 * along / len)
        * (1.0 - smoothstep(0.8 * len, len, along));
    // Forward scattering: towards the sun (up the beam) brighter.
    let towards = 0.35 + 0.65 * pow(max(dot(d, -in.axis), 0.0), 4.0);
    let haze = exp(-length(mid - eye) / material.params.y);
    let light = 1.0 - exp(-(t.y - t.x) * across * DENSITY);
    let alpha = material.params.x * light * fall * towards * haze;
    if (alpha < 0.002) {
        discard;
    }
    return vec4<f32>(material.colour.rgb, min(alpha, 1.0));
}

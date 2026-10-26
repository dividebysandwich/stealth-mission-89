// The flown jet's afterburner plume (afterburner_flame.rs): nested cones behind the nozzle and
// a disc over its exit, additive. Brightest where the view looks through the cone's middle,
// fading towards its silhouette and its tip: a yellow-white core with shock diamonds, a sheath
// blue-violet at the nozzle turning orange, and the nozzle's glowing exit.

#import bevy_pbr::{
    mesh_functions,
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
}

struct FlameMaterial {
    // x: intensity 0..1, y: seconds (flicker), z: shock diamonds 0..1.
    params: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: FlameMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    // Aircraft frame, the plume along +z.
    @location(0) position: vec3<f32>,
    // x: 0 (nozzle) .. 1 (tip), y: layer (0 core, 1 sheath, 2 nozzle exit).
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertex(v: Vertex) -> VertexOutput {
    let world_from_local = mesh_functions::get_world_from_local(v.instance_index);
    let world = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(v.position, 1.0)).xyz;
    // The cones' normals point out from the axis, the disc's along it.
    var n = vec3<f32>(v.position.xy, 0.0);
    if (v.uv.y > 1.5 || dot(n, n) < 1e-8) {
        n = vec3<f32>(0.0, 0.0, 1.0);
    }
    var out: VertexOutput;
    out.clip_position = position_world_to_clip(world);
    out.world = world;
    out.normal = (world_from_local * vec4<f32>(n, 0.0)).xyz;
    out.uv = v.uv;
    return out;
}

const TAU: f32 = 6.28318530718;
const CORE_HOT: vec3<f32> = vec3<f32>(1.0, 0.85, 0.55);
const CORE_TIP: vec3<f32> = vec3<f32>(1.0, 0.45, 0.12);
const SHEATH_ROOT: vec3<f32> = vec3<f32>(0.4, 0.32, 1.0);
const SHEATH: vec3<f32> = vec3<f32>(1.0, 0.42, 0.12);
const EXIT: vec3<f32> = vec3<f32>(1.0, 0.62, 0.28);
// Shock diamonds along the core (per its length).
const DIAMONDS: f32 = 3.5;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = clamp(in.uv.x, 0.0, 1.0);
    let layer = in.uv.y;
    let p = material.params;
    let facing = abs(dot(normalize(in.normal), normalize(view.world_position - in.world)));
    let flicker = 0.85 + 0.1 * sin(p.y * 47.0 + t * 9.0) + 0.05 * sin(p.y * 83.0 - t * 21.0);
    var rgb: vec3<f32>;
    var a: f32;
    if (layer > 1.5) {
        rgb = EXIT * 1.8;
        a = 0.9;
    } else if (layer > 0.5) {
        rgb = mix(SHEATH_ROOT, SHEATH, smoothstep(0.1, 0.35, t));
        a = 0.35 * pow(facing, 1.5) * smoothstep(0.0, 0.08, t) * pow(1.0 - t, 1.5) * flicker;
    } else {
        let d = pow(0.5 + 0.5 * cos(TAU * t * DIAMONDS), 8.0) * (1.0 - t);
        rgb = mix(CORE_HOT * 1.6, CORE_TIP, smoothstep(0.1, 0.9, t));
        a = 0.7 * facing * pow(1.0 - t, 1.2) * flicker * (1.0 - 0.5 * p.z + 1.5 * p.z * d);
    }
    a *= p.x;
    if (a <= 0.003) {
        discard;
    }
    return vec4<f32>(rgb, a);
}

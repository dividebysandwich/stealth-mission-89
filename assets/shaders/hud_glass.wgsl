// HUD combiner glass: the HUD canvas collimated to infinity. Each fragment looks up the HUD
// pixel in the direction it is seen from the eye (cockpit frame = the glass mesh's local
// frame), so the symbology stays on the outside world wherever the head is and only shows
// through the glass.

#import bevy_pbr::{
    mesh_functions,
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
}

// Glass tint where the HUD is blank (rgb, alpha).
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> tint: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var hud_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var hud_sampler: sampler;
// Canvas pixel of the boresight (xy), canvas pixels per unit of the tangent plane (zw).
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> proj: vec4<f32>;
// x: 1 = modern HUD (monochrome ink drawn in the HUD colour), y: brightness, z: the colour
// (`HudColour`: green, amber, cyan, white).
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> style: vec4<f32>;

// The modern HUD's colours (linear), indexed by style.z.
const HUD_COLOURS: array<vec3<f32>, 4> = array<vec3<f32>, 4>(
    vec3<f32>(0.18, 1.0, 0.32),
    vec3<f32>(1.0, 0.55, 0.08),
    vec3<f32>(0.2, 0.85, 1.0),
    vec3<f32>(0.92, 0.95, 0.92),
);

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // Eye to vertex in the cockpit frame (linear, so it interpolates exactly).
    @location(0) dir: vec3<f32>,
};

@vertex
fn vertex(v: Vertex) -> VertexOutput {
    let world_from_local = mesh_functions::get_world_from_local(v.instance_index);
    let world = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(v.position, 1.0));
    let eye = (mesh_functions::get_local_from_world(v.instance_index) * vec4<f32>(view.world_position, 1.0)).xyz;
    var out: VertexOutput;
    out.clip_position = position_world_to_clip(world.xyz);
    out.dir = v.position - eye;
    return out;
}

// Rotated grid taps across a pixel (`canvas`).
const GRID: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(0.125, 0.375),
    vec2<f32>(0.375, -0.125),
    vec2<f32>(-0.125, -0.375),
    vec2<f32>(-0.375, 0.125),
);

// The canvas over this pixel: four taps on a rotated grid across the pixel's footprint (in a
// headset about a texel), averaged. The canvas is inked or not, so one tap a pixel shows its
// texels' stair steps on every slanted line, and MSAA doesn't help (one shading sample a
// pixel). Colour is weighted by alpha.
fn canvas(p: vec2<f32>, size: vec2<f32>) -> vec4<f32> {
    let px = dpdx(p);
    let py = dpdy(p);
    var sum = vec4<f32>(0.0);
    for (var i = 0; i < 4; i++) {
        let o = GRID[i].x * px + GRID[i].y * py;
        let c = textureSampleLevel(hud_texture, hud_sampler, (p + o) / size, 0.0);
        sum += vec4<f32>(c.rgb * c.a, c.a);
    }
    return sum * 0.25;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let z = max(-in.dir.z, 1e-4);
    let p = vec2<f32>(proj.x + proj.z * in.dir.x / z, proj.y - proj.w * in.dir.y / z);
    let size = vec2<f32>(textureDimensions(hud_texture));
    let inside = p.x >= 0.0 && p.y >= 0.0 && p.x < size.x && p.y < size.y;
    let c = canvas(p, size);
    if style.x > 0.5 {
        // Modern: ink coverage (a one-channel texture, linear filtered: in r, alpha 1).
        let core = c.r;
        let a = select(0.0, clamp(core * 1.2, 0.0, 1.0), inside);
        if a <= 0.01 {
            if tint.a <= 0.0 {
                discard;
            }
            return tint;
        }
        var colours = HUD_COLOURS;
        let rgb = colours[clamp(u32(style.z), 0u, 3u)] * style.y * (0.75 + 0.6 * core);
        return vec4<f32>(rgb, a);
    }
    // The ink over the tint over what is behind.
    let a = select(0.0, c.a, inside);
    if a <= 0.01 {
        if tint.a <= 0.0 {
            discard;
        }
        return tint;
    }
    let out_a = a + (1.0 - a) * tint.a;
    return vec4<f32>((c.rgb + (1.0 - a) * tint.a * tint.rgb) / out_a, out_a);
}

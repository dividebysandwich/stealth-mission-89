// The layered clouds' temporal accumulation: this frame's march blended with the history,
// reprojected through the previous frame's projection (and the floating origin's move), its
// colour clamped to this frame's neighbourhood.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_fast_clouds::layered::{LayeredGlobals, LayeredView}

@group(0) @binding(0) var current: texture_2d<f32>;
@group(0) @binding(1) var depths: texture_2d<f32>;
@group(0) @binding(2) var history: texture_2d<f32>;
@group(0) @binding(3) var history_sampler: sampler;
@group(0) @binding(4) var<uniform> view: View;
@group(0) @binding(5) var<uniform> layered: LayeredView;
@group(0) @binding(6) var<uniform> globals: LayeredGlobals;

// How much of the history is kept.
const FEEDBACK: f32 = 0.88;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(in.position.xy);
    let c = textureLoad(current, pixel, 0);
    if (layered.shift.w == 0.0) {
        return c;
    }
    let res = globals.params.y;
    let full = (vec2<f32>(pixel) + 0.5) * res;
    let ndc = vec2<f32>(full.x / view.viewport.z * 2.0 - 1.0, 1.0 - full.y / view.viewport.w * 2.0);
    // The ray in view space, then turned into the world: the near plane's point minus the
    // camera would lose the direction to rounding with the camera far from the render origin.
    let v = view.view_from_clip * vec4<f32>(ndc, 1.0, 1.0);
    let rd = normalize((view.world_from_view * vec4<f32>(v.xyz / v.w, 0.0)).xyz);
    let cam = view.world_position;
    let scale = length(view.world_from_view[2].xyz);
    let forward = -view.world_from_view[2].xyz / scale;
    let z = textureLoad(depths, pixel, 0).y;
    let t = z * scale / max(dot(rd, forward), 1e-4);
    let p = cam + rd * t + layered.shift.xyz;
    let clip = layered.prev_clip_from_world * vec4<f32>(p, 1.0);
    if (clip.w <= 0.0) {
        return c;
    }
    let prev = clip.xy / clip.w;
    let uv_view = vec2<f32>(prev.x * 0.5 + 0.5, 0.5 - prev.y * 0.5);
    if (any(uv_view < vec2<f32>(0.0)) || any(uv_view > vec2<f32>(1.0))) {
        return c;
    }
    let size = vec2<f32>(textureDimensions(history));
    let uv = uv_view * (view.viewport.zw / res) / size;
    var h = textureSampleLevel(history, history_sampler, uv, 0.0);
    // Neighbourhood clamp: no ghosts of clouds that moved on.
    var lo = c;
    var hi = c;
    let last = vec2<i32>(textureDimensions(current)) - 1;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let n = textureLoad(current, clamp(pixel + vec2<i32>(x, y), vec2<i32>(0), last), 0);
            lo = min(lo, n);
            hi = max(hi, n);
        }
    }
    h = clamp(h, lo, hi);
    return mix(c, h, FEEDBACK);
}

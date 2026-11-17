// The layered clouds over the view: upsampled (bilinear), kept off geometry nearer than where
// the clouds start (`fragment`); then where they are opaque, their depth into the view's depth
// buffer (`depth`), so transparent geometry behind them is hidden.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_fast_clouds::layered::{LayeredGlobals, NO_CLOUD}

@group(0) @binding(0) var clouds: texture_2d<f32>;
@group(0) @binding(1) var clouds_sampler: sampler;
@group(0) @binding(2) var depths: texture_2d<f32>;
@group(0) @binding(3) var<uniform> view: View;
@group(0) @binding(4) var<uniform> globals: LayeredGlobals;
#ifdef MULTISAMPLED
@group(0) @binding(5) var depth_texture: texture_depth_multisampled_2d;
#else ifdef WEBGL2
@group(0) @binding(5) var depth_texture: texture_2d<f32>;
#else
@group(0) @binding(5) var depth_texture: texture_depth_2d;
#endif
@group(0) @binding(6) var depth_sampler: sampler;

fn scene_depth(p: vec2<i32>) -> f32 {
#ifdef WEBGL2
    let size = vec2<f32>(textureDimensions(depth_texture));
    return textureSampleLevel(depth_texture, depth_sampler, (vec2<f32>(p) + 0.5) / size, 0.0).x;
#else
    return textureLoad(depth_texture, p, 0);
#endif
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let res = globals.params.y;
    let local = in.position.xy - view.viewport.xy;
    let texel = vec2<i32>(local / res);
    let first = textureLoad(depths, texel, 0).x;
    let d = scene_depth(vec2<i32>(in.position.xy));
    let near = view.clip_from_view[3][2];
    let scene_z = select(1e30, near / d, d > 0.0);
    if (scene_z < first * 0.97) {
        discard;
    }
    let size = vec2<f32>(textureDimensions(clouds));
    let c = textureSampleLevel(clouds, clouds_sampler, local / res / size, 0.0);
    if (c.a <= 0.0) {
        discard;
    }
    return c;
}

struct DepthOut {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

// Reverse infinite z: depth = near / view z. Colour is not written.
@fragment
fn depth(in: FullscreenVertexOutput) -> DepthOut {
    let res = globals.params.y;
    let texel = vec2<i32>((in.position.xy - view.viewport.xy) / res);
    let z = textureLoad(depths, texel, 0).z;
    if (z >= NO_CLOUD) { discard; }
    let near = view.clip_from_view[3][2];
    var out: DepthOut;
    out.color = vec4<f32>(0.0);
    out.depth = near / max(z, near);
    return out;
}

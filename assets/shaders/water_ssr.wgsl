// Screen space reflections on calm water (water_ssr.rs): after the main passes, each pixel of
// water the palette shader marked (its alpha: 1 - the share of the picture the reflection
// takes) sends its view ray, mirrored off the water and wobbled by fine ripples, out across the
// depth buffer; what it meets there (a ship, the shore, an aircraft) shows in the water instead
// of the sky the palette shader mirrored. Where it meets nothing on the screen the water keeps
// its own reflection. Every pixel leaves with alpha 1.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// Positions are relative to the eye (the world matrices lose a pixel's ray to rounding
// kilometres from the floating origin).
struct Params {
    clip_from_eye: mat4x4<f32>,
    eye_from_clip: mat4x4<f32>,
    // The eye (xyz), the near plane (w); the view's forward direction.
    eye: vec4<f32>,
    forward: vec4<f32>,
    // The floating origin's x, z (metres, wrapped), the clock (s), the picture's width.
    ripple: vec4<f32>,
    // The picture's size in pixels (xy), the projection's y scale (z), the sea's level (w).
    size: vec4<f32>,
};

@group(0) @binding(0) var colour: texture_2d<f32>;
#ifdef MULTISAMPLED
@group(0) @binding(1) var depth: texture_depth_multisampled_2d;
#else
@group(0) @binding(1) var depth: texture_depth_2d;
#endif
@group(0) @binding(2) var<uniform> params: Params;

// March steps (spaced evenly in the log of the distance from the first to `REACH` metres: the
// hills on the horizon mirror in water only a few hundred metres off), the first step's share
// of the eye's distance to the water, and the halving steps at a hit.
const STEPS: i32 = 64;
const FIRST: f32 = 0.01;
const REACH: f32 = 40000.0;
const REFINE: i32 = 7;
// A hit is a crossing at most this many halved steps (the step over 2^REFINE) and this share of
// the distance behind what the picture shows.
const HIT_STEPS: f32 = 4.0 / 128.0;
const HIT_SHARE: f32 = 0.004;
// The ripples' slope (calm water's catspaws: the wobble of what it mirrors).
const RIPPLE: f32 = 0.035;
// A reflection darkens a little in the water.
const TINT: vec3<f32> = vec3<f32>(0.86, 0.9, 0.92);

// The distance along the view of a (reversed, infinite) depth.
fn view_depth(d: f32) -> f32 {
    return params.eye.w / d;
}

fn depth_at(px: vec2<i32>) -> f32 {
#ifdef MULTISAMPLED
    return textureLoad(depth, px, 0);
#else
    return textureLoad(depth, px, 0);
#endif
}

fn value_hash(ix: i32, iy: i32, seed: u32) -> f32 {
    var h = u32(ix) * 374761393u + u32(iy) * 668265263u + seed * 2246822519u;
    h = (h ^ (h >> 13u)) * 1274126177u;
    return f32(h ^ (h >> 16u)) / 4294967295.0;
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
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// The slope of noise of `cell` metres at p, drifting.
fn slope(p: vec2<f32>, cell: f32, drift: vec2<f32>, seed: u32) -> vec2<f32> {
    let q = p / cell + drift;
    let e = 0.15;
    let n0 = value_noise(q, seed);
    return vec2<f32>(value_noise(q + vec2<f32>(e, 0.0), seed) - n0, value_noise(q + vec2<f32>(0.0, e), seed) - n0) / (e * cell);
}

// A ripple of `cell` metres where a pixel covers `foot` metres of water: gone once it is under
// a pixel or two (else it flickers from pixel to pixel), the water there as flat as on average.
fn shown(cell: f32, foot: f32) -> f32 {
    return 1.0 - smoothstep(0.1 * cell, 0.5 * cell, foot);
}

// The water's normal at p (metres, from the floating origin): calm, a few drifting ripples.
fn ripple_normal(p: vec2<f32>, t: f32, foot: f32) -> vec3<f32> {
    var s = slope(p, 0.8, vec2<f32>(t * 0.21, t * 0.13), 3u) * 0.45 * shown(0.8, foot)
        + slope(p, 2.2, vec2<f32>(-t * 0.07, t * 0.11), 5u) * 0.8 * shown(2.2, foot)
        + slope(p, 6.0, vec2<f32>(t * 0.03, -t * 0.04), 7u) * 1.4 * shown(6.0, foot);
    s *= RIPPLE;
    return normalize(vec3<f32>(-s.x, 1.0, -s.y));
}

fn to_uv(clip: vec4<f32>) -> vec3<f32> {
    let n = clip.xyz / clip.w;
    return vec3<f32>(n.x * 0.5 + 0.5, 0.5 - n.y * 0.5, n.z);
}

// Halving between the last step in front (`lo`) and one behind what the picture shows (`hi`):
// where the ray meets it, if it does. A ray that went by behind a thin thing (a tree, a mast
// kilometres off, where a step is hundreds of metres long) is still far behind it at the
// crossing: no hit (-1), the march goes on; else smeared down the water, each tree on a far
// shore mirrored as a long streaky block.
fn crossing(start: vec3<f32>, r: vec3<f32>, lo_in: f32, hi_in: f32, size: vec2<f32>) -> f32 {
    var lo = lo_in;
    var hi = hi_in;
    for (var i = 0; i < REFINE; i++) {
        let mid = 0.5 * (lo + hi);
        let cq = params.clip_from_eye * vec4<f32>(start + r * mid, 1.0);
        let uv = to_uv(cq);
        let sd = depth_at(vec2<i32>(clamp(uv.xy, vec2<f32>(0.0), vec2<f32>(0.9999)) * size));
        if (sd > 0.0 && uv.z < sd) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let cq = params.clip_from_eye * vec4<f32>(start + r * hi, 1.0);
    let uv = to_uv(cq);
    let sd = depth_at(vec2<i32>(clamp(uv.xy, vec2<f32>(0.0), vec2<f32>(0.9999)) * size));
    if (sd <= 0.0) {
        return -1.0;
    }
    // How far behind it the ray is there, against the halved step's length (and a share of
    // the distance: a pixel's depth spans that much far off).
    let behind = cq.w - view_depth(sd);
    if (behind > max(2.0, (hi_in - lo_in) * HIT_STEPS + cq.w * HIT_SHARE)) {
        return -1.0;
    }
    return hi;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let px = vec2<i32>(in.position.xy);
    let c = textureLoad(colour, px, 0);
    let weight = 1.0 - c.a;
    if (weight <= 0.004) {
        return vec4<f32>(c.rgb, 1.0);
    }
    let d0 = depth_at(px);
    if (d0 <= 0.0) {
        return vec4<f32>(c.rgb, 1.0);
    }
    let size = params.size.xy;
    let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0);
    // The point (from the eye) from its distance along the view (`view_depth`) on the pixel's
    // ray: the inverse projection alone is metres off kilometres out.
    let on_near = params.eye_from_clip * vec4<f32>(ndc, 1.0, 1.0);
    let ray = normalize(on_near.xyz / on_near.w);
    let p = ray * (view_depth(d0) / max(dot(ray, params.forward.xyz), 1e-4));
    // Only the sea (what else leaves its alpha under 1, cloud sheets cut out by their coverage,
    // is far above it).
    if (abs(p.y + params.eye.y - params.size.w) > 2.0) {
        return vec4<f32>(c.rgb, 1.0);
    }
    let near = params.eye.w;
    let v = normalize(p);
    let dist = length(p);
    // The water a pixel covers (m), along the view (the longer way at a flat angle).
    let foot = dist * 2.0 / (params.size.z * size.y) / max(-v.y, 0.02);
    let n = ripple_normal(p.xz + params.eye.xz + params.ripple.xy, params.ripple.z, foot);
    var r = reflect(v, n);
    r.y = max(r.y, 0.002);
    // Off the water a little (its own pixels never meet the ray).
    let start = p + vec3<f32>(0.0, 0.05, 0.0);
    let first = max(dist * FIRST, 0.25);
    let grow = pow(REACH / first, 1.0 / f32(STEPS));
    var s = first;
    var last = 0.0;
    var hit = -1.0;
    for (var i = 0; i < STEPS; i++) {
        let q = start + r * s;
        let cq = params.clip_from_eye * vec4<f32>(q, 1.0);
        if (cq.w <= near) {
            break;
        }
        let uv = to_uv(cq);
        if (any(uv.xy < vec2<f32>(0.0)) || any(uv.xy > vec2<f32>(1.0))) {
            break;
        }
        let sd = depth_at(vec2<i32>(uv.xy * size));
        // Behind what the picture shows there (reversed depth: smaller is farther): the ray
        // crossed it between the last step and this one, or went by behind it.
        if (sd > 0.0 && uv.z < sd) {
            let met = crossing(start, r, last, s, size);
            if (met >= 0.0) {
                hit = met;
                break;
            }
        }
        last = s;
        s *= grow;
    }
    if (hit < 0.0) {
        // Nothing near: what the picture shows far off in the mirrored direction, if it is the
        // background there (the sky, the hills on the horizon, the clouds) or past the reach.
        let inf = params.clip_from_eye * vec4<f32>(r, 0.0);
        if (inf.w <= 0.0) {
            return vec4<f32>(c.rgb, 1.0);
        }
        let uv = vec2<f32>(inf.x / inf.w * 0.5 + 0.5, 0.5 - inf.y / inf.w * 0.5);
        if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
            return vec4<f32>(c.rgb, 1.0);
        }
        let fp = vec2<i32>(clamp(uv, vec2<f32>(0.0), vec2<f32>(0.9999)) * size);
        // (The sky dome and the far scenery stand beyond the march's reach.)
        let fd = depth_at(fp);
        if (fd > 0.0 && view_depth(fd) < REACH) {
            return vec4<f32>(c.rgb, 1.0);
        }
        let far = textureLoad(colour, fp, 0);
        let edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
        return vec4<f32>(mix(c.rgb, far.rgb * TINT, weight * smoothstep(0.0, 0.08, edge)), 1.0);
    }
    let uv = to_uv(params.clip_from_eye * vec4<f32>(start + r * hit, 1.0));
    let hp = vec2<i32>(clamp(uv.xy, vec2<f32>(0.0), vec2<f32>(0.9999)) * size);
    let h = textureLoad(colour, hp, 0);
    // Water in water: nothing to show (the ray went under a wave).
    if (h.a < 0.99) {
        return vec4<f32>(c.rgb, 1.0);
    }
    // Faded out towards the picture's edges (where the rays leave it) and with the ray's
    // length.
    let edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    let fade = smoothstep(0.0, 0.08, edge);
    return vec4<f32>(mix(c.rgb, h.rgb * TINT, weight * fade), 1.0);
}

// The layered clouds' march (layered/mod.rs): one fullscreen pass per view at a reduced
// resolution (RES_SCALE view pixels per marched pixel along each axis) through the deck and
// the convective clouds' band. MULTISAMPLED / WEBGL2 read the scene depth as the hull march
// does (wavelet_cloud.wgsl).
//
// Out: premultiplied colour and coverage, and depths (view z, view units): where the clouds
// start (x), the coverage-weighted mean (y, reprojection) and where they turn opaque (z).

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_fast_clouds::layered::{LayeredGlobals, NO_CLOUD}

#ifdef MULTISAMPLED
@group(0) @binding(0) var depth_texture: texture_depth_multisampled_2d;
#else ifdef WEBGL2
@group(0) @binding(0) var depth_texture: texture_2d<f32>;
#else
@group(0) @binding(0) var depth_texture: texture_depth_2d;
#endif
@group(0) @binding(1) var<uniform> view: View;
@group(0) @binding(2) var<uniform> globals: LayeredGlobals;
@group(0) @binding(3) var weather_map: texture_2d<f32>;
@group(0) @binding(4) var deck_map: texture_2d<f32>;
@group(0) @binding(5) var map_sampler: sampler;
@group(0) @binding(6) var shape_noise: texture_3d<f32>;
@group(0) @binding(7) var detail_noise: texture_3d<f32>;
@group(0) @binding(8) var noise_sampler: sampler;
@group(0) @binding(9) var depth_sampler: sampler;
@group(0) @binding(10) var coverage_map: texture_2d<f32>;

const RES_SCALE: i32 = #{RES_SCALE};
const MAX_STEPS: i32 = #{MARCH_STEPS};
const PI: f32 = 3.141592653589793;

// Extinction per metre at density 1.
const SIGMA: f32 = 0.045;
// Shortest primary step (m): thin stretches take fewer steps.
const MIN_STEP: f32 = 30.0;
// The convective clouds are marched this far (m; inside the weather map), the deck this far
// before its far part is drawn as one slab.
const BAND_RANGE: f32 = 44000.0;
const DECK_RANGE: f32 = 40000.0;
// Scene depth beyond this (m) is the sky's background, not geometry.
const SKY_DEPTH: f32 = 400000.0;
// Noise tiles (m).
const NOISE_PERIOD: f32 = 24000.0;
const CUMULUS_TILE: f32 = 3000.0;
const STORM_TILE: f32 = 6000.0;
const DECK_TILE: f32 = 8000.0;
const DETAIL_TILE: f32 = 800.0;

fn sat(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

fn remap(v: f32, lo: f32, hi: f32) -> f32 {
    return (v - lo) / max(hi - lo, 1e-4);
}

fn depth(p: vec2<i32>) -> f32 {
#ifdef WEBGL2
    let size = vec2<f32>(textureDimensions(depth_texture));
    return textureSampleLevel(depth_texture, depth_sampler, (vec2<f32>(p) + 0.5) / size, 0.0).x;
#else
    return textureLoad(depth_texture, p, 0);
#endif
}

// The farthest scene depth of a marched pixel's block (the composite keeps the clouds off
// nearer geometry, by where they start).
fn scene_depth_at(pixel: vec2<i32>) -> f32 {
    let base = vec2<i32>(view.viewport.xy);
    if (RES_SCALE == 1) {
        return depth(base + pixel);
    }
    let last = vec2<i32>(textureDimensions(depth_texture)) - 1;
    let p = base + pixel * RES_SCALE;
    let q = min(p + (RES_SCALE - 1), last);
    return min(
        min(depth(min(p, last)), depth(vec2(q.x, p.y))),
        min(depth(vec2(p.x, q.y)), depth(q)),
    );
}

fn ramp(k: f32) -> vec3<f32> {
    let f = clamp(k, 0.0, 15.0);
    let i = u32(floor(f));
    return mix(globals.ramp[i].rgb, globals.ramp[min(i + 1u, 15u)].rgb, fract(f));
}

// Interleaved gradient noise, moving each frame: the march's start offset.
fn ign(p: vec2<f32>) -> f32 {
    let q = p + 5.588238 * f32(u32(globals.params.x) % 64u);
    return fract(52.9829189 * fract(dot(q, vec2<f32>(0.06711056, 0.00583715))));
}

// Henyey-Greenstein, 1 for isotropic scattering.
fn hg(c: f32, g: f32) -> f32 {
    let g2 = g * g;
    return (1.0 - g2) / max(pow(1.0 + g2 - 2.0 * g * c, 1.5), 1e-3);
}

fn phase(c: f32, k: f32) -> f32 {
    return mix(hg(c, 0.8 * k), hg(c, -0.3 * k), 0.5);
}

// Noise position of a render position.
fn noise_pos(p: vec3<f32>) -> vec3<f32> {
    let x = p.x + globals.noise.x;
    let z = p.z + globals.noise.y;
    return vec3<f32>(x - NOISE_PERIOD * floor(x / NOISE_PERIOD), p.y, z - NOISE_PERIOD * floor(z / NOISE_PERIOD));
}

fn map_uv(p: vec3<f32>) -> vec2<f32> {
    let rel = (p.xz - globals.map.xy) / (2.0 * globals.map.z);
    return vec2<f32>(0.5 + rel.x, 0.5 - rel.y);
}

fn shape_at(np: vec3<f32>, tile: f32) -> vec4<f32> {
    return textureSampleLevel(shape_noise, noise_sampler, np / tile, 0.0);
}

fn detail_at(np: vec3<f32>) -> f32 {
    let d = textureSampleLevel(detail_noise, noise_sampler, np / DETAIL_TILE, 0.0);
    return d.r * 0.625 + d.g * 0.25 + d.b * 0.125;
}

fn shape_fbm(s: vec4<f32>) -> f32 {
    return s.g * 0.625 + s.b * 0.25 + s.a * 0.125;
}

// A sample of the clouds: density, height in its cloud 0..1, darkening, and how deep in its
// cloud's body it is 0..1 (the core: dark in storm cells, a little shaded in any thick cloud).
struct Sample {
    density: f32,
    height: f32,
    darken: f32,
    core: f32,
}

// The deck's coverage 0..1 at a render position (its field over the threshold, soft edged).
fn deck_cover(p: vec3<f32>) -> f32 {
    let uv = map_uv(p);
    let thr = globals.deck.z;
    let field = textureSampleLevel(deck_map, map_sampler, uv, 0.0).r;
    // A wide transition: the deck thins out over a few hundred metres past its edge.
    let local = smoothstep(thr - 0.07, thr + 0.07, field);
    // Past the map: the layer's average.
    let off = max(abs(uv.x - 0.5), abs(uv.y - 0.5));
    return mix(local, globals.deck_field.x, smoothstep(0.42, 0.5, off));
}

fn deck_density(p: vec3<f32>, np: vec3<f32>, detail: bool) -> Sample {
    var s = Sample(0.0, 0.0, globals.deck_field.y, 0.0);
    if (globals.deck.w == 0.0) { return s; }
    let h = (p.y - globals.deck.x) / max(globals.deck.y - globals.deck.x, 1.0);
    if (h < 0.0 || h > 1.0) { return s; }
    let cover = deck_cover(p);
    if (cover <= 0.001) { return s; }
    let n = shape_at(np, DECK_TILE);
    let fbm = shape_fbm(n);
    // A lumpy base and a top heaped where the noise is, thinning at the edges of the field.
    let bottom = 0.18 * (1.0 - n.b);
    let top = 0.55 + 0.45 * n.g;
    let profile = smoothstep(bottom, bottom + 0.12, h) * (1.0 - smoothstep(top - 0.3, top, h));
    let shape = sat(remap(n.r * 0.6 + fbm * 0.4, 0.1, 0.8)) * profile;
    var d = sat(remap(shape, 1.0 - cover, 1.0)) * cover;
    if (detail && d > 0.0) {
        // The edges fray more than the body.
        let e = detail_at(np);
        let fray = mix(0.85, 0.3, smoothstep(0.3, 0.9, cover));
        d = sat(remap(d, mix(e, 1.0 - e, sat(h * 3.0)) * fray, 1.0));
    }
    // Thin towards the edges: soft outlines rather than hard ones.
    s.density = d * 1.2 * smoothstep(0.0, 0.85, cover);
    s.height = h;
    s.core = cover * smoothstep(0.1, 0.6, h) * (1.0 - smoothstep(0.7, 1.0, h));
    return s;
}

// Large scale noise tile (m): the shapes' variety from cloud to cloud.
const LARGE_TILE: f32 = 16000.0;
// Small fragments' noise tile (m).
const FRAGMENT_TILE: f32 = 1600.0;
// How far the footprints are warped (m).
const WARP: f32 = 1400.0;

// The large scale noise at p (the footprints' warp in .gb) and the weather map there, warped.
fn warped_weather(p: vec3<f32>, np: vec3<f32>, big: vec4<f32>) -> vec4<f32> {
    let warp = (vec2<f32>(big.g, big.b) - 0.5) * WARP;
    let uv = map_uv(p + vec3<f32>(warp.x, 0.0, warp.y));
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
        return vec4<f32>(8.0, 0.0, 0.0, 0.0);
    }
    return textureSampleLevel(weather_map, map_sampler, uv, 0.0);
}

fn large_noise(np: vec3<f32>) -> vec4<f32> {
    return shape_at(vec3<f32>(np.x, 0.0, np.z), LARGE_TILE);
}

// How far a storm cell's anvil is blown downwind at its top (m).
const ANVIL_SHEAR: f32 = 3500.0;

// A storm cell: a towering column under an anvil spreading wide and blown downwind, from the
// weather map's nearest cell.
fn storm_cell(p: vec3<f32>, np: vec3<f32>, big: vec4<f32>, detail: bool) -> Sample {
    var s = Sample(0.0, 0.0, 0.0, 0.0);
    var w = warped_weather(p, np, big);
    if (w.r >= 3.0 || w.g < 1.5) { return s; }
    var h = (p.y - w.b - globals.params.w) / max(w.a - w.b, 1.0);
    // The anvil is blown downwind: high up, the cell is found upwind.
    if (h > 0.55) {
        let shear = globals.layer.yz * ANVIL_SHEAR * smoothstep(0.55, 0.95, h);
        let ws = warped_weather(p - vec3<f32>(shear.x, 0.0, shear.y), np, big);
        if (ws.g >= 1.5 && ws.r < w.r + 0.5) {
            w = ws;
            h = (p.y - w.b - globals.params.w) / max(w.a - w.b, 1.0);
        }
    }
    let dr = w.r;
    if (dr >= 3.0 || w.g < 1.5 || h < 0.0 || h > 1.0) { return s; }
    let fill = w.g - 2.0;
    let anvil = smoothstep(0.66, 0.8, h);
    let rr = mix(0.95 - 0.15 * h, 2.1, anvil);
    let top_cut = 1.0 - smoothstep(0.95, 1.0, h);
    let cover = sat((1.0 - dr / rr) * 1.4) * fill * top_cut;
    if (cover <= 0.001) { return s; }
    var n = shape_at(np, STORM_TILE);
    if (anvil > 0.0) {
        // The anvil's ice: fibrous, stretched sideways rather than billowing.
        let f = shape_at(vec3<f32>(np.x, np.y * 5.0, np.z), STORM_TILE * 1.5);
        n = mix(n, f, anvil);
    }
    let fbm = shape_fbm(n);
    let shape = sat(remap(n.r * 0.5 + fbm * 0.5, 0.25, 0.8)) * smoothstep(0.0, 0.04, h);
    var d = sat(remap(shape, 1.0 - cover, 1.0)) * cover;
    if (detail && d > 0.0) {
        let e = detail_at(np);
        let fray = mix(1.0, 0.65, smoothstep(0.2, 0.7, cover));
        let billow = mix(e, 1.0 - e, sat(h * 6.0));
        d = sat(remap(d, mix(billow, e, anvil) * fray, 1.0));
    }
    d *= smoothstep(0.0, 0.8, cover) * 2.4;
    d += smoothstep(0.55, 0.95, cover) * 0.8;
    s.density = d;
    s.height = h;
    s.darken = globals.band.w;
    s.core = cover * sat(cover * 1.5);
    return s;
}

// The cumulus layer, built as UnityVolumetricCloudsURP / HDRP build theirs: one large Perlin-
// Worley noise thresholded by the coverage (`remap(noise, 1 - curve * coverage *
// (1 - shapeFactor), 1) * coverage^2`) under per-height density and erosion curves (its
// Cloudy preset), eroded by inverted Worley noise and a finer micro erosion. The clouds'
// sizes and shapes come from the noise; the coverage (the application's cumulus, soft and
// max blended, over a thin background coverage from the layer's cover) says where it may
// rise into cloud. Where the coverage is moderate, coverage^2 leaves whole areas of
// translucent wisps.
const CU_SHAPE_TILE: f32 = 14000.0;
const CU_EROSION_TILE: f32 = 900.0;
const CU_MICRO_TILE: f32 = 330.0;
const CU_EROSION_FACTOR: f32 = 0.8;
const CU_MICRO_FACTOR: f32 = 0.65;
// Density multiplier (Unity's densityMultiplier^2 * 2 with 0.4, times its sigma 0.04 over ours).
const CU_DENSITY: f32 = 0.45;

fn cumulus_layer(p: vec3<f32>, np: vec3<f32>, big: vec4<f32>, detail: bool) -> Sample {
    var s = Sample(0.0, 0.0, 0.0, 0.0);
    if (globals.cumulus.z == 0.0 && globals.layer.x <= 0.0) { return s; }
    let warp = (vec2<f32>(big.g, big.b) - 0.5) * 900.0;
    let uv = map_uv(p + vec3<f32>(warp.x, 0.0, warp.y));
    var c = vec4<f32>(0.0, -60000.0, -60000.0, 0.0);
    if (all(uv >= vec2<f32>(0.0)) && all(uv <= vec2<f32>(1.0))) {
        c = textureSampleLevel(coverage_map, map_sampler, uv, 0.0);
    }
    // Background coverage between the application's clouds: thin clouds and wisps.
    let sky = globals.layer.x;
    // Large scale noise clusters the clouds: whole regions denser or emptier than the
    // individual clouds would make them.
    let cluster = sat(big.r * 1.4 - 0.2);
    let cov = max(c.r * mix(0.5, 1.05, cluster), 0.75 * sky * mix(0.1, 1.3, cluster));
    if (cov <= 0.01) { return s; }
    // Heights: the nearby clouds' (lowest base, highest top), else the layer's.
    var base = globals.cumulus.x;
    var top = mix(globals.cumulus.x, globals.cumulus.y, 0.6);
    if (c.g + globals.params.w > globals.cumulus.x - 2000.0 && c.g > -50000.0) {
        base = -c.b + globals.params.w;
        top = c.g + globals.params.w;
    }
    let h = (p.y - base) / max(top - base, 1.0);
    // Each area its own height (Unity's max cloud height channel).
    let he = h / mix(0.45, 1.0, big.a);
    if (h < 0.0 || he > 1.0) { return s; }
    // Cloudy preset curves: density (0, 0) (0.15, 1) (1, 0.1); erosion (0, 1) (0.1, 0.9) (1, 1).
    let dcurve = smoothstep(0.0, 0.15, he) * mix(1.0, 0.1, sat((he - 0.15) / 0.85));
    let ecurve = select(mix(0.9, 1.0, (he - 0.1) / 0.9), mix(1.0, 0.9, he / 0.1), he < 0.1);
    // Unity's shape factor (0.9 Cloudy, 0.95 Sparse, 0.5 Overcast) follows the layer's cover
    // here: a broken sky rises into cloud over most of its coverage, a scattered one in its
    // noise's peaks.
    let shape_factor = mix(0.1, 1.0, mix(0.75, 0.3, sat(sky * 1.4))) * ecurve;
    let erosion_factor = CU_EROSION_FACTOR * ecurve;
    // Several scales: big masses, mid-sized clouds and smaller ones, not one size of blob.
    let n_mid = shape_at(np, CU_SHAPE_TILE);
    let n_big = shape_at(np, CU_SHAPE_TILE * 2.9);
    let raw = sat(n_mid.r * 0.5 + n_big.r * 0.3 + (shape_fbm(n_mid) - 0.5) * 0.5 + 0.1);
    let low = mix(1.0, raw, shape_factor);
    let threshold = 1.0 - dcurve * cov * (1.0 - shape_factor);
    var d = sat(remap(low, threshold, 1.0)) * cov * cov;
    if (d <= 0.0) { return s; }
    if (detail) {
        let e = (1.0 - textureSampleLevel(detail_noise, noise_sampler, np / CU_EROSION_TILE, 0.0).r)
            * erosion_factor * 0.75 * cov;
        d = remap(d, e, 1.0);
        let f = (1.0 - textureSampleLevel(detail_noise, noise_sampler, np / CU_MICRO_TILE, 0.0).r)
            * CU_MICRO_FACTOR * ecurve * 0.5 * cov;
        d = remap(d, f, 1.0);
    } else {
        // The light march skips the erosion: an average of it instead.
        d -= erosion_factor * 0.1;
    }
    d = max(d, 0.0);
    s.density = d * CU_DENSITY;
    s.height = he;
    s.darken = globals.map.w;
    s.core = cov * cov * smoothstep(0.1, 0.5, he);
    return s;
}

fn convective_density(p: vec3<f32>, np: vec3<f32>, detail: bool) -> Sample {
    if (globals.band.z == 0.0 || p.y < globals.band.x || p.y > globals.band.y) {
        return Sample(0.0, 0.0, 0.0, 0.0);
    }
    let big = large_noise(np);
    let a = storm_cell(p, np, big, detail);
    let b = cumulus_layer(p, np, big, detail);
    if (a.density >= b.density) {
        return Sample(a.density + b.density, a.height, a.darken, a.core);
    }
    return Sample(a.density + b.density, b.height, b.darken, b.core);
}

fn density(p: vec3<f32>, detail: bool) -> Sample {
    let np = noise_pos(p);
    let a = deck_density(p, np, detail);
    let b = convective_density(p, np, detail);
    if (a.density >= b.density) {
        return Sample(a.density + b.density, a.height, a.darken, a.core);
    }
    return Sample(a.density + b.density, b.height, b.darken, b.core);
}

// Optical depth towards the sun: a short march (exponential steps) and the deck's shade from
// above.
const LIGHT_STEPS: array<f32, 6> = array<f32, 6>(20.0, 40.0, 80.0, 160.0, 320.0, 640.0);

fn light_depth(p: vec3<f32>, sun: vec3<f32>) -> f32 {
    var od = 0.0;
    var t = 0.0;
    for (var i = 0; i < 6; i++) {
        let len = LIGHT_STEPS[i];
        let q = p + sun * (t + 0.5 * len);
        od += density(q, false).density * len;
        t += len;
    }
    // Two far samples: thick masses (storm cells, dense decks) shade their cores. Moderate
    // weights, so where they leave the cloud the shade doesn't jump.
    od += density(p + sun * (t + 300.0), false).density * 500.0;
    od += density(p + sun * (t + 1800.0), false).density * 700.0;
    // Under the deck: its shade where the sun's ray meets its base.
    if (globals.deck.w > 0.0 && p.y < globals.deck.x && sun.y > 0.05) {
        let q = p + sun * ((globals.deck.x - p.y) / sun.y);
        od += deck_cover(q) * (globals.deck.y - globals.deck.x) * 0.4;
    }
    return od * SIGMA;
}

// Light of the line light (a lightning channel) at p.
fn line_glow(p: vec3<f32>) -> f32 {
    let i = globals.line_a.w;
    if (i <= 0.0) { return 0.0; }
    let a = globals.line_a.xyz;
    let ab = globals.line_b.xyz - a;
    let t = sat(dot(p - a, ab) / max(dot(ab, ab), 1e-3));
    let d = length(a + ab * t - p);
    let reach = globals.line_b.w;
    let core = d / (reach * 0.06);
    let halo = d / (reach * 0.25);
    let fade = 1.0 - smoothstep(0.5 * reach, 1.5 * reach, d);
    return i * fade * (1.6 / (1.0 + core * core) + 2.0 / (1.0 + halo * halo));
}

struct March {
    light: vec3<f32>,
    transmittance: f32,
    // Coverage-weighted distance and the weights' sum.
    t_sum: f32,
    w_sum: f32,
    t_first: f32,
    t_opaque: f32,
    // How far the march got (the deck past it is drawn as a slab).
    reached: f32,
}

fn slab(y: f32, dy: f32, lo: f32, hi: f32) -> vec2<f32> {
    if (abs(dy) < 1e-5) {
        return select(vec2<f32>(1e20, -1e20), vec2<f32>(0.0, 1e20), y >= lo && y <= hi);
    }
    let a = (lo - y) / dy;
    let b = (hi - y) / dy;
    return vec2<f32>(max(min(a, b), 0.0), max(a, b));
}

// Whether there may be cloud at p (no noise read): within the deck where its field covers, or
// within a convective cloud's reach and heights.
fn occupied(p: vec3<f32>) -> bool {
    if (globals.deck.w > 0.0 && p.y >= globals.deck.x && p.y <= globals.deck.y && deck_cover(p) > 0.001) {
        return true;
    }
    // The cumulus layer: its background coverage may raise cloud anywhere in it.
    if (globals.cumulus.z > 0.0 && p.y >= globals.cumulus.x && p.y <= globals.cumulus.y) {
        return true;
    }
    if (globals.band.z > 0.0 && p.y >= globals.band.x && p.y <= globals.band.y) {
        // Read as the density reads it (warped), so cloud edges aren't cut off.
        let np = noise_pos(p);
        let big = large_noise(np);
        let w = warped_weather(p, np, big);
        let h = (p.y - w.b - globals.params.w) / max(w.a - w.b, 1.0);
        if (w.r < 3.0 && h >= -0.05 && h <= 1.05) {
            return true;
        }
        // A storm cell's anvil, blown downwind.
        if (w.r >= 3.0 || w.g >= 1.5) {
            let shear = globals.layer.yz * ANVIL_SHEAR;
            let ws = warped_weather(p - vec3<f32>(shear.x, 0.0, shear.y), np, big);
            return ws.g >= 1.5 && ws.r < 3.0;
        }
    }
    return false;
}

// Ambient occlusion: coarse cloud above and on four sides of p (the sky's light comes from
// there), so valleys between billows and folds in thick cloud shade, seen from above too.
// Fixed directions: a turning pattern would leave a fine grain the accumulation can't hide.
fn occlusion(p: vec3<f32>) -> f32 {
    var od = density(p + vec3<f32>(0.0, 120.0, 0.0), false).density * 120.0;
    od += density(p + vec3<f32>(0.0, 400.0, 0.0), false).density * 250.0;
    od += density(p + vec3<f32>(250.0, 60.0, 0.0), false).density * 75.0;
    od += density(p + vec3<f32>(-250.0, 60.0, 0.0), false).density * 75.0;
    od += density(p + vec3<f32>(0.0, 60.0, 250.0), false).density * 75.0;
    od += density(p + vec3<f32>(0.0, 60.0, -250.0), false).density * 75.0;
    return od * SIGMA;
}

// Clear air above the clouds (m of haze distance).
const CLEAR_HAZE: f32 = 80000.0;

// The haze distance: the weather's visibility (under and among the clouds), lengthening to
// clear air's as the eye climbs above the highest layer.
fn haze_distance(cam: vec3<f32>) -> f32 {
    let haze = globals.sun.w;
    if (haze <= 0.0) { return haze; }
    var top = -1e9;
    if (globals.deck.w > 0.0) { top = max(top, globals.deck.y); }
    if (globals.band.z > 0.0) { top = max(top, globals.band.y); }
    if (top < -1e8) { return haze; }
    return mix(haze, max(haze, CLEAR_HAZE), smoothstep(0.0, 2000.0, cam.y - top));
}

// Primary step length at distance t (m): fine near the eye, longer far out.
fn step_at(t: f32) -> f32 {
    return clamp(MIN_STEP + 0.012 * t, MIN_STEP, 500.0);
}

fn march(m: ptr<function, March>, cam: vec3<f32>, rd: vec3<f32>, t0: f32, t1: f32, jitter: f32) {
    if (t1 <= t0) { return; }
    let sun = globals.sun.xyz;
    let cos_t = dot(rd, sun);
    let haze = haze_distance(cam);
    var t = t0 + jitter * step_at(t0);
    var taken = 0;
    var skipped = false;
    var backed = false;
    // Long stretches (looking down across the whole band from high up) take longer steps, so
    // the budget lasts to the far end.
    let least = (t1 - t0) / f32(3 * MAX_STEPS);
    for (var i = 0; i < 8 * MAX_STEPS; i++) {
        if (t >= t1 || (*m).transmittance < 0.01 || taken >= 2 * MAX_STEPS) { break; }
        let ds = max(step_at(t), least * smoothstep(4000.0, 12000.0, t));
        let p = cam + rd * t;
        // Empty space: long strides until something may be there.
        if (!occupied(p)) {
            t += 3.0 * ds;
            skipped = true;
            continue;
        }
        // Coming out of a stride: back up to where the cloud may begin (once).
        if (skipped && !backed) {
            t = max(t - 2.0 * ds, t0);
            skipped = false;
            backed = true;
            continue;
        }
        skipped = false;
        backed = false;
        let s = density(p, true);
        if (s.density <= 0.0) {
            t += ds;
            continue;
        }
        // The budget counts the samples with cloud (their light march is the cost); empty
        // ones inside the layers' reach only cost the density.
        taken += 1;
        let sigma = s.density * SIGMA;
        let step_t = exp(-sigma * ds);
        // Sunlight: three multiple scattering octaves of a two-lobe phase, Beer's law along
        // the light march, a powder term darkening the sunward edges seen from the sun's side,
        // and light diffused through the cloud above (thinner parts glow).
        let tau = light_depth(p, sun);
        // Only the upper parts: an underside's light comes from below and the sides.
        let ao = mix(1.0, exp(-0.1 * occlusion(p)), sat(s.height * 2.0));
        var scatter = 0.0;
        var a = 1.0;
        var b = 1.0;
        var c = 1.0;
        for (var o = 0; o < 3; o++) {
            scatter += a * exp(-tau * b) * phase(cos_t, c);
            a *= 0.5;
            b *= 0.5;
            c *= 0.5;
        }
        let powder = mix(1.0, 1.0 - exp(-s.density * 6.0), 0.6 * sat(0.5 - 0.5 * cos_t));
        // Deep in dense cloud little light gets through: the cores of storm cells go dark.
        let diffuse = 0.5 * exp(-0.045 * tau) * mix(0.5, 1.0, ao);
        // Soft saturation: sunlit tops keep their gradients (powder edges, billows' shade,
        // occlusion) instead of all clipping to the brightest colour.
        let energy = scatter * powder * 0.75 + diffuse;
        let sunlit = 1.0 - exp(-1.5 * energy);
        // Ambient: the sky above, the ground below.
        let ambient = mix(ramp(10.0) * 0.6, ramp(3.0), sat(s.height * 1.4)) * mix(mix(0.5, 0.2, sat(s.darken * 3.0)), 1.0, exp(-0.03 * tau)) * mix(0.35, 1.0, ao);
        var light = mix(ambient, globals.ramp[0].rgb, sunlit) * mix(0.65, 1.0, ao)
            + globals.ramp[0].rgb * min(max(energy - 1.5, 0.0), 1.0) * 0.2;
        // Dark clouds (storms) darken further the deeper the light has to go and the deeper in
        // their body (an anvil's core too, lit from above as it is); thick cloud of any kind
        // shades a little in its core.
        light *= max(1.0 - s.darken * (0.8 + 1.3 * sat(tau / 30.0) + 1.2 * s.core), 0.04);
        light *= 1.0 - 0.3 * s.core * s.core;
        light += globals.flash.rgb + globals.line_color.rgb * line_glow(p);
        // Haze: towards the sunlit cloud colour with distance.
        if (haze > 0.0) {
            light = mix(light, globals.ramp[0].rgb, sat(t / (1.5 * haze)));
        }
        let w = (*m).transmittance * (1.0 - step_t);
        (*m).light += light * w;
        (*m).t_sum += t * w;
        (*m).w_sum += w;
        if ((*m).t_first >= 1e19) {
            (*m).t_first = max(t - ds, 0.0);
        }
        (*m).transmittance *= step_t;
        if ((*m).transmittance < 0.5 && (*m).t_opaque >= 1e19) {
            (*m).t_opaque = t;
        }
        t += ds;
    }
    (*m).reached = max((*m).reached, min(t, t1));
}

struct FragOut {
    @location(0) color: vec4<f32>,
    @location(1) depths: vec4<f32>,
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> FragOut {
    let pixel = vec2<i32>(in.position.xy);
    // The ray through the middle of the block of view pixels.
    let full = (vec2<f32>(pixel) + 0.5) * f32(RES_SCALE);
    let ndc = vec2<f32>(full.x / view.viewport.z * 2.0 - 1.0, 1.0 - full.y / view.viewport.w * 2.0);
    // The ray in view space, then turned into the world: the near plane's point minus the
    // camera would lose the direction to rounding with the camera far from the render origin.
    let v = view.view_from_clip * vec4<f32>(ndc, 1.0, 1.0);
    let rd = normalize((view.world_from_view * vec4<f32>(v.xyz / v.w, 0.0)).xyz);
    let cam = view.world_position;
    let scale = length(view.world_from_view[2].xyz);
    let forward = -view.world_from_view[2].xyz / scale;
    let to_z = dot(rd, forward) / scale;

    let scene_depth = scene_depth_at(pixel);
    let near = view.clip_from_view[3][2];
    var t_scene = select(1e9, near / scene_depth * scale / max(dot(rd, forward), 1e-4), scene_depth > 0.0);
    // The sky's background sphere (hundreds of km away) is not in front of the clouds: past
    // SKY_DEPTH the scene is open sky (else the deck stops short of the horizon).
    if (t_scene > SKY_DEPTH) {
        t_scene = 1e9;
    }

    var m = March(vec3<f32>(0.0), 1.0, 0.0, 0.0, 1e20, 1e20, 0.0);
    let jitter = ign(in.position.xy);
    // The deck and the band: one march where they overlap, else one after the other.
    var a = vec2<f32>(1e20, -1e20);
    var deck_span = vec2<f32>(1e20, -1e20);
    if (globals.deck.w > 0.0) {
        deck_span = slab(cam.y, rd.y, globals.deck.x, globals.deck.y);
        // Marched for DECK_RANGE from where the ray enters it (from high above, that is far away).
        a = vec2<f32>(deck_span.x, min(min(deck_span.y, deck_span.x + DECK_RANGE), t_scene));
    }
    var b = vec2<f32>(1e20, -1e20);
    if (globals.band.z > 0.0) {
        let s = slab(cam.y, rd.y, globals.band.x, globals.band.y);
        let horizontal = max(length(rd.xz), 1e-4);
        b = vec2<f32>(s.x, min(min(s.y, BAND_RANGE / horizontal), t_scene));
    }
    let a_on = a.y > a.x;
    let b_on = b.y > b.x;
    if (a_on && b_on && a.x <= b.y && b.x <= a.y) {
        march(&m, cam, rd, min(a.x, b.x), max(a.y, b.y), jitter);
    } else if (a_on && b_on) {
        let first = select(b, a, a.x < b.x);
        let second = select(a, b, a.x < b.x);
        march(&m, cam, rd, first.x, first.y, jitter);
        march(&m, cam, rd, second.x, second.y, jitter);
    } else if (a_on) {
        march(&m, cam, rd, a.x, a.y, jitter);
    } else if (b_on) {
        march(&m, cam, rd, b.x, b.y, jitter);
    }

    // The deck past the march: one slab at its average coverage, lit as a flat sheet.
    if (globals.deck.w > 0.0 && m.transmittance > 0.01) {
        // From where the march stopped (its range, or its step budget) to the deck's far side.
        let f0 = max(deck_span.x, min(a.y, m.reached));
        // To the horizon: a ray just off level meets the deck very far away.
        let f1 = min(deck_span.y, t_scene);
        if (f1 > f0) {
            let cover = globals.deck_field.x;
            let alpha = 1.0 - exp(-(f1 - f0) * cover / 1500.0);
            let from_above = cam.y > globals.deck.y;
            let sun_up = max(globals.sun.y, 0.0);
            // As the marched deck looks there: hazed alike, so the two meet without a seam.
            var c = select(ramp(8.0), mix(ramp(3.0), globals.ramp[0].rgb, 0.5 * sun_up), from_above)
                * (1.0 - globals.deck_field.y);
            let haze = haze_distance(cam);
            if (haze > 0.0) {
                c = mix(c, globals.ramp[0].rgb, sat(f0 / (1.5 * haze)));
            }
            let w = m.transmittance * alpha;
            m.light += c * w;
            m.t_sum += f0 * w;
            m.w_sum += w;
            if (m.t_first >= 1e19) {
                m.t_first = f0;
            }
            m.transmittance *= 1.0 - alpha;
            if (m.transmittance < 0.5 && m.t_opaque >= 1e19) {
                m.t_opaque = f0;
            }
        }
    }

    var out: FragOut;
    // Nearly opaque is opaque: the last few percent would let bright things behind (the sea's
    // glitter) show through thick cloud.
    var alpha = 1.0 - m.transmittance;
    var rgb = m.light;
    if (m.transmittance < 0.03) {
        rgb /= max(alpha, 1e-3);
        alpha = 1.0;
    }
    rgb = mix(rgb, globals.fog.rgb * alpha, globals.fog.a);
    out.color = vec4<f32>(rgb, alpha);
    // Where nothing was drawn, reprojection takes the start of the layers (or the sky).
    var t_mean = select(min(min(a.x, b.x), 1e5), m.t_sum / max(m.w_sum, 1e-6), m.w_sum > 1e-4);
    t_mean = min(t_mean, 1e5);
    out.depths = vec4<f32>(
        select(min(m.t_first * to_z, NO_CLOUD), NO_CLOUD, m.t_first >= 1e19),
        min(t_mean * to_z, NO_CLOUD),
        select(min(m.t_opaque * to_z, NO_CLOUD), NO_CLOUD, m.t_opaque >= 1e19),
        0.0,
    );
    return out;
}

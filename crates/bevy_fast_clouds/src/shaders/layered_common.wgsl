// Shared by the layered clouds' passes (layered/mod.rs `LayeredGlobals`, render.rs).
#define_import_path bevy_fast_clouds::layered

struct LayeredGlobals {
    // Direction to the sun, haze distance.
    sun: vec4<f32>,
    flash: vec4<f32>,
    // Line light: one end and intensity, the other end and reach, colour.
    line_a: vec4<f32>,
    line_b: vec4<f32>,
    line_color: vec4<f32>,
    // Fog over the clouds (inside a cloud): colour, amount.
    fog: vec4<f32>,
    // Shade ramp: 0 sunlit (and the haze colour) .. 15 darkest.
    ramp: array<vec4<f32>, 16>,
    // Deck: base and top (render y), threshold, present.
    deck: vec4<f32>,
    // Deck: mean coverage, darkening, field seed, field unit (metres).
    deck_field: vec4<f32>,
    deck_cells: vec4<f32>,
    deck_weights: vec4<f32>,
    // Convective clouds: lowest base and highest top (render y), present, storm darkening.
    band: vec4<f32>,
    // Maps: centre x and z (render), half span (m), cumulus darkening.
    map: vec4<f32>,
    // Noise offset (render position + this, modulo NOISE_PERIOD).
    noise: vec4<f32>,
    // Frame counter, resolution scale, primary steps, render y of the world's zero.
    params: vec4<f32>,
    // Cumulus coverage 0..1, downwind direction (render x, z).
    layer: vec4<f32>,
}

// Per view: the previous frame's projection, the floating origin's move since (xyz) and
// whether there is a history (w).
struct LayeredView {
    prev_clip_from_world: mat4x4<f32>,
    shift: vec4<f32>,
}

// No cloud (the largest half float).
const NO_CLOUD: f32 = 65504.0;

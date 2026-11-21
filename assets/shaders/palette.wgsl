// Palette-indexed rendering that reproduces classic MCGA drawing:
// colour words (high byte = fill mode), per-object distance haze down 16-colour ramps,
// and the sky/ground band gradient (background sphere and AND/OR terrain overlays).

#import bevy_pbr::{
    mesh_functions,
    mesh_view_bindings::{view, directional_shadow_textures, directional_shadow_textures_comparison_sampler},
    view_transformations::position_world_to_clip,
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
}

const MAX_LOCAL_LIGHTS: u32 = 32u;
const LIGHT_SHADOWS: u32 = 1u;
const LIGHT_LOCAL: u32 = 2u;
const LIGHT_MOON_KEY: u32 = 4u;
const FLAG_CLOUD: u32 = 8u;
// Smoke plume volume (fires.rs), optionally lit by a fire at its base.
const TAG_SMOKE: u32 = 8192u;
const TAG_SMOKE_FIRE: u32 = 16384u;
// Warm object (palette_material.rs `TAG_WARM`).
const TAG_WARM: u32 = 2048u;
const TAG_HOT: u32 = 131072u;
const TAG_DOT_NEAR: u32 = 4096u;
// Aircraft spotting dot (palette_material.rs `TAG_DOT_SPOT`).
const TAG_DOT_SPOT: u32 = 32768u;
// Dot only drawn from DOT_FAR_MIN metres away (palette_material.rs `TAG_DOT_FAR`).
const TAG_DOT_FAR: u32 = 2147483648u;
const TAG_VERTEX_HAZE: u32 = 1073741824u;
// The rolling hills' ground (palette_material.rs `TAG_HILLS`): shaded by its slope in the
// classic look too.
const TAG_HILLS: u32 = 65536u;
// A vertex lit as flat ground whatever the face (palette_material.rs `WORD_FLAT_LIT`, above its
// colour word: the hills' skirts).
const WORD_FLAT_LIT: u32 = 65536u;
// A glowing lamp's dot batch (palette_material.rs `FLAG_GLOW`, `Effects::city_lights`).
const FLAG_GLOW: u32 = 1024u;
// A building's wall with windows lit at night (palette_material.rs `WORD_WINDOWS`).
const WORD_WINDOWS: u32 = 1048576u;
const DOT_FAR_MIN: f32 = 3000.0;
// Share of a moonless night's darkness on aircraft and their spotting dots: barely visible.
const AIRCRAFT_NIGHT: f32 = 0.95;
// Cloud colours: the grey ramp (its base entry is the horizon haze colour).
const CLOUD_RAMP: u32 = 0xE0u;

struct Globals {
    // Live palette (linear RGB).
    palette: array<vec4<f32>, 256>,
    // Sky band boundaries (16, packed 4 per vec4), in 320x200 screen units at f = 256.
    sky_ring: array<vec4<f32>, 4>,
    // Ground band boundaries below the horizon (16), same units; depend on altitude.
    ground_g: array<vec4<f32>, 4>,
    // x: haze enabled, y: metres per world unit, z: FLIR (tracking camera), w: modern mode
    params: vec4<f32>,
    // Direction to the sun (xyz), strength (w).
    sun: vec4<f32>,
    // Ambient (rgb), lighting flags (w).
    ambient: vec4<f32>,
    // x: local light count, y: daylight 0 (night) .. 1 (day), z: night vision goggles (2: with
    // the goggles effect, halos and stronger grain),
    // w: flat colours (SM89_FLAT_SHADER bits: 1 all, 2 background, 4 objects, 8 clouds and smoke).
    light_info: vec4<f32>,
    // x: XR dot angular size in radians (0 = flat screen), y: animation clock (seconds,
    // wraps every hour), zw: tracking camera picture size (pixels).
    dots: vec4<f32>,
    // Two entries per light: (position, range), (colour * intensity, unused).
    local_lights: array<vec4<f32>, 64>,
    // Weather: deck base / top (metres, eye height frame), deck present (z), whiteout 0..1 (w).
    clouds: vec4<f32>,
    // x: haze distance scale, y: lightning flash 0..1, z: visibility under clouds (metres),
    // w: gloom 0..1 (grey, dim days under an overcast and in storms).
    weather: vec4<f32>,
    // Cloud texture: floating origin (x, z) modulo NOISE_PERIOD, seed (z); stratus layer
    // coverage 0..1 (w).
    noise: vec4<f32>,
    // Stratus layer coverage: threshold 0..1 (x), noise seed (y), floating origin x/z (z, w).
    stratus: vec4<f32>,
    // Lightning channel as a line light: one end (xyz) and intensity (w), the other end (xyz)
    // and reach in metres (w).
    bolt_a: vec4<f32>,
    bolt_b: vec4<f32>,
    // Continuous time of day: direction to the sun (xyz), on (w = 1); direction to the moon
    // (xyz), star visibility 0..1 (w).
    sky_sun: vec4<f32>,
    sky_moon: vec4<f32>,
    // x: darkness of a night without moonlight 0..1 (0 under night vision goggles), y: the
    // strength of its red horizon glow, zw: the recon cameras' viewport size.
    sky_night: vec4<f32>,
    // The recon cameras' palette (linear RGB): the day's, as in the near infrared.
    recon_palette: array<vec4<f32>, 256>,
    // Direction to the moon (xyz) and its light's strength (w), for modern shading.
    moon: vec4<f32>,
    // The key light's shadow maps: number of cascades (y, 0 without shadows), normal bias (z)
    // and depth bias (w); the cascades' far bounds; their light space projections and texel
    // sizes, the direction to the light (palette.rs `shadow_cascades`: Bevy's, as in its
    // `lights` binding, which the shader leaves alone: indexing it costs the Steam Frame's
    // GPU hundreds of constants on every draw).
    shadow: vec4<f32>,
    shadow_bounds: vec4<f32>,
    shadow_cascades: array<mat4x4<f32>, 4>,
    shadow_texel: vec4<f32>,
    shadow_light: vec4<f32>,
    // Storm haze under an overcast: strength 0..1 (x), visibility in metres (y).
    storm_haze: vec4<f32>,
    // Combat flash (flashes.rs) as a point light: position (xyz) and intensity (w), linear
    // colour (xyz) and reach in metres (w).
    burst: vec4<f32>,
    burst_colour: vec4<f32>,
    // Ground mist's strength 0..1 (x), depth (y, metres) and the ground it lies on (w, metres);
    // cloud shadows' strength (z).
    atmos: vec4<f32>,
    // Cloud shadows (weather.rs `CloudShadows`): the grid's corner x, z, square size and the
    // cumulus base height; cumulus shadows on (x), stratus shadows' strength (y), the average
    // shade past the grid (z).
    shadow_grid: vec4<f32>,
    shadow_far: vec4<f32>,
    // Night sky extras: aurora strength (x), shooting stars (y), twinkling (z); desert
    // specks (w).
    sky_fx: vec4<f32>,
    // Retro output: VGA colours (x), scanlines (y).
    retro: vec4<f32>,
    // The shadow grid's squares (up to four cloud numbers, 0: none) and the cloud discs
    // (centre x, z where the sun sees them from the base height, radius, darkness).
    shadow_cells: array<vec4<f32>, 144>,
    shadow_clouds: array<vec4<f32>, 112>,
    // The stars: the columns of the rotation from the world's frame to the star catalogue's
    // (stars.rs `frame`).
    star_frame: array<vec4<f32>, 3>,
    // The sea: sea state 0..8 (x), downwind direction in the world's x / z plane (yz),
    // whitecaps 0..1 (w).
    sea: vec4<f32>,
    // The simulated ocean (ocean.rs `Ocean::globals`): on (x), the waves' rise in the mesh
    // (y), the mesh drawn (z); the open water mask's corner (xy, from the floating origin),
    // span (z), present (w); the mesh levels' centres (two a vec4).
    ocean: vec4<f32>,
    ocean_mask: vec4<f32>,
    ocean_levels: array<vec4<f32>, 4>,
    // The ground detail (`PALETTE_DETAIL`): x 1 when drawn, yz the floating origin's x and z
    // modulo DETAIL_PERIOD.
    // The towns' lights (`PALETTE_CITY`): x the lit windows' strength, y the range inside which
    // the walls show their windows (the window lamps only beyond it).
    city: vec4<f32>,
    // The floating origin (xyz): the windows stay put on the walls.
    city_origin: vec4<f32>,
    ground_detail: vec4<f32>,
    // Reflections (`PALETTE_GLASS`, `PALETTE_WATER`): x 1 on glass, y 1 on water.
    reflections: vec4<f32>,
};

struct PaletteMaterial {
    // Colour word: low byte = palette index, high byte = fill mode (FF solid, FE AND, FD OR,
    // FC screen door, FB invisible, < FB two-colour checkerboard).
    word: u32,
    // bit 0: background sphere (sky/ground gradient), bit 1: line/dot (always solid low byte)
    flags: u32,
    // The lighting options are on (the PALETTE_LIT variant; unused here).
    lit: u32,
    // The atmosphere effects compiled in (the PALETTE_MIST ... PALETTE_RETRO variants; unused
    // here).
    fx: u32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: PaletteMaterial;
// Storage, or a uniform buffer where there are none (WebGL2: palette_material.rs).
#if AVAILABLE_STORAGE_BUFFER_BINDINGS > 0
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> globals: Globals;
#else
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> globals: Globals;
#endif

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
#ifdef DOT_BATCH
    // Batched dots: four vertices at each dot's centre, spread by their quad corner.
    @location(1) corner: vec2<f32>,
#endif
#ifdef DOT_COLOUR
    // Palette colour of each dot in a batch of several colours (dots.rs `dot_batch_coloured`).
    @location(2) colour: u32,
#endif
#ifdef VERTEX_WORD
    // Colour word of each vertex of a mesh of several colours (palette_material.rs
    // `ATTRIBUTE_PALETTE_WORD`).
    @location(3) word: u32,
#endif
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) @interpolate(flat) fog: u32,
    // Unrounded fog level (modern mode: smooth haze between ramp entries).
    @location(2) @interpolate(flat) fog_f: f32,
    // Cloud volumes, in the cloud's local space (unit radius, y 0..1): position (w: noise
    // offset of this cloud), eye (w: 1 cumulus, 2 tower, 3 smoke, 4 smoke over a fire), sun
    // direction, scale (metres per local unit on each axis).
    @location(3) puff_pos: vec4<f32>,
    @location(4) @interpolate(flat) puff_eye: vec4<f32>,
    @location(5) @interpolate(flat) puff_sun: vec3<f32>,
    @location(6) @interpolate(flat) puff_scale: vec3<f32>,
    // Bit 0: warm object or dot light (FLIR); bit 1: aircraft (level code 5); bit 2: spotting
    // dot; bit 3: the hills' ground; bit 4: lit as flat ground; bit 5: hot (a unit, a fire or a
    // dot light: white in the FLIR picture); bits 6 and 7: a field's crop rows run along x or
    // along z (the ground detail's furrows); bit 8: the scenery's ground (`WORD_GROUND`); bit 9:
    // a wall with windows (`WORD_WINDOWS`); bits 16..26: its direction (`WORD_WALL_ANGLE`).
    @location(7) @interpolate(flat) warm: u32,
#ifdef DOT_COLOUR
    @location(8) @interpolate(flat) dot_colour: u32,
#endif
#ifdef VERTEX_WORD
    @location(9) @interpolate(flat) word: u32,
#endif
#ifdef DOT_BATCH
    // A dot's quad corner (xy, -1..1) and its size over the lamp's (z: above 1 a goggles halo).
    @location(10) dot_uv: vec3<f32>,
#endif
#ifdef OCEAN_GRID
    // The waves' mesh: the undisplaced point (xy), the level (z), the waves' rise (w).
    @location(11) ocean: vec4<f32>,
#endif
};

// Dot lights: size relative to one 320x200 screen pixel and lower bound in pixels.
const DOT_SIZE: f32 = 0.6;
const DOT_MIN_PIXELS: f32 = 2.0;
// Aircraft lights (level code 5) up close: drawn as a lamp this many metres across, up to
// AIRCRAFT_DOT_MAX times their size far away.
const AIRCRAFT_DOT_METRES: f32 = 0.3;
const AIRCRAFT_DOT_MAX: f32 = 5.0;
// Dot lights are seen up to DOT_RANGE metres, or DOT_VIS_FACTOR times the weather's visibility
// if less (lights carry further than the visibility); dim ones (TAG_DOT_NEAR: runway edges,
// taxiways) DOT_NEAR times that.
const DOT_RANGE: f32 = 30000.0;
const DOT_VIS_FACTOR: f32 = 1.5;
const DOT_NEAR: f32 = 0.4;
// Spotting dots (TAG_DOT_SPOT) vanish once their aircraft (its size in metres in tag bits
// 16..) spans more than SPOT_PIXELS in the view, and sit that size towards the eye so its
// model can't hide them.
const SPOT_PIXELS: f32 = 12.0;
// Under night vision goggles the bright lights (aircraft lights, the bigger airfield lights)
// bloom: a halo this many times the lamp's size around it.
const NVG_HALO: f32 = 3.5;

// Glowing lamps (`FLAG_GLOW`): a soft point whose core is GLOW_PIXELS across on a 1080-line
// screen GLOW_REF metres away for the brightest, shrinking with the square root of the
// distance (and of the brightness); below GLOW_MIN pixels it stays that size and dims instead
// (the light of a point too small to see spread over a pixel or two), so the far lamps fade
// into glitter. The quad reaches GLOW_QUAD times the core for the falloff.
const GLOW_PIXELS: f32 = 2.4;
const GLOW_REF: f32 = 1500.0;
const GLOW_MIN: f32 = 1.2;
// Lamps outshine what the screen can show: their cores saturate.
const GLOW_GAIN: f32 = 2.0;
const GLOW_MAX: f32 = 7.0;
const GLOW_QUAD: f32 = 2.2;
const GLOW_FALLOFF: f32 = 4.5;
// The dimmest lamps (level 0) are seen up to this share of DOT_RANGE.
const GLOW_RANGE_DIM: f32 = 0.3;
// Window lamps fade in over this many metres beyond the walls' range (`Globals::city.y`).
const GLOW_WINDOW_FADE: f32 = 500.0;

// A glowing lamp's quad size in pixels (x) and brightness (y; 0 hidden), from its packed word
// (dots.rs `glow_word`) and distance; `focal`: this view's focal length in pixels.
fn glow_lamp(word: u32, dist: f32, focal: f32) -> vec2<f32> {
    let level = f32((word >> 8u) & 15u) / 15.0;
    let kind = (word >> 12u) & 3u;
    let seed = word >> 18u;
    var b = GLOW_GAIN * (0.25 + 0.75 * level);
    // Screen pixels at a 60 degree view (the zoomed tracking camera doesn't magnify them; the
    // headset's eyes see them their angular size).
    let scale = min(view.viewport.w / 1080.0, focal / 935.0);
    var core = GLOW_PIXELS * scale * sqrt((0.25 + 0.75 * level) * GLOW_REF / max(dist, 30.0));
    let least = GLOW_MIN * max(scale, 0.75);
    if (core < least) {
        b *= core / least;
        core = least;
    }
    core = min(core, GLOW_MAX * scale);
    var range = DOT_RANGE * (GLOW_RANGE_DIM + (1.0 - GLOW_RANGE_DIM) * level);
    if (globals.weather.z > 0.0) {
        range = min(range, globals.weather.z * DOT_VIS_FACTOR);
    }
    b *= 1.0 - smoothstep(range * 0.6, range, dist);
    if (kind == 1u) {
        b *= smoothstep(globals.city.y, globals.city.y + GLOW_WINDOW_FADE, dist);
    }
    // Far lamps twinkle a little through the air.
    let twinkle = sin(globals.dots.y * (2.0 + f32(seed & 7u)) + f32(seed)) * smoothstep(3000.0, 9000.0, dist);
    b *= 1.0 + 0.25 * twinkle;
    return vec2<f32>(core * GLOW_QUAD, b);
}

// A glowing lamp's colour (linear): its tint (dots.rs `GlowTint`), else the palette's.
fn glow_colour(word: u32, palette: vec3<f32>) -> vec3<f32> {
    switch ((word >> 14u) & 15u) {
        // Sodium: orange-yellow, each lamp a little more orange or more yellow.
        case 1u: { return mix(vec3<f32>(1.0, 0.5, 0.12), vec3<f32>(1.0, 0.74, 0.3), f32((word >> 18u) & 15u) / 15.0); }
        case 2u: { return vec3<f32>(0.85, 0.9, 1.0); }
        case 3u: { return vec3<f32>(0.6, 1.0, 0.85); }
        case 4u: { return mix(vec3<f32>(1.0, 0.58, 0.24), vec3<f32>(1.0, 0.78, 0.45), f32((word >> 22u) & 7u) / 7.0); }
        case 5u: { return vec3<f32>(0.75, 0.85, 1.0); }
        case 6u: { return vec3<f32>(0.25, 1.0, 0.35); }
        case 7u: { return vec3<f32>(0.3, 0.45, 1.0); }
        default: { return palette; }
    }
}

// World units per model unit, by level code in the mesh tag (bits 0..2):
// 0: LOD0 (0.5), 1: LOD1 (1), 2: LOD2 (4), 3: LOD3 (16), 4: LOD4 (64), 5: aircraft (1/8)
fn level_unit(code: u32) -> f32 {
    switch code {
        case 0u: { return 0.5; }
        case 1u: { return 1.0; }
        case 2u: { return 4.0; }
        case 3u: { return 16.0; }
        case 4u: { return 64.0; }
        default: { return 0.125; }
    }
}

@vertex
fn vertex(v: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_from_local = mesh_functions::get_world_from_local(v.instance_index);
#ifdef OCEAN_GRID
    let placed = ocean_vertex(v.position);
    let world = placed[0];
    out.ocean = placed[1];
#else
    let world = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(v.position, 1.0));
#endif
    out.world_position = world.xyz;
    out.clip_position = position_world_to_clip(world.xyz);
    let tag = mesh_functions::get_tag(v.instance_index);
    out.warm = select(0u, 1u, (tag & (TAG_WARM | 128u)) != 0u) | select(0u, 2u, (tag & 7u) == 5u)
        | select(0u, 4u, (tag & (TAG_DOT_SPOT | 128u)) == (TAG_DOT_SPOT | 128u))
        | select(0u, 8u, (tag & (TAG_HILLS | TAG_SMOKE | TAG_DOT_SPOT | 128u)) == TAG_HILLS)
        | select(0u, 32u, (tag & 128u) != 0u
            || (tag & (TAG_WARM | TAG_HOT | TAG_SMOKE | TAG_DOT_SPOT)) == (TAG_WARM | TAG_HOT));
#ifdef DOT_COLOUR
    out.dot_colour = v.colour;
#endif
#ifdef VERTEX_WORD
    out.word = v.word & 0xFFFFu;
    out.warm |= select(0u, 16u, (v.word & WORD_FLAT_LIT) != 0u);
    out.warm |= ((v.word >> 17u) & 15u) << 6u;
    // A window wall's direction (`WORD_WALL_ANGLE`) in bits 16..26.
    out.warm |= ((v.word >> 21u) & 0x7FFu) << 16u;
#endif
    // Smoke volumes: the view ray and the sun in the plume's local space (inverse of the
    // scaled rotation m = diag(1/s^2) m^T).
    out.puff_pos = vec4<f32>(0.0);
    out.puff_eye = vec4<f32>(0.0);
    out.puff_sun = vec3<f32>(0.0);
    out.puff_scale = vec3<f32>(1.0);
    if ((tag & TAG_SMOKE) != 0u) {
        let m = mat3x3<f32>(world_from_local[0].xyz, world_from_local[1].xyz, world_from_local[2].xyz);
        let s2 = vec3<f32>(dot(m[0], m[0]), dot(m[1], m[1]), dot(m[2], m[2]));
        let o = world_from_local[3].xyz;
        let eye_local = (transpose(m) * (view.world_position - o)) / s2;
        // Plumes carry their noise offset in the tag's high bits (stays put when the floating
        // origin moves). puff_eye.w: 1 smoke, 2 smoke over a fire.
        out.puff_pos = vec4<f32>(v.position, f32(tag >> 16u) * 0.01);
        out.puff_eye = vec4<f32>(eye_local, select(1.0, 2.0, (tag & TAG_SMOKE_FIRE) != 0u));
        out.puff_sun = normalize((transpose(m) * globals.sun.xyz) / s2);
        out.puff_scale = sqrt(s2);
    }

    // Dot lights (dots.rs `dot_batch`: four vertices at each dot's centre) spread by their quad
    // corner to a constant size in this view's pixels (so the tracking camera's zoom doesn't
    // magnify them); in XR at most a 320x200 pixel's angular size.
#ifdef DOT_BATCH
    out.dot_uv = vec3<f32>(v.corner * 2.0, 1.0);
    if (((tag >> 7u) & 1u) == 1u) {
        var center = world.xyz;
        let spot = (tag & TAG_DOT_SPOT) != 0u;
        let spot_m = f32((tag >> 16u) & 0xFFu);
        if (spot) {
            center = center + normalize(view.world_position - center) * spot_m;
        }
        let corner = v.corner;
        let clip = position_world_to_clip(center);
        let vp = view.viewport.zw;
        var px = max(vp.y / 200.0 * DOT_SIZE, DOT_MIN_PIXELS);
        if (globals.dots.x > 0.0) {
            px = min(px, max(globals.dots.x * 0.5 * vp.y * view.clip_from_view[1][1], DOT_MIN_PIXELS));
        }
        if ((tag & 7u) == 5u) {
            let lamp = AIRCRAFT_DOT_METRES * 0.5 * vp.y * view.clip_from_view[1][1] / max(clip.w, 0.01);
            px = max(px, min(lamp, px * AIRCRAFT_DOT_MAX));
        }
        // Larger lights (PAPI): size multiplier in tag bits 8..10, or the size in tenths in
        // bits 24..30 (palette_material.rs `dot_size_tenths`).
        let tenths = (tag >> 24u) & 0x7Fu;
        if (tenths != 0u) {
            px = px * f32(tenths) / 10.0;
        } else {
            px = px * f32(((tag >> 8u) & 7u) + 1u);
        }
        // Only the blended dot materials (palette_material.rs `FLAG_DOT`): an opaque copy of a
        // light would show its whole quad.
        let bright = (tag & 7u) == 5u || tenths > 10u || ((tag >> 8u) & 7u) != 0u;
        let blended = (material.flags & 128u) != 0u;
        var glowing = false;
#ifdef DOT_COLOUR
        // Glowing lamps: their own size, and their brightness for the fragment (negative).
        if ((material.flags & FLAG_GLOW) != 0u) {
            glowing = true;
            let lamp = glow_lamp(v.colour, distance(center, view.world_position), 0.5 * vp.y * view.clip_from_view[1][1]);
            px = lamp.x;
            out.dot_uv.z = -lamp.y;
        }
#endif
        if (globals.light_info.z > 1.5 && bright && blended && !spot && !glowing) {
            px = px * NVG_HALO;
            out.dot_uv.z = NVG_HALO;
        }
        out.world_position = center;
        out.clip_position = clip + vec4<f32>(corner * px * 2.0 / vp * clip.w, 0.0, 0.0);
        var range = DOT_RANGE;
        if (globals.weather.z > 0.0) {
            range = min(range, globals.weather.z * DOT_VIS_FACTOR);
        }
        if ((tag & TAG_DOT_NEAR) != 0u) {
            range = range * DOT_NEAR;
        }
        let span = spot_m * 0.5 * vp.y * view.clip_from_view[1][1] / max(clip.w, 0.01);
        let dist = distance(center, view.world_position);
        let near = (tag & TAG_DOT_FAR) != 0u && dist < DOT_FAR_MIN;
        if (dist > range || near || (spot && span > SPOT_PIXELS) || (glowing && out.dot_uv.z > -1e-3)) {
            // Out of range (or a spotted aircraft close enough to see, or a far-only dot too
            // close): a degenerate quad outside the clip volume.
            out.clip_position = vec4<f32>(0.0, 0.0, -1.0, 1.0);
        }
    }
#endif

    // Per-object haze: fog = clamp((depth/512 in object units) - sizeClass, 0, 31) >> 1
    let size_class = (tag >> 3u) & 7u;
    let no_fog = (tag >> 6u) & 1u;
    var origin = world_from_local[3].xyz;
    if ((tag & TAG_VERTEX_HAZE) != 0u && (tag & (TAG_SMOKE | TAG_DOT_SPOT | 128u)) == 0u) {
        origin = world.xyz;
    }
    let forward = -view.world_from_view[2].xyz;
    let depth_m = max(dot(origin - view.world_position, forward), 0.0);
    let depth_units = depth_m * max(globals.weather.x, 1.0) / (globals.params.y * level_unit(tag & 7u));
    let d = i32(depth_units / 512.0) - i32(size_class);
    var fog = u32(clamp(d, 0, 31)) >> 1u;
    var fog_f = clamp(depth_units / 512.0 - f32(size_class), 0.0, 31.0) / 2.0;
    if (no_fog == 1u || globals.params.x == 0.0) {
        fog = 0u;
        fog_f = 0.0;
    }
    out.fog = fog;
    out.fog_f = fog_f;
    return out;
}

fn ring(arr: array<vec4<f32>, 4>, i: u32) -> f32 {
    let v = arr[i >> 2u];
    switch i & 3u {
        case 0u: { return v.x; }
        case 1u: { return v.y; }
        case 2u: { return v.z; }
        default: { return v.w; }
    }
}

// Band index (0..15) of the sky (elevation >= 0) or ground (below the horizon) for a view
// direction, using the classic screen-unit band boundaries: R = 256 * tan(angle).
fn band(dir: vec3<f32>) -> vec2<u32> {
    let horiz = max(length(dir.xz), 1e-6);
    let r = 256.0 * abs(dir.y) / horiz;
    var k = 0u;
    if (dir.y >= 0.0) {
        let sky = globals.sky_ring;
        for (var i = 1u; i < 16u; i++) {
            if (r >= ring(sky, i)) { k = i; }
        }
        return vec2<u32>(0x80u + k, 1u);
    }
    let g = globals.ground_g;
    for (var i = 1u; i < 16u; i++) {
        if (r >= ring(g, i)) { k = i; }
    }
    return vec2<u32>(0x70u + k, 0u);
}

// Position within the bands: (band colour base, band k, fraction towards band k + 1).
fn band_f(dir: vec3<f32>) -> vec3<f32> {
    let horiz = max(length(dir.xz), 1e-6);
    let r = 256.0 * abs(dir.y) / horiz;
    var arr = globals.ground_g;
    var base = 0x70u;
    if (dir.y >= 0.0) {
        arr = globals.sky_ring;
        base = 0x80u;
    }
    var k = 0u;
    for (var i = 1u; i < 16u; i++) {
        if (r >= ring(arr, i)) { k = i; }
    }
    var f = 0.0;
    if (k < 15u) {
        let a = ring(arr, k);
        let b = ring(arr, k + 1u);
        if (b > a) { f = clamp((r - a) / (b - a), 0.0, 1.0); }
    }
    return vec3<f32>(f32(base), f32(k), f);
}

fn pal(i: u32) -> vec3<f32> {
    if (recon_view()) {
        return globals.recon_palette[i & 0xFFu].rgb;
    }
    return globals.palette[i & 0xFFu].rgb;
}

// A recon camera's view (recon.rs): it sees by day and night alike, like a near infrared camera.
fn recon_view() -> bool {
    return all(abs(view.viewport.zw - globals.sky_night.zw) < vec2<f32>(0.5));
}

fn haze(idx: u32, fog: u32) -> u32 {
    if (idx < 0xA0u || fog == 0u) {
        return idx;
    }
    return max(idx & 0xF0u, idx - min(fog, idx));
}

// Modern mode: haze between ramp entries.
fn haze_smooth(idx: u32, fog: f32) -> vec3<f32> {
    if (idx < 0xA0u || fog <= 0.0) {
        return pal(idx);
    }
    let lo = haze(idx, u32(floor(fog)));
    let hi = haze(idx, u32(floor(fog)) + 1u);
    return mix(pal(lo), pal(hi), fract(fog));
}

// Moonlight's colour: a cool blue white.
const MOON_TINT: vec3<f32> = vec3<f32>(0.75, 0.85, 1.0);

fn sun_moon(n: vec3<f32>) -> vec3<f32> {
    return globals.ambient.rgb
        + vec3<f32>(globals.sun.w * max(dot(n, globals.sun.xyz), 0.0))
        + MOON_TINT * globals.moon.w * max(dot(n, globals.moon.xyz), 0.0);
}

// Modern mode: flat lighting by the sun and the moon (no shadows) on a face with normal n,
// relative to flat ground (the palette's brightness): faces turned to the light brighter,
// turned away darker. A low sun lights walls facing it far more than the ground: capped.
const SKY_LIGHT_MAX: f32 = 2.0;

fn sky_light(n: vec3<f32>) -> vec3<f32> {
    return min(sun_moon(n) / sun_moon(vec3<f32>(0.0, 1.0, 0.0)), vec3<f32>(SKY_LIGHT_MAX));
}

// The rolling hills' slopes (mesh tag `TAG_HILLS`): shaded as by the key light (the sun, or the
// moon when brighter) from its azimuth at a fixed height, so they show under a high sun too.
// Relative to flat ground (1 there): `HILL_SHADE` of the difference, within the limits.
const HILL_SHADE: f32 = 1.4;
const HILL_LIGHT_UP: f32 = 0.57;
const HILL_SHADE_MIN: f32 = 0.5;
const HILL_SHADE_MAX: f32 = 1.4;

fn hill_shade(face: vec3<f32>, dir: vec3<f32>) -> f32 {
    var n = normalize(face);
    if (dot(n, dir) > 0.0) { n = -n; }
    let key = select(globals.moon.xyz, globals.sun.xyz, globals.sun.w >= globals.moon.w);
    var h = vec2<f32>(key.x, key.z);
    // Overhead: from the north west.
    if (length(h) < 1e-3) { h = vec2<f32>(-1.0, -1.0); }
    h = normalize(h) * sqrt(1.0 - HILL_LIGHT_UP * HILL_LIGHT_UP);
    let f = dot(n, vec3<f32>(h.x, HILL_LIGHT_UP, h.y)) / HILL_LIGHT_UP;
    return clamp(1.0 + HILL_SHADE * (f - 1.0), HILL_SHADE_MIN, HILL_SHADE_MAX);
}

// Modern mode: flat lighting of a face. `face`: the face normal from the position's
// derivatives (`Deriv`).
fn shade(face: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    var n = normalize(face);
    if (dot(n, dir) > 0.0) { n = -n; }
    return sky_light(n);
}

// Specular highlights from the local lights: strength (x) and Blinn-Phong exponent (y).
const GLOSS_AIRCRAFT: vec2<f32> = vec2<f32>(0.6, 48.0);
const GLOSS_OBJECT: vec2<f32> = vec2<f32>(0.2, 16.0);
const GLOSS_GROUND: vec2<f32> = vec2<f32>(0.15, 24.0);

// Lighting options: the palette colour lit by the sun (with shadow maps) and local lights,
// using the flat face normal. Unlit parts keep roughly the classic brightness.
// `dark`: the night's darkness on everything but the local lights; `gloss`: their specular
// highlights (`GLOSS_*`).
fn lit(rgb: vec3<f32>, world: vec3<f32>, face: vec3<f32>, dir: vec3<f32>, clip: vec4<f32>, flags: u32, dark: f32, gloss: vec2<f32>) -> vec3<f32> {
    var n = normalize(face);
    if (dot(n, dir) > 0.0) { n = -n; }
    return lit_surface(rgb, world, n, dir, clip, flags, dark, gloss);
}

// The key light's shadow at `world` (1 lit, 0 shadowed): Bevy's cascade lookup cut down to
// one cascade and one sample (its own, with cascade blending and the filter and PCSS paths,
// costs the Steam Frame about 8 ms). Classic shaders: hard edged (the nearest texel); modern:
// the hardware's 2x2 comparison filter.
fn sun_shadow(world: vec3<f32>, n: vec3<f32>, view_z: f32, hard: bool) -> f32 {
    // The index and the cascade bounds come from the CPU (lighting.rs): indexing Bevy's light
    // array in a loop costs the Steam Frame most of the time the lookup takes.
    let z = vec4<f32>(-view_z);
    let ci = u32(dot(select(vec4<f32>(0.0), vec4<f32>(1.0), z >= globals.shadow_bounds), vec4<f32>(1.0)));
    if (ci >= u32(globals.shadow.y)) {
        return 1.0;
    }
    let p = world + globals.shadow.z * globals.shadow_texel[ci] * n
        + globals.shadow.w * globals.shadow_light.xyz;
    let c = globals.shadow_cascades[ci] * vec4<f32>(p, 1.0);
    let ndc = c.xyz / c.w;
    if (any(abs(ndc.xy) > vec2<f32>(1.0)) || ndc.z < 0.0 || ndc.z > 1.0) {
        return 1.0;
    }
    let uv = ndc.xy * vec2<f32>(0.5, -0.5) + 0.5;
    // The key light is the only one with shadow maps: its cascades are the first layers.
    let layer = i32(ci);
#ifdef NO_ARRAY_TEXTURES_SUPPORT
    // WebGL2 (GLSL can't load from a depth texture): the comparison filter, hard edged or not.
    return textureSampleCompareLevel(directional_shadow_textures, directional_shadow_textures_comparison_sampler, uv, ndc.z);
#else
    if (hard) {
        let size = vec2<f32>(textureDimensions(directional_shadow_textures));
        let d = textureLoad(directional_shadow_textures, vec2<i32>(uv * size), layer, 0);
        return select(0.0, 1.0, ndc.z >= d);
    }
    return textureSampleCompareLevel(directional_shadow_textures, directional_shadow_textures_comparison_sampler, uv, layer, ndc.z);
#endif
}

fn lit_surface(rgb: vec3<f32>, world: vec3<f32>, n: vec3<f32>, dir: vec3<f32>, clip: vec4<f32>, flags: u32, dark: f32, gloss: vec2<f32>) -> vec3<f32> {
    var light = globals.ambient.rgb;
    var spec = vec3<f32>(0.0);
    if ((flags & LIGHT_SHADOWS) != 0u) {
        // The shadow maps are the key light's: the sun, or the moon when it is the brighter.
        let moon_key = (flags & LIGHT_MOON_KEY) != 0u;
        let sun_l = vec3<f32>(globals.sun.w * max(dot(n, globals.sun.xyz), 0.0));
        let moon_l = MOON_TINT * globals.moon.w * max(dot(n, globals.moon.xyz), 0.0);
        var shadow = 1.0;
#ifdef PALETTE_SHADOWS
        if (globals.shadow.y > 0.0 && !offscreen_view() && dot(select(sun_l, moon_l, moon_key), vec3<f32>(1.0)) > 0.0) {
            let view_z = (view.view_from_world * vec4<f32>(world, 1.0)).z;
            shadow = sun_shadow(world, n, view_z, !MODERN);
        }
#endif
        light += select(sun_l * shadow, sun_l, moon_key) + select(moon_l, moon_l * shadow, moon_key);
        // Sunlit flat ground keeps the palette's brightness, shadows go darker (lighting.rs
        // sets the ambient light for that).
        light = min(light / sun_moon(vec3<f32>(0.0, 1.0, 0.0)), vec3<f32>(SKY_LIGHT_MAX));
    } else if (MODERN) {
        light = sky_light(n);
    } else {
        light = vec3<f32>(1.0);
    }
    light *= 1.0 - dark;
#ifdef PALETTE_LOCAL
    if ((flags & LIGHT_LOCAL) != 0u) {
        let count = min(u32(globals.light_info.x), MAX_LOCAL_LIGHTS);
        let to_eye = -normalize(dir);
        for (var i = 0u; i < count; i++) {
            let p = globals.local_lights[2u * i];
            let c = globals.local_lights[2u * i + 1u];
            let to_light = p.xyz - world;
            let d = length(to_light);
            if (d < p.w) {
                let l = to_light / max(d, 1e-3);
                var att = (1.0 - d / p.w) * (1.0 - d / p.w);
                // A spotlight (lighting.rs `encode_spot`) lights its cone only.
                if (c.w > 0.5) {
                    att *= smoothstep(SPOT_OUTER, SPOT_INNER, dot(-l, spot_direction(c.w)));
                }
                let facing = dot(n, l);
                light += c.rgb * att * (max(facing, 0.0) * 0.7 + 0.3);
                // Blinn-Phong highlight in the light's own colour.
                if (facing > 0.0) {
                    let h = normalize(l + to_eye);
                    spec += c.rgb * att * gloss.x * pow(max(dot(n, h), 0.0), gloss.y);
                }
            }
        }
    }
#endif
    return rgb * light + spec;
}

// A spotlight's cone: cosines of the half angles where it starts to fade and where it is dark.
const SPOT_INNER: f32 = 0.990;
const SPOT_OUTER: f32 = 0.966;

// A spotlight's direction from its colour's w (lighting.rs `encode_spot`): 1 + u + 1024 v, the
// direction folded onto an octahedron.
fn spot_direction(w: f32) -> vec3<f32> {
    let k = w - 1.0;
    let v = floor(k / 1024.0);
    let o = vec2<f32>(k - v * 1024.0, v) / 1023.0 * 2.0 - 1.0;
    var n = vec3<f32>(o.x, 1.0 - abs(o.x) - abs(o.y), o.y);
    let t = max(-n.y, 0.0);
    n.x += select(t, -t, n.x >= 0.0);
    n.z += select(t, -t, n.z >= 0.0);
    return normalize(n);
}

// Cloud texture: Perlin noise in world space. The lattice wraps every NOISE_PERIOD metres
// horizontally (the floating origin is handed over modulo that), so the pattern stays put
// when the origin moves.
const NOISE_PERIOD: f32 = 81920.0;
const NOISE_CELL: f32 = 1280.0;
const NOISE_OCTAVES: u32 = 5u;

fn noise_hash(p: vec3<i32>) -> u32 {
    var h = (u32(p.x) * 0x8DA6B343u) ^ (u32(p.y) * 0xD8163841u) ^ (u32(p.z) * 0xCB1AB31Fu) ^ u32(globals.noise.z);
    h ^= h >> 16u;
    h *= 0x7FEB352Du;
    h ^= h >> 15u;
    h *= 0x846CA68Bu;
    h ^= h >> 16u;
    return h;
}

// Gradient at a lattice corner (Perlin's 12 cube-edge directions) dotted with the offset.
fn noise_grad(h: u32, f: vec3<f32>) -> f32 {
    let g = h % 12u;
    let u = select(f.y, f.x, g < 8u);
    let v = select(select(f.z, f.x, g == 12u || g == 14u), f.y, g < 4u);
    return select(u, -u, (g & 1u) != 0u) + select(v, -v, (g & 2u) != 0u);
}

fn perlin(p: vec3<f32>, period: i32) -> f32 {
    let i = vec3<i32>(floor(p));
    let f = fract(p);
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    var c: array<f32, 8>;
    for (var k = 0u; k < 8u; k++) {
        let o = vec3<i32>(i32(k & 1u), i32((k >> 1u) & 1u), i32((k >> 2u) & 1u));
        let q = i + o;
        let w = vec3<i32>(((q.x % period) + period) % period, q.y, ((q.z % period) + period) % period);
        c[k] = noise_grad(noise_hash(w), f - vec3<f32>(o));
    }
    let x0 = mix(mix(c[0], c[1], u.x), mix(c[2], c[3], u.x), u.y);
    let x1 = mix(mix(c[4], c[5], u.x), mix(c[6], c[7], u.x), u.y);
    return mix(x0, x1, u.z);
}

// About -1..1; octaves finer than a few pixels fade out with distance.
// --- Stratus coverage (same field as stealth_mission::Weather::stratus_field) ------------------

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

// The field at a render-space position, in map units (64 per km, y southward).
fn stratus_field(world: vec3<f32>) -> f32 {
    let map_unit = 32.0 * globals.params.y;
    let east = world.x + globals.stratus.z;
    let north = -(world.z + globals.stratus.w);
    let m = vec2<f32>(east / map_unit, 32768.0 - north / map_unit);
    let seed = u32(globals.stratus.y);
    return 0.6 * value_noise(m / 512.0, seed) + 0.3 * value_noise(m / 160.0, seed + 1u) + 0.1 * value_noise(m / 48.0, seed + 2u);
}

// Stratus coverage 0..1 at a position (soft edge just above the threshold). Far away the
// gaps would alias into speckle: there it fades to the layer's average coverage.
fn stratus_cover(world: vec3<f32>) -> f32 {
    if (globals.clouds.z == 0.0) { return 0.0; }
    let thr = globals.stratus.x;
    let local = smoothstep(thr, thr + 0.035, stratus_field(world));
    let d = length(world - view.world_position);
    return mix(local, globals.noise.w, smoothstep(25000.0, 70000.0, d));
}

fn cloud_noise(world: vec3<f32>, dist: f32) -> f32 {
    let p = world + vec3<f32>(globals.noise.x, 0.0, globals.noise.y);
    var sum = 0.0;
    var amp = 0.6;
    var cell = NOISE_CELL;
    for (var o = 0u; o < NOISE_OCTAVES; o++) {
        let fade = 1.0 - smoothstep(cell * 15.0, cell * 40.0, dist);
        sum += amp * fade * perlin(p / cell, i32(NOISE_PERIOD / cell));
        amp *= 0.5;
        cell *= 0.5;
    }
    return sum;
}

// Cloud colour at `world` on a face with normal n (facing the viewer) seen `dist` metres
// away: tops light, undersides dark, sun side lighter, textured by noise, hazing to the
// horizon colour with distance.
fn cloud_color(noise: f32, n: vec3<f32>, dist: f32, modern: bool) -> vec3<f32> {
    let shading = (1.0 - n.y) * 4.0 + 4.0 - 3.0 * max(dot(n, globals.sun.xyz), 0.0);
    return cloud_ramp(shading + 4.5 * noise, dist, modern);
}

// Grey ramp entry k (0 lightest), hazed with distance.
fn cloud_ramp(shade: f32, dist: f32, modern: bool) -> vec3<f32> {
    let k = clamp(shade, 0.0, 14.0);
    let fog = clamp(dist / max(globals.weather.z, 1.0) * 15.0, 0.0, 15.0);
    let f = max(k - fog, 0.0);
    if (modern) {
        return mix(pal(CLOUD_RAMP + u32(floor(f))), pal(CLOUD_RAMP + u32(floor(f)) + 1u), fract(f));
    }
    return pal(CLOUD_RAMP + u32(round(f)));
}

// 4x4 ordered dither threshold (classic mode's soft cloud edges).
// Angular radius of the sun's disc (drawn about three times its real size).
const SUN_RADIUS: f32 = 0.014;

// The sun's colour at an elevation (sine): orange red at the horizon, warm white up high.
fn sun_colour(h: f32) -> vec3<f32> {
    return mix(vec3<f32>(1.0, 0.42, 0.1), vec3<f32>(1.0, 0.93, 0.78), smoothstep(-0.02, 0.3, h));
}

// Modern shaders, continuous time of day: the glare of the sun over everything in its
// direction, a bright core and wide soft halos, dimmed by an overcast and gone below the horizon.
fn sun_glare(dir: vec3<f32>) -> vec3<f32> {
    if (globals.sky_sun.w == 0.0 || !MODERN) {
        return vec3<f32>(0.0);
    }
    let s = normalize(globals.sky_sun.xyz);
    let up = smoothstep(-0.03, 0.02, s.y);
    if (up <= 0.0) {
        return vec3<f32>(0.0);
    }
    let c = max(dot(normalize(dir), s), 0.0);
    let strength = clamp(globals.sun.w / 0.3, 0.0, 1.0) * up;
    let glow = pow(c, 900.0) * 0.9 + pow(c, 60.0) * 0.16 + pow(c, 8.0) * 0.05;
    return sun_colour(s.y) * glow * strength;
}

// Angular radius of the moon's disc (drawn about five times its real size).
const MOON_RADIUS: f32 = 0.022;
const MOON_COLOR: vec3<f32> = vec3<f32>(0.82, 0.80, 0.74);
// The star catalogue's cells (stars.rs `ROWS`, `EQUATOR_CELLS`), in a texture (stars.rs
// `grid`). A star is a point STAR_PIXELS pixels in radius (at least STAR_MIN_RADIUS radians),
// up to STAR_GROW times that for the brightest.
const STAR_ROWS: f32 = 512.0;
const STAR_COLS: f32 = 1020.0;
const STAR_PIXELS: f32 = 0.55;
const STAR_MIN_RADIUS: f32 = 0.00015;
const STAR_GROW: f32 = 2.0;
const PI: f32 = 3.14159265;
const TAU: f32 = 6.28318531;

@group(#{MATERIAL_BIND_GROUP}) @binding(2) var star_grid: texture_2d<f32>;
// The ocean (ocean.rs): the cascades' displacement and foam (Dx, Dy, Dz, foam) and
// derivatives (dDy/dx, dDy/dz, dDx/dx, dDz/dz), one layer each; the open water round the
// viewpoint (r: water, g: how high the waves rise).
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var ocean_disp: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var ocean_deriv: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var ocean_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var ocean_mask: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var ocean_mask_sampler: sampler;

// The cascades' patches (metres; ocean/spectrum.rs `CASCADE_LENGTHS`) and the waves' mesh
// (ocean/grid.rs): the finest level's cell, cells from a level's centre to its edge, levels.
const OCEAN_LENGTHS: vec3<f32> = vec3<f32>(640.0, 80.0, 10.0);
const OCEAN_CELL0: f32 = 0.5;
const OCEAN_HALF: f32 = 32.0;
const OCEAN_LEVELS: u32 = 8u;

fn ocean_level_centre(l: u32) -> vec2<f32> {
    let v = globals.ocean_levels[l >> 1u];
    return select(v.zw, v.xy, (l & 1u) == 0u);
}

// The open water mask at a point (from the floating origin): water (x), the waves' rise (y).
fn ocean_mask_at(p: vec2<f32>) -> vec2<f32> {
    if (globals.ocean_mask.w == 0.0) {
        return vec2<f32>(0.0);
    }
    let uv = (p - globals.ocean_mask.xy) / globals.ocean_mask.z;
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
        return vec2<f32>(0.0);
    }
    return textureSampleLevel(ocean_mask, ocean_mask_sampler, uv, 0.0).rg;
}

// A cascade's mip level for a footprint of `foot` metres.
fn ocean_lod(foot: f32, len: f32) -> f32 {
    let n = f32(textureDimensions(ocean_deriv).x);
    return log2(max(foot * n / len, 1.0));
}

// The waves' normal and foam at a point of the flat sea (from the floating origin), seen with
// a footprint of `foot` metres: the cascades' slopes (the horizontal displacement's
// stretching taken out), let down near the shore.
fn ocean_surface(p: vec2<f32>, foot: f32) -> vec4<f32> {
    let w = p + globals.noise.xy;
    var d = vec4<f32>(0.0);
    var foam = 0.0;
    for (var c = 0u; c < 3u; c = c + 1u) {
        let len = OCEAN_LENGTHS[c];
        let lod = ocean_lod(foot, len);
        d += textureSampleLevel(ocean_deriv, ocean_sampler, w / len, c, lod);
        if (c < 2u) {
            foam += textureSampleLevel(ocean_disp, ocean_sampler, w / len, c, lod).w;
        }
    }
    var rise = 1.0;
    if (globals.ocean_mask.w != 0.0) {
        let uv = (p - globals.ocean_mask.xy) / globals.ocean_mask.z;
        if (all(uv >= vec2<f32>(0.0)) && all(uv <= vec2<f32>(1.0))) {
            rise = textureSampleLevel(ocean_mask, ocean_mask_sampler, uv, 0.0).g;
        }
    }
    let slope = vec2<f32>(d.x / max(1.0 + d.z, 0.2), d.y / max(1.0 + d.w, 0.2)) * rise;
    return vec4<f32>(normalize(vec3<f32>(-slope.x, 1.0, -slope.y)), clamp(foam * rise, 0.0, 1.0));
}

#ifdef OCEAN_GRID
// The waves' mesh (ocean/grid.rs): a vertex (cell x, level, cell z) round its level's centre,
// morphed onto the coarser level's grid near the level's edge, displaced by the waves. The
// undisplaced point (xy), the level (z) and the waves' rise (w) for the fragment.
fn ocean_vertex(cell: vec3<f32>) -> array<vec4<f32>, 2> {
    let l = u32(cell.y + 0.5);
    let s = OCEAN_CELL0 * exp2(f32(l));
    let c = ocean_level_centre(l);
    let edge = max(abs(cell.x), abs(cell.z)) / OCEAN_HALF;
    var p = c + cell.xz * s;
    // Sampled as finely as the cell, the coarser level's mip at the edge (where the vertices
    // are the coarser level's): both levels see the same heights there.
    var foot = s;
    if (l + 1u < OCEAN_LEVELS) {
        let k = clamp((edge - 0.75) / 0.2, 0.0, 1.0);
        let g = cell.xz * 0.5;
        p -= (g - floor(g)) * 2.0 * s * k;
        foot = s * (1.0 + k);
    }
    var rise = ocean_mask_at(p).y * globals.ocean.y;
    if (l + 1u == OCEAN_LEVELS) {
        // The outermost level settles to the flat sea round it.
        rise *= 1.0 - smoothstep(0.8, 1.0, edge);
    }
    let w = p + globals.noise.xy;
    let n = f32(textureDimensions(ocean_disp).x);
    var d = vec3<f32>(0.0);
    for (var i = 0u; i < 3u; i = i + 1u) {
        let len = OCEAN_LENGTHS[i];
        let lod = log2(max(foot * n / len, 1.0));
        d += textureSampleLevel(ocean_disp, ocean_sampler, w / len, i, lod).xyz;
    }
    d *= rise;
    return array<vec4<f32>, 2>(
        vec4<f32>(p.x + d.x, d.y, p.y + d.z, 1.0),
        vec4<f32>(p, f32(l), rise),
    );
}
#endif

// Cells in a row of the star grid (stars.rs `row_cells`).
fn star_row_cells(row: f32) -> f32 {
    return max(floor(STAR_COLS * cos(((row + 0.5) / STAR_ROWS - 0.5) * PI) + 0.5), 1.0);
}

// The light of the star of cell `col` of `n` in grid row `row` at catalogue direction `e`:
// a point `r` radians in radius (more for the brightest), seen down to magnitude `lim`, `ext`
// magnitudes fainter through the air.
fn star_light(row: f32, col: f32, n: f32, e: vec3<f32>, r: f32, lim: f32, ext: f32) -> vec3<f32> {
    let t = textureLoad(star_grid, vec2<i32>(i32(col), i32(row)), 0);
    let mag = (t.b * 255.0 - 1.0) / 25.0 - 2.0 + ext;
    if (t.b == 0.0 || mag > lim + 0.5) {
        return vec3<f32>(0.0);
    }
    let ra = (col + t.r) / n * TAU;
    let dec = (row + t.g) / STAR_ROWS * PI - 0.5 * PI;
    let s = vec3<f32>(cos(dec) * cos(ra), cos(dec) * sin(ra), sin(dec));
    // A hard edged point: unfiltered, it pops between pixels as the view moves, so the stars
    // twinkle.
    let grow = 1.0 + (STAR_GROW - 1.0) * clamp((1.0 - mag) / 2.5, 0.0, 1.0);
    if (length(e - s) >= r * grow) {
        return vec3<f32>(0.0);
    }
    // Its colour by its B-V index, blue white to orange, paler than in a telescope.
    let bv = t.a * 255.0 / 80.0 - 0.5;
    var c = mix(vec3<f32>(0.72, 0.82, 1.0), vec3<f32>(1.0, 0.97, 0.92), smoothstep(-0.3, 0.4, bv));
    c = mix(c, vec3<f32>(1.0, 0.78, 0.55), smoothstep(0.4, 1.6, bv));
    // Less contrast than the sky's (2.5 times the light a magnitude): the faint ones show.
    return c * min(pow(10.0, -0.3 * (mag - 1.0)), 1.0) * smoothstep(lim + 0.5, lim - 0.5, mag);
}

// The stars' light in direction `nd` (world, above the horizon), visibility `stars`, `px`
// radians a pixel: the catalogue's stars where they stand (`Globals::star_frame`).
fn star_field(nd: vec3<f32>, stars: f32, px: f32) -> vec3<f32> {
    let f = globals.star_frame;
    let e = mat3x3<f32>(f[0].xyz, f[1].xyz, f[2].xyz) * nd;
    let up = (asin(clamp(e.z, -1.0, 1.0)) / PI + 0.5) * STAR_ROWS;
    let row = clamp(floor(up), 0.0, STAR_ROWS - 1.0);
    let n = star_row_cells(row);
    let across = fract(atan2(e.y, e.x) / TAU) * n;
    let col = min(floor(across), n - 1.0);
    // The faintest seen: magnitude 6 on a dark night, 3.3 under a full moon, none by day.
    let dim = 1.0 - stars;
    let lim = 6.0 - 7.5 * dim * dim;
    // Extinction: 0.2 magnitudes for each air mass more than overhead.
    let ext = 0.2 * (1.0 / max(nd.y, 0.03) - 1.0);
    let r = max(px * STAR_PIXELS, STAR_MIN_RADIUS);
    var light = star_light(row, col, n, e, r, lim, ext);
    // A star near its cell's edge reaches over it: the neighbour's star nearest this pixel too.
    let reach = r * STAR_GROW;
    let side = reach * n / (TAU * max(sqrt(1.0 - e.z * e.z), 1e-3));
    let fx = across - col;
    if (fx < side) {
        light += star_light(row, select(col - 1.0, n - 1.0, col == 0.0), n, e, r, lim, ext);
    } else if (fx > 1.0 - side) {
        light += star_light(row, select(col + 1.0, 0.0, col + 1.0 >= n), n, e, r, lim, ext);
    }
    let fy = up - row;
    let high = reach * STAR_ROWS / PI;
    var row2 = -1.0;
    if (fy < high && row > 0.0) {
        row2 = row - 1.0;
    } else if (fy > 1.0 - high && row < STAR_ROWS - 1.0) {
        row2 = row + 1.0;
    }
    if (row2 >= 0.0) {
        let n2 = star_row_cells(row2);
        let col2 = min(floor(fract(atan2(e.y, e.x) / TAU) * n2), n2 - 1.0);
        light += star_light(row2, col2, n2, e, r, lim, ext);
    }
#ifdef PALETTE_SKY_FX
    // Low stars twinkle (through more air), each at its own pace.
    let low = 1.0 - smoothstep(0.05, 0.5, nd.y);
    let h = hash2(vec2<f32>(col, row));
    light *= max(1.0 + globals.sky_fx.z * low * 0.7 * sin(globals.dots.y * (7.0 + 9.0 * h) + h * 400.0), 0.0);
#endif
    // Gone in the haze at the horizon.
    return light * smoothstep(0.0, 0.05, nd.y);
}

fn hash2(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

// Night sky extras (`Globals::sky_fx`). The northern lights: curtains of green, red at their
// tops, hanging over the northern sky with rays running through them, slowly moving.
fn aurora(nd: vec3<f32>) -> vec3<f32> {
    let s = globals.sky_fx.x;
    if (s <= 0.0 || nd.y <= 0.0) {
        return vec3<f32>(0.0);
    }
    let az = atan2(nd.x, -nd.z);
    let north = exp(-az * az / 1.2);
    let t = globals.dots.y;
    let above = asin(nd.y) - (0.18 + 0.06 * sin(az * 3.0 + t * 0.05) + 0.03 * sin(az * 7.0 - t * 0.08));
    if (above < -0.03 || north < 0.01) {
        return vec3<f32>(0.0);
    }
    let height = 0.25 + 0.08 * sin(az * 5.0 + t * 0.03);
    let curtain = smoothstep(-0.03, 0.01, above) * exp(-max(above, 0.0) / height * 2.2);
    let rays = 0.6 + 0.4 * sin(az * 90.0 + 3.0 * sin(az * 11.0 + t * 0.2) + t * 0.4) * (0.4 + 0.6 * sin(az * 23.0 - t * 0.15));
    let c = mix(vec3<f32>(0.02, 0.28, 0.11), vec3<f32>(0.18, 0.02, 0.07), smoothstep(0.08, 0.3, above));
    return c * curtain * rays * north * s;
}

// A shooting star now and then (`Globals::sky_fx` y): a short bright streak across the sky,
// placed and timed by the clock: in one period of `METEOR_PERIOD` s out of 1 / `METEOR_CHANCE`
// (some every two minutes).
const METEOR_PERIOD: f32 = 7.0;
const METEOR_LIFE: f32 = 0.7;
const METEOR_CHANCE: f32 = 0.05;
fn meteor(nd: vec3<f32>, px: f32) -> f32 {
    if (globals.sky_fx.y <= 0.0 || nd.y <= 0.0) {
        return 0.0;
    }
    let t = globals.dots.y;
    let k = floor(t / METEOR_PERIOD);
    let p = (t - k * METEOR_PERIOD) / METEOR_LIFE;
    if (p > 1.0 || hash2(vec2<f32>(k, 3.7)) > METEOR_CHANCE) {
        return 0.0;
    }
    let az = hash2(vec2<f32>(k, 1.1)) * 6.2831853;
    let el = 0.35 + 0.6 * hash2(vec2<f32>(k, 2.3));
    let start = vec3<f32>(cos(el) * sin(az), sin(el), -cos(el) * cos(az));
    let east = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), start));
    let up = cross(start, east);
    let a = hash2(vec2<f32>(k, 5.9)) * 6.2831853;
    // Falling, mostly: the streak heads downwards.
    let dir = normalize(cos(a) * east - (0.6 + 0.4 * abs(sin(a))) * up) * 0.25;
    let head = normalize(start + dir * p);
    let tail = normalize(start + dir * max(p - 0.35, 0.0));
    let ab = head - tail;
    let along = clamp(dot(nd - tail, ab) / max(dot(ab, ab), 1e-8), 0.0, 1.0);
    let d = length(nd - tail - ab * along);
    let w = max(px * 0.8, 0.0006);
    return (1.0 - smoothstep(w * 0.5, w * 1.5, d)) * along * (1.0 - 0.6 * p);
}


// Continuous time of day: the sun, stars and the moon over the sky colour (`nd` normalised, above the
// horizon). The moon is lit from the true sun direction, so its phase shows by itself. `px`:
// radians per pixel (`Deriv`).
fn night_sky(bg_in: vec3<f32>, nd: vec3<f32>, px: f32) -> vec3<f32> {
    var bg = bg_in;
    let daylight = clamp(globals.light_info.y, 0.0, 1.0);
    let stars = globals.sky_moon.w;
    // Only in the night variant (palette_material.rs `FX_STARS`, on while the stars show): the
    // stars' code costs the background half its waves on the Steam Frame.
#ifdef PALETTE_STARS
    if (stars > 0.0) {
        bg += star_field(nd, stars, px);
#ifdef PALETTE_SKY_FX
        bg += (aurora(nd) + vec3<f32>(1.0, 0.95, 0.85) * meteor(nd, px)) * stars;
#endif
    }
#endif
    // The sun's disc, a soft edge against the sky.
    let sun = normalize(globals.sky_sun.xyz);
    if (sun.y > -SUN_RADIUS) {
        let a = acos(clamp(dot(nd, sun), -1.0, 1.0));
        let edge = 1.0 - smoothstep(SUN_RADIUS - px, SUN_RADIUS + px, a);
        bg = mix(bg, sun_colour(sun.y) * 1.6, edge);
    }
    let m = normalize(globals.sky_moon.xyz);
    let c = dot(nd, m);
    if (m.y > -MOON_RADIUS && c > cos(MOON_RADIUS * 8.0)) {
        // The disc as a sphere seen from the earth: its normal at this pixel.
        let o = (nd - m * c) / MOON_RADIUS;
        let r2 = dot(o, o);
        let fw = px / MOON_RADIUS;
        let edge = 1.0 - smoothstep(1.0 - fw, 1.0 + fw, sqrt(r2));
        if (edge > 0.0) {
            let n = normalize(o - m * sqrt(max(1.0 - r2, 0.0)));
            let lit = smoothstep(-0.05, 0.05, dot(n, normalize(globals.sky_sun.xyz)));
            // Earthshine on the dark side; faint by day.
            let shade = mix(0.04, 1.0, lit) * mix(1.0, 0.35, daylight);
            let disc = MOON_COLOR * shade;
            bg = mix(bg, max(bg, disc), edge);
        }
        // A soft halo around the lit moon at night, fading out before the edge of this test.
        let a = acos(min(c, 1.0)) / MOON_RADIUS;
        let halo = exp(-a * a * 0.25) * smoothstep(8.0, 5.0, a) * 0.12 * (1.0 - daylight);
        bg += MOON_COLOR * halo * globals.sky_moon.w;
    }
    return bg;
}

// A night without moonlight (continuous time of day): sky, ground and clouds go black, only
// lights and stars show, aircraft barely; the horizon keeps a deep red glow. 0 (moonlit) .. 1.
// Never for the recon cameras.
fn night_dark() -> f32 {
    if (recon_view()) {
        return 0.0;
    }
    return globals.sky_night.x;
}

// The red glow over the horizon of a dark night by sky band (8-bit sRGB red).
fn night_red(k: u32) -> vec3<f32> {
    var red = array<f32, 10>(64.0, 48.0, 36.0, 28.0, 24.0, 16.0, 12.0, 8.0, 6.0, 3.0);
    if (k >= 10u) { return vec3<f32>(0.0); }
    let r = red[k] / 255.0 * globals.sky_night.y;
    return pow(min(vec3<f32>(r, 0.0, r * 0.07), vec3<f32>(1.0)), vec3<f32>(2.2));
}

// The sky or ground colour `bg` darkened for the night (`dir`: view direction; sky band `b`
// as from `band_f`, its fraction ignored in classic mode).
fn night_background(bg: vec3<f32>, dir: vec3<f32>, b: vec3<f32>, modern: bool) -> vec3<f32> {
    let dark = night_dark();
    if (dark <= 0.0) {
        return bg;
    }
    var c = bg * (1.0 - dark);
    if (dir.y >= 0.0) {
        let k = u32(b.y);
        var red = night_red(k);
        if (modern) {
            red = mix(red, night_red(k + 1u), b.z);
        }
        c += red * dark;
    }
    return c;
}

fn bayer4(p: vec2<u32>) -> f32 {
    var m = array<f32, 16>(0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
    return (m[(p.y & 3u) * 4u + (p.x & 3u)] + 0.5) / 16.0;
}

// Retro output (`Globals::retro`): VGA colours, the 6-bit DAC's 64 steps per channel with a
// 4x4 ordered dither (x); scanlines, every other line of a CRT's darker (y).
fn retro(rgb: vec3<f32>, frag: vec2<f32>) -> vec3<f32> {
    var c = rgb;
    if (globals.retro.x > 0.0) {
        let s = pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
        let q = floor(s * 63.0 + bayer4(vec2<u32>(frag))) / 63.0;
        c = pow(min(q, vec3<f32>(1.0)), vec3<f32>(2.2));
    }
    if (globals.retro.y > 0.0) {
        let period = max(floor(view.viewport.w / 300.0), 2.0);
        c *= select(1.0, SCANLINE, fract(frag.y / period) >= 0.5);
    }
    return c;
}
const SCANLINE: f32 = 0.6;

// Inside a cloud: everything but the cockpit fades into it. Under an overcast or in a storm
// the day turns greyer and dimmer.
fn whiteout(rgb: vec3<f32>, dist: f32) -> vec3<f32> {
    let g = globals.weather.w;
    let grey = mix(rgb, vec3<f32>(dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722))), 0.4 * g) * (1.0 - 0.3 * g);
    let w = globals.clouds.w * smoothstep(3.0, 60.0, dist);
    return mix(grey, pal(CLOUD_RAMP + 2u) * (1.0 - night_dark()), w);
}

// Ground mist's density: metres of it the view crosses for 1 - 1/e of the way into it.
const MIST_DEPTH: f32 = 1200.0;

// Ground mist (modern shading, dawn, dusk and night): a thin layer over the ground, the
// valleys and the sea, in the horizon's haze colour. Seen from above it is barely there below
// the aircraft and thick towards the horizon, where the view crosses a long way of it; the
// hills stand out of it.
fn ground_mist(rgb: vec3<f32>, world: vec3<f32>, dist: f32) -> vec3<f32> {
    let s = globals.atmos.x;
    let top = globals.atmos.w + globals.atmos.y;
    if (s <= 0.0 || world.y >= top) {
        return rgb;
    }
    let eye_y = view.world_position.y;
    var path = dist;
    if (eye_y > top) {
        let up = (eye_y - world.y) / max(dist, 1.0);
        path = min(dist, (top - world.y) / max(up, 1e-3));
    }
    let a = s * (1.0 - exp(-path / MIST_DEPTH));
    return mix(rgb, pal(CLOUD_RAMP) * (1.0 - night_dark()), a);
}

const SHADOW_GRID: f32 = 12.0;

// Modern or classic shading (`Globals::params` w), compiled in: the classic paths' code costs
// the Steam Frame's GPU even skipped.
#ifdef PALETTE_MODERN
const MODERN: bool = true;
#else
const MODERN: bool = false;
#endif

// Cloud shadows: the shade of the clouds above `world` where the key light's ray from it meets
// their base (the sun's, or the moon's when it is the brighter): the stratus deck's (unless a
// full overcast hides it everywhere), and the cumulus and storm towers' (round, soft edged)
// from the grid of them round the eye, fading to their average shade past it.
fn cloud_shadow(world: vec3<f32>) -> f32 {
    let s = globals.atmos.z;
    let moon_key = (u32(globals.ambient.w) & LIGHT_MOON_KEY) != 0u;
    let sun = select(globals.sun.xyz, globals.moon.xyz, moon_key);
    if (s <= 0.0 || sun.y < 0.05) {
        return 1.0;
    }
    var shade = 0.0;
    let base = globals.clouds.x;
    if (globals.clouds.z != 0.0 && world.y < base) {
        shade = globals.shadow_far.y * stratus_cover(world + sun * ((base - world.y) / sun.y));
    }
    let g = globals.shadow_grid;
    if (globals.shadow_far.x > 0.0 && world.y < g.w) {
        let p = world.xz + sun.xz * ((g.w - world.y) / sun.y);
        let q = (p - g.xy) / g.z;
        var local = 0.0;
        if (all(q >= vec2<f32>(0.0)) && all(q < vec2<f32>(SHADOW_GRID))) {
            let cell = globals.shadow_cells[u32(q.y) * u32(SHADOW_GRID) + u32(q.x)];
            for (var k = 0u; k < 4u; k++) {
                let n = cell[k];
                if (n < 0.5) {
                    break;
                }
                let c = globals.shadow_clouds[u32(n) - 1u];
                local = max(local, c.w * (1.0 - smoothstep(0.45, 1.0, length(p - c.xy) / c.z)));
            }
        }
        let off = abs(q - vec2<f32>(SHADOW_GRID * 0.5)) / (SHADOW_GRID * 0.5);
        let edge = max(off.x, off.y);
        shade = max(shade, mix(local, globals.shadow_far.z, smoothstep(0.75, 1.0, edge)));
    }
    return 1.0 - s * shade;
}

// The cloud shadows and ground mist at `world`, `dist` metres from the eye (each only in the
// variants with it).
fn atmosphere(rgb: vec3<f32>, world: vec3<f32>, dist: f32) -> vec3<f32> {
    var c = rgb;
#ifdef PALETTE_CLOUD_SHADOWS
    c *= cloud_shadow(world);
#endif
#ifdef PALETTE_MIST
    c = ground_mist(c, world, dist);
#endif
    return c;
}

// How thick the storm haze is at the visibility distance (1 - exp(-this)).
const STORM_HAZE_DEPTH: f32 = 2.0;

// In a storm the air under the deck is hazy with rain, the more the deck covers the sky: the
// colour seen along view direction `nd` at `dist` metres fades into grey over the part of the
// ray below the deck's base.
fn storm_haze(rgb: vec3<f32>, nd: vec3<f32>, dist: f32) -> vec3<f32> {
    let s = globals.storm_haze.x;
    if (s <= 0.0) {
        return rgb;
    }
    let base = globals.clouds.x;
    let y = view.world_position.y;
    var path = 0.0;
    if (y < base) {
        path = select(dist, min(dist, (base - y) / nd.y), nd.y > 0.0);
    } else if (nd.y < 0.0) {
        path = max(dist - (y - base) / -nd.y, 0.0);
    }
    let a = s * (1.0 - exp(-STORM_HAZE_DEPTH * path / globals.storm_haze.y));
    return mix(rgb, pal(CLOUD_RAMP + 3u) * (1.0 - night_dark()), a);
}

// --- Smoke plumes ----------------------------------------------------------------------------
// Smoke of burning wrecks and fires (fires.rs): a column rising from the ground (local y 0..1
// inside a unit cylinder), widening with height and leaning downwind (local +x). The noise
// scrolls upwards so the smoke billows; dark grey from the cloud ramp, self-shadowed, its base lit orange by the fire underneath.

const SMOKE_STEPS: u32 = 12u;
// XR (two large eye views on a mobile GPU): fewer steps, a single noise octave and no
// self-shadow sample — about a sixth of the noise evaluations, so a screen full of plumes
// can't stall the GPU long enough to lose the device.
const SMOKE_STEPS_XR: u32 = 6u;
// Plume lean (local x at the top) and radius at the base / top.
const SMOKE_LEAN: f32 = 0.4;
const SMOKE_R0: f32 = 0.06;
const SMOKE_R1: f32 = 0.58;
// Rise speed of the noise (metres per second).
const SMOKE_RISE: f32 = 9.0;
const FIRE_COLOR: vec3<f32> = vec3<f32>(1.0, 0.42, 0.1);
// The proxy mesh (fires.rs) circumscribes the unit cylinder (y 0..1) the plume lives in:
// radius 1 / cos(pi / 16).
const SMOKE_PROXY: f32 = 1.0196;

// Ray parameter range (t0, t1) inside the unit cylinder (empty when t0 >= t1).
fn cylinder_bounds(ro: vec3<f32>, rd: vec3<f32>) -> vec2<f32> {
    let a = rd.x * rd.x + rd.z * rd.z;
    let b = ro.x * rd.x + ro.z * rd.z;
    let c = ro.x * ro.x + ro.z * ro.z - 1.0;
    var t = vec2<f32>(-1e9, 1e9);
    if (a > 1e-9) {
        let disc = b * b - a * c;
        if (disc < 0.0) { return vec2<f32>(1.0, 0.0); }
        let sq = sqrt(disc);
        t = vec2<f32>((-b - sq) / a, (-b + sq) / a);
    } else if (c > 0.0) {
        return vec2<f32>(1.0, 0.0);
    }
    if (abs(rd.y) > 1e-6) {
        let ty = vec2<f32>(-ro.y / rd.y, (1.0 - ro.y) / rd.y);
        t = vec2<f32>(max(t.x, min(ty.x, ty.y)), min(t.y, max(ty.x, ty.y)));
    } else if (ro.y < 0.0 || ro.y > 1.0) {
        return vec2<f32>(1.0, 0.0);
    }
    return t;
}

// The proxy is drawn with both windings and back faces culled, so from outside both the near
// side and (reversed) the far side are rasterised, from inside only the far side, and every
// fragment is front facing: whether to march from this fragment (local point p, eye ro) is
// decided by its geometry instead — from outside the side facing the eye, from inside the far
// side.
fn proxy_fragment(p: vec3<f32>, ro: vec3<f32>) -> bool {
    let inside = dot(ro.xz, ro.xz) < SMOKE_PROXY * SMOKE_PROXY && ro.y > 0.0 && ro.y < 1.0;
    var n: vec3<f32>;
    if (p.y > 0.9999) {
        n = vec3<f32>(0.0, 1.0, 0.0);
    } else if (p.y < 0.0001) {
        n = vec3<f32>(0.0, -1.0, 0.0);
    } else {
        n = vec3<f32>(p.x, 0.0, p.z);
    }
    return inside || dot(n, ro - p) > 0.0;
}

// `start`: local height where the smoke has risen clear of the flames.
fn smoke_density(p: vec3<f32>, aspect: vec3<f32>, seed: f32, rise: f32, start: f32, detail: bool) -> f32 {
    let y = p.y;
    if (y <= 0.0 || y >= 1.0) { return 0.0; }
    let cx = SMOKE_LEAN * pow(y, 1.6);
    let r = mix(SMOKE_R0, SMOKE_R1, pow(y, 0.7));
    var shape = 1.0 - length(vec2<f32>(p.x - cx, p.z)) / r;
    // Thins out and ends towards the top.
    shape -= smoothstep(0.55, 1.0, y) * 1.1;
    if (shape < -0.6) { return 0.0; }
    // Noise in units of the plume's width, rising (faster at the core).
    let pn = p * aspect + vec3<f32>(seed, -rise, seed * 0.7);
    var n = perlin(pn * 2.2, 1024);
    if (detail) {
        n += 0.5 * perlin((pn + vec3<f32>(0.0, -rise * 0.6, 0.0)) * 5.3, 1024);
    }
    return clamp((shape + 0.65 * n) * 3.0, 0.0, 1.0) * (1.0 - 0.55 * y) * smoothstep(start * 0.3, start * 2.0, y);
}

fn smoke_volume(in: VertexOutput, dir: vec3<f32>, modern: bool) -> vec4<f32> {
    let dist = length(dir);
    let fire = in.puff_eye.w > 1.5;
    let ro = in.puff_eye.xyz;
    let rd = in.puff_pos.xyz - ro;
    if (!proxy_fragment(in.puff_pos.xyz, ro)) { discard; }
    let tb = cylinder_bounds(ro, rd);
    let t0 = max(tb.x, 0.0);
    let t1 = tb.y;
    if (t1 <= t0) { discard; }
    let aspect = in.puff_scale / in.puff_scale.x;
    let m_per_t = length(rd * in.puff_scale);
    let xr = globals.dots.x > 0.0;
    let steps = select(SMOKE_STEPS, SMOKE_STEPS_XR, xr);
    let dt = (t1 - t0) / f32(steps);
    let ds = dt * m_per_t;
    // Opaque within about a fifth of the plume's width.
    let thick = in.puff_scale.x * 0.2;
    let rise = globals.dots.y * SMOKE_RISE / in.puff_scale.x;
    let jitter = bayer4(vec2<u32>(in.clip_position.xy));
    let sun = in.puff_sun * 0.08;
    // Smoke starts above the flames (about one base width up); the fire lights the bottom of
    // the plume.
    let start = select(0.0, in.puff_scale.x * SMOKE_R0 / in.puff_scale.y, fire);
    let glow_top = start * 4.0;
    var trans = 1.0;
    var light = 0.0;
    var glow = 0.0;
    for (var i = 0u; i < steps; i++) {
        let p = ro + rd * (t0 + (f32(i) + jitter) * dt);
        let dens = smoke_density(p, aspect, in.puff_pos.w, rise, start, !xr);
        if (dens <= 0.001) { continue; }
        let a = 1.0 - exp(-dens * ds / thick);
        var lit = clamp(0.5 + 0.25 * p.y, 0.0, 1.0);
        if (!xr) {
            let towards_sun = smoke_density(p + sun, aspect, in.puff_pos.w, rise, start, false);
            lit = clamp(0.4 + (dens - towards_sun) * 1.4 + 0.25 * p.y, 0.0, 1.0);
        }
        light += trans * a * lit;
        if (fire) {
            let g = 1.0 - smoothstep(start, glow_top, p.y);
            glow += trans * a * g * g;
        }
        trans *= 1.0 - a;
        if (trans < 0.03) { break; }
    }
    var alpha = 1.0 - trans;
    if (alpha < 0.01) { discard; }
    // Darker than cloud: the lower half of the grey ramp.
    var rgb = cloud_ramp(7.0 + (1.0 - light / alpha) * 6.5, dist, modern);
    // The fire lights the smoke above it, most at night.
    rgb += FIRE_COLOR * min(glow / alpha, 1.0) * mix(0.5, 0.15, globals.light_info.y);
    rgb = flash(whiteout(storm_haze(rgb, normalize(dir), dist), dist), view.world_position + dir);
    if (!modern) {
        // Classic: a screen door instead of blending.
        if (alpha < bayer4(vec2<u32>(in.clip_position.xy) + vec2<u32>(2u, 1u))) { discard; }
        alpha = 1.0;
    }
    return vec4<f32>(rgb, alpha);
}

const BOLT_COLOR: vec3<f32> = vec3<f32>(0.55, 0.58, 0.75);

// Light of the current lightning channel at a point: falls off with the distance from the
// channel (a line light), zero beyond its reach.
fn bolt_glow(world: vec3<f32>) -> f32 {
    let i = globals.bolt_a.w;
    if (i <= 0.0) { return 0.0; }
    let a = globals.bolt_a.xyz;
    let ab = globals.bolt_b.xyz - a;
    let t = clamp(dot(world - a, ab) / max(dot(ab, ab), 1e-3), 0.0, 1.0);
    let d = length(world - (a + ab * t));
    let reach = globals.bolt_b.w;
    // A bright core around the channel and a wide, weaker halo.
    let core = d / (reach * 0.06);
    let halo = d / (reach * 0.3);
    return i * (1.0 - smoothstep(0.3 * reach, reach, d)) * (1.6 / (1.0 + core * core) + 0.3 / (1.0 + halo * halo));
}

// Lightning reads far brighter at night than by day.
fn flash_scale() -> f32 {
    return mix(1.0, 0.3, globals.light_info.y);
}

// Light of the current combat flash (a flak burst, an explosion, a launch) at a point: as a
// lightning channel's, from a point.
fn burst_glow(world: vec3<f32>) -> vec3<f32> {
    let i = globals.burst.w;
    if (i <= 0.0) { return vec3<f32>(0.0); }
    let d = length(world - globals.burst.xyz);
    let reach = globals.burst_colour.w;
    let core = d / (reach * 0.06);
    let halo = d / (reach * 0.3);
    return globals.burst_colour.rgb * i * (1.0 - smoothstep(0.3 * reach, reach, d)) * (1.6 / (1.0 + core * core) + 0.3 / (1.0 + halo * halo));
}

// Lightning: the far-reaching flash over everything plus the channel's local light; and the
// combat flash's light.
fn flash(rgb: vec3<f32>, world: vec3<f32>) -> vec3<f32> {
#ifdef PALETTE_BURST
    return rgb + (BOLT_COLOR * (globals.weather.y + bolt_glow(world)) + burst_glow(world)) * flash_scale();
#else
    return rgb + BOLT_COLOR * (globals.weather.y + bolt_glow(world)) * flash_scale();
#endif
}

// Night vision goggles (light_info.z): the image intensified to a green phosphor picture,
// with more gain the darker it is, a contrast curve and a little grain.
fn nvg(rgb: vec3<f32>, clip: vec4<f32>) -> vec3<f32> {
    if (globals.light_info.z == 0.0) { return rgb; }
    let lum = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let gain = mix(14.0, 2.5, clamp(globals.light_info.y, 0.0, 1.0));
    var v = sqrt(1.0 - exp(-lum * gain));
    v = smoothstep(0.12, 0.95, v);
    let p = vec2<u32>(clip.xy);
    var h = p.x * 0x27d4eb2du ^ p.y * 0x165667b1u ^ u32(globals.dots.y * 60.0) * 0x9e3779b9u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    // Scintillation: the tube's grain, with the goggles effect strongest in the dark.
    let grain = select(0.06, mix(0.16, 0.05, v), globals.light_info.z > 1.5);
    v = clamp(v + (f32(h >> 8u) / 16777216.0 - 0.5) * grain, 0.0, 1.0);
    // Phosphor green, near white in the highlights; sRGB -> linear.
    let c = mix(vec3<f32>(0.0, 0.04, 0.01), vec3<f32>(0.3, 0.85, 0.2), v) + vec3<f32>(0.5, 0.15, 0.4) * v * v * v;
    return pow(min(c, vec3<f32>(1.0)), vec3<f32>(2.2));
}

// FLIR (tracking camera only, picked by its picture size): a black and white thermal picture
// with the hot units, fires and dot lights (mesh tag `TAG_HOT`) white, the warm buildings
// (`TAG_WARM`) a little lighter than the dark scenery, so the units stand out among them;
// haze fades them into the background (the hot ones less).
// The targeting pod's or a recon camera's picture: no shadow maps there (lighting.rs
// `no_offscreen_shadows`).
fn offscreen_view() -> bool {
    return recon_view() || all(abs(view.viewport.zw - globals.dots.zw) < vec2<f32>(0.5));
}

fn flir_view() -> bool {
    return globals.params.z != 0.0 && all(abs(view.viewport.zw - globals.dots.zw) < vec2<f32>(0.5));
}

// The warm buildings' level (before the shade's share), and the most haze takes of the hot.
const FLIR_WARM: f32 = 0.5;
const FLIR_HOT_HAZE: f32 = 0.4;

fn flir(rgb: vec3<f32>, warm: u32, fog_f: f32) -> vec3<f32> {
    let s = sqrt(dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722)));
    let cold = 0.3 + 0.25 * s;
    var v = cold;
    if ((warm & 32u) != 0u) {
        v = mix(0.95 + 0.05 * s, cold, clamp(fog_f / 24.0, 0.0, FLIR_HOT_HAZE));
    } else if ((warm & 1u) != 0u) {
        v = mix(FLIR_WARM + 0.12 * s, cold, clamp(fog_f / 24.0, 0.0, 0.6));
    }
    return vec3<f32>(pow(v, 2.2));
}

// Water (palette_material.rs FLAG_WATER): sparse specks of about a pixel, lighter and
// darker, pinned to the world at random spots of a grid (the floating origin is handed over
// modulo NOISE_PERIOD, a multiple of the spacing), so the height over the sea can be judged
// low down. They fade out with height, and in the distance before they crowd into noise.
const FLAG_WATER: u32 = 256u;
const WATER_GRID: f32 = 12.0;
// Share of the grid cells with a speck.
const WATER_SPECKS: f32 = 0.35;
const WATER_DOTS_HIGH: vec2<f32> = vec2<f32>(80.0, 300.0);
// Least speck radius up close (metres); otherwise a dot light's size.
const WATER_SPECK_M: f32 = 0.25;
// The desert's specks (linear): the sand darkened to brown, and pale sand to mix in.
const SAND_DARK: vec3<f32> = vec3<f32>(0.5, 0.4, 0.3);
const SAND_LIGHT: vec3<f32> = vec3<f32>(0.85, 0.7, 0.45);

// A dot light's size in this view's pixels (the vertex shader's DOT_BATCH).
fn dot_pixels() -> f32 {
    let vp = view.viewport.zw;
    var px = max(vp.y / 200.0 * DOT_SIZE, DOT_MIN_PIXELS);
    if (globals.dots.x > 0.0) {
        px = min(px, max(globals.dots.x * 0.5 * vp.y * view.clip_from_view[1][1], DOT_MIN_PIXELS));
    }
    return px;
}

// A ground plane offset in screen pixels, from the plane's change per pixel across and down.
fn pixels(off: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec2<f32> {
    let det = dx.x * dy.y - dx.y * dy.x;
    if (abs(det) < 1e-9) {
        return vec2<f32>(1e4);
    }
    return vec2<f32>(off.x * dy.y - off.y * dy.x, dx.x * off.y - dx.y * off.x) / det;
}

// Specks pinned to the ground plane at `xz`, seen from `h` metres above it: their weight 0..1
// (x) and whether the one here is a light one (y).
fn specks(xz: vec2<f32>, h: f32, deriv: Deriv, modern: bool) -> vec2<f32> {
    let fade = (1.0 - smoothstep(WATER_DOTS_HIGH.x, WATER_DOTS_HIGH.y, h))
        * (1.0 - smoothstep(0.15, 0.4, deriv.foot / WATER_GRID));
    if (fade <= 0.0) {
        return vec2<f32>(0.0);
    }
    let p = (xz + globals.noise.xy) / WATER_GRID;
    let cell = floor(p);
    let n = hash2(cell);
    if (n >= WATER_SPECKS) {
        return vec2<f32>(0.0);
    }
    let at = cell + vec2<f32>(hash2(cell + 17.0), hash2(cell + 43.0));
    // Distance from the speck in screen pixels (a little world size up close).
    let off = (p - at) * WATER_GRID;
    let px = length(pixels(off, deriv.ground_dx, deriv.ground_dy));
    let r = max(dot_pixels() * 0.5, WATER_SPECK_M / max(deriv.foot, 1e-3));
    var k = 1.0 - smoothstep(r - 0.3, r + 0.3, px);
    if (!modern) {
        k = select(0.0, 1.0, px < r);
    }
    return vec2<f32>(k * fade, select(0.0, 1.0, n < WATER_SPECKS * 0.5));
}

fn water_dots(rgb: vec3<f32>, world: vec3<f32>, deriv: Deriv, modern: bool) -> vec3<f32> {
    if (abs(normalize(deriv.face).y) <= 0.95) {
        return rgb;
    }
    let s = specks(world.xz, view.world_position.y - world.y, deriv, modern);
    let tone = select(rgb * 0.4, rgb * 0.4 + vec3<f32>(0.4), s.y > 0.5);
    return mix(rgb, tone, s.x);
}

// The desert theaters' ground gradient (globals.sky_fx.w): the water's specks in earth
// colours, a darker brown and a pale sand, on the ground plane at `xz`.
fn sand_dots(rgb: vec3<f32>, xz: vec2<f32>, deriv: Deriv, modern: bool) -> vec3<f32> {
    let s = specks(xz, view.world_position.y, deriv, modern);
    let tone = select(rgb * SAND_DARK, mix(rgb, SAND_LIGHT, 0.5), s.y > 0.5);
    return mix(rgb, tone, s.x);
}

// --- Ground detail (`Effects::ground_detail`, `PALETTE_DETAIL`) ------------------------------
// Flat ground seen up close: crop rows and the sprayer's tramlines on the farmland's fields,
// tufts on the grass, clods on bare earth, grain on concrete and tarmac. A height pattern lit
// by the key light (bumps) and a brightness pattern, pinned to the world: `ground_detail.yz` is
// the floating origin modulo DETAIL_PERIOD, which every pattern's period divides. Each part
// fades out as it gets too small on screen, and nothing is left beyond DETAIL_FOOT metres a
// pixel.
const DETAIL_PERIOD: f32 = 1024.0;
const DETAIL_FOOT: f32 = 2.0;
// Spacing of the crop rows and of the tramlines (a pair of wheel tracks 1.8 m apart, 0.5 m
// wide), metres.
const FURROW_M: f32 = 0.8;
const TRAMLINE_M: f32 = 32.0;
// The bumps' light: the key light's azimuth at the hills' height (`hill_shade`), within these.
const DETAIL_SHADE_MIN: f32 = 0.7;
const DETAIL_SHADE_MAX: f32 = 1.3;

// Value noise on cells of `cell` metres (a power of two dividing DETAIL_PERIOD), repeating
// every DETAIL_PERIOD: its value 0..1 (x) and its slope per metre along x and z (yz).
fn detail_noise(p: vec2<f32>, cell: f32, seed: u32) -> vec3<f32> {
    let n = i32(DETAIL_PERIOD / cell) - 1;
    let q = p / cell;
    let f = floor(q);
    let i = vec2<i32>(f);
    let t = q - f;
    let u = t * t * (3.0 - 2.0 * t);
    let du = 6.0 * t * (1.0 - t);
    let a = value_hash(i.x & n, i.y & n, seed);
    let b = value_hash((i.x + 1) & n, i.y & n, seed);
    let c = value_hash(i.x & n, (i.y + 1) & n, seed);
    let d = value_hash((i.x + 1) & n, (i.y + 1) & n, seed);
    let k = a - b - c + d;
    let v = a + (b - a) * u.x + (c - a) * u.y + k * u.x * u.y;
    return vec3<f32>(v, vec2<f32>(du.x * (b - a + k * u.y), du.y * (c - a + k * u.x)) / cell);
}

// Weight of a pattern of `size` metres at `foot` metres a pixel: whole while it spans five
// pixels, gone at two.
fn detail_fade(size: f32, foot: f32) -> f32 {
    return 1.0 - smoothstep(size * 0.2, size * 0.5, foot);
}

// A stripe `half` metres either side of its middle, `d` metres away, at `foot` metres a pixel:
// its share of the pixel.
fn detail_stripe(d: f32, half: f32, foot: f32) -> f32 {
    return clamp((half - d) / max(foot, 1e-3) + 0.5, 0.0, 1.0);
}

// The ground's colour `rgb` at `world` with the detail of its kind: a field's rows (`rows` 1
// along x, 2 along z), else by the colour: grey is concrete or tarmac, green grass, the rest
// bare earth. `foot`: metres a pixel.
fn ground_detail(rgb: vec3<f32>, world: vec3<f32>, foot: f32, rows: u32) -> vec3<f32> {
    if (foot >= DETAIL_FOOT || globals.ground_detail.x == 0.0) {
        return rgb;
    }
    let p = world.xz + globals.ground_detail.yz;
    let top = max(rgb.r, max(rgb.g, rgb.b));
    let spread = top - min(rgb.r, min(rgb.g, rgb.b));
    var tone = 1.0;
    var slope = vec2<f32>(0.0);
    if (rows != 0u) {
        // Across the rows (x along z, y along x), the rows bending a little.
        let across = select(p.x, p.y, rows == 1u);
        let axis = select(vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0), rows == 1u);
        let bend = detail_noise(p, 8.0, 7u).x;
        let ph = (across + (bend - 0.5) * 0.3) * TAU / FURROW_M;
        let w = detail_fade(FURROW_M, foot);
        slope = axis * 0.07 * TAU / FURROW_M * cos(ph) * w;
        tone = 1.0 + 0.1 * sin(ph) * w;
        // The crop thicker and thinner along the rows.
        tone *= 1.0 + 0.18 * (detail_noise(p, 2.0, 11u).x - 0.5) * detail_fade(2.0, foot);
        let t = abs(fract(across / TRAMLINE_M) - 0.5) * TRAMLINE_M;
        tone *= 1.0 - 0.25 * detail_stripe(abs(t - 0.9), 0.25, foot) * detail_fade(4.0, foot);
    } else if (spread < 0.12 * top) {
        // Concrete (light) or tarmac (dark): fine grain with lighter stones in the dark, and
        // patches.
        let g = detail_noise(p, 0.125, 13u);
        let w = detail_fade(0.125, foot);
        let stones = select(0.12, 0.3, top < 0.15);
        tone = 1.0 + stones * (g.x - 0.5) * w
            + 0.08 * (detail_noise(p, 0.5, 23u).x - 0.5) * detail_fade(0.5, foot)
            + 0.1 * (detail_noise(p, 2.0, 17u).x - 0.5) * detail_fade(2.0, foot)
            + 0.06 * (detail_noise(p, 16.0, 19u).x - 0.5) * detail_fade(16.0, foot);
        slope = g.yz * 0.01 * w;
    } else {
        // Grass (tufts) or bare earth (clods): two sizes of bumps and patches.
        let green = rgb.g >= rgb.r;
        let small = select(0.25, 0.5, green);
        let n1 = detail_noise(p, small, 3u);
        let n2 = detail_noise(p, small * 4.0, 5u);
        let w1 = detail_fade(small, foot);
        let w2 = detail_fade(small * 4.0, foot);
        slope = n1.yz * small * 0.25 * w1 + n2.yz * small * 0.6 * w2;
        tone = 1.0 + 0.2 * (n1.x - 0.5) * w1 + 0.16 * (n2.x - 0.5) * w2
            + 0.12 * (detail_noise(p, 8.0, 9u).x - 0.5) * detail_fade(8.0, foot);
    }
    let n = normalize(vec3<f32>(-slope.x, 1.0, -slope.y));
    let key = select(globals.moon.xyz, globals.sun.xyz, globals.sun.w >= globals.moon.w);
    var h = vec2<f32>(key.x, key.z);
    if (length(h) < 1e-3) { h = vec2<f32>(-1.0, -1.0); }
    h = normalize(h) * sqrt(1.0 - HILL_LIGHT_UP * HILL_LIGHT_UP);
    let lit = dot(n, vec3<f32>(h.x, HILL_LIGHT_UP, h.y)) / HILL_LIGHT_UP;
    return rgb * max(tone, 0.0) * clamp(lit, DETAIL_SHADE_MIN, DETAIL_SHADE_MAX);
}

// Reflections (`PALETTE_GLASS`): the sky's bands in a mirrored direction (its lowest band
// for one turned below the horizon).
fn mirrored_sky(r: vec3<f32>) -> vec3<f32> {
    let b = band_f(vec3<f32>(r.x, max(r.y, 0.0), r.z));
    let k = u32(b.y);
    let c = mix(pal(u32(b.x) + k), pal(u32(b.x) + min(k + 1u, 15u)), b.z);
    return c * (1.0 - night_dark());
}

// The sun's and the moon's glint in a mirrored direction; `sharp`: its Phong exponent.
fn glint(r: vec3<f32>, sharp: f32) -> vec3<f32> {
    let s = pow(max(dot(r, globals.sun.xyz), 0.0), sharp) * globals.sun.w * sun_colour(globals.sun.y);
    let m = pow(max(dot(r, globals.moon.xyz), 0.0), sharp) * globals.moon.w * MOON_TINT;
    return s + m;
}

// Glass (canopies, windows: colours 0x90 and 0x91) mirrors the sky, more at a glancing angle
// (Schlick's Fresnel), and the sun or the moon as a sharp glint.
const GLASS_F0: f32 = 0.3;
const GLASS_GLINT: f32 = 4.0;
const GLASS_SHEEN: f32 = 0.35;

fn glass_reflection(rgb: vec3<f32>, face: vec3<f32>, dir: vec3<f32>, dark: f32) -> vec3<f32> {
    var n = normalize(face);
    if (dot(n, dir) > 0.0) { n = -n; }
    let v = normalize(dir);
    let r = reflect(v, n);
    let f = GLASS_F0 + (1.0 - GLASS_F0) * pow(1.0 - clamp(-dot(v, n), 0.0, 1.0), 5.0);
    let shine = glint(r, 600.0) * GLASS_GLINT + glint(r, 40.0) * GLASS_SHEEN;
    return mix(rgb, mirrored_sky(r), f) + shine * (1.0 - dark);
}

// Water (`PALETTE_WATER`, FLAG_WATER polygons): the sky mirrored in waves, more at a glancing
// angle, the sun's or the moon's path on them and its sparkle. The wavelengths divide
// NOISE_PERIOD (the waves are placed by `globals.noise.xy`, the floating origin), so they keep
// their place when the origin moves; waves under a few pixels fade out.
const WATER_F0: f32 = 0.02;
const WATER_MIRROR: f32 = 0.85;
const WATER_PATH: f32 = 1.6;
const WATER_SPARKLE: f32 = 7.0;
const WATER_GLITTER: f32 = 3.5;
// Wave height in the calmest and the roughest patches (`water_patches`), and their tint.
const WATER_CALM: f32 = 0.45;
const WATER_ROUGH: f32 = 1.5;
const WATER_TINT: f32 = 0.18;

fn water_wave(p: vec2<f32>, t: f32, d: vec2<f32>, len: f32, slope: f32, foot: f32) -> vec2<f32> {
    let k = TAU / len;
    // Deep water waves: speed sqrt(g / k).
    let phase = k * (dot(d, p) - sqrt(9.81 / k) * t);
    return d * slope * cos(phase) * (1.0 - smoothstep(0.08 * len, 0.4 * len, foot));
}

// A wave's height in irregular groups: smooth noise over a few of its wavelengths, drifting.
fn water_group(p: vec2<f32>, t: f32, len: f32, i: u32) -> f32 {
    let q = p / (len * 5.0) + vec2<f32>(f32(i) * 31.7 + t * 0.03, f32(i) * 17.1 - t * 0.02);
    return 0.3 + 1.4 * value_noise(q, 11u + i);
}

// The sea state (globals.sea.x, 0..8): the waves' slope relative to a moderate sea (state 3)...
fn sea_slope() -> f32 {
    let s = globals.sea.x;
    return select(1.0 + (s - 3.0) * 0.28, 0.35 + 0.65 * s / 3.0, s < 3.0);
}

// ... the long waves' length relative to it ...
fn sea_length() -> f32 {
    return 1.0 + max(globals.sea.x - 3.0, 0.0) * 0.35;
}

// ... and a wave direction turned with the wind (the first wave, (0.8, 0.6), runs downwind),
// the more of them running downwind the rougher the sea.
fn sea_dir(d: vec2<f32>) -> vec2<f32> {
    let wd = globals.sea.yz;
    let rot = vec2<f32>(wd.x * 0.8 + wd.y * 0.6, wd.y * 0.8 - wd.x * 0.6);
    let r = vec2<f32>(d.x * rot.x - d.y * rot.y, d.x * rot.y + d.y * rot.x);
    return normalize(mix(r, wd, 0.6 * smoothstep(1.0, 5.0, globals.sea.x)) + vec2<f32>(1e-4, 0.0));
}

fn water_normal(p: vec2<f32>, t: f32, foot: f32, rough: f32) -> vec3<f32> {
    // Against a regular lattice of highlights: the waves' positions warped by smooth noise
    // (48 m cells, drifting), each wave's height in its own groups, a fine noise ripple on top.
    // The wind turns them and the sea state makes them steeper, the long ones longer.
    let q = p / 48.0 + vec2<f32>(t * 0.03, -t * 0.02);
    let w = p + (vec2<f32>(value_noise(q, 3u), value_noise(q + 17.3, 5u)) - 0.5) * 14.0;
    let l = sea_length();
    var s = water_wave(w, t, sea_dir(vec2<f32>(0.8, 0.6)), 64.0 * l, 0.045 * water_group(p, t, 64.0 * l, 0u), foot);
    s += water_wave(w, t, sea_dir(vec2<f32>(-0.6, 0.8)), 40.0 * l, 0.05 * water_group(p, t, 40.0 * l, 1u), foot);
    s += water_wave(w, t, sea_dir(vec2<f32>(0.96, -0.28)), 20.0 * sqrt(l), 0.06 * water_group(p, t, 20.0, 2u), foot);
    s += water_wave(w, t, sea_dir(vec2<f32>(-0.28, -0.96)), 12.8, 0.07 * water_group(p, t, 12.8, 3u), foot);
    s += water_wave(w, t, sea_dir(vec2<f32>(0.6, -0.8)), 5.12, 0.08 * water_group(p, t, 5.12, 4u), foot);
    s += water_wave(w, t, sea_dir(vec2<f32>(-0.92, 0.39)), 2.5, 0.09 * water_group(p, t, 2.5, 5u), foot);
    // The ripple: the slope of 3 m value noise, gone under a few pixels.
    let r = p / 3.0 + vec2<f32>(t * 0.2, t * 0.13);
    let r0 = value_noise(r, 7u);
    let ripple = vec2<f32>(value_noise(r + vec2<f32>(0.15, 0.0), 7u) - r0, value_noise(r + vec2<f32>(0.0, 0.15), 7u) - r0);
    s += ripple * 0.5 * (1.0 - smoothstep(0.3, 1.5, foot));
    s *= rough * sea_slope();
    return normalize(vec3<f32>(-s.x, 1.0, -s.y));
}

// Whitecaps (globals.sea.w 0..1, from about sea state 3): foam on the crests, lines across the
// wind driven downwind, flaring up and dying away; in the distance the average foam's tint.
fn whitecaps(rgb: vec3<f32>, p: vec2<f32>, t: f32, foot: f32) -> vec3<f32> {
    let caps = globals.sea.w;
    if (caps <= 0.0) {
        return rgb;
    }
    let wd = globals.sea.yz;
    // Along the wind and across it (metres), drifting downwind.
    let speed = 2.0 + 3.0 * globals.sea.x;
    let q = vec2<f32>(dot(p, wd) - t * speed, dot(p, vec2<f32>(-wd.y, wd.x)));
    let n = value_noise(q / vec2<f32>(3.0, 9.0), 23u) * 0.6
        + value_noise(q / vec2<f32>(1.2, 3.5) + vec2<f32>(t * 0.4, 0.0), 29u) * 0.4;
    // Each patch of crests flares up and dies away.
    let life = value_noise(q / 40.0 + vec2<f32>(t * 0.05, 0.0), 31u);
    let thr = 1.0 - 0.32 * caps * (0.4 + 0.6 * life);
    let near = smoothstep(thr, thr + 0.06, n);
    // Past a few metres a pixel the foam is a mean tint (how much of the sea is white).
    let mean = 0.18 * caps * caps;
    let k = smoothstep(0.3, 1.5, foot);
    let foam = mix(near, mean, k);
    return mix(rgb, foam_white(), foam);
}

// How white the simulated sea is: its foam (the cascades' coarse texels) broken up by fine
// noise drifting with the wind into streaks and bubbles; a mean tint where the noise is too
// fine to see.
fn ocean_foam(foam: f32, p: vec2<f32>, t: f32, foot: f32) -> f32 {
    let wd = globals.sea.yz;
    let q = vec2<f32>(dot(p, wd) - t * 0.6, dot(p, vec2<f32>(-wd.y, wd.x)));
    let n = value_noise(q / vec2<f32>(2.5, 0.9), 37u) * 0.65 + value_noise(q / 0.45, 41u) * 0.35;
    let fine = smoothstep(0.0, 0.18, foam - (1.0 - n) * 0.75);
    let mean = smoothstep(0.05, 0.9, foam) * 0.55;
    return mix(fine, mean, smoothstep(0.15, 0.6, foot));
}

// Foam's colour: white as bright as the sky above.
fn foam_white() -> vec3<f32> {
    let sky = mirrored_sky(vec3<f32>(0.0, 1.0, 0.0));
    return vec3<f32>(min(dot(sky, vec3<f32>(0.3, 0.59, 0.11)) * 2.4, 0.92));
}

// Low frequency Perlin noise over the water (about -1..1, two octaves of 640 m and 2.56 km
// cells drifting slowly): calmer and rougher patches, so the waves don't repeat. The cells
// divide NOISE_PERIOD, so the patches stay put when the floating origin moves.
fn water_patches(p: vec2<f32>, t: f32) -> f32 {
    let a = perlin(vec3<f32>(p.x / 640.0, t * 0.004, p.y / 640.0), i32(NOISE_PERIOD / 640.0));
    let b = perlin(vec3<f32>(p.x / 2560.0, t * 0.002 + 7.3, p.y / 2560.0), i32(NOISE_PERIOD / 2560.0));
    return clamp(0.6 * a + 0.8 * b, -1.0, 1.0);
}

fn water_hash(p: vec3<f32>) -> vec2<f32> {
    let h = fract(sin(vec2<f32>(dot(p, vec3<f32>(127.1, 311.7, 74.7)), dot(p, vec3<f32>(269.5, 183.3, 246.1)))) * 43758.5453);
    return h;
}

// Sparkle of a light in direction `l`: facets of about two pixels (in power-of-two sizes),
// each living `SPARKLE_LIFE` seconds from its own moment, tilted at random for that life, fading
// in and out (sin²) where it mirrors the light, at its own strength.
const SPARKLE_LIFE: f32 = 3.0;
// Glitter: each dot's cycle (seconds); it shows for the first half of it, fading in and out.
const GLITTER_CYCLE: f32 = 5.0;
// Glitter fades out between these distances from the eye (metres).
const GLITTER_NEAR: f32 = 800.0;
const GLITTER_FAR: f32 = 4000.0;

// A cell's number, kept small for the hash.
fn water_cell(c: vec2<f32>) -> vec2<f32> {
    return c - 4096.0 * floor(c / 4096.0);
}

fn water_sparkle(p: vec2<f32>, v: vec3<f32>, n: vec3<f32>, l: vec3<f32>, foot: f32, t: f32) -> f32 {
    let cell = exp2(ceil(log2(max(foot * 2.0, 0.01))));
    let c = water_cell(floor(p / cell));
    let age = t / SPARKLE_LIFE + water_hash(vec3<f32>(c, 41.0)).x;
    let h = water_hash(vec3<f32>(c, floor(age)));
    let tilt = (h - 0.5) * 0.35;
    let r = reflect(v, normalize(n + vec3<f32>(tilt.x, 0.0, tilt.y)));
    let fade = sin(PI * fract(age));
    let strength = 0.35 + 0.65 * water_hash(vec3<f32>(c, floor(age) + 0.5)).x;
    return smoothstep(0.996, 0.9995, dot(r, l)) * fade * fade * strength;
}

// Glitter of a light in direction `l` anywhere on the water, not only where its path is:
// round dots of about two pixels, each showing for half its `GLITTER_CYCLE` from its own moment
// (fading in and out), at a new place in its cell and its own strength each time, more of
// them the more they face the light and the rougher the water.
fn water_glitter(p: vec2<f32>, r: vec3<f32>, l: vec3<f32>, across: f32, t: f32, rough: f32) -> f32 {
    let cell = exp2(ceil(log2(max(across * 3.0, 0.01))));
    let k = floor(p / cell);
    let c = water_cell(k);
    let age = t / GLITTER_CYCLE + water_hash(vec3<f32>(c, 17.0)).x;
    let life = floor(age);
    let h = water_hash(vec3<f32>(c, life + 0.25));
    let at = (k + 0.25 + 0.5 * water_hash(vec3<f32>(c, life + 0.75))) * cell;
    let dot_ = 1.0 - smoothstep(0.2 * cell, 0.4 * cell, distance(p, at));
    let shown = sin(PI * clamp(2.0 * fract(age), 0.0, 1.0));
    let toward = max(dot(r, l), 0.0);
    let density = (0.012 + 0.05 * toward * toward) * rough;
    let on = select(0.0, 1.0, h.y < density) * dot_;
    let strength = 0.3 + 0.7 * fract(h.x * 13.0);
    return on * shown * shown * strength * (0.35 + 0.65 * toward) * smoothstep(-0.02, 0.1, l.y);
}

// How much of the sun (x) and the moon (y) reaches the water at `world`, for its highlights:
// none in the shadow of vehicles and scenery (the key light's shadow maps, with the shadows
// option) or of the clouds (with cloud shadows; any real shade hides them).
fn water_light(world: vec3<f32>) -> vec2<f32> {
    var key = 1.0;
    let flags = u32(globals.ambient.w);
#ifdef PALETTE_SHADOWS
    if ((flags & LIGHT_SHADOWS) != 0u && globals.shadow.y > 0.0 && !offscreen_view()) {
        let view_z = (view.view_from_world * vec4<f32>(world, 1.0)).z;
        key = sun_shadow(world, vec3<f32>(0.0, 1.0, 0.0), view_z, false);
    }
#endif
#ifdef PALETTE_CLOUD_SHADOWS
    key *= clamp(1.0 - 4.0 * (1.0 - cloud_shadow(world)), 0.0, 1.0);
#endif
    return select(vec2<f32>(key, 1.0), vec2<f32>(1.0, key), (flags & LIGHT_MOON_KEY) != 0u);
}

// The point of flat water under this pixel (`frag`: its framebuffer position), where the view
// ray meets the plane of `world` (the interpolated position); `world` itself off flat water.
// The sea's polygons are kilometres across: their interpolated position, a few metres over the
// water, was off by up to half a metre, differently each frame as the eye moved, and the
// waves and their glitter shook.
fn flat_water_point(frag: vec2<f32>, world: vec3<f32>, face: vec3<f32>) -> vec3<f32> {
    let eye = view.world_position;
    if (abs(normalize(face).y) < 0.999 || eye.y <= world.y) {
        return world;
    }
    let ndc = vec2<f32>(
        (frag.x - view.viewport.x) / view.viewport.z * 2.0 - 1.0,
        1.0 - (frag.y - view.viewport.y) / view.viewport.w * 2.0,
    );
    // A point some way along the ray (reversed infinite depth: 0.001 is a thousand near
    // planes out).
    let q = view.world_from_clip * vec4<f32>(ndc, 0.001, 1.0);
    let ray = q.xyz / q.w - eye;
    if (ray.y >= -1e-6) {
        return world;
    }
    return eye + ray * ((eye.y - world.y) / -ray.y);
}

// `wave`: the point of the sea (from the floating origin) whose waves light this pixel (the
// waves' mesh: before its displacement).
fn water_reflection(rgb: vec3<f32>, world: vec3<f32>, wave: vec2<f32>, dir: vec3<f32>, foot: f32, across: f32, dark: f32) -> vec3<f32> {
    let v = normalize(dir);
    let p = world.xz + globals.noise.xy;
    let t = globals.dots.y;
    let m = water_patches(p, t);
    // Fewer calm patches the rougher the sea.
    let calm = mix(WATER_CALM, WATER_ROUGH * 0.8, smoothstep(2.0, 6.0, globals.sea.x));
    let rough = calm + (WATER_ROUGH - calm) * smoothstep(-0.6, 0.6, m);
    // The simulated sea's slopes and foam, else the rippled flat sea's.
    var n: vec3<f32>;
    var foam = -1.0;
#ifdef PALETTE_OCEAN
    if (globals.ocean.x > 0.5) {
        let o = ocean_surface(wave, foot);
        n = o.xyz;
        foam = o.w;
    } else {
        n = water_normal(p, t, foot, rough);
    }
#else
    n = water_normal(p, t, foot, rough);
#endif
    var r = reflect(v, n);
    r.y = abs(r.y);
    let f = WATER_F0 + (1.0 - WATER_F0) * pow(1.0 - clamp(-dot(v, n), 0.0, 1.0), 5.0);
    let mirror = f * WATER_MIRROR * (0.8 + 0.4 * smoothstep(-0.6, 0.6, m));
    var c = mix(rgb * (1.0 + WATER_TINT * m), mirrored_sky(r), mirror);
    let lights = water_light(world);
    let sun = globals.sun;
    let moon = globals.moon;
    let far = 1.0 - smoothstep(GLITTER_NEAR, GLITTER_FAR, length(dir));
    // The light's path, its sparkle and the glitter.
    let sun_hl = pow(max(dot(r, sun.xyz), 0.0), 250.0) * WATER_PATH
        + water_sparkle(p, v, n, sun.xyz, foot, t) * WATER_SPARKLE
        + water_glitter(p, r, sun.xyz, across, t, rough) * WATER_GLITTER * far;
    let moon_hl = pow(max(dot(r, moon.xyz), 0.0), 250.0) * WATER_PATH
        + water_sparkle(p, v, n, moon.xyz, foot, t) * WATER_SPARKLE
        + water_glitter(p, r, moon.xyz, across, t, rough) * WATER_GLITTER * far;
    c += (sun_hl * sun.w * sun_colour(sun.y) * lights.x + moon_hl * moon.w * MOON_TINT * lights.y)
        * (1.0 - dark);
    if (foam >= 0.0) {
        return mix(c, foam_white(), ocean_foam(foam, wave + globals.noise.xy, t, foot));
    }
    return whitecaps(c, p, t, foot);
}

// Screen-space derivatives, taken at the start of `fragment`: WebGPU only allows them in
// uniform control flow, which a branch or return on a varying ends.
struct Deriv {
    // Face normal (unnormalised, either side).
    face: vec3<f32>,
    // Radians per pixel of the view direction (from the smooth direction: cell distances in
    // the night sky jump at cell edges).
    sky_px: f32,
    // Metres per pixel on the surface, and the ground plane's x/z change per pixel across and
    // down the screen (the water dots).
    foot: f32,
    ground_dx: vec2<f32>,
    ground_dy: vec2<f32>,
}

// The colour word: the material's, or the vertex's.
// Lit windows on the buildings' walls at night (`PALETTE_CITY`, `WORD_WINDOWS`): a grid of
// storeys and bays, each window lit or dark by a hash of its cell (a share that varies over
// the walls), warm or cool; where the cells are only a pixel or two the wall glows with their
// average instead of shimmering.
const WIN_STOREY: f32 = 3.0;
const WIN_BAY: f32 = 3.2;
// Half the window's width and height in its cell, and its centre's height in the storey.
const WIN_HALF: vec2<f32> = vec2<f32>(0.4, 0.33);
// Bays lit together.
const WIN_RUN: f32 = 4.0;
const WIN_SILL: f32 = 0.5;
// Lit share of the windows: between these by patches of the walls.
const WIN_LIT: vec2<f32> = vec2<f32>(0.2, 0.65);
// Patches (cells) and the walls' bins along their normal (metres) of the lit share.
const WIN_PATCH: f32 = 5.0;
const WIN_WALL_BIN: f32 = 40.0;
const WIN_WARM: vec3<f32> = vec3<f32>(1.0, 0.62, 0.28);
const WIN_COOL: vec3<f32> = vec3<f32>(0.7, 0.82, 1.0);
const WIN_COOL_SHARE: f32 = 0.3;
const WIN_GLOW: f32 = 1.1;
// Beyond the windows' resolution the walls glow with this share of their average (dimmer
// than the true average: a lit wall should read as dark with specks, not as a lamp).
const WIN_FAR: f32 = 0.12;

// `angle`: the wall's outward direction in WALL_ANGLES steps from x towards z (the face's
// direction from the derivatives wobbles from pixel to pixel: far from the theater's origin
// that scrambled the windows).
const WALL_ANGLES: f32 = 2048.0;

fn city_windows(world: vec3<f32>, face: vec3<f32>, angle: u32) -> vec3<f32> {
    if (abs(normalize(face).y) > 0.3) {
        return vec3<f32>(0.0);
    }
    let p = world + globals.city_origin.xyz;
    let a = f32(angle) / WALL_ANGLES * 6.2831853;
    let nh = vec2<f32>(cos(a), sin(a));
    let t = vec2<f32>(-nh.y, nh.x);
    let cell = vec2<f32>(dot(p.xz, t) / WIN_BAY, p.y / WIN_STOREY);
    let ci = vec2<i32>(floor(cell));
    // Walls of one facing apart: by their distance along the normal.
    let wall = u32(i32(floor(dot(p.xz, nh) / WIN_WALL_BIN)) + 0x4000) * 4u + u32(dot(nh, vec2<f32>(1.0, 2.0)) + 3.0);
    let quarter = vec2<i32>(floor(cell / WIN_PATCH));
    let share = mix(WIN_LIT.x, WIN_LIT.y, value_hash(quarter.x, quarter.y, wall ^ 0x51u));
    // Lit in runs (an office, a flat) of a few bays sharing their light, staggered storey by
    // storey, not window by window.
    let stagger = i32(value_hash(0, ci.y, wall ^ 0x3Cu) * WIN_RUN);
    let run = i32(floor(f32(ci.x + stagger) / WIN_RUN));
    let h = value_hash(run, ci.y, wall);
    let h2 = value_hash(ci.y, run, wall ^ 0xA7u);
    let lit = select(0.0, 1.0, h < share);
    let tint = select(WIN_WARM, WIN_COOL, h2 < WIN_COOL_SHARE) * (0.5 + 0.5 * fract(h2 * 7.0));
    // The window's rectangle in its cell, its edges smoothed over a pixel.
    let aa = max(fwidth(cell), vec2<f32>(1e-4));
    let off = abs(fract(cell) - vec2<f32>(0.5, WIN_SILL));
    // Each axis averaged on its own where its cells are under a pixel or two: a wall seen
    // edge on keeps its storeys' bands.
    let k = smoothstep(vec2<f32>(0.6), vec2<f32>(1.5), aa);
    let inside = vec2<f32>(
        1.0 - smoothstep(WIN_HALF.x - aa.x, WIN_HALF.x + aa.x, off.x),
        1.0 - smoothstep(WIN_HALF.y - aa.y, WIN_HALF.y + aa.y, off.y),
    );
    let cover = mix(inside, 2.0 * WIN_HALF, k);
    let far = max(k.x, k.y);
    let light = mix(lit * tint, share * mix(WIN_WARM, WIN_COOL, WIN_COOL_SHARE) * 0.75 * WIN_FAR, far);
    return cover.x * cover.y * light * WIN_GLOW;
}

fn colour_word(in: VertexOutput) -> u32 {
#ifdef VERTEX_WORD
    return in.word;
#else
    return material.word;
#endif
}

// The background sphere's derivatives on the ground plane (y = 0) where the view ray meets it.
fn background_ground(world: vec3<f32>, deriv: ptr<function, Deriv>) {
    let ray = world - view.world_position;
    let ground = view.world_position + ray * (view.world_position.y / max(-ray.y, 1e-4));
    (*deriv).foot = length(fwidth(ground));
    (*deriv).ground_dx = dpdx(ground.xz);
    (*deriv).ground_dy = dpdy(ground.xz);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var deriv: Deriv;
    deriv.face = cross(dpdx(in.world_position), dpdy(in.world_position));
    if ((in.warm & 16u) != 0u) {
        deriv.face = vec3<f32>(0.0, 1.0, 0.0);
    }
    deriv.sky_px = max(length(fwidth(normalize(in.world_position - view.world_position))), 1e-5);
    deriv.foot = length(fwidth(in.world_position));
    deriv.ground_dx = dpdx(in.world_position.xz);
    deriv.ground_dy = dpdy(in.world_position.xz);
#ifdef OCEAN_GRID
    // The waves' mesh: only over the open water, and not where the finer level inside it is.
    // A level reaches one of the finer level's cells into it (where the finer level lies on
    // this level's grid, so the two match): a fragment is dropped for all its samples by its
    // pixel's centre, and an edge that only just met would leave the samples outside the
    // finer level uncovered under multisampling (lines of the sky round the levels).
    if (ocean_mask_at(in.ocean.xy).x < 0.5) {
        discard;
    }
    let level = u32(in.ocean.z + 0.5);
    if (level > 0u) {
        let half = (OCEAN_HALF - 1.0) * OCEAN_CELL0 * exp2(f32(level - 1u));
        if (all(abs(in.ocean.xy - ocean_level_centre(level - 1u)) < vec2<f32>(half))) {
            discard;
        }
    }
#else ifdef PALETTE_OCEAN
    // The flat sea under the waves' mesh, reaching half a cell under its flattened edge (the
    // levels' overlap above).
    if ((material.flags & FLAG_WATER) != 0u && globals.ocean.z > 0.5) {
        let last = OCEAN_LEVELS - 1u;
        let half = (OCEAN_HALF - 0.5) * OCEAN_CELL0 * exp2(f32(last));
        let p = in.world_position.xz;
        if (all(abs(p - ocean_level_centre(last)) < vec2<f32>(half)) && ocean_mask_at(p).x >= 0.5) {
            discard;
        }
    }
#endif
#ifdef PALETTE_BACKGROUND
#ifdef PALETTE_SAND
    // The background sphere: the ground plane (y = 0) where the view ray meets it (the
    // desert's specks, the ground detail).
    background_ground(in.world_position, &deriv);
#else ifdef PALETTE_DETAIL
    background_ground(in.world_position, &deriv);
#endif
#endif
    if ((u32(globals.light_info.w) & 1u) != 0u) {
        return vec4<f32>(pal(colour_word(in) & 0xFFu), 1.0);
    }
    if (flir_view()) {
        // Background: cold sky, cool ground.
        if ((material.flags & 4u) != 0u) {
            let dir = in.world_position - view.world_position;
            return vec4<f32>(vec3<f32>(pow(select(0.2, 0.35, dir.y < 0.0), 2.2)), 1.0);
        }
        let c = shade_fragment(in, deriv);
        if (c.a < 0.0) {
            return vec4<f32>(flir(c.rgb, in.warm, in.fog_f) * -c.a, 0.0);
        }
        return vec4<f32>(flir(c.rgb, in.warm, in.fog_f), c.a);
    }
    let c = shade_fragment(in, deriv);
    // A glowing lamp (`FLAG_GLOW`): premultiplied by its falloff, alpha 0, added to what is
    // behind it.
    if (c.a < 0.0) {
#ifdef PALETTE_RETRO
        return vec4<f32>(retro(nvg(c.rgb, in.clip_position), in.clip_position.xy) * -c.a, 0.0);
#else
        return vec4<f32>(nvg(c.rgb, in.clip_position) * -c.a, 0.0);
#endif
    }
    let glare = sun_glare(in.world_position - view.world_position);
#ifdef PALETTE_RETRO
    return vec4<f32>(retro(nvg(c.rgb + glare * c.a, in.clip_position), in.clip_position.xy), c.a);
#else
    return vec4<f32>(nvg(c.rgb + glare * c.a, in.clip_position), c.a);
#endif
}

fn shade_fragment(in: VertexOutput, deriv: Deriv) -> vec4<f32> {
    let dir = in.world_position - view.world_position;
    let modern = MODERN;
    // One pipeline per kind of material (palette_material.rs `PaletteKey`): a shader with
    // every branch needs so many registers that the Frame's GPU runs half as many fragments
    // at once.
#ifdef PALETTE_CLOUD
    {
        if ((u32(globals.light_info.w) & 8u) != 0u) {
            return vec4<f32>(pal(CLOUD_RAMP + 4u), 1.0);
        }
        let d = length(dir);
        if (in.puff_eye.w > 0.0) {
            return smoke_volume(in, dir, modern);
        }
        // Stratus sheet: only where the layer's coverage field says so.
        var cover = stratus_cover(in.world_position);
        if (modern) {
            if (cover <= 0.0) { discard; }
        } else {
            if (cover < bayer4(vec2<u32>(in.clip_position.xy))) { discard; }
            cover = 1.0;
        }
        var n = normalize(deriv.face);
        if (dot(n, dir) > 0.0) { n = -n; }
        let noise = cloud_noise(in.world_position, d);
        let c = cloud_color(noise, n, d, modern) * (1.0 - night_dark());
        return vec4<f32>(flash(whiteout(storm_haze(c, dir / d, d), d), in.world_position), cover);
    }
#else ifdef PALETTE_BACKGROUND
    {
        if ((u32(globals.light_info.w) & 2u) != 0u) {
            return vec4<f32>(pal(select(0x7Cu, 0x88u, dir.y >= 0.0)), 1.0);
        }
        // Background: sky bands above the horizon, ground gradient below.
        if ((material.flags & 4u) != 0u) {
            return vec4<f32>(pal(select(0x7Cu, 0x88u, dir.y >= 0.0)), 1.0);
        }
        var bg: vec3<f32>;
        let b = band_f(dir);
        if (modern) {
            let k = u32(b.y);
            let k1 = min(k + 1u, 15u);
            bg = mix(pal(u32(b.x) + k), pal(u32(b.x) + k1), b.z);
        } else {
            bg = pal(band(dir).x);
        }
        let eye = view.world_position;
#ifdef PALETTE_SAND
        if (globals.sky_fx.w != 0.0 && dir.y < 0.0 && eye.y > 0.0) {
            bg = sand_dots(bg, eye.xz + dir.xz * (eye.y / -dir.y), deriv, modern);
        }
#endif
#ifdef PALETTE_DETAIL
        if (dir.y < 0.0 && eye.y > 0.0) {
            bg = ground_detail(bg, eye + dir * (eye.y / -dir.y), deriv.foot, 0u);
        }
#endif
        // Lighting options: the ground gradient is lit like flat ground where the view ray
        // meets the ground plane (y = 0), so it takes shadows and local lights.
        var ground_lit = false;
#ifdef PALETTE_LIT
        let flags = u32(globals.ambient.w);
        if (flags != 0u && dir.y < 0.0 && eye.y > 0.0) {
            let ground = eye + dir * (-eye.y / dir.y);
            bg = lit_surface(bg, ground, vec3<f32>(0.0, 1.0, 0.0), dir, in.clip_position, flags, night_dark(), GLOSS_GROUND);
            ground_lit = true;
        }
#endif
        if (!ground_lit) {
            bg = night_background(bg, dir, b, modern);
        }
        if (globals.sky_sun.w > 0.0 && dir.y > 0.0) {
            bg = night_sky(bg, normalize(dir), deriv.sky_px);
        }
        // Stratus layer beyond its planes: its underside over the sky below it, its top over
        // the ground above it, where the layer's coverage field has cloud. Only in the variant
        // for a sky with a deck (`FX_STRATUS`).
#ifdef PALETTE_STRATUS
        // Unless the layered clouds draw the deck (`storm_haze.z`).
        if (globals.clouds.z != 0.0 && globals.storm_haze.z == 0.0) {
            let eye_y = eye.y;
            let nd = normalize(dir);
            var d = -1.0;
            var n = vec3<f32>(0.0, -1.0, 0.0);
            if (eye_y <= globals.clouds.x && nd.y > 0.0) {
                d = (globals.clouds.x - eye_y) / nd.y;
            } else if (eye_y >= globals.clouds.y && nd.y < 0.0) {
                d = (eye_y - globals.clouds.y) / -nd.y;
                n = vec3<f32>(0.0, 1.0, 0.0);
            } else if (eye_y > globals.clouds.x && eye_y < globals.clouds.y) {
                bg = mix(bg, pal(CLOUD_RAMP + 2u) * (1.0 - night_dark()), stratus_cover(eye));
            }
            if (d > 0.0) {
                let hit = eye + nd * d;
                let c = cloud_color(cloud_noise(hit, d), n, d, modern) * (1.0 - night_dark());
                bg = mix(bg, c, stratus_cover(hit));
            }
        }
#endif
        // Lightning: the channel's light where the view ray meets cloud or ground, and the
        // glow of a strike over the sky in its direction.
        let to_bolt = (globals.bolt_a.xyz + globals.bolt_b.xyz) * 0.5 - eye;
        let bolt_d = length(to_bolt);
        let along = min(bolt_d, 60000.0);
        let sheet = pow(max(dot(normalize(dir), to_bolt / max(bolt_d, 1.0)), 0.0), 6.0) * globals.bolt_a.w * 0.25 * flash_scale();
        // The ground gradient lies on the plane y = 0; the sky is far.
        let nd = normalize(dir);
        let haze_d = select(1e9, max(eye.y, 0.0) / -nd.y, nd.y < 0.0);
        // The ground gradient's mist and cloud shadows, where the ray meets the ground.
        if (nd.y < 0.0) {
            let ground = eye + nd * haze_d;
            bg = atmosphere(bg, ground, haze_d);
        }
        bg = storm_haze(bg, nd, haze_d);
        bg = flash(whiteout(bg, 1e4), eye + nd * along) + BOLT_COLOR * sheet;
        // A distant combat flash's halo over the sky (where the view ray passes it; one near
        // the eye lights what is around it, not the sky).
#ifdef PALETTE_BURST
        if (globals.burst.w > 0.0) {
            let burst_d = length(globals.burst.xyz - eye);
            let far = smoothstep(0.5, 2.0, burst_d / globals.burst_colour.w);
            bg += burst_glow(eye + nd * burst_d) * flash_scale() * far;
        }
#endif
        return vec4<f32>(bg, 1.0);
    }
#else
    let word = colour_word(in);
    let hi = (word >> 8u) & 0xFFu;
    var lo = word & 0xFFu;
#ifdef DOT_COLOUR
    lo = in.dot_colour & 0xFFu;
#endif
    if ((u32(globals.light_info.w) & 4u) != 0u) {
        return vec4<f32>(pal(haze(lo, in.fog)), 1.0);
    }
    let px = vec2<u32>(in.clip_position.xy);
    let is_line = (material.flags & 2u) != 0u;
    var rgb: vec3<f32>;
    if (is_line || hi == 0xFFu) {
        if (modern) {
            rgb = haze_smooth(lo, in.fog_f);
        } else {
            rgb = pal(haze(lo, in.fog));
        }
    } else if (hi == 0xFBu) {
        discard;
    } else if (hi == 0xFCu) {
        if (((px.x + px.y) & 1u) == 1u) { discard; }
        rgb = pal(lo);
    } else if (hi == 0xFEu || hi == 0xFDu) {
        // AND / OR with the ground gradient underneath; on the hills the gradient of the flat
        // ground under the point (else hillsides above the eye took the sky's bands).
        var gdir = dir;
        if ((in.warm & 8u) != 0u) {
            gdir = vec3<f32>(dir.x, -max(view.world_position.y, 0.5), dir.z);
        }
        if (modern) {
            let b = band_f(gdir);
            let k = u32(b.x) + u32(b.y);
            let k1 = u32(b.x) + min(u32(b.y) + 1u, 15u);
            let c0 = select(k | lo, k & lo, hi == 0xFEu);
            let c1 = select(k1 | lo, k1 & lo, hi == 0xFEu);
            rgb = mix(pal(c0), pal(c1), b.z);
        } else {
            rgb = pal(select(band(gdir).x | lo, band(gdir).x & lo, hi == 0xFEu));
        }
    } else {
        // Two-colour checkerboard dither (modern: the average of both colours).
        if (modern) {
            rgb = (pal(lo) + pal(hi)) * 0.5;
        } else {
            rgb = pal(select(lo, hi, ((px.x + px.y) & 1u) == 1u));
        }
    }
    // The water's dots, the low-power stand-in for its reflections (`PALETTE_WATER`).
#ifndef PALETTE_WATER
    if ((material.flags & FLAG_WATER) != 0u) {
        rgb = water_dots(rgb, in.world_position, deriv, modern);
        if (modern && abs(normalize(deriv.face).y) > 0.95) {
            rgb = whitecaps(rgb, in.world_position.xz + globals.noise.xy, globals.dots.y, deriv.foot);
        }
    }
#endif
#ifdef PALETTE_DETAIL
    // Flat ground: the hills, the scenery's ground and what lies at the flat world's level (not
    // a ship's deck); not warm objects, aircraft, dots or water.
    if (!is_line && (in.warm & 39u) == 0u && (material.flags & FLAG_WATER) == 0u
        && ((in.warm & 264u) != 0u || in.world_position.y < 1.0)
        && abs(normalize(deriv.face).y) > 0.8) {
        rgb = ground_detail(rgb, in.world_position, deriv.foot, (in.warm >> 6u) & 3u);
    }
#endif
    if (!is_line && (in.warm & 8u) != 0u) {
        rgb = rgb * hill_shade(deriv.face, dir);
    }
    // A dark night blacks out the scenery; lights (lines and dots) show, aircraft barely.
    // Lines in scenery colours (rivers, roads: material flag 64) go dark with the ground.
    var dark = select(night_dark(), 0.0, is_line && (material.flags & 64u) == 0u);
    if ((in.warm & 4u) != 0u) {
        dark = night_dark() * AIRCRAFT_NIGHT;
    } else if ((in.warm & 2u) != 0u) {
        dark *= AIRCRAFT_NIGHT;
    }
#ifdef PALETTE_LIT
    let flags = u32(globals.ambient.w);
    if (!is_line && flags != 0u) {
        let gloss = select(GLOSS_OBJECT, GLOSS_AIRCRAFT, (in.warm & 2u) != 0u);
        rgb = lit(rgb, in.world_position, deriv.face, dir, in.clip_position, flags, dark, gloss);
    } else
#endif
    {
        if ((modern || (in.warm & 8u) != 0u) && !is_line) {
            rgb = rgb * shade(deriv.face, dir);
        }
        rgb = rgb * (1.0 - dark);
    }
#ifdef PALETTE_GLASS
    if (!is_line && hi == 0xFFu && (lo == 0x90u || lo == 0x91u)) {
        rgb = glass_reflection(rgb, deriv.face, dir, dark);
    }
#endif
#ifdef PALETTE_CITY
    if (!is_line && (in.warm & 512u) != 0u && globals.city.x > 0.0 && !recon_view()) {
        rgb += city_windows(in.world_position, deriv.face, (in.warm >> 16u) & 0x7FFu) * globals.city.x;
    }
#endif
#ifdef PALETTE_WATER
    if (!is_line && (material.flags & FLAG_WATER) != 0u) {
        let across = min(length(deriv.ground_dx), length(deriv.ground_dy));
#ifdef OCEAN_GRID
        let wave = in.ocean.xy;
        rgb = water_reflection(rgb, in.world_position, wave, dir, deriv.foot, across, dark);
#else
        let at = flat_water_point(in.clip_position.xy, in.world_position, deriv.face);
        rgb = water_reflection(rgb, at, at.xz, at - view.world_position, deriv.foot, across, dark);
#endif
    }
#endif
    let d = length(dir);
    if (!is_line) {
        rgb = atmosphere(rgb, in.world_position, d);
    }
    var alpha = 1.0;
#ifdef DOT_BATCH
#ifdef DOT_COLOUR
    // A glowing lamp: its colour through the storm haze and the whiteout (their light only,
    // not their veil: it is added to what is behind), and its falloff for `fragment` to scale
    // it by (as a negative alpha).
    if (in.dot_uv.z < 0.0) {
        let nd = dir / max(d, 1e-3);
        let c = glow_colour(in.dot_colour, rgb);
        let seen = whiteout(storm_haze(c, nd, d), d) - whiteout(storm_haze(vec3<f32>(0.0), nd, d), d);
        let r2 = dot(in.dot_uv.xy, in.dot_uv.xy);
        return vec4<f32>(max(seen, vec3<f32>(0.0)), min(-exp(-GLOW_FALLOFF * r2) * -in.dot_uv.z, -1e-6));
    }
#endif
    // A goggles halo: the lamp solid in the middle, its bloom fading out round it.
    if (in.dot_uv.z > 1.0) {
        let r = (length(in.dot_uv.xy) * in.dot_uv.z - 1.0) / (in.dot_uv.z - 1.0);
        let k = clamp(1.0 - r, 0.0, 1.0);
        alpha = select(0.6 * k * k, 1.0, r <= 0.0);
    }
#endif
    return vec4<f32>(flash(whiteout(storm_haze(rgb, dir / max(d, 1e-3), d), d), in.world_position), alpha);
#endif
}

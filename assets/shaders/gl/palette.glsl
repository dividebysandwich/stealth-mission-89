// The palette material for the OpenGL headset renderer (src/glvr), palette.wgsl translated:
// colour words, haze, face shading, the background (sky bands, night sky, stratus beyond its
// planes, lightning), clouds and smoke plumes, dot lights, the lighting options (local lights;
// no sun shadows yet), FLIR, night vision. Both eyes in one pass (GL_OVR_multiview2; one view
// with SINGLE). Compiled with VERTEX or FRAGMENT, and BACKGROUND, CLOUD (palette_material.rs
// `KIND_CLOUD`) or DOT (dots.rs `dot_batch`) for those kinds; LIT for the lighting options.

layout(std140) uniform Globals {
    vec4 palette[256];
    vec4 sky_ring[4];
    vec4 ground_g[4];
    // x: haze enabled, y: metres per world unit, z: FLIR, w: modern mode
    vec4 params;
    vec4 sun;
    vec4 ambient;
    vec4 light_info;
    vec4 dots;
    vec4 local_lights[64];
    vec4 clouds;
    vec4 weather;
    vec4 noise;
    vec4 stratus;
    vec4 bolt_a;
    vec4 bolt_b;
    vec4 sky_sun;
    vec4 sky_moon;
    vec4 sky_night;
    vec4 recon_palette[256];
    vec4 moon;
    vec4 shadow;
    vec4 shadow_bounds;
    // Storm haze under an overcast: strength 0..1 (x), visibility in metres (y).
    vec4 storm_haze;
    vec4 burst;
    vec4 burst_colour;
    vec4 atmos;
    vec4 shadow_grid;
    vec4 shadow_far;
    vec4 sky_fx;
    vec4 retro;
    vec4 shadow_cells[144];
    vec4 shadow_clouds[112];
} g;

uniform mat4 u_view_proj[2];
uniform vec3 u_eye[2];
uniform vec3 u_forward[2];
uniform uint u_word;
uniform uint u_flags;
// The image's size in pixels, each eye's projection y scale, whether a dot batch carries a
// colour per dot.
uniform vec2 u_viewport;
uniform float u_proj_y[2];
uniform uint u_dot_colour;

const uint MAX_LOCAL_LIGHTS = 32u;
const uint LIGHT_SHADOWS = 1u;
const uint LIGHT_LOCAL = 2u;
const uint LIGHT_MOON_KEY = 4u;
const uint TAG_SMOKE = 8192u;
const uint TAG_SMOKE_FIRE = 16384u;
const uint TAG_WARM = 2048u;
const uint TAG_DOT_NEAR = 4096u;
const uint TAG_DOT_SPOT = 32768u;
const uint TAG_DOT_FAR = 2147483648u;
const uint TAG_VERTEX_HAZE = 1073741824u;
const float DOT_FAR_MIN = 3000.0;
const float AIRCRAFT_NIGHT = 0.95;
const uint CLOUD_RAMP = 0xE0u;
const uint WORD_PER_VERTEX = 0xFFFFFFFFu;
const float DOT_SIZE = 0.6;
const float DOT_MIN_PIXELS = 2.0;

#ifdef VERTEX
#ifndef SINGLE
layout(num_views = 2) in;
#endif
layout(location = 0) in vec3 a_position;
#ifdef DOT
layout(location = 2) in vec2 a_corner;
layout(location = 3) in uint a_colour;
flat out uint v_dot_colour;
// The quad corner (xy, -1..1) and the dot's size over the lamp's (z: above 1 a goggles halo).
out vec3 v_dot_uv;
const float NVG_HALO = 3.5;
#endif
// Per instance: the model matrix (locations 4..7) and the mesh tag.
layout(location = 4) in mat4 a_model;
layout(location = 8) in uint a_tag;
// The colour word of each vertex, with u_word WORD_PER_VERTEX (a mesh of several colours).
layout(location = 9) in uint a_word;
flat out uint v_word;
out vec3 v_world;
flat out uint v_fog;
flat out float v_fog_f;
// Bit 0: warm object or dot light (FLIR); bit 1: aircraft; bit 2: spotting dot.
flat out uint v_warm;
#ifdef CLOUD
// Smoke volumes, in the plume's local space (unit radius, y 0..1): position (w: noise offset),
// eye (w: 1 smoke, 2 smoke over a fire), sun direction, scale (metres per local unit).
out vec4 v_puff_pos;
flat out vec4 v_puff_eye;
flat out vec3 v_puff_sun;
flat out vec3 v_puff_scale;
#endif

const float AIRCRAFT_DOT_METRES = 0.3;
const float AIRCRAFT_DOT_MAX = 5.0;
const float DOT_RANGE = 30000.0;
const float DOT_VIS_FACTOR = 1.5;
const float DOT_NEAR = 0.4;
const float SPOT_PIXELS = 12.0;

float level_unit(uint code) {
    switch (code) {
        case 0u: return 0.5;
        case 1u: return 1.0;
        case 2u: return 4.0;
        case 3u: return 16.0;
        case 4u: return 64.0;
        default: return 0.125;
    }
}

void main() {
    vec3 eye = u_eye[VIEW];
    vec4 world = a_model * vec4(a_position, 1.0);
    v_world = world.xyz;
    gl_Position = u_view_proj[VIEW] * world;
    v_word = u_word == WORD_PER_VERTEX ? a_word : u_word;
    uint tag = a_tag;
    v_warm = ((tag & (TAG_WARM | 128u)) != 0u ? 1u : 0u) | (((tag & 7u) == 5u) ? 2u : 0u)
        | ((tag & (TAG_DOT_SPOT | 128u)) == (TAG_DOT_SPOT | 128u) ? 4u : 0u);
#ifdef CLOUD
    v_puff_pos = vec4(0.0);
    v_puff_eye = vec4(0.0);
    v_puff_sun = vec3(0.0);
    v_puff_scale = vec3(1.0);
    if ((tag & TAG_SMOKE) != 0u) {
        mat3 m = mat3(a_model);
        vec3 s2 = vec3(dot(m[0], m[0]), dot(m[1], m[1]), dot(m[2], m[2]));
        vec3 o = a_model[3].xyz;
        vec3 eye_local = (transpose(m) * (eye - o)) / s2;
        v_puff_pos = vec4(a_position, float(tag >> 16u) * 0.01);
        v_puff_eye = vec4(eye_local, (tag & TAG_SMOKE_FIRE) != 0u ? 2.0 : 1.0);
        v_puff_sun = normalize((transpose(m) * g.sun.xyz) / s2);
        v_puff_scale = sqrt(s2);
    }
#endif
#ifdef DOT
    // Spread by the quad corner to a constant size in pixels: in XR at most a 320x200 pixel's
    // angular size.
    v_dot_colour = u_dot_colour != 0u ? a_colour : (u_word & 0xFFu);
    v_dot_uv = vec3(a_corner * 2.0, 1.0);
    if (((tag >> 7u) & 1u) == 1u) {
        vec3 center = world.xyz;
        bool spot = (tag & TAG_DOT_SPOT) != 0u;
        float spot_m = float((tag >> 16u) & 0xFFu);
        if (spot) {
            center = center + normalize(eye - center) * spot_m;
        }
        vec4 clip = u_view_proj[VIEW] * vec4(center, 1.0);
        vec2 vp = u_viewport;
        float py = abs(u_proj_y[VIEW]);
        float px = max(vp.y / 200.0 * DOT_SIZE, DOT_MIN_PIXELS);
        if (g.dots.x > 0.0) {
            px = min(px, max(g.dots.x * 0.5 * vp.y * py, DOT_MIN_PIXELS));
        }
        if ((tag & 7u) == 5u) {
            float lamp = AIRCRAFT_DOT_METRES * 0.5 * vp.y * py / max(clip.w, 0.01);
            px = max(px, min(lamp, px * AIRCRAFT_DOT_MAX));
        }
        uint tenths = (tag >> 24u) & 0x7Fu;
        px = tenths != 0u ? px * float(tenths) / 10.0 : px * float(((tag >> 8u) & 7u) + 1u);
        bool bright = (tag & 7u) == 5u || tenths > 10u || ((tag >> 8u) & 7u) != 0u;
        if (g.light_info.z > 1.5 && bright && !spot) {
            px *= NVG_HALO;
            v_dot_uv.z = NVG_HALO;
        }
        v_world = center;
        gl_Position = clip + vec4(a_corner * px * 2.0 / vp * clip.w, 0.0, 0.0);
        float range = DOT_RANGE;
        if (g.weather.z > 0.0) {
            range = min(range, g.weather.z * DOT_VIS_FACTOR);
        }
        if ((tag & TAG_DOT_NEAR) != 0u) {
            range = range * DOT_NEAR;
        }
        float span = spot_m * 0.5 * vp.y * py / max(clip.w, 0.01);
        float dist = distance(center, eye);
        bool near = (tag & TAG_DOT_FAR) != 0u && dist < DOT_FAR_MIN;
        if (dist > range || near || (spot && span > SPOT_PIXELS)) {
            // Out of range: a degenerate quad outside the clip volume.
            gl_Position = vec4(0.0, 0.0, -1.0, 1.0);
        }
    }
#endif
    // Per-object haze: fog = clamp((depth/512 in object units) - sizeClass, 0, 31) >> 1
    uint size_class = (tag >> 3u) & 7u;
    uint no_fog = (tag >> 6u) & 1u;
    vec3 origin = a_model[3].xyz;
    if ((tag & TAG_VERTEX_HAZE) != 0u && (tag & (TAG_SMOKE | TAG_DOT_SPOT | 128u)) == 0u) {
        origin = world.xyz;
    }
    float depth_m = max(dot(origin - eye, u_forward[VIEW]), 0.0);
    float depth_units = depth_m * max(g.weather.x, 1.0) / (g.params.y * level_unit(tag & 7u));
    int d = int(depth_units / 512.0) - int(size_class);
    uint fog = uint(clamp(d, 0, 31)) >> 1u;
    float fog_f = clamp(depth_units / 512.0 - float(size_class), 0.0, 31.0) / 2.0;
    if (no_fog == 1u || g.params.x == 0.0) {
        fog = 0u;
        fog_f = 0.0;
    }
    v_fog = fog;
    v_fog_f = fog_f;
}
#endif

#ifdef FRAGMENT
#ifdef DOT
flat in uint v_dot_colour;
in vec3 v_dot_uv;
#endif
flat in uint v_word;
in vec3 v_world;
flat in uint v_fog;
flat in float v_fog_f;
flat in uint v_warm;
#ifdef CLOUD
in vec4 v_puff_pos;
flat in vec4 v_puff_eye;
flat in vec3 v_puff_sun;
flat in vec3 v_puff_scale;
#endif
layout(location = 0) out vec4 o_colour;

vec3 eye_pos() {
    return u_eye[VIEW];
}

float ring(vec4 arr[4], uint i) {
    return arr[i >> 2u][i & 3u];
}

// A recon camera's view (recon.rs), by its picture size: it sees by day and night alike.
bool recon_view() {
    return all(lessThan(abs(u_viewport - g.sky_night.zw), vec2(0.5)));
}

vec3 pal(uint i) {
    if (recon_view()) {
        return g.recon_palette[i & 0xFFu].rgb;
    }
    return g.palette[i & 0xFFu].rgb;
}

uint haze(uint idx, uint fog) {
    if (idx < 0xA0u || fog == 0u) {
        return idx;
    }
    return max(idx & 0xF0u, idx - min(fog, idx));
}

vec3 haze_smooth(uint idx, float fog) {
    if (idx < 0xA0u || fog <= 0.0) {
        return pal(idx);
    }
    uint lo = haze(idx, uint(floor(fog)));
    uint hi = haze(idx, uint(floor(fog)) + 1u);
    return mix(pal(lo), pal(hi), fract(fog));
}

uvec2 band(vec3 dir) {
    float horiz = max(length(dir.xz), 1e-6);
    float r = 256.0 * abs(dir.y) / horiz;
    uint k = 0u;
    if (dir.y >= 0.0) {
        for (uint i = 1u; i < 16u; i++) {
            if (r >= ring(g.sky_ring, i)) { k = i; }
        }
        return uvec2(0x80u + k, 1u);
    }
    for (uint i = 1u; i < 16u; i++) {
        if (r >= ring(g.ground_g, i)) { k = i; }
    }
    return uvec2(0x70u + k, 0u);
}

vec3 band_f(vec3 dir) {
    float horiz = max(length(dir.xz), 1e-6);
    float r = 256.0 * abs(dir.y) / horiz;
    bool sky = dir.y >= 0.0;
    uint base = sky ? 0x80u : 0x70u;
    uint k = 0u;
    for (uint i = 1u; i < 16u; i++) {
        if (r >= (sky ? ring(g.sky_ring, i) : ring(g.ground_g, i))) { k = i; }
    }
    float f = 0.0;
    if (k < 15u) {
        float a = sky ? ring(g.sky_ring, k) : ring(g.ground_g, k);
        float b = sky ? ring(g.sky_ring, k + 1u) : ring(g.ground_g, k + 1u);
        if (b > a) { f = clamp((r - a) / (b - a), 0.0, 1.0); }
    }
    return vec3(float(base), float(k), f);
}

const vec3 MOON_TINT = vec3(0.75, 0.85, 1.0);
const float SKY_LIGHT_MAX = 2.0;

vec3 sun_moon(vec3 n) {
    return g.ambient.rgb + vec3(g.sun.w * max(dot(n, g.sun.xyz), 0.0))
        + MOON_TINT * g.moon.w * max(dot(n, g.moon.xyz), 0.0);
}

vec3 sky_light(vec3 n) {
    return min(sun_moon(n) / sun_moon(vec3(0.0, 1.0, 0.0)), vec3(SKY_LIGHT_MAX));
}

vec3 shade(vec3 face, vec3 dir) {
    vec3 n = normalize(face);
    if (dot(n, dir) > 0.0) { n = -n; }
    return sky_light(n);
}

const vec2 GLOSS_AIRCRAFT = vec2(0.6, 48.0);
const vec2 GLOSS_OBJECT = vec2(0.2, 16.0);
const vec2 GLOSS_GROUND = vec2(0.15, 24.0);

// The lighting options: the palette colour lit by the sun and the moon (shadow maps not yet)
// and the local lights, with the flat face normal n.
// A spotlight's cone: cosines of the half angles where it starts to fade and where it is dark.
const float SPOT_INNER = 0.990;
const float SPOT_OUTER = 0.966;

// A spotlight's direction from its colour's w (lighting.rs `encode_spot`): 1 + u + 1024 v, the
// direction folded onto an octahedron.
vec3 spot_direction(float w) {
    float k = w - 1.0;
    float v = floor(k / 1024.0);
    vec2 o = vec2(k - v * 1024.0, v) / 1023.0 * 2.0 - 1.0;
    vec3 n = vec3(o.x, 1.0 - abs(o.x) - abs(o.y), o.y);
    float t = max(-n.y, 0.0);
    n.x += n.x >= 0.0 ? -t : t;
    n.z += n.z >= 0.0 ? -t : t;
    return normalize(n);
}

vec3 lit_surface(vec3 rgb, vec3 world, vec3 n, vec3 dir, uint flags, float dark, vec2 gloss) {
    vec3 light = g.ambient.rgb;
    vec3 spec = vec3(0.0);
    if ((flags & LIGHT_SHADOWS) != 0u) {
        vec3 sun_l = vec3(g.sun.w * max(dot(n, g.sun.xyz), 0.0));
        vec3 moon_l = MOON_TINT * g.moon.w * max(dot(n, g.moon.xyz), 0.0);
        light += sun_l + moon_l;
        light = min(light / sun_moon(vec3(0.0, 1.0, 0.0)), vec3(SKY_LIGHT_MAX));
    } else if (g.params.w != 0.0) {
        light = sky_light(n);
    } else {
        light = vec3(1.0);
    }
    light *= 1.0 - dark;
    if ((flags & LIGHT_LOCAL) != 0u) {
        uint count = min(uint(g.light_info.x), MAX_LOCAL_LIGHTS);
        vec3 to_eye = -normalize(dir);
        for (uint i = 0u; i < count; i++) {
            vec4 p = g.local_lights[2u * i];
            vec4 c = g.local_lights[2u * i + 1u];
            vec3 to_light = p.xyz - world;
            float d = length(to_light);
            if (d < p.w) {
                vec3 l = to_light / max(d, 1e-3);
                float att = (1.0 - d / p.w) * (1.0 - d / p.w);
                // A spotlight (lighting.rs `encode_spot`) lights its cone only.
                if (c.w > 0.5) {
                    att *= smoothstep(SPOT_OUTER, SPOT_INNER, dot(-l, spot_direction(c.w)));
                }
                float facing = dot(n, l);
                light += c.rgb * att * (max(facing, 0.0) * 0.7 + 0.3);
                if (facing > 0.0) {
                    vec3 h = normalize(l + to_eye);
                    spec += c.rgb * att * gloss.x * pow(max(dot(n, h), 0.0), gloss.y);
                }
            }
        }
    }
    return rgb * light + spec;
}

vec3 lit(vec3 rgb, vec3 world, vec3 face, vec3 dir, uint flags, float dark, vec2 gloss) {
    vec3 n = normalize(face);
    if (dot(n, dir) > 0.0) { n = -n; }
    return lit_surface(rgb, world, n, dir, flags, dark, gloss);
}

// Cloud texture: Perlin noise in world space, the lattice wrapping every NOISE_PERIOD metres.
const float NOISE_PERIOD = 81920.0;
const float NOISE_CELL = 1280.0;
const uint NOISE_OCTAVES = 5u;

uint noise_hash(ivec3 p) {
    uint h = (uint(p.x) * 0x8DA6B343u) ^ (uint(p.y) * 0xD8163841u) ^ (uint(p.z) * 0xCB1AB31Fu) ^ uint(g.noise.z);
    h ^= h >> 16u;
    h *= 0x7FEB352Du;
    h ^= h >> 15u;
    h *= 0x846CA68Bu;
    h ^= h >> 16u;
    return h;
}

float noise_grad(uint h, vec3 f) {
    uint gr = h % 12u;
    float u = gr < 8u ? f.x : f.y;
    float v = gr < 4u ? f.y : ((gr == 12u || gr == 14u) ? f.x : f.z);
    return ((gr & 1u) != 0u ? -u : u) + ((gr & 2u) != 0u ? -v : v);
}

float perlin(vec3 p, int period) {
    ivec3 i = ivec3(floor(p));
    vec3 f = fract(p);
    vec3 u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    float c[8];
    for (uint k = 0u; k < 8u; k++) {
        ivec3 o = ivec3(int(k & 1u), int((k >> 1u) & 1u), int((k >> 2u) & 1u));
        ivec3 q = i + o;
        ivec3 w = ivec3(((q.x % period) + period) % period, q.y, ((q.z % period) + period) % period);
        c[k] = noise_grad(noise_hash(w), f - vec3(o));
    }
    float x0 = mix(mix(c[0], c[1], u.x), mix(c[2], c[3], u.x), u.y);
    float x1 = mix(mix(c[4], c[5], u.x), mix(c[6], c[7], u.x), u.y);
    return mix(x0, x1, u.z);
}

float value_hash(int ix, int iy, uint seed) {
    uint h = (uint(ix) * 0x8DA6B343u) ^ (uint(iy) * 0xD8163841u) ^ (seed * 0xCB1AB31Fu);
    h ^= h >> 16u;
    h *= 0x7FEB352Du;
    h ^= h >> 15u;
    h *= 0x846CA68Bu;
    h ^= h >> 16u;
    return float(h >> 8u) / 16777216.0;
}

float value_noise(vec2 p, uint seed) {
    vec2 f = floor(p);
    ivec2 i = ivec2(f);
    vec2 t = p - f;
    vec2 u = t * t * (3.0 - 2.0 * t);
    float a = value_hash(i.x, i.y, seed);
    float b = value_hash(i.x + 1, i.y, seed);
    float c = value_hash(i.x, i.y + 1, seed);
    float d = value_hash(i.x + 1, i.y + 1, seed);
    float top = a + (b - a) * u.x;
    float bottom = c + (d - c) * u.x;
    return top + (bottom - top) * u.y;
}

// The stratus field at a render-space position (stealth_mission::Weather::stratus_field).
float stratus_field(vec3 world) {
    float map_unit = 32.0 * g.params.y;
    float east = world.x + g.stratus.z;
    float north = -(world.z + g.stratus.w);
    vec2 m = vec2(east / map_unit, 32768.0 - north / map_unit);
    uint seed = uint(g.stratus.y);
    return 0.6 * value_noise(m / 512.0, seed) + 0.3 * value_noise(m / 160.0, seed + 1u) + 0.1 * value_noise(m / 48.0, seed + 2u);
}

float stratus_cover(vec3 world) {
    if (g.clouds.z == 0.0) { return 0.0; }
    float thr = g.stratus.x;
    float local_ = smoothstep(thr, thr + 0.035, stratus_field(world));
    float d = length(world - eye_pos());
    return mix(local_, g.noise.w, smoothstep(25000.0, 70000.0, d));
}

float cloud_noise(vec3 world, float dist) {
    vec3 p = world + vec3(g.noise.x, 0.0, g.noise.y);
    float sum = 0.0;
    float amp = 0.6;
    float cell = NOISE_CELL;
    for (uint o = 0u; o < NOISE_OCTAVES; o++) {
        float fade = 1.0 - smoothstep(cell * 15.0, cell * 40.0, dist);
        sum += amp * fade * perlin(p / cell, int(NOISE_PERIOD / cell));
        amp *= 0.5;
        cell *= 0.5;
    }
    return sum;
}

vec3 cloud_ramp(float shade_, float dist, bool modern) {
    float k = clamp(shade_, 0.0, 14.0);
    float fog = clamp(dist / max(g.weather.z, 1.0) * 15.0, 0.0, 15.0);
    float f = max(k - fog, 0.0);
    if (modern) {
        return mix(pal(CLOUD_RAMP + uint(floor(f))), pal(CLOUD_RAMP + uint(floor(f)) + 1u), fract(f));
    }
    return pal(CLOUD_RAMP + uint(round(f)));
}

vec3 cloud_color(float noise, vec3 n, float dist, bool modern) {
    float shading = (1.0 - n.y) * 4.0 + 4.0 - 3.0 * max(dot(n, g.sun.xyz), 0.0);
    return cloud_ramp(shading + 4.5 * noise, dist, modern);
}

const float SUN_RADIUS = 0.014;

vec3 sun_colour(float h) {
    return mix(vec3(1.0, 0.42, 0.1), vec3(1.0, 0.93, 0.78), smoothstep(-0.02, 0.3, h));
}

// Modern shaders, continuous time of day: the sun's glare over everything in its direction.
vec3 sun_glare(vec3 dir) {
    if (g.sky_sun.w == 0.0 || g.params.w == 0.0) {
        return vec3(0.0);
    }
    vec3 s = normalize(g.sky_sun.xyz);
    float up = smoothstep(-0.03, 0.02, s.y);
    if (up <= 0.0) {
        return vec3(0.0);
    }
    float c = max(dot(normalize(dir), s), 0.0);
    float strength = clamp(g.sun.w / 0.3, 0.0, 1.0) * up;
    float glow = pow(c, 900.0) * 0.9 + pow(c, 60.0) * 0.16 + pow(c, 8.0) * 0.05;
    return sun_colour(s.y) * glow * strength;
}

const float MOON_RADIUS = 0.022;
const vec3 MOON_COLOR = vec3(0.82, 0.80, 0.74);
const float STAR_CELL = 0.012;
const float STAR_SHARE = 0.035;

float hash2(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

// palette.wgsl `aurora` and `meteor`.
vec3 aurora(vec3 nd) {
    float s = g.sky_fx.x;
    if (s <= 0.0 || nd.y <= 0.0) { return vec3(0.0); }
    float az = atan(nd.x, -nd.z);
    float north = exp(-az * az / 1.2);
    float t = g.dots.y;
    float above = asin(nd.y) - (0.18 + 0.06 * sin(az * 3.0 + t * 0.05) + 0.03 * sin(az * 7.0 - t * 0.08));
    if (above < -0.03 || north < 0.01) { return vec3(0.0); }
    float height = 0.25 + 0.08 * sin(az * 5.0 + t * 0.03);
    float curtain = smoothstep(-0.03, 0.01, above) * exp(-max(above, 0.0) / height * 2.2);
    float rays = 0.6 + 0.4 * sin(az * 90.0 + 3.0 * sin(az * 11.0 + t * 0.2) + t * 0.4) * (0.4 + 0.6 * sin(az * 23.0 - t * 0.15));
    vec3 c = mix(vec3(0.02, 0.28, 0.11), vec3(0.18, 0.02, 0.07), smoothstep(0.08, 0.3, above));
    return c * curtain * rays * north * s;
}

const float METEOR_PERIOD = 7.0;
const float METEOR_LIFE = 0.7;
float meteor(vec3 nd, float px) {
    if (g.sky_fx.y <= 0.0 || nd.y <= 0.0) { return 0.0; }
    float t = g.dots.y;
    float k = floor(t / METEOR_PERIOD);
    float p = (t - k * METEOR_PERIOD) / METEOR_LIFE;
    if (p > 1.0 || hash2(vec2(k, 3.7)) > 0.5) { return 0.0; }
    float az = hash2(vec2(k, 1.1)) * 6.2831853;
    float el = 0.35 + 0.6 * hash2(vec2(k, 2.3));
    vec3 start = vec3(cos(el) * sin(az), sin(el), -cos(el) * cos(az));
    vec3 east = normalize(cross(vec3(0.0, 1.0, 0.0), start));
    vec3 up = cross(start, east);
    float a = hash2(vec2(k, 5.9)) * 6.2831853;
    vec3 dir = normalize(cos(a) * east - (0.6 + 0.4 * abs(sin(a))) * up) * 0.25;
    vec3 head = normalize(start + dir * p);
    vec3 tail = normalize(start + dir * max(p - 0.35, 0.0));
    vec3 ab = head - tail;
    float along = clamp(dot(nd - tail, ab) / max(dot(ab, ab), 1e-8), 0.0, 1.0);
    float d = length(nd - tail - ab * along);
    float w = max(px * 0.8, 0.0006);
    return (1.0 - smoothstep(w * 0.5, w * 1.5, d)) * along * (1.0 - 0.6 * p);
}


// Continuous time of day: the sun, stars and the moon over the sky colour (`nd` normalised).
vec3 night_sky(vec3 bg_in, vec3 nd, float px) {
    vec3 bg = bg_in;
    float daylight = clamp(g.light_info.y, 0.0, 1.0);
    float stars = g.sky_moon.w;
    if (stars > 0.0) {
        float az = atan(nd.x, -nd.z);
        float el = asin(clamp(nd.y, -1.0, 1.0));
        float row = floor(el / STAR_CELL);
        float w = STAR_CELL / max(cos((row + 0.5) * STAR_CELL), 0.05);
        vec2 cell = vec2(floor(az / w), row);
        float h = hash2(cell);
        if (h < STAR_SHARE) {
            float c_az = (cell.x + 0.5 + 0.6 * (hash2(cell + 7.1) - 0.5)) * w;
            float c_el = (cell.y + 0.5 + 0.6 * (hash2(cell + 3.3) - 0.5)) * STAR_CELL;
            vec3 star = vec3(cos(c_el) * sin(c_az), sin(c_el), -cos(c_el) * cos(c_az));
            float d = length(nd - star);
            float r = max(STAR_CELL * 0.03, px * 0.55);
            float core = d < r ? 1.0 : 0.0;
            float mag = 0.35 + 0.65 * pow(h / STAR_SHARE, 3.0);
            float low = 1.0 - smoothstep(0.05, 0.5, nd.y);
            mag *= max(1.0 + g.sky_fx.z * low * 0.7 * sin(g.dots.y * (7.0 + 9.0 * hash2(cell + 1.9)) + h * 400.0), 0.0);
            float alt_fade = smoothstep(0.0, 0.15, nd.y);
            bg += vec3(0.9, 0.92, 1.0) * core * mag * stars * alt_fade;
        }
        bg += (aurora(nd) + vec3(1.0, 0.95, 0.85) * meteor(nd, px)) * stars;
    }
    vec3 sun = normalize(g.sky_sun.xyz);
    if (sun.y > -SUN_RADIUS) {
        float a = acos(clamp(dot(nd, sun), -1.0, 1.0));
        float edge = 1.0 - smoothstep(SUN_RADIUS - px, SUN_RADIUS + px, a);
        bg = mix(bg, sun_colour(sun.y) * 1.6, edge);
    }
    vec3 m = normalize(g.sky_moon.xyz);
    float c = dot(nd, m);
    if (m.y > -MOON_RADIUS && c > cos(MOON_RADIUS * 8.0)) {
        vec3 o = (nd - m * c) / MOON_RADIUS;
        float r2 = dot(o, o);
        float fw = px / MOON_RADIUS;
        float edge = 1.0 - smoothstep(1.0 - fw, 1.0 + fw, sqrt(r2));
        if (edge > 0.0) {
            vec3 n = normalize(o - m * sqrt(max(1.0 - r2, 0.0)));
            float lit_ = smoothstep(-0.05, 0.05, dot(n, normalize(g.sky_sun.xyz)));
            float sh = mix(0.04, 1.0, lit_) * mix(1.0, 0.35, daylight);
            vec3 disc = MOON_COLOR * sh;
            bg = mix(bg, max(bg, disc), edge);
        }
        float a = acos(min(c, 1.0)) / MOON_RADIUS;
        float halo = exp(-a * a * 0.25) * smoothstep(8.0, 5.0, a) * 0.12 * (1.0 - daylight);
        bg += MOON_COLOR * halo * g.sky_moon.w;
    }
    return bg;
}

float night_dark() {
    if (recon_view()) {
        return 0.0;
    }
    return g.sky_night.x;
}

vec3 night_red(uint k) {
    const float red[10] = float[10](64.0, 48.0, 36.0, 28.0, 24.0, 16.0, 12.0, 8.0, 6.0, 3.0);
    if (k >= 10u) { return vec3(0.0); }
    float r = red[k] / 255.0 * g.sky_night.y;
    return pow(min(vec3(r, 0.0, r * 0.07), vec3(1.0)), vec3(2.2));
}

vec3 night_background(vec3 bg, vec3 dir, vec3 b, bool modern) {
    float dark = night_dark();
    if (dark <= 0.0) {
        return bg;
    }
    vec3 c = bg * (1.0 - dark);
    if (dir.y >= 0.0) {
        uint k = uint(b.y);
        vec3 red = night_red(k);
        if (modern) {
            red = mix(red, night_red(k + 1u), b.z);
        }
        c += red * dark;
    }
    return c;
}

float bayer4(uvec2 p) {
    const float m[16] = float[16](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
    return (m[(p.y & 3u) * 4u + (p.x & 3u)] + 0.5) / 16.0;
}

// palette.wgsl `retro`: VGA colours (the headset has no scanlines).
vec3 retro(vec3 rgb, vec2 frag) {
    if (g.retro.x <= 0.0) { return rgb; }
    vec3 s = pow(max(rgb, vec3(0.0)), vec3(1.0 / 2.2));
    vec3 q = floor(s * 63.0 + bayer4(uvec2(frag))) / 63.0;
    return pow(min(q, vec3(1.0)), vec3(2.2));
}

// Inside a cloud everything but the cockpit fades into it; an overcast turns the day greyer.
vec3 whiteout(vec3 rgb, float dist) {
    float gloom = g.weather.w;
    vec3 grey = mix(rgb, vec3(dot(rgb, vec3(0.2126, 0.7152, 0.0722))), 0.4 * gloom) * (1.0 - 0.3 * gloom);
    float w = g.clouds.w * smoothstep(3.0, 60.0, dist);
    return mix(grey, pal(CLOUD_RAMP + 2u) * (1.0 - night_dark()), w);
}

// How thick the storm haze is at the visibility distance (1 - exp(-this)).
const float STORM_HAZE_DEPTH = 2.0;

// In a storm the air under the deck is hazy with rain, the more the deck covers the sky: the
// colour seen along view direction `nd` at `dist` metres fades into grey over the part of the
// ray below the deck's base.
const float MIST_DEPTH = 1200.0;

// palette.wgsl `ground_mist`.
vec3 ground_mist(vec3 rgb, vec3 world, float dist) {
    float s = g.atmos.x;
    float top = g.atmos.w + g.atmos.y;
    if (s <= 0.0 || world.y >= top) { return rgb; }
    float eye_y = eye_pos().y;
    float path = dist;
    if (eye_y > top) {
        float up = (eye_y - world.y) / max(dist, 1.0);
        path = min(dist, (top - world.y) / max(up, 1e-3));
    }
    float a = s * (1.0 - exp(-path / MIST_DEPTH));
    return mix(rgb, pal(CLOUD_RAMP) * (1.0 - night_dark()), a);
}

// palette.wgsl `cloud_shadow`.
const float SHADOW_GRID = 12.0;

float cloud_shadow(vec3 world) {
    float s = g.atmos.z;
    vec3 sun = g.sun.xyz;
    if (s <= 0.0 || sun.y < 0.05) { return 1.0; }
    float shade = 0.0;
    float base = g.clouds.x;
    if (g.clouds.z != 0.0 && world.y < base) {
        shade = g.shadow_far.y * stratus_cover(world + sun * ((base - world.y) / sun.y));
    }
    vec4 grid = g.shadow_grid;
    if (g.shadow_far.x > 0.0 && world.y < grid.w) {
        vec2 p = world.xz + sun.xz * ((grid.w - world.y) / sun.y);
        vec2 q = (p - grid.xy) / grid.z;
        float local = 0.0;
        if (all(greaterThanEqual(q, vec2(0.0))) && all(lessThan(q, vec2(SHADOW_GRID)))) {
            vec4 cell = g.shadow_cells[int(q.y) * int(SHADOW_GRID) + int(q.x)];
            for (int k = 0; k < 4; k++) {
                float n = cell[k];
                if (n < 0.5) { break; }
                vec4 c = g.shadow_clouds[int(n) - 1];
                local = max(local, c.w * (1.0 - smoothstep(0.45, 1.0, length(p - c.xy) / c.z)));
            }
        }
        vec2 off = abs(q - vec2(SHADOW_GRID * 0.5)) / (SHADOW_GRID * 0.5);
        shade = max(shade, mix(local, g.shadow_far.z, smoothstep(0.75, 1.0, max(off.x, off.y))));
    }
    return 1.0 - s * shade;
}

vec3 storm_haze(vec3 rgb, vec3 nd, float dist) {
    float s = g.storm_haze.x;
    if (s <= 0.0) {
        return rgb;
    }
    float base = g.clouds.x;
    float y = eye_pos().y;
    float path = 0.0;
    if (y < base) {
        path = nd.y > 0.0 ? min(dist, (base - y) / nd.y) : dist;
    } else if (nd.y < 0.0) {
        path = max(dist - (y - base) / -nd.y, 0.0);
    }
    float a = s * (1.0 - exp(-STORM_HAZE_DEPTH * path / g.storm_haze.y));
    return mix(rgb, pal(CLOUD_RAMP + 3u) * (1.0 - night_dark()), a);
}

const vec3 BOLT_COLOR = vec3(0.55, 0.58, 0.75);

float bolt_glow(vec3 world) {
    float i = g.bolt_a.w;
    if (i <= 0.0) { return 0.0; }
    vec3 a = g.bolt_a.xyz;
    vec3 ab = g.bolt_b.xyz - a;
    float t = clamp(dot(world - a, ab) / max(dot(ab, ab), 1e-3), 0.0, 1.0);
    float d = length(world - (a + ab * t));
    float reach = g.bolt_b.w;
    float core = d / (reach * 0.06);
    float halo = d / (reach * 0.3);
    return i * (1.0 - smoothstep(0.3 * reach, reach, d)) * (1.6 / (1.0 + core * core) + 0.3 / (1.0 + halo * halo));
}

float flash_scale() {
    return mix(1.0, 0.3, g.light_info.y);
}

vec3 burst_glow(vec3 world) {
    float i = g.burst.w;
    if (i <= 0.0) { return vec3(0.0); }
    float d = length(world - g.burst.xyz);
    float reach = g.burst_colour.w;
    float core = d / (reach * 0.06);
    float halo = d / (reach * 0.3);
    return g.burst_colour.rgb * i * (1.0 - smoothstep(0.3 * reach, reach, d)) * (1.6 / (1.0 + core * core) + 0.3 / (1.0 + halo * halo));
}

vec3 flash(vec3 rgb, vec3 world) {
    return rgb + (BOLT_COLOR * (g.weather.y + bolt_glow(world)) + burst_glow(world)) * flash_scale();
}

vec3 nvg(vec3 rgb) {
    if (g.light_info.z == 0.0) { return rgb; }
    float lum = dot(rgb, vec3(0.2126, 0.7152, 0.0722));
    float gain = mix(14.0, 2.5, clamp(g.light_info.y, 0.0, 1.0));
    float v = smoothstep(0.12, 0.95, sqrt(1.0 - exp(-lum * gain)));
    uvec2 p = uvec2(gl_FragCoord.xy);
    uint h = p.x * 0x27d4eb2du ^ p.y * 0x165667b1u ^ uint(g.dots.y * 60.0) * 0x9e3779b9u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    // Scintillation: the tube's grain, with the goggles effect strongest in the dark.
    float grain = g.light_info.z > 1.5 ? mix(0.16, 0.05, v) : 0.06;
    v = clamp(v + (float(h >> 8u) / 16777216.0 - 0.5) * grain, 0.0, 1.0);
    vec3 c = mix(vec3(0.0, 0.04, 0.01), vec3(0.3, 0.85, 0.2), v) + vec3(0.5, 0.15, 0.4) * v * v * v;
    return pow(min(c, vec3(1.0)), vec3(2.2));
}

// The tracking camera's view in FLIR, by its picture size.
bool flir_view() {
    return g.params.z != 0.0 && all(lessThan(abs(u_viewport - g.dots.zw), vec2(0.5)));
}

vec3 flir(vec3 rgb) {
    float s = sqrt(dot(rgb, vec3(0.2126, 0.7152, 0.0722)));
    float cold = 0.3 + 0.25 * s;
    float v = cold;
    if ((v_warm & 1u) != 0u) {
        v = mix(0.85 + 0.15 * s, cold, clamp(v_fog_f / 24.0, 0.0, 0.6));
    }
    return vec3(pow(v, 2.2));
}

#ifdef CLOUD
const uint SMOKE_STEPS = 12u;
const uint SMOKE_STEPS_XR = 6u;
const float SMOKE_LEAN = 0.4;
const float SMOKE_R0 = 0.06;
const float SMOKE_R1 = 0.58;
const float SMOKE_RISE = 9.0;
const vec3 FIRE_COLOR = vec3(1.0, 0.42, 0.1);
const float SMOKE_PROXY = 1.0196;

vec2 cylinder_bounds(vec3 ro, vec3 rd) {
    float a = rd.x * rd.x + rd.z * rd.z;
    float b = ro.x * rd.x + ro.z * rd.z;
    float c = ro.x * ro.x + ro.z * ro.z - 1.0;
    vec2 t = vec2(-1e9, 1e9);
    if (a > 1e-9) {
        float disc = b * b - a * c;
        if (disc < 0.0) { return vec2(1.0, 0.0); }
        float sq = sqrt(disc);
        t = vec2((-b - sq) / a, (-b + sq) / a);
    } else if (c > 0.0) {
        return vec2(1.0, 0.0);
    }
    if (abs(rd.y) > 1e-6) {
        vec2 ty = vec2(-ro.y / rd.y, (1.0 - ro.y) / rd.y);
        t = vec2(max(t.x, min(ty.x, ty.y)), min(t.y, max(ty.x, ty.y)));
    } else if (ro.y < 0.0 || ro.y > 1.0) {
        return vec2(1.0, 0.0);
    }
    return t;
}

bool proxy_fragment(vec3 p, vec3 ro) {
    bool inside = dot(ro.xz, ro.xz) < SMOKE_PROXY * SMOKE_PROXY && ro.y > 0.0 && ro.y < 1.0;
    vec3 n;
    if (p.y > 0.9999) {
        n = vec3(0.0, 1.0, 0.0);
    } else if (p.y < 0.0001) {
        n = vec3(0.0, -1.0, 0.0);
    } else {
        n = vec3(p.x, 0.0, p.z);
    }
    return inside || dot(n, ro - p) > 0.0;
}

float smoke_density(vec3 p, vec3 aspect, float seed, float rise, float start, bool detail) {
    float y = p.y;
    if (y <= 0.0 || y >= 1.0) { return 0.0; }
    float cx = SMOKE_LEAN * pow(y, 1.6);
    float r = mix(SMOKE_R0, SMOKE_R1, pow(y, 0.7));
    float shape_ = 1.0 - length(vec2(p.x - cx, p.z)) / r;
    shape_ -= smoothstep(0.55, 1.0, y) * 1.1;
    if (shape_ < -0.6) { return 0.0; }
    vec3 pn = p * aspect + vec3(seed, -rise, seed * 0.7);
    float n = perlin(pn * 2.2, 1024);
    if (detail) {
        n += 0.5 * perlin((pn + vec3(0.0, -rise * 0.6, 0.0)) * 5.3, 1024);
    }
    return clamp((shape_ + 0.65 * n) * 3.0, 0.0, 1.0) * (1.0 - 0.55 * y) * smoothstep(start * 0.3, start * 2.0, y);
}

vec4 smoke_volume(vec3 dir, bool modern) {
    float dist = length(dir);
    bool fire = v_puff_eye.w > 1.5;
    vec3 ro = v_puff_eye.xyz;
    vec3 rd = v_puff_pos.xyz - ro;
    if (!proxy_fragment(v_puff_pos.xyz, ro)) { discard; }
    vec2 tb = cylinder_bounds(ro, rd);
    float t0 = max(tb.x, 0.0);
    float t1 = tb.y;
    if (t1 <= t0) { discard; }
    vec3 aspect = v_puff_scale / v_puff_scale.x;
    float m_per_t = length(rd * v_puff_scale);
    bool xr = g.dots.x > 0.0;
    uint steps = xr ? SMOKE_STEPS_XR : SMOKE_STEPS;
    float dt = (t1 - t0) / float(steps);
    float ds = dt * m_per_t;
    float thick = v_puff_scale.x * 0.2;
    float rise = g.dots.y * SMOKE_RISE / v_puff_scale.x;
    float jitter = bayer4(uvec2(gl_FragCoord.xy));
    vec3 sun = v_puff_sun * 0.08;
    float start = fire ? v_puff_scale.x * SMOKE_R0 / v_puff_scale.y : 0.0;
    float glow_top = start * 4.0;
    float trans = 1.0;
    float light = 0.0;
    float glow = 0.0;
    for (uint i = 0u; i < steps; i++) {
        vec3 p = ro + rd * (t0 + (float(i) + jitter) * dt);
        float dens = smoke_density(p, aspect, v_puff_pos.w, rise, start, !xr);
        if (dens <= 0.001) { continue; }
        float a = 1.0 - exp(-dens * ds / thick);
        float lit_ = clamp(0.5 + 0.25 * p.y, 0.0, 1.0);
        if (!xr) {
            float towards_sun = smoke_density(p + sun, aspect, v_puff_pos.w, rise, start, false);
            lit_ = clamp(0.4 + (dens - towards_sun) * 1.4 + 0.25 * p.y, 0.0, 1.0);
        }
        light += trans * a * lit_;
        if (fire) {
            float gg = 1.0 - smoothstep(start, glow_top, p.y);
            glow += trans * a * gg * gg;
        }
        trans *= 1.0 - a;
        if (trans < 0.03) { break; }
    }
    float alpha = 1.0 - trans;
    if (alpha < 0.01) { discard; }
    vec3 rgb = cloud_ramp(7.0 + (1.0 - light / alpha) * 6.5, dist, modern);
    rgb += FIRE_COLOR * min(glow / alpha, 1.0) * mix(0.5, 0.15, g.light_info.y);
    rgb = flash(whiteout(storm_haze(rgb, normalize(dir), dist), dist), eye_pos() + dir);
    if (!modern) {
        if (alpha < bayer4(uvec2(gl_FragCoord.xy) + uvec2(2u, 1u))) { discard; }
        alpha = 1.0;
    }
    return vec4(rgb, alpha);
}
#endif

// Water (palette_material.rs FLAG_WATER): sparse specks of about a pixel, lighter and
// darker, pinned to the world at random spots of a grid (the floating origin is handed over
// modulo NOISE_PERIOD, a multiple of the spacing), so the height over the sea can be judged
// low down. They fade out with height, and in the distance before they crowd into noise.
const uint FLAG_WATER = 256u;
const float WATER_GRID = 12.0;
// Share of the grid cells with a speck.
const float WATER_SPECKS = 0.35;
const vec2 WATER_DOTS_HIGH = vec2(80.0, 300.0);
// Least speck radius up close (metres); otherwise a dot light's size.
const float WATER_SPECK_M = 0.25;

// A dot light's size in this view's pixels (the vertex shader's DOT).
float dot_pixels() {
    vec2 vp = u_viewport;
    float px = max(vp.y / 200.0 * DOT_SIZE, DOT_MIN_PIXELS);
    if (g.dots.x > 0.0) {
        px = min(px, max(g.dots.x * 0.5 * vp.y * abs(u_proj_y[VIEW]), DOT_MIN_PIXELS));
    }
    return px;
}

// A ground plane offset in screen pixels, from the plane's change per pixel across and down.
vec2 pixels(vec2 off, vec2 dx, vec2 dy) {
    float det = dx.x * dy.y - dx.y * dy.x;
    if (abs(det) < 1e-9) {
        return vec2(1e4);
    }
    return vec2(off.x * dy.y - off.y * dy.x, dx.x * off.y - dx.y * off.x) / det;
}

vec3 water_dots(vec3 rgb, vec3 world, vec3 face, float foot, vec2 gdx, vec2 gdy, bool modern) {
    float h = eye_pos().y - world.y;
    bool level = abs(normalize(face).y) > 0.95;
    float fade = (1.0 - smoothstep(WATER_DOTS_HIGH.x, WATER_DOTS_HIGH.y, h))
        * (1.0 - smoothstep(0.15, 0.4, foot / WATER_GRID));
    if (!level || fade <= 0.0) {
        return rgb;
    }
    vec2 p = (world.xz + g.noise.xy) / WATER_GRID;
    vec2 cell = floor(p);
    float n = hash2(cell);
    if (n >= WATER_SPECKS) {
        return rgb;
    }
    vec2 at = cell + vec2(hash2(cell + 17.0), hash2(cell + 43.0));
    // Distance from the speck in screen pixels (a little world size up close).
    vec2 off = (p - at) * WATER_GRID;
    float px = length(pixels(off, gdx, gdy));
    float r = max(dot_pixels() * 0.5, WATER_SPECK_M / max(foot, 1e-3));
    float k = modern ? 1.0 - smoothstep(r - 0.3, r + 0.3, px) : (px < r ? 1.0 : 0.0);
    vec3 tone = n < WATER_SPECKS * 0.5 ? rgb * 0.4 + vec3(0.4) : rgb * 0.4;
    return mix(rgb, tone, k * fade);
}

vec4 shade_fragment(vec3 face, float sky_px, float foot, vec2 gdx, vec2 gdy) {
    vec3 dir = v_world - eye_pos();
    bool modern = g.params.w != 0.0;
#if defined(CLOUD)
    if ((uint(g.light_info.w) & 8u) != 0u) {
        return vec4(pal(CLOUD_RAMP + 4u), 1.0);
    }
    float d = length(dir);
    if (v_puff_eye.w > 0.0) {
        return smoke_volume(dir, modern);
    }
    // Stratus sheet: only where the layer's coverage field says so.
    float cover = stratus_cover(v_world);
    if (modern) {
        if (cover <= 0.0) { discard; }
    } else {
        if (cover < bayer4(uvec2(gl_FragCoord.xy))) { discard; }
        cover = 1.0;
    }
    vec3 n = normalize(face);
    if (dot(n, dir) > 0.0) { n = -n; }
    float noise = cloud_noise(v_world, d);
    vec3 c = cloud_color(noise, n, d, modern) * (1.0 - night_dark());
    return vec4(flash(whiteout(storm_haze(c, dir / d, d), d), v_world), cover);
#elif defined(BACKGROUND)
    if ((uint(g.light_info.w) & 2u) != 0u) {
        return vec4(pal(dir.y >= 0.0 ? 0x88u : 0x7Cu), 1.0);
    }
    // The FLIR camera's background: flat sky and ground.
    if ((u_flags & 4u) != 0u) {
        return vec4(pal(dir.y >= 0.0 ? 0x88u : 0x7Cu), 1.0);
    }
    vec3 bg;
    vec3 b = band_f(dir);
    if (modern) {
        uint k = uint(b.y);
        uint k1 = min(k + 1u, 15u);
        bg = mix(pal(uint(b.x) + k), pal(uint(b.x) + k1), b.z);
    } else {
        bg = pal(band(dir).x);
    }
    vec3 eye = eye_pos();
    bool ground_lit = false;
#ifdef LIT
    uint flags = uint(g.ambient.w);
    if (flags != 0u && dir.y < 0.0 && eye.y > 0.0) {
        vec3 ground = eye + dir * (-eye.y / dir.y);
        bg = lit_surface(bg, ground, vec3(0.0, 1.0, 0.0), dir, flags, night_dark(), GLOSS_GROUND);
        ground_lit = true;
    }
#endif
    if (!ground_lit) {
        bg = night_background(bg, dir, b, modern);
    }
    if (g.sky_sun.w > 0.0 && dir.y > 0.0) {
        bg = night_sky(bg, normalize(dir), sky_px);
    }
    // Stratus beyond its planes: its underside over the sky below it, its top over the
    // ground above it.
    if (g.clouds.z != 0.0) {
        float eye_y = eye.y;
        vec3 nd = normalize(dir);
        float dd = -1.0;
        vec3 n = vec3(0.0, -1.0, 0.0);
        if (eye_y <= g.clouds.x && nd.y > 0.0) {
            dd = (g.clouds.x - eye_y) / nd.y;
        } else if (eye_y >= g.clouds.y && nd.y < 0.0) {
            dd = (eye_y - g.clouds.y) / -nd.y;
            n = vec3(0.0, 1.0, 0.0);
        } else if (eye_y > g.clouds.x && eye_y < g.clouds.y) {
            bg = mix(bg, pal(CLOUD_RAMP + 2u) * (1.0 - night_dark()), stratus_cover(eye));
        }
        if (dd > 0.0) {
            vec3 hit = eye + nd * dd;
            vec3 c = cloud_color(cloud_noise(hit, dd), n, dd, modern) * (1.0 - night_dark());
            bg = mix(bg, c, stratus_cover(hit));
        }
    }
    // Lightning: the channel's light where the view ray meets cloud or ground, and the glow
    // of a strike over the sky in its direction.
    vec3 to_bolt = (g.bolt_a.xyz + g.bolt_b.xyz) * 0.5 - eye;
    float bolt_d = length(to_bolt);
    float along = min(bolt_d, 60000.0);
    float sheet = pow(max(dot(normalize(dir), to_bolt / max(bolt_d, 1.0)), 0.0), 6.0) * g.bolt_a.w * 0.25 * flash_scale();
    // The ground gradient lies on the plane y = 0; the sky is far.
    vec3 nd = normalize(dir);
    float haze_d = nd.y < 0.0 ? max(eye.y, 0.0) / -nd.y : 1e9;
    if (nd.y < 0.0) {
        vec3 ground = eye + nd * haze_d;
        bg = ground_mist(bg * cloud_shadow(ground), ground, haze_d);
    }
    bg = storm_haze(bg, nd, haze_d);
    bg = flash(whiteout(bg, 1e4), eye + nd * along) + BOLT_COLOR * sheet;
    if (g.burst.w > 0.0) {
        float burst_d = length(g.burst.xyz - eye);
        float far = smoothstep(0.5, 2.0, burst_d / g.burst_colour.w);
        bg += burst_glow(eye + nd * burst_d) * flash_scale() * far;
    }
    return vec4(bg, 1.0);
#else
    uint hi = (v_word >> 8u) & 0xFFu;
    uint lo = v_word & 0xFFu;
#ifdef DOT
    lo = v_dot_colour;
#endif
    if ((uint(g.light_info.w) & 4u) != 0u) {
        return vec4(pal(haze(lo, v_fog)), 1.0);
    }
    uvec2 px = uvec2(gl_FragCoord.xy);
    bool is_line = (u_flags & 2u) != 0u;
    vec3 rgb;
    if (is_line || hi == 0xFFu) {
        rgb = modern ? haze_smooth(lo, v_fog_f) : pal(haze(lo, v_fog));
    } else if (hi == 0xFBu) {
        discard;
    } else if (hi == 0xFCu) {
        if (((px.x + px.y) & 1u) == 1u) { discard; }
        rgb = pal(lo);
    } else if (hi == 0xFEu || hi == 0xFDu) {
        if (modern) {
            vec3 b = band_f(dir);
            uint k = uint(b.x) + uint(b.y);
            uint k1 = uint(b.x) + min(uint(b.y) + 1u, 15u);
            uint c0 = hi == 0xFEu ? (k & lo) : (k | lo);
            uint c1 = hi == 0xFEu ? (k1 & lo) : (k1 | lo);
            rgb = mix(pal(c0), pal(c1), b.z);
        } else {
            uint k = band(dir).x;
            rgb = pal(hi == 0xFEu ? (k & lo) : (k | lo));
        }
    } else {
        rgb = modern ? (pal(lo) + pal(hi)) * 0.5 : pal(((px.x + px.y) & 1u) == 1u ? hi : lo);
    }
    if ((u_flags & FLAG_WATER) != 0u) {
        rgb = water_dots(rgb, v_world, face, foot, gdx, gdy, modern);
    }
    // A dark night blacks out the scenery; lights (lines and dots) show, aircraft barely.
    float dark = (is_line && (u_flags & 64u) == 0u) ? 0.0 : night_dark();
    if ((v_warm & 4u) != 0u) {
        dark = night_dark() * AIRCRAFT_NIGHT;
    } else if ((v_warm & 2u) != 0u) {
        dark *= AIRCRAFT_NIGHT;
    }
#ifdef LIT
    uint flags = uint(g.ambient.w);
    if (!is_line && flags != 0u) {
        vec2 gloss = (v_warm & 2u) != 0u ? GLOSS_AIRCRAFT : GLOSS_OBJECT;
        rgb = lit(rgb, v_world, face, dir, flags, dark, gloss);
    } else
#endif
    {
        if (modern && !is_line) {
            rgb = rgb * shade(face, dir);
        }
        rgb = rgb * (1.0 - dark);
    }
    float dl = length(dir);
    if (!is_line) {
        rgb = ground_mist(rgb * cloud_shadow(v_world), v_world, dl);
    }
    float alpha = 1.0;
#ifdef DOT
    if (v_dot_uv.z > 1.0) {
        float r = (length(v_dot_uv.xy) * v_dot_uv.z - 1.0) / (v_dot_uv.z - 1.0);
        float k = clamp(1.0 - r, 0.0, 1.0);
        alpha = r <= 0.0 ? 1.0 : 0.6 * k * k;
    }
#endif
    return vec4(flash(whiteout(storm_haze(rgb, dir / max(dl, 1e-3), dl), dl), v_world), alpha);
#endif
}

void main() {
    vec3 face = cross(dFdx(v_world), dFdy(v_world));
    float sky_px = max(length(fwidth(normalize(v_world - eye_pos()))), 1e-5);
    float foot = length(fwidth(v_world));
    vec2 gdx = dFdx(v_world.xz);
    vec2 gdy = dFdy(v_world.xz);
    if ((uint(g.light_info.w) & 1u) != 0u) {
        o_colour = vec4(pal(v_word & 0xFFu), 1.0);
        return;
    }
    if (flir_view()) {
#ifdef BACKGROUND
        if ((u_flags & 4u) != 0u) {
            vec3 dir = v_world - eye_pos();
            o_colour = vec4(vec3(pow(dir.y < 0.0 ? 0.35 : 0.2, 2.2)), 1.0);
            return;
        }
#endif
        vec4 c = shade_fragment(face, sky_px, foot, gdx, gdy);
        o_colour = vec4(flir(c.rgb), c.a);
        return;
    }
    vec4 c = shade_fragment(face, sky_px, foot, gdx, gdy);
    vec3 glare = sun_glare(v_world - eye_pos());
    o_colour = vec4(retro(nvg(c.rgb + glare * c.a), gl_FragCoord.xy), c.a);
}
#endif

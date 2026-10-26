// Effects for the OpenGL headset renderer (src/glvr), as their WGSL shaders: explosion
// particles (particle.wgsl, PARTICLE), smoke trails and contrails (trail.wgsl, TRAIL), blurred
// rotor discs (rotor.wgsl, ROTOR), lightning channels (bolt.wgsl, BOLT), the afterburner's
// plume (flame.wgsl, FLAME) and the landing lights' beams (beam.wgsl, BEAM, on the flame's
// vertex shader). Compiled with
// VERTEX or FRAGMENT and one of those defined; both eyes in one pass. The recon look is left
// out; the tracking camera's picture (drawn offscreen) can be a FLIR view.

layout(std140) uniform Globals {
    vec4 palette[256];
    vec4 sky_ring[4];
    vec4 ground_g[4];
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
} g;

uniform mat4 u_view_proj[2];
uniform vec3 u_eye[2];
uniform vec3 u_right[2];
uniform vec3 u_up[2];
uniform mat4 u_model;
uniform vec2 u_viewport;
// The material's uniform, as its bytes (rotor_blur.rs `RotorParams`, the trail's, the bolt's,
// the flame's and the beam's `params`).
uniform uvec4 u_raw[2];

vec4 raw_vec4(int i) {
    return uintBitsToFloat(u_raw[i]);
}

#ifdef VERTEX
#ifndef SINGLE
layout(num_views = 2) in;
#endif
layout(location = 0) in vec3 a_position;
#ifdef PARTICLE
// xy: corner (-1 .. 1), z: radius (metres), w: rotation (radians).
layout(location = 1) in vec4 a_corner;
layout(location = 2) in vec4 a_colour;
// x: glow 0 .. 1, y: noise seed.
layout(location = 3) in vec2 a_look;
out vec2 v_uv;
out vec4 v_colour;
out vec2 v_look;
#endif
#if defined(TRAIL) || defined(BOLT)
layout(location = 1) in vec3 a_tangent;
// x: signed half width (metres), y: alpha (trails) or branch brightness (bolts).
layout(location = 2) in vec2 a_side;
out vec2 v_side;
#endif
#ifdef ROTOR
layout(location = 1) in vec2 a_uv;
out vec3 v_world;
out vec2 v_uv;
flat out vec3 v_axis_u;
flat out vec3 v_axis_v;
flat out vec3 v_normal;
#endif
#if defined(FLAME) || defined(BEAM)
// x: 0 (nozzle) .. 1 (tip), y: layer (0 core, 1 sheath, 2 nozzle exit).
layout(location = 1) in vec2 a_uv;
out vec3 v_world;
out vec3 v_normal;
out vec2 v_uv;
#endif

void main() {
    vec3 eye = u_eye[VIEW];
    vec3 centre = (u_model * vec4(a_position, 1.0)).xyz;
#ifdef PARTICLE
    float c = cos(a_corner.w);
    float s = sin(a_corner.w);
    vec2 k = vec2(a_corner.x * c - a_corner.y * s, a_corner.x * s + a_corner.y * c) * a_corner.z;
    gl_Position = u_view_proj[VIEW]
        * vec4(centre + u_right[VIEW] * k.x + u_up[VIEW] * k.y, 1.0);
    v_uv = a_corner.xy;
    v_colour = a_colour;
    v_look = a_look;
#endif
#if defined(TRAIL) || defined(BOLT)
    vec3 t = normalize((u_model * vec4(a_tangent, 0.0)).xyz);
    vec3 to_eye = eye - centre;
    vec3 s = cross(t, to_eye);
    if (dot(s, s) < 1e-8) {
        s = cross(t, vec3(0.0, 1.0, 0.0));
    }
    s = normalize(s);
#ifdef BOLT
    float w = max(abs(a_side.x), length(to_eye) * raw_vec4(0).y) * sign(a_side.x);
#else
    float w = a_side.x;
#endif
    gl_Position = u_view_proj[VIEW] * vec4(centre + s * w, 1.0);
    v_side = vec2(sign(a_side.x), a_side.y);
#endif
#ifdef ROTOR
    mat3 m = mat3(u_model);
    uint axis = u_raw[1].y;
    vec3 u = vec3(1.0, 0.0, 0.0);
    vec3 w = vec3(0.0, 0.0, 1.0);
    vec3 n = vec3(0.0, 1.0, 0.0);
    if (axis == 1u) {
        u = vec3(0.0, 0.0, 1.0);
        w = vec3(0.0, 1.0, 0.0);
        n = vec3(1.0, 0.0, 0.0);
    } else if (axis == 2u) {
        u = vec3(0.0, 1.0, 0.0);
        w = vec3(1.0, 0.0, 0.0);
        n = vec3(0.0, 0.0, 1.0);
    }
    v_world = centre;
    gl_Position = u_view_proj[VIEW] * vec4(centre, 1.0);
    v_uv = a_uv;
    v_axis_u = normalize(m * u);
    v_axis_v = normalize(m * w);
    v_normal = normalize(m * n);
#endif
#if defined(FLAME) || defined(BEAM)
    // The cones' normals point out from the axis, the disc's along it.
    vec3 n = vec3(a_position.xy, 0.0);
    if (a_uv.y > 1.5 || dot(n, n) < 1e-8) {
        n = vec3(0.0, 0.0, 1.0);
    }
    v_world = centre;
    v_normal = mat3(u_model) * n;
    v_uv = a_uv;
    gl_Position = u_view_proj[VIEW] * vec4(centre, 1.0);
#endif
}
#endif

#ifdef FRAGMENT
layout(location = 0) out vec4 o_colour;
#ifdef PARTICLE
in vec2 v_uv;
in vec4 v_colour;
in vec2 v_look;
#endif
#if defined(TRAIL) || defined(BOLT)
in vec2 v_side;
#endif
#ifdef ROTOR
in vec3 v_world;
in vec2 v_uv;
flat in vec3 v_axis_u;
flat in vec3 v_axis_v;
flat in vec3 v_normal;
#endif
#if defined(FLAME) || defined(BEAM)
in vec3 v_world;
in vec3 v_normal;
in vec2 v_uv;
#endif

// Night vision goggles (light_info.z): the picture intensified to a green phosphor one.
vec3 nvg(vec3 rgb) {
    if (g.light_info.z == 0.0) { return rgb; }
    float lum = dot(rgb, vec3(0.2126, 0.7152, 0.0722));
    float gain = mix(14.0, 2.5, clamp(g.light_info.y, 0.0, 1.0));
    float v = smoothstep(0.12, 0.95, sqrt(1.0 - exp(-lum * gain)));
    uvec2 p = uvec2(gl_FragCoord.xy);
    uint h = p.x * 0x27d4eb2du ^ p.y * 0x165667b1u ^ uint(g.dots.y * 60.0) * 0x9e3779b9u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    v = clamp(v + (float(h >> 8u) / 16777216.0 - 0.5) * 0.06, 0.0, 1.0);
    vec3 c = mix(vec3(0.0, 0.04, 0.01), vec3(0.3, 0.85, 0.2), v) + vec3(0.5, 0.15, 0.4) * v * v * v;
    return pow(min(c, vec3(1.0)), vec3(2.2));
}

// The tracking camera's view in FLIR, by its picture size.
bool flir_view() {
    return g.params.z != 0.0 && all(lessThan(abs(u_viewport - g.dots.zw), vec2(0.5)));
}

float bayer4(uvec2 p) {
    const float m[16] = float[16](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
    return (m[(p.y & 3u) * 4u + (p.x & 3u)] + 0.5) / 16.0;
}

#ifdef PARTICLE
const float NIGHT = 0.18;

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float noise2(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}

void main() {
    float r = length(v_uv);
    if (r >= 1.0) { discard; }
    float n = noise2(v_uv * 2.6 + vec2(v_look.y * 17.0, v_look.y * 31.0));
    float edge = 0.55 + 0.45 * n;
    float a = v_colour.a * (1.0 - smoothstep(edge * 0.55, edge, r));
    if (a <= 0.004) { discard; }
    float glow = v_look.x;
    vec3 rgb = v_colour.rgb * (1.0 + glow * (1.0 - r) * 0.9);
    float light = mix(NIGHT, 1.0, clamp(g.light_info.y, 0.0, 1.0));
    rgb = mix(rgb * light * (0.8 + 0.35 * n + 0.15 * v_uv.y), rgb, glow);
    // FLIR: hot fire white, smoke a cool grey.
    if (flir_view()) {
        rgb = mix(vec3(0.12), vec3(1.0), glow);
    } else {
        rgb = nvg(rgb);
    }
    if (g.params.w == 0.0) {
        if (a < bayer4(uvec2(gl_FragCoord.xy) + uvec2(1u, 3u))) { discard; }
        a = 1.0;
    }
    o_colour = vec4(rgb, a);
}
#endif

#ifdef TRAIL
const float NIGHT = 0.22;

void main() {
    vec4 params = raw_vec4(0);
    float edge = 1.0 - v_side.x * v_side.x;
    float a = clamp(v_side.y, 0.0, 1.0) * edge * params.y;
    if (a <= 0.003) { discard; }
    // FLIR: cool grey smoke.
    if (flir_view()) {
        o_colour = vec4(vec3(0.12), a);
        return;
    }
    float light = mix(NIGHT, 1.0, clamp(g.light_info.y, 0.0, 1.0));
    o_colour = vec4(nvg(g.palette[uint(params.x) & 0xFFu].rgb * light), a);
}
#endif

#ifdef BOLT
const vec3 CORE = vec3(1.0, 1.0, 1.0);
const vec3 EDGE = vec3(0.35, 0.5, 1.0);

void main() {
    float u = abs(v_side.x);
    float a = (1.0 - smoothstep(0.3, 1.0, u)) * raw_vec4(0).x * v_side.y;
    if (a <= 0.003) { discard; }
    o_colour = vec4(mix(CORE * 1.6, EDGE, smoothstep(0.12, 0.7, u)), a);
}
#endif

#ifdef FLAME
const float TAU = 6.28318530718;
const vec3 CORE_HOT = vec3(1.0, 0.85, 0.55);
const vec3 CORE_TIP = vec3(1.0, 0.45, 0.12);
const vec3 SHEATH_ROOT = vec3(0.4, 0.32, 1.0);
const vec3 SHEATH = vec3(1.0, 0.42, 0.12);
const vec3 EXIT = vec3(1.0, 0.62, 0.28);
const float DIAMONDS = 3.5;

void main() {
    float t = clamp(v_uv.x, 0.0, 1.0);
    float layer = v_uv.y;
    vec4 p = raw_vec4(0);
    float facing = abs(dot(normalize(v_normal), normalize(u_eye[VIEW] - v_world)));
    float flicker = 0.85 + 0.1 * sin(p.y * 47.0 + t * 9.0) + 0.05 * sin(p.y * 83.0 - t * 21.0);
    vec3 rgb;
    float a;
    if (layer > 1.5) {
        rgb = EXIT * 1.8;
        a = 0.9;
    } else if (layer > 0.5) {
        rgb = mix(SHEATH_ROOT, SHEATH, smoothstep(0.1, 0.35, t));
        a = 0.35 * pow(facing, 1.5) * smoothstep(0.0, 0.08, t) * pow(1.0 - t, 1.5) * flicker;
    } else {
        float d = pow(0.5 + 0.5 * cos(TAU * t * DIAMONDS), 8.0) * (1.0 - t);
        rgb = mix(CORE_HOT * 1.6, CORE_TIP, smoothstep(0.1, 0.9, t));
        a = 0.7 * facing * pow(1.0 - t, 1.2) * flicker * (1.0 - 0.5 * p.z + 1.5 * p.z * d);
    }
    a *= p.x;
    if (a <= 0.003) { discard; }
    o_colour = vec4(rgb, a);
}
#endif

#ifdef BEAM
const vec3 COLOUR = vec3(1.0, 0.95, 0.85);
const int STEPS = 16;

vec2 beam_coords(vec3 q, vec3 apex, vec3 axis, vec4 p) {
    float along = dot(q - apex, axis);
    return vec2(along, length(q - apex - axis * along) / (p.z + p.y * max(along, 0.0)));
}

float density(vec3 q, vec3 apex, vec3 axis, vec4 p) {
    vec2 c = beam_coords(q, apex, axis, p);
    if (c.x < 0.0 || c.x > p.w || c.y > 1.0) { return 0.0; }
    return pow(1.0 - c.x / p.w, 1.5) * (1.0 - c.y * c.y);
}

void main() {
    vec4 p = raw_vec4(0);
    // The lamp and the beam's direction.
    vec3 apex = u_model[3].xyz;
    vec3 axis = -normalize(u_model[2].xyz);
    vec3 eye = u_eye[VIEW];
    vec3 d = normalize(v_world - eye);
    // Only where the view ray leaves the beam (its back faces).
    if (gl_FrontFacing) { discard; }
    float exit_s = length(v_world - eye);
    float chord = length(vec2(p.w, 2.0 * (p.z + p.y * p.w)));
    float start = max(exit_s - chord, 0.0);
    float ds = (exit_s - start) / float(STEPS);
    float sum = 0.0;
    for (int i = 0; i < STEPS; i++) {
        sum += density(eye + d * (start + (float(i) + 0.5) * ds), apex, axis, p);
    }
    float s = clamp(dot(apex - eye, d), start, exit_s);
    float r = length(eye + d * s - apex);
    float a = p.x * (0.03 * sum * ds + 0.8 * exp(-r * r / 0.5));
    if (a <= 0.003) { discard; }
    o_colour = vec4(COLOUR, min(a, 1.0));
}
#endif

#ifdef ROTOR
const float TAU = 6.28318530718;
const float AIRCRAFT_NIGHT = 0.95;
const float SWEEP = 0.9;
const float GHOST = 0.2;
const float TRAIL_FRAC = 0.22;
const float ROOT = 0.22;
const float ROOT_WIDTH = 0.1;
const float TIP = 0.06;
const float TIP_FROM = 0.945;
const float TIP_LIGHT = 2.2;
const float BANDS = 0.12;
const float BAND_FREQ = 46.0;
const float GLINT = 0.7;
const float GLINT_WIDTH = 0.035;
const vec2 GLINT_FACE = vec2(0.8, 0.97);
const float UNDERSIDE = 0.6;
const float MAX_ALPHA = 0.9;

float band(float r, float lo, float hi, float soft) {
    return smoothstep(lo - soft, lo + soft, r) * (1.0 - smoothstep(hi - soft, hi + soft, r));
}

void main() {
    // RotorParams: colour, blades, hub, chord | blur, axis.
    uint colour = u_raw[0].x;
    float n_blades = float(u_raw[0].y);
    float hub = uintBitsToFloat(u_raw[0].z);
    float chord = uintBitsToFloat(u_raw[0].w);
    float blur = uintBitsToFloat(u_raw[1].x);
    float r = length(v_uv);
    float aa = max(fwidth(r), 1e-4);
    if (blur <= 0.0) { discard; }
    float theta = atan(v_uv.y, v_uv.x);
    float cover = clamp(n_blades * chord / (TAU * max(r, 0.05)), 0.0, 1.0);
    float a = cover * SWEEP;
    float s = fract(theta / TAU * n_blades);
    float gap = TAU * r / n_blades;
    float ahead = s * gap;
    float lead = 1.0 - smoothstep(chord * 0.5, chord * 0.5 + max(chord * 0.6, aa * 2.0), ahead);
    float trail = exp(-(1.0 - s) / TRAIL_FRAC);
    a += max(lead, trail) * GHOST * mix(1.0, 0.55, r);
    float root = band(r, hub * 0.7, hub + ROOT_WIDTH, 0.02);
    a += root * ROOT;
    float tip = band(r, TIP_FROM, 0.995, max(aa, 0.006));
    a += tip * TIP;
    float band_fade = 1.0 - smoothstep(0.2, 0.6, aa * BAND_FREQ);
    a *= 1.0 + BANDS * band_fade * sin(r * BAND_FREQ);
    vec3 eye = normalize(u_eye[VIEW] - v_world);
    float facing = dot(eye, v_normal);
    float path = 1.0 / max(abs(facing), 0.12);
    a = 1.0 - pow(1.0 - clamp(a, 0.0, 0.99), path);
    a *= (1.0 - smoothstep(1.0 - max(aa * 1.5, 0.008), 1.0 + aa, r)) * smoothstep(hub * 0.5, hub * 0.75, r);
    a = min(a, MAX_ALPHA) * blur;
    // FLIR: the blades are cool.
    if (flir_view()) {
        o_colour = vec4(vec3(0.1), a);
        return;
    }
    vec3 rgb = g.palette[colour & 0xFFu].rgb;
    rgb = mix(rgb, rgb * TIP_LIGHT, tip);
    rgb = rgb * mix(1.0, 0.75, root);
    if (facing < 0.0) {
        rgb = rgb * UNDERSIDE;
    }
    float dark = g.sky_night.x * AIRCRAFT_NIGHT;
    rgb = rgb * (1.0 - dark);
    vec3 sun = g.sun.xyz;
    float sun_side = dot(sun, v_normal);
    if (g.sun.w > 0.0 && sun_side * facing > 0.0) {
        vec3 h = normalize(sun + eye);
        float hn = abs(dot(h, v_normal));
        vec2 hp = vec2(dot(h, v_axis_u), dot(h, v_axis_v));
        vec2 radial = v_uv / max(r, 1e-4);
        float across = dot(normalize(hp + vec2(1e-5, 0.0)), radial);
        float streak = exp(-across * across / (2.0 * GLINT_WIDTH * GLINT_WIDTH));
        float glint = streak * smoothstep(GLINT_FACE.x, GLINT_FACE.y, hn) * GLINT
            * clamp(g.light_info.y, 0.0, 1.0) * (1.0 - dark) * smoothstep(hub, hub + 0.1, r)
            * (1.0 - smoothstep(0.97, 1.0, r)) * blur;
        rgb = rgb + vec3(1.0, 0.96, 0.88) * glint / max(a, 0.05);
        a = min(a + glint * 0.35, MAX_ALPHA);
    }
    if (g.params.w == 0.0) {
        if (a < bayer4(uvec2(gl_FragCoord.xy))) { discard; }
        o_colour = vec4(nvg(min(rgb, vec3(1.0))), 1.0);
        return;
    }
    if (a <= 0.002) { discard; }
    o_colour = vec4(nvg(rgb), a);
}
#endif
#endif

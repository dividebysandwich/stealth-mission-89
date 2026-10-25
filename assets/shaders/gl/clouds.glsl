// The volumetric clouds for the OpenGL headset renderer (src/glvr), as bevy_fast_clouds'
// wavelet_cloud.wgsl: each cloud's box rasterised, a fixed-step march through it, into a
// reduced-resolution image of both eyes (premultiplied colour and coverage, and the view
// distance where the clouds begin, the nearest kept), composited over the eyes' image later
// (COMPOSITE: that distance as the depth, so nearer geometry hides them). The march isn't
// clipped by the scene's depth (it isn't drawn yet). Compiled with VERTEX or FRAGMENT;
// J1..J3 for the finer scales, MARCH_STEPS, DITHER, SUN_SHADOW as in the WGSL.

#ifdef COMPOSITE
uniform sampler2DArray u_clouds;
uniform sampler2DArray u_distance;
// Near plane (reversed depth: depth = near / view distance).
uniform float u_near;

#ifdef VERTEX
#ifndef SINGLE
layout(num_views = 2) in;
#endif
out vec2 v_uv;

void main() {
    // One triangle over the view.
    vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
    v_uv = p;
    gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT
in vec2 v_uv;
layout(location = 0) out vec4 o_colour;

void main() {
    vec4 c = texture(u_clouds, vec3(v_uv, float(VIEW)));
    if (c.a <= 0.002) { discard; }
    // The nearest distance of the low resolution texels around, so clouds never spread over
    // nearer geometry's edges.
    float d = textureLod(u_distance, vec3(v_uv, float(VIEW)), 0.0).r;
    gl_FragDepth = d >= 65000.0 ? 0.0 : clamp(u_near / max(d, u_near), 0.0, 1.0);
    o_colour = c;
}
#endif
#else
struct GpuVolume {
    vec3 aabb_min;
    float j_max;
    vec3 aabb_max;
    float h_max;
    // Sphere offset, sphere count, seed, camera inside.
    uvec4 misc;
    // Fill, core, darken.
    vec4 fill;
};

layout(std140) uniform CloudGlobals {
    // Direction to the sun, haze distance.
    vec4 sun;
    // Max march span, sigma_t, phase g, LOD false colour.
    vec4 params0;
    // j cap, keep alpha, interior span, frame counter.
    vec4 params1;
    vec4 params2;
    vec4 flash;
    vec4 line_a;
    vec4 line_b;
    vec4 line_color;
    vec4 fog;
    vec4 ramp[16];
} globals;

layout(std430, binding = 0) readonly buffer Volumes {
    GpuVolume volumes[];
};
layout(std430, binding = 1) readonly buffer Spheres {
    vec4 spheres[];
};

uniform mat4 u_view_proj[2];
uniform vec3 u_eye[2];
uniform vec3 u_forward[2];
// The batch's first volume.
uniform uint u_first;

#ifdef VERTEX
#ifndef SINGLE
layout(num_views = 2) in;
#endif
out vec3 v_world;
flat out uint v_volume;

const vec3 CUBE_UNIT[36] = vec3[36](
    vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 1.0), vec3(1.0, 1.0, 1.0),
    vec3(0.0, 0.0, 1.0), vec3(1.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0),
    vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0),
    vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(1.0, 1.0, 0.0),
    vec3(1.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0), vec3(1.0, 1.0, 0.0),
    vec3(1.0, 0.0, 1.0), vec3(1.0, 1.0, 0.0), vec3(1.0, 1.0, 1.0),
    vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 1.0),
    vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 0.0),
    vec3(0.0, 1.0, 1.0), vec3(1.0, 1.0, 1.0), vec3(1.0, 1.0, 0.0),
    vec3(0.0, 1.0, 1.0), vec3(1.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), vec3(1.0, 0.0, 1.0),
    vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 1.0), vec3(0.0, 0.0, 1.0)
);

void main() {
    uint inst = u_first + uint(gl_InstanceID);
    GpuVolume vol = volumes[inst];
    vec3 world = mix(vol.aabb_min, vol.aabb_max, CUBE_UNIT[gl_VertexID]);
    gl_Position = u_view_proj[VIEW] * vec4(world, 1.0);
    v_world = world;
    v_volume = inst;
}
#endif

#ifdef FRAGMENT
in vec3 v_world;
flat in uint v_volume;
layout(location = 0) out vec4 o_colour;
// View distance where the cloud begins (blended to the nearest).
layout(location = 1) out vec4 o_distance;

const float PI = 3.141592653589793;
const float HALF_PI = 1.5707963267948966;
const float INV_SQRT3 = 0.57735026919;
const float INV_SQRT2 = 0.70710678118;
const float NO_CLOUD = 65504.0;

const vec3 N0 = vec3(1.0, 0.0, 0.0);
const vec3 E1_0 = vec3(0.0, 1.0, 0.0);
const vec3 N1 = vec3(0.0, 1.0, 0.0);
const vec3 E1_1 = vec3(0.0, 0.0, 1.0);
const vec3 N2 = vec3(0.0, 0.0, 1.0);
const vec3 E1_2 = vec3(1.0, 0.0, 0.0);
const vec3 N3 = vec3(INV_SQRT3, INV_SQRT3, INV_SQRT3);
const vec3 E1_3 = vec3(INV_SQRT2, -INV_SQRT2, 0.0);
const vec3 N4 = vec3(INV_SQRT3, -INV_SQRT3, INV_SQRT3);
const vec3 E1_4 = vec3(INV_SQRT2, INV_SQRT2, 0.0);
const vec3 N5 = vec3(INV_SQRT3, INV_SQRT3, -INV_SQRT3);
const vec3 E1_5 = vec3(INV_SQRT2, -INV_SQRT2, 0.0);

float sat(float x) {
    return clamp(x, 0.0, 1.0);
}

float malvar_window(float tau) {
    float t = clamp(tau, -1.0, 1.0);
    return sin(HALF_PI * (t + 1.0));
}

float dst_iv_dot(float tau, vec4 c) {
    float a = PI * tau;
    return sin(0.5 * a) * c.x + sin(1.5 * a) * c.y + sin(2.5 * a) * c.z + sin(3.5 * a) * c.w;
}

float hash_u32(uint x_in) {
    uint x = x_in * 747796405u + 2891336453u;
    uint word = ((x >> ((x >> 28u) + 4u)) ^ x) * 277803737u;
    word = (word >> 22u) ^ word;
    return float(word >> 8u) / 16777215.0;
}

float cell_hash(uint j, int k, uint orient, uint seed) {
    return hash_u32(seed + j * 1973u + uint(k) * 9277u + orient * 3697u);
}

vec4 coeffs(uint j, int k, uint orient, uint seed, float amp, vec4 mask) {
    float h = cell_hash(j, k, orient, seed);
    return vec4(1.0, 0.70, 0.45, 0.28) * (0.40 + 0.60 * h) * amp * mask;
}

float ridge(vec3 uv, vec3 n, vec3 e1, uint j, uint orient, uint seed, float amp, vec4 mask, vec3 rd) {
    float orient_keep = 1.0 - smoothstep(0.55, 0.92, abs(dot(n, rd)));
    float scale = exp2(float(j));
    float t_n = scale * dot(uv, n);
    float k0 = floor(t_n);
    float tau0 = t_n - k0;
    float tau1 = tau0 - 1.0;
    vec4 c0 = coeffs(j, int(k0), orient, seed, amp, mask);
    vec4 c1 = coeffs(j, int(k0) + 1, orient, seed, amp, mask);
    float packet = malvar_window(tau0) * dst_iv_dot(tau0, c0) + malvar_window(tau1) * dst_iv_dot(tau1, c1);
    float frac_ = scale * dot(uv, e1) - floor(scale * dot(uv, e1));
    float trans = 0.5 * (1.0 + cos(PI * clamp(2.0 * frac_ - 1.0, -1.0, 1.0)));
    return packet * trans * orient_keep;
}

float sphere_occ(vec3 p, uint offset, uint local_, uint count) {
    float live = local_ < count ? 1.0 : 0.0;
    vec4 s = spheres[min(offset + local_, uint(spheres.length()) - 1u)];
    vec3 d = p - s.xyz;
    float x = 1.0 - sqrt(dot(d, d)) / max(s.w, 0.001);
    float t = sat(x);
    return live * t * t * (3.0 - 2.0 * t);
}

float occupancy_volume(GpuVolume vol, vec3 p) {
    uint offset = vol.misc.x;
    uint count = vol.misc.y;
    float d = 0.0;
    for (uint i = 0u; i < 12u; i++) {
        d = max(d, sphere_occ(p, offset, i, count));
    }
    return d;
}

float ridge_band(vec3 uv, uint j, uint seed, float occ, vec4 mask, vec3 rd) {
    float amp = occ / (1.0 + float(j));
    float keep = mix(1.0, sat((globals.params1.y - cell_hash(j, 0, 0u, seed) + 0.001) * 80.0), sat(float(j) - 1.0));
    return keep * (
        ridge(uv, N0, E1_0, j, 0u, seed, amp, mask, rd)
        + ridge(uv, N1, E1_1, j, 1u, seed, amp, mask, rd)
        + ridge(uv, N2, E1_2, j, 2u, seed, amp, mask, rd)
        + ridge(uv, N3, E1_3, j, 3u, seed, amp, mask, rd)
        + ridge(uv, N4, E1_4, j, 4u, seed, amp, mask, rd)
        + ridge(uv, N5, E1_5, j, 5u, seed, amp, mask, rd)
    );
}

vec2 density_volume(GpuVolume vol, vec3 p, vec3 rd) {
    float fill = max(vol.fill.x, 0.0);
    float occ = sat(occupancy_volume(vol, p) * fill);
    if (occ < 0.02) {
        return vec2(0.0, occ);
    }
    vec3 extent = max(vol.aabb_max - vol.aabb_min, vec3(0.001));
    vec3 uv_raw = (p - vol.aabb_min) / extent;
    vec3 uv = clamp(uv_raw, vec3(0.0), vec3(1.0));
    vec3 outside = min(uv_raw, vec3(1.0) - uv_raw);
    float inside = sat(min(outside.x, min(outside.y, outside.z)) * 64.0);
    vec4 mask = vec4(sat(vol.h_max), sat(vol.h_max - 1.0), sat(vol.h_max - 2.0), sat(vol.h_max - 3.0));
    uint seed = vol.misc.z;
    float rho = ridge_band(uv, 0u, seed, occ, mask, rd);
#ifdef J1
    rho += ridge_band(uv, 1u, seed, occ, mask, rd);
#endif
#ifdef J2
    rho += ridge_band(uv, 2u, seed, occ, mask, rd);
#endif
#ifdef J3
    rho += ridge_band(uv, 3u, seed, occ, mask, rd);
#endif
    float core = vol.fill.y;
    return vec2(max(mix(rho * (0.35 + 0.75 * occ), occ * 1.4 - 0.45 + rho * 0.9, core), 0.0) * inside * fill, occ);
}

vec2 ray_aabb(vec3 ro, vec3 rd, vec3 bmin, vec3 bmax) {
    vec3 inv = 1.0 / rd;
    vec3 t0 = (bmin - ro) * inv;
    vec3 t1 = (bmax - ro) * inv;
    vec3 tmin3 = min(t0, t1);
    vec3 tmax3 = max(t0, t1);
    return vec2(max(max(tmin3.x, tmin3.y), tmin3.z), min(min(tmax3.x, tmax3.y), tmax3.z));
}

vec2 sphere_span(GpuVolume vol, vec3 ro, vec3 rd) {
    vec2 span = vec2(1e20, -1e20);
    uint last = uint(spheres.length()) - 1u;
    for (uint i = 0u; i < 12u; i++) {
        if (i >= vol.misc.y) { break; }
        vec4 s = spheres[min(vol.misc.x + i, last)];
        vec3 oc = ro - s.xyz;
        float b = dot(oc, rd);
        float h = b * b - (dot(oc, oc) - s.w * s.w);
        if (h > 0.0) {
            float q = sqrt(h);
            span = vec2(min(span.x, -b - q), max(span.y, -b + q));
        }
    }
    return span;
}

vec3 lod_tint(float j_max, float j_cap) {
    float t = j_max / max(j_cap, 1.0);
    return mix(vec3(0.25, 0.45, 1.0), vec3(1.0, 0.85, 0.35), t);
}

float hg_phase(float cos_theta, float g) {
    float g2 = g * g;
    return (1.0 - g2) / max(pow(1.0 + g2 - 2.0 * g * cos_theta, 1.5), 0.001);
}

float line_glow(GpuVolume vol, vec3 p, float shadow_reach) {
    float i = globals.line_a.w;
    if (i <= 0.0) { return 0.0; }
    vec3 a = globals.line_a.xyz;
    vec3 ab = globals.line_b.xyz - a;
    float t = sat(dot(p - a, ab) / max(dot(ab, ab), 1e-3));
    vec3 to_channel = a + ab * t - p;
    float d = length(to_channel);
    float reach = globals.line_b.w;
    float core = d / (reach * 0.06);
    float halo = d / (reach * 0.25);
    float fade = 1.0 - smoothstep(0.5 * reach, 1.5 * reach, d);
    if (fade <= 0.0) { return 0.0; }
    vec3 towards = p + to_channel / max(d, 1.0) * min(0.5 * d, shadow_reach);
    float lit = 1.0 - 0.9 * occupancy_volume(vol, towards);
    return i * fade * (1.6 / (1.0 + core * core) + 2.0 * lit / (1.0 + halo * halo));
}

vec3 ramp(float k) {
    float f = clamp(k, 0.0, 15.0);
    uint i = uint(floor(f));
    return mix(globals.ramp[i].rgb, globals.ramp[min(i + 1u, 15u)].rgb, fract(f));
}

float bayer4(uvec2 p) {
    const float m[16] = float[16](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
    return (m[(p.y & 3u) * 4u + (p.x & 3u)] + 0.5) / 16.0;
}

float ign(vec2 p) {
    vec2 q = p + 5.588238 * float(uint(globals.params1.w) % 64u);
    return fract(52.9829189 * fract(dot(q, vec2(0.06711056, 0.00583715))));
}

void main() {
    GpuVolume vol = volumes[v_volume];
    vec3 cam = u_eye[VIEW];
    bool interior = vol.misc.w != 0u;
    vec3 to_hull = v_world - cam;
    vec3 rd = to_hull / max(length(to_hull), 1e-5);
    vec2 hull = ray_aabb(cam, rd, vol.aabb_min, vol.aabb_max);
    float t0 = max(hull.x, length(to_hull));
    float span_cap = globals.params0.x;
    if (interior) {
        t0 = 0.0;
        span_cap = globals.params1.z;
    }
    vec2 spheres_hit = sphere_span(vol, cam, rd);
    t0 = max(t0, spheres_hit.x);
    float t1 = min(min(hull.y, spheres_hit.y), t0 + span_cap);
    if (t1 <= t0) { discard; }
    float span = t1 - t0;
    float step_ = span / float(MARCH_STEPS);
    uvec2 p_px = uvec2(gl_FragCoord.xy);
#ifdef DITHER
    float jitter = bayer4(p_px + uvec2(1u, 2u));
#else
    float jitter = ign(gl_FragCoord.xy);
#endif
    float sigma_t = globals.params0.y;
    vec3 sun = globals.sun.xyz;
    float haze = globals.sun.w;
    float forward_lift = 0.8 * (hg_phase(dot(rd, sun), globals.params0.z) - 1.0);
    vec3 extent = max(vol.aabb_max - vol.aabb_min, vec3(0.001));
    float shadow_reach = 0.3 * min(extent.x, min(extent.y, extent.z));
    float transmittance = 1.0;
    float t_front = NO_CLOUD;
    float shade_sum = 0.0;
    float glow_sum = 0.0;
    for (int s = 0; s < MARCH_STEPS; s++) {
        if (transmittance < 0.03) {
            break;
        }
        float t = t0 + (float(s) + jitter) * step_;
        vec3 p = cam + rd * t;
        vec2 d = density_volume(vol, p, rd);
        float dens = d.x;
        if (dens <= 0.0) {
            continue;
        }
        float a = 1.0 - exp(-sigma_t * dens * step_);
        float w = transmittance * a;
        if (t_front == NO_CLOUD) {
            t_front = t;
        }
        float height = sat((p.y - vol.aabb_min.y) / extent.y);
#ifdef SUN_SHADOW
        float shadow = occupancy_volume(vol, p + sun * shadow_reach);
#else
        float shadow = 0.7 * d.y;
#endif
        float k = 1.0 + 6.0 * shadow + 4.0 * (1.0 - height) - forward_lift * (1.0 - shadow);
        if (haze > 0.0) {
            k -= t / haze * 15.0;
        }
        shade_sum += w * max(k, 0.0);
        glow_sum += w * line_glow(vol, p, shadow_reach);
        transmittance *= 1.0 - a;
    }
    float alpha = 1.0 - transmittance;
    if (alpha <= 0.0) { discard; }
    float k = shade_sum / alpha;
    float glow = glow_sum / alpha;
#ifdef DITHER
    if (alpha < bayer4(p_px)) { discard; }
    vec3 rgb = globals.ramp[uint(clamp(floor(k + bayer4(p_px + uvec2(2u, 3u))), 0.0, 15.0))].rgb;
#else
    vec3 rgb = ramp(k);
#endif
    rgb *= 1.0 - vol.fill.z;
    rgb += globals.flash.rgb + globals.line_color.rgb * glow;
    rgb = mix(rgb, globals.fog.rgb, globals.fog.a);
    if (globals.params0.w > 0.0) {
        rgb = lod_tint(vol.j_max, globals.params1.x);
    }
#ifdef DITHER
    o_colour = vec4(rgb, 1.0);
#else
    o_colour = vec4(rgb * alpha, alpha);
#endif
    o_distance = vec4(min(t_front * max(dot(rd, u_forward[VIEW]), 1e-4), NO_CLOUD));
}
#endif
#endif

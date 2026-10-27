// Lit and unlit textured surfaces for the OpenGL headset renderer (src/glvr): the cockpit
// (cockpit.wgsl's materials), the kneeboard and the other StandardMaterial meshes. Lit by the
// world's sun, moon and ambient light (palette.wgsl's `sky_light`, relative to flat ground),
// not Bevy's PBR. Compiled with VERTEX or FRAGMENT defined; both eyes in one pass.

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
    vec4 storm_haze;
    vec4 burst;
    vec4 burst_colour;
} g;

uniform mat4 u_view_proj[2];
uniform vec3 u_eye[2];
uniform mat4 u_model;
uniform mat3 u_normal;
uniform vec4 u_base;
uniform vec4 u_emissive;
// bit 0: base colour texture, 1: emissive texture, 2: unlit, 3: never darkened, 4: alpha mask
uniform uint u_flags;
uniform float u_cutoff;
// x: darkness of a night without moonlight (cockpit3d/shading.rs `CockpitShading::night`)
uniform vec4 u_night;
// The night flood light (rgb) and the master caution (w) (`CockpitShading::flood`).
uniform vec4 u_flood;
uniform sampler2D u_base_tex;
uniform sampler2D u_emissive_tex;

#ifdef VERTEX
#ifndef SINGLE
layout(num_views = 2) in;
#endif
layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_normal;
layout(location = 2) in vec2 a_uv;
out vec3 v_world;
out vec3 v_normal;
out vec2 v_uv;

void main() {
    vec4 world = u_model * vec4(a_position, 1.0);
    v_world = world.xyz;
    v_normal = u_normal * a_normal;
    v_uv = a_uv;
    gl_Position = u_view_proj[VIEW] * world;
}
#endif

#ifdef FRAGMENT
in vec3 v_world;
in vec3 v_normal;
in vec2 v_uv;
layout(location = 0) out vec4 o_colour;

const vec3 MOON_TINT = vec3(0.75, 0.85, 1.0);
const float SKY_LIGHT_MAX = 2.0;

vec3 sun_moon(vec3 n) {
    return g.ambient.rgb + vec3(g.sun.w * max(dot(n, g.sun.xyz), 0.0))
        + MOON_TINT * g.moon.w * max(dot(n, g.moon.xyz), 0.0);
}

vec3 sky_light(vec3 n) {
    return min(sun_moon(n) / max(sun_moon(vec3(0.0, 1.0, 0.0)), vec3(1e-4)), vec3(SKY_LIGHT_MAX));
}

// A combat flash's light (palette.glsl `burst_glow`), brighter by night.
vec3 burst_glow(vec3 world) {
    float i = g.burst.w;
    if (i <= 0.0) { return vec3(0.0); }
    float d = length(world - g.burst.xyz);
    float reach = g.burst_colour.w;
    float core = d / (reach * 0.06);
    float halo = d / (reach * 0.3);
    return g.burst_colour.rgb * i * mix(1.0, 0.3, g.light_info.y) * (1.0 - smoothstep(0.3 * reach, reach, d)) * (1.6 / (1.0 + core * core) + 0.3 / (1.0 + halo * halo));
}

void main() {
    vec4 base = u_base;
    if ((u_flags & 1u) != 0u) {
        base *= texture(u_base_tex, v_uv);
    }
    if ((u_flags & 16u) != 0u && base.a < u_cutoff) {
        discard;
    }
    vec3 e = u_emissive.rgb;
    if ((u_flags & 2u) != 0u) {
        e *= texture(u_emissive_tex, v_uv).rgb;
    }
    float dark = (u_flags & 8u) != 0u ? 0.0 : u_night.x;
    vec3 rgb;
    if ((u_flags & 4u) != 0u) {
        // Unlit: dark at night but where it glows (cockpit.wgsl).
        float glow = clamp(max(max(e.r, e.g), e.b) * 8.0, 0.0, 1.0);
        rgb = base.rgb * mix(1.0 - dark, 1.0, glow);
    } else {
        vec3 n = v_normal;
        if (dot(n, n) < 1e-8) {
            n = cross(dFdx(v_world), dFdy(v_world));
        }
        n = normalize(n);
        if (!gl_FrontFacing) {
            n = -n;
        }
        rgb = base.rgb * sky_light(n) * (1.0 - dark) + e;
    }
    if ((u_flags & 8u) == 0u) {
        rgb += base.rgb * burst_glow(v_world);
        // The flood light and the master caution's amber pulse (cockpit.wgsl `flood_light`).
        float pulse = u_flood.w * max(sin(g.dots.y * 6.2831853 * 1.5), 0.0);
        rgb += base.rgb * (u_flood.rgb + vec3(0.1, 0.05, 0.0) * pulse);
    }
    o_colour = vec4(rgb, base.a);
}
#endif

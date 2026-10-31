// The HUD combiner glass and the helmet display for the OpenGL headset renderer (src/glvr), as
// hud_glass.wgsl: the HUD canvas collimated to infinity, looked up in the direction each
// fragment is seen from the eye, in the glass mesh's frame. Both eyes in one pass.

uniform mat4 u_view_proj[2];
uniform vec3 u_eye[2];
uniform mat4 u_model;
// The world in the mesh's frame (the inverse of u_model).
uniform mat4 u_local;
// Glass tint where the HUD is blank (rgb, alpha).
uniform vec4 u_tint;
// Canvas pixel of the boresight (xy), canvas pixels per unit of the tangent plane (zw).
uniform vec4 u_proj;
// x: 1 = modern HUD (monochrome ink drawn in HUD green), y: brightness.
uniform vec4 u_style;
uniform sampler2D u_base_tex;

#ifdef VERTEX
#ifndef SINGLE
layout(num_views = 2) in;
#endif
layout(location = 0) in vec3 a_position;
out vec3 v_dir;

void main() {
    vec4 world = u_model * vec4(a_position, 1.0);
    vec3 eye = (u_local * vec4(u_eye[VIEW], 1.0)).xyz;
    v_dir = a_position - eye;
    gl_Position = u_view_proj[VIEW] * world;
}
#endif

#ifdef FRAGMENT
in vec3 v_dir;
layout(location = 0) out vec4 o_colour;

const vec3 HUD_GREEN = vec3(0.18, 1.0, 0.32);

// Rotated grid taps across a pixel (`canvas`).
const vec2 GRID[4] = vec2[4](vec2(0.125, 0.375), vec2(0.375, -0.125), vec2(-0.125, -0.375), vec2(-0.375, 0.125));

// The canvas over this pixel: four taps on a rotated grid across the pixel's footprint (about
// a texel in the headset), averaged. The canvas is inked or not, so one tap a pixel shows its
// texels' stair steps on every slanted line, and MSAA doesn't help (one shading sample a
// pixel). Colour is weighted by alpha.
vec4 canvas(vec2 p, vec2 size) {
    vec2 dx = dFdx(p);
    vec2 dy = dFdy(p);
    vec4 sum = vec4(0.0);
    for (int i = 0; i < 4; i++) {
        vec4 c = textureLod(u_base_tex, (p + GRID[i].x * dx + GRID[i].y * dy) / size, 0.0);
        sum += vec4(c.rgb * c.a, c.a);
    }
    return sum * 0.25;
}

void main() {
    float z = max(-v_dir.z, 1e-4);
    vec2 p = vec2(u_proj.x + u_proj.z * v_dir.x / z, u_proj.y - u_proj.w * v_dir.y / z);
    vec2 size = vec2(textureSize(u_base_tex, 0));
    bool inside = p.x >= 0.0 && p.y >= 0.0 && p.x < size.x && p.y < size.y;
    // (The canvas's rows go top down, as Bevy's images.)
    vec4 c = canvas(p, size);
    if (u_style.x > 0.5) {
        // (A one-channel texture: the coverage is in r, its alpha is 1.)
        float a = inside ? clamp(c.r * 1.2, 0.0, 1.0) : 0.0;
        if (a <= 0.01) {
            if (u_tint.a <= 0.0) { discard; }
            o_colour = u_tint;
            return;
        }
        o_colour = vec4(HUD_GREEN * u_style.y * (0.75 + 0.6 * c.r), a);
        return;
    }
    // The ink over the tint over what is behind.
    float a = inside ? c.a : 0.0;
    if (a <= 0.01) {
        if (u_tint.a <= 0.0) { discard; }
        o_colour = u_tint;
        return;
    }
    float out_a = a + (1.0 - a) * u_tint.a;
    o_colour = vec4((c.rgb + (1.0 - a) * u_tint.a * u_tint.rgb) / out_a, out_a);
}
#endif

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

void main() {
    float z = max(-v_dir.z, 1e-4);
    vec2 p = vec2(u_proj.x + u_proj.z * v_dir.x / z, u_proj.y - u_proj.w * v_dir.y / z);
    vec2 size = vec2(textureSize(u_base_tex, 0));
    bool inside = p.x >= 0.0 && p.y >= 0.0 && p.x < size.x && p.y < size.y;
    // (The canvas's rows go top down, as Bevy's images.)
    vec4 c = textureLod(u_base_tex, p / size, 0.0);
    if (u_style.x > 0.5) {
        float a = inside ? clamp(c.a * 1.2, 0.0, 1.0) : 0.0;
        if (a <= 0.01) {
            if (u_tint.a <= 0.0) { discard; }
            o_colour = u_tint;
            return;
        }
        o_colour = vec4(HUD_GREEN * u_style.y * (0.75 + 0.6 * c.a), a);
        return;
    }
    if (inside && c.a > 0.5) {
        o_colour = vec4(c.rgb, 1.0);
        return;
    }
    if (u_tint.a <= 0.0) { discard; }
    o_colour = u_tint;
}
#endif

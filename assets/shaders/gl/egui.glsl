// egui's meshes for the OpenGL headset renderer (src/glvr): the menus drawn into their images.
// egui blends in gamma space: the colours stay sRGB throughout, the framebuffer's sRGB
// encoding off; textures (sRGB images) are sampled back into sRGB.

uniform vec2 u_screen;
uniform sampler2D u_base_tex;

#ifdef VERTEX
// Points (y down), texture coordinates, premultiplied sRGBA.
layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;
layout(location = 2) in vec4 a_colour;
out vec2 v_uv;
out vec4 v_colour;

void main() {
    // (The image's rows go top down, OpenGL's bottom up: y down lands the right way up.)
    gl_Position = vec4(2.0 * a_pos / u_screen - 1.0, 0.0, 1.0);
    v_uv = a_uv;
    v_colour = a_colour;
}
#endif

#ifdef FRAGMENT
in vec2 v_uv;
in vec4 v_colour;
layout(location = 0) out vec4 o_colour;

vec3 srgb(vec3 l) {
    return mix(12.92 * l, 1.055 * pow(l, vec3(1.0 / 2.4)) - 0.055, step(vec3(0.0031308), l));
}

void main() {
    vec4 t = texture(u_base_tex, v_uv);
    o_colour = v_colour * vec4(srgb(t.rgb), t.a);
}
#endif

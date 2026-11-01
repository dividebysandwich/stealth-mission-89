// Pixelize (pixelize.rs): the picture in square blocks of whole screen pixels, each block
// showing the pixel at its centre, like the 320x200 picture of an MS-DOS game blown up with
// nearest neighbour filtering.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct Pixelize {
    // x: the side of a block, in screen pixels.
    params: vec4<f32>,
};

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: Pixelize;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(screen_texture));
    let block = max(settings.params.x, 1.0);
    let centre = (floor(in.position.xy / block) + 0.5) * block;
    let texel = vec2<i32>(clamp(floor(centre), vec2<f32>(0.0), size - 1.0));
    return textureLoad(screen_texture, texel, 0);
}

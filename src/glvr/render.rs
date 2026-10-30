//! Draws the world and the cockpit into the headset's images with OpenGL: both eyes in one
//! pass (GL_OVR_multiview2), multisampled in tile memory
//! (GL_OVR_multiview_multisampled_render_to_texture) so no multisampled image is stored, depth
//! invalidated at the end. That one pass a frame, straight into the runtime's image, is what
//! Valve's density map layers foveate where the eyes look.

use super::extract::{
    Clouds, DrawList, Effect, EffectItem, EguiView, GlassItem, Item, LitItem, ON_TOP, Offscreen,
};
use crate::palette_material::{FLAG_BACKGROUND, FLAG_CLOUD_COVERAGE, FLAG_CLOUD_SOFT};
use bevy::asset::AssetId;
use bevy::camera::primitives::Aabb;
use bevy::ecs::entity::Entity;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use glow::HasContext;
use openxr as xr;
use std::collections::HashMap;
use std::ffi::c_void;

const PALETTE_SHADER: &str = include_str!("../../assets/shaders/gl/palette.glsl");
const LIT_SHADER: &str = include_str!("../../assets/shaders/gl/lit.glsl");
const GLASS_SHADER: &str = include_str!("../../assets/shaders/gl/glass.glsl");
const EFFECTS_SHADER: &str = include_str!("../../assets/shaders/gl/effects.glsl");
const EGUI_SHADER: &str = include_str!("../../assets/shaders/gl/egui.glsl");
const CLOUDS_SHADER: &str = include_str!("../../assets/shaders/gl/clouds.glsl");
/// Near plane in the eye's space, as bevy_mod_xr's XR cameras (the rig's scale makes it more in
/// the world).
const NEAR: f32 = 0.1;
/// GL_LOWER_LEFT / GL_ZERO_TO_ONE (glClipControl, for reversed depth).
const LOWER_LEFT: u32 = 0x8CA1;
const ZERO_TO_ONE: u32 = 0x935F;
/// GL_TEXTURE_MAX_ANISOTROPY.
const MAX_ANISOTROPY: u32 = 0x84FE;
/// The texture unit holding the star catalogue's cells (`u_stars`) throughout: the others
/// use units 0 and 1.
const STAR_UNIT: u32 = 2;

type MultiviewFn = unsafe extern "system" fn(u32, u32, u32, i32, i32, i32);
type MultisampleMultiviewFn = unsafe extern "system" fn(u32, u32, u32, i32, i32, i32, i32);
type ClipControlFn = unsafe extern "system" fn(u32, u32);

struct GpuMesh {
    vao: glow::VertexArray,
    buffers: Vec<glow::Buffer>,
    count: i32,
    indexed: Option<u32>,
    mode: u32,
    /// Carries a colour per vertex (batched dot lights).
    colours: bool,
}

/// A shader program and its uniforms (those it lacks are None).
struct Program {
    program: glow::Program,
    view_proj: Option<glow::UniformLocation>,
    eye: Option<glow::UniformLocation>,
    forward: Option<glow::UniformLocation>,
    model: Option<glow::UniformLocation>,
    word: Option<glow::UniformLocation>,
    flags: Option<glow::UniformLocation>,
    normal: Option<glow::UniformLocation>,
    base: Option<glow::UniformLocation>,
    emissive: Option<glow::UniformLocation>,
    cutoff: Option<glow::UniformLocation>,
    night: Option<glow::UniformLocation>,
    flood: Option<glow::UniformLocation>,
    local: Option<glow::UniformLocation>,
    tint: Option<glow::UniformLocation>,
    proj: Option<glow::UniformLocation>,
    style: Option<glow::UniformLocation>,
    viewport: Option<glow::UniformLocation>,
    proj_y: Option<glow::UniformLocation>,
    dot_colour: Option<glow::UniformLocation>,
    right: Option<glow::UniformLocation>,
    up: Option<glow::UniformLocation>,
    raw: Option<glow::UniformLocation>,
    screen: Option<glow::UniformLocation>,
    first: Option<glow::UniformLocation>,
}

/// The world's programs: palette objects, the background sphere, dot lights and the effects
/// (`Effect::ALL`'s order), for both eyes or for one view.
struct Programs {
    /// Without and with the lighting options (`LIT`).
    object: [Program; 2],
    background: [Program; 2],
    cloud: [Program; 2],
    dot: Program,
    effects: Vec<Program>,
}

impl Programs {
    fn new(gl: &glow::Context, single: bool) -> Result<Self, String> {
        let one = if single { "#define SINGLE\n" } else { "" };
        let effects = ["PARTICLE", "TRAIL", "ROTOR", "BOLT", "FLAME", "BEAM"]
            .iter()
            .map(|d| compile(gl, EFFECTS_SHADER, &format!("{one}#define {d}")))
            .collect::<Result<Vec<_>, _>>()?;
        let both = |kind: &str| -> Result<[Program; 2], String> {
            Ok([
                compile(gl, PALETTE_SHADER, &format!("{one}{kind}"))?,
                compile(gl, PALETTE_SHADER, &format!("{one}{kind}\n#define LIT"))?,
            ])
        };
        Ok(Programs {
            object: both("")?,
            background: both("#define BACKGROUND")?,
            cloud: both("#define CLOUD")?,
            dot: compile(gl, PALETTE_SHADER, &format!("{one}#define DOT"))?,
            effects,
        })
    }

    fn all(&self) -> impl Iterator<Item = &Program> {
        self.object
            .iter()
            .chain(&self.background)
            .chain(&self.cloud)
            .chain([&self.dot])
            .chain(&self.effects)
    }
}

/// A mesh's data for the render thread.
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub uvs: Option<Vec<[f32; 2]>>,
    /// Colour word per vertex of a mesh of several colours (`ATTRIBUTE_PALETTE_WORD`).
    pub words: Option<Vec<u32>>,
    /// Batched dot lights' palette colour per vertex (dots.rs `ATTRIBUTE_DOT_COLOUR`).
    pub colours: Option<Vec<u32>>,
    /// Effects' attributes: shader location, components, values.
    pub extra: Vec<(u32, i32, Vec<f32>)>,
    pub indices: Option<Indices>,
    pub topology: PrimitiveTopology,
}

/// An RGBA8 (or one-channel R8) image for the render thread: its mip levels' pixels one after
/// the other.
pub struct TextureData {
    pub size: (u32, u32),
    pub levels: u32,
    pub data: Vec<u8>,
    pub srgb: bool,
    /// R8: one byte a pixel.
    pub one_channel: bool,
    pub nearest: bool,
    pub anisotropy: u16,
}

/// Meshes and images coming and going, from the main thread.
pub enum AssetMsg {
    Mesh(AssetId<Mesh>, MeshData),
    ForgetMesh(AssetId<Mesh>),
    Texture(AssetId<Image>, TextureData),
    ForgetTexture(AssetId<Image>),
}

/// An eye in the world: world to clip, its position and view direction.
#[derive(Clone, Copy)]
pub struct EyeView {
    pub view_proj: Mat4,
    /// The eye's right and up directions.
    pub right: Vec3,
    pub up: Vec3,
    /// The projection's y scale.
    pub proj_y: f32,
    pub eye: Vec3,
    pub forward: Vec3,
    /// World units per view unit (the rig's scale in VR).
    pub scale: f32,
}

/// The eyes of `views` (local space) in the world, from the rig's tracking root `rig` (the eye
/// camera, at the cockpit's eye point and the recentring offset); their fields of view
/// `widen`ed.
pub fn eye_views(rig: Mat4, views: &[xr::View], widen: f32) -> Vec<EyeView> {
    views
        .iter()
        .take(2)
        .map(|v| {
            let (p, q) = pose(v);
            let world_from_eye = rig * Transform::from_translation(p).with_rotation(q).to_matrix();
            let proj = projection(v.fov, widen);
            EyeView {
                view_proj: proj * world_from_eye.inverse(),
                right: world_from_eye.x_axis.truncate().normalize(),
                up: world_from_eye.y_axis.truncate().normalize(),
                proj_y: proj.y_axis.y,
                eye: world_from_eye.w_axis.truncate(),
                forward: -world_from_eye.z_axis.truncate().normalize(),
                scale: world_from_eye.z_axis.truncate().length(),
            }
        })
        .collect()
}

/// The view of a camera at `world_from_view` with a perspective projection (vertical field of
/// view `fov`, `aspect`, `near`).
pub fn camera_view(world_from_view: Mat4, fov: f32, aspect: f32, near: f32) -> EyeView {
    let t = (fov * 0.5).tan();
    let proj = projection_tan(-t * aspect, t * aspect, -t, t, near);
    EyeView {
        view_proj: proj * world_from_view.inverse(),
        proj_y: proj.y_axis.y,
        right: world_from_view.x_axis.truncate().normalize(),
        up: world_from_view.y_axis.truncate().normalize(),
        eye: world_from_view.w_axis.truncate(),
        forward: -world_from_view.z_axis.truncate().normalize(),
        scale: world_from_view.z_axis.truncate().length(),
    }
}

/// A view's pose as position and rotation.
pub fn pose(v: &xr::View) -> (Vec3, Quat) {
    let p = v.pose.position;
    let o = v.pose.orientation;
    (Vec3::new(p.x, p.y, p.z), Quat::from_xyzw(o.x, o.y, o.z, o.w))
}

/// Per-frame counts for the log.
#[derive(Default, Clone, Copy)]
pub struct Stats {
    pub draws: u32,
    pub culled: u32,
    /// Distinct meshes among the world's and the dots' draws (what instancing would draw).
    pub meshes: u32,
    /// GPU time of a frame drawn a few frames ago (timestamps before and after its pass).
    pub gpu_ms: Option<f32>,
}

/// Byte offset of `Globals.ambient.w` (the lighting flags): after the palette, the sky and
/// ground bands, params and sun (266 vec4s).
const AMBIENT_W: usize = 266 * 16 + 12;

/// Words (4 bytes) of an instance in the instance buffer: the model matrix and the mesh tag.
const INSTANCE_WORDS: usize = 17;

/// Palette meshes of one mesh and material: their instances' place in the instance buffer.
struct Group {
    mesh: AssetId<Mesh>,
    word: u32,
    flags: u32,
    first: usize,
    count: usize,
}

/// `items` grouped by mesh and material, their instances appended to `instances`.
fn groups<'a>(items: impl Iterator<Item = &'a Item>, instances: &mut Vec<u32>) -> Vec<Group> {
    let mut items: Vec<&Item> = items.collect();
    items.sort_unstable_by_key(|i| (i.mesh, i.word, i.flags));
    let mut out: Vec<Group> = Vec::new();
    for i in items {
        let first = instances.len() / INSTANCE_WORDS;
        instances.extend(i.model.to_cols_array().map(f32::to_bits));
        instances.push(i.tag);
        match out.last_mut() {
            Some(g) if (g.mesh, g.word, g.flags) == (i.mesh, i.word, i.flags) => g.count += 1,
            _ => out.push(Group { mesh: i.mesh, word: i.word, flags: i.flags, first, count: 1 }),
        }
    }
    out
}

/// Frames of GPU timestamps in flight: read back this many frames after they were taken.
const TIMERS: usize = 4;

pub struct Renderer {
    pub gl: glow::Context,
    /// The world's programs for both eyes, and for the offscreen views.
    eyes: Programs,
    single: Programs,
    lit: Program,
    glass: Program,
    /// egui's program, its vertex array and buffers, and a white texture for its untextured
    /// meshes.
    egui: (Program, glow::VertexArray, glow::Buffer, glow::Buffer, glow::Texture),
    /// The clouds: programs by (finest scale, march steps, dither, sun shadow), their globals,
    /// volumes and spheres, and an empty vertex array (the boxes are made in the shader).
    cloud_programs: std::cell::RefCell<HashMap<(u32, u32, bool, bool), Program>>,
    cloud_buffers: [glow::Buffer; 3],
    cloud_vao: glow::VertexArray,
    /// The march's target (both eyes' layers: colour, distance; its size) and the composite's
    /// program.
    cloud_target: Option<(glow::Texture, glow::Texture, glow::Framebuffer, (u32, u32))>,
    cloud_composite: Program,
    /// World units per view unit of the eyes the clouds were marched for.
    cloud_scale: f32,
    /// The offscreen views' targets by image: a depth buffer and a framebuffer (the colour
    /// texture is in `textures`).
    targets: HashMap<AssetId<Image>, (glow::Renderbuffer, glow::Framebuffer, (u32, u32))>,
    globals: glow::Buffer,
    /// The palette meshes' instances of this frame: model matrix and mesh tag each.
    instances: glow::Buffer,
    meshes: HashMap<AssetId<Mesh>, GpuMesh>,
    textures: HashMap<AssetId<Image>, (glow::Texture, (u32, u32), u32)>,
    depth: Option<(glow::Texture, (u32, u32))>,
    framebuffers: HashMap<u32, glow::Framebuffer>,
    multiview: MultiviewFn,
    multisample: Option<MultisampleMultiviewFn>,
    samples: i32,
    pub stats: Stats,
    /// Images read back this frame (`DrawList::readbacks`): the entity and the pixels.
    pub readouts: Vec<(Entity, Vec<u8>)>,
    /// Timestamps before and after each frame's pass, a ring of `TIMERS`.
    timers: Vec<(glow::Query, glow::Query)>,
    frame: usize,
}

fn compile(gl: &glow::Context, shader: &str, defines: &str) -> Result<Program, String> {
    // SAFETY: the context is current; the shaders are this module's.
    unsafe {
        let program = gl.create_program()?;
        let mut shaders = Vec::new();
        for (kind, stage) in [(glow::VERTEX_SHADER, "VERTEX"), (glow::FRAGMENT_SHADER, "FRAGMENT")]
        {
            let s = gl.create_shader(kind)?;
            let views = if defines.contains("SINGLE") {
                "#define VIEW 0u\n"
            } else {
                "#extension GL_OVR_multiview2 : require\n#define VIEW gl_ViewID_OVR\n"
            };
            let source = format!("#version 450 core\n{views}#define {stage}\n{defines}\n{shader}");
            gl.shader_source(s, &source);
            gl.compile_shader(s);
            if !gl.get_shader_compile_status(s) {
                return Err(format!("{stage} {defines}: {}", gl.get_shader_info_log(s)));
            }
            gl.attach_shader(program, s);
            shaders.push(s);
        }
        // (The shaders give their attributes' locations.)
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            return Err(format!("linking {defines}: {}", gl.get_program_info_log(program)));
        }
        for s in shaders {
            gl.delete_shader(s);
        }
        if let Some(index) = gl.get_uniform_block_index(program, "Globals") {
            gl.uniform_block_binding(program, index, 0);
        }
        if let Some(index) = gl.get_uniform_block_index(program, "CloudGlobals") {
            gl.uniform_block_binding(program, index, 1);
        }
        let at = |name: &str| gl.get_uniform_location(program, name);
        gl.use_program(Some(program));
        gl.uniform_1_i32(at("u_base_tex").as_ref(), 0);
        gl.uniform_1_i32(at("u_emissive_tex").as_ref(), 1);
        gl.uniform_1_i32(at("u_stars").as_ref(), STAR_UNIT as i32);
        Ok(Program {
            program,
            view_proj: at("u_view_proj"),
            eye: at("u_eye"),
            forward: at("u_forward"),
            model: at("u_model"),
            word: at("u_word"),
            flags: at("u_flags"),
            normal: at("u_normal"),
            base: at("u_base"),
            emissive: at("u_emissive"),
            cutoff: at("u_cutoff"),
            night: at("u_night"),
            flood: at("u_flood"),
            local: at("u_local"),
            tint: at("u_tint"),
            proj: at("u_proj"),
            style: at("u_style"),
            viewport: at("u_viewport"),
            proj_y: at("u_proj_y"),
            dot_colour: at("u_dot_colour"),
            right: at("u_right"),
            up: at("u_up"),
            raw: at("u_raw"),
            screen: at("u_screen"),
            first: at("u_first"),
        })
    }
}

/// The projection of an eye with field of view `fov` (its tangents times `widen`): reversed
/// depth, infinitely far.
fn projection(fov: xr::Fovf, widen: f32) -> Mat4 {
    let (l, r) = (fov.angle_left.tan() * widen, fov.angle_right.tan() * widen);
    let (d, u) = (fov.angle_down.tan() * widen, fov.angle_up.tan() * widen);
    projection_tan(l, r, d, u, NEAR)
}

/// The projection with the edges' tangents l, r, d, u: reversed depth, infinitely far.
fn projection_tan(l: f32, r: f32, d: f32, u: f32, near: f32) -> Mat4 {
    let (w, h) = (r - l, u - d);
    Mat4::from_cols(
        Vec4::new(2.0 / w, 0.0, 0.0, 0.0),
        Vec4::new(0.0, 2.0 / h, 0.0, 0.0),
        Vec4::new((r + l) / w, (u + d) / h, 0.0, -1.0),
        Vec4::new(0.0, 0.0, near, 0.0),
    )
}

/// The six planes of the frustum of `m` (world to clip, reversed depth), each as
/// (normal, distance) with the inside positive; the far plane is left out.
pub fn planes(m: Mat4) -> [Vec4; 5] {
    let r = |i: usize| m.row(i);
    [r(3) + r(0), r(3) - r(0), r(3) + r(1), r(3) - r(1), r(3) - r(2)]
}

/// Whether the box `aabb` in the space of `t` is at least partly inside `planes`.
pub fn inside(planes: &[Vec4; 5], t: &GlobalTransform, aabb: &Aabb) -> bool {
    let m = t.affine();
    let centre = m.transform_point3a(aabb.center);
    let he = aabb.half_extents;
    let (x, y, z) = (m.matrix3.x_axis.abs(), m.matrix3.y_axis.abs(), m.matrix3.z_axis.abs());
    let extent = x * he.x + y * he.y + z * he.z;
    planes.iter().all(|p| {
        let n = bevy::math::Vec3A::from(p.truncate());
        n.dot(centre) + n.abs().dot(extent) + p.w >= 0.0
    })
}

impl Renderer {
    pub fn new(
        gl: glow::Context,
        get: impl Fn(&str) -> *const c_void,
        samples: i32,
    ) -> Result<Self, String> {
        let exts = gl.supported_extensions();
        if !exts.contains("GL_OVR_multiview2") {
            return Err("OpenGL has no GL_OVR_multiview2".into());
        }
        // SAFETY: the context is current.
        let (renderer, version) = unsafe {
            (gl.get_parameter_string(glow::RENDERER), gl.get_parameter_string(glow::VERSION))
        };
        let msrtt = exts.contains("GL_OVR_multiview_multisampled_render_to_texture");
        info!("glvr: {renderer}, OpenGL {version}; multiview MSRTT {msrtt}");
        let load = |name: &str| {
            let f = get(name);
            (!f.is_null()).then_some(f)
        };
        // SAFETY: the extensions' entry points, with their signatures.
        let (multiview, multisample, clip_control) = unsafe {
            (
                std::mem::transmute::<*const c_void, MultiviewFn>(
                    load("glFramebufferTextureMultiviewOVR")
                        .ok_or("no glFramebufferTextureMultiviewOVR")?,
                ),
                load("glFramebufferTextureMultisampleMultiviewOVR")
                    .filter(|_| msrtt)
                    .map(|f| std::mem::transmute::<*const c_void, MultisampleMultiviewFn>(f)),
                std::mem::transmute::<*const c_void, ClipControlFn>(
                    load("glClipControl").ok_or("no glClipControl")?,
                ),
            )
        };
        // SAFETY: as above.
        unsafe { clip_control(LOWER_LEFT, ZERO_TO_ONE) };
        let eyes = Programs::new(&gl, false)?;
        let single = Programs::new(&gl, true)?;
        let lit = compile(&gl, LIT_SHADER, "")?;
        let glass = compile(&gl, GLASS_SHADER, "")?;
        let egui = Self::egui_setup(&gl)?;
        let cloud_composite = compile(&gl, CLOUDS_SHADER, "#define COMPOSITE")?;
        // SAFETY: the context is current.
        let (cloud_buffers, cloud_vao) = unsafe {
            (
                [gl.create_buffer()?, gl.create_buffer()?, gl.create_buffer()?],
                gl.create_vertex_array()?,
            )
        };

        // SAFETY: the context is current.
        let (globals, instances) = unsafe { (gl.create_buffer()?, gl.create_buffer()?) };
        Self::star_grid(&gl)?;
        let samples = if multisample.is_some() { samples } else { 0 };
        // SAFETY: the context is current.
        let timers = unsafe {
            (0..TIMERS)
                .map(|_| Ok((gl.create_query()?, gl.create_query()?)))
                .collect::<Result<Vec<_>, String>>()?
        };
        Ok(Renderer {
            gl,
            eyes,
            single,
            lit,
            glass,
            egui,
            cloud_programs: Default::default(),
            cloud_buffers,
            cloud_vao,
            cloud_target: None,
            cloud_composite,
            cloud_scale: 1.0,
            targets: HashMap::new(),
            globals,
            instances,
            meshes: HashMap::new(),
            textures: HashMap::new(),
            depth: None,
            framebuffers: HashMap::new(),
            multiview,
            multisample,
            samples,
            stats: Stats::default(),
            readouts: Vec::new(),
            timers,
            frame: 0,
        })
    }

    /// Takes a mesh or an image from the main thread, or drops one.
    pub fn apply(&mut self, msg: AssetMsg) {
        match msg {
            AssetMsg::Mesh(id, data) => {
                self.drop_mesh(id);
                if let Some(gpu) = Self::upload(&self.gl, &data) {
                    self.meshes.insert(id, gpu);
                }
            }
            AssetMsg::ForgetMesh(id) => self.drop_mesh(id),
            AssetMsg::Texture(id, data) => self.upload_texture(id, &data),
            AssetMsg::ForgetTexture(id) => {
                if let Some((t, _, _)) = self.textures.remove(&id) {
                    // SAFETY: the context is current; the texture is this renderer's.
                    unsafe { self.gl.delete_texture(t) };
                }
            }
        }
    }

    fn drop_mesh(&mut self, id: AssetId<Mesh>) {
        if let Some(m) = self.meshes.remove(&id) {
            // SAFETY: the context is current; the objects are this renderer's.
            unsafe {
                self.gl.delete_vertex_array(m.vao);
                for b in m.buffers {
                    self.gl.delete_buffer(b);
                }
            }
        }
    }

    fn upload(gl: &glow::Context, mesh: &MeshData) -> Option<GpuMesh> {
        let positions = &mesh.positions;
        let mode = match mesh.topology {
            PrimitiveTopology::TriangleList => glow::TRIANGLES,
            PrimitiveTopology::TriangleStrip => glow::TRIANGLE_STRIP,
            PrimitiveTopology::LineList => glow::LINES,
            PrimitiveTopology::LineStrip => glow::LINE_STRIP,
            PrimitiveTopology::PointList => glow::POINTS,
        };
        // SAFETY: the context is current; the data outlives the calls.
        unsafe {
            let vao = gl.create_vertex_array().ok()?;
            gl.bind_vertex_array(Some(vao));
            let mut buffers = Vec::new();
            let mut attribute = |location: u32, components: i32, bytes: &[u8]| -> Option<()> {
                let vbo = gl.create_buffer().ok()?;
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STATIC_DRAW);
                gl.enable_vertex_attrib_array(location);
                gl.vertex_attrib_pointer_f32(location, components, glow::FLOAT, false, 0, 0);
                buffers.push(vbo);
                Some(())
            };
            let bytes =
                |v: &[f32]| std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), v.len() * 4);
            attribute(0, 3, bytes(positions.as_flattened()))?;
            if let Some(n) = &mesh.normals {
                attribute(1, 3, bytes(n.as_flattened()))?;
            }
            if let Some(uv) = &mesh.uvs {
                attribute(2, 2, bytes(uv.as_flattened()))?;
            }
            for (location, n, values) in &mesh.extra {
                attribute(*location, *n, bytes(values))?;
            }
            if let Some(c) = &mesh.colours {
                let vbo = gl.create_buffer().ok()?;
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                let b = std::slice::from_raw_parts(c.as_ptr().cast::<u8>(), c.len() * 4);
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, b, glow::STATIC_DRAW);
                gl.enable_vertex_attrib_array(3);
                gl.vertex_attrib_pointer_i32(3, 1, glow::UNSIGNED_INT, 0, 0);
                buffers.push(vbo);
            }
            if let Some(w) = &mesh.words {
                let vbo = gl.create_buffer().ok()?;
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                let b = std::slice::from_raw_parts(w.as_ptr().cast::<u8>(), w.len() * 4);
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, b, glow::STATIC_DRAW);
                gl.enable_vertex_attrib_array(9);
                gl.vertex_attrib_pointer_i32(9, 1, glow::UNSIGNED_INT, 0, 0);
                buffers.push(vbo);
            }
            let (count, indexed) = match &mesh.indices {
                Some(indices) => {
                    let ebo = gl.create_buffer().ok()?;
                    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
                    buffers.push(ebo);
                    match indices {
                        Indices::U16(i) => {
                            let b =
                                std::slice::from_raw_parts(i.as_ptr().cast::<u8>(), i.len() * 2);
                            gl.buffer_data_u8_slice(
                                glow::ELEMENT_ARRAY_BUFFER,
                                b,
                                glow::STATIC_DRAW,
                            );
                            (i.len() as i32, Some(glow::UNSIGNED_SHORT))
                        }
                        Indices::U32(i) => {
                            let b =
                                std::slice::from_raw_parts(i.as_ptr().cast::<u8>(), i.len() * 4);
                            gl.buffer_data_u8_slice(
                                glow::ELEMENT_ARRAY_BUFFER,
                                b,
                                glow::STATIC_DRAW,
                            );
                            (i.len() as i32, Some(glow::UNSIGNED_INT))
                        }
                    }
                }
                None => (positions.len() as i32, None),
            };
            gl.bind_vertex_array(None);
            Some(GpuMesh { vao, buffers, count, indexed, mode, colours: mesh.colours.is_some() })
        }
    }

    /// The star catalogue's cells (`stars::grid`) on texture unit `STAR_UNIT`, for good.
    fn star_grid(gl: &glow::Context) -> Result<(), String> {
        use crate::stars::{ROWS, WIDTH};
        // SAFETY: the context is current; the grid is WIDTH x ROWS RGBA8 texels.
        unsafe {
            let t = gl.create_texture()?;
            gl.active_texture(glow::TEXTURE0 + STAR_UNIT);
            gl.bind_texture(glow::TEXTURE_2D, Some(t));
            gl.tex_storage_2d(glow::TEXTURE_2D, 1, glow::RGBA8, WIDTH as i32, ROWS as i32);
            for (param, value) in [
                (glow::TEXTURE_MIN_FILTER, glow::NEAREST),
                (glow::TEXTURE_MAG_FILTER, glow::NEAREST),
            ] {
                gl.tex_parameter_i32(glow::TEXTURE_2D, param, value as i32);
            }
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                0,
                WIDTH as i32,
                ROWS as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&crate::stars::grid().0)),
            );
            gl.active_texture(glow::TEXTURE0);
        }
        Ok(())
    }

    /// Uploads image `id` (all its mip levels): into the texture it has if it is the same
    /// size, else a new one.
    fn upload_texture(&mut self, id: AssetId<Image>, data: &TextureData) {
        let gl = &self.gl;
        let (w, h) = data.size;
        let (internal, format, bytes) = match (data.one_channel, data.srgb) {
            (true, _) => (glow::R8, glow::RED, 1),
            (false, true) => (glow::SRGB8_ALPHA8, glow::RGBA, 4),
            (false, false) => (glow::RGBA8, glow::RGBA, 4),
        };
        // SAFETY: the context is current; the data covers the levels read.
        unsafe {
            let texture = match self.textures.get(&id) {
                Some(&(t, size, levels)) if size == data.size && levels == data.levels => t,
                old => {
                    if let Some(&(t, _, _)) = old {
                        gl.delete_texture(t);
                    }
                    let Ok(t) = gl.create_texture() else { return };
                    gl.bind_texture(glow::TEXTURE_2D, Some(t));
                    gl.tex_storage_2d(
                        glow::TEXTURE_2D,
                        data.levels as i32,
                        internal,
                        w as i32,
                        h as i32,
                    );
                    let (min, mag) = match (data.nearest, data.levels > 1) {
                        (true, true) => (glow::NEAREST_MIPMAP_NEAREST, glow::NEAREST),
                        (true, false) => (glow::NEAREST, glow::NEAREST),
                        (false, true) => (glow::LINEAR_MIPMAP_LINEAR, glow::LINEAR),
                        (false, false) => (glow::LINEAR, glow::LINEAR),
                    };
                    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, min as i32);
                    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, mag as i32);
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_S,
                        glow::CLAMP_TO_EDGE as i32,
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_T,
                        glow::CLAMP_TO_EDGE as i32,
                    );
                    if data.anisotropy > 1 {
                        gl.tex_parameter_f32(
                            glow::TEXTURE_2D,
                            MAX_ANISOTROPY,
                            data.anisotropy as f32,
                        );
                    }
                    self.textures.insert(id, (t, data.size, data.levels));
                    t
                }
            };
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            let (mut lw, mut lh, mut offset, mut uploaded) = (w as usize, h as usize, 0, 0);
            for level in 0..data.levels {
                let len = lw * lh * bytes;
                let Some(pixels) = data.data.get(offset..offset + len) else { break };
                uploaded += 1;
                gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    level as i32,
                    0,
                    0,
                    lw as i32,
                    lh as i32,
                    format,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(pixels)),
                );
                offset += len;
                (lw, lh) = ((lw / 2).max(1), (lh / 2).max(1));
            }
            // Levels the image doesn't carry are made from the first.
            if uploaded > 0 && uploaded < data.levels {
                gl.generate_mipmap(glow::TEXTURE_2D);
            }
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// The framebuffer drawing into image `texture` (both layers), multisampled in tile memory.
    fn framebuffer(
        &mut self,
        texture: glow::Texture,
        size: (u32, u32),
    ) -> Result<glow::Framebuffer, String> {
        let gl = &self.gl;
        if self.depth.is_none_or(|(_, s)| s != size) {
            // SAFETY: the context is current.
            unsafe {
                let depth = gl.create_texture()?;
                gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(depth));
                gl.tex_storage_3d(
                    glow::TEXTURE_2D_ARRAY,
                    1,
                    glow::DEPTH_COMPONENT32F,
                    size.0 as i32,
                    size.1 as i32,
                    2,
                );
                gl.bind_texture(glow::TEXTURE_2D_ARRAY, None);
                self.depth = Some((depth, size));
            }
            self.framebuffers.clear();
        }
        let key = texture.0.get();
        if let Some(&fb) = self.framebuffers.get(&key) {
            return Ok(fb);
        }
        let depth = self.depth.expect("the depth").0;
        // SAFETY: the context is current; both textures have two layers.
        unsafe {
            let fb = gl.create_framebuffer()?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
            for (attachment, tex) in
                [(glow::COLOR_ATTACHMENT0, texture), (glow::DEPTH_ATTACHMENT, depth)]
            {
                match self.multisample {
                    Some(ms) if self.samples > 0 => {
                        ms(glow::FRAMEBUFFER, attachment, tex.0.get(), 0, self.samples, 0, 2)
                    }
                    _ => (self.multiview)(glow::FRAMEBUFFER, attachment, tex.0.get(), 0, 0, 2),
                }
            }
            let complete =
                gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE;
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            if !complete {
                gl.delete_framebuffer(fb);
                return Err("OpenGL can't draw into the headset's images".into());
            }
            self.framebuffers.insert(key, fb);
            Ok(fb)
        }
    }

    /// Draws `list` seen by the eyes `views` into image `texture`, `size` big: first the
    /// offscreen views (the tracking camera) into their images.
    pub fn draw(
        &mut self,
        list: &DrawList,
        views: &[xr::View],
        texture: glow::Texture,
        size: (u32, u32),
        srgb: bool,
    ) -> Result<(), String> {
        let fb = self.framebuffer(texture, size)?;
        let timer = self.timers[self.frame % TIMERS];
        let mut gpu_ms = None;
        // SAFETY: the context is current; the queries and buffers are this renderer's.
        unsafe {
            let gl = &self.gl;
            if self.frame >= TIMERS
                && gl.get_query_parameter_u32(timer.1, glow::QUERY_RESULT_AVAILABLE) != 0
            {
                let start = gl.get_query_parameter_u64(timer.0, glow::QUERY_RESULT);
                let end = gl.get_query_parameter_u64(timer.1, glow::QUERY_RESULT);
                gpu_ms = Some(end.saturating_sub(start) as f32 / 1e6);
            }
            gl.query_counter(timer.0, glow::TIMESTAMP);
            if !list.globals.is_empty() {
                gl.bind_buffer(glow::UNIFORM_BUFFER, Some(self.globals));
                gl.buffer_data_u8_slice(glow::UNIFORM_BUFFER, &list.globals, glow::DYNAMIC_DRAW);
                gl.bind_buffer_base(glow::UNIFORM_BUFFER, 0, Some(self.globals));
            }
            // (The offscreen images are sRGB.)
            gl.enable(glow::FRAMEBUFFER_SRGB);
        }
        let mut stats = Stats { culled: list.culled, ..default() };
        for view in &list.egui {
            self.draw_egui(view)?;
        }
        for off in &list.offscreen {
            self.draw_offscreen(off, &mut stats)?;
        }
        for &(entity, image) in &list.readbacks {
            if let Some(pixels) = self.read_target(image) {
                self.readouts.push((entity, pixels));
            }
        }
        if !srgb {
            // SAFETY: the context is current.
            unsafe { self.gl.disable(glow::FRAMEBUFFER_SRGB) };
        }
        let eyes = eye_views(list.rig, views, 1.0);
        // The clouds' march first, at reduced resolution, into its own target; composited in
        // the eyes' pass (one pass: switching targets within it would split it).
        let clouds = match &list.volumetric {
            Some(c) => self.march_clouds(c, &eyes, size, &mut stats)?,
            None => false,
        };
        let programs = [&self.lit, &self.glass, &self.cloud_composite];
        // SAFETY: the context is current.
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
            self.gl.viewport(0, 0, size.0 as i32, size.1 as i32);
        }
        self.clear(Vec4::new(0.0, 0.0, 0.0, 1.0));
        self.set_views(self.eyes.all().chain(programs), &eyes, size);
        let distinct: std::collections::HashSet<_> =
            list.items.iter().chain(&list.dots).map(|i| (i.mesh, i.word, i.flags)).collect();
        stats.meshes = distinct.len() as u32;
        // Opaque first: the cockpit (it hides much of the world), the world, the background
        // sphere behind them; then what blends, far to near.
        self.use_program(&self.lit);
        for item in list.lit.iter().filter(|i| !i.blend) {
            self.draw_lit(item);
            stats.draws += 1;
        }
        let eye = eyes.first().map_or(Vec3::ZERO, |e| e.eye);
        self.draw_world(&self.eyes, list, eye, clouds, &mut stats);
        let mut blended: Vec<&LitItem> =
            list.lit.iter().filter(|i| i.blend && i.flags & ON_TOP == 0).collect();
        blended.sort_by(|a, b| {
            let d = |i: &LitItem| i.model.w_axis.truncate().distance_squared(eye);
            d(b).total_cmp(&d(a))
        });
        if !blended.is_empty() {
            // SAFETY: the context is current.
            unsafe {
                self.gl.enable(glow::BLEND);
                self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                self.gl.depth_mask(false);
            }
            self.use_program(&self.lit);
            for item in blended {
                self.draw_lit(item);
                stats.draws += 1;
            }
        }
        // The HUD glass and the helmet display last, over everything behind them.
        if !list.glass.is_empty() {
            // SAFETY: the context is current.
            unsafe {
                self.gl.enable(glow::BLEND);
                self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                self.gl.depth_mask(false);
                self.gl.disable(glow::CULL_FACE);
            }
            self.use_program(&self.glass);
            for item in &list.glass {
                self.draw_glass(item);
                stats.draws += 1;
            }
            // SAFETY: the context is current.
            unsafe { self.gl.enable(glow::CULL_FACE) };
        }
        // Over everything (the tooltip's screen): not depth tested.
        let on_top: Vec<&LitItem> = list.lit.iter().filter(|i| i.flags & ON_TOP != 0).collect();
        if !on_top.is_empty() {
            // SAFETY: the context is current.
            unsafe {
                self.gl.enable(glow::BLEND);
                self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                self.gl.depth_mask(false);
                self.gl.disable(glow::DEPTH_TEST);
            }
            self.use_program(&self.lit);
            for item in on_top {
                self.draw_lit(item);
                stats.draws += 1;
            }
            // SAFETY: the context is current.
            unsafe { self.gl.enable(glow::DEPTH_TEST) };
        }
        let gl = &self.gl;
        // SAFETY: the context is current.
        unsafe {
            gl.disable(glow::BLEND);
            gl.depth_mask(true);
            gl.bind_vertex_array(None);
            gl.invalidate_framebuffer(glow::FRAMEBUFFER, &[glow::DEPTH_ATTACHMENT]);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.query_counter(timer.1, glow::TIMESTAMP);
            gl.flush();
        }
        self.frame += 1;
        stats.gpu_ms = gpu_ms;
        self.stats = stats;
        Ok(())
    }

    /// Clears the framebuffer bound to `colour` and the far depth, with the state the world's
    /// opaque meshes are drawn with.
    fn clear(&self, colour: Vec4) {
        let gl = &self.gl;
        // SAFETY: the context is current.
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::GREATER);
            gl.depth_mask(true);
            gl.disable(glow::BLEND);
            gl.enable(glow::CULL_FACE);
            gl.cull_face(glow::BACK);
            gl.front_face(glow::CCW);
            gl.clear_color(colour.x, colour.y, colour.z, colour.w);
            gl.clear_depth_f32(0.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
    }

    /// The views' uniforms of `programs`: `eyes` (one or two), the image `size`.
    fn set_views<'a>(
        &self,
        programs: impl Iterator<Item = &'a Program>,
        eyes: &[EyeView],
        size: (u32, u32),
    ) {
        let flat: Vec<f32> = eyes.iter().flat_map(|e| e.view_proj.to_cols_array()).collect();
        let pos: Vec<f32> = eyes.iter().flat_map(|e| e.eye.to_array()).collect();
        let fwd: Vec<f32> = eyes.iter().flat_map(|e| e.forward.to_array()).collect();
        let proj_y: Vec<f32> = eyes.iter().map(|e| e.proj_y).collect();
        let right: Vec<f32> = eyes.iter().flat_map(|e| e.right.to_array()).collect();
        let up: Vec<f32> = eyes.iter().flat_map(|e| e.up.to_array()).collect();
        let gl = &self.gl;
        for p in programs {
            // SAFETY: the context is current; the program is this renderer's.
            unsafe {
                gl.use_program(Some(p.program));
                gl.uniform_2_f32(p.viewport.as_ref(), size.0 as f32, size.1 as f32);
                gl.uniform_1_f32_slice(p.proj_y.as_ref(), &proj_y);
                gl.uniform_matrix_4_f32_slice(p.view_proj.as_ref(), false, &flat);
                gl.uniform_3_f32_slice(p.eye.as_ref(), &pos);
                gl.uniform_3_f32_slice(p.forward.as_ref(), &fwd);
                gl.uniform_3_f32_slice(p.right.as_ref(), &right);
                gl.uniform_3_f32_slice(p.up.as_ref(), &up);
            }
        }
    }

    /// The framebuffer drawing into image `image` (`size` big): its texture (in `textures`,
    /// where the screens showing it find it, filtered `linear` or nearest) and a depth buffer,
    /// made the first time.
    fn target(
        &mut self,
        image: AssetId<Image>,
        size: (u32, u32),
        linear: bool,
    ) -> Result<glow::Framebuffer, String> {
        let (w, h) = size;
        if self.targets.get(&image).is_none_or(|t| t.2 != size) {
            let gl = &self.gl;
            // SAFETY: the context is current; the objects are this renderer's.
            unsafe {
                if let Some((depth, fb, _)) = self.targets.remove(&image) {
                    gl.delete_renderbuffer(depth);
                    gl.delete_framebuffer(fb);
                }
                if let Some((t, _, _)) = self.textures.remove(&image) {
                    gl.delete_texture(t);
                }
                let colour = gl.create_texture()?;
                gl.bind_texture(glow::TEXTURE_2D, Some(colour));
                gl.tex_storage_2d(glow::TEXTURE_2D, 1, glow::SRGB8_ALPHA8, w as i32, h as i32);
                let filter = if linear { glow::LINEAR } else { glow::NEAREST };
                for (k, v) in [
                    (glow::TEXTURE_MIN_FILTER, filter),
                    (glow::TEXTURE_MAG_FILTER, filter),
                    (glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE),
                    (glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE),
                ] {
                    gl.tex_parameter_i32(glow::TEXTURE_2D, k, v as i32);
                }
                let depth = gl.create_renderbuffer()?;
                gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth));
                gl.renderbuffer_storage(
                    glow::RENDERBUFFER,
                    glow::DEPTH_COMPONENT32F,
                    w as i32,
                    h as i32,
                );
                let fb = gl.create_framebuffer()?;
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
                gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0,
                    glow::TEXTURE_2D,
                    Some(colour),
                    0,
                );
                gl.framebuffer_renderbuffer(
                    glow::FRAMEBUFFER,
                    glow::DEPTH_ATTACHMENT,
                    glow::RENDERBUFFER,
                    Some(depth),
                );
                let complete =
                    gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE;
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                if !complete {
                    return Err("OpenGL can't draw into an offscreen image".into());
                }
                self.targets.insert(image, (depth, fb, size));
                // The screens showing the image sample this texture.
                self.textures.insert(image, (colour, size, 1));
            }
        }
        Ok(self.targets[&image].1)
    }

    /// The pixels of offscreen image `image` (RGBA, rows top down as the image's), waited for.
    fn read_target(&self, image: AssetId<Image>) -> Option<Vec<u8>> {
        let &(_, fb, (w, h)) = self.targets.get(&image)?;
        let mut pixels = vec![0u8; w as usize * h as usize * 4];
        // SAFETY: the context is current; `pixels` holds the whole image.
        unsafe {
            self.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(fb));
            self.gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
            self.gl.read_pixels(
                0,
                0,
                w as i32,
                h as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut pixels)),
            );
            self.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
        }
        Some(pixels)
    }

    fn egui_setup(
        gl: &glow::Context,
    ) -> Result<(Program, glow::VertexArray, glow::Buffer, glow::Buffer, glow::Texture), String>
    {
        let program = compile(gl, EGUI_SHADER, "#define SINGLE")?;
        // SAFETY: the context is current.
        unsafe {
            let vao = gl.create_vertex_array()?;
            let (vbo, ebo) = (gl.create_buffer()?, gl.create_buffer()?);
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            // epaint's Vertex: pos (2 f32), uv (2 f32), colour (4 u8), 20 bytes.
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 20, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 20, 8);
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 4, glow::UNSIGNED_BYTE, true, 20, 16);
            gl.bind_vertex_array(None);
            let white = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(white));
            gl.tex_storage_2d(glow::TEXTURE_2D, 1, glow::SRGB8_ALPHA8, 1, 1);
            gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                0,
                1,
                1,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&[255; 4])),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
            Ok((program, vao, vbo, ebo, white))
        }
    }

    /// Draws egui `view`'s meshes into its image, in gamma space as egui blends.
    fn draw_egui(&mut self, view: &EguiView) -> Result<(), String> {
        let fb = self.target(view.image, view.size, true)?;
        let (program, vao, vbo, ebo, white) = &self.egui;
        let gl = &self.gl;
        let (w, h) = (view.size.0 as f32, view.size.1 as f32);
        let ppp = view.ppp;
        // SAFETY: the context is current; the objects are this renderer's; the meshes' data
        // outlives the calls.
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
            gl.viewport(0, 0, w as i32, h as i32);
            gl.disable(glow::FRAMEBUFFER_SRGB);
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::CULL_FACE);
            gl.disable(glow::SCISSOR_TEST);
            let c = view.clear;
            gl.clear_color(c.x, c.y, c.z, c.w);
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.enable(glow::BLEND);
            gl.blend_func_separate(
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE_MINUS_DST_ALPHA,
                glow::ONE,
            );
            gl.enable(glow::SCISSOR_TEST);
            gl.use_program(Some(program.program));
            gl.uniform_2_f32(program.screen.as_ref(), w / ppp, h / ppp);
            gl.active_texture(glow::TEXTURE0);
            gl.bind_vertex_array(Some(*vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(*vbo));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(*ebo));
            for job in &view.jobs {
                if job.indices.is_empty() {
                    continue;
                }
                // (The image's rows go top down: so do the scissor's, here.)
                let [x0, y0, x1, y1] = job.clip.map(|v| v * ppp);
                let (x0, y0) = (x0.clamp(0.0, w), y0.clamp(0.0, h));
                let (x1, y1) = (x1.clamp(x0, w), y1.clamp(y0, h));
                if x1 <= x0 || y1 <= y0 {
                    continue;
                }
                gl.scissor(x0 as i32, y0 as i32, (x1 - x0).ceil() as i32, (y1 - y0).ceil() as i32);
                let texture =
                    job.texture.and_then(|t| self.textures.get(&t)).map_or(*white, |t| t.0);
                gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                let v = &job.vertices;
                let bytes = std::slice::from_raw_parts(
                    v.as_ptr().cast::<u8>(),
                    std::mem::size_of_val(&v[..]),
                );
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STREAM_DRAW);
                let i = &job.indices;
                let bytes = std::slice::from_raw_parts(i.as_ptr().cast::<u8>(), i.len() * 4);
                gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytes, glow::STREAM_DRAW);
                gl.draw_elements(glow::TRIANGLES, i.len() as i32, glow::UNSIGNED_INT, 0);
            }
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::BLEND);
            gl.enable(glow::FRAMEBUFFER_SRGB);
            gl.bind_vertex_array(None);
        }
        Ok(())
    }

    /// Draws offscreen view `off` into its image's texture.
    fn draw_offscreen(&mut self, off: &Offscreen, stats: &mut Stats) -> Result<(), String> {
        let (w, h) = off.size;
        // (The tracking camera's picture is sampled nearest, as its image says.)
        let fb = self.target(off.image, off.size, false)?;
        // SAFETY: the context is current.
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
            self.gl.viewport(0, 0, w as i32, h as i32);
        }
        self.clear(off.clear);
        // Drawn upside down: the image's rows go top down (as Bevy's, how the screens showing
        // it sample it), OpenGL's bottom up. The winding turns with it.
        let view = EyeView {
            view_proj: Mat4::from_scale(Vec3::new(1.0, -1.0, 1.0)) * off.view.view_proj,
            ..off.view
        };
        // SAFETY: the context is current.
        unsafe { self.gl.front_face(glow::CW) };
        self.set_views(self.single.all(), std::slice::from_ref(&view), off.size);
        self.draw_world(&self.single, &off.list, off.view.eye, false, stats);
        // SAFETY: the context is current.
        unsafe {
            self.gl.disable(glow::BLEND);
            self.gl.depth_mask(true);
            self.gl.front_face(glow::CCW);
            self.gl.invalidate_framebuffer(glow::FRAMEBUFFER, &[glow::DEPTH_ATTACHMENT]);
        }
        Ok(())
    }

    /// Draws `list`'s world (palette meshes, the background sphere, dot lights, effects) with
    /// `programs`, the blended ones sorted from `eye`; the framebuffer cleared before.
    fn draw_world(
        &self,
        programs: &Programs,
        list: &DrawList,
        eye: Vec3,
        volumetric: bool,
        stats: &mut Stats,
    ) {
        // The lighting options' variants while they are on (palette_material.rs
        // `sync_lighting`): the flags in `Globals.ambient.w`.
        let lit = list
            .globals
            .get(AMBIENT_W..AMBIENT_W + 4)
            .is_some_and(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]) != 0.0)
            as usize;
        // The palette meshes, instanced by mesh and material.
        let mut instances = Vec::new();
        let world =
            groups(list.items.iter().filter(|i| i.flags & FLAG_BACKGROUND == 0), &mut instances);
        let background =
            groups(list.items.iter().filter(|i| i.flags & FLAG_BACKGROUND != 0), &mut instances);
        let dots = groups(list.dots.iter(), &mut instances);
        let kind = |i: &&Item| match i.flags {
            f if f & FLAG_CLOUD_SOFT != 0 => 2,
            f if f & FLAG_CLOUD_COVERAGE != 0 => 1,
            _ => 0,
        };
        let clouds: [Vec<Group>; 3] = std::array::from_fn(|k| {
            groups(list.clouds.iter().filter(|i| kind(i) == k), &mut instances)
        });
        // SAFETY: the context is current; the buffer is this renderer's.
        unsafe {
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.instances));
            let bytes =
                std::slice::from_raw_parts(instances.as_ptr().cast::<u8>(), instances.len() * 4);
            self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STREAM_DRAW);
        }
        // Opaque: the world, the cloud surfaces, the stratus sheets (alpha to coverage), the
        // background sphere behind them.
        for (p, groups, coverage) in [
            (&programs.object[lit], &world, false),
            (&programs.cloud[lit], &clouds[0], false),
            (&programs.cloud[lit], &clouds[1], true),
            (&programs.background[lit], &background, false),
        ] {
            if groups.is_empty() {
                continue;
            }
            self.use_program(p);
            if coverage {
                // SAFETY: the context is current.
                unsafe { self.gl.enable(glow::SAMPLE_ALPHA_TO_COVERAGE) };
            }
            for g in groups {
                self.draw_group(p, g);
                stats.draws += 1;
            }
            if coverage {
                // SAFETY: the context is current.
                unsafe { self.gl.disable(glow::SAMPLE_ALPHA_TO_COVERAGE) };
            }
        }
        if volumetric {
            self.composite_clouds();
            stats.draws += 1;
        }
        // Smoke plumes, blended (their proxies have both windings, back faces culled).
        if !clouds[2].is_empty() {
            // SAFETY: the context is current.
            unsafe {
                self.gl.enable(glow::BLEND);
                self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                self.gl.depth_mask(false);
            }
            let p = &programs.cloud[lit];
            self.use_program(p);
            for g in &clouds[2] {
                self.draw_group(p, g);
                stats.draws += 1;
            }
        }
        // Dot lights: depth tested, not written (dots.rs).
        if !dots.is_empty() {
            // SAFETY: the context is current.
            unsafe {
                self.gl.enable(glow::BLEND);
                self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                self.gl.depth_mask(false);
            }
            let p = &programs.dot;
            self.use_program(p);
            for g in &dots {
                if let Some(m) = self.meshes.get(&g.mesh) {
                    // SAFETY: the context is current.
                    unsafe { self.gl.uniform_1_u32(p.dot_colour.as_ref(), m.colours as u32) };
                }
                self.draw_group(p, g);
                stats.draws += 1;
            }
        }
        // Effects, far to near: alpha blended, lightning and the afterburner adding their light.
        if !list.effects.is_empty() {
            let mut effects: Vec<&EffectItem> = list.effects.iter().collect();
            effects.sort_by(|a, b| {
                let d = |i: &EffectItem| i.model.w_axis.truncate().distance_squared(eye);
                d(b).total_cmp(&d(a))
            });
            // SAFETY: the context is current.
            unsafe {
                self.gl.enable(glow::BLEND);
                self.gl.depth_mask(false);
                self.gl.disable(glow::CULL_FACE);
            }
            let mut current = None;
            for item in effects {
                let i = Effect::ALL.iter().position(|e| *e == item.effect).unwrap_or(0);
                let p = &programs.effects[i];
                if current != Some(i) {
                    self.use_program(p);
                    // SAFETY: the context is current.
                    unsafe {
                        if matches!(item.effect, Effect::Bolt | Effect::Flame | Effect::Beam) {
                            self.gl.blend_func_separate(
                                glow::SRC_ALPHA,
                                glow::ONE,
                                glow::ONE,
                                glow::ONE_MINUS_SRC_ALPHA,
                            );
                        } else {
                            self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                        }
                    }
                    current = Some(i);
                }
                self.draw_effect(p, item);
                stats.draws += 1;
            }
            // SAFETY: the context is current.
            unsafe { self.gl.enable(glow::CULL_FACE) };
        }
    }

    /// The volumetric clouds' march into their reduced-resolution target (both eyes' layers):
    /// each batch's boxes, premultiplied over each other, the nearest distance kept; from
    /// inside a box its far side. Whether there is a target to composite.
    fn march_clouds(
        &mut self,
        c: &Clouds,
        eyes: &[EyeView],
        size: (u32, u32),
        stats: &mut Stats,
    ) -> Result<bool, String> {
        let small = (size.0.div_ceil(c.scale), size.1.div_ceil(c.scale));
        self.cloud_scale = eyes.first().map_or(1.0, |e| e.scale);
        if self.cloud_target.is_none_or(|t| t.3 != small) {
            let gl = &self.gl;
            // SAFETY: the context is current; the objects are this renderer's.
            unsafe {
                if let Some((a, b, fb, _)) = self.cloud_target.take() {
                    gl.delete_texture(a);
                    gl.delete_texture(b);
                    gl.delete_framebuffer(fb);
                }
                let layered = |format: u32, filter: u32| -> Result<glow::Texture, String> {
                    let t = gl.create_texture()?;
                    gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(t));
                    gl.tex_storage_3d(
                        glow::TEXTURE_2D_ARRAY,
                        1,
                        format,
                        small.0 as i32,
                        small.1 as i32,
                        2,
                    );
                    for (k, v) in [
                        (glow::TEXTURE_MIN_FILTER, filter),
                        (glow::TEXTURE_MAG_FILTER, filter),
                        (glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE),
                        (glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE),
                    ] {
                        gl.tex_parameter_i32(glow::TEXTURE_2D_ARRAY, k, v as i32);
                    }
                    Ok(t)
                };
                // Colour upsampled smoothly; the distance nearest (never blended across edges).
                let colour = layered(glow::RGBA16F, glow::LINEAR)?;
                let distance = layered(glow::R32F, glow::NEAREST)?;
                let fb = gl.create_framebuffer()?;
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
                (self.multiview)(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0,
                    colour.0.get(),
                    0,
                    0,
                    2,
                );
                (self.multiview)(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT1,
                    distance.0.get(),
                    0,
                    0,
                    2,
                );
                gl.draw_buffers(&[glow::COLOR_ATTACHMENT0, glow::COLOR_ATTACHMENT1]);
                let complete =
                    gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE;
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                if !complete {
                    return Err("OpenGL can't draw the clouds' target".into());
                }
                self.cloud_target = Some((colour, distance, fb, small));
            }
        }
        let fb = self.cloud_target.expect("the clouds' target").2;
        // SAFETY: the context is current.
        unsafe {
            let gl = &self.gl;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
            gl.viewport(0, 0, small.0 as i32, small.1 as i32);
            gl.disable(glow::DEPTH_TEST);
            gl.depth_mask(false);
            gl.clear_buffer_f32_slice(glow::COLOR, 0, &[0.0; 4]);
            gl.clear_buffer_f32_slice(glow::COLOR, 1, &[65504.0; 4]);
            gl.enable(glow::BLEND);
            gl.blend_func_draw_buffer(0, glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            gl.blend_equation_draw_buffer(0, glow::FUNC_ADD);
            gl.blend_func_draw_buffer(1, glow::ONE, glow::ONE);
            gl.blend_equation_draw_buffer(1, glow::MIN);
        }
        self.draw_clouds(c, eyes, small, stats);
        // SAFETY: the context is current.
        unsafe {
            let gl = &self.gl;
            gl.blend_equation(glow::FUNC_ADD);
            gl.disable(glow::BLEND);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_mask(true);
        }
        Ok(true)
    }

    /// The clouds over the eyes' image (the framebuffer bound): their distance as the depth,
    /// tested (nearer geometry hides them), not written.
    fn composite_clouds(&self) {
        let Some((colour, distance, _, _)) = self.cloud_target else { return };
        let p = &self.cloud_composite;
        let gl = &self.gl;
        // SAFETY: the context is current; the objects are this renderer's.
        unsafe {
            gl.use_program(Some(p.program));
            gl.uniform_1_i32(gl.get_uniform_location(p.program, "u_clouds").as_ref(), 0);
            gl.uniform_1_i32(gl.get_uniform_location(p.program, "u_distance").as_ref(), 1);
            // The distances are in world units, the depth in the (scaled) eyes' units.
            let near = NEAR * self.cloud_scale;
            gl.uniform_1_f32(gl.get_uniform_location(p.program, "u_near").as_ref(), near);
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(colour));
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(distance));
            gl.active_texture(glow::TEXTURE0);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            gl.depth_mask(false);
            gl.disable(glow::CULL_FACE);
            gl.bind_vertex_array(Some(self.cloud_vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            gl.bind_vertex_array(None);
            gl.enable(glow::CULL_FACE);
            gl.disable(glow::BLEND);
            gl.depth_mask(true);
        }
    }

    /// The clouds' boxes (`march_clouds`).
    fn draw_clouds(&self, c: &Clouds, eyes: &[EyeView], size: (u32, u32), stats: &mut Stats) {
        let gl = &self.gl;
        let as_bytes = |v: &[f32]| {
            // SAFETY: plain floats.
            unsafe { std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), v.len() * 4) }
        };
        let words = |v: &[u32]| {
            // SAFETY: plain words.
            unsafe { std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), v.len() * 4) }
        };
        // SAFETY: the context is current; the buffers are this renderer's.
        unsafe {
            gl.bind_buffer(glow::UNIFORM_BUFFER, Some(self.cloud_buffers[0]));
            gl.buffer_data_u8_slice(glow::UNIFORM_BUFFER, as_bytes(&c.globals), glow::STREAM_DRAW);
            gl.bind_buffer_base(glow::UNIFORM_BUFFER, 1, Some(self.cloud_buffers[0]));
            gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(self.cloud_buffers[1]));
            gl.buffer_data_u8_slice(
                glow::SHADER_STORAGE_BUFFER,
                words(&c.volumes),
                glow::STREAM_DRAW,
            );
            gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, 0, Some(self.cloud_buffers[1]));
            gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(self.cloud_buffers[2]));
            gl.buffer_data_u8_slice(
                glow::SHADER_STORAGE_BUFFER,
                as_bytes(&c.spheres),
                glow::STREAM_DRAW,
            );
            gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, 1, Some(self.cloud_buffers[2]));
            gl.bind_vertex_array(Some(self.cloud_vao));
            gl.enable(glow::CULL_FACE);
        }
        for (batch, &(first, count)) in c.batches.iter().enumerate() {
            if count == 0 {
                continue;
            }
            let lod = batch as u32 % 4;
            let key = (lod, c.steps[lod as usize].max(1), c.dither, c.sun_shadow);
            let mut programs = self.cloud_programs.borrow_mut();
            let p = match programs.get(&key) {
                Some(p) => p,
                None => {
                    let mut defines = format!("#define MARCH_STEPS {}\n", key.1);
                    for j in 1..=lod {
                        defines += &format!("#define J{j}\n");
                    }
                    if c.dither {
                        defines += "#define DITHER\n";
                    }
                    if c.sun_shadow {
                        defines += "#define SUN_SHADOW\n";
                    }
                    match compile(gl, CLOUDS_SHADER, &defines) {
                        Ok(p) => programs.entry(key).or_insert(p),
                        Err(e) => {
                            warn!("glvr: the clouds' shader: {e}");
                            return;
                        }
                    }
                }
            };
            self.set_views(std::iter::once(p), eyes, size);
            // SAFETY: the context is current; the program is this renderer's.
            unsafe {
                gl.use_program(Some(p.program));
                gl.cull_face(if batch >= 4 { glow::FRONT } else { glow::BACK });
                gl.uniform_1_u32(p.first.as_ref(), first);
                gl.draw_arrays_instanced(glow::TRIANGLES, 0, 36, count as i32);
            }
            stats.draws += 1;
        }
        // SAFETY: the context is current.
        unsafe {
            gl.cull_face(glow::BACK);
            gl.bind_vertex_array(None);
        }
    }

    /// A fence after everything asked of the GPU so far: the image just drawn is released to
    /// the runtime only once it has passed (`wait`). The runtime may show an image as soon as
    /// it is released, and one half drawn showed its last contents (a frame of a few frames
    /// ago, in one eye or both).
    pub fn fence(&self) -> Option<glow::Fence> {
        // SAFETY: the context is current.
        unsafe {
            let fence = self.gl.fence_sync(glow::SYNC_GPU_COMMANDS_COMPLETE, 0).ok();
            // Submitted now: Zink's glFlush leaves the frame's work queued (it started only when
            // the image was released, or waited for); a wait that flushes and doesn't wait
            // sends it.
            if let Some(f) = fence {
                self.gl.client_wait_sync(f, glow::SYNC_FLUSH_COMMANDS_BIT, 0);
            }
            fence
        }
    }

    /// Waits for `fence` (`fence`).
    pub fn wait(&self, fence: glow::Fence) {
        // SAFETY: the context is current; the fence is this renderer's.
        unsafe {
            self.gl.client_wait_sync(fence, glow::SYNC_FLUSH_COMMANDS_BIT, 100_000_000);
            self.gl.delete_sync(fence);
        }
    }

    fn use_program(&self, p: &Program) {
        // SAFETY: the context is current.
        unsafe { self.gl.use_program(Some(p.program)) };
    }

    /// Saves both eyes of image `texture` as `xr-eye0.png` and `xr-eye1.png` (as `SM89_XR_SHOT`
    /// does for Bevy's headset images): what the headset is given, foveation and all.
    pub fn save(&self, texture: glow::Texture, size: (u32, u32)) -> Result<(), String> {
        let gl = &self.gl;
        let (w, h) = (size.0 as usize, size.1 as usize);
        for layer in 0..2 {
            let mut pixels = vec![0u8; w * h * 4];
            // SAFETY: the context is current; `pixels` holds the whole layer.
            unsafe {
                let fb = gl.create_framebuffer()?;
                gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(fb));
                gl.framebuffer_texture_layer(
                    glow::READ_FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0,
                    Some(texture),
                    0,
                    layer,
                );
                gl.read_pixels(
                    0,
                    0,
                    size.0 as i32,
                    size.1 as i32,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelPackData::Slice(Some(&mut pixels)),
                );
                gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
                gl.delete_framebuffer(fb);
            }
            // OpenGL's rows go bottom up.
            let rows: Vec<u8> = pixels.chunks_exact(w * 4).rev().flatten().copied().collect();
            let path = format!("xr-eye{layer}.png");
            image::save_buffer(&path, &rows, size.0, size.1, image::ColorType::Rgba8)
                .map_err(|e| format!("{path}: {e}"))?;
            info!("glvr: saved {path}");
        }
        Ok(())
    }

    fn bind_mesh(&self, id: AssetId<Mesh>) -> Option<&GpuMesh> {
        let m = self.meshes.get(&id)?;
        // SAFETY: the context is current; the vertex array is this renderer's.
        unsafe { self.gl.bind_vertex_array(Some(m.vao)) };
        Some(m)
    }

    fn submit(&self, m: &GpuMesh) {
        // SAFETY: the context is current; the mesh's vertex array is bound.
        unsafe {
            match m.indexed {
                Some(kind) => self.gl.draw_elements(m.mode, m.count, kind, 0),
                None => self.gl.draw_arrays(m.mode, 0, m.count),
            }
        }
    }

    /// Draws group `g`'s instances (in the instance buffer).
    fn draw_group(&self, p: &Program, g: &Group) {
        let Some(m) = self.bind_mesh(g.mesh) else { return };
        let gl = &self.gl;
        let stride = INSTANCE_WORDS as i32 * 4;
        let base = (g.first * INSTANCE_WORDS * 4) as i32;
        // SAFETY: the context is current; the mesh's vertex array is bound and the instance
        // buffer holds the group's instances.
        unsafe {
            gl.uniform_1_u32(p.word.as_ref(), g.word);
            gl.uniform_1_u32(p.flags.as_ref(), g.flags);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.instances));
            for c in 0..4 {
                gl.enable_vertex_attrib_array(4 + c);
                gl.vertex_attrib_pointer_f32(
                    4 + c,
                    4,
                    glow::FLOAT,
                    false,
                    stride,
                    base + c as i32 * 16,
                );
                gl.vertex_attrib_divisor(4 + c, 1);
            }
            gl.enable_vertex_attrib_array(8);
            gl.vertex_attrib_pointer_i32(8, 1, glow::UNSIGNED_INT, stride, base + 64);
            gl.vertex_attrib_divisor(8, 1);
            let n = g.count as i32;
            match m.indexed {
                Some(kind) => gl.draw_elements_instanced(m.mode, m.count, kind, 0, n),
                None => gl.draw_arrays_instanced(m.mode, 0, m.count, n),
            }
        }
    }

    fn draw_effect(&self, p: &Program, item: &EffectItem) {
        let gl = &self.gl;
        // SAFETY: the context is current.
        unsafe {
            gl.uniform_matrix_4_f32_slice(p.model.as_ref(), false, &item.model.to_cols_array());
            gl.uniform_4_u32_slice(p.raw.as_ref(), &item.raw);
        }
        if let Some(m) = self.bind_mesh(item.mesh) {
            self.submit(m);
        }
    }

    fn draw_glass(&self, item: &GlassItem) {
        let Some(&(texture, _, _)) = self.textures.get(&item.texture) else { return };
        let p = &self.glass;
        let gl = &self.gl;
        // SAFETY: the context is current; the texture is this renderer's.
        unsafe {
            gl.uniform_matrix_4_f32_slice(p.model.as_ref(), false, &item.model.to_cols_array());
            gl.uniform_matrix_4_f32_slice(
                p.local.as_ref(),
                false,
                &item.model.inverse().to_cols_array(),
            );
            gl.uniform_4_f32_slice(p.tint.as_ref(), &item.tint.to_array());
            gl.uniform_4_f32_slice(p.proj.as_ref(), &item.proj.to_array());
            gl.uniform_4_f32_slice(p.style.as_ref(), &item.style.to_array());
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        }
        if let Some(m) = self.bind_mesh(item.mesh) {
            self.submit(m);
        }
    }

    fn draw_lit(&self, item: &LitItem) {
        let p = &self.lit;
        let gl = &self.gl;
        let base_tex = item.base_texture.and_then(|id| self.textures.get(&id)).map(|t| t.0);
        let emissive_tex = item.emissive_texture.and_then(|id| self.textures.get(&id)).map(|t| t.0);
        // A texture still on its way: not drawn yet.
        if item.base_texture.is_some() && base_tex.is_none() {
            return;
        }
        let flags = item.flags | base_tex.map_or(0, |_| 1) | emissive_tex.map_or(0, |_| 2);
        let normal = Mat3::from_mat4(item.model).inverse().transpose();
        // SAFETY: the context is current; the textures are this renderer's.
        unsafe {
            gl.uniform_matrix_4_f32_slice(p.model.as_ref(), false, &item.model.to_cols_array());
            gl.uniform_matrix_3_f32_slice(p.normal.as_ref(), false, &normal.to_cols_array());
            gl.uniform_4_f32_slice(p.base.as_ref(), &item.base.to_array());
            gl.uniform_4_f32_slice(p.emissive.as_ref(), &item.emissive.to_array());
            gl.uniform_1_u32(p.flags.as_ref(), flags);
            gl.uniform_1_f32(p.cutoff.as_ref(), item.cutoff);
            gl.uniform_4_f32_slice(p.night.as_ref(), &item.night.to_array());
            gl.uniform_4_f32_slice(p.flood.as_ref(), &item.flood.to_array());
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, base_tex);
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, emissive_tex);
            if item.double_sided {
                gl.disable(glow::CULL_FACE);
            }
        }
        if let Some(m) = self.bind_mesh(item.mesh) {
            self.submit(m);
        }
        if item.double_sided {
            // SAFETY: the context is current.
            unsafe { gl.enable(glow::CULL_FACE) };
        }
    }
}

//! What the render thread draws, taken from the game's world after each update: the meshes
//! seen by the eye cameras (frustum culled) with their transforms and materials (the palette
//! world; the cockpit's and the other textured StandardMaterial meshes), the shader globals,
//! and the rig's eye point. Meshes and images go to the render thread once (again when
//! changed), on their own channel so a draw list replaced before it was drawn loses none.

use super::render::{
    AssetMsg, EyeView, MeshData, TextureData, camera_view, eye_views, inside, planes,
};
use crate::afterburner_flame::FlameMaterial;
use crate::camera::{MAIN_BACKGROUND_LAYER, MAIN_LOD_LAYER, OffscreenCamera};
use crate::cockpit3d::CockpitMaterial;
use crate::cockpit3d::hud_glass::{HmdMaterial, HudGlassMaterial};
use crate::explosions::ParticleMaterial;
use crate::landing_lights::BeamMaterial;
use crate::lightning::BoltMaterial;
use crate::palette::Palette;
use crate::palette_material::{FLAG_CLOUD, FLAG_DOT, PaletteMaterial};
use crate::rotor_blur::RotorMaterial;
use crate::trails::TrailMaterial;
use bevy::asset::AssetId;
use bevy::camera::RenderTarget;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::image::{ImageFilterMode, ImageSampler};
use bevy::mesh::{MeshTag, MeshVertexAttribute, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::storage::ShaderBuffer;
use openxr as xr;
use std::collections::HashSet;
use std::sync::mpsc::Sender;

/// The eyes' fields of view are widened this much for culling: the draw list is drawn a frame
/// after the poses it was culled with.
const CULL_MARGIN: f32 = 1.15;

/// A palette mesh to draw: its transform, mesh tag and palette material.
#[derive(Clone, Copy)]
pub struct Item {
    pub mesh: AssetId<Mesh>,
    pub model: Mat4,
    pub tag: u32,
    pub word: u32,
    pub flags: u32,
}

/// `LitItem::flags` bit of what is drawn over everything (the tooltip's screen), last.
pub const ON_TOP: u32 = 1 << 8;

/// A textured (cockpit or StandardMaterial) mesh to draw.
pub struct LitItem {
    pub mesh: AssetId<Mesh>,
    pub model: Mat4,
    /// Linear RGBA.
    pub base: Vec4,
    pub emissive: Vec4,
    pub base_texture: Option<AssetId<Image>>,
    pub emissive_texture: Option<AssetId<Image>>,
    /// lit.glsl's `u_flags` bits 2.. (unlit 4, never darkened 8, alpha mask 16).
    pub flags: u32,
    pub cutoff: f32,
    /// The cockpit's night darkness (`CockpitShading::night`) and flood light
    /// (`CockpitShading::flood`), zero for others.
    pub night: Vec4,
    pub flood: Vec4,
    pub blend: bool,
    pub double_sided: bool,
}

/// The HUD glass or the helmet display: the HUD canvas collimated to infinity.
pub struct GlassItem {
    pub mesh: AssetId<Mesh>,
    pub model: Mat4,
    pub tint: Vec4,
    pub proj: Vec4,
    pub style: Vec4,
    pub texture: AssetId<Image>,
}

/// An effect's kind (its program in render.rs): explosion particles, trails, rotor discs,
/// lightning, the afterburner's plume, the landing lights' beams.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    Particle,
    Trail,
    Rotor,
    Bolt,
    Flame,
    Beam,
}

impl Effect {
    pub const ALL: [Effect; 6] =
        [Effect::Particle, Effect::Trail, Effect::Rotor, Effect::Bolt, Effect::Flame, Effect::Beam];

    /// The mesh's attributes besides positions, and their shader locations.
    fn attributes(self) -> &'static [(u32, MeshVertexAttribute)] {
        match self {
            Effect::Particle => &[
                (1, crate::explosions::ATTRIBUTE_CORNER),
                (2, crate::explosions::ATTRIBUTE_COLOUR),
                (3, crate::explosions::ATTRIBUTE_LOOK),
            ],
            Effect::Trail | Effect::Bolt => {
                &[(1, crate::trails::ATTRIBUTE_TANGENT), (2, crate::trails::ATTRIBUTE_SIDE)]
            }
            Effect::Rotor | Effect::Flame | Effect::Beam => &[(1, Mesh::ATTRIBUTE_UV_0)],
        }
    }
}

/// A particle batch, trail, rotor disc or lightning channel to draw: its material's uniform as
/// its bytes.
pub struct EffectItem {
    pub effect: Effect,
    pub mesh: AssetId<Mesh>,
    pub model: Mat4,
    pub raw: [u32; 8],
}

/// A camera drawing into an image: the image, its size, the view, the clear colour (linear).
pub struct Offscreen {
    pub image: AssetId<Image>,
    pub size: (u32, u32),
    pub view: EyeView,
    pub clear: Vec4,
    pub list: DrawList,
}

/// A menu drawn with egui into an image (the VR menu's panel): its paint jobs.
pub struct EguiView {
    pub image: AssetId<Image>,
    pub size: (u32, u32),
    /// Pixels per egui point.
    pub ppp: f32,
    pub clear: Vec4,
    pub jobs: Vec<EguiJob>,
}

/// An egui mesh: its clip rectangle (points: min x, min y, max x, max y), its texture (None:
/// white, or one not drawn yet).
pub struct EguiJob {
    pub clip: [f32; 4],
    pub vertices: Vec<bevy_egui::egui::epaint::Vertex>,
    pub indices: Vec<u32>,
    pub texture: Option<AssetId<Image>>,
}

/// The volumetric clouds of the frame (bevy_fast_clouds' `CloudFrame`), packed for the
/// shader: globals (std140), volumes and spheres (std430), the batches (exterior LOD 0..3,
/// interior LOD 0..3: first, count), the march steps per LOD, the look.
pub struct Clouds {
    pub globals: Vec<f32>,
    pub volumes: Vec<u32>,
    pub spheres: Vec<f32>,
    pub batches: [(u32, u32); 8],
    pub steps: [u32; 4],
    pub dither: bool,
    pub sun_shadow: bool,
    /// The march's pixels are this many view pixels across.
    pub scale: u32,
}

/// One update's picture.
#[derive(Default)]
pub struct DrawList {
    /// The rig's eye camera (the cockpit's eye point) in the world.
    pub rig: Mat4,
    /// The palette shader's `Globals`.
    pub globals: Vec<u8>,
    pub items: Vec<Item>,
    /// Batched dot lights (drawn after everything opaque).
    pub dots: Vec<Item>,
    /// Clouds and smoke plumes (palette_material.rs `FLAG_CLOUD`).
    pub clouds: Vec<Item>,
    pub lit: Vec<LitItem>,
    pub glass: Vec<GlassItem>,
    pub effects: Vec<EffectItem>,
    pub culled: u32,
    /// The views drawn into images first (the tracking camera), with what they see.
    pub offscreen: Vec<Offscreen>,
    /// The menus drawn into images first.
    pub egui: Vec<EguiView>,
    /// Images read back to the main thread after they are drawn (recon.rs's photos): the
    /// `Readback` entity and its image.
    pub readbacks: Vec<(Entity, AssetId<Image>)>,
    /// The volumetric clouds, if there are any.
    pub volumetric: Option<Clouds>,
    /// Haptic pulses asked for (`Hand::haptic`), per hand.
    pub haptics: [f32; 2],
}

/// The eyes and the controllers as the render thread last saw them (local space).
#[derive(Clone)]
pub struct Eyes {
    pub views: Vec<xr::View>,
    pub controllers: [super::controllers::Controller; 2],
}

type Common = (
    &'static Mesh3d,
    &'static GlobalTransform,
    &'static InheritedVisibility,
    Option<&'static RenderLayers>,
    Option<&'static Aabb>,
    Has<NoFrustumCulling>,
);

/// A mesh's common components, as a query gives them.
type CommonItem<'a> = (
    &'a Mesh3d,
    &'a GlobalTransform,
    &'a InheritedVisibility,
    Option<&'a RenderLayers>,
    Option<&'a Aabb>,
    bool,
);

type PaletteQuery = (Common, &'static MeshMaterial3d<PaletteMaterial>, Option<&'static MeshTag>);
type CockpitQuery = (Common, &'static MeshMaterial3d<CockpitMaterial>);
type StandardQuery = (Common, &'static MeshMaterial3d<StandardMaterial>);
type HudQuery = (Common, &'static MeshMaterial3d<HudGlassMaterial>);
type HmdQuery = (Common, &'static MeshMaterial3d<HmdMaterial>);
type ParticleQuery = (Common, &'static MeshMaterial3d<ParticleMaterial>);
type TrailQuery = (Common, &'static MeshMaterial3d<TrailMaterial>);
type RotorQuery = (Common, &'static MeshMaterial3d<RotorMaterial>);
type BoltQuery = (Common, &'static MeshMaterial3d<BoltMaterial>);
type FlameQuery = (Common, &'static MeshMaterial3d<FlameMaterial>);
type BeamQuery = (Common, &'static MeshMaterial3d<BeamMaterial>);

/// What a mesh is drawn with: whether it is seen.
struct Cull<'a> {
    layers: RenderLayers,
    default_layer: RenderLayers,
    frusta: &'a [[Vec4; 5]],
}

impl Cull<'_> {
    /// Whether `c` is seen: None if hidden or on other layers, Some(false) if outside the
    /// eyes' frusta.
    fn seen(&self, c: &CommonItem) -> Option<bool> {
        let (_, transform, visible, layers, aabb, no_cull) = *c;
        if !visible.get() || !layers.unwrap_or(&self.default_layer).intersects(&self.layers) {
            return None;
        }
        if !no_cull
            && let Some(aabb) = aabb
            && !self.frusta.is_empty()
            && !self.frusta.iter().any(|f| inside(f, transform, aabb))
        {
            return Some(false);
        }
        Some(true)
    }
}

/// The queries of the meshes drawn, by material.
struct Queries {
    palette: QueryState<PaletteQuery>,
    cockpit: QueryState<CockpitQuery>,
    standard: QueryState<StandardQuery>,
    hud: QueryState<HudQuery>,
    hmd: QueryState<HmdQuery>,
    particles: QueryState<ParticleQuery>,
    trails: QueryState<TrailQuery>,
    rotors: QueryState<RotorQuery>,
    bolts: QueryState<BoltQuery>,
    flames: QueryState<FlameQuery>,
    beams: QueryState<BeamQuery>,
    offscreen: QueryState<OffscreenQuery, With<OffscreenCamera>>,
    egui: QueryState<EguiQuery>,
    readbacks: QueryState<(Entity, &'static crate::recon::ImageReadback)>,
    tips: QueryState<TipQuery>,
    ui_cameras: QueryState<(&'static Camera, &'static RenderTarget), With<Camera2d>>,
    ui_nodes: QueryState<UiNodeQuery>,
}

type TipQuery = (Common, &'static MeshMaterial3d<crate::cockpit3d::tooltips::TipScreenMaterial>);

/// A bevy_ui node as drawn here: its box, background, outline, text.
type UiNodeQuery = (
    &'static bevy::ui::ComputedNode,
    &'static bevy::ui::UiGlobalTransform,
    &'static InheritedVisibility,
    &'static bevy::ui::ComputedUiTargetCamera,
    Option<&'static BackgroundColor>,
    Option<&'static Outline>,
    Option<&'static bevy::text::TextLayoutInfo>,
    Option<&'static TextColor>,
);

/// The egui contexts drawing into images.
type EguiQuery = (
    Entity,
    &'static Camera,
    &'static RenderTarget,
    &'static bevy_egui::EguiRenderOutput,
    Option<&'static bevy_egui::EguiOutput>,
);

impl Queries {
    fn new(world: &mut World) -> Self {
        Queries {
            palette: world.query(),
            cockpit: world.query(),
            standard: world.query(),
            hud: world.query(),
            hmd: world.query(),
            particles: world.query(),
            trails: world.query(),
            rotors: world.query(),
            bolts: world.query(),
            flames: world.query(),
            beams: world.query(),
            offscreen: world.query_filtered::<OffscreenQuery, With<OffscreenCamera>>(),
            egui: world.query(),
            readbacks: world.query(),
            tips: world.query(),
            ui_cameras: world.query_filtered(),
            ui_nodes: world.query(),
        }
    }
}

/// The cameras drawing into images (the tracking camera).
type OffscreenQuery = (
    &'static Camera,
    &'static RenderTarget,
    &'static GlobalTransform,
    &'static Projection,
    Option<&'static RenderLayers>,
);

pub struct Extractor {
    assets: Sender<AssetMsg>,
    /// Meshes and images the render thread has.
    sent: HashSet<AssetId<Mesh>>,
    sent_images: HashSet<AssetId<Image>>,
    /// Images cameras draw into: the render thread draws them, they are never sent.
    targets: HashSet<AssetId<Image>>,
    /// Readbacks asked of the render thread already.
    read: HashSet<Entity>,
    queries: Option<Queries>,
    camera: Option<QueryState<&'static GlobalTransform>>,
}

impl Extractor {
    pub fn new(assets: Sender<AssetMsg>) -> Self {
        Extractor {
            assets,
            sent: HashSet::new(),
            sent_images: HashSet::new(),
            targets: HashSet::new(),
            read: HashSet::new(),
            queries: None,
            camera: None,
        }
    }

    /// Meshes changed or gone: the render thread drops them, they are sent again when drawn.
    pub fn forget(&mut self, ids: impl Iterator<Item = AssetId<Mesh>>) {
        for id in ids {
            if self.sent.remove(&id) {
                let _ = self.assets.send(AssetMsg::ForgetMesh(id));
            }
        }
    }

    /// Images changed (sent again when drawn: the canvases every frame they change) or gone
    /// (`true`: the render thread drops them).
    pub fn forget_images(&mut self, ids: impl Iterator<Item = (AssetId<Image>, bool)>) {
        for (id, gone) in ids {
            if self.sent_images.remove(&id) && gone {
                let _ = self.assets.send(AssetMsg::ForgetTexture(id));
            }
        }
    }

    /// Sends mesh `id` if the render thread doesn't have it: whether it has it now.
    fn send_mesh(&mut self, meshes: &Assets<Mesh>, id: AssetId<Mesh>) -> bool {
        self.send_mesh_with(meshes, id, &[])
    }

    /// `send_mesh`, with the attributes `extra` too (at their shader locations).
    fn send_mesh_with(
        &mut self,
        meshes: &Assets<Mesh>,
        id: AssetId<Mesh>,
        extra: &[(u32, MeshVertexAttribute)],
    ) -> bool {
        if self.sent.contains(&id) {
            return true;
        }
        let Some(mut data) = meshes.get(id).and_then(mesh_data) else { return false };
        if let Some(mesh) = meshes.get(id) {
            data.extra = extra_attributes(mesh, extra);
        }
        let _ = self.assets.send(AssetMsg::Mesh(id, data));
        self.sent.insert(id);
        true
    }

    fn send_image(&mut self, images: &Assets<Image>, id: AssetId<Image>) {
        if self.sent_images.contains(&id) || self.targets.contains(&id) {
            return;
        }
        let Some(image) = images.get(id) else { return };
        if let Some(data) = texture_data(image) {
            let _ = self.assets.send(AssetMsg::Texture(id, data));
        }
        // (An image of a format not handled is not tried again until it changes.)
        self.sent_images.insert(id);
    }

    /// The draw list of `world`, culled to `eyes` (everything while there are none yet).
    pub fn extract(&mut self, world: &mut World, eyes: Option<&Eyes>) -> DrawList {
        // The eyes are drawn from the tracking root.
        let root = world.get_resource::<crate::camera::XrRoot>().and_then(|r| r.0);
        let camera = self.camera.get_or_insert_with(|| world.query::<&GlobalTransform>());
        let rig =
            root.and_then(|r| camera.get(world, r).ok()).map_or(Mat4::IDENTITY, |t| t.to_matrix());
        let frusta: Vec<[Vec4; 5]> = eyes
            .map(|e| {
                eye_views(rig, &e.views, CULL_MARGIN).iter().map(|v| planes(v.view_proj)).collect()
            })
            .unwrap_or_default();
        let globals = world
            .get_resource::<Palette>()
            .and_then(|p| world.resource::<Assets<ShaderBuffer>>().get(&p.globals))
            .and_then(|b| b.data.clone())
            .unwrap_or_default();
        let cull = Cull {
            layers: RenderLayers::from_layers(&[0, MAIN_BACKGROUND_LAYER, MAIN_LOD_LAYER]),
            default_layer: RenderLayers::default(),
            frusta: &frusta,
        };
        let mut list = DrawList { rig, globals, ..default() };
        if let Some(mut hands) = world.get_resource_mut::<crate::cockpit3d::Hands>() {
            for (i, hand) in hands.hands.iter_mut().enumerate() {
                list.haptics[i] = std::mem::take(&mut hand.haptic);
            }
        }
        let mut q = self.queries.take().unwrap_or_else(|| Queries::new(world));
        let world: &World = world;
        let images = world.resource::<Assets<Image>>();
        let cameras: Vec<_> = q
            .offscreen
            .iter(world)
            .filter(|(c, ..)| c.is_active)
            .filter_map(|(c, target, t, projection, layers)| {
                let RenderTarget::Image(target) = target else { return None };
                let Projection::Perspective(p) = projection else { return None };
                let image = images.get(&target.handle)?;
                let clear = match c.clear_color {
                    ClearColorConfig::Custom(c) => Vec4::from_array(c.to_linear().to_f32_array()),
                    _ => Vec4::new(0.0, 0.0, 0.0, 1.0),
                };
                let view = camera_view(t.to_matrix(), p.fov, p.aspect_ratio, p.near);
                Some((
                    target.handle.id(),
                    image.size(),
                    view,
                    clear,
                    layers.cloned().unwrap_or_default(),
                ))
            })
            .collect();
        self.targets.extend(cameras.iter().map(|c| c.0));
        self.egui_into(&mut q, world, &mut list);
        self.ui_into(&mut q, world, &mut list);
        list.volumetric = clouds(world);
        // Readbacks of the images drawn here, each asked for once.
        let live: HashSet<Entity> = q.readbacks.iter(world).map(|(e, _)| e).collect();
        self.read.retain(|e| live.contains(e));
        for (e, r) in q.readbacks.iter(world) {
            let image = &r.0;
            if self.targets.contains(&image.id()) && self.read.insert(e) {
                list.readbacks.push((e, image.id()));
            }
        }
        self.world_into(&mut q, world, &cull, &mut list);
        self.cockpit_into(&mut q, world, &cull, &mut list);
        // The cameras drawing into images, each with what it sees.
        for (image, size, view, clear, layers) in cameras {
            let frusta = [planes(view.view_proj)];
            let cull = Cull { layers, default_layer: RenderLayers::default(), frusta: &frusta };
            let mut sub = DrawList::default();
            self.world_into(&mut q, world, &cull, &mut sub);
            list.offscreen.push(Offscreen {
                image,
                size: (size.x, size.y),
                view,
                clear,
                list: sub,
            });
        }
        self.queries = Some(q);
        list
    }

    /// The egui contexts drawing into images (not into the absent window): their paint jobs,
    /// their textures sent.
    fn egui_into(&mut self, q: &mut Queries, world: &World, list: &mut DrawList) {
        use bevy_egui::egui;
        let images = world.resource::<Assets<Image>>();
        let managed = world.get_resource::<bevy_egui::EguiManagedTextures>();
        for (entity, camera, target, output, ppp) in q.egui.iter(world) {
            if !camera.is_active {
                continue;
            }
            let RenderTarget::Image(target) = target else { continue };
            let Some(image) = images.get(&target.handle) else { continue };
            let id = target.handle.id();
            self.targets.insert(id);
            let clear = match camera.clear_color {
                ClearColorConfig::Custom(c) => Vec4::from_array(c.to_srgba().to_f32_array()),
                _ => Vec4::ZERO,
            };
            let mut jobs = Vec::new();
            for job in &output.paint_jobs {
                let egui::epaint::Primitive::Mesh(mesh) = &job.primitive else { continue };
                let texture = match mesh.texture_id {
                    egui::TextureId::Managed(t) => {
                        managed.and_then(|m| m.0.get(&(entity, t))).map(|m| m.handle.id())
                    }
                    // (User textures: the campaign's unit picture, not drawn yet.)
                    egui::TextureId::User(_) => None,
                };
                if let Some(t) = texture {
                    self.send_image(images, t);
                }
                let r = job.clip_rect;
                jobs.push(EguiJob {
                    clip: [r.min.x, r.min.y, r.max.x, r.max.y],
                    vertices: mesh.vertices.clone(),
                    indices: mesh.indices.clone(),
                    texture,
                });
            }
            list.egui.push(EguiView {
                image: id,
                size: (image.width(), image.height()),
                ppp: ppp.map_or(1.0, |p| p.pixels_per_point),
                clear,
                jobs,
            });
        }
    }

    /// The bevy_ui nodes drawn into images (the cockpit tooltip in VR): their backgrounds,
    /// outlines and text as egui meshes (gamma space, premultiplied, as egui's), in the order
    /// bevy_ui stacks them.
    fn ui_into(&mut self, q: &mut Queries, world: &World, list: &mut DrawList) {
        use bevy_egui::egui::{Color32, Pos2, epaint::Vertex};
        let Some(stack) = world.get_resource::<bevy::ui::UiStack>() else { return };
        let images = world.resource::<Assets<Image>>();
        let mut views: Vec<(Entity, EguiView)> = Vec::new();
        let colour = |c: Color| {
            let [r, g, b, a] = c.to_srgba().to_u8_array();
            Color32::from_rgba_unmultiplied(r, g, b, a)
        };
        for &e in &stack.uinodes {
            let Ok((node, transform, visible, target, background, outline, text, text_colour)) =
                q.ui_nodes.get(world, e)
            else {
                continue;
            };
            let Some(camera) = target.get() else { continue };
            if !visible.get() || node.size() == Vec2::ZERO {
                continue;
            }
            let i = match views.iter().position(|(c, _)| *c == camera) {
                Some(i) => i,
                None => {
                    let Ok((cam, RenderTarget::Image(t))) = q.ui_cameras.get(world, camera) else {
                        continue;
                    };
                    let Some(image) = images.get(&t.handle) else { continue };
                    if !cam.is_active {
                        continue;
                    }
                    let clear = match cam.clear_color {
                        ClearColorConfig::Custom(c) => {
                            Vec4::from_array(c.to_srgba().to_f32_array())
                        }
                        _ => Vec4::ZERO,
                    };
                    self.targets.insert(t.handle.id());
                    let view = EguiView {
                        image: t.handle.id(),
                        size: (image.width(), image.height()),
                        ppp: 1.0,
                        clear,
                        jobs: Vec::new(),
                    };
                    views.push((camera, view));
                    views.len() - 1
                }
            };
            let (w, h) = (views[i].1.size.0 as f32, views[i].1.size.1 as f32);
            let jobs = &mut views[i].1.jobs;
            let mut quad = |texture: Option<AssetId<Image>>,
                            min: Vec2,
                            max: Vec2,
                            uv: [Vec2; 2],
                            c: Color32| {
                if jobs.last().is_none_or(|j| j.texture != texture) {
                    jobs.push(EguiJob {
                        clip: [0.0, 0.0, w, h],
                        vertices: Vec::new(),
                        indices: Vec::new(),
                        texture,
                    });
                }
                let job = jobs.last_mut().expect("a job");
                let n = job.vertices.len() as u32;
                for (p, t) in [
                    (min, uv[0]),
                    (Vec2::new(max.x, min.y), Vec2::new(uv[1].x, uv[0].y)),
                    (max, uv[1]),
                    (Vec2::new(min.x, max.y), Vec2::new(uv[0].x, uv[1].y)),
                ] {
                    job.vertices.push(Vertex {
                        pos: Pos2::new(p.x, p.y),
                        uv: Pos2::new(t.x, t.y),
                        color: c,
                    });
                }
                job.indices.extend([n, n + 1, n + 2, n, n + 2, n + 3]);
            };
            let centre = transform.translation;
            let half = node.size() * 0.5;
            let none = [Vec2::ZERO, Vec2::ZERO];
            if let Some(b) = background
                && b.0.alpha() > 0.0
            {
                quad(None, centre - half, centre + half, none, colour(b.0));
            }
            if let Some(o) = outline
                && node.outline_width > 0.0
            {
                let (t, c) = (node.outline_width, colour(o.color));
                let outer = half + Vec2::splat(node.outline_offset + t);
                let inner = outer - Vec2::splat(t);
                let (lo, hi) = (centre - outer, centre + outer);
                quad(None, lo, Vec2::new(hi.x, lo.y + t), none, c);
                quad(None, Vec2::new(lo.x, hi.y - t), hi, none, c);
                quad(
                    None,
                    Vec2::new(lo.x, centre.y - inner.y),
                    Vec2::new(lo.x + t, centre.y + inner.y),
                    none,
                    c,
                );
                quad(
                    None,
                    Vec2::new(hi.x - t, centre.y - inner.y),
                    Vec2::new(hi.x, centre.y + inner.y),
                    none,
                    c,
                );
            }
            if let Some(text) = text {
                let c = colour(text_colour.map_or(Color::WHITE, |t| t.0));
                let origin = centre + node.content_box().min;
                for g in &text.glyphs {
                    let Some(atlas) = images.get(g.atlas_info.texture) else { continue };
                    let size = Vec2::new(atlas.width() as f32, atlas.height() as f32);
                    let r = g.atlas_info.rect;
                    let c = if g.atlas_info.is_alpha_mask { c } else { Color32::WHITE };
                    let p = origin + g.position;
                    let half = r.size() * 0.5;
                    self.send_image(images, g.atlas_info.texture);
                    quad(
                        Some(g.atlas_info.texture),
                        p - half,
                        p + half,
                        [r.min / size, r.max / size],
                        c,
                    );
                }
            }
        }
        list.egui.extend(views.into_iter().map(|(_, v)| v));
    }

    /// The world's meshes seen with `cull`: the palette ones and the effects.
    fn world_into(&mut self, q: &mut Queries, world: &World, cull: &Cull, list: &mut DrawList) {
        let meshes = world.resource::<Assets<Mesh>>();
        let materials = world.resource::<Assets<PaletteMaterial>>();
        for (common, material, tag) in q.palette.iter(world) {
            match cull.seen(&common) {
                None => continue,
                Some(false) => {
                    list.culled += 1;
                    continue;
                }
                Some(true) => {}
            }
            let Some(m) = materials.get(&material.0) else { continue };
            let flags = m.params.flags;
            let id = common.0.0.id();
            if !self.send_mesh(meshes, id) {
                continue;
            }
            let item = Item {
                mesh: id,
                model: common.1.to_matrix(),
                tag: tag.map_or(0, |t| t.0),
                word: m.params.word,
                flags,
            };
            if flags & FLAG_CLOUD != 0 {
                list.clouds.push(item);
            } else if flags & FLAG_DOT != 0 {
                list.dots.push(item);
            } else {
                list.items.push(item);
            }
        }
        for (common, _) in q.particles.iter(world) {
            self.effect_item(cull, list, meshes, &common, Effect::Particle, [0; 8]);
        }
        let materials = world.resource::<Assets<TrailMaterial>>();
        for (common, material) in q.trails.iter(world) {
            let Some(m) = materials.get(&material.0) else { continue };
            self.effect_item(cull, list, meshes, &common, Effect::Trail, vec4_raw(m.params));
        }
        let materials = world.resource::<Assets<RotorMaterial>>();
        for (common, material) in q.rotors.iter(world) {
            let Some(m) = materials.get(&material.0) else { continue };
            let p = &m.params;
            let raw = [
                p.colour,
                p.blades,
                p.hub.to_bits(),
                p.chord.to_bits(),
                p.blur.to_bits(),
                p.axis,
                0,
                0,
            ];
            self.effect_item(cull, list, meshes, &common, Effect::Rotor, raw);
        }
        let materials = world.resource::<Assets<BoltMaterial>>();
        for (common, material) in q.bolts.iter(world) {
            let Some(m) = materials.get(&material.0) else { continue };
            self.effect_item(cull, list, meshes, &common, Effect::Bolt, vec4_raw(m.params));
        }
        let materials = world.resource::<Assets<FlameMaterial>>();
        for (common, material) in q.flames.iter(world) {
            let Some(m) = materials.get(&material.0) else { continue };
            self.effect_item(cull, list, meshes, &common, Effect::Flame, vec4_raw(m.params));
        }
        let materials = world.resource::<Assets<BeamMaterial>>();
        for (common, material) in q.beams.iter(world) {
            let Some(m) = materials.get(&material.0) else { continue };
            self.effect_item(cull, list, meshes, &common, Effect::Beam, vec4_raw(m.params));
        }
    }

    /// The textured meshes (the cockpit and others) and the HUD glass seen with `cull`.
    fn cockpit_into(&mut self, q: &mut Queries, world: &World, cull: &Cull, list: &mut DrawList) {
        let meshes = world.resource::<Assets<Mesh>>();
        let images = world.resource::<Assets<Image>>();
        let materials = world.resource::<Assets<CockpitMaterial>>();
        for (common, material) in q.cockpit.iter(world) {
            let Some(m) = materials.get(&material.0) else { continue };
            let bright = if m.extension.bright { 8 } else { 0 };
            let lit = Lit {
                m: &m.base,
                night: m.extension.night,
                flood: m.extension.flood,
                flags: bright,
            };
            self.lit_item(cull, list, meshes, images, &common, lit);
        }
        let materials = world.resource::<Assets<StandardMaterial>>();
        for (common, material) in q.standard.iter(world) {
            let Some(m) = materials.get(&material.0) else { continue };
            let lit = Lit { m, night: Vec4::ZERO, flood: Vec4::ZERO, flags: 0 };
            self.lit_item(cull, list, meshes, images, &common, lit);
        }
        // The cockpit tooltip's screen (VR), drawn over everything.
        let materials = world.resource::<Assets<crate::cockpit3d::tooltips::TipScreenMaterial>>();
        for (common, material) in q.tips.iter(world) {
            let Some(m) = materials.get(&material.0) else { continue };
            let lit = Lit { m: &m.base, night: Vec4::ZERO, flood: Vec4::ZERO, flags: ON_TOP };
            self.lit_item(cull, list, meshes, images, &common, lit);
        }
        let materials = world.resource::<Assets<HudGlassMaterial>>();
        for (common, material) in q.hud.iter(world) {
            let Some(m) = materials.get(&material.0) else { continue };
            let glass = (m.tint, m.proj, m.style, m.image.id());
            self.glass_item(cull, list, meshes, images, &common, glass);
        }
        let materials = world.resource::<Assets<HmdMaterial>>();
        for (common, material) in q.hmd.iter(world) {
            let Some(m) = materials.get(&material.0) else { continue };
            let glass = (m.tint, m.proj, m.style, m.image.id());
            self.glass_item(cull, list, meshes, images, &common, glass);
        }
    }

    fn effect_item(
        &mut self,
        cull: &Cull,
        list: &mut DrawList,
        meshes: &Assets<Mesh>,
        common: &CommonItem,
        effect: Effect,
        raw: [u32; 8],
    ) {
        if cull.seen(common) != Some(true) {
            return;
        }
        let id = common.0.0.id();
        if !self.send_mesh_with(meshes, id, effect.attributes()) {
            return;
        }
        list.effects.push(EffectItem { effect, mesh: id, model: common.1.to_matrix(), raw });
    }

    fn glass_item(
        &mut self,
        cull: &Cull,
        list: &mut DrawList,
        meshes: &Assets<Mesh>,
        images: &Assets<Image>,
        common: &CommonItem,
        (tint, proj, style, texture): (Vec4, Vec4, Vec4, AssetId<Image>),
    ) {
        if cull.seen(common) != Some(true) {
            return;
        }
        let id = common.0.0.id();
        if !self.send_mesh(meshes, id) {
            return;
        }
        self.send_image(images, texture);
        list.glass.push(GlassItem {
            mesh: id,
            model: common.1.to_matrix(),
            tint,
            proj,
            style,
            texture,
        });
    }

    fn lit_item(
        &mut self,
        cull: &Cull,
        list: &mut DrawList,
        meshes: &Assets<Mesh>,
        images: &Assets<Image>,
        common: &CommonItem,
        lit: Lit,
    ) {
        match cull.seen(common) {
            None => return,
            Some(false) => {
                list.culled += 1;
                return;
            }
            Some(true) => {}
        }
        let id = common.0.0.id();
        if !self.send_mesh(meshes, id) {
            return;
        }
        let m = lit.m;
        let base_texture = m.base_color_texture.as_ref().map(|h| h.id());
        let emissive_texture = m.emissive_texture.as_ref().map(|h| h.id());
        for t in [base_texture, emissive_texture].into_iter().flatten() {
            self.send_image(images, t);
        }
        let (blend, mask, cutoff) = match m.alpha_mode {
            AlphaMode::Opaque => (false, false, 0.0),
            AlphaMode::Mask(c) => (false, true, c),
            AlphaMode::AlphaToCoverage => (false, true, 0.5),
            _ => (true, false, 0.0),
        };
        list.lit.push(LitItem {
            mesh: id,
            model: common.1.to_matrix(),
            base: Vec4::from_array(m.base_color.to_linear().to_f32_array()),
            emissive: Vec4::from_array(m.emissive.to_f32_array()),
            base_texture,
            emissive_texture,
            flags: lit.flags | if m.unlit { 4 } else { 0 } | if mask { 16 } else { 0 },
            cutoff,
            night: lit.night,
            flood: lit.flood,
            blend,
            double_sided: m.double_sided || m.cull_mode.is_none(),
        });
    }
}

/// A textured mesh's material: Bevy's, the cockpit's night and lit.glsl flags.
struct Lit<'a> {
    m: &'a StandardMaterial,
    night: Vec4,
    flood: Vec4,
    flags: u32,
}

/// The clouds' frame packed for the shader (clouds.glsl), when it has volumes.
fn clouds(world: &World) -> Option<Clouds> {
    let f = world.get_resource::<bevy_fast_clouds::render::CloudFrame>()?;
    if f.volumes.is_empty() {
        return None;
    }
    let g = &f.globals;
    let mut globals: Vec<f32> =
        [g.sun, g.params0, g.params1, g.params2, g.flash, g.line_a, g.line_b, g.line_color, g.fog]
            .iter()
            .flat_map(|v| v.to_array())
            .collect();
    globals.extend(g.ramp.iter().flat_map(|v| v.to_array()));
    let volumes = f
        .volumes
        .iter()
        .flat_map(|v| {
            let mut w = [0u32; 16];
            for (i, x) in v.aabb_min.to_array().into_iter().chain([v.j_max]).enumerate() {
                w[i] = x.to_bits();
            }
            for (i, x) in v.aabb_max.to_array().into_iter().chain([v.h_max]).enumerate() {
                w[4 + i] = x.to_bits();
            }
            w[8..12].copy_from_slice(&v.misc.to_array());
            for (i, x) in v.fill.to_array().into_iter().enumerate() {
                w[12 + i] = x.to_bits();
            }
            w
        })
        .collect();
    // (Never empty: the shader indexes it.)
    let mut spheres: Vec<f32> = f.spheres.iter().flat_map(|s| s.to_array()).collect();
    if spheres.is_empty() {
        spheres.extend([0.0; 4]);
    }
    Some(Clouds {
        globals,
        volumes,
        spheres,
        batches: f.batches.map(|b| (b.x, b.y)),
        steps: f.steps,
        dither: f.dither,
        sun_shadow: f.sun_shadow,
        scale: f.resolution_scale.max(1),
    })
}

/// A vec4 uniform as its bytes.
fn vec4_raw(v: Vec4) -> [u32; 8] {
    let [x, y, z, w] = v.to_array().map(f32::to_bits);
    [x, y, z, w, 0, 0, 0, 0]
}

/// The float attributes `which` of `mesh` at their shader locations.
fn extra_attributes(
    mesh: &Mesh,
    which: &[(u32, MeshVertexAttribute)],
) -> Vec<(u32, i32, Vec<f32>)> {
    which
        .iter()
        .filter_map(|&(location, attribute)| {
            let (n, data): (i32, Vec<f32>) = match mesh.attribute(attribute)? {
                VertexAttributeValues::Float32x2(v) => (2, v.as_flattened().to_vec()),
                VertexAttributeValues::Float32x3(v) => (3, v.as_flattened().to_vec()),
                VertexAttributeValues::Float32x4(v) => (4, v.as_flattened().to_vec()),
                _ => return None,
            };
            Some((location, n, data))
        })
        .collect()
}

/// What the render thread needs of `mesh`: positions, normals, UVs, indices, topology.
fn mesh_data(mesh: &Mesh) -> Option<MeshData> {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return None;
    };
    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(n)) => Some(n.clone()),
        _ => None,
    };
    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(uv)) => Some(uv.clone()),
        _ => None,
    };
    let colours = match mesh.attribute(crate::dots::ATTRIBUTE_DOT_COLOUR) {
        Some(VertexAttributeValues::Uint32(c)) => Some(c.clone()),
        _ => None,
    };
    let words = match mesh.attribute(crate::palette_material::ATTRIBUTE_PALETTE_WORD) {
        Some(VertexAttributeValues::Uint32(w)) => Some(w.clone()),
        _ => None,
    };
    Some(MeshData {
        positions: positions.clone(),
        normals,
        uvs,
        words,
        colours,
        extra: Vec::new(),
        indices: mesh.indices().cloned(),
        topology: mesh.primitive_topology(),
    })
}

/// What the render thread needs of `image`: RGBA8 and R8 images only so far.
fn texture_data(image: &Image) -> Option<TextureData> {
    let format = image.texture_descriptor.format;
    let srgb = match format {
        TextureFormat::Rgba8UnormSrgb => true,
        TextureFormat::Rgba8Unorm | TextureFormat::R8Unorm => false,
        other => {
            warn!("glvr: images of format {other:?} aren't drawn yet");
            return None;
        }
    };
    let (nearest, anisotropy) = match &image.sampler {
        ImageSampler::Descriptor(d) => {
            (d.mag_filter == ImageFilterMode::Nearest, d.anisotropy_clamp)
        }
        ImageSampler::Default => (false, 1),
    };
    Some(TextureData {
        size: (image.width(), image.height()),
        levels: image.texture_descriptor.mip_level_count.max(1),
        data: image.data.clone()?,
        srgb,
        one_channel: format == TextureFormat::R8Unorm,
        nearest,
        anisotropy,
    })
}

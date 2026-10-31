//! The menus' preview of the chosen aircraft: its model turning slowly in a box above the
//! VEHICLE choices. An offscreen camera draws the model, alone on its own layer far below the
//! menu's scene, into a picture egui shows; both exist only while a menu shows the preview.
//! The model is drawn in daylight whatever the menu's scene: its materials are twins of the
//! scene's that read shader globals of their own (the theater's day palette, a high sun).

use bevy::camera::RenderTarget;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use bevy_egui::egui;

use crate::aircraft::AircraftKind;
use crate::camera::{OffscreenCamera, PREVIEW_LAYER};
use crate::models::{LodBias, LodMesh, ModelAssets, ModelLibrary};
use crate::palette::Palette;
use crate::palette_material::{Globals, PaletteMaterial};
use crate::rotor_blur::RotorMaterial;

/// Twins of materials, by the material they were made from.
type Twins<M> = std::collections::HashMap<AssetId<M>, Handle<M>>;
use crate::settings::{AppState, Settings};

/// Picture size (pixels).
const W: u32 = 520;
const H: u32 = 260;
/// Where the model turns: far below the menu's scene, out of its camera's sight.
const AT: Vec3 = Vec3::new(0.0, -30_000.0, 0.0);
/// Vertical field of view (degrees), turn rate (radians a second), and the camera's height
/// above the model (radians).
const FOV: f32 = 30.0;
const TURN: f32 = 0.5;
const PITCH: f32 = 0.3;
/// The share of the picture the turning model fills at most (in width or height).
const FILL: f32 = 1.0;
/// Turns sampled for the framing, and the height of its slices (metres).
const SAMPLES: usize = 24;
const SLICE: f32 = 0.25;

pub struct AircraftPreviewPlugin;

impl Plugin for AircraftPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AircraftPreview>()
            .add_systems(Update, update.run_if(resource_exists::<crate::palette::Palette>))
            .add_systems(OnExit(AppState::Menu), cleanup);
    }
}

/// The preview's picture and what draws it.
#[derive(Resource, Default)]
pub struct AircraftPreview {
    image: Option<Handle<Image>>,
    /// The picture as an egui texture (`menu_ui` registers it).
    pub(crate) texture: Option<egui::TextureId>,
    /// A menu showed the preview since the last update.
    requested: bool,
    camera: Option<Entity>,
    model: Option<(AircraftKind, Entity)>,
    /// How far from its upright axis the model's meshes reached so far, by height (`SLICE`
    /// slices, in its own frame): turning rotors sweep their discs into it, so the framing
    /// settles instead of breathing with the blades.
    profile: std::collections::BTreeMap<i32, f32>,
    /// The daylight shader globals, and the scene's materials' twins reading them.
    globals: Option<Handle<ShaderBuffer>>,
    day: Twins<PaletteMaterial>,
    day_rotors: Twins<RotorMaterial>,
    yaw: f32,
}

impl AircraftPreview {
    pub(crate) fn image(&self) -> Option<Handle<Image>> {
        self.image.clone()
    }
}

/// The marker of the turning model's root.
#[derive(Component)]
struct PreviewModel;

/// A mesh of the model drawn with its daylight twin material.
#[derive(Component)]
struct DayLit;

/// The twin of material `m` reading the shader globals `globals` (made once).
fn twin<M: Asset + Clone>(
    m: &Handle<M>,
    twins: &mut Twins<M>,
    materials: &mut Assets<M>,
    globals: impl FnOnce(&mut M),
) -> Option<Handle<M>> {
    if let Some(t) = twins.get(&m.id()) {
        return Some(t.clone());
    }
    let mut twin = materials.get(m)?.clone();
    globals(&mut twin);
    let twin = materials.add(twin);
    twins.insert(m.id(), twin.clone());
    Some(twin)
}

/// Shader globals of a clear day: the theater's day palette lit as by the classic shading's
/// sun, without shadows, haze, clouds or the night sky.
fn day_globals(palette: &Palette, settings: &Settings) -> Globals {
    let mut g = Globals::default();
    for i in 0..256 {
        let c = palette.theater.day.color(i as u8).to_linear();
        g.palette[i] = Vec4::new(c.red, c.green, c.blue, 1.0);
        g.recon_palette[i] = g.palette[i];
    }
    g.params = Vec4::new(0.0, crate::world::WU as f32, 0.0, 0.0);
    // As `lighting::update_sun` at noon without the modern balance.
    let sun = 0.45;
    g.sun = crate::lighting::sun_direction().extend(sun);
    g.ambient = Vec4::new(1.0 - sun * 0.6, 1.0 - sun * 0.6, 1.0 - sun * 0.55, 0.0);
    g.light_info.y = 1.0;
    g.dots.x = if settings.vr { crate::dots::XR_ANGLE } else { 0.0 };
    g.retro = crate::palette::retro(settings);
    g
}

/// The preview across the column, a rule under it (blank until its picture is drawn).
pub(crate) fn show(ui: &mut egui::Ui, preview: &mut AircraftPreview) {
    preview.requested = true;
    let w = ui.available_width();
    let size = egui::vec2(w, w * H as f32 / W as f32);
    match preview.texture {
        Some(tex) => {
            ui.image((tex, size));
        }
        None => {
            ui.allocate_exact_size(size, egui::Sense::hover());
        }
    }
    ui.separator();
}

fn despawn(commands: &mut Commands, preview: &mut AircraftPreview) {
    let model = preview.model.take().map(|(_, e)| e);
    for e in [preview.camera.take(), model].into_iter().flatten() {
        commands.entity(e).try_despawn();
    }
}

fn cleanup(mut commands: Commands, mut preview: ResMut<AircraftPreview>) {
    despawn(&mut commands, &mut preview);
    preview.requested = false;
    // (The scene's palette, and with it its materials, is made anew with the next menu.)
    preview.day.clear();
    preview.day_rotors.clear();
}

/// Spawns the camera and the chosen aircraft's model while a menu shows the preview (despawns
/// them when none does), turns the model and keeps it framed.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update(
    mut commands: Commands,
    mut preview: ResMut<AircraftPreview>,
    settings: Res<Settings>,
    mut library: ResMut<ModelLibrary>,
    server: Res<AssetServer>,
    assets: ModelAssets,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time<Real>>,
    children: Query<&Children>,
    meshes: Query<Option<&RenderLayers>, With<Mesh3d>>,
    bounds: Query<(&Aabb, &GlobalTransform, &InheritedVisibility)>,
    mut roots: Query<(&mut Transform, &GlobalTransform), With<PreviewModel>>,
    mut cams: Query<&mut Transform, (With<Camera>, Without<PreviewModel>)>,
    (mut buffers, mut materials, mut rotor_materials, lit, rotors): (
        ResMut<Assets<ShaderBuffer>>,
        ResMut<Assets<PaletteMaterial>>,
        ResMut<Assets<RotorMaterial>>,
        Query<&MeshMaterial3d<PaletteMaterial>, Without<DayLit>>,
        Query<&MeshMaterial3d<RotorMaterial>, Without<DayLit>>,
    ),
) {
    let preview = &mut *preview;
    if !std::mem::take(&mut preview.requested) {
        despawn(&mut commands, preview);
        return;
    }
    let image = preview
        .image
        .get_or_insert_with(|| images.add(crate::camera::offscreen_target(W, H)))
        .clone();
    if preview.camera.is_none() {
        let [r, g, b, _] = crate::menu::FIELD.to_array();
        preview.camera = Some(
            commands
                .spawn((
                    OffscreenCamera,
                    Camera3d::default(),
                    Camera {
                        order: -4,
                        clear_color: ClearColorConfig::Custom(Color::srgb_u8(r, g, b)),
                        ..default()
                    },
                    RenderTarget::Image(image.into()),
                    Projection::Perspective(PerspectiveProjection {
                        fov: FOV.to_radians(),
                        aspect_ratio: W as f32 / H as f32,
                        near: 0.5,
                        far: 1000.0,
                        ..default()
                    }),
                    Tonemapping::None,
                    Msaa::Off,
                    Transform::from_translation(AT + Vec3::Z * 30.0),
                    RenderLayers::layer(PREVIEW_LAYER),
                ))
                .id(),
        );
        if let (Some(c), Some(g)) = (preview.camera, &preview.globals) {
            commands.entity(c).insert(crate::camera::OffscreenGlobals(g.clone()));
        }
    }
    // The chosen aircraft (again when another is chosen).
    let kind = settings.aircraft;
    if preview.model.is_some_and(|(k, _)| k != kind) {
        let (_, e) = preview.model.take().expect("checked above");
        commands.entity(e).try_despawn();
    }
    if preview.model.is_none() {
        let spec = kind.spec();
        let file = library.load(&server, "aircraft");
        let Some(model) = assets.spawn(
            &mut commands,
            &file,
            spec.model,
            crate::palette_material::LEVEL_AIRCRAFT,
            true,
        ) else {
            return;
        };
        let scale = crate::player::AIRCRAFT_MODEL_SCALE as f32;
        // Its finest detail level, however far the menu's camera is.
        commands.entity(model).insert((
            Transform::from_xyz(0.0, spec.model_lift, 0.0).with_scale(Vec3::splat(scale)),
            LodBias(1e9),
        ));
        let root = commands
            .spawn((
                PreviewModel,
                Transform::from_translation(AT).with_rotation(Quat::from_rotation_y(preview.yaw)),
                Visibility::default(),
            ))
            .add_child(model)
            .id();
        preview.model = Some((kind, root));
        preview.profile.clear();
        return;
    }
    let Some((_, root)) = preview.model else { return };
    let Ok((mut tf, global)) = roots.get_mut(root) else { return };
    // Its meshes on the preview's layer, kept there (`apply_model_state` moves a detail level's
    // meshes between the main and the tracking camera's layers).
    let layer = RenderLayers::layer(PREVIEW_LAYER);
    for e in children.iter_descendants(root) {
        if let Ok(layers) = meshes.get(e)
            && layers != Some(&layer)
        {
            commands.entity(e).remove::<LodMesh>().insert(layer.clone());
        }
    }
    // Lit by day.
    let fresh = preview.globals.is_none();
    let globals = preview
        .globals
        .get_or_insert_with(|| {
            // Bound as the scene's are (`Palette::new`).
            let mut b = ShaderBuffer::from(Globals::default());
            b.buffer_description.usage |= bevy::render::render_resource::BufferUsages::UNIFORM;
            buffers.add(b)
        })
        .clone();
    if (fresh || settings.is_changed())
        && let Some(mut b) = buffers.get_mut(&globals)
    {
        b.set_data(day_globals(&assets.palette, &settings));
    }
    if fresh && let Some(c) = preview.camera {
        commands.entity(c).insert(crate::camera::OffscreenGlobals(globals.clone()));
    }
    for e in children.iter_descendants(root) {
        if let Ok(m) = lit.get(e)
            && let Some(t) =
                twin(&m.0, &mut preview.day, &mut materials, |t| t.globals = globals.clone())
        {
            commands.entity(e).insert((MeshMaterial3d(t), DayLit));
        } else if let Ok(m) = rotors.get(e)
            && let Some(t) = twin(&m.0, &mut preview.day_rotors, &mut rotor_materials, |t| {
                t.globals = globals.clone()
            })
        {
            commands.entity(e).insert((MeshMaterial3d(t), DayLit));
        }
    }
    preview.yaw = (preview.yaw + TURN * time.delta_secs()) % std::f32::consts::TAU;
    tf.rotation = Quat::from_rotation_y(preview.yaw);
    // Framed by its shown meshes' boxes, in the root's own frame (the same however it turns).
    let to_root = global.affine().inverse();
    for e in children.iter_descendants(root) {
        let Ok((aabb, mesh, vis)) = bounds.get(e) else { continue };
        if !vis.get() {
            continue;
        }
        let (c, h) = (Vec3::from(aabb.center), Vec3::from(aabb.half_extents));
        for i in 0..8 {
            let corner =
                Vec3::new([-1.0, 1.0][i & 1], [-1.0, 1.0][i >> 1 & 1], [-1.0, 1.0][i >> 2]);
            let p = to_root.transform_point3(mesh.transform_point(c + h * corner));
            let r = preview.profile.entry((p.y / SLICE).floor() as i32).or_default();
            *r = r.max(p.xz().length());
        }
    }
    if preview.profile.is_empty() {
        return;
    }
    let Some(Ok(mut cam)) = preview.camera.map(|c| cams.get_mut(c)) else { return };
    let (target, eye, aim) = framing(&preview.profile);
    *cam = Transform::from_translation(AT + eye).looking_at(AT + target, Vec3::Y);
    // Up or down to the middle of the picture.
    cam.rotate_local_x(aim);
}

/// Where the camera looks at a model turning round its upright axis, reaching `profile` from
/// it (by height), and from where, both from the model's position; and how far it then tilts
/// up so the model, at every turn, fills the picture centred. Each slice's reach is swept round
/// the axis and the angles it reaches seen from the camera measured: the camera goes closer
/// till they fill the picture, and tilts to their middle.
fn framing(profile: &std::collections::BTreeMap<i32, f32>) -> (Vec3, Vec3, f32) {
    let circles: Vec<(f32, f32)> = profile
        .iter()
        .flat_map(|(&k, &r)| [(k as f32 * SLICE, r), ((k + 1) as f32 * SLICE, r)])
        .collect();
    let (bottom, top) = (circles[0].0, circles[circles.len() - 1].0);
    let reach = circles.iter().map(|c| c.1).fold(0.0, f32::max);
    let target = Vec3::new(0.0, (bottom + top) / 2.0, 0.0);
    let back = Vec3::new(0.0, PITCH.sin(), PITCH.cos());
    let (fwd, up) = (-back, Vec3::new(0.0, PITCH.cos(), -PITCH.sin()));
    let points: Vec<Vec3> = (0..SAMPLES)
        .map(|k| k as f32 / SAMPLES as f32 * std::f32::consts::TAU)
        .flat_map(|a| circles.iter().map(move |&(y, r)| Vec3::new(r * a.sin(), y, r * a.cos())))
        .collect();
    // The lowest and highest angle above the view's axis, and the widest to either side.
    let angles = |distance: f32| {
        let eye = target + back * distance;
        let (mut low, mut high, mut side) = (f32::MAX, f32::MIN, 0f32);
        for &p in &points {
            let v = p - eye;
            let depth = v.dot(fwd).max(0.1);
            let a = (v.dot(up) / depth).atan();
            (low, high) = (low.min(a), high.max(a));
            side = side.max((v.x / depth).atan().abs());
        }
        (low, high, side)
    };
    let fov = FOV.to_radians();
    let wide = 2.0 * ((fov / 2.0).tan() * W as f32 / H as f32).atan();
    // From well outside it, closer while the model looks smaller than the picture (the angles
    // shrink about as the distance grows).
    let mut distance = 2.0 * reach.max(top - bottom) / (fov / 2.0).tan();
    for _ in 0..4 {
        let (low, high, side) = angles(distance);
        distance *= ((high - low) / fov).max(2.0 * side / wide) / FILL;
    }
    let (low, high, _) = angles(distance);
    (target, target + back * distance, (low + high) / 2.0)
}

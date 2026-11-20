//! A camera picture for the scenes' screens: a targeting pod's, a missile's seeker, a drone's
//! camera (rendered to an image the cockpit's or console's screen shows), and the symbols
//! drawn over it.

use bevy::camera::RenderTarget;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::ShotEntity;
use crate::canvas::Canvas;
use crate::font::stroke_text;
use crate::world::WorldPos;

/// The pod camera of a scene.
#[derive(Component)]
pub struct PodCamera;

/// A scene's thing (a cabin, a console) wants a camera picture: its size (pixels) and field
/// of view (radians high); `fit_pods` makes it and puts `PodImage` there.
#[derive(Component)]
pub struct WantsPod(pub (u32, u32), pub f32);

/// The picture made for it.
#[derive(Component)]
pub struct PodImage(pub Handle<Image>);

/// Makes the pictures wanted.
pub fn fit_pods(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    palette: Res<crate::palette::Palette>,
    wants: Query<(Entity, &WantsPod), Without<PodImage>>,
) {
    for (e, w) in &wants {
        let (_, image) = spawn_pod(&mut commands, &mut images, &mut meshes, &palette, w.0, w.1);
        commands.entity(e).insert(PodImage(image));
    }
}

/// Spawns a camera rendering `w` x `h` pixels into a new image (the scene's), looking with a
/// vertical field of view of `fov` radians; the camera and its image.
pub fn spawn_pod(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    palette: &crate::palette::Palette,
    (w, h): (u32, u32),
    fov: f32,
) -> (Entity, Handle<Image>) {
    let image = images.add(crate::camera::offscreen_target(w, h));
    let cam = commands
        .spawn((
            PodCamera,
            ShotEntity,
            crate::camera::OffscreenCamera,
            Camera3d::default(),
            Camera { order: -1, clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() },
            RenderTarget::Image(image.clone().into()),
            Projection::Perspective(PerspectiveProjection {
                fov,
                near: 0.5,
                far: 600_000.0,
                aspect_ratio: w as f32 / h as f32,
                ..default()
            }),
            Tonemapping::None,
            Msaa::Off,
            WorldPos::default(),
            Transform::default(),
            bevy::camera::visibility::RenderLayers::from_layers(&[
                0,
                crate::camera::TRACKCAM_BACKGROUND_LAYER,
                crate::camera::MAIN_LOD_LAYER,
            ]),
        ))
        .id();
    crate::camera::spawn_background(
        commands,
        cam,
        meshes,
        palette,
        crate::camera::TRACKCAM_BACKGROUND_LAYER,
    );
    (cam, image)
}

/// The scenes' pod cameras, to aim.
pub type Pods<'w, 's> = Query<
    'w,
    's,
    (&'static mut WorldPos, &'static mut Transform),
    (With<PodCamera>, Without<super::SceneCamera>),
>;

/// Points a pod camera from `from` at `at` (Bevy metres).
pub fn aim_pod(q: &mut Pods, from: DVec3, at: DVec3) {
    for (mut p, mut tf) in q.iter_mut() {
        p.0 = from;
        let dir = (at - from).as_vec3().normalize_or(Vec3::NEG_Z);
        tf.rotation = Transform::IDENTITY.looking_to(dir, Vec3::Y).rotation;
    }
}

/// The camera overlay's ink (the HUD's green, as the MFDs' camera pages).
pub const INK: u8 = crate::hud::HUD_COLOURS.0;
/// The MFD's bezel band (drawing units).
const BAND: f32 = 16.0;

/// Text at a place.
fn text(c: &mut Canvas, x: f32, y: f32, s: &str, size: f32) {
    stroke_text(c, x, y, s, size, 1.0, INK);
}

/// Text ending at `x`.
fn text_right(c: &mut Canvas, x: f32, y: f32, s: &str, size: f32) {
    let w = crate::font::stroke_width(s, size);
    text(c, x - w, y, s, size);
}

/// A seeker's or pod's overlay over its picture (the canvas left see-through): the crosshair,
/// a box once tracking, the title and mode at the top, the range and a time or count below,
/// the sensor's name (TV, FLIR) over them.
pub fn overlay(
    c: &mut Canvas,
    title: &str,
    mode: &str,
    locked: bool,
    bottom: (&str, &str),
    sensor: &str,
) {
    c.clear(0);
    let (w, h) = (c.w as f32, c.h as f32);
    let o = Vec2::new(w / 2.0, h / 2.0);
    for d in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
        c.stroke(o + d * 5.0, o + d * 24.0, 1.3, INK);
    }
    if locked {
        c.frame(o - Vec2::splat(9.0), o + Vec2::splat(9.0), 1.3, INK);
    }
    let y0 = BAND + 4.0;
    text(c, 40.0, y0, title, 7.0);
    text_right(c, w - 40.0, y0, mode, 7.0);
    text(c, 40.0, h - BAND - 14.0, bottom.0, 7.0);
    text_right(c, w - 40.0, h - BAND - 14.0, bottom.1, 7.0);
    text(c, 40.0, h - BAND - 24.0, sensor, 6.0);
}

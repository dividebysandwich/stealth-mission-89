//! Right-MFD tracking camera: a second 3D camera at the aircraft looking at the locked target,
//! or along the camera direction keys (`/ , . m`) with the photo lens depression, rendered into
//! a small texture with the focal length 256 times the zoom over the 77x60 MFD rectangle.
//! The cockpit shows the texture behind the right MFD screen (the page's text is drawn over it);
//! it is rendered at the screen's resolution (2x the MFD's pixels for the classic panel, 6x for the
//! modern one's 231x180 display).

use bevy::camera::RenderTarget;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::image::ImageSampler;
use bevy::prelude::*;

use crate::camera::{View, ViewState};
use crate::combat::{Combat, CombatSystems, map_wu, remote_motion, unit_position};
use crate::flight::{Aircraft, Player, bam};
use crate::mfd::{CamDir, MfdState, TrackCam};
use crate::settings::{AppState, SetupSet};
use crate::world::{WorldPos, wu_to_bevy};

pub struct TrackCamPlugin;

impl Plugin for TrackCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, flir_background.run_if(in_state(AppState::Flight)))
            .add_systems(OnEnter(AppState::Flight), spawn_camera.in_set(SetupSet::Spawn))
            .add_systems(OnExit(AppState::Flight), despawn_camera)
            .add_systems(
                Update,
                update_camera.after(CombatSystems).run_if(in_state(AppState::Flight)),
            );
    }
}

/// Render scale over the classic MFD pixels (77x60).
fn scale(settings: &crate::settings::Settings) -> u32 {
    if settings.hires_displays() { 6 } else { 2 }
}
/// Photo lens depression below the nose (0x6EF BAM).
const LENS_DEPRESSION: f32 = 0x6EF as f32;

#[derive(Component)]
pub struct TrackCamera;

/// The texture the tracking camera renders into.
#[derive(Resource, Clone)]
pub struct TrackCamImage(pub Handle<Image>);

/// The texture it renders into at a drone command center (`WIDE`).
#[derive(Resource, Clone)]
pub struct TrackCamWide(pub Handle<Image>);

fn create_image(images: &mut Assets<Image>, scale: u32) -> Handle<Image> {
    let mut image = crate::camera::offscreen_target(77 * scale, 60 * scale);
    image.sampler = ImageSampler::nearest();
    images.add(image)
}

/// The drone command center's wide picture (`station::drone`): its size in pixels at f = 256,
/// square pixels.
pub const WIDE: (f32, f32) = (180.0, 76.0);

/// The wide picture's projection for zoom step z: f = 256 times its magnification in pixels.
fn wide_projection(z: u8) -> PerspectiveProjection {
    let f = 256.0 * crate::mfd::cam_magnification(z);
    let (half_w, half_h) = (WIDE.0 / 2.0 / f, WIDE.1 / 2.0 / f);
    PerspectiveProjection {
        fov: 2.0 * half_h.atan(),
        aspect_ratio: half_w / half_h,
        near: 20.0,
        far: 600_000.0,
        ..default()
    }
}

/// Projection for zoom step z: f = 256 times its magnification in pixels, vertical 0.75 f,
/// 77x60 viewport.
fn projection(z: u8) -> PerspectiveProjection {
    let f = 256.0 * crate::mfd::cam_magnification(z);
    let half_h = 30.0 / (0.75 * f);
    let half_w = 38.5 / f;
    PerspectiveProjection {
        fov: 2.0 * half_h.atan(),
        aspect_ratio: half_w / half_h,
        near: 20.0,
        far: 600_000.0,
        ..default()
    }
}

fn spawn_camera(
    mut commands: Commands,
    image: Option<Res<TrackCamImage>>,
    wide: Option<Res<TrackCamWide>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    palette: Option<Res<crate::palette::Palette>>,
    settings: Res<crate::settings::Settings>,
) {
    let scale = scale(&settings);
    let image =
        match image.filter(|i| images.get(&i.0).is_some_and(|img| img.width() == 77 * scale)) {
            Some(i) => i.0.clone(),
            None => {
                let h = create_image(&mut images, scale);
                commands.insert_resource(TrackCamImage(h.clone()));
                h
            }
        };
    let (ww, wh) = (WIDE.0 as u32 * scale, WIDE.1 as u32 * scale);
    if wide.is_none_or(|i| images.get(&i.0).is_none_or(|img| img.width() != ww)) {
        let mut img = crate::camera::offscreen_target(ww, wh);
        img.sampler = ImageSampler::nearest();
        commands.insert_resource(TrackCamWide(images.add(img)));
    }
    let cam = commands
        .spawn((
            TrackCamera,
            crate::camera::OffscreenCamera,
            Camera3d::default(),
            Camera {
                order: -1,
                is_active: false,
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            RenderTarget::Image(image.into()),
            Projection::Perspective(projection(0)),
            Tonemapping::None,
            Msaa::Off,
            WorldPos::default(),
            Transform::default(),
        ))
        .id();
    if let Some(palette) = palette {
        let sphere = crate::camera::spawn_background(
            &mut commands,
            cam,
            &mut meshes,
            &palette,
            crate::camera::TRACKCAM_BACKGROUND_LAYER,
        );
        commands.entity(sphere).insert(TrackCamBackground);
        commands.entity(cam).insert(bevy::camera::visibility::RenderLayers::from_layers(&[
            0,
            crate::camera::TRACKCAM_BACKGROUND_LAYER,
            crate::camera::TRACKCAM_LOD_LAYER,
        ]));
    }
}

fn despawn_camera(mut commands: Commands, q: Query<Entity, With<TrackCamera>>) {
    for e in &q {
        commands.entity(e).try_despawn();
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn update_camera(
    combat: Option<Res<Combat>>,
    mfd: Res<MfdState>,
    view: Res<ViewState>,
    settings: Res<crate::settings::Settings>,
    mut info: ResMut<TrackCam>,
    papi: Res<crate::scenery::PapiSites>,
    mut sites: ResMut<crate::world::StreamSites>,
    player: Query<(&Aircraft, &WorldPos), (With<Player>, Without<TrackCamera>)>,
    mut cam: Query<
        (&mut Camera, &mut Projection, &mut WorldPos, &mut Transform, &mut RenderTarget),
        With<TrackCamera>,
    >,
    console: Option<Res<crate::station::ConsoleEo>>,
    (image, wide_image): (Option<Res<TrackCamImage>>, Option<Res<TrackCamWide>>),
) {
    let Ok((mut camera, mut proj, mut pos, mut tf, mut target)) = cam.single_mut() else {
        return;
    };
    // The drone command center's wide picture into its own image, else the MFD's.
    let wide = console.as_deref().is_some_and(|c| c.wide);
    if let (Some(normal), Some(wide_image)) = (image, wide_image) {
        let want = if wide { &wide_image.0 } else { &normal.0 };
        if !matches!(&*target, RenderTarget::Image(t) if t.handle == *want) {
            *target = RenderTarget::Image(want.clone().into());
        }
    }
    // An air defence console's camera on its mast (`station`).
    if let Some(eo) = console {
        let want = view.view == View::Cockpit && !eo.idle;
        if camera.is_active != want {
            camera.is_active = want;
        }
        info.live = false;
        aim(&mut tf, &mut pos, &mut proj, eo.dir, eo.up, eo.eye, eo.zoom, eo.wide);
        if sites.pod != eo.target {
            sites.pod = eo.target;
        }
        return;
    }
    let ac = player.single().ok();
    let want = mfd.camera_shown() && view.view == View::Cockpit && combat.is_some() && ac.is_some();
    if camera.is_active != want {
        camera.is_active = want;
    }
    info.live = want;
    // The ground looked at is streamed in like around the aircraft, so far-away targets show up.
    let mut look = None;
    let site = |sites: &mut crate::world::StreamSites, p: Option<bevy::math::DVec3>| {
        if sites.pod != p {
            sites.pod = p;
        }
    };
    let (Some(combat), Some((a, apos))) = (combat, ac) else { return site(&mut sites, None) };
    if !want {
        return site(&mut sites, None);
    }
    let eye = apos.0;
    if mfd.ils {
        // ILS: the runway ahead with its PAPI, zoomed just enough to read the lights.
        let (h, _, _) = a.euler();
        match ils_view(&papi.0, eye, h) {
            Some(v) => {
                tf.rotation = Transform::IDENTITY.looking_to(v.dir, Vec3::Y).rotation;
                if pos.0 != eye {
                    pos.0 = eye;
                }
                if let Projection::Perspective(p) = proj.as_mut() {
                    *p = view_projection(v.tan_half_w);
                }
                let magnification = BASE_TAN_HALF_W / v.tan_half_w;
                *info = TrackCam {
                    live: true,
                    name: ["ILS".into(), format!("RWY {:02}", v.runway)],
                    range_km: Some(v.range_m / 1000.0),
                    bearing: None,
                    zoom: crate::mfd::cam_zoom_for(magnification) + 1,
                };
                return site(&mut sites, None);
            }
            None => info.name = ["ILS".into(), "NO APPROACH".into()],
        }
    }
    // Modern avionics: the targeting pod in area or point track.
    if mfd.modern
        && let Some(dir) = crate::avionics::pod_direction(&mfd.av.tgp, a.orientation, eye)
    {
        let range_m = mfd.av.tgp.los_ground.map(|g| (g - eye).length() as f32);
        // AUTO: by the range to a point track or the lock the pod follows (combat set its
        // zoom), else 4X.
        let targeted = match mfd.av.tgp.mode {
            crate::avionics::TgpMode::Point(_) => true,
            crate::avionics::TgpMode::Slaved => info.zoom > 0,
            _ => false,
        };
        let zoom = mfd.cam_zoom.unwrap_or_else(|| match range_m.filter(|_| targeted) {
            Some(r) => crate::combat::camera_zoom((r / (32.0 * crate::world::WU as f32)) as i32),
            None => crate::mfd::CAM_ZOOM_DEFAULT,
        });
        *info = TrackCam {
            live: true,
            name: [
                match mfd.av.tgp.mode {
                    crate::avionics::TgpMode::Point(_) => "POINT",
                    crate::avionics::TgpMode::Snowplow => "SNOWPLOW",
                    _ => "AREA",
                }
                .into(),
                String::new(),
            ],
            range_km: range_m.map(|r| r / 1000.0),
            bearing: None,
            zoom: zoom + 1,
        };
        aim(&mut tf, &mut pos, &mut proj, dir, None, eye, zoom, false);
        return site(&mut sites, mfd.av.tgp.los_ground);
    }
    let sim = &combat.sim;
    let t = sim.targeting();
    let target = if let Some(o) = t.ground.and_then(|g| sim.objects().get(g)) {
        let (e, n) = map_wu(o.x, o.y);
        look = Some(wu_to_bevy(e + 16.0, n - 16.0, 0.0));
        look
    } else if let Some((i, u)) = t.air.and_then(|i| sim.flights().get(i).map(|u| (i, *u))) {
        let (e, n, z) = unit_position(combat.prev_flights.get(i), &u, combat.alpha);
        Some(wu_to_bevy(e, n, z))
    } else if let Some(((e, n, z), _, _)) = t.pilot.and_then(|k| remote_motion(&combat, k)) {
        Some(wu_to_bevy(e, n, z))
    } else {
        None
    };
    let dir = match target {
        Some(p) if (p - eye).length() > 1.0 => (p - eye).as_vec3().normalize(),
        // The gunship's slaved pod: along its sight.
        _ if let Some(d) = mfd.av.tgp.los.filter(|_| settings.aircraft.spec().turret) => d,
        _ => {
            let (h, _, _) = a.euler();
            let d = match mfd.cam_dir {
                CamDir::Ahead => 0.0,
                CamDir::Right => 0.25,
                CamDir::Rear => 0.5,
                CamDir::Left => 0.75,
            } * std::f32::consts::TAU;
            let p = -bam(LENS_DEPRESSION);
            let yaw = h + d;
            Vec3::new(yaw.sin() * p.cos(), p.sin(), -yaw.cos() * p.cos())
        }
    };
    // A manual zoom; AUTO with nothing locked: 4X.
    if let Some(z) = mfd.cam_zoom {
        info.zoom = z + 1;
    } else if target.is_none() {
        info.zoom = crate::mfd::CAM_ZOOM_DEFAULT + 1;
    }
    aim(&mut tf, &mut pos, &mut proj, dir, None, eye, info.zoom.saturating_sub(1), false);
    site(&mut sites, look);
}

/// Points the camera from `eye` along `dir` (rolled to put `up` at the top) with zoom step `z`.
#[allow(clippy::too_many_arguments)]
fn aim(
    tf: &mut Transform,
    pos: &mut WorldPos,
    proj: &mut Projection,
    dir: Vec3,
    up: Option<Vec3>,
    eye: bevy::math::DVec3,
    z: u8,
    wide: bool,
) {
    let up = up.unwrap_or(if dir.y.abs() > 0.99 { Vec3::NEG_Z } else { Vec3::Y });
    tf.rotation = Transform::IDENTITY.looking_to(dir, up).rotation;
    if pos.0 != eye {
        pos.0 = eye;
    }
    let want_proj = if wide { wide_projection(z) } else { projection(z) };
    if let Projection::Perspective(p) = proj
        && (p.fov - want_proj.fov).abs() > 1e-6
    {
        *p = want_proj;
    }
}

/// Half width (tangent) of the unzoomed camera: 38.5 pixels at f = 256.
const BASE_TAN_HALF_W: f32 = 38.5 / 256.0;
/// Widest ILS view (about 30 degrees each side).
const MAX_TAN_HALF_W: f32 = 0.58;

/// Projection with a given half-width tangent, keeping the MFD's 77x60 picture shape.
fn view_projection(tan_half_w: f32) -> PerspectiveProjection {
    let tan_half_h = tan_half_w * (30.0 / 0.75) / 38.5;
    PerspectiveProjection {
        fov: 2.0 * tan_half_h.atan(),
        aspect_ratio: tan_half_w / tan_half_h,
        near: 20.0,
        far: 600_000.0,
        ..default()
    }
}

struct IlsView {
    dir: Vec3,
    tan_half_w: f32,
    range_m: f32,
    /// Runway number (landing heading / 10).
    runway: u32,
}

/// The nearest runway end the aircraft is approaching (ahead, within 25 km and the approach
/// cone): the camera looks at the runway centreline abeam its PAPI, as wide as the PAPI needs.
fn ils_view(
    sites: &[crate::scenery::PapiSite],
    eye: bevy::math::DVec3,
    heading: f32,
) -> Option<IlsView> {
    let (site, _, _) =
        crate::scenery::approach_to(sites, bevy::math::DVec2::new(eye.x, -eye.z), heading)?;
    let aim = bevy::math::DVec3::new(site.aim.x, 1.0, -site.aim.y);
    let d = aim - eye;
    let range = d.length();
    let landing = -site.toward;
    let deg = (landing.x.atan2(landing.y).to_degrees() + 360.0) % 360.0;
    let runway = match (deg / 10.0).round() as u32 {
        0 => 36,
        n => n,
    };
    Some(IlsView {
        dir: d.as_vec3().normalize(),
        // Zoomed in far out, widening on short final so the PAPI stays in the picture.
        tan_half_w: ((site.half_width / range) as f32)
            .clamp(BASE_TAN_HALF_W * 0.05, MAX_TAN_HALF_W),
        range_m: range as f32,
        runway,
    })
}

#[derive(Component)]
struct TrackCamBackground;

/// FLIR: the camera's sky and ground become flat (sky 0x88, ground 0x7C).
fn flir_background(
    mfd: Res<MfdState>,
    palette: Option<Res<crate::palette::Palette>>,
    mut q: Query<
        &mut MeshMaterial3d<crate::palette_material::PaletteMaterial>,
        With<TrackCamBackground>,
    >,
) {
    let Some(palette) = palette else { return };
    let want = if mfd.flir { &palette.background_flir } else { &palette.background };
    for mut m in &mut q {
        if m.0 != *want {
            m.0 = want.clone();
        }
    }
}

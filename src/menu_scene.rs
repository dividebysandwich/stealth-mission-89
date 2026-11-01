//! The scene behind the main menu: an F-117A on the apron at Tabor on one side of the screen,
//! the control tower behind it on the other, the menu between them, on a clear full-moon night in
//! the classic palette with the moon over the field, casting shadows, an RAH-66 hovering on the
//! menu's right and a few aircraft circling in the distance. In VR the player stands on the apron, life size, with the menu's panel between
//! the tower and the aircraft.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy_egui::input::EguiWantsInput;

use crate::models::{ModelAssets, ModelLibrary, ModelState};
use crate::palette::{Palette, TimeOfDay};
use crate::palette_material::{LEVEL_AIRCRAFT, PaletteMaterial};
use crate::settings::{AppState, Settings};
use crate::world::{WU, WorldAnchor, WorldPos};

pub struct MenuScenePlugin;

impl Plugin for MenuScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), spawn_scene)
            .add_systems(OnExit(AppState::Menu), (despawn_scene, crate::world::unload_theater))
            .add_systems(
                Update,
                (
                    spawn_aircraft,
                    spawn_heli,
                    hover_heli,
                    spawn_traffic,
                    fly_traffic,
                    aircraft_lights,
                    mouse_look,
                    fit_view,
                    match_overlay_msaa,
                )
                    .run_if(resource_exists::<MenuScene>),
            );
        #[cfg(feature = "xr")]
        app.add_systems(Update, face_panel.run_if(resource_exists::<MenuScene>));
    }
}

/// The scene is drawn behind the menu.
#[derive(Resource)]
pub struct MenuScene;

/// Everything spawned for the scene.
#[derive(Component)]
struct SceneEntity;

/// The aircraft on the apron (its model is spawned once the models are loaded).
#[derive(Component)]
struct SceneAircraft;

/// The RAH-66 hovering over the apron (its model is spawned once the models are loaded).
#[derive(Component)]
struct SceneHeli;

/// One of the aircraft circling in the distance (index into `TRAFFIC`).
#[derive(Component)]
struct Traffic(usize);

#[derive(Component)]
pub(crate) struct SceneCamera;

/// Where the camera looks, turned by the mouse as in flight: radians of yaw (anticlockwise from
/// above, as `Transform`'s) and pitch (up).
#[derive(Resource)]
struct MenuLook {
    yaw: f32,
    pitch: f32,
    /// The right button went down on the scene, not on a menu window (the planner's map and the
    /// campaign's designation use the right click): the view turns until it is released.
    turning: bool,
}

impl Default for MenuLook {
    fn default() -> Self {
        Self { yaw: -CAMERA[3].to_radians(), pitch: CAMERA[4].to_radians(), turning: false }
    }
}

/// VR: where the player stands, the XR tracking root under it.
#[derive(Component)]
struct StandingPoint;

const THEATER: &str = "central_europe";
/// The airbase whose control tower the scene is laid out from (the tower nearest it); the
/// runway runs north and south 265 m east of the tower, its middle 150 m north of it; the
/// parallel taxiway some 80 m east of the tower. The tower and the aircraft stand on the apron
/// west of the taxiway.
const BASE: &str = "Tabor";
/// The theater's airbase and control tower models.
const BASE_SHAPE: usize = 21;
const TOWER_SHAPE: u8 = 39;
/// The tower (world units east, north) if the theater has none near the base.
const TOWER_FALLBACK: (f64, f64) = (567_552.0, 173_056.0);
/// The camera (metres east, up and north of the tower's foot), its heading (degrees clockwise
/// from north) and pitch.
const CAMERA: [f32; 5] = [-40.0, 4.5, 215.0, 146.0, 7.0];
/// The aircraft (metres east and north of the tower's foot) and its heading (degrees): turned
/// some 50 degrees from facing the camera, its profile towards the menu.
const AIRCRAFT: [f32; 3] = [30.0, 170.0, 250.0];
/// The RAH-66 hovering on the menu's right (metres east, up and north of the tower's foot) and
/// its heading (degrees): nose to the left of the camera, so it shows its profile.
const HELI: [f32; 4] = [-19.0, 12.0, 147.0, 75.0];
const VR_HELI: [f32; 4] = [-10.0, 7.0, 157.0, 70.0];
/// Its hover, as a pilot holds it: drifts (metres, periods in seconds) to the side, fore and aft,
/// up and down, and in heading (degrees), each two waves out of step.
const HELI_SIDE: [[f32; 2]; 2] = [[0.7, 7.3], [0.12, 3.1]];
const HELI_FORE: [[f32; 2]; 2] = [[0.5, 8.9], [0.1, 3.7]];
const HELI_UP: [[f32; 2]; 2] = [[0.3, 5.3], [0.08, 2.3]];
const HELI_YAW: [[f32; 2]; 2] = [[2.5, 11.3], [0.8, 4.1]];
/// Its hover attitude, nose up (radians).
const HELI_PITCH: f32 = 0.03;
/// The moon's azimuth (degrees clockwise from north) and elevation: between the tower and the
/// menu.
const MOON: [f32; 2] = [166.0, 14.0];
/// VR: the menu's panel covers some 60 by 40 degrees, so the player stands nearer the aircraft
/// (metres east and north of the tower's foot), facing the panel the camera's way: the aircraft 31
/// degrees to the left, the tower 31 degrees to the right, the moon above the tower.
const VR_STAND: [f32; 2] = [-20.0, 190.0];
const VR_AIRCRAFT: [f32; 3] = [30.0, 170.0, 240.0];
const VR_MOON: [f32; 2] = [174.0, 28.0];
/// Aircraft circling in the distance beyond the field: model number (aircraft file), circle
/// centre (metres east and north of the tower's foot), radius and altitude (metres), speed (m/s,
/// positive anticlockwise) and where on the circle it starts (turns from east).
const TRAFFIC: [(i32, [f32; 6]); 4] = [
    (23, [500.0, -650.0, 500.0, 130.0, 90.0, 0.0]),
    // Its wingman.
    (23, [500.0, -650.0, 500.0, 130.0, 90.0, -0.05]),
    (24, [1080.0, -1450.0, 900.0, 250.0, -85.0, 0.4]),
    (23, [1500.0, -1000.0, 1100.0, 350.0, -130.0, 0.7]),
];
/// VR: the player's eye height over the apron (metres).
const VR_EYE: f32 = 1.75;
/// The classic night palette's seed (`TimeOfDay::seed`): a few steps from dusk towards night,
/// the purple horizon under a dark blue sky.
const TOD_SEED: u8 = 3;
/// The narrowest horizontal field of view (degrees), so the tower and the aircraft stay on the
/// screen in narrow windows.
const MIN_WIDTH_FOV: f32 = 72.0;

/// The scene's control tower (world units east, north), found once in the theater file.
fn tower() -> (f64, f64) {
    static TOWER: std::sync::OnceLock<(f64, f64)> = std::sync::OnceLock::new();
    *TOWER.get_or_init(|| {
        let t = crate::world::Theater::load(THEATER);
        let w = &t.world;
        w.objects
            .iter()
            .find(|o| (o.object_idx & 0x7F) as usize == BASE_SHAPE && w.name(o.name) == BASE)
            .and_then(|o| {
                let (e, n) = crate::world::map_to_wu(o.x, o.y);
                t.nearest_tile_object(TOWER_SHAPE, e, n)
            })
            .unwrap_or(TOWER_FALLBACK)
    })
}

/// A point `east`, `up`, `north` metres from the tower's foot, in Bevy metres.
fn at_tower(east: f32, up: f32, north: f32) -> DVec3 {
    let tower = tower();
    crate::world::wu_to_bevy(tower.0, tower.1, 0.0)
        + DVec3::new(east as f64, up as f64, -(north as f64))
}

/// Direction (Bevy: x east, y up, z south) at an azimuth and elevation in degrees.
fn direction(az: f32, el: f32) -> Vec3 {
    let (az, el) = (az.to_radians(), el.to_radians());
    Vec3::new(az.sin() * el.cos(), el.sin(), -az.cos() * el.cos())
}

#[allow(clippy::too_many_arguments)]
fn spawn_scene(
    mut commands: Commands,
    settings: Res<Settings>,
    mut streamed: ResMut<crate::world::StreamedCells>,
    mut overrides: ResMut<crate::world::TileOverrides>,
    mut materials: ResMut<Assets<PaletteMaterial>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tod: ResMut<TimeOfDay>,
    mut sky: ResMut<crate::sky::Sky>,
    mut viewpoint: ResMut<crate::palette::Viewpoint>,
    root: Option<Res<crate::camera::XrRoot>>,
) {
    let (cam, moon) = (CAMERA, if settings.vr { VR_MOON } else { MOON });
    commands.insert_resource(MenuScene);
    commands.insert_resource(MenuLook::default());
    crate::world::insert_theater(&mut commands, THEATER, None, &mut streamed, &mut overrides);
    // A night in the classic palette, held still.
    *tod = TimeOfDay::default();
    tod.night_mission = true;
    tod.seed = TOD_SEED;
    tod.frozen = true;
    let palette = Palette::new(THEATER, &tod, &mut materials, &mut buffers);
    // The full moon: the sun stands opposite, below the horizon. As in the continuous time of
    // day the moon lights the field (and casts the shadows); the palette stays the classic one
    // (`sky::update_sky` runs only in flight).
    *sky = crate::sky::Sky::default();
    let m = direction(moon[0], moon[1]);
    (sky.on, sky.moon, sky.sun, sky.moonlight, sky.stars) = (true, m, -m, 1.0, 0.4);
    crate::lighting::spawn_lights(&mut commands, &settings);
    if settings.vr {
        viewpoint.altitude_wu = (VR_EYE as f64 / WU) as f32;
        // `face_panel` turns and moves it once the panel is placed.
        let stand = commands
            .spawn((
                StandingPoint,
                SceneEntity,
                WorldAnchor,
                WorldPos(at_tower(VR_STAND[0], 0.0, VR_STAND[1])),
                Transform::from_rotation(Quat::from_rotation_y(-cam[3].to_radians())),
                Visibility::default(),
            ))
            .id();
        if let Some(r) = root.and_then(|r| r.0) {
            commands.entity(r).insert((crate::camera::EyeCamera, Transform::IDENTITY));
            let sphere = crate::camera::spawn_background(
                &mut commands,
                r,
                &mut meshes,
                &palette,
                crate::camera::MAIN_BACKGROUND_LAYER,
            );
            commands.entity(sphere).insert(SceneEntity);
            commands.entity(stand).add_child(r);
        }
        commands.insert_resource(palette);
        return;
    }
    viewpoint.altitude_wu = (cam[1] as f64 / WU) as f32;
    let camera = commands
        .spawn((
            Camera3d::default(),
            Camera { order: 0, ..default() },
            bevy::core_pipeline::tonemapping::Tonemapping::None,
            Projection::Perspective(PerspectiveProjection {
                near: 0.5,
                far: 600_000.0,
                ..default()
            }),
            SceneCamera,
            SceneEntity,
            WorldAnchor,
            WorldPos(at_tower(cam[0], cam[1], cam[2])),
            Transform::from_rotation(Quat::from_euler(
                EulerRot::YXZ,
                -cam[3].to_radians(),
                cam[4].to_radians(),
                0.0,
            )),
            bevy::camera::visibility::RenderLayers::from_layers(&[
                0,
                crate::camera::MAIN_BACKGROUND_LAYER,
                crate::camera::MAIN_LOD_LAYER,
            ]),
        ))
        .id();
    crate::camera::spawn_background(
        &mut commands,
        camera,
        &mut meshes,
        &palette,
        crate::camera::MAIN_BACKGROUND_LAYER,
    );
    commands.insert_resource(palette);
}

/// The F-117A on the apron, once its model file is loaded.
fn spawn_aircraft(
    mut commands: Commands,
    mut library: ResMut<ModelLibrary>,
    server: Res<AssetServer>,
    assets: ModelAssets,
    existing: Query<(), With<SceneAircraft>>,
    settings: Res<Settings>,
) {
    if !existing.is_empty() {
        return;
    }
    let spec = crate::aircraft::AircraftKind::F117.spec();
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
    let plane = if settings.vr { VR_AIRCRAFT } else { AIRCRAFT };
    let scale = crate::player::AIRCRAFT_MODEL_SCALE as f32;
    // Gear down (slots 5 and 6), navigation lights on (12).
    let mut state = ModelState::default();
    (state.0[5], state.0[6], state.0[12]) = (1, 2, 1);
    commands.entity(model).insert((
        Transform::from_xyz(0.0, spec.model_lift, 0.0).with_scale(Vec3::splat(scale)),
        state,
    ));
    commands
        .spawn((
            SceneAircraft,
            SceneEntity,
            WorldPos(at_tower(plane[0], 0.0, plane[1])),
            Transform::from_rotation(Quat::from_rotation_y(-plane[2].to_radians())),
            Visibility::default(),
        ))
        .add_child(model);
}

/// The RAH-66 over the apron, gear up and lights on, once its model file is loaded (its rotors
/// turn with every model's).
fn spawn_heli(
    mut commands: Commands,
    mut library: ResMut<ModelLibrary>,
    server: Res<AssetServer>,
    assets: ModelAssets,
    existing: Query<(), With<SceneHeli>>,
) {
    if !existing.is_empty() {
        return;
    }
    let spec = crate::aircraft::AircraftKind::RAH66.spec();
    let file = library.load(&server, "aircraft");
    let Some(model) = assets.spawn(&mut commands, &file, spec.model, LEVEL_AIRCRAFT, true) else {
        return;
    };
    let scale = crate::player::AIRCRAFT_MODEL_SCALE as f32;
    // Navigation lights on (12).
    let mut state = ModelState::default();
    state.0[12] = 1;
    commands.entity(model).insert((
        Transform::from_xyz(0.0, spec.model_lift, 0.0).with_scale(Vec3::splat(scale)),
        state,
    ));
    commands
        .spawn((
            SceneHeli,
            SceneEntity,
            WorldPos::default(),
            Transform::default(),
            Visibility::default(),
        ))
        .add_child(model);
}

/// The RAH-66 sways in its hover as a pilot holding it: it drifts a little and comes back, tilted
/// the way it accelerates (a bank for each sideways correction, the nose down to move forward).
fn hover_heli(
    time: Res<Time>,
    settings: Res<Settings>,
    mut q: Query<(&mut WorldPos, &mut Transform), With<SceneHeli>>,
) {
    let t = time.elapsed_secs();
    let [east, up, north, heading] = if settings.vr { VR_HELI } else { HELI };
    // A drift's offset and acceleration (m/s^2).
    let drift = |waves: [[f32; 2]; 2], phase: f32| {
        waves.iter().enumerate().fold((0.0, 0.0), |(x, acc), (i, &[amp, period])| {
            let w = std::f32::consts::TAU / period;
            let s = amp * (w * t + phase + i as f32 * 2.1).sin();
            (x + s, acc - w * w * s)
        })
    };
    let (side, side_acc) = drift(HELI_SIDE, 0.0);
    let (fore, fore_acc) = drift(HELI_FORE, 1.3);
    let (rise, _) = drift(HELI_UP, 0.7);
    let (yaw, _) = drift(HELI_YAW, 2.9);
    let h = heading.to_radians();
    let (dir, right) = (Vec2::new(h.sin(), h.cos()), Vec2::new(h.cos(), -h.sin()));
    let off = dir * fore + right * side;
    for (mut pos, mut tf) in &mut q {
        pos.0 = at_tower(east + off.x, up + rise, north + off.y);
        // The rotor's tilt makes the acceleration: bank = sideways / g, pitch = -forward / g.
        tf.rotation = Quat::from_rotation_y(-(h + yaw.to_radians()))
            * Quat::from_rotation_x(HELI_PITCH - fore_acc / 9.81)
            * Quat::from_rotation_z(-side_acc / 9.81);
    }
}

/// The circling aircraft, once their model file is loaded.
fn spawn_traffic(
    mut commands: Commands,
    mut library: ResMut<ModelLibrary>,
    server: Res<AssetServer>,
    assets: ModelAssets,
    existing: Query<(), With<Traffic>>,
) {
    if !existing.is_empty() {
        return;
    }
    let file = library.load(&server, "aircraft");
    for (i, &(model, _)) in TRAFFIC.iter().enumerate() {
        let name = format!("aircraft_{model:03}");
        let Some(e) = assets.spawn(&mut commands, &file, &name, LEVEL_AIRCRAFT, true) else {
            return;
        };
        // Airborne with the gear up (slot 13), navigation lights on (12).
        let mut state = ModelState::default();
        (state.0[12], state.0[13]) = (1, 1);
        let scale = crate::player::AIRCRAFT_MODEL_SCALE as f32;
        commands.entity(e).insert((
            Traffic(i),
            SceneEntity,
            WorldPos::default(),
            Transform::from_scale(Vec3::splat(scale)),
            state,
            crate::models::LodRange(40.0),
        ));
    }
}

/// The circling aircraft along their circles, banked into the turn.
fn fly_traffic(time: Res<Time>, mut q: Query<(&Traffic, &mut WorldPos, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (traffic, mut pos, mut tf) in &mut q {
        let [east, north, radius, alt, speed, start] = TRAFFIC[traffic.0].1;
        let a = start * std::f32::consts::TAU + t * speed / radius;
        pos.0 = at_tower(east + radius * a.cos(), alt, north + radius * a.sin());
        // Flying along the circle: anticlockwise for positive speeds.
        let (de, dn) = (-a.sin() * speed.signum(), a.cos() * speed.signum());
        let heading = de.atan2(dn);
        let bank = (speed * speed / (9.81 * radius)).atan() * speed.signum();
        tf.rotation = Quat::from_rotation_y(-heading) * Quat::from_rotation_z(bank);
    }
}

/// The aircraft's and the circling ones' anti-collision lights flash; their lights follow the
/// palette's (slots 10/11).
#[allow(clippy::type_complexity)]
fn aircraft_lights(
    time: Res<Time>,
    palette: Option<Res<Palette>>,
    aircraft: Query<&Children, Or<(With<SceneAircraft>, With<SceneHeli>)>>,
    mut states: Query<&mut ModelState, Without<Traffic>>,
    mut traffic: Query<(&Traffic, &mut ModelState)>,
) {
    let lights = palette.is_some_and(|p| p.lights_on()) as i16;
    let t = time.elapsed_secs();
    for (traffic, mut s) in &mut traffic {
        let phase = (traffic.0 as f32 * 0.618).fract();
        for (slot, v) in crate::sim_render::anticollision(t, phase, true)
            .into_iter()
            .chain([(10, lights), (11, lights)])
        {
            if s.0[slot] != v {
                s.0[slot] = v;
            }
        }
    }
    for children in &aircraft {
        let mut it = states.iter_many_mut(children);
        while let Some(mut s) = it.fetch_next() {
            for (slot, v) in crate::sim_render::anticollision(t, 0.0, true)
                .into_iter()
                .chain([(10, lights), (11, lights)])
            {
                if s.0[slot] != v {
                    s.0[slot] = v;
                }
            }
        }
    }
}

/// Desktop mouse look, as in flight: hold the right button on the scene and move the mouse to turn
/// the view round the camera, which stays where it is.
fn mouse_look(
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    egui: Res<EguiWantsInput>,
    mut look: ResMut<MenuLook>,
    mut cams: Query<&mut Transform, With<SceneCamera>>,
    mut cursor: Query<&mut CursorOptions>,
) {
    // No scene camera in the headset.
    if cams.is_empty() {
        return;
    }
    if buttons.just_pressed(MouseButton::Right) {
        look.turning = !egui.wants_any_pointer_input();
    }
    let held = look.turning && buttons.pressed(MouseButton::Right);
    if !held {
        look.turning = false;
    }
    if let Ok(mut c) = cursor.single_mut() {
        let (grab, visible) =
            if held { (CursorGrabMode::Locked, false) } else { (CursorGrabMode::None, true) };
        if c.grab_mode != grab {
            c.grab_mode = grab;
            c.visible = visible;
        }
    }
    if !held || motion.delta == Vec2::ZERO {
        return;
    }
    const K: f32 = 0.004;
    look.yaw -= motion.delta.x * K;
    look.pitch = (look.pitch - motion.delta.y * K).clamp(-1.55, 1.55);
    for mut tf in &mut cams {
        tf.rotation = Quat::from_euler(EulerRot::YXZ, look.yaw, look.pitch, 0.0);
    }
}

/// A narrow window widens the view (Bevy keeps the vertical field of view).
fn fit_view(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut cams: Query<&mut Projection, With<SceneCamera>>,
) {
    let Ok(w) = windows.single() else { return };
    let aspect = w.width() / w.height().max(1.0);
    let half = (MIN_WIDTH_FOV.to_radians() / 2.0).tan();
    let fov = std::f32::consts::FRAC_PI_4.max(2.0 * (half / aspect).atan());
    for mut p in &mut cams {
        if let Projection::Perspective(p) = &mut *p
            && (p.fov - fov).abs() > 1e-4
        {
            p.fov = fov;
        }
    }
}

/// The menu's egui camera draws over the scene: its MSAA has to be the scene camera's (else
/// it gets a picture of its own that nothing clears).
fn match_overlay_msaa(
    scene: Query<&Msaa, (With<SceneCamera>, Without<crate::menu::MenuCamera>)>,
    mut overlay: Query<&mut Msaa, With<crate::menu::MenuCamera>>,
) {
    let Ok(&want) = scene.single() else { return };
    for mut m in &mut overlay {
        if *m != want {
            *m = want;
        }
    }
}

/// VR: the panel goes up in front of the player wherever they face in the tracking space; the
/// standing point turns so the panel faces the scene's way, and moves so the head is at the
/// standing place at eye height.
#[cfg(feature = "xr")]
#[allow(clippy::type_complexity)]
fn face_panel(
    panels: Query<&Transform, (With<crate::vr_menu::VrMenuPanel>, Changed<Transform>)>,
    mut stand: Query<
        (&mut WorldPos, &mut Transform),
        (With<StandingPoint>, Without<crate::vr_menu::VrMenuPanel>),
    >,
) {
    let Ok(panel) = panels.single() else { return };
    let Ok((mut pos, mut tf)) = stand.single_mut() else { return };
    // The head in the tracking space (vr_menu::place_panel).
    let flat = panel.forward().as_vec3();
    let head = panel.translation - flat * crate::vr_menu::DISTANCE + Vec3::Y * 0.1;
    let yaw = (-flat.x).atan2(-flat.z);
    let rot = Quat::from_rotation_y(-CAMERA[3].to_radians() - yaw);
    let place = at_tower(VR_STAND[0], VR_EYE - head.y, VR_STAND[1]);
    pos.0 = place - (rot * Vec3::new(head.x, 0.0, head.z)).as_dvec3();
    tf.rotation = rot;
}

fn despawn_scene(
    mut commands: Commands,
    q: Query<Entity, With<SceneEntity>>,
    root: Option<Res<crate::camera::XrRoot>>,
    look: Option<Res<MenuLook>>,
    mut cursor: Query<&mut CursorOptions>,
) {
    // The menu closed with the view still turning: free the cursor.
    if look.is_some_and(|l| l.turning)
        && let Ok(mut c) = cursor.single_mut()
    {
        c.grab_mode = CursorGrabMode::None;
        c.visible = true;
    }
    // VR: the tracking root leaves the standing point before it goes.
    if let Some(r) = root.and_then(|r| r.0) {
        commands
            .entity(r)
            .remove::<(ChildOf, crate::camera::EyeCamera)>()
            .insert(Transform::IDENTITY);
    }
    for e in &q {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<MenuScene>();
    commands.remove_resource::<MenuLook>();
    commands.remove_resource::<Palette>();
}

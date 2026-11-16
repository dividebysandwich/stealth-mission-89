//! The scene behind the main menu: an F-117A on the apron at Tabor on one side of the screen,
//! the control tower behind it on the other, the menu between them, on a clear full-moon night in
//! the classic palette with the moon over the field, casting shadows, an RAH-66 hovering on the
//! menu's right and two aircraft circling in the distance; now and then a C-5 takes off from the
//! runway, an airliner-high one draws its contrails past the moon or a helicopter crosses in
//! the distance (`passes`). In VR the player stands on the apron, life size, with the menu's panel between
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
                    schedule_passes,
                    fly_passes,
                    aircraft_lights,
                    mouse_look,
                    walk,
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
pub(crate) struct MenuLook {
    yaw: f32,
    pitch: f32,
    /// The right button went down on the scene, not on a menu window (the planner's map and the
    /// campaign's designation use the right click): the view turns until it is released.
    turning: bool,
    /// On foot: the height of the jump over the standing eye (metres) and its vertical speed
    /// (m/s).
    lift: f32,
    climb: f32,
    /// Seconds since a walking key was last held.
    idle: f32,
    /// The joysticks' axes have been seen at rest since the bindings last changed: until then
    /// they don't walk or turn, so that an axis resting off centre (a worn stick, a throttle
    /// bound as pitch) doesn't walk on for ever and keep the main menu away.
    sticks_rested: bool,
}

impl MenuLook {
    /// The main menu is hidden while the player walks, and for `MENU_RETURN` seconds after.
    pub(crate) fn hides_menu(&self) -> bool {
        self.idle < MENU_RETURN
    }
}

impl Default for MenuLook {
    fn default() -> Self {
        Self {
            yaw: -CAMERA[3].to_radians(),
            pitch: CAMERA[4].to_radians(),
            turning: false,
            lift: 0.0,
            climb: 0.0,
            idle: MENU_RETURN,
            sticks_rested: false,
        }
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
const CAMERA: [f32; 5] = [-40.0, VR_EYE, 215.0, 146.0, 7.0];
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
const TRAFFIC: [(i32, [f32; 6]); 2] = [
    (23, [500.0, -650.0, 500.0, 130.0, 90.0, 0.0]),
    (24, [1080.0, -1450.0, 900.0, 250.0, -85.0, 0.4]),
];
/// VR: the player's eye height over the apron (metres); also the standing eye height on the
/// desktop, where the player walks the apron.
const VR_EYE: f32 = 1.75;
/// On foot: walking and running speeds (m/s), the height of a jump (metres: a standing jump of
/// a man of 1.80 m lifts him some half a metre), gravity (m/s^2) and the turn rate of a stick
/// pushed over (radians per second: on the desktop, in the headset, up and down).
const WALK: f32 = 5.0;
const RUN: f32 = 12.0;
const JUMP: f32 = 0.5;
const GRAVITY: f32 = 9.81;
const TURN: f32 = 2.0;
const TURN_VR: f32 = 1.0;
const LOOK: f32 = 1.5;
/// How long the player stands still (seconds) before the main menu comes back.
const MENU_RETURN: f32 = 0.5;
/// The joysticks' dead zone on foot (their own, set for flying, may be none).
const STICK_DEAD: f32 = 0.15;
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
    commands.insert_resource(Passes::new());
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
    mut states: Query<&mut ModelState, (Without<Traffic>, Without<Passer>)>,
    mut traffic: Query<
        (Entity, Option<&Traffic>, &mut ModelState),
        Or<(With<Traffic>, With<Passer>)>,
    >,
) {
    let lights = palette.is_some_and(|p| p.lights_on()) as i16;
    let t = time.elapsed_secs();
    for (e, traffic, mut s) in &mut traffic {
        let n = traffic.map_or(e.index_u32() as usize, |t| t.0);
        let phase = (n as f32 * 0.618).fract();
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

/// Desktop mouse look, as in flight: hold the right button (`Action::MouseLook`) on the scene and move the mouse to turn
/// the view round the camera, which stays where it is.
fn mouse_look(
    pointer: Res<crate::input::FlightControls>,
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
    if pointer.mouse_look_pressed {
        look.turning = !egui.wants_any_pointer_input();
    }
    let held = look.turning && pointer.mouse_look;
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

/// What the player's hands ask for on foot, from the keyboard or the controllers.
#[derive(Default)]
struct Steps {
    /// Walking: x to the right, y forward (-1..1).
    go: Vec2,
    /// Turning to the right and looking up (-1..1).
    turn: f32,
    look: f32,
    run: bool,
    jump: bool,
    /// A walking key is held or a stick is pushed (the main menu steps aside).
    moving: bool,
    /// The joysticks' axes alone (walk x and y, turn, look), past `STICK_DEAD`.
    stick: Vec4,
}

/// The gamepads' and the joysticks' inputs: the stick (a gamepad's left one) walks, the rudder, the
/// slew's X or the gamepad's right stick turns, the slew's Y (the right stick's) looks up and
/// down, the fire button (a gamepad's A) jumps and the gun trigger (a gamepad's shoulder
/// triggers and left stick click) runs. The joysticks' are the ones the flight mapping binds.
fn read_devices(
    gamepads: &Query<&Gamepad>,
    config: &crate::hotas::InputConfig,
    hotas: &crate::hotas::Hotas,
) -> Steps {
    use crate::hotas::{AxisControl, ButtonTarget};
    let mut s = Steps::default();
    let more = crate::input::stronger;
    if config.gamepad_defaults {
        for pad in gamepads {
            let axis =
                |a: GamepadAxis, dead: f32| pad.get(a).filter(|v| v.abs() > dead).unwrap_or(0.0);
            s.go.x = more(s.go.x, axis(GamepadAxis::LeftStickX, 0.1));
            s.go.y = more(s.go.y, axis(GamepadAxis::LeftStickY, 0.1));
            s.turn = more(s.turn, axis(GamepadAxis::RightStickX, 0.15));
            s.look = more(s.look, axis(GamepadAxis::RightStickY, 0.15));
            s.run |= [
                GamepadButton::RightTrigger,
                GamepadButton::RightTrigger2,
                GamepadButton::LeftThumb,
            ]
            .into_iter()
            .any(|b| pad.pressed(b));
            s.jump |= pad.just_pressed(GamepadButton::South);
        }
    }
    for m in hotas.axes(config) {
        let Some(raw) = hotas.axis(&m.device, m.code) else { continue };
        let v = m.output(raw);
        let v = if v.abs() > STICK_DEAD { v } else { 0.0 };
        let stick = &mut s.stick;
        match m.control {
            AxisControl::Roll => stick.x = more(stick.x, v),
            // Pushing the stick forward is a negative pitch.
            AxisControl::Pitch => stick.y = more(stick.y, -v),
            AxisControl::Yaw | AxisControl::SlewX => stick.z = more(stick.z, v),
            AxisControl::SlewY => stick.w = more(stick.w, v),
            _ => {}
        }
    }
    for m in hotas.buttons(config) {
        match m.target {
            ButtonTarget::Gun => s.run |= hotas.active(&m.device, m.source, false),
            ButtonTarget::Action(crate::input::Action::FireWeapon) => {
                s.jump |= hotas.just_pressed(&m.device, m.source)
            }
            _ => {}
        }
    }
    s
}

/// Adds the joysticks' axes to the steps once they have been seen at rest (`sticks_rested`),
/// none while the bindings dialog is open (binding an axis moves it); then whether the player
/// is walking.
fn add_sticks(s: &mut Steps, rested: &mut bool, binding: bool) {
    if binding {
        *rested = false;
        *s = Steps::default();
        return;
    }
    *rested |= s.stick == Vec4::ZERO;
    if *rested {
        let more = crate::input::stronger;
        s.go = Vec2::new(more(s.go.x, s.stick.x), more(s.go.y, s.stick.y));
        s.turn = more(s.turn, s.stick.z);
        s.look = more(s.look, s.stick.w);
    }
    s.moving = s.go != Vec2::ZERO;
}

/// Walking, as a pilot on foot over the apron: W A S D (the keys at those places, Z Q S D on an
/// AZERTY keyboard) or a gamepad's or a joystick's stick (`read_devices`) move over the ground,
/// Shift or the run button runs, Space or the jump button jumps as a man of 1.80 m does. The
/// camera follows the hills. In the headset the standing point walks the way the head faces, and
/// the rudder turns it round the head.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn walk(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    (egui, dialog): (Res<EguiWantsInput>, Res<crate::input_dialog::InputDialog>),
    devices: (Query<&Gamepad>, Res<crate::hotas::InputConfig>, Res<crate::hotas::Hotas>),
    theater: Option<Res<crate::world::Theater>>,
    heads: Res<crate::camera::HeadPoses>,
    mut look: ResMut<MenuLook>,
    mut cams: Query<(&mut WorldPos, &mut Transform), (With<SceneCamera>, Without<StandingPoint>)>,
    mut stands: Query<(&mut WorldPos, &mut Transform), (With<StandingPoint>, Without<SceneCamera>)>,
    mut viewpoint: ResMut<crate::palette::Viewpoint>,
) {
    // The desktop camera, or the headset once it is tracked.
    if cams.is_empty() && (stands.is_empty() || heads.0.is_empty()) {
        return;
    }
    let dt = time.delta_secs().min(0.1);
    let (gamepads, config, hotas) = devices;
    // A text field or a shortcut (Ctrl, Alt) is using the keys.
    // The bindings dialog takes keys, buttons and axes to bind.
    let free = !dialog.open
        && !egui.wants_any_keyboard_input()
        && !keys.any_pressed([
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
            KeyCode::AltLeft,
            KeyCode::AltRight,
        ]);
    let mut s = read_devices(&gamepads, &config, &hotas);
    // Esc brings the menu back from a stick that won't rest.
    if keys.just_pressed(KeyCode::Escape) {
        look.sticks_rested = false;
    }
    add_sticks(&mut s, &mut look.sticks_rested, dialog.open);
    if free {
        let mut dir = Vec2::ZERO;
        for (key, d) in [
            (KeyCode::KeyW, Vec2::Y),
            (KeyCode::KeyS, Vec2::NEG_Y),
            (KeyCode::KeyA, Vec2::NEG_X),
            (KeyCode::KeyD, Vec2::X),
        ] {
            if keys.pressed(key) {
                dir += d;
                s.moving = true;
            }
        }
        if dir != Vec2::ZERO {
            s.go = dir.normalize();
        }
        s.run |= keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        s.jump |= keys.just_pressed(KeyCode::Space);
    }
    look.idle = if s.moving { 0.0 } else { (look.idle + dt).min(MENU_RETURN) };
    let pace = s.go.clamp_length_max(1.0) * if s.run { RUN } else { WALK };
    // The jump: up at the speed that takes a man's height, then back down under gravity.
    let was = look.lift;
    if s.jump && was <= 0.0 {
        look.climb = (2.0 * GRAVITY * JUMP).sqrt();
    }
    if look.climb != 0.0 || look.lift > 0.0 {
        look.lift += look.climb * dt;
        look.climb -= GRAVITY * dt;
        if look.lift <= 0.0 {
            (look.lift, look.climb) = (0.0, 0.0);
        }
    }
    let rise = (look.lift - was) as f64;
    let ground = |p: DVec3| theater.as_ref().map_or(0.0, |t| t.hills.ground_m(p.x, -p.z) as f64);
    if let Ok((mut pos, mut tf)) = cams.single_mut() {
        look.yaw -= s.turn * TURN * dt;
        look.pitch = (look.pitch + s.look * LOOK * dt).clamp(-1.55, 1.55);
        if s.turn != 0.0 || s.look != 0.0 {
            tf.rotation = Quat::from_euler(EulerRot::YXZ, look.yaw, look.pitch, 0.0);
        }
        // Along the view's heading (not its pitch).
        let step = Mat2::from_angle(-look.yaw) * Vec2::new(pace.x, -pace.y) * dt;
        let before = ground(pos.0);
        pos.0 += DVec3::new(step.x as f64, 0.0, step.y as f64);
        pos.0.y += ground(pos.0) - before + rise;
        viewpoint.altitude_wu = (pos.0.y / WU) as f32;
    } else if let Ok((mut pos, mut tf)) = stands.single_mut() {
        // The headset: forward is where the head faces in the tracking space.
        let head = heads.0[0];
        let face = head.forward().as_vec3();
        let face = Vec3::new(face.x, 0.0, face.z).normalize_or(Vec3::NEG_Z);
        let step = tf.rotation
            * Quat::from_rotation_arc(Vec3::NEG_Z, face)
            * Vec3::new(pace.x, 0.0, -pace.y)
            * dt;
        // Turning swings the standing point round the head.
        let turned = Quat::from_rotation_y(-s.turn * TURN_VR * dt) * tf.rotation;
        let at_head = Vec3::new(head.translation.x, 0.0, head.translation.z);
        let before = ground(pos.0);
        pos.0 += (tf.rotation * at_head - turned * at_head + step).as_dvec3();
        tf.rotation = turned.normalize();
        pos.0.y += ground(pos.0) - before + rise;
        viewpoint.altitude_wu = ((pos.0.y + head.translation.y as f64) / WU) as f32;
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
    commands.remove_resource::<Passes>();
    commands.remove_resource::<Palette>();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pushed(stick: Vec4) -> Steps {
        Steps { stick, ..default() }
    }

    #[test]
    fn a_stick_resting_off_centre_never_walks() {
        let mut rested = false;
        for _ in 0..100 {
            let mut s = pushed(Vec4::new(0.0, 0.4, 0.0, 0.0));
            add_sticks(&mut s, &mut rested, false);
            assert!(!s.moving);
        }
        // Once centred, it walks.
        let mut s = pushed(Vec4::ZERO);
        add_sticks(&mut s, &mut rested, false);
        let mut s = pushed(Vec4::new(0.0, 0.4, 0.0, 0.0));
        add_sticks(&mut s, &mut rested, false);
        assert!(s.moving);
        // Binding in the dialog stops it, and it must rest again after.
        let mut s = pushed(Vec4::new(0.0, 0.4, 0.0, 0.0));
        add_sticks(&mut s, &mut rested, true);
        assert!(!s.moving && !rested);
        let mut s = pushed(Vec4::new(0.0, 0.4, 0.0, 0.0));
        add_sticks(&mut s, &mut rested, false);
        assert!(!s.moving);
    }
}

// --- Passes: the occasional aircraft crossing the scene -----------------------------------------

/// The kinds of pass, each on its own schedule: a C-5 taking off from the runway, a C-5 at
/// cruise height drawing contrails past the moon, a helicopter crossing in the distance.
#[derive(Clone, Copy, PartialEq, Debug)]
enum PassKind {
    Takeoff,
    Contrail,
    Heli,
}

const PASS_KINDS: [PassKind; 3] = [PassKind::Takeoff, PassKind::Contrail, PassKind::Heli];

impl PassKind {
    /// Seconds before the first pass, and between passes (from one's end to the next start).
    fn wait(self, first: bool) -> std::ops::Range<f32> {
        match (self, first) {
            (PassKind::Takeoff, true) => 15.0..35.0,
            (PassKind::Takeoff, false) => 120.0..240.0,
            (PassKind::Contrail, true) => 30.0..60.0,
            (PassKind::Contrail, false) => 90.0..200.0,
            (PassKind::Heli, true) => 6.0..20.0,
            (PassKind::Heli, false) => 50.0..130.0,
        }
    }
}

/// Seconds until each kind's next pass (`PASS_KINDS` order); a kind waits while its pass is
/// on. `SM89_MENU_PASSES=<takeoff>,<contrail>,<heli>` sets the first waits (screenshots); a
/// negative one starts that pass at once, that many seconds into it.
#[derive(Resource)]
struct Passes([f32; 3]);

impl Passes {
    fn new() -> Self {
        let mut wait = PASS_KINDS.map(|k| rand::random_range(k.wait(true)));
        if let Ok(v) = std::env::var("SM89_MENU_PASSES") {
            for (w, x) in wait.iter_mut().zip(v.split(',')) {
                if let Ok(x) = x.trim().parse() {
                    *w = x;
                }
            }
        }
        Self(wait)
    }
}

/// An aircraft on a pass: its kind, seconds since it began, and how it flies: the take-off's
/// state (metres east, up, north of the tower's foot, speed m/s, heading and pitch radians), or
/// a straight line's start, direction and speed (east, north; m/s) at a height.
#[derive(Component)]
struct Passer {
    kind: PassKind,
    t: f32,
    pos: Vec3,
    speed: f32,
    heading: f32,
    pitch: f32,
    dir: Vec2,
    /// A straight pass ends after this many seconds.
    length: f32,
}

/// The take-off: the runway (metres east of the tower) and where the roll starts (north),
/// southbound; the acceleration on the roll and in the climb (m/s^2), the rotation speed, the
/// climb angle (radians), the climb speed, when the gear comes up and the turn starts (seconds
/// after lift-off), the turn's bank and the heading it turns to (radians, clockwise from north).
const RUNWAY_EAST: f32 = 265.0;
const ROLL_START: f32 = 1100.0;
const ROLL_ACCEL: f32 = 2.2;
const CLIMB_ACCEL: f32 = 0.6;
const ROTATE: f32 = 70.0;
const CLIMB: f32 = 0.12;
const CLIMB_SPEED: f32 = 95.0;
const GEAR_UP: f32 = 5.0;
const TURN_AT: f32 = 25.0;
const TURN_BANK: f32 = 0.3;
const TURN_TO: f32 = 2.2;
/// A pass ends this far from the tower (metres).
const PASS_GONE: f32 = 25_000.0;
/// The high pass: its height (metres), how far above the moon it crosses (degrees), the half
/// sweep each side of the moon (degrees) and its speed (m/s).
const HIGH_ALT: f32 = 8500.0;
const HIGH_OVER_MOON: f32 = 1.5;
const HIGH_SWEEP: f32 = 28.0;
const HIGH_SPEED: f32 = 230.0;
/// The helicopter's pass: its distance from the viewer (metres), its bearing off the view's
/// heading (degrees), its height over the ground (metres), its half sweep (degrees) and speed
/// (m/s).
const HELI_RANGE: std::ops::Range<f32> = 1200.0..2200.0;
const HELI_BEARING: std::ops::Range<f32> = -15.0..25.0;
const HELI_AGL: std::ops::Range<f32> = 60.0..150.0;
const HELI_SWEEP: f32 = 55.0;
const HELI_SPEED: f32 = 50.0;

/// Starts each kind's pass when its wait is over (once the models are loaded).
#[allow(clippy::too_many_arguments)]
fn schedule_passes(
    mut commands: Commands,
    time: Res<Time>,
    mut library: ResMut<ModelLibrary>,
    server: Res<AssetServer>,
    assets: ModelAssets,
    settings: Res<Settings>,
    mut passes: ResMut<Passes>,
    on: Query<&Passer>,
    theater: Option<Res<crate::world::Theater>>,
) {
    use crate::aircraft::AircraftKind;
    let dt = time.delta_secs();
    let file = library.load(&server, "aircraft");
    let (viewer, moon) = if settings.vr {
        (Vec2::from(VR_STAND), VR_MOON)
    } else {
        (Vec2::new(CAMERA[0], CAMERA[2]), MOON)
    };
    for (i, kind) in PASS_KINDS.into_iter().enumerate() {
        if on.iter().any(|p| p.kind == kind) {
            continue;
        }
        passes.0[i] -= dt;
        if passes.0[i] > 0.0 {
            continue;
        }
        let craft = match kind {
            PassKind::Takeoff | PassKind::Contrail => AircraftKind::C5,
            PassKind::Heli => [AircraftKind::UH60, AircraftKind::CH47, AircraftKind::AH64]
                [rand::random_range(0..3)],
        };
        let spec = craft.spec();
        let Some(e) = assets.spawn(&mut commands, &file, spec.model, LEVEL_AIRCRAFT, true) else {
            return;
        };
        let ahead = -passes.0[i];
        passes.0[i] = rand::random_range(kind.wait(false));
        let side = if rand::random::<bool>() { 1.0 } else { -1.0 };
        let mut p = Passer {
            kind,
            t: 0.0,
            pos: Vec3::new(RUNWAY_EAST, 0.0, ROLL_START),
            speed: 0.0,
            heading: std::f32::consts::PI,
            pitch: 0.0,
            dir: Vec2::ZERO,
            length: 0.0,
        };
        // Nav lights on (12); the gear down (5, 6 as the own model's) or up (13).
        let mut state = ModelState::default();
        state.0[12] = 1;
        match kind {
            PassKind::Takeoff => {
                (state.0[5], state.0[6]) = (1, 2);
                state.0[crate::aircraft::FLAP_SLOT] =
                    crate::aircraft::flap_slot_value(crate::aircraft::C5_FLAP_ANGLE);
            }
            PassKind::Contrail => {
                // Across the line to the moon, a little above it.
                let (az, el) = (moon[0].to_radians(), (moon[1] + HIGH_OVER_MOON).to_radians());
                let range = HIGH_ALT / el.tan();
                let towards = Vec2::new(az.sin(), az.cos());
                let half = range * HIGH_SWEEP.to_radians().tan();
                p.dir = Vec2::new(towards.y, -towards.x) * side;
                p.pos = (viewer + towards * range - p.dir * half).extend(HIGH_ALT).xzy();
                (p.speed, p.length) = (HIGH_SPEED, 2.0 * half / HIGH_SPEED);
                state.0[13] = 1;
                commands
                    .entity(e)
                    .insert(crate::trails::Contrails(crate::aircraft::contrail_emitters(craft)));
            }
            PassKind::Heli => {
                let range = rand::random_range(HELI_RANGE);
                let az = (CAMERA[3] + rand::random_range(HELI_BEARING)).to_radians();
                let towards = Vec2::new(az.sin(), az.cos());
                let half = range * HELI_SWEEP.to_radians().tan();
                p.dir = Vec2::new(towards.y, -towards.x) * side;
                let start = viewer + towards * range - p.dir * half;
                p.pos = start.extend(rand::random_range(HELI_AGL)).xzy();
                (p.speed, p.length) = (HELI_SPEED, 2.0 * half / HELI_SPEED);
                // Nose down to fly on.
                p.pitch = -0.08;
                state.0[13] = 1;
            }
        }
        if kind != PassKind::Takeoff {
            p.heading = p.dir.x.atan2(p.dir.y);
        }
        let mut bank = 0.0;
        while p.t < ahead {
            bank = step(&mut p, &mut state, 0.05);
        }
        let scale = crate::player::AIRCRAFT_MODEL_SCALE as f32;
        // Where it starts (the contrails begin there).
        let (at, rotation) = place(&p, bank, spec.model_lift, theater.as_deref());
        commands.entity(e).insert((
            SceneEntity,
            WorldPos(at),
            Transform::from_scale(Vec3::splat(scale)).with_rotation(rotation),
            state,
            crate::models::LodRange(if kind == PassKind::Contrail { 2000.0 } else { 200.0 }),
            ModelLift(spec.model_lift),
            p,
        ));
    }
}

/// The height of a pass's model's origin over its wheels (`AircraftSpec::model_lift`).
#[derive(Component)]
struct ModelLift(f32);

/// Flies the passes: the take-off rolls, rotates, climbs out with the gear and flaps coming up
/// and turns away; the others fly their straight lines (the helicopter over the ground). Each
/// is gone at its end.
fn fly_passes(
    mut commands: Commands,
    time: Res<Time>,
    theater: Option<Res<crate::world::Theater>>,
    mut q: Query<(Entity, &mut Passer, &ModelLift, &mut WorldPos, &mut Transform, &mut ModelState)>,
) {
    let dt = time.delta_secs().min(0.1);
    for (e, mut p, lift, mut pos, mut tf, mut state) in &mut q {
        let bank = step(&mut p, &mut state, dt);
        (pos.0, tf.rotation) = place(&p, bank, lift.0, theater.as_deref());
        let done = match p.kind {
            PassKind::Takeoff => p.pos.xz().length() > PASS_GONE,
            _ => p.t > p.length,
        };
        if done {
            commands.entity(e).despawn();
        }
    }
}

/// A pass's model's place (Bevy metres; `lift` its origin over its wheels; the helicopter
/// over the ground) and attitude.
fn place(
    p: &Passer,
    bank: f32,
    lift: f32,
    theater: Option<&crate::world::Theater>,
) -> (DVec3, Quat) {
    let mut world = at_tower(p.pos.x, p.pos.y + lift, p.pos.z);
    if p.kind == PassKind::Heli
        && let Some(t) = theater
    {
        world.y += t.hills.ground_m(world.x, -world.z) as f64;
    }
    let rotation = Quat::from_rotation_y(-p.heading)
        * Quat::from_rotation_x(p.pitch)
        * Quat::from_rotation_z(bank);
    (world, rotation)
}

/// One step of a pass: its bank.
fn step(p: &mut Passer, state: &mut ModelState, dt: f32) -> f32 {
    p.t += dt;
    let mut bank = 0.0;
    match p.kind {
        PassKind::Takeoff => take_off(p, &mut bank, state, dt),
        PassKind::Contrail | PassKind::Heli => {
            let at = p.dir * p.speed * dt;
            p.pos += Vec3::new(at.x, 0.0, at.y);
        }
    }
    bank
}

/// One step of the take-off: the roll down the runway, the rotation and lift-off, the climb
/// with the gear coming up and the flaps after it, then a climbing turn; `bank` is set in the
/// turn.
fn take_off(p: &mut Passer, bank: &mut f32, state: &mut ModelState, dt: f32) {
    let airborne = p.pos.y > 0.0 || p.speed >= ROTATE + 4.0;
    let accel = if airborne { CLIMB_ACCEL } else { ROLL_ACCEL };
    p.speed = (p.speed + accel * dt).min(CLIMB_SPEED);
    // The nose comes up from the rotation speed, the climb follows it.
    let pitch_to = if p.speed >= ROTATE { CLIMB + 0.03 } else { 0.0 };
    p.pitch += (pitch_to - p.pitch).clamp(-0.03 * dt, 0.03 * dt);
    let climb = if airborne { (p.pitch - 0.03).max(0.0) } else { 0.0 };
    let horizontal = p.speed * climb.cos();
    p.pos += Vec3::new(
        p.heading.sin() * horizontal * dt,
        p.speed * climb.sin() * dt,
        p.heading.cos() * horizontal * dt,
    );
    if !airborne {
        return;
    }
    // Seconds since the wheels left: from the climb's height.
    let up_for = p.pos.y / (p.speed * CLIMB.sin()).max(1.0);
    if up_for > GEAR_UP && state.0[5] != 0 {
        (state.0[5], state.0[6], state.0[13]) = (0, -1, 1);
    }
    if up_for > 2.0 * GEAR_UP && state.0[crate::aircraft::FLAP_SLOT] != 0 {
        state.0[crate::aircraft::FLAP_SLOT] = crate::aircraft::flap_slot_value(0.0);
    }
    // A climbing left turn (from south towards the south-east) once well clear.
    if up_for > TURN_AT {
        let left = p.heading - TURN_TO;
        if left > 0.0 {
            *bank = TURN_BANK;
            let rate = 9.81 * TURN_BANK.tan() / p.speed;
            p.heading -= (rate * dt).min(left);
        }
    }
}

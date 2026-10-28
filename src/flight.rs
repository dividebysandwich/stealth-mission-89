//! Player flight models. The classic one is a frame-based F-117A flight step, integrated per
//! second instead of per frame; the modern one (`modern`) is a point-mass aerodynamic model with
//! fly-by-wire control laws, tuned to the same performance. Both share the engine, fuel and
//! ground contact rules below.
//!
//! Classic model: State is kept in the classic units so the table constants apply directly:
//! velocity = knots * 27, angles in BAM
//! (0x10000 = 360 degrees), altitude and positions in world units ("feet"), thrust 0..100,
//! g in 1/16 g. Per-frame terms that the classic step does not divide by fs (stall nose drop) are
//! scaled with a nominal 15 frames per sim second.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::input::{Action, ActionEvent, FlightControls};
use crate::settings::{Settings, SetupSet};
use crate::world::{Theater, WorldPos, bevy_to_wu, wu_to_bevy};

pub mod heli;
pub mod modern;
pub mod terrain_follow;

pub struct FlightPlugin;

impl Plugin for FlightPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FlightConstants::default())
            .init_resource::<terrain_follow::TerrainRadar>()
            .add_systems(
                OnEnter(crate::settings::AppState::Flight),
                apply_settings.in_set(SetupSet::Mission),
            )
            .add_systems(
                Update,
                (handle_actions, terrain_follow::scan, step_flight)
                    .chain()
                    .in_set(FlightSystems)
                    .run_if(in_state(crate::settings::AppState::Flight)),
            );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlightSystems;

/// The flown aircraft's constants; crash protection picks the "no crashes" landing skill (before the combat simulation starts).
fn apply_settings(
    settings: Res<Settings>,
    mut k: ResMut<FlightConstants>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
) {
    // A campaign's rules are its host's (a coop server's).
    let no_crashes = campaign.map_or(settings.no_crashes, |c| c.rules.no_crashes);
    *k = constants_for(settings.aircraft, no_crashes);
}

/// The constants of an aircraft, crash protection picking the "no crashes" landing skill.
pub fn constants_for(aircraft: crate::aircraft::AircraftKind, no_crashes: bool) -> FlightConstants {
    let mut k = aircraft.spec().flight();
    k.landing = if no_crashes { 0 } else { 1 };
    k
}

/// BAM -> radians.
pub fn bam(v: f32) -> f32 {
    v / 65536.0 * TAU
}

/// Radians -> BAM.
pub fn to_bam(r: f32) -> f32 {
    r / TAU * 65536.0
}

/// Flight tables: the load factor of a level turn by bank, and the take-off runway direction per
/// classic theater (assets/sim/units.ron).
#[derive(Resource)]
pub struct SimTables {
    /// Sixteenths of a g by bank in steps of 360/256 degrees (`|roll| >> 8`), mirrored past 90.
    pub roll_gee: Vec<u8>,
    /// Runway direction per theater.
    pub start_dir: Vec<i16>,
}

/// Highest load factor a bank alone asks for (sixteenths of a g).
const MAX_BANK_GEE: f32 = 9.0 * 16.0;

impl SimTables {
    pub fn load() -> Self {
        let roll_gee = (0..128)
            .map(|i| {
                let bank =
                    (if i < 64 { i } else { 127 - i }) as f32 * std::f32::consts::TAU / 256.0;
                (16.0 / bank.cos().max(1e-3)).min(MAX_BANK_GEE).round() as u8
            })
            .collect();
        let text = crate::asset_path::read_to_string("sim/units.ron").unwrap_or_default();
        let start_dir = match stealth_sim::units::tables(&text) {
            Ok(t) => t.start_dir.to_vec(),
            Err(e) => {
                warn!("sim/units.ron: {e:#}");
                vec![1; 9]
            }
        };
        Self { roll_gee, start_dir }
    }
}

/// Flight model constants of the flown aircraft (F-117A defaults, F-15 SE II values in comments).
#[derive(Resource, Debug, Clone)]
pub struct FlightConstants {
    /// Gravity term in the target speed: sinMul(pitch, 120) (F-15: 80).
    pub gravity_term: f32,
    /// Thrust to knots: *720/100 (F-15: *8; the F-16's dry thrust, its afterburner adds more).
    pub thrust_to_knots: f32,
    pub max_knots: f32,
    /// 1 g stall speed at sea level, knots (F-15: 100).
    pub stall_knots: f32,
    /// Velocity lag time constant (s).
    pub speed_lag: f32,
    /// Engine spool: thrust += (set - thrust) / spool_div per second.
    pub spool_div: f32,
    /// Fuel burn every 2 s: set^2 / fuel_div + 2 (F-15: 750).
    pub fuel_div: f32,
    /// Nominal frames per sim second, for per-frame terms.
    pub fs: f32,
    /// Landing skill L: 0 no crashes, 1 easy, 2 realistic.
    pub landing: u8,
    /// Starting fuel (lbs).
    pub fuel: f32,
    /// Classic model: highest load factor (1/16 g) and roll rate factor.
    pub max_gees: f32,
    pub roll_rate: f32,
    /// Modern model airframe.
    pub aero: modern::Aero,
    /// Helicopters: the rotor, flown by the helicopter model (`heli`) instead.
    pub heli: Option<heli::Rotor>,
    /// The autopilot follows the terrain (`terrain_follow`) instead of holding the altitude.
    pub terrain_following: bool,
}

impl Default for FlightConstants {
    fn default() -> Self {
        Self {
            gravity_term: 120.0,
            thrust_to_knots: 7.2,
            max_knots: 899.0,
            stall_knots: 120.0,
            speed_lag: 16.0,
            spool_div: 4.0,
            fuel_div: 1000.0,
            fs: 15.0,
            landing: 1,
            fuel: 10000.0,
            max_gees: 128.0,
            roll_rate: 1.0,
            aero: modern::Aero::F117,
            heli: None,
            terrain_following: false,
        }
    }
}

impl FlightConstants {
    /// The engine has an afterburner.
    pub fn afterburner(&self) -> bool {
        self.heli.is_none() && self.aero.ab_thrust_ratio > 0.0
    }

    /// F-16C: the F-15's gravity term, faster, 9 g and a quicker roll.
    pub fn f16() -> Self {
        Self {
            gravity_term: 80.0,
            thrust_to_knots: 7.0,
            max_knots: 1150.0,
            stall_knots: 110.0,
            fuel_div: 750.0,
            max_gees: 144.0,
            roll_rate: 1.6,
            aero: modern::Aero::F16,
            ..default()
        }
    }

    /// Tornado IDS: as the F-16's, a little slower to roll and pull, 11,000 lb of fuel; the
    /// autopilot follows the terrain.
    pub fn tornado() -> Self {
        Self {
            gravity_term: 80.0,
            thrust_to_knots: 7.0,
            max_knots: 1150.0,
            stall_knots: 125.0,
            fuel_div: 750.0,
            fuel: 11000.0,
            max_gees: 120.0,
            roll_rate: 1.3,
            aero: modern::Aero::TORNADO,
            terrain_following: true,
            ..default()
        }
    }

    /// A-10C: slow (the classic model's top speed about 380 kt), 7.3 g, 10,700 lb of fuel burnt
    /// sparingly by its turbofans.
    pub fn a10() -> Self {
        Self {
            thrust_to_knots: 5.2,
            max_knots: 450.0,
            stall_knots: 115.0,
            fuel_div: 1300.0,
            fuel: 10700.0,
            max_gees: 117.0,
            roll_rate: 1.2,
            aero: modern::Aero::A10,
            ..default()
        }
    }

    /// UH-60A: the helicopter model; 2400 lb of fuel.
    pub fn uh60() -> Self {
        Self { fuel: 2400.0, heli: Some(heli::Rotor::UH60), ..default() }
    }

    /// AH-1Z: the helicopter model; 2600 lb of fuel.
    pub fn ah1z() -> Self {
        Self { fuel: 2600.0, heli: Some(heli::Rotor::AH1Z), ..default() }
    }

    /// RAH-66: the helicopter model; 2600 lb of fuel.
    pub fn rah66() -> Self {
        Self { fuel: 2600.0, heli: Some(heli::Rotor::RAH66), ..default() }
    }
}

/// Why the flight ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FlightEnd {
    Crash,
    BadLanding,
    Landed,
}

/// Effects of combat damage on the flight model (set by the combat simulation).
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FlightDamage {
    /// ENG: thrust capped at this percentage.
    pub throttle_cap: Option<f32>,
    /// FC: extra stick jitter added to the turbulence term.
    pub stick_jitter: f32,
    /// AV: autopilot off and unavailable.
    pub autopilot_failed: bool,
    /// FIRE (fuel leak): extra fuel units lost per 2 s while the throttle is not idle.
    pub fuel_leak: f32,
    /// More than 8 hits: the stall nose drop is doubled.
    pub heavy_stall: bool,
    /// More than 4 hits: any fast ground contact outside a runway is fatal.
    pub fragile: bool,
    /// Damaged control surfaces (realistic damage): constant roll and pitch input, oscillating
    /// roll and pitch input amplitudes (fractions of full stick) and the oscillation period (s).
    pub roll_trim: f32,
    pub pitch_trim: f32,
    pub roll_swing: f32,
    pub pitch_swing: f32,
    pub swing_period: f32,
}

impl FlightDamage {
    /// Stick input (roll, pitch; fractions of full stick) the damaged control surfaces add at
    /// `t` seconds.
    pub fn control_bias(&self, t: f32) -> (f32, f32) {
        let s = if self.swing_period > 0.0 {
            (t * std::f32::consts::TAU / self.swing_period).sin()
        } else {
            0.0
        };
        (self.roll_trim + self.roll_swing * s, self.pitch_trim + self.pitch_swing * s)
    }
}

/// The player's aircraft state.
#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Aircraft {
    /// Orientation (Bevy frame: forward -Z, up Y).
    pub orientation: Quat,
    /// knots * 27
    pub velocity: f32,
    pub thrust: f32,
    pub set_thrust: f32,
    /// The afterburner (`FlightConstants::afterburner`): its setting past the throttle's
    /// detent (None: off; 0 lowest .. 1 full), and how far it is lit (0..1, it lights and
    /// goes out over a moment). The throttle is at full dry thrust while it is set.
    #[serde(default)]
    pub set_afterburner: Option<f32>,
    #[serde(default)]
    pub afterburner: f32,
    /// A swing wing's position (`modern::SwingWing`): 0 fully forward .. 1 fully back.
    #[serde(default)]
    pub wing_sweep: f32,
    /// World units ("feet") above sea level.
    pub altitude: f32,
    pub ground_altitude: f32,
    /// wu per second along the flight path.
    pub climb_rate: f32,
    /// 1/16 g
    pub gees: f32,
    /// velocity units
    pub stall_speed: f32,
    pub fuel: f32,
    pub gear_down: bool,
    pub flaps: bool,
    pub brakes: bool,
    pub bay_open: bool,
    /// Autopilot target altitude (viewZ), 0 = off.
    pub autopilot_alt: f32,
    /// The terrain following radar's flight path angle command (rad) while the autopilot
    /// follows the terrain, flown instead of the altitude hold.
    #[serde(skip)]
    pub tf_gamma: Option<f32>,
    /// The terrain following radar's height of what lies below (altitude units), mountains
    /// and objects included (`ground_altitude` is the flat ground's).
    #[serde(skip)]
    pub radar_ground: Option<f32>,
    pub on_ground: bool,
    pub in_landing_corridor: bool,
    /// Has been airborne since the start (landing only counts afterwards).
    pub has_flown: bool,
    pub ended: Option<FlightEnd>,
    /// Angle of attack trim (BAM).
    pub trim: f32,
    /// Lift force (0..0x2000) of the classic model.
    pub lift: f32,
    /// HUD flight path marker offset below the nose (BAM).
    pub fpm: f32,
    pub roll_input: f32,
    pub pitch_input: f32,
    /// Bearing to the current waypoint (BAM), for the autopilot.
    pub waypoint_bearing: f32,
    /// Seconds of simulated time since the start.
    pub time: f32,
    pub damage: FlightDamage,
    /// Stick trim (x roll right, y pitch up; fractions of full stick), added to the stick.
    pub stick_trim: Vec2,
    /// Auto trim (`Settings::auto_trim`): with the stick released, the trim moves to hold the
    /// attitude.
    pub auto_trim: bool,
    fuel_timer: f32,
    /// Seconds a held throttle key has pushed against the afterburner's detent.
    #[serde(default)]
    detent_push: f32,
    stopped_timer: f32,
    /// Modern model state (unused by the classic one).
    modern: modern::State,
    /// Helicopter model state.
    heli: heli::State,
}

impl Aircraft {
    /// The throttle lever (0 idle .. 1 full) of the setting; with an afterburner (`ab`) dry
    /// thrust takes the travel up to its detent (`AB_DETENT`), the afterburner the travel past
    /// it and its gap.
    pub fn lever(&self, ab: bool) -> f32 {
        match self.set_afterburner {
            Some(l) if ab => AB_DETENT + AB_GAP + l * (1.0 - AB_DETENT - AB_GAP),
            _ if ab => self.set_thrust / 100.0 * AB_DETENT,
            _ => self.set_thrust / 100.0,
        }
    }

    /// Sets the throttle from a lever position (as `lever`): in the detent's gap it stays at
    /// full dry thrust.
    pub fn set_lever(&mut self, t: f32, ab: bool) {
        let t = t.clamp(0.0, 1.0);
        if !ab {
            self.set_thrust = (t * 100.0).round();
            self.set_afterburner = None;
        } else if t <= AB_DETENT {
            self.set_thrust = (t / AB_DETENT * 100.0).round();
            self.set_afterburner = None;
        } else {
            self.set_thrust = 100.0;
            self.set_afterburner = (t >= AB_DETENT + AB_GAP)
                .then(|| (t - AB_DETENT - AB_GAP) / (1.0 - AB_DETENT - AB_GAP));
        }
    }

    pub fn new_on_runway(fuel: f32) -> Self {
        Self {
            orientation: Quat::IDENTITY,
            velocity: 0.0,
            thrust: 0.0,
            set_thrust: 0.0,
            set_afterburner: None,
            afterburner: 0.0,
            wing_sweep: 0.0,
            altitude: 0.0,
            ground_altitude: 0.0,
            climb_rate: 0.0,
            gees: 16.0,
            stall_speed: 0.0,
            fuel,
            gear_down: true,
            flaps: false,
            brakes: false,
            bay_open: false,
            autopilot_alt: 0.0,
            tf_gamma: None,
            radar_ground: None,
            on_ground: true,
            in_landing_corridor: true,
            has_flown: false,
            ended: None,
            trim: 0.0,
            lift: 0.0,
            roll_input: 0.0,
            pitch_input: 0.0,
            waypoint_bearing: 0.0,
            time: 0.0,
            damage: FlightDamage::default(),
            stick_trim: Vec2::ZERO,
            auto_trim: false,
            fuel_timer: 0.0,
            detent_push: 0.0,
            stopped_timer: 0.0,
            fpm: 0.0,
            modern: modern::State::default(),
            heli: heli::State::turning(),
        }
    }

    /// Rotor speed (1 = 100%; helicopters).
    pub fn rotor_speed(&self) -> f32 {
        self.heli.nr
    }

    /// Main rotor angle as drawn (rad; helicopters).
    pub fn rotor_angle(&self) -> f32 {
        self.heli.angle
    }

    /// Engine torque (1 = 100%; helicopters).
    pub fn torque(&self) -> f32 {
        self.heli.torque
    }

    /// Translational lift 0..1 (helicopters): the blade slap in the rotor sound.
    pub fn translational_lift(&self) -> f32 {
        self.heli.etl
    }

    pub fn knots(&self) -> f32 {
        self.velocity / 27.0
    }

    /// Autopilot engaged?
    pub fn autopilot(&self) -> bool {
        self.autopilot_alt > 0.0
    }

    /// The autopilot follows the terrain (the terrain following radar has a command).
    pub fn terrain_following(&self) -> bool {
        self.autopilot() && self.tf_gamma.is_some()
    }

    /// Direction of flight (Bevy frame, unit vector): the modern model's velocity, or for the
    /// classic model the nose lowered by the angle of attack trim.
    pub fn path_dir(&self) -> Vec3 {
        if let Some(v) = self.modern.velocity().or(self.heli.velocity()) {
            return v;
        }
        let (h, p, _) = self.euler();
        Self::from_euler(h, p - bam(self.trim), 0.0) * Vec3::NEG_Z
    }

    /// Compressed altitude used for rendering, AI and detection.
    pub fn view_z(&self) -> f32 {
        view_z(self.altitude)
    }

    /// (heading, pitch, roll) in radians; heading clockwise from north, roll positive = right wing down.
    pub fn euler(&self) -> (f32, f32, f32) {
        let f = self.orientation * Vec3::NEG_Z;
        let r = self.orientation * Vec3::X;
        let u = self.orientation * Vec3::Y;
        let heading = f.x.atan2(-f.z).rem_euclid(TAU);
        let pitch = f.y.clamp(-1.0, 1.0).asin();
        let roll = (-r.y).atan2(u.y);
        (heading, pitch, roll)
    }

    pub fn from_euler(heading: f32, pitch: f32, roll: f32) -> Quat {
        Quat::from_rotation_y(-heading)
            * Quat::from_rotation_x(pitch)
            * Quat::from_rotation_z(-roll)
    }
}

/// Altitude compression (above 8192 halved, above 16384 quartered).
pub fn view_z(alt: f32) -> f32 {
    if alt < 8192.0 {
        alt
    } else if alt < 16384.0 {
        8192.0 + (alt - 8192.0) / 2.0
    } else {
        12288.0 + (alt - 16384.0) / 4.0
    }
}

/// Marks the player's aircraft.
#[derive(Component)]
pub struct Player;

fn handle_actions(
    mut actions: MessageReader<ActionEvent>,
    mut q: Query<&mut Aircraft, With<Player>>,
    k: Res<FlightConstants>,
) {
    let Ok(mut a) = q.single_mut() else {
        actions.clear();
        return;
    };
    let heli = k.heli.is_some();
    let ab = k.afterburner();
    for ActionEvent(action) in actions.read() {
        match action {
            // Helicopters: the collective moves while its keys are held
            // (`FlightControls::collective_rate`); fixed gear unless it retracts, no autopilot,
            // the brakes are the parking brake.
            Action::ThrottleUp | Action::ThrottleDown | Action::Autopilot if heli => {}
            Action::Gear if k.heli.is_some_and(|r| r.retract_gear.is_none()) => {}
            // '=' / '-' / '+' / '_' (they do not release the brakes in the F-117A). Past full
            // dry thrust they step through the afterburner; full power lights it at full dry
            // thrust.
            Action::ThrottleUp if ab && a.set_thrust >= 100.0 => {
                a.set_afterburner = Some(a.set_afterburner.map_or(0.0, |l| (l + AB_STEP).min(1.0)))
            }
            Action::ThrottleUp => {
                a.set_thrust =
                    (a.set_thrust + if a.set_thrust < 10.0 { 5.0 } else { 10.0 }).min(100.0)
            }
            Action::ThrottleDown if a.set_afterburner.is_some() => {
                a.set_afterburner = a.set_afterburner.map(|l| l - AB_STEP).filter(|&l| l >= 0.0)
            }
            Action::ThrottleDown => a.set_thrust = (a.set_thrust - 10.0).max(0.0),
            Action::ThrottleMax if ab && a.set_thrust >= 100.0 => a.set_afterburner = Some(1.0),
            Action::ThrottleMax => a.set_thrust = 100.0,
            Action::ThrottleIdle => {
                a.set_thrust = 0.0;
                a.set_afterburner = None;
            }
            Action::Gear => {
                if !a.on_ground {
                    a.gear_down = !a.gear_down;
                }
            }
            Action::Flaps => a.flaps = !a.flaps,
            Action::TrimReset => a.stick_trim = Vec2::ZERO,
            Action::Brakes => a.brakes = !a.brakes,
            // Bay doors (8) are run by the combat simulation, which mirrors them into `bay_open`.
            Action::Autopilot if !a.damage.autopilot_failed => {
                // Holds the current altitude, at least 500 ft.
                a.autopilot_alt = if a.autopilot() { 0.0 } else { a.view_z().max(500.0) };
            }
            _ => {}
        }
    }
}

/// Largest trim (fractions of full stick), and how fast the trim switch moves it (per second).
const TRIM_MAX: f32 = 0.6;
const TRIM_RATE: f32 = 0.1;
/// Stick deflection below which the stick counts as released for the auto trim.
const TRIM_STICK_FREE: f32 = 0.05;

impl Aircraft {
    /// Forgets the flight models' velocity vectors, so they start again from `velocity` and
    /// `orientation` (after moving the aircraft).
    pub fn restart_motion(&mut self) {
        self.modern = modern::State::default();
        self.heli = heli::State::default();
    }

    /// Moves a swing wing towards where its schedule puts it for the Mach number (the classic
    /// model's as well, which only draws it).
    fn sweep_wing(&mut self, k: &FlightConstants, dt: f32) {
        let Some(w) = k.aero.swing_wing else { return };
        if self.ended.is_some() {
            return;
        }
        let (_, sound) = modern::atmosphere(self.altitude);
        let target = w.scheduled(self.velocity / 16.0 / sound, self.flaps);
        let step = dt / w.travel;
        self.wing_sweep += (target - self.wing_sweep).clamp(-step, step);
    }

    /// Moves the trim by `d` (fractions of full stick), within its limits.
    fn move_trim(&mut self, d: Vec2) {
        self.stick_trim =
            (self.stick_trim + d).clamp(Vec2::splat(-TRIM_MAX), Vec2::splat(TRIM_MAX));
    }
}

/// (roll, pitch) stick released, for the auto trim.
fn stick_released(controls: &FlightControls) -> (bool, bool) {
    (controls.roll.abs() < TRIM_STICK_FREE, controls.pitch.abs() < TRIM_STICK_FREE)
}

/// Stick axis -1..1 to the classic -7..7 steps (joy = 0x80 + deflection).
fn stick_steps(axis: f32) -> f32 {
    let joy = (128.0 + axis.clamp(-1.0, 1.0) * 127.0).round() as i32;
    let mut r = (joy >> 4) - 8;
    if r < 0 {
        r += 1;
    }
    r as f32
}

/// Signed BAM difference a - b in -0x8000..0x8000.
fn bam_diff(a: f32, b: f32) -> f32 {
    (a - b + 32768.0).rem_euclid(65536.0) - 32768.0
}

#[allow(clippy::too_many_arguments)]
fn step_flight(
    time: Res<Time>,
    controls: Res<FlightControls>,
    k: Res<FlightConstants>,
    tables: Res<SimTables>,
    settings: Res<Settings>,
    theater: Option<Res<Theater>>,
    mut q: Query<(&mut Aircraft, &mut WorldPos, &mut Transform), With<Player>>,
) {
    // Sub-steps (at most 1/60 s classic, 1/120 s modern) keep accelerated test runs accurate.
    let total = time.delta_secs().min(1.0);
    let max_dt = if settings.classic_flight && k.heli.is_none() { 1.0 / 60.0 } else { 1.0 / 120.0 };
    for (mut a, mut pos, mut tf) in &mut q {
        a.auto_trim = settings.auto_trim;
        let mut remaining = total;
        while remaining > 1e-6 {
            let dt = remaining.min(max_dt);
            remaining -= dt;
            if a.ended.is_none() {
                a.move_trim(controls.trim.clamp(Vec2::NEG_ONE, Vec2::ONE) * TRIM_RATE * dt);
            }
            if let Some(r) = &k.heli {
                heli::step(&mut a, &mut pos, &mut tf, dt, &controls, &k, r, theater.as_deref());
            } else if settings.classic_flight {
                step_one(&mut a, &mut pos, &mut tf, dt, &controls, &k, &tables, theater.as_deref());
            } else {
                modern::step(&mut a, &mut pos, &mut tf, dt, &controls, &k, theater.as_deref());
            }
            a.sweep_wing(&k, dt);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn step_one(
    a: &mut Aircraft,
    pos: &mut WorldPos,
    tf: &mut Transform,
    dt: f32,
    controls: &FlightControls,
    k: &FlightConstants,
    tables: &SimTables,
    theater: Option<&Theater>,
) {
    let l = k.landing as f32;
    {
        // The clock keeps running after the end (scripts and screenshots use it).
        a.time += dt;
        if a.ended.is_some() {
            return;
        }
        let (_, pitch, roll) = a.euler();
        let (head_b, pitch_b, roll_b) = {
            let (h, p, r) = a.euler();
            (to_bam(h), to_bam(p), to_bam(r))
        };
        let air = !a.on_ground;
        let vz = a.view_z();
        let knots = a.knots();

        // --- stick
        let r = stick_steps(controls.roll);
        let p = stick_steps(controls.pitch);
        let mut roll_input = (r.abs() + 2.0) * r * 2.0 * k.roll_rate;
        let mut pitch_input = if p < 0.0 { (6.0 * p / 2.0).trunc() } else { 6.0 * p };
        if !air && pitch_input < 0.0 && pitch_b <= 0.0 {
            pitch_input = 0.0;
        }
        if roll_input != 0.0 || pitch_input != 0.0 || a.damage.autopilot_failed {
            a.autopilot_alt = 0.0;
        }

        // --- autopilot: steer to the waypoint bearing, hold altitude
        if a.autopilot() {
            let e = bam_diff(a.waypoint_bearing, head_b).clamp(-5120.0, 5120.0) * 2.0;
            roll_input = ((e - roll_b) / 64.0).clamp(-24.0, 24.0);
            let t = match a.tf_gamma {
                // Terrain following: the nose where it puts the flight path on the command.
                Some(g) => {
                    let path = to_bam(a.path_dir().y.clamp(-1.0, 1.0).asin());
                    (pitch_b + to_bam(g) - path).clamp(-8192.0, 8192.0)
                }
                None => ((a.autopilot_alt - vz) * 16.0 - a.trim).clamp(-5120.0, 3072.0),
            };
            let gain = if a.tf_gamma.is_some() { 24.0 } else { 8.0 };
            pitch_input = ((t - pitch_b) / 128.0).clamp(-gain, gain);
        }

        // --- turbulence (stronger below 500 ft and with the gear down)
        let mut turb = if k.landing > 0 { ((500.0 - vz) * knots / 8192.0).floor() } else { 0.0 };
        if a.gear_down {
            turb += ((knots - 200.0) / 16.0).clamp(0.0, 32.0);
        }
        if a.flaps {
            turb = (turb / 2.0).floor();
        }
        turb += a.damage.stick_jitter;
        if air {
            let (r, p) = a.damage.control_bias(a.time);
            roll_input += (r + a.stick_trim.x) * 126.0;
            pitch_input += (p + a.stick_trim.y) * 42.0;
            // Auto trim: with the stick released, the trim takes out what still rotates the
            // aircraft (not in a stall, whose nose drop it would fight).
            let (roll_free, pitch_free) = stick_released(controls);
            if a.auto_trim && !a.autopilot() && a.velocity > a.stall_speed {
                let d = Vec2::new(
                    if roll_free { -roll_input / 126.0 } else { 0.0 },
                    if pitch_free { -pitch_input / 42.0 } else { 0.0 },
                );
                a.move_trim(d * CLASSIC_TRIM_GAIN * dt);
            }
        }
        if turb > 0.0 && air {
            roll_input += rand::random_range(0.0..turb) - turb / 2.0;
            pitch_input += (rand::random_range(0.0..turb) - turb / 2.0) / 2.0;
        }

        // --- auto pull-up ("barometric altimeter"; the no-crash and easy skills, here
        // crash protection only, so the ground can always be hit otherwise)
        // (Not while terrain following, which flies lower and clears the ground itself.)
        if !a.gear_down
            && pitch_input <= 0.0
            && a.velocity > a.stall_speed
            && k.landing == 0
            && roll_b.abs() < 12288.0
            && !a.terrain_following()
        {
            let kk = ((a.trim - pitch_b) / 8.0 - vz + 200.0) / 8.0;
            if kk > 0.0 {
                pitch_input = kk.clamp(0.0, 32.0);
            }
        }

        engine(a, controls, k, dt);

        // --- g-load: bank angle table plus the full pull
        let idx = ((roll_b.abs() as i32 >> 8) & 0x7F) as usize;
        let table_g = tables.roll_gee.get(idx).copied().unwrap_or(16) as f32;
        a.gees = table_g;
        if air {
            a.gees += pitch_input;
        }
        if a.gees > k.max_gees {
            a.gees = k.max_gees;
            pitch_input = (k.max_gees - table_g).clamp(0.0, pitch_input.max(0.0));
        }

        // --- speed, drag, stall, lift
        let thrust = a.thrust + CLASSIC_AFTERBURNER * a.afterburner;
        let mut s = (thrust - pitch.sin() * k.gravity_term) * k.thrust_to_knots;
        let mut cs = k.stall_knots * ((a.altitude / 64.0).floor() + 1024.0) / 1024.0;
        s *= ((vz / 128.0).floor() + 1024.0) / 1024.0;
        s *= (100.0 - (a.fuel / 512.0).floor()) / 90.0;
        s *= (k.max_gees - a.gees) / k.max_gees;
        cs = ((a.gees.max(0.0) * 4.0).sqrt().floor() * cs / 8.0).abs();
        if a.flaps {
            cs -= cs / 4.0;
            s -= s / 16.0;
        }
        if a.gear_down {
            s -= s / 8.0;
        }
        a.stall_speed = cs * 27.0;
        let target = s.clamp(0.0, k.max_knots) * 27.0;
        a.velocity += (target - a.velocity) / k.speed_lag * dt;
        let x = ((target - a.velocity) / 4.0).clamp(0.0, a.stall_speed);
        a.lift = ((a.stall_speed - x) * 3072.0 / (a.velocity.abs() + 1.0)).min(8192.0);
        a.trim = roll.cos() * (a.lift - 512.0);
        let hud_lift = (a.stall_speed * 3072.0 / (a.velocity.abs() + 1.0)).min(8192.0);
        a.fpm = roll.cos() * (hud_lift - 512.0);

        // --- brakes (ground brakes depend on the landing skill; airborne speed brake)
        if a.brakes {
            if !air {
                a.velocity -= (4.0 - l) * 216.0 * dt;
            } else {
                a.velocity -= a.velocity / 16.0 * dt;
            }
        }
        a.velocity = a.velocity.max(0.0);

        // --- rotation
        let mut yaw = (roll.sin() * a.gees * 16.0 * 128.0) / ((a.velocity / 256.0).floor() + 32.0)
            * pitch.cos();
        // Rudder (extension): up to 10 deg/s in the air.
        let rudder = controls.yaw.clamp(-1.0, 1.0);
        yaw += rudder * 1820.0;
        if !air {
            // Nosewheel steering: by the stick (classic), or by the rudder.
            yaw = if rudder != 0.0 { rudder * 8064.0 } else { roll_input * 64.0 };
            roll_input = 0.0;
            if a.knots() < cs {
                pitch_input = 0.0;
            }
        }
        let d_roll = bam(roll_input * 128.0 * dt);
        let d_pitch = bam(pitch_input * 128.0 * dt);
        let d_yaw = bam(yaw * dt);
        a.orientation = (Quat::from_rotation_y(-d_yaw)
            * a.orientation
            * Quat::from_rotation_x(d_pitch)
            * Quat::from_rotation_z(-d_roll))
        .normalize();

        // Stall: the nose drops (per classic frame, scaled with fs).
        if air && a.stall_speed > a.velocity {
            let shift = (if k.landing == 2 { 2.0 } else { 4.0 })
                / if a.damage.heavy_stall { 2.0 } else { 1.0 };
            let drop = bam((a.stall_speed - a.velocity) / shift * k.fs * dt);
            let (h, p, r) = a.euler();
            a.orientation = Aircraft::from_euler(h, (p - drop).max(-1.5), r);
        }
        if !air {
            let (h, p, _) = a.euler();
            let p = if p < 0.0 || a.knots() < cs { 0.0 } else { p };
            a.orientation = Aircraft::from_euler(h, p, 0.0);
        }

        // --- integration
        let (head2, pitch2, roll2) = a.euler();
        let prev_alt = a.altitude;
        a.climb_rate = a.velocity / 16.0 * (pitch2 - bam(a.trim)).sin();
        a.altitude += a.climb_rate * dt;
        let horiz = pitch2.cos() * a.velocity / 16.0;
        let (mut east, mut north, _) = bevy_to_wu(pos.0);
        east += (horiz * head2.sin() * dt) as f64;
        north += (horiz * head2.cos() * dt) as f64;

        ground_contact(a, k, false, roll2, prev_alt, east, north, theater, dt);

        a.roll_input = roll_input;
        a.pitch_input = pitch_input;
        // The 3D view uses the compressed altitude.
        pos.0 = wu_to_bevy(east, north, a.view_z() as f64);
        tf.rotation = a.orientation;
    }
}

/// Classic model auto trim: the share of the remaining rotation trimmed out per second.
const CLASSIC_TRIM_GAIN: f32 = 2.0;

/// How fast a held throttle key moves the throttle (percent per second).
const THROTTLE_RATE: f32 = 50.0;

/// Throttle lever travel at the afterburner's detent (full dry thrust), and the gap past it
/// before the afterburner lights: the detent's deadzone.
pub const AB_DETENT: f32 = 0.82;
pub const AB_GAP: f32 = 0.05;
/// Seconds a held throttle key pushes at full dry thrust before it passes the detent; the
/// afterburner setting a throttle key press steps.
const AB_PUSH: f32 = 0.5;
const AB_STEP: f32 = 0.25;
/// The lowest afterburner setting's share of the afterburner's thrust.
pub const AB_MIN: f32 = 0.3;
/// Shares of the afterburner's thrust it gains and loses a second, lighting and going out.
const AB_LIGHT: f32 = 1.5;
const AB_OUT: f32 = 4.0;
/// The full afterburner's fuel burn, in full dry thrust burns.
const AB_FUEL: f32 = 3.0;
/// The classic model: thrust (percent) the full afterburner adds.
const CLASSIC_AFTERBURNER: f32 = 22.0;

/// Thrust spool and fuel burn (engine damage caps the thrust).
fn engine(a: &mut Aircraft, controls: &FlightControls, k: &FlightConstants, dt: f32) {
    let ab = k.afterburner();
    if let Some(t) = controls.throttle {
        a.set_lever(t, ab);
    } else if controls.throttle_rate > 0.0 && ab && a.set_thrust >= 100.0 {
        // A held key pushes through the detent after a moment, then through the afterburner.
        match a.set_afterburner {
            Some(l) => {
                let l = l + controls.throttle_rate * THROTTLE_RATE / 100.0 * dt;
                a.set_afterburner = Some(l.min(1.0));
            }
            None => {
                a.detent_push += dt;
                if a.detent_push >= AB_PUSH {
                    a.set_afterburner = Some(0.0);
                }
            }
        }
    } else if controls.throttle_rate < 0.0 && a.set_afterburner.is_some() {
        let l =
            a.set_afterburner.unwrap_or(0.0) + controls.throttle_rate * THROTTLE_RATE / 100.0 * dt;
        a.set_afterburner = (l >= 0.0).then_some(l);
    } else if controls.throttle_rate != 0.0 {
        a.set_thrust =
            (a.set_thrust + controls.throttle_rate * THROTTLE_RATE * dt).clamp(0.0, 100.0);
    }
    if controls.throttle_rate <= 0.0 || a.set_afterburner.is_some() {
        a.detent_push = 0.0;
    }
    if !ab {
        a.set_afterburner = None;
    }
    let set = a.damage.throttle_cap.map_or(a.set_thrust, |cap| a.set_thrust.min(cap));
    // The afterburner lights once the engine is at full dry thrust (not with a damaged one).
    let burn = match a.set_afterburner {
        Some(l) if set >= 100.0 && a.thrust >= 95.0 && a.fuel > 0.0 => AB_MIN + (1.0 - AB_MIN) * l,
        _ => 0.0,
    };
    a.afterburner = if burn > a.afterburner {
        (a.afterburner + AB_LIGHT * dt).min(burn)
    } else {
        (a.afterburner - AB_OUT * dt).max(burn)
    };
    a.thrust += (set - a.thrust) / k.spool_div * dt;
    if set > a.thrust {
        a.thrust = (a.thrust + k.fs * dt).min(set);
    } else {
        a.thrust = set;
    }
    a.fuel_timer += dt;
    while a.fuel_timer >= 2.0 {
        a.fuel_timer -= 2.0;
        if a.set_thrust != 0.0 {
            a.fuel -= (a.set_thrust * a.set_thrust / k.fuel_div).floor() + 2.0 + a.damage.fuel_leak;
        }
        a.fuel -= (a.afterburner * AB_FUEL * 100.0 * 100.0 / k.fuel_div).floor();
    }
    if a.fuel <= 0.0 {
        a.fuel = 0.0;
        a.thrust = 0.0;
        a.afterburner = 0.0;
    }
}

/// Ground altitude and landing corridor, touchdown, ground crashes and the safe landing end.
/// `a.altitude` has been integrated from `prev_alt`; `a.climb_rate` is the sink at contact.
/// A helicopter (`heli`) judges its own touchdowns, and its base is the whole airfield.
#[allow(clippy::too_many_arguments)]
fn ground_contact(
    a: &mut Aircraft,
    k: &FlightConstants,
    heli: bool,
    roll: f32,
    prev_alt: f32,
    east: f64,
    north: f64,
    theater: Option<&Theater>,
    dt: f32,
) {
    // --- landing corridor at the nearest friendly base
    let (corridor, ground) =
        theater.map(|t| landing_corridor(t, east, north, k.landing, heli)).unwrap_or((false, 0.0));
    a.in_landing_corridor = corridor;
    a.ground_altitude = ground;
    a.altitude = a.altitude.clamp(a.ground_altitude, 60000.0);
    let on_ground_now = a.altitude <= a.ground_altitude;
    if on_ground_now && prev_alt > a.ground_altitude && !heli {
        touchdown(a, roll, k.landing as f32);
    }
    if on_ground_now {
        a.climb_rate = 0.0;
        if !a.in_landing_corridor && a.knots() > 50.0 && (k.landing != 0 || a.damage.fragile) {
            a.ended = Some(FlightEnd::Crash);
            info!("crash: ground contact outside a runway at {:.0} kt", a.knots());
        }
    } else {
        a.has_flown = true;
    }
    a.on_ground = on_ground_now;

    // Stopped on a friendly runway after flying: "Safe Landing" ends the mission.
    if a.on_ground && a.in_landing_corridor && a.has_flown && a.knots() <= 1.0 {
        a.stopped_timer += dt;
        if a.stopped_timer > 2.0 && a.ended.is_none() {
            a.ended = Some(FlightEnd::Landed);
            info!("safe landing");
        }
    } else {
        a.stopped_timer = 0.0;
    }
}

/// Touchdown in a landing corridor.
fn touchdown(a: &mut Aircraft, roll: f32, l: f32) {
    if !a.in_landing_corridor {
        return;
    }
    let sink = (-a.climb_rate).max(0.0);
    let limit = if a.ground_altitude > 0.0 { 256.0 } else { 128.0 };
    let max_roll = if l > 0.0 { bam(24.0 / l * 256.0) } else { f32::MAX };
    if sink * l > limit || (l > 0.0 && (!a.gear_down || roll.abs() > max_roll)) {
        a.ended = Some(FlightEnd::BadLanding);
        info!("bad landing: sink {sink:.0} gear {} roll {:.1}", a.gear_down, roll.to_degrees());
    }
}

/// (in corridor, ground altitude): runway box around friendly bases (flags 0x500, not 0x800):
/// |dx| <= 5, |dy| <= 64 map units (+4/+8 on easy); carriers 8 x 30 at deck height 128. A
/// helicopter's is the whole airfield, |dx| <= 16 (the sim's `objectives`, keep in step).
fn landing_corridor(t: &Theater, east: f64, north: f64, landing: u8, heli: bool) -> (bool, f32) {
    let (mx, my) = (east / 32.0, 32768.0 - north / 32.0);
    let (ex, ey) = if landing == 1 { (4.0, 8.0) } else { (0.0, 0.0) };
    for o in &t.world.objects {
        let flags = o.target_flags as u16;
        if flags & 0x500 == 0 || flags & 0x800 != 0 {
            continue;
        }
        let carrier = flags & 0x200 != 0;
        let (bx, by) = if carrier {
            (8.0, 30.0)
        } else if heli {
            (HELI_BASE_HALF_WIDTH, 64.0)
        } else {
            (5.0, 64.0)
        };
        if (mx - o.x as f64).abs() <= bx + ex && (my - o.y as f64).abs() <= by + ey {
            return (true, if carrier { 128.0 } else { 0.0 });
        }
    }
    (false, 0.0)
}

/// Half width (map units) of a base's landing area for helicopters: the airfield beside the
/// runway (`landing_corridor`).
pub const HELI_BASE_HALF_WIDTH: f64 = 16.0;

#[cfg(test)]
mod tests {
    use super::*;

    /// Flies one model for `secs` (no theater: no runways, ground at 0), with the controls set
    /// by `ctl` every step. Returns the aircraft and the distance flown (wu).
    fn fly(
        classic: bool,
        a: Aircraft,
        secs: f32,
        landing: u8,
        ctl: impl FnMut(&Aircraft, &mut FlightControls),
    ) -> (Aircraft, f64) {
        fly_with(FlightConstants { landing, ..default() }, classic, a, secs, ctl)
    }

    fn fly_with(
        k: FlightConstants,
        classic: bool,
        mut a: Aircraft,
        secs: f32,
        mut ctl: impl FnMut(&Aircraft, &mut FlightControls),
    ) -> (Aircraft, f64) {
        let tables = SimTables::load();
        let mut pos = WorldPos(wu_to_bevy(0.0, 0.0, a.view_z() as f64));
        let mut tf = Transform::default();
        let mut c = FlightControls::default();
        let dt = 1.0 / 120.0;
        for _ in 0..(secs / dt) as usize {
            ctl(&a, &mut c);
            if classic {
                step_one(&mut a, &mut pos, &mut tf, dt, &c, &k, &tables, None);
            } else {
                modern::step(&mut a, &mut pos, &mut tf, dt, &c, &k, None);
            }
            a.sweep_wing(&k, dt);
        }
        let (e, n, _) = bevy_to_wu(pos.0);
        (a, e.hypot(n))
    }

    pub(super) fn airborne(alt: f32, kts: f32, thrust: f32) -> Aircraft {
        let mut a = Aircraft::new_on_runway(10000.0);
        a.altitude = alt;
        a.on_ground = false;
        a.has_flown = true;
        a.gear_down = false;
        a.velocity = kts * 27.0;
        a.set_thrust = thrust;
        a.thrust = thrust;
        a
    }

    /// Altitude hold by the autopilot (both models), straight ahead.
    fn level(mut a: Aircraft) -> Aircraft {
        a.autopilot_alt = a.view_z();
        a.waypoint_bearing = 0.0;
        a
    }

    #[test]
    fn performance_compared() {
        for classic in [true, false] {
            let name = if classic { "classic" } else { "modern" };
            for alt in [1000.0, 15000.0] {
                let (a, _) = fly(classic, level(airborne(alt, 400.0, 100.0)), 240.0, 1, |_, _| {});
                println!(
                    "{name}: max speed at {alt} ft: {:.0} kt (alt {:.0})",
                    a.knots(),
                    a.altitude
                );
                assert!((480.0..680.0).contains(&a.knots()), "{name} max speed {}", a.knots());
            }
            let mut t500 = None;
            let mut t = 0.0;
            fly(classic, level(airborne(1000.0, 300.0, 100.0)), 90.0, 1, |a, _| {
                t += 1.0 / 120.0;
                if a.knots() >= 500.0 && t500.is_none() {
                    t500 = Some(t);
                }
            });
            println!("{name}: 300 to 500 kt at full throttle: {t500:?} s");
            // Idle at 1000 ft with the altitude held: speed it falls to.
            let (a, _) = fly(classic, level(airborne(1000.0, 300.0, 0.0)), 120.0, 1, |_, _| {});
            println!(
                "{name}: idle, altitude held: {:.0} kt alt {:.0} stall {:.0} {:?}",
                a.knots(),
                a.altitude,
                a.stall_speed / 27.0,
                a.ended
            );
            // Take-off run: full throttle, pull at 160 kt.
            let mut lift_off = None;
            let mut t = 0.0;
            // No theater, so no runway: crash protection keeps the ground roll legal.
            let (a, _) = fly(classic, Aircraft::new_on_runway(10000.0), 60.0, 0, |a, c| {
                c.throttle = Some(1.0);
                c.pitch = if a.knots() > 160.0 { 0.6 } else { 0.0 };
                t += 1.0 / 120.0;
                if !a.on_ground && lift_off.is_none() {
                    lift_off = Some((t, a.knots()));
                }
            });
            println!(
                "{name}: lift-off {lift_off:?}, after 60 s {:.0} kt alt {:.0}",
                a.knots(),
                a.altitude
            );
            assert!(lift_off.is_some(), "{name} did not take off");
            // Full pull, bank 75 deg, 400 kt at 10000 ft for 10 s: heading change and speed.
            let mut a0 = airborne(10000.0, 400.0, 100.0);
            a0.orientation = Aircraft::from_euler(0.0, 0.0, 75f32.to_radians());
            let (a, _) = fly(classic, a0, 10.0, 1, |_, c| c.pitch = 1.0);
            let (h, p, r) = a.euler();
            println!(
                "{name}: 10 s full pull: heading {:.0} pitch {:.0} roll {:.0} {:.0} kt {:.1} g alt {:.0}",
                h.to_degrees(),
                p.to_degrees(),
                r.to_degrees(),
                a.knots(),
                a.gees / 16.0,
                a.altitude
            );
            // Roll rate: full stick for 1 s at 400 kt.
            let (a, _) = fly(classic, airborne(10000.0, 400.0, 70.0), 1.0, 1, |_, c| c.roll = 1.0);
            println!("{name}: roll after 1 s full stick {:.0} deg", a.euler().2.to_degrees());
            // Hands off at 400 kt: the path stays level.
            let (a, _) = fly(classic, airborne(5000.0, 400.0, 70.0), 20.0, 1, |_, _| {});
            println!(
                "{name}: 20 s hands off: alt {:.0} pitch {:.1} {:.0} kt",
                a.altitude,
                a.euler().1.to_degrees(),
                a.knots()
            );
        }
    }

    /// Full afterburner.
    fn reheat(mut a: Aircraft) -> Aircraft {
        a.set_afterburner = Some(1.0);
        a.afterburner = 1.0;
        a
    }

    /// Tornado IDS (modern, the wings sweeping): subsonic low down without the afterburners,
    /// about Mach 1.2 with them, past Mach 1.6 high up; a 1 g stall about 130 kt with the slats
    /// and flaps out, a good deal faster clean.
    #[test]
    fn tornado_performance() {
        let k = FlightConstants::tornado;
        let run = |a: Aircraft, secs: f32| fly_with(k(), false, level(a), secs, |_, _| {}).0;
        let a = run(airborne(1000.0, 500.0, 100.0), 240.0);
        println!(
            "Tornado: dry max speed at 1000 ft: {:.0} kt, sweep {:.2}",
            a.knots(),
            a.wing_sweep
        );
        assert!((560.0..680.0).contains(&a.knots()), "dry sea level {}", a.knots());
        let a = run(reheat(airborne(1000.0, 500.0, 100.0)), 240.0);
        println!("Tornado: max speed at 1000 ft: {:.0} kt, sweep {:.2}", a.knots(), a.wing_sweep);
        assert!((700.0..840.0).contains(&a.knots()), "sea level {}", a.knots());
        assert!(a.wing_sweep > 0.99, "swept back {}", a.wing_sweep);
        let a = run(reheat(airborne(36000.0, 600.0, 100.0)), 300.0);
        println!("Tornado: max speed at 36000 ft: {:.0} kt", a.knots());
        assert!(a.knots() > 920.0, "at altitude {}", a.knots());
        // The stall speeds the warnings use, at 1 g.
        let mut slow = airborne(1000.0, 180.0, 60.0);
        let clean = run(slow.clone(), 2.0);
        slow.flaps = true;
        let flaps = run(slow, 6.0);
        let (clean, flaps) = (clean.stall_speed / 27.0, flaps.stall_speed / 27.0);
        println!("Tornado: stall {clean:.0} kt clean, {flaps:.0} kt with the flaps");
        assert!((115.0..145.0).contains(&flaps), "flaps {flaps}");
        assert!(clean > flaps + 20.0, "clean {clean}");
    }

    /// A-10C (modern): about 380 kt low down at full throttle, no afterburner, a 1 g stall about
    /// 120 kt clean and slower with the flaps.
    #[test]
    fn a10_performance() {
        let k = FlightConstants::a10;
        assert!(!k().afterburner());
        let run = |a: Aircraft, secs: f32| fly_with(k(), false, level(a), secs, |_, _| {}).0;
        let a = run(airborne(1000.0, 300.0, 100.0), 240.0);
        println!("A-10: max speed at 1000 ft: {:.0} kt", a.knots());
        assert!((355.0..405.0).contains(&a.knots()), "sea level {}", a.knots());
        let a = run(airborne(20000.0, 300.0, 100.0), 300.0);
        println!("A-10: max speed at 20000 ft: {:.0} kt", a.knots());
        let mut slow = airborne(1000.0, 170.0, 60.0);
        let clean = run(slow.clone(), 2.0);
        slow.flaps = true;
        let flaps = run(slow, 6.0);
        let (clean, flaps) = (clean.stall_speed / 27.0, flaps.stall_speed / 27.0);
        println!("A-10: stall {clean:.0} kt clean, {flaps:.0} kt with the flaps");
        assert!((110.0..135.0).contains(&clean), "clean {clean}");
        assert!(flaps < clean - 8.0, "flaps {flaps}");
        // The classic model's top speed.
        let c = fly_with(k(), true, level(airborne(1000.0, 300.0, 100.0)), 120.0, |_, _| {}).0;
        println!("A-10 classic: max speed {:.0} kt", c.knots());
        assert!((340.0..420.0).contains(&c.knots()), "classic {}", c.knots());
    }

    /// F-16: Mach 1.2 class at sea level, past 1050 kt at 36000 ft (modern) with the afterburner,
    /// subsonic without it; 9 g and a fast roll.
    #[test]
    fn f16_performance() {
        let k = FlightConstants::f16;
        let (a, _) = fly_with(k(), false, level(airborne(1000.0, 500.0, 100.0)), 240.0, |_, _| {});
        println!("F-16 modern: dry max speed at 1000 ft: {:.0} kt", a.knots());
        assert!((580.0..680.0).contains(&a.knots()), "dry sea level {}", a.knots());
        let (a, _) = fly_with(k(), false, level(airborne(36000.0, 500.0, 100.0)), 400.0, |_, _| {});
        println!("F-16 modern: dry max speed at 36000 ft: {:.0} kt", a.knots());
        assert!((520.0..650.0).contains(&a.knots()), "dry at altitude {}", a.knots());
        // Half throttle is a cruise, not nearly full speed.
        let (a, _) = fly_with(k(), false, level(airborne(10000.0, 450.0, 50.0)), 240.0, |_, _| {});
        println!("F-16 modern: half throttle at 10000 ft: {:.0} kt", a.knots());
        assert!((400.0..540.0).contains(&a.knots()), "half throttle {}", a.knots());
        for classic in [false, true] {
            let name = if classic { "classic" } else { "modern" };
            let (a, _) = fly_with(
                k(),
                classic,
                level(reheat(airborne(1000.0, 500.0, 100.0))),
                240.0,
                |_, _| {},
            );
            println!("F-16 {name}: max speed at 1000 ft: {:.0} kt", a.knots());
            assert!((700.0..840.0).contains(&a.knots()), "{name} sea level {}", a.knots());
            let (a, _) = fly_with(
                k(),
                classic,
                level(reheat(airborne(36000.0, 600.0, 100.0))),
                // (The full afterburner empties the tanks in about six minutes.)
                300.0,
                |_, _| {},
            );
            println!("F-16 {name}: max speed at 36000 ft: {:.0} kt", a.knots());
            // The classic model's altitude gain is about +27% at 36000 ft.
            assert!(
                a.knots() > if classic { 900.0 } else { 1050.0 },
                "{name} at altitude {}",
                a.knots()
            );
            let (a, _) =
                fly_with(k(), classic, airborne(10000.0, 400.0, 70.0), 1.0, |_, c| c.roll = 1.0);
            let roll = a.euler().2.to_degrees();
            println!("F-16 {name}: roll after 1 s full stick {roll:.0} deg");
        }
        // Modern: a full pull (wings level) at 500 kt goes well past the F-117A's 7 g towards the
        // 9 g limit (the limiters close in on it gradually), rolls past 200 deg/s.
        let a0 = airborne(10000.0, 500.0, 100.0);
        let mut peak: f32 = 0.0;
        fly_with(k(), false, a0, 5.0, |a, c| {
            c.pitch = 1.0;
            peak = peak.max(a.gees / 16.0);
        });
        println!("F-16 modern: peak load {peak:.1} g");
        assert!((7.6..9.05).contains(&peak), "peak {peak}");
        let mut turned = 0.0;
        let mut last = 0.0;
        fly_with(k(), false, airborne(10000.0, 400.0, 70.0), 1.5, |a, c| {
            c.roll = 1.0;
            let r = a.euler().2;
            let mut d = r - last;
            if d < -std::f32::consts::PI {
                d += std::f32::consts::TAU;
            }
            turned += d;
            last = r;
        });
        println!("F-16 modern: rolled {:.0} deg in 1.5 s", turned.to_degrees());
        assert!(turned.to_degrees() > 300.0, "rolled {}", turned.to_degrees());
    }

    /// Hard turns bleed the energy (a full pull from a 75 degree bank at full power, the F-16's
    /// afterburner lit), and the F-16 doesn't leap to speed even with it.
    #[test]
    fn hard_turns_bleed_speed() {
        for (name, k) in [("F-117", FlightConstants::default()), ("F-16", FlightConstants::f16())] {
            let mut a0 = reheat(airborne(5000.0, 400.0, 100.0));
            a0.orientation = Aircraft::from_euler(0.0, 0.0, 75f32.to_radians());
            let (a, _) = fly_with(k.clone(), false, a0, 5.0, |_, c| c.pitch = 1.0);
            println!("{name}: 5 s full pull from 400 kt: {:.0} kt", a.knots());
            assert!(a.knots() < 370.0, "{name} {}", a.knots());
        }
        let mut t = 0.0;
        let mut t450 = None;
        fly_with(
            FlightConstants::f16(),
            false,
            level(reheat(airborne(1000.0, 300.0, 100.0))),
            20.0,
            |a, _| {
                t += 1.0 / 120.0;
                if a.knots() >= 450.0 && t450.is_none() {
                    t450 = Some(t);
                }
            },
        );
        println!("F-16: 300 to 450 kt with the afterburner: {t450:?} s");
        assert!(t450.is_some_and(|t| t > 8.5), "{t450:?}");
    }

    /// The throttle lever with an afterburner: dry thrust up to the detent, full dry thrust in
    /// its gap, the afterburner past it; without one the whole travel is dry thrust.
    #[test]
    fn afterburner_lever() {
        let mut a = Aircraft::new_on_runway(10000.0);
        for (t, thrust, ab) in [
            (0.0, 0.0, None),
            (AB_DETENT / 2.0, 50.0, None),
            (AB_DETENT, 100.0, None),
            (AB_DETENT + AB_GAP / 2.0, 100.0, None),
            (AB_DETENT + AB_GAP, 100.0, Some(0.0)),
            (1.0, 100.0, Some(1.0)),
        ] {
            a.set_lever(t, true);
            assert_eq!(a.set_thrust, thrust, "lever {t}");
            match (a.set_afterburner, ab) {
                (Some(x), Some(y)) => assert!((x - y).abs() < 1e-5, "lever {t}: {x}"),
                (x, y) => assert_eq!(x, y, "lever {t}"),
            }
            assert!((a.lever(true) - t).abs() < 1e-5 || ab.is_none() && t > AB_DETENT);
        }
        a.set_lever(0.9, false);
        assert_eq!((a.set_thrust, a.set_afterburner), (90.0, None));
    }

    /// A held throttle key stops at full dry thrust, then pushes through the detent; the lit
    /// afterburner burns several times the fuel and speeds the F-16 up.
    #[test]
    fn afterburner_keys_and_fuel() {
        let k = FlightConstants::f16;
        let a0 = airborne(10000.0, 400.0, 90.0);
        let mut lit_at = None;
        let mut t = 0.0;
        let (a, _) = fly_with(k(), false, a0.clone(), 3.0, |a, c| {
            c.throttle_rate = 1.0;
            t += 1.0 / 120.0;
            if lit_at.is_none() && a.set_afterburner.is_some() {
                lit_at = Some(t);
            }
        });
        let lit_at = lit_at.expect("through the detent");
        assert!(lit_at > 0.2 + AB_PUSH, "the detent holds the key a moment: {lit_at}");
        assert!(a.afterburner > AB_MIN, "lit {}", a.afterburner);
        // Back below it.
        let (b, _) = fly_with(k(), false, a.clone(), 3.0, |_, c| c.throttle_rate = -1.0);
        assert!(b.set_afterburner.is_none() && b.afterburner == 0.0);
        // A minute at full dry thrust and with the full afterburner.
        let run = |a: Aircraft| fly_with(k(), false, level(a), 60.0, |_, _| {}).0;
        let dry = run(airborne(10000.0, 400.0, 100.0));
        let wet = run(reheat(airborne(10000.0, 400.0, 100.0)));
        let burnt = |x: &Aircraft| 10000.0 - x.fuel;
        assert!(burnt(&wet) > 3.0 * burnt(&dry), "{} vs {}", burnt(&wet), burnt(&dry));
        assert!(wet.knots() > dry.knots() + 50.0, "{} vs {}", wet.knots(), dry.knots());
        // The F-117A has none.
        let (c, _) =
            fly(false, airborne(10000.0, 400.0, 100.0), 2.0, 1, |_, c| c.throttle_rate = 1.0);
        assert!(c.set_afterburner.is_none() && c.afterburner == 0.0);
    }

    /// The Tornado's wings go back as it speeds up and forward as it slows down or lowers the
    /// flaps, taking their time; swept back it is faster at full throttle.
    #[test]
    fn tornado_wings_sweep_with_speed() {
        let k = FlightConstants::tornado;
        let slow = fly_with(k(), false, level(airborne(5000.0, 300.0, 70.0)), 10.0, |_, _| {}).0;
        assert_eq!(slow.wing_sweep, 0.0);
        let mut fast = airborne(5000.0, 600.0, 100.0);
        let (a, _) = fly_with(k(), false, level(fast.clone()), 2.0, |_, _| {});
        assert!(a.wing_sweep > 0.2 && a.wing_sweep < 0.5, "moving back: {}", a.wing_sweep);
        let (a, _) = fly_with(k(), false, level(fast.clone()), 10.0, |_, _| {});
        assert!(a.wing_sweep > 0.95, "back: {}", a.wing_sweep);
        // Fully forward again with the flaps, whatever the speed.
        let mut b = a.clone();
        b.flaps = true;
        let (b, _) = fly_with(k(), false, b, 8.0, |_, _| {});
        assert_eq!(b.wing_sweep, 0.0);
        // With the wings held forward it is slower at full throttle low down.
        fast.velocity = 500.0 * 27.0;
        let swept = fly_with(k(), false, level(reheat(fast.clone())), 60.0, |_, _| {}).0;
        let mut held = FlightConstants::tornado();
        held.aero.swing_wing.as_mut().unwrap().mach = (5.0, 6.0);
        let forward = fly_with(held, false, level(reheat(fast)), 60.0, |_, _| {}).0;
        assert!(
            swept.knots() > forward.knots() + 20.0,
            "swept {} vs forward {}",
            swept.knots(),
            forward.knots()
        );
        // The classic model draws them too.
        let (c, _) = fly_with(k(), true, level(airborne(5000.0, 600.0, 100.0)), 10.0, |_, _| {});
        assert!(c.wing_sweep > 0.95, "classic: {}", c.wing_sweep);
    }

    #[test]
    fn brakes() {
        for classic in [true, false] {
            for throttle in [0.0, 0.3, 0.6, 1.0] {
                // Rolling at 120 kt with the brakes on (no runway: crash protection).
                let mut a0 = Aircraft::new_on_runway(10000.0);
                a0.velocity = 120.0 * 27.0;
                a0.brakes = true;
                let mut stop = None;
                let mut t = 0.0;
                let (a, d) = fly(classic, a0, 30.0, 0, |a, c| {
                    c.throttle = Some(throttle);
                    t += 1.0 / 120.0;
                    if a.knots() < 1.0 && stop.is_none() {
                        stop = Some(t);
                    }
                });
                println!(
                    "classic {classic} throttle {throttle}: stopped {stop:?} after {d:.0} wu, {:.0} kt after 30 s",
                    a.knots()
                );
            }
            // Speed brake: 10 s at idle from 400 kt, altitude held, with and without.
            let speeds = [false, true].map(|brakes| {
                let mut a0 = level(airborne(5000.0, 400.0, 0.0));
                a0.brakes = brakes;
                fly(classic, a0, 10.0, 1, |_, _| {}).0.knots()
            });
            println!(
                "classic {classic}: 10 s idle from 400 kt: {:.0} kt, with speed brake {:.0} kt",
                speeds[0], speeds[1]
            );
            assert!(speeds[1] < speeds[0] - 50.0);
        }
    }

    /// Modern model: pulling and releasing the stick leaves the nose where it was.
    #[test]
    fn release_holds_nose() {
        for kts in [250.0, 450.0] {
            let mut t = 0.0;
            let mut at_release = 0.0;
            let mut lowest = f32::MAX;
            fly(false, airborne(8000.0, kts, 70.0), 4.0, 1, |a, c| {
                t += 1.0 / 120.0;
                let pitch = a.euler().1.to_degrees();
                if t < 1.5 {
                    c.pitch = 0.5;
                    at_release = pitch;
                } else {
                    c.pitch = 0.0;
                    lowest = lowest.min(pitch);
                }
            });
            println!("{kts} kt: pitch at release {at_release:.1}, lowest after {lowest:.1}");
            assert!(at_release > 3.0, "the pull raised the nose ({at_release:.1})");
            assert!(
                lowest > at_release - 1.0,
                "{kts} kt: nose dropped from {at_release:.1} to {lowest:.1}"
            );
        }
    }

    /// Modern model: hands off in a 45 degree bank, the turn stays (nearly) level.
    #[test]
    fn banked_turn_holds_altitude() {
        let mut a0 = airborne(8000.0, 350.0, 80.0);
        a0.orientation = Aircraft::from_euler(0.0, 0.0, 45f32.to_radians());
        let (a, _) = fly(false, a0, 10.0, 1, |_, _| {});
        let (h, _, r) = a.euler();
        println!(
            "10 s in a 45 deg bank: alt {:.0} heading {:.0} roll {:.0}",
            a.altitude,
            h.to_degrees(),
            r.to_degrees()
        );
        assert!((a.altitude - 8000.0).abs() < 150.0, "altitude {}", a.altitude);
        assert!(h.to_degrees() > 20.0, "the aircraft turned");
    }

    /// Modern model: the flaps lower the stall speed and allow a higher angle of attack, and
    /// level flight with them out still needs a positive one (the nose above the horizon).
    #[test]
    fn flaps() {
        let stall = |flaps: bool| {
            let mut a0 = airborne(3000.0, 200.0, 30.0);
            a0.flaps = flaps;
            let (a, _) = fly(false, a0, 0.5, 1, |_, _| {});
            a.stall_speed / 27.0
        };
        let (clean, flaps) = (stall(false), stall(true));
        println!("stall speed clean {clean:.0} kt, flaps {flaps:.0} kt");
        assert!(flaps < clean * 0.85, "flaps {flaps:.0} kt vs clean {clean:.0} kt");
        // Full pull at low speed: the angle of attack reached.
        let max_alpha = |flaps: bool| {
            let mut a0 = airborne(3000.0, 160.0, 100.0);
            a0.flaps = flaps;
            let mut m = 0.0f32;
            fly(false, a0, 4.0, 1, |a, c| {
                c.pitch = 1.0;
                m = m.max(bam(a.fpm).to_degrees());
            });
            m
        };
        let (clean, flaps) = (max_alpha(false), max_alpha(true));
        println!("max angle of attack clean {clean:.1}, flaps {flaps:.1}");
        assert!(flaps > clean + 2.0);
        // Trimmed level flight with the flaps out, fast for them: the 1 g angle of attack (the
        // nose over the flight path) stays positive.
        let mut a0 = airborne(3000.0, 280.0, 60.0);
        a0.flaps = true;
        let (a, _) = fly(false, a0, 0.5, 1, |_, _| {});
        let pitch = a.euler().1.to_degrees();
        println!("flaps out at 280 kt: pitch {pitch:.1}");
        assert!(pitch > 2.0, "nose at {pitch:.1} with the flaps out");
    }

    /// A damaged elevator out of trim: hands off, the nose runs away without the auto trim and
    /// stays with it (both models); the trim switch moves the trim.
    #[test]
    fn auto_trim_takes_out_damage() {
        for classic in [true, false] {
            let fly_damaged = |auto: bool| {
                let mut a0 = airborne(8000.0, 400.0, 70.0);
                a0.damage.pitch_trim = 0.3;
                a0.damage.roll_trim = -0.2;
                a0.auto_trim = auto;
                fly(classic, a0, 20.0, 1, |_, _| {}).0
            };
            let (free, trimmed) = (fly_damaged(false), fly_damaged(true));
            let (_, p0, r0) = free.euler();
            let (_, p1, r1) = trimmed.euler();
            println!(
                "classic {classic}: damaged, 20 s hands off: pitch {:.0} roll {:.0} alt {:.0}; auto trim pitch {:.1} roll {:.1} alt {:.0} trim {:?}",
                p0.to_degrees(),
                r0.to_degrees(),
                free.altitude,
                p1.to_degrees(),
                r1.to_degrees(),
                trimmed.altitude,
                trimmed.stick_trim
            );
            assert!(r1.to_degrees().abs() < 15.0, "classic {classic}: roll {}", r1.to_degrees());
            assert!(
                (trimmed.altitude - 8000.0).abs() < 500.0,
                "classic {classic}: altitude {}",
                trimmed.altitude
            );
            assert!(trimmed.stick_trim.y < -0.2 && trimmed.stick_trim.x > 0.1);
        }
        let mut a = airborne(8000.0, 400.0, 70.0);
        a.move_trim(Vec2::new(0.0, 5.0));
        assert_eq!(a.stick_trim.y, TRIM_MAX);
    }

    #[test]
    fn ground_crash_possible() {
        for classic in [true, false] {
            // Dive at the ground, gear up: crashes unless crash protection is on.
            let mut a0 = airborne(1500.0, 400.0, 70.0);
            a0.orientation = Aircraft::from_euler(0.0, -20f32.to_radians(), 0.0);
            let (a, _) = fly(classic, a0.clone(), 30.0, 1, |_, _| {});
            assert_eq!(a.ended, Some(FlightEnd::Crash), "classic {classic}");
            let (a, _) = fly(classic, a0, 30.0, 0, |_, _| {});
            assert_eq!(a.ended, None, "classic {classic} with crash protection");
            println!("classic {classic}: protected alt {:.0}", a.altitude);
        }
    }
}

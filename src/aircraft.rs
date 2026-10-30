//! Flyable aircraft: the exterior model, the cockpit(s), where the pilot's eye sits in the model
//! and the flight model constants of each type.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::cockpit3d::layout::PanelStyle;
use crate::flight::FlightConstants;
use crate::player::AIRCRAFT_MODEL_SCALE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum AircraftKind {
    #[default]
    F117,
    F16,
    UH60,
    AH1Z,
    Tornado,
    A10,
    RAH66,
    AV8B,
    OH58,
}

/// The exterior model drawn from the cockpit: without its canopy glass, and without the parts
/// reaching into the cockpit.
pub struct CockpitView {
    /// Palette colours of the canopy glass.
    pub canopy_colours: &'static [u8],
    /// Box around the eye (aircraft frame, metres from the eye point, -Z forward): triangles
    /// with a corner or their centre inside are left out.
    pub cut: (Vec3, Vec3),
    /// Also drawn from inside (back faces): a cabin without an interior model of its own.
    pub two_sided: bool,
    /// The model's articulated parts are drawn too: the rotors overhead, the swing wings.
    pub parts: bool,
}

pub struct AircraftSpec {
    /// Aircraft model node of the exterior.
    pub model: &'static str,
    /// Height of the model origin above the aircraft position (the ground contact point), m.
    pub model_lift: f32,
    /// Design eye point in the aircraft frame (-Z forward, +Y up), m.
    pub eye: Vec3,
    /// The exterior model as seen from the cockpit: the pilot looks out over its nose and wings.
    pub cockpit_view: Option<CockpitView>,
    /// Cockpit models (glb stem under `models/`): one, or the modern and the classic one.
    pub cockpits: &'static [(PanelStyle, &'static str)],
    /// How much larger than life the cockpit models are built (the F-117A's, after the classic
    /// one): drawn at life size, in the exterior model's canopy.
    pub cockpit_size: f32,
    /// VR cockpit size at 100% (rig scale: the fraction of life size the cockpit looks in the
    /// headset), what felt right in each.
    pub vr_scale: f32,
    /// Height of the real aircraft's eye above the runway (m), kept in VR: the rig scale makes
    /// the world look as much smaller as the cockpit, so the cockpit is raised until the ground
    /// looks this far below (None: never raised).
    pub vr_eye_height: Option<f32>,
    /// Modern HUD scale: canvas pixels per unit of the tangent plane (`hud::modern`), set by
    /// the combiner's size.
    pub hud_scale: f32,
    /// The modern HUD scale in VR, where the symbology filling the combiner on a screen looks
    /// far too large.
    pub vr_hud_scale: f32,
    /// Weapons in a bay (else on pylons: always "open", seen by radars as the F-117A's open bay).
    pub bomb_bay: bool,
    /// Flown by the helicopter flight model (`flight::heli`): lands anywhere flat, no ejection
    /// seat, and no gun unless it has a turret.
    pub helicopter: bool,
    /// A turret cannon aimed with the targeting pod, and no troops (the AH-1Z, the RAH-66).
    pub turret: bool,
    /// A mast-mounted sight (aircraft frame, m): the targeting pod looks from there, always up
    /// and slaved as a turret's sight is (the OH-58D's).
    pub sight: Option<Vec3>,
    /// A helmet-mounted display (`hmd.rs`).
    pub hmd: Hmd,
    /// The cannon's muzzle (aircraft frame, m), where it is not at the nose.
    pub muzzle: Option<Vec3>,
    /// Where the damage smoke and fire come out, the contrails either side of it (aircraft
    /// frame, m).
    pub exhaust: Vec3,
    /// The afterburning engines' nozzle exits (aircraft frame, m) and their radius (m), where
    /// the afterburner plumes start (`afterburner_flame.rs`).
    pub nozzles: &'static [Vec3],
    pub nozzle_radius: f32,
    /// A swing wing (`flight::modern::SwingWing`): the model's outer wings are parts turning
    /// about their pivots with state slots `WING_SLOTS`, drawn at the forward sweep.
    pub swing_wing: bool,
    /// Vectored thrust (`flight::modern::Vtol`): the model's nozzles are parts turning down
    /// about their pivots with state slot `NOZZLE_SLOT`, the HUD shows their angle.
    pub vtol: bool,
    flight: fn() -> FlightConstants,
}

/// The model state slots of a swing wing's right and left outer wing: their angle back from
/// the forward sweep (16-bit binary angle; the left one negative).
pub const WING_SLOTS: (usize, usize) = (21, 22);

/// The state slot values of a swing wing at `at` (0 fully forward .. 1 fully back, the
/// leading edge from `forward` to `back` rad).
pub fn wing_slot_values(at: f32, forward: f32, back: f32) -> (i16, i16) {
    let bam = ((back - forward) * at.clamp(0.0, 1.0) / std::f32::consts::TAU * 65536.0) as i16;
    (bam, -bam)
}

/// The model state slots of the OH-58D's left and right weapons pylon (`Sim::pylon_looks`).
pub const PYLON_SLOTS: (usize, usize) = (26, 27);

/// The model state slot of a V/STOL type's nozzles: their angle down from aft.
pub const NOZZLE_SLOT: usize = 25;

/// The nozzles' state slot value at `angle` (rad down from aft): the model turns a part by minus
/// its slot angle about its x axis, which turns the exits (aft, +Z) down for a negative value.
pub fn nozzle_slot_value(angle: f32) -> i16 {
    -((angle / std::f32::consts::TAU * 65536.0) as i16)
}

/// A helmet-mounted display's symbology (`hmd.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hmd {
    None,
    /// The AH-1Z's: the sight reticle, the turret gun and the targeting pod as well.
    Full,
    /// The F-16's HMCS: only the horizon, the route, the lock and the Sidewinder seeker.
    Hmcs,
}

impl AircraftSpec {
    /// The exterior model's number (`model`: "aircraft_NNN").
    pub fn model_number(&self) -> i32 {
        self.model.trim_start_matches("aircraft_").parse().unwrap_or(-1)
    }

    /// Size of the cockpit model against the F-16's (life size): the units of the cockpit
    /// frame (`cockpit3d::CockpitRoot`) per metre.
    pub fn cockpit_model_scale(&self) -> f32 {
        self.cockpit_size
    }

    pub fn flight(&self) -> FlightConstants {
        (self.flight)()
    }

    /// A troop helicopter: its missions are insertions.
    pub fn transport(&self) -> bool {
        self.helicopter && !self.turret && self.sight.is_none()
    }

    /// The targeting pod is a turret on the airframe (an attack helicopter's sight, a
    /// mast-mounted sight): always up, slaved to the lock or the helmet sight.
    pub fn slaved_pod(&self) -> bool {
        self.turret || self.sight.is_some()
    }

    /// Where the targeting pod looks from (aircraft frame, m from the aircraft's position).
    pub fn pod_origin(&self) -> Vec3 {
        self.sight.unwrap_or(Vec3::ZERO)
    }

    /// The cockpit flown with the cockpit style setting (aircraft with one cockpit ignore it).
    pub fn cockpit(&self, classic: bool) -> (PanelStyle, &'static str) {
        self.cockpits
            .iter()
            .copied()
            .find(|(s, _)| (*s == PanelStyle::Classic) == classic)
            .unwrap_or(self.cockpits[0])
    }
}

/// A point of the aircraft model (model units, -Z forward) in the aircraft frame, m.
const fn model_point(lift: f32, x: f32, y: f32, z: f32) -> Vec3 {
    let s = AIRCRAFT_MODEL_SCALE as f32;
    Vec3::new(x * s, lift + y * s, z * s)
}

/// The airfields' ground overlays (theater detail level 2) are lifted 0.32 m and a step per
/// overlay (world.rs): the wheels stand on them.
const RUNWAY_Y: f32 = 0.34;
/// The F-117A's model origin is on its flat underside, the wheels 1.65 m below it
/// (tools/models/f117.py).
const F117_LIFT: f32 = 1.65 + RUNWAY_Y;
/// A point of the F-117A's builder (tools/models/f117.py: metres, x right, s aft of the nose, z
/// above the flat underside) in the aircraft frame.
const fn f117_point(x: f32, s: f32, z: f32) -> Vec3 {
    const K: f32 = 16.384;
    model_point(F117_LIFT, x * K, z * K, -(10.6 - s) * K)
}
/// The F-16's wheels are 1.75 m below its model origin (tools/models/aircraft.py `F16_GROUND`).
const F16_LIFT: f32 = 1.75 + RUNWAY_Y;

/// How far below its origin an aircraft model's gear reaches (metres, tools/models/aircraft.py
/// `*_GROUND`, f117.py), for the models that have a landing gear.
pub fn gear_depth(model: i32) -> Option<f32> {
    match model {
        3 | 26 => Some(1.65),
        7 => Some(1.7),
        23 => Some(F16_LIFT - RUNWAY_Y),
        45 => Some(TORNADO_LIFT - RUNWAY_Y),
        32 => Some(A10_LIFT - RUNWAY_Y),
        47 => Some(HARRIER_LIFT - RUNWAY_Y),
        24 => Some(F117_LIFT - RUNWAY_Y),
        _ => None,
    }
}

/// Where an aircraft model's landing light is (metres from its model origin, -Z forward): on
/// the nose gear's leg just under the body, a helicopter's under its nose (the builders in
/// tools/models), else a little under and ahead of the origin.
pub fn landing_light(model: i32) -> Vec3 {
    match model {
        3 | 26 => Vec3::new(0.0, -0.62, -4.0),
        7 => Vec3::new(0.0, -0.57, -4.6),
        23 => f16_point(0.0, 2.20, -1.04) - Vec3::Y * F16_LIFT,
        24 => f117_point(0.0, 4.80, -0.12) - Vec3::Y * F117_LIFT,
        25 => uh60_point(0.0, 3.0, 0.53) - Vec3::Y * UH60_LIFT,
        32 => a10_point(0.30, 5.10, -0.66) - Vec3::Y * A10_LIFT,
        43 => ah1z_point(0.0, 2.5, 0.70) - Vec3::Y * AH1Z_LIFT,
        45 => tornado_point(0.0, 3.85, -0.90) - Vec3::Y * TORNADO_LIFT,
        46 => rah66_point(0.0, 2.5, 0.57) - Vec3::Y * RAH66_LIFT,
        47 => harrier_point(0.0, 2.30, -0.78) - Vec3::Y * HARRIER_LIFT,
        48 => oh58_point(0.0, 2.3, 0.55) - Vec3::Y * OH58_LIFT,
        // The Hind (tools/models/mi24.py, its origin on the ground).
        102 => Vec3::new(0.0, 0.92, -3.6),
        _ => Vec3::new(0.0, -1.0, -4.0),
    }
}

/// Height of the airfields' surfaces over the ground (metres).
pub const GEAR_GROUND: f32 = RUNWAY_Y;
/// A point of the F-16's builder (metres: x right, y forward, z up) in the aircraft frame: the
/// model is centred on its length, 0.09 m behind the builder's origin.
const fn f16_point(x: f32, y: f32, z: f32) -> Vec3 {
    const K: f32 = 16.384;
    model_point(F16_LIFT, x * K, z * K, -(y + 0.09) * K)
}

/// Behind the middle, at the engines (the helicopters').
const EXHAUST: Vec3 = Vec3::new(0.0, 0.3, 8.0);
/// The F-117A's exhaust slots, 15 m behind the nose (10.6 m ahead of the model origin).
const F117_EXHAUST: Vec3 = Vec3::new(0.0, F117_LIFT + 0.4, 4.6);

static F117: AircraftSpec = AircraftSpec {
    model: "aircraft_024",
    model_lift: F117_LIFT,
    // Under the canopy's glass, a little ahead of its apex (s 5.1 m, z 2.26 m), the head 0.3 m
    // under the panes.
    eye: f117_point(0.0, 4.9, 1.85),
    cockpit_view: Some(CockpitView {
        canopy_colours: &[144, 145],
        cut: (Vec3::new(-0.60, -1.20, -1.10), Vec3::new(0.60, 0.30, 0.90)),
        two_sided: false,
        parts: false,
    }),
    cockpits: &[(PanelStyle::Modern, "cockpit_modern"), (PanelStyle::Classic, "cockpit_classic")],
    // tools/models/build_cockpit_f117.py SCALE.
    cockpit_size: 1.4706,
    // Life size, as the F-16's.
    vr_scale: 1.25 * 0.7,
    vr_eye_height: Some(2.7),
    hud_scale: crate::hud::modern::SCALE,
    // About the F-16's angular size: the symbology in the middle half of the combiner.
    vr_hud_scale: 2.0 * crate::hud::modern::SCALE,
    bomb_bay: true,
    helicopter: false,
    turret: false,
    hmd: Hmd::None,
    muzzle: None,
    sight: None,
    exhaust: F117_EXHAUST,
    nozzles: &[],
    nozzle_radius: 0.0,
    swing_wing: false,
    vtol: false,
    flight: FlightConstants::default,
};

static F16: AircraftSpec = AircraftSpec {
    model: "aircraft_023",
    model_lift: F16_LIFT,
    // Under the canopy bubble (y 2.1..5.0 m, z 0.5..1.25 m), a little ahead of the helmet in the
    // three-view and 0.3 m under the bubble's top.
    eye: f16_point(0.0, 3.25, 0.93),
    cockpit_view: Some(CockpitView {
        canopy_colours: &[144, 145],
        cut: (Vec3::new(-0.40, -1.15, -0.85), Vec3::new(0.40, 0.40, 0.80)),
        two_sided: false,
        parts: false,
    }),
    cockpits: &[(PanelStyle::F16, "cockpit_f16")],
    cockpit_size: 1.0,
    // The symbology (about 920 x 470 canvas pixels) inside the combiner (x +-0.1675, z -0.12 ..
    // 0.095 at 0.70 m in front of the eye, tools/models/build_cockpit_f16.py).
    vr_scale: 1.25 * 0.7,
    vr_eye_height: None,
    hud_scale: 2080.0,
    vr_hud_scale: 2080.0,
    bomb_bay: false,
    helicopter: false,
    turret: false,
    hmd: Hmd::Hmcs,
    muzzle: None,
    // The nozzle.
    sight: None,
    exhaust: f16_point(0.0, -7.2, 0.04),
    nozzles: &[f16_point(0.0, -7.2, 0.04)],
    nozzle_radius: 0.45,
    swing_wing: false,
    vtol: false,
    flight: FlightConstants::f16,
};

/// A point of the UH-60A model (tools/models/uh60.py: metres, x right, y forward, z up, origin
/// on the ground under the mast, `K` model units per metre) in the aircraft frame.
const fn uh60_point(x: f32, y: f32, z: f32) -> Vec3 {
    const K: f32 = 16.38;
    model_point(UH60_LIFT, x * K, z * K, -y * K)
}

/// The UH-60A's model origin is on the ground under its wheels; clear of the ground overlays.
const UH60_LIFT: f32 = 0.05;

static UH60: AircraftSpec = AircraftSpec {
    model: "aircraft_025",
    model_lift: UH60_LIFT,
    // The pilot's (right) seat, behind the windshield top (y 3.05) and ahead of the door post
    // (y 2.40), head under the roof.
    eye: uh60_point(0.52, 2.62, 1.90),
    cockpit_view: Some(CockpitView {
        // The windshield, the door and the cabin windows: open from inside.
        canopy_colours: &[144, 145],
        cut: (Vec3::ZERO, Vec3::ZERO),
        two_sided: true,
        parts: true,
    }),
    cockpits: &[(PanelStyle::Uh60, "cockpit_uh60")],
    cockpit_size: 1.0,
    // Life size, as the F-16's.
    vr_scale: 1.25 * 0.7,
    vr_eye_height: None,
    hud_scale: crate::hud::modern::SCALE,
    vr_hud_scale: 2.0 * crate::hud::modern::SCALE,
    bomb_bay: false,
    helicopter: true,
    turret: false,
    hmd: Hmd::None,
    muzzle: None,
    sight: None,
    exhaust: EXHAUST,
    nozzles: &[],
    nozzle_radius: 0.0,
    swing_wing: false,
    vtol: false,
    flight: FlightConstants::uh60,
};

/// A point of the AH-1Z model (tools/models/ah1z.py, as `uh60_point`) in the aircraft frame.
const fn ah1z_point(x: f32, y: f32, z: f32) -> Vec3 {
    const K: f32 = 16.38;
    model_point(AH1Z_LIFT, x * K, z * K, -y * K)
}

/// The AH-1Z's model origin is on the ground under its skids; clear of the ground overlays.
const AH1Z_LIFT: f32 = 0.05;

static AH1Z: AircraftSpec = AircraftSpec {
    model: "aircraft_043",
    model_lift: AH1Z_LIFT,
    // The pilot's (rear) seat: behind the step of the canopy (y 2.30) and ahead of its aft end
    // (y 1.10), the head under the roof (z 2.50), the sills (z 1.5) at the elbows.
    eye: ah1z_point(0.0, 1.70, 2.12),
    cockpit_view: Some(CockpitView {
        // The canopy: open from inside; the hull drawn from inside round the tandem cockpits.
        canopy_colours: &[144, 145],
        cut: (Vec3::ZERO, Vec3::ZERO),
        two_sided: true,
        parts: true,
    }),
    cockpits: &[(PanelStyle::Ah1z, "cockpit_ah1z")],
    cockpit_size: 1.0,
    vr_scale: 1.25 * 0.7,
    vr_eye_height: None,
    hud_scale: crate::hud::modern::SCALE,
    vr_hud_scale: 2.0 * crate::hud::modern::SCALE,
    bomb_bay: false,
    helicopter: true,
    turret: true,
    hmd: Hmd::Full,
    // The chin turret's barrels.
    muzzle: Some(ah1z_point(0.0, 5.30, 0.58)),
    sight: None,
    exhaust: EXHAUST,
    nozzles: &[],
    nozzle_radius: 0.0,
    swing_wing: false,
    vtol: false,
    flight: FlightConstants::ah1z,
};

/// A point of the RAH-66 model (tools/models/comanche.py, as `ah1z_point`) in the aircraft frame.
const fn rah66_point(x: f32, y: f32, z: f32) -> Vec3 {
    const K: f32 = 16.38;
    model_point(RAH66_LIFT, x * K, z * K, -y * K)
}

/// The RAH-66's model origin is on the ground under its wheels; clear of the ground overlays.
const RAH66_LIFT: f32 = 0.05;

static RAH66: AircraftSpec = AircraftSpec {
    model: "aircraft_046",
    model_lift: RAH66_LIFT,
    // The pilot's (front) seat: behind the windshield's bow (y 2.48 at the sill, 2.71 at the
    // roof) and ahead of the gunner's (y 1.84), the head under the flat roof (z 2.51), the sills
    // (z 1.47) at the elbows.
    eye: rah66_point(0.0, 2.10, 2.06),
    cockpit_view: Some(CockpitView {
        // The canopy: open from inside; the hull drawn from inside round the tandem cockpits.
        canopy_colours: &[144, 145],
        cut: (Vec3::ZERO, Vec3::ZERO),
        two_sided: true,
        parts: true,
    }),
    cockpits: &[(PanelStyle::Rah66, "cockpit_rah66")],
    cockpit_size: 1.0,
    vr_scale: 1.25 * 0.7,
    vr_eye_height: None,
    hud_scale: crate::hud::modern::SCALE,
    vr_hud_scale: 2.0 * crate::hud::modern::SCALE,
    bomb_bay: true,
    helicopter: true,
    turret: true,
    hmd: Hmd::Full,
    // The chin turret's barrels.
    muzzle: Some(rah66_point(0.0, 4.75, 0.56)),
    sight: None,
    exhaust: EXHAUST,
    nozzles: &[],
    nozzle_radius: 0.0,
    swing_wing: false,
    vtol: false,
    flight: FlightConstants::rah66,
};

/// A point of the OH-58D model (tools/models/kiowa.py, as `ah1z_point`) in the aircraft frame.
const fn oh58_point(x: f32, y: f32, z: f32) -> Vec3 {
    const K: f32 = 16.38;
    model_point(OH58_LIFT, x * K, z * K, -y * K)
}

/// The OH-58D's model origin is on the ground under its skids; clear of the ground overlays.
const OH58_LIFT: f32 = 0.05;

static OH58: AircraftSpec = AircraftSpec {
    model: "aircraft_048",
    model_lift: OH58_LIFT,
    // The pilot's (right) seat: behind the windshield's top (y 1.44) over the front door
    // (y 0.88..1.48), the head under the roof (z 1.81), the window sills (z 1.1) at the elbows.
    eye: oh58_point(0.36, 1.10, 1.45),
    cockpit_view: Some(CockpitView {
        // The windshield and the door windows: open from inside; the hull drawn from inside.
        canopy_colours: &[144, 145],
        cut: (Vec3::ZERO, Vec3::ZERO),
        two_sided: true,
        parts: true,
    }),
    cockpits: &[(PanelStyle::Ah1z, "cockpit_ah1z")],
    cockpit_size: 1.0,
    vr_scale: 1.25 * 0.7,
    vr_eye_height: None,
    hud_scale: crate::hud::modern::SCALE,
    vr_hud_scale: 2.0 * crate::hud::modern::SCALE,
    bomb_bay: false,
    helicopter: true,
    turret: false,
    // The mast-mounted sight's ball above the rotor.
    sight: Some(oh58_point(0.0, 0.04, 3.62)),
    hmd: Hmd::Full,
    // The machine gun pods fire along the nose.
    muzzle: None,
    exhaust: EXHAUST,
    nozzles: &[],
    nozzle_radius: 0.0,
    swing_wing: false,
    vtol: false,
    flight: FlightConstants::oh58,
};

/// The Tornado's wheels are 1.90 m below its model origin (tools/models/aircraft.py
/// `TORNADO_GROUND`).
const TORNADO_LIFT: f32 = 1.90 + RUNWAY_Y;
/// A point of the Tornado's builder (tools/models/aircraft.py `tornado`: metres, x right, y
/// forward, z up) in the aircraft frame: the model is centred on its length, 0.105 m ahead of
/// the builder's origin.
const fn tornado_point(x: f32, y: f32, z: f32) -> Vec3 {
    const K: f32 = 16.384;
    model_point(TORNADO_LIFT, x * K, z * K, -(y - 0.105) * K)
}

static TORNADO: AircraftSpec = AircraftSpec {
    model: "aircraft_045",
    model_lift: TORNADO_LIFT,
    // The pilot's (front) seat under the canopy (y 1.4..5.95 m, sill z 0.5, top 1.32 m), 0.3 m
    // under its top and ahead of the navigator's.
    eye: tornado_point(0.0, 4.25, 1.00),
    cockpit_view: Some(CockpitView {
        canopy_colours: &[144, 145],
        cut: (Vec3::new(-0.45, -1.15, -0.85), Vec3::new(0.45, 0.40, 0.80)),
        two_sided: false,
        // The outer wings.
        parts: true,
    }),
    cockpits: &[(PanelStyle::Tornado, "cockpit_tornado")],
    cockpit_size: 1.0,
    vr_scale: 1.25 * 0.7,
    vr_eye_height: None,
    // As the F-16's: the symbology (about 920 x 470 canvas pixels) inside the combiner of the
    // same angular size (x +-0.18, z -0.105 .. 0.095 at 0.80 m in front of the eye,
    // tools/models/build_cockpit_tornado.py).
    hud_scale: 2080.0,
    vr_hud_scale: 2080.0,
    bomb_bay: false,
    helicopter: false,
    turret: false,
    hmd: Hmd::None,
    muzzle: None,
    // Between the two nozzles.
    sight: None,
    exhaust: tornado_point(0.0, -7.35, -0.15),
    nozzles: &[tornado_point(0.55, -7.35, -0.15), tornado_point(-0.55, -7.35, -0.15)],
    nozzle_radius: 0.38,
    swing_wing: true,
    vtol: false,
    flight: FlightConstants::tornado,
};

/// The A-10's wheels are 2.06 m below its model origin (tools/models/aircraft.py `A10_GROUND`).
const A10_LIFT: f32 = 2.06 + RUNWAY_Y;
/// A point of the A-10's builder (tools/models/aircraft.py `a10`: metres, x right, y forward, z
/// up) in the aircraft frame: the model is centred on its length, 0.13 m ahead of the builder's
/// origin.
const fn a10_point(x: f32, y: f32, z: f32) -> Vec3 {
    const K: f32 = 16.384;
    model_point(A10_LIFT, x * K, z * K, -(y - 0.13) * K)
}

static A10: AircraftSpec = AircraftSpec {
    model: "aircraft_032",
    model_lift: A10_LIFT,
    // Under the bubble canopy (y 3.05..6.66 m, sill z 0.95, top 1.63 m), 0.38 m under its top.
    eye: a10_point(0.0, 4.55, 1.25),
    cockpit_view: Some(CockpitView {
        canopy_colours: &[144, 145],
        cut: (Vec3::new(-0.50, -1.15, -0.85), Vec3::new(0.50, 0.40, 0.80)),
        two_sided: false,
        parts: false,
    }),
    cockpits: &[(PanelStyle::A10, "cockpit_a10")],
    cockpit_size: 1.0,
    vr_scale: 1.25 * 0.7,
    vr_eye_height: None,
    // The F-16's, scaled to the taller combiner (x +-0.17, z -0.13 .. 0.11 at 0.70 m in front
    // of the eye, tools/models/build_cockpit_a10.py).
    hud_scale: 1900.0,
    vr_hud_scale: 1900.0,
    bomb_bay: false,
    helicopter: false,
    turret: false,
    hmd: Hmd::None,
    // The GAU-8's muzzle under the nose.
    muzzle: Some(a10_point(0.0, 8.36, -0.34)),
    // Between the two engines.
    sight: None,
    exhaust: a10_point(0.0, -5.0, 0.80),
    nozzles: &[],
    nozzle_radius: 0.0,
    swing_wing: false,
    vtol: false,
    flight: FlightConstants::a10,
};

/// The AV-8B's wheels are 1.56 m below its model origin (tools/models/aircraft.py
/// `HARRIER_GROUND`), on the ground line of its nose-up attitude.
const HARRIER_LIFT: f32 = 1.56 + RUNWAY_Y;
/// A point of the AV-8B's builder (tools/models/aircraft.py `harrier`: metres, x right, y
/// forward, z up) in the aircraft frame: the model is centred on its length at the builder's
/// origin.
const fn harrier_point(x: f32, y: f32, z: f32) -> Vec3 {
    const K: f32 = 16.384;
    model_point(HARRIER_LIFT, x * K, z * K, -y * K)
}

static AV8B: AircraftSpec = AircraftSpec {
    model: "aircraft_047",
    model_lift: HARRIER_LIFT,
    // Under the raised bubble (y 3.15..5.9 m, sill z 0.57, top 1.27 m over the eye), 0.27 m
    // under its top.
    eye: harrier_point(0.0, 4.30, 1.00),
    cockpit_view: Some(CockpitView {
        canopy_colours: &[144, 145],
        cut: (Vec3::new(-0.52, -1.15, -0.97), Vec3::new(0.52, 0.40, 0.80)),
        two_sided: false,
        // The front nozzles show beside the intakes.
        parts: true,
    }),
    cockpits: &[(PanelStyle::Av8b, "cockpit_av8b")],
    cockpit_size: 1.0,
    vr_scale: 1.25 * 0.7,
    vr_eye_height: None,
    // As the F-16's: the symbology inside the combiner of about the same angular size (x
    // +-0.15, z -0.13 .. 0.075 at 0.66 m in front of the eye, tools/models/build_cockpit_av8b.py).
    hud_scale: 2080.0,
    vr_hud_scale: 2080.0,
    bomb_bay: false,
    helicopter: false,
    turret: false,
    hmd: Hmd::None,
    // The GAU-12 in the left pod.
    muzzle: Some(harrier_point(-0.38, 2.95, -0.85)),
    // Between the hot rear nozzles.
    sight: None,
    exhaust: harrier_point(0.0, -1.10, -0.35),
    nozzles: &[],
    nozzle_radius: 0.0,
    swing_wing: false,
    vtol: true,
    flight: FlightConstants::av8b,
};

impl AircraftKind {
    pub const ALL: [AircraftKind; 9] = [
        AircraftKind::F117,
        AircraftKind::F16,
        AircraftKind::Tornado,
        AircraftKind::A10,
        AircraftKind::AV8B,
        AircraftKind::UH60,
        AircraftKind::AH1Z,
        AircraftKind::RAH66,
        AircraftKind::OH58,
    ];

    /// What another pilot flies.
    pub fn of_remote(p: &stealth_sim::net::RemotePilot) -> Self {
        if p.comanche {
            AircraftKind::RAH66
        } else if p.kiowa {
            AircraftKind::OH58
        } else if p.gunship {
            AircraftKind::AH1Z
        } else if p.helicopter {
            AircraftKind::UH60
        } else if p.tornado {
            AircraftKind::Tornado
        } else if p.a10 {
            AircraftKind::A10
        } else if p.harrier {
            AircraftKind::AV8B
        } else if p.external_stores {
            AircraftKind::F16
        } else {
            AircraftKind::F117
        }
    }

    pub fn spec(self) -> &'static AircraftSpec {
        match self {
            AircraftKind::F117 => &F117,
            AircraftKind::F16 => &F16,
            AircraftKind::UH60 => &UH60,
            AircraftKind::AH1Z => &AH1Z,
            AircraftKind::Tornado => &TORNADO,
            AircraftKind::A10 => &A10,
            AircraftKind::RAH66 => &RAH66,
            AircraftKind::AV8B => &AV8B,
            AircraftKind::OH58 => &OH58,
        }
    }

    /// Command-line name (`--aircraft`).
    pub fn id(self) -> &'static str {
        match self {
            AircraftKind::F117 => "f117",
            AircraftKind::F16 => "f16",
            AircraftKind::UH60 => "uh60",
            AircraftKind::AH1Z => "ah1z",
            AircraftKind::Tornado => "tornado",
            AircraftKind::A10 => "a10",
            AircraftKind::RAH66 => "rah66",
            AircraftKind::AV8B => "av8b",
            AircraftKind::OH58 => "oh58",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let s = s.to_ascii_lowercase().replace(['-', '_'], "");
        Self::ALL.into_iter().find(|k| {
            k.id() == s
                || (s == "f117a" && *k == AircraftKind::F117)
                || (s == "f16c" && *k == AircraftKind::F16)
                || (matches!(s.as_str(), "uh60a" | "blackhawk") && *k == AircraftKind::UH60)
                || (matches!(s.as_str(), "ah1" | "viper") && *k == AircraftKind::AH1Z)
                || (matches!(s.as_str(), "comanche" | "rah66a") && *k == AircraftKind::RAH66)
                || (matches!(s.as_str(), "tornadoids" | "ids") && *k == AircraftKind::Tornado)
                || (matches!(s.as_str(), "a10c" | "warthog" | "thunderbolt")
                    && *k == AircraftKind::A10)
                || (matches!(s.as_str(), "harrier" | "av8" | "av8bplus" | "harrierii")
                    && *k == AircraftKind::AV8B)
                || (matches!(s.as_str(), "kiowa" | "oh58d" | "kiowawarrior")
                    && *k == AircraftKind::OH58)
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            AircraftKind::F117 => "F-117A Nighthawk",
            AircraftKind::F16 => "F-16C Fighting Falcon",
            AircraftKind::UH60 => "UH-60A Black Hawk",
            AircraftKind::AH1Z => "AH-1Z Viper",
            AircraftKind::Tornado => "Tornado IDS",
            AircraftKind::A10 => "A-10C Thunderbolt II",
            AircraftKind::RAH66 => "RAH-66 Comanche",
            AircraftKind::AV8B => "AV-8B Harrier II",
            AircraftKind::OH58 => "OH-58D Kiowa Warrior",
        }
    }
}

/// Every cockpit model file of the flyable aircraft.
pub fn cockpit_files() -> Vec<&'static str> {
    let mut v: Vec<&str> =
        AircraftKind::ALL.iter().flat_map(|k| k.spec().cockpits.iter().map(|(_, f)| *f)).collect();
    v.sort_unstable();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Accessor bounds of the primitives of `node` drawn in the given palette colours (glTF JSON
    /// chunk only).
    fn primitive_bounds(node: &str, colours: &[u8]) -> (Vec3, Vec3) {
        let path = crate::asset_path::asset("models/aircraft.glb");
        let data = std::fs::read(&path).unwrap();
        let len = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
        let j: serde_json::Value = serde_json::from_slice(&data[20..20 + len]).unwrap();
        let n = j["nodes"].as_array().unwrap().iter().find(|n| n["name"] == node).unwrap();
        let mesh = &j["meshes"][n["mesh"].as_u64().unwrap() as usize];
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        let mut found = 0;
        for p in mesh["primitives"].as_array().unwrap() {
            let mat =
                j["materials"][p["material"].as_u64().unwrap() as usize]["name"].as_str().unwrap();
            if !colours.iter().any(|c| mat == format!("pal_{c:03}")) {
                continue;
            }
            found += 1;
            let acc = &j["accessors"][p["attributes"]["POSITION"].as_u64().unwrap() as usize];
            let v = |k: &str| {
                let a = acc[k].as_array().unwrap();
                Vec3::new(
                    a[0].as_f64().unwrap() as f32,
                    a[1].as_f64().unwrap() as f32,
                    a[2].as_f64().unwrap() as f32,
                )
            };
            lo = lo.min(v("min"));
            hi = hi.max(v("max"));
        }
        assert!(found > 0, "no canopy primitives in {node}");
        (lo, hi)
    }

    /// The jets' eye inside the bounds of their canopy glass.
    #[test]
    fn eye_under_the_canopy() {
        for kind in [
            AircraftKind::F16,
            AircraftKind::F117,
            AircraftKind::Tornado,
            AircraftKind::A10,
            AircraftKind::AV8B,
        ] {
            let spec = kind.spec();
            let (lo, hi) = primitive_bounds(
                &format!("{}_lod0", spec.model),
                spec.cockpit_view.as_ref().unwrap().canopy_colours,
            );
            let s = AIRCRAFT_MODEL_SCALE as f32;
            let eye = (spec.eye - Vec3::Y * spec.model_lift) / s;
            assert!(
                eye.cmpgt(lo).all() && eye.cmplt(hi).all(),
                "{kind:?} eye {eye} outside the canopy {lo}..{hi}"
            );
        }
    }

    /// The UH-60A's eye: in the cockpit between the windshield top and the door post
    /// (tools/models/uh60.py STATIONS), under the roof, inside the side.
    #[test]
    fn uh60_eye_in_the_cockpit() {
        let spec = AircraftKind::UH60.spec();
        let s = AIRCRAFT_MODEL_SCALE as f32 * 16.38;
        let eye = (spec.eye - Vec3::Y * spec.model_lift) / s;
        let (x, y, z) = (eye.x, -eye.z, eye.y);
        assert!((2.40..3.05).contains(&y), "eye {y} m from the mast");
        assert!((1.40..2.20).contains(&z), "eye {z} m high");
        assert!(x.abs() < 1.0, "eye {x} m to the side");
    }

    /// The AH-1Z's eye: in the pilot's cockpit between the canopy's step and its aft end
    /// (tools/models/ah1z.py STATIONS), under the roof, above the sills.
    #[test]
    fn ah1z_eye_in_the_cockpit() {
        let spec = AircraftKind::AH1Z.spec();
        let s = AIRCRAFT_MODEL_SCALE as f32 * 16.38;
        let eye = (spec.eye - Vec3::Y * spec.model_lift) / s;
        let (x, y, z) = (eye.x, -eye.z, eye.y);
        assert!((1.10..2.30).contains(&y), "eye {y} m from the mast");
        assert!((1.60..2.45).contains(&z), "eye {z} m high");
        assert!(x.abs() < 0.3, "eye {x} m to the side");
    }

    /// The RAH-66's eye: in the pilot's front cockpit between the windshield's bow and the
    /// gunner's (tools/models/comanche.py STATIONS), under the flat roof, above the sills.
    #[test]
    fn rah66_eye_in_the_cockpit() {
        let spec = AircraftKind::RAH66.spec();
        let s = AIRCRAFT_MODEL_SCALE as f32 * 16.38;
        let eye = (spec.eye - Vec3::Y * spec.model_lift) / s;
        let (x, y, z) = (eye.x, -eye.z, eye.y);
        assert!((1.84..2.48).contains(&y), "eye {y} m from the mast");
        assert!((1.60..2.40).contains(&z), "eye {z} m high");
        assert!(x.abs() < 0.3, "eye {x} m to the side");
    }

    /// The OH-58D's eye: in the right seat behind the windshield's top, over the front door
    /// (tools/models/kiowa.py STATIONS), under the roof, inside the side; the sight above the
    /// rotor.
    #[test]
    fn oh58_eye_in_the_cockpit() {
        let spec = AircraftKind::OH58.spec();
        let s = AIRCRAFT_MODEL_SCALE as f32 * 16.38;
        let eye = (spec.eye - Vec3::Y * spec.model_lift) / s;
        let (x, y, z) = (eye.x, -eye.z, eye.y);
        assert!((0.88..1.44).contains(&y), "eye {y} m from the mast");
        assert!((1.20..1.70).contains(&z), "eye {z} m high");
        assert!((0.2..0.6).contains(&x), "eye {x} m right");
        let sight = (spec.sight.unwrap() - Vec3::Y * spec.model_lift) / s;
        assert!((3.3..3.9).contains(&sight.y), "sight {} m high", sight.y);
        assert!(!spec.transport() && spec.slaved_pod());
    }

    #[test]
    fn names_round_trip() {
        for k in AircraftKind::ALL {
            assert_eq!(AircraftKind::parse(k.id()), Some(k));
            assert!(!k.spec().cockpits.is_empty());
        }
        assert_eq!(AircraftKind::parse("F-16"), Some(AircraftKind::F16));
        assert_eq!(AircraftKind::parse("UH-60"), Some(AircraftKind::UH60));
        assert_eq!(AircraftKind::parse("blackhawk"), Some(AircraftKind::UH60));
        assert_eq!(AircraftKind::parse("AH-1Z"), Some(AircraftKind::AH1Z));
        assert_eq!(AircraftKind::parse("viper"), Some(AircraftKind::AH1Z));
        assert_eq!(AircraftKind::parse("Tornado-IDS"), Some(AircraftKind::Tornado));
        assert_eq!(AircraftKind::parse("A-10C"), Some(AircraftKind::A10));
        assert_eq!(AircraftKind::parse("warthog"), Some(AircraftKind::A10));
        assert_eq!(AircraftKind::parse("RAH-66"), Some(AircraftKind::RAH66));
        assert_eq!(AircraftKind::parse("Comanche"), Some(AircraftKind::RAH66));
        assert_eq!(AircraftKind::parse("AV-8B"), Some(AircraftKind::AV8B));
        assert_eq!(AircraftKind::parse("Harrier"), Some(AircraftKind::AV8B));
        assert_eq!(AircraftKind::parse("OH-58D"), Some(AircraftKind::OH58));
        assert_eq!(AircraftKind::parse("Kiowa"), Some(AircraftKind::OH58));
    }

    /// The Harrier's four nozzles: parts pitching about x on the nozzle slot, pivoted on the
    /// fuselage's sides; a positive nozzle angle turns their exits down.
    #[test]
    fn harrier_nozzles_are_parts() {
        let path = crate::asset_path::asset("models/aircraft.glb");
        let data = std::fs::read(&path).unwrap();
        let len = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
        let j: serde_json::Value = serde_json::from_slice(&data[20..20 + len]).unwrap();
        let model = AircraftKind::AV8B.spec().model;
        let mut sides = vec![];
        for k in 1..=4 {
            let name = format!("{model}_lod0_part{k}");
            let n = j["nodes"].as_array().unwrap().iter().find(|n| n["name"] == name).unwrap();
            assert_eq!(n["extras"]["state_slot"].as_u64(), Some(NOZZLE_SLOT as u64), "{name}");
            assert_eq!(n["extras"]["axis"].as_u64(), Some(1), "{name}");
            let x = n["translation"][0].as_f64().unwrap() as f32 / 16.384;
            assert!((0.8..1.3).contains(&x.abs()), "{name} pivot {x} m out");
            sides.push(x.signum());
        }
        assert_eq!(sides.iter().sum::<f32>(), 0.0);
        // As `models::apply_model_state` turns them: the exit, aft, goes straight down at 90.
        let v = nozzle_slot_value(90f32.to_radians());
        let angle = -(v as u16 as f32) / 65536.0 * std::f32::consts::TAU;
        let exit = Quat::from_rotation_x(angle) * Vec3::Z;
        assert!(exit.y < -0.999, "{exit}");
        assert_eq!(nozzle_slot_value(0.0), 0);
    }

    /// The Tornado's outer wings: two parts on the slots the game sweeps them with, pivoted
    /// either side of the centreline in the gloves.
    #[test]
    fn tornado_wings_are_parts() {
        let path = crate::asset_path::asset("models/aircraft.glb");
        let data = std::fs::read(&path).unwrap();
        let len = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
        let j: serde_json::Value = serde_json::from_slice(&data[20..20 + len]).unwrap();
        let node = |name: &str| {
            j["nodes"].as_array().unwrap().iter().find(|n| n["name"] == name).unwrap().clone()
        };
        let model = AircraftKind::Tornado.spec().model;
        for (part, slot, side) in [("_part1", WING_SLOTS.0, 1.0), ("_part2", WING_SLOTS.1, -1.0)] {
            let n = node(&format!("{model}_lod0{part}"));
            assert_eq!(n["extras"]["state_slot"].as_u64(), Some(slot as u64), "{part}");
            assert_eq!(n["extras"]["axis"].as_u64(), Some(0), "{part}");
            let x = n["translation"][0].as_f64().unwrap() as f32 / 16.384;
            assert!(x * side > 1.0 && x * side < 2.0, "{part} pivot {x} m out");
        }
        // Swept back, the right wing turns its tip aft (the model turns a part by minus its
        // slot angle about up, -Z forward).
        let (r, l) = wing_slot_values(1.0, 25f32.to_radians(), 67f32.to_radians());
        assert_eq!(r, -l);
        let angle = -(r as u16 as f32) / 65536.0 * std::f32::consts::TAU;
        let tip = Quat::from_rotation_y(angle) * Vec3::X;
        assert!(tip.z > 0.6 && (angle.to_degrees() + 42.0).abs() < 0.1, "{tip} {angle}");
    }
}

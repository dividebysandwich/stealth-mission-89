//! Loadout editor (otherwise the mission generator picks the stores): the stores on each
//! hardpoint, the F-117A's bays (two real ones, two more inside the wings without realistic
//! weapons), the F-16's nine pylon stations or the AH-1Z's six.

use bevy::prelude::*;
use bevy_egui::egui;
use serde::{Deserialize, Serialize};
use stealth_mission::{Mission, Plane};

use crate::aircraft::AircraftKind;
use crate::menu::{AMBER, BRIGHT, DIM, GREEN, MenuState, big_button, facts, section};

/// The weapons (index = weapon type): name, what it is for.
pub const WEAPONS: [(&str, &str); 21] = [
    ("AIM-9M Sidewinder", "Air-to-air, infrared homing, short range"),
    ("AIM-120A AMRAAM", "Air-to-air, radar homing, medium range"),
    ("AGM-88A HARM", "Anti-radiation missile: radars and SAM sites"),
    ("Penguin 3 ASM", "Anti-ship missile"),
    ("AGM-84A Harpoon", "Long range anti-ship missile"),
    ("AGM-65 Maverick", "TV guided missile: vehicles, bunkers, buildings"),
    ("GBU-12 Paveway", "Laser guided bomb: hard targets"),
    ("Mk 20 Rockeye", "Cluster bomb: soft and area targets"),
    ("Durandal", "Runway cratering bomb"),
    ("Mk 82 LDGP", "500 lb free-fall bomb"),
    ("Mk 82 Snakeye", "500 lb retarded bomb for low releases"),
    ("Mk 20 Rockeye II", "Cluster bomb: armour and area targets"),
    ("Mk 122 Fireye", "Incendiary bomb"),
    ("CBU-72 FAE", "Fuel-air explosive: area targets"),
    ("Mk 35 incendiary", "Incendiary cluster bomb"),
    ("ISCB-1 mines", "Area denial mines"),
    ("IR/optical camera", "Reconnaissance photos of the target"),
    ("1900 lb fuel tank", "Extra fuel, transferred when released"),
    ("Special cargo", "Supplies or cargo for the mission"),
    ("20 mm cannon", "The gun (not a store)"),
    (
        "AGM-114 Hellfire",
        "Laser guided anti-armour missile: flies to where the targeting pod looks",
    ),
];
/// How each weapon is used (index = weapon type), shown under its role; ranges in the HUD's km.
/// Missiles: the longest range is the LOCK oval's (the lock range of assets/sim/units.ron,
/// 7/8 of it against the ground, the Hellfire the pod's 8 km), the shortest the launch zone's
/// (dlz.rs) from the given heights, level at 450 kt, with either guidance. Guided bombs (the
/// sim's Rockeye II and CBU-72 too) are in range below 2.5 times the height (targeting.rs, the
/// height compressed above 8,192 ft). The other bombs fall ballistically (ballistics.rs) from
/// a level release at 500 kt.
const EMPLOYMENT: [&[&str]; 21] = [
    &["Range: 1-12 km, farther head-on"],
    &["Range: 1-38 km, farther head-on"],
    &["Range: 9-50 km", "Launch: 5,000-20,000 ft"],
    &["Range: 6-14 km (classic guidance: 13-14 km)", "Launch: 100-500 ft"],
    &["Range: 6-41 km", "Launch: 2,000-5,000 ft"],
    &["Range: 5-10 km", "Launch: 2,000-5,000 ft"],
    &[
        "Range: 1.5 km to 2.5 times the height, LOCK steady (14 km from 15,000 ft, 18 km from 25,000 ft)",
        "Release: 15,000-25,000 ft",
    ],
    MEDIUM_RETARDED,
    &["Range: 0.9-1.3 km short of the runway", "Release: 200-500 ft"],
    &["Range: 5-11 km short of the target", "Release: 3,000-20,000 ft"],
    LOW_RETARDED,
    GUIDED_CLUSTER,
    &["Range: 0.9-2.2 km short of the target", "Release: 100-500 ft"],
    GUIDED_CLUSTER,
    MEDIUM_RETARDED,
    LOW_RETARDED,
    &["Range: photos from 2-8 km (the shorter, the more opponents), nose on the target"],
    &[],
    &["Range: release on \"Drop supplies now\", within 4 km of the drop zone"],
    &[],
    &["Range: 4-8 km", "Launch: 2,000-3,000 ft"],
];
const GUIDED_CLUSTER: &[&str] = &[
    "Range: 1.5 km to 2.5 times the height, LOCK steady (3.7 km from 3,000 ft, 6 km from 5,000 ft)",
    "Release: 3,000-5,000 ft",
];
const MEDIUM_RETARDED: &[&str] =
    &["Range: 1.1-1.6 km short of the target", "Release: 300-1,000 ft"];
const LOW_RETARDED: &[&str] = &["Range: 0.9-1.3 km short of the target", "Release: 200-500 ft"];
/// Weapon types kept out of the real F-117A's bays (realistic weapons).
const AIR_TO_AIR: [u8; 2] = [0, 1];
/// Rounds of each weapon a bay holds.
const BAY_QTY: [u8; 21] = [4, 3, 1, 2, 1, 2, 2, 2, 2, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 1, 4];
/// Rounds on a pylon (with a triple ejector rack for the 500 lb bombs, a four-rail launcher for
/// the Hellfires).
const PYLON_QTY: [u8; 21] = [1, 1, 1, 1, 1, 1, 1, 2, 2, 3, 3, 2, 2, 1, 2, 1, 1, 1, 1, 1, 4];
/// The Hellfire's weapon type.
pub const HELLFIRE: u8 = 20;

/// A bay or a pylon.
pub struct Hardpoint {
    pub name: &'static str,
    /// Short label on the drawing.
    pub label: &'static str,
    /// Position seen from above: metres right of the centreline, metres aft of the nose.
    pub pos: (f32, f32),
    pub bay: bool,
    /// Not on the real aircraft.
    pub fantasy: bool,
    /// The same station on the other side.
    pub mirror: Option<usize>,
    pub loads: &'static [u8],
}

impl Hardpoint {
    pub fn rounds(&self, w: u8) -> u8 {
        let t = if self.bay { &BAY_QTY } else { &PYLON_QTY };
        t.get(w as usize).copied().unwrap_or(0)
    }
}

pub struct Layout {
    /// The exterior model's outline seen from above (metres: x right of the centreline, y aft
    /// of the nose).
    pub outline: &'static [(f32, f32)],
    pub half_span: f32,
    pub length: f32,
    pub hardpoints: &'static [Hardpoint],
}

const ALL: [u8; 19] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];
const NO_AIR_TO_AIR: [u8; 17] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];

// tools/models/planform.py
/// aircraft_024 (tools/models/f117.py) seen from above: half span 6.48 m, length 20.26 m.
const F117_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (6.48, 14.96),
    (6.48, 16.48),
    (5.88, 17.14),
    (2.92, 13.66),
    (2.66, 13.60),
    (2.58, 13.68),
    (0.84, 16.10),
    (0.86, 16.42),
    (1.74, 19.28),
    (1.74, 20.26),
    (0.34, 17.56),
    (0.34, 17.08),
    (0.00, 17.06),
    (-0.34, 17.08),
    (-0.34, 17.56),
    (-1.74, 20.26),
    (-1.74, 19.28),
    (-0.86, 16.42),
    (-0.84, 16.10),
    (-2.58, 13.68),
    (-2.66, 13.60),
    (-2.92, 13.66),
    (-5.88, 17.14),
    (-6.48, 16.48),
    (-6.48, 14.96),
];
/// aircraft_043 (tools/models/ah1z.py) seen from above, without its rotors: half span (the
/// wingtip rails) 2.32 m, length 15.16 m.
const AH1Z_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (0.18, 0.02),
    (0.30, 0.20),
    (0.24, 0.56),
    (0.34, 0.66),
    (0.44, 1.12),
    (0.50, 4.10),
    (1.10, 4.10),
    (1.10, 2.70),
    (1.16, 2.68),
    (1.18, 5.04),
    (2.24, 5.14),
    (2.30, 4.80),
    (2.32, 5.20),
    (2.30, 6.44),
    (2.24, 6.36),
    (1.18, 6.44),
    (1.18, 7.74),
    (0.98, 7.76),
    (1.14, 8.62),
    (0.88, 8.80),
    (0.72, 8.82),
    (0.56, 8.48),
    (0.36, 8.48),
    (0.28, 8.72),
    (0.16, 11.84),
    (1.58, 12.00),
    (1.60, 12.78),
    (0.14, 12.76),
    (0.12, 14.02),
    (0.00, 15.16),
    (-0.12, 14.02),
    (-0.14, 12.76),
    (-1.60, 12.78),
    (-1.58, 12.00),
    (-0.16, 11.84),
    (-0.28, 8.72),
    (-0.36, 8.48),
    (-0.56, 8.48),
    (-0.72, 8.82),
    (-0.88, 8.80),
    (-1.14, 8.62),
    (-0.98, 7.76),
    (-1.18, 7.74),
    (-1.18, 6.44),
    (-2.24, 6.36),
    (-2.30, 6.44),
    (-2.32, 5.20),
    (-2.30, 4.80),
    (-2.24, 5.14),
    (-1.18, 5.04),
    (-1.16, 2.68),
    (-1.10, 2.70),
    (-1.10, 4.10),
    (-0.50, 4.10),
    (-0.44, 1.12),
    (-0.34, 0.66),
    (-0.24, 0.56),
    (-0.30, 0.20),
    (-0.18, 0.02),
];
/// aircraft_025 (tools/models/uh60.py) seen from above, without its rotors: half span (the
/// stabilator) 2.16 m, length 15.1 m.
const UH60_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (0.55, 0.22),
    (0.92, 0.57),
    (1.08, 0.90),
    (1.18, 1.47),
    (1.20, 2.12),
    (1.20, 7.37),
    (0.41, 8.17),
    (0.32, 10.02),
    (0.21, 12.32),
    (2.16, 13.32),
    (2.16, 14.20),
    (0.17, 13.90),
    (0.10, 15.10),
    (-0.10, 15.10),
    (-0.17, 13.90),
    (-2.16, 14.20),
    (-2.16, 13.32),
    (-0.21, 12.32),
    (-0.32, 10.02),
    (-0.41, 8.17),
    (-1.20, 7.37),
    (-1.20, 2.12),
    (-1.18, 1.47),
    (-1.08, 0.90),
    (-0.92, 0.57),
    (-0.55, 0.22),
];
/// aircraft_045 (tools/models/aircraft.py `tornado`, the wings forward) seen from above: half
/// span 6.98 m, length 17.04 m.
const TORNADO_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (0.00, 0.56),
    (0.30, 1.32),
    (0.50, 2.56),
    (0.66, 5.34),
    (0.70, 5.40),
    (1.40, 5.40),
    (1.40, 6.36),
    (2.34, 8.68),
    (3.58, 9.26),
    (3.80, 9.26),
    (3.82, 8.96),
    (3.96, 8.96),
    (4.00, 9.46),
    (6.68, 10.70),
    (6.98, 10.76),
    (6.98, 11.64),
    (6.90, 11.74),
    (1.76, 10.60),
    (1.76, 9.96),
    (1.30, 9.94),
    (1.14, 12.58),
    (3.42, 14.50),
    (3.42, 15.74),
    (0.78, 15.68),
    (0.58, 17.04),
    (0.50, 17.04),
    (0.36, 15.96),
    (0.02, 15.94),
    (0.00, 16.22),
    (-0.02, 15.94),
    (-0.36, 15.96),
    (-0.50, 17.04),
    (-0.58, 17.04),
    (-0.78, 15.68),
    (-3.42, 15.74),
    (-3.42, 14.50),
    (-1.14, 12.58),
    (-1.30, 9.94),
    (-1.76, 9.96),
    (-1.76, 10.60),
    (-6.90, 11.74),
    (-6.98, 11.64),
    (-6.98, 10.76),
    (-6.68, 10.70),
    (-4.00, 9.46),
    (-3.96, 8.96),
    (-3.82, 8.96),
    (-3.80, 9.26),
    (-3.58, 9.26),
    (-2.34, 8.68),
    (-1.40, 6.36),
    (-1.40, 5.40),
    (-0.70, 5.40),
    (-0.66, 5.34),
    (-0.50, 2.56),
    (-0.30, 1.32),
    (-0.00, 0.56),
];
/// aircraft_023 (tools/models/aircraft.py `f16`) seen from above: half span 4.70 m, length 15.88 m.
const F16_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (0.44, 1.62),
    (0.84, 4.58),
    (1.34, 6.84),
    (4.34, 9.26),
    (4.58, 9.24),
    (4.60, 8.10),
    (4.64, 7.78),
    (4.70, 8.10),
    (4.70, 11.14),
    (4.60, 11.14),
    (4.56, 10.66),
    (1.04, 10.68),
    (1.04, 12.00),
    (2.82, 13.46),
    (2.82, 14.34),
    (1.00, 14.34),
    (0.98, 14.08),
    (0.48, 14.06),
    (0.44, 14.44),
    (0.26, 14.46),
    (0.00, 15.88),
    (-0.26, 14.46),
    (-0.44, 14.44),
    (-0.48, 14.06),
    (-0.98, 14.08),
    (-1.00, 14.34),
    (-2.82, 14.34),
    (-2.82, 13.46),
    (-1.04, 12.00),
    (-1.04, 10.68),
    (-4.56, 10.66),
    (-4.60, 11.14),
    (-4.70, 11.14),
    (-4.70, 8.10),
    (-4.64, 7.78),
    (-4.60, 8.10),
    (-4.58, 9.24),
    (-4.34, 9.26),
    (-1.34, 6.84),
    (-0.84, 4.58),
    (-0.44, 1.62),
];

const fn bay(
    name: &'static str,
    label: &'static str,
    x: f32,
    y: f32,
    fantasy: bool,
    mirror: usize,
    loads: &'static [u8],
) -> Hardpoint {
    Hardpoint { name, label, pos: (x, y), bay: true, fantasy, mirror: Some(mirror), loads }
}

const fn pylon(
    name: &'static str,
    label: &'static str,
    x: f32,
    y: f32,
    mirror: Option<usize>,
    loads: &'static [u8],
) -> Hardpoint {
    Hardpoint { name, label, pos: (x, y), bay: false, fantasy: false, mirror, loads }
}

/// The real F-117A: two bays, no air-to-air missiles.
static F117_REAL: [Hardpoint; 2] = [
    bay("Left bay", "L", -0.52, 8.85, false, 1, &NO_AIR_TO_AIR),
    bay("Right bay", "R", 0.52, 8.85, false, 0, &NO_AIR_TO_AIR),
];
/// The fantasy F-117A: two more bays, inside the wing roots.
static F117_FANTASY: [Hardpoint; 4] = [
    bay("Left bay", "L", -0.52, 8.85, false, 1, &ALL),
    bay("Right bay", "R", 0.52, 8.85, false, 0, &ALL),
    bay("Left wing bay", "LW", -3.1, 10.5, true, 3, &ALL),
    bay("Right wing bay", "RW", 3.1, 10.5, true, 2, &ALL),
];
const TIP: [u8; 2] = [0, 1];
const WING_MID: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
const WING_INNER: [u8; 13] = [2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17];
const CENTRE: [u8; 3] = [16, 17, 18];
/// The F-16's stations 1 (left wingtip) to 9 (right wingtip).
static F16_STATIONS: [Hardpoint; 9] = [
    pylon("Station 1, left wingtip", "1", -4.66, 9.70, Some(8), &TIP),
    pylon("Station 2, left outer wing", "2", -3.88, 9.81, Some(7), &TIP),
    pylon("Station 3, left wing", "3", -2.93, 9.43, Some(6), &WING_MID),
    pylon("Station 4, left inner wing", "4", -1.89, 9.12, Some(5), &WING_INNER),
    pylon("Station 5, centreline", "5", 0.00, 8.79, None, &CENTRE),
    pylon("Station 6, right inner wing", "6", 1.89, 9.12, Some(3), &WING_INNER),
    pylon("Station 7, right wing", "7", 2.93, 9.43, Some(2), &WING_MID),
    pylon("Station 8, right outer wing", "8", 3.88, 9.81, Some(1), &TIP),
    pylon("Station 9, right wingtip", "9", 4.66, 9.70, Some(0), &TIP),
];

/// The Tornado's wing pylons (swivelling with the sweep) carry the tanks, the fuselage's
/// forward shoulder pylons the heavy stores and the camera; its Sidewinders go on the wing
/// pylons' rails.
const TORNADO_WING: [u8; 15] = [2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 0];
const TORNADO_BELLY: [u8; 16] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18];
/// The Tornado IDS's stations: the Sidewinder rails and pylons under the outer wings, and the
/// fuselage's shoulder pylons fore and aft.
static TORNADO_STATIONS: [Hardpoint; 8] = [
    pylon("Left wing, Sidewinder rail", "1", -4.20, 9.40, Some(7), &TIP),
    pylon("Left wing pylon", "2", -3.90, 9.80, Some(6), &TORNADO_WING),
    pylon("Left shoulder, forward", "3", -0.80, 7.40, Some(5), &TORNADO_BELLY),
    pylon("Left shoulder, aft", "4", -0.80, 11.30, Some(4), &WING_INNER),
    pylon("Right shoulder, aft", "5", 0.80, 11.30, Some(3), &WING_INNER),
    pylon("Right shoulder, forward", "6", 0.80, 7.40, Some(2), &TORNADO_BELLY),
    pylon("Right wing pylon", "7", 3.90, 9.80, Some(1), &TORNADO_WING),
    pylon("Right wing, Sidewinder rail", "8", 4.20, 9.40, Some(0), &TIP),
];

static F117_REAL_LAYOUT: Layout =
    Layout { outline: F117_OUTLINE, half_span: 6.48, length: 20.26, hardpoints: &F117_REAL };
static F117_FANTASY_LAYOUT: Layout =
    Layout { outline: F117_OUTLINE, half_span: 6.48, length: 20.26, hardpoints: &F117_FANTASY };
static F16_LAYOUT: Layout =
    Layout { outline: F16_OUTLINE, half_span: 4.70, length: 15.88, hardpoints: &F16_STATIONS };
static TORNADO_LAYOUT: Layout = Layout {
    outline: TORNADO_OUTLINE,
    half_span: 6.98,
    length: 17.04,
    hardpoints: &TORNADO_STATIONS,
};
/// The AH-1Z's stations 1 (left wingtip rail) to 6 (right wingtip rail).
static AH1Z_STATIONS: [Hardpoint; 6] = [
    pylon("Station 1, left wingtip rail", "1", -2.30, 5.6, Some(5), &SIDEWINDER),
    pylon("Station 2, left outer pylon", "2", -1.75, 5.5, Some(4), &HELLFIRES),
    pylon("Station 3, left inner pylon", "3", -1.05, 5.4, Some(3), &HELLFIRES),
    pylon("Station 4, right inner pylon", "4", 1.05, 5.4, Some(2), &HELLFIRES),
    pylon("Station 5, right outer pylon", "5", 1.75, 5.5, Some(1), &HELLFIRES),
    pylon("Station 6, right wingtip rail", "6", 2.30, 5.6, Some(0), &SIDEWINDER),
];
const SIDEWINDER: [u8; 1] = [0];
const HELLFIRES: [u8; 1] = [HELLFIRE];

static AH1Z_LAYOUT: Layout =
    Layout { outline: AH1Z_OUTLINE, half_span: 2.32, length: 15.16, hardpoints: &AH1Z_STATIONS };
/// The UH-60A carries troops, no stores.
static UH60_LAYOUT: Layout =
    Layout { outline: UH60_OUTLINE, half_span: 2.16, length: 15.1, hardpoints: &[] };

/// The hardpoints of an aircraft (`realistic`: the F-117A's real two bays).
pub fn layout(aircraft: AircraftKind, realistic: bool) -> &'static Layout {
    match aircraft {
        AircraftKind::F117 if realistic => &F117_REAL_LAYOUT,
        AircraftKind::F117 => &F117_FANTASY_LAYOUT,
        AircraftKind::F16 => &F16_LAYOUT,
        AircraftKind::Tornado => &TORNADO_LAYOUT,
        AircraftKind::UH60 => &UH60_LAYOUT,
        AircraftKind::AH1Z => &AH1Z_LAYOUT,
    }
}

/// Weapon types the mission has none of (random shortages).
fn out_of_stock(m: &Mission, w: u8) -> bool {
    !AIR_TO_AIR.contains(&w) && m.weapon_unavailable.get(w as usize).is_some_and(|&u| u != 0)
}

/// The stores flown: a weapon type (or nothing) on each hardpoint.
#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlannedLoadout {
    /// The mission it belongs to (`PlannedRoute::key`).
    pub mission: (String, u8, u32),
    pub aircraft: AircraftKind,
    pub realistic: bool,
    pub stores: Vec<Option<u8>>,
}

/// File the menu hands the loadout to the VR process in (`--loadout`).
pub const LOADOUT_FILE: &str = "loadout.json";

impl PlannedLoadout {
    /// The generator's stores: in the F-117A's bays as they are, on the F-16 and the Tornado
    /// each on the first free pair of stations that carries it, and Sidewinders on the free
    /// wingtips or rails; the AH-1Z full.
    pub fn of(m: &Mission, aircraft: AircraftKind) -> Self {
        let realistic = m.settings.plane == Plane::Lockheed;
        let lay = layout(aircraft, realistic);
        let briefed = m.loadout.bays.iter().take(m.loadout.usable_bays).map(|b| b.weapon as u8);
        let mut stores = vec![None; lay.hardpoints.len()];
        match aircraft {
            AircraftKind::F117 => {
                for (s, w) in stores.iter_mut().zip(briefed) {
                    *s = Some(w);
                }
            }
            AircraftKind::UH60 => {}
            // Whatever the briefing: the Sidewinders on the rails, a full launcher of Hellfires
            // on every pylon.
            AircraftKind::AH1Z => {
                for (s, h) in stores.iter_mut().zip(lay.hardpoints) {
                    *s = h.loads.first().copied();
                }
            }
            AircraftKind::F16 | AircraftKind::Tornado => {
                let tornado = aircraft == AircraftKind::Tornado;
                for w in briefed.chain([0]) {
                    // Stations preferred for each kind of store (pairs by their left station).
                    let order: &[usize] = match w {
                        0 if tornado => &[0, 1],
                        0 => &[0, 1, 2],
                        1 if tornado => &[0],
                        1 => &[1, 0, 2],
                        16 | 18 if tornado => &[2],
                        16 | 18 => &[4],
                        17 if tornado => &[1],
                        17 => &[4, 3],
                        _ if tornado => &[2, 3, 1],
                        _ => &[2, 3],
                    };
                    if w == 0 && stores.contains(&Some(0)) {
                        continue;
                    }
                    if let Some(&i) = order
                        .iter()
                        .find(|&&i| stores[i].is_none() && lay.hardpoints[i].loads.contains(&w))
                    {
                        stores[i] = Some(w);
                        if let Some(j) = lay.hardpoints[i].mirror {
                            stores[j] = Some(w);
                        }
                    }
                }
            }
        }
        Self { mission: crate::mission::PlannedRoute::key(m), aircraft, realistic, stores }
    }

    pub fn fits(&self, m: &Mission, aircraft: AircraftKind) -> bool {
        self.mission == crate::mission::PlannedRoute::key(m)
            && self.aircraft == aircraft
            && self.realistic == (m.settings.plane == Plane::Lockheed)
            && self.stores.len() == self.layout().hardpoints.len()
    }

    pub fn layout(&self) -> &'static Layout {
        layout(self.aircraft, self.realistic)
    }

    /// The simulation's stations: (weapon type, rounds), -1 empty.
    pub fn stations(&self) -> Vec<(i32, i32)> {
        self.layout()
            .hardpoints
            .iter()
            .zip(&self.stores)
            .map(|(h, s)| s.map_or((-1, 0), |w| (w as i32, h.rounds(w) as i32)))
            .collect()
    }

    /// The station selected at the start: the one with the briefed main weapon, else the first
    /// loaded one.
    pub fn first_choice(&self, m: &Mission) -> usize {
        let main = m.loadout.bays.first().map(|b| b.weapon as u8);
        self.stores
            .iter()
            .position(|s| s.is_some() && *s == main)
            .or_else(|| self.stores.iter().position(Option::is_some))
            .unwrap_or(0)
    }

    /// The stores in words, e.g. "4 AIM-9M Sidewinder, 2 GBU-12 Paveway".
    pub fn summary(&self) -> String {
        let mut totals: Vec<(u8, u32)> = Vec::new();
        for (h, s) in self.layout().hardpoints.iter().zip(&self.stores) {
            if let Some(w) = *s {
                match totals.iter_mut().find(|t| t.0 == w) {
                    Some(t) => t.1 += h.rounds(w) as u32,
                    None => totals.push((w, h.rounds(w) as u32)),
                }
            }
        }
        if totals.is_empty() {
            return "None".into();
        }
        totals
            .iter()
            .map(|(w, n)| format!("{n} {}", WEAPONS[*w as usize].0))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The loadout window.
#[derive(Resource, Default)]
pub struct LoadoutWindow {
    pub open: bool,
    selected: usize,
    mirror: Option<bool>,
}

impl LoadoutWindow {
    pub fn opened(open: bool) -> Self {
        Self { open, ..default() }
    }
}

pub fn loadout_ui(
    ctx: &egui::Context,
    w: &mut LoadoutWindow,
    state: &mut MenuState,
    aircraft: AircraftKind,
) {
    if !w.open {
        return;
    }
    let Some(Ok(m)) = state.mission.clone() else {
        w.open = false;
        return;
    };
    let lo = state.loadout.get_or_insert_with(|| PlannedLoadout::of(&m, aircraft));
    if !lo.fits(&m, aircraft) {
        *lo = PlannedLoadout::of(&m, aircraft);
    }
    let screen = ctx.content_rect();
    let size = egui::vec2(
        (screen.width() - 32.0).clamp(600.0, 1100.0),
        (screen.height() - 32.0).clamp(400.0, 760.0),
    );
    egui::Window::new("Loadout")
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .fixed_size(size)
        .frame(egui::Frame::window(&ctx.global_style()).inner_margin(12.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("LOADOUT").heading().color(BRIGHT));
                ui.label(
                    egui::RichText::new(format!("  {}", aircraft.name().to_ascii_uppercase()))
                        .color(DIM),
                );
            });
            ui.separator();
            editor(ui, w, lo, &m, size.y);
            ui.separator();
            ui.horizontal(|ui| {
                if facts(
                    ui.add(big_button("BRIEFED")),
                    &["The stores the mission was briefed with"],
                )
                .clicked()
                {
                    *lo = PlannedLoadout::of(&m, aircraft);
                }
                if facts(ui.add(big_button("CLEAR")), &["Every hardpoint empty"]).clicked() {
                    lo.stores.iter_mut().for_each(|s| *s = None);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(big_button("DONE").fill(DIM)).clicked() {
                        w.open = false;
                    }
                });
            });
        });
}

/// The editor: the aircraft's hardpoints and the stores of the selected one (the menu's
/// loadout window, the campaign's briefing). `height`: the space it may take.
pub fn editor(
    ui: &mut egui::Ui,
    w: &mut LoadoutWindow,
    lo: &mut PlannedLoadout,
    m: &Mission,
    height: f32,
) {
    let lay = lo.layout();
    w.selected = w.selected.min(lay.hardpoints.len().saturating_sub(1));
    let mirror = w.mirror.get_or_insert(true);
    let size = egui::vec2(ui.available_width(), height);
    ui.horizontal_top(|ui| {
        let side = 380.0;
        let draw = egui::vec2(ui.available_width() - side - 12.0, ui.available_height() - 60.0);
        if let Some(i) = diagram(ui, lay, &lo.stores, w.selected, draw) {
            w.selected = i;
        }
        ui.vertical(|ui| {
            ui.set_width(side);
            let Some(hp) = lay.hardpoints.get(w.selected) else {
                section(ui, "No hardpoints");
                ui.label(egui::RichText::new("This aircraft carries no stores").color(DIM));
                return;
            };
            section(ui, hp.name);
            if hp.fantasy {
                ui.label(egui::RichText::new("Not on the real aircraft").small().color(AMBER));
            }
            let mut pick = lo.stores[w.selected];
            // The column's full width: the scroll bar on its right edge, clear of the names.
            let list = egui::ScrollArea::vertical().max_height(size.y - 260.0);
            list.auto_shrink([false, true]).show(ui, |ui| {
                ui.radio_value(&mut pick, None, "Empty");
                for &wpn in hp.loads {
                    let (name, role) = WEAPONS[wpn as usize];
                    let stock = !out_of_stock(m, wpn);
                    let r = ui.add_enabled(
                        stock,
                        egui::RadioButton::new(
                            pick == Some(wpn),
                            format!("{}x {name}", hp.rounds(wpn)),
                        ),
                    );
                    let r = r.on_disabled_hover_text("None in stock for this mission");
                    let mut lines = vec![role];
                    lines.extend(EMPLOYMENT[wpn as usize]);
                    if facts(r, &lines).clicked() {
                        pick = Some(wpn);
                    }
                }
            });
            if let Some(j) = hp.mirror {
                ui.add_space(4.0);
                facts(
                    ui.checkbox(mirror, "Same on both sides"),
                    &[&format!("Loads {} too", lay.hardpoints[j].name.to_ascii_lowercase())],
                );
            }
            if pick != lo.stores[w.selected] {
                lo.stores[w.selected] = pick;
                if *mirror && let Some(j) = hp.mirror {
                    lo.stores[j] = pick;
                }
            }
            section(ui, "Stores");
            ui.label(egui::RichText::new(lo.summary()).color(GREEN));
            if m.loadout.bays.iter().any(|b| b.weapon == 16 || b.weapon == 18) {
                ui.label(
                    egui::RichText::new("The mission needs its camera or special equipment")
                        .small()
                        .color(AMBER),
                );
            }
        });
    });
}

/// The aircraft from above with its hardpoints; returns the one clicked.
fn diagram(
    ui: &mut egui::Ui,
    lay: &Layout,
    stores: &[Option<u8>],
    selected: usize,
    size: egui::Vec2,
) -> Option<usize> {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let s = (rect.width() / (2.0 * lay.half_span + 3.0)).min(rect.height() / (lay.length + 2.0));
    let origin = egui::pos2(rect.center().x, rect.center().y - lay.length * s / 2.0);
    let at = |x: f32, y: f32| origin + egui::vec2(x * s, y * s);
    let p = ui.painter_at(rect);
    let outline: Vec<egui::Pos2> = lay.outline.iter().map(|&(x, y)| at(x, y)).collect();
    p.add(egui::Shape::closed_line(outline, egui::Stroke::new(1.5, DIM)));
    let mut clicked = None;
    for (i, hp) in lay.hardpoints.iter().enumerate() {
        let c = at(hp.pos.0, hp.pos.1);
        let r = if hp.bay {
            egui::Rect::from_center_size(c, egui::vec2(0.8 * s, 3.4 * s))
        } else {
            egui::Rect::from_center_size(c, egui::vec2(16.0, 16.0))
        };
        let resp = ui.interact(r.expand(4.0), ui.id().with(("hardpoint", i)), egui::Sense::click());
        if resp.clicked() {
            clicked = Some(i);
        }
        let colour = if i == selected {
            BRIGHT
        } else if resp.hovered() {
            GREEN
        } else if hp.fantasy {
            AMBER
        } else {
            DIM
        };
        let fill = if stores[i].is_some() && !hp.bay {
            DIM.gamma_multiply(0.6)
        } else {
            egui::Color32::TRANSPARENT
        };
        let width = if i == selected { 2.5 } else { 1.5 };
        if hp.bay {
            p.rect(r, 0.0, fill, egui::Stroke::new(width, colour), egui::StrokeKind::Inside);
        } else {
            p.circle(c, 7.0, fill, egui::Stroke::new(width, colour));
        }
        let label_at = if hp.bay {
            r.center_bottom() + egui::vec2(0.0, 10.0)
        } else {
            c - egui::vec2(0.0, 18.0)
        };
        p.text(
            label_at,
            egui::Align2::CENTER_CENTER,
            hp.label,
            egui::FontId::monospace(12.0),
            colour,
        );
        // The store's name turned along the bay, or down from the pylon.
        let Some(w) = stores[i] else { continue };
        let name = WEAPONS[w as usize].0.split(' ').next().unwrap_or("").to_string();
        let galley =
            ui.ctx().fonts_mut(|f| f.layout_no_wrap(name, egui::FontId::monospace(11.0), colour));
        let size = galley.size();
        let quarter = std::f32::consts::FRAC_PI_2;
        let shape = if hp.bay {
            egui::epaint::TextShape::new(r.center() - size / 2.0, galley, colour)
                .with_angle_and_anchor(quarter, egui::Align2::CENTER_CENTER)
        } else {
            let start = c + egui::vec2(0.0, 12.0);
            egui::epaint::TextShape::new(start - egui::vec2(0.0, size.y / 2.0), galley, colour)
                .with_angle_and_anchor(quarter, egui::Align2::LEFT_CENTER)
        };
        p.add(shape);
    }
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mission(seed: u16, realistic: bool) -> Mission {
        let data = crate::mission::MissionData::load().expect("mission data");
        crate::mission::generate(
            &data,
            &crate::mission::settings(
                3,
                1,
                stealth_mission::MissionKind::Strike,
                1,
                realistic,
                None,
                None,
            ),
            seed,
        )
        .unwrap()
    }

    #[test]
    fn weapon_tables_match_the_sim_data() {
        let data = crate::mission::MissionData::load().expect("mission data");
        let t = &data.0.tables;
        assert_eq!(t.weapons.len(), BAY_QTY.len());
        for (w, &qty) in t.weapons.iter().zip(&BAY_QTY) {
            assert_eq!(w.bay, qty, "{}", w.name);
        }
    }

    #[test]
    fn missile_ranges_match_the_lock_ranges() {
        let sim = stealth_host::theater::sim_tables().expect("sim tables");
        for w in 0..6 {
            let lock = sim.missile(sim.weapons[w].spec as i32).lock_range as f64;
            // 7/8 of it against the ground, less a little for the height.
            let km = if w < 2 { lock } else { (lock * 0.875 - 0.5).ceil() };
            let range = format!("-{km} km");
            assert!(EMPLOYMENT[w][0].contains(&range), "{}: {range}", WEAPONS[w].0);
        }
    }

    #[test]
    fn the_f117_flies_the_briefed_bays() {
        for realistic in [false, true] {
            let m = mission(7, realistic);
            let lo = PlannedLoadout::of(&m, AircraftKind::F117);
            assert!(lo.fits(&m, AircraftKind::F117));
            let bays: Vec<(i32, i32)> = m
                .loadout
                .bays
                .iter()
                .take(m.loadout.usable_bays)
                .map(|b| (b.weapon as i32, b.count as i32))
                .collect();
            assert_eq!(lo.stations(), bays);
            assert_eq!(lo.first_choice(&m), 0);
        }
    }

    #[test]
    fn the_f16_carries_the_briefed_stores_in_pairs() {
        for seed in 0..40 {
            let m = mission(seed, false);
            let lo = PlannedLoadout::of(&m, AircraftKind::F16);
            for (i, (h, s)) in F16_STATIONS.iter().zip(&lo.stores).enumerate() {
                if let Some(w) = s {
                    assert!(h.loads.contains(w), "station {} {w}", i + 1);
                    if let Some(j) = h.mirror {
                        assert_eq!(lo.stores[j], *s);
                    }
                }
            }
            let main = m.loadout.bays[0].weapon as u8;
            assert_eq!(lo.stores[lo.first_choice(&m)], Some(main), "seed {seed}");
            assert_eq!(lo.stores[0], Some(0));
        }
    }

    #[test]
    fn the_editor_draws_every_aircraft() {
        let m = mission(7, false);
        for a in AircraftKind::ALL {
            let mut lo = PlannedLoadout::of(&m, a);
            let mut w = LoadoutWindow::opened(true);
            let ctx = egui::Context::default();
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                editor(ui, &mut w, &mut lo, &m, 600.0);
            });
            out.textures_delta.clear();
        }
    }
}

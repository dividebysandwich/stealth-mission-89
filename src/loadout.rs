//! Loadout editor (otherwise the mission generator picks the stores): the stores on each
//! hardpoint, the F-117A's bays (two real ones, two more inside the wings without realistic
//! weapons), the F-16's nine pylon stations, the A-10's eleven, the AH-1Z's six or the OH-58D's
//! two.

use bevy::prelude::*;
use bevy_egui::egui;
use serde::{Deserialize, Serialize};
use stealth_mission::{Mission, Plane};

use crate::aircraft::AircraftKind;
use crate::layout::{Split, screen_window};
use crate::menu::{AMBER, BRIGHT, DIM, GREEN, MenuState, big_button, facts, icon, section};
use stealth_i18n::{t, t_lines};

/// The weapons (index = weapon type): name (the real one, in every language), the message of
/// what it is for.
pub const WEAPONS: [(&str, &str); 24] = [
    ("AIM-9M Sidewinder", "weapon-role-0"),
    ("AIM-120A AMRAAM", "weapon-role-1"),
    ("AGM-88A HARM", "weapon-role-2"),
    ("Penguin 3 ASM", "weapon-role-3"),
    ("AGM-84A Harpoon", "weapon-role-4"),
    ("AGM-65 Maverick", "weapon-role-5"),
    ("GBU-12 Paveway", "weapon-role-6"),
    ("Mk 20 Rockeye", "weapon-role-7"),
    ("Durandal", "weapon-role-8"),
    ("Mk 82 LDGP", "weapon-role-9"),
    ("Mk 82 Snakeye", "weapon-role-10"),
    ("Mk 20 Rockeye II", "weapon-role-11"),
    ("Mk 122 Fireye", "weapon-role-12"),
    ("CBU-72 FAE", "weapon-role-13"),
    ("Mk 35 incendiary", "weapon-role-14"),
    ("ISCB-1 mines", "weapon-role-15"),
    ("IR/optical camera", "weapon-role-16"),
    ("1900 lb fuel tank", "weapon-role-17"),
    ("Special cargo", "weapon-role-18"),
    ("20 mm cannon", "weapon-role-19"),
    ("AGM-114 Hellfire", "weapon-role-20"),
    ("FIM-92 Stinger", "weapon-role-21"),
    ("M3P .50 cal gun pod", "weapon-role-22"),
    ("CBU-105 SFW", "weapon-role-23"),
];
/// How each weapon is used (index = weapon type; message ids, "" for nothing to say), shown
/// under its role; ranges in the HUD's km.
/// Missiles: the longest range is the LOCK oval's (the lock range of assets/sim/units.ron,
/// 7/8 of it against the ground, the Hellfire the pod's 8 km), the shortest the launch zone's
/// (dlz.rs) from the given heights, level at 450 kt, with either guidance. Guided bombs (the
/// sim's Rockeye II and CBU-72 too) are in range below 2.5 times the height (targeting.rs, the
/// height compressed above 8,192 ft). The other bombs fall ballistically (ballistics.rs) from
/// a level release at 500 kt.
/// The window's size, frame included.
const LOADOUT_SIZE: egui::Vec2 = egui::vec2(1132.0, 792.0);

const EMPLOYMENT: [&str; 24] = [
    "employment-0",
    "employment-1",
    "employment-2",
    "employment-3",
    "employment-4",
    "employment-5",
    "employment-6",
    MEDIUM_RETARDED,
    "employment-8",
    "employment-9",
    LOW_RETARDED,
    GUIDED_CLUSTER,
    "employment-12",
    GUIDED_CLUSTER,
    MEDIUM_RETARDED,
    LOW_RETARDED,
    "employment-16",
    "",
    "employment-18",
    "",
    "employment-20",
    "employment-21",
    "employment-22",
    "employment-23",
];
const GUIDED_CLUSTER: &str = "employment-guided-cluster";
const MEDIUM_RETARDED: &str = "employment-medium-retarded";
const LOW_RETARDED: &str = "employment-low-retarded";

/// How weapon `w` is used, in the player's language.
fn employment(w: usize) -> &'static [&'static str] {
    match EMPLOYMENT.get(w) {
        Some(&id) if !id.is_empty() => t_lines!(id),
        _ => &[],
    }
}

/// Weapon types kept out of the real F-117A's bays (realistic weapons).
const AIR_TO_AIR: [u8; 3] = [0, 1, STINGER];
/// Rounds of each weapon a bay holds.
const BAY_QTY: [u8; 24] = [4, 3, 1, 2, 1, 2, 2, 2, 2, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 1, 4, 6, 1, 2];
/// Rounds on a pylon (with a triple ejector rack for the 500 lb bombs, a four-rail launcher for
/// the Hellfires, a twin launcher for the Stingers).
const PYLON_QTY: [u8; 24] =
    [1, 1, 1, 1, 1, 1, 1, 2, 2, 3, 3, 2, 2, 1, 2, 1, 1, 1, 1, 1, 4, 2, 1, 1];
/// The Hellfire's, the Stinger's, the machine gun pod's and the CBU-105's weapon types.
pub const HELLFIRE: u8 = 20;
pub const STINGER: u8 = 21;
pub const GUN_POD: u8 = 22;
pub const CBU105: u8 = 23;

/// A bay or a pylon.
pub struct Hardpoint {
    /// Message id of its name.
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
    /// Rounds of a weapon here other than a bay's or a pylon's usual (weapon, rounds).
    pub qty: &'static [(u8, u8)],
    /// Stations whose stores would hit this one's: loading one empties the other.
    pub excludes: &'static [usize],
}

impl Hardpoint {
    const fn excluding(self, excludes: &'static [usize]) -> Self {
        Hardpoint { excludes, ..self }
    }

    const fn carrying(self, qty: &'static [(u8, u8)]) -> Self {
        Hardpoint { qty, ..self }
    }

    pub fn rounds(&self, w: u8) -> u8 {
        if let Some(&(_, n)) = self.qty.iter().find(|q| q.0 == w) {
            return n;
        }
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

const ALL: [u8; 20] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, CBU105];
const NO_AIR_TO_AIR: [u8; 18] =
    [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, CBU105];

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
/// aircraft_048 (tools/models/kiowa.py) seen from above, without its rotors and pylons: half span
/// (the skids) 1.18 m, length 10.32 m.
const OH58_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (0.00, 0.18),
    (0.16, 0.20),
    (0.42, 0.38),
    (0.66, 0.72),
    (0.72, 1.02),
    (0.70, 1.96),
    (1.12, 1.96),
    (1.12, 1.26),
    (1.18, 1.26),
    (1.18, 3.86),
    (1.12, 3.86),
    (1.10, 3.50),
    (0.68, 3.52),
    (0.22, 5.68),
    (0.18, 7.02),
    (0.92, 7.04),
    (0.92, 7.52),
    (0.16, 7.50),
    (0.10, 9.32),
    (0.04, 9.34),
    (0.00, 10.32),
    (-0.04, 9.34),
    (-0.10, 9.32),
    (-0.16, 7.50),
    (-0.92, 7.52),
    (-0.92, 7.04),
    (-0.18, 7.02),
    (-0.22, 5.68),
    (-0.68, 3.52),
    (-1.10, 3.50),
    (-1.12, 3.86),
    (-1.18, 3.86),
    (-1.18, 1.26),
    (-1.12, 1.26),
    (-1.12, 1.96),
    (-0.70, 1.96),
    (-0.72, 1.02),
    (-0.66, 0.72),
    (-0.42, 0.38),
    (-0.16, 0.20),
    (0.00, 0.18),
];
/// aircraft_046 (tools/models/comanche.py) seen from above, without its rotors, the bay doors shut:
/// half span (the tail's end plates) 1.58 m, length 13.44 m.
const RAH66_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (0.32, 0.08),
    (0.56, 0.44),
    (0.56, 0.96),
    (0.86, 2.36),
    (0.96, 3.30),
    (1.16, 3.34),
    (1.16, 3.84),
    (1.08, 3.86),
    (1.10, 6.74),
    (1.00, 7.88),
    (0.70, 8.60),
    (0.44, 10.98),
    (0.26, 11.06),
    (0.26, 12.12),
    (1.44, 12.10),
    (1.58, 12.16),
    (1.58, 12.78),
    (1.30, 13.28),
    (1.24, 13.10),
    (0.06, 13.16),
    (0.00, 13.44),
    (-0.06, 13.16),
    (-1.24, 13.10),
    (-1.30, 13.28),
    (-1.58, 12.78),
    (-1.58, 12.16),
    (-1.44, 12.10),
    (-0.26, 12.12),
    (-0.26, 11.06),
    (-0.44, 10.98),
    (-0.70, 8.60),
    (-1.00, 7.88),
    (-1.10, 6.74),
    (-1.08, 3.86),
    (-1.16, 3.84),
    (-1.16, 3.34),
    (-0.96, 3.30),
    (-0.86, 2.36),
    (-0.56, 0.96),
    (-0.56, 0.44),
    (-0.32, 0.08),
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
/// aircraft_049 (tools/models/chinook.py) seen from above, without its rotors: half span (the
/// aircraft_030 seen from above: half span 33.98 m, length 75.36 m.
const C5_OUTLINE: &[(f32, f32)] = &[(0.00, 0.00), (0.54, 0.14), (1.24, 0.74), (2.20, 2.40), (3.16, 5.38), (3.36, 6.40), (3.56, 8.50), (3.56, 21.36), (3.80, 22.56), (10.96, 26.24), (11.18, 26.24), (11.16, 25.34), (10.86, 25.28), (10.68, 24.08), (10.70, 22.62), (10.84, 22.40), (11.84, 22.40), (11.90, 22.32), (12.04, 22.40), (13.04, 22.40), (13.18, 22.62), (13.20, 24.08), (13.02, 25.28), (12.72, 25.34), (12.62, 27.08), (17.72, 29.70), (17.94, 29.68), (17.94, 29.38), (17.90, 28.98), (17.66, 28.96), (17.60, 28.88), (17.42, 27.66), (17.44, 26.30), (17.50, 26.18), (17.58, 26.06), (18.60, 26.04), (18.66, 25.98), (18.78, 26.04), (19.80, 26.06), (19.94, 26.30), (19.96, 27.66), (19.76, 28.94), (19.48, 28.98), (19.38, 30.54), (31.78, 36.90), (32.90, 37.96), (33.50, 38.90), (33.98, 40.42), (33.96, 42.58), (24.60, 40.16), (24.58, 38.38), (22.38, 37.78), (22.16, 37.78), (22.14, 40.48), (22.02, 41.88), (21.88, 42.24), (21.78, 41.88), (21.66, 40.48), (21.66, 37.58), (19.52, 37.00), (19.26, 37.02), (19.24, 39.84), (19.14, 41.08), (19.00, 41.48), (18.88, 41.08), (18.78, 39.84), (18.78, 36.82), (16.42, 36.16), (16.02, 36.12), (16.00, 38.86), (15.88, 40.26), (15.74, 40.62), (15.64, 40.26), (15.52, 38.86), (15.54, 35.94), (12.80, 35.18), (12.42, 35.14), (12.38, 38.04), (12.28, 39.28), (12.14, 39.68), (12.02, 39.28), (11.92, 38.04), (11.92, 34.96), (8.74, 34.08), (8.44, 34.06), (8.42, 36.86), (8.30, 38.28), (8.16, 38.64), (8.06, 38.28), (7.94, 36.86), (7.96, 33.90), (4.76, 33.00), (4.32, 32.98), (4.32, 33.70), (4.48, 33.72), (4.48, 34.08), (4.60, 34.10), (4.60, 35.12), (4.48, 35.14), (4.48, 35.88), (4.60, 35.90), (4.60, 36.34), (5.08, 36.36), (5.10, 35.90), (5.50, 35.90), (5.50, 36.92), (5.10, 36.92), (5.08, 36.46), (4.62, 36.46), (4.60, 36.92), (4.48, 36.94), (4.48, 37.30), (4.32, 37.32), (4.32, 39.32), (4.20, 40.58), (3.86, 42.22), (3.56, 42.90), (3.56, 53.64), (3.50, 55.30), (3.22, 58.26), (2.42, 62.72), (1.62, 66.60), (9.50, 70.90), (10.38, 71.94), (10.38, 73.74), (9.48, 73.90), (0.70, 72.46), (0.46, 74.24), (0.00, 75.36), (-0.46, 74.24), (-0.70, 72.46), (-9.48, 73.90), (-10.38, 73.74), (-10.38, 71.94), (-9.50, 70.90), (-1.62, 66.60), (-2.42, 62.72), (-3.22, 58.26), (-3.50, 55.30), (-3.56, 53.64), (-3.56, 42.90), (-3.86, 42.22), (-4.20, 40.58), (-4.32, 39.32), (-4.32, 37.32), (-4.48, 37.30), (-4.48, 36.94), (-4.60, 36.92), (-4.62, 36.46), (-5.08, 36.46), (-5.10, 36.92), (-5.50, 36.92), (-5.50, 35.90), (-5.10, 35.90), (-5.08, 36.36), (-4.60, 36.34), (-4.60, 35.90), (-4.48, 35.88), (-4.48, 35.14), (-4.60, 35.12), (-4.60, 34.10), (-4.48, 34.08), (-4.48, 33.72), (-4.32, 33.70), (-4.32, 32.98), (-4.76, 33.00), (-7.96, 33.90), (-7.94, 36.86), (-8.06, 38.28), (-8.16, 38.64), (-8.30, 38.28), (-8.42, 36.86), (-8.44, 34.06), (-8.74, 34.08), (-11.92, 34.96), (-11.92, 38.04), (-12.02, 39.28), (-12.14, 39.68), (-12.28, 39.28), (-12.38, 38.04), (-12.42, 35.14), (-12.80, 35.18), (-15.54, 35.94), (-15.52, 38.86), (-15.64, 40.26), (-15.74, 40.62), (-15.88, 40.26), (-16.00, 38.86), (-16.02, 36.12), (-16.42, 36.16), (-18.78, 36.82), (-18.78, 39.84), (-18.88, 41.08), (-19.00, 41.48), (-19.14, 41.08), (-19.24, 39.84), (-19.26, 37.02), (-19.52, 37.00), (-21.66, 37.58), (-21.66, 40.48), (-21.78, 41.88), (-21.88, 42.24), (-22.02, 41.88), (-22.14, 40.48), (-22.16, 37.78), (-22.38, 37.78), (-24.58, 38.38), (-24.60, 40.16), (-33.96, 42.58), (-33.98, 40.42), (-33.50, 38.90), (-32.90, 37.96), (-31.78, 36.90), (-19.38, 30.54), (-19.48, 28.98), (-19.76, 28.94), (-19.96, 27.66), (-19.94, 26.30), (-19.80, 26.06), (-18.78, 26.04), (-18.66, 25.98), (-18.60, 26.04), (-17.58, 26.06), (-17.50, 26.18), (-17.44, 26.30), (-17.42, 27.66), (-17.60, 28.88), (-17.66, 28.96), (-17.90, 28.98), (-17.94, 29.38), (-17.94, 29.68), (-17.72, 29.70), (-12.62, 27.08), (-12.72, 25.34), (-13.02, 25.28), (-13.20, 24.08), (-13.18, 22.62), (-13.04, 22.40), (-12.04, 22.40), (-11.90, 22.32), (-11.84, 22.40), (-10.84, 22.40), (-10.70, 22.62), (-10.68, 24.08), (-10.86, 25.28), (-11.16, 25.34), (-11.18, 26.24), (-10.96, 26.24), (-3.80, 22.56), (-3.56, 21.36), (-3.56, 8.50), (-3.36, 6.40), (-3.16, 5.38), (-2.20, 2.40), (-1.24, 0.74), (-0.54, 0.14)];
/// sponsons) 1.94 m, length 15.84 m.
const CH47_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (0.34, 0.00),
    (0.74, 0.14),
    (0.92, 0.30),
    (1.18, 0.74),
    (1.32, 1.44),
    (1.32, 3.92),
    (1.72, 4.02),
    (1.92, 4.16),
    (1.94, 5.76),
    (1.92, 12.40),
    (1.88, 12.66),
    (1.76, 12.82),
    (1.64, 14.12),
    (1.32, 14.16),
    (1.30, 15.34),
    (1.08, 15.60),
    (1.08, 15.84),
    (0.00, 15.84),
    (-1.08, 15.84),
    (-1.08, 15.60),
    (-1.30, 15.34),
    (-1.32, 14.16),
    (-1.64, 14.12),
    (-1.76, 12.82),
    (-1.88, 12.66),
    (-1.92, 12.40),
    (-1.94, 5.76),
    (-1.92, 4.16),
    (-1.72, 4.02),
    (-1.32, 3.92),
    (-1.32, 1.44),
    (-1.18, 0.74),
    (-0.92, 0.30),
    (-0.74, 0.14),
    (-0.34, 0.00),
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
/// aircraft_032 (tools/models/aircraft.py `a10`) seen from above: half span 8.78 m, length 16.44 m.
const A10_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (0.10, 0.04),
    (0.10, 0.26),
    (0.22, 0.36),
    (0.34, 0.70),
    (0.64, 2.46),
    (0.70, 7.00),
    (2.26, 7.10),
    (2.32, 6.56),
    (2.56, 6.24),
    (2.80, 6.56),
    (2.88, 7.18),
    (8.50, 7.64),
    (8.78, 7.76),
    (8.78, 9.56),
    (8.58, 9.66),
    (2.10, 10.06),
    (2.06, 12.22),
    (1.94, 12.92),
    (1.70, 13.36),
    (1.16, 13.36),
    (0.90, 12.86),
    (0.80, 11.86),
    (0.50, 11.84),
    (0.38, 14.24),
    (2.82, 14.30),
    (2.86, 14.08),
    (2.92, 14.46),
    (2.86, 16.00),
    (2.86, 15.66),
    (2.80, 15.60),
    (0.32, 15.64),
    (0.00, 16.44),
    (-0.32, 15.64),
    (-2.80, 15.60),
    (-2.86, 15.66),
    (-2.86, 16.00),
    (-2.92, 14.46),
    (-2.86, 14.08),
    (-2.82, 14.30),
    (-0.38, 14.24),
    (-0.50, 11.84),
    (-0.80, 11.86),
    (-0.90, 12.86),
    (-1.16, 13.36),
    (-1.70, 13.36),
    (-1.94, 12.92),
    (-2.06, 12.22),
    (-2.10, 10.06),
    (-8.58, 9.66),
    (-8.78, 9.56),
    (-8.78, 7.76),
    (-8.50, 7.64),
    (-2.88, 7.18),
    (-2.80, 6.56),
    (-2.56, 6.24),
    (-2.32, 6.56),
    (-2.26, 7.10),
    (-0.70, 7.00),
    (-0.64, 2.46),
    (-0.34, 0.70),
    (-0.22, 0.36),
    (-0.10, 0.26),
    (-0.10, 0.04),
];
/// aircraft_023 (tools/models/aircraft.py `f16`) seen from above: half span 4.84 m, length 15.90 m.
const F16_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (0.42, 1.54),
    (0.84, 4.62),
    (1.34, 6.86),
    (3.62, 8.70),
    (3.90, 8.70),
    (3.90, 8.92),
    (4.52, 9.42),
    (4.74, 9.40),
    (4.74, 8.10),
    (4.78, 7.78),
    (4.84, 8.16),
    (4.84, 11.16),
    (4.74, 11.16),
    (4.72, 10.68),
    (3.92, 10.68),
    (3.90, 11.08),
    (3.82, 11.08),
    (3.80, 10.68),
    (1.02, 10.70),
    (1.02, 12.00),
    (2.80, 13.46),
    (2.80, 14.36),
    (1.00, 14.36),
    (0.98, 14.10),
    (0.48, 14.08),
    (0.44, 14.44),
    (0.24, 14.52),
    (0.00, 15.90),
    (-0.24, 14.52),
    (-0.44, 14.44),
    (-0.48, 14.08),
    (-0.98, 14.10),
    (-1.00, 14.36),
    (-2.80, 14.36),
    (-2.80, 13.46),
    (-1.02, 12.00),
    (-1.02, 10.70),
    (-3.80, 10.68),
    (-3.82, 11.08),
    (-3.90, 11.08),
    (-3.92, 10.68),
    (-4.72, 10.68),
    (-4.74, 11.16),
    (-4.84, 11.16),
    (-4.84, 8.16),
    (-4.78, 7.78),
    (-4.74, 8.10),
    (-4.74, 9.40),
    (-4.52, 9.42),
    (-3.90, 8.92),
    (-3.90, 8.70),
    (-3.62, 8.70),
    (-1.34, 6.86),
    (-0.84, 4.62),
    (-0.42, 1.54),
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
    Hardpoint {
        name,
        label,
        pos: (x, y),
        bay: true,
        fantasy,
        mirror: Some(mirror),
        loads,
        qty: &[],
        excludes: &[],
    }
}

const fn pylon(
    name: &'static str,
    label: &'static str,
    x: f32,
    y: f32,
    mirror: Option<usize>,
    loads: &'static [u8],
) -> Hardpoint {
    Hardpoint {
        name,
        label,
        pos: (x, y),
        bay: false,
        fantasy: false,
        mirror,
        loads,
        qty: &[],
        excludes: &[],
    }
}

/// The real F-117A: two bays, no air-to-air missiles.
static F117_REAL: [Hardpoint; 2] = [
    bay("hardpoint-left-bay", "L", -0.52, 8.85, false, 1, &NO_AIR_TO_AIR),
    bay("hardpoint-right-bay", "R", 0.52, 8.85, false, 0, &NO_AIR_TO_AIR),
];
/// The fantasy F-117A: two more bays, inside the wing roots.
static F117_FANTASY: [Hardpoint; 4] = [
    bay("hardpoint-left-bay", "L", -0.52, 8.85, false, 1, &ALL),
    bay("hardpoint-right-bay", "R", 0.52, 8.85, false, 0, &ALL),
    bay("hardpoint-left-wing-bay", "LW", -3.1, 10.5, true, 3, &ALL),
    bay("hardpoint-right-wing-bay", "RW", 3.1, 10.5, true, 2, &ALL),
];
const TIP: [u8; 2] = [0, 1];
const WING_MID: [u8; 18] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, HELLFIRE, CBU105];
const WING_INNER: [u8; 13] = [2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17];
const F16_INNER: [u8; 15] = [2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, HELLFIRE, CBU105];
const CENTRE: [u8; 3] = [16, 17, 18];
/// Three Hellfires on a wing pylon's triple ejector rack (BRU-42).
const TER_HELLFIRES: [(u8, u8); 1] = [(HELLFIRE, 3)];
/// The wing pylons' stores beside the inner ones' leave no room for a rack of the fat or widely
/// finned ones (the Snakeye's retarders, the Fireye, the Mk 35): one straight on the pylon
/// (tools/models/store_clearance.py).
const F16_MID_QTY: [(u8, u8); 4] = [(HELLFIRE, 3), (10, 1), (12, 1), (14, 1)];
/// The F-16's stations 1 (left wingtip) to 9 (right wingtip); the wing pylons take a triple
/// ejector rack of Hellfires.
static F16_STATIONS: [Hardpoint; 9] = [
    pylon("hardpoint-station-1-left-wingtip", "1", -4.80, 9.70, Some(8), &TIP),
    pylon("hardpoint-station-2-left-outer-wing", "2", -3.88, 9.81, Some(7), &TIP),
    pylon("hardpoint-station-3-left-wing", "3", -2.93, 9.43, Some(6), &WING_MID)
        .carrying(&F16_MID_QTY),
    pylon("hardpoint-station-4-left-inner-wing", "4", -1.89, 9.12, Some(5), &F16_INNER)
        .carrying(&TER_HELLFIRES),
    pylon("hardpoint-station-5-centreline", "5", 0.00, 8.79, None, &CENTRE),
    pylon("hardpoint-station-6-right-inner-wing", "6", 1.89, 9.12, Some(3), &F16_INNER)
        .carrying(&TER_HELLFIRES),
    pylon("hardpoint-station-7-right-wing", "7", 2.93, 9.43, Some(2), &WING_MID)
        .carrying(&F16_MID_QTY),
    pylon("hardpoint-station-8-right-outer-wing", "8", 3.88, 9.81, Some(1), &TIP),
    pylon("hardpoint-station-9-right-wingtip", "9", 4.80, 9.70, Some(0), &TIP),
];

/// The Tornado's wing pylons (swivelling with the sweep) carry the tanks, the fuselage's
/// forward shoulder pylons the heavy stores and the camera; its Sidewinders go on the wing
/// pylons' rails.
const TORNADO_WING: [u8; 16] = [2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 0, CBU105];
const TORNADO_BELLY: [u8; 17] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18, CBU105];
/// The Tornado IDS's stations: the Sidewinder rails and pylons under the outer wings, and the
/// fuselage's shoulder pylons fore and aft. The rail is drawn further out than its pylon's
/// side, where it is apart from the pylon.
/// Two Snakeyes forward (a full rack's retarders reach the gear), one Fireye aft.
const TORNADO_FORWARD_QTY: [(u8, u8); 1] = [(10, 2)];
const TORNADO_AFT_QTY: [(u8, u8); 1] = [(12, 1)];
static TORNADO_STATIONS: [Hardpoint; 8] = [
    pylon("hardpoint-tornado-left-wing-rail", "1", -5.20, 10.70, Some(7), &TIP),
    pylon("hardpoint-tornado-left-wing-pylon", "2", -3.60, 10.20, Some(6), &TORNADO_WING),
    pylon("hardpoint-tornado-left-shoulder-forward", "3", -0.80, 7.15, Some(5), &TORNADO_BELLY)
        .carrying(&TORNADO_FORWARD_QTY),
    pylon("hardpoint-tornado-left-shoulder-aft", "4", -0.80, 11.50, Some(4), &WING_INNER)
        .carrying(&TORNADO_AFT_QTY),
    pylon("hardpoint-tornado-right-shoulder-aft", "5", 0.80, 11.50, Some(3), &WING_INNER)
        .carrying(&TORNADO_AFT_QTY),
    pylon("hardpoint-tornado-right-shoulder-forward", "6", 0.80, 7.15, Some(2), &TORNADO_BELLY)
        .carrying(&TORNADO_FORWARD_QTY),
    pylon("hardpoint-tornado-right-wing-pylon", "7", 3.60, 10.20, Some(1), &TORNADO_WING),
    pylon("hardpoint-tornado-right-wing-rail", "8", 5.20, 10.70, Some(0), &TIP),
];

/// The A-10's outer wing pylons carry the Mavericks and the bombs, the centre section's the
/// heavy bombs and the tanks, the fuselage's the lighter bombs; the centreline the tank, the
/// camera or the cargo. No anti-ship or anti-radar missiles.
const A10_WING: [u8; 12] = [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, CBU105];
const A10_INNER: [u8; 12] = [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, CBU105];
const A10_BELLY: [u8; 7] = [6, 7, 9, 10, 11, 12, 14];
/// The fuselage stations' racks of the fat or widely finned stores would meet: one of them each.
const A10_BELLY_QTY: [(u8, u8); 5] = [(7, 1), (10, 1), (11, 1), (12, 1), (14, 1)];
/// The A-10's eleven stations: 1 (left outer wing, the Sidewinders) to 11.
static A10_STATIONS: [Hardpoint; 11] = [
    pylon("hardpoint-a10-1-left-outer-wing", "1", -5.95, 8.63, Some(10), &SIDEWINDER),
    pylon("hardpoint-a10-2-left-wing", "2", -4.75, 8.53, Some(9), &A10_WING),
    pylon("hardpoint-a10-3-left-wing", "3", -3.55, 8.43, Some(8), &A10_WING),
    pylon("hardpoint-a10-4-left-centre-wing", "4", -1.65, 8.27, Some(7), &A10_INNER),
    pylon("hardpoint-a10-5-left-fuselage", "5", -0.45, 8.11, Some(6), &A10_BELLY)
        .excluding(&[5])
        .carrying(&A10_BELLY_QTY),
    pylon("hardpoint-a10-6-centreline", "6", 0.0, 8.11, None, &CENTRE).excluding(&[4, 6]),
    pylon("hardpoint-a10-7-right-fuselage", "7", 0.45, 8.11, Some(4), &A10_BELLY)
        .excluding(&[5])
        .carrying(&A10_BELLY_QTY),
    pylon("hardpoint-a10-8-right-centre-wing", "8", 1.65, 8.27, Some(3), &A10_INNER),
    pylon("hardpoint-a10-9-right-wing", "9", 3.55, 8.43, Some(2), &A10_WING),
    pylon("hardpoint-a10-10-right-wing", "10", 4.75, 8.53, Some(1), &A10_WING),
    pylon("hardpoint-a10-11-right-outer-wing", "11", 5.95, 8.63, Some(0), &SIDEWINDER),
];

static F117_REAL_LAYOUT: Layout =
    Layout { outline: F117_OUTLINE, half_span: 6.48, length: 20.26, hardpoints: &F117_REAL };
static F117_FANTASY_LAYOUT: Layout =
    Layout { outline: F117_OUTLINE, half_span: 6.48, length: 20.26, hardpoints: &F117_FANTASY };
static F16_LAYOUT: Layout =
    Layout { outline: F16_OUTLINE, half_span: 4.84, length: 15.90, hardpoints: &F16_STATIONS };
static A10_LAYOUT: Layout =
    Layout { outline: A10_OUTLINE, half_span: 8.78, length: 16.44, hardpoints: &A10_STATIONS };
/// The AV-8B's outer pylons carry the Mavericks, the bombs and Sidewinders, the intermediate ones
/// (on the outriggers' fairings, ahead of the legs) the heavier stores but the Rockeye pairs, the
/// inboard ones (by the intakes) the tanks, the camera, the cargo and the Harpoon too; the
/// Sidewinder rails outboard. No centreline station: the gun pods and the bicycle gear's legs
/// leave no room for a store between them.
const AV8B_OUTER: [u8; 11] = [0, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15];
const AV8B_MIDDLE: [u8; 10] = [5, 6, 8, 9, 10, 12, 13, 14, 15, CBU105];
const AV8B_INNER: [u8; 16] = [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, CBU105];
/// The AV-8B's pylons are close together: the fat or widely finned stores one to a pylon, and on
/// the intermediate ones (by the outrigger legs) the Mk 82s and Durandals too.
const AV8B_OUTER_QTY: [(u8, u8); 3] = [(10, 1), (12, 1), (14, 1)];
const AV8B_MIDDLE_QTY: [(u8, u8); 5] = [(8, 1), (9, 1), (10, 1), (12, 1), (14, 1)];
const AV8B_INNER_QTY: [(u8, u8); 3] = [(10, 1), (12, 1), (14, 1)];
/// The AV-8B's stations: the left Sidewinder rail (1) to the right one (9), numbered as the
/// real ones (5, the centreline, not fitted).
static AV8B_STATIONS: [Hardpoint; 8] = [
    pylon("hardpoint-av8b-1-left-rail", "1", -3.90, 8.64, Some(7), &SIDEWINDER),
    pylon("hardpoint-station-2-left-outer-wing", "2", -3.28, 8.69, Some(6), &AV8B_OUTER)
        .carrying(&AV8B_OUTER_QTY),
    pylon("hardpoint-av8b-3-left-middle-wing", "3", -2.60, 8.19, Some(5), &AV8B_MIDDLE)
        .carrying(&AV8B_MIDDLE_QTY),
    pylon("hardpoint-av8b-4-left-inboard-wing", "4", -1.88, 7.66, Some(4), &AV8B_INNER)
        .carrying(&AV8B_INNER_QTY),
    pylon("hardpoint-av8b-6-right-inboard-wing", "6", 1.88, 7.66, Some(3), &AV8B_INNER)
        .carrying(&AV8B_INNER_QTY),
    pylon("hardpoint-av8b-7-right-middle-wing", "7", 2.60, 8.19, Some(2), &AV8B_MIDDLE)
        .carrying(&AV8B_MIDDLE_QTY),
    pylon("hardpoint-station-8-right-outer-wing", "8", 3.28, 8.69, Some(1), &AV8B_OUTER)
        .carrying(&AV8B_OUTER_QTY),
    pylon("hardpoint-av8b-9-right-rail", "9", 3.90, 8.64, Some(0), &SIDEWINDER),
];
static AV8B_LAYOUT: Layout =
    Layout { outline: AV8B_OUTLINE, half_span: 4.46, length: 14.52, hardpoints: &AV8B_STATIONS };

/// The F-22A: a Sidewinder in each side bay (on the intakes' flanks), any one store in each half
/// of the main bay under the fuselage (two AIM-120s, two GBU-32s and the like).
const F22_MAIN: [u8; 21] =
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, HELLFIRE, CBU105];
const F22_SIDE_QTY: [(u8, u8); 1] = [(0, 1)];
static F22_STATIONS: [Hardpoint; 4] = [
    bay("hardpoint-f22-left-side-bay", "LS", -1.75, 8.85, false, 3, &SIDEWINDER)
        .carrying(&F22_SIDE_QTY),
    bay("hardpoint-f22-left-main-bay", "L", -0.45, 9.45, false, 2, &F22_MAIN),
    bay("hardpoint-f22-right-main-bay", "R", 0.45, 9.45, false, 1, &F22_MAIN),
    bay("hardpoint-f22-right-side-bay", "RS", 1.75, 8.85, false, 0, &SIDEWINDER)
        .carrying(&F22_SIDE_QTY),
];
static F22_LAYOUT: Layout =
    Layout { outline: F22_OUTLINE, half_span: 6.76, length: 18.62, hardpoints: &F22_STATIONS };
static TORNADO_LAYOUT: Layout = Layout {
    outline: TORNADO_OUTLINE,
    half_span: 6.98,
    length: 17.04,
    hardpoints: &TORNADO_STATIONS,
};
/// The AH-1Z's stations 1 (left wingtip rail) to 6 (right wingtip rail).
static AH1Z_STATIONS: [Hardpoint; 6] = [
    pylon("hardpoint-station-1-left-wingtip-rail", "1", -2.30, 5.6, Some(5), &SIDEWINDER),
    pylon("hardpoint-station-2-left-outer-pylon", "2", -1.75, 5.5, Some(4), &HELLFIRES),
    pylon("hardpoint-station-3-left-inner-pylon", "3", -1.05, 5.4, Some(3), &HELLFIRES),
    pylon("hardpoint-station-4-right-inner-pylon", "4", 1.05, 5.4, Some(2), &HELLFIRES),
    pylon("hardpoint-station-5-right-outer-pylon", "5", 1.75, 5.5, Some(1), &HELLFIRES),
    pylon("hardpoint-station-6-right-wingtip-rail", "6", 2.30, 5.6, Some(0), &SIDEWINDER),
];
/// aircraft_075 (tools/models/aircraft.py `f22`) seen from above, the bays shut: half span 6.76
/// m, length 18.62 m.
const F22_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (0.48, 1.10),
    (0.70, 2.08),
    (0.92, 4.82),
    (1.88, 5.60),
    (2.08, 7.48),
    (2.08, 8.30),
    (6.76, 12.38),
    (6.76, 13.62),
    (6.04, 14.28),
    (5.90, 14.38),
    (3.42, 15.16),
    (3.42, 15.38),
    (4.56, 16.84),
    (4.56, 18.14),
    (2.82, 18.62),
    (1.30, 17.34),
    (1.08, 17.36),
    (1.08, 17.64),
    (1.04, 17.58),
    (0.78, 17.64),
    (0.72, 17.58),
    (0.46, 17.64),
    (0.18, 17.58),
    (0.04, 17.66),
    (0.00, 17.86),
    (-0.04, 17.66),
    (-0.18, 17.58),
    (-0.46, 17.64),
    (-0.72, 17.58),
    (-0.78, 17.64),
    (-1.04, 17.58),
    (-1.08, 17.64),
    (-1.08, 17.36),
    (-1.30, 17.34),
    (-2.82, 18.62),
    (-4.56, 18.14),
    (-4.56, 16.84),
    (-3.42, 15.38),
    (-3.42, 15.16),
    (-5.90, 14.38),
    (-6.04, 14.28),
    (-6.76, 13.62),
    (-6.76, 12.38),
    (-2.08, 8.30),
    (-2.08, 7.48),
    (-1.88, 5.60),
    (-0.92, 4.82),
    (-0.70, 2.08),
    (-0.48, 1.10),
];
/// aircraft_047 (tools/models/aircraft.py `harrier`) seen from above: half span 4.46 m, length
/// 14.52 m.
const AV8B_OUTLINE: &[(f32, f32)] = &[
    (0.00, 0.00),
    (0.50, 1.40),
    (0.56, 3.62),
    (1.20, 3.68),
    (1.22, 4.92),
    (1.14, 6.04),
    (4.46, 8.46),
    (4.46, 9.28),
    (3.34, 9.24),
    (3.32, 9.42),
    (3.22, 9.42),
    (3.18, 9.22),
    (2.70, 9.22),
    (2.60, 10.16),
    (2.48, 9.80),
    (2.46, 9.20),
    (0.66, 9.16),
    (0.32, 11.96),
    (2.08, 13.30),
    (2.08, 13.80),
    (0.14, 13.44),
    (0.10, 14.32),
    (0.00, 14.52),
    (-0.10, 14.32),
    (-0.14, 13.44),
    (-2.08, 13.80),
    (-2.08, 13.30),
    (-0.32, 11.96),
    (-0.66, 9.16),
    (-2.46, 9.20),
    (-2.48, 9.80),
    (-2.60, 10.16),
    (-2.70, 9.22),
    (-3.18, 9.22),
    (-3.22, 9.42),
    (-3.32, 9.42),
    (-3.34, 9.24),
    (-4.46, 9.28),
    (-4.46, 8.46),
    (-1.14, 6.04),
    (-1.22, 4.92),
    (-1.20, 3.68),
    (-0.56, 3.62),
    (-0.50, 1.40),
];
const SIDEWINDER: [u8; 1] = [0];
const HELLFIRES: [u8; 1] = [HELLFIRE];

static AH1Z_LAYOUT: Layout =
    Layout { outline: AH1Z_OUTLINE, half_span: 2.32, length: 15.16, hardpoints: &AH1Z_STATIONS };
/// The RAH-66's bay doors, left and right: each swings up and out with three Hellfires or six
/// Stingers on its inside.
static RAH66_STATIONS: [Hardpoint; 2] =
    [door("Left bay door", "L", -1.0, 1), door("Right bay door", "R", 1.0, 0)];

const fn door(name: &'static str, label: &'static str, x: f32, mirror: usize) -> Hardpoint {
    Hardpoint {
        name,
        label,
        pos: (x, 5.3),
        bay: true,
        fantasy: false,
        mirror: Some(mirror),
        loads: &[HELLFIRE, STINGER],
        qty: &[(HELLFIRE, 3), (STINGER, 6)],
        excludes: &[],
    }
}

static RAH66_LAYOUT: Layout =
    Layout { outline: RAH66_OUTLINE, half_span: 1.58, length: 13.44, hardpoints: &RAH66_STATIONS };
/// The OH-58D's universal weapons pylons, left and right: two Hellfires, two Stingers or a
/// .50 cal machine gun pod each, or none (the pylon left off).
static OH58_STATIONS: [Hardpoint; 2] = [
    Hardpoint {
        name: "hardpoint-oh58-left-pylon",
        label: "L",
        pos: (-1.36, 3.04),
        bay: false,
        fantasy: false,
        mirror: Some(1),
        loads: &OH58_LOADS,
        qty: &[(HELLFIRE, 2), (STINGER, 2), (GUN_POD, 1)],
        excludes: &[],
    },
    Hardpoint {
        name: "hardpoint-oh58-right-pylon",
        label: "R",
        pos: (1.36, 3.04),
        bay: false,
        fantasy: false,
        mirror: Some(0),
        loads: &OH58_LOADS,
        qty: &[(HELLFIRE, 2), (STINGER, 2), (GUN_POD, 1)],
        excludes: &[],
    },
];
const OH58_LOADS: [u8; 3] = [HELLFIRE, STINGER, GUN_POD];

static OH58_LAYOUT: Layout =
    Layout { outline: OH58_OUTLINE, half_span: 1.40, length: 10.32, hardpoints: &OH58_STATIONS };
/// The UH-60A carries troops, no stores.
static UH60_LAYOUT: Layout =
    Layout { outline: UH60_OUTLINE, half_span: 2.16, length: 15.1, hardpoints: &[] };
/// The CH-47 slings cargo, no stores.
static CH47_LAYOUT: Layout =
    Layout { outline: CH47_OUTLINE, half_span: 1.94, length: 15.84, hardpoints: &[] };

/// The C-5 carries its cargo inside: the airdrop pallets in the hold, dropped over the ramp.
static C5_STATIONS: [Hardpoint; 1] = [Hardpoint {
    name: "hardpoint-c5-hold",
    label: "HOLD",
    pos: (0.0, 40.0),
    bay: true,
    fantasy: false,
    mirror: None,
    loads: &[18],
    qty: &[(18, 4)],
    excludes: &[],
}];
static C5_LAYOUT: Layout =
    Layout { outline: C5_OUTLINE, half_span: 33.98, length: 75.36, hardpoints: &C5_STATIONS };

/// The hardpoints of an aircraft (`realistic`: the F-117A's real two bays).
pub fn layout(aircraft: AircraftKind, realistic: bool) -> &'static Layout {
    match aircraft {
        AircraftKind::F117 if realistic => &F117_REAL_LAYOUT,
        AircraftKind::F117 => &F117_FANTASY_LAYOUT,
        AircraftKind::F16 => &F16_LAYOUT,
        AircraftKind::Tornado => &TORNADO_LAYOUT,
        AircraftKind::A10 => &A10_LAYOUT,
        AircraftKind::AV8B => &AV8B_LAYOUT,
        AircraftKind::F22 => &F22_LAYOUT,
        AircraftKind::UH60 => &UH60_LAYOUT,
        AircraftKind::AH1Z => &AH1Z_LAYOUT,
        AircraftKind::RAH66 => &RAH66_LAYOUT,
        AircraftKind::OH58 => &OH58_LAYOUT,
        AircraftKind::CH47 => &CH47_LAYOUT,
        AircraftKind::C5 => &C5_LAYOUT,
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
    /// The generator's stores: in the F-117A's bays as they are, on the F-16, the Tornado, the
    /// A-10 and the AV-8B each on the first free pair of stations that carries it, and Sidewinders on the free
    /// wingtips or rails; the AH-1Z full, the RAH-66 with Hellfires in both bays, the OH-58D
    /// with Hellfires on the left and a gun pod on the right.
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
            // Sidewinders in the side bays, the first two briefed stores in the main bay.
            AircraftKind::F22 => {
                stores[0] = Some(0);
                stores[3] = Some(0);
                let main: Vec<u8> = briefed.filter(|w| F22_MAIN.contains(w)).take(2).collect();
                for (i, &w) in [1usize, 2].into_iter().zip(main.iter().cycle()) {
                    stores[i] = Some(w);
                }
            }
            AircraftKind::UH60 | AircraftKind::CH47 => {}
            // The pallets for an airdrop, else an empty hold (the cargo it lands is the
            // mission's, not a store).
            AircraftKind::C5 => {
                stores[0] = briefed.into_iter().any(|w| w == 18).then_some(18);
            }
            AircraftKind::OH58 => {
                stores[0] = Some(HELLFIRE);
                stores[1] = Some(GUN_POD);
            }
            // Whatever the briefing: the Sidewinders on the rails, a full launcher of Hellfires
            // on every pylon.
            AircraftKind::AH1Z | AircraftKind::RAH66 => {
                for (s, h) in stores.iter_mut().zip(lay.hardpoints) {
                    *s = h.loads.first().copied();
                }
            }
            AircraftKind::F16 | AircraftKind::Tornado | AircraftKind::A10 | AircraftKind::AV8B => {
                let tornado = aircraft == AircraftKind::Tornado;
                let a10 = aircraft == AircraftKind::A10;
                let harrier = aircraft == AircraftKind::AV8B;
                // The mission's camera or cargo first: its station must not be blocked.
                let mut briefed: Vec<u8> = briefed.collect();
                briefed.sort_by_key(|w| !matches!(w, 16 | 18));
                for w in briefed.into_iter().chain([0]) {
                    // Stations preferred for each kind of store (pairs by their left station).
                    let order: &[usize] = match w {
                        0 if harrier => &[0, 1],
                        16 | 18 if harrier => &[3],
                        17 if harrier => &[3, 2],
                        4 if harrier => &[3],
                        5 if harrier => &[1, 2],
                        _ if harrier => &[2, 3, 1],
                        0 if a10 => &[0],
                        16 | 18 if a10 => &[5],
                        17 if a10 => &[3],
                        5 if a10 => &[2, 1],
                        _ if a10 => &[1, 2, 3, 4],
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
                    let free = |i: usize| {
                        let h = &lay.hardpoints[i];
                        stores[i].is_none() && h.excludes.iter().all(|&j| stores[j].is_none())
                    };
                    if let Some(&i) = order.iter().find(|&&i| {
                        free(i)
                            && lay.hardpoints[i].mirror.is_none_or(free)
                            && lay.hardpoints[i].loads.contains(&w)
                    }) {
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

    /// Puts `store` on station `i` (and its mirror too with `mirror`), emptying the stations
    /// whose stores would hit it.
    pub fn load(&mut self, i: usize, store: Option<u8>, mirror: bool) {
        let lay = self.layout();
        let mut put = |i: usize| {
            self.stores[i] = store;
            if store.is_some() {
                for &j in lay.hardpoints[i].excludes {
                    self.stores[j] = None;
                }
            }
        };
        put(i);
        if mirror && let Some(j) = lay.hardpoints[i].mirror {
            put(j);
        }
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
            return t!("loadout-none").into();
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
    let window = screen_window(ctx, "Loadout", LOADOUT_SIZE);
    let panel =
        crate::menu::Panel::new(t!("menu-loadout")).note(aircraft.name().to_ascii_uppercase());
    panel.window(window, ctx, |ui| {
        let split = Split::new(ui, "loadout");
        split.body_sized(ui, |ui, room| editor(ui, w, lo, &m, room, &|_| None));
        split.footer(ui, |ui| {
            crate::menu::footer_rule(ui);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if facts(
                    ui.add(big_button(t!("loadout-briefed")).icon(icon::BRIEFED)),
                    t_lines!("loadout-briefed.facts"),
                )
                .clicked()
                {
                    *lo = PlannedLoadout::of(&m, aircraft);
                }
                if facts(
                    ui.add(big_button(t!("loadout-clear")).icon(icon::CLEAR)),
                    t_lines!("loadout-clear.facts"),
                )
                .clicked()
                {
                    lo.stores.iter_mut().for_each(|s| *s = None);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(big_button(t!("loadout-done")).icon(icon::DONE).primary()).clicked() {
                        w.open = false;
                    }
                });
            });
        });
    });
}

/// The editor: the aircraft's hardpoints and the stores of the selected one (the menu's
/// loadout window, the campaign's briefing). `height`: the space it may take; `stock`: how
/// many of a weapon type the base has (None: it never runs out).
pub fn editor(
    ui: &mut egui::Ui,
    w: &mut LoadoutWindow,
    lo: &mut PlannedLoadout,
    m: &Mission,
    height: f32,
    stock: &dyn Fn(u8) -> Option<u32>,
) {
    let lay = lo.layout();
    w.selected = w.selected.min(lay.hardpoints.len().saturating_sub(1));
    let mirror = w.mirror.get_or_insert(true);
    let size = egui::vec2(ui.available_width(), height);
    ui.horizontal_top(|ui| {
        // Half the width for the stores (more of a narrow window), the rest for the diagram.
        let wide = ui.available_width();
        let side = (wide * 0.5).min(380.0).max((wide * 0.6).min(280.0));
        let draw = egui::vec2((wide - side - 12.0).max(0.0), height.max(0.0));
        if let Some(i) = diagram(ui, lay, &lo.stores, w.selected, draw) {
            w.selected = i;
        }
        ui.vertical(|ui| {
            ui.set_width(side);
            let Some(hp) = lay.hardpoints.get(w.selected) else {
                section(ui, t!("loadout-no-hardpoints"));
                ui.label(egui::RichText::new(t!("loadout-no-stores")).color(DIM));
                return;
            };
            section(ui, t!(hp.name));
            if hp.fantasy {
                ui.label(egui::RichText::new(t!("loadout-fantasy")).small().color(AMBER));
            }
            let mut pick = lo.stores[w.selected];
            // Rounds of each weapon the other stations take.
            let stations = lo.stations();
            let elsewhere = |wpn: u8| -> u32 {
                let other = stations.iter().enumerate().filter(|(i, _)| *i != w.selected);
                other.filter(|(_, s)| s.0 == wpn as i32).map(|(_, s)| s.1.max(0) as u32).sum()
            };
            // The column's full width: the scroll bar on its right edge, clear of the names.
            let list = egui::ScrollArea::vertical().max_height((size.y - 200.0).max(80.0));
            list.auto_shrink([false, true]).show(ui, |ui| {
                ui.radio_value(&mut pick, None, t!("loadout-empty"));
                for &wpn in hp.loads {
                    let (name, role) = WEAPONS[wpn as usize];
                    let left = stock(wpn);
                    let base_has = left.is_none_or(|n| n >= hp.rounds(wpn) as u32 + elsewhere(wpn));
                    let in_stock = !out_of_stock(m, wpn) && (base_has || pick == Some(wpn));
                    let r = ui.add_enabled(
                        in_stock,
                        egui::RadioButton::new(
                            pick == Some(wpn),
                            if let Some(n) = left.filter(|_| in_stock) {
                                format!(
                                    "{}x {name} {}",
                                    hp.rounds(wpn),
                                    t!("loadout-at-base", count = n)
                                )
                            } else if in_stock {
                                format!("{}x {name}", hp.rounds(wpn))
                            } else {
                                // Said in the name too: VR has no hover text.
                                format!(
                                    "{}x {name} {}",
                                    hp.rounds(wpn),
                                    t!("loadout-none-in-stock")
                                )
                            },
                        ),
                    );
                    let why =
                        if base_has { t!("loadout-out-of-stock") } else { t!("loadout-base-out") };
                    let r = r.on_disabled_hover_text(why);
                    let mut lines = vec![t!(role)];
                    lines.extend(employment(wpn as usize));
                    if facts(r, &lines).clicked() {
                        pick = Some(wpn);
                    }
                }
            });
            if let Some(j) = hp.mirror {
                ui.add_space(4.0);
                let other = t!(lay.hardpoints[j].name).to_lowercase();
                facts(
                    ui.checkbox(mirror, t!("loadout-mirror")),
                    &[&t!("loadout-mirror.facts", station = other)],
                );
            }
            if !hp.excludes.is_empty() {
                let names: Vec<_> = hp.excludes.iter().map(|&j| lay.hardpoints[j].label).collect();
                let text = match names[..] {
                    [a, b] => t!("loadout-empties-two", first = a, second = b),
                    _ => t!("loadout-empties", stations = names.join(", ")),
                };
                ui.label(egui::RichText::new(text).small().color(AMBER));
            }
            if pick != lo.stores[w.selected] {
                lo.load(w.selected, pick, *mirror);
            }
            section(ui, t!("loadout-stores"));
            ui.label(egui::RichText::new(lo.summary()).color(GREEN));
            if m.loadout.bays.iter().any(|b| b.weapon == 16 || b.weapon == 18) {
                ui.label(egui::RichText::new(t!("loadout-needs-equipment")).small().color(AMBER));
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
        for w in [0, 1, 2, 3, 4, 5, STINGER as usize] {
            let lock = sim.missile(sim.weapons[w].spec as i32).lock_range as f64;
            // 7/8 of it against the ground, less a little for the height.
            let km =
                if AIR_TO_AIR.contains(&(w as u8)) { lock } else { (lock * 0.875 - 0.5).ceil() };
            let range = format!("-{km} km");
            assert!(employment(w)[0].contains(&range), "{}: {range}", WEAPONS[w].0);
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

    /// The F-22A: Sidewinders in the side bays, the briefed stores in the main bay's halves.
    #[test]
    fn the_f22_flies_sidewinders_and_the_briefed_stores() {
        for seed in 0..40 {
            let m = mission(seed, false);
            let lo = PlannedLoadout::of(&m, AircraftKind::F22);
            assert!(lo.fits(&m, AircraftKind::F22));
            let st = lo.stations();
            assert_eq!((st[0], st[3]), ((0, 1), (0, 1)), "seed {seed}");
            let main = m.loadout.bays[0].weapon as u8;
            if F22_MAIN.contains(&main) {
                assert_eq!(lo.stores[1], Some(main), "seed {seed}");
            }
            assert!(lo.stores[1].is_some() && lo.stores[2].is_some(), "seed {seed}");
        }
        // Every store fits in each half of the main bay, one type at a time.
        for w in F22_MAIN {
            assert!(F22_STATIONS[1].rounds(w) >= 1 && F22_STATIONS[2].rounds(w) >= 1, "{w}");
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
                editor(ui, &mut w, &mut lo, &m, 600.0, &|_| None);
            });
            out.textures_delta.clear();
        }
    }
}

//! What the mission generator needs from the simulation's units (`assets/sim/units.ron`): the
//! aircraft's game speed and range, the radars' ids and the player's stores. The formulas are
//! documented at the top of that file; the simulation builds its own tables from it too.

use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

/// The target classes of a store's `effect`, in the order of a model's `shape_class & 0xF`.
pub const TARGET_CLASSES: [&str; 16] = [
    "shelter",
    "runway",
    "aircraft",
    "building",
    "bridge",
    "bunker",
    "depot",
    "launcher",
    "radar",
    "rig",
    "industry",
    "pen",
    "small_ship",
    "large_ship",
    "armour",
    "unused",
];

/// Game speed of an aircraft: 280 + 200 x Mach, or from km/h for helicopters (0.68 x) and
/// drones (0.6 x).
pub fn game_speed(mach: Option<f64>, kmh: Option<f64>, helicopter: bool) -> Option<i16> {
    match (mach, kmh) {
        (Some(m), _) => Some(round_to(280.0 + 200.0 * m, 5.0)),
        (None, Some(k)) if helicopter => Some(round_to(0.68 * k, 5.0)),
        (None, Some(k)) => Some(round_to(0.6 * k, 5.0)),
        (None, None) => None,
    }
}

/// Game range of an aircraft: 0.75 x its combat radius (km).
pub fn game_range(radius_km: f64) -> i16 {
    round_to(0.75 * radius_km, 10.0)
}

pub(crate) fn round_to(v: f64, step: f64) -> i16 {
    ((v / step).round() * step) as i16
}

#[derive(Deserialize)]
struct Radar {
    id: String,
}

#[derive(Deserialize, Default, PartialEq)]
enum Kind {
    #[default]
    Plane,
    Helicopter,
    Drone,
}

#[derive(Deserialize)]
struct Aircraft {
    id: String,
    name: String,
    nato: String,
    mach: Option<f64>,
    kmh: Option<f64>,
    radius_km: f64,
    #[serde(default)]
    kind: Kind,
}

#[derive(Deserialize)]
struct Weapon {
    id: String,
    short: String,
    long: String,
    #[serde(default)]
    bay: u8,
    #[serde(default)]
    effect: BTreeMap<String, i8>,
}

#[derive(Deserialize)]
struct Units {
    radars: Vec<Radar>,
    aircraft: Vec<Aircraft>,
    weapons: Vec<Weapon>,
}

/// An aircraft type as the generator uses it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlaneInfo {
    pub id: String,
    pub name: String,
    pub nick: String,
    pub speed: i16,
    pub range: i16,
}

/// A store the player can carry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WeaponInfo {
    pub id: String,
    /// As the briefing lists it, e.g. "2X GBU-12 PAVEWAY".
    pub name: String,
    /// Rounds a bay holds.
    pub bay: u8,
    /// How well it destroys each target class (0 cannot .. 8).
    pub effect: [i8; 16],
}

/// The units the generator reads.
pub(crate) struct UnitTables {
    pub radars: Vec<String>,
    pub planes: Vec<PlaneInfo>,
    pub weapons: Vec<WeaponInfo>,
}

pub(crate) fn read(text: &str) -> Result<UnitTables> {
    let u: Units = ron::from_str(text).context("units.ron")?;
    let planes = u
        .aircraft
        .iter()
        .map(|a| {
            let speed = game_speed(a.mach, a.kmh, a.kind == Kind::Helicopter)
                .ok_or_else(|| anyhow!("aircraft `{}` without a speed", a.id))?;
            Ok(PlaneInfo {
                id: a.id.clone(),
                name: a.name.clone(),
                nick: a.nato.clone(),
                speed,
                range: game_range(a.radius_km),
            })
        })
        .collect::<Result<_>>()?;
    let weapons =
        u.weapons
            .iter()
            .map(|w| {
                let mut effect = [0i8; 16];
                for (class, &v) in &w.effect {
                    let k = TARGET_CLASSES.iter().position(|c| c == class).ok_or_else(|| {
                        anyhow!("weapon `{}`: unknown target class `{class}`", w.id)
                    })?;
                    effect[k] = v;
                }
                let name = format!("{}X {} {}", w.bay.max(1), w.short, w.long).to_uppercase();
                Ok(WeaponInfo { id: w.id.clone(), name, bay: w.bay, effect })
            })
            .collect::<Result<_>>()?;
    Ok(UnitTables { radars: u.radars.into_iter().map(|r| r.id).collect(), planes, weapons })
}

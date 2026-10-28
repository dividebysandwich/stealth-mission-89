//! The simulation's units (`assets/sim/units.ron`): radars, aircraft, missiles and stores
//! described by real-world figures, turned into the simulation's game-scaled tables. The
//! formulas are documented at the top of the file.

use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use stealth_formats::THEATERS;

use stealth_mission::units::{game_range, game_speed};

use crate::data::{AircraftKind, AircraftType, MissileType, ThreatType, WeaponType};

#[derive(Deserialize)]
struct Radar {
    id: String,
    name: String,
    search_km: f64,
    year: i32,
    #[serde(default)]
    doppler: bool,
    #[serde(default)]
    airborne: bool,
    missile: Option<String>,
}

#[derive(Deserialize)]
struct Aircraft {
    id: String,
    name: String,
    nato: String,
    mach: Option<f64>,
    kmh: Option<f64>,
    radius_km: f64,
    agility: i16,
    radar: String,
    model: i16,
    #[serde(default)]
    kind: AircraftKind,
}

#[derive(Deserialize, Clone, Copy, PartialEq)]
enum Family {
    Sam,
    Air,
    Ground,
    Special,
}

/// Seeker of a missile: the simulation's class number.
#[derive(Deserialize, Clone, Copy)]
enum Seeker {
    RearIr,
    Ir,
    Command,
    SemiActive,
    TrackViaMissile,
    AntiRadiation,
    AntiShip,
    Optical,
    PlayerAam,
    GuidedBomb,
    RetardedBomb,
    FreeFallBomb,
    Dropped,
}

impl Seeker {
    fn class(self) -> i16 {
        match self {
            Seeker::RearIr => -1,
            Seeker::Ir => 0,
            Seeker::Command => 1,
            Seeker::SemiActive => 2,
            Seeker::TrackViaMissile => 3,
            Seeker::AntiRadiation => 4,
            Seeker::AntiShip => 5,
            Seeker::Optical => 6,
            Seeker::PlayerAam => 7,
            Seeker::GuidedBomb => 28,
            Seeker::RetardedBomb | Seeker::Dropped => 29,
            Seeker::FreeFallBomb => 30,
        }
    }
}

#[derive(Deserialize)]
struct Missile {
    id: String,
    name: String,
    family: Family,
    range_km: Option<f64>,
    mach: Option<f64>,
    g: Option<f64>,
    lock: Option<i16>,
    speed: Option<i16>,
    turn: Option<i16>,
    seeker: Seeker,
    model: i16,
}

#[derive(Deserialize)]
struct Weapon {
    id: String,
    short: String,
    long: String,
    missile: String,
    picture: i16,
    #[serde(default)]
    effect: BTreeMap<String, i8>,
}

#[derive(Deserialize)]
struct Units {
    radars: Vec<Radar>,
    aircraft: Vec<Aircraft>,
    missiles: Vec<Missile>,
    weapons: Vec<Weapon>,
    flares: i16,
    chaff: i16,
    decoys: i16,
    gun_rounds: i16,
    runway_north: BTreeMap<String, bool>,
}

pub use stealth_mission::units::TARGET_CLASSES;

/// The game-scaled tables.
pub struct Tables {
    pub threats: Vec<ThreatType>,
    pub aircraft: Vec<AircraftType>,
    pub missiles: Vec<MissileType>,
    pub weapons: Vec<WeaponType>,
    pub compat: Vec<[i8; 16]>,
    pub maneuver: Vec<i16>,
    pub stores_init: [i16; 4],
    pub gun_ammo_init: i16,
    pub start_dir: [i16; 9],
}

fn round_to(v: f64, step: f64) -> i16 {
    ((v / step).round() * step) as i16
}

fn index_of<'a>(ids: impl Iterator<Item = &'a String>, id: &str, what: &str) -> Result<i16> {
    ids.into_iter()
        .position(|i| i == id)
        .map(|i| i as i16)
        .ok_or_else(|| anyhow!("unknown {what} `{id}`"))
}

/// Reads `units.ron` and builds the simulation's tables.
pub fn tables(text: &str) -> Result<Tables> {
    let u: Units = ron::from_str(text).context("units.ron")?;
    let missile_ids = || u.missiles.iter().map(|m| &m.id);
    let radar_ids = || u.radars.iter().map(|r| &r.id);

    let mut threats = Vec::new();
    for r in &u.radars {
        let missile =
            r.missile.as_deref().map(|m| index_of(missile_ids(), m, "missile")).transpose()?;
        threats.push(ThreatType {
            name: r.name.clone(),
            lethality: if r.search_km <= 0.0 {
                0
            } else {
                round_to(60.0 + 0.7 * r.search_km, 10.0).clamp(50, 500)
            },
            tier: if r.search_km <= 0.0 {
                0
            } else {
                (((r.year - 1945) as f64 / 6.5).round() as i16).clamp(2, 8)
            },
            flags: r.doppler as i16 | (r.airborne as i16) << 1,
            missile,
        });
    }

    let mut aircraft = Vec::new();
    for a in &u.aircraft {
        let radar = match a.radar.as_str() {
            "none" => 0,
            "friendly" => -1,
            id => index_of(radar_ids(), id, "radar")?,
        };
        let max_speed = game_speed(a.mach, a.kmh, a.kind == AircraftKind::Helicopter)
            .ok_or_else(|| anyhow!("aircraft `{}` without a speed", a.id))?;
        aircraft.push(AircraftType {
            name: a.name.clone(),
            nato: a.nato.clone(),
            max_speed,
            range: game_range(a.radius_km),
            agility: a.agility,
            radar,
            model: a.model,
            unknown: 2,
            kills: 0,
            kind: a.kind,
        });
    }

    let mut missiles = Vec::new();
    for m in &u.missiles {
        let (lock, speed, turn) = if m.family == Family::Special {
            (m.lock.unwrap_or(0), m.speed.unwrap_or(0), m.turn.unwrap_or(1))
        } else {
            let range = m.range_km.ok_or_else(|| anyhow!("missile `{}` without a range", m.id))?;
            let lock = match m.family {
                Family::Sam => (2.75 * range).min(150.0),
                Family::Air => 4.0 + 0.45 * range,
                _ => 4.0 + 0.35 * range,
            };
            let mach = m.mach.unwrap_or(0.0);
            let g = m.g.unwrap_or(5.0);
            (
                lock.round() as i16,
                round_to(400.0 + 450.0 * mach, 50.0).min(2000),
                ((1.2 * g.sqrt()).round() as i16).clamp(1, 8),
            )
        };
        missiles.push(MissileType {
            name: m.name.clone(),
            lock_range: lock,
            max_speed: speed,
            class: m.seeker.class(),
            turn,
            model: m.model,
        });
    }

    let mut weapons = Vec::new();
    let mut compat = Vec::new();
    for w in &u.weapons {
        let spec = match w.missile.as_str() {
            "camera" => -1,
            "fuel" => -2,
            "guns" => 0,
            id => index_of(missile_ids(), id, "missile")?,
        };
        weapons.push(WeaponType {
            short_name: w.short.clone(),
            long_name: w.long.clone(),
            spec,
            picture: w.picture,
        });
        let mut row = [0i8; 16];
        for (class, v) in &w.effect {
            let k = TARGET_CLASSES
                .iter()
                .position(|c| c == class)
                .ok_or_else(|| anyhow!("weapon `{}`: unknown target class `{class}`", w.id))?;
            row[k] = *v;
        }
        compat.push(row);
    }

    let mut start_dir = [1i16; 9];
    for (k, t) in THEATERS.iter().enumerate() {
        let north =
            *u.runway_north.get(*t).ok_or_else(|| anyhow!("no runway direction for {t}"))?;
        start_dir[k] = if north { 1 } else { -1 };
    }

    Ok(Tables {
        threats,
        aircraft,
        missiles,
        weapons,
        compat,
        maneuver: maneuver_table(),
        stores_init: [0, u.flares, u.chaff, u.decoys],
        gun_ammo_init: u.gun_rounds,
        start_dir,
    })
}

/// The AI's dogfight roll command for skill 0..2, the target's aspect (0 head-on .. 4 the
/// target behind and pointing the same way) and the target's bearing sector off the nose
/// (0..3 to the right, 4..7 to the left, 45 degrees each): the roll in quarters of a full
/// command (+ right), or 256 for a climbing pull-up, 512 for a split-S.
pub fn maneuver(skill: i32, aspect: i32, sector: i32) -> i16 {
    let side = if sector < 4 { 1 } else { -1 };
    // 0: ahead .. 3: behind, on that side.
    let off = if sector < 4 { sector } else { 7 - sector };
    let max = [3, 4, 4][skill.clamp(0, 2) as usize];
    // The target sits behind and points at us: break (the best pilots go vertical).
    let threatened = (3..=5).contains(&aspect) && off >= 2;
    if threatened {
        return match skill {
            0 => (-side * 3) as i16,
            1 => (-side * 4) as i16,
            _ if (aspect + sector) % 3 == 0 => 512,
            _ if off == 3 => 256,
            _ => (-side * 4) as i16,
        };
    }
    // Head-on: lead the turn gently; otherwise turn after the target harder the further off
    // the nose it is.
    let head_on = matches!(aspect, 0 | 1 | 7);
    let roll = if head_on { (off + 1).min(max - 1) } else { (1 + off).min(max) };
    (side * roll.max(1)) as i16
}

fn maneuver_table() -> Vec<i16> {
    let mut t = Vec::with_capacity(192);
    for skill in 0..3 {
        for aspect in 0..8 {
            for sector in 0..8 {
                t.push(maneuver(skill, aspect, sector));
            }
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shipped() -> Tables {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/sim/units.ron"
        ))
        .unwrap();
        tables(&text).unwrap()
    }

    /// The shipped units resolve, keep the indices the theaters and the code use, and come out
    /// in the simulation's ranges.
    #[test]
    fn units_build_the_tables() {
        let t = shipped();
        assert_eq!(t.threats.len(), 27);
        assert_eq!(t.aircraft.len(), 33);
        assert_eq!(t.missiles.len(), 49);
        assert_eq!((t.weapons.len(), t.compat.len()), (22, 22));
        assert!(
            t.threats
                .iter()
                .skip(1)
                .all(|r| (50..=500).contains(&r.lethality) && (2..=8).contains(&r.tier))
        );
        assert!(t.aircraft.iter().all(|a| (100..=900).contains(&a.max_speed)));
        assert!(
            t.missiles
                .iter()
                .all(|m| (0..=2600).contains(&m.max_speed) && (0..=8).contains(&m.turn))
        );
        assert_eq!(t.weapons[19].spec, 0, "guns");
        assert_eq!(t.weapons[16].spec, -1, "camera");
        assert_eq!(t.aircraft[22].name, "F-16");
        assert_eq!(t.missiles[t.weapons[1].spec as usize].name, "AIM-120");
    }

    /// The dogfight rules turn after a target ahead, break from one behind.
    #[test]
    fn maneuvers_follow_the_target() {
        assert!(maneuver(1, 4, 1) > 0, "target ahead right: turn right");
        assert!(maneuver(1, 4, 6) < 0, "target ahead left: turn left");
        assert!(maneuver(0, 4, 3) < 0, "attacker behind right: break left");
        assert!(matches!(maneuver(2, 4, 3), 256 | 512 | -4));
    }
}

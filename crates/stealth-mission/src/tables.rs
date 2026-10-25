//! The mission generator's rules (`assets/mission/missions.ron`) with the aircraft and stores
//! of `assets/sim/units.ron`, compiled for the generator.

use anyhow::{Context, Result, anyhow, ensure};
use serde::{Deserialize, Serialize};

pub use crate::units::{PlaneInfo, WeaponInfo};

/// What a theater model is as a target. A theater's world names the role of each of its
/// models; models without one are never targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Role {
    Airstrip,
    SamRadar,
    MobileSam,
    EarlyWarningRadar,
    Refinery,
    TankFarm,
    Hangar,
    Headquarters,
    Carrier,
    Frigate,
    PatrolBoat,
    SamMissile,
    OilRig,
    NuclearPlant,
    SubPen,
    CargoShip,
    MissileSub,
    SupplyDump,
    Bridge,
    /// An airbase's runways: the aircraft missions' target.
    Airbase,
    ParkedAircraft,
    Camp,
    Palace,
    ChemicalPlant,
    MobileScud,
    BioPlant,
    MissileBoat,
    PumpingStation,
    ScudLauncher,
    AirDefenceCommand,
    Armour,
    Mosque,
    FishingBoat,
    SupplyDepot,
    SsMissile,
}

impl Role {
    /// Placed on the coarse terrain level (found among the LOD3 tiles' objects).
    pub fn coarse(self) -> bool {
        self == Self::SubPen
    }

    /// Never the enemy's forward position a helicopter mission goes for: airfields and
    /// aircraft, warships and submarine pens, nuclear, chemical and biological plants,
    /// palaces, mosques.
    pub fn rear_area(self) -> bool {
        matches!(
            self,
            Self::Airstrip
                | Self::Hangar
                | Self::Carrier
                | Self::Frigate
                | Self::NuclearPlant
                | Self::SubPen
                | Self::MissileSub
                | Self::Airbase
                | Self::ParkedAircraft
                | Self::Palace
                | Self::ChemicalPlant
                | Self::BioPlant
                | Self::Mosque
        )
    }
}

/// What the player has to do at a target. The number is the objective code the flight uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Objective {
    Photo = 1,
    Destroy = 2,
    SupplyDrop = 3,
    Airstrip = 4,
    InterceptRoute = 5,
    InterceptLeaving = 6,
    InterceptHome = 7,
    Patrol = 8,
}

impl Objective {
    pub fn code(self) -> i32 {
        self as i32
    }

    /// Against an aircraft (flight unit 0) rather than a place.
    pub fn aircraft(self) -> bool {
        self.code() >= 5
    }
}

/// Rules of engagement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Roe {
    Covert = 0,
    Limited = 1,
    War = 2,
}

fn all_roe() -> Vec<Roe> {
    vec![Roe::Covert, Roe::Limited, Roe::War]
}

fn one() -> u8 {
    1
}

/// Target code flags (the low byte of a target's code word).
pub mod code {
    pub const DEADLINE: i32 = 0x02;
    pub const EXCLUSIVE: i32 = 0x08;
    pub const AREA: i32 = 0x10;
    pub const RUBBLE: i32 = 0x20;
}

/// A kind of mission job, as `missions.ron` describes it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub objective: Objective,
    pub target: Role,
    #[serde(default = "all_roe")]
    pub roe: Vec<Roe>,
    #[serde(default = "one")]
    pub weight: u8,
    #[serde(default)]
    pub deadline: bool,
    #[serde(default)]
    pub exclusive: bool,
    #[serde(default)]
    pub area: bool,
    #[serde(default)]
    pub rubble: bool,
    /// special_targets model standing in for the target, 0 none.
    #[serde(default)]
    pub model: u8,
    /// Aircraft id of the special aircraft.
    #[serde(default)]
    pub aircraft: Option<String>,
    #[serde(default)]
    pub fighters: bool,
    /// Intelligence summaries, one is picked per mission.
    pub briefings: Vec<String>,
    /// `aircraft` as an aircraft type, filled in by [`MissionTables::read`].
    #[serde(skip)]
    pub plane: Option<i16>,
}

impl Template {
    /// The target's code word: the flags, the special model in the high byte.
    pub fn code(&self) -> i32 {
        let flags = [
            (self.deadline, code::DEADLINE),
            (self.exclusive, code::EXCLUSIVE),
            (self.area, code::AREA),
            (self.rubble, code::RUBBLE),
        ];
        flags.iter().filter(|f| f.0).fold(0, |c, f| c | f.1) | (self.model as i32) << 8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Training {
    strike: String,
    air: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Rules {
    templates: Vec<Template>,
    training: Training,
    threat_ladders: Vec<Vec<String>>,
    shortages: Vec<(String, String)>,
}

fn find<'a>(mut ids: impl Iterator<Item = &'a String>, id: &str, what: &str) -> Result<usize> {
    ids.position(|i| i == id).ok_or_else(|| anyhow!("unknown {what} `{id}`"))
}

/// The generator's tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTables {
    pub templates: Vec<Template>,
    /// Templates of a training mission: (strike, air).
    pub training: (usize, usize),
    /// Per radar type: (upgrade, downgrade).
    pub threat_steps: Vec<(i16, i16)>,
    /// Stores that can run short, each with its stand-in (weapon types).
    pub shortages: Vec<(u8, u8)>,
    pub planes: Vec<PlaneInfo>,
    pub weapons: Vec<WeaponInfo>,
}

impl MissionTables {
    /// Compiles `missions.ron` against `units.ron`.
    pub fn read(missions: &str, units: &str) -> Result<Self> {
        let rules: Rules = ron::from_str(missions).context("missions.ron")?;
        let u = crate::units::read(units)?;
        let mut templates = rules.templates;
        for t in &mut templates {
            ensure!(!t.briefings.is_empty(), "template `{}` without briefings", t.id);
            if let Some(a) = &t.aircraft {
                t.plane = Some(find(u.planes.iter().map(|p| &p.id), a, "aircraft")? as i16);
            }
        }
        let template = |id: &str| find(templates.iter().map(|t| &t.id), id, "template");
        let training = (template(&rules.training.strike)?, template(&rules.training.air)?);
        let radar = |id: &str| find(u.radars.iter(), id, "radar").map(|i| i as i16);
        let mut threat_steps: Vec<(i16, i16)> =
            (0..u.radars.len() as i16).map(|i| (i, i)).collect();
        for ladder in &rules.threat_ladders {
            let rungs = ladder.iter().map(|id| radar(id)).collect::<Result<Vec<_>>>()?;
            for (k, &r) in rungs.iter().enumerate() {
                let up = rungs.get(k + 1).copied().unwrap_or(r);
                let down = if k > 0 { rungs[k - 1] } else { r };
                threat_steps[r as usize] = (up, down);
            }
        }
        let weapon = |id: &str| find(u.weapons.iter().map(|w| &w.id), id, "weapon");
        let shortages = rules
            .shortages
            .iter()
            .map(|(w, s)| Ok((weapon(w)? as u8, weapon(s)? as u8)))
            .collect::<Result<_>>()?;
        Ok(Self {
            templates,
            training,
            threat_steps,
            shortages,
            planes: u.planes,
            weapons: u.weapons,
        })
    }

    /// Aircraft type `t`, or an empty entry for an unknown type.
    pub fn plane(&self, t: i16) -> PlaneInfo {
        self.planes.get(t as usize).cloned().unwrap_or(PlaneInfo {
            id: String::new(),
            name: String::new(),
            nick: String::new(),
            speed: 0,
            range: 0,
        })
    }

    pub fn weapon_name(&self, w: i16) -> &str {
        self.weapons.get(w as usize).map_or("", |w| w.name.as_str())
    }

    /// Rounds of weapon `w` in a bay.
    pub fn weapon_qty(&self, w: i16) -> u8 {
        self.weapons.get(w as usize).map_or(0, |w| w.bay)
    }

    /// Effect of weapon `w` on target class `class`.
    pub fn effect(&self, w: usize, class: usize) -> i8 {
        self.weapons.get(w).map_or(0, |w| w.effect[class & 0xF])
    }
}

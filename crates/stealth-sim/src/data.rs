use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use stealth_formats::{THEATERS, terrain};

use crate::math::Trig;

/// Table sizes of the classic units (the newer ones follow them, `assets/sim/units.ron`).
pub const N_THREATS: usize = 23;
pub const N_AIRCRAFT: usize = 29;
pub const N_MISSILES: usize = 41;
pub const N_WEAPONS: usize = 20;

/// Radar type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreatType {
    pub name: String,
    /// Maximum detection range in km (R = map >> 6).
    pub lethality: i16,
    pub tier: i16,
    /// bit 0 Doppler, bit 1 omni-directional airborne (AWACS).
    pub flags: i16,
    /// Missile a site of this type fires (the remake's own sites, appended after the classic
    /// table); None: the missile with the same index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missile: Option<i16>,
}

/// Aircraft type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AircraftType {
    pub name: String,
    pub nato: String,
    pub max_speed: i16,
    pub range: i16,
    pub agility: i16,
    /// Radar type index; 0 = none, -1 = friendly.
    pub radar: i16,
    /// Aircraft model index (aircraft.glb).
    pub model: i16,
    pub unknown: i16,
    /// Initial kill counter.
    pub kills: i16,
    /// How the AI flies it (absent from the classic data: all fixed-wing).
    #[serde(default, skip_serializing_if = "AircraftKind::is_plane")]
    pub kind: AircraftKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AircraftKind {
    #[default]
    Plane,
    /// Hovers, takes off vertically, flies low.
    Helicopter,
    /// One-way attack drone: flies its route to the target and never fights.
    Drone,
}

impl AircraftKind {
    pub fn is_plane(&self) -> bool {
        *self == Self::Plane
    }
}

/// Missile / SAM / player weapon projectile type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissileType {
    pub name: String,
    /// km.
    pub lock_range: i16,
    /// speed = max_speed >> 6.
    pub max_speed: i16,
    /// Seeker class: <= 0 infrared, > 0 radar; 4..=6 guided air-to-ground, 7 air-to-air.
    pub class: i16,
    pub turn: i16,
    /// Aircraft model index (aircraft.glb).
    pub model: i16,
}

/// Player store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponType {
    pub short_name: String,
    pub long_name: String,
    /// Index into the missile table; -1 camera, -2 extra fuel, 0 guns.
    pub spec: i16,
    pub picture: i16,
}

/// Per-theater tile-object data used by the nearest-tile-object search and the wreck shape test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterTiles {
    pub grid: terrain::Grid,
    /// `objects[level-1][tile type]` = (x, y, shape) for LOD levels 1 and 2.
    pub objects: [Vec<Vec<(i16, i16, u8)>>; 2],
    /// Theater model attribute byte per shape (bit 0x20: the model has its own wreck state).
    pub model_attr: Vec<u8>,
    /// Remake theaters: the real towns (their defenders stand round the edge).
    #[serde(default)]
    pub towns: Vec<Town>,
    /// The mountains (`assets/theaters/<code>.relief`, read beside the tiles).
    #[serde(skip)]
    pub relief: crate::relief::Relief,
    /// The airfields' helipads (`assets/theaters/<code>.helipads`, read beside the tiles).
    #[serde(skip)]
    pub helipads: Vec<(i32, i32)>,
    /// The airfields' runways (`assets/theaters/<code>.runways`, read beside the tiles).
    #[serde(skip)]
    pub runways: Vec<Runway>,
}

/// An airfield's runway as the scenery lays it out (metres east, north): its middle, its
/// direction, half its length and width, and the side away from the apron (unit vectors).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Runway {
    pub c: (f64, f64),
    pub axis: (f64, f64),
    pub half_len: f64,
    pub half_width: f64,
    pub out: (f64, f64),
}

/// A town: centre (map units, y south) and radius (map units).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Town {
    pub x: i32,
    pub y: i32,
    pub r: i32,
}

impl Town {
    /// A town of a theater file (metres east, north; radius in metres).
    pub fn from_metres(x: f64, y: f64, r: f64) -> Self {
        let map = |m: f64| (m / crate::aaa::M_PER_MAP).round() as i32;
        Self { x: map(x), y: 0x8000 - map(y), r: map(r).max(1) }
    }
}

/// A town of a theater file (its other fields are the scenery's).
#[derive(Deserialize)]
struct TownSite {
    x: f64,
    y: f64,
    r: f64,
}

/// All static data the simulation needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimData {
    pub threats: Vec<ThreatType>,
    pub aircraft: Vec<AircraftType>,
    pub missiles: Vec<MissileType>,
    pub weapons: Vec<WeaponType>,
    /// Weapon-vs-category effectiveness `compat[w][cat]`, a row per weapon.
    pub compat: Vec<[i8; 16]>,
    /// Dogfight manoeuvre table, 3 x 8 x 8 int16.
    pub maneuver: Vec<i16>,
    pub trig: Trig,
    /// Countermeasure stores at start ([_, flare, chaff, decoy]).
    pub stores_init: [i16; 4],
    /// Gun rounds at start.
    pub gun_ammo_init: i16,
    /// Runway direction per theater: +1 start south heading north, -1 the reverse.
    pub start_dir: [i16; 9],
    /// One per classic theater, stored in `theaters/<code>.json` beside `units.ron`.
    #[serde(skip)]
    pub theaters: Vec<TheaterTiles>,
}

impl SimData {
    /// Read `units.ron` and `theaters/<code>.json` from `dir` (`assets/sim`), or the theater's
    /// tiles from `../theaters/<code>.ron` where there is no such file.
    pub fn load_json(dir: &Path) -> Result<Self> {
        Self::load_with(|name| {
            std::fs::read(dir.join(name)).with_context(|| format!("{}", dir.join(name).display()))
        })
    }

    /// [`SimData::load_json`] with the files' contents from `read` (a name such as `units.ron`).
    pub fn load_with(read: impl Fn(&str) -> Result<Vec<u8>>) -> Result<Self> {
        let text = String::from_utf8(read("units.ron")?).context("units.ron")?;
        let t = crate::units::tables(&text)?;
        let mut d = Self {
            threats: t.threats,
            aircraft: t.aircraft,
            missiles: t.missiles,
            weapons: t.weapons,
            compat: t.compat,
            maneuver: t.maneuver,
            trig: Trig::analytic(),
            stores_init: t.stores_init,
            gun_ammo_init: t.gun_ammo_init,
            start_dir: t.start_dir,
            theaters: vec![],
        };
        d.theaters = THEATERS
            .iter()
            .map(|code| {
                let mut t: TheaterTiles = match read(&format!("theaters/{code}.json")) {
                    Ok(json) => serde_json::from_slice(&json)
                        .with_context(|| format!("theaters/{code}.json"))?,
                    // A rebuilt theater: its tiles from the theater file.
                    Err(_) => TheaterTiles::read_ron(&read(&format!("../theaters/{code}.ron"))?)
                        .with_context(|| format!("theaters/{code}.ron"))?,
                };
                t.relief = TheaterTiles::read_relief(&read, code);
                t.helipads = TheaterTiles::read_helipads(&read, code);
                t.runways = TheaterTiles::read_runways(&read, code);
                Ok(t)
            })
            .collect::<Result<_>>()?;
        Ok(d)
    }

    /// Write `theaters/<code>.json` under `dir`.
    pub fn write_json(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir.join("theaters"))?;
        for (t, code) in self.theaters.iter().zip(THEATERS) {
            std::fs::write(
                dir.join(format!("theaters/{}.json", code.to_ascii_lowercase())),
                serde_json::to_string(t)?,
            )?;
        }
        Ok(())
    }

    pub fn threat(&self, i: i32) -> Option<&ThreatType> {
        usize::try_from(i).ok().and_then(|i| self.threats.get(i))
    }

    pub fn aircraft_type(&self, i: i32) -> &AircraftType {
        &self.aircraft[(i.max(0) as usize).min(self.aircraft.len() - 1)]
    }

    pub fn missile(&self, i: i32) -> &MissileType {
        &self.missiles[(i.max(0) as usize).min(self.missiles.len() - 1)]
    }

    pub fn weapon(&self, i: i32) -> &WeaponType {
        &self.weapons[(i.max(0) as usize).min(self.weapons.len() - 1)]
    }

    /// This data with tiles for theater `theater` if the exported data has none (remake
    /// theaters).
    pub fn with_theater(&self, theater: usize, tiles: impl FnOnce() -> TheaterTiles) -> Self {
        let mut d = self.clone();
        if d.theaters.get(theater).is_none() {
            let t = tiles();
            d.theaters.resize(theater, t.clone());
            d.theaters.push(t);
        }
        d
    }

    /// Weapon-vs-category effectiveness given the category directly.
    pub fn compat_cat(&self, w: i32, cat: i32) -> i32 {
        self.compat.get(w.max(0) as usize).map_or(0, |r| r[(cat & 0xF) as usize] as i32)
    }
}

/// The part of a theater file (`assets/theaters/<code>.ron`) the simulation uses.
#[derive(Deserialize)]
struct TheaterFile {
    grid: terrain::Grid,
    tiles: terrain::TileSets,
    #[serde(default)]
    model_attr: Vec<u8>,
    #[serde(default)]
    towns: Vec<TownSite>,
}

impl TheaterTiles {
    /// The tile data of a theater file.
    pub fn read_ron(text: &[u8]) -> Result<Self> {
        let t: TheaterFile = ron::from_str(std::str::from_utf8(text)?)?;
        let towns = t.towns.iter().map(|s| Town::from_metres(s.x, s.y, s.r)).collect();
        Ok(Self { towns, ..Self::from_tilesets(&t.grid, &t.tiles, &t.model_attr) })
    }

    /// A theater's tile data from its terrain (remake theaters, which the exported classic
    /// data does not have).
    pub fn from_tilesets(
        grid: &terrain::Grid,
        tiles: &terrain::TileSets,
        model_attr: &[u8],
    ) -> Self {
        let level = |l: usize| -> Vec<Vec<(i16, i16, u8)>> {
            tiles
                .tiles
                .get(l)
                .map(|v| {
                    v.iter().map(|t| t.iter().map(|o| (o.x, o.y, o.shape)).collect()).collect()
                })
                .unwrap_or_default()
        };
        Self {
            grid: grid.clone(),
            objects: [level(1), level(2)],
            model_attr: model_attr.to_vec(),
            towns: Vec::new(),
            relief: Default::default(),
            helipads: Vec::new(),
            runways: Vec::new(),
        }
    }

    /// A theater's relief (`theaters/<code>.relief` through `read`, from `assets/sim`): flat
    /// without one.
    pub fn read_relief(
        read: impl Fn(&str) -> Result<Vec<u8>>,
        code: &str,
    ) -> crate::relief::Relief {
        let name = format!("../theaters/{}.relief", code.to_ascii_lowercase());
        read(&name).ok().and_then(crate::relief::Relief::from_bytes).unwrap_or_default()
    }

    /// A theater's helipads (`theaters/<code>.helipads` through `read`, from `assets/sim`): a
    /// list of world unit points (x east, y south) made by the game (`--write-helipads`); none
    /// without one.
    pub fn read_helipads(read: impl Fn(&str) -> Result<Vec<u8>>, code: &str) -> Vec<(i32, i32)> {
        let name = format!("../theaters/{}.helipads", code.to_ascii_lowercase());
        read(&name)
            .ok()
            .and_then(|b| ron::from_str(std::str::from_utf8(&b).ok()?).ok())
            .unwrap_or_default()
    }

    /// A theater's runways (`theaters/<code>.runways` through `read`, from `assets/sim`): a list
    /// made by the game (`--write-helipads`); none without one.
    pub fn read_runways(read: impl Fn(&str) -> Result<Vec<u8>>, code: &str) -> Vec<Runway> {
        let name = format!("../theaters/{}.runways", code.to_ascii_lowercase());
        read(&name)
            .ok()
            .and_then(|b| ron::from_str(std::str::from_utf8(&b).ok()?).ok())
            .unwrap_or_default()
    }

    /// LOD1/LOD2 tile type at (col, row) (row counted north-up), -1 outside the grid.
    pub fn cell(&self, level: usize, col: i32, row: i32) -> i32 {
        let dim = terrain::GRID_DIM[level];
        if col < 0 || row < 0 || col >= dim || row >= dim {
            return -1;
        }
        self.grid.tile(level, col, row) as i32
    }

    pub fn objects_of(&self, level: usize, cell: i32) -> &[(i16, i16, u8)] {
        if cell < 0 {
            return &[];
        }
        self.objects.get(level - 1).and_then(|l| l.get(cell as usize)).map_or(&[], Vec::as_slice)
    }
}

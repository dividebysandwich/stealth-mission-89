//! A theater's relief: the highest ground (mountains, hills) in each 2 km cell. The simulation's
//! world is otherwise a flat plane with objects on it; the AI's aircraft keep clear of the
//! mountains with it. Made from the theater models by the game (`--write-relief`) and kept in
//! `assets/theaters/<code>.relief`: `DIM` x `DIM` bytes, row 0 at the north edge, the height
//! in `STEP_M` metre steps (rounded up).

use std::sync::Arc;

/// Cells per side.
pub const DIM: usize = 256;
/// Metres per height step.
pub const STEP_M: f64 = 32.0;
/// Map units (y south) per cell side: the theater is 0x8000 map units across.
const CELL_MAP: i32 = 0x8000 / DIM as i32;
/// Metres per altitude unit.
const WU_M: f64 = 512_000.0 / 1_048_576.0;

#[derive(Clone, Default)]
pub struct Relief(Arc<Vec<u8>>);

impl std::fmt::Debug for Relief {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Relief({} cells)", self.0.len())
    }
}

impl Relief {
    /// A relief file's contents (None if it is not one).
    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        (bytes.len() == DIM * DIM).then(|| Self(Arc::new(bytes)))
    }

    /// Heights in metres per cell (row 0 north), rounded up to the steps.
    pub fn from_metres(heights: &[f32]) -> Self {
        let step = |h: &f32| (h.max(0.0) as f64 / STEP_M).ceil().min(255.0) as u8;
        Self(Arc::new(heights.iter().map(step).collect()))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// The highest ground in the cell of map point (`x`, `y` south), in altitude units (0 for
    /// the flat theaters or outside the theater).
    pub fn height(&self, x: i32, y: i32) -> i32 {
        if self.0.is_empty() || !(0..0x8000).contains(&x) || !(0..0x8000).contains(&y) {
            return 0;
        }
        let (col, row) = ((x / CELL_MAP) as usize, (y / CELL_MAP) as usize);
        (self.0[row * DIM + col] as f64 * STEP_M / WU_M) as i32
    }

    /// The ground's height in metres at map point (`x`, `y` south), blended between the cells'
    /// centres (for the maps; `height` is the cell's highest).
    pub fn smooth_m(&self, x: f64, y: f64) -> f32 {
        if self.0.is_empty() {
            return 0.0;
        }
        let cell = |c: i64, r: i64| {
            let (c, r) = (c.clamp(0, DIM as i64 - 1), r.clamp(0, DIM as i64 - 1));
            self.0[r as usize * DIM + c as usize] as f32
        };
        let (fx, fy) = (x / CELL_MAP as f64 - 0.5, y / CELL_MAP as f64 - 0.5);
        let (c, r) = (fx.floor() as i64, fy.floor() as i64);
        let (tx, ty) = ((fx - c as f64) as f32, (fy - r as f64) as f32);
        let top = cell(c, r) * (1.0 - tx) + cell(c + 1, r) * tx;
        let bottom = cell(c, r + 1) * (1.0 - tx) + cell(c + 1, r + 1) * tx;
        (top * (1.0 - ty) + bottom * ty) * STEP_M as f32
    }
}

/// Line of sight samples: one every half cell (1 km).
const LOS_STEP: i32 = CELL_MAP / 2;

impl Relief {
    /// Whether the mountains leave the line of sight open between two points (map x, y south,
    /// altitude units), sampled every kilometre between them.
    pub fn sight_open(&self, a: (i32, i32, i32), b: (i32, i32, i32)) -> bool {
        if self.0.is_empty() {
            return true;
        }
        let (dx, dy) = ((b.0 - a.0) as f64, (b.1 - a.1) as f64);
        let steps = (dx.hypot(dy) / LOS_STEP as f64) as usize;
        (1..steps).all(|i| {
            let t = i as f64 / steps as f64;
            let ground = self.smooth_m(a.0 as f64 + dx * t, a.1 as f64 + dy * t) as f64 / WU_M;
            ground <= a.2 as f64 + (b.2 - a.2) as f64 * t
        })
    }

    /// The ground (smoothed, altitude units) at a map point.
    pub fn ground(&self, x: i32, y: i32) -> i32 {
        (self.smooth_m(x as f64, y as f64) as f64 / WU_M) as i32
    }
}

/// Look-ahead steps over the rolling hills (map units, 0 to 3 km).
const HILLS_AHEAD: [i32; 7] = [0, 16, 32, 64, 96, 128, 192];

/// Look-ahead steps along the heading (map units, one cell each).
const AHEAD: [i32; 4] = [0, CELL_MAP, 2 * CELL_MAP, 3 * CELL_MAP];

/// Eye height of a ground unit's crew or sights (altitude units, about 3 m).
pub const GROUND_EYE: i32 = 6;

impl crate::Sim {
    /// The rolling hills' ground at a map point (x east, y south), altitude units.
    pub(crate) fn hill_alt(&self, x: i32, y: i32) -> i32 {
        self.tiles.as_ref().map_or(0, |t| t.hills.ground_alt(x, y))
    }

    /// Whether the mountains and hills leave the line of sight open from a ground unit's eye
    /// (map x, y south) to a point (map x, y, altitude units).
    pub(crate) fn sight_from_ground(&self, x: i32, y: i32, to: (i32, i32, i32)) -> bool {
        let Some(t) = self.tiles.as_ref() else { return true };
        let ground = t.relief.ground(x, y).max(t.hills.ground_alt(x, y));
        let from = (x, y, ground + GROUND_EYE);
        t.relief.sight_open(from, to) && t.hills.sight_open(from, to)
    }

    /// The highest of the rolling hills under a flight unit and up to `HILLS_AHEAD` ahead of
    /// it, in altitude units.
    pub(crate) fn hills_ahead(&self, u: &crate::types::FlightUnit) -> i32 {
        let Some(t) = self.tiles.as_ref().filter(|t| !t.hills.is_flat()) else { return 0 };
        HILLS_AHEAD
            .iter()
            .map(|&d| {
                let x = u.x + self.trig.sin_mul(u.heading, d);
                let y = u.y - self.trig.cos_mul(u.heading, d);
                t.hills.ground_alt(x, y)
            })
            .max()
            .unwrap_or(0)
    }

    /// The highest ground, the mountains or the rolling hills, under a flight unit and ahead.
    pub(crate) fn ground_ahead(&self, u: &crate::types::FlightUnit) -> i32 {
        self.terrain_ahead(u).max(self.hills_ahead(u))
    }

    /// The highest ground under a flight unit and up to three cells (6 km) ahead of it, in
    /// altitude units.
    pub(crate) fn terrain_ahead(&self, u: &crate::types::FlightUnit) -> i32 {
        let Some(relief) = self.tiles.as_ref().map(|t| &t.relief) else { return 0 };
        AHEAD
            .iter()
            .map(|&d| {
                let x = u.x + self.trig.sin_mul(u.heading, d);
                let y = u.y - self.trig.cos_mul(u.heading, d);
                relief.height(x, y)
            })
            .max()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ridge between a radar and a low aircraft hides it; above the ridge it is seen.
    #[test]
    fn ridge_blocks_the_sight() {
        let mut m = vec![0.0; DIM * DIM];
        for row in 0..DIM {
            m[row * DIM + 100] = 1500.0;
        }
        let r = Relief::from_metres(&m);
        let x = |col: i32| col * CELL_MAP + CELL_MAP / 2;
        let y = 128 * CELL_MAP;
        let site = (x(90), y, 20);
        assert!(!r.sight_open(site, (x(110), y, 1000)));
        assert!(r.sight_open(site, (x(110), y, 8000)));
        assert!(r.sight_open(site, (x(98), y, 300)), "this side of the ridge");
        assert!(Relief::default().sight_open(site, (x(110), y, 0)));
    }

    #[test]
    fn heights_by_map_point() {
        let mut m = vec![0.0; DIM * DIM];
        // A 1000 m peak in the cell second from the north edge, third from the west.
        m[DIM + 2] = 1000.0;
        let r = Relief::from_metres(&m);
        // 32 steps of 32 m (1024 m) in 0.488 m altitude units.
        assert_eq!(r.height(2 * 128 + 5, 128 + 5), 2097);
        assert_eq!(r.height(5, 5), 0);
        assert_eq!(r.height(-1, 5), 0);
        assert_eq!(Relief::default().height(300, 200), 0);
    }

    /// Central Europe's relief comes with the sim data; a unit flying at its highest cell sees
    /// it ahead from 6 km out, and not once past it.
    #[test]
    fn mountains_ahead() {
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let data = crate::SimData::load_json(&assets.join("sim")).expect("assets/sim");
        let relief = &data.theaters[3].relief;
        let top = (0..DIM * DIM).max_by_key(|&i| relief.bytes()[i]).unwrap();
        let (x, y) = ((top % DIM) as i32 * CELL_MAP + 64, (top / DIM) as i32 * CELL_MAP + 64);
        let peak = relief.height(x, y);
        assert!(peak > 4000, "{peak}");
        let g = stealth_mission::GameData::load_json(&assets.join("mission")).unwrap();
        let s = stealth_mission::MissionSettings { theater: 3, ..Default::default() };
        let m = stealth_mission::generate(&g, &s, stealth_mission::Seed::Srand(0x1234)).unwrap();
        let sim = crate::Sim::with_options(&data, &m, Default::default());
        // Heading north (0), 5 km south of the peak, then 3 cells north of it.
        let mut u = crate::types::FlightUnit { x, y: y + 320, heading: 0, ..Default::default() };
        assert_eq!(sim.terrain_ahead(&u), peak);
        u.y = y - 3 * CELL_MAP;
        assert!(sim.terrain_ahead(&u) < peak);
    }
}

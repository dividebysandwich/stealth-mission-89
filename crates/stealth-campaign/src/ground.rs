//! Where convoys can drive: a grid of 250 m cells over the theater, land that is not
//! mountains or too steep a hillside, and water narrow enough to cross (a river: bridged or
//! forded), and the routes over it (climbing the hills costs more than going round them).

use std::collections::{BinaryHeap, HashMap};

/// Cells per side, and map units per cell side (250 m).
pub const DIM: usize = 2048;
pub const CELL: i32 = 0x8000 / DIM as i32;
/// Water this many cells across or fewer (along a row or a column) can be crossed.
const NARROW_WATER: usize = 2;
/// Ground higher than this (m) is mountains.
pub const MOUNTAIN_M: f32 = 100.0;
/// Driving over water costs this many times as much as over land.
const WATER_COST: f32 = 8.0;
/// A place's centre is reached from the nearest passable cell this many cells away at most.
const SNAP: i32 = 6;

/// Hillsides steeper than this (rise per metre between the cells' centres) are too steep to
/// drive up, and each unit of rise per metre costs this many times the distance.
const STEEP: f32 = 0.36;
const CLIMB_COST: f32 = 4.0;
/// Metres per cell side.
const CELL_M: f32 = 250.0;

const BLOCKED: u8 = 0;
const LAND: u8 = 1;
const FORD: u8 = 2;

/// The relief file's layout (`stealth_sim::relief`): 256 x 256 cells of 2 km, row 0 north, in
/// 32 m steps.
const RELIEF_DIM: usize = 256;
const RELIEF_STEP_M: f32 = 32.0;

pub struct GroundGrid {
    pass: Vec<u8>,
    /// The rolling hills' height at each cell's centre (m; empty for a flat theater).
    height: Vec<f32>,
    /// Connected areas (0: blocked).
    area: Vec<u32>,
}

impl std::fmt::Debug for GroundGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GroundGrid")
    }
}

/// The relief's height (m) at the centre of grid cell (col, row), blended between the relief's
/// cells like `Relief::smooth_m`.
fn relief_m(relief: &[u8], col: usize, row: usize) -> f32 {
    let scale = (DIM / RELIEF_DIM) as f32;
    let (fx, fy) = ((col as f32 + 0.5) / scale - 0.5, (row as f32 + 0.5) / scale - 0.5);
    let cell = |c: i64, r: i64| {
        let (c, r) = (c.clamp(0, RELIEF_DIM as i64 - 1), r.clamp(0, RELIEF_DIM as i64 - 1));
        relief[r as usize * RELIEF_DIM + c as usize] as f32
    };
    let (c, r) = (fx.floor() as i64, fy.floor() as i64);
    let (tx, ty) = (fx - c as f32, fy - r as f32);
    let top = cell(c, r) * (1.0 - tx) + cell(c + 1, r) * tx;
    let bottom = cell(c, r + 1) * (1.0 - tx) + cell(c + 1, r + 1) * tx;
    (top * (1.0 - ty) + bottom * ty) * RELIEF_STEP_M
}

const STEPS: [(i32, i32, f32); 8] = [
    (1, 0, 1.0),
    (-1, 0, 1.0),
    (0, 1, 1.0),
    (0, -1, 1.0),
    (1, 1, std::f32::consts::SQRT_2),
    (1, -1, std::f32::consts::SQRT_2),
    (-1, 1, std::f32::consts::SQRT_2),
    (-1, -1, std::f32::consts::SQRT_2),
];

impl GroundGrid {
    /// The grid from the water at each cell's centre (`TheaterData::water_grid(DIM)`) and the
    /// theater's relief (none: flat).
    pub fn build(
        water: &[bool],
        relief: Option<&[u8]>,
        hills: &stealth_mission::hills::Hills,
    ) -> Self {
        assert_eq!(water.len(), DIM * DIM);
        let height: Vec<f32> = if hills.is_flat() {
            Vec::new()
        } else {
            let m = CELL_M as f64;
            (0..DIM * DIM)
                .map(|i| {
                    let (c, r) = ((i % DIM) as f64, (i / DIM) as f64);
                    hills.height_m((c + 0.5) * m, (DIM as f64 - r - 0.5) * m)
                })
                .collect()
        };
        // A hillside too steep to drive: the rise to a neighbour's centre.
        let steep = |c: usize, r: usize| {
            !height.is_empty()
                && STEPS.iter().any(|&(dc, dr, len)| {
                    let (nc, nr) = (c as i32 + dc, r as i32 + dr);
                    (0..DIM as i32).contains(&nc)
                        && (0..DIM as i32).contains(&nr)
                        && (height[nr as usize * DIM + nc as usize] - height[r * DIM + c]).abs()
                            > STEEP * CELL_M * len
                })
        };
        let relief = relief.filter(|r| r.len() == RELIEF_DIM * RELIEF_DIM);
        // Runs of water along the rows and the columns.
        let mut across = vec![0u16; DIM * DIM];
        let mut down = vec![0u16; DIM * DIM];
        for r in 0..DIM {
            let mut c = 0;
            while c < DIM {
                if !water[r * DIM + c] {
                    c += 1;
                    continue;
                }
                let start = c;
                while c < DIM && water[r * DIM + c] {
                    c += 1;
                }
                for k in start..c {
                    across[r * DIM + k] = (c - start).min(u16::MAX as usize) as u16;
                }
            }
        }
        for c in 0..DIM {
            let mut r = 0;
            while r < DIM {
                if !water[r * DIM + c] {
                    r += 1;
                    continue;
                }
                let start = r;
                while r < DIM && water[r * DIM + c] {
                    r += 1;
                }
                for k in start..r {
                    down[k * DIM + c] = (r - start).min(u16::MAX as usize) as u16;
                }
            }
        }
        let mut pass = vec![BLOCKED; DIM * DIM];
        for r in 0..DIM {
            for c in 0..DIM {
                let i = r * DIM + c;
                if relief.is_some_and(|h| relief_m(h, c, r) > MOUNTAIN_M) || steep(c, r) {
                    continue;
                }
                pass[i] = if !water[i] {
                    LAND
                } else if (across[i].min(down[i]) as usize) <= NARROW_WATER {
                    FORD
                } else {
                    BLOCKED
                };
            }
        }
        let area = areas(&pass);
        Self { pass, height, area }
    }

    fn cell(x: i32, y: i32) -> Option<(i32, i32)> {
        let (c, r) = (x.div_euclid(CELL), y.div_euclid(CELL));
        (0..DIM as i32).contains(&c).then_some(())?;
        (0..DIM as i32).contains(&r).then_some((c, r))
    }

    fn at(&self, c: i32, r: i32) -> u8 {
        if !(0..DIM as i32).contains(&c) || !(0..DIM as i32).contains(&r) {
            return BLOCKED;
        }
        self.pass[r as usize * DIM + c as usize]
    }

    /// A vehicle can be at map point (x, y south).
    pub fn passable(&self, x: i32, y: i32) -> bool {
        Self::cell(x, y).is_some_and(|(c, r)| self.at(c, r) != BLOCKED)
    }

    /// Dry land a vehicle can stand on at map point (x, y).
    pub fn dry(&self, x: i32, y: i32) -> bool {
        Self::cell(x, y).is_some_and(|(c, r)| self.at(c, r) == LAND)
    }

    /// The nearest dry cell to map point (x, y) within [`SNAP`] cells.
    fn snap(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let (c0, r0) = Self::cell(x, y)?;
        let mut best: Option<(i32, (i32, i32))> = None;
        for dr in -SNAP..=SNAP {
            for dc in -SNAP..=SNAP {
                let d = dc * dc + dr * dr;
                if d <= SNAP * SNAP
                    && self.at(c0 + dc, r0 + dr) == LAND
                    && best.is_none_or(|(b, _)| d < b)
                {
                    best = Some((d, (c0 + dc, r0 + dr)));
                }
            }
        }
        best.map(|(_, p)| p)
    }

    /// The connected area a place at map point (x, y) is reached in (None: none near it).
    pub fn area(&self, x: i32, y: i32) -> Option<u32> {
        let (c, r) = self.snap(x, y)?;
        Some(self.area[r as usize * DIM + c as usize])
    }

    /// The way to drive from map point `from` to `to` (map units, both ends included), at
    /// most `limit` map units long; None if there is none.
    pub fn route(&self, from: (i32, i32), to: (i32, i32), limit: i32) -> Option<Vec<(i32, i32)>> {
        let a = self.snap(from.0, from.1)?;
        let b = self.snap(to.0, to.1)?;
        let idx = |(c, r): (i32, i32)| r as usize * DIM + c as usize;
        if self.area[idx(a)] != self.area[idx(b)] {
            return None;
        }
        let limit = limit as f32 / CELL as f32;
        let h = |(c, r): (i32, i32)| {
            let (dx, dy) = (((c - b.0).abs()) as f32, ((r - b.1).abs()) as f32);
            dx.max(dy) + (std::f32::consts::SQRT_2 - 1.0) * dx.min(dy)
        };
        // Costs in 1/16 cells (integers for the heap).
        let key = |f: f32| (f * 16.0) as u32;
        let mut open = BinaryHeap::new();
        let mut best: HashMap<usize, (f32, usize)> = HashMap::new();
        best.insert(idx(a), (0.0, usize::MAX));
        open.push(std::cmp::Reverse((key(h(a)), idx(a))));
        let mut found = false;
        while let Some(std::cmp::Reverse((_, i))) = open.pop() {
            if i == idx(b) {
                found = true;
                break;
            }
            let g = best[&i].0;
            let (c, r) = ((i % DIM) as i32, (i / DIM) as i32);
            for &(dc, dr, len) in &STEPS {
                let (nc, nr) = (c + dc, r + dr);
                let kind = self.at(nc, nr);
                if kind == BLOCKED {
                    continue;
                }
                // No cutting a blocked corner.
                if dc != 0
                    && dr != 0
                    && (self.at(c + dc, r) == BLOCKED || self.at(c, r + dr) == BLOCKED)
                {
                    continue;
                }
                let n = idx((nc, nr));
                let climb = if self.height.is_empty() {
                    0.0
                } else {
                    (self.height[n] - self.height[i]).abs() / CELL_M
                };
                let step = if kind == FORD { len * WATER_COST } else { len + climb * CLIMB_COST };
                let ng = g + step;
                // Water costs more but is no longer to drive: the limit counts the distance.
                if ng + h((nc, nr)) > limit * WATER_COST
                    || best.get(&n).is_some_and(|&(b, _)| b <= ng)
                {
                    continue;
                }
                best.insert(n, (ng, i));
                open.push(std::cmp::Reverse((key(ng + h((nc, nr))), n)));
            }
        }
        if !found {
            return None;
        }
        let mut cells = vec![idx(b)];
        while let Some(&(_, p)) = best.get(cells.last().unwrap()) {
            if p == usize::MAX {
                break;
            }
            cells.push(p);
        }
        cells.reverse();
        let cells: Vec<(i32, i32)> =
            cells.iter().map(|&i| ((i % DIM) as i32, (i / DIM) as i32)).collect();
        let length: f32 = cells
            .windows(2)
            .map(|w| (((w[1].0 - w[0].0).pow(2) + (w[1].1 - w[0].1).pow(2)) as f32).sqrt())
            .sum();
        if length > limit {
            return None;
        }
        let centre = |(c, r): (i32, i32)| (c * CELL + CELL / 2, r * CELL + CELL / 2);
        // From the place itself to the centre of the cell next to it a vehicle can stand in
        // (the lines in sight go between centres), and the same at the far end.
        let mut out = vec![from];
        out.extend(self.pull(&cells).into_iter().map(centre));
        out.push(to);
        out.dedup();
        Some(out)
    }

    /// The cells of a path that matter: from each, straight on to the furthest one in sight
    /// over dry land.
    fn pull(&self, cells: &[(i32, i32)]) -> Vec<(i32, i32)> {
        let mut out = vec![cells[0]];
        let mut i = 0;
        while i + 1 < cells.len() {
            let mut j = i + 1;
            while j + 1 < cells.len() && self.clear(cells[i], cells[j + 1]) {
                j += 1;
            }
            out.push(cells[j]);
            i = j;
        }
        out
    }

    /// Every cell on the line from the centre of cell `a` to that of `b` is dry land (looked at
    /// every map unit).
    fn clear(&self, a: (i32, i32), b: (i32, i32)) -> bool {
        let centre = |(c, r): (i32, i32)| (c * CELL + CELL / 2, r * CELL + CELL / 2);
        let ((x0, y0), (x1, y1)) = (centre(a), centre(b));
        let n = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
        (0..=n).all(|k| {
            let (x, y) = (x0 + (x1 - x0) * k / n, y0 + (y1 - y0) * k / n);
            self.at(x.div_euclid(CELL), y.div_euclid(CELL)) == LAND
        })
    }
}

/// Connected areas of the passable cells (8 neighbours, no cutting a blocked corner).
fn areas(pass: &[u8]) -> Vec<u32> {
    let mut area = vec![0u32; DIM * DIM];
    let at = |c: i32, r: i32| {
        (0..DIM as i32).contains(&c)
            && (0..DIM as i32).contains(&r)
            && pass[r as usize * DIM + c as usize] != BLOCKED
    };
    let mut next = 0;
    let mut stack = Vec::new();
    for start in 0..DIM * DIM {
        if pass[start] == BLOCKED || area[start] != 0 {
            continue;
        }
        next += 1;
        area[start] = next;
        stack.push(start);
        while let Some(i) = stack.pop() {
            let (c, r) = ((i % DIM) as i32, (i / DIM) as i32);
            for &(dc, dr, _) in &STEPS {
                let (nc, nr) = (c + dc, r + dr);
                if !at(nc, nr) || dc != 0 && dr != 0 && (!at(c + dc, r) || !at(c, r + dr)) {
                    continue;
                }
                let n = nr as usize * DIM + nc as usize;
                if area[n] == 0 {
                    area[n] = next;
                    stack.push(n);
                }
            }
        }
    }
    area
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Land everywhere but a river two cells wide down column 100..102 and a lake at
    /// columns 300..400.
    fn grid() -> GroundGrid {
        let mut water = vec![false; DIM * DIM];
        for r in 0..DIM {
            for c in (100..102).chain(300..400) {
                water[r * DIM + c] = true;
            }
        }
        GroundGrid::build(&water, None, &Default::default())
    }

    #[test]
    fn rivers_are_crossed_and_lakes_are_not() {
        let g = grid();
        let row = 500 * CELL;
        let r = g.route((50 * CELL, row), (200 * CELL, row), 0x8000).expect("over the river");
        assert_eq!(r.first(), Some(&(50 * CELL, row)));
        assert_eq!(r.last(), Some(&(200 * CELL, row)));
        assert!(g.route((200 * CELL, row), (500 * CELL, row), 0x8000).is_none());
        assert_ne!(g.area(200 * CELL, row), g.area(500 * CELL, row));
    }

    #[test]
    fn a_route_longer_than_the_limit_is_none() {
        let g = grid();
        let row = 500 * CELL;
        assert!(g.route((110 * CELL, row), (290 * CELL, row), 100 * CELL).is_none());
        let r = g.route((110 * CELL, row), (290 * CELL, row), 200 * CELL).unwrap();
        // Straight across open land: no turns.
        assert_eq!(r.len(), 4, "{r:?}");
    }

    #[test]
    fn mountains_block() {
        let mut relief = vec![0u8; RELIEF_DIM * RELIEF_DIM];
        // A ridge 400 m high along relief column 20 (grid columns ~160..168).
        for r in 0..RELIEF_DIM {
            relief[r * RELIEF_DIM + 20] = 13;
        }
        let g = GroundGrid::build(&vec![false; DIM * DIM], Some(&relief), &Default::default());
        let row = 500 * CELL;
        assert!(!g.passable(164 * CELL, row));
        assert!(g.route((150 * CELL, row), (180 * CELL, row), 0x8000).is_none());
    }

    /// Over the rolling hills (gentle, none too steep) a convoy drives on, a little round
    /// their tops.
    #[test]
    fn routes_over_the_hills() {
        let full = stealth_mission::hills::Hills::from_bytes(vec![255; 1024 * 1024]).unwrap();
        let g = GroundGrid::build(&vec![false; DIM * DIM], None, &full);
        let (from, to) = ((12000, 16000), (12640, 16000));
        let route = g.route(from, to, 2000).expect("a way over the hills");
        let length: f64 = route
            .windows(2)
            .map(|w| ((w[1].0 - w[0].0) as f64).hypot((w[1].1 - w[0].1) as f64))
            .sum();
        assert!(length < 640.0 * 1.4, "{length}");
    }

    /// The highlands' steep hillsides are no place for a site; the flat world is.
    #[test]
    fn steep_hillsides_found() {
        let full = stealth_mission::hills::Hills::from_bytes(vec![255; 1024 * 1024]).unwrap();
        let t = crate::Terrain::new(None, None, full);
        let steep = (0..40).flat_map(|i| (0..40).map(move |j| (12000 + i * 13, 16000 + j * 11)));
        assert!(steep.clone().any(|(x, y)| t.is_steep(x, y, 6)));
        assert!(!steep.clone().all(|(x, y)| t.is_steep(x, y, 6)));
        assert!(!crate::Terrain::default().is_steep(12000, 16000, 6));
    }
}

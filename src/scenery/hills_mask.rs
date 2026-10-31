//! The hills' mask (`stealth_sim::hills`, written with `--write-hills`): where the rolling
//! hills may rise. Flat at water (seas, lakes, river lines), concrete, the theater's raised
//! objects, towns and airfields (their approaches too), rising over `RAMP` cells from there,
//! scaled by the regional variety and the theater's own amount of hills.

use bevy::math::DVec2;
use stealth_sim::hills::{self, CELL_M, DIM, THEATER_M};

use super::airfield::Airfield;
use super::ground::Ground;
use super::network::{Network, TownKind};

/// Cells from flat ground to the full height.
const RAMP: f32 = 4.0;
/// The share of the hills left in towns and villages (their houses stand on gentle slopes),
/// and the cells from there to the full height.
const TOWN_HILLS: f32 = 0.25;
const VILLAGE_HILLS: f32 = 0.5;
const TOWN_RAMP: f32 = 2.0;
/// Samples per cell side for water and concrete.
const SAMPLES: usize = 4;
/// Flat ground round a town beyond its edge (metres), and round an airfield.
const TOWN_MARGIN: f64 = 300.0;
const FIELD_MARGIN: f64 = 700.0;
/// The approaches beyond the runway ends kept flat (metres along the runway).
const APPROACH: f64 = 3000.0;

/// How hilly each theater is (0..1, by theater id); flat if not listed.
const THEATERS: [(&str, f32); 1] = [("central_europe", 1.0)];

pub fn amount(theater: &str) -> f32 {
    THEATERS.iter().find(|(t, _)| *t == theater).map_or(0.0, |&(_, a)| a)
}

/// The centre of a mask cell (metres east, north).
fn centre(col: usize, row: usize) -> DVec2 {
    DVec2::new((col as f64 + 0.5) * CELL_M, THEATER_M - (row as f64 + 0.5) * CELL_M)
}

/// The cell of a point, if inside the theater.
fn cell(p: DVec2) -> Option<usize> {
    let (c, r) = ((p.x / CELL_M).floor(), ((THEATER_M - p.y) / CELL_M).floor());
    let n = DIM as f64;
    ((0.0..n).contains(&c) && (0.0..n).contains(&r)).then(|| r as usize * DIM + c as usize)
}

pub fn build(ground: &Ground, net: &Network, fields: &[Airfield], amount: f32) -> Vec<u8> {
    let mut flat = vec![false; DIM * DIM];
    // Water and concrete under the cell, and the raised objects' clearances.
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows = DIM.div_ceil(threads);
    std::thread::scope(|s| {
        for (k, chunk) in flat.chunks_mut(rows * DIM).enumerate() {
            s.spawn(move || {
                for (i, f) in chunk.iter_mut().enumerate() {
                    let (col, row) = (i % DIM, k * rows + i / DIM);
                    let base = centre(col, row) - DVec2::splat(CELL_M * 0.5);
                    let step = CELL_M / SAMPLES as f64;
                    *f = (0..=SAMPLES).any(|a| {
                        (0..=SAMPLES).any(|b| {
                            ground.needs_flat(base + DVec2::new(a as f64, b as f64) * step)
                        })
                    });
                }
            });
        }
    });
    let (mut town, mut village) = (vec![false; DIM * DIM], vec![false; DIM * DIM]);
    let mut mark = |p: DVec2| {
        if let Some(i) = cell(p) {
            flat[i] = true;
        }
    };
    for &(a, b) in &ground.water_lines {
        let n = (a.distance(b) / (CELL_M * 0.25)).ceil().max(1.0) as usize;
        for i in 0..=n {
            mark(a.lerp(b, i as f64 / n as f64));
        }
    }
    // Areas: every cell whose centre is inside.
    let area = |flat: &mut [bool], min: DVec2, max: DVec2, inside: &dyn Fn(DVec2) -> bool| {
        let (Some(a), Some(b)) = (cell(DVec2::new(min.x, max.y)), cell(DVec2::new(max.x, min.y)))
        else {
            return;
        };
        for row in a / DIM..=b / DIM {
            for col in a % DIM..=b % DIM {
                if inside(centre(col, row)) {
                    flat[row * DIM + col] = true;
                }
            }
        }
    };
    let clamp = |p: DVec2| p.clamp(DVec2::ZERO, DVec2::splat(THEATER_M - 1.0));
    for t in &net.towns {
        let r = DVec2::splat(t.r * 2.0 + TOWN_MARGIN + CELL_M);
        let (lo, hi) = (clamp(t.c - r), clamp(t.c + r));
        match t.kind {
            TownKind::Village => area(&mut village, lo, hi, &|p| t.near(p, 1.0, 0.0)),
            TownKind::Town => area(&mut town, lo, hi, &|p| t.near(p, 1.0, 0.0)),
            TownKind::City => area(&mut flat, lo, hi, &|p| t.near(p, 1.0, TOWN_MARGIN)),
        }
    }
    for f in fields {
        let w = &f.runway;
        let (along, across) = (w.half_len + APPROACH, w.half_width + FIELD_MARGIN);
        let r = DVec2::splat(along + across);
        area(&mut flat, clamp(w.c - r), clamp(w.c + r), &|p| {
            let d = p - w.c;
            d.dot(w.axis).abs() < along && d.dot(w.axis.perp()).abs() < across
        });
        let pts = f.pads.iter().flatten().copied().chain(f.buildings.iter().map(|b| b.p));
        let (mut lo, mut hi) = (w.c, w.c);
        for p in pts {
            (lo, hi) = (lo.min(p), hi.max(p));
        }
        let m = DVec2::splat(FIELD_MARGIN);
        area(&mut flat, clamp(lo - m), clamp(hi + m), &|_| true);
    }
    let (dist, town, village) = (distance(&flat), distance(&town), distance(&village));
    (0..DIM * DIM)
        .map(|i| {
            let ramp = ((dist[i] - 1.0) / RAMP)
                .min(TOWN_HILLS + (1.0 - TOWN_HILLS) * town[i] / TOWN_RAMP)
                .min(VILLAGE_HILLS + (1.0 - VILLAGE_HILLS) * village[i] / TOWN_RAMP)
                .clamp(0.0, 1.0);
            let c = centre(i % DIM, i / DIM);
            let v = ramp * hills::region(c.x, c.y) * amount;
            (v * 255.0).round() as u8
        })
        .collect()
}

/// Cells to the nearest flat cell (chamfer distance; 0 on them).
fn distance(flat: &[bool]) -> Vec<f32> {
    let mut d: Vec<f32> = flat.iter().map(|&f| if f { 0.0 } else { 1e6 }).collect();
    let n = DIM as isize;
    let diag = std::f32::consts::SQRT_2;
    let at = |c: isize, r: isize| (0..n).contains(&c) && (0..n).contains(&r);
    for pass in 0..2 {
        let (rows, cols): (Vec<isize>, Vec<isize>) = if pass == 0 {
            ((0..n).collect(), (0..n).collect())
        } else {
            ((0..n).rev().collect(), (0..n).rev().collect())
        };
        let s: isize = if pass == 0 { -1 } else { 1 };
        for &r in &rows {
            for &c in &cols {
                let i = (r * n + c) as usize;
                let mut v = d[i];
                for (dc, dr, w) in [(s, 0, 1.0), (0, s, 1.0), (s, s, diag), (-s, s, diag)] {
                    let (cc, rr) = (c + dc, r + dr);
                    if at(cc, rr) {
                        v = v.min(d[(rr * n + cc) as usize] + w);
                    }
                }
                d[i] = v;
            }
        }
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distances_from_flat() {
        let mut flat = vec![false; DIM * DIM];
        flat[500 * DIM + 500] = true;
        let d = distance(&flat);
        assert_eq!(d[500 * DIM + 500], 0.0);
        assert_eq!(d[500 * DIM + 503], 3.0);
        assert_eq!(d[497 * DIM + 500], 3.0);
        assert!((d[502 * DIM + 502] - 2.0 * std::f32::consts::SQRT_2).abs() < 1e-4);
    }
}

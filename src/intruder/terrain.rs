//! What the A-6's track radar sees of the ground ahead: heights over a fan of bearings and
//! ranges, for the vertical display's terrain clearance picture and the horizontal display's
//! terrain clearance plan and elevation scan (`vdi`, `scope`).

use bevy::math::DVec2;
use bevy::prelude::*;

use super::{IntruderPanel, PhdMode, Power, VdiDisplay};
use crate::aircraft::AircraftKind;
use crate::flight::{Aircraft, Player};
use crate::ground_radar::{Ground, TheaterGround};
use crate::world::WorldPos;

/// Bearings across the fan and ranges along each.
pub const BEARINGS: usize = 49;
pub const RANGES: usize = 40;
/// The fan's half width (rad) and its reach (m: 10 nautical miles).
pub const HALF: f32 = 30.0 * std::f32::consts::PI / 180.0;
pub const REACH: f32 = 18_520.0;
/// Bearings scanned a frame.
const PER_FRAME: usize = 7;

/// The range (m) of step `i` along a bearing: closer together near the aircraft.
pub fn range_at(i: usize) -> f32 {
    let t = (i + 1) as f32 / RANGES as f32;
    REACH * t * t
}

/// The bearing (rad right of the nose) of column `i`.
pub fn bearing_at(i: usize) -> f32 {
    (i as f32 / (BEARINGS - 1) as f32 * 2.0 - 1.0) * HALF
}

#[derive(Resource)]
pub struct TerrainScan {
    /// The radar looks (a terrain picture is asked for, in the air).
    pub on: bool,
    /// The ground's height (m above the sea) per bearing and range, as last scanned.
    pub ground: Vec<f32>,
    /// The next bearing to scan.
    next: usize,
}

impl Default for TerrainScan {
    fn default() -> Self {
        Self { on: false, ground: vec![0.0; BEARINGS * RANGES], next: 0 }
    }
}

impl TerrainScan {
    pub fn height(&self, bearing: usize, range: usize) -> f32 {
        self.ground[bearing.min(BEARINGS - 1) * RANGES + range.min(RANGES - 1)]
    }

    /// The elevation (rad above the level) of the highest ground between `near` and `far`
    /// metres on a bearing, seen from `alt` metres; None with none of its steps in there.
    pub fn elevation(&self, bearing: usize, near: f32, far: f32, alt: f32) -> Option<f32> {
        (0..RANGES)
            .map(|i| (range_at(i), i))
            .filter(|(r, _)| *r > near && *r <= far)
            .map(|(r, i)| ((self.height(bearing, i) - alt) / r).atan())
            .reduce(f32::max)
    }

    /// The ground along the nose: range (m), height against `alt` (m) and whether the radar
    /// sees it over what is nearer.
    pub fn profile(&self, alt: f32) -> Vec<(f32, f32, bool)> {
        let mid = BEARINGS / 2;
        let mut top = f32::NEG_INFINITY;
        (0..RANGES)
            .map(|i| {
                let (r, h) = (range_at(i), self.height(mid, i) - alt);
                let slope = h / r;
                let seen = slope >= top;
                top = top.max(slope);
                (r, h, seen)
            })
            .collect()
    }

    /// Scans `PER_FRAME` more bearings from `own` (m east, north) heading `heading` (rad).
    pub fn scan(&mut self, own: DVec2, heading: f32, ground: &impl Ground) {
        for _ in 0..PER_FRAME {
            let b = self.next;
            self.next = (self.next + 1) % BEARINGS;
            let (s, c) = (heading + bearing_at(b)).sin_cos();
            for i in 0..RANGES {
                let r = range_at(i) as f64;
                self.ground[b * RANGES + i] =
                    ground.height(own.x + s as f64 * r, own.y + c as f64 * r);
            }
        }
    }
}

pub(super) fn scan(
    settings: Res<crate::settings::Settings>,
    panel: Res<IntruderPanel>,
    aircraft: Query<(&Aircraft, &WorldPos), With<Player>>,
    combat: Option<Res<crate::combat::Combat>>,
    theater: Option<Res<crate::world::Theater>>,
    scenery: Option<Res<crate::scenery::Scenery>>,
    mut scan: ResMut<TerrainScan>,
) {
    let wanted = panel.vdi == VdiDisplay::TerrainClearance
        || (panel.phd_power == Power::On && panel.phd_mode != PhdMode::Search);
    let flying = aircraft
        .single()
        .ok()
        .filter(|(a, _)| settings.aircraft == AircraftKind::A6 && wanted && a.ended.is_none());
    let (Some((a, pos)), Some(combat)) = (flying, combat) else {
        if scan.on {
            *scan = TerrainScan::default();
        }
        return;
    };
    let hills = theater.as_ref().map(|t| t.hills.clone()).unwrap_or_default();
    let relief = combat.sim.relief();
    let ground = TheaterGround { hills: &hills, relief: &relief, scenery: scenery.as_deref() };
    // A whole fan at once when the radar comes on.
    let passes = if scan.on { 1 } else { BEARINGS.div_ceil(PER_FRAME) };
    scan.on = true;
    for _ in 0..passes {
        scan.scan(DVec2::new(pos.0.x, -pos.0.z), a.euler().0, &ground);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenery::Cover;

    /// Flat land with a ridge 300 m high from 4 to 5 km north.
    struct Ridge;

    impl Ground for Ridge {
        fn height(&self, _e: f64, n: f64) -> f32 {
            if (4000.0..5000.0).contains(&n) { 300.0 } else { 0.0 }
        }

        fn cover(&self, _e: f64, _n: f64, _r: f64) -> Cover {
            Cover::Open
        }
    }

    #[test]
    fn a_ridge_ahead_stands_up_and_hides_the_ground_behind() {
        let mut scan = TerrainScan::default();
        for _ in 0..BEARINGS.div_ceil(PER_FRAME) {
            scan.scan(DVec2::ZERO, 0.0, &Ridge);
        }
        let mid = BEARINGS / 2;
        // From 150 m the ridge is above the level, the plain before it below.
        assert!(scan.elevation(mid, 3900.0, 5100.0, 150.0).unwrap() > 0.02);
        assert!(scan.elevation(mid, 500.0, 3500.0, 150.0).unwrap() < 0.0);
        let profile = scan.profile(150.0);
        let behind = profile.iter().find(|p| p.0 > 5200.0 && p.0 < 7000.0).unwrap();
        assert!(!behind.2, "the ground just behind the ridge is in its shadow");
        assert!(profile.iter().any(|p| p.1 > 100.0 && p.2));
        assert!(range_at(0) < 20.0 && (range_at(RANGES - 1) - REACH).abs() < 1.0);
        assert!((bearing_at(0) + HALF).abs() < 1e-5 && bearing_at(mid).abs() < 1e-5);
    }
}

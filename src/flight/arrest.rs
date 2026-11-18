//! A carrier landing's arrested stop. Any touchdown on the deck catches a wire (the game has no
//! bolters, nor a tailhook to lower): the wire pulls the aircraft to a stop in a steady
//! deceleration, whatever its throttle, well short of the deck's end. Both fixed-wing flight
//! models ask [`Aircraft::wire_speed`] for their ground roll's speed while it holds them.

use serde::{Deserialize, Serialize};
use stealth_sim::catapult::{LANDING_BOX, pivot};

use super::Aircraft;
use crate::world::WU;

/// The run-out an arrested landing takes (m), at least this much (a touchdown far up the
/// deck), and how far short of the deck's end it stops at the most.
const RUN_OUT_M: f64 = 100.0;
const MIN_RUN_OUT_M: f64 = 30.0;
const END_MARGIN_M: f64 = 20.0;

/// Caught by a wire: the speed at touchdown (velocity units), how far it has pulled out (wu),
/// and how far it stops in (wu).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Trap {
    pub v0: f32,
    pub rolled: f32,
    pub stop: f32,
}

impl Trap {
    /// A touchdown at `speed` (velocity units) at (east, north) wu, rolling along `track` (rad,
    /// clockwise from north), on the deck of the carrier whose world object is at (map) `obj`.
    pub fn catch(speed: f32, east: f64, north: f64, track: f64, obj: (i32, i32)) -> Trap {
        let room = room_ahead(east, north, track, obj) * WU;
        let stop = RUN_OUT_M.min(room - END_MARGIN_M).max(MIN_RUN_OUT_M);
        Trap { v0: speed.max(0.0), rolled: 0.0, stop: (stop / WU) as f32 }
    }
}

/// The deck ahead of (east, north) wu along `track` (rad, clockwise from north) to the edge of
/// the carrier's landing box round its middle (wu).
pub fn room_ahead(east: f64, north: f64, track: f64, obj: (i32, i32)) -> f64 {
    let (pe, pn) = pivot(obj.0, obj.1);
    let (hw, hl) = (LANDING_BOX.0 * 32.0, LANDING_BOX.1 * 32.0);
    let (dx, dy) = (track.sin(), track.cos());
    let exit = |p: f64, d: f64, half: f64| {
        if d.abs() < 1e-9 { f64::INFINITY } else { ((half * d.signum()) - p) / d }
    };
    exit(east - pe, dx, hw).min(exit(north - pn, dy, hl)).max(0.0)
}

impl Aircraft {
    /// The wire's hold on this step's roll: the speed along the nose (velocity units) while it
    /// pulls the aircraft out (and holds it, stopped), else None. The thrust does not matter.
    pub fn wire_speed(&mut self, dt: f32) -> Option<f32> {
        let mut t = self.trap?;
        let left = (1.0 - t.rolled / t.stop.max(1.0)).max(0.0);
        let v = t.v0 * left.sqrt();
        t.rolled += v / 16.0 * dt;
        self.trap = Some(t);
        self.brakes = false;
        Some(if t.rolled >= t.stop { 0.0 } else { v })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_within_the_run_out() {
        let mut a = Aircraft::new_on_runway(5000.0);
        let v0 = 150.0 * 27.0;
        a.trap = Some(Trap { v0, rolled: 0.0, stop: (RUN_OUT_M / WU) as f32 });
        let dt = 1.0 / 60.0;
        let mut run = 0.0;
        let mut steps = 0;
        while let Some(v) = a.wire_speed(dt) {
            if v == 0.0 {
                break;
            }
            run += v / 16.0 * dt;
            steps += 1;
            assert!(steps < 6000);
        }
        let run_m = run as f64 * WU;
        assert!(run_m <= RUN_OUT_M + 1.0 && run_m > RUN_OUT_M * 0.9, "{run_m}");
        // About 2.3 s and 3 g from 150 kt, as a real trap.
        let secs = steps as f32 * dt;
        assert!((1.5..3.5).contains(&secs), "{secs}");
        // Held once stopped.
        assert_eq!(a.wire_speed(dt), Some(0.0));
    }

    #[test]
    fn stops_short_of_the_deck_end() {
        let obj = (0x4000, 0x4000);
        let (pe, pn) = pivot(obj.0, obj.1);
        let half = LANDING_BOX.1 * 32.0;
        // Touching down 60 m from the bow, heading for it.
        let north = pn + half - 60.0 / WU;
        let t = Trap::catch(140.0 * 27.0, pe, north, 0.0, obj);
        let stop_m = t.stop as f64 * WU;
        assert!((stop_m - (60.0 - END_MARGIN_M)).abs() < 0.5, "{stop_m}");
        // From the stern: the whole run-out.
        let t = Trap::catch(140.0 * 27.0, pe, pn - half + 10.0, 0.0, obj);
        assert!((t.stop as f64 * WU - RUN_OUT_M).abs() < 0.5);
        // Along the angled deck, to port.
        let room = room_ahead(pe, pn, -9f64.to_radians(), obj);
        assert!(room > 0.0 && room <= half / 9f64.to_radians().cos() + 1.0);
    }
}

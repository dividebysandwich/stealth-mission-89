//! The operator's marks on the waterfalls: a contact's trace marked on a sensor (with the mark
//! letter chosen, A to H) becomes a bearing line from that sensor on the MAP page. The same
//! letter marked on several sensors crosses their lines: the least-squares point of the lines
//! is the contact's estimated position, its error ring smaller the more sensors hold it and the
//! wider the angles between them.
//!
//! A mark follows its trace from report to report (the trace nearest it within `FOLLOW`
//! degrees, heard over the noise); with none it stays where it was and is lost after `LOST`
//! seconds, until its trace comes back near it. A mark fades from the map over `FADE` seconds
//! from when its trace was last seen, and is gone after that (the map stays clear); its line still
//! counts for the estimate while it fades (an echo's mark is only seen for a moment). The marks are
//! the operator's own (his client's).

use super::sonar::bearing_off;

/// Mark letters.
pub const IDS: usize = 8;
/// A mark follows a trace this close (degrees); a trace this weak (dB) is not followed.
pub const FOLLOW: f32 = 10.0;
const WEAKEST: i8 = 2;
/// Seconds without its trace before a mark is lost; before it has faded away.
const LOST: f32 = 3.0;
pub const FADE: f32 = 300.0;
/// How well a marked bearing is known (degrees, one sigma).
const BEARING_SIGMA: f64 = 4.0;

/// A mark: its letter (0 A .. 7 H), the sensor (a buoy's id, 0 the dipping sonar), the bearing
/// (degrees true), the trace's signal, and when its trace was last seen (the console's clock).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark {
    pub id: u8,
    pub sensor: u16,
    pub bearing: f32,
    pub snr: i8,
    pub seen: f32,
}

impl Mark {
    pub fn letter(&self) -> char {
        letter(self.id)
    }

    pub fn lost(&self, now: f32) -> bool {
        now - self.seen > LOST
    }

    pub fn faded(&self, now: f32) -> bool {
        now - self.seen > FADE
    }
}

/// A colour faded with age (s) over `fade` seconds: full the first third, grey the second, dark
/// the last; None after.
pub fn faded_colour(col: u8, age: f32, fade: f32) -> Option<u8> {
    use crate::station::scope::{DARK, GREY};
    match age / fade {
        f if f < 1.0 / 3.0 => Some(col),
        f if f < 2.0 / 3.0 => Some(GREY),
        f if f < 1.0 => Some(DARK),
        _ => None,
    }
}

pub fn letter(id: u8) -> char {
    (b'A' + id % IDS as u8) as char
}

/// The trace a mark at `bearing` holds in a report: the nearest contact (not a ping) within
/// `FOLLOW`, heard over the noise.
pub fn trace_near(s: &stealth_sim::asw::Sample, bearing: f32) -> Option<&stealth_sim::asw::Line> {
    s.lines
        .iter()
        .filter(|l| !l.ping && l.snr >= WEAKEST)
        .filter_map(|l| l.bearing.map(|b| (l, bearing_off(b as f32, bearing).abs())))
        .filter(|&(_, d)| d <= FOLLOW)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(l, _)| l)
}

/// A sensor's new report: its marks move onto their traces.
pub fn follow(marks: &mut [Mark], sensor: u16, s: &stealth_sim::asw::Sample, now: f32) {
    for m in marks.iter_mut().filter(|m| m.sensor == sensor) {
        if let Some(l) = trace_near(s, m.bearing) {
            let b = l.bearing.unwrap_or_default() as f32;
            // Smoothly onto it: the reports' bearings jitter.
            m.bearing = (m.bearing + bearing_off(b, m.bearing) * 0.35).rem_euclid(360.0);
            m.snr = l.snr;
            m.seen = now;
        }
    }
}

/// Where a letter's marks put the contact: the least-squares point of their bearing lines
/// from their sensors (m east, north), and its error (m). None with fewer than two sensors, or
/// lines too nearly parallel to cross.
pub fn estimate(lines: &[([f64; 2], f32)]) -> Option<([f64; 2], f64)> {
    if lines.len() < 2 {
        return None;
    }
    // Sum of n n^T over the lines' normals, and of n n^T p.
    let (mut a, mut b, mut c, mut rx, mut ry) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for &(p, brg) in lines {
        let t = (brg as f64).to_radians();
        let (nx, ny) = (t.cos(), -t.sin());
        a += nx * nx;
        b += nx * ny;
        c += ny * ny;
        let d = nx * p[0] + ny * p[1];
        rx += nx * d;
        ry += ny * d;
    }
    let det = a * c - b * b;
    if det.abs() < 0.05 * lines.len() as f64 {
        return None;
    }
    let at = [(c * rx - b * ry) / det, (a * ry - b * rx) / det];
    // Each line misses the point by the bearing's error at its range; the lines together, as
    // many as there are and as much across each other as they cross (the determinant), pin it
    // tighter.
    let mut sum = 0.0;
    for &(p, _) in lines {
        let r = (at[0] - p[0]).hypot(at[1] - p[1]);
        sum += (r * BEARING_SIGMA.to_radians()).powi(2);
    }
    let per_line = (sum / lines.len() as f64).sqrt();
    let spread = (det / (lines.len() as f64).powi(2)).sqrt().clamp(0.15, 1.0);
    let err = per_line / (lines.len() as f64 - 1.0).sqrt() / spread;
    Some((at, err.max(50.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lines from sensors round a contact cross on it; more of them, the smaller the error.
    #[test]
    fn more_sensors_pin_the_contact_tighter() {
        let target = [3000.0, 4000.0];
        let line = |p: [f64; 2]| {
            let b = (target[0] - p[0]).atan2(target[1] - p[1]).to_degrees().rem_euclid(360.0) as f32;
            (p, b)
        };
        let two = [line([0.0, 0.0]), line([6000.0, 0.0])];
        let (at, e2) = estimate(&two).expect("crossed");
        assert!((at[0] - target[0]).hypot(at[1] - target[1]) < 1.0);
        let four = [line([0.0, 0.0]), line([6000.0, 0.0]), line([0.0, 8000.0]), line([6000.0, 8000.0])];
        let (_, e4) = estimate(&four).expect("crossed");
        assert!(e4 < e2, "four {e4:.0} m, two {e2:.0} m");
        // Parallel lines don't cross.
        assert!(estimate(&[([0.0, 0.0], 10.0), ([100.0, 0.0], 10.0)]).is_none());
    }

    #[test]
    fn a_mark_follows_its_trace_and_is_lost_without_it() {
        use stealth_sim::asw::{Line, Sample};
        let mut marks = [Mark { id: 0, sensor: 3, bearing: 90.0, snr: 10, seen: 0.0 }];
        let s = |b: u16, snr: i8| Sample { lines: vec![Line { snr, bearing: Some(b), ..Default::default() }], ..Default::default() };
        for k in 0..20 {
            follow(&mut marks, 3, &s(95, 10), k as f32 * 0.25);
        }
        assert!((marks[0].bearing - 95.0).abs() < 0.5, "{}", marks[0].bearing);
        // Too weak: it stays, and is lost in time.
        for k in 20..40 {
            follow(&mut marks, 3, &s(97, 0), k as f32 * 0.25);
        }
        assert!((marks[0].bearing - 95.0).abs() < 0.5);
        assert!(marks[0].lost(10.0));
        assert!(!marks[0].faded(10.0) && marks[0].faded(400.0));
        assert_eq!(faded_colour(11, 10.0, FADE), Some(11));
        assert_eq!(faded_colour(11, 250.0, FADE), Some(crate::station::scope::DARK));
        assert_eq!(faded_colour(11, 301.0, FADE), None);
    }
}

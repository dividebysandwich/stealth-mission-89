//! The Tornado's terrain following radar display (the E-scope over the left MFD): range ahead
//! across (0 to `RANGE_KM`, ticks every kilometre), the elevation angle from the flight path up.
//! Two curves: the terrain as the radar sees it (`flight::terrain_follow::TerrainRadar`; the
//! returns in the shadow of nearer terrain left out) and the steering template the terrain is
//! compared with. The radar measures angles, so level ground under the aircraft is a curve
//! rising towards the flight path with the range; while the terrain stays under the template
//! the aircraft may descend, where it rises above it the aircraft climbs.

use bevy::prelude::*;

use crate::canvas::Canvas;
use crate::flight::terrain_follow::{TerrainRadar, template};
use crate::font::Fonts;

/// Range shown (km).
const RANGE_KM: f32 = 5.0;
/// Elevation angles (degrees from the flight path) at the baseline and the top of the plot.
const ANGLE_LO: f32 = -15.0;
const ANGLE_HI: f32 = 10.0;
/// The scope's colours: the face, dim marks and the traces.
const FACE: u8 = 5;
const DIM: u8 = 4;
const TRACE: u8 = 10;

/// Draws the scope on a square canvas (transparent round the face).
pub fn draw(c: &mut Canvas, fonts: &Fonts, radar: &TerrainRadar) {
    c.px.fill(0);
    let n = c.w.min(c.h) as f32;
    let centre = Vec2::splat(n / 2.0);
    let r = n / 2.0 - 1.0;
    for y in 0..c.h {
        for x in 0..c.w {
            if (Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - centre).length() <= r {
                c.set(x as i32, y as i32, FACE);
            }
        }
    }
    // The plot: the baseline low across the face, the range from its left end.
    let (x0, x1) = (n * 0.14, n * 0.86);
    let (top, base) = (n * 0.2, n * 0.7);
    let w = (n / 120.0).max(1.0);
    let at = |d: f32, angle: f32| {
        let y = base - (angle.to_degrees() - ANGLE_LO) / (ANGLE_HI - ANGLE_LO) * (base - top);
        Vec2::new(x0 + d / (RANGE_KM * 1000.0) * (x1 - x0), y.min(base))
    };
    c.stroke(Vec2::new(x0, base), Vec2::new(x1, base), w, TRACE);
    let scale = (n / 100.0).round().max(1.0) as i32;
    for km in 1..RANGE_KM as i32 {
        let x = x0 + km as f32 / RANGE_KM * (x1 - x0);
        c.stroke(Vec2::new(x, base - n * 0.025), Vec2::new(x, base + n * 0.025), w, TRACE);
        let label = km.to_string();
        let lw = fonts.width(label.as_bytes()) * scale;
        fonts.text_scaled(
            c,
            x as i32 - lw / 2,
            (base + n * 0.05) as i32,
            label.as_bytes(),
            TRACE,
            scale,
        );
    }
    // The flight path: a dim mark at either side.
    let y = at(0.0, 0.0f32.to_radians()).y;
    for (a, b) in [(x0 - n * 0.06, x0 - n * 0.02), (x1 + n * 0.02, x1 + n * 0.06)] {
        c.stroke(Vec2::new(a, y), Vec2::new(b, y), w, DIM);
    }
    if !radar.on {
        return;
    }
    c.set_clip(x0 as i32, top as i32, x1 as i32, base as i32 + 1);
    // The template: the zero command line along the flight path.
    let max_d = RANGE_KM * 1000.0;
    let steps = 60;
    let curve: Vec<Vec2> = (1..=steps)
        .map(|i| {
            let d = max_d * i as f32 / steps as f32;
            at(d, template(d, radar.speed).atan2(d))
        })
        .collect();
    c.polyline(&curve, w * 1.5, TRACE);
    // The terrain: the returns the beam reaches, broken where nearer terrain hides it.
    let mut run: Vec<Vec2> = Vec::new();
    for &(d, h, seen) in radar.returns.iter().filter(|r| r.0 > 0.0 && r.0 <= max_d) {
        if seen {
            run.push(at(d, h.atan2(d) - radar.gamma));
        } else if !run.is_empty() {
            c.polyline(&std::mem::take(&mut run), w * 1.5, TRACE);
        }
    }
    if run.len() > 1 {
        c.polyline(&run, w * 1.5, TRACE);
    }
    c.reset_clip();
}

//! The A-6's round scopes: the pilot's horizontal display (PHD), the bombardier/navigator's
//! direct view radar indicator (DVRI) and his sensor scope's overlay over the pod's picture.
//! Test patterns for now (range rings, a sweep line and the scope's name), until the radar
//! pictures are drawn.

use bevy::prelude::*;

use super::{IntruderPanel, PhdMode, Power, disc, ink};
use crate::canvas::Canvas;
use crate::font::{Align, Fonts, stroke_text_at};

/// The round scopes' canvas (drawing units) and its texels per unit; the sensor scope's
/// overlay (4:3).
pub const CANVAS: (usize, usize) = (320, 320);
pub const SENSOR_CANVAS: (usize, usize) = (320, 240);
pub const SCALE: usize = 2;

/// The horizontal display's green tube, the direct view indicator's orange storage tube and
/// the sensor scope's pale green symbols.
pub fn phd_colours() -> std::sync::Arc<[u32; 256]> {
    with_ring(super::phosphor([3, 12, 5], [80, 255, 90]))
}

pub fn dvri_colours() -> std::sync::Arc<[u32; 256]> {
    with_ring(super::phosphor([40, 14, 2], [255, 150, 30]))
}

/// Colours past the phosphor's ramp: the heading ring's black metal, its white engraving and
/// the fixed index's orange.
const RING: u8 = 101;
const WHITE: u8 = 102;
const INDEX: u8 = 103;

fn with_ring(ramp: std::sync::Arc<[u32; 256]>) -> std::sync::Arc<[u32; 256]> {
    let mut lut = *ramp;
    lut[RING as usize] = u32::from_le_bytes([10, 10, 11, 255]);
    lut[WHITE as usize] = u32::from_le_bytes([225, 225, 215, 255]);
    lut[INDEX as usize] = u32::from_le_bytes([255, 150, 40, 255]);
    std::sync::Arc::new(lut)
}

pub fn sensor_colours() -> std::sync::Arc<[u32; 256]> {
    super::phosphor([4, 8, 5], [170, 255, 180])
}

/// The scope's tube inside its heading ring: its radius as a fraction of the canvas' half
/// width (the cockpit model's glass covers that much, `tools/models/build_cockpit_a6.py`).
pub const TUBE: f32 = 0.76;

/// The heading ring round the tube: a compass card turned to the heading (rad) under the fixed
/// index at the top, ticks every 5 degrees, longer every 10, numbered (tens of degrees) every 30.
fn heading_ring(c: &mut Canvas, heading: f32) {
    let mid = Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0);
    let rad = mid.x.min(mid.y);
    let (inner, outer) = (rad * (TUBE + 0.03), rad * 0.99);
    for i in 0..72 {
        let deg = i as f32 * 5.0;
        let a = deg.to_radians() - heading;
        let d = Vec2::new(a.sin(), -a.cos());
        let len = if i % 2 == 0 { 0.4 } else { 0.22 };
        c.stroke(mid + d * inner, mid + d * (inner + (outer - inner) * len), 1.0, WHITE);
        if i % 6 == 0 {
            let p = mid + d * (inner + (outer - inner) * 0.7);
            let label = format!("{}", i * 5 / 10);
            stroke_text_at(c, p.x, p.y, &label, rad * 0.075, 1.0, WHITE, Align::Centre);
        }
    }
    // The fixed index.
    let top = mid - Vec2::Y * outer;
    c.triangle(
        [top + Vec2::Y * rad * 0.07, top + Vec2::X * rad * 0.035, top - Vec2::X * rad * 0.035],
        INDEX,
    );
}

/// Range rings, cross lines, a sweep line turning with `time` and the name, inside the tube;
/// the ring round it black.
fn pattern(c: &mut Canvas, name: &str, rings: i32, time: f32) {
    let mid = Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0);
    let rad = mid.x.min(mid.y) * TUBE - 2.0;
    disc(c, mid, rad + 2.0, ink(0.06));
    for i in 1..=rings {
        c.ring(mid, rad * i as f32 / rings as f32, 1.0, ink(0.4));
    }
    c.stroke(mid - Vec2::X * rad, mid + Vec2::X * rad, 1.0, ink(0.3));
    c.stroke(mid - Vec2::Y * rad, mid + Vec2::Y * rad, 1.0, ink(0.3));
    let a = time * 1.2;
    c.stroke(mid, mid + Vec2::new(a.sin(), -a.cos()) * rad, 1.6, ink(0.9));
    stroke_text_at(c, mid.x, mid.y + rad * 0.5, name, 12.0, 1.5, ink(0.75), Align::Centre);
}

/// The tube dark (off) or faintly lit (standby), the ring round it.
fn tube(c: &mut Canvas, glow: f32) {
    let mid = Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0);
    c.clear(RING);
    disc(c, mid, mid.x.min(mid.y) * TUBE, if glow > 0.0 { ink(glow) } else { 0 });
}

/// Provisional: the pilot's horizontal display's test pattern, by its mode and power switches,
/// in its heading ring.
pub fn draw_phd(c: &mut Canvas, _fonts: &Fonts, panel: &IntruderPanel, heading: f32, time: f32) {
    c.reset_clip();
    match panel.phd_power {
        Power::Off => tube(c, 0.0),
        Power::Standby => tube(c, 0.04),
        Power::On => {
            tube(c, 0.0);
            let name = match panel.phd_mode {
                PhdMode::Tcppi => "PHD TCPPI",
                PhdMode::EScan => "PHD E-SCAN",
                PhdMode::Search => "PHD SRCH",
            };
            pattern(c, name, 3, time);
        }
    }
    heading_ring(c, heading);
}

/// Provisional: the direct view radar indicator's test pattern, by the search radar's power,
/// in its heading ring.
pub fn draw_dvri(c: &mut Canvas, _fonts: &Fonts, panel: &IntruderPanel, heading: f32, time: f32) {
    c.reset_clip();
    match panel.search_power {
        Power::Off => tube(c, 0.0),
        Power::Standby => tube(c, 0.04),
        Power::On => {
            tube(c, 0.0);
            let name = format!("DVRI {} MI", panel.search_range.miles());
            pattern(c, &name, 4, time);
        }
    }
    heading_ring(c, heading);
}

/// Provisional: the sensor scope's overlay over the pod's picture (index 0 see-through): a
/// reticle and the pod's name.
pub fn draw_sensor(c: &mut Canvas, _fonts: &Fonts, _panel: &IntruderPanel) {
    c.reset_clip();
    c.clear(crate::canvas::TRANSPARENT);
    let (w, h) = (c.w as f32, c.h as f32);
    let mid = Vec2::new(w / 2.0, h / 2.0);
    for d in [Vec2::X, Vec2::Y] {
        c.stroke(mid + d * 10.0, mid + d * 40.0, 1.2, ink(0.9));
        c.stroke(mid - d * 10.0, mid - d * 40.0, 1.2, ink(0.9));
    }
    stroke_text_at(c, 30.0, h - 14.0, "TRIM", 8.0, 1.2, ink(0.8), Align::Centre);
}

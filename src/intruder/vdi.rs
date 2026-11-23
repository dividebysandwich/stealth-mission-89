//! The pilot's vertical display indicator (VDI): a test pattern for now (a horizon, a pitch
//! ladder and the display's name on the green tube), until the contact analog picture is drawn.

use bevy::prelude::*;

use super::{IntruderPanel, VdiDisplay, disc, ink};
use crate::canvas::Canvas;
use crate::font::{Align, Fonts, stroke_text_at};

/// The canvas (drawing units, 4:3) and its texels per unit.
pub const CANVAS: (usize, usize) = (320, 240);
pub const SCALE: usize = 2;

/// The tube's phosphor: dark green glass, bright yellow-green.
pub fn colours() -> std::sync::Arc<[u32; 256]> {
    super::phosphor([3, 14, 6], [70, 255, 96])
}

/// Provisional: the test pattern, by the DISPLAY button that is in.
pub fn draw(c: &mut Canvas, _fonts: &Fonts, panel: &IntruderPanel, pitch: f32, roll: f32) {
    c.reset_clip();
    c.clear(0);
    let (w, h) = (c.w as f32, c.h as f32);
    let mid = Vec2::new(w / 2.0, h / 2.0);
    if panel.vdi == VdiDisplay::Off {
        return;
    }
    // The raster's glow.
    c.clear(ink(0.05));
    if panel.vdi == VdiDisplay::Standby {
        stroke_text_at(c, mid.x, mid.y, "STBY", 14.0, 1.5, ink(0.4), Align::Centre);
        return;
    }
    // Horizon and pitch lines every 10 degrees, turned by the bank.
    let per_deg = h / 50.0;
    let (sin, cos) = roll.sin_cos();
    let along = Vec2::new(cos, sin);
    let down = Vec2::new(-sin, cos);
    let at = |deg: f32| mid + down * (pitch.to_degrees() - deg) * per_deg;
    c.stroke(at(0.0) - along * w, at(0.0) + along * w, 2.0, ink(0.9));
    for deg in [-30.0, -20.0, -10.0, 10.0, 20.0, 30.0f32] {
        let p = at(deg);
        let half = if deg > 0.0 { 34.0 } else { 22.0 };
        c.stroke(p - along * half, p + along * half, 1.2, ink(0.55));
    }
    // The fixed aircraft symbol and the name.
    c.stroke(mid - Vec2::X * 26.0, mid - Vec2::X * 8.0, 2.0, ink(1.0));
    c.stroke(mid + Vec2::X * 8.0, mid + Vec2::X * 26.0, 2.0, ink(1.0));
    disc(c, mid, 2.0, ink(1.0));
    let name = if panel.vdi == VdiDisplay::TerrainClearance { "VDI TC" } else { "VDI" };
    stroke_text_at(c, mid.x, h - 26.0, name, 12.0, 1.5, ink(0.7), Align::Centre);
    if let Some(mi) = panel.range.miles() {
        stroke_text_at(c, w - 62.0, 34.0, &format!("{mi} MI"), 8.0, 1.2, ink(0.6), Align::Centre);
    }
}

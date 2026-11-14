//! The front on the war map: where the places held by each side meet. A grey band is the
//! contested ground between them, a line runs through its middle, and the places taken or lost
//! in the last hour are marked.
//!
//! The ground is split by nearest place: a point belongs to the side whose place is nearer,
//! and it is contested where the two are about as near. That field is sampled on a grid over
//! the theater; the band is a picture of it, the line its zero crossing (marching squares) so
//! that it stays sharp at any zoom. Both are made again only when a place changes hands.

use bevy_egui::egui;
use stealth_campaign::{CampaignState, Side};
use stealth_i18n::t;

use crate::map_draw::{FRIENDLY_BLUE, Layer, MISSILE_RED, MapLayers, MapTips, MapView};
use crate::menu::{BRIGHT, DIM};

/// Samples per side over the theater (0..0x8000 map units: 2 km each).
const GRID: usize = 256;
const THEATER: f32 = 0x8000 as f32;
/// Map units per km (15.625 m each).
const PER_KM: f32 = 64.0;
/// Half the contested band's width: where the nearest friendly and enemy places are within
/// this of each other (km of difference).
const BAND_KM: f32 = 14.0;
/// No front further than this from every place (open sea, empty desert).
const REACH_KM: f32 = 140.0;
/// A place taken or lost within this (campaign seconds) is marked.
pub(super) const RECENT: f64 = 3600.0;

/// The front's band, line and the key it was made for (who holds which place).
#[derive(Clone)]
struct Front {
    key: Vec<Side>,
    band: egui::TextureHandle,
    /// Line segments in map units.
    line: Vec<[egui::Vec2; 2]>,
}

/// Signed distance difference at every sample: positive on the friendly side (km), and
/// whether the sample is within reach of a place.
fn field(st: &CampaignState) -> Option<(Vec<f32>, Vec<bool>)> {
    let pts = |side: Side| -> Vec<egui::Vec2> {
        st.places
            .iter()
            .filter(|p| p.owner == side)
            .map(|p| egui::vec2(p.x as f32, p.y as f32))
            .collect()
    };
    let (blue, red) = (pts(Side::Blue), pts(Side::Red));
    if blue.is_empty() || red.is_empty() {
        return None;
    }
    let nearest = |set: &[egui::Vec2], at: egui::Vec2| {
        set.iter().map(|p| (*p - at).length_sq()).fold(f32::MAX, f32::min).sqrt()
    };
    let cell = THEATER / GRID as f32;
    let mut diff = vec![0.0; (GRID + 1) * (GRID + 1)];
    let mut near = vec![false; (GRID + 1) * (GRID + 1)];
    for j in 0..=GRID {
        for i in 0..=GRID {
            let at = egui::vec2(i as f32 * cell, j as f32 * cell);
            let (b, r) = (nearest(&blue, at), nearest(&red, at));
            diff[j * (GRID + 1) + i] = (r - b) / PER_KM;
            near[j * (GRID + 1) + i] = b.min(r) / PER_KM <= REACH_KM;
        }
    }
    Some((diff, near))
}

/// The band's picture: grey where contested, fading out at its edges.
fn band_image(diff: &[f32], near: &[bool]) -> egui::ColorImage {
    let mut img = egui::ColorImage::filled([GRID, GRID], egui::Color32::TRANSPARENT);
    for j in 0..GRID {
        for i in 0..GRID {
            let k = j * (GRID + 1) + i;
            if !near[k] {
                continue;
            }
            let d = diff[k].abs();
            if d < BAND_KM {
                // Fullest at the line, gone at the band's edge.
                let a = 1.0 - (d / BAND_KM).powi(2);
                img[(i, j)] = egui::Color32::from_rgba_unmultiplied(150, 150, 150, (a * 120.0) as u8);
            }
        }
    }
    img
}

/// The zero crossing of the field as line segments (map units), within reach of a place.
fn contour(diff: &[f32], near: &[bool]) -> Vec<[egui::Vec2; 2]> {
    let cell = THEATER / GRID as f32;
    let at = |i: usize, j: usize| egui::vec2(i as f32 * cell, j as f32 * cell);
    let v = |i: usize, j: usize| diff[j * (GRID + 1) + i];
    let mut out = Vec::new();
    for j in 0..GRID {
        for i in 0..GRID {
            if !(near[j * (GRID + 1) + i] && near[(j + 1) * (GRID + 1) + i + 1]) {
                continue;
            }
            // The cell's corners in turn and where the field crosses zero on each edge.
            let corners = [(i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)];
            let mut cross = Vec::with_capacity(4);
            for e in 0..4 {
                let (a, b) = (corners[e], corners[(e + 1) % 4]);
                let (va, vb) = (v(a.0, a.1), v(b.0, b.1));
                if (va > 0.0) != (vb > 0.0) {
                    let t = va / (va - vb);
                    cross.push(at(a.0, a.1) + (at(b.0, b.1) - at(a.0, a.1)) * t);
                }
            }
            if cross.len() >= 2 {
                out.push([cross[0], cross[1]]);
            }
            if cross.len() == 4 {
                out.push([cross[2], cross[3]]);
            }
        }
    }
    out
}

/// Who holds each place: the front changes with it.
fn key(st: &CampaignState) -> Vec<Side> {
    st.places.iter().map(|p| p.owner).collect()
}

/// Draws the front (band and line) under the places, if its layer is shown.
pub(super) fn draw(painter: &egui::Painter, view: &MapView, st: &CampaignState, layers: &MapLayers) {
    if !layers.shows(Layer::Front) {
        return;
    }
    let ctx = painter.ctx();
    let id = egui::Id::new("campaign_front");
    let now = key(st);
    let mut front = ctx.data(|d| d.get_temp::<Option<Front>>(id)).flatten();
    if front.as_ref().is_none_or(|f| f.key != now) {
        front = field(st).map(|(diff, near)| Front {
            key: now.clone(),
            band: ctx.load_texture(
                "campaign front",
                band_image(&diff, &near),
                egui::TextureOptions::LINEAR,
            ),
            line: contour(&diff, &near),
        });
        ctx.data_mut(|d| d.insert_temp(id, front.clone()));
    }
    let Some(f) = front else { return };
    let theater = egui::Rect::from_min_max(view.to_screen(0.0, 0.0), view.to_screen(THEATER, THEATER));
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    painter.image(f.band.id(), theater, uv, egui::Color32::WHITE);
    let dark = egui::Stroke::new(4.0, egui::Color32::from_black_alpha(140));
    let light = egui::Stroke::new(1.8, egui::Color32::from_gray(225));
    let segs: Vec<[egui::Pos2; 2]> =
        f.line.iter().map(|s| [view.to_screen(s[0].x, s[0].y), view.to_screen(s[1].x, s[1].y)]).collect();
    let clip = view.rect.expand(4.0);
    for stroke in [dark, light] {
        for s in segs.iter().filter(|s| clip.contains(s[0]) || clip.contains(s[1])) {
            painter.line_segment(*s, stroke);
        }
    }
}

/// Marks the places taken or lost within the last hour: a ring and TAKEN or LOST, fading with
/// the time; their tooltips say when.
pub(super) fn recent(
    painter: &egui::Painter,
    view: &MapView,
    st: &CampaignState,
    layers: &MapLayers,
    tips: &mut MapTips,
    k: f32,
) {
    if !layers.shows(Layer::Front) {
        return;
    }
    for p in &st.places {
        let Some(at) = p.captured_at else { continue };
        let age = st.clock - at;
        if !(0.0..RECENT).contains(&age) || p.owner == Side::Neutral {
            continue;
        }
        let fade = 1.0 - (age / RECENT) as f32 * 0.6;
        let won = p.owner == Side::Blue;
        let colour = if won { FRIENDLY_BLUE } else { MISSILE_RED };
        let c = view.to_screen(p.x as f32, p.y as f32);
        // A slow pulse while it is fresh.
        let t = painter.ctx().input(|i| i.time) as f32;
        let pulse = if age < RECENT / 4.0 { 1.0 + 0.15 * (t * 3.0).sin() } else { 1.0 };
        if age < RECENT / 4.0 {
            painter.ctx().request_repaint();
        }
        painter.circle_stroke(c, 16.0 * k * pulse, egui::Stroke::new(2.5, colour.gamma_multiply(fade)));
        let label = if won { t!("map-front-taken") } else { t!("map-front-lost") };
        let font = egui::FontId::monospace(12.0);
        let g = painter.layout_no_wrap(label.to_string(), font, egui::Color32::BLACK);
        let box_ = egui::Rect::from_center_size(
            c + egui::vec2(0.0, 16.0 * k + 12.0),
            g.size() + egui::vec2(10.0, 4.0),
        );
        painter.rect_filled(box_, 2.0, colour.gamma_multiply(fade));
        painter.galley(box_.center() - g.size() / 2.0, g, egui::Color32::BLACK);
        let mins = (age / 60.0).floor() as i64;
        let line = if won { t!("mt-taken-ago", mins = mins) } else { t!("mt-lost-ago", mins = mins) };
        tips.add(c, label, crate::mission::clean_text(&p.name), colour).line(line).reach(16.0 * k);
    }
}

/// The front's legend line, with its layer's checkbox.
pub(super) fn legend(ui: &mut egui::Ui, layers: &mut MapLayers) {
    layers.legend(ui, Layer::Front, t!("map-legend-front"), |p, c0| {
        let band = egui::Rect::from_center_size(c0, egui::vec2(16.0, 10.0));
        p.rect_filled(band, 0.0, egui::Color32::from_gray(150).gamma_multiply(0.5));
        p.line_segment(
            [c0 - egui::vec2(0.0, 6.0), c0 + egui::vec2(0.0, 6.0)],
            egui::Stroke::new(1.8, egui::Color32::from_gray(225)),
        );
    });
    ui.label(egui::RichText::new(t!("map-legend-front.note")).small().color(DIM));
    let _ = BRIGHT;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two places a known distance apart: the line runs halfway between them, across the
    /// line joining them.
    #[test]
    fn the_line_runs_between_the_two_sides() {
        let mut diff = vec![0.0; (GRID + 1) * (GRID + 1)];
        let near = vec![true; (GRID + 1) * (GRID + 1)];
        let cell = THEATER / GRID as f32;
        let (bx, rx) = (8000.0_f32, 16000.0_f32);
        for j in 0..=GRID {
            for i in 0..=GRID {
                let x = i as f32 * cell;
                diff[j * (GRID + 1) + i] = ((x - rx).abs() - (x - bx).abs()) / PER_KM;
            }
        }
        let line = contour(&diff, &near);
        assert!(!line.is_empty());
        for s in &line {
            for p in s {
                assert!((p.x - 12000.0).abs() < cell, "{p:?}");
            }
        }
    }
}

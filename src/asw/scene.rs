//! The sensor operator's sonar page as the menu's scene shows it (`menu_scene`): buoys in the
//! water and the dipping sonar, their waterfalls already full of a submarine's trace, a
//! ship's, a pinging buoy's pings and their echoes, and filling on.

use bevy::prelude::*;
use stealth_sim::asw::{BuoyKind, Line, SENSE_HZ, Sample};

use super::AswUi;
use super::sonar::{self, SensorRow, SonarView};
use crate::canvas::CanvasTexture;
use crate::font::{Align, stroke_text_at};

/// The sensors: id (0 the dip), kind, depth (m), the submarine's and the ship's bearing from it
/// (degrees) and how they drift (degrees a second), its distance (km).
const SENSORS: [(u16, Option<BuoyKind>, f32, [f32; 4], f32); 5] = [
    (3, Some(BuoyKind::Difar), 120.0, [62.0, 0.06, 214.0, -0.15], 2.1),
    (4, Some(BuoyKind::Difar), 120.0, [118.0, -0.05, 251.0, -0.1], 3.4),
    (5, Some(BuoyKind::Dicass), 120.0, [88.0, 0.03, 236.0, -0.12], 2.8),
    (6, Some(BuoyKind::Bt), 0.0, [0.0; 4], 4.0),
    (0, None, 90.0, [79.0, 0.04, 228.0, -0.13], 0.0),
];
/// The pinging buoy pings every so many seconds.
const PING_EVERY: f32 = 10.0;

/// The scene's sonar: the page's state and the clock of its reports.
pub struct SonarScene {
    pub ui: AswUi,
    t: f32,
    seq: u16,
}

impl SonarScene {
    /// The waterfalls already full (a picture's worth of reports).
    pub fn new(seed: u32) -> Self {
        let mut ui = AswUi { sensor: Some(3), ..Default::default() };
        for &(id, kind, ..) in &SENSORS {
            if kind != Some(BuoyKind::Bt) {
                let w =
                    sonar::Waterfall { rng: seed | 1 | ((id as u32) << 20), ..Default::default() };
                ui.falls.insert(id, w);
            }
        }
        let mut s = SonarScene { ui, t: -110.0, seq: 0 };
        while s.t < 0.0 {
            s.report();
        }
        s.ui.cursor.insert(3, 64.0);
        s
    }

    /// One report of every sensor (`SENSE_HZ` a second).
    fn report(&mut self) {
        self.t += 1.0 / SENSE_HZ as f32;
        self.seq = self.seq.wrapping_add(1);
        let t = self.t;
        let ping = (t.rem_euclid(PING_EVERY)) < 1.0 / SENSE_HZ as f32;
        for &(id, kind, _, [sub, sub_drift, ship, ship_drift], _) in &SENSORS {
            if kind == Some(BuoyKind::Bt) {
                continue;
            }
            let wobble = |k: f32| (t * k).sin() * 2.0;
            let bearing = |b: f32, d: f32| (b + d * t).rem_euclid(360.0) as u16;
            let mut lines = vec![
                Line {
                    snr: (6.0 + wobble(0.11) * 2.0) as i8,
                    bearing: Some(bearing(sub + wobble(0.3) * 0.3, sub_drift)),
                    freqs: [50, 300],
                    ..Default::default()
                },
                Line {
                    snr: 14,
                    bearing: Some(bearing(ship, ship_drift)),
                    freqs: [120, 0],
                    ship: true,
                    ..Default::default()
                },
            ];
            // The pinging buoy's pings, heard by all; their echo off the submarine.
            if kind == Some(BuoyKind::Dicass) || id == 0 {
                let since = t.rem_euclid(PING_EVERY);
                if since > 1.6 && since < 1.6 + 1.0 / SENSE_HZ as f32 {
                    lines.push(Line {
                        snr: 18,
                        bearing: Some(bearing(sub, sub_drift)),
                        range: Some(2400),
                        doppler: 3,
                        echo: true,
                        ..Default::default()
                    });
                }
            }
            let s = Sample {
                seq: self.seq,
                noise: 52,
                lines,
                ping: ping && kind == Some(BuoyKind::Dicass),
                heard: ping && kind != Some(BuoyKind::Dicass),
                due: Vec::new(),
            };
            if let Some(w) = self.ui.falls.get_mut(&id) {
                w.push(&s, t);
            }
        }
    }

    /// The clock on `dt` seconds: new reports into the waterfalls.
    pub fn step(&mut self, dt: f32) {
        let end = self.t + dt;
        while self.t + 1.0 / SENSE_HZ as f32 <= end {
            self.report();
        }
        self.ui.t += dt;
    }

    /// Draws the page on the console's screen (`colours`: the console's palette).
    pub fn draw(&mut self, tex: &mut CanvasTexture, colours: std::sync::Arc<[u32; 256]>) {
        let rows = SENSORS
            .iter()
            .map(|&(id, kind, depth, _, km)| SensorRow {
                id,
                kind,
                depth,
                active: kind == Some(BuoyKind::Dicass),
                pings: 6,
                km: (id != 0).then_some(km),
            })
            .collect();
        let view = SonarView { layer: Some(85), hover: true, buoys: 4, rows };
        let prev = tex.backdrop.take();
        tex.backdrop = sonar::paint(&mut tex.canvas, &mut self.ui, &view, prev);
        let cv = &mut tex.canvas;
        for (k, name) in ["SONAR", "WPN", "MAP"].into_iter().enumerate() {
            let x = 160.0 + k as f32 * 160.0;
            let col =
                if k == 0 { crate::station::scope::YELLOW } else { crate::station::scope::GREY };
            stroke_text_at(cv, x, 472.0, name, 9.0, 1.2, col, Align::Centre);
        }
        tex.transparent = false;
        tex.colours = Some(colours);
        tex.dirty = true;
    }
}

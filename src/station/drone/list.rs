//! The drone list (the screen top right): the drones in the air with their batteries, how long
//! each can still fly, the stock and the rail.

use bevy::prelude::*;
use stealth_sim::uav::{MAX_AIRBORNE, UavMode};

use super::super::scope::{BLACK, CYAN, DARK, GREEN, GREY, RED, WHITE, YELLOW};
use super::{DroneUi, center_uavs};
use crate::canvas::CanvasTexture;
use crate::cockpit3d::{ConsoleCanvas, ConsoleScreen, INFO_CANVAS};
use crate::combat::Combat;
use crate::font::{Align, stroke_text_at};
use crate::station::Manning;

/// Redraws a second.
const RATE: f32 = 4.0;
const ROW: f32 = 38.0;

pub(super) fn draw(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    ui: Res<DroneUi>,
    time: Res<Time<Real>>,
    mut since: Local<f32>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTexture)>,
) {
    let (Some(m), Some(combat)) = (manning, combat) else { return };
    if !m.drones() {
        return;
    }
    *since += time.delta_secs();
    if *since < 1.0 / RATE && !ui.is_changed() {
        return;
    }
    *since = 0.0;
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Info) else {
        return;
    };
    tex.dirty = true;
    let cv = &mut tex.canvas;
    cv.reset_clip();
    cv.clear(BLACK);
    let (w, h) = (INFO_CANVAS.0 as f32, INFO_CANVAS.1 as f32);
    let sim = &combat.sim;
    let uavs = center_uavs(&combat, &m);
    let title = format!("DRONES AIRBORNE {}/{MAX_AIRBORNE}", uavs.len());
    stroke_text_at(cv, 8.0, 12.0, &title, 7.0, 1.0, WHITE, Align::Left);
    cv.stroke(Vec2::new(6.0, 22.0), Vec2::new(w - 6.0, 22.0), 1.0, DARK);
    for (i, u) in uavs.iter().enumerate() {
        let y = 30.0 + i as f32 * ROW;
        let chosen = ui.selected == Some(u.id);
        let col = if chosen { WHITE } else { CYAN };
        stroke_text_at(cv, 8.0, y + 6.0, &u.label(), 7.0, 1.0, col, Align::Left);
        let (mode, mcol) = match u.mode {
            UavMode::Manual => ("MANUAL", YELLOW),
            UavMode::Dive => ("DIVE", RED),
            m => (m.name(), GREEN),
        };
        stroke_text_at(cv, w - 8.0, y + 6.0, mode, 6.0, 0.9, mcol, Align::Right);
        // The battery: a bar, the charge, the time left.
        let charge = u.battery.clamp(0.0, 1.0);
        let bcol = if charge < 0.2 {
            RED
        } else if charge < 0.4 {
            YELLOW
        } else {
            GREEN
        };
        let (x0, x1, by) = (8.0, 190.0, y + 18.0);
        cv.frame(Vec2::new(x0, by - 5.0), Vec2::new(x1, by + 5.0), 1.0, GREY);
        let fill = x0 + 2.0 + (x1 - x0 - 4.0) * charge;
        if fill > x0 + 2.0 {
            cv.fill_rect(x0 as i32 + 2, by as i32 - 3, (fill - x0 - 2.0) as i32, 7, bcol);
        }
        let end = u.endurance() as i32;
        let text = format!("{:.0}% {}:{:02}", charge * 100.0, end / 3600, end / 60 % 60);
        stroke_text_at(cv, w - 8.0, by, &text, 6.0, 0.9, bcol, Align::Right);
    }
    if uavs.is_empty() {
        stroke_text_at(cv, w / 2.0, 70.0, "NONE", 7.0, 1.0, GREY, Align::Centre);
    }
    let [stork, fp2] = sim.uav_stock(m.unit);
    let wait = sim.uav_rail_wait(m.unit);
    let rail = if wait > 0 { format!("RAIL {wait} S") } else { "RAIL READY".into() };
    let stock = format!("STOCK: STORK {stork}  FP-2 {fp2}  {rail}");
    let col = if stork + fp2 > 0 { GREY } else { RED };
    stroke_text_at(cv, 8.0, h - 10.0, &stock, 6.0, 0.9, col, Align::Left);
}

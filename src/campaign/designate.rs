//! New air defence sites on the WAR MAP and AIR DEFENSE maps: the buttons that start a
//! designation, the spot under the pointer (allowed or why not), the click that designates it,
//! and the sites waiting for their cargo.

use bevy::math::DVec2;
use bevy_egui::egui;
use stealth_campaign::{CampaignState, DefenceKind, footprint};
use stealth_sim::Sim;
use stealth_sim::aaa::M_PER_MAP;

use super::{Action, ActiveCampaign};
use crate::map_draw::{FRIENDLY_BLUE, MISSILE_RED, MapView};
use crate::menu::{BRIGHT, DIM, GREEN, section};
use crate::scenery::Scenery;

/// Metres of open ground (no mountain, no theater object) the helipad and the unit need.
const GROUND_ROOM: f64 = 60.0;

/// The GEPARD / PATRIOT / DRONES new site buttons (the one active again: cancel).
pub(super) fn buttons(ui: &mut egui::Ui, c: &mut ActiveCampaign) {
    section(ui, "New site");
    // Framed buttons sharing the column's width, the one being designated lit.
    ui.horizontal(|ui| {
        let w = (ui.available_width() - ui.spacing().item_spacing.x * 2.0) / 3.0;
        let kinds = [
            (DefenceKind::Gepard, "GEPARD"),
            (DefenceKind::Patriot, "PATRIOT"),
            (DefenceKind::Drones, "DRONES"),
        ];
        for (kind, label) in kinds {
            let on = c.ui.designate == Some(kind);
            let button = egui::Button::new(label).min_size(egui::vec2(w, 30.0)).selected(on);
            if ui.add(button).clicked() {
                c.ui.designate = if on { None } else { Some(kind) };
                c.ui.error = None;
            }
        }
    });
    let hint = match c.ui.designate {
        Some(_) => "Click the map where it goes - right click: cancel",
        None => "Designate a spot: UH-60s fly its cargo to a helipad there within 24 h",
    };
    ui.label(egui::RichText::new(hint).small().color(DIM));
}

/// The map point (x east, y south) in metres east, north.
fn metres(x: i32, y: i32) -> DVec2 {
    DVec2::new((x as f64 + 0.5) * M_PER_MAP, (0x8000 as f64 - y as f64 - 0.5) * M_PER_MAP)
}

/// Why a new site may not go to map point (x, y), None if it may: the campaign's rules, and
/// open ground (the theater's mountains and objects stay where they are).
pub(super) fn refusal(
    st: &CampaignState,
    sim: &Sim,
    scenery: Option<&Scenery>,
    x: i32,
    y: i32,
    kind: DefenceKind,
) -> Option<String> {
    let aaa = stealth_host::glue::world_view(sim).units;
    if let Some(why) = st.deploy_refusal(x, y, kind, &aaa) {
        return Some(why);
    }
    if scenery.is_some_and(|s| !s.is_open(metres(x, y), GROUND_ROOM)) {
        return Some("Not on open ground".into());
    }
    None
}

/// While a designation is under way: the site's ground under the pointer (green: it may go
/// there; red with the reason), and a click designates it. True if it took the map's click.
pub(super) fn pointer(
    ui: &egui::Ui,
    painter: &egui::Painter,
    view: &MapView,
    resp: &egui::Response,
    c: &mut ActiveCampaign,
    sim: &Sim,
    scenery: Option<&Scenery>,
) -> bool {
    let Some(kind) = c.ui.designate else { return false };
    if resp.secondary_clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        c.ui.designate = None;
        return true;
    }
    let Some(pos) = resp.hover_pos() else { return false };
    let m = view.to_map(pos);
    let (x, y) = (m.x.round() as i32, m.y.round() as i32);
    let why = refusal(c.state(), sim, scenery, x, y, kind);
    let col = if why.is_none() { GREEN } else { MISSILE_RED };
    let r = (footprint(kind) as f32 / view.scale).max(4.0);
    painter.circle(pos, r, col.gamma_multiply(0.12), egui::Stroke::new(1.5, col));
    pad(painter, pos, col);
    let what = match kind {
        DefenceKind::Gepard => "Gepard site",
        DefenceKind::Patriot => "Patriot site",
        DefenceKind::Drones => "drone command center",
    };
    let text = why.clone().unwrap_or_else(|| format!("New {what} - click to designate"));
    painter.text(
        pos + egui::vec2(12.0, -12.0),
        egui::Align2::LEFT_BOTTOM,
        text,
        egui::FontId::monospace(12.0),
        if why.is_none() { BRIGHT } else { MISSILE_RED },
    );
    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    if resp.clicked() && why.is_none() {
        c.actions.push(Action::DesignateSite { x, y, kind });
        c.ui.designate = None;
        c.ui.error = None;
    }
    resp.clicked()
}

/// Test aid: `--campaign-designate gepard|patriot|drones` designates a site once, at the first spot
/// round the parked base where one may go.
pub(super) fn test_aid(c: &mut ActiveCampaign, sim: &Sim, scenery: Option<&Scenery>) {
    let Some(arg) = std::env::args().skip_while(|a| a != "--campaign-designate").nth(1) else {
        return;
    };
    if std::mem::replace(&mut c.ui.test_designated, true) {
        return;
    }
    let kind = match arg.as_str() {
        "patriot" => DefenceKind::Patriot,
        "drones" => DefenceKind::Drones,
        _ => DefenceKind::Gepard,
    };
    let st = c.state();
    let Some(b) = c.parked.and_then(|b| st.site(b as u16)).map(|s| (s.x, s.y)) else { return };
    let spot = (1..60)
        .flat_map(|r| (0..16).map(move |k| (r, k)))
        .map(|(r, k)| {
            let t = k as f64 / 16.0 * std::f64::consts::TAU;
            (b.0 + (t.cos() * r as f64 * 32.0) as i32, b.1 + (t.sin() * r as f64 * 32.0) as i32)
        })
        .find(|&(x, y)| refusal(st, sim, scenery, x, y, kind).is_none());
    if let Some((x, y)) = spot {
        c.actions.push(Action::DesignateSite { x, y, kind });
    }
}

/// A helipad symbol: a square with an H.
fn pad(painter: &egui::Painter, p: egui::Pos2, col: egui::Color32) {
    let r = egui::Rect::from_center_size(p, egui::vec2(11.0, 11.0));
    painter.rect_stroke(r, 0.0, egui::Stroke::new(1.5, col), egui::StrokeKind::Middle);
    let s = egui::Stroke::new(1.2, col);
    painter.line_segment([p + egui::vec2(-2.5, -3.0), p + egui::vec2(-2.5, 3.0)], s);
    painter.line_segment([p + egui::vec2(2.5, -3.0), p + egui::vec2(2.5, 3.0)], s);
    painter.line_segment([p + egui::vec2(-2.5, 0.0), p + egui::vec2(2.5, 0.0)], s);
}

/// The sites waiting for their cargo: the helipad, loads in and time left.
pub(super) fn waiting(painter: &egui::Painter, view: &MapView, st: &CampaignState) {
    for d in st.deployments.iter().filter(|d| !d.built) {
        let p = view.to_screen(d.x as f32, d.y as f32);
        pad(painter, p, FRIENDLY_BLUE);
        let left = (d.expires - st.clock).max(0.0) as i32;
        let text = format!(
            "{} {}/{} {}:{:02}",
            match d.kind {
                DefenceKind::Gepard => "GEPARD",
                DefenceKind::Patriot => "PATRIOT",
                DefenceKind::Drones => "DRONES",
            },
            d.delivered.len(),
            d.loads,
            left / 3600,
            left / 60 % 60
        );
        painter.text(
            p + egui::vec2(9.0, 6.0),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::monospace(11.0),
            FRIENDLY_BLUE,
        );
    }
}

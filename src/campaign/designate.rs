//! New air defence sites on the WAR MAP and AIR DEFENSE maps: the buttons that start a
//! designation, the spot under the pointer (allowed or why not), the click that designates it,
//! and the sites waiting for their cargo.

use bevy::math::DVec2;
use bevy_egui::egui;
use stealth_campaign::{CampaignState, DefenceKind, fast_track_cost, footprint};
use stealth_i18n::t;
use stealth_sim::Sim;
use stealth_sim::aaa::M_PER_MAP;

use super::{Action, ActiveCampaign};
use crate::map_draw::{FRIENDLY_BLUE, MISSILE_RED, MapView};
use crate::menu::{BRIGHT, DIM, GREEN, Panel, big_button, icon, section};
use crate::scenery::Scenery;

/// Metres of open ground (no mountain, no theater object) the helipad and the unit need.
const GROUND_ROOM: f64 = 60.0;

/// The kinds of new site, in the buttons' order.
const KINDS: [DefenceKind; 4] =
    [DefenceKind::Gepard, DefenceKind::Nasams, DefenceKind::Patriot, DefenceKind::Drones];

/// The GEPARD / NASAMS / PATRIOT / DRONES new site buttons (the one active again: cancel).
pub(super) fn buttons(ui: &mut egui::Ui, c: &mut ActiveCampaign) {
    section(ui, t!("ds-new-site"));
    // Framed buttons sharing the column's width, the one being designated lit: all in a row,
    // two rows of two where the column is too narrow for that, one under the other where even
    // two side by side do not fit their labels.
    let gap = ui.spacing().item_spacing.x;
    let font = egui::TextStyle::Button.resolve(ui.style());
    let pad = ui.spacing().button_padding.x * 2.0 + 4.0;
    let widest = KINDS
        .iter()
        .map(|&kind| {
            let label = kind_label(kind).to_string();
            ui.painter().layout_no_wrap(label, font.clone(), BRIGHT).size().x + pad
        })
        .fold(0.0, f32::max);
    // What shows of the column (the war map's side column reaches under the window's edge
    // in a small window).
    let full = ui.available_width().min(ui.clip_rect().right() - ui.cursor().left() - 2.0);
    let per_row = [KINDS.len(), 2, 1]
        .into_iter()
        .find(|&n| widest <= (full - gap * (n - 1) as f32) / n as f32)
        .unwrap_or(1);
    let w = (full - gap * (per_row - 1) as f32) / per_row as f32;
    for row in KINDS.chunks(per_row) {
        ui.horizontal(|ui| {
            for &kind in row {
                let on = c.ui.designate == Some(kind);
                let button =
                    egui::Button::new(kind_label(kind)).min_size(egui::vec2(w, 48.0)).selected(on);
                if ui.add(button).clicked() {
                    c.ui.designate = if on { None } else { Some(kind) };
                    c.ui.error = None;
                }
            }
        });
    }
    let hint = match c.ui.designate {
        Some(_) => t!("ds-click-hint"),
        None => t!("ds-hint"),
    };
    ui.label(egui::RichText::new(hint).small().color(DIM));
    fast_track_buttons(ui, c);
}

/// "GEPARD" (the map's and the buttons' name of a site's kind).
pub(super) fn kind_label(kind: DefenceKind) -> &'static str {
    match kind {
        DefenceKind::Gepard => "GEPARD",
        DefenceKind::Patriot => "PATRIOT",
        DefenceKind::Drones => t!("ad-drones"),
        DefenceKind::Nasams => "NASAMS",
    }
}

/// The sites waiting for their cargo, each with a FAST-TRACK button: built at once for
/// materiel (after a confirmation, `confirm`).
fn fast_track_buttons(ui: &mut egui::Ui, c: &mut ActiveCampaign) {
    let st = c.state();
    let waiting: Vec<(u32, String, f64)> = st
        .deployments
        .iter()
        .filter(|d| !d.built)
        .map(|d| {
            let what =
                format!("{} {}/{} - {}", kind_label(d.kind), d.delivered.len(), d.loads, d.name);
            (d.id, what, fast_track_cost(d))
        })
        .collect();
    if waiting.is_empty() {
        return;
    }
    let materiel = st.blue.materiel;
    let over = st.outcome.is_some();
    ui.add_space(6.0);
    ui.label(egui::RichText::new(t!("ds-waiting")).small().color(DIM));
    for (id, what, cost) in waiting {
        ui.label(egui::RichText::new(what).small().color(FRIENDLY_BLUE));
        let label = t!("ds-fast-track", cost = cost.round() as i64);
        let button = egui::Button::new(egui::RichText::new(label).color(FRIENDLY_BLUE));
        if ui
            .add_enabled(!over && materiel >= cost, button)
            .on_hover_text(t!("ds-fast-track.facts"))
            .on_disabled_hover_text(t!("ds-fast-track.disabled"))
            .clicked()
        {
            c.ui.fast_track = Some(id);
            c.ui.error = None;
        }
    }
}

/// Fast-tracking a site asks first: the materiel it costs.
pub(super) fn confirm(ctx: &egui::Context, c: &mut ActiveCampaign) {
    let Some(id) = c.ui.fast_track else { return };
    let Some(d) = c.state().deployment(id).filter(|d| !d.built) else {
        c.ui.fast_track = None;
        return;
    };
    let (cost, site) = (fast_track_cost(d).round() as i64, d.label().to_string());
    let window = egui::Window::new("Fast-track construction")
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .order(egui::Order::Foreground);
    let text = t!("ds-fast-track-confirm.text", cost = cost, site = site);
    let mut close = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    Panel::new(t!("ds-fast-track-confirm")).window(window, ctx, |ui| {
        ui.label(egui::RichText::new(text).color(DIM));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let spend = t!("ds-fast-track-spend", cost = cost);
            let spend = big_button(&spend).icon(icon::CONTINUE);
            if ui.add(spend).clicked() {
                c.actions.push(Action::FastTrackSite { id });
                c.ui.error = None;
                close = true;
            }
            if ui.add(big_button(t!("cancel")).icon(icon::CANCEL)).clicked() {
                close = true;
            }
        });
    });
    if close {
        c.ui.fast_track = None;
    }
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
        return Some(t!("ds-not-open-ground").into());
    }
    // (A coop client's campaign has no terrain: the sim's hills tell it.)
    let at = metres(x, y);
    if sim.hills().slope_within(at.x, at.y, GROUND_ROOM) > stealth_sim::hills::SITE_SLOPE {
        return Some(t!("ds-too-steep").into());
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
    let text = why.clone().unwrap_or_else(|| {
        t!(match kind {
            DefenceKind::Gepard => "ds-new-gepard",
            DefenceKind::Patriot => "ds-new-patriot",
            DefenceKind::Drones => "ds-new-drones",
            DefenceKind::Nasams => "ds-new-nasams",
        })
        .to_string()
    });
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

/// Test aid: `--campaign-designate gepard|patriot|nasams|drones` designates a site once, at the
/// first spot round the parked base where one may go (with a `+built` suffix, `nasams+built`,
/// it is fast-tracked as soon as it waits).
pub(super) fn test_aid(c: &mut ActiveCampaign, sim: &Sim, scenery: Option<&Scenery>) {
    let Some(arg) = std::env::args().skip_while(|a| a != "--campaign-designate").nth(1) else {
        return;
    };
    let (arg, built) = match arg.strip_suffix("+built") {
        Some(a) => (a.to_string(), true),
        None => (arg, false),
    };
    if built && !c.ui.test_fast_tracked {
        let id = c.state().deployments.iter().find(|d| !d.built).map(|d| d.id);
        if let Some(id) = id {
            c.ui.test_fast_tracked = true;
            c.actions.push(Action::FastTrackSite { id });
        }
    }
    if std::mem::replace(&mut c.ui.test_designated, true) {
        return;
    }
    let kind = match arg.as_str() {
        "patriot" => DefenceKind::Patriot,
        "drones" => DefenceKind::Drones,
        "nasams" => DefenceKind::Nasams,
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
            kind_label(d.kind),
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

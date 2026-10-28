//! The Ops screen's AIR DEFENSE tab: the friendly Patriot batteries and Gepards on the war map
//! with their reach, and beside it a list of them: their ammunition, the enemy aircraft their
//! radars see, who sits at the console, and a button to man it.

use bevy_egui::egui;
use stealth_campaign::Side;
use stealth_sim::aaa::{AaaKind, M_PER_MAP};
use stealth_sim::station::{GEPARD_REACH, GEPARD_ROUNDS, StationCmd, mannable};
use stealth_sim::{Sim, fflags};

use super::ops::{fit_map, man_refusal};
use super::{Action, ActiveCampaign};
use crate::map_draw::{self, FRIENDLY_BLUE, MISSILE_RED, MapView};
use crate::menu::{AMBER, BRIGHT, DIM, GREEN, section};

/// The list's width (points).
const LIST_W: f32 = 430.0;
/// Patriot and Gepard reach on the map (map units).
const PATRIOT_REACH: f32 = stealth_sim::campaign::PATRIOT_REACH as f32;
const GEPARD_MAP_REACH: f32 = (GEPARD_REACH / M_PER_MAP) as f32;

/// A friendly air defence site a player can man.
struct Site {
    unit: usize,
    x: i32,
    y: i32,
    gepard: bool,
    /// The nearest place's name.
    near: String,
    destroyed: bool,
    /// The player at its console.
    manned: Option<u8>,
    /// Rounds ready (missiles on the launchers or the Gepard's shells), the most it holds,
    /// its stockpile, launchers reloading, launchers left (Patriot).
    ready: i32,
    full: i32,
    stock: i32,
    reloading: usize,
    launchers: usize,
    /// Enemy aircraft its radar sees: type and range (km), nearest first.
    enemies: Vec<(String, f32)>,
}

fn sites(c: &ActiveCampaign, sim: &Sim) -> Vec<Site> {
    let st = c.state();
    let mut out: Vec<Site> = (0..sim.aaa_units().len())
        .filter(|&k| sim.aaa_blue(k) && mannable(sim.aaa_units()[k].kind))
        .map(|k| {
            let u = sim.aaa_units()[k];
            let gepard = u.kind == AaaKind::Gepard;
            let near = st
                .places
                .iter()
                .min_by_key(|p| stealth_campaign::dist(p.x - u.x, p.y - u.y))
                .map_or_else(String::new, |p| p.name.clone());
            let units = if gepard { vec![k] } else { sim.battery_launchers(k) };
            let alive: Vec<usize> =
                units.iter().copied().filter(|&l| !sim.aaa_units()[l].destroyed).collect();
            let per = if gepard { GEPARD_ROUNDS } else { stealth_sim::campaign::PATRIOT_ROUNDS };
            let at = u.metres();
            let mut enemies: Vec<(String, f32)> = sim
                .flights()
                .iter()
                .enumerate()
                .filter(|&(i, f)| {
                    let up = fflags::EXISTS | fflags::AIRBORNE;
                    f.flags & up == up
                        && f.flags & fflags::DESTROYED == 0
                        && f.speed != 0
                        && !sim.friendly_unit(i, f.spec)
                        && sim.campaign_flight(i).is_some()
                })
                .filter_map(|(i, f)| {
                    let (p, _) = sim.track_alive(i)?;
                    let d = (p[0] - at[0]).hypot(p[1] - at[1]);
                    (d <= crate::station::RADAR_REACH)
                        .then(|| (sim.aircraft_name(f.spec).to_string(), (d / 1000.0) as f32))
                })
                .collect();
            enemies.sort_by(|a, b| a.1.total_cmp(&b.1));
            Site {
                unit: k,
                x: u.x,
                y: u.y,
                gepard,
                near,
                destroyed: u.destroyed,
                manned: sim.manned_by(k),
                ready: alive.iter().map(|&l| sim.aaa_units()[l].rounds).sum(),
                full: alive.len() as i32 * per,
                stock: sim.stockpile(k),
                reloading: units.iter().filter(|&&l| sim.refill_left(l) > 0.0).count(),
                launchers: alive.len(),
                enemies,
            }
        })
        .collect();
    // Patriot batteries first, then the Gepards, each west to east.
    out.sort_by_key(|s| (s.gepard, s.x));
    out
}

/// The site's short name: P1, P2 .. for the batteries, G1 .. for the Gepards.
fn tag(sites: &[Site], i: usize) -> String {
    let s = &sites[i];
    let n = sites[..=i].iter().filter(|o| o.gepard == s.gepard).count();
    format!("{}{n}", if s.gepard { "G" } else { "P" })
}

/// The AIR DEFENSE tab.
pub(super) fn tab(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    sim: &Sim,
    tex: Option<&egui::TextureHandle>,
    scenery: Option<&crate::scenery::Scenery>,
    size: egui::Vec2,
) {
    let sites = sites(c, sim);
    if c.ui.defence.is_some_and(|u| !sites.iter().any(|s| s.unit == u)) {
        c.ui.defence = None;
    }
    ui.horizontal_top(|ui| {
        map(ui, c, sim, &sites, tex, scenery, egui::vec2(size.x - LIST_W, size.y));
        ui.vertical(|ui| {
            ui.set_width(LIST_W - 12.0);
            super::designate::buttons(ui, c);
            ui.add_space(6.0);
            list(ui, c, sim, &sites, size.y - 80.0);
        });
    });
}

/// The war map, dimmed, with the sites and their reach on it and the enemy aircraft the
/// friendly side sees. A click on a site selects it.
fn map(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    sim: &Sim,
    sites: &[Site],
    tex: Option<&egui::TextureHandle>,
    scenery: Option<&crate::scenery::Scenery>,
    size: egui::Vec2,
) {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    fit_map(c, rect);
    let view = MapView { rect, centre: c.ui.centre, scale: c.ui.scale };
    let (mut centre, mut scale) = (c.ui.centre, c.ui.scale);
    view.pan_zoom(ui, &resp, &mut centre, &mut scale);
    (c.ui.centre, c.ui.scale) = (centre, scale);
    let painter = ui.painter_at(rect);
    map_draw::picture(&painter, &view, tex);
    painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(140));
    let at = |x: i32, y: i32| view.to_screen(x as f32, y as f32);
    let st = c.state();
    for p in &st.places {
        let col = match p.owner {
            Side::Blue => FRIENDLY_BLUE,
            Side::Red => MISSILE_RED,
            Side::Neutral => DIM,
        }
        .gamma_multiply(0.5);
        painter.circle_filled(at(p.x, p.y), 2.5, col);
    }
    // The reach: filled, the selected site's brighter; then the symbols over all of them.
    for s in sites.iter().filter(|s| !s.destroyed) {
        let reach = if s.gepard { GEPARD_MAP_REACH } else { PATRIOT_REACH };
        let r = (reach / view.scale).max(4.0);
        let chosen = c.ui.defence == Some(s.unit);
        let (fill, stroke) = if chosen { (0.16, 2.0) } else { (0.07, 1.2) };
        painter.circle(
            at(s.x, s.y),
            r,
            FRIENDLY_BLUE.gamma_multiply(fill),
            egui::Stroke::new(stroke, if chosen { BRIGHT } else { FRIENDLY_BLUE }),
        );
    }
    let pic = super::picture::picture(st, sim);
    for a in &pic.fighters {
        super::picture::aircraft(&painter, at(a.x, a.y), a, MISSILE_RED);
    }
    for a in &pic.friendly {
        super::picture::aircraft(&painter, at(a.x, a.y), a, FRIENDLY_BLUE.gamma_multiply(0.6));
    }
    let mut hovered = None;
    let mut named = Vec::new();
    for (i, s) in sites.iter().enumerate() {
        let p = at(s.x, s.y);
        let kind = if s.gepard { AaaKind::Gepard } else { AaaKind::PatriotRadar };
        if s.destroyed {
            painter.circle_stroke(p, 5.0, egui::Stroke::new(1.5, DIM));
        } else {
            super::picture::defence(&painter, p, kind);
        }
        if s.manned.is_some() {
            painter.circle_stroke(p, 9.0, egui::Stroke::new(1.5, GREEN));
        }
        // Named unless another's name is there already (crowded sites: the list has them).
        if !named.iter().any(|q: &egui::Pos2| q.distance(p) < 16.0) {
            named.push(p);
            let col = if s.destroyed { DIM } else { BRIGHT };
            painter.text(
                p + egui::vec2(8.0, -6.0),
                egui::Align2::LEFT_BOTTOM,
                tag(sites, i),
                egui::FontId::monospace(11.0),
                col,
            );
        }
        if resp.hover_pos().is_some_and(|h| h.distance(p) < 10.0) {
            hovered = Some(s.unit);
        }
    }
    super::designate::waiting(&painter, &view, c.state());
    let designating = c.ui.designate.is_some();
    super::designate::pointer(ui, &painter, &view, &resp, c, sim, scenery);
    if let Some(u) = hovered.filter(|_| !designating) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        if resp.clicked() {
            c.ui.defence = Some(u);
            c.ui.defence_scroll = true;
        }
    }
    painter.text(
        rect.left_bottom() + egui::vec2(8.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        "Click a site to select it - wheel: zoom, drag: pan",
        egui::FontId::monospace(11.0),
        DIM,
    );
}

/// The sites one under the other: what each has and sees, and the way in.
fn list(ui: &mut egui::Ui, c: &mut ActiveCampaign, sim: &Sim, sites: &[Site], height: f32) {
    section(ui, "Air defense sites");
    if sites.is_empty() {
        ui.label(egui::RichText::new("No Patriot battery or Gepard on our side").color(DIM));
        return;
    }
    egui::ScrollArea::vertical().id_salt("air_defence").max_height(height - 30.0).show(ui, |ui| {
        for (i, s) in sites.iter().enumerate() {
            let chosen = c.ui.defence == Some(s.unit);
            let frame = egui::Frame::group(ui.style())
                .stroke(egui::Stroke::new(
                    if chosen { 2.0 } else { 1.0 },
                    if chosen { BRIGHT } else { DIM.gamma_multiply(0.5) },
                ))
                .inner_margin(6.0);
            let r = frame.show(ui, |ui| {
                ui.set_width(LIST_W - 40.0);
                entry(ui, c, sim, s, &tag(sites, i));
            });
            // Not `interact(click)`: the frame's response sits over the ENTER button and
            // would take its clicks.
            let r = r.response;
            if r.contains_pointer() && ui.input(|i| i.pointer.primary_clicked()) {
                c.ui.defence = Some(s.unit);
            }
            if chosen && std::mem::take(&mut c.ui.defence_scroll) {
                r.scroll_to_me(Some(egui::Align::Center));
            }
            ui.add_space(4.0);
        }
    });
}

fn entry(ui: &mut egui::Ui, c: &mut ActiveCampaign, sim: &Sim, s: &Site, tag: &str) {
    ui.horizontal(|ui| {
        let kind = if s.gepard { "GEPARD" } else { "PATRIOT" };
        ui.label(egui::RichText::new(format!("{tag} {kind}")).strong().color(FRIENDLY_BLUE));
        ui.label(egui::RichText::new(format!("near {}", s.near)).color(DIM));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let why = man_refusal(c, sim, s.unit);
            let button = egui::Button::new("ENTER").min_size(egui::vec2(80.0, 24.0));
            let r = ui.add_enabled(why.is_none(), button);
            if r.clicked() {
                c.ui.error = None;
                c.actions.push(Action::Station(StationCmd::Man { unit: s.unit as u16 }));
            }
            if let Some(why) = why {
                r.on_disabled_hover_text(why);
            }
        });
    });
    if s.destroyed {
        ui.label(egui::RichText::new("DESTROYED").color(MISSILE_RED));
        return;
    }
    let ammo_col = if s.ready == 0 {
        MISSILE_RED
    } else if s.ready * 2 < s.full {
        AMBER
    } else {
        GREEN
    };
    ui.horizontal(|ui| {
        let ammo = if s.gepard {
            format!("{} / {} rds", s.ready, s.full)
        } else {
            format!("{} / {} msl on {} launchers", s.ready, s.full, s.launchers)
        };
        ui.label(egui::RichText::new(ammo).monospace().color(ammo_col));
        let unit = if s.gepard { "rds" } else { "msl" };
        ui.label(
            egui::RichText::new(format!("- stock {} {unit}", s.stock))
                .monospace()
                .color(if s.stock > 0 { BRIGHT } else { MISSILE_RED }),
        );
        if s.reloading > 0 {
            ui.label(egui::RichText::new("RELOADING").monospace().color(AMBER));
        }
    });
    let operator = match s.manned {
        Some(p) if p == c.me.0 => "You".to_string(),
        Some(p) => {
            c.state().player_name(stealth_campaign::PlayerId(p)).unwrap_or("A player").to_string()
        }
        None => "Unmanned (automatic)".to_string(),
    };
    ui.label(egui::RichText::new(format!("Operator: {operator}")).color(if s.manned.is_some() {
        GREEN
    } else {
        DIM
    }));
    if s.enemies.is_empty() {
        ui.label(egui::RichText::new("No enemy aircraft on radar").color(DIM));
    } else {
        let shown: Vec<String> =
            s.enemies.iter().take(3).map(|(n, km)| format!("{n} {km:.0} km")).collect();
        let more = s.enemies.len().saturating_sub(3);
        let more = if more > 0 { format!(" +{more}") } else { String::new() };
        ui.label(
            egui::RichText::new(format!(
                "{} hostile on radar: {}{more}",
                s.enemies.len(),
                shown.join(", ")
            ))
            .color(MISSILE_RED),
        );
    }
}

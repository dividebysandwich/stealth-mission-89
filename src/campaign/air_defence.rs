//! The Ops screen's AIR DEFENSE tab: the friendly Patriot batteries, Gepards and drone command
//! centers on the war map with their reach, and beside it a list of them: their ammunition (a
//! command center's drones), the enemy aircraft their radars see, who sits at the console, and a
//! button to man it.

use bevy_egui::egui;
use stealth_campaign::Side;
use stealth_sim::aaa::{GroundKind, M_PER_MAP};
use stealth_sim::station::{
    FP2_STOCK, GEPARD_REACH, GEPARD_ROUNDS, GEPARD_STOCK, INTERCEPTOR_STOCK, PATRIOT_STOCK,
    STORK_STOCK, StationCmd, mannable,
};
use stealth_sim::{Sim, fflags};

use super::ops::{fit_map, man_refusal};
use super::{Action, ActiveCampaign};
use crate::map_draw::{self, FRIENDLY_BLUE, MISSILE_RED, MapTips, MapView};
use crate::menu::{AMBER, BRIGHT, DIM, GREEN, icon, icon_text, section};

/// The list's width (points).
const LIST_W: f32 = 430.0;
/// A card's field name, ready count, stock bar and stock count widths (points).
const KEY_W: f32 = 52.0;
const BAR_W: f32 = 120.0;
const NUM_W: f32 = 80.0;
const STOCK_W: f32 = 32.0;
/// Patriot and Gepard reach on the map (map units).
const PATRIOT_REACH: f32 = stealth_sim::campaign::PATRIOT_REACH as f32;
const GEPARD_MAP_REACH: f32 = (GEPARD_REACH / M_PER_MAP) as f32;

/// A friendly air defence site a player can man.
struct Site {
    unit: usize,
    x: i32,
    y: i32,
    kind: GroundKind,
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
    let mut out: Vec<Site> = (0..sim.units().len())
        .filter(|&k| sim.aaa_blue(k) && mannable(sim.units()[k].kind))
        .map(|k| {
            let u = sim.units()[k];
            let gepard = u.kind == GroundKind::Gepard;
            let drones = u.kind == GroundKind::DroneCenter;
            let near = st
                .places
                .iter()
                .min_by_key(|p| stealth_campaign::dist(p.x - u.x, p.y - u.y))
                .map_or_else(String::new, |p| p.name.clone());
            let units = match u.kind {
                GroundKind::Gepard => vec![k],
                GroundKind::DroneCenter => Vec::new(),
                _ => sim.battery_launchers(k),
            };
            let alive: Vec<usize> =
                units.iter().copied().filter(|&l| !sim.units()[l].destroyed).collect();
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
                kind: u.kind,
                near,
                destroyed: u.destroyed,
                manned: sim.manned_by(k),
                ready: if drones {
                    sim.uavs_of(k).count() as i32
                } else {
                    alive.iter().map(|&l| sim.units()[l].rounds).sum()
                },
                full: if drones {
                    stealth_sim::uav::MAX_AIRBORNE as i32
                } else {
                    alive.len() as i32 * per
                },
                stock: sim.stockpile(k),
                reloading: units.iter().filter(|&&l| sim.refill_left(l) > 0.0).count(),
                launchers: alive.len(),
                enemies,
            }
        })
        .collect();
    // The drone command centers first, then the Patriot batteries, then the Gepards, each west
    // to east.
    out.sort_by_key(|s| (order(s.kind), s.x));
    out
}

fn order(kind: GroundKind) -> u8 {
    match kind {
        GroundKind::DroneCenter => 0,
        GroundKind::Gepard => 2,
        _ => 1,
    }
}

/// The site's short name: P1, P2 .. for the batteries, G1 .. for the Gepards, D1 .. for the
/// drone command centers.
fn tag(sites: &[Site], i: usize) -> String {
    let s = &sites[i];
    let n = sites[..=i].iter().filter(|o| o.kind == s.kind).count();
    let letter = match s.kind {
        GroundKind::Gepard => "G",
        GroundKind::DroneCenter => "D",
        _ => "P",
    };
    format!("{letter}{n}")
}

/// The AIR DEFENSE tab.
pub(super) fn tab(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    sim: &Sim,
    planner: &mut crate::planner::Planner,
    scenery: Option<&crate::scenery::Scenery>,
    size: egui::Vec2,
) {
    let sites = sites(c, sim);
    if c.ui.defence.is_some_and(|u| !sites.iter().any(|s| s.unit == u)) {
        c.ui.defence = None;
    }
    ui.horizontal_top(|ui| {
        map(ui, c, sim, &sites, planner, scenery, egui::vec2(size.x - LIST_W, size.y));
        ui.vertical(|ui| {
            ui.set_width(LIST_W - 12.0);
            super::designate::buttons(ui, c);
            ui.add_space(6.0);
            map_draw::symbology_switch(ui);
            list(ui, c, sim, &sites, size.y - 104.0);
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
    planner: &mut crate::planner::Planner,
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
    planner.draw_map(&painter, &view, &c.mission.theater_code.to_ascii_lowercase());
    painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(140));
    let at = |x: i32, y: i32| view.to_screen(x as f32, y as f32);
    let st = c.state();
    let k = map_draw::BIG;
    let mut tips = MapTips::grown(k);
    for p in &st.places {
        let col = match p.owner {
            Side::Blue => FRIENDLY_BLUE,
            Side::Red => MISSILE_RED,
            Side::Neutral => DIM,
        }
        .gamma_multiply(0.5);
        painter.circle_filled(at(p.x, p.y), 2.5 * k, col);
        super::ops::place_tip(&mut tips, at(p.x, p.y), p, st.clock, true);
    }
    // The reach: filled, the selected site's brighter; then the symbols over all of them.
    for s in sites.iter().filter(|s| !s.destroyed) {
        let reach = match s.kind {
            GroundKind::Gepard => GEPARD_MAP_REACH,
            // A drone command center's: as far as its surveillance drones see from over it.
            GroundKind::DroneCenter => (stealth_sim::uav::STORK_SENSOR / M_PER_MAP) as f32,
            _ => PATRIOT_REACH,
        };
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
        map_draw::scaled(&painter, at(a.x, a.y), k, |p, c| {
            super::picture::aircraft(p, c, a, MISSILE_RED);
        });
        super::picture::aircraft_tip(&mut tips, at(a.x, a.y), a, false);
    }
    for a in &pic.friendly {
        map_draw::scaled(&painter, at(a.x, a.y), k, |p, c| {
            super::picture::aircraft(p, c, a, FRIENDLY_BLUE.gamma_multiply(0.6));
        });
        super::picture::aircraft_tip(&mut tips, at(a.x, a.y), a, true);
    }
    let mut hovered = None;
    let mut named = Vec::new();
    for (i, s) in sites.iter().enumerate() {
        let p = at(s.x, s.y);
        let kind = s.kind;
        if s.destroyed {
            painter.circle_stroke(p, 5.0 * k, egui::Stroke::new(1.5, DIM));
        } else {
            map_draw::scaled(&painter, p, k, |pa, p| super::picture::defence(pa, p, kind, 0.8));
        }
        if s.manned.is_some() {
            painter.circle_stroke(p, map_draw::halo(9.0) * k, egui::Stroke::new(1.5, GREEN));
        }
        site_tip(&mut tips, p, s, &tag(sites, i));
        // Named unless another's name is there already (crowded sites: the list has them).
        if !named.iter().any(|q: &egui::Pos2| q.distance(p) < 16.0 * k) {
            named.push(p);
            let col = if s.destroyed { DIM } else { BRIGHT };
            painter.text(
                p + k * if map_draw::app6() {
                    egui::vec2(10.0, -8.0)
                } else {
                    egui::vec2(8.0, -6.0)
                },
                egui::Align2::LEFT_BOTTOM,
                tag(sites, i),
                egui::FontId::monospace(11.0),
                col,
            );
        }
        if resp.hover_pos().is_some_and(|h| h.distance(p) < 10.0 * k) {
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
    if !designating {
        tips.show(&resp, hovered.map(|_| "Click to select it"));
    }
    painter.text(
        rect.left_bottom() + egui::vec2(8.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        "Click a site to select it - wheel: zoom, drag: pan",
        egui::FontId::monospace(11.0),
        DIM,
    );
}

/// A site's tooltip: what it is, its tag and place, its rounds and the enemies its radar sees.
fn site_tip(tips: &mut MapTips, p: egui::Pos2, s: &Site, tag: &str) {
    let kind = super::picture::defence_name(s.kind);
    let tip = tips.add(p, kind, tag, FRIENDLY_BLUE);
    let (what, destroyed) = (s.kind, s.destroyed);
    tip.symbol(move |p, c| {
        if destroyed {
            p.circle_stroke(c, 5.0, egui::Stroke::new(1.5, DIM));
        } else {
            super::picture::defence(p, c, what, 0.8);
        }
    });
    if !s.near.is_empty() {
        tip.line(format!("Near {}", crate::mission::clean_text(&s.near)));
    }
    if s.destroyed {
        tip.line("Destroyed");
        return;
    }
    let what = if s.kind == GroundKind::DroneCenter { "Drones up" } else { "Rounds ready" };
    tip.line(format!("{what} {} / {}, stock {}", s.ready, s.full, s.stock));
    match s.enemies.first() {
        Some((name, km)) => {
            let n = s.enemies.len();
            tip.line(format!("{n} enemy aircraft on radar, nearest {name} at {km:.0} km"));
        }
        None => {
            tip.line("No enemy aircraft on radar");
        }
    }
    if s.manned.is_some() {
        tip.line("Manned by a player");
    }
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
        let kind = match s.kind {
            GroundKind::Gepard => "GEPARD",
            GroundKind::DroneCenter => "DRONES",
            _ => "PATRIOT",
        };
        ui.label(egui::RichText::new(format!("{tag} {kind}")).strong().color(FRIENDLY_BLUE));
        ui.label(egui::RichText::new(format!("near {}", s.near)).color(DIM));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let why = man_refusal(c, sim, s.unit);
            let button = egui::Button::new(icon_text(ui, icon::CONTROL, "ENTER"))
                .min_size(egui::vec2(80.0, 24.0));
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
    let drones = s.kind == GroundKind::DroneCenter;
    let ammo_col = if drones || s.ready * 2 >= s.full {
        GREEN
    } else if s.ready > 0 {
        AMBER
    } else {
        MISSILE_RED
    };
    let stock_col = |n: i32| if n > 0 { BRIGHT } else { MISSILE_RED };
    ui.horizontal(|ui| {
        key(ui, "OP");
        let operator = match s.manned {
            Some(p) if p == c.me.0 => egui::RichText::new("You").color(GREEN),
            Some(p) => egui::RichText::new(
                c.state().player_name(stealth_campaign::PlayerId(p)).unwrap_or("A player"),
            )
            .color(GREEN),
            None => egui::RichText::new("auto").color(DIM),
        };
        value(ui, operator, NUM_W);
        key(ui, "RDR");
        match s.enemies.first() {
            None => {
                ui.label(egui::RichText::new("clear").monospace().color(DIM));
            }
            Some((_, km)) => {
                let text = format!("{} - {km:.0} km", s.enemies.len());
                let list: Vec<String> =
                    s.enemies.iter().map(|(n, km)| format!("{n} {km:.0} km")).collect();
                ui.label(egui::RichText::new(text).monospace().color(MISSILE_RED))
                    .on_hover_text(list.join("\n"));
            }
        }
    });
    ui.horizontal(|ui| {
        key(ui, if drones { "UP" } else { "READY" });
        let ready = if s.kind != GroundKind::Gepard && !drones {
            format!("{}/{} {}L", s.ready, s.full, s.launchers)
        } else {
            format!("{}/{}", s.ready, s.full)
        };
        let col = if s.reloading > 0 { AMBER } else { ammo_col };
        let r = value(ui, egui::RichText::new(ready).color(col), NUM_W);
        if s.reloading > 0 {
            r.on_hover_text(format!("{} reloading", s.reloading));
        }
        key(ui, "STOCK");
        if drones {
            // One short bar for each kind of drone (Stork, FP-2, interceptor), together as
            // wide as the others' bar and number.
            let w = BAR_W + STOCK_W + ui.spacing().item_spacing.x;
            let layout = egui::Layout::left_to_right(egui::Align::Center);
            ui.allocate_ui_with_layout(egui::vec2(w, 16.0), layout, |ui| {
                ui.set_min_width(w);
                ui.spacing_mut().item_spacing.x = 2.0;
                let stock = sim.uav_stock(s.unit);
                let full = [STORK_STOCK, FP2_STOCK, INTERCEPTOR_STOCK];
                let names = [("S", "Stork"), ("F", "FP-2"), ("I", "Interceptors")];
                for ((n, full), (short, name)) in stock.into_iter().zip(full).zip(names) {
                    let r = ui.label(egui::RichText::new(short).monospace().color(DIM));
                    r.on_hover_text(name);
                    // Room for the letter, the one-digit count and the gaps.
                    bar(ui, n, full, w / 3.0 - 30.0, stock_col(n));
                    value(ui, egui::RichText::new(n.to_string()).color(stock_col(n)), 12.0);
                    ui.add_space(4.0);
                }
            });
        } else {
            let full = if s.kind == GroundKind::Gepard { GEPARD_STOCK } else { PATRIOT_STOCK };
            bar(ui, s.stock, full, BAR_W, stock_col(s.stock));
            value(ui, egui::RichText::new(s.stock.to_string()).color(stock_col(s.stock)), STOCK_W);
        }
    });
}

/// A field's name, in a fixed width so the fields line up from card to card.
fn key(ui: &mut egui::Ui, text: &str) {
    value(ui, egui::RichText::new(text).color(DIM), KEY_W);
}

/// A monospace value in a fixed width.
fn value(ui: &mut egui::Ui, text: egui::RichText, w: f32) -> egui::Response {
    let layout = egui::Layout::left_to_right(egui::Align::Center);
    ui.allocate_ui_with_layout(egui::vec2(w, 16.0), layout, |ui| {
        ui.set_min_width(w);
        ui.add(egui::Label::new(text.monospace()).truncate());
    })
    .response
}

/// A bar `n` of `full` filled.
fn bar(ui: &mut egui::Ui, n: i32, full: i32, w: f32, col: egui::Color32) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(w, 10.0), egui::Sense::hover());
    let frac = (n as f32 / full.max(1) as f32).clamp(0.0, 1.0);
    let p = ui.painter();
    p.rect_filled(r, 0.0, col.gamma_multiply(0.12));
    p.rect_filled(egui::Rect::from_min_size(r.min, egui::vec2(w * frac, r.height())), 0.0, col);
    p.rect_stroke(
        r,
        0.0,
        egui::Stroke::new(1.0, col.gamma_multiply(0.5)),
        egui::StrokeKind::Inside,
    );
}

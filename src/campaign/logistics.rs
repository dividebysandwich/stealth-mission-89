//! The Ops screen's LOGISTICS tab: the friendly airbases, the production base first, each with
//! what it is out of and the cargo flight coming; the one selected with its munitions one by
//! one (the production base's making too); the cargo flights under way; and both sides'
//! forces and materiel.

use bevy_egui::egui;
use stealth_campaign::tuning::PRODUCTION_CAP;
use stealth_campaign::{CargoState, Munition, ObjectId, Side, SiteKind, Stock};
use stealth_i18n::t;

use super::ActiveCampaign;
use crate::map_draw::{FRIENDLY_BLUE, MISSILE_RED};
use crate::menu::{AMBER, BRIGHT, DIM, GREEN, section};

/// The list's width, and a detail row's name, count and bar widths (points).
const LIST_W: f32 = 380.0;
const NAME_W: f32 = 170.0;
const COUNT_W: f32 = 110.0;
const BAR_W: f32 = 160.0;
/// Below this share of a full stock a munition runs low.
const LOW: f64 = 0.25;

/// What the tab shows beside the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    Base(ObjectId),
    Forces,
}

/// "4:05".
fn mmss(secs: f64) -> String {
    let s = secs.max(0.0).ceil() as i64;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// The friendly airbases with a stock: the production base first, then by name.
fn bases(c: &ActiveCampaign) -> Vec<(ObjectId, String)> {
    let st = c.state();
    let prod = st.production_base(Side::Blue);
    let mut out: Vec<(ObjectId, String)> = st
        .stocks
        .iter()
        .filter(|(b, s)| s.side == Some(Side::Blue) && st.owner(**b) == Some(Side::Blue))
        .filter_map(|(b, _)| st.site(*b).filter(|s| s.kind == SiteKind::Airfield))
        .map(|s| (s.object, s.name.clone()))
        .collect();
    out.sort_by_key(|(b, name)| (Some(*b) != prod, name.clone()));
    out
}

/// The munitions a base is out of, and running low on.
fn shortages(s: &Stock) -> (Vec<Munition>, Vec<Munition>) {
    let side = Side::Blue;
    let out = s.empty(side);
    let low = Munition::of_side(side)
        .iter()
        .copied()
        .filter(|&m| s.count(m) > 0 && (s.count(m) as f64) < m.full() * LOW)
        .collect();
    (out, low)
}

fn names(ms: &[Munition]) -> String {
    ms.iter().map(|m| m.name().tr()).collect::<Vec<_>>().join(", ")
}

pub fn tab(ui: &mut egui::Ui, c: &mut ActiveCampaign, size: egui::Vec2) {
    if !c.state().munitions_on() {
        super::ops::forces(ui, c);
        return;
    }
    let list = bases(c);
    let st = c.state();
    if st.production_base(Side::Blue).is_none()
        && let Some(t) = st.production_target(Side::Blue).and_then(|t| st.site(t))
    {
        let text = t!("lg-no-production", base = t.name.as_str());
        ui.label(egui::RichText::new(text).color(AMBER));
        ui.add_space(4.0);
    }
    let valid = match c.ui.logistics {
        Some(Pick::Base(b)) => list.iter().any(|l| l.0 == b),
        Some(Pick::Forces) => true,
        None => false,
    };
    if !valid {
        c.ui.logistics = list.first().map(|l| Pick::Base(l.0));
    }
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(LIST_W);
            base_list(ui, c, &list, size.y);
        });
        ui.separator();
        ui.vertical(|ui| {
            egui::ScrollArea::vertical().id_salt("logistics_detail").show(ui, |ui| {
                match c.ui.logistics {
                    Some(Pick::Base(b)) => base_detail(ui, c, b),
                    Some(Pick::Forces) => super::ops::forces(ui, c),
                    None => {}
                }
            });
        });
    });
}

/// The airbases one under the other, then the forces.
fn base_list(ui: &mut egui::Ui, c: &mut ActiveCampaign, list: &[(ObjectId, String)], h: f32) {
    let chosen = c.ui.logistics == Some(Pick::Forces);
    if ui.selectable_label(chosen, t!("lg-forces")).clicked() {
        c.ui.logistics = Some(Pick::Forces);
    }
    ui.add_space(6.0);
    section(ui, t!("lg-bases"));
    let prod = c.state().production_base(Side::Blue);
    egui::ScrollArea::vertical().id_salt("logistics").max_height(h - 80.0).show(ui, |ui| {
        for (b, name) in list {
            let chosen = c.ui.logistics == Some(Pick::Base(*b));
            let frame = egui::Frame::group(ui.style())
                .stroke(egui::Stroke::new(
                    if chosen { 2.0 } else { 1.0 },
                    if chosen { BRIGHT } else { DIM.gamma_multiply(0.5) },
                ))
                .inner_margin(6.0);
            let r = frame.show(ui, |ui| {
                ui.set_width(LIST_W - 30.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(name).strong().color(FRIENDLY_BLUE));
                    if prod == Some(*b) {
                        ui.label(egui::RichText::new(t!("lg-production")).small().color(AMBER));
                    }
                });
                let st = c.state();
                if let Some(s) = st.stocks.get(b) {
                    let (out, low) = shortages(s);
                    let (text, col) = if !out.is_empty() {
                        (t!("lg-out-of", items = names(&out)), MISSILE_RED)
                    } else if !low.is_empty() {
                        (t!("lg-low", items = names(&low)), AMBER)
                    } else {
                        (t!("lg-stocked").to_string(), GREEN)
                    };
                    ui.add(egui::Label::new(egui::RichText::new(text).color(col)).truncate());
                }
                if let Some(r) = st.cargo_runs.iter().find(|r| r.to == *b) {
                    let text = match r.state {
                        CargoState::Scheduled { launch_at } => {
                            t!("lg-cargo-due", time = mmss(launch_at - st.clock))
                        }
                        CargoState::Airborne => t!("lg-cargo-up").to_string(),
                        CargoState::Player { player } => t!(
                            "lg-cargo-player",
                            pilot = st.player_name(player).unwrap_or("").to_string()
                        ),
                    };
                    ui.label(egui::RichText::new(text).color(AMBER));
                }
            });
            let r = r.response.interact(egui::Sense::click());
            if r.clicked() {
                c.ui.logistics = Some(Pick::Base(*b));
            }
            ui.add_space(4.0);
        }
    });
}

/// A base's munitions; the production base's making; the cargo flights.
fn base_detail(ui: &mut egui::Ui, c: &ActiveCampaign, b: ObjectId) {
    let st = c.state();
    let Some(s) = st.stocks.get(&b) else { return };
    let name = st.site(b).map_or(String::new(), |s| s.name.clone());
    let prod = st.production_base(Side::Blue) == Some(b);
    section(ui, &name);
    if prod {
        ui.label(egui::RichText::new(t!("lg-production-base")).color(AMBER));
    }
    if !st.site(b).is_some_and(|s| s.alive()) {
        let id = if prod { "lg-bombed-production" } else { "lg-bombed" };
        ui.label(egui::RichText::new(t!(id)).color(MISSILE_RED));
    }
    ui.add_space(4.0);
    egui::Grid::new("munitions").num_columns(if prod { 5 } else { 3 }).spacing([12.0, 4.0]).show(
        ui,
        |ui| {
            ui.label(egui::RichText::new(t!("lg-munition")).color(DIM));
            ui.label(egui::RichText::new(t!("lg-stock")).color(DIM));
            ui.label("");
            if prod {
                ui.label(egui::RichText::new(t!("lg-made")).color(DIM));
                ui.label("");
            }
            ui.end_row();
            for &m in Munition::of_side(Side::Blue) {
                let n = s.count(m);
                let full = if prod { m.full() * PRODUCTION_CAP } else { m.full() };
                let col = if n == 0 {
                    MISSILE_RED
                } else if (n as f64) < m.full() * LOW {
                    AMBER
                } else {
                    BRIGHT
                };
                cell(ui, egui::RichText::new(m.name().tr()).color(BRIGHT), NAME_W);
                let count = format!("{n} / {full:.0}");
                cell(ui, egui::RichText::new(count).monospace().color(col), COUNT_W);
                bar(ui, n as f32 / full as f32, BAR_W, col);
                if prod {
                    let rate = m.per_hour();
                    let per =
                        if rate >= 10.0 { format!("{rate:.0}") } else { format!("{rate:.1}") };
                    let made = t!("lg-per-hour", count = per);
                    cell(ui, egui::RichText::new(made).monospace().color(DIM), 70.0);
                    if n as f64 >= full {
                        ui.label(egui::RichText::new(t!("lg-at-cap")).color(DIM));
                    } else {
                        let frac = s.progress(m);
                        let next = (1.0 - frac) * 3600.0 / rate;
                        ui.horizontal(|ui| {
                            bar(ui, frac as f32, 60.0, GREEN);
                            let text = t!("lg-next-in", time = mmss(next));
                            ui.label(egui::RichText::new(text).monospace().color(DIM));
                        });
                    }
                }
                ui.end_row();
            }
        },
    );
    ui.add_space(4.0);
    ui.label(egui::RichText::new(t!("lg-never-out")).small().color(DIM));
    ui.add_space(10.0);
    cargo_flights(ui, c);
}

/// The friendly cargo flights scheduled or in the air.
fn cargo_flights(ui: &mut egui::Ui, c: &ActiveCampaign) {
    let st = c.state();
    section(ui, t!("lg-cargo-flights"));
    let runs: Vec<_> = st.cargo_runs.iter().filter(|r| r.side == Side::Blue).collect();
    if runs.is_empty() {
        ui.label(egui::RichText::new(t!("lg-no-cargo")).color(DIM));
        return;
    }
    let name = |b: ObjectId| st.site(b).map_or(String::new(), |s| s.name.clone());
    for r in runs {
        ui.horizontal(|ui| {
            let route = t!("lg-cargo-route", from = name(r.from), to = name(r.to));
            ui.label(egui::RichText::new(route).strong().color(FRIENDLY_BLUE));
            let (text, col) = match r.state {
                CargoState::Scheduled { launch_at } => {
                    (t!("lg-cargo-due", time = mmss(launch_at - st.clock)), AMBER)
                }
                CargoState::Airborne => (t!("lg-cargo-up").to_string(), GREEN),
                CargoState::Player { player } => (
                    t!("lg-cargo-player", pilot = st.player_name(player).unwrap_or("").to_string()),
                    GREEN,
                ),
            };
            ui.label(egui::RichText::new(text).color(col));
        });
        let load: Vec<String> =
            r.load.iter().map(|&(m, n)| format!("{n} {}", m.name().tr())).collect();
        let text = t!("lg-cargo-load", items = load.join(", "));
        ui.add(egui::Label::new(egui::RichText::new(text).color(DIM)).wrap());
        ui.add_space(4.0);
    }
}

/// A label in a fixed width (smaller if it is too long: a translation).
fn cell(ui: &mut egui::Ui, text: egui::RichText, w: f32) {
    let layout = egui::Layout::left_to_right(egui::Align::Center);
    ui.allocate_ui_with_layout(egui::vec2(w, 16.0), layout, |ui| {
        ui.set_min_width(w);
        ui.add(egui::Label::new(text).truncate());
    });
}

/// A bar `frac` filled.
fn bar(ui: &mut egui::Ui, frac: f32, w: f32, col: egui::Color32) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(w, 10.0), egui::Sense::hover());
    let frac = frac.clamp(0.0, 1.0);
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

/// A place's part in the friendly side's production: its production base, or the enemy
/// airfield to take for it (with only carriers left).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Production {
    Base,
    Target,
}

impl Production {
    /// The map tooltip's line.
    pub(super) fn tip(self) -> &'static str {
        match self {
            Production::Base => t!("mt-production-base"),
            Production::Target => t!("mt-production-target"),
        }
    }
}

/// The friendly production base's mark beside its airfield symbol at `c` (`k`: the map's
/// symbol scale): a factory, its sawtooth roof (faint: the airfield to take for it).
pub(super) fn production_mark(p: &egui::Painter, c: egui::Pos2, k: f32, kind: Production) {
    let o = c + egui::vec2(-11.0, -7.0) * k;
    let col = if kind == Production::Base { AMBER } else { AMBER.gamma_multiply(0.55) };
    let s = egui::Stroke::new(1.5, col);
    let pt = |x: f32, y: f32| o + egui::vec2(x, y) * k;
    p.add(egui::Shape::line(
        vec![
            pt(-5.0, 4.0),
            pt(-5.0, -1.0),
            pt(-2.0, -4.0),
            pt(-2.0, -1.0),
            pt(1.0, -4.0),
            pt(1.0, -1.0),
            pt(4.0, -4.0),
            pt(4.0, 4.0),
            pt(-5.0, 4.0),
        ],
        s,
    ));
}

/// The place's airfield is the friendly production base, or the one to take for it.
pub(super) fn production_of(
    st: &stealth_campaign::CampaignState,
    p: &stealth_campaign::Place,
) -> Option<Production> {
    let a = p.airfield?;
    if p.owner == Side::Blue && Some(a) == st.production_base(Side::Blue) {
        Some(Production::Base)
    } else if Some(a) == st.production_target(Side::Blue) {
        Some(Production::Target)
    } else {
        None
    }
}

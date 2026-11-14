//! The accepted mission's briefing, over a whole screen: the route map under three steps, the
//! aircraft (and the base it flies from), its loadout, and the route itself, with TAKE OFF
//! always at the top right. A pilot's first briefing walks through all three steps before
//! TAKE OFF is lit (`briefing.json` remembers it is done).

use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy_egui::egui;
use stealth_campaign::{Aircraft as Airframe, Side};
use stealth_host::glue::sortie_mission;
use stealth_i18n::{Text, t};

use super::{
    AMBER, Action, ActiveCampaign, BG, BRIGHT, DIM, FRIENDLY_BLUE, GREEN, MISSILE_RED, MapLayers,
    MapTips, MapView, MapWeather, Role, affiliation, describe_offer, dist, group, group_label,
    icon, kind_colour, kind_name, kind_of, km, place_legend, place_tip, preview, shell,
    shows_place,
};
use crate::map_draw;

/// A step of the briefing, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Aircraft,
    Loadout,
    Route,
}

impl Step {
    const ALL: [Step; 3] = [Step::Aircraft, Step::Loadout, Step::Route];

    fn index(self) -> usize {
        self as usize
    }

    fn label(self) -> &'static str {
        match self {
            Step::Aircraft => t!("debrief-aircraft"),
            Step::Loadout => t!("menu-loadout"),
            Step::Route => t!("planner-route"),
        }
    }

    fn next(self) -> Option<Step> {
        Step::ALL.get(self.index() + 1).copied()
    }
}

/// A mission accepted from the board, briefed before take-off.
pub struct Briefing {
    pub(super) offer: u32,
    pub(super) base: u16,
    /// The aircraft to fly it in.
    aircraft: Airframe,
    loadout: crate::loadout::PlannedLoadout,
    window: crate::loadout::LoadoutWindow,
    step: Step,
    /// The steps looked at (the first briefing's walk through them).
    seen: [bool; 3],
    /// The route's vertical profile is unfolded.
    profile: bool,
    /// The route to fly (None until the editor first shows it). In a coop game the plan's,
    /// with this player's changes not sent yet.
    route: Option<crate::mission::PlannedRoute>,
    /// Changed here, not sent to the plan yet.
    dirty: bool,
    /// When the last change went (egui time), and until when the plan's route (base) is not
    /// taken over (its echo is on the way).
    sent_at: f64,
    hold_until: f64,
    base_hold_until: f64,
}

impl Briefing {
    pub(super) fn new(
        offer: u32,
        base: u16,
        aircraft: Airframe,
        loadout: crate::loadout::PlannedLoadout,
        step: Step,
    ) -> Self {
        let mut seen = [false; 3];
        seen[step.index()] = true;
        Self {
            offer,
            base,
            aircraft,
            loadout,
            window: Default::default(),
            step,
            seen,
            profile: true,
            route: None,
            dirty: false,
            sent_at: 0.0,
            hold_until: 0.0,
            base_hold_until: 0.0,
        }
    }

    fn go(&mut self, step: Step) {
        self.step = step;
        self.seen[step.index()] = true;
    }
}

/// Test aid: `--campaign-accept aircraft|loadout|route` opens the first mission's briefing at
/// that step (`map`: the route).
pub(super) fn test_accept() -> Option<Step> {
    let args: Vec<String> = std::env::args().collect();
    let p = args.iter().position(|a| a == "--campaign-accept").and_then(|i| args.get(i + 1))?;
    Some(match p.as_str() {
        "aircraft" => Step::Aircraft,
        "loadout" => Step::Loadout,
        _ => Step::Route,
    })
}

/// The pilot has walked through a briefing's steps once (`briefing.json`).
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, Copy)]
#[serde(default)]
struct BriefingPrefs {
    walked: bool,
}

const PREFS_FILE: &str = "briefing.json";

static WALKED: LazyLock<AtomicBool> = LazyLock::new(|| {
    let walked = crate::prefs::load_json::<BriefingPrefs>(PREFS_FILE).unwrap_or_default().walked;
    // Test aid: `--first-briefing` briefs as the first time.
    AtomicBool::new(walked && !std::env::args().any(|a| a == "--first-briefing"))
});

fn walked() -> bool {
    WALKED.load(Ordering::Relaxed)
}

fn set_walked() {
    if walked() {
        return;
    }
    WALKED.store(true, Ordering::Relaxed);
    if let Err(e) = crate::prefs::save_json(PREFS_FILE, &BriefingPrefs { walked: true }) {
        bevy::log::warn!("{PREFS_FILE}: {e:#}");
    }
}

/// Seconds between a dragged point's updates to the group.
const SEND_EVERY: f64 = 0.1;
/// Seconds the group's route is not taken over after a change of this player's went (until
/// its echo is back).
const HOLD: f64 = 1.0;
/// The summary strip's height under the top bar.
const STRIP_H: f32 = 50.0;

/// The briefing's screen.
pub(super) fn briefing(
    ctx: &egui::Context,
    c: &mut ActiveCampaign,
    sim: &stealth_sim::Sim,
    planner: &mut crate::planner::Planner,
    weather: Option<&str>,
    map_wx: &MapWeather,
) {
    let Some(mut b) = c.ui.briefing.take() else { return };
    let Some(o) = c.state().board.iter().find(|o| o.id == b.offer).cloned() else {
        c.ui.error = Some(Text::new("ce-off-board"));
        return;
    };
    let now = ctx.input(|i| i.time);
    // A coop game: the group's plan sets the base (unless this player's choice is on its way).
    let coop = c.role != Role::Local;
    let plan = c.state().plan(b.offer).filter(|_| coop).cloned();
    if let Some(p) = &plan
        && now >= b.base_hold_until
        && p.base != b.base
    {
        b.base = p.base;
        b.route = None;
    }
    let taken = plan.as_ref().is_some_and(|p| p.crew.len() > 1 || !p.flying.is_empty());
    let mut m = sortie_mission(&c.mission, c.state(), &preview(&o, b.base, b.aircraft));
    // What the base has of the weapons that run out.
    let stock_of = |c: &ActiveCampaign, base: u16| -> Vec<(u8, u32)> {
        (0..crate::loadout::WEAPONS.len() as u8)
            .filter_map(|w| {
                let mu = stealth_campaign::Munition::of_weapon(w as i32)?;
                let n = c.state().available(base, mu);
                (n != u32::MAX).then_some((w, n))
            })
            .collect()
    };
    let screen = ctx.content_rect();
    let room = shell::Room::of(screen.width());
    let top = egui::Rect::from_min_size(screen.min, egui::vec2(screen.width(), shell::TOP_H));
    let strip = egui::Rect::from_min_size(top.left_bottom(), egui::vec2(screen.width(), STRIP_H));
    let map_rect = egui::Rect::from_min_max(strip.left_bottom(), screen.max);
    let ready = walked() || b.seen.iter().all(|&s| s);
    let mut back = false;
    let mut take_off = false;
    let mut edit = None;
    // The map under it all, the top bar and the strip over it.
    egui::Area::new(egui::Id::new("briefing_screen"))
        .fixed_pos(screen.min)
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            ui.set_min_size(screen.size());
            ui.scope_builder(egui::UiBuilder::new().max_rect(map_rect), |ui| {
                edit = route_map(ui, c, sim, planner, &m, &mut b, map_wx, map_rect);
            });
            let edge = egui::Stroke::new(2.0, crate::menu::FRAME);
            ui.painter().rect_filled(top, 0.0, BG);
            ui.painter().hline(top.x_range(), top.bottom() - 1.0, edge);
            ui.scope_builder(egui::UiBuilder::new().max_rect(top), |ui| {
                (back, take_off) = top_bar(ui, c, &o, &mut b, ready, room);
            });
            ui.painter().rect_filled(strip, 0.0, BG.gamma_multiply(0.94));
            ui.painter().hline(strip.x_range(), strip.bottom() - 1.0, edge);
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(strip.shrink2(egui::vec2(16.0, 0.0))),
                |ui| {
                    summary(ui, c, &o, &b, &m, weather);
                },
            );
        });
    // The step's panel over the map.
    let panel_at = map_rect.left_top() + egui::vec2(shell::GAP, shell::GAP);
    match b.step {
        Step::Aircraft => {
            let w = 880.0_f32.min(map_rect.width() - 2.0 * shell::GAP);
            let max_h = map_rect.height() - 2.0 * shell::GAP;
            egui::Area::new(egui::Id::new("briefing_aircraft"))
                .fixed_pos(panel_at)
                .order(egui::Order::Middle)
                .show(ctx, |ui| {
                    shell::overlay_frame().inner_margin(egui::Margin::symmetric(18, 12)).show(
                        ui,
                        |ui| {
                            ui.set_width(w - 36.0);
                            ui.set_max_height(max_h - 24.0);
                            step_heading(ui, Step::Aircraft, t!("brief-choose-aircraft"));
                            egui::ScrollArea::vertical()
                                .id_salt("aircraft_step")
                                .max_height(max_h - 170.0)
                                .show(ui, |ui| {
                                    let changed = aircraft_step(ui, c, &o, &mut b, taken, now);
                                    if changed {
                                        m = sortie_mission(
                                            &c.mission,
                                            c.state(),
                                            &preview(&o, b.base, b.aircraft),
                                        );
                                        b.loadout = crate::loadout::PlannedLoadout::of(
                                            &m,
                                            kind_of(b.aircraft),
                                        );
                                        b.route = None;
                                        planner.refit();
                                    }
                                });
                            step_footer(ui, &mut b, ready, None);
                        },
                    );
                });
        }
        Step::Loadout => {
            let w = 1400.0_f32.min(map_rect.width() - 2.0 * shell::GAP);
            let h = map_rect.height() - 2.0 * shell::GAP;
            let stock = stock_of(c, b.base);
            let stock = |w: u8| stock.iter().find(|s| s.0 == w).map(|s| s.1);
            egui::Area::new(egui::Id::new("briefing_loadout"))
                .fixed_pos(panel_at)
                .order(egui::Order::Middle)
                .show(ctx, |ui| {
                    shell::overlay_frame().inner_margin(egui::Margin::symmetric(18, 12)).show(
                        ui,
                        |ui| {
                            ui.set_width(w - 36.0);
                            let title = format!(
                                "{}  ·  {}",
                                t!("menu-loadout"),
                                kind_of(b.aircraft).name().to_uppercase()
                            );
                            step_heading(ui, Step::Loadout, &title);
                            let size = egui::vec2(w - 36.0, (h - 24.0 - 160.0).max(300.0));
                            ui.allocate_ui(size, |ui| {
                                ui.set_min_size(size);
                                crate::loadout::editor_large(
                                    ui,
                                    &mut b.window,
                                    &mut b.loadout,
                                    &m,
                                    size,
                                    &stock,
                                );
                            });
                            let briefed =
                                crate::loadout::PlannedLoadout::of(&m, kind_of(b.aircraft));
                            if step_footer(ui, &mut b, ready, Some(t!("loadout-briefed"))) {
                                b.loadout = briefed;
                            }
                        },
                    );
                });
        }
        Step::Route => {}
    }
    // This player's changes to the group's route.
    if let Some(route) = edit {
        c.actions.push(Action::EditPlan { offer: o.id, base: b.base, route });
    }
    if take_off {
        if coop
            && std::mem::take(&mut b.dirty)
            && let Some(r) = &b.route
        {
            let route = r.points.clone();
            c.actions.push(Action::EditPlan { offer: o.id, base: b.base, route });
        }
        set_walked();
        c.ui.error = None;
        c.aircraft = b.aircraft;
        c.actions.push(Action::TakeMission {
            offer: o.id,
            base: Some(b.base),
            aircraft: b.aircraft,
            loadout: b.loadout.clone(),
            route: b.route.clone(),
        });
        return;
    }
    if !back {
        c.ui.briefing = Some(b);
    } else if coop {
        c.actions.push(Action::SelectMission { offer: None, base: b.base });
    }
}

/// The top bar: BACK, the mission, the steps, TAKE OFF. Returns (back, take off) pressed.
fn top_bar(
    ui: &mut egui::Ui,
    c: &ActiveCampaign,
    o: &stealth_campaign::MissionOffer,
    b: &mut Briefing,
    ready: bool,
    room: shell::Room,
) -> (bool, bool) {
    let mut back = false;
    let mut take_off = false;
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        back = shell::button(
            ui,
            Some(icon::BACK),
            t!("back"),
            false,
            shell::TOP_H - 4.0,
            Some(shell::RAIL_W.max(130.0)),
        )
        .clicked();
        ui.add_space(16.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            let head = format!("{}  ·  {}", t!("ops-briefing"), kind_name(o.kind));
            ui.label(egui::RichText::new(head).small().color(kind_colour(o.kind)));
            let title = crate::mission::clean_text(&o.title.tr());
            ui.label(egui::RichText::new(title).size(19.0).color(BRIGHT));
        });
        // The steps, in the middle.
        let steps_w = 3.0 * 210.0;
        let right_w = 260.0;
        let space = (ui.available_width() - steps_w - right_w) / 2.0;
        ui.add_space(space.max(12.0));
        for step in Step::ALL {
            let on = b.step == step;
            let done = b.seen[step.index()] && !on;
            let w = if room == shell::Room::Small { 70.0 } else { 210.0 };
            let (r, resp) =
                ui.allocate_exact_size(egui::vec2(w, shell::TOP_H - 2.0), egui::Sense::click());
            let p = ui.painter();
            if on {
                p.rect_filled(r, 0.0, crate::menu::HOVER);
                p.hline(r.x_range(), r.bottom() - 2.0, egui::Stroke::new(4.0, crate::menu::EDGE));
            } else if resp.hovered() {
                p.rect_filled(r, 0.0, crate::menu::FIELD);
            }
            let n_at = egui::pos2(r.left() + 30.0, r.center().y);
            shell::step_number(p, n_at, step.index() + 1, on, done);
            if room != shell::Room::Small {
                let ink = if on {
                    BRIGHT
                } else if done {
                    GREEN
                } else {
                    DIM
                };
                p.text(
                    n_at + egui::vec2(26.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    step.label().to_uppercase(),
                    egui::FontId::monospace(17.0),
                    ink,
                );
            }
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                b.go(step);
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.add_space(6.0);
            let button = crate::menu::big_button(t!("ops-take-off"))
                .icon(icon::FLY)
                .primary()
                .min_size(egui::vec2(180.0, 50.0));
            let r = ui.add_enabled(ready, button);
            take_off = r.clicked();
            if ready && c.state().outcome.is_none() {
                ui.add(egui::Spinner::new().color(GREEN)).on_hover_text(t!("ops-running"));
            }
        });
    });
    (back, take_off)
}

/// The strip under the top bar: the aircraft and base, the route, the stores, the weather, the
/// group; the last error.
fn summary(
    ui: &mut egui::Ui,
    c: &ActiveCampaign,
    o: &stealth_campaign::MissionOffer,
    b: &Briefing,
    m: &stealth_mission::Mission,
    weather: Option<&str>,
) {
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 22.0;
        let d = km(dist(m.takeoff.position.x, m.takeoff.position.y, o.x, o.y));
        let item = |ui: &mut egui::Ui, glyph: char, text: String, colour: egui::Color32| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let (r, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                shell::paint_icon(ui.painter(), r.center(), glyph, 15.0, DIM);
                ui.label(egui::RichText::new(text).color(colour));
            });
        };
        let from = t!(
            "brief-from",
            aircraft = kind_of(b.aircraft).name(),
            base = m.takeoff.name.as_str(),
            km = d.round() as i64
        );
        item(ui, icon::FLIGHTS, from, BRIGHT);
        item(ui, icon::LOADOUT, b.loadout.summary(), GREEN);
        if let Some(line) = weather {
            item(ui, icon::WEATHER, line.to_string(), DIM);
        }
        if c.role != Role::Local {
            group_label(ui, c, o);
            let (_, flying) = group(c, o.id);
            if !flying.is_empty() {
                let text = t!("ops-route-locked", pilots = flying.join(", "));
                ui.label(egui::RichText::new(text).color(AMBER));
            }
        }
        if let Some(e) = &c.ui.error {
            ui.label(egui::RichText::new(e.tr()).color(AMBER));
        }
        let _ = describe_offer(c, o);
    });
}

/// A step panel's heading: its number and title.
fn step_heading(ui: &mut egui::Ui, step: Step, title: &str) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(egui::vec2(36.0, 44.0), egui::Sense::hover());
        shell::step_number(ui.painter(), r.center(), step.index() + 1, true, false);
        ui.label(egui::RichText::new(title).size(22.0).color(BRIGHT));
    });
    let r = ui.max_rect();
    ui.painter().hline(
        r.x_range(),
        ui.cursor().top() + 2.0,
        egui::Stroke::new(1.0, crate::menu::FRAME),
    );
    ui.add_space(10.0);
}

/// A step panel's foot: the previous step, and the next (lit while the first briefing walks
/// through them).
fn step_footer(ui: &mut egui::Ui, b: &mut Briefing, ready: bool, reset: Option<&str>) -> bool {
    let mut reset_clicked = false;
    ui.add_space(8.0);
    let r = ui.max_rect();
    ui.painter().hline(r.x_range(), ui.cursor().top(), egui::Stroke::new(1.0, crate::menu::FRAME));
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        if let Some(prev) = b.step.index().checked_sub(1).map(|i| Step::ALL[i]) {
            let text = prev.label().to_uppercase();
            if shell::button(ui, Some(icon::BACK), &text, false, 48.0, None).clicked() {
                b.go(prev);
            }
        }
        if let Some(reset) = reset {
            ui.add_space(8.0);
            reset_clicked =
                shell::button(ui, Some(icon::BRIEFED), reset, false, 48.0, None).clicked();
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(next) = b.step.next() {
                let text = t!("brief-next", step = next.label().to_uppercase());
                let button = crate::menu::big_button(&text)
                    .icon(icon::NEXT)
                    .min_size(egui::vec2(200.0, 52.0));
                let button = if ready { button } else { button.primary() };
                if ui.add(button).clicked() {
                    b.go(next);
                }
            }
        });
    });
    reset_clicked
}

/// Step 1: the base to fly from and the aircraft. Whether either changed.
fn aircraft_step(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    o: &stealth_campaign::MissionOffer,
    b: &mut Briefing,
    taken: bool,
    now: f64,
) -> bool {
    let mut changed = false;
    // The bases, nearest first.
    let mut bases: Vec<(u16, String, i32)> = c
        .state()
        .player_bases()
        .into_iter()
        .filter_map(|id| {
            let site = c.state().site(id)?;
            Some((id, site.name.clone(), dist(site.x, site.y, o.x, o.y)))
        })
        .collect();
    bases.sort_by_key(|x| x.2);
    crate::menu::section(ui, t!("ops-from"));
    ui.add_enabled_ui(!taken, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
            for (id, name, d) in &bases {
                let on = *id == b.base;
                let text = format!("{name}  {:.0} km", km(*d));
                let glyph = on.then_some(icon::DONE);
                if shell::button(ui, glyph, &text, on, 48.0, None).clicked() && !on {
                    b.base = *id;
                    c.ui.base = Some(*id);
                    b.base_hold_until = now + HOLD;
                    if c.role != Role::Local {
                        c.actions.push(Action::SelectMission { offer: Some(o.id), base: *id });
                    }
                    changed = true;
                }
            }
        });
    });
    if taken {
        ui.label(egui::RichText::new(t!("ops-planned-by-group")).small().color(DIM));
    }
    ui.add_space(10.0);
    let base = c.state().site(b.base).map_or(String::new(), |s| s.name.clone());
    crate::menu::section(
        ui,
        &t!("brief-aircraft-for", base = base.as_str(), kind = kind_name(o.kind)),
    );
    // The aircraft that can fly it first, then those that cannot (and why).
    let mut all: Vec<(Airframe, Option<String>)> = crate::aircraft::AircraftKind::ALL
        .into_iter()
        .map(super::super::airframe_of)
        .map(|a| (a, c.state().offer_flyable(o, a).err().map(|e| e.tr())))
        .collect();
    all.sort_by_key(|a| a.1.is_some());
    let w = ui.available_width();
    let per_row = ((w + 10.0) / 220.0).floor().max(1.0) as usize;
    let card_w = (w - 10.0 * (per_row - 1) as f32) / per_row as f32;
    for row in all.chunks(per_row) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            for (a, why) in row {
                if aircraft_card(ui, *a, b.aircraft == *a, why.as_deref(), card_w)
                    && b.aircraft != *a
                {
                    b.aircraft = *a;
                    changed = true;
                }
            }
        });
        ui.add_space(10.0);
    }
    changed
}

/// An aircraft's card: its outline, name and what it is for; greyed out with the reason when
/// it cannot fly the mission. Whether it was clicked.
fn aircraft_card(ui: &mut egui::Ui, a: Airframe, on: bool, why: Option<&str>, w: f32) -> bool {
    let kind = kind_of(a);
    let enabled = why.is_none();
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(w, 150.0),
        if enabled { egui::Sense::click() } else { egui::Sense::hover() },
    );
    let hovered = enabled && resp.hovered();
    let (fill, edge, width) = shell::card_colours(on, hovered, enabled);
    let p = ui.painter();
    p.rect_filled(rect, 0.0, fill);
    p.rect_stroke(rect, 0.0, egui::Stroke::new(width, edge), egui::StrokeKind::Inside);
    let ink = |c: egui::Color32| if enabled { c } else { c.gamma_multiply(0.4) };
    let shape = egui::Rect::from_center_size(
        rect.center_top() + egui::vec2(0.0, 40.0),
        egui::vec2(w - 40.0, 60.0),
    );
    crate::loadout::silhouette(p, shape, kind, ink(crate::menu::EDGE));
    if on {
        shell::paint_icon(
            p,
            rect.right_top() + egui::vec2(-20.0, 20.0),
            icon::ACCEPT,
            22.0,
            crate::menu::EDGE,
        );
    }
    let left = rect.left() + 12.0;
    p.text(
        egui::pos2(left, rect.top() + 82.0),
        egui::Align2::LEFT_TOP,
        kind.name(),
        egui::FontId::monospace(15.0),
        ink(BRIGHT),
    );
    let facts = crate::i18n::aircraft_facts(kind);
    let line = match why {
        Some(why) => why.to_string(),
        // (An aircraft without notes yet: none.)
        None => {
            facts.first().copied().filter(|f| !f.starts_with("aircraft-")).unwrap_or("").to_string()
        }
    };
    let colour = if why.is_some() { AMBER.gamma_multiply(0.8) } else { DIM };
    let g = p.layout(line, egui::FontId::monospace(12.0), colour, w - 24.0);
    p.galley(egui::pos2(left, rect.top() + 104.0), g, colour);
    if enabled { resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() } else { false }
}

/// The route map under the steps (the editor itself at the ROUTE step): drag, add and remove
/// waypoints over the radars friendly aircraft have detected (and that still stand), the
/// campaign's places and air picture over the map. In a coop game the route is the group's
/// (locked while one of them flies it): what to send it.
#[allow(clippy::too_many_arguments)]
fn route_map(
    ui: &mut egui::Ui,
    c: &ActiveCampaign,
    sim: &stealth_sim::Sim,
    planner: &mut crate::planner::Planner,
    m: &stealth_mission::Mission,
    b: &mut Briefing,
    weather: &MapWeather,
    rect: egui::Rect,
) -> Option<Vec<crate::mission::RoutePoint>> {
    let now = ui.input(|i| i.time);
    let plan = c.state().plan(b.offer).filter(|_| c.role != Role::Local);
    let locked = plan.is_some_and(|p| !p.flying.is_empty());
    let route = b.route.get_or_insert_with(|| {
        planner.refit();
        crate::mission::PlannedRoute::of(m)
    });
    // The group's route (the others' changes), unless this player's own are on the way.
    if let Some(p) = plan
        && (locked || (!planner.dragging() && !b.dirty && now >= b.hold_until))
    {
        b.dirty = false;
        if p.route.is_empty() {
            route.points = crate::mission::PlannedRoute::of(m).points;
        } else if route.points != p.route {
            route.points = p.route.clone();
        }
    }
    let before = route.points.clone();
    let st = c.state();
    let known = |i: usize| st.site(i as u16).is_some_and(|x| x.spotted && x.alive());
    // The SAM sites are the editor's own rings.
    let mut pic = super::super::picture::picture(st, sim);
    pic.sams.clear();
    let draw = |painter: &egui::Painter, view: &MapView, layers: &MapLayers, tips: &mut MapTips| {
        for p in st.places.iter().filter(|p| shows_place(layers, p)) {
            let col = match p.owner {
                Side::Blue => FRIENDLY_BLUE,
                Side::Red => MISSILE_RED,
                Side::Neutral => DIM,
            };
            let c0 = view.to_screen(p.x as f32, p.y as f32);
            if map_draw::app6() {
                map_draw::app6_place(painter, c0, affiliation(p.owner), p.airfield.is_some(), 0.9);
            } else if p.airfield.is_some() {
                let r = egui::Rect::from_center_size(c0, egui::vec2(9.0, 9.0));
                painter.rect_stroke(r, 0.0, egui::Stroke::new(1.5, col), egui::StrokeKind::Middle);
            } else {
                painter.circle_stroke(c0, 4.5, egui::Stroke::new(1.5, col));
            }
            painter.text(
                c0 + egui::vec2(7.0, 7.0),
                egui::Align2::LEFT_TOP,
                &p.name,
                egui::FontId::monospace(12.0),
                col,
            );
            let production = super::super::logistics::production_of(st, p);
            if let Some(kind) = production {
                super::super::logistics::production_mark(painter, c0, 1.0, kind);
            }
            place_tip(tips, c0, p, st.clock, false, production);
        }
        super::super::picture::draw(painter, view, &pic, layers, tips, 1.0, None);
    };
    let legend = |ui: &mut egui::Ui, layers: &mut MapLayers| {
        place_legend(ui, layers);
        super::super::picture::legend(ui, layers, None);
    };
    planner.fly(kind_of(b.aircraft), true);
    let panel = 420.0_f32.min(rect.width() * 0.32);
    crate::planner::route_screen(
        ui,
        planner,
        m,
        route,
        &known,
        Some(crate::planner::Overlay { draw: &draw, legend: &legend }),
        *weather,
        rect,
        panel,
        &mut b.profile,
        !locked && b.step == Step::Route,
        b.step == Step::Route,
    );
    plan?;
    b.dirty |= route.points != before;
    // A dragged point goes a few times a second, everything else at once.
    if b.dirty && (!planner.dragging() || now - b.sent_at >= SEND_EVERY) {
        b.dirty = false;
        b.sent_at = now;
        b.hold_until = now + HOLD;
        return Some(route.points.clone());
    }
    None
}

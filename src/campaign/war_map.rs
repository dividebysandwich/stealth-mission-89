//! The campaign screen's one map, filling the screen under the top bar: who holds which place
//! and the front between them, the insertions, the air picture, the board's missions as
//! markers over their objectives, and in its AIR DEFENSE view the friendly sites with their
//! reach. Views (WAR, AIR DEFENSE, GROUND, WEATHER) are sets of layers, each fine tuned under
//! LAYERS; the zoom buttons are there for the headset, whose stick only zooms.
//!
//! A click on a mission marker opens the MISSIONS pane with it, on an air defence site the AIR
//! DEFENSE pane, on another unit its live view.

use bevy_egui::egui;
use stealth_campaign::{InsertionState, MissionOffer, Side};
use stealth_i18n::t;

use super::ActiveCampaign;
use super::ops::{Pane, affiliation, fit_map, kind_colour, place_legend, place_tip, shows_place};
use super::shell::{self, GAP, SQUARE};
use crate::map_draw::{
    self, FRIENDLY_BLUE, Layer, MISSILE_RED, MapLayers, MapTips, MapView, MapWeather,
};
use crate::menu::{AMBER, BRIGHT, DIM, GREEN, icon};

/// A view of the map: the layers it starts with (each its own, kept for the session).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MapPreset {
    #[default]
    War,
    AirDefence,
    Ground,
    Weather,
}

impl MapPreset {
    pub const ALL: [MapPreset; 4] =
        [MapPreset::War, MapPreset::AirDefence, MapPreset::Ground, MapPreset::Weather];

    /// Its layers' name in egui's memory.
    fn id(self) -> &'static str {
        match self {
            MapPreset::War => "war",
            MapPreset::AirDefence => "war-air-defence",
            MapPreset::Ground => "war-ground",
            MapPreset::Weather => "war-weather",
        }
    }

    /// What it hides at first.
    fn off(self) -> Vec<Layer> {
        use Layer::*;
        let weather = map_draw::WEATHER_LAYERS;
        let mut off = match self {
            MapPreset::War => vec![MissileRange, EngagementRange, DefenceReach],
            MapPreset::AirDefence => vec![
                Insertions,
                Armour,
                Guns,
                Patriots,
                PatriotReach,
                Nasams,
                NasamsReach,
                Gepards,
                MissileRange,
                EngagementRange,
                Missions,
                Front,
            ],
            MapPreset::Ground => vec![
                EnemyFighters,
                FriendlyAircraft,
                FriendlyHelicopters,
                Patriots,
                PatriotReach,
                Nasams,
                NasamsReach,
                Gepards,
                MissileRange,
                EngagementRange,
                DefenceReach,
            ],
            MapPreset::Weather => {
                vec![MissileRange, EngagementRange, DefenceReach, Insertions, Armour, Guns]
            }
        };
        if self != MapPreset::Weather {
            off.extend(weather);
        }
        off
    }

    pub fn label(self) -> &'static str {
        match self {
            MapPreset::War => t!("ops-view-war"),
            MapPreset::AirDefence => t!("ops-tab-air-defence"),
            MapPreset::Ground => t!("ops-view-ground"),
            MapPreset::Weather => t!("ops-view-weather"),
        }
    }

    fn icon(self) -> char {
        match self {
            MapPreset::War => icon::WAR,
            MapPreset::AirDefence => icon::AIR_DEFENCE,
            MapPreset::Ground => icon::GROUND,
            MapPreset::Weather => icon::WEATHER,
        }
    }
}

/// The board's missions at one objective: a marker over it.
struct Marker {
    at: egui::Pos2,
    rect: egui::Rect,
    offers: Vec<u32>,
}

/// The objective's name for its marker: the place, else the mission's own.
fn marker_name(st: &stealth_campaign::CampaignState, o: &MissionOffer) -> String {
    st.objective_place(&o.objective)
        .and_then(|p| st.place(p))
        .map_or_else(|| crate::mission::clean_text(&o.title.tr()), |p| p.name.clone())
        .to_uppercase()
}

/// Draws the markers, the selected mission's lit; returns where they are.
fn markers(
    painter: &egui::Painter,
    view: &MapView,
    st: &stealth_campaign::CampaignState,
    selected: Option<u32>,
) -> Vec<Marker> {
    // One marker per objective place (a city and its airfield together), else per target.
    let mut groups: Vec<(i32, i32, Option<u16>, Vec<&MissionOffer>)> = Vec::new();
    for o in &st.board {
        let place = st.objective_place(&o.objective);
        let (x, y) = place.and_then(|p| st.place(p)).map_or((o.x, o.y), |p| (p.x, p.y));
        match groups
            .iter_mut()
            .find(|g| (place.is_some() && g.2 == place) || (g.0 == x && g.1 == y))
        {
            Some(g) => g.3.push(o),
            None => groups.push((x, y, place, vec![o])),
        }
    }
    let font = egui::FontId::monospace(13.0);
    let mut out = Vec::new();
    for (x, y, _, offers) in groups {
        let at = view.to_screen(x as f32, y as f32);
        if !view.rect.expand(60.0).contains(at) {
            continue;
        }
        let lit = offers.iter().any(|o| Some(o.id) == selected);
        // Red for a defence (a raid coming, an intercept), else amber.
        let urgent = offers.iter().any(|o| kind_colour(o.kind) == MISSILE_RED);
        let colour = if urgent { MISSILE_RED } else { AMBER };
        let text = format!("{} · {}", marker_name(st, offers[0]), offers.len());
        let g = painter.layout_no_wrap(text, font.clone(), egui::Color32::BLACK);
        let stem = 18.0;
        let size = g.size() + egui::vec2(34.0, 10.0);
        let mut rect =
            egui::Rect::from_min_size(egui::pos2(at.x - size.x / 2.0, at.y - stem - size.y), size);
        // Over another marker: stacked above it.
        while let Some(o) = out.iter().find(|m: &&Marker| m.rect.expand(2.0).intersects(rect)) {
            rect = rect.translate(egui::vec2(0.0, o.rect.top() - rect.bottom() - 4.0));
        }
        painter.line_segment([at, rect.center_bottom()], egui::Stroke::new(2.0, colour));
        painter.rect_filled(rect.expand(2.0), 2.0, egui::Color32::BLACK);
        painter.rect_filled(rect, 2.0, if lit { BRIGHT } else { colour });
        if lit {
            painter.rect_stroke(
                rect.expand(3.0),
                3.0,
                egui::Stroke::new(3.0, colour),
                egui::StrokeKind::Outside,
            );
        }
        let icon = if urgent { icon::DEFEND } else { icon::MISSIONS };
        shell::paint_icon(
            painter,
            egui::pos2(rect.left() + 14.0, rect.center().y),
            icon,
            15.0,
            egui::Color32::BLACK,
        );
        painter.galley(
            egui::pos2(rect.left() + 26.0, rect.center().y - g.size().y / 2.0),
            g,
            egui::Color32::BLACK,
        );
        out.push(Marker { at, rect, offers: offers.iter().map(|o| o.id).collect() });
    }
    out
}

/// The mission markers' legend line.
fn markers_legend(ui: &mut egui::Ui, layers: &mut MapLayers) {
    layers.legend(ui, Layer::Missions, t!("ops-legend-missions"), |p, c0| {
        let r = egui::Rect::from_center_size(c0, egui::vec2(16.0, 9.0));
        p.rect_filled(r, 1.0, AMBER);
    });
}

/// The map: everything on it, the clicks on it, and its controls over it. `pane`: the width the
/// open pane covers at its left (what is selected is kept clear of it).
#[allow(clippy::too_many_arguments)]
pub(super) fn map(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    sim: &stealth_sim::Sim,
    unit_view: &mut super::UnitView,
    planner: &mut crate::planner::Planner,
    weather: &MapWeather,
    scenery: Option<&crate::scenery::Scenery>,
    sites: &[super::air_defence::Site],
    pane: f32,
) {
    let rect = ui.max_rect();
    let resp = ui.interact(rect, ui.id().with("war map"), egui::Sense::click_and_drag());
    ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);
    fit_map(c, rect);
    // What shows of the map beside the pane.
    let clear = egui::Rect::from_min_max(egui::pos2(rect.left() + pane, rect.top()), rect.max);
    // Something picked in a list: brought into view beside the pane.
    if let Some(f) = c.ui.focus.take() {
        let view = MapView { rect, centre: c.ui.centre, scale: c.ui.scale };
        if !clear.shrink(80.0).contains(view.to_screen(f.x, f.y)) {
            c.ui.centre = f - (clear.center() - rect.center()) * c.ui.scale;
        }
    }
    // The zoom buttons' (around what shows beside the pane).
    if let Some(z) = c.ui.zoom.take() {
        let view = MapView { rect, centre: c.ui.centre, scale: c.ui.scale };
        let mid = view.to_map(clear.center());
        c.ui.scale = (c.ui.scale * z).clamp(1.0, 64.0);
        c.ui.centre = mid - (clear.center() - rect.center()) * c.ui.scale;
    }
    let view = MapView { rect, centre: c.ui.centre, scale: c.ui.scale };
    let (mut centre, mut scale) = (c.ui.centre, c.ui.scale);
    view.pan_zoom(ui, &resp, &mut centre, &mut scale);
    (c.ui.centre, c.ui.scale) = (centre, scale);
    let preset = c.ui.preset;
    let mut layers = MapLayers::of(ui.ctx(), preset.id(), &preset.off());
    let painter = ui.painter_at(rect);
    planner.draw_map(&painter, &view, &c.mission.theater_code.to_ascii_lowercase());
    if preset == MapPreset::AirDefence {
        painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(140));
    }
    map_draw::weather(&painter, &view, "war", weather, &layers);
    let (parked, selected, defence) = (c.parked, c.ui.selected, c.ui.defence);
    let test_taken = &mut c.ui.test_taken;
    let st = c.host.as_ref().map(|h| &h.state).or(c.copy.as_ref()).expect("campaign");
    let at = |x: i32, y: i32| view.to_screen(x as f32, y as f32);
    // The war map's symbols twice the size of the other maps'.
    let k = 2.0 * map_draw::BIG;
    let mut tips = MapTips::grown(k);
    super::front::draw(&painter, &view, st, &layers);
    // Insertion routes.
    for ins in st.insertions.iter().filter(|_| layers.shows(Layer::Insertions)) {
        if !matches!(ins.state, InsertionState::Pending { .. } | InsertionState::Enroute) {
            continue;
        }
        if let (Some(a), Some(b)) = (st.site(ins.from), st.site(ins.target)) {
            let pts = [at(a.x, a.y), at(b.x, b.y)];
            painter.extend(egui::Shape::dashed_line(
                &pts,
                egui::Stroke::new(1.5, FRIENDLY_BLUE),
                6.0,
                4.0,
            ));
        }
    }
    // The places.
    let dim = preset == MapPreset::AirDefence;
    for p in st.places.iter().filter(|p| shows_place(&layers, p)) {
        let col = match p.owner {
            Side::Blue => FRIENDLY_BLUE,
            Side::Red => MISSILE_RED,
            Side::Neutral => DIM,
        };
        let c0 = at(p.x, p.y);
        let production = super::logistics::production_of(st, p);
        if dim {
            painter.circle_filled(c0, 2.5 * map_draw::BIG, col.gamma_multiply(0.5));
        } else {
            let (owner, airfield) = (affiliation(p.owner), p.airfield.is_some());
            map_draw::scaled(&painter, c0, k, |pa, c| {
                map_draw::place_symbol(pa, c, owner, airfield, true);
            });
            if p.intel(st.clock) {
                let r = map_draw::halo(10.0) * k;
                painter.circle_stroke(c0, r, egui::Stroke::new(1.0, BRIGHT));
            }
            if let Some(kind) = production {
                super::logistics::production_mark(&painter, c0, k, kind);
            }
            if 1.0 / view.scale > 0.012 || p.owner != Side::Neutral {
                painter.text(
                    c0 + egui::vec2(7.0, -7.0) * k,
                    egui::Align2::LEFT_BOTTOM,
                    &p.name,
                    egui::FontId::monospace(12.0),
                    col,
                );
            }
        }
        place_tip(&mut tips, c0, p, st.clock, true, production);
    }
    super::front::recent(&painter, &view, st, &layers, &mut tips, k);
    // What the friendly side sees: its aircraft, enemy fighters on radar, known defences.
    let pic = super::picture::picture(st, sim);
    let watched = unit_view.target.map(|unit| (unit, blink(ui.ctx())));
    super::picture::draw(&painter, &view, &pic, &layers, &mut tips, k, watched);
    if layers.shows(Layer::DefenceReach) {
        super::air_defence::overlay(&painter, &view, sites, defence, &mut tips, map_draw::BIG);
    }
    test_watch(sim, unit_view, &pic, test_taken);
    super::designate::waiting(&painter, &view, st);
    if pic.player.is_none()
        && let Some(b) = parked.and_then(|b| st.site(b as u16))
    {
        painter.circle_stroke(at(b.x, b.y), 8.0 * k, egui::Stroke::new(1.5, GREEN));
        tips.add(at(b.x, b.y), t!("mt-parked"), "", GREEN).symbol(|p, c| {
            p.circle_stroke(c, 8.0, egui::Stroke::new(1.5, GREEN));
        });
    }
    // The selected mission's target, and its way from its base.
    if let Some(o) = selected.and_then(|id| st.board.iter().find(|o| o.id == id)) {
        let colour = kind_colour(o.kind);
        let base = c.ui.base.unwrap_or(o.base);
        if let Some(b) = st.site(base) {
            let pts = [at(b.x, b.y), at(o.x, o.y)];
            painter.extend(egui::Shape::dashed_line(
                &pts,
                egui::Stroke::new(2.5, colour),
                10.0,
                6.0,
            ));
        }
        painter.circle_stroke(at(o.x, o.y), 14.0 * k, egui::Stroke::new(3.0, colour));
    }
    let marks = if layers.shows(Layer::Missions) {
        markers(&painter, &view, st, selected)
    } else {
        Vec::new()
    };
    // The controls over the map: its views at the top left (beside the pane), LAYERS and the
    // zoom at the top right.
    let mut over: Vec<egui::Rect> = Vec::new();
    let presets = egui::Rect::from_min_size(
        clear.left_top() + egui::vec2(GAP, GAP),
        egui::vec2((clear.width() - 2.0 * GAP - 300.0).clamp(200.0, 560.0), 48.0),
    );
    let mut picked_preset = None;
    ui.scope_builder(egui::UiBuilder::new().max_rect(presets), |ui| {
        let options: Vec<(MapPreset, Option<char>, &str)> =
            MapPreset::ALL.iter().map(|&p| (p, Some(p.icon()), p.label())).collect();
        picked_preset = shell::segmented(ui, preset, &options, 48.0);
        over.push(ui.min_rect());
    });
    let controls = egui::Rect::from_min_max(
        egui::pos2(rect.right() - GAP - 300.0, rect.top() + GAP),
        egui::pos2(rect.right() - GAP, rect.bottom() - GAP),
    );
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(controls).layout(egui::Layout::top_down(egui::Align::Max)),
        |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            if shell::button(
                ui,
                Some(icon::LAYERS),
                t!("planner-layers"),
                layers.panel,
                SQUARE,
                None,
            )
            .clicked()
            {
                layers.panel = !layers.panel;
            }
            if layers.panel {
                shell::overlay_frame().show(ui, |ui| {
                    ui.set_width(280.0);
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        egui::ScrollArea::vertical().max_height(rect.height() * 0.6).show(
                            ui,
                            |ui| {
                                map_draw::symbology_switch(ui);
                                ui.add_space(4.0);
                                super::front::legend(ui, &mut layers);
                                markers_legend(ui, &mut layers);
                                place_legend(ui, &mut layers);
                                layers.legend(
                                    ui,
                                    Layer::Insertions,
                                    t!("ops-legend-insertion"),
                                    |p, c0| {
                                        p.extend(egui::Shape::dashed_line(
                                            &[c0 - egui::vec2(8.0, 0.0), c0 + egui::vec2(8.0, 0.0)],
                                            egui::Stroke::new(1.5, FRIENDLY_BLUE),
                                            6.0,
                                            4.0,
                                        ));
                                    },
                                );
                                layers.legend(
                                    ui,
                                    Layer::DefenceReach,
                                    t!("ops-legend-defence-reach"),
                                    |p, c0| {
                                        p.circle(
                                            c0,
                                            6.0,
                                            FRIENDLY_BLUE.gamma_multiply(0.2),
                                            egui::Stroke::new(1.2, FRIENDLY_BLUE),
                                        );
                                    },
                                );
                                super::picture::legend(ui, &mut layers, Some(&pic));
                                map_draw::weather_legend(ui, &mut layers, weather.snow);
                            },
                        );
                    });
                });
            } else {
                for (glyph, tip, z) in [
                    (icon::ZOOM_IN, t!("map-zoom-in"), Some(0.7)),
                    (icon::ZOOM_OUT, t!("map-zoom-out"), Some(1.0 / 0.7)),
                    (icon::FIT, t!("map-fit"), None),
                ] {
                    if shell::square_button(ui, glyph, tip).clicked() {
                        match z {
                            Some(z) => c.ui.zoom = Some(z),
                            None => c.ui.fitted = false,
                        }
                    }
                }
            }
            over.push(ui.min_rect());
        },
    );
    // Placing a new site: what to do, and the way out (no right click in the headset).
    if let Some(kind) = c.ui.designate {
        let w = 640.0_f32.min(clear.width() - 2.0 * GAP);
        let r = egui::Rect::from_min_size(
            egui::pos2(clear.center().x - w / 2.0, clear.top() + GAP + 60.0),
            egui::vec2(w, 64.0),
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(r), |ui| {
            shell::overlay_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    shell::paint_icon(
                        ui.painter(),
                        ui.cursor().left_center() + egui::vec2(10.0, 0.0),
                        icon::SINGLE_MISSION,
                        20.0,
                        BRIGHT,
                    );
                    ui.add_space(26.0);
                    let text = t!("ds-banner", kind = super::designate::kind_label(kind));
                    ui.label(egui::RichText::new(text).size(16.0).color(BRIGHT));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if shell::button(ui, Some(icon::CANCEL), t!("cancel"), false, 44.0, None)
                            .clicked()
                        {
                            c.ui.designate = None;
                        }
                    });
                });
            });
            over.push(ui.min_rect());
        });
    }
    if let Some(p) = picked_preset {
        c.ui.preset = p;
        c.ui.preset_before = None;
    }
    // What is under the pointer (not the controls): its tooltip; a click picks it.
    let designating = c.ui.designate.is_some();
    let hover = resp.hover_pos().filter(|&p| !over.iter().any(|r| r.contains(p)));
    if !designating && let Some(pos) = hover {
        let mark =
            marks.iter().find(|m| m.rect.expand(4.0).contains(pos) || m.at.distance(pos) < 14.0);
        let site = super::air_defence::hit(&view, sites, pos, map_draw::BIG * 2.0)
            .filter(|_| layers.shows(Layer::DefenceReach) || preset != MapPreset::AirDefence);
        let picked = super::picture::pick(&view, &pic, pos, k);
        if let Some(m) = mark {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            tips.show(&resp, Some(t!("mt-click-missions")));
            if resp.clicked() {
                let keep = c.ui.selected.filter(|s| m.offers.contains(s));
                c.ui.selected = keep.or(m.offers.first().copied());
                c.ui.base = None;
                c.ui.error = None;
                c.ui.open(Some(Pane::Missions));
            }
        } else if let Some(unit) = site {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            tips.show(&resp, Some(t!("mt-click-select")));
            if resp.clicked() {
                c.ui.defence = Some(unit);
                c.ui.defence_scroll = true;
                c.ui.open(Some(Pane::AirDefence));
            }
        } else {
            tips.show(&resp, picked.as_ref().map(|_| t!("mt-click-watch")));
            if let Some((unit, name)) = picked {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    unit_view.watch(unit, name);
                }
            }
        }
    }
    layers.keep(ui.ctx(), preset.id());
    super::designate::pointer(ui, &painter, &view, &resp, c, sim, scenery);
}

/// The colour of the selection's outline now: black to white and back, once a second.
fn blink(ctx: &egui::Context) -> egui::Color32 {
    ctx.request_repaint();
    let t = ctx.input(|i| i.time) as f32;
    let white = 0.5 - 0.5 * (t * std::f32::consts::TAU).cos();
    egui::Color32::from_gray((white * 255.0).round() as u8)
}

/// Test aid: `--campaign-watch` watches the first friendly aircraft, `--campaign-watch
/// armour` the first tank of a column on the move (`fp2`, `zsu`, `nasams` others).
fn test_watch(
    sim: &stealth_sim::Sim,
    unit_view: &mut super::UnitView,
    pic: &super::picture::Picture,
    test_taken: &mut bool,
) {
    use super::unit_view::UnitRef;
    if unit_view.target.is_some() || *test_taken {
        return;
    }
    let watch = std::env::args().skip_while(|a| a != "--campaign-watch").nth(1);
    let ground = |kind: stealth_sim::aaa::GroundKind, blue: bool, moving: bool| {
        (0..sim.units().len()).find(|&k| {
            sim.units()[k].kind == kind
                && !sim.units()[k].destroyed
                && sim.aaa_blue(k) == blue
                && (!moving || sim.unit_moving(k))
        })
    };
    let (unit, name, orbit) = match watch.as_deref() {
        Some("fp2") => match pic.friendly.iter().find(|a| a.spec == stealth_sim::uav::FP2_SPEC) {
            Some(a) => (
                UnitRef::Flight { index: a.unit, spec: a.spec },
                sim.aircraft_name(a.spec).to_string(),
                true,
            ),
            None => return,
        },
        Some("zsu") => match ground(stealth_sim::aaa::GroundKind::Zsu, false, true) {
            Some(k) => (UnitRef::Aaa(k), "ZSU-23-4".to_string(), true),
            None => return,
        },
        Some("armour") => match pic.armour.iter().find(|v| v.heading.is_some()) {
            Some(v) => (UnitRef::Aaa(v.index), v.kind.name().to_string(), false),
            None => return,
        },
        Some("nasams") => match ground(stealth_sim::aaa::GroundKind::NasamsLauncher, true, false) {
            Some(k) => (UnitRef::Aaa(k), "NASAMS".to_string(), true),
            None => return,
        },
        _ if std::env::args().any(|a| a == "--campaign-watch") => match pic.friendly.first() {
            Some(a) => (
                UnitRef::Flight { index: a.unit, spec: a.spec },
                sim.aircraft_name(a.spec).to_string(),
                false,
            ),
            None => return,
        },
        _ => return,
    };
    *test_taken = true;
    unit_view.watch(unit, name);
    if orbit {
        unit_view.test_orbit();
    }
}

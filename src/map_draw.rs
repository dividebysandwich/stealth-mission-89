//! Theater map drawing shared by the mission planner and the debrief: the view's projection,
//! the map picture, the threat and airfield symbols, the weather, and the layers a player can
//! hide (the checkboxes of a map's legend lines), the symbology they draw units in (the game's
//! own symbols or APP-6), and the tooltip of what is under the pointer.

use std::collections::{BTreeSet, HashSet};
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy_egui::egui;

use stealth_mission::weather::{CloudKind, STORM_CELL};
use stealth_mission::world::flags as F;
use stealth_mission::{Mission, Weather};

use crate::app6::{self, Affiliation, Dimension, Icon};
use crate::menu::DIM;

pub const MISSILE_RED: egui::Color32 = egui::Color32::from_rgb(255, 70, 50);
pub const UAV_PINK: egui::Color32 = egui::Color32::from_rgb(255, 120, 200);
pub const FRIENDLY_BLUE: egui::Color32 = egui::Color32::from_rgb(90, 160, 255);
pub const ENGAGE_YELLOW: egui::Color32 = egui::Color32::from_rgb(255, 220, 40);
const BOLT_YELLOW: egui::Color32 = egui::Color32::from_rgb(255, 235, 90);
/// Stratus and cumulonimbus on the map: the cloud ramp's light and dark greys.
const CLOUD_GREYS: [[f32; 3]; 2] = [[215.0, 218.0, 225.0], [120.0, 124.0, 135.0]];
/// Rain and snow from light to heavy (the weather radar's colours).
const RAIN_RAMP: [[f32; 3]; 3] = [[110.0, 235.0, 110.0], [240.0, 220.0, 40.0], [230.0, 50.0, 40.0]];
const SNOW_RAMP: [[f32; 3]; 3] =
    [[160.0, 210.0, 255.0], [80.0, 130.0, 255.0], [200.0, 100.0, 255.0]];

/// A view of the map: map units (x east, y south) around `centre`, `scale` map units per
/// screen point, drawn into `rect`.
#[derive(Debug, Clone, Copy)]
pub struct MapView {
    pub rect: egui::Rect,
    pub centre: egui::Vec2,
    pub scale: f32,
}

impl MapView {
    pub fn to_screen(&self, x: f32, y: f32) -> egui::Pos2 {
        self.rect.center() + (egui::vec2(x, y) - self.centre) / self.scale
    }

    pub fn to_map(&self, p: egui::Pos2) -> egui::Vec2 {
        self.centre + (p - self.rect.center()) * self.scale
    }

    /// Centre and scale that show the box `lo..hi` (map units) with a margin.
    pub fn fit(rect: egui::Rect, lo: egui::Vec2, hi: egui::Vec2) -> (egui::Vec2, f32) {
        let scale =
            ((hi.x - lo.x) / rect.width()).max((hi.y - lo.y) / rect.height()).max(4.0) * 1.4;
        ((lo + hi) / 2.0, scale)
    }

    /// Wheel zoom around the pointer and drag to pan (map units are written back).
    pub fn pan_zoom(
        &self,
        ui: &egui::Ui,
        resp: &egui::Response,
        centre: &mut egui::Vec2,
        scale: &mut f32,
    ) {
        if let Some(pos) = resp.hover_pos() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let before = self.to_map(pos);
                *scale = (*scale * (-scroll * 0.002).exp()).clamp(1.0, 64.0);
                *centre = before - (pos - self.rect.center()) * *scale;
            }
        }
        if resp.dragged() {
            *centre -= resp.drag_delta() * *scale;
        }
    }
}

/// The maps' symbology (`maps.json` in the player's config): APP-6 or, by default, the game's
/// own symbols.
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, Copy, PartialEq)]
#[serde(default)]
struct MapPrefs {
    app6: bool,
}

const PREFS_FILE: &str = "maps.json";

static APP6: LazyLock<AtomicBool> = LazyLock::new(|| {
    AtomicBool::new(crate::prefs::load_json::<MapPrefs>(PREFS_FILE).unwrap_or_default().app6)
});

/// The maps draw units in APP-6.
pub fn app6() -> bool {
    APP6.load(Ordering::Relaxed)
}

/// The maps' SYMBOLS switch: CLASSIC or APP-6, for every map (and kept between runs).
pub fn symbology_switch(ui: &mut egui::Ui) {
    let mut on = app6();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Symbols").small().color(crate::menu::BRIGHT));
        ui.selectable_value(&mut on, false, egui::RichText::new("CLASSIC").small());
        ui.selectable_value(&mut on, true, egui::RichText::new("APP-6").small());
    });
    if on != app6() {
        APP6.store(on, Ordering::Relaxed);
        if let Err(e) = crate::prefs::save_json(PREFS_FILE, &MapPrefs { app6: on }) {
            bevy::log::warn!("{PREFS_FILE}: {e:#}");
        }
    }
}

/// How far out a ring around a unit's symbol goes (a ring of `r` around the game's own).
pub fn halo(r: f32) -> f32 {
    if app6() { r + 6.0 } else { r }
}

/// A place in APP-6: an installation (an airfield with its aircraft), its frame by owner, `k`
/// its size.
pub fn app6_place(
    painter: &egui::Painter,
    c: egui::Pos2,
    side: Affiliation,
    airfield: bool,
    k: f32,
) {
    let icon = if airfield { Icon::FixedWing } else { Icon::None };
    app6::Symbol::new(side, Dimension::Installation, icon).draw(painter, c, k, 1.0);
}

/// The map picture (the whole theater), or a note while it is being prepared.
pub fn picture(painter: &egui::Painter, view: &MapView, tex: Option<&egui::TextureHandle>) {
    painter.rect_filled(view.rect, 0.0, egui::Color32::BLACK);
    match tex {
        Some(tex) => {
            let r = egui::Rect::from_min_max(
                view.to_screen(0.0, 0.0),
                view.to_screen(0x8000 as f32, 0x8000 as f32),
            );
            painter.image(
                tex.id(),
                r,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
        None => {
            painter.text(
                view.rect.center(),
                egui::Align2::CENTER_CENTER,
                "LOADING MAP...",
                egui::FontId::monospace(18.0),
                DIM,
            );
        }
    }
}

/// A SAM site's missile range: a dotted red circle.
pub fn missile_circle(c: egui::Pos2, r: f32, alpha: f32) -> Vec<egui::Shape> {
    let n = ((r * std::f32::consts::TAU / 2.0) as usize).clamp(24, 720);
    let pts: Vec<egui::Pos2> = (0..=n)
        .map(|k| c + egui::Vec2::angled(k as f32 / n as f32 * std::f32::consts::TAU) * r)
        .collect();
    egui::Shape::dashed_line(
        &pts,
        egui::Stroke::new(1.4, MISSILE_RED.gamma_multiply(alpha * 0.8)),
        3.0,
        4.0,
    )
}

/// Where a unit opens fire: a thin yellow circle.
pub fn engagement_circle(c: egui::Pos2, r: f32, alpha: f32) -> egui::Shape {
    let stroke = egui::Stroke::new(1.2, ENGAGE_YELLOW.gamma_multiply(alpha * 0.9));
    egui::Shape::circle_stroke(c, r, stroke)
}

/// A site: a red dot (APP-6: a hostile air defence missile unit) with its name once zoomed in
/// far enough.
/// How much larger the campaign's war and air defence maps draw their symbols.
pub const BIG: f32 = 1.5;

/// A symbol drawn by `draw` at `c`, then scaled `k` times about `c` (its strokes with it).
pub fn scaled(
    painter: &egui::Painter,
    c: egui::Pos2,
    k: f32,
    draw: impl FnOnce(&egui::Painter, egui::Pos2),
) {
    if k == 1.0 {
        return draw(painter, c);
    }
    let layer = painter.layer_id();
    let start = painter.ctx().graphics_mut(|g| g.entry(layer).next_idx());
    draw(painter, c);
    painter.ctx().graphics_mut(|g| {
        let list = g.entry(layer);
        let end = list.next_idx();
        let t = egui::emath::TSTransform::new(c.to_vec2() * (1.0 - k), k);
        for i in start.0..end.0 {
            list.mutate_shape(egui::layers::ShapeIdx(i), |c| c.shape.transform(t));
        }
    });
}

/// A SAM site's symbol and name, `k` times the symbol's size.
pub fn site(
    painter: &egui::Painter,
    view: &MapView,
    c: egui::Pos2,
    name: &str,
    alpha: f32,
    k: f32,
) {
    scaled(painter, c, k, |p, c| site_symbol(p, c, alpha));
    if 1.0 / view.scale > 0.02 {
        let off = if app6() { egui::vec2(9.0, -8.0) } else { egui::vec2(5.0, -5.0) };
        painter.text(
            c + off * k,
            egui::Align2::LEFT_BOTTOM,
            name,
            egui::FontId::monospace(11.0),
            MISSILE_RED.gamma_multiply(alpha),
        );
    }
}

/// A site's symbol, without its name.
pub fn site_symbol(painter: &egui::Painter, c: egui::Pos2, alpha: f32) {
    if app6() {
        app6::Symbol::new(Affiliation::Hostile, Dimension::Ground, Icon::AirDefenceMissile)
            .draw(painter, c, 0.8, alpha);
    } else {
        painter.circle_filled(c, 3.0, MISSILE_RED.gamma_multiply(alpha));
    }
}

/// Draws a map symbol centred on a point, at its size on the map.
type DrawSymbol = Box<dyn Fn(&egui::Painter, egui::Pos2)>;

/// A thing drawn on a map, for the tooltip of what is under the pointer: where its symbol is
/// (screen), how close the pointer comes to it, what it is, its name, more lines, and its
/// symbol (shown beside them).
pub struct Tip {
    at: egui::Pos2,
    reach: f32,
    kind: String,
    name: String,
    colour: egui::Color32,
    lines: Vec<String>,
    symbol: Option<DrawSymbol>,
}

impl Tip {
    /// Its symbol, drawn by `draw` as on the map (the tooltip enlarges it).
    pub fn symbol(&mut self, draw: impl Fn(&egui::Painter, egui::Pos2) + 'static) -> &mut Self {
        self.symbol = Some(Box::new(draw));
        self
    }

    /// A line under the name.
    pub fn line(&mut self, text: impl Into<String>) -> &mut Self {
        self.lines.push(text.into());
        self
    }

    /// The pointer is over it this close to its centre (screen points): a town's area, wider
    /// than a symbol.
    pub fn reach(&mut self, r: f32) -> &mut Self {
        self.reach = r;
        self
    }
}

/// Most things one tooltip lists (crowded symbols).
const TIPS_MAX: usize = 5;

/// What a map has drawn, for the tooltip of what is under the pointer: every zoomable map
/// (the planner's, the debrief's, the campaign's war and air defence maps).
pub struct MapTips(Vec<Tip>, f32);

impl Default for MapTips {
    fn default() -> Self {
        Self(Vec::new(), 1.0)
    }
}

impl MapTips {
    /// For a map drawing its symbols `k` times their size: each reaches as much further.
    pub fn grown(k: f32) -> Self {
        Self(Vec::new(), k)
    }

    /// A symbol drawn at `at`: `kind` what it is ("Airfield"), `name` which one (may be
    /// empty), `colour` its side's.
    pub fn add(
        &mut self,
        at: egui::Pos2,
        kind: impl Into<String>,
        name: impl Into<String>,
        colour: egui::Color32,
    ) -> &mut Tip {
        self.0.push(Tip {
            at,
            reach: halo(10.0) * self.1,
            kind: kind.into(),
            name: name.into(),
            colour,
            lines: Vec::new(),
            symbol: None,
        });
        self.0.last_mut().expect("just pushed")
    }

    /// The things under `pos`, nearest first.
    fn under(&self, pos: egui::Pos2) -> Vec<&Tip> {
        let mut under: Vec<(f32, &Tip)> =
            self.0.iter().map(|t| (t.at.distance(pos), t)).filter(|(d, t)| *d <= t.reach).collect();
        under.sort_by(|a, b| a.0.total_cmp(&b.0));
        under.into_iter().map(|(_, t)| t).collect()
    }

    /// The tooltip of what is under the pointer on the map `resp` (none while a button is held:
    /// a pan, a drag); `hint` a last line (what a click does).
    pub fn show(&self, resp: &egui::Response, hint: Option<&str>) {
        let Some(pos) = resp.hover_pos() else { return };
        if resp.ctx.input(|i| i.pointer.any_down()) {
            return;
        }
        let under = self.under(pos);
        if under.is_empty() {
            return;
        }
        egui::Tooltip::for_widget(resp).at_pointer().gap(12.0).show(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            // As wide as its longest line: near the screen's edge egui would wrap the words.
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            // The symbols' column: each as high as the kind and name lines (wider for the APP-6
            // frames).
            let shown = &under[..under.len().min(TIPS_MAX)];
            let high = ui.text_style_height(&egui::TextStyle::Small)
                + ui.spacing().item_spacing.y
                + ui.text_style_height(&egui::TextStyle::Body);
            let column = shown
                .iter()
                .any(|t| t.symbol.is_some())
                .then(|| egui::vec2(if app6() { high * 1.5 } else { high }, high));
            // The lines between the things, drawn once the width is known: a separator would
            // take the width the tooltip had last frame (a wider one's) and keep it.
            let mut rules = Vec::new();
            for (k, t) in shown.iter().enumerate() {
                if k > 0 {
                    rules.push(ui.cursor().top() + 3.0);
                    ui.add_space(6.0);
                }
                ui.horizontal_top(|ui| {
                    if let Some(size) = column {
                        let (_, rect) = ui.allocate_space(size);
                        if let Some(draw) = &t.symbol {
                            enlarged(ui, rect, draw);
                        }
                    }
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&t.kind).small().color(t.colour));
                        if !t.name.is_empty() {
                            let name = egui::RichText::new(&t.name).strong();
                            ui.label(name.color(crate::menu::BRIGHT));
                        }
                        for line in &t.lines {
                            ui.label(egui::RichText::new(line).small().color(DIM));
                        }
                    });
                });
            }
            if under.len() > TIPS_MAX {
                let more = format!("+{} more", under.len() - TIPS_MAX);
                ui.label(egui::RichText::new(more).small().color(DIM));
            }
            if let Some(hint) = hint {
                ui.add_space(3.0);
                ui.label(egui::RichText::new(hint).small().color(crate::menu::BRIGHT));
            }
            let (x, stroke) =
                (ui.min_rect().x_range(), ui.visuals().widgets.noninteractive.bg_stroke);
            for y in rules {
                ui.painter().hline(x, y, stroke);
            }
        });
    }
}

/// A map symbol drawn by `draw` in `rect` of a tooltip: drawn at its size on the map, then
/// scaled up (its strokes with it) to fill the rect.
fn enlarged(ui: &egui::Ui, rect: egui::Rect, draw: &DrawSymbol) {
    let layer = ui.layer_id();
    let start = ui.ctx().graphics_mut(|g| g.entry(layer).next_idx());
    draw(ui.painter(), rect.center());
    ui.ctx().graphics_mut(|g| {
        let list = g.entry(layer);
        let end = list.next_idx();
        let bounds = list
            .all_entries()
            .skip(start.0)
            .take(end.0 - start.0)
            .fold(egui::Rect::NOTHING, |b, c| b.union(c.shape.visual_bounding_rect()));
        if !bounds.is_positive() {
            return;
        }
        let k = (rect.width() / bounds.width()).min(rect.height() / bounds.height());
        let shift = rect.center().to_vec2() - bounds.center().to_vec2() * k;
        let t = egui::emath::TSTransform::new(shift, k);
        // The shapes only: the clip rect stays the tooltip's.
        for i in start.0..end.0 {
            list.mutate_shape(egui::layers::ShapeIdx(i), |c| c.shape.transform(t));
        }
    });
}

/// A theater object's side by its `flags`.
pub fn affiliation_of(flags: i16) -> Affiliation {
    if flags & F::FRIENDLY != 0 {
        Affiliation::Friend
    } else if flags & F::NEUTRAL != 0 {
        Affiliation::Neutral
    } else {
        Affiliation::Hostile
    }
}

/// A side's name and colour on the maps.
pub fn side(side: Affiliation) -> (&'static str, egui::Color32) {
    match side {
        Affiliation::Friend => ("Friendly", FRIENDLY_BLUE),
        Affiliation::Neutral => ("Neutral", DIM),
        Affiliation::Hostile => ("Enemy", MISSILE_RED),
    }
}

/// A theater object's name: its place's, else its shape's.
pub fn object_name(m: &Mission, i: usize) -> String {
    let Some(o) = m.world.objects.get(i) else { return String::new() };
    let place = m.world.name(o.name as i32);
    let name = if place.is_empty() { m.world.name(o.shape() as i32) } else { place };
    crate::mission::clean_text(name)
}

/// Places a town's tooltip reaches from its centre (map units, about 3 km).
const TOWN_REACH: f32 = 192.0;

/// The mission's named places the map draws no symbol for (the towns of the map picture, the
/// depots, the missile sites): their tooltips only, with a city's symbol.
pub fn landmark_tips(view: &MapView, m: &Mission, tips: &mut MapTips) {
    let w = &m.world;
    for (i, o) in w.objects.iter().enumerate().skip(w.ground_unit_count as usize) {
        let (x, y) = (o.x as u16 as f32, o.y as u16 as f32);
        if o.flags & (F::AIRFIELD | F::SHIP | F::CARRIER) != 0
            || (x, y) == (0.0, 0.0)
            || w.name(o.name as i32).is_empty()
        {
            continue;
        }
        let kind = match w.name(o.shape() as i32) {
            "Town" => "Town",
            "Village" => "Village",
            "Depot" => "Depot",
            "SS Missile" => "Missile site",
            _ => "City",
        };
        let owner = affiliation_of(o.flags);
        let (name, colour) = side(owner);
        tips.add(view.to_screen(x, y), kind, object_name(m, i), colour)
            .reach((TOWN_REACH / view.scale).clamp(8.0, 40.0))
            .line(name)
            .symbol(move |p, c| place_symbol(p, c, owner, false, false));
    }
}

/// The mission's airfields: blue friendly, dim neutral, red hostile squares; and their `tips`.
pub fn airfields(painter: &egui::Painter, view: &MapView, m: &Mission, tips: &mut MapTips) {
    for (i, o) in m.world.objects.iter().enumerate().filter(|(_, o)| o.flags & F::AIRFIELD != 0) {
        let p = view.to_screen(o.x as u16 as f32, o.y as u16 as f32);
        let kind = if o.flags & F::CARRIER != 0 { "Aircraft carrier" } else { "Airfield" };
        let owner = affiliation_of(o.flags);
        let (name, colour) = side(owner);
        place_symbol(painter, p, owner, true, false);
        let tip = tips.add(p, kind, object_name(m, i), colour).line(name);
        tip.symbol(move |p, c| place_symbol(p, c, owner, true, false));
        if i as i16 == m.takeoff.object_index {
            tip.line("Your takeoff base");
        }
        if i as i16 == m.landing.object_index {
            tip.line("Your landing base");
        }
    }
}

/// A place in its side's colour: an airfield a square, a city a ring, `filled` (the campaign's
/// war map) or not; APP-6: an installation (with aircraft for an airfield).
pub fn place_symbol(
    painter: &egui::Painter,
    p: egui::Pos2,
    owner: Affiliation,
    airfield: bool,
    filled: bool,
) {
    if app6() {
        return app6_place(painter, p, owner, airfield, 0.9);
    }
    let col = side(owner).1;
    let stroke = egui::Stroke::new(1.5, col);
    match (airfield, filled) {
        (true, true) => {
            let r = egui::Rect::from_center_size(p, egui::vec2(10.0, 10.0));
            painter.rect_filled(r, 0.0, col.gamma_multiply(0.5));
            painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Middle);
        }
        (true, false) => {
            let r = egui::Rect::from_center_size(p, egui::vec2(8.0, 8.0));
            painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Middle);
        }
        (false, true) => {
            painter.circle_filled(p, 5.0, col.gamma_multiply(0.5));
            painter.circle_stroke(p, 5.0, stroke);
        }
        (false, false) => {
            painter.circle_stroke(p, 4.5, stroke);
        }
    }
}

/// A drone on the map: a pink cross (APP-6: a hostile UAV).
pub fn drone_cross(painter: &egui::Painter, p: egui::Pos2) {
    if app6() {
        app6::Symbol::new(Affiliation::Hostile, Dimension::Air, Icon::Uav)
            .draw(painter, p, 0.8, 1.0);
        return;
    }
    let stroke = egui::Stroke::new(1.5, UAV_PINK);
    painter.line_segment([p + egui::vec2(-4.0, -4.0), p + egui::vec2(4.0, 4.0)], stroke);
    painter.line_segment([p + egui::vec2(-4.0, 4.0), p + egui::vec2(4.0, -4.0)], stroke);
}

/// A legend line: a symbol drawn by `draw` into a small box, then its meaning.
pub fn legend(ui: &mut egui::Ui, text: &str, draw: impl FnOnce(&egui::Painter, egui::Pos2)) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(legend_box(), egui::Sense::hover());
        draw(&ui.painter_at(rect.expand(1.0)), rect.center());
        ui.label(egui::RichText::new(text).small().color(crate::menu::BRIGHT));
    });
}

/// A legend line's symbol box: wider for the APP-6 frames.
fn legend_box() -> egui::Vec2 {
    if app6() { egui::vec2(30.0, 16.0) } else { egui::vec2(18.0, 14.0) }
}

/// What a map shows, each a layer the player can hide.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Layer {
    Airfields,
    Cities,
    Insertions,
    FriendlyAircraft,
    FriendlyHelicopters,
    EnemyFighters,
    Patriots,
    PatriotReach,
    Gepards,
    Sams,
    Guns,
    /// Tanks and infantry fighting vehicles, and the friendly columns' routes.
    Armour,
    PulseRadar,
    DopplerRadar,
    MissileRange,
    /// Where a unit opens fire.
    EngagementRange,
    Drones,
    Clouds,
    /// Rain or snow.
    Precipitation,
    Thunderstorms,
}

/// The weather layers, off until asked for.
pub const WEATHER_LAYERS: [Layer; 3] = [Layer::Clouds, Layer::Precipitation, Layer::Thunderstorms];

/// The layers a map hides, and the air defence types (by name) it hides altogether.
#[derive(Clone, Debug, Default)]
pub struct MapLayers {
    hidden: HashSet<Layer>,
    hidden_types: BTreeSet<String>,
    /// The map's LAYERS panel is open.
    pub panel: bool,
}

impl MapLayers {
    pub fn shows(&self, layer: Layer) -> bool {
        !self.hidden.contains(&layer)
    }

    pub fn shows_type(&self, name: &str) -> bool {
        !self.hidden_types.contains(name)
    }

    fn id(map: &str) -> egui::Id {
        egui::Id::new(("map_layers", map))
    }

    /// The layers of map `map`, kept in egui's memory for the session: at first all shown but
    /// `off`.
    pub fn of(ctx: &egui::Context, map: &str, off: &[Layer]) -> Self {
        ctx.data_mut(|d| {
            d.get_temp_mut_or_insert_with(Self::id(map), || Self {
                hidden: off.iter().copied().collect(),
                ..Default::default()
            })
            .clone()
        })
    }

    pub fn keep(self, ctx: &egui::Context, map: &str) {
        ctx.data_mut(|d| d.insert_temp(Self::id(map), self));
    }

    /// A legend line with its layer's checkbox in front (clicking the text toggles it too).
    pub fn legend(
        &mut self,
        ui: &mut egui::Ui,
        layer: Layer,
        text: &str,
        draw: impl FnOnce(&egui::Painter, egui::Pos2),
    ) {
        let mut on = self.shows(layer);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.checkbox(&mut on, "");
            let (rect, _) = ui.allocate_exact_size(legend_box(), egui::Sense::hover());
            draw(&ui.painter_at(rect.expand(1.0)), rect.center());
            let col = if on { crate::menu::BRIGHT } else { DIM };
            let label = egui::Label::new(egui::RichText::new(text).small().color(col))
                .sense(egui::Sense::click());
            if ui.add(label).clicked() {
                on = !on;
            }
        });
        if on {
            self.hidden.remove(&layer);
        } else {
            self.hidden.insert(layer);
        }
    }

    /// A checkbox per air defence type (folded away under `title`).
    pub fn types(&mut self, ui: &mut egui::Ui, title: &str, types: BTreeSet<&str>) {
        if types.is_empty() {
            return;
        }
        egui::CollapsingHeader::new(egui::RichText::new(title).small().color(crate::menu::BRIGHT))
            .id_salt(title)
            .show(ui, |ui| {
                for name in types {
                    let mut on = self.shows_type(name);
                    if ui.checkbox(&mut on, egui::RichText::new(name).small()).changed() {
                        if on {
                            self.hidden_types.remove(name);
                        } else {
                            self.hidden_types.insert(name.to_string());
                        }
                    }
                }
            });
    }
}

/// Precipitation reaches low flying aircraft as snow below this freezing level (m).
const SNOW_BELOW: f32 = 1000.0;
/// Side of the weather pictures (texels over the whole theater, 500 m each).
const WEATHER_SIZE: usize = 1024;
/// Stratus field samples per side, interpolated between (the field is smooth).
const STRATUS_SAMPLES: usize = 512;
/// A changing weather is drawn again at most this often (seconds).
const WEATHER_EVERY: f64 = 10.0;

/// The weather a map shows: the clouds, and whether what falls from them is snow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapWeather {
    pub weather: Weather,
    pub snow: bool,
}

impl MapWeather {
    /// The clouds of `weather` under the freezing level `freezing_m` (metres).
    pub fn new(weather: Weather, freezing_m: f32) -> Self {
        Self { weather, snow: freezing_m < SNOW_BELOW }
    }
}

/// A map's weather pictures, kept in egui's memory (a texture only exists in its context).
#[derive(Clone)]
struct WeatherPictures {
    weather: MapWeather,
    /// egui time they were made.
    made: f64,
    clouds: egui::TextureHandle,
    precipitation: egui::TextureHandle,
}

/// Paints `top` (unmultiplied colour, alpha) over `under`.
fn over(under: [f32; 4], top: [f32; 3], a: f32) -> [f32; 4] {
    let out = a + under[3] * (1.0 - a);
    if out <= 0.0 {
        return [0.0; 4];
    }
    let mix = |k: usize| (top[k] * a + under[k] * under[3] * (1.0 - a)) / out;
    [mix(0), mix(1), mix(2), out]
}

fn ramp(stops: &[[f32; 3]; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0) * 2.0;
    let (i, f) = if t >= 1.0 { (1, t - 1.0) } else { (0, t) };
    std::array::from_fn(|k| stops[i][k] + (stops[i + 1][k] - stops[i][k]) * f)
}

fn image(px: &[[f32; 4]]) -> egui::ColorImage {
    let px = px
        .iter()
        .map(|&[r, g, b, a]| {
            egui::Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, (a * 255.0) as u8)
        })
        .collect();
    egui::ColorImage::new([WEATHER_SIZE, WEATHER_SIZE], px)
}

/// The whole theater's clouds and precipitation (rain under the storm cells, heaviest in
/// their cores, and lighter rain under a storm's stratus), as the lines of sight see them.
fn weather_images(wx: &MapWeather) -> (egui::ColorImage, egui::ColorImage) {
    let w = &wx.weather;
    let n = WEATHER_SIZE;
    let texel = 0x8000 as f32 / n as f32;
    let mut stratus = vec![0.0f32; n * n];
    if w.stratus.present() {
        let s = STRATUS_SAMPLES;
        let step = 0x8000 as f32 / s as f32;
        let field: Vec<f32> = (0..=s)
            .flat_map(|j| (0..=s).map(move |i| (i, j)))
            .map(|(i, j)| w.stratus_field(i as f32 * step, j as f32 * step))
            .collect();
        let t = w.stratus_threshold as f32 / 65535.0;
        for (k, c) in stratus.iter_mut().enumerate() {
            let x = ((k % n) as f32 + 0.5) * texel / step;
            let y = ((k / n) as f32 + 0.5) * texel / step;
            let (i, j) = ((x as usize).min(s - 1), (y as usize).min(s - 1));
            let (fx, fy) = (x - i as f32, y - j as f32);
            let at = |i: usize, j: usize| field[j * (s + 1) + i];
            let top = at(i, j) + (at(i + 1, j) - at(i, j)) * fx;
            let bottom = at(i, j + 1) + (at(i + 1, j + 1) - at(i, j + 1)) * fx;
            let f = top + (bottom - top) * fy;
            // A soft edge round the covered field.
            *c = ((f - t) / 0.03 + 0.5).clamp(0.0, 1.0);
        }
    }
    // Storm towers (soft edged discs) and their rain; the small cumulus are left out.
    let mut storms = vec![0.0f32; n * n];
    let mut rain = vec![0.0f32; n * n];
    for gy in 0..=0x8000 / STORM_CELL {
        for gx in 0..=0x8000 / STORM_CELL {
            let Some(c) = w.cloud(CloudKind::Cumulonimbus, gx, gy) else { continue };
            let (cx, cy, r) = (c.x as f32 / texel, c.y as f32 / texel, c.r as f32 / texel);
            let (i0, i1) = ((cx - r - 1.0).max(0.0) as usize, ((cx + r + 1.0) as usize).min(n - 1));
            let (j0, j1) = ((cy - r - 1.0).max(0.0) as usize, ((cy + r + 1.0) as usize).min(n - 1));
            for j in j0..=j1 {
                for i in i0..=i1 {
                    let d = egui::vec2(i as f32 + 0.5 - cx, j as f32 + 0.5 - cy).length();
                    let a = (r - d + 0.5).clamp(0.0, 1.0);
                    let k = j * n + i;
                    storms[k] = storms[k].max(a);
                    let core = (1.0 - d / r).max(0.0).powf(0.6);
                    rain[k] = rain[k].max(a * (0.25 + 0.75 * core));
                }
            }
        }
    }
    let clouds: Vec<[f32; 4]> = (0..n * n)
        .map(|k| {
            let px = over([0.0; 4], CLOUD_GREYS[0], 0.45 * stratus[k]);
            over(px, CLOUD_GREYS[1], 0.7 * storms[k])
        })
        .collect();
    let stops = if wx.snow { &SNOW_RAMP } else { &RAIN_RAMP };
    let precipitation: Vec<[f32; 4]> = (0..n * n)
        .map(|k| {
            // It rains under a storm's stratus too (weather.rs `raining`).
            let i = if w.has_storms() { rain[k].max(0.3 * stratus[k]) } else { rain[k] };
            if i <= 0.0 {
                return [0.0; 4];
            }
            let [r, g, b] = ramp(stops, i);
            [r, g, b, (0.2 + 0.65 * i) * (i / 0.05).min(1.0)]
        })
        .collect();
    (image(&clouds), image(&precipitation))
}

/// The weather over map `map`: the clouds, rain or snow and the thunderstorm cells, each if
/// its layer is shown. The pictures follow a changing weather every few seconds.
pub fn weather(
    painter: &egui::Painter,
    view: &MapView,
    map: &str,
    wx: &MapWeather,
    layers: &MapLayers,
) {
    if !wx.weather.any() || !WEATHER_LAYERS.iter().any(|&l| layers.shows(l)) {
        return;
    }
    let ctx = painter.ctx();
    let id = egui::Id::new(("map_weather", map));
    let now = ctx.input(|i| i.time);
    let mut pics = ctx.data(|d| d.get_temp::<WeatherPictures>(id));
    // A changing weather every few seconds, another one (another mission or phase) at once.
    let stale = |p: &WeatherPictures| {
        p.weather != *wx
            && (now - p.made >= WEATHER_EVERY
                || p.weather.weather.seed != wx.weather.seed
                || p.weather.snow != wx.snow)
    };
    if pics.as_ref().is_none_or(stale) {
        let (clouds, precipitation) = weather_images(wx);
        let options = egui::TextureOptions::LINEAR;
        let p = WeatherPictures {
            weather: *wx,
            made: now,
            clouds: ctx.load_texture(format!("{map} clouds"), clouds, options),
            precipitation: ctx.load_texture(format!("{map} precipitation"), precipitation, options),
        };
        ctx.data_mut(|d| d.insert_temp(id, p.clone()));
        pics = Some(p);
    }
    let Some(pics) = pics else { return };
    let theater = egui::Rect::from_min_max(
        view.to_screen(0.0, 0.0),
        view.to_screen(0x8000 as f32, 0x8000 as f32),
    );
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    if layers.shows(Layer::Clouds) {
        painter.image(pics.clouds.id(), theater, uv, egui::Color32::WHITE);
    }
    if layers.shows(Layer::Precipitation) {
        painter.image(pics.precipitation.id(), theater, uv, egui::Color32::WHITE);
    }
    if layers.shows(Layer::Thunderstorms) {
        // The cells of the pictures' weather, in view.
        let w = &pics.weather.weather;
        let (lo, hi) = (view.to_map(view.rect.min), view.to_map(view.rect.max));
        let cell = STORM_CELL as f32;
        let range = |a: f32, b: f32| {
            (a / cell).floor().max(0.0) as i32..=(b / cell).floor().min(0x8000 as f32 / cell) as i32
        };
        for gy in range(lo.y - cell, hi.y + cell) {
            for gx in range(lo.x - cell, hi.x + cell) {
                if let Some(c) = w.cloud(CloudKind::Cumulonimbus, gx, gy) {
                    lightning(painter, view.to_screen(c.x as f32, c.y as f32));
                }
            }
        }
    }
}

/// A thunderstorm cell: a small yellow lightning bolt, outlined dark.
pub fn lightning(painter: &egui::Painter, c: egui::Pos2) {
    let pts: Vec<egui::Pos2> = [(2.5, -6.0), (-2.5, 0.5), (2.0, 0.5), (-2.0, 6.0)]
        .map(|(x, y)| c + egui::vec2(x, y))
        .into();
    painter.add(egui::Shape::line(
        pts.clone(),
        egui::Stroke::new(3.5, egui::Color32::from_black_alpha(160)),
    ));
    painter.add(egui::Shape::line(pts, egui::Stroke::new(1.8, BOLT_YELLOW)));
}

/// The weather layers' legend lines.
pub fn weather_legend(ui: &mut egui::Ui, layers: &mut MapLayers, snow: bool) {
    let grey = |k: usize| {
        let [r, g, b] = CLOUD_GREYS[k];
        egui::Color32::from_rgb(r as u8, g as u8, b as u8)
    };
    layers.legend(ui, Layer::Clouds, "Clouds / storm towers", |p, c0| {
        p.circle_filled(c0 - egui::vec2(3.0, 0.0), 5.0, grey(0).gamma_multiply(0.8));
        p.circle_filled(c0 + egui::vec2(4.0, 0.0), 4.0, grey(1).gamma_multiply(0.9));
    });
    let (stops, text) = if snow {
        (&SNOW_RAMP, "Snow, light to heavy")
    } else {
        (&RAIN_RAMP, "Rain, light to heavy")
    };
    layers.legend(ui, Layer::Precipitation, text, |p, c0| {
        for k in 0..6 {
            let [r, g, b] = ramp(stops, k as f32 / 5.0);
            let x = c0.x - 9.0 + k as f32 * 3.0;
            p.rect_filled(
                egui::Rect::from_min_size(egui::pos2(x, c0.y - 4.0), egui::vec2(3.0, 8.0)),
                0.0,
                egui::Color32::from_rgb(r as u8, g as u8, b as u8),
            );
        }
    });
    layers.legend(ui, Layer::Thunderstorms, "Thunderstorm cell", lightning);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_start_in_the_classic_symbols() {
        assert!(!MapPrefs::default().app6);
        let p: MapPrefs = serde_json::from_str("{}").unwrap();
        assert!(!p.app6);
    }
}

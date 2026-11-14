//! The campaign screen's frame: a top bar, the icon rail down the left that opens a pane over
//! the map, the map under all of it, and the action bar at the bottom right with the main button
//! for what is selected; and the large controls they are made of (rail buttons, tiles,
//! segmented switches, square icon buttons) sized for a mouse and for the headset's laser alike.

use bevy_egui::egui;

use crate::menu::{BG, BRIGHT, DIM, EDGE, FIELD, FRAME, GREEN, HOVER, LIT_BOTTOM, LIT_TOP};

/// The top bar's height, the rail's width, a pane's width (a wide one's), the gap between the
/// map's edges and what floats over it.
pub(crate) const TOP_H: f32 = 60.0;
pub(crate) const RAIL_W: f32 = 96.0;
pub(crate) const PANE_W: f32 = 460.0;
pub(crate) const PANE_NARROW_W: f32 = 420.0;
pub(crate) const WIDE_W: f32 = 760.0;
pub(crate) const GAP: f32 = 16.0;
/// The least size of anything clicked: about 2 degrees on the headset's panel.
pub(crate) const TARGET: f32 = 48.0;
/// A square icon button's side (map controls, a pane's close).
pub(crate) const SQUARE: f32 = 56.0;
/// Seconds a pane takes to slide in.
const SLIDE: f64 = 0.15;

/// How much a window this wide shows of the top bar (its chips shorten, then drop).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Room {
    /// Under 1300 points: icons for the weather and the way out, no brand.
    Small,
    /// Under 2100: the chips' labels gone.
    Medium,
    Full,
}

impl Room {
    pub(super) fn of(width: f32) -> Self {
        if width < 1300.0 {
            Room::Small
        } else if width < 2100.0 {
            Room::Medium
        } else {
            Room::Full
        }
    }
}

/// A pane's width on a screen this wide (`wide`: a pane of tables, the debrief).
pub(crate) fn pane_width(screen: f32, wide: bool) -> f32 {
    let want = if wide {
        WIDE_W
    } else if screen < 1400.0 {
        PANE_NARROW_W
    } else {
        PANE_W
    };
    want.min(screen - RAIL_W)
}

/// How far in a pane opened at `since` (egui time) has slid: 0 hidden, 1 in place.
pub(crate) fn slide(ctx: &egui::Context, since: f64) -> f32 {
    let t = ((ctx.input(|i| i.time) - since) / SLIDE).clamp(0.0, 1.0) as f32;
    if t < 1.0 {
        ctx.request_repaint();
    }
    1.0 - (1.0 - t) * (1.0 - t)
}

/// The frame of what floats over the map: a pane, the action bar, a briefing's panel.
pub(crate) fn overlay_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(BG.gamma_multiply(0.97))
        .stroke(egui::Stroke::new(2.0, EDGE))
        .inner_margin(egui::Margin::symmetric(14, 12))
}

/// A quieter frame for small things over the map (the ticker's lines, hints).
pub(crate) fn chip_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(BG.gamma_multiply(0.92))
        .stroke(egui::Stroke::new(1.0, FRAME))
        .inner_margin(egui::Margin::symmetric(10, 6))
}

/// The icon font's id at `size`.
fn icon_font(size: f32) -> egui::FontId {
    egui::FontId::new(size, crate::menu::icon_family())
}

/// Whether the icon font is in (painting with it before would panic).
fn icons(ctx: &egui::Context) -> bool {
    ctx.fonts(|f| f.definitions().families.contains_key(&crate::menu::icon_family()))
}

/// Paints an icon centred at `at`.
pub(crate) fn paint_icon(
    p: &egui::Painter,
    at: egui::Pos2,
    icon: char,
    size: f32,
    colour: egui::Color32,
) {
    if icons(p.ctx()) {
        p.text(at, egui::Align2::CENTER_CENTER, icon, icon_font(size), colour);
    }
}

/// A text shrunk until it fits `width` (a translation longer than the English).
fn fitted(
    ui: &egui::Ui,
    text: &str,
    size: f32,
    width: f32,
    colour: egui::Color32,
) -> std::sync::Arc<egui::Galley> {
    let mut size = size;
    loop {
        let g =
            ui.painter().layout_no_wrap(text.to_string(), egui::FontId::monospace(size), colour);
        if g.size().x <= width || size <= 8.0 {
            return g;
        }
        size -= 0.5;
    }
}

/// What a rail button carries beside its icon.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Badge {
    None,
    /// A count (new missions, unread messages).
    Count(usize),
    /// Something new to look at (a debrief).
    Dot,
}

impl Badge {
    /// No count at zero.
    pub(super) fn filter_zero(self) -> Self {
        if self == Badge::Count(0) { Badge::None } else { self }
    }
}

/// A rail button: the icon over its label, lit while its pane is open.
pub(crate) fn rail_button(
    ui: &mut egui::Ui,
    icon: char,
    label: &str,
    on: bool,
    badge: Badge,
) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 80.0), egui::Sense::click());
    let p = ui.painter();
    let hovered = resp.hovered();
    if on || hovered {
        p.rect_filled(rect, 0.0, if on { HOVER } else { FIELD });
    }
    if on {
        let bar = egui::Rect::from_min_size(rect.min, egui::vec2(4.0, rect.height()));
        p.rect_filled(bar, 0.0, EDGE);
    }
    let ink = if on || hovered { BRIGHT } else { GREEN };
    paint_icon(p, rect.center() - egui::vec2(0.0, 12.0), icon, 28.0, ink);
    let text = fitted(ui, label, 12.0, rect.width() - 8.0, if on { BRIGHT } else { DIM });
    p.galley(rect.center() + egui::vec2(-text.size().x / 2.0, 12.0), text, BRIGHT);
    let at = rect.center() + egui::vec2(20.0, -30.0);
    match badge {
        Badge::None => {}
        Badge::Dot => {
            p.circle_filled(at + egui::vec2(4.0, 4.0), 6.0, crate::menu::AMBER);
        }
        Badge::Count(n) => {
            let text = if n > 99 { "99+".to_string() } else { n.to_string() };
            let g = p.layout_no_wrap(text, egui::FontId::monospace(13.0), egui::Color32::BLACK);
            let w = (g.size().x + 10.0).max(22.0);
            let r = egui::Rect::from_min_size(at - egui::vec2(0.0, 3.0), egui::vec2(w, 22.0));
            p.rect_filled(r, 11.0, crate::menu::AMBER);
            p.galley(r.center() - g.size() / 2.0, g, egui::Color32::BLACK);
        }
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A square button with an icon only (a map control, a pane's close); what it does on hover.
pub(crate) fn square_button(ui: &mut egui::Ui, icon: char, tip: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::Vec2::splat(SQUARE), egui::Sense::click());
    let hovered = resp.hovered();
    let p = ui.painter();
    p.rect_filled(rect, 0.0, if hovered { HOVER } else { BG.gamma_multiply(0.92) });
    let edge = if hovered { EDGE } else { FRAME };
    p.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, edge), egui::StrokeKind::Inside);
    paint_icon(p, rect.center(), icon, 24.0, if hovered { BRIGHT } else { GREEN });
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if tip.is_empty() { resp } else { resp.on_hover_text(tip) }
}

/// A framed button `height` high with an optional icon, lit when `on` (a base to fly from, a
/// briefing step's NEXT, a site to designate); wide as its text unless `width`.
pub(crate) fn button(
    ui: &mut egui::Ui,
    icon: Option<char>,
    text: &str,
    on: bool,
    height: f32,
    width: Option<f32>,
) -> egui::Response {
    tile(ui, icon, text, None, on, height, width)
}

/// A button with a second, dimmer line under its title (a site's weapon and cost, a station's
/// store).
pub(crate) fn tile(
    ui: &mut egui::Ui,
    icon: Option<char>,
    title: &str,
    sub: Option<&str>,
    on: bool,
    height: f32,
    width: Option<f32>,
) -> egui::Response {
    let enabled = ui.is_enabled();
    let size = if sub.is_some() { 15.0 } else { 17.0 };
    let title_g =
        ui.painter().layout_no_wrap(title.to_string(), egui::FontId::monospace(size), BRIGHT);
    let sub_g =
        sub.map(|s| ui.painter().layout_no_wrap(s.to_string(), egui::FontId::monospace(12.0), DIM));
    let lead = if icon.is_some() { 30.0 } else { 0.0 };
    let natural =
        lead + title_g.size().x.max(sub_g.as_ref().map_or(0.0, |g| g.size().x)) + 2.0 * 14.0;
    let w = width.unwrap_or(natural).min(ui.available_width().max(TARGET));
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(w, height.max(TARGET)), egui::Sense::click());
    let hovered = enabled && resp.hovered();
    let p = ui.painter();
    let (fill, edge, stroke) = if on {
        (HOVER, EDGE, 2.0)
    } else if hovered {
        (HOVER, EDGE, 1.0)
    } else {
        (FIELD, FRAME, 1.0)
    };
    p.rect_filled(rect, 0.0, fill);
    p.rect_stroke(rect, 0.0, egui::Stroke::new(stroke, edge), egui::StrokeKind::Inside);
    let ink = |c: egui::Color32| if enabled { c } else { c.gamma_multiply(0.45) };
    let title_ink = ink(if on || hovered { BRIGHT } else { GREEN });
    let sub_ink = ink(if on { GREEN } else { DIM });
    // The texts, shrunk to the button if they are wider (a translation).
    let room = rect.width() - lead - 20.0;
    let title_g = fitted(ui, title, size, room, title_ink);
    let sub_g = sub.map(|s| fitted(ui, s, 12.0, room, sub_ink));
    let block = title_g.size().y + sub_g.as_ref().map_or(0.0, |g| g.size().y + 2.0);
    let centred = width.is_some() || sub.is_some();
    let left = if centred && icon.is_none() && sub.is_none() {
        rect.center().x - title_g.size().x / 2.0
    } else {
        rect.left() + 12.0 + lead
    };
    if let Some(icon) = icon {
        paint_icon(
            p,
            egui::pos2(rect.left() + 12.0 + 12.0, rect.center().y),
            icon,
            20.0,
            title_ink,
        );
    }
    let top = rect.center().y - block / 2.0;
    let (sub_left, title_left) = if sub.is_some() && icon.is_none() && width.is_some() {
        let tl = rect.center().x - title_g.size().x / 2.0;
        let sl = rect.center().x - sub_g.as_ref().map_or(0.0, |g| g.size().x) / 2.0;
        (sl, tl)
    } else {
        (left, left)
    };
    let h = title_g.size().y;
    p.galley(egui::pos2(title_left, top), title_g, title_ink);
    if let Some(g) = sub_g {
        p.galley(egui::pos2(sub_left, top + h + 2.0), g, sub_ink);
    }
    if enabled { resp.on_hover_cursor(egui::CursorIcon::PointingHand) } else { resp }
}

/// A row of options as one framed switch, the chosen one lit (a map view, the radar coverage's
/// altitude, a list's filter). Returns the one clicked.
pub(crate) fn segmented<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    value: T,
    options: &[(T, Option<char>, &str)],
    height: f32,
) -> Option<T> {
    let n = options.len().max(1) as f32;
    let w = ui.available_width();
    let mut picked = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for &(v, icon, label) in options {
            let (rect, resp) =
                ui.allocate_exact_size(egui::vec2(w / n, height.max(40.0)), egui::Sense::click());
            let on = v == value;
            let hovered = resp.hovered();
            let p = ui.painter();
            p.rect_filled(rect, 0.0, if on || hovered { HOVER } else { BG.gamma_multiply(0.92) });
            p.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, FRAME), egui::StrokeKind::Inside);
            if on {
                let bar = egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.bottom() - 3.0),
                    rect.right_bottom(),
                );
                p.rect_filled(bar, 0.0, EDGE);
            }
            let ink = if on {
                BRIGHT
            } else if hovered {
                GREEN
            } else {
                DIM
            };
            let lead = if icon.is_some() { 22.0 } else { 0.0 };
            let g = fitted(ui, label, 14.0, rect.width() - lead - 10.0, ink);
            let left = rect.center().x - (lead + g.size().x) / 2.0;
            if let Some(icon) = icon {
                paint_icon(p, egui::pos2(left + 8.0, rect.center().y), icon, 15.0, ink);
            }
            p.galley(egui::pos2(left + lead, rect.center().y - g.size().y / 2.0), g, ink);
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() && !on {
                picked = Some(v);
            }
        }
    });
    picked
}

/// A switch with its label (SAME ON BOTH SIDES): returns whether it was flipped.
pub(crate) fn switch(ui: &mut egui::Ui, on: &mut bool, label: &str) -> egui::Response {
    let g = ui.painter().layout_no_wrap(label.to_string(), egui::FontId::monospace(16.0), BRIGHT);
    let size = egui::vec2(58.0 + 12.0 + g.size().x, TARGET);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    if resp.clicked() {
        *on = !*on;
    }
    let p = ui.painter();
    let track = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.center().y - 15.0),
        egui::vec2(58.0, 30.0),
    );
    let (fill, edge) = if *on { (LIT_TOP, EDGE) } else { (FIELD, FRAME) };
    p.rect(track, 15.0, fill, egui::Stroke::new(2.0, edge), egui::StrokeKind::Inside);
    let knob = if *on { track.right() - 15.0 } else { track.left() + 15.0 };
    p.circle_filled(egui::pos2(knob, track.center().y), 10.0, if *on { BRIGHT } else { DIM });
    p.galley(egui::pos2(track.right() + 12.0, rect.center().y - g.size().y / 2.0), g, BRIGHT);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A pane's or a panel's heading: its icon and title, and a close button at the right when
/// `close`. Returns whether it was closed.
pub(crate) fn heading(ui: &mut egui::Ui, icon: Option<char>, title: &str, close: bool) -> bool {
    let mut closed = false;
    ui.horizontal(|ui| {
        ui.set_min_height(SQUARE);
        if let Some(icon) = icon {
            let (r, _) = ui.allocate_exact_size(egui::vec2(28.0, SQUARE), egui::Sense::hover());
            paint_icon(ui.painter(), r.center(), icon, 24.0, BRIGHT);
        }
        ui.label(egui::RichText::new(title).size(22.0).color(BRIGHT));
        if close {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                closed = square_button(ui, crate::menu::icon::CANCEL, "").clicked();
            });
        }
    });
    let r = ui.max_rect();
    let y = ui.cursor().top() + 2.0;
    ui.painter().hline(r.x_range(), y, egui::Stroke::new(1.0, FRAME));
    ui.add_space(10.0);
    closed
}

/// The number of a briefing step in its circle: lit for the one shown, ticked for one seen.
pub(crate) fn step_number(p: &egui::Painter, at: egui::Pos2, n: usize, on: bool, done: bool) {
    let r = 15.0;
    if done && !on {
        p.circle_filled(at, r, GREEN);
        paint_icon(p, at, crate::menu::icon::DONE, 18.0, BG);
    } else {
        p.circle_stroke(at, r, egui::Stroke::new(2.0, if on { EDGE } else { DIM }));
        p.text(
            at,
            egui::Align2::CENTER_CENTER,
            n.to_string(),
            egui::FontId::monospace(15.0),
            if on { BRIGHT } else { DIM },
        );
    }
}

/// A lit panel's look for a primary choice drawn by hand (an aircraft card).
pub(crate) fn card_colours(
    on: bool,
    hovered: bool,
    enabled: bool,
) -> (egui::Color32, egui::Color32, f32) {
    if !enabled {
        (FIELD.gamma_multiply(0.6), FRAME.gamma_multiply(0.6), 1.0)
    } else if on {
        (LIT_BOTTOM.gamma_multiply(0.45), EDGE, 3.0)
    } else if hovered {
        (HOVER, EDGE, 1.0)
    } else {
        (FIELD, FRAME, 1.0)
    }
}

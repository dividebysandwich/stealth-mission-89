//! What the menu windows share to fit any window size: a size that depends on the screen and not
//! on what the window holds, a body that scrolls above a footer that always shows, and groups of
//! options that flow into as many columns as there is room for.
//!
//! - `screen_window`: the window's size comes from the screen alone, so it keeps its height as
//!   the settings or the briefing text change, and it never reaches past the screen.
//! - `Split`: the contents scroll in what is left above the footer (BACK, SAVE + EXIT),
//!   which is never pushed off the screen.
//! - `Flow`: cards (a group of options, the briefing) side by side while they fit, then wrapping
//!   under each other, as a CSS flex-wrap.

use bevy_egui::egui;
use std::cell::Cell;

/// Space between a panel's edge and its contents: a window's contents are this much smaller than
/// the window on each side.
const FRAME: f32 = crate::menu::PANEL_MARGIN;
/// What a window leaves free around it on the screen, on each side.
const SCREEN_GAP: f32 = 8.0;
/// The space between cards.
const GAP: f32 = 8.0;
/// A footer's height before it has been measured (a rule, a line of text and a row of buttons).
const FOOTER_GUESS: f32 = 70.0;
/// The least height a body gets, however small the window.
const BODY_LEAST: f32 = 80.0;

/// The window's size (frame included) for a `desired` one on this `screen`: the desired size
/// where it fits, else what the screen allows.
fn window_size(screen: egui::Vec2, desired: egui::Vec2) -> egui::Vec2 {
    let room = screen - egui::Vec2::splat(2.0 * SCREEN_GAP);
    desired.min(room).max(egui::Vec2::splat(2.0 * FRAME))
}

/// The size `screen_window` gives a window that is `desired` big.
pub(crate) fn fitted(ctx: &egui::Context, desired: egui::Vec2) -> egui::Vec2 {
    window_size(ctx.content_rect().size(), desired)
}

/// A menu window in the middle of the screen, `desired` big (frame included, as `fixed_size`
/// takes it) or as big as the screen allows. Its size does not depend on its contents.
pub(crate) fn screen_window(
    ctx: &egui::Context,
    id: &str,
    desired: egui::Vec2,
) -> egui::Window<'static> {
    egui::Window::new(id.to_string())
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .fixed_size(fitted(ctx, desired))
}

/// The height the body gets in `ui`: all of what is left but for the footer.
fn body_height(ui: &egui::Ui, footer: f32) -> f32 {
    (ui.available_height() - footer - ui.spacing().item_spacing.y).max(BODY_LEAST)
}

/// The least rows a long list is worth showing in a box: with fewer, a dropdown.
const LIST_LEAST_ROWS: f32 = 4.0;

/// How a long list (the theaters, the aircraft) is shown at this window size.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    /// All of it.
    Full,
    /// In a box this high, that scrolls: as many rows as the window leaves room for.
    Rows(f32),
    /// Not even a few rows fit: a dropdown.
    Dropdown,
}

/// What a long list measures, from the last frame that showed it as rows: neither depends on the
/// window's size nor on the mode, so the decision never flips back and forth.
#[derive(Clone, Copy)]
struct Measure {
    /// The height of all its rows.
    full: f32,
    /// The height of the rest of its card: what the list never takes.
    rest: f32,
}

/// How a long list is shown (`Split::list`).
#[derive(Clone, Copy)]
pub(crate) struct ListFit {
    id: egui::Id,
    mode: Mode,
}

impl ListFit {
    /// Whether it is a dropdown (`rows` shows the others).
    pub(crate) fn is_dropdown(self) -> bool {
        self.mode == Mode::Dropdown
    }

    /// The rows, all of them or in a box that scrolls where the window does not have the room.
    /// `add` is told whether this is the first frame of a box: the time to scroll to the selected
    /// row (`Response::scroll_to_me`).
    pub(crate) fn rows(self, ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui, bool)) {
        let max = match self.mode {
            Mode::Rows(height) => height,
            _ => f32::INFINITY,
        };
        // Only a box scrolls to the selected row, and only when it appears.
        let boxed = matches!(self.mode, Mode::Rows(_));
        let first_key = self.id.with("seen");
        let first = boxed && !ui.data(|d| d.get_temp::<bool>(first_key)).unwrap_or(false);
        let out = egui::ScrollArea::vertical()
            .id_salt(self.id)
            .auto_shrink([false, true])
            .max_height(max)
            .show(ui, |ui| add(ui, first));
        ui.data_mut(|d| {
            if boxed {
                d.insert_temp(first_key, true);
            }
            // Not `inner_rect`: a box that is not limited takes all the room it is given until
            // it has seen its contents.
            let shown = out.content_size.y.min(max);
            d.insert_temp(self.id.with("shown"), (out.content_size.y, shown));
        });
    }
}

/// A window's contents in two parts: a body that scrolls in the height the footer leaves, and a
/// footer that stays at the bottom. They are two calls, `body` then `footer`, so that each can
/// borrow what it draws from.
pub(crate) struct Split {
    id: &'static str,
    key: egui::Id,
    /// The footer's height in the last frame (it is laid out after the body).
    footer: f32,
    /// A long list (`list`) has not been measured yet.
    measuring: Cell<bool>,
    /// The body also scrolls sideways (`wide`).
    wide: bool,
}

impl Split {
    /// `id` tells this pair of areas from the window's others.
    pub(crate) fn new(ui: &egui::Ui, id: &'static str) -> Self {
        let key = ui.id().with(("footer height", id));
        let footer = ui.data(|d| d.get_temp::<f32>(key)).unwrap_or(FOOTER_GUESS);
        Self { id, key, footer, measuring: Cell::new(false), wide: false }
    }

    /// The body scrolls sideways too, for rows that can be wider than the window: only for a
    /// window of a fixed size (`fixed_size`), which the scroll area can't widen.
    pub(crate) fn wide(mut self) -> Self {
        self.wide = true;
        self
    }

    /// How the long list `name` (the theaters, the aircraft) is shown at this window size: all
    /// of it where it fits, else in a box with as many rows as there is room for, else as a
    /// dropdown. The window's first frame lays it out unseen, to measure it. Its card is then
    /// reported with `lists`.
    pub(crate) fn list(&self, ui: &egui::Ui, name: &'static str) -> ListFit {
        let id = self.key.with(("list", name));
        let measure = ui.data(|d| d.get_temp::<Measure>(id));
        let mode = match measure {
            None => {
                self.measuring.set(true);
                Mode::Full
            }
            Some(m) => {
                let room = body_height(ui, self.footer) - m.rest;
                let spacing = ui.spacing();
                let least = LIST_LEAST_ROWS * (spacing.interact_size.y + spacing.item_spacing.y);
                if room + 1.0 >= m.full {
                    Mode::Full
                } else if room >= least {
                    Mode::Rows(room)
                } else {
                    Mode::Dropdown
                }
            }
        };
        if !matches!(mode, Mode::Rows(_)) {
            // The selected row is scrolled to again the next time the rows are in a box.
            ui.data_mut(|d| d.remove_temp::<bool>(id.with("seen")));
        }
        ListFit { id, mode }
    }

    /// Reports the card that holds the list `fit`, `card` high: what `Flow::card` returns.
    /// A dropdown is not measured: the last measure with rows stays.
    pub(crate) fn lists(&self, ui: &egui::Ui, fit: ListFit, card: f32) {
        let shown = ui.data_mut(|d| d.remove_temp::<(f32, f32)>(fit.id.with("shown")));
        let Some((full, shown)) = shown else { return };
        let now = Measure { full, rest: (card - shown).max(0.0) };
        let was = ui.data(|d| d.get_temp::<Measure>(fit.id));
        if was.is_none_or(|w| (w.full - now.full).abs() > 0.5 || (w.rest - now.rest).abs() > 0.5) {
            ui.data_mut(|d| d.insert_temp(fit.id, now));
            ui.ctx().request_repaint();
        }
    }

    /// The scrolling body, all the height the window has left but for the footer's. What is
    /// wider than the window is clipped (a scroll area would widen the window to fit it), unless
    /// the body is `wide`.
    pub(crate) fn body(&self, ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
        self.body_sized(ui, |ui, _| add(ui));
    }

    /// `body` for contents that fill the room: `add` is given the body's height, and the width
    /// is the `ui`'s.
    pub(crate) fn body_sized(&self, ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui, f32)) {
        let height = body_height(ui, self.footer);
        let rect =
            egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), height));
        // The first frame lays the lists out in full to measure them, without showing it.
        let mut builder = egui::UiBuilder::new().max_rect(rect);
        if self.measuring.get() {
            builder = builder.invisible();
        }
        let mut body = ui.new_child(builder);
        body.set_clip_rect(rect.intersect(ui.clip_rect()));
        let scroll =
            if self.wide { egui::ScrollArea::both() } else { egui::ScrollArea::vertical() };
        scroll
            .id_salt(self.id)
            .auto_shrink([false, false])
            .min_scrolled_height(height)
            .max_height(height)
            .show(&mut body, |ui| add(ui, height));
        ui.allocate_rect(rect, egui::Sense::hover());
    }

    /// The footer under the body.
    pub(crate) fn footer(self, ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
        let now = ui.scope(add).response.rect.height();
        if (now - self.footer).abs() > 0.5 {
            ui.data_mut(|d| d.insert_temp(self.key, now));
            ui.ctx().request_repaint();
        }
    }
}

/// The contents of a window that is as high as they are, scrolling where they would be higher
/// than the screen: `reserved` is the height the window has outside them (its header, a footer).
pub(crate) fn fit_scroll(
    ui: &mut egui::Ui,
    id: &'static str,
    reserved: f32,
    add: impl FnOnce(&mut egui::Ui),
) {
    let room = ui.ctx().content_rect().height() - 2.0 * (SCREEN_GAP + FRAME) - reserved;
    egui::ScrollArea::vertical()
        .id_salt(id)
        .auto_shrink([false, true])
        .max_height(room.max(BODY_LEAST))
        .show(ui, add);
}

/// How wide a card in a `Flow` is.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Span {
    /// This wide, or the whole row if the row is narrower.
    Fixed(f32),
    /// What is left of the row, at least this much: else on a row of its own.
    Fill(f32),
}

/// Cards laid out left to right, wrapping to a new row where the next does not fit. Each row is
/// as high as its tallest card. A card's height is not bounded, so what it holds must not fill the
/// height it is given (a `with_layout` goes in a `ui.horizontal`).
pub(crate) struct Flow {
    origin: egui::Pos2,
    width: f32,
    /// The width used in this row, the gap after its last card excluded.
    used: f32,
    /// The top of this row, and its height so far.
    top: f32,
    height: f32,
    cards: usize,
}

impl Flow {
    /// A flow over the width left in `ui`, from where its cursor is.
    pub(crate) fn new(ui: &egui::Ui) -> Self {
        Self {
            origin: ui.cursor().min,
            width: ui.available_width(),
            used: 0.0,
            top: 0.0,
            height: 0.0,
            cards: 0,
        }
    }

    /// Adds the next card: `add` draws it in a column `span` wide.
    /// Returns the card's height.
    pub(crate) fn card(
        &mut self,
        ui: &mut egui::Ui,
        span: Span,
        add: impl FnOnce(&mut egui::Ui),
    ) -> f32 {
        let (new_row, width) = place(self.width, self.used, span);
        if new_row {
            self.top += self.height + GAP;
            (self.used, self.height) = (0.0, 0.0);
        }
        let x = if self.used > 0.0 { self.used + GAP } else { 0.0 };
        let at = self.origin + egui::vec2(x, self.top);
        let mut card = ui.new_child(
            egui::UiBuilder::new()
                .id_salt(("flow card", self.cards))
                .max_rect(egui::Rect::from_min_size(at, egui::vec2(width, 100_000.0)))
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        card.set_width(width);
        add(&mut card);
        // A card whose contents are wider than its column (a translated label, a drop-down)
        // pushes the next ones to the right instead of being drawn over by them.
        self.used = x + card.min_rect().width().max(width);
        let height = card.min_rect().height();
        self.height = self.height.max(height);
        self.cards += 1;
        height
    }

    /// Takes the space the cards use in `ui`.
    pub(crate) fn finish(self, ui: &mut egui::Ui) {
        let size = egui::vec2(self.width, self.top + self.height);
        ui.allocate_rect(egui::Rect::from_min_size(self.origin, size), egui::Sense::hover());
    }
}

/// Where a card goes in a row of `width` that has `used` filled (0 for an empty one): whether in
/// a new row, and how wide.
fn place(width: f32, used: f32, span: Span) -> (bool, f32) {
    let need = match span {
        Span::Fixed(w) | Span::Fill(w) => w,
    };
    let mut left = if used > 0.0 { width - used - GAP } else { width };
    let new_row = used > 0.0 && left < need;
    if new_row {
        left = width;
    }
    let card = match span {
        Span::Fixed(w) => w.min(left),
        Span::Fill(_) => left,
    };
    (new_row, card.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A window keeps its size where the screen has room and never reaches past it.
    #[test]
    fn window_fits_the_screen() {
        let desired = egui::vec2(1000.0, 700.0);
        assert_eq!(window_size(egui::vec2(1920.0, 1080.0), desired), desired);
        let small = window_size(egui::vec2(800.0, 600.0), desired);
        assert_eq!(small, egui::vec2(784.0, 584.0));
        let tiny = window_size(egui::vec2(10.0, 10.0), desired);
        assert!(tiny.min_elem() > 0.0);
    }

    /// The single mission's three columns share a row of 720 and the summary wraps under them,
    /// then at 1300 it joins them; at 400 each column is a row.
    #[test]
    fn cards_wrap_where_they_do_not_fit() {
        let columns = [Span::Fixed(210.0), Span::Fixed(220.0), Span::Fixed(260.0)];
        let rows = |width: f32| {
            let (mut used, mut rows) = (0.0, 1);
            for span in columns.into_iter().chain([Span::Fill(300.0)]) {
                let (new_row, w) = place(width, used, span);
                if new_row {
                    rows += 1;
                    used = 0.0;
                }
                used += w + if used > 0.0 { GAP } else { 0.0 };
            }
            rows
        };
        assert_eq!(rows(1300.0), 1);
        assert_eq!(rows(720.0), 2);
        assert_eq!(rows(450.0), 3);
        assert_eq!(rows(250.0), 4);
    }

    #[test]
    fn a_card_never_outgrows_its_row() {
        assert_eq!(place(180.0, 0.0, Span::Fixed(210.0)), (false, 180.0));
        assert_eq!(place(1000.0, 218.0, Span::Fill(300.0)), (false, 774.0));
        assert_eq!(place(500.0, 218.0, Span::Fill(300.0)), (true, 500.0));
    }

    /// One frame of a window of height `screen` with a body holding a card: 100 high, plus a list
    /// of rows 400 high; says how the list was shown.
    fn frame(ctx: &egui::Context, screen: f32) -> Mode {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, screen),
            )),
            ..Default::default()
        };
        let mut mode = Mode::Full;
        let mut output = ctx.run_ui(input, |ui| {
            let split = Split::new(ui, "test");
            let fit = split.list(ui, "rows");
            mode = fit.mode;
            split.body(ui, |ui| {
                let mut flow = Flow::new(ui);
                let card = flow.card(ui, Span::Fixed(200.0), |ui| {
                    ui.allocate_space(egui::vec2(10.0, 100.0));
                    if !fit.is_dropdown() {
                        fit.rows(ui, |ui, _| {
                            for _ in 0..20 {
                                ui.allocate_space(egui::vec2(10.0, 20.0));
                            }
                        });
                    }
                });
                split.lists(ui, fit, card);
                flow.finish(ui);
            });
            split.footer(ui, |ui| {
                ui.label("footer");
            });
        });
        // The fonts' textures are not painted here.
        output.textures_delta.clear();
        mode
    }

    /// The list is in full where it fits, in a box of the rows that fit in a smaller window, and a
    /// dropdown where not even a few do; the same window size always gives the same mode.
    #[test]
    fn lists_take_the_room_the_window_has() {
        let ctx = egui::Context::default();
        // Not measured yet: tried in full.
        assert_eq!(frame(&ctx, 1500.0), Mode::Full);
        assert_eq!(frame(&ctx, 1500.0), Mode::Full);
        // Smaller: a box that scrolls, as high as the room.
        let small = frame(&ctx, 400.0);
        assert!(matches!(small, Mode::Rows(h) if h > 80.0 && h < 400.0), "{small:?}");
        assert_eq!(frame(&ctx, 400.0), small);
        assert_eq!(frame(&ctx, 400.0), small);
        // Not even a few rows: a dropdown, and it stays one in that window.
        assert_eq!(frame(&ctx, 150.0), Mode::Dropdown);
        assert_eq!(frame(&ctx, 150.0), Mode::Dropdown);
        // The window grows: the rows are back.
        assert!(matches!(frame(&ctx, 400.0), Mode::Rows(_)));
        assert_eq!(frame(&ctx, 1500.0), Mode::Full);
    }
}

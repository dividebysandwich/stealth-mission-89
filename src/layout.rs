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

/// A window's contents in two parts: a body that scrolls in the height the footer leaves, and a
/// footer that stays at the bottom. They are two calls, `body` then `footer`, so that each can
/// borrow what it draws from.
pub(crate) struct Split {
    id: &'static str,
    key: egui::Id,
    /// The footer's height in the last frame (it is laid out after the body).
    footer: f32,
}

impl Split {
    /// `id` tells this pair of areas from the window's others.
    pub(crate) fn new(ui: &egui::Ui, id: &'static str) -> Self {
        let key = ui.id().with(("footer height", id));
        let footer = ui.data(|d| d.get_temp::<f32>(key)).unwrap_or(FOOTER_GUESS);
        Self { id, key, footer }
    }

    /// The scrolling body, all the height the window has left but for the footer's. What is
    /// wider than the window is clipped: a scroll area would widen the window to fit it.
    pub(crate) fn body(&self, ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
        self.body_sized(ui, |ui, _| add(ui));
    }

    /// `body` for contents that fill the room: `add` is given the body's height, and the width
    /// is the `ui`'s.
    pub(crate) fn body_sized(
        &self,
        ui: &mut egui::Ui,
        add: impl FnOnce(&mut egui::Ui, f32),
    ) {
        let height =
            (ui.available_height() - self.footer - ui.spacing().item_spacing.y).max(BODY_LEAST);
        let rect = egui::Rect::from_min_size(
            ui.cursor().min,
            egui::vec2(ui.available_width(), height),
        );
        let mut body = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        body.set_clip_rect(rect.intersect(ui.clip_rect()));
        egui::ScrollArea::vertical()
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
    pub(crate) fn card(&mut self, ui: &mut egui::Ui, span: Span, add: impl FnOnce(&mut egui::Ui)) {
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
        self.used = x + width;
        self.height = self.height.max(card.min_rect().height());
        self.cards += 1;
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
}

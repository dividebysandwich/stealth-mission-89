//! Key guide: the keyboard bindings (as mapped now in the CONTROLS dialog) grouped by topic, the
//! mouse on the desktop and the VR controllers in the headset, drawn as a scrolling list. Shown
//! on the kneeboard's KEYS page and from the in-flight menu's KEY GUIDE; the CONTROLS dialog
//! lists the bindings in the same groups. The in-flight menu's guide can be narrowed to the rows
//! whose description holds a search ([`filter`]), as the CONTROLS dialog can.

use bevy::prelude::*;

use crate::canvas::Canvas;
use crate::font::{Align, stroke_has, stroke_text_at, stroke_text_keep_case};
use crate::hotas::{ButtonTarget, InputConfig, VrHand};
use crate::input::{Action, Mods};
use stealth_i18n::{t, tf};

/// What a row's keys are: the keys bound to these actions (" / " between them), the gun's, or
/// a fixed text.
enum Keys {
    Actions(&'static [Action]),
    Gun,
}

/// The keyboard sections, by topic (message ids of their titles and rows). Inner spaces of key
/// texts are `_` (kept on one line).
const SECTIONS: &[(&str, &[(Keys, &str)])] = {
    use Action::*;
    use Keys::*;
    &[
        (
            "kg-section-flight",
            &[
                (Actions(&[PitchDown, PitchUp]), "kg-pitch-down"),
                (Actions(&[RollLeft, RollRight]), "kg-roll-left"),
                (Actions(&[YawLeft, YawRight]), "kg-yaw-left"),
                (Actions(&[TrimNoseDown, TrimNoseUp]), "kg-trim-nose-down"),
                (Actions(&[TrimLeft, TrimRight]), "kg-trim-left"),
                (Actions(&[TrimReset]), "kg-trim-reset"),
                (Actions(&[StickSensitivity]), "kg-stick-sensitivity"),
                (Actions(&[ThrottleUp]), "kg-throttle-up"),
                (Actions(&[ThrottleDown]), "kg-throttle-down"),
                (Actions(&[ThrottleMax]), "kg-throttle-max"),
                (Actions(&[ThrottleIdle]), "kg-throttle-idle"),
                (Actions(&[Autopilot]), "kg-autopilot"),
            ],
        ),
        (
            "kg-section-aircraft",
            &[
                (Actions(&[Gear]), "kg-gear"),
                (Actions(&[Flaps]), "kg-flaps"),
                (Actions(&[Brakes]), "kg-brakes"),
                (Actions(&[BayDoors]), "kg-bay-doors"),
                (Actions(&[MasterCaution]), "kg-master-caution"),
                (Actions(&[Eject]), "kg-eject"),
            ],
        ),
        (
            "kg-section-weapons",
            &[
                (Actions(&[SelectWeapon]), "kg-select-weapon"),
                (Actions(&[FireWeapon]), "kg-fire-weapon"),
                (Gun, "kg-gun"),
                (Actions(&[BombMode]), "kg-bomb-mode"),
            ],
        ),
        (
            "kg-section-targets",
            &[
                (Actions(&[NearestTarget]), "kg-nearest-target"),
                (Actions(&[SelectTarget]), "kg-select-target"),
                (Actions(&[DesignateTarget]), "kg-designate-target"),
                (Actions(&[ClearTarget]), "kg-clear-target"),
                (Actions(&[CamAhead]), "kg-cam-ahead"),
                (Actions(&[CamRear]), "kg-cam-rear"),
                (Actions(&[CamLeft]), "kg-cam-left"),
                (Actions(&[CamRight]), "kg-cam-right"),
                (Actions(&[CameraZoomIn, CameraZoomOut]), "kg-camera-zoom-in"),
                (Actions(&[LockCursor]), "kg-lock-cursor"),
                (Actions(&[SwapSoi]), "kg-swap-soi"),
            ],
        ),
        (
            "kg-section-defenses",
            &[
                (Actions(&[Flare]), "kg-flare"),
                (Actions(&[Chaff]), "kg-chaff"),
                (Actions(&[Decoy]), "kg-decoy"),
                (Actions(&[IrJammer]), "kg-ir-jammer"),
                (Actions(&[Ecm]), "kg-ecm"),
            ],
        ),
        (
            "kg-section-displays",
            &[
                (Actions(&[HudMode]), "kg-hud-mode"),
                (Actions(&[HudDeclutter]), "kg-hud-declutter"),
                (Actions(&[Maps]), "kg-maps"),
                (Actions(&[MapZoomIn, MapZoomOut]), "kg-map-zoom-in"),
                (Actions(&[MapOrientation]), "kg-map-orientation"),
                (Actions(&[Ordnance]), "kg-ordnance"),
                (Actions(&[MissionOrders]), "kg-mission-orders"),
                (Actions(&[Flir]), "kg-flir"),
                (Actions(&[Nvg]), "kg-nvg"),
                (Actions(&[Ils]), "kg-ils"),
            ],
        ),
        (
            "kg-section-navigation",
            &[
                (Actions(&[SelectWaypoint]), "kg-select-waypoint"),
                (Actions(&[ChangeWaypoint]), "kg-change-waypoint"),
                (Actions(&[ResetWaypoint]), "kg-reset-waypoint"),
                (Actions(&[LastWaypoint, NextWaypoint]), "kg-last-waypoint"),
            ],
        ),
        (
            "kg-section-views",
            &[
                (Actions(&[ViewCockpit]), "kg-view-cockpit"),
                (Actions(&[LookAhead]), "kg-look-ahead"),
                (Actions(&[LookLeft]), "kg-look-left"),
                (Actions(&[LookRight]), "kg-look-right"),
                (Actions(&[LookRear]), "kg-look-rear"),
                (Actions(&[ViewSlot]), "kg-view-slot"),
                (Actions(&[ViewChase]), "kg-view-chase"),
                (Actions(&[ViewSide]), "kg-view-side"),
                (Actions(&[ViewMissile]), "kg-view-missile"),
                (Actions(&[ViewTactical]), "kg-view-tactical"),
                (Actions(&[ViewInverseTactical]), "kg-view-inverse-tactical"),
                (Actions(&[ViewFree]), "kg-view-free"),
                (Actions(&[Zoom, Unzoom]), "kg-zoom"),
                (Actions(&[WideAngle]), "kg-wide-angle"),
            ],
        ),
        (
            "kg-section-game",
            &[
                (Actions(&[Menu]), "kg-menu"),
                (Actions(&[Kneeboard]), "kg-kneeboard"),
                (Actions(&[Chat]), "kg-chat"),
                (Actions(&[TimeAccel, TimeNormal]), "kg-time-accel"),
                (Actions(&[Volume]), "kg-volume"),
                (Actions(&[Detail]), "kg-detail"),
                (Actions(&[DayNight]), "kg-day-night"),
                (Actions(&[Training]), "kg-training"),
                (Actions(&[Resupply]), "kg-resupply"),
                (Actions(&[Quit]), "kg-quit"),
            ],
        ),
    ]
};

/// The bindable targets by topic (titles in the player's language), in the guide's order (the
/// CONTROLS dialog lists them so).
pub fn groups() -> Vec<(&'static str, Vec<ButtonTarget>)> {
    SECTIONS
        .iter()
        .map(|(title, rows)| {
            let targets = rows
                .iter()
                .flat_map(|(keys, _)| match keys {
                    Keys::Actions(actions) => {
                        actions.iter().map(|&a| ButtonTarget::Action(a)).collect()
                    }
                    Keys::Gun => vec![ButtonTarget::Gun],
                })
                .collect();
            (t!(title), targets)
        })
        .collect()
}

/// A line of the guide: a section heading or a row (keys, what they do, whether any are bound).
pub enum Line {
    Heading(String),
    Row(String, String, bool),
}

/// A row of fixed keys: its message has the keys, its `.desc` what they do.
fn row(id: &str) -> Line {
    Line::Row(tf!(id).to_string(), tf!(&format!("{id}.desc")).to_string(), true)
}

/// The guide for the current bindings: the mouse on the desktop, the controllers in VR. Drawn
/// with the stroke font, so in [`stealth_i18n::flight_language`].
pub fn lines(config: &InputConfig, vr: bool) -> Vec<Line> {
    let mut out = Vec::new();
    if vr {
        out.push(Line::Heading(tf!("kg-vr-controllers").into()));
        out.push(row("kg-vr-grip"));
        out.push(row("kg-vr-fingertip"));
        out.push(row("kg-vr-left-stick"));
        out.push(row("kg-vr-stick"));
        for hand in [VrHand::Left, VrHand::Right] {
            for m in config.vr_buttons.iter().filter(|m| m.hand == hand) {
                let side = if hand == VrHand::Left { "L" } else { "R" };
                let keys = format!("{side}_{}", m.input.name(hand).replace(' ', "_"));
                let what = match m.target {
                    ButtonTarget::Gun => tf!("kg-gun-target").to_string(),
                    ButtonTarget::Action(a) => crate::cockpit3d::tooltips::action_text(a).0,
                };
                out.push(Line::Row(keys, what, true));
            }
        }
    } else {
        out.push(Line::Heading(tf!("kg-mouse").into()));
        out.push(row("kg-mouse-right-drag"));
        out.push(row("kg-mouse-hold-right"));
        out.push(row("kg-mouse-left-click"));
        out.push(row("kg-mouse-point"));
        out.push(row("kg-mouse-wheel"));
    }
    for (title, rows) in SECTIONS {
        out.push(Line::Heading(tf!(title).into()));
        for (keys, what) in *rows {
            let bound: Vec<String> = match keys {
                Keys::Actions(actions) => {
                    actions.iter().map(|&a| keys_of(config, ButtonTarget::Action(a))).collect()
                }
                Keys::Gun => vec![keys_of(config, ButtonTarget::Gun)],
            };
            let any = bound.iter().any(|k| !k.is_empty());
            let text = if any {
                bound.iter().map(|k| if k.is_empty() { "-" } else { k }).collect::<Vec<_>>()
            } else {
                vec![tf!("kg-unbound")]
            };
            out.push(Line::Row(text.join(" / "), tf!(what).to_string(), any));
        }
    }
    out
}

/// Longest search (characters) of the key guide and the input mapping, and their caret's blink
/// half-period (seconds), the Windows console's.
pub const SEARCH_MAX: usize = 32;
const CARET_BLINK: f64 = 0.53;

/// Whether a search's caret shows at `time` (seconds): shown, then hidden, as a console's.
pub fn caret_shown(time: f64) -> bool {
    ((time / CARET_BLINK) as u64).is_multiple_of(2)
}

/// A search as matched by [`holds`]: trimmed, in small letters without accents.
pub fn search_query(search: &str) -> String {
    stealth_i18n::fold_accents(search.trim()).to_lowercase()
}

/// Whether `text` holds a [`search_query`] in any case and with or without accents ("ecran"
/// finds "Écran"); every text holds an empty one. Texts are searched as shown, in the language
/// they are shown in.
pub fn holds(text: &str, query: &str) -> bool {
    query.is_empty() || stealth_i18n::fold_accents(text).to_lowercase().contains(query)
}

/// The rows whose description holds `search` ([`holds`]), under their headings; all of them for
/// an empty search.
pub fn filter(lines: Vec<Line>, search: &str) -> Vec<Line> {
    let query = search_query(search);
    if query.is_empty() {
        return lines;
    }
    let mut out = Vec::new();
    let mut heading = None;
    for l in lines {
        match l {
            Line::Heading(_) => heading = Some(l),
            Line::Row(_, ref what, _) if holds(what, &query) => {
                out.extend(heading.take());
                out.push(l);
            }
            Line::Row(..) => {}
        }
    }
    out
}

/// The keys bound to a target, ", " between them ("" if none).
fn keys_of(config: &InputConfig, target: ButtonTarget) -> String {
    config
        .keys
        .iter()
        .filter(|m| m.target == target)
        .map(|m| key_text(m.mods, m.key))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Stands for a key printed "_" in a key text (where `_` is a space), drawn as `_`.
const UNDERSCORE: char = '\u{E000}';

/// A key as the layout prints it if the stroke font has its characters, else by its US name.
fn key_text(mods: Mods, key: KeyCode) -> String {
    let name = crate::keyboard_layout::key_name(key)
        .filter(|n| !n.trim().is_empty() && n.chars().all(stroke_has))
        .map(|n| n.replace('_', &UNDERSCORE.to_string()))
        .unwrap_or_else(|| us_name(&crate::input::qwerty_key_label(Mods::None, key)));
    match mods {
        Mods::None => name,
        Mods::Shift => format!("SHIFT_{name}"),
        Mods::Alt => format!("ALT_{name}"),
    }
}

/// A key's short name from its `KeyCode` name ("ArrowUp" -> "UP", "NumpadAdd" -> "NUM PLUS").
fn us_name(name: &str) -> String {
    fn sign(s: &str) -> &str {
        match s {
            "Add" => "PLUS",
            "Subtract" => "MINUS",
            "Multiply" => "*",
            "Divide" => "/",
            "Decimal" => ".",
            s => s,
        }
    }
    if let Some(rest) = name.strip_prefix("Numpad") {
        return format!("NUM_{}", sign(rest).to_ascii_uppercase());
    }
    match name {
        "ArrowUp" => "UP",
        "ArrowDown" => "DOWN",
        "ArrowLeft" => "LEFT",
        "ArrowRight" => "RIGHT",
        "PageUp" => "PGUP",
        "PageDown" => "PGDN",
        "Escape" => "ESC",
        "Insert" => "INS",
        "Delete" => "DEL",
        "Equal" => "=",
        "Minus" => "-",
        "BracketLeft" => "[",
        "BracketRight" => "]",
        "Slash" => "/",
        "Backslash" => "\\",
        "Period" => ".",
        "Comma" => ",",
        "Semicolon" => ";",
        "Quote" => "'",
        "Backquote" => "`",
        "ShiftLeft" | "ShiftRight" => "SHIFT",
        "ControlLeft" | "ControlRight" => "CTRL",
        "AltLeft" | "AltRight" => "ALT",
        n => return n.to_ascii_uppercase(),
    }
    .to_string()
}

/// Colours and text size of the list.
pub struct Style {
    /// Text size, 1 = the kneeboard's.
    pub size: f32,
    /// Heading text and the bar behind it.
    pub heading: u8,
    pub bar: u8,
    pub keys: u8,
    pub text: u8,
    /// Unbound keys, the scroll bar's track.
    pub dim: u8,
}

/// Glyph height and pen of the rows and the headings, line spacing, the keys' column width.
const TEXT: f32 = 16.0;
const PEN: f32 = 2.2;
const HEAD_TEXT: f32 = 18.0;
const HEAD_PEN: f32 = 2.6;
const LINE: f32 = 25.0;
const ROW_GAP: f32 = 9.0;
const BAR: f32 = 30.0;
const KEYS_W: f32 = 236.0;
const SCROLLBAR: f32 = 6.0;

/// Draws the guide in `area` (x0, y0, x1, y1) scrolled down by `scroll` pixels, which is kept
/// within the list; a scroll bar on the right while it does not all fit.
pub fn draw(c: &mut Canvas, lines: &[Line], area: [f32; 4], scroll: &mut f32, style: &Style) {
    let [x0, y0, x1, y1] = area;
    let z = style.size;
    let (text, pen, head_text, head_pen) =
        (TEXT * z, PEN * z.max(0.8), HEAD_TEXT * z, HEAD_PEN * z);
    let (line, row_gap, bar, keys_w) = (LINE * z, ROW_GAP * z, BAR * z, KEYS_W * z);
    let keys_x = x0 + 10.0 * z;
    let text_x = keys_x + keys_w + 18.0 * z;
    let text_w = x1 - SCROLLBAR - 12.0 - text_x;
    // Lay out: each row's key and description lines, and the list's height.
    let mut laid = Vec::with_capacity(lines.len());
    let mut y = 0.0;
    for (i, l) in lines.iter().enumerate() {
        match l {
            Line::Heading(_) => {
                if i > 0 {
                    y += 12.0 * z;
                }
                laid.push((y, Vec::new(), Vec::new()));
                y += bar + row_gap;
            }
            Line::Row(keys, what, _) => {
                let k = crate::kneeboard::wrap(keys, keys_w, text);
                let w = crate::kneeboard::wrap(&what.to_uppercase(), text_w, text);
                let n = k.len().max(w.len()).max(1);
                laid.push((y, k, w));
                y += n as f32 * line + row_gap;
            }
        }
    }
    let view = y1 - y0;
    let max = (y - view).max(0.0);
    *scroll = scroll.clamp(0.0, max);
    c.set_clip(x0 as i32, y0 as i32, x1 as i32, y1 as i32);
    for (l, (ly, k, w)) in lines.iter().zip(&laid) {
        let top = y0 + ly - *scroll;
        if top > y1 || top + 4.0 * line < y0 {
            continue;
        }
        match l {
            Line::Heading(title) => {
                c.fill_rect(
                    x0 as i32,
                    top as i32,
                    (x1 - x0 - SCROLLBAR - 6.0) as i32,
                    bar as i32,
                    style.bar,
                );
                let mid = top + bar / 2.0;
                stroke_text_at(
                    c,
                    keys_x,
                    mid,
                    title,
                    head_text,
                    head_pen,
                    style.heading,
                    Align::Left,
                );
            }
            Line::Row(_, _, bound) => {
                let kc = if *bound { style.keys } else { style.dim };
                for (j, s) in k.iter().enumerate() {
                    let mid = top + line * (j as f32 + 0.5);
                    stroke_text_keep_case(
                        c,
                        keys_x,
                        mid - text / 2.0,
                        &s.replace('_', " ").replace(UNDERSCORE, "_"),
                        text,
                        pen,
                        kc,
                    );
                }
                for (j, s) in w.iter().enumerate() {
                    let mid = top + line * (j as f32 + 0.5);
                    stroke_text_at(c, text_x, mid, s, text, pen, style.text, Align::Left);
                }
            }
        }
    }
    c.reset_clip();
    if max > 0.0 {
        let bx = (x1 - SCROLLBAR) as i32;
        c.fill_rect(bx, y0 as i32, SCROLLBAR as i32, view as i32, style.dim);
        let h = (view * view / y).max(24.0);
        let t = y0 + (view - h) * *scroll / max;
        c.fill_rect(bx, t as i32, SCROLLBAR as i32, h as i32, style.heading);
    }
}

/// Scroll per mouse wheel notch and per second of a fully pushed stick or a held key (pixels).
pub const WHEEL_STEP: f32 = 66.0;
pub const SCROLL_SPEED: f32 = 900.0;

/// Mouse wheel notches this frame (up positive).
pub fn wheel_notches(scroll: &bevy::input::mouse::AccumulatedMouseScroll) -> f32 {
    match scroll.unit {
        bevy::input::mouse::MouseScrollUnit::Line => scroll.delta.y,
        bevy::input::mouse::MouseScrollUnit::Pixel => scroll.delta.y / 50.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::stroke_width;

    /// Every action is in a section, and a key text fits the font.
    #[test]
    fn every_action_listed() {
        let listed: Vec<Action> = SECTIONS
            .iter()
            .flat_map(|(_, rows)| rows.iter())
            .filter_map(|(k, _)| match k {
                Keys::Actions(a) => Some(*a),
                Keys::Gun => None,
            })
            .flatten()
            .copied()
            .collect();
        for a in crate::input::all_actions() {
            assert_eq!(listed.iter().filter(|&&l| l == a).count(), 1, "{a:?} in the key guide");
        }
        let lines = lines(&InputConfig::default(), false);
        for l in &lines {
            if let Line::Row(keys, _, _) = l {
                assert!(
                    keys.chars().all(|c| c == '_' || c == UNDERSCORE || stroke_has(c)),
                    "{keys}"
                );
                let longest = keys.split_whitespace().map(|w| stroke_width(w, TEXT));
                assert!(longest.fold(0.0, f32::max) <= KEYS_W, "{keys}");
            }
        }
        assert!(lines.iter().any(|l| matches!(l, Line::Row(k, _, true) if k == "UP / DOWN")));
        let rudder = |l: &Line| matches!(l, Line::Row(_, w, true) if w == "Rudder left / right");
        assert!(lines.iter().any(rudder));
        let mut config = InputConfig::default();
        config.keys.retain(|k| {
            !matches!(k.target, ButtonTarget::Action(Action::YawLeft | Action::YawRight))
        });
        let unbound = super::lines(&config, false);
        assert!(unbound.iter().any(|l| matches!(l, Line::Row(k, _, false) if k == "UNBOUND")));
        assert!(!unbound.iter().any(rudder));
    }

    /// A search keeps the rows holding it in any case, with their headings only.
    #[test]
    fn search_filters_rows() {
        let all = || lines(&InputConfig::default(), false);
        assert_eq!(filter(all(), "  ").len(), all().len());
        let found = filter(all(), "TarGet");
        let rows: Vec<&str> = found
            .iter()
            .filter_map(|l| match l {
                Line::Row(_, w, _) => Some(w.as_str()),
                Line::Heading(_) => None,
            })
            .collect();
        assert!(rows.contains(&"Lock the nearest target ahead"));
        assert!(rows.iter().all(|w| w.to_lowercase().contains("target")));
        assert!(!rows.contains(&"Tracking camera ahead"));
        let headings = found.iter().filter(|l| matches!(l, Line::Heading(_))).count();
        assert_eq!(headings, 1);
        assert!(matches!(&found[0], Line::Heading(h) if h == "TARGETS"));
        assert!(filter(all(), "zzz").is_empty());
        assert!(holds("Gun", ""));
        assert!(holds("Weapon bay doors", &search_query(" BAY ")));
        assert!(holds("Écran de gauche", &search_query("ecran")));
        assert!(holds("Ecran de gauche", &search_query("Écran")));
        assert!(caret_shown(0.0) && !caret_shown(0.6) && caret_shown(1.1));
    }

    #[test]
    fn key_names() {
        assert_eq!(us_name("NumpadEnter"), "NUM_ENTER");
        assert_eq!(us_name("NumpadAdd"), "NUM_PLUS");
        assert_eq!(us_name("PageDown"), "PGDN");
        assert_eq!(us_name("F10"), "F10");
    }
}

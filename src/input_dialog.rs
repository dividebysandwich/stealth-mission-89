//! Input mapping dialog (opened from the main menu): binds keyboard keys and joystick / HOTAS
//! axes and buttons to the flight controls and every action, picks what the VR controller
//! buttons do, and saves the mapping as JSON. A search field at its top keeps only the axes and
//! actions whose name holds the typed text, and the VR inputs whose name or action does (as the
//! key guide's search; Esc or its X clears it). An action a VR button can be set to holding it
//! keeps every VR input, with their lists narrowed to the matching actions; in VR it types with
//! the virtual keyboard (`text_entry.rs`). The field takes the keyboard once clicked; until then
//! a key or a joystick button pressed jumps to the row it is bound to and highlights it.

use std::collections::{HashMap, HashSet};
use stealth_i18n::t;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::hotas::{
    AxisControl, AxisMapping, ButtonTarget, Device, Hotas, InputConfig, MouseInput, Slot, Source,
    VrGrab, VrHand, VrInput, action_name, default_vr_buttons,
};
use crate::input::{Action, Mods, default_key_mappings};
use crate::key_guide::{SEARCH_MAX, caret_shown, holds, search_query};
use crate::layout::Split;
use crate::menu::{AMBER, BRIGHT, DIM, GREEN, big_button, icon, icon_text};
use crate::text_entry::{Charset, virtual_keyboard};

#[derive(Resource, Default)]
pub struct InputDialog {
    pub open: bool,
    capture: Option<Capture>,
    status: Option<String>,
    /// What is on disk, to save only real changes.
    saved: Option<InputConfig>,
    /// The search field's text.
    search: String,
    /// The row a pressed key or joystick button jumped to (highlighted), and whether it is still
    /// to be scrolled into view.
    jump: Option<(ButtonTarget, bool)>,
    /// UNDO: the mapping before the last binding or reset, and after it (UNDO is offered while
    /// nothing else changed it).
    undo: Option<(InputConfig, InputConfig)>,
}

impl InputDialog {
    /// `--open-input`: start with the dialog open (screenshots).
    pub fn from_args() -> Self {
        Self { open: std::env::args().any(|a| a == "--open-input"), ..default() }
    }

    /// Waiting for a key or a joystick input to bind: the gamepad doesn't navigate the menus
    /// meanwhile (`menu_nav.rs`), so that its B (Esc there) binds rather than cancels.
    pub fn capturing(&self) -> bool {
        self.open && self.capture.is_some()
    }

    /// Closes the dialog, dropping a capture, the status and the search.
    pub fn close(&mut self) {
        self.open = false;
        self.capture = None;
        self.status = None;
        self.search.clear();
        self.jump = None;
        self.undo = None;
    }

    /// The status of a binding made: what was bound to what, and what lost the input to it.
    fn bound(&mut self, what: Binding, input: &str, lost: Vec<String>) {
        let mut text = format!("{} -> {}", what.name().to_uppercase(), input);
        if !lost.is_empty() {
            text.push_str("  ");
            text.push_str(&t!("input-was", what = lost.join(", ").to_uppercase()));
        }
        self.status = Some(text);
    }

    /// Jumps to the row a key or joystick button pressed this frame is bound to (none while a
    /// key is being captured or the search field or another control has the keyboard).
    fn jump_to_pressed(
        &mut self,
        ctx: &egui::Context,
        config: &InputConfig,
        hotas: &Hotas,
        keys: &ButtonInput<KeyCode>,
    ) {
        if self.capture.is_some() || ctx.memory(|m| m.focused().is_some()) {
            return;
        }
        let key = pressed_key(keys).map(|(mods, key)| {
            let target = config.keys.iter().find(|m| m.mods == mods && m.key == key);
            (target.map(|m| m.target), crate::input::key_label(mods, key))
        });
        let button = || {
            if let Some(m) = hotas.buttons(config).find(|m| hotas.just_pressed(&m.device, m.source))
            {
                return Some((Some(m.target), binding_label(&m.device, &m.source.label())));
            }
            hotas.devices().into_iter().find_map(|d| {
                let code = d.just_pressed().next()?;
                Some((None, binding_label(&d.key, &Source::Button(code).label())))
            })
        };
        let Some((target, label)) = key.or_else(button) else { return };
        match target {
            Some(target) => {
                if !holds(&Binding::Key(target).name(), &search_query(&self.search)) {
                    self.search.clear();
                }
                self.status = Some(format!(
                    "{} -> {}",
                    label,
                    Binding::Key(target).name().to_ascii_uppercase()
                ));
                self.jump = Some((target, true));
            }
            None => {
                self.status = Some(format!("{label} IS NOT BOUND"));
                self.jump = None;
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Binding {
    Axis(AxisControl),
    Button(ButtonTarget),
    Key(ButtonTarget),
}

impl Binding {
    fn name(self) -> String {
        match self {
            Binding::Axis(a) => a.name().to_string(),
            Binding::Button(ButtonTarget::Gun) | Binding::Key(ButtonTarget::Gun) => {
                stealth_i18n::t!("kg-gun-target").to_string()
            }
            Binding::Button(ButtonTarget::Action(a)) | Binding::Key(ButtonTarget::Action(a)) => {
                action_name(a)
            }
        }
    }
}

/// A device input that moved during a capture.
enum Found {
    Axis(u32),
    Button(Source),
}

/// Waiting for a device input to bind; the state when it started is the baseline, so axes
/// resting off centre (throttles) and buttons already held don't bind.
struct Capture {
    what: Binding,
    axes: HashMap<(String, u32), f32>,
    buttons: HashSet<(String, u32)>,
}

impl Capture {
    fn new<'a>(what: Binding, devices: impl IntoIterator<Item = &'a Device>) -> Self {
        let mut c = Self { what, axes: HashMap::new(), buttons: HashSet::new() };
        for d in devices {
            c.axes.extend(d.axes.iter().map(|(code, v)| ((d.key.clone(), *code), *v)));
            c.buttons.extend(d.buttons.iter().map(|code| (d.key.clone(), *code)));
        }
        c
    }

    /// The first input that moved since the capture started.
    fn poll<'a>(
        &mut self,
        devices: impl IntoIterator<Item = &'a Device>,
    ) -> Option<(String, Found)> {
        let mut found = None;
        for d in devices {
            // Buttons held at the start count again once released.
            self.buttons.retain(|(n, c)| *n != d.key || d.buttons.contains(c));
            for &code in &d.buttons {
                if matches!(self.what, Binding::Button(_))
                    && !self.buttons.contains(&(d.key.clone(), code))
                {
                    found = found.or(Some((d.key.clone(), Found::Button(Source::Button(code)))));
                }
            }
            for (&code, &v) in &d.axes {
                // Analog triggers already have a button above. Their resting value is -1:
                // releasing one must not bind the negative half of its axis as a button.
                if matches!(self.what, Binding::Button(_)) && d.button_axes.contains(&code) {
                    continue;
                }
                // An axis seen for the first time is its own baseline.
                let base = *self.axes.entry((d.key.clone(), code)).or_insert(v);
                if (v - base).abs() > 0.5 {
                    let input = match self.what {
                        Binding::Axis(_) => Found::Axis(code),
                        Binding::Button(_) if v > 0.5 => Found::Button(Source::AxisPositive(code)),
                        Binding::Button(_) if v < -0.5 => Found::Button(Source::AxisNegative(code)),
                        Binding::Button(_) => continue,
                        Binding::Key(_) => continue,
                    };
                    found = found.or(Some((d.key.clone(), input)));
                }
            }
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A key or mouse button bound to another action names the action that had it.
    #[test]
    fn a_binding_names_what_lost_its_input() {
        let config = InputConfig::default();
        let hotas = Hotas::empty();
        let roll_left = ButtonTarget::Action(Action::RollLeft);
        let brakes = Binding::Key(ButtonTarget::Action(Action::Brakes));
        let lost = losing(&config, &hotas, brakes, Taking::Key(Mods::None, KeyCode::ArrowLeft));
        assert_eq!(lost, vec![Binding::Key(roll_left).name()]);
        // Its own key again: nothing lost.
        let lost = losing(
            &config,
            &hotas,
            Binding::Key(roll_left),
            Taking::Key(Mods::None, KeyCode::ArrowLeft),
        );
        assert!(lost.is_empty());
        let lost = losing(&config, &hotas, brakes, Taking::Mouse(MouseInput::Right));
        assert_eq!(lost.len(), 1);
    }

    #[test]
    fn trigger_can_be_bound_again_after_release() {
        for code in [0x30002, 0x30005] {
            let mut d = Device::default();
            d.key = "Xbox".into();
            d.button_changed(code, 0.0, true);
            let what = Binding::Button(ButtonTarget::Gun);
            let mut capture = Capture::new(what, [&d]);
            d.button_changed(code, 1.0, true);
            assert!(matches!(
                capture.poll([&d]),
                Some((_, Found::Button(Source::Button(c)))) if c == code
            ));

            // Start another capture while still holding the trigger from the first binding.
            let mut capture = Capture::new(what, [&d]);
            assert!(capture.poll([&d]).is_none());
            d.button_changed(code, 0.0, true);
            assert!(capture.poll([&d]).is_none(), "release is not an axis-negative binding");
            d.button_changed(code, 1.0, true);
            assert!(matches!(
                capture.poll([&d]),
                Some((_, Found::Button(Source::Button(c)))) if c == code
            ));
        }
    }

    #[test]
    fn axis_capture_accepts_triggers_but_not_digital_buttons() {
        let mut d = Device::default();
        d.button_changed(2, 0.0, true);
        let mut capture = Capture::new(Binding::Axis(AxisControl::Throttle), [&d]);
        d.button_changed(10, 1.0, false);
        assert!(capture.poll([&d]).is_none());
        d.button_changed(2, 0.5, true);
        assert!(matches!(capture.poll([&d]), Some((_, Found::Axis(2)))));
    }

    #[test]
    fn raw_hotas_axes_still_bind_in_both_directions() {
        for value in [-1.0, 1.0] {
            let mut d = Device::default();
            d.axes.insert(42, 0.0);
            let mut button = Capture::new(Binding::Button(ButtonTarget::Gun), [&d]);
            let mut axis = Capture::new(Binding::Axis(AxisControl::Roll), [&d]);
            d.axes.insert(42, value);
            let expected =
                if value > 0.0 { Source::AxisPositive(42) } else { Source::AxisNegative(42) };
            assert!(matches!(button.poll([&d]), Some((_, Found::Button(s))) if s == expected));
            assert!(matches!(axis.poll([&d]), Some((_, Found::Axis(42)))));
        }
    }
}

/// The input a binding takes, to name what else had it.
enum Taking<'a> {
    Key(Mods, KeyCode),
    Mouse(MouseInput),
    Axis(&'a str, u32),
    Button(&'a str, Source),
}

/// The other controls and targets that lose the input to `what` (the built-in layouts' too):
/// their names, each once.
fn losing(config: &InputConfig, hotas: &Hotas, what: Binding, input: Taking) -> Vec<String> {
    let target = match what {
        Binding::Axis(_) => None,
        Binding::Button(t) | Binding::Key(t) => Some(t),
    };
    let other = |t: ButtonTarget| (Some(t) != target).then(|| Binding::Key(t).name());
    let mut names: Vec<String> = match input {
        Taking::Key(mods, key) => config
            .keys
            .iter()
            .filter(|m| m.mods == mods && m.key == key)
            .filter_map(|m| other(m.target))
            .collect(),
        Taking::Mouse(button) => config
            .mouse
            .iter()
            .filter(|m| m.button == button)
            .filter_map(|m| other(m.target))
            .collect(),
        Taking::Axis(device, code) => hotas
            .axes(config)
            .filter(|m| m.device == device && m.code == code)
            .filter(|m| what != Binding::Axis(m.control))
            .map(|m| m.control.name().to_string())
            .collect(),
        Taking::Button(device, source) => hotas
            .buttons(config)
            .filter(|m| m.device == device && m.source == source)
            .filter_map(|m| other(m.target))
            .collect(),
    };
    names.dedup();
    names
}

/// "Device: Axis 1", the device name shortened but its ids and number kept.
fn binding_label(device: &str, source: &str) -> String {
    let (name, ids) = device.rfind(" [").map_or((device, ""), |i| device.split_at(i));
    let name: String = name.chars().take(28usize.saturating_sub(ids.len()).max(12)).collect();
    format!("{name}{ids}: {source}")
}

/// A binding in the dialog's rows: the device's name cut short (without its USB ids), the whole
/// `binding_label` on hover, so that a long name doesn't push the row's buttons off the window.
fn binding_cell(ui: &mut egui::Ui, device: &str, source: &str, builtin: bool) -> egui::Response {
    /// Characters of the device's name shown.
    const NAME: usize = 16;
    let name = device.rfind(" [").map_or(device, |i| &device[..i]);
    let short = if name.chars().count() > NAME {
        format!("{}…", name.chars().take(NAME - 1).collect::<String>().trim_end())
    } else {
        name.to_string()
    };
    let text = egui::RichText::new(format!("{short}: {source}"));
    if builtin {
        ui.label(text.color(GREEN)).on_hover_text(format!(
            "{}\n{}",
            binding_label(device, source),
            t!("input-builtin")
        ))
    } else {
        ui.label(text).on_hover_text(binding_label(device, source))
    }
}

/// An axis row's binding, invert, dead zone and value cells (`builtin`: the built-in layout's,
/// drawn as such).
fn axis_cells(ui: &mut egui::Ui, m: &mut AxisMapping, hotas: &Hotas, builtin: bool) {
    binding_cell(ui, &m.device, &t!("input-axis", n = m.code & 0xffff), builtin);
    ui.checkbox(&mut m.invert, t!("input-invert"));
    if m.control.lever() {
        ui.label("");
    } else {
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 70.0;
            ui.add(egui::Slider::new(&mut m.deadzone, 0.0..=0.5).fixed_decimals(2))
                .on_hover_text(t!("input-dead-zone"));
            curve_menu(ui, m, hotas);
        });
    }
    let v = hotas.axis(&m.device, m.code).map(|raw| m.output(raw));
    let fill = match (m.control, v) {
        (_, None) => 0.0,
        (AxisControl::Throttle | AxisControl::Nozzle, Some(v)) => v,
        (_, Some(v)) => (v + 1.0) / 2.0,
    };
    let text = v.map_or(t!("input-move-it").to_string(), |v| format!("{v:+.2}"));
    ui.add(egui::ProgressBar::new(fill).desired_width(90.0).text(text));
}

/// CURVE beside a stick axis's dead zone: its response curve and saturation, with the curve
/// drawn and the axis's place on it.
fn curve_menu(ui: &mut egui::Ui, m: &mut AxisMapping, hotas: &Hotas) {
    let shaped = m.expo > 0.0 || m.saturation < 1.0;
    let label =
        if shaped { format!("{} *", t!("input-curve")) } else { t!("input-curve").to_string() };
    let body = |ui: &mut egui::Ui| {
        ui.set_min_width(240.0);
        ui.spacing_mut().slider_width = 150.0;
        let range = crate::hotas::EXPO_RANGE;
        let mut expo = m.expo * 100.0;
        let slider = egui::Slider::new(&mut expo, range.start() * 100.0..=range.end() * 100.0)
            .step_by(5.0)
            .fixed_decimals(0)
            .suffix("%")
            .text(t!("input-expo"));
        if ui.add(slider).on_hover_text(t!("input-expo.facts")).changed() {
            m.expo = expo / 100.0;
        }
        let range = crate::hotas::SATURATION_RANGE;
        let mut sat = m.saturation * 100.0;
        let slider = egui::Slider::new(&mut sat, range.start() * 100.0..=range.end() * 100.0)
            .step_by(5.0)
            .fixed_decimals(0)
            .suffix("%")
            .text(t!("input-saturation"));
        if ui.add(slider).on_hover_text(t!("input-saturation.facts")).changed() {
            m.saturation = sat / 100.0;
        }
        // The curve over the travel from the centre (both halves alike), the axis on it.
        let (rect, _) = ui.allocate_exact_size(egui::vec2(240.0, 120.0), egui::Sense::hover());
        let p = ui.painter_at(rect);
        p.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, DIM), egui::StrokeKind::Inside);
        let at = |x: f32, y: f32| {
            egui::pos2(rect.left() + x * rect.width(), rect.bottom() - y * rect.height())
        };
        p.line_segment(
            [at(0.0, 0.0), at(1.0, 1.0)],
            egui::Stroke::new(1.0, DIM.gamma_multiply(0.5)),
        );
        let mut straight = m.clone();
        straight.invert = false;
        let points: Vec<egui::Pos2> =
            (0..=48).map(|i| i as f32 / 48.0).map(|x| at(x, straight.output(x))).collect();
        p.add(egui::Shape::line(points, egui::Stroke::new(2.0, GREEN)));
        if let Some(raw) = hotas.axis(&m.device, m.code) {
            let x = if m.invert { -raw } else { raw };
            p.circle_filled(at(x.abs().min(1.0), m.output(raw).abs()), 4.0, AMBER);
        }
        ui.ctx().request_repaint();
        if shaped && ui.button(icon_text(ui, icon::RESET, t!("input-curve-straight"))).clicked() {
            (m.expo, m.saturation) = (0.0, 1.0);
        }
    };
    ui.menu_button(label, body);
}

/// Modifier keys, which only count together with another key.
const MODIFIERS: [KeyCode; 8] = [
    KeyCode::ShiftLeft,
    KeyCode::ShiftRight,
    KeyCode::AltLeft,
    KeyCode::AltRight,
    KeyCode::ControlLeft,
    KeyCode::ControlRight,
    KeyCode::SuperLeft,
    KeyCode::SuperRight,
];

/// The first key pressed this frame (other than a modifier or Esc), with the modifier held.
fn pressed_key(keys: &ButtonInput<KeyCode>) -> Option<(Mods, KeyCode)> {
    let key = keys
        .get_just_pressed()
        .copied()
        .find(|k| !MODIFIERS.contains(k) && *k != KeyCode::Escape)?;
    let mods = if keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
        Mods::Alt
    } else if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        Mods::Shift
    } else {
        Mods::None
    };
    Some((mods, key))
}

/// The mouse button pressed this frame, as egui sees it.
fn pressed_mouse(ui: &egui::Ui) -> Option<MouseInput> {
    use egui::PointerButton as P;
    ui.input(|i| {
        [
            (P::Primary, MouseInput::Left),
            (P::Secondary, MouseInput::Right),
            (P::Middle, MouseInput::Middle),
            (P::Extra1, MouseInput::Back),
            (P::Extra2, MouseInput::Forward),
        ]
        .into_iter()
        .find_map(|(p, b)| i.pointer.button_pressed(p).then_some(b))
    })
}

/// Draws the dialog when open (called from the menu's egui pass); `vr`: in the headset, where
/// the search field types with the virtual keyboard.
pub fn show(
    ctx: &egui::Context,
    dialog: &mut InputDialog,
    config: &mut InputConfig,
    hotas: &Hotas,
    keys: &ButtonInput<KeyCode>,
    vr: bool,
) {
    if !dialog.open {
        return;
    }
    if dialog.saved.is_none() {
        dialog.saved = Some(config.clone());
    }
    // Esc cancels a capture, else clears the search.
    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    if dialog.capture.is_none() && escape {
        dialog.search.clear();
    }
    if let Some(cap) = &mut dialog.capture {
        if escape {
            dialog.capture = None;
            dialog.status = None;
        } else if let Binding::Key(target) = cap.what {
            if let Some((mods, key)) = pressed_key(keys) {
                let what = cap.what;
                let lost = losing(config, hotas, what, Taking::Key(mods, key));
                let before = config.clone();
                config.bind_key(target, mods, key);
                dialog.undo = Some((before, config.clone()));
                dialog.bound(what, &crate::input::key_label(mods, key), lost);
                dialog.capture = None;
            }
        } else if let Some((device, input)) = cap.poll(hotas.devices()) {
            let what = cap.what;
            let before = config.clone();
            let (label, lost) = match (what, input) {
                (Binding::Axis(control), Found::Axis(code)) => {
                    let lost = losing(config, hotas, what, Taking::Axis(&device, code));
                    config.bind_axis(control, device.clone(), code);
                    (t!("input-axis", n = code & 0xffff), lost)
                }
                (Binding::Button(target), Found::Button(source)) => {
                    let lost = losing(config, hotas, what, Taking::Button(&device, source));
                    config.bind_button(target, device.clone(), source);
                    (source.label(), lost)
                }
                _ => unreachable!("capture only reports inputs of its binding's kind"),
            };
            dialog.undo = Some((before, config.clone()));
            dialog.bound(what, &binding_label(&device, &label), lost);
            dialog.capture = None;
        }
        // Keep polling while nothing else redraws the window.
        ctx.request_repaint();
    }
    dialog.jump_to_pressed(ctx, config, hotas, keys);

    let value = |control| {
        hotas
            .axes(config)
            .find(|m| m.control == control)
            .and_then(|m| hotas.axis(&m.device, m.code).map(|raw| m.output(raw)))
    };
    let stick = Stick {
        roll: value(AxisControl::Roll).unwrap_or(0.0),
        pitch: value(AxisControl::Pitch).unwrap_or(0.0),
        yaw: value(AxisControl::Yaw).unwrap_or(0.0),
        throttle: value(AxisControl::Throttle),
    };
    // The preview follows the stick without other repaint triggers.
    ctx.request_repaint();

    let screen = ctx.content_rect();
    let mut open = true;
    let mut start: Option<Binding> = None;
    let window = egui::Window::new("INPUT MAPPING")
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .fixed_size(egui::vec2(
            // The screen's width, but not so wide that the rows' cells drift apart.
            (screen.width() - 80.0).clamp(560.0, 1400.0),
            (screen.height() - 120.0).max(300.0),
        ));
    crate::menu::Panel::new(t!("input-title")).note(t!("input-note")).window(window, ctx, |ui| {
        // Rows wider than the window (long key lists) scroll sideways rather than clip.
        let split = Split::new(ui, "input_mapping").wide();
        search_field(ui, &mut dialog.search, dialog.capture.is_some(), vr);
        ui.separator();

        // What the search keeps: the axes and the actions (by group) by name, the VR inputs.
        let query = search_query(&dialog.search);
        let axes: Vec<AxisControl> =
            AxisControl::ALL.into_iter().filter(|c| holds(c.name(), &query)).collect();
        let groups: Vec<(&str, Vec<ButtonTarget>)> = crate::key_guide::groups()
            .into_iter()
            .map(|(title, targets)| {
                (
                    title,
                    targets
                        .into_iter()
                        .filter(|&t| holds(&Binding::Key(t).name(), &query))
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_, targets)| !targets.is_empty())
            .collect();
        let vr_targets = vr_targets();
        let vr_matching: Vec<ButtonTarget> =
            vr_targets.iter().copied().filter(|&t| holds(&target_name(Some(t)), &query)).collect();
        let vr_inputs = vr_rows(config, &query, !vr_matching.is_empty());

        split.body(ui, |ui| {
            // The joysticks and the preview scroll with the rest, at its top.
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width((ui.available_width() - PREVIEW.x - 16.0).max(200.0));
                    let devices = hotas.devices();
                    if devices.is_empty() {
                        ui.colored_label(AMBER, t!("input-no-joysticks"));
                    } else {
                        for d in &devices {
                            ui.label(egui::RichText::new(format!("> {}", d.key)).color(GREEN));
                        }
                    }
                    ui.checkbox(&mut config.gamepad_defaults, t!("input-gamepad"))
                        .on_hover_text(t!("input-gamepad.facts"));
                    ui.checkbox(&mut config.device_defaults, t!("input-device-defaults"))
                        .on_hover_text(t!("input-device-defaults.facts"));
                    if crate::ffb::SUPPORTED && hotas.ffb_base().is_some() {
                        let mut percent = config.ffb_strength * 100.0;
                        ui.horizontal(|ui| {
                            ui.label(t!("input-ffb"));
                            let slider = egui::Slider::new(&mut percent, 0.0..=100.0)
                                .step_by(5.0)
                                .fixed_decimals(0)
                                .suffix("%");
                            if ui.add(slider).on_hover_text(t!("input-ffb.facts")).changed() {
                                config.ffb_strength = percent / 100.0;
                            }
                        });
                    }
                    // UNDO while the mapping is as the last binding or reset left it.
                    if dialog.undo.as_ref().is_some_and(|(_, after)| after != &*config) {
                        dialog.undo = None;
                    }
                    ui.horizontal_wrapped(|ui| {
                        match &dialog.status {
                            Some(text) => ui.colored_label(AMBER, text),
                            None => ui.label(egui::RichText::new(t!("input-help")).color(DIM)),
                        };
                        if dialog.undo.is_some()
                            && ui
                                .button(icon_text(ui, icon::RESET, t!("input-undo")))
                                .on_hover_text(t!("input-undo.facts"))
                                .clicked()
                            && let Some((before, _)) = dialog.undo.take()
                        {
                            *config = before;
                            dialog.status = None;
                        }
                    });
                    ui.label(egui::RichText::new(t!("input-preview-help")).color(DIM).small());
                });
                preview(ui, stick);
            });
            ui.separator();
            if axes.is_empty() && groups.is_empty() && vr_inputs.is_empty() {
                ui.label(egui::RichText::new(t!("input-no-match")).color(DIM));
                return;
            }
            if !axes.is_empty() {
                ui.label(
                    egui::RichText::new(format!("[ {} ]", t!("input-axes")))
                        .color(AMBER)
                        .size(14.0),
                );
                egui::Grid::new("axes").num_columns(6).spacing([14.0, 6.0]).striped(true).show(
                    ui,
                    |ui| {
                        ui.label("");
                        ui.label(egui::RichText::new(t!("input-joystick")).color(GREEN));
                        ui.label("");
                        ui.label(egui::RichText::new(t!("input-dead-zone")).color(GREEN));
                        ui.end_row();
                        for &control in &axes {
                            let what = Binding::Axis(control);
                            ui.label(control.name());
                            let capturing = dialog.capture.as_ref().is_some_and(|c| c.what == what);
                            let builtin = hotas.default_axis(config, control).cloned();
                            let mapping = config.axes.iter_mut().find(|m| m.control == control);
                            match (mapping, builtin) {
                                (Some(m), _) => axis_cells(ui, m, hotas, false),
                                // The built-in layout's: changing its invert or dead zone makes
                                // it the player's.
                                (None, Some(mut m)) => {
                                    let before = m.clone();
                                    axis_cells(ui, &mut m, hotas, true);
                                    if m != before {
                                        config.axes.push(m);
                                    }
                                }
                                (None, None) => {
                                    ui.label(
                                        egui::RichText::new(if capturing {
                                            t!("input-move-axis")
                                        } else {
                                            "-"
                                        })
                                        .color(DIM),
                                    );
                                    ui.label("");
                                    ui.label("");
                                    ui.label("");
                                }
                            }
                            row_buttons(ui, what, capturing, &mut start, || {
                                if config.axes.iter().any(|m| m.control == control) {
                                    config.axes.retain(|m| m.control != control);
                                } else {
                                    config.cleared_defaults.push(Slot::Axis(control));
                                }
                            });
                            ui.end_row();
                        }
                    },
                );
                ui.add_space(8.0);
            }
            if !groups.is_empty() {
                ui.label(
                    egui::RichText::new(format!("[ {} ]", t!("input-buttons")))
                        .color(AMBER)
                        .size(14.0),
                );
                egui::Grid::new("buttons").num_columns(5).spacing([14.0, 4.0]).striped(true).show(
                    ui,
                    |ui| {
                        ui.label("");
                        ui.label(egui::RichText::new(t!("input-keyboard-mouse")).color(GREEN));
                        ui.label("");
                        ui.label(egui::RichText::new(t!("input-joystick")).color(GREEN));
                        ui.end_row();
                        for (target, title) in groups.into_iter().flat_map(|(title, targets)| {
                            targets
                                .into_iter()
                                .enumerate()
                                .map(move |(i, t)| (t, (i == 0).then_some(title)))
                        }) {
                            if let Some(title) = title {
                                ui.label(egui::RichText::new(title).color(AMBER).strong());
                                ui.end_row();
                            }
                            let key = Binding::Key(target);
                            // Behind the row it jumped to: a highlight, set once the row is
                            // laid out.
                            let highlight = ui.painter().add(egui::Shape::Noop);
                            let name = ui.label(key.name());
                            let capturing_key =
                                dialog.capture.as_ref().is_some_and(|c| c.what == key);
                            let labels: Vec<String> = config
                                .keys_for(target)
                                .map(|m| m.label())
                                .chain(config.mouse_for(target).map(|b| b.label()))
                                .collect();
                            let text = match (capturing_key, labels.is_empty()) {
                                (true, _) => t!("input-press-key-or-mouse").to_string(),
                                (false, true) => "-".to_string(),
                                (false, false) => labels.join(", "),
                            };
                            // Many keys are cut short (all on hover) rather than widening
                            // the rows past the window.
                            let full = text.clone();
                            ui.scope(|ui| {
                                ui.set_max_width(KEYS_WIDTH);
                                let label = egui::Label::new(egui::RichText::new(text).color(
                                    if labels.is_empty() || capturing_key { DIM } else { BRIGHT },
                                ))
                                .truncate();
                                let label = ui.add(label);
                                if labels.len() > 1 {
                                    label.on_hover_text(full);
                                }
                            });
                            ui.horizontal(|ui| {
                                let add = egui::Button::new(if capturing_key {
                                    t!("input-waiting")
                                } else {
                                    "+"
                                })
                                .selected(capturing_key);
                                let add = ui.add(add).on_hover_text(t!("input-add-key"));
                                if add.clicked() && !capturing_key {
                                    start = Some(key);
                                }
                                // A mouse button pressed on WAITING binds it (so that the
                                // clicks elsewhere in the dialog don't).
                                if capturing_key
                                    && add.hovered()
                                    && let Some(button) = pressed_mouse(ui)
                                {
                                    let lost = losing(config, hotas, key, Taking::Mouse(button));
                                    let before = config.clone();
                                    config.bind_mouse(target, button);
                                    dialog.undo = Some((before, config.clone()));
                                    dialog.bound(key, &button.label(), lost);
                                    dialog.capture = None;
                                }
                                if ui
                                    .button(icon_text(ui, icon::CLEAR, t!("fm-key-clear")))
                                    .clicked()
                                {
                                    config.keys.retain(|m| m.target != target);
                                    config.mouse.retain(|m| m.target != target);
                                }
                            });
                            let what = Binding::Button(target);
                            let capturing = dialog.capture.as_ref().is_some_and(|c| c.what == what);
                            let builtin = hotas.default_button(config, target);
                            match config.button(target).or(builtin) {
                                Some(m) => binding_cell(
                                    ui,
                                    &m.device,
                                    &m.source.label(),
                                    builtin.is_some(),
                                ),
                                None => ui.label(
                                    egui::RichText::new(if capturing {
                                        t!("input-press-button")
                                    } else {
                                        "-"
                                    })
                                    .color(DIM),
                                ),
                            };
                            let buttons = row_buttons(ui, what, capturing, &mut start, || {
                                if config.button(target).is_some() {
                                    config.buttons.retain(|m| m.target != target);
                                } else {
                                    config.cleared_defaults.push(Slot::Button(target));
                                }
                            });
                            if let Some((jumped, scroll)) = &mut dialog.jump
                                && *jumped == target
                            {
                                let rect = name.rect.union(buttons.rect).expand(3.0);
                                ui.painter().set(
                                    highlight,
                                    egui::Shape::rect_filled(rect, 2.0, AMBER.gamma_multiply(0.25)),
                                );
                                if std::mem::take(scroll) {
                                    ui.scroll_to_rect(rect, Some(egui::Align::Center));
                                }
                            }
                            ui.end_row();
                        }
                    },
                );
                ui.add_space(8.0);
            }
            if !vr_inputs.is_empty() {
                vr_buttons(ui, config, &vr_inputs, &query, &vr_targets, &vr_matching);
            }
        });
        split.footer(ui, |ui| {
            ui.add_space(8.0);
            crate::menu::footer_rule(ui);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .add(big_button(t!("input-default-keys")).icon(icon::RESET))
                    .on_hover_text(t!("input-default-keys.facts"))
                    .clicked()
                {
                    let before = config.clone();
                    config.keys = default_key_mappings();
                    config.mouse = crate::hotas::default_mouse_mappings();
                    dialog.undo = Some((before, config.clone()));
                    dialog.status = Some(t!("input-default-keys-done").to_string());
                }
                // The joysticks' own bindings go, the built-in layouts of known devices are back.
                if ui
                    .add(big_button(t!("input-default-joystick")).icon(icon::RESET))
                    .on_hover_text(t!("input-default-joystick.facts"))
                    .clicked()
                {
                    let before = config.clone();
                    config.axes.clear();
                    config.buttons.clear();
                    config.device_defaults = true;
                    config.gamepad_defaults = true;
                    config.cleared_defaults.clear();
                    dialog.undo = Some((before, config.clone()));
                    dialog.status = Some(t!("input-default-joystick-done").to_string());
                }
                if ui
                    .add(big_button(t!("input-clear-all")).icon(icon::CLEAR))
                    .on_hover_text(t!("input-clear-all.facts"))
                    .clicked()
                {
                    let before = config.clone();
                    config.axes.clear();
                    config.buttons.clear();
                    // The built-in layouts too (the checkbox brings them back).
                    config.device_defaults = false;
                    config.cleared_defaults.clear();
                    dialog.undo = Some((before, config.clone()));
                    dialog.status = Some(t!("input-clear-all-done").to_string());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(big_button(t!("input-done")).icon(icon::DONE))
                        .on_hover_text(t!("input-done.facts"))
                        .clicked()
                    {
                        open = false;
                    }
                });
            });
        });
    });

    if let Some(what) = start {
        dialog.status = None;
        dialog.jump = None;
        dialog.capture = Some(Capture::new(what, hotas.devices()));
    }
    if dialog.saved.as_ref() != Some(config) {
        match config.save() {
            Ok(_) => dialog.saved = Some(config.clone()),
            Err(e) => {
                dialog.status = Some(t!("input-save-failed", error = e.to_string()));
                dialog.saved = Some(config.clone());
            }
        }
    }
    if !open {
        dialog.close();
    }
}

/// The search field across the dialog, as the key guide's: inside its frame a magnifier, the
/// text with a slowly blinking `_` caret as a console's while it has the keyboard (a hint while
/// empty without it) and an X clearing it; in VR clicking it opens the virtual keyboard under
/// it. It has the keyboard once clicked (until Esc), never while a key is being captured.
fn search_field(ui: &mut egui::Ui, search: &mut String, capturing: bool, vr: bool) {
    /// The magnifier's and the X's squares.
    const ICON: f32 = 22.0;
    const NAME: &str = "input_mapping_search";
    let id = egui::Id::new(NAME);
    let font = egui::FontId::monospace(16.0);
    let empty = search.is_empty();
    let col = if empty { DIM } else { BRIGHT };
    let frame = egui::Frame::new()
        .fill(ui.visuals().text_edit_bg_color())
        .stroke(egui::Stroke::new(1.0, col))
        .inner_margin(egui::Margin::symmetric(6, 3));
    let clicked = frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(ICON, ICON), egui::Sense::hover());
            let lens = rect.center() - egui::vec2(2.0, 2.0);
            ui.painter().circle_stroke(lens, 6.0, egui::Stroke::new(2.0, col));
            ui.painter().line_segment(
                [lens + egui::vec2(4.5, 4.5), lens + egui::vec2(9.0, 9.0)],
                egui::Stroke::new(2.4, col),
            );
            // Up to the X's square at the right end, kept while empty so the field stays put.
            let width = ui.available_width() - ICON - ui.spacing().item_spacing.x;
            // The hint makes way for the caret once the field has the keyboard.
            let hint = if ui.memory(|m| m.has_focus(id)) { "" } else { t!("fm-type-to-search") };
            let out = ui
                .scope(|ui| {
                    // The field's own caret is hidden: the `_` is drawn instead.
                    ui.visuals_mut().text_cursor.stroke = egui::Stroke::NONE;
                    egui::TextEdit::singleline(search)
                        .id(id)
                        .frame(egui::Frame::NONE)
                        .char_limit(SEARCH_MAX)
                        .hint_text(egui::RichText::new(hint).color(DIM))
                        .text_color(BRIGHT)
                        .font(font.clone())
                        .desired_width(width)
                        .show(ui)
                })
                .inner;
            if out.response.has_focus()
                && caret_shown(ui.input(|i| i.time))
                && let Some(cursor) = out.cursor_range
            {
                let at = out.galley_pos + out.galley.pos_from_cursor(cursor.primary).min.to_vec2();
                // Within the field: its text's clip rectangle ends with the text, before the
                // caret after it.
                ui.painter().with_clip_rect(out.response.rect).text(
                    at,
                    egui::Align2::LEFT_TOP,
                    "_",
                    font,
                    BRIGHT,
                );
            }
            let sense = if empty { egui::Sense::hover() } else { egui::Sense::click() };
            let (rect, x) = ui.allocate_exact_size(egui::vec2(ICON, ICON), sense);
            if !empty {
                if x.hovered() {
                    ui.painter().rect_filled(rect, 2.0, ui.visuals().widgets.hovered.weak_bg_fill);
                }
                let (c, r) = (rect.center(), 6.0);
                let stroke = egui::Stroke::new(2.0, BRIGHT);
                ui.painter().line_segment([c + egui::vec2(-r, -r), c + egui::vec2(r, r)], stroke);
                ui.painter().line_segment([c + egui::vec2(-r, r), c + egui::vec2(r, -r)], stroke);
                if x.on_hover_text(t!("input-clear-search")).clicked() {
                    search.clear();
                    ui.memory_mut(|m| m.request_focus(id));
                }
            }
            out.response.clicked()
        })
        .inner
    });
    if vr {
        virtual_keyboard(ui, NAME, id, search, SEARCH_MAX, Charset::Text, clicked.inner);
    }
    // Esc (which also clears the search) gives the keyboard back to the jump to a row.
    if capturing || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        ui.memory_mut(|m| m.surrender_focus(id));
    }
}

fn target_name(t: Option<ButtonTarget>) -> String {
    match t {
        None => "-".to_string(),
        Some(t) => Binding::Button(t).name(),
    }
}

/// A VR input's name for both hands ("A / X" where they differ).
fn vr_input_name(input: VrInput) -> String {
    let (l, r) = (input.name(VrHand::Left), input.name(VrHand::Right));
    if l == r { l.to_string() } else { format!("{l} / {r}") }
}

/// What a VR controller button can be set to: every action but the stick keys, which are for
/// the keyboard and joystick buttons (VR has the real stick).
fn vr_targets() -> Vec<ButtonTarget> {
    crate::key_guide::groups()
        .into_iter()
        .flat_map(|(_, targets)| targets)
        .filter(|t| {
            // The head looks around in VR, the fingertips press the controls.
            !matches!(t, ButtonTarget::Action(a)
                if a.stick_direction().is_some()
                    || a.look_direction().is_some()
                    || matches!(a, Action::MouseLook | Action::ClickControl))
        })
        .collect()
}

/// The VR inputs whose name or action of either hand holds a [`search_query`]; all of them
/// when `target_found` (an action they could be set to holds it), as any of them can take it.
fn vr_rows(config: &InputConfig, query: &str, target_found: bool) -> Vec<VrInput> {
    VrInput::ALL
        .into_iter()
        .filter(|&input| {
            target_found
                || holds(&vr_input_name(input), query)
                || [VrHand::Left, VrHand::Right]
                    .into_iter()
                    .any(|hand| holds(&target_name(config.vr_button(hand, input)), query))
        })
        .collect()
}

/// VR controller buttons: a target picked per hand for `inputs` (no capture: the controllers
/// are usually not awake at the desktop menu). Each list holds the `targets`, or only the
/// `matching` ones (those holding the [`search_query`]) unless the query holds the input's name.
fn vr_buttons(
    ui: &mut egui::Ui,
    config: &mut InputConfig,
    inputs: &[VrInput],
    query: &str,
    targets: &[ButtonTarget],
    matching: &[ButtonTarget],
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("[ {} ]", t!("kg-vr-controllers"))).color(AMBER).size(14.0),
        );
        if ui.button(icon_text(ui, icon::RESET, t!("input-defaults"))).clicked() {
            config.vr_buttons = default_vr_buttons();
            config.vr_grab = Default::default();
        }
    });
    ui.label(egui::RichText::new(t!("input-vr-trigger-note")).color(DIM).small());
    ui.label(egui::RichText::new(t!("input-vr-grab-note")).color(DIM).small());
    egui::Grid::new("vr_buttons").num_columns(3).spacing([14.0, 4.0]).striped(true).show(
        ui,
        |ui| {
            ui.label("");
            ui.label(egui::RichText::new(t!("input-left-hand")).color(GREEN));
            ui.label(egui::RichText::new(t!("input-right-hand")).color(GREEN));
            ui.end_row();
            // How each hand grabs: the input, and whether a press takes and the next lets go.
            ui.label(t!("input-vr-grab"));
            for hand in [VrHand::Left, VrHand::Right] {
                let grab = &mut config.vr_grab[hand as usize];
                egui::ComboBox::from_id_salt(("vr-grab", hand))
                    .width(190.0)
                    .selected_text(grab.input.name(hand))
                    .show_ui(ui, |ui| {
                        for input in VrGrab::INPUTS {
                            ui.selectable_value(&mut grab.input, input, input.name(hand));
                        }
                    });
            }
            ui.end_row();
            ui.label(t!("input-vr-grab-toggle"));
            for hand in [VrHand::Left, VrHand::Right] {
                ui.checkbox(&mut config.vr_grab[hand as usize].toggle, "");
            }
            ui.end_row();
            for &input in inputs {
                let name = vr_input_name(input);
                let narrow = !matching.is_empty() && !holds(&name, query);
                let listed = if narrow { matching } else { targets };
                ui.label(name);
                for hand in [VrHand::Left, VrHand::Right] {
                    // The grab input does nothing else (its mapping is kept for later).
                    if config.vr_grabs(hand, input) {
                        ui.label(egui::RichText::new(t!("input-vr-grab-bound")).color(DIM));
                        continue;
                    }
                    let current = config.vr_button(hand, input);
                    let mut picked = current;
                    egui::ComboBox::from_id_salt(("vr", hand, input))
                        .width(190.0)
                        .selected_text(target_name(current))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut picked, None, "-");
                            for &t in listed {
                                ui.selectable_value(&mut picked, Some(t), target_name(Some(t)));
                            }
                        });
                    if picked != current {
                        config.bind_vr(hand, input, picked);
                    }
                }
                ui.end_row();
            }
        },
    );
}

fn row_buttons(
    ui: &mut egui::Ui,
    what: Binding,
    capturing: bool,
    start: &mut Option<Binding>,
    clear: impl FnOnce(),
) -> egui::Response {
    ui.horizontal(|ui| {
        let bind =
            egui::Button::new(if capturing { t!("input-waiting") } else { t!("input-bind") })
                .selected(capturing);
        if ui.add(bind).clicked() && !capturing {
            *start = Some(what);
        }
        if ui.button(icon_text(ui, icon::CLEAR, t!("fm-key-clear"))).clicked() {
            clear();
        }
    })
    .response
}

/// Mapped axis outputs, for the preview.
#[derive(Clone, Copy)]
struct Stick {
    roll: f32,
    pitch: f32,
    yaw: f32,
    /// None: no throttle axis (or it hasn't moved yet).
    throttle: Option<f32>,
}

/// The widest the keyboard column of the button rows gets (points).
const KEYS_WIDTH: f32 = 260.0;

const PREVIEW: egui::Vec2 = egui::vec2(300.0, 190.0);

/// Planform outline (x right, z forward), clockwise from the nose, and the faceted ridge above it.
const OUTLINE: [(f32, f32); 8] = [
    (0.0, 3.0),
    (2.6, -1.6),
    (1.0, -1.0),
    (0.55, -2.1),
    (0.0, -1.7),
    (-0.55, -2.1),
    (-1.0, -1.0),
    (-2.6, -1.6),
];
const RIDGE: Vec3 = Vec3::new(0.0, 0.4, -0.4);
/// V tails (root front, root rear, tip).
const TAILS: [[Vec3; 3]; 2] = [
    [Vec3::new(0.35, 0.15, -1.2), Vec3::new(0.55, 0.15, -1.95), Vec3::new(1.15, 0.95, -2.25)],
    [Vec3::new(-0.35, 0.15, -1.2), Vec3::new(-0.55, 0.15, -1.95), Vec3::new(-1.15, 0.95, -2.25)],
];
/// Exhaust slot centres.
const SLOTS: [Vec3; 2] = [Vec3::new(0.32, 0.12, -1.8), Vec3::new(-0.32, 0.12, -1.8)];

/// Exhaust colour: dull red at idle through orange and yellow to blue-white at full thrust.
fn exhaust_color(t: f32) -> egui::Color32 {
    const STOPS: [(f32, [f32; 3]); 4] = [
        (0.0, [110.0, 25.0, 10.0]),
        (0.4, [235.0, 95.0, 20.0]),
        (0.75, [255.0, 205.0, 80.0]),
        (1.0, [200.0, 225.0, 255.0]),
    ];
    let t = t.clamp(0.0, 1.0);
    let i = STOPS.iter().rposition(|s| s.0 <= t).unwrap_or(0).min(STOPS.len() - 2);
    let (a, b) = (STOPS[i], STOPS[i + 1]);
    let f = (t - a.0) / (b.0 - a.0);
    let c: Vec<u8> = (0..3).map(|k| (a.1[k] + (b.1[k] - a.1[k]) * f) as u8).collect();
    egui::Color32::from_rgb(c[0], c[1], c[2])
}

/// A small F-117 seen from behind and above, deflected by the stick, rudder and throttle.
fn preview(ui: &mut egui::Ui, s: Stick) {
    let (rect, _) = ui.allocate_exact_size(PREVIEW, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, DIM), egui::StrokeKind::Inside);
    // Stick right banks right, pull pitches up, right rudder yaws right.
    let attitude = Quat::from_rotation_y(s.yaw * 0.35)
        * Quat::from_rotation_x(-s.pitch * 0.35)
        * Quat::from_rotation_z(-s.roll * 0.6);
    let view = Quat::from_rotation_x(-0.32);
    let centre = rect.center() + egui::vec2(0.0, 6.0);
    let project = |p: Vec3| {
        let v = view * p;
        let depth = v.z + 13.0;
        centre + egui::vec2(v.x, -v.y) * (380.0 / depth)
    };
    let depth = |p: Vec3| (view * p).z;

    // Fixed ground grid as the reference.
    let grid = egui::Stroke::new(1.0, egui::Color32::from_rgb(12, 50, 34));
    for i in -4..=4 {
        let x = i as f32 * 1.5;
        painter.line_segment(
            [project(Vec3::new(x, -2.0, -4.0)), project(Vec3::new(x, -2.0, 8.0))],
            grid,
        );
    }
    for i in -2..=5 {
        let z = i as f32 * 1.5;
        painter.line_segment(
            [project(Vec3::new(-6.0, -2.0, z)), project(Vec3::new(6.0, -2.0, z))],
            grid,
        );
    }

    // Facets: the ridge fan over the outline, the tails; far ones first.
    let outline: Vec<Vec3> = OUTLINE.iter().map(|&(x, z)| Vec3::new(x, 0.0, z)).collect();
    let mut faces: Vec<[Vec3; 3]> =
        (0..outline.len()).map(|i| [RIDGE, outline[i], outline[(i + 1) % outline.len()]]).collect();
    faces.extend(TAILS);
    let mut faces: Vec<[Vec3; 3]> = faces.into_iter().map(|f| f.map(|p| attitude * p)).collect();
    faces.sort_by(|a, b| {
        let d = |f: &[Vec3; 3]| f.iter().map(|p| depth(*p)).sum::<f32>();
        d(b).total_cmp(&d(a))
    });
    let light = Vec3::new(0.3, 1.0, -0.4).normalize();
    for f in faces {
        let n = (f[1] - f[0]).cross(f[2] - f[0]).normalize_or_zero();
        let lit = n.dot(light).abs();
        let g = (20.0 + 70.0 * lit) as u8;
        let fill = egui::Color32::from_rgb(g / 5, g, g * 2 / 3);
        painter.add(egui::Shape::convex_polygon(
            f.iter().map(|p| project(*p)).collect(),
            fill,
            egui::Stroke::new(1.0, GREEN),
        ));
    }

    // Exhaust flames behind the slots: colour and length from the throttle.
    let t = s.throttle.unwrap_or(0.0);
    let color = exhaust_color(t);
    let len = 0.6 + 2.2 * t;
    let glow = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 70);
    for slot in SLOTS {
        let wide = [
            slot + Vec3::new(-0.38, 0.0, 0.05),
            slot + Vec3::new(0.38, 0.0, 0.05),
            slot + Vec3::new(0.0, 0.0, -len * 1.3),
        ];
        let flame = [
            slot + Vec3::new(-0.25, 0.0, 0.0),
            slot + Vec3::new(0.25, 0.0, 0.0),
            slot + Vec3::new(0.0, 0.0, -len),
        ];
        painter.circle_filled(project(attitude * slot), 4.0 + 8.0 * t, glow);
        painter.add(egui::Shape::convex_polygon(
            wide.iter().map(|p| project(attitude * *p)).collect(),
            glow,
            egui::Stroke::NONE,
        ));
        painter.add(egui::Shape::convex_polygon(
            flame.iter().map(|p| project(attitude * *p)).collect(),
            color,
            egui::Stroke::NONE,
        ));
    }

    // Readouts.
    let dir = |v: f32, neg: &str, pos: &str| match v {
        v if v > 0.02 => format!("{} {:.0}%", t!(pos), v * 100.0),
        v if v < -0.02 => format!("{} {:.0}%", t!(neg), -v * 100.0),
        _ => t!("input-centre").to_string(),
    };
    let font = egui::FontId::monospace(11.0);
    let names = [t!("input-roll"), t!("input-pitch"), t!("input-rudder")];
    let pad = names.iter().map(|n| n.chars().count()).max().unwrap_or(0) + 1;
    let lines = [
        format!("{:<pad$}{}", names[0], dir(s.roll, "input-left", "input-right")),
        format!("{:<pad$}{}", names[1], dir(s.pitch, "input-down", "input-up")),
        format!("{:<pad$}{}", names[2], dir(s.yaw, "input-left", "input-right")),
    ];
    for (i, l) in lines.iter().enumerate() {
        painter.text(
            rect.left_top() + egui::vec2(6.0, 5.0 + i as f32 * 13.0),
            egui::Align2::LEFT_TOP,
            l,
            font.clone(),
            GREEN,
        );
    }
    let throttle = t!("input-throttle");
    let thr =
        s.throttle.map_or(format!("{throttle} -"), |t| format!("{throttle} {:.0}%", t * 100.0));
    painter.text(
        rect.right_bottom() + egui::vec2(-6.0, -5.0),
        egui::Align2::RIGHT_BOTTOM,
        thr,
        font,
        if s.throttle.is_some() { color } else { DIM },
    );
}

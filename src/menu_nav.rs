//! The menus without a mouse (the main menu and its screens, the debrief): egui moves the
//! keyboard focus with the arrow keys and Tab and presses the focused button with Enter or Space,
//! sliders and lists take left / right. This adds the gamepad (d-pad as the arrows, A as Enter,
//! B as Esc) and the first press when nothing has the focus yet: an arrow or the d-pad then
//! focuses the first control, as Tab does. While the input dialog waits for an input to bind,
//! the gamepad is left to it. Joysticks (`hotas::Joystick`) don't navigate: gilrs makes them
//! gamepads, their buttons A, B and so on.
//!
//! The in-flight menu has its own (`flight_menu.rs`); in VR the menus are pointed at.

use bevy::prelude::*;
use bevy_egui::input::EguiInputEvent;
use bevy_egui::{EguiContext, EguiInputSet, PrimaryEguiContext, egui};

use crate::settings::{AppState, Settings};

pub struct MenuNavPlugin;

impl Plugin for MenuNavPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            pad_and_first_press
                .after(EguiInputSet::ReadBevyMessages)
                .before(EguiInputSet::WriteEguiEvents)
                .run_if(in_state(AppState::Menu).or_else(in_state(AppState::Debrief))),
        );
    }
}

/// Gamepad presses as the keys egui knows; the up and down arrows (keys or d-pad) as Shift+Tab
/// and Tab, which go through the controls in order (egui's own arrow-key moves look for the
/// nearest control on that side and miss rows and columns that don't line up); with nothing
/// focused any arrow focuses the first control.
fn pad_and_first_press(
    gamepads: Query<&Gamepad, Without<crate::hotas::Joystick>>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<Settings>,
    mut contexts: Query<(Entity, &mut EguiContext), With<PrimaryEguiContext>>,
    mut out: MessageWriter<EguiInputEvent>,
    dialog: Option<Res<crate::input_dialog::InputDialog>>,
) {
    // Binding an input: every joystick and gamepad button is the binding's.
    if settings.vr || dialog.is_some_and(|d| d.capturing()) {
        return;
    }
    let Ok((context, mut ctx)) = contexts.single_mut() else { return };
    let ctx = ctx.get_mut();
    let pad = |b: GamepadButton| gamepads.iter().any(|g| g.just_pressed(b));
    let mut press = |key: egui::Key, modifiers: egui::Modifiers| {
        for pressed in [true, false] {
            out.write(EguiInputEvent {
                context,
                event: egui::Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed,
                    repeat: false,
                    modifiers,
                },
            });
        }
    };
    let none = egui::Modifiers::NONE;
    // A text field keeps the arrow keys (the caret) and Enter.
    let typing = ctx.text_edit_focused();
    let unfocused = ctx.memory(|m| m.focused().is_none());
    let held = |k: KeyCode| keys.pressed(k);
    let modified = held(KeyCode::ShiftLeft)
        || held(KeyCode::ShiftRight)
        || held(KeyCode::AltLeft)
        || held(KeyCode::AltRight)
        || held(KeyCode::ControlLeft)
        || held(KeyCode::ControlRight);
    // The gamepad can't type: it goes on from a text field (a slider's number box becomes one
    // once focused); the keyboard's arrows stay with it.
    let presses = Presses {
        up: pad(GamepadButton::DPadUp),
        down: pad(GamepadButton::DPadDown),
        left: pad(GamepadButton::DPadLeft),
        right: pad(GamepadButton::DPadRight),
        confirm: pad(GamepadButton::South),
        back: pad(GamepadButton::East),
        ..default()
    };
    let arrow_keys = Presses {
        up: keys.just_pressed(KeyCode::ArrowUp) && !modified,
        down: keys.just_pressed(KeyCode::ArrowDown) && !modified,
        any_arrow_key: [KeyCode::ArrowLeft, KeyCode::ArrowRight]
            .into_iter()
            .any(|k| keys.just_pressed(k)),
        ..default()
    };
    for (key, modifiers) in arrow_keys.keys(unfocused, typing) {
        press(key, modifiers.unwrap_or(none));
    }
    for (key, modifiers) in presses.keys(unfocused, false) {
        press(key, modifiers.unwrap_or(none));
    }
}

/// This frame's navigation presses.
#[derive(Default)]
struct Presses {
    up: bool,
    down: bool,
    /// The d-pad's left and right (the keyboard's go to egui as they are, `any_arrow_key`).
    left: bool,
    right: bool,
    any_arrow_key: bool,
    confirm: bool,
    back: bool,
}

impl Presses {
    /// The keys for egui (None: no modifier).
    fn keys(&self, unfocused: bool, typing: bool) -> Vec<(egui::Key, Option<egui::Modifiers>)> {
        use egui::Key;
        let mut keys = Vec::new();
        let arrow = self.up || self.down || self.left || self.right || self.any_arrow_key;
        if unfocused && arrow {
            keys.push((Key::Tab, None));
        } else if !typing {
            if self.down {
                keys.push((Key::Tab, None));
            }
            if self.up {
                keys.push((Key::Tab, Some(egui::Modifiers::SHIFT)));
            }
            if self.left {
                keys.push((Key::ArrowLeft, None));
            }
            if self.right {
                keys.push((Key::ArrowRight, None));
            }
        }
        if self.confirm && !typing {
            keys.push((Key::Enter, None));
        }
        if self.back {
            keys.push((Key::Escape, None));
        }
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Key;

    #[test]
    fn arrows_go_through_the_controls() {
        let down = Presses { down: true, ..default() };
        // Nothing focused: the first control.
        assert_eq!(down.keys(true, false), vec![(Key::Tab, None)]);
        let up = Presses { up: true, ..default() };
        assert_eq!(up.keys(false, false), vec![(Key::Tab, Some(egui::Modifiers::SHIFT))]);
        // A text field keeps them.
        assert!(down.keys(false, true).is_empty());
        let a = Presses { confirm: true, back: false, ..default() };
        assert_eq!(a.keys(false, false), vec![(Key::Enter, None)]);
        let right = Presses { right: true, ..default() };
        assert_eq!(right.keys(false, false), vec![(Key::ArrowRight, None)]);
    }
}

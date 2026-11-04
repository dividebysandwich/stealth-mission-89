//! Gamepads on Android. gilrs has no Android backend, so the patched winit (vendor/winit, its
//! `platform::android::gamepad`) queues the gamepads' axes and buttons, and this turns them into
//! Bevy's raw gamepad events: they fly and choose in the menus as on the desktop
//! (`input::read_gamepad`), though not through the HOTAS mapping, which reads gilrs.
//!
//! Android's standard layout: the left stick X/Y, the right stick Z/RZ, the triggers
//! LTRIGGER/RTRIGGER (or BRAKE/GAS), the D-pad the hat axes or its keys. A pad is connected
//! when it first reports; Android's input events don't say when it leaves.

use bevy::input::gamepad::{GamepadAxis, GamepadButton};
#[cfg(target_os = "android")]
use bevy::input::gamepad::{
    GamepadConnection, GamepadConnectionEvent, RawGamepadAxisChangedEvent,
    RawGamepadButtonChangedEvent, RawGamepadEvent,
};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

#[cfg(target_os = "android")]
pub struct AndroidGamepadPlugin;

#[cfg(target_os = "android")]
impl Plugin for AndroidGamepadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pads>().add_systems(PreUpdate, read.before(bevy::input::InputSystems));
    }
}

/// The axes winit reports, in the order of its `gamepad::AXES`: X, Y, Z, RZ, LTRIGGER,
/// RTRIGGER, BRAKE, GAS, HAT_X, HAT_Y.
type Axes = [f32; 10];

/// The sticks, with Bevy's up as +1.
fn sticks([x, y, z, rz, ..]: Axes) -> [(GamepadAxis, f32); 4] {
    [
        (GamepadAxis::LeftStickX, x),
        (GamepadAxis::LeftStickY, -y),
        (GamepadAxis::RightStickX, z),
        (GamepadAxis::RightStickY, -rz),
    ]
}

/// The buttons on axes: the triggers and the hat (the D-pad).
fn axis_buttons([.., lt, rt, brake, gas, hat_x, hat_y]: Axes) -> [(GamepadButton, f32); 6] {
    let hat = |v: f32| if v > 0.5 { 1.0 } else { 0.0 };
    [
        (GamepadButton::LeftTrigger2, lt.max(brake).clamp(0.0, 1.0)),
        (GamepadButton::RightTrigger2, rt.max(gas).clamp(0.0, 1.0)),
        (GamepadButton::DPadLeft, hat(-hat_x)),
        (GamepadButton::DPadRight, hat(hat_x)),
        (GamepadButton::DPadUp, hat(-hat_y)),
        (GamepadButton::DPadDown, hat(hat_y)),
    ]
}

/// The button of an Android key code (KEYCODE_BUTTON_*, KEYCODE_DPAD_*).
fn button(keycode: u32) -> Option<GamepadButton> {
    Some(match keycode {
        19 => GamepadButton::DPadUp,
        20 => GamepadButton::DPadDown,
        21 => GamepadButton::DPadLeft,
        22 => GamepadButton::DPadRight,
        23 | 96 => GamepadButton::South,
        97 => GamepadButton::East,
        98 => GamepadButton::C,
        99 => GamepadButton::West,
        100 => GamepadButton::North,
        101 => GamepadButton::Z,
        102 => GamepadButton::LeftTrigger,
        103 => GamepadButton::RightTrigger,
        104 => GamepadButton::LeftTrigger2,
        105 => GamepadButton::RightTrigger2,
        106 => GamepadButton::LeftThumb,
        107 => GamepadButton::RightThumb,
        108 => GamepadButton::Start,
        109 => GamepadButton::Select,
        110 => GamepadButton::Mode,
        // KEYCODE_BUTTON_1..16.
        188..=203 => GamepadButton::Other((keycode - 188) as u8),
        _ => return None,
    })
}

/// A gamepad by its Android device id.
#[derive(Default)]
struct Pad {
    entity: Option<Entity>,
    /// The buttons whose keys are down.
    keys: Vec<GamepadButton>,
    /// The buttons' values from the axes (the triggers, the hat).
    analog: HashMap<GamepadButton, f32>,
    /// The values last sent.
    sent_axes: HashMap<GamepadAxis, f32>,
    sent_buttons: HashMap<GamepadButton, f32>,
}

impl Pad {
    /// A button's value: pressed by its key or pulled on its axis, whichever is further.
    fn value(&self, b: GamepadButton) -> f32 {
        let key = if self.keys.contains(&b) { 1.0 } else { 0.0 };
        self.analog.get(&b).copied().unwrap_or(0.0).max(key)
    }
}

#[derive(Resource, Default)]
struct Pads(HashMap<i32, Pad>);

/// What one gamepad event changes: the axes and buttons to send.
#[derive(Default, Debug, PartialEq)]
struct Changes {
    axes: Vec<(GamepadAxis, f32)>,
    buttons: Vec<(GamepadButton, f32)>,
}

impl Pad {
    fn axes(&mut self, values: Axes) -> Changes {
        let mut changes = Changes::default();
        for (a, v) in sticks(values) {
            if self.sent_axes.insert(a, v) != Some(v) {
                changes.axes.push((a, v));
            }
        }
        for (b, v) in axis_buttons(values) {
            self.analog.insert(b, v);
            self.changed(b, &mut changes);
        }
        changes
    }

    fn key(&mut self, keycode: u32, pressed: bool) -> Changes {
        let mut changes = Changes::default();
        if let Some(b) = button(keycode) {
            self.keys.retain(|&k| k != b);
            if pressed {
                self.keys.push(b);
            }
            self.changed(b, &mut changes);
        }
        changes
    }

    fn changed(&mut self, b: GamepadButton, changes: &mut Changes) {
        let v = self.value(b);
        if self.sent_buttons.get(&b).copied().unwrap_or(0.0) != v {
            self.sent_buttons.insert(b, v);
            changes.buttons.push((b, v));
        }
    }
}

#[cfg(target_os = "android")]
fn read(
    mut commands: Commands,
    mut pads: ResMut<Pads>,
    mut events: MessageWriter<RawGamepadEvent>,
    mut connections: MessageWriter<GamepadConnectionEvent>,
    mut button_events: MessageWriter<RawGamepadButtonChangedEvent>,
    mut axis_events: MessageWriter<RawGamepadAxisChangedEvent>,
) {
    use winit::platform::android::gamepad::{self, GamepadEvent};
    for event in gamepad::drain() {
        let (device, changes) = match event {
            GamepadEvent::Axes { device, values } => {
                (device, pads.0.entry(device).or_default().axes(values))
            }
            GamepadEvent::Button { device, keycode, pressed } => {
                (device, pads.0.entry(device).or_default().key(keycode.into(), pressed))
            }
        };
        let pad = pads.0.get_mut(&device).expect("inserted above");
        let entity = *pad.entity.get_or_insert_with(|| {
            let entity = commands.spawn_empty().id();
            let event = GamepadConnectionEvent::new(
                entity,
                GamepadConnection::Connected {
                    name: format!("Android gamepad {device}"),
                    vendor_id: None,
                    product_id: None,
                },
            );
            info!("gamepad {device} connected");
            events.write(event.clone().into());
            connections.write(event);
            entity
        });
        for (a, v) in changes.axes {
            events.write(RawGamepadAxisChangedEvent::new(entity, a, v).into());
            axis_events.write(RawGamepadAxisChangedEvent::new(entity, a, v));
        }
        for (b, v) in changes.buttons {
            events.write(RawGamepadButtonChangedEvent::new(entity, b, v).into());
            button_events.write(RawGamepadButtonChangedEvent::new(entity, b, v));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sticks are sent as they move, up positive; the hat and the triggers are buttons.
    #[test]
    fn axes_become_sticks_and_buttons() {
        let mut pad = Pad::default();
        let c = pad.axes([0.5, -1.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, -1.0, 0.0]);
        assert!(c.axes.contains(&(GamepadAxis::LeftStickX, 0.5)));
        assert!(c.axes.contains(&(GamepadAxis::LeftStickY, 1.0)));
        assert!(c.buttons.contains(&(GamepadButton::RightTrigger2, 0.8)));
        assert!(c.buttons.contains(&(GamepadButton::DPadLeft, 1.0)));
        assert!(!c.buttons.iter().any(|b| b.0 == GamepadButton::LeftTrigger2));
        // Nothing moved: nothing sent.
        let c = pad.axes([0.5, -1.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, -1.0, 0.0]);
        assert_eq!(c, Changes::default());
        // The hat back in the middle releases the D-pad.
        let c = pad.axes([0.5, -1.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(c.buttons, vec![(GamepadButton::DPadLeft, 0.0)]);
    }

    /// A button pressed by its key and by its axis is released when both let go.
    #[test]
    fn keys_and_axes_share_a_button() {
        let mut pad = Pad::default();
        assert_eq!(pad.key(96, true).buttons, vec![(GamepadButton::South, 1.0)]);
        assert_eq!(pad.key(96, false).buttons, vec![(GamepadButton::South, 0.0)]);
        // The trigger's key and its axis.
        pad.axes([0.0, 0.0, 0.0, 0.0, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(pad.key(104, true).buttons, vec![(GamepadButton::LeftTrigger2, 1.0)]);
        assert_eq!(pad.key(104, false).buttons, vec![(GamepadButton::LeftTrigger2, 0.3)]);
        // A key that is no gamepad's button.
        assert_eq!(pad.key(4, true), Changes::default());
    }
}

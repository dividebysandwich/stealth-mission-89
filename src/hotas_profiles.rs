//! Built-in layouts for well-known flight sticks, throttles and pedals: when one is connected,
//! its axes and buttons do what its layout says unless the player's mapping binds that control
//! (or that axis or button) itself, or cleared the layout's binding (`InputConfig`).
//!
//! A layout names the device's inputs as its HID report does (axis usages, 1-based button
//! numbers as Windows' game controller panel shows them, hat switches); `Platform` turns them into
//! the raw codes gilrs reports: Linux evdev codes (the kernel's hid-input numbering) or Windows
//! Gaming Input indices (buttons by number, axes in the order of their usages, hats as switches).
//! Other platforms (macOS, the browser, Android) get no built-in layout.

use crate::hotas::{AxisControl, AxisMapping, ButtonMapping, ButtonTarget, Source};
use crate::input::Action;

/// A HID Generic Desktop axis usage. `Slider2` is a second slider usage, which Linux drops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hid {
    X,
    Y,
    Z,
    Rx,
    Ry,
    Rz,
    Slider,
    Slider2,
    Dial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// A device input usable as a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum In {
    /// A button by its 1-based number.
    B(u16),
    /// A hat switch (0 the first) pushed one way.
    Hat(u8, Dir),
}

/// An axis the layout binds: the control, the axis, and whether it reads the other way round
/// from what `AxisMapping::new` assumes (a stick forward and a throttle lever forward reading
/// negative).
pub type AxisBind = (AxisControl, Hid, bool);

/// A known device's layout.
#[derive(Debug)]
pub struct Profile {
    pub name: &'static str,
    /// USB vendor and product ids (each mode or variant of the device).
    pub ids: &'static [(u16, u16)],
    /// Every axis the device reports, in HID usage order (Windows numbers them so).
    pub axes: &'static [Hid],
    pub bind_axes: &'static [AxisBind],
    pub buttons: &'static [(ButtonTarget, In)],
}

/// How the platform's gilrs backend numbers a device's inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// evdev: event type << 16 | event code.
    Linux,
    /// Windows Gaming Input: kind (0 button, 1 axis, 2 switch) << 16 | index.
    Windows,
}

pub const PLATFORM: Option<Platform> = if cfg!(target_os = "linux") {
    Some(Platform::Linux)
} else if cfg!(windows) {
    Some(Platform::Windows)
} else {
    None
};

const EV_KEY: u32 = 1 << 16;
const EV_ABS: u32 = 3 << 16;
const ABS_HAT0X: u32 = 0x10;
/// hid-input's joystick buttons: 1 to 16 from BTN_JOYSTICK, then BTN_TRIGGER_HAPPY1 to 40.
const BTN_JOYSTICK: u32 = 0x120;
const BTN_TRIGGER_HAPPY: u32 = 0x2c0;
const BTN_TRIGGER_HAPPY40: u32 = 0x2e7;

/// Whether a raw axis code is a hat's (each hat two axes reading -1, 0 or 1), as the platform's
/// gilrs backend numbers them; false where that isn't known.
pub fn is_hat(code: u32) -> bool {
    match PLATFORM {
        Some(Platform::Linux) => {
            code & !0xffff == EV_ABS && (ABS_HAT0X..ABS_HAT0X + 8).contains(&(code & 0xffff))
        }
        Some(Platform::Windows) => code >> 16 == 2,
        None => false,
    }
}

impl Platform {
    /// The raw code of an axis of `p`.
    pub fn axis(self, p: &Profile, a: Hid) -> Option<u32> {
        match self {
            Platform::Linux => {
                let code = match a {
                    Hid::X => 0,
                    Hid::Y => 1,
                    Hid::Z => 2,
                    Hid::Rx => 3,
                    Hid::Ry => 4,
                    Hid::Rz => 5,
                    // hid-input maps a usage to ABS_<usage & 0xf>: ABS_THROTTLE, ABS_RUDDER.
                    Hid::Slider => 6,
                    Hid::Dial => 7,
                    Hid::Slider2 => return None,
                };
                Some(EV_ABS | code)
            }
            Platform::Windows => {
                let i = p.axes.iter().position(|&x| x == a)?;
                Some((1 << 16) | i as u32)
            }
        }
    }

    /// The raw source of a button or hat direction.
    pub fn source(self, input: In) -> Option<Source> {
        match (self, input) {
            (_, In::B(0)) => None,
            (Platform::Linux, In::B(n)) => {
                let i = u32::from(n) - 1;
                let code = if i < 16 { BTN_JOYSTICK + i } else { BTN_TRIGGER_HAPPY + i - 16 };
                (code <= BTN_TRIGGER_HAPPY40).then_some(Source::Button(EV_KEY | code))
            }
            (Platform::Windows, In::B(n)) => Some(Source::Button(u32::from(n) - 1)),
            (_, In::Hat(h, dir)) => {
                let (x, y) = match self {
                    Platform::Linux => {
                        let x = EV_ABS | (ABS_HAT0X + 2 * u32::from(h));
                        (x, x + 1)
                    }
                    Platform::Windows => {
                        let x = (2 << 16) | (2 * u32::from(h));
                        (x, x + 1)
                    }
                };
                // Both read a hat pushed up or left negative.
                Some(match dir {
                    Dir::Up => Source::AxisNegative(y),
                    Dir::Down => Source::AxisPositive(y),
                    Dir::Left => Source::AxisNegative(x),
                    Dir::Right => Source::AxisPositive(x),
                })
            }
        }
    }
}

impl Profile {
    /// The layout's bindings for the device mappings call `device`.
    pub fn mappings(
        &self,
        device: &str,
        platform: Platform,
    ) -> (Vec<AxisMapping>, Vec<ButtonMapping>) {
        let axes = self
            .bind_axes
            .iter()
            .filter_map(|&(control, hid, flip)| {
                let mut m =
                    AxisMapping::new(control, device.to_string(), platform.axis(self, hid)?);
                m.invert ^= flip;
                Some(m)
            })
            .collect();
        let buttons = self
            .buttons
            .iter()
            .filter_map(|&(target, input)| {
                Some(ButtonMapping {
                    target,
                    device: device.to_string(),
                    source: platform.source(input)?,
                })
            })
            .collect();
        (axes, buttons)
    }
}

/// The layout of a device by its USB ids.
pub fn find(usb_id: (u16, u16)) -> Option<&'static Profile> {
    PROFILES.iter().find(|p| p.ids.contains(&usb_id))
}

use Action as A;
use AxisControl as C;
use Dir::*;
use Hid::*;
use In::*;

const fn act(a: Action) -> ButtonTarget {
    ButtonTarget::Action(a)
}

const GUN: ButtonTarget = ButtonTarget::Gun;

/// The conventions every layout follows, as far as the device's controls allow: a lone stick's
/// trigger fires the selected weapon and its thumb button the gun (as the VR controllers do); a
/// HOTAS stick with a weapon release button fires the gun with the trigger and the weapon with
/// the release. The point-of-view hat looks around (trims on the Warthog, as its trim hat), a
/// target management hat locks (up: nearest target ahead, right: next target, down: drop the
/// locks, left: designate), a countermeasures hat dispenses (up: flare, down: chaff, right:
/// decoy, left: ECM), a throttle's slew stick or mouse slews the sensor and its push locks under
/// the cursor. Ejection is never on a default button.
pub static PROFILES: &[Profile] = &[
    // Button numbers and axes: EDRefCard's device templates (Elite Dangerous binding reference),
    // OpenFlight / Buzzec device notes and community Elite Dangerous binding files.
    Profile {
        name: "Thrustmaster T.16000M",
        ids: &[(0x044f, 0xb10a)],
        axes: &[X, Y, Rz, Slider],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Rz, false),
            (C::Throttle, Slider, false),
        ],
        buttons: &[
            (act(A::FireWeapon), B(1)),
            (GUN, B(2)),
            (act(A::SelectWeapon), B(3)),
            (act(A::NearestTarget), B(4)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
            // Left base: top row 5 6 7, bottom row 10 9 8; right base: top 13 12 11, bottom
            // 14 15 16.
            (act(A::Flare), B(5)),
            (act(A::Chaff), B(6)),
            (act(A::Decoy), B(7)),
            (act(A::DesignateTarget), B(10)),
            (act(A::ClearTarget), B(9)),
            (act(A::SelectTarget), B(8)),
            (act(A::Gear), B(13)),
            (act(A::Flaps), B(12)),
            (act(A::Brakes), B(11)),
            (act(A::BayDoors), B(14)),
            (act(A::Autopilot), B(15)),
            (act(A::LockCursor), B(16)),
        ],
    },
    Profile {
        name: "Thrustmaster TWCS Throttle",
        ids: &[(0x044f, 0xb687)],
        // Rx, Ry and the slider: the T.Flight rudder pedals plugged into it.
        axes: &[X, Y, Z, Rx, Ry, Rz, Slider, Dial],
        bind_axes: &[
            (C::Throttle, Z, false),
            (C::Yaw, Rz, false),
            (C::SlewX, X, false),
            (C::SlewY, Y, false),
        ],
        buttons: &[
            (act(A::SwapSoi), B(1)),
            (act(A::Brakes), B(2)),
            (act(A::Gear), B(3)),
            (act(A::Flaps), B(4)),
            (act(A::BayDoors), B(5)),
            (act(A::LockCursor), B(6)),
            // Middle hat: buttons 7 to 10, castle hat 11 to 14 (up, right, down, left).
            (act(A::Flare), B(7)),
            (act(A::Decoy), B(8)),
            (act(A::Chaff), B(9)),
            (act(A::Ecm), B(10)),
            (act(A::NearestTarget), B(11)),
            (act(A::SelectTarget), B(12)),
            (act(A::ClearTarget), B(13)),
            (act(A::DesignateTarget), B(14)),
            (act(A::CameraZoomIn), Hat(0, Up)),
            (act(A::CameraZoomOut), Hat(0, Down)),
            (act(A::Ptt), Hat(0, Left)),
            (act(A::Autopilot), Hat(0, Right)),
        ],
    },
    Profile {
        name: "Thrustmaster T.Flight HOTAS X / 4",
        ids: &[(0x044f, 0xb108), (0x044f, 0xb67b)],
        // The slider: the throttle's rudder rocker (the twist steers by default).
        axes: &[X, Y, Z, Rz, Slider],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Rz, false),
            (C::Throttle, Z, false),
        ],
        buttons: &[
            (act(A::FireWeapon), B(1)),
            (GUN, B(2)),
            (act(A::SelectWeapon), B(3)),
            (act(A::NearestTarget), B(4)),
            (act(A::Flare), B(5)),
            (act(A::Chaff), B(6)),
            (act(A::Gear), B(7)),
            (act(A::BayDoors), B(8)),
            (act(A::Brakes), B(9)),
            (act(A::Flaps), B(10)),
            (act(A::SelectTarget), B(11)),
            (act(A::Menu), B(12)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "Thrustmaster T.Flight Stick X",
        ids: &[(0x044f, 0xb106)],
        axes: &[X, Y, Rz, Slider],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Rz, false),
            (C::Throttle, Slider, false),
        ],
        buttons: &[
            (act(A::FireWeapon), B(1)),
            (GUN, B(2)),
            (act(A::SelectWeapon), B(3)),
            (act(A::NearestTarget), B(4)),
            (act(A::Flare), B(5)),
            (act(A::Chaff), B(6)),
            (act(A::Gear), B(7)),
            (act(A::BayDoors), B(8)),
            (act(A::Brakes), B(9)),
            (act(A::Flaps), B(10)),
            (act(A::SelectTarget), B(11)),
            (act(A::Menu), B(12)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "Thrustmaster HOTAS Warthog Joystick",
        ids: &[(0x044f, 0x0402)],
        axes: &[X, Y],
        bind_axes: &[(C::Roll, X, false), (C::Pitch, Y, false)],
        buttons: &[
            (GUN, B(1)),
            (act(A::FireWeapon), B(2)),
            (act(A::LockCursor), B(3)),
            (act(A::Autopilot), B(4)),
            (act(A::SelectWeapon), B(5)),
            // TMS 7 to 10, DMS 11 to 14, CMS 15 to 18 (up, right, down, left) and 19 (push).
            (act(A::NearestTarget), B(7)),
            (act(A::SelectTarget), B(8)),
            (act(A::ClearTarget), B(9)),
            (act(A::DesignateTarget), B(10)),
            (act(A::CameraZoomIn), B(11)),
            (act(A::SwapSoi), B(12)),
            (act(A::CameraZoomOut), B(13)),
            (act(A::Flare), B(15)),
            (act(A::Decoy), B(16)),
            (act(A::Chaff), B(17)),
            (act(A::Ecm), B(18)),
            (act(A::TrimNoseDown), Hat(0, Up)),
            (act(A::TrimNoseUp), Hat(0, Down)),
            (act(A::TrimLeft), Hat(0, Left)),
            (act(A::TrimRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "Thrustmaster HOTAS Warthog Throttle",
        ids: &[(0x044f, 0x0404)],
        // Z the right lever, Rz the left one, the slider the friction lever.
        axes: &[X, Y, Z, Rz, Slider],
        bind_axes: &[(C::Throttle, Z, false), (C::SlewX, X, false), (C::SlewY, Y, false)],
        buttons: &[
            (act(A::LockCursor), B(1)),
            // Mic switch: 2 push, 3 to 6 up, right, down, left.
            (act(A::Ptt), B(2)),
            (act(A::CameraZoomIn), B(3)),
            (act(A::CameraZoomOut), B(5)),
            // Speed brake aft.
            (act(A::Brakes), B(7)),
            (act(A::Gear), B(15)),
            (act(A::BayDoors), B(20)),
            (act(A::MasterCaution), B(21)),
            (act(A::Flaps), B(23)),
            (act(A::Autopilot), B(26)),
            // The coolie hat.
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "Logitech Extreme 3D Pro",
        ids: &[(0x046d, 0xc215)],
        axes: &[X, Y, Rz, Slider],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Rz, false),
            (C::Throttle, Slider, false),
        ],
        buttons: &[
            (act(A::FireWeapon), B(1)),
            (GUN, B(2)),
            // Top: 3 and 4 below, 5 and 6 above; base: 7 to 12.
            (act(A::SelectWeapon), B(3)),
            (act(A::NearestTarget), B(4)),
            (act(A::Flare), B(5)),
            (act(A::Chaff), B(6)),
            (act(A::Gear), B(7)),
            (act(A::Flaps), B(8)),
            (act(A::Brakes), B(9)),
            (act(A::BayDoors), B(10)),
            (act(A::SelectTarget), B(11)),
            (act(A::Menu), B(12)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "Saitek X52",
        ids: &[(0x06a3, 0x0255), (0x06a3, 0x075c)],
        axes: &[X, Y, Z, Rx, Ry, Rz, Slider],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Rz, false),
            (C::Throttle, Z, false),
        ],
        buttons: &[
            (GUN, B(1)),
            // Fire, under the safety cover.
            (act(A::FireWeapon), B(2)),
            (act(A::SelectWeapon), B(3)),
            (act(A::NearestTarget), B(4)),
            (act(A::SwapSoi), B(5)),
            (act(A::Ptt), B(6)),
            (act(A::Flare), B(7)),
            (act(A::Chaff), B(8)),
            // Toggles T1 to T6.
            (act(A::Gear), B(9)),
            (act(A::Flaps), B(10)),
            (act(A::Brakes), B(11)),
            (act(A::BayDoors), B(13)),
            (act(A::Autopilot), B(14)),
            // The stick's second hat: 16 down, 17 right, 18 up, 19 left.
            (act(A::ClearTarget), B(16)),
            (act(A::SelectTarget), B(17)),
            (act(A::DesignateTarget), B(19)),
            // The throttle's hat: 20 up, 21 right, 22 down, 23 left.
            (act(A::CameraZoomIn), B(20)),
            (act(A::Decoy), B(21)),
            (act(A::CameraZoomOut), B(22)),
            (act(A::Ecm), B(23)),
            (act(A::LockCursor), B(31)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "Saitek X52 Pro",
        ids: &[(0x06a3, 0x0762)],
        axes: &[X, Y, Z, Rx, Ry, Rz, Slider],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Rz, false),
            (C::Throttle, Z, false),
        ],
        buttons: &[
            (GUN, B(1)),
            (act(A::FireWeapon), B(2)),
            (act(A::SelectWeapon), B(3)),
            (act(A::NearestTarget), B(4)),
            (act(A::SwapSoi), B(5)),
            (act(A::Ptt), B(6)),
            (act(A::Flare), B(7)),
            (act(A::Chaff), B(8)),
            (act(A::Gear), B(9)),
            (act(A::Flaps), B(10)),
            (act(A::Brakes), B(11)),
            (act(A::BayDoors), B(13)),
            (act(A::Autopilot), B(14)),
            (act(A::LockCursor), B(16)),
            // The stick's second hat: 20 to 23, the throttle's 24 to 27 (up, right, down,
            // left).
            (act(A::SelectTarget), B(21)),
            (act(A::ClearTarget), B(22)),
            (act(A::DesignateTarget), B(23)),
            (act(A::CameraZoomIn), B(24)),
            (act(A::Decoy), B(25)),
            (act(A::CameraZoomOut), B(26)),
            (act(A::Ecm), B(27)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "Saitek X-55 Rhino Stick",
        ids: &[(0x0738, 0x2215)],
        axes: &[X, Y, Rz],
        bind_axes: &[(C::Roll, X, false), (C::Pitch, Y, false), (C::Yaw, Rz, false)],
        buttons: X5X_STICK_BUTTONS,
    },
    Profile {
        name: "Logitech X56 Rhino Stick",
        ids: &[(0x0738, 0x2221)],
        // Rx and Ry: the thumb stick.
        axes: &[X, Y, Rx, Ry, Rz],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Rz, false),
            (C::LookX, Rx, false),
            (C::LookY, Ry, false),
        ],
        buttons: X5X_STICK_BUTTONS,
    },
    Profile {
        name: "Saitek X-55 Rhino Throttle",
        ids: &[(0x0738, 0xa215)],
        axes: &[X, Y, Z, Rx, Ry, Rz, Slider, Slider2],
        bind_axes: &[(C::Throttle, X, false)],
        buttons: X5X_THROTTLE_BUTTONS,
    },
    Profile {
        name: "Logitech X56 Rhino Throttle",
        ids: &[(0x0738, 0xa221)],
        // X and Y the levers (X both while they are locked together), Rx and Ry the slew
        // stick.
        axes: &[X, Y, Z, Rx, Ry, Rz, Slider, Slider2],
        bind_axes: &[(C::Throttle, X, false), (C::SlewX, Rx, false), (C::SlewY, Ry, false)],
        buttons: X5X_THROTTLE_BUTTONS,
    },
    Profile {
        name: "Saitek Pro Flight Rudder Pedals",
        ids: &[(0x06a3, 0x0763), (0x06a3, 0x0764)],
        // X and Y: the toe brakes.
        axes: &[X, Y, Rz],
        bind_axes: &[(C::Yaw, Rz, false)],
        buttons: &[],
    },
    Profile {
        name: "Thrustmaster T-Rudder",
        ids: &[(0x044f, 0xb679)],
        axes: &[X, Y, Z],
        bind_axes: &[(C::Yaw, Z, false)],
        buttons: &[],
    },
    Profile {
        name: "CH Pro Pedals",
        ids: &[(0x068e, 0x00f2), (0x068e, 0x0501)],
        axes: &[X, Y, Z],
        bind_axes: &[(C::Yaw, Z, false)],
        buttons: &[],
    },
    Profile {
        name: "CH Fighterstick",
        ids: &[(0x068e, 0x00f3)],
        // Z: the wheel on the base.
        axes: &[X, Y, Z],
        bind_axes: &[(C::Roll, X, false), (C::Pitch, Y, false)],
        buttons: &[
            (GUN, B(1)),
            (act(A::FireWeapon), B(2)),
            (act(A::SelectWeapon), B(3)),
            (act(A::Ptt), B(4)),
            // Hat 2: 5 to 8, castle 9 to 12, thumb hat 13 to 16 (up, right, down, left).
            (act(A::NearestTarget), B(5)),
            (act(A::SelectTarget), B(6)),
            (act(A::ClearTarget), B(7)),
            (act(A::DesignateTarget), B(8)),
            (act(A::Flare), B(9)),
            (act(A::Decoy), B(10)),
            (act(A::Chaff), B(11)),
            (act(A::Ecm), B(12)),
            (act(A::CameraZoomIn), B(13)),
            (act(A::SwapSoi), B(14)),
            (act(A::CameraZoomOut), B(15)),
            (act(A::LockCursor), B(16)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "CH Combatstick",
        ids: &[(0x068e, 0x00f4)],
        axes: &[X, Y, Z],
        bind_axes: &[(C::Roll, X, false), (C::Pitch, Y, false)],
        buttons: &[
            (GUN, B(1)),
            (act(A::SelectWeapon), B(2)),
            (act(A::FireWeapon), B(3)),
            (act(A::Ptt), B(4)),
            (act(A::LockCursor), B(5)),
            (act(A::SwapSoi), B(6)),
            // The second hat: 7 up, 8 right, 9 down, 10 left.
            (act(A::NearestTarget), B(7)),
            (act(A::SelectTarget), B(8)),
            (act(A::ClearTarget), B(9)),
            (act(A::DesignateTarget), B(10)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "CH Pro Throttle",
        ids: &[(0x068e, 0x00f1)],
        // X and Y: the mouse stick.
        axes: &[X, Y, Z],
        bind_axes: &[(C::Throttle, Z, false), (C::SlewX, X, false), (C::SlewY, Y, false)],
        buttons: &[
            // Castle hat: 1 right, 2 down, 3 left, 4 up.
            (act(A::Decoy), B(1)),
            (act(A::Chaff), B(2)),
            (act(A::Ecm), B(3)),
            (act(A::Flare), B(4)),
            // Bottom hat: 5 right, 6 down, 7 left, 8 up.
            (act(A::SelectTarget), B(5)),
            (act(A::ClearTarget), B(6)),
            (act(A::DesignateTarget), B(7)),
            (act(A::NearestTarget), B(8)),
            // Rear hat: 9 down, 11 up.
            (act(A::CameraZoomOut), B(9)),
            (act(A::CameraZoomIn), B(11)),
            (act(A::LockCursor), B(13)),
            (act(A::Brakes), B(14)),
            (act(A::Gear), B(15)),
            (act(A::Ptt), B(16)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "VKB Gladiator NXT EVO",
        // Right, left, right with the OmniThrottle adapter.
        ids: &[(0x231d, 0x0200), (0x231d, 0x0201), (0x231d, 0x3200)],
        // Z: the base's throttle wheel, Rx and Ry: the C1 thumb stick.
        axes: &[X, Y, Z, Rx, Ry, Rz, Slider, Slider2],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Rz, false),
            (C::Throttle, Z, false),
            (C::SlewX, Rx, false),
            (C::SlewY, Ry, false),
        ],
        buttons: &[
            (GUN, B(1)),
            // A2, the red thumb button.
            (act(A::FireWeapon), B(3)),
            (act(A::SelectWeapon), B(4)),
            (act(A::Ptt), B(5)),
            // A3: 6 to 9 (up, right, down, left), 10 push; A4: 11 to 14, 15 push.
            (act(A::NearestTarget), B(6)),
            (act(A::SelectTarget), B(7)),
            (act(A::ClearTarget), B(8)),
            (act(A::DesignateTarget), B(9)),
            (act(A::SwapSoi), B(10)),
            (act(A::Flare), B(11)),
            (act(A::Decoy), B(12)),
            (act(A::Chaff), B(13)),
            (act(A::Ecm), B(14)),
            (act(A::LockCursor), B(20)),
            // The pinky lever pulled.
            (act(A::Brakes), B(22)),
            (act(A::Gear), B(27)),
            (act(A::Flaps), B(28)),
            (act(A::BayDoors), B(29)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "VKB Gunfighter SCG",
        ids: &[(0x231d, 0x0126), (0x231d, 0x0127)],
        axes: &[X, Y, Rx, Ry, Rz],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Rz, false),
            (C::SlewX, Rx, false),
            (C::SlewY, Ry, false),
        ],
        buttons: &[
            (GUN, B(1)),
            (act(A::FireWeapon), B(3)),
            (act(A::SelectWeapon), B(4)),
            (act(A::Ptt), B(5)),
            // A4: 6 to 9 (up, right, down, left), 10 push; A3: 11 to 14, 15 push.
            (act(A::NearestTarget), B(6)),
            (act(A::SelectTarget), B(7)),
            (act(A::ClearTarget), B(8)),
            (act(A::DesignateTarget), B(9)),
            (act(A::SwapSoi), B(10)),
            (act(A::Flare), B(11)),
            (act(A::Decoy), B(12)),
            (act(A::Chaff), B(13)),
            (act(A::Ecm), B(14)),
            (act(A::LockCursor), B(25)),
            (act(A::Brakes), B(27)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "VKB Gunfighter MCG",
        ids: &[(0x231d, 0x0125)],
        // Z: the twist; Rx and Ry the master mode hat's, Rz and the slider the gate control's
        // analog axes, the second slider the brake lever.
        axes: &[X, Y, Z, Rx, Ry, Rz, Slider, Slider2],
        bind_axes: &[(C::Roll, X, false), (C::Pitch, Y, false), (C::Yaw, Z, false)],
        buttons: &[
            (GUN, B(1)),
            (act(A::SelectWeapon), B(3)),
            (act(A::Autopilot), B(4)),
            (act(A::FireWeapon), B(6)),
            (act(A::Ptt), B(7)),
            // RESET hat 10 to 13, PC hat 14 to 17 and 18 push, DC hat 19 to 22 and 23 push
            // (up, right, down, left).
            (act(A::TrimNoseDown), B(10)),
            (act(A::TrimRight), B(11)),
            (act(A::TrimNoseUp), B(12)),
            (act(A::TrimLeft), B(13)),
            (act(A::NearestTarget), B(14)),
            (act(A::SelectTarget), B(15)),
            (act(A::ClearTarget), B(16)),
            (act(A::DesignateTarget), B(17)),
            (act(A::LockCursor), B(18)),
            (act(A::Flare), B(19)),
            (act(A::Decoy), B(20)),
            (act(A::Chaff), B(21)),
            (act(A::Ecm), B(22)),
            (act(A::SwapSoi), B(23)),
            (act(A::PanUp), Hat(0, Up)),
            (act(A::PanDown), Hat(0, Down)),
            (act(A::PanLeft), Hat(0, Left)),
            (act(A::PanRight), Hat(0, Right)),
        ],
    },
    Profile {
        name: "Virpil Constellation Alpha Prime (right) on WarBRD-D",
        ids: &[(0x3344, 0x43f5)],
        // Z: the twist, Rx and Ry the thumb stick, the slider the brake lever.
        axes: &[X, Y, Z, Rx, Ry, Slider],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Z, false),
            (C::SlewX, Rx, false),
            (C::SlewY, Ry, false),
        ],
        buttons: &[
            // The flip trigger pulled.
            (act(A::FireWeapon), B(3)),
            (GUN, B(4)),
            (act(A::LockCursor), B(6)),
            (act(A::SelectWeapon), B(7)),
            // Face hats: 8 to 12 and 14 to 18, thumb hat 23 to 27 (push, up, left, down,
            // right).
            (act(A::SwapSoi), B(8)),
            (act(A::NearestTarget), B(9)),
            (act(A::DesignateTarget), B(10)),
            (act(A::ClearTarget), B(11)),
            (act(A::SelectTarget), B(12)),
            (act(A::Ptt), B(13)),
            (act(A::Flare), B(15)),
            (act(A::Ecm), B(16)),
            (act(A::Chaff), B(17)),
            (act(A::Decoy), B(18)),
            (act(A::PanUp), B(24)),
            (act(A::PanLeft), B(25)),
            (act(A::PanDown), B(26)),
            (act(A::PanRight), B(27)),
        ],
    },
    Profile {
        name: "Virpil Constellation Alpha (right) on MongoosT-50CM3",
        ids: &[(0x3344, 0x0388)],
        axes: &[X, Y, Z, Rx, Ry, Slider],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Z, false),
            (C::SlewX, Rx, false),
            (C::SlewY, Ry, false),
        ],
        buttons: &[
            (act(A::FireWeapon), B(2)),
            (GUN, B(3)),
            (act(A::LockCursor), B(5)),
            (act(A::SelectWeapon), B(6)),
            // Face hats: 7 to 11 and 13 to 17, thumb hat 25 to 29 (push, up, right, down,
            // left).
            (act(A::SwapSoi), B(7)),
            (act(A::Flare), B(8)),
            (act(A::Decoy), B(9)),
            (act(A::Chaff), B(10)),
            (act(A::Ecm), B(11)),
            (act(A::Ptt), B(12)),
            (act(A::NearestTarget), B(14)),
            (act(A::SelectTarget), B(15)),
            (act(A::ClearTarget), B(16)),
            (act(A::DesignateTarget), B(17)),
            (act(A::PanUp), B(26)),
            (act(A::PanRight), B(27)),
            (act(A::PanDown), B(28)),
            (act(A::PanLeft), B(29)),
        ],
    },
    Profile {
        name: "Virpil AeroMax-R",
        ids: &[(0x3344, 0x44c4)],
        // Z: the twist, Rx and Ry the mini stick, Rz the brake lever.
        axes: &[X, Y, Z, Rx, Ry, Rz],
        bind_axes: &[
            (C::Roll, X, false),
            (C::Pitch, Y, false),
            (C::Yaw, Z, false),
            (C::SlewX, Rx, false),
            (C::SlewY, Ry, false),
        ],
        buttons: &[
            (GUN, B(1)),
            (act(A::SelectWeapon), B(5)),
            (act(A::LockCursor), B(6)),
            // Thumb hat 7 to 11, hat 1 14 to 18, hat 2 20 to 24 (push, up, right, down, left).
            (act(A::PanUp), B(8)),
            (act(A::PanRight), B(9)),
            (act(A::PanDown), B(10)),
            (act(A::PanLeft), B(11)),
            (act(A::Ptt), B(12)),
            (act(A::FireWeapon), B(13)),
            (act(A::SwapSoi), B(14)),
            (act(A::NearestTarget), B(15)),
            (act(A::SelectTarget), B(16)),
            (act(A::ClearTarget), B(17)),
            (act(A::DesignateTarget), B(18)),
            (act(A::Brakes), B(19)),
            (act(A::Flare), B(21)),
            (act(A::Decoy), B(22)),
            (act(A::Chaff), B(23)),
            (act(A::Ecm), B(24)),
        ],
    },
    // Thrustmaster's TARGET header (target.tmh) and Viper TQS manual.
    Profile {
        name: "Thrustmaster Viper TQS",
        ids: &[(0x044f, 0x0412)],
        // X and Y: the radar cursor, Rx the top knob, Ry the side thumb wheel.
        axes: &[X, Y, Z, Rx, Ry],
        bind_axes: &[(C::Throttle, Z, false), (C::SlewX, X, false), (C::SlewY, Y, false)],
        buttons: &[(act(A::SwapSoi), B(5)), (act(A::LockCursor), B(12))],
    },
    // The ids from an MSFS profile export and SimHub diagnostics; the grips' buttons are not
    // documented, so only the stick.
    Profile {
        name: "MOZA AB6 FFB Base",
        ids: &[(0x346e, 0x1002)],
        axes: &[X, Y],
        bind_axes: &[(C::Roll, X, false), (C::Pitch, Y, false)],
        buttons: &[],
    },
    // SDL's flight stick list; the grip on the base varies, so only the stick.
    Profile {
        name: "Virpil MongoosT-50CM3 Stick",
        ids: &[(0x3344, 0x4391), (0x3344, 0x8390)],
        axes: &[X, Y],
        bind_axes: &[(C::Roll, X, false), (C::Pitch, Y, false)],
        buttons: &[],
    },
    // Buzzec/virpil (buttons), owners' DCS profiles (axes); right and left hand.
    Profile {
        name: "Virpil MongoosT-50CM3 Throttle",
        ids: &[(0x3344, 0x0194), (0x3344, 0x8194)],
        // X and Y: the mini stick, Z the slider, Rx and Ry the left and right levers, Rz the
        // flaps lever.
        axes: &[X, Y, Z, Rx, Ry, Rz],
        bind_axes: &[(C::Throttle, Rx, false), (C::SlewX, X, false), (C::SlewY, Y, false)],
        buttons: &[
            (act(A::Ptt), B(4)),
            // Index hat: 8 push, 9 down, 10 right, 11 up, 12 left.
            (act(A::Ecm), B(8)),
            (act(A::Chaff), B(9)),
            (act(A::Decoy), B(10)),
            (act(A::Flare), B(11)),
            (act(A::LockCursor), B(13)),
            (act(A::Brakes), B(21)),
            // Upper hat: 22 push, 23 down, 24 back, 25 up, 26 forward.
            (act(A::SwapSoi), B(22)),
            (act(A::ClearTarget), B(23)),
            (act(A::DesignateTarget), B(24)),
            (act(A::NearestTarget), B(25)),
            (act(A::SelectTarget), B(26)),
            // Lower hat up and down.
            (act(A::CameraZoomOut), B(28)),
            (act(A::CameraZoomIn), B(30)),
            // B1 to B6 on the base.
            (act(A::Gear), B(38)),
            (act(A::Flaps), B(39)),
            (act(A::BayDoors), B(40)),
            (act(A::Autopilot), B(41)),
            (act(A::MasterCaution), B(42)),
        ],
    },
    // A raw HID dump of the Mk.IV (VKB lets owners change the ids).
    Profile {
        name: "VKB T-Rudder",
        ids: &[(0x231d, 0x011f)],
        // Ry and Rz: the virtual toe brakes.
        axes: &[Rx, Ry, Rz],
        bind_axes: &[(C::Yaw, Rx, false)],
        buttons: &[],
    },
    // github.com/RoystonS/honeycomb (bravo-controls.md); the ids from BetterBravoLights.
    Profile {
        name: "Honeycomb Bravo Throttle Quadrant",
        ids: &[(0x294b, 0x1901)],
        // Levers 1 to 6: Y, X, Rz, Ry, Rx, Z.
        axes: &[X, Y, Z, Rx, Ry, Rz],
        bind_axes: &[(C::Throttle, Y, false), (C::Nozzle, X, false)],
        buttons: &[
            (act(A::Autopilot), B(8)),
            // The autopilot knob turned.
            (act(A::CameraZoomIn), B(13)),
            (act(A::CameraZoomOut), B(14)),
            // The flaps lever pushed down (it springs back).
            (act(A::Flaps), B(15)),
            // The trim wheel.
            (act(A::TrimNoseDown), B(22)),
            (act(A::TrimNoseUp), B(23)),
            // The gear lever holds 31 up, 32 down: either move toggles the gear.
            (act(A::Gear), B(31)),
            (act(A::Gear), B(32)),
        ],
    },
];

/// The X-55 and X56 sticks: 1 trigger, 2 A (fire), 3 B, 4 C (the thumb stick's push on the
/// X56), 5 D (pinky), 6 pinky trigger, H1 7 to 10 and H2 11 to 14 (up, right, down, left).
const X5X_STICK_BUTTONS: &[(ButtonTarget, In)] = &[
    (GUN, B(1)),
    (act(A::FireWeapon), B(2)),
    (act(A::SelectWeapon), B(3)),
    (act(A::LockCursor), B(4)),
    (act(A::Ptt), B(5)),
    (act(A::SwapSoi), B(6)),
    (act(A::NearestTarget), B(7)),
    (act(A::SelectTarget), B(8)),
    (act(A::ClearTarget), B(9)),
    (act(A::DesignateTarget), B(10)),
    (act(A::Flare), B(11)),
    (act(A::Decoy), B(12)),
    (act(A::Chaff), B(13)),
    (act(A::Ecm), B(14)),
    (act(A::PanUp), Hat(0, Up)),
    (act(A::PanDown), Hat(0, Down)),
    (act(A::PanLeft), Hat(0, Left)),
    (act(A::PanRight), Hat(0, Right)),
];

/// The X-55 and X56 throttles: 1 E, SW1 to SW6 6 to 11, H3 20 to 23 and H4 24 to 27 (up,
/// right, down, left), 32 the X56's slew stick push.
const X5X_THROTTLE_BUTTONS: &[(ButtonTarget, In)] = &[
    (act(A::Brakes), B(1)),
    (act(A::Gear), B(6)),
    (act(A::Flaps), B(7)),
    (act(A::BayDoors), B(8)),
    (act(A::Autopilot), B(9)),
    (act(A::Flare), B(20)),
    (act(A::Decoy), B(21)),
    (act(A::Chaff), B(22)),
    (act(A::Ecm), B(23)),
    (act(A::CameraZoomIn), B(24)),
    (act(A::CameraZoomOut), B(26)),
    (act(A::LockCursor), B(32)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn linux_codes() {
        let p =
            Profile { name: "", ids: &[], axes: &[X, Y, Rz, Slider], bind_axes: &[], buttons: &[] };
        let l = Platform::Linux;
        assert_eq!(l.axis(&p, Slider), Some(0x30006));
        assert_eq!(l.axis(&p, Rz), Some(0x30005));
        assert_eq!(l.source(B(1)), Some(Source::Button(0x10120)));
        assert_eq!(l.source(B(16)), Some(Source::Button(0x1012f)));
        assert_eq!(l.source(B(17)), Some(Source::Button(0x102c0)));
        assert_eq!(l.source(B(56)), Some(Source::Button(0x102e7)));
        assert_eq!(l.source(B(57)), None);
        assert_eq!(l.source(Hat(0, Up)), Some(Source::AxisNegative(0x30011)));
        assert_eq!(l.source(Hat(1, Right)), Some(Source::AxisPositive(0x30012)));
    }

    #[test]
    fn windows_codes() {
        let p =
            Profile { name: "", ids: &[], axes: &[X, Y, Rz, Slider], bind_axes: &[], buttons: &[] };
        let w = Platform::Windows;
        assert_eq!(w.axis(&p, Rz), Some(0x10002));
        assert_eq!(w.axis(&p, Slider), Some(0x10003));
        assert_eq!(w.axis(&p, Z), None);
        assert_eq!(w.source(B(1)), Some(Source::Button(0)));
        assert_eq!(w.source(Hat(0, Down)), Some(Source::AxisPositive(0x20001)));
        assert_eq!(w.source(Hat(1, Left)), Some(Source::AxisNegative(0x20002)));
    }

    /// Every layout names axes its device has, binds a control and a target once and an input
    /// once, and no two layouts claim a device.
    #[test]
    fn layouts_are_consistent() {
        let mut ids = HashSet::new();
        for p in PROFILES {
            assert!(!p.ids.is_empty(), "{}", p.name);
            for id in p.ids {
                assert!(ids.insert(*id), "{} {id:04x?} twice", p.name);
            }
            let mut controls = HashSet::new();
            for &(c, hid, _) in p.bind_axes {
                assert!(p.axes.contains(&hid), "{}: {hid:?} not on the device", p.name);
                assert!(controls.insert(c), "{}: {c:?} twice", p.name);
            }
            let mut used = HashSet::new();
            for &(_, hid, _) in p.bind_axes {
                assert!(used.insert(format!("{hid:?}")), "{}: {hid:?} twice", p.name);
            }
            // A target twice only for a lever's two positions (the gear's).
            let mut targets = HashSet::new();
            let mut inputs = HashSet::new();
            for &(t, input) in p.buttons {
                assert!(targets.insert(t) || t == act(A::Gear), "{}: {t:?} twice", p.name);
                assert!(inputs.insert(input), "{}: {input:?} twice", p.name);
            }
            for platform in [Platform::Linux, Platform::Windows] {
                let (axes, buttons) = p.mappings("d", platform);
                assert_eq!(buttons.len(), p.buttons.len(), "{} {platform:?}", p.name);
                if platform == Platform::Windows {
                    assert_eq!(axes.len(), p.bind_axes.len(), "{}", p.name);
                }
            }
        }
    }
}

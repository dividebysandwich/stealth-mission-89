//! Pushable cockpit controls.
//!
//! Panel buttons are caps carrying a face of the instrument atlas (a lamp or label, so lit/unlit
//! comes from the panel drawing code); the layout says which faces are caps. The modern
//! panel's MFD bezel buttons are caps too, with page-dependent labels (`mfd::osb`). Extra
//! buttons on two side panels carry text labels. Every control has a local frame with +Z pointing out of
//! its face. A VR fingertip touching a control highlights it (with a light haptic tick) and
//! pulling the trigger then fires it; a mouse click or a script `press=` fires it directly.

use bevy::prelude::*;

use super::Hands;
use super::layout::Effect;
use super::pick::{Picker, Pointed};
use crate::camera::{View, ViewState};
use crate::flight::{Aircraft, Player};
use crate::hud::{HudMode, HudState};
use crate::input::{Action, ActionEvent};
use crate::mfd::{CamDir, MfdCommand, MfdState, OsbFn, cmd_action, osb};

/// Presses the 3D-cockpit control bound to an action (scripts).
#[derive(Message, Debug, Clone, Copy)]
pub struct PressControl(pub Action);

/// An air defence console's button was pressed.
#[derive(Message, Debug, Clone, Copy)]
pub struct ConsoleButton(pub super::layout::ConsoleKey);

/// What the air defence console's buttons show (`station` sets it): those not used at this
/// site (hidden, not pressable) and those lit, with their light's colour.
#[derive(Resource, Default, PartialEq)]
pub struct ConsolePanel {
    pub hidden: Vec<super::layout::ConsoleKey>,
    pub lit: Vec<(super::layout::ConsoleKey, u8)>,
}

/// A console button `ConsolePanel` hides.
#[derive(Component)]
pub(super) struct Unused;

/// A console screen was clicked (mouse) or tapped (VR fingertip and trigger) at `uv` (0..1
/// across the screen, from its top left).
#[derive(Message, Debug, Clone, Copy)]
pub struct ScreenClick {
    pub screen: super::displays::ConsoleScreen,
    pub uv: Vec2,
}

impl Effect {
    /// The key action a script `press=` names for this control (None: blank bezel button).
    pub(super) fn action(self, mfd: &MfdState) -> Option<Action> {
        Some(match self {
            Effect::Action(a) => a,
            Effect::HudMode(_) => Action::HudMode,
            Effect::LeftPage(_) => Action::Maps,
            Effect::CamCycle => Action::CamAhead,
            Effect::TimeToggle => Action::TimeAccel,
            Effect::Console(_) => return None,
            Effect::Osb(side, slot) => match osb(side, slot, mfd)?.1 {
                OsbFn::Cmd(c) => cmd_action(c),
                OsbFn::Action(a) => a,
                OsbFn::Readout => return None,
            },
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// Cap on the panel carrying an atlas face (uv min, max), size in panel pixels.
    Cap,
    /// MFD bezel button.
    Osb,
    /// Labelled button on a side panel.
    Side,
    /// Landing gear lever (left of the panel).
    GearLever,
    /// Ejection handle (front of the seat pan).
    EjectHandle,
}

/// Fingertip radius.
const FINGER: f32 = 0.008;
/// A fingertip touches a control within this distance of its face (in front or pushed in), and
/// this far beyond its footprint.
const TOUCH_GAP: f32 = 0.012;
const TOUCH_MARGIN: f32 = 0.006;

#[derive(Component)]
pub(super) struct Control {
    kind: Kind,
    pub(super) effect: Effect,
    half: Vec2,
    height: f32,
    travel: f32,
    /// Current push depth (metres), from the fingers or a click animation.
    depth: f32,
    /// A VR fingertip touches it.
    touched: bool,
    /// Seconds left of a click/script press animation.
    click: f32,
}

/// The moving part of a control (cap, lever arm).
#[derive(Component)]
pub(super) struct Moving;

/// Frame around a control shown while a fingertip touches it.
#[derive(Component)]
pub(super) struct Highlight;

impl Control {
    /// A control from the cockpit model's object properties: `kind` (cap, osb, side, gear,
    /// eject), `control` (the effect, e.g. `Action(Gear)`, `HudMode(1)`, `Osb(Left, 3)`),
    /// `half` (half width and height of the face, metres), `height` (face above the origin) and
    /// `travel` (press depth).
    pub(super) fn from_model(extras: &serde_json::Value) -> Option<Self> {
        let kind = match extras["kind"].as_str()? {
            "cap" => Kind::Cap,
            "osb" => Kind::Osb,
            "side" => Kind::Side,
            "gear" => Kind::GearLever,
            "eject" => Kind::EjectHandle,
            _ => return None,
        };
        let effect = parse_effect(extras["control"].as_str()?)?;
        let f = |v: &serde_json::Value| v.as_f64().map(|x| x as f32);
        let half = Vec2::new(f(&extras["half"][0])?, f(&extras["half"][1])?);
        Some(Self {
            kind,
            effect,
            half,
            height: f(&extras["height"])?,
            travel: f(&extras["travel"])?,
            depth: 0.0,
            touched: false,
            click: 0.0,
        })
    }

    pub(super) fn is_eject(&self) -> bool {
        self.kind == Kind::EjectHandle
    }

    /// A VR fingertip touches it.
    pub(super) fn touched(&self) -> bool {
        self.touched
    }

    /// The centre of its face, in the frame of `tf` (its own transform).
    pub(super) fn face_centre(&self, tf: &Transform) -> Vec3 {
        tf.translation + tf.rotation * Vec3::Z * self.height
    }

    /// Where the ray o + t d (cockpit frame, `tf` the control's) goes through the face
    /// rectangle: t.
    pub(super) fn face_hit(&self, tf: &Transform, o: Vec3, d: Vec3) -> Option<f32> {
        let n = tf.rotation * Vec3::Z;
        let plane = tf.translation + n * self.height;
        let denom = d.dot(n);
        if denom.abs() < 1e-9 {
            return None;
        }
        let t = (plane - o).dot(n) / denom;
        let local = tf.rotation.inverse() * (o + d * t - tf.translation);
        (t > 0.0 && local.x.abs() <= self.half.x && local.y.abs() <= self.half.y).then_some(t)
    }
}

/// Parses an effect as written in the cockpit model (`Effect`'s `Debug` text).
pub(super) fn parse_effect(s: &str) -> Option<Effect> {
    let s = s.trim();
    let (name, args) = match s.split_once('(') {
        Some((n, rest)) => {
            (n.trim(), rest.strip_suffix(')')?.split(',').map(str::trim).collect::<Vec<_>>())
        }
        None => (s, Vec::new()),
    };
    Some(match (name, args.as_slice()) {
        ("Action", [a]) => Effect::Action(crate::input::action_by_name(a)?),
        ("HudMode", [n]) => Effect::HudMode(n.parse().ok()?),
        ("LeftPage", ["Map"]) => Effect::LeftPage(crate::mfd::LeftMfd::Map),
        ("LeftPage", ["Tac"]) => Effect::LeftPage(crate::mfd::LeftMfd::Tac),
        ("CamCycle", []) => Effect::CamCycle,
        ("Console", [key, rest @ ..]) => {
            use super::layout::ConsoleKey as K;
            let n = || rest.first().and_then(|n| n.parse::<u8>().ok());
            Effect::Console(match *key {
                "Tracks" => K::Tracks,
                "Site" => K::Site,
                "Lock" => K::Lock,
                "Drop" => K::Drop,
                "Guided" => K::Guided,
                "Semi" => K::Semi,
                "Fire" => K::Fire,
                "Launcher" => K::Launcher(n()?),
                "Refill" => K::Refill(n()?),
                "Flir" => K::Flir,
                "ZoomIn" => K::ZoomIn,
                "ZoomOut" => K::ZoomOut,
                "RangeIn" => K::RangeIn,
                "RangeOut" => K::RangeOut,
                "Leave" => K::Leave,
                _ => return None,
            })
        }
        ("TimeToggle", []) => Effect::TimeToggle,
        ("Osb", [side, n]) => Effect::Osb(
            match *side {
                "Left" => crate::mfd::MfdSide::Left,
                "Right" => crate::mfd::MfdSide::Right,
                "Third" => crate::mfd::MfdSide::Third,
                _ => return None,
            },
            n.parse().ok()?,
        ),
        _ => return None,
    })
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Fire<'w, 's> {
    actions: MessageWriter<'w, ActionEvent>,
    mfd_commands: MessageWriter<'w, MfdCommand>,
    hud: Res<'w, HudState>,
    mfd: Res<'w, MfdState>,
    accel: Res<'w, crate::combat::TimeAccel>,
    console: MessageWriter<'w, ConsoleButton>,
    _p: std::marker::PhantomData<&'s ()>,
}

impl Fire<'_, '_> {
    fn fire(&mut self, effect: Effect) {
        match effect {
            Effect::Action(a) => {
                self.actions.write(ActionEvent(a));
            }
            Effect::HudMode(target) => {
                let cur = match self.hud.mode {
                    HudMode::Nav => 0,
                    HudMode::Air => 1,
                    HudMode::Gnd => 2,
                };
                // A-G again switches between CCIP and CCRP; A-A again goes back to NAV (the
                // F-16's A-A / A-G).
                if target == 2 && cur == 2 {
                    self.actions.write(ActionEvent(Action::BombMode));
                    return;
                }
                let target = if target == cur { 0 } else { target };
                for _ in 0..(target + 3 - cur) % 3 {
                    self.actions.write(ActionEvent(Action::HudMode));
                }
            }
            Effect::LeftPage(p) => {
                if self.mfd.left != p {
                    self.actions.write(ActionEvent(Action::Maps));
                }
            }
            Effect::Osb(side, slot) => match osb(side, slot, &self.mfd).map(|o| o.1) {
                Some(OsbFn::Cmd(c)) => {
                    self.mfd_commands.write(MfdCommand(c));
                }
                Some(OsbFn::Action(a)) => {
                    self.actions.write(ActionEvent(a));
                }
                Some(OsbFn::Readout) | None => {}
            },
            Effect::CamCycle => {
                let next = match self.mfd.cam_dir {
                    CamDir::Ahead => Action::CamRight,
                    CamDir::Right => Action::CamRear,
                    CamDir::Rear => Action::CamLeft,
                    CamDir::Left => Action::CamAhead,
                };
                self.actions.write(ActionEvent(next));
            }
            Effect::TimeToggle => {
                self.actions.write(ActionEvent(if self.accel.on {
                    Action::TimeNormal
                } else {
                    Action::TimeAccel
                }));
            }
            Effect::Console(key) => {
                self.console.write(ConsoleButton(key));
            }
        }
    }
}

/// The control a key action presses: panel caps and bezel buttons before the side panel
/// duplicates; of two bezel buttons with the same key action (MAP / TAC), the one not selected.
pub(super) fn press_target<'a>(
    controls: impl Iterator<Item = (Entity, &'a Control)>,
    a: Action,
    mfd: &MfdState,
) -> Option<Entity> {
    let rank = |c: &Control| {
        let lit = matches!(c.effect, Effect::Osb(side, slot) if osb(side, slot, mfd).is_some_and(|o| o.2));
        (!matches!(c.kind, Kind::Cap | Kind::Osb), lit)
    };
    controls
        .filter(|(_, c)| c.effect.action(mfd) == Some(a))
        .min_by_key(|(_, c)| rank(c))
        .map(|(e, _)| e)
}

/// Seconds a clicked control stays pushed in.
const CLICK_SECONDS: f32 = 0.18;
/// A VR fingertip taps a console screen this close to it (metres).
const SCREEN_TAP: f32 = 0.025;

/// Fingertip touches with the trigger, mouse clicks and script presses.
#[allow(clippy::too_many_arguments)]
pub(super) fn interact(
    time: Res<Time<Real>>,
    settings: Res<crate::settings::Settings>,
    view: Res<ViewState>,
    mut hands: ResMut<Hands>,
    mut controls: Query<(Entity, &mut Control, &Transform), Without<Unused>>,
    mut was_touching: Local<[Option<Entity>; 2]>,
    mut fire: Fire,
    mut presses: MessageReader<PressControl>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut picker: Picker,
    menu: Res<crate::flight_menu::FlightMenu>,
    hide: Option<Res<crate::camera::HideOwnAircraft>>,
    screens: Query<&super::displays::ConsoleCanvas>,
    mut clicks: MessageWriter<ScreenClick>,
) {
    let dt = time.delta_secs();
    // The cockpit is not there (VR, the campaign's Ops screen): nothing to touch.
    if hide.is_some_and(|h| h.0) {
        presses.clear();
        for hand in hands.hands.iter_mut() {
            hand.touching = false;
        }
        *was_touching = [None; 2];
        for (_, mut c, _) in &mut controls {
            c.click = 0.0;
            c.depth = 0.0;
            if c.touched {
                c.touched = false;
            }
        }
        return;
    }
    // With the menu open only the MENU button works (it closes the menu again).
    let usable = |c: &Control| !menu.open || c.effect == Effect::Action(Action::Menu);
    for PressControl(a) in presses.read() {
        let found = press_target(controls.iter().map(|(e, c, _)| (e, c)), *a, &fire.mfd);
        if let Some(e) = found
            && let Ok((_, mut c, _)) = controls.get_mut(e)
        {
            c.click = CLICK_SECONDS;
            fire.fire(c.effect);
        } else {
            warn!("no 3D cockpit control for {a:?}");
        }
    }
    let cockpit_view =
        matches!(view.view, View::Cockpit | View::LookLeft | View::LookRight | View::LookRear);

    // Mouse click: the control under the cursor, unless something in front hides it.
    if !settings.vr
        && cockpit_view
        && buttons.just_pressed(MouseButton::Left)
        && let Some(ray) = picker.cursor_ray()
    {
        let picked = {
            let all: Vec<_> = controls.iter().collect();
            picker.pick(ray, &all, usable)
        };
        if let Some(Pointed::Control(e)) = picked
            && let Ok((_, mut c, _)) = controls.get_mut(e)
            && usable(&c)
        {
            c.click = CLICK_SECONDS;
            fire.fire(c.effect);
        }
        // A console screen: where on it.
        if let Some(Pointed::Tip(e)) = picked
            && !menu.open
            && let Ok(screen) = screens.get(e)
            && let Some((_, uv)) = picker.uv_at(e, ray)
        {
            clicks.write(ScreenClick { screen: screen.0, uv });
        }
    }

    // VR: a fingertip on a console screen and the trigger taps it there (seen from the eye, so
    // where the finger shows on the picture).
    if cockpit_view && !menu.open && !screens.is_empty() {
        for hand in hands.hands.iter_mut() {
            if !hand.active || hand.touching || !hand.trigger_pulled() {
                continue;
            }
            let Some((ray, reach)) = picker.eye_through(hand.finger) else { continue };
            let picked = {
                let all: Vec<_> = controls.iter().collect();
                picker.pick_within(ray, reach + SCREEN_TAP, &all, |_| false)
            };
            if let Some(Pointed::Tip(e)) = picked
                && let Ok(screen) = screens.get(e)
                && let Some((t, uv)) = picker.uv_at(e, ray)
                && t > reach - SCREEN_TAP
            {
                hand.haptic = hand.haptic.max(0.4);
                clicks.write(ScreenClick { screen: screen.0, uv });
            }
        }
    }

    // VR fingertips: each hand touches the control whose face is nearest its fingertip (not
    // while it holds the stick or throttle); the trigger fires it and holds it pushed in.
    let mut touching: [Option<(f32, Entity)>; 2] = [None; 2];
    for (e, mut c, tf) in &mut controls {
        c.click = (c.click - dt).max(0.0);
        for (h, hand) in hands.hands.iter().enumerate() {
            // The ejection handle has to be grabbed and pulled (no accidental pokes).
            if !hand.active || !cockpit_view || c.kind == Kind::EjectHandle || !usable(&c) {
                continue;
            }
            let local = tf.rotation.inverse() * (hand.finger - tf.translation);
            let inside = local.x.abs() <= c.half.x + TOUCH_MARGIN
                && local.y.abs() <= c.half.y + TOUCH_MARGIN
                && local.z > -0.04;
            if !inside {
                continue;
            }
            let face = c.height + FINGER;
            let gap = local.z - face;
            if hand.squeeze < 0.4
                && gap < TOUCH_GAP
                && gap > -c.travel - TOUCH_GAP
                && touching[h].is_none_or(|(g, _)| gap.abs() < g)
            {
                touching[h] = Some((gap.abs(), e));
            }
        }
        c.depth = if c.click > 0.0 { c.travel } else { 0.0 };
    }
    for (h, hand) in hands.hands.iter_mut().enumerate() {
        let touched = touching[h].map(|(_, e)| e);
        hand.touching = touched.is_some();
        if touched.is_some() && touched != was_touching[h] {
            hand.haptic = hand.haptic.max(0.1);
        }
        was_touching[h] = touched;
        if let Some(e) = touched
            && let Ok((_, mut c, _)) = controls.get_mut(e)
        {
            if hand.trigger_pulled() {
                hand.haptic = hand.haptic.max(0.6);
                c.click = CLICK_SECONDS;
                fire.fire(c.effect);
            }
            if hand.trigger > 0.7 {
                c.depth = c.travel;
            }
        }
    }
    for (e, mut c, _) in &mut controls {
        let touched = touching.iter().flatten().any(|&(_, t)| t == e);
        if c.touched != touched {
            c.touched = touched;
        }
    }
}

/// Moves the caps in, the gear lever to the gear position, and shows the touched controls'
/// highlights.
pub(super) fn animate(
    controls: Query<(&Control, &Children)>,
    mut moving: Query<(&mut Transform, &Children), With<Moving>>,
    mut highlights: Query<&mut Visibility, With<Highlight>>,
    aircraft: Query<&Aircraft, With<Player>>,
) {
    let gear = aircraft.single().is_ok_and(|a| a.gear_down);
    for (c, children) in &controls {
        for child in children.iter() {
            if let Ok((mut tf, parts)) = moving.get_mut(child) {
                for part in parts.iter() {
                    if let Ok(mut v) = highlights.get_mut(part) {
                        v.set_if_neq(if c.touched {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        });
                    }
                }
                let mut t = Transform::from_xyz(0.0, 0.0, -c.depth);
                if c.kind == Kind::GearLever {
                    t.rotation = Quat::from_rotation_x(if gear { 0.5 } else { -0.5 });
                }
                if *tf != t {
                    *tf = t;
                }
            }
        }
    }
}

/// Hides the console's unused buttons and lights its lit ones (`ConsolePanel`).
pub(super) fn console_panel(
    mut commands: Commands,
    panel: Res<ConsolePanel>,
    fonts: Res<crate::font::Fonts>,
    controls: Query<(Entity, &Control, Has<Unused>)>,
    mut labels: Query<(Entity, &mut super::displays::SideLabel, &mut crate::canvas::CanvasTexture)>,
    parents: Query<&ChildOf>,
) {
    let key = |e: Entity| match controls.get(e).ok()?.1.effect {
        Effect::Console(k) => Some(k),
        _ => None,
    };
    for (e, c, unused) in &controls {
        let Effect::Console(k) = c.effect else { continue };
        let hide = panel.hidden.contains(&k);
        if hide != unused {
            let mut ec = commands.entity(e);
            if hide {
                ec.insert((Unused, Visibility::Hidden));
            } else {
                ec.remove::<Unused>().insert(Visibility::Inherited);
            }
        }
    }
    for (e, mut label, mut tex) in &mut labels {
        let Some(k) = parents.iter_ancestors(e).find_map(key) else { continue };
        let lit = panel.lit.iter().find(|(l, _)| *l == k).map(|&(_, c)| c);
        if label.lit != lit {
            label.lit = lit;
            super::displays::draw_side_label(&mut tex.canvas, &fonts, &label.text, lit);
            tex.dirty = true;
        }
    }
}

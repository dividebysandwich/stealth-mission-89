//! Stick (right console, or a centre stick between the knees) and throttle (left console).
//!
//! VR: squeeze the grip near the stick to take it. Tilting the controller (relative to how it
//! was held when grabbed) deflects pitch (tilt back = nose up) and roll; twisting it about the
//! stick's axis is the rudder. Letting go re-centres the stick. A centre stick between the knees
//! (the UH-60A's cyclic, the Tornado's) is moved by the hand instead: its grip follows the hand
//! about the pivot, so it doesn't swing further than the hand holding it; so, more and more, does
//! a long side stick.
//! Squeeze near the throttle handle and slide it along its rails: forward = more thrust; it stays where it is
//! released, with a haptic tick every 10 %. With an afterburner the lever has a detent at full dry
//! thrust (a tick, and a gap of no change past it); further forward the afterburner lights and goes out with
//! a rumble of its own. While it burns both controllers shake, harder with its setting.
//! The controller buttons are mapped in the input mapping (`InputConfig::vr_buttons`); a trigger
//! only counts while its hand holds the stick or throttle.
//! A V/STOL type's nozzle lever (the AV-8B's, just inboard of the throttle) is taken the same way
//! by the left hand (the nearer of the two handles): pulled back it swings the nozzles down, a
//! tick every 10 degrees and a firm one on the hover stop (90 degrees) and the end stops.
//! The ejection handle is pulled up with either hand. A hand carrying the kneeboard grabs none.
//! Without VR both animate from the keyboard/gamepad controls.

use bevy::prelude::*;

use super::{
    BUTTON_PRIMARY, BUTTON_SECONDARY, BUTTON_STICK_DOWN, BUTTON_STICK_LEFT, BUTTON_STICK_RIGHT,
    BUTTON_STICK_UP, BUTTON_THUMB, Hands,
};
use crate::flight::{AB_DETENT, AB_GAP, Aircraft, FlightConstants, Player};
use crate::hotas::{ButtonTarget, InputConfig, VrHand, VrInput};
use crate::input::{Action, ActionEvent, FlightControls};

/// Left thumbstick deflection below which it does not slew the cursor.
const SLEW_DEADZONE: f32 = 0.15;
/// Grip centre above the pivot, unless the model says (the stick's `grip`).
const GRIP_HEIGHT: f32 = 0.11;
/// Grip heights over which pitch and roll go from the controller's tilt (a short side stick) to
/// the hand's position about the pivot (a long one).
const LONG_STICK: (f32, f32) = (0.2, 0.5);
/// A stick pivoting this close to the centreline (cockpit frame) is a centre stick: always moved
/// by the hand's position.
const CENTRE_STICK: f32 = 0.05;
/// Full deflection (radians).
const STICK_TILT: f32 = 0.35;
const STICK_TWIST: f32 = 0.45;
/// Rudder deadzone (fraction of full twist); a haptic tick on leaving and re-entering it.
const TWIST_DEADZONE: f32 = 0.15;
/// Re-entering needs this much less twist, so hand jitter at the edge doesn't buzz.
const TWIST_HYSTERESIS: f32 = 0.03;

/// Positions in the cockpit frame taken from the cockpit model: the stick pivot (`stick`
/// object), the throttle handle at idle (`throttle` object) and at full thrust (the
/// `throttle_full` anchor), and the ejection handle (the `eject` control). A helicopter's
/// collective is a throttle lever swinging up about its pivot (the `throttle_pivot` anchor).
#[derive(Resource, Clone, Copy, Debug)]
pub struct CockpitAnchors {
    pub stick_pivot: Vec3,
    /// The stick's grip centre above the pivot.
    pub grip_height: f32,
    pub throttle_idle: Vec3,
    pub throttle_full: Vec3,
    /// The collective's pivot (none: the throttle slides along its rail).
    pub throttle_pivot: Option<Vec3>,
    /// A V/STOL type's nozzle lever: its knob forward (nozzles aft, the `nozzle` object) and
    /// pulled back to the braking stop (the `nozzle_down` anchor), swinging about its pivot
    /// (`nozzle_pivot`). None of them: no nozzle lever.
    pub nozzle_aft: Option<Vec3>,
    pub nozzle_down: Option<Vec3>,
    pub nozzle_pivot: Option<Vec3>,
    pub eject: Vec3,
}

impl Default for CockpitAnchors {
    fn default() -> Self {
        Self {
            stick_pivot: Vec3::new(0.40, -0.48, -0.22),
            grip_height: GRIP_HEIGHT,
            throttle_idle: Vec3::new(-0.40, -0.40, -0.12),
            throttle_full: Vec3::new(-0.40, -0.40, -0.30),
            throttle_pivot: None,
            nozzle_aft: None,
            nozzle_down: None,
            nozzle_pivot: None,
            eject: Vec3::new(0.0, -0.74, -0.20),
        }
    }
}

impl CockpitAnchors {
    fn throttle_handle(&self, t: f32) -> Vec3 {
        match self.throttle_pivot {
            Some(p) => p + self.throttle_rotation(t) * (self.throttle_idle - p),
            None => self.throttle_idle.lerp(self.throttle_full, t.clamp(0.0, 1.0)),
        }
    }

    /// The collective's rotation about its pivot at setting `t` (none for a sliding throttle).
    fn throttle_rotation(&self, t: f32) -> Quat {
        let Some(p) = self.throttle_pivot else { return Quat::IDENTITY };
        let (a, b) = (self.throttle_idle - p, self.throttle_full - p);
        if a.length_squared() < 1e-6 || b.length_squared() < 1e-6 {
            return Quat::IDENTITY;
        }
        Quat::IDENTITY
            .slerp(Quat::from_rotation_arc(a.normalize(), b.normalize()), t.clamp(0.0, 1.0))
    }

    /// Throttle setting change for a hand movement (along the rail).
    fn throttle_travel(&self, d: Vec3) -> f32 {
        let rail = self.throttle_full - self.throttle_idle;
        d.dot(rail) / rail.length_squared().max(1e-6)
    }

    /// The nozzle lever's pivot and its knob's offsets from it forward and back (None: no
    /// nozzle lever).
    fn nozzle_arc(&self) -> Option<(Vec3, Vec3, Vec3)> {
        let (p, a, b) = (self.nozzle_pivot?, self.nozzle_aft?, self.nozzle_down?);
        let (a, b) = (a - p, b - p);
        (a.length_squared() > 1e-6 && b.length_squared() > 1e-6).then_some((p, a, b))
    }

    /// The nozzle lever's rotation about its pivot at lever `t`.
    fn nozzle_rotation(&self, t: f32) -> Quat {
        let Some((_, a, b)) = self.nozzle_arc() else { return Quat::IDENTITY };
        Quat::IDENTITY
            .slerp(Quat::from_rotation_arc(a.normalize(), b.normalize()), t.clamp(0.0, 1.0))
    }

    /// The nozzle lever's knob at lever `t`.
    fn nozzle_handle(&self, t: f32) -> Option<Vec3> {
        let (p, a, _) = self.nozzle_arc()?;
        Some(p + self.nozzle_rotation(t) * a)
    }

    /// How far round its arc (lever units: 1 = forward to the braking stop) a point is from
    /// the knob's forward position, seen along the pivot's axis.
    fn nozzle_angle(&self, at: Vec3) -> f32 {
        let Some((p, a, b)) = self.nozzle_arc() else { return 0.0 };
        let axis = a.cross(b).normalize_or_zero();
        let full = a.angle_between(b).max(1e-3);
        let v = at - p;
        axis.dot(a.cross(v)).atan2(a.dot(v)) / full
    }
}

/// The nozzles' travel: lever 1 (the braking stop) sets them this far down from aft (degrees).
pub(super) const NOZZLE_TRAVEL_DEG: f32 = 98.5;
/// The nozzle lever's hover stop (90 degrees) and the 10-degree ticks.
const NOZZLE_HOVER: f32 = 90.0 / NOZZLE_TRAVEL_DEG;
const NOZZLE_NOTCH: f32 = 10.0 / NOZZLE_TRAVEL_DEG;

/// Reach for grabbing (metres from the grip centre).
const GRAB_REACH: f32 = 0.10;

#[derive(Component)]
pub(super) struct Stick;

#[derive(Component)]
pub(super) struct ThrottleLever;

#[derive(Component)]
pub(super) struct NozzleLever;

#[derive(Default)]
pub(super) struct Grabs {
    stick: Option<StickGrab>,
    throttle: Option<ThrottleGrab>,
    nozzle: Option<NozzleGrab>,
    eject: Option<(usize, f32)>,
    /// A held VR button set the wheel brakes (they come off when it is let go).
    brakes: bool,
    /// The throttle hand's rumble playing, and seconds into it.
    rumble: Option<(Rumble, f32)>,
    /// The stick's stall buffet: seconds left of the current knock or gap, and its amplitude.
    buffet: (f32, f32),
}

/// A haptic pattern: (start s, end s, amplitude at start, at end) pulses.
type Rumble = &'static [(f32, f32, f32, f32)];

/// The afterburner lighting: two hard knocks and a fading rumble; going out: one soft fade.
const RUMBLE_AB_ON: Rumble =
    &[(0.0, 0.06, 1.0, 1.0), (0.12, 0.18, 1.0, 1.0), (0.24, 0.7, 0.8, 0.15)];
const RUMBLE_AB_OFF: Rumble = &[(0.0, 0.25, 0.6, 0.05)];
/// Both controllers' shake while the afterburner burns: amplitude at its lowest and full
/// setting, and its uneven flutter (amplitude +-, two rates in Hz).
const AB_SHAKE: (f32, f32) = (0.2, 0.4);
const AB_FLUTTER: (f32, f32, f32) = (0.08, 7.3, 2.9);

/// The afterburner's shake `t` seconds into the flight at a setting `ab` (0 out .. 1 full).
fn ab_shake(ab: f32, t: f32) -> f32 {
    if ab <= 0.0 {
        return 0.0;
    }
    let (amp, f1, f2) = AB_FLUTTER;
    let flutter =
        amp * (std::f32::consts::TAU * f1 * t).sin() * (std::f32::consts::TAU * f2 * t).cos();
    AB_SHAKE.0 + (AB_SHAKE.1 - AB_SHAKE.0) * ab + flutter
}

/// The stall buffet's depth (`shake::stall_buffet`) at which the stick's rumble is full.
const BUFFET_DEEP: f32 = 2.0;
/// Knock length (s) and the gap between knocks (s, shortening as the stall deepens).
const BUFFET_KNOCK: (f32, f32) = (0.03, 0.09);
const BUFFET_GAP: (f32, f32) = (0.05, 0.35);

/// The stick's stall buffet: irregular knocks of random length, strength and spacing, sparse and
/// light at the edge of the buffet and a hard, near-constant hammering deep in the stall. `state`
/// is `Grabs::buffet`; `rng` draws in 0..1.
fn stall_rumble(depth: f32, dt: f32, state: &mut (f32, f32), mut rng: impl FnMut() -> f32) -> f32 {
    if depth <= 0.0 {
        *state = (0.0, 0.0);
        return 0.0;
    }
    let d = (depth / BUFFET_DEEP).min(1.0);
    state.0 -= dt;
    if state.0 <= 0.0 {
        if rng() < 0.35 + 0.5 * d {
            state.0 = BUFFET_KNOCK.0 + (BUFFET_KNOCK.1 - BUFFET_KNOCK.0) * rng();
            state.1 = (0.15 + 0.85 * d) * (0.5 + 0.5 * rng());
        } else {
            state.0 = (BUFFET_GAP.0 + (BUFFET_GAP.1 - BUFFET_GAP.0) * rng()) * (1.0 - 0.7 * d);
            state.1 = 0.0;
        }
    }
    state.1
}

/// The rumble's amplitude `t` seconds in (None: over).
fn rumble_at(r: Rumble, t: f32) -> Option<f32> {
    if r.last().is_none_or(|p| t >= p.1) {
        return None;
    }
    Some(
        r.iter()
            .find(|p| (p.0..p.1).contains(&t))
            .map_or(0.0, |p| p.2 + (p.3 - p.2) * (t - p.0) / (p.1 - p.0)),
    )
}

/// The afterburner stays on this far back of where it lights (hand jitter at the edge).
const AB_HYSTERESIS: f32 = 0.015;

/// Where a throttle lever position is: 0 dry thrust, 1 in the detent, 2 afterburner.
fn throttle_zone(t: f32) -> u8 {
    if t >= AB_DETENT + AB_GAP {
        2
    } else if t >= AB_DETENT {
        1
    } else {
        0
    }
}

struct StickGrab {
    rot0: Quat,
    /// Hand position from the pivot when grabbed.
    p0: Vec3,
    /// Twist outside the rudder deadzone.
    twisted: bool,
}

struct ThrottleGrab {
    /// Hand position when grabbed.
    p0: Vec3,
    t0: f32,
    /// Last 10 % detent passed (for the haptic ticks).
    notch: i32,
    /// The lever's afterburner zone (`throttle_zone`; with an afterburner).
    zone: u8,
}

struct NozzleGrab {
    /// The hand's angle round the lever's arc when grabbed (`CockpitAnchors::nozzle_angle`).
    a0: f32,
    t0: f32,
    /// Last 10-degree tick passed.
    notch: i32,
}

/// Throttle detents per full travel (one haptic tick each).
const THROTTLE_NOTCHES: f32 = 10.0;

/// Twist of a rotation about an axis (swing-twist decomposition), radians.
fn twist_about(q: Quat, axis: Vec3) -> f32 {
    let v = Vec3::new(q.x, q.y, q.z);
    let p = axis * v.dot(axis);
    let t = Quat::from_xyzw(p.x, p.y, p.z, q.w);
    if t.length_squared() < 1e-9 {
        return 0.0;
    }
    let t = t.normalize();
    let angle = 2.0 * t.w.clamp(-1.0, 1.0).acos();
    let sign = if Vec3::new(t.x, t.y, t.z).dot(axis) >= 0.0 { 1.0 } else { -1.0 };
    let a = sign * angle;
    // Wrap to -pi..pi.
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

/// Stick angles of a hand rotation since the grab: (pitch: back = +, roll: right = +).
/// The stick axis is tilted as the controller was; twist about the axis is ignored here.
fn stick_angles(delta: Quat) -> (f32, f32) {
    let v = delta * Vec3::Y;
    (v.z.atan2(v.y), v.x.atan2(v.y))
}

/// Stick angles of the hand moving about the pivot from `p0` to `p` (as `stick_angles`).
fn stick_angles_moved(p0: Vec3, p: Vec3) -> (f32, f32) {
    (p.z.atan2(p.y) - p0.z.atan2(p0.y), p.x.atan2(p.y) - p0.x.atan2(p0.y))
}

/// How much the hand's position (rather than the controller's tilt) moves a stick: fully a
/// centre stick, a side stick by its length.
fn stick_follow(pivot: Vec3, grip_height: f32) -> f32 {
    if pivot.x.abs() < CENTRE_STICK {
        return 1.0;
    }
    ((grip_height - LONG_STICK.0) / (LONG_STICK.1 - LONG_STICK.0)).clamp(0.0, 1.0)
}

/// `Hand::buttons` bit of a controller input (the trigger is analog).
fn vr_bit(input: VrInput) -> u8 {
    match input {
        VrInput::Trigger => 0,
        VrInput::Primary => BUTTON_PRIMARY,
        VrInput::Secondary => BUTTON_SECONDARY,
        VrInput::ThumbClick => BUTTON_THUMB,
        VrInput::ThumbUp => BUTTON_STICK_UP,
        VrInput::ThumbDown => BUTTON_STICK_DOWN,
        VrInput::ThumbLeft => BUTTON_STICK_LEFT,
        VrInput::ThumbRight => BUTTON_STICK_RIGHT,
    }
}

/// VR grabbing: writes `FlightControls` (after the keyboard/gamepad) and the mapped controller
/// buttons.
#[allow(clippy::too_many_arguments)]
pub(super) fn grab_levers(
    mut hands: ResMut<Hands>,
    config: Res<InputConfig>,
    mut grabs: Local<Grabs>,
    mut controls: ResMut<FlightControls>,
    mut actions: MessageWriter<ActionEvent>,
    aircraft: Query<&Aircraft, With<Player>>,
    (block, console): (Res<crate::input::InputBlock>, Res<crate::input::ConsoleStick>),
    console_throttle: Res<crate::input::ConsoleThrottle>,
    anchors: Res<CockpitAnchors>,
    hide: Option<Res<crate::camera::HideOwnAircraft>>,
    k: Res<FlightConstants>,
    time: Res<Time>,
) {
    // The cockpit is not there (VR, the campaign's Ops screen): nothing is held, and of the
    // mapped buttons only the in-flight menu's works.
    if hide.is_some_and(|h| h.0) {
        if grabs.nozzle.is_some() {
            controls.nozzle = None;
        }
        *grabs = Grabs::default();
        for hand in hands.hands.iter_mut() {
            hand.lever = false;
        }
        let [left, right] = &hands.hands;
        for (side, hand) in [(VrHand::Left, left), (VrHand::Right, right)] {
            for m in config.vr_buttons.iter().filter(|m| m.hand == side) {
                if hand.active
                    && m.target == ButtonTarget::Action(Action::Menu)
                    && m.input != VrInput::Trigger
                    && hand.just_pressed(vr_bit(m.input))
                {
                    actions.write(ActionEvent(Action::Menu));
                }
            }
        }
        return;
    }
    let ab = k.afterburner();
    let set_thrust =
        console_throttle.0.unwrap_or_else(|| aircraft.single().map_or(0.0, |a| a.lever(ab)));
    let brakes_on = aircraft.single().is_ok_and(|a| a.brakes);
    let set_nozzle = aircraft.single().map_or(0.0, |a| a.nozzle_lever);
    let [left, right] = &mut hands.hands;

    // --- stick (right hand)
    let grip_centre = anchors.stick_pivot + Vec3::Y * anchors.grip_height;
    if !right.active || right.squeeze < 0.4 {
        if grabs.stick.take().is_some() {
            right.haptic = right.haptic.max(0.2);
        }
    } else if grabs.stick.is_none()
        && !right.carrying
        && right.squeeze > 0.6
        && right.grip.translation.distance(grip_centre) < GRAB_REACH
    {
        grabs.stick = Some(StickGrab {
            rot0: right.grip.rotation,
            p0: right.grip.translation - anchors.stick_pivot,
            twisted: false,
        });
        right.haptic = right.haptic.max(0.4);
    }
    if let Some(g) = &mut grabs.stick {
        let delta = right.grip.rotation * g.rot0.inverse();
        let (tp, tr) = stick_angles(delta);
        let (mp, mr) = stick_angles_moved(g.p0, right.grip.translation - anchors.stick_pivot);
        let w = stick_follow(anchors.stick_pivot, anchors.grip_height);
        let (p, r) = (tp + (mp - tp) * w, tr + (mr - tr) * w);
        controls.pitch = (p / STICK_TILT).clamp(-1.0, 1.0);
        controls.roll = (r / STICK_TILT).clamp(-1.0, 1.0);
        // Twisting clockwise (seen from above) = nose right.
        let twist = twist_about(delta, Vec3::Y);
        let yaw = -twist / STICK_TWIST;
        let edge = if g.twisted { TWIST_DEADZONE - TWIST_HYSTERESIS } else { TWIST_DEADZONE };
        if (yaw.abs() > edge) != g.twisted {
            g.twisted = !g.twisted;
            right.haptic = right.haptic.max(0.15);
        }
        // Rescaled past the deadzone so the rudder starts from zero at its edge.
        controls.yaw = if g.twisted {
            (yaw.signum() * (yaw.abs() - TWIST_DEADZONE).max(0.0) / (1.0 - TWIST_DEADZONE))
                .clamp(-1.0, 1.0)
        } else {
            0.0
        };
    }

    // --- throttle and nozzle lever (left hand): the nearer handle
    let handle = anchors.throttle_handle(set_thrust);
    let to_throttle = left.grip.translation.distance(handle);
    let to_nozzle = anchors
        .nozzle_handle(set_nozzle)
        .map_or(f32::INFINITY, |h| left.grip.translation.distance(h));
    if !left.active || left.squeeze < 0.4 {
        if grabs.throttle.take().is_some() {
            left.haptic = left.haptic.max(0.2);
        }
        if grabs.nozzle.take().is_some() {
            controls.nozzle = None;
            left.haptic = left.haptic.max(0.2);
        }
    } else if grabs.throttle.is_none()
        && grabs.nozzle.is_none()
        && !left.carrying
        && left.squeeze > 0.6
        && to_throttle.min(to_nozzle) < GRAB_REACH
    {
        if to_nozzle < to_throttle {
            let a0 = anchors.nozzle_angle(left.grip.translation);
            let notch = (set_nozzle / NOZZLE_NOTCH).floor() as i32;
            grabs.nozzle = Some(NozzleGrab { a0, t0: set_nozzle, notch });
        } else {
            let notch = (set_thrust * THROTTLE_NOTCHES).floor() as i32;
            let zone = throttle_zone(set_thrust);
            grabs.throttle =
                Some(ThrottleGrab { p0: left.grip.translation, t0: set_thrust, notch, zone });
        }
        left.haptic = left.haptic.max(0.4);
    }
    if let Some(g) = &mut grabs.nozzle {
        let moved = anchors.nozzle_angle(left.grip.translation) - g.a0;
        let t = (g.t0 + moved).clamp(0.0, 1.0);
        let before = controls.nozzle.unwrap_or(set_nozzle);
        // A firm tick on the end stops and the hover stop, a light one every 10 degrees (with
        // hysteresis, so hand jitter on one doesn't buzz).
        let crossed = |at: f32| (before < at) != (t < at);
        if (t >= 1.0 && before < 1.0) || (t <= 0.0 && before > 0.0) || crossed(NOZZLE_HOVER) {
            left.haptic = left.haptic.max(0.5);
        }
        let pos = t / NOZZLE_NOTCH;
        if pos >= (g.notch + 1) as f32 + 0.1 || pos < g.notch as f32 - 0.1 {
            g.notch = pos.floor() as i32;
            left.haptic = left.haptic.max(0.15);
        }
        controls.nozzle = Some(t);
    }
    let mut rumble = None;
    if let Some(g) = &mut grabs.throttle {
        let mut t = (g.t0 + anchors.throttle_travel(left.grip.translation - g.p0)).clamp(0.0, 1.0);
        if ab {
            if g.zone == 2 && t >= AB_DETENT + AB_GAP - AB_HYSTERESIS {
                t = t.max(AB_DETENT + AB_GAP);
            }
            let zone = throttle_zone(t);
            match (g.zone, zone) {
                (a, b) if a == b => {}
                (_, 2) => rumble = Some(RUMBLE_AB_ON),
                (2, _) => rumble = Some(RUMBLE_AB_OFF),
                // Into the detent from either side, or through it.
                _ => left.haptic = left.haptic.max(0.6),
            }
            g.zone = zone;
        }
        // A light tick at each detent, a firm one at the idle and full stops.
        let before = set_thrust;
        if (t >= 1.0 && before < 1.0) || (t <= 0.0 && before > 0.0) {
            left.haptic = left.haptic.max(0.5);
        }
        // Hysteresis so hand jitter on a detent doesn't buzz. A collective has none.
        let pos = t * THROTTLE_NOTCHES;
        if anchors.throttle_pivot.is_none()
            && (pos >= (g.notch + 1) as f32 + 0.1 || pos < g.notch as f32 - 0.1)
        {
            g.notch = pos.floor() as i32;
            left.haptic = left.haptic.max(0.15);
        }
        controls.throttle = Some(t);
    }
    if let Some(r) = rumble {
        grabs.rumble = Some((r, 0.0));
    }
    if let Some((r, t)) = &mut grabs.rumble {
        match rumble_at(r, *t) {
            Some(amp) => left.haptic = left.haptic.max(amp),
            None => grabs.rumble = None,
        }
        if let Some((_, t)) = &mut grabs.rumble {
            *t += time.delta_secs();
        }
    }
    // The stall buffet shakes the hand on the stick.
    let depth = aircraft
        .single()
        .ok()
        .filter(|a| a.ended.is_none() && grabs.stick.is_some())
        .map_or(0.0, crate::shake::stall_buffet);
    let buffet = stall_rumble(depth, time.delta_secs(), &mut grabs.buffet, rand::random::<f32>);
    right.haptic = right.haptic.max(buffet);
    let shake = aircraft
        .single()
        .ok()
        .filter(|a| a.ended.is_none())
        .map_or(0.0, |a| ab_shake(a.afterburner, time.elapsed_secs()));
    if shake > 0.0 {
        for h in [&mut *left, &mut *right] {
            h.haptic = h.haptic.max(shake);
        }
    }

    // --- sensor/cursor slew: the left thumbstick
    if left.active && !block.0 && !left.scrolling && left.thumb.length() > SLEW_DEADZONE {
        controls.slew = left.thumb.clamp(Vec2::splat(-1.0), Vec2::ONE);
    }

    // --- mapped buttons; a trigger only while its hand holds the throttle / stick (otherwise it
    // presses the cockpit control under the fingertip), but for the brakes
    let mut brakes_held = false;
    for (side, hand, holding) in [
        (VrHand::Left, &*left, grabs.throttle.is_some() || grabs.nozzle.is_some()),
        (VrHand::Right, &*right, grabs.stick.is_some()),
    ] {
        if !hand.active {
            continue;
        }
        for m in config.vr_buttons.iter().filter(|m| m.hand == side) {
            // The brakes also on the trigger of a free hand that touches no control.
            let brakes = m.target == ButtonTarget::Action(Action::Brakes)
                && (!hand.touching || grabs.brakes);
            let (held, pressed) = match m.input {
                VrInput::Trigger if !holding && !brakes => continue,
                VrInput::ThumbUp
                | VrInput::ThumbDown
                | VrInput::ThumbLeft
                | VrInput::ThumbRight
                    if hand.scrolling =>
                {
                    continue;
                }
                VrInput::Trigger => (hand.trigger > 0.7, hand.trigger_pulled()),
                input => (hand.held(vr_bit(input)), hand.just_pressed(vr_bit(input))),
            };
            match m.target {
                // At an air defence console the trigger (fire) fires its gun, not the gun button.
                ButtonTarget::Gun => controls.gun |= held && !block.0,
                ButtonTarget::Action(Action::FireWeapon) if held && (!block.0 || console.0) => {
                    controls.fire = true;
                    if pressed {
                        actions.write(ActionEvent(Action::FireWeapon));
                    }
                }
                // The wheel brakes are on while the button is held (released only if it set them).
                ButtonTarget::Action(Action::Brakes) if !block.0 => {
                    brakes_held |= held;
                    if pressed && !brakes_on {
                        actions.write(ActionEvent(Action::Brakes));
                        grabs.brakes = true;
                    }
                }
                ButtonTarget::Action(Action::LockCursor) if held && !block.0 => {
                    controls.mark = true;
                    if pressed {
                        actions.write(ActionEvent(Action::LockCursor));
                    }
                }
                ButtonTarget::Action(a) if a.trim_direction().is_some() => {
                    if held && !block.0 {
                        controls.trim += a.trim_direction().unwrap_or_default();
                    }
                }
                ButtonTarget::Action(a) if pressed && (!block.0 || a == Action::Menu) => {
                    actions.write(ActionEvent(a));
                }
                ButtonTarget::Action(_) => {}
            }
        }
    }
    if grabs.brakes && !brakes_held {
        grabs.brakes = false;
        if brakes_on {
            actions.write(ActionEvent(Action::Brakes));
        }
    }

    // --- ejection handle: grab with either hand and pull up 6 cm
    let handle = anchors.eject;
    for (i, h) in [&mut *left, &mut *right].into_iter().enumerate() {
        let is_grabbing_other = (i == 1 && grabs.stick.is_some())
            || (i == 0 && (grabs.throttle.is_some() || grabs.nozzle.is_some()));
        match grabs.eject {
            Some((who, y0)) if who == i => {
                if !h.active || h.squeeze < 0.4 {
                    grabs.eject = None;
                } else if h.grip.translation.y - y0 > 0.06 {
                    actions.write(ActionEvent(Action::Eject));
                    h.haptic = 1.0;
                    grabs.eject = None;
                }
            }
            None if h.active
                && !is_grabbing_other
                && !h.carrying
                && h.squeeze > 0.6
                && h.grip.translation.distance(handle) < 0.07 =>
            {
                grabs.eject = Some((i, h.grip.translation.y));
                h.haptic = h.haptic.max(0.3);
            }
            _ => {}
        }
    }
    left.lever = grabs.throttle.is_some()
        || grabs.nozzle.is_some()
        || grabs.eject.is_some_and(|(who, _)| who == 0);
    right.lever = grabs.stick.is_some() || grabs.eject.is_some_and(|(who, _)| who == 1);
}

/// Stick, throttle and nozzle lever follow the controls (whatever the input source).
#[allow(clippy::type_complexity)]
pub(super) fn animate_levers(
    controls: Res<FlightControls>,
    console_throttle: Res<crate::input::ConsoleThrottle>,
    aircraft: Query<&Aircraft, With<Player>>,
    mut stick: Query<&mut Transform, (With<Stick>, Without<ThrottleLever>, Without<NozzleLever>)>,
    mut throttle: Query<&mut Transform, (With<ThrottleLever>, Without<NozzleLever>)>,
    mut nozzle: Query<&mut Transform, With<NozzleLever>>,
    anchors: Res<CockpitAnchors>,
    k: Res<FlightConstants>,
) {
    if let Ok(mut tf) = stick.single_mut() {
        tf.rotation = Quat::from_rotation_x(controls.pitch * STICK_TILT)
            * Quat::from_rotation_z(-controls.roll * STICK_TILT)
            * Quat::from_rotation_y(-controls.yaw * STICK_TWIST);
    }
    if let (Ok(mut tf), Ok(a)) = (throttle.single_mut(), aircraft.single()) {
        let t = controls.throttle.or(console_throttle.0).unwrap_or(a.lever(k.afterburner()));
        tf.translation = anchors.throttle_handle(t);
        tf.rotation = anchors.throttle_rotation(t);
    }
    if let (Ok(mut tf), Ok(a)) = (nozzle.single_mut(), aircraft.single()) {
        let t = controls.nozzle.unwrap_or(a.nozzle_lever);
        if let Some(at) = anchors.nozzle_handle(t) {
            tf.translation = at;
            tf.rotation = anchors.nozzle_rotation(t);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The stall buffet: nothing outside it, knocks with gaps at its edge, and on average harder
    /// and more often the deeper the stall.
    #[test]
    fn stall_buffet_rumble() {
        let mut seed = 12345u32;
        let mut rng = move || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as f32 / u32::MAX as f32
        };
        let mut state = (0.0, 0.0);
        assert_eq!(stall_rumble(0.0, 0.011, &mut state, &mut rng), 0.0);
        let mut mean = |depth: f32| {
            let mut state = (0.0, 0.0);
            let amps: Vec<f32> =
                (0..2000).map(|_| stall_rumble(depth, 0.011, &mut state, &mut rng)).collect();
            assert!(amps.contains(&0.0) || depth >= BUFFET_DEEP);
            assert!(amps.iter().all(|&a| a <= 1.0));
            amps.iter().sum::<f32>() / amps.len() as f32
        };
        let (edge, stall, deep) = (mean(0.3), mean(1.0), mean(2.0));
        assert!(edge > 0.0 && edge < stall && stall < deep, "{edge} {stall} {deep}");
    }

    /// The afterburner's rumbles: knocks with a gap between them, then a fade; over at their end.
    #[test]
    fn afterburner_rumbles() {
        assert_eq!(rumble_at(RUMBLE_AB_ON, 0.03), Some(1.0));
        assert_eq!(rumble_at(RUMBLE_AB_ON, 0.09), Some(0.0));
        let fading = rumble_at(RUMBLE_AB_ON, 0.5).unwrap();
        assert!(0.15 < fading && fading < 0.8, "{fading}");
        assert_eq!(rumble_at(RUMBLE_AB_ON, 0.7), None);
        assert!(rumble_at(RUMBLE_AB_OFF, 0.1).unwrap() < 0.6);
        assert_eq!(rumble_at(RUMBLE_AB_OFF, 0.3), None);
        assert_eq!(ab_shake(0.0, 1.0), 0.0);
        let (low, full) = (ab_shake(0.01, 0.0), ab_shake(1.0, 0.0));
        assert!(0.15 < low && low < full && full <= 0.5, "{low} {full}");
        assert_eq!(
            [0.5, AB_DETENT + 0.01, AB_DETENT + AB_GAP, 1.0].map(throttle_zone),
            [0, 1, 2, 2]
        );
    }

    /// The nozzle lever swings about its pivot: forward at 0, the braking stop at 1, the hover
    /// stop between; the hand's angle round the arc reads back as the lever position.
    #[test]
    fn nozzle_lever_arc() {
        let p = Vec3::new(-0.30, -0.66, 0.10);
        let (fwd, back) = (40f32.to_radians(), -35f32.to_radians());
        let knob = |a: f32| p + 0.13 * Vec3::new(0.0, a.cos(), -a.sin());
        let anchors = CockpitAnchors {
            nozzle_pivot: Some(p),
            nozzle_aft: Some(knob(fwd)),
            nozzle_down: Some(knob(back)),
            ..default()
        };
        for t in [0.0, NOZZLE_HOVER, 1.0] {
            let at = anchors.nozzle_handle(t).unwrap();
            assert!(at.distance(knob(fwd + (back - fwd) * t)) < 1e-4, "{t}: {at}");
            assert!((anchors.nozzle_angle(at) - t).abs() < 1e-3, "{t}");
        }
        // Further out along the lever reads the same.
        let far = p + (anchors.nozzle_handle(0.5).unwrap() - p) * 1.5;
        assert!((anchors.nozzle_angle(far) - 0.5).abs() < 1e-3);
        assert!(CockpitAnchors::default().nozzle_handle(0.5).is_none());
    }

    /// A centre stick's grip stays in the hand, however short (the Tornado's, 0.39 m): moving
    /// the hand 10 cm back tilts the stick so its grip moves 10 cm back, and moving it sideways
    /// rolls. A short side stick still goes by the controller's tilt, a long one by the hand.
    #[test]
    fn centre_stick_follows_the_hand() {
        let side = Vec3::new(0.28, -0.66, -0.22);
        assert_eq!(stick_follow(side, 0.11), 0.0);
        assert_eq!(stick_follow(side, 0.59), 1.0);
        let centre = Vec3::new(0.0, -0.99, -0.32);
        for height in [0.39, 0.59] {
            assert_eq!(stick_follow(centre, height), 1.0, "{height}");
        }
        let height = 0.39;
        let p0 = Vec3::Y * height;
        let (pitch, roll) = stick_angles_moved(p0, p0 + Vec3::new(0.0, 0.0, 0.10));
        let grip = Quat::from_rotation_x(pitch) * p0;
        assert!((grip.z - 0.10).abs() < 0.005, "grip at {grip}");
        assert!(roll.abs() < 1e-6);
        let (_, roll) = stick_angles_moved(p0, p0 + Vec3::X * 0.05);
        assert!(roll > 0.0, "hand right = roll right");
    }
}

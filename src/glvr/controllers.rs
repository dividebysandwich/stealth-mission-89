//! The controllers in the GL headset session: the actions and bindings of `crate::xr` (the
//! cockpit action set), read on the render thread each frame at the frame's display time and
//! handed to the main thread with the eyes, where `read_hands` makes them the game's `Hands`.

use crate::camera::{RigOffset, View, ViewState};
use crate::cockpit3d::{
    BUTTON_MENU, BUTTON_PRIMARY, BUTTON_SECONDARY, BUTTON_STICK_DOWN, BUTTON_STICK_LEFT,
    BUTTON_STICK_RIGHT, BUTTON_STICK_UP, BUTTON_THUMB, Hands,
};
use bevy::prelude::*;
use openxr as xr;

/// One controller as the runtime reports it: poses in the local space, while tracked.
#[derive(Clone, Copy, Default)]
pub struct Controller {
    pub squeeze: f32,
    pub trigger: f32,
    /// BUTTON_PRIMARY | BUTTON_SECONDARY | BUTTON_THUMB | BUTTON_MENU.
    pub buttons: u8,
    pub thumb: Vec2,
    pub grip: Option<Transform>,
    pub aim: Option<Transform>,
}

pub struct Controllers {
    set: xr::ActionSet,
    // (The pose actions stay alive while their spaces are used.)
    _grip: xr::Action<xr::Posef>,
    _aim: xr::Action<xr::Posef>,
    squeeze: xr::Action<f32>,
    trigger: xr::Action<f32>,
    primary: xr::Action<bool>,
    secondary: xr::Action<bool>,
    thumb: xr::Action<bool>,
    thumbstick: xr::Action<xr::Vector2f>,
    menu: xr::Action<bool>,
    haptic: xr::Action<xr::Haptic>,
    /// /user/hand/left, /user/hand/right
    hands: [xr::Path; 2],
    /// Grip and aim spaces of each hand.
    spaces: [[xr::Space; 2]; 2],
}

fn err(what: &'static str) -> impl Fn(xr::sys::Result) -> String {
    move |e| format!("{what}: {e}")
}

impl Controllers {
    /// The actions, their suggested bindings (as `crate::xr::suggest_bindings`), attached to
    /// `session`.
    pub fn new<G: xr::Graphics>(
        instance: &xr::Instance,
        session: &xr::Session<G>,
    ) -> Result<Self, String> {
        let path = |s: &str| instance.string_to_path(s).map_err(err("an OpenXR path"));
        let hands = [path("/user/hand/left")?, path("/user/hand/right")?];
        let set =
            instance.create_action_set("cockpit", "Cockpit", 0).map_err(err("the action set"))?;
        let e = err("an action");
        let grip = set.create_action("grip_pose", "Grip pose", &hands).map_err(&e)?;
        let aim = set.create_action("aim_pose", "Aim pose", &hands).map_err(&e)?;
        let squeeze = set.create_action("squeeze", "Grab", &hands).map_err(&e)?;
        let trigger = set.create_action("trigger", "Trigger", &hands).map_err(&e)?;
        let primary = set.create_action("primary", "Primary button", &hands).map_err(&e)?;
        let secondary = set.create_action("secondary", "Secondary button", &hands).map_err(&e)?;
        let thumb = set.create_action("thumb_click", "Thumbstick click", &hands).map_err(&e)?;
        let thumbstick = set.create_action("thumbstick", "Thumbstick", &hands).map_err(&e)?;
        let menu = set.create_action("menu", "Menu / recentre", &hands).map_err(&e)?;
        let haptic = set.create_action("haptic", "Haptics", &hands).map_err(&e)?;

        let both = |s: &str| vec![format!("/user/hand/left/{s}"), format!("/user/hand/right/{s}")];
        let per_hand = |l: &str, r: &str| {
            vec![format!("/user/hand/left/{l}"), format!("/user/hand/right/{r}")]
        };
        let left = |s: &str| vec![format!("/user/hand/left/{s}")];
        // (profile, squeeze, trigger, primary, secondary, thumb, menu)
        type B = Vec<String>;
        let profiles: Vec<(&str, B, B, B, B, B, B)> = vec![
            (
                "/interaction_profiles/oculus/touch_controller",
                both("input/squeeze/value"),
                both("input/trigger/value"),
                per_hand("input/x/click", "input/a/click"),
                per_hand("input/y/click", "input/b/click"),
                both("input/thumbstick/click"),
                left("input/menu/click"),
            ),
            (
                "/interaction_profiles/valve/index_controller",
                both("input/squeeze/value"),
                both("input/trigger/value"),
                both("input/a/click"),
                both("input/b/click"),
                both("input/thumbstick/click"),
                vec![],
            ),
            (
                "/interaction_profiles/htc/vive_controller",
                both("input/squeeze/click"),
                both("input/trigger/value"),
                both("input/trackpad/click"),
                vec![],
                vec![],
                both("input/menu/click"),
            ),
            (
                "/interaction_profiles/microsoft/motion_controller",
                both("input/squeeze/click"),
                both("input/trigger/value"),
                both("input/trackpad/click"),
                vec![],
                both("input/thumbstick/click"),
                both("input/menu/click"),
            ),
            (
                "/interaction_profiles/khr/simple_controller",
                vec![],
                both("input/select/click"),
                vec![],
                vec![],
                vec![],
                both("input/menu/click"),
            ),
        ];
        for (profile, sq, tr, pr, se, th, me) in profiles {
            let mut paths: Vec<(xr::Path, usize)> = Vec::new();
            let mut add = |which: usize, bindings: &B| -> Result<(), String> {
                for b in bindings {
                    paths.push((path(b)?, which));
                }
                Ok(())
            };
            add(0, &both("input/grip/pose"))?;
            add(1, &both("input/aim/pose"))?;
            add(2, &both("output/haptic"))?;
            add(3, &sq)?;
            add(4, &tr)?;
            add(5, &pr)?;
            add(6, &se)?;
            add(7, &th)?;
            if !th.is_empty() {
                add(8, &both("input/thumbstick"))?;
            }
            add(9, &me)?;
            let bindings: Vec<xr::Binding> = paths
                .iter()
                .map(|&(p, which)| match which {
                    0 => xr::Binding::new(&grip, p),
                    1 => xr::Binding::new(&aim, p),
                    2 => xr::Binding::new(&haptic, p),
                    3 => xr::Binding::new(&squeeze, p),
                    4 => xr::Binding::new(&trigger, p),
                    5 => xr::Binding::new(&primary, p),
                    6 => xr::Binding::new(&secondary, p),
                    7 => xr::Binding::new(&thumb, p),
                    8 => xr::Binding::new(&thumbstick, p),
                    _ => xr::Binding::new(&menu, p),
                })
                .collect();
            // (A runtime without the profile refuses it: the others still bind.)
            if let Err(e) = instance.suggest_interaction_profile_bindings(path(profile)?, &bindings)
            {
                info!("glvr: no bindings for {profile}: {e}");
            }
        }
        session.attach_action_sets(&[&set]).map_err(err("attaching the actions"))?;
        let space = |a: &xr::Action<xr::Posef>, hand: xr::Path| {
            a.create_space(session, hand, xr::Posef::IDENTITY).map_err(err("a hand's space"))
        };
        let spaces = [
            [space(&grip, hands[0])?, space(&aim, hands[0])?],
            [space(&grip, hands[1])?, space(&aim, hands[1])?],
        ];
        Ok(Controllers {
            set,
            _grip: grip,
            _aim: aim,
            squeeze,
            trigger,
            primary,
            secondary,
            thumb,
            thumbstick,
            menu,
            haptic,
            hands,
            spaces,
        })
    }

    /// Both controllers at `time`, their poses in `base`.
    pub fn read<G: xr::Graphics>(
        &self,
        session: &xr::Session<G>,
        base: &xr::Space,
        time: xr::Time,
    ) -> [Controller; 2] {
        if let Err(e) = session.sync_actions(&[xr::ActiveActionSet::new(&self.set)]) {
            debug!("glvr: syncing the actions: {e}");
            return [Controller::default(); 2];
        }
        let mut out = [Controller::default(); 2];
        for (i, c) in out.iter_mut().enumerate() {
            let path = self.hands[i];
            let f = |a: &xr::Action<f32>| a.state(session, path).map_or(0.0, |s| s.current_state);
            let b = |a: &xr::Action<bool>| a.state(session, path).is_ok_and(|s| s.current_state);
            c.squeeze = f(&self.squeeze);
            c.trigger = f(&self.trigger);
            c.buttons = (b(&self.primary) as u8 * BUTTON_PRIMARY)
                | (b(&self.secondary) as u8 * BUTTON_SECONDARY)
                | (b(&self.thumb) as u8 * BUTTON_THUMB)
                | (b(&self.menu) as u8 * BUTTON_MENU);
            c.thumb = self
                .thumbstick
                .state(session, path)
                .map_or(Vec2::ZERO, |s| Vec2::new(s.current_state.x, s.current_state.y));
            let locate = |space: &xr::Space| {
                let l = space.locate(base, time).ok()?;
                let flags = xr::SpaceLocationFlags::POSITION_VALID
                    | xr::SpaceLocationFlags::ORIENTATION_VALID;
                l.location_flags.contains(flags).then(|| {
                    let (p, o) = (l.pose.position, l.pose.orientation);
                    Transform::from_xyz(p.x, p.y, p.z)
                        .with_rotation(Quat::from_xyzw(o.x, o.y, o.z, o.w))
                })
            };
            c.grip = locate(&self.spaces[i][0]);
            c.aim = locate(&self.spaces[i][1]);
        }
        out
    }

    /// A short pulse of `amount` (0..1) on hand `i`.
    pub fn pulse<G: xr::Graphics>(&self, session: &xr::Session<G>, i: usize, amount: f32) {
        let v = xr::HapticVibration::new()
            .amplitude(amount.min(1.0))
            .duration(xr::Duration::from_nanos(25_000_000))
            .frequency(xr::FREQUENCY_UNSPECIFIED);
        if let Err(e) = self.haptic.apply_feedback(session, self.hands[i], &v) {
            debug!("glvr: haptics: {e}");
        }
    }
}

/// The controllers and eyes the render thread saw last (local space).
#[derive(Resource, Default)]
pub struct XrInput {
    pub controllers: [Controller; 2],
    /// The eyes' poses, for recentring.
    pub eyes: Vec<Transform>,
}

/// The controllers into `Hands` (cockpit frame = the scaled rig offset * the local space), as
/// `crate::xr::read_hands`.
pub fn read_hands(
    input: Res<XrInput>,
    offset: Res<RigOffset>,
    settings: Res<crate::settings::Settings>,
    view: Res<ViewState>,
    manning: Option<Res<crate::station::Manning>>,
    mut hands: ResMut<Hands>,
) {
    let cockpit_view = view.view == View::Cockpit;
    let offset = offset.scaled(settings.vr_hand_scale(manning.is_some()));
    for (hand, c) in hands.hands.iter_mut().zip(&input.controllers) {
        hand.prev_buttons = hand.buttons;
        hand.prev_trigger = hand.trigger;
        hand.squeeze = c.squeeze;
        hand.trigger = c.trigger;
        hand.buttons = c.buttons;
        hand.thumb = c.thumb;
        // Stick directions engage at 70 % and release below 50 % (no chatter at the edge).
        let v = c.thumb;
        for (bit, amount) in [
            (BUTTON_STICK_UP, v.y),
            (BUTTON_STICK_DOWN, -v.y),
            (BUTTON_STICK_RIGHT, v.x),
            (BUTTON_STICK_LEFT, -v.x),
        ] {
            let edge = if hand.prev_buttons & bit != 0 { 0.5 } else { 0.7 };
            if amount > edge {
                hand.buttons |= bit;
            }
        }
        hand.aim_raw = c.aim;
        let grip = c.grip.map(|g| offset * g);
        let aim = c.aim.map(|a| offset * a);
        hand.active = cockpit_view && grip.is_some();
        if let Some(g) = grip {
            hand.grip = g;
            hand.finger = aim.map_or(g.transform_point(Vec3::new(0.0, -0.02, -0.09)), |a| {
                a.transform_point(Vec3::new(0.0, 0.0, -0.02))
            });
        }
    }
}

/// Frames until the pending recentre (None: none pending).
#[derive(Resource, Default)]
pub struct Recenter(pub Option<u8>);

/// Puts the head at the design eye point, facing forward (`crate::xr::recenter`): at the start
/// of a flight and with the left hand's menu button.
pub fn recenter(
    mut pending: ResMut<Recenter>,
    input: Res<XrInput>,
    hands: Res<Hands>,
    mut offset: ResMut<RigOffset>,
) {
    if hands.hands[0].just_pressed(BUTTON_MENU) {
        pending.0 = Some(0);
    }
    match &mut pending.0 {
        None => return,
        Some(n @ 1..) => {
            *n -= 1;
            return;
        }
        Some(0) => {}
    }
    let n = input.eyes.len();
    if n == 0 {
        return;
    }
    let pos = input.eyes.iter().map(|t| t.translation).sum::<Vec3>() / n as f32;
    if pos == Vec3::ZERO {
        return; // not tracked yet
    }
    let fwd = input.eyes[0].forward().as_vec3();
    let yaw = (-fwd.x).atan2(-fwd.z);
    let rot = Quat::from_rotation_y(-yaw);
    offset.0 = Transform::from_translation(-(rot * pos)).with_rotation(rot);
    info!("glvr: recentred on the head at {pos:.2}");
    pending.0 = None;
}

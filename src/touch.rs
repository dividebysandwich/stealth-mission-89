//! Touch flight controls (the Android app; `--touch` on the desktop, the left mouse button as a
//! finger): a throttle along the left edge, a stick wherever the right thumb comes down in the
//! lower right, and buttons, drawn over the view. A finger keeps the control it came down on
//! until it lifts, so several work at once.
//!
//! A finger on none of them is the mouse pointer (the Android app has no other): it presses
//! the cockpit's controls and the MFD buttons, turns the kneeboard's pages and chooses in the
//! in-flight menu, which reads the mouse. While that menu is open the controls are hidden and
//! every finger points.

use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use stealth_i18n::t;

use crate::hotas::ButtonTarget;
use crate::input::{Action, ActionEvent, FlightControls, InputBlock, InputSystemsSet};
use crate::settings::{AppState, Settings};

/// Whether the touch controls are on: the Android app, or `--touch` (not in VR).
pub fn enabled(settings: &Settings) -> bool {
    !settings.vr && (cfg!(target_os = "android") || std::env::args().any(|a| a == "--touch"))
}

pub struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchState>()
            .add_systems(
                PreUpdate,
                (
                    assign.before(InputSystemsSet).after(bevy::input::InputSystems),
                    apply.after(InputSystemsSet),
                )
                    .run_if(in_state(AppState::Flight)),
            )
            .add_systems(OnEnter(AppState::Flight), spawn)
            .add_systems(OnExit(AppState::Flight), release)
            .add_systems(Update, place.run_if(in_state(AppState::Flight)));
        #[cfg(target_os = "android")]
        app.add_systems(
            PreUpdate,
            back_key.before(InputSystemsSet).after(bevy::input::InputSystems),
        );
    }
}

/// What a touch button does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Press {
    /// As a key bound to it: the gun and fire held, the others once per press.
    Target(ButtonTarget),
    /// The cockpit, else the chase view.
    View,
}

/// A round button: its label's message id, what it does, its centre from a corner of the screen
/// and its radius, in units of `unit` (x from the left edge if positive, else from the right;
/// y from the top if positive, else from the bottom).
struct Button {
    label: &'static str,
    press: Press,
    at: (f32, f32),
    radius: f32,
}

const fn action(a: Action) -> Press {
    Press::Target(ButtonTarget::Action(a))
}

/// The left thumb's buttons beside the throttle, and a row along the top right.
const BUTTONS: &[Button] = &[
    Button {
        label: "touch-fire",
        press: action(Action::FireWeapon),
        at: (2.5, -1.2),
        radius: 0.85,
    },
    Button {
        label: "touch-gun",
        press: Press::Target(ButtonTarget::Gun),
        at: (2.5, -3.0),
        radius: 0.6,
    },
    Button {
        label: "touch-target",
        press: action(Action::NearestTarget),
        at: (2.5, -4.5),
        radius: 0.6,
    },
    Button {
        label: "touch-weapon",
        press: action(Action::SelectWeapon),
        at: (4.1, -0.9),
        radius: 0.6,
    },
    Button { label: "touch-flare", press: action(Action::Flare), at: (4.1, -2.3), radius: 0.6 },
    Button { label: "touch-chaff", press: action(Action::Chaff), at: (4.1, -3.7), radius: 0.6 },
    Button { label: "touch-menu", press: action(Action::Menu), at: (-0.8, 0.8), radius: 0.55 },
    Button { label: "touch-view", press: Press::View, at: (-2.1, 0.8), radius: 0.55 },
    Button { label: "touch-map", press: action(Action::Maps), at: (-3.4, 0.8), radius: 0.55 },
    Button { label: "touch-gear", press: action(Action::Gear), at: (-4.7, 0.8), radius: 0.55 },
    Button { label: "touch-brakes", press: action(Action::Brakes), at: (-6.0, 0.8), radius: 0.55 },
];

/// The stick's travel (units): a finger this far from where it came down is full deflection.
const STICK_RADIUS: f32 = 1.6;
/// The stick takes fingers that come down right of this fraction of the width, below the top
/// row of buttons.
const STICK_LEFT: f32 = 0.6;
const STICK_TOP: f32 = 1.6;

/// The screen's layout for a window this big (logical pixels).
#[derive(Clone, Copy, Debug)]
struct Layout {
    size: Vec2,
    unit: f32,
}

impl Layout {
    fn new(size: Vec2) -> Self {
        Self { size, unit: (size.y / 8.0).clamp(36.0, 90.0) }
    }

    /// A button's centre and radius.
    fn button(&self, b: &Button) -> (Vec2, f32) {
        let x = if b.at.0 >= 0.0 { b.at.0 * self.unit } else { self.size.x + b.at.0 * self.unit };
        let y = if b.at.1 >= 0.0 { b.at.1 * self.unit } else { self.size.y + b.at.1 * self.unit };
        (Vec2::new(x, y), b.radius * self.unit)
    }

    /// The throttle's slot: its top and bottom centre.
    fn throttle(&self) -> (Vec2, Vec2) {
        let x = 0.9 * self.unit;
        (Vec2::new(x, 1.6 * self.unit), Vec2::new(x, self.size.y - 0.6 * self.unit))
    }

    /// The throttle (0..1) for a finger at `p`.
    fn throttle_at(&self, p: Vec2) -> f32 {
        let (top, bottom) = self.throttle();
        ((bottom.y - p.y) / (bottom.y - top.y)).clamp(0.0, 1.0)
    }

    /// The control a finger coming down at `p` takes.
    fn hit(&self, p: Vec2) -> Control {
        for (i, b) in BUTTONS.iter().enumerate() {
            let (c, r) = self.button(b);
            // A little larger than drawn: fingers are blunt.
            if p.distance(c) <= r * 1.15 {
                return Control::Button(i);
            }
        }
        let (top, bottom) = self.throttle();
        if p.x <= top.x + 0.8 * self.unit
            && p.y >= top.y - 0.4 * self.unit
            && p.y <= bottom.y + 0.6 * self.unit
        {
            return Control::Throttle;
        }
        if p.x >= self.size.x * STICK_LEFT && p.y >= STICK_TOP * self.unit {
            return Control::Stick { centre: p };
        }
        Control::Pointer
    }

    /// The stick (roll, pitch) for a finger at `p` that came down at `centre`: right rolls
    /// right, down pulls. Finer near the centre.
    fn stick(&self, centre: Vec2, p: Vec2) -> Vec2 {
        let d = ((p - centre) / (STICK_RADIUS * self.unit)).clamp(Vec2::splat(-1.0), Vec2::ONE);
        d.signum() * d.abs().powf(1.5)
    }

    /// Where the stick is drawn while no finger holds it.
    fn stick_rest(&self) -> Vec2 {
        Vec2::new(self.size.x - 2.8 * self.unit, self.size.y - 2.6 * self.unit)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Control {
    Button(usize),
    Throttle,
    Stick {
        centre: Vec2,
    },
    /// The mouse pointer.
    Pointer,
}

/// The fingers down and the controls they hold.
#[derive(Resource, Default)]
struct TouchState {
    /// (finger, control, where it is now, came down this frame).
    fingers: Vec<(u64, Control, Vec2, bool)>,
    /// The throttle last set (shown on the slot), None until a finger moves it.
    throttle: Option<f32>,
    /// The throttle `apply` last wrote: it is written only when it moves, as an axis.
    throttle_sent: Option<f32>,
    /// The pointer's finger lifted last frame: the cursor leaves this frame.
    pointer_up: bool,
    /// The desktop's left mouse button is down (from its messages: the game's `ButtonInput` is
    /// reset while the button holds a control).
    mouse_down: bool,
    /// The window's layout this frame.
    layout: Option<Layout>,
    /// The controls are hidden (the in-flight menu is open).
    hidden: bool,
}

/// The desktop's left mouse button, as a finger (`--touch`).
const MOUSE_FINGER: u64 = u64::MAX;

/// Gives each new finger its control, and moves the mouse pointer with a pointing finger: the
/// cursor where it is and the left button down while it is (Android); the desktop's real mouse
/// is hidden from the game while it holds a control.
fn assign(
    touches: Res<Touches>,
    mut buttons: MessageReader<bevy::input::mouse::MouseButtonInput>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    block: Res<InputBlock>,
    mut state: ResMut<TouchState>,
) {
    let Ok(mut window) = window.single_mut() else { return };
    let layout = Layout::new(Vec2::new(window.width(), window.height()));
    state.layout = Some(layout);
    state.hidden = block.0;
    let mut down: Vec<(u64, Vec2, bool)> =
        touches.iter().map(|t| (t.id(), t.position(), touches.just_pressed(t.id()))).collect();
    let mut clicked = false;
    for b in buttons.read().filter(|b| b.button == MouseButton::Left) {
        state.mouse_down = b.state.is_pressed();
        clicked |= state.mouse_down;
    }
    if !cfg!(target_os = "android")
        && state.mouse_down
        && let Some(p) = window.cursor_position()
    {
        down.push((MOUSE_FINGER, p, clicked));
    }
    let old = std::mem::take(&mut state.fingers);
    for (id, p, _) in down {
        let control = match old.iter().find(|f| f.0 == id) {
            Some(&(_, c, _, _)) => (c, false),
            // While the menu is open every finger points.
            None if block.0 => (Control::Pointer, true),
            None => (layout.hit(p), true),
        };
        state.fingers.push((id, control.0, p, control.1));
    }
    // The pointer: on Android a finger, on the desktop the mouse itself.
    let pointer = state.fingers.iter().find(|f| f.1 == Control::Pointer).map(|f| (f.0, f.2));
    if cfg!(target_os = "android") {
        // Set without change detection: winit can't move Android's (absent) cursor, and would
        // log that each time.
        match pointer {
            Some((_, p)) => {
                window.bypass_change_detection().set_cursor_position(Some(p));
                mouse.press(MouseButton::Left);
                state.pointer_up = false;
            }
            None if mouse.pressed(MouseButton::Left) => {
                mouse.release(MouseButton::Left);
                state.pointer_up = true;
            }
            None if state.pointer_up => {
                window.bypass_change_detection().set_cursor_position(None);
                state.pointer_up = false;
            }
            None => {}
        }
    } else if state.fingers.iter().any(|f| f.0 == MOUSE_FINGER && f.1 != Control::Pointer) {
        mouse.reset(MouseButton::Left);
    }
    if state.fingers.iter().all(|f| f.1 != Control::Throttle) {
        state.throttle_sent = None;
    }
}

/// The controls the fingers hold, over the keyboard's (as the joysticks do).
fn apply(
    mut state: ResMut<TouchState>,
    mut controls: ResMut<FlightControls>,
    mut actions: MessageWriter<ActionEvent>,
    view: Option<Res<crate::camera::ViewState>>,
) {
    let Some(layout) = state.layout else { return };
    let mut held = [false; BUTTONS.len()];
    let mut pressed = [false; BUTTONS.len()];
    let mut throttle = None;
    for &(_, control, p, new) in &state.fingers {
        match control {
            Control::Button(i) => {
                held[i] = true;
                pressed[i] |= new;
            }
            Control::Throttle => throttle = Some(layout.throttle_at(p)),
            Control::Stick { centre } => {
                let s = layout.stick(centre, p);
                controls.roll = crate::input::stronger(controls.roll, s.x);
                controls.pitch = crate::input::stronger(controls.pitch, s.y);
            }
            Control::Pointer => {}
        }
    }
    if let Some(t) = throttle {
        state.throttle = Some(t);
        if state.throttle_sent.is_none_or(|s| (s - t).abs() > 0.01) {
            state.throttle_sent = Some(t);
            controls.throttle = Some(t);
        }
    }
    if state.hidden {
        return;
    }
    for (i, b) in BUTTONS.iter().enumerate() {
        match b.press {
            Press::Target(target) => crate::hotas::hold_button(
                target,
                held[i],
                pressed[i],
                false,
                &mut controls,
                &mut actions,
            ),
            Press::View if pressed[i] => {
                let cockpit = view.as_ref().is_some_and(|v| v.view == crate::camera::View::Cockpit);
                actions.write(ActionEvent(if cockpit {
                    Action::ViewChase
                } else {
                    Action::ViewCockpit
                }));
            }
            Press::View => {}
        }
    }
}

/// Leaving the flight: no finger holds anything, the pointer's button is up.
fn release(mut state: ResMut<TouchState>, mut mouse: ResMut<ButtonInput<MouseButton>>) {
    *state = TouchState { mouse_down: state.mouse_down, ..default() };
    if cfg!(target_os = "android") {
        mouse.release(MouseButton::Left);
    }
}

/// Android's back key (or gesture) opens and closes the in-flight menu.
#[cfg(target_os = "android")]
fn back_key(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    mut actions: MessageWriter<ActionEvent>,
) {
    // winit names it by its meaning only (its physical key is unidentified).
    if keys.just_pressed(bevy::input::keyboard::Key::BrowserBack) {
        actions.write(ActionEvent(Action::Menu));
    }
}

const INK: Color = Color::srgba(0.47, 1.0, 0.67, 0.6);
const HELD: Color = Color::srgba(0.24, 0.55, 0.35, 0.45);
const FILL: Color = Color::srgba(0.0, 0.0, 0.0, 0.25);
/// The stick's ring while no finger holds it.
const IDLE: Color = Color::srgba(0.47, 1.0, 0.67, 0.25);

/// A drawn part of the controls, placed each frame by `place`.
#[derive(Component, Clone, Copy, PartialEq)]
enum Part {
    Root,
    Button(usize),
    Throttle,
    ThrottleMark,
    ThrottleLabel,
    StickRing,
    StickKnob,
}

/// The controls' nodes over the view (translucent), placed when the flight's first frame
/// knows the window's size.
fn spawn(mut commands: Commands, font: Res<crate::i18n::UiFont>) {
    let part = |p: Part| {
        (
            p,
            Node { position_type: PositionType::Absolute, border: UiRect::all(px(2)), ..default() },
            BackgroundColor(FILL),
            BorderColor::all(INK),
        )
    };
    let label = |text: &str, size: f32| (Text::new(text), font.at(size), TextColor(INK));
    commands
        .spawn((
            Part::Root,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Visibility::Hidden,
            // Under the in-flight menu and the end of mission's overlay.
            GlobalZIndex(-10),
            Pickable::IGNORE,
            DespawnOnExit(AppState::Flight),
        ))
        .with_children(|p| {
            for (i, b) in BUTTONS.iter().enumerate() {
                p.spawn(part(Part::Button(i))).with_child(label(t!(b.label), 14.0));
            }
            p.spawn(part(Part::Throttle));
            p.spawn((part(Part::ThrottleMark), Visibility::Hidden));
            p.spawn((
                Part::ThrottleLabel,
                Node { position_type: PositionType::Absolute, ..default() },
            ))
            .with_child(label(t!("touch-throttle"), 12.0));
            p.spawn(part(Part::StickRing));
            p.spawn(part(Part::StickKnob));
        });
}

/// Places, sizes and lights the controls' nodes for the window and the fingers.
fn place(
    state: Res<TouchState>,
    mut parts: Query<(
        &Part,
        &mut Node,
        &mut Visibility,
        Option<&mut BackgroundColor>,
        Option<&mut BorderColor>,
        Option<&Children>,
    )>,
    mut text: Query<&mut TextFont>,
) {
    let Some(layout) = state.layout else { return };
    let held = |c: Control| state.fingers.iter().any(|f| f.1 == c);
    let stick = state.fingers.iter().find_map(|f| match f.1 {
        Control::Stick { centre } => Some((centre, f.2)),
        _ => None,
    });
    let radius = STICK_RADIUS * layout.unit;
    let (top, bottom) = layout.throttle();
    let half = 0.35 * layout.unit;
    for (part, mut node, mut visibility, fill, border, children) in &mut parts {
        // A box from its top left corner and size, or a circle from its centre and radius.
        let mut place = |min: Vec2, size: Vec2| {
            node.left = px(min.x);
            node.top = px(min.y);
            node.width = px(size.x);
            node.height = px(size.y);
        };
        let mut round = |c: Vec2, r: f32| {
            place(c - Vec2::splat(r), Vec2::splat(2.0 * r));
            BorderRadius::MAX
        };
        let mut lit = None;
        match *part {
            Part::Root => {
                *visibility = if state.hidden { Visibility::Hidden } else { Visibility::Inherited };
                continue;
            }
            Part::Button(i) => {
                let (c, r) = layout.button(&BUTTONS[i]);
                node.border_radius = round(c, r);
                node.justify_content = JustifyContent::Center;
                node.align_items = AlignItems::Center;
                lit = Some(held(Control::Button(i)));
                for &child in children.into_iter().flatten() {
                    let size = bevy::text::FontSize::Px((r * 0.55).clamp(10.0, 22.0));
                    if let Ok(mut font) = text.get_mut(child)
                        && font.font_size != size
                    {
                        font.font_size = size;
                    }
                }
            }
            Part::Throttle => {
                place(top - Vec2::new(half, 0.0), Vec2::new(2.0 * half, bottom.y - top.y));
                node.border_radius = BorderRadius::all(px(6));
                lit = Some(held(Control::Throttle));
            }
            Part::ThrottleMark => {
                let Some(t) = state.throttle else { continue };
                *visibility = Visibility::Inherited;
                let y = bottom.y - t * (bottom.y - top.y);
                place(Vec2::new(top.x - 1.3 * half, y - 3.0), Vec2::new(2.6 * half, 6.0));
                lit = Some(true);
            }
            Part::ThrottleLabel => {
                node.left = px(top.x - half);
                node.width = px(2.0 * half);
                node.top = px(top.y - 0.45 * layout.unit);
                node.justify_content = JustifyContent::Center;
            }
            Part::StickRing => {
                node.border_radius = round(stick.map_or(layout.stick_rest(), |s| s.0), radius);
            }
            Part::StickKnob => {
                let (centre, at) = stick.unwrap_or((layout.stick_rest(), layout.stick_rest()));
                let at = centre + (at - centre).clamp_length_max(radius);
                node.border_radius = round(at, 0.45 * layout.unit);
                lit = Some(stick.is_some());
            }
        }
        if let Some(mut fill) = fill {
            fill.0 = if lit == Some(true) { HELD } else { FILL };
            if *part == Part::ThrottleMark {
                fill.0 = INK;
            }
        }
        if let Some(mut border) = border {
            let idle = matches!(part, Part::StickRing | Part::StickKnob) && stick.is_none();
            *border = BorderColor::all(if idle { IDLE } else { INK });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Vec2 = Vec2::new(915.0, 412.0);

    /// Each button is where a finger takes it, and none overlaps another or the throttle.
    #[test]
    fn buttons_are_hit_where_drawn() {
        let l = Layout::new(SCREEN);
        for (i, b) in BUTTONS.iter().enumerate() {
            let (c, r) = l.button(b);
            assert!(
                c.x - r >= 0.0 && c.x + r <= SCREEN.x && c.y - r >= 0.0 && c.y + r <= SCREEN.y,
                "{}",
                b.label
            );
            assert_eq!(l.hit(c), Control::Button(i), "{}", b.label);
        }
        let (top, bottom) = l.throttle();
        assert_eq!(l.hit((top + bottom) / 2.0), Control::Throttle);
        assert_eq!(l.hit(l.stick_rest()), Control::Stick { centre: l.stick_rest() });
        // The middle of the screen (the cockpit's displays) points.
        assert_eq!(l.hit(SCREEN / 2.0), Control::Pointer);
    }

    /// The throttle's slot runs from idle at the bottom to full at the top; the stick rolls
    /// right to the right and pulls dragged down, full at its radius.
    #[test]
    fn throttle_and_stick_follow_the_finger() {
        let l = Layout::new(SCREEN);
        let (top, bottom) = l.throttle();
        assert_eq!(l.throttle_at(bottom), 0.0);
        assert_eq!(l.throttle_at(top), 1.0);
        let c = l.stick_rest();
        let full = STICK_RADIUS * l.unit;
        assert_eq!(l.stick(c, c + Vec2::new(full, 0.0)), Vec2::new(1.0, 0.0));
        assert_eq!(l.stick(c, c + Vec2::new(0.0, 2.0 * full)), Vec2::new(0.0, 1.0));
        let small = l.stick(c, c + Vec2::new(-full / 4.0, 0.0));
        assert!(small.x < 0.0 && small.x > -0.25);
    }

    /// A finger on the stick and one on FIRE: the stick deflects, the weapon fires and the fire
    /// button is held; the throttle is set once as it moves.
    #[test]
    fn fingers_reach_the_flight_controls() {
        let mut app = App::new();
        let l = Layout::new(SCREEN);
        let c = l.stick_rest();
        let fire = l.button(&BUTTONS[0]).0;
        let (top, _) = l.throttle();
        app.insert_resource(TouchState {
            fingers: vec![
                (1, Control::Stick { centre: c }, c + Vec2::new(STICK_RADIUS * l.unit, 0.0), false),
                (2, Control::Button(0), fire, true),
                (3, Control::Throttle, top, false),
            ],
            layout: Some(l),
            ..default()
        })
        .init_resource::<FlightControls>()
        .add_message::<ActionEvent>()
        .add_systems(Update, apply);
        app.update();
        let controls = app.world().resource::<FlightControls>();
        assert_eq!(controls.roll, 1.0);
        assert!(controls.fire);
        assert_eq!(controls.throttle, Some(1.0));
        let messages = app.world().resource::<Messages<ActionEvent>>();
        let sent: Vec<Action> = messages.get_cursor().read(messages).map(|e| e.0).collect();
        assert_eq!(sent, vec![Action::FireWeapon]);
        // The throttle held still is not sent again.
        app.world_mut().resource_mut::<FlightControls>().throttle = None;
        app.update();
        assert_eq!(app.world().resource::<FlightControls>().throttle, None);
    }
}

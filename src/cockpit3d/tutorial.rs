//! Interactive tutorial (single missions, any theater and mission): one step at a time, from
//! the takeoff to the landing, a box says what to do next and which keys do it, an arrow points
//! at the cockpit part to press or look at, and that part blinks sky blue.
//!
//! A step ends when its condition holds (gear up, bay open, GND mode, a page shown...), however
//! it was done: a click in the cockpit, a key, the HOTAS or a VR fingertip; steps already done
//! are skipped. Only what has no state (the weapon choice, a designation) waits for its key
//! action. The steps follow the flight: jets or helicopters, a bomb bay or pylons, armed or
//! not, a strike or an air-to-air mission, the classic or the modern avionics (whose MFD pages
//! are formats).
//!
//! The arrow points at the part while the view shows it; otherwise it sticks to the window's
//! edge on the part's side, the box following it. Outside the cockpit views the box asks for the
//! cockpit view. The blinking overlay is the part's own meshes drawn again over everything
//! (`OnTop`), so it hugs their shape (and shows in the GL headset too); the box and the arrow
//! are desktop only for now.
//!
//! Every text is in `STEPS` (to translate them one day; the aviation terms stay English).

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::MeshAabb;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use stealth_mission::MissionKind;

use super::CockpitRoot;
use super::controls::{Control, Highlight, press_target};
use super::displays::CanvasRole;
use super::layout::Effect;
use super::levers::{Stick, ThrottleLever};
use super::tooltips::{FONT, OnTop, TipScreenMaterial, keys_text, wrap};
use crate::avionics::Format;
use crate::camera::{EyeCamera, View, ViewState};
use crate::flight::{Aircraft, Player};
use crate::hotas::InputConfig;
use crate::hud::{HudMode as Hud, HudState};
use crate::input::{Action, ActionEvent, FlightControls};
use crate::mfd::{LeftMfd, MfdCmd, MfdSide, MfdState, OsbFn, RightMfd, osb};
use crate::settings::{AppState, Settings};

/// Sky blue: the blinking overlay, the arrow and the box's outline.
const SKY: Color = Color::srgb(0.53, 0.81, 0.98);
/// The arrow's size on the screen (logical pixels) and its picture's (pixels, pointing right).
const ARROW: f32 = 56.0;
const ARROW_IMAGE: u32 = 64;
/// Gap between the arrow's tip and the part, and between the arrow and the box.
const GAP: f32 = 10.0;
const BOX_GAP: f32 = 14.0;
/// The arrow's tip stays this far inside the window.
const EDGE: f32 = 12.0;
/// A part this near the window's centre gets the arrow from the top left.
const NEAR_CENTRE: f32 = 120.0;
/// The arrow's back and forth (pixels, per second).
const BOB: f32 = 6.0;
const BOB_RATE: f32 = 1.2;
/// The overlay's blinks per second.
const BLINK_RATE: f32 = 1.5;
/// Seconds the last step's box stays up.
const LAST_STEP_SECONDS: f32 = 15.0;

/// What a step shows.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Target {
    /// Nothing: the box alone, at the top of the window.
    #[default]
    None,
    /// The control a key action presses (as a script `press=`).
    Action(Action),
    /// That control, else the one its key action presses.
    Effect(Effect),
    /// The bezel button giving that display command.
    Cmd(MfdCmd),
    Display(CanvasRole),
    Stick,
    /// The throttle (a helicopter's collective).
    Throttle,
}

/// An MFD page as the steps know it (with the modern avionics: a format, or a format's page).
#[derive(Debug, Clone, Copy, PartialEq)]
enum Page {
    Orders,
    Weapons,
    Waypoints,
    /// The classic TAC page, the modern HSD.
    Tactical,
    Map,
}

/// The flight the steps are chosen for.
#[derive(Debug, Clone, Copy, Default)]
struct Flight {
    helicopter: bool,
    bomb_bay: bool,
    /// Carries weapons (not a troop helicopter).
    armed: bool,
    /// An air-to-air mission (else a strike).
    air: bool,
}

/// The flights a step is for.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Only {
    All,
    Jets,
    Helicopters,
    BombBay,
    Armed,
    Strike,
    AirToAir,
}

impl Only {
    fn fits(self, f: Flight) -> bool {
        match self {
            Only::All => true,
            Only::Jets => !f.helicopter,
            Only::Helicopters => f.helicopter,
            Only::BombBay => f.bomb_bay,
            Only::Armed => f.armed,
            Only::Strike => f.armed && !f.air,
            Only::AirToAir => f.armed && f.air,
        }
    }
}

/// What the steps read.
struct Ctx<'a> {
    a: &'a Aircraft,
    hud: &'a HudState,
    mfd: &'a MfdState,
    obj: stealth_sim::Objectives,
    /// The trigger is held.
    fire: bool,
    /// Key actions since the step began.
    seen: &'a [Action],
    /// Seconds on the step.
    dwell: f32,
}

impl Ctx<'_> {
    fn saw(&self, any: &[Action]) -> bool {
        self.seen.iter().any(|a| any.contains(a))
    }

    fn knots(&self) -> f32 {
        self.a.velocity / 27.0
    }

    /// The MFD showing a page (None: not shown).
    fn page(&self, page: Page) -> Option<MfdSide> {
        let m = self.mfd;
        if m.modern {
            let av = &m.av;
            return match page {
                Page::Orders => av.shown(Format::Sms).filter(|_| av.sms_ord),
                Page::Weapons => av.shown(Format::Sms).filter(|_| !av.sms_ord),
                Page::Waypoints => av.shown(Format::Map).filter(|_| av.map_way),
                Page::Tactical => av.shown(Format::Hsd),
                Page::Map => av.shown(Format::Map),
            };
        }
        let (side, shown) = match page {
            Page::Orders => (MfdSide::Right, m.right == RightMfd::Ord),
            Page::Weapons => (MfdSide::Right, m.right == RightMfd::Wpn),
            Page::Waypoints => (MfdSide::Right, m.right == RightMfd::Way),
            Page::Tactical => (MfdSide::Left, m.left == LeftMfd::Tac),
            Page::Map => (MfdSide::Left, m.left == LeftMfd::Map),
        };
        shown.then_some(side)
    }

    /// The button that shows a page: with the modern avionics the format's own button when the
    /// format shows (ORD, WPN, WAY), else the format's button.
    fn page_button(&self, page: Page) -> Target {
        if !self.mfd.modern {
            return match page {
                Page::Orders => Target::Action(Action::MissionOrders),
                Page::Weapons => Target::Action(Action::Ordnance),
                Page::Waypoints => Target::Action(Action::SelectWaypoint),
                Page::Tactical => Target::Effect(Effect::LeftPage(LeftMfd::Tac)),
                Page::Map => Target::Effect(Effect::LeftPage(LeftMfd::Map)),
            };
        }
        let (format, own, side) = match page {
            Page::Orders => (Format::Sms, Some(MfdCmd::Sms(true)), MfdSide::Right),
            Page::Weapons => (Format::Sms, Some(MfdCmd::Sms(false)), MfdSide::Right),
            Page::Waypoints => (Format::Map, Some(MfdCmd::MapWay), MfdSide::Right),
            Page::Tactical => (Format::Hsd, None, MfdSide::Left),
            Page::Map => (Format::Map, None, MfdSide::Left),
        };
        match own {
            Some(cmd) if self.mfd.av.shown(format).is_some() => Target::Cmd(cmd),
            _ => Target::Cmd(MfdCmd::Format(side, format)),
        }
    }

    /// The display showing a page.
    fn page_display(&self, page: Page) -> Target {
        self.page(page).map_or(Target::None, |side| {
            Target::Display(match side {
                MfdSide::Left => CanvasRole::Left,
                MfdSide::Right => CanvasRole::Right,
                MfdSide::Third => CanvasRole::Third,
            })
        })
    }
}

struct Step {
    only: Only,
    phase: &'static str,
    title: &'static str,
    detail: &'static str,
    keys: &'static [Action],
    target: fn(&Ctx) -> Target,
    done: fn(&Ctx) -> bool,
}

const TAKEOFF: &str = "TAKEOFF";
const NAVIGATION: &str = "NAVIGATION";
const ATTACK: &str = "ATTACK";
const RETURN: &str = "RETURN";
const LANDING: &str = "LANDING";

/// The lesson: a mission from the runway to the stop after landing.
static STEPS: &[Step] = {
    use Action::*;
    use Target as T;
    &[
        Step {
            only: Only::Jets,
            phase: TAKEOFF,
            title: "Full throttle",
            detail: "The engine runs: push the throttle to full power for the takeoff run",
            keys: &[ThrottleMax, ThrottleUp],
            target: |_| T::Throttle,
            done: |c| c.a.set_thrust >= 95.0 || !c.a.on_ground,
        },
        Step {
            only: Only::Jets,
            phase: TAKEOFF,
            title: "Flaps down",
            detail: "More lift: the aircraft leaves the runway at a lower speed",
            keys: &[Flaps],
            target: |_| T::Action(Flaps),
            done: |c| c.a.flaps || !c.a.on_ground,
        },
        Step {
            only: Only::Jets,
            phase: TAKEOFF,
            title: "Rotate at about 160 knots",
            detail: "Pull the stick back gently until the aircraft leaves the runway, then climb",
            keys: &[PitchUp],
            target: |_| T::Stick,
            done: |c| !c.a.on_ground,
        },
        Step {
            only: Only::Helicopters,
            phase: TAKEOFF,
            title: "Lift off",
            detail: "Raise the collective until the helicopter leaves the ground",
            keys: &[ThrottleUp],
            target: |_| T::Throttle,
            done: |c| !c.a.on_ground,
        },
        Step {
            only: Only::Jets,
            phase: TAKEOFF,
            title: "Gear up",
            detail: "Once climbing: the gear out is slow and seen by the radars",
            keys: &[Gear],
            target: |_| T::Action(Gear),
            done: |c| !c.a.gear_down,
        },
        Step {
            only: Only::Jets,
            phase: TAKEOFF,
            title: "Flaps up",
            detail: "Less drag for the climb and the cruise",
            keys: &[Flaps],
            target: |_| T::Action(Flaps),
            done: |c| !c.a.flaps,
        },
        Step {
            only: Only::All,
            phase: NAVIGATION,
            title: "Mission orders page",
            detail: "An MFD shows the orders: the primary and secondary targets",
            keys: &[MissionOrders],
            target: |c| c.page_button(Page::Orders),
            done: |c| c.page(Page::Orders).is_some(),
        },
        Step {
            only: Only::All,
            phase: NAVIGATION,
            title: "Read your orders",
            detail: "The targets to destroy and the base to land at",
            keys: &[],
            target: |c| c.page_display(Page::Orders),
            done: |c| c.dwell >= 8.0,
        },
        Step {
            only: Only::All,
            phase: NAVIGATION,
            title: "Tactical page",
            detail: "The threats and the targets around the aircraft",
            keys: &[Maps],
            target: |c| c.page_button(Page::Tactical),
            done: |c| c.page(Page::Tactical).is_some(),
        },
        Step {
            only: Only::All,
            phase: NAVIGATION,
            title: "Map page",
            detail: "The route, the waypoints and the enemy radars",
            keys: &[Maps],
            target: |c| c.page_button(Page::Map),
            done: |c| c.page(Page::Map).is_some(),
        },
        Step {
            only: Only::All,
            phase: NAVIGATION,
            title: "Waypoint page",
            detail: "The current waypoint, its bearing and distance",
            keys: &[SelectWaypoint],
            target: |c| c.page_button(Page::Waypoints),
            done: |c| c.page(Page::Waypoints).is_some(),
        },
        Step {
            only: Only::Jets,
            phase: NAVIGATION,
            title: "Autopilot",
            detail: "Flies to the current waypoint; any stick input takes the controls back",
            keys: &[Autopilot],
            target: |_| T::Action(Autopilot),
            done: |c| c.a.autopilot_alt > 0.0,
        },
        Step {
            only: Only::Strike,
            phase: ATTACK,
            title: "Air-to-ground mode",
            detail: "GND mode: the HUD shows the ground targets and the bombing cues",
            keys: &[HudMode],
            target: |_| T::Effect(Effect::HudMode(2)),
            done: |c| c.hud.mode == Hud::Gnd,
        },
        Step {
            only: Only::AirToAir,
            phase: ATTACK,
            title: "Air-to-air mode",
            detail: "AIR mode: the HUD shows the aircraft targets",
            keys: &[HudMode],
            target: |_| T::Effect(Effect::HudMode(1)),
            done: |c| c.hud.mode == Hud::Air,
        },
        Step {
            only: Only::Armed,
            phase: ATTACK,
            title: "Weapons page",
            detail: "The weapons on board and the selected one",
            keys: &[Ordnance],
            target: |c| c.page_button(Page::Weapons),
            done: |c| c.page(Page::Weapons).is_some(),
        },
        Step {
            only: Only::Armed,
            phase: ATTACK,
            title: "Select a weapon",
            detail: "Bombs for a ground target, missiles for an aircraft; after the last, the gun",
            keys: &[SelectWeapon],
            target: |_| T::Action(SelectWeapon),
            done: |c| c.saw(&[SelectWeapon]),
        },
        Step {
            only: Only::BombBay,
            phase: ATTACK,
            title: "Open the weapon bay",
            detail: "Weapons are released with open bays only; open bays are seen by the radars",
            keys: &[BayDoors],
            target: |_| T::Action(BayDoors),
            done: |c| c.a.bay_open,
        },
        Step {
            only: Only::Armed,
            phase: ATTACK,
            title: "Designate the target",
            detail: "Lock the target: steady within weapon range",
            keys: &[DesignateTarget, NearestTarget, SelectTarget],
            target: |_| T::Action(DesignateTarget),
            done: |c| c.saw(&[DesignateTarget, NearestTarget, SelectTarget]),
        },
        Step {
            only: Only::Armed,
            phase: ATTACK,
            title: "Fire",
            detail: "In range, release the selected weapon",
            keys: &[FireWeapon],
            target: |_| T::Stick,
            done: |c| c.fire || c.obj.primary_done || c.obj.secondary_done,
        },
        Step {
            only: Only::BombBay,
            phase: ATTACK,
            title: "Close the weapon bay",
            detail: "Back to stealth: open bays are seen by the radars",
            keys: &[BayDoors],
            target: |_| T::Action(BayDoors),
            done: |c| !c.a.bay_open,
        },
        Step {
            only: Only::All,
            phase: RETURN,
            title: "Navigation mode",
            detail: "NAV mode: the HUD steers to the current waypoint",
            keys: &[HudMode],
            target: |_| T::Effect(Effect::HudMode(0)),
            done: |c| c.hud.mode == Hud::Nav,
        },
        Step {
            only: Only::All,
            phase: RETURN,
            title: "Steer to the landing base",
            detail: "Next waypoint until the base (after both targets it is the next one)",
            keys: &[NextWaypoint, ChangeWaypoint],
            target: |_| T::Action(NextWaypoint),
            done: |c| {
                c.saw(&[NextWaypoint, ChangeWaypoint, LastWaypoint])
                    || c.obj.primary_done && c.obj.secondary_done
            },
        },
        Step {
            only: Only::All,
            phase: LANDING,
            title: "ILS",
            detail: "Instrument landing system: the HUD steering bars lead to the runway",
            keys: &[Ils],
            target: |_| T::Action(Ils),
            done: |c| c.mfd.ils,
        },
        Step {
            only: Only::Jets,
            phase: LANDING,
            title: "Reduce the throttle",
            detail: "Slow down for the approach, below 250 knots",
            keys: &[ThrottleDown, ThrottleIdle],
            target: |_| T::Throttle,
            done: |c| c.a.set_thrust <= 60.0,
        },
        Step {
            only: Only::Jets,
            phase: LANDING,
            title: "Gear down",
            detail: "Three green lamps when it is down",
            keys: &[Gear],
            target: |_| T::Action(Gear),
            done: |c| c.a.gear_down,
        },
        Step {
            only: Only::Jets,
            phase: LANDING,
            title: "Flaps down",
            detail: "A lower approach speed",
            keys: &[Flaps],
            target: |_| T::Action(Flaps),
            done: |c| c.a.flaps,
        },
        Step {
            only: Only::Jets,
            phase: LANDING,
            title: "Touch down",
            detail: "Keep the green circle (on speed) and a gentle descent to the runway",
            keys: &[],
            target: |_| T::Display(CanvasRole::Approach),
            done: |c| c.a.on_ground && c.a.has_flown,
        },
        Step {
            only: Only::Helicopters,
            phase: LANDING,
            title: "Land",
            detail: "Slow to a hover over the base, then lower the collective gently",
            keys: &[ThrottleDown],
            target: |_| T::Throttle,
            done: |c| c.a.on_ground && c.a.has_flown,
        },
        Step {
            only: Only::Jets,
            phase: LANDING,
            title: "Brakes",
            detail: "Throttle to idle and brake to a stop",
            keys: &[Brakes, ThrottleIdle],
            target: |_| T::Action(Brakes),
            done: |c| c.a.on_ground && (c.a.brakes || c.knots() < 5.0),
        },
        Step {
            only: Only::All,
            phase: LANDING,
            title: "Mission complete",
            detail: "End of the tutorial. The in-flight menu switches it off",
            keys: &[Menu],
            target: |_| T::None,
            done: |_| false,
        },
    ]
};

/// Where the lesson is.
#[derive(Resource, Default)]
pub(super) struct Tutorial {
    /// Switched on, in a single mission.
    on: bool,
    /// Index in `STEPS`.
    step: usize,
    /// The step's number among the flight's steps and their count (from 1).
    number: (usize, usize),
    /// Seconds on the step (not while the menu is open).
    dwell: f32,
    /// Key actions since the step began (each once).
    seen: Vec<Action>,
    /// What the step shows, and the cockpit part that is.
    target: Target,
    part: Option<Entity>,
    /// The part blinking, and its overlays.
    glowing: Option<Entity>,
    glows: Vec<Entity>,
}

impl Tutorial {
    fn current(&self) -> Option<&'static Step> {
        STEPS.get(self.step)
    }
}

/// The arrow's picture, the overlay's material and the box's font.
#[derive(Resource, Clone)]
pub(super) struct TutorialLook {
    arrow: Handle<Image>,
    glow: Handle<TipScreenMaterial>,
    font: Handle<Font>,
}

#[derive(Component)]
pub(super) struct TutorialArrow;

#[derive(Component)]
pub(super) struct TutorialBox;

#[derive(Component)]
pub(super) struct TutorialLine(usize);

/// A part's mesh drawn again in sky blue over everything.
#[derive(Component)]
pub(super) struct TutorialGlow;

fn make_look(
    images: &mut Assets<Image>,
    materials: &mut Assets<TipScreenMaterial>,
    fonts: &mut Assets<Font>,
) -> TutorialLook {
    let size = Extent3d { width: ARROW_IMAGE, height: ARROW_IMAGE, depth_or_array_layers: 1 };
    let arrow = Image::new(
        size,
        TextureDimension::D2,
        arrow_pixels(ARROW_IMAGE),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    TutorialLook {
        arrow: images.add(arrow),
        glow: materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: SKY.with_alpha(0.4),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                fog_enabled: false,
                ..default()
            },
            extension: OnTop {},
        }),
        font: fonts.add(Font::from_bytes(FONT.to_vec())),
    }
}

/// An arrow pointing right (+X), white with a dark outline (tinted by the node).
fn arrow_pixels(n: u32) -> Vec<u8> {
    let s = n as f32;
    // The shaft from the left to the middle, the head from there to the right edge.
    let inside = |x: f32, y: f32| {
        let dy = (y - s / 2.0).abs();
        if x < s * 0.06 || x > s * 0.97 {
            false
        } else if x < s * 0.5 {
            dy <= s * 0.11
        } else {
            dy <= (s * 0.97 - x) * 0.85
        }
    };
    const R: f32 = 2.5;
    let outline = |x: f32, y: f32| {
        [(-R, 0.0), (R, 0.0), (0.0, -R), (0.0, R), (-R, -R), (R, R), (-R, R), (R, -R)]
            .iter()
            .any(|&(dx, dy)| inside(x + dx, y + dy))
    };
    let mut px = Vec::with_capacity((n * n * 4) as usize);
    for j in 0..n {
        for i in 0..n {
            let (x, y) = (i as f32 + 0.5, j as f32 + 0.5);
            px.extend_from_slice(&if inside(x, y) {
                [255, 255, 255, 255]
            } else if outline(x, y) {
                [10, 20, 30, 220]
            } else {
                [0, 0, 0, 0]
            });
        }
    }
    px
}

/// The arrow and the box (hidden), the lesson from its start. The first flight makes their
/// look (a flight started from the command line enters before `Startup`).
pub(super) fn spawn_overlay(
    mut commands: Commands,
    look: Option<Res<TutorialLook>>,
    mut tutorial: ResMut<Tutorial>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TipScreenMaterial>>,
    mut fonts: ResMut<Assets<Font>>,
) {
    *tutorial = Tutorial::default();
    let look = match look {
        Some(look) => look.clone(),
        None => {
            let look = make_look(&mut images, &mut materials, &mut fonts);
            commands.insert_resource(look.clone());
            look
        }
    };
    commands.spawn((
        TutorialArrow,
        ImageNode { color: SKY, ..ImageNode::new(look.arrow) },
        Node {
            position_type: PositionType::Absolute,
            width: px(ARROW),
            height: px(ARROW),
            ..default()
        },
        UiTransform::IDENTITY,
        GlobalZIndex(9),
        Visibility::Hidden,
        DespawnOnExit(AppState::Flight),
    ));
    let line = |i: usize, size: f32, color: Color| {
        (
            TutorialLine(i),
            Text::new(""),
            TextFont {
                font: look.font.clone().into(),
                font_size: bevy::text::FontSize::Px(size),
                ..default()
            },
            TextColor(color),
            TextLayout::no_wrap(),
            Node::default(),
        )
    };
    commands.spawn((
        TutorialBox,
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            row_gap: px(3.0),
            padding: UiRect::axes(px(10.0), px(7.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.9)),
        Outline::new(px(2.0), Val::ZERO, SKY.with_alpha(0.8)),
        GlobalZIndex(9),
        Visibility::Hidden,
        DespawnOnExit(AppState::Flight),
        children![
            line(0, 12.0, Color::srgb(0.55, 0.65, 0.75)),
            line(1, 18.0, SKY),
            line(2, 14.0, Color::srgb(0.85, 0.87, 0.9)),
            line(3, 13.0, Color::srgb(1.0, 0.85, 0.3)),
        ],
    ));
}

/// Moves on while the current step is done or not for this flight, and says what the step
/// shows.
#[allow(clippy::too_many_arguments)]
pub(super) fn advance(
    time: Res<Time<Real>>,
    settings: Res<Settings>,
    menu: Res<crate::flight_menu::FlightMenu>,
    mission: Option<Res<crate::mission::ActiveMission>>,
    mut tutorial: ResMut<Tutorial>,
    mut actions: MessageReader<ActionEvent>,
    aircraft: Query<&Aircraft, With<Player>>,
    (hud, mfd, controls): (Res<HudState>, Res<MfdState>, Res<FlightControls>),
    combat: Option<Res<crate::combat::Combat>>,
    (career, campaign): (
        Option<Res<crate::career::CareerFlight>>,
        Option<Res<crate::campaign::ActiveCampaign>>,
    ),
) {
    let tutorial = &mut *tutorial;
    tutorial.on =
        settings.tutorial && crate::mission::single_mission(career.as_deref(), campaign.as_deref());
    if !tutorial.on {
        actions.clear();
        return;
    }
    for ActionEvent(a) in actions.read() {
        if !tutorial.seen.contains(a) {
            tutorial.seen.push(*a);
        }
    }
    if !menu.open {
        tutorial.dwell += time.delta_secs();
    }
    let spec = settings.aircraft.spec();
    let flight = Flight {
        helicopter: spec.helicopter,
        bomb_bay: spec.bomb_bay,
        armed: !spec.transport(),
        air: mission.is_some_and(|m| {
            matches!(
                m.0.settings.kind,
                MissionKind::Air | MissionKind::AirTraining | MissionKind::DroneDefense
            )
        }),
    };
    let Ok(a) = aircraft.single() else { return };
    let obj = combat.map(|c| c.sim.objectives()).unwrap_or_default();
    tutorial.target = Target::None;
    while let Some(step) = tutorial.current() {
        if step.only.fits(flight) {
            let ctx = Ctx {
                a,
                hud: &hud,
                mfd: &mfd,
                obj,
                fire: controls.fire,
                seen: &tutorial.seen,
                dwell: tutorial.dwell,
            };
            if !(step.done)(&ctx) {
                tutorial.target = (step.target)(&ctx);
                break;
            }
            tutorial.dwell = 0.0;
            tutorial.seen.clear();
        }
        tutorial.step += 1;
    }
    let fits = |s: &&Step| s.only.fits(flight);
    tutorial.number = (
        STEPS[..tutorial.step.min(STEPS.len())].iter().filter(fits).count() + 1,
        STEPS.iter().filter(fits).count(),
    );
}

/// The world bounds (min, max) of each cockpit mesh, in its own frame.
type MeshBounds = HashMap<AssetId<Mesh>, Option<(Vec3, Vec3)>>;

type BoxView = (&'static mut Node, &'static mut Visibility, &'static ComputedNode);
type LineView = (&'static TutorialLine, &'static mut Text, &'static mut Node);

/// The arrow, the box and its lines.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Overlay<'w, 's> {
    arrows: Query<
        'w,
        's,
        (&'static mut Node, &'static mut Visibility, &'static mut UiTransform),
        With<TutorialArrow>,
    >,
    boxes: Query<'w, 's, BoxView, (With<TutorialBox>, Without<TutorialArrow>)>,
    lines: Query<'w, 's, LineView, (Without<TutorialBox>, Without<TutorialArrow>)>,
}

/// The cockpit parts a step points at.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Parts<'w, 's> {
    controls: Query<'w, 's, (Entity, &'static Control, &'static Transform)>,
    displays: Query<'w, 's, (Entity, &'static CanvasRole), With<Mesh3d>>,
    sticks: Query<'w, 's, Entity, With<Stick>>,
    throttles: Query<'w, 's, Entity, With<ThrottleLever>>,
    root: Query<'w, 's, &'static GlobalTransform, With<CockpitRoot>>,
    children: Query<'w, 's, &'static Children>,
    /// The cockpit's meshes (not the overlays).
    meshes: Query<'w, 's, (&'static Mesh3d, &'static GlobalTransform), Without<TutorialGlow>>,
    mesh_assets: Res<'w, Assets<Mesh>>,
}

impl Parts<'_, '_> {
    /// The part a target is (None: not in this cockpit).
    fn entity(&self, target: Target, mfd: &MfdState) -> Option<Entity> {
        let by_action =
            |a: Action| press_target(self.controls.iter().map(|(e, c, _)| (e, c)), a, mfd);
        let find = |f: &dyn Fn(Effect) -> bool| {
            self.controls.iter().find(|(_, c, _)| f(c.effect)).map(|(e, ..)| e)
        };
        match target {
            Target::None => None,
            Target::Action(a) => by_action(a),
            Target::Effect(effect) => {
                find(&|e| e == effect).or_else(|| by_action(effect.action(mfd)?))
            }
            Target::Cmd(cmd) => find(&|e| {
                matches!(e, Effect::Osb(side, slot)
                    if osb(side, slot, mfd).is_some_and(|o| o.1 == OsbFn::Cmd(cmd)))
            }),
            Target::Display(role) => {
                self.displays.iter().find(|(_, r)| **r == role).map(|(e, _)| e)
            }
            Target::Stick => self.sticks.iter().next(),
            Target::Throttle => self.throttles.iter().next(),
        }
    }

    /// Where the arrow points (world): a control's face centre, else the middle of the part's
    /// meshes (the cockpit's have no `Aabb`: they are not frustum culled).
    fn point(&self, e: Entity, cache: &mut MeshBounds) -> Option<Vec3> {
        if let Ok((_, c, tf)) = self.controls.get(e) {
            return Some(self.root.single().ok()?.transform_point(c.face_centre(tf)));
        }
        let (mut lo, mut hi) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
        let mut stack = vec![e];
        while let Some(e) = stack.pop() {
            if let Ok((mesh, tf)) = self.meshes.get(e) {
                let bounds = *cache.entry(mesh.id()).or_insert_with(|| {
                    let aabb = self.mesh_assets.get(mesh.id())?.compute_aabb()?;
                    Some((aabb.min().into(), aabb.max().into()))
                });
                if let Some((min, max)) = bounds {
                    for k in 0..8 {
                        let corner = Vec3::new(
                            if k & 1 == 0 { min.x } else { max.x },
                            if k & 2 == 0 { min.y } else { max.y },
                            if k & 4 == 0 { min.z } else { max.z },
                        );
                        let p = tf.transform_point(corner);
                        (lo, hi) = (lo.min(p), hi.max(p));
                    }
                }
            }
            if let Ok(kids) = self.children.get(e) {
                stack.extend(kids.iter());
            }
        }
        (lo.x <= hi.x).then(|| (lo + hi) / 2.0)
    }
}

/// Finds the step's part, and makes it blink: its meshes drawn again in sky blue (not the
/// touch highlight frames).
#[allow(clippy::too_many_arguments)]
pub(super) fn glow(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mfd: Res<MfdState>,
    mut tutorial: ResMut<Tutorial>,
    parts: Parts,
    look: Res<TutorialLook>,
    mut materials: ResMut<Assets<TipScreenMaterial>>,
    glows: Query<(), With<TutorialGlow>>,
    highlights: Query<(), With<Highlight>>,
) {
    let tutorial = &mut *tutorial;
    tutorial.part = tutorial.on.then(|| parts.entity(tutorial.target, &mfd)).flatten();
    // The overlays went with a rebuilt cockpit: made again.
    let gone = tutorial.glows.iter().any(|&g| !glows.contains(g));
    if tutorial.part != tutorial.glowing || gone {
        for g in tutorial.glows.drain(..) {
            if let Ok(mut ec) = commands.get_entity(g) {
                ec.despawn();
            }
        }
        tutorial.glowing = tutorial.part;
        let mut stack: Vec<Entity> = tutorial.part.into_iter().collect();
        while let Some(e) = stack.pop() {
            if highlights.contains(e) {
                continue;
            }
            if let Ok((mesh, _)) = parts.meshes.get(e) {
                let g = commands
                    .spawn((
                        TutorialGlow,
                        Mesh3d(mesh.0.clone()),
                        MeshMaterial3d(look.glow.clone()),
                        Transform::IDENTITY,
                        Visibility::Inherited,
                        NotShadowCaster,
                        NoFrustumCulling,
                        ChildOf(e),
                    ))
                    .id();
                tutorial.glows.push(g);
            }
            if let Ok(kids) = parts.children.get(e) {
                stack.extend(kids.iter().filter(|k| !glows.contains(*k)));
            }
        }
    }
    if tutorial.glowing.is_some()
        && let Some(mut m) = materials.get_mut(&look.glow)
    {
        let wave = 0.5 + 0.5 * (std::f32::consts::TAU * BLINK_RATE * time.elapsed_secs()).sin();
        m.base.base_color = SKY.with_alpha(0.15 + 0.45 * wave);
    }
}

/// Where the ray from `centre` along `dir` leaves the rectangle `min..max`.
fn edge_point(centre: Vec2, dir: Vec2, min: Vec2, max: Vec2) -> Vec2 {
    let t = |c: f32, d: f32, lo: f32, hi: f32| {
        if d > 1e-6 {
            (hi - c) / d
        } else if d < -1e-6 {
            (lo - c) / d
        } else {
            f32::INFINITY
        }
    };
    let s = t(centre.x, dir.x, min.x, max.x).min(t(centre.y, dir.y, min.y, max.y));
    centre + dir * s
}

/// The arrow's tip and direction (window, logical pixels): at the part while it is in the
/// window (`seen`: where it shows), else on the window's edge on its side (`towards`: its
/// direction on the screen).
fn arrow_for(seen: Option<Vec2>, towards: Vec2, window: Vec2) -> (Vec2, Vec2) {
    let centre = window / 2.0;
    match seen {
        Some(p) => {
            // From the window's centre towards the part.
            let away = p - centre;
            let dir = if away.length() > NEAR_CENTRE {
                away.normalize()
            } else {
                Vec2::new(1.0, 1.0).normalize()
            };
            (p - dir * GAP, dir)
        }
        None => {
            let dir = towards.normalize_or(Vec2::Y);
            (edge_point(centre, dir, Vec2::splat(EDGE), window - EDGE), dir)
        }
    }
}

/// Shows the step's box and the arrow pointing at its part (desktop). Not with the menu or
/// the kneeboard over the view.
#[allow(clippy::too_many_arguments)]
pub(super) fn show(
    time: Res<Time<Real>>,
    settings: Res<Settings>,
    view: Res<ViewState>,
    menu: Res<crate::flight_menu::FlightMenu>,
    hide: Option<Res<crate::camera::HideOwnAircraft>>,
    kneeboard: Option<Res<crate::kneeboard::Kneeboard>>,
    tutorial: Res<Tutorial>,
    input: Res<InputConfig>,
    parts: Parts,
    mut cache: Local<MeshBounds>,
    cams: Query<(&Camera, &GlobalTransform), With<EyeCamera>>,
    windows: Query<&Window>,
    mut overlay: Overlay,
) {
    let step = tutorial.current().filter(|_| {
        tutorial.on
            && !settings.vr
            && !menu.open
            && !hide.is_some_and(|h| h.0)
            && !kneeboard.is_some_and(|k| k.overlay_shown())
            && !(tutorial.step + 1 == STEPS.len() && tutorial.dwell > LAST_STEP_SECONDS)
    });
    let (Ok((mut arrow, mut arrow_vis, mut turn)), Ok((mut node, mut box_vis, computed))) =
        (overlay.arrows.single_mut(), overlay.boxes.single_mut())
    else {
        return;
    };
    let window = windows.single().ok().map(|w| w.size());
    let (Some(step), Some(window)) = (step, window) else {
        arrow_vis.set_if_neq(Visibility::Hidden);
        box_vis.set_if_neq(Visibility::Hidden);
        return;
    };
    let cockpit_view =
        matches!(view.view, View::Cockpit | View::LookLeft | View::LookRight | View::LookRear);

    // The box's lines; outside the cockpit, how to get back to it.
    let (number, count) = tutorial.number;
    let header = format!("INTERACTIVE TUTORIAL  {number}/{count}  {}", step.phase);
    let (detail, keys) = if tutorial.part.is_some() && !cockpit_view {
        ("Switch to the cockpit view to see it", keys_text(&[Action::ViewCockpit], &input))
    } else {
        (step.detail, keys_text(step.keys, &input))
    };
    let texts = [header, wrap(step.title), wrap(detail), wrap(&keys)];
    for (TutorialLine(i), mut text, mut line) in &mut overlay.lines {
        let s = &texts[*i];
        if text.0 != *s {
            text.0.clone_from(s);
        }
        let display = if s.is_empty() { Display::None } else { Display::Flex };
        if line.display != display {
            line.display = display;
        }
    }
    box_vis.set_if_neq(Visibility::Inherited);
    let size = computed.size() * computed.inverse_scale_factor();

    let at = tutorial.part.filter(|_| cockpit_view).and_then(|e| parts.point(e, &mut cache));
    let Some(((camera, cam_tf), at)) = cams.single().ok().zip(at) else {
        // Nothing to point at: the box at the top of the window.
        arrow_vis.set_if_neq(Visibility::Hidden);
        place(&mut node, Vec2::new((window.x - size.x) / 2.0, 70.0));
        return;
    };
    let local = cam_tf.affine().inverse().transform_point3(at);
    let seen = (local.z < 0.0)
        .then(|| camera.world_to_viewport(cam_tf, at).ok())
        .flatten()
        .filter(|p| p.cmpge(Vec2::splat(EDGE)).all() && p.cmple(window - EDGE).all());
    let (tip, dir) = arrow_for(seen, Vec2::new(local.x, -local.y), window);
    let bob = dir * BOB * (std::f32::consts::TAU * BOB_RATE * time.elapsed_secs()).sin();
    place(&mut arrow, tip - dir * ARROW / 2.0 + bob - ARROW / 2.0);
    let rotation = UiTransform::from_rotation(Rot2::radians(dir.y.atan2(dir.x)));
    if *turn != rotation {
        *turn = rotation;
    }
    arrow_vis.set_if_neq(Visibility::Inherited);

    // The box behind the arrow's tail, inside the window.
    let tail = tip - dir * (ARROW + BOX_GAP);
    let reach = (dir.x.abs() * size.x + dir.y.abs() * size.y) / 2.0;
    let corner = tail - dir * reach - size / 2.0;
    place(&mut node, corner.clamp(Vec2::splat(4.0), (window - size - 4.0).max(Vec2::splat(4.0))));
}

/// Puts a node's top left corner there.
fn place(node: &mut Node, corner: Vec2) {
    if node.left != px(corner.x) || node.top != px(corner.y) {
        node.left = px(corner.x);
        node.top = px(corner.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arrow_sticks_to_the_window_edge() {
        let w = Vec2::new(800.0, 600.0);
        let (lo, hi) = (Vec2::splat(EDGE), w - EDGE);
        let c = w / 2.0;
        assert_eq!(edge_point(c, Vec2::X, lo, hi), Vec2::new(w.x - EDGE, 300.0));
        assert_eq!(edge_point(c, -Vec2::X, lo, hi), Vec2::new(EDGE, 300.0));
        assert_eq!(edge_point(c, Vec2::Y, lo, hi), Vec2::new(400.0, w.y - EDGE));
        assert_eq!(edge_point(c, -Vec2::Y, lo, hi), Vec2::new(400.0, EDGE));
        // Diagonally: the nearer edge (the top and bottom ones in a wide window).
        let p = edge_point(c, Vec2::new(1.0, 1.0).normalize(), lo, hi);
        assert!((p.y - (w.y - EDGE)).abs() < 1e-3 && p.x < w.x - EDGE, "{p}");
    }

    #[test]
    fn the_arrow_points_at_a_part_in_view() {
        let w = Vec2::new(800.0, 600.0);
        // Right of the centre: the arrow comes from the left, its tip just short of the part.
        let (tip, dir) = arrow_for(Some(Vec2::new(700.0, 300.0)), Vec2::ZERO, w);
        assert_eq!(dir, Vec2::X);
        assert_eq!(tip, Vec2::new(700.0 - GAP, 300.0));
        // Off the window to the left (or behind, to the left): on the left edge.
        let (tip, dir) = arrow_for(None, Vec2::new(-2.0, 0.0), w);
        assert_eq!((tip, dir), (Vec2::new(EDGE, 300.0), -Vec2::X));
    }

    fn ctx<'a>(a: &'a Aircraft, hud: &'a HudState, mfd: &'a MfdState) -> Ctx<'a> {
        Ctx { a, hud, mfd, obj: default(), fire: false, seen: &[], dwell: 0.0 }
    }

    #[test]
    fn every_step_says_what_to_do() {
        for s in STEPS {
            assert!(!s.title.is_empty() && !s.detail.is_empty(), "{}", s.title);
        }
        // The last one stays up (until it times out), whatever the flight.
        let (a, hud, mfd) =
            (Aircraft::new_on_runway(0.0), HudState::default(), MfdState::default());
        let last = STEPS.last().unwrap();
        assert!(last.only == Only::All && !(last.done)(&ctx(&a, &hud, &mfd)));
    }

    /// Every flight takes off, flies and lands: a step for each.
    #[test]
    fn every_flight_has_a_takeoff_and_a_landing() {
        for helicopter in [false, true] {
            for (bomb_bay, armed, air) in
                [(true, true, false), (false, true, true), (false, false, false)]
            {
                let f = Flight { helicopter, bomb_bay, armed, air };
                let steps: Vec<_> = STEPS.iter().filter(|s| s.only.fits(f)).collect();
                for phase in [TAKEOFF, NAVIGATION, RETURN, LANDING] {
                    assert!(steps.iter().any(|s| s.phase == phase), "{f:?}: {phase}");
                }
                assert_eq!(steps.iter().any(|s| s.phase == ATTACK), armed, "{f:?}");
            }
        }
    }

    /// The pages read the same with the classic and the modern avionics (the modern: formats).
    #[test]
    fn pages_follow_the_avionics() {
        let (a, hud) = (Aircraft::new_on_runway(0.0), HudState::default());
        let mut mfd = MfdState { modern: false, ..default() };
        let page = |mfd: &MfdState, p| ctx(&a, &hud, mfd).page(p);
        // Classic: the map on the left, the orders on the right.
        assert_eq!(page(&mfd, Page::Map), Some(MfdSide::Left));
        assert_eq!(page(&mfd, Page::Tactical), None);
        assert_eq!(page(&mfd, Page::Orders), Some(MfdSide::Right));
        // Modern: the HSD on the left, the SMS's orders on the right.
        mfd.modern = true;
        assert_eq!(page(&mfd, Page::Tactical), Some(MfdSide::Left));
        assert_eq!(page(&mfd, Page::Map), None);
        assert_eq!(page(&mfd, Page::Orders), Some(MfdSide::Right));
        assert_eq!(page(&mfd, Page::Weapons), None);
        let button = |mfd: &MfdState, p| ctx(&a, &hud, mfd).page_button(p);
        assert_eq!(button(&mfd, Page::Weapons), Target::Cmd(MfdCmd::Sms(false)));
        assert_eq!(
            button(&mfd, Page::Map),
            Target::Cmd(MfdCmd::Format(MfdSide::Left, Format::Map))
        );
        // Every button a modern page step points at is on the MFDs.
        for p in [Page::Orders, Page::Weapons, Page::Waypoints, Page::Tactical, Page::Map] {
            let Target::Cmd(cmd) = button(&mfd, p) else { panic!("{p:?}") };
            let on_mfd = [MfdSide::Left, MfdSide::Right].into_iter().any(|side| {
                (0..crate::mfd::OSB_COUNT)
                    .any(|slot| osb(side, slot, &mfd).is_some_and(|o| o.1 == OsbFn::Cmd(cmd)))
            });
            assert!(on_mfd, "{p:?}");
        }
    }

    /// The panel buttons the jet steps point at are in the jets' cockpits (else only the box
    /// shows).
    #[test]
    fn step_buttons_are_in_the_cockpits() {
        use super::super::layout::{PanelLayout, PanelStyle};
        let mfd = MfdState::default();
        for (style, bay) in
            [(PanelStyle::Classic, true), (PanelStyle::Modern, true), (PanelStyle::F16, false)]
        {
            let effects: Vec<Effect> =
                PanelLayout::for_style(style).items.iter().filter_map(|p| p.cap).collect();
            let wanted: &[Action] = if bay {
                &[Action::Flaps, Action::Autopilot, Action::BayDoors]
            } else {
                &[Action::Flaps, Action::Autopilot]
            };
            for &a in wanted {
                assert!(effects.iter().any(|e| e.action(&mfd) == Some(a)), "{style:?}: {a:?}");
            }
        }
    }
}

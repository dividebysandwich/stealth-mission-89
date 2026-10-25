//! Interactive tutorial (single missions, any theater and mission): one step at a time, from
//! the takeoff to the landing, a box says what to do next and which keys do it, an arrow points
//! at the cockpit part to press or look at, and that part blinks sky blue.
//!
//! The lesson follows the mission: the departure, each objective in the route's order (the
//! primary target, then the secondary), the return. An objective's steps lead there (the
//! autopilot again after the first one) and through what it asks for: destroy a target with
//! the store on board that does the most against it (a ship, a radar, a bridge...), shoot an
//! aircraft down with the missile that locks the farthest, photograph it, drop supplies on it,
//! or land there.
//!
//! A step ends when its condition holds (gear up, bay open, GND mode, a page shown...), however
//! it was done: a click in the cockpit, a key, the HOTAS or a VR fingertip; steps already done
//! are skipped. An objective's steps are looked at again from its first one every frame, so a
//! step undone (the bay shut, the lock lost, the store spent) shows again. The steps follow the
//! flight: jets or helicopters, a bomb bay or pylons, the classic or the modern avionics (whose
//! MFD pages are formats).
//!
//! The arrow points at the part while the view shows it; otherwise it sticks to the window's
//! edge on the part's side, the box following it. Outside the cockpit views the box asks for the
//! cockpit view. The blinking overlay is the part's own meshes drawn again over everything
//! (`OnTop`), so it hugs their shape (and shows in the GL headset too); the box and the arrow
//! are desktop only for now.
//!
//! Every text is in the steps, `why` and `what` (to translate them one day; the aviation terms
//! stay English).

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::MeshAabb;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use stealth_mission::{Mission, MissionKind};
use stealth_sim::{SPECIAL_EQUIPMENT, SimData};

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
use crate::hud::{HudMode as Hud, HudState, Waypoints, map_units};
use crate::input::Action;
use crate::mfd::{CamDir, LeftMfd, MfdCmd, MfdSide, MfdState, OsbFn, RightMfd, osb};
use crate::settings::{AppState, Settings};
use crate::world::{WU, WorldPos};

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
/// Seconds a threat's box stays up after it is over (no flicker as the lamps blink).
const ALERT_HOLD: f32 = 3.0;
/// Seconds a warning (a lasting state: fire, low fuel) is shown each time it begins.
const WARNING_SECONDS: f32 = 12.0;
/// The attack begins this near an objective (km), the landing this near the base.
const ATTACK_KM: f32 = 25.0;
const HOME_KM: f32 = 20.0;
/// Map units (32 world units) per km.
const MU_PER_KM: f32 = (1000.0 / (32.0 * WU)) as f32;
/// The camera's index in the weapon table (sim.json `weapons`).
const CAMERA: i32 = 16;
/// The `compat` row of gunfire (the simulation's rounds hitting the ground).
const GUNFIRE: i32 = 18;

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
    /// The tracking camera (the modern targeting pod).
    Camera,
}

/// What an objective asks for.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Aim {
    /// Destroy something on the ground or at sea.
    Ground,
    /// Shoot down aircraft: the special aircraft, a patrol, the drones.
    Air,
    /// Photograph it (photo recon).
    Photo,
    /// Drop the supplies over its radio beacon.
    Drop,
    /// Land there: a secret airstrip, the troops' landing zone (helicopter).
    Land,
}

impl Aim {
    /// What a mission target asks for (None: no target).
    fn of(t: &stealth_mission::Target, kind: MissionKind) -> Option<Aim> {
        use stealth_mission::{drones::DRONE_INTERCEPT, insertion::INSERT};
        Some(match t.mission_type {
            0 => return None,
            1 => Aim::Photo,
            3 => Aim::Drop,
            4 => Aim::Land,
            5..=8 => Aim::Air,
            DRONE_INTERCEPT if kind == MissionKind::DroneDefense => Aim::Air,
            INSERT => Aim::Land,
            _ => Aim::Ground,
        })
    }
}

/// An objective of the mission, as the steps need it.
#[derive(Debug, Clone)]
struct Objective {
    /// 0 primary, 1 secondary.
    k: usize,
    aim: Aim,
    /// What it is ("Tank Farm at Sunan", "the An-72").
    name: String,
    /// What it is to the weapons (the `compat` column).
    class: u8,
    /// Where it is (map units, y south), or the flight unit to shoot down.
    at: IVec2,
    unit: Option<usize>,
    /// It is over at the deadline.
    deadline: bool,
}

/// The mission's objectives, the primary first.
fn objectives(m: &Mission) -> Vec<Objective> {
    [&m.primary, &m.secondary]
        .into_iter()
        .enumerate()
        .filter_map(|(k, t)| {
            let aim = Aim::of(t, m.settings.kind)?;
            // The special aircraft is an air mission's primary.
            let special =
                m.special_aircraft.as_ref().filter(|_| k == 0 && (5..=8).contains(&t.mission_type));
            let name = match special {
                Some(s) => format!("the {}", s.plane_name),
                None if aim == Aim::Air => "the attack drones".into(),
                None if t.place_name.is_empty() => t.shape_name.clone(),
                None => format!("{} at {}", t.shape_name, t.place_name),
            };
            Some(Objective {
                k,
                aim,
                name,
                class: t.weapon_class,
                at: IVec2::new(t.position.x, t.position.y),
                unit: special.map(|s| s.flight_unit),
                deadline: t.has_deadline,
            })
        })
        .collect()
}

/// A store to fire: a weapon (its index in the weapon table), or the gun.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Store {
    Weapon(i32),
    Gun,
}

/// The store to use on an objective: the camera, the supplies, the air-to-air missile that
/// locks the farthest, or the weapon that does the most against the target (`compat`); else
/// the gun when it does something (None: nothing carried fits, or a landing). `carried`: the
/// weapons on board and their rounds.
fn best_store(
    aim: Aim,
    class: u8,
    carried: &[(i32, i32)],
    gun: bool,
    data: &SimData,
) -> Option<Store> {
    let carried = || carried.iter().filter(|&&(_, n)| n > 0).map(|&(w, _)| w);
    let reach = |w: i32| {
        let m = data.missile(data.weapon(w).spec as i32);
        (data.weapon(w).spec > 0 && m.class == 7).then_some(m.lock_range)
    };
    let gun = gun.then_some(Store::Gun);
    match aim {
        Aim::Land => None,
        Aim::Photo => carried().find(|&w| w == CAMERA).map(Store::Weapon),
        Aim::Drop => carried().find(|&w| w == SPECIAL_EQUIPMENT).map(Store::Weapon),
        Aim::Air => carried()
            .filter_map(|w| Some((w, reach(w)?)))
            .min_by_key(|&(_, r)| std::cmp::Reverse(r))
            .map(|(w, _)| Store::Weapon(w))
            .or(gun),
        Aim::Ground => carried()
            .filter(|&w| w != SPECIAL_EQUIPMENT)
            .map(|w| (w, data.compat_cat(w, class.into())))
            .filter(|&(_, c)| c > 0)
            .min_by_key(|&(_, c)| std::cmp::Reverse(c))
            .map(|(w, _)| Store::Weapon(w))
            .or(gun.filter(|_| data.compat_cat(GUNFIRE, class.into()) > 0)),
    }
}

/// A store's name in the steps ("Harpoon (AGM-84A)").
fn store_name(store: Option<Store>, data: &SimData) -> String {
    match store {
        None => "selected weapon".into(),
        Some(Store::Gun) => "gun".into(),
        Some(Store::Weapon(CAMERA)) => "camera".into(),
        Some(Store::Weapon(SPECIAL_EQUIPMENT)) => "supplies".into(),
        Some(Store::Weapon(w)) => {
            let t = data.weapon(w);
            format!("{} ({})", t.long_name, t.short_name)
        }
    }
}

/// Why a store is the one to use (`{what}`: the target's kind).
fn why(aim: Aim, store: Option<Store>) -> &'static str {
    match (aim, store) {
        (Aim::Ground, Some(Store::Weapon(_))) => {
            "Against {what}, the weapon on board that does the most damage"
        }
        (Aim::Ground, Some(Store::Gun)) => {
            "Against {what}, with nothing on board that fits: the gun"
        }
        (Aim::Air, Some(Store::Weapon(_))) => {
            "Against an aircraft, the air-to-air missile on board that locks the farthest"
        }
        (Aim::Air, Some(Store::Gun)) => {
            "Against an aircraft, with no missile left: the gun, close in"
        }
        (Aim::Photo, _) => "The camera takes the photo, with the bay open",
        (Aim::Drop, _) => "The supplies are dropped over the radio beacon",
        _ => "",
    }
}

/// A target's kind by its `compat` column.
fn what(class: u8) -> &'static str {
    match class {
        1 => "an airfield",
        2 => "parked aircraft",
        3 => "buildings",
        4 => "a bridge",
        5 => "a hardened building",
        6 => "a depot",
        7 => "missile launchers",
        8 => "a radar",
        9 => "an oil rig",
        10 => "an industrial plant",
        11 => "a submarine pen",
        12 => "a ship",
        13 => "a large ship",
        14 => "tanks",
        _ => "the target",
    }
}

/// The part of the flight a step is in.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Leg {
    Departure,
    /// Index in `Tutorial::objectives`.
    Objective(usize),
    Return,
}

/// The flight the steps are chosen for.
#[derive(Debug, Clone, Copy)]
struct Flight {
    helicopter: bool,
    bomb_bay: bool,
    /// What the leg's objective asks for (None: the departure, the return).
    aim: Option<Aim>,
}

/// The flights a step is for.
fn all(_: Flight) -> bool {
    true
}
fn jets(f: Flight) -> bool {
    !f.helicopter
}
fn helicopters(f: Flight) -> bool {
    f.helicopter
}
/// An objective a store is fired at.
fn stores(f: Flight) -> bool {
    f.aim.is_some_and(|a| a != Aim::Land)
}
fn bay(f: Flight) -> bool {
    f.bomb_bay && stores(f)
}
/// An objective on the ground to lock: to destroy or to photograph.
fn ground(f: Flight) -> bool {
    matches!(f.aim, Some(Aim::Ground | Aim::Photo))
}
/// An objective needing the HUD's attack mode.
fn attack(f: Flight) -> bool {
    matches!(f.aim, Some(Aim::Ground | Aim::Photo | Aim::Air))
}
fn destroy(f: Flight) -> bool {
    f.aim == Some(Aim::Ground)
}
fn air(f: Flight) -> bool {
    f.aim == Some(Aim::Air)
}
fn photo(f: Flight) -> bool {
    f.aim == Some(Aim::Photo)
}
fn supplies(f: Flight) -> bool {
    f.aim == Some(Aim::Drop)
}
fn land_jets(f: Flight) -> bool {
    f.aim == Some(Aim::Land) && !f.helicopter
}
fn land_helicopters(f: Flight) -> bool {
    f.aim == Some(Aim::Land) && f.helicopter
}

/// How a leg stands this frame.
#[derive(Debug, Clone, Copy, Default)]
struct LegState {
    /// Its objective is done, or can no longer be (the time is up, the aircraft escaped).
    over: bool,
    /// Near enough to attack (to land, going home).
    near: bool,
    /// The current waypoint leads there; is past it (a waypoint back leads there).
    on_route: bool,
    past: bool,
    /// Its target is locked (the aircraft to shoot down; any ground lock).
    locked: bool,
    /// The store to use there.
    best: Option<Store>,
}

/// What the steps read.
struct Ctx<'a> {
    a: &'a Aircraft,
    hud: &'a HudState,
    mfd: &'a MfdState,
    /// Seconds on the step.
    dwell: f32,
    /// The step's leg.
    leg: LegState,
    /// The selected store (None: nothing on board).
    selected: Option<Store>,
    /// The threat and countermeasure lamps, the flares, chaff and decoys left.
    lights: stealth_sim::Lights,
    stores: (i32, i32, i32),
    /// A radar tracks the aircraft.
    tracked: bool,
    /// A fire burns on board.
    fire: bool,
}

impl Ctx<'_> {
    fn knots(&self) -> f32 {
        self.a.velocity / 27.0
    }

    /// The leg's store is selected (or there is none to select).
    fn best_selected(&self) -> bool {
        self.leg.best.is_none_or(|b| self.selected == Some(b))
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
                Page::Map => av.shown(Format::Map).filter(|_| !av.map_way),
                Page::Camera => av.shown(Format::Tgp),
            };
        }
        let (side, shown) = match page {
            Page::Orders => (MfdSide::Right, m.right == RightMfd::Ord),
            Page::Weapons => (MfdSide::Right, m.right == RightMfd::Wpn),
            Page::Waypoints => (MfdSide::Right, m.right == RightMfd::Way),
            Page::Tactical => (MfdSide::Left, m.left == LeftMfd::Tac),
            Page::Map => (MfdSide::Left, m.left == LeftMfd::Map),
            Page::Camera => (MfdSide::Right, m.right == RightMfd::Camera),
        };
        shown.then_some(side)
    }

    /// The button that shows a page: with the modern avionics the format's own button when the
    /// format shows (ORD, WPN, WAY; WAY again back to the map), else the format's button.
    fn page_button(&self, page: Page) -> Target {
        if !self.mfd.modern {
            return match page {
                Page::Orders => Target::Action(Action::MissionOrders),
                Page::Weapons => Target::Action(Action::Ordnance),
                Page::Waypoints => Target::Action(Action::SelectWaypoint),
                Page::Tactical => Target::Effect(Effect::LeftPage(LeftMfd::Tac)),
                Page::Map => Target::Effect(Effect::LeftPage(LeftMfd::Map)),
                Page::Camera => Target::Action(Action::CamAhead),
            };
        }
        let (format, own, side) = match page {
            Page::Orders => (Format::Sms, Some(MfdCmd::Sms(true)), MfdSide::Right),
            Page::Weapons => (Format::Sms, Some(MfdCmd::Sms(false)), MfdSide::Right),
            Page::Waypoints => (Format::Map, Some(MfdCmd::MapWay), MfdSide::Right),
            Page::Tactical => (Format::Hsd, None, MfdSide::Left),
            Page::Map => (Format::Map, Some(MfdCmd::MapWay), MfdSide::Left),
            Page::Camera => (Format::Tgp, None, MfdSide::Right),
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

    /// The display showing the way: the TAC page, else the map.
    fn way_display(&self) -> Target {
        match self.page_display(Page::Tactical) {
            Target::None => self.page_display(Page::Map),
            t => t,
        }
    }
}

struct Step {
    only: fn(Flight) -> bool,
    phase: &'static str,
    /// `{target}` (`{TARGET}`) is the leg's objective, `{name}` what it is, `{weapon}` the
    /// store to use, `{why}` why that one, `{what}` the target's kind.
    title: &'static str,
    detail: &'static str,
    keys: &'static [Action],
    target: fn(&Ctx) -> Target,
    done: fn(&Ctx) -> bool,
}

const TAKEOFF: &str = "TAKEOFF";
const NAVIGATION: &str = "NAVIGATION";
const OBJECTIVE: &str = "{TARGET}";
const RETURN: &str = "RETURN";
const LANDING: &str = "LANDING";

/// From the runway to the first objective's route.
static DEPARTURE: &[Step] = {
    use Action::*;
    use Target as T;
    &[
        Step {
            only: jets,
            phase: TAKEOFF,
            title: "Flaps down",
            detail: "Still stopped, before the takeoff run: more lift, a takeoff at a lower speed",
            keys: &[Flaps],
            target: |_| T::Action(Flaps),
            done: |c| c.a.flaps || !c.a.on_ground,
        },
        Step {
            only: jets,
            phase: TAKEOFF,
            title: "Full throttle",
            detail: "The engine runs: push the throttle to full power for the takeoff run",
            keys: &[ThrottleMax, ThrottleUp],
            target: |_| T::Throttle,
            done: |c| c.a.set_thrust >= 95.0 || !c.a.on_ground,
        },
        Step {
            only: jets,
            phase: TAKEOFF,
            title: "Rotate at about 160 knots",
            detail: "Pull the stick back gently until the aircraft leaves the runway, then climb",
            keys: &[PitchUp],
            target: |_| T::Stick,
            done: |c| !c.a.on_ground,
        },
        Step {
            only: helicopters,
            phase: TAKEOFF,
            title: "Lift off",
            detail: "Raise the collective until the helicopter leaves the ground",
            keys: &[ThrottleUp],
            target: |_| T::Throttle,
            done: |c| !c.a.on_ground,
        },
        Step {
            only: jets,
            phase: TAKEOFF,
            title: "Gear up",
            detail: "Once climbing: the gear out is slow and seen by the radars",
            keys: &[Gear],
            target: |_| T::Action(Gear),
            done: |c| !c.a.gear_down,
        },
        Step {
            only: jets,
            phase: TAKEOFF,
            title: "Flaps up",
            detail: "Less drag for the climb and the cruise",
            keys: &[Flaps],
            target: |_| T::Action(Flaps),
            done: |c| !c.a.flaps,
        },
        Step {
            only: all,
            phase: NAVIGATION,
            title: "Mission orders page",
            detail: "An MFD shows the orders: the primary and secondary targets",
            keys: &[MissionOrders],
            target: |c| c.page_button(Page::Orders),
            done: |c| c.page(Page::Orders).is_some(),
        },
        Step {
            only: all,
            phase: NAVIGATION,
            title: "Read your orders",
            detail: "The primary target first, then the secondary, then the base to land at",
            keys: &[],
            target: |c| c.page_display(Page::Orders),
            done: |c| c.dwell >= 8.0,
        },
        Step {
            only: all,
            phase: NAVIGATION,
            title: "Tactical page",
            detail: "The threats and the targets around the aircraft",
            keys: &[Maps],
            target: |c| c.page_button(Page::Tactical),
            done: |c| c.page(Page::Tactical).is_some(),
        },
        Step {
            only: all,
            phase: NAVIGATION,
            title: "Map page",
            detail: "The route, the waypoints and the enemy radars",
            keys: &[Maps],
            target: |c| c.page_button(Page::Map),
            done: |c| c.page(Page::Map).is_some(),
        },
        Step {
            only: all,
            phase: NAVIGATION,
            title: "Waypoint page",
            detail: "The current waypoint, its bearing and distance",
            keys: &[SelectWaypoint],
            target: |c| c.page_button(Page::Waypoints),
            done: |c| c.page(Page::Waypoints).is_some(),
        },
    ]
};

/// An objective, from its route to the way out: looked at again from the first step every
/// frame (`over`: done, or no longer possible, ends the steps up to the way out).
static OBJECTIVE_STEPS: &[Step] = {
    use Action::*;
    use Target as T;
    &[
        Step {
            only: all,
            phase: OBJECTIVE,
            title: "Waypoint to the {target}",
            detail: "The HUD and the autopilot steer to the current waypoint: step to the one \
                     leading to {name}",
            keys: &[NextWaypoint, LastWaypoint],
            target: |c| T::Action(if c.leg.past { LastWaypoint } else { NextWaypoint }),
            done: |c| c.leg.on_route || c.leg.near || c.leg.over,
        },
        Step {
            only: jets,
            phase: OBJECTIVE,
            title: "Autopilot to the {target}",
            detail: "Flies the waypoints to {name}; any stick input takes the controls back",
            keys: &[Autopilot],
            target: |_| T::Action(Autopilot),
            done: |c| {
                c.a.autopilot_alt > 0.0 || c.a.damage.autopilot_failed || c.leg.near || c.leg.over
            },
        },
        Step {
            only: jets,
            phase: OBJECTIVE,
            title: "Autopilot out of order",
            detail: "Avionics damage: fly by hand to {name}, the HUD's \
                     steerpoint diamond shows the way",
            keys: &[],
            target: |_| T::Stick,
            done: |c| !c.a.damage.autopilot_failed || c.leg.near || c.leg.over,
        },
        Step {
            only: all,
            phase: OBJECTIVE,
            title: "Fly to the {target}",
            detail: "Towards {name}, until about 25 km from it: the tactical page shows the way. \
                     Tip: accelerate time for the cruise (it stops by itself on a threat)",
            keys: &[TimeAccel],
            target: |c| c.way_display(),
            done: |c| c.leg.near || c.leg.over,
        },
        Step {
            only: ground,
            phase: OBJECTIVE,
            title: "Air-to-ground mode",
            detail: "GND mode: the HUD shows the ground targets and the weapon cues",
            keys: &[HudMode],
            target: |_| T::Effect(Effect::HudMode(2)),
            done: |c| c.hud.mode == Hud::Gnd || c.leg.over,
        },
        Step {
            only: air,
            phase: OBJECTIVE,
            title: "Air-to-air mode",
            detail: "AIR mode: the HUD shows the aircraft and the missile cues",
            keys: &[HudMode],
            target: |_| T::Effect(Effect::HudMode(1)),
            done: |c| c.hud.mode == Hud::Air || c.leg.over,
        },
        Step {
            only: stores,
            phase: OBJECTIVE,
            title: "Weapons page",
            detail: "The stores on board and the selected one",
            keys: &[Ordnance],
            target: |c| c.page_button(Page::Weapons),
            done: |c| c.page(Page::Weapons).is_some() || c.best_selected() || c.leg.over,
        },
        Step {
            only: stores,
            phase: OBJECTIVE,
            title: "Select the {weapon}",
            detail: "{why}",
            keys: &[SelectWeapon],
            target: |_| T::Action(SelectWeapon),
            done: |c| c.best_selected() || c.leg.over,
        },
        Step {
            only: bay,
            phase: OBJECTIVE,
            title: "Open the weapon bay",
            detail: "Stores leave through open bays only; open bays are seen by the radars",
            keys: &[BayDoors],
            target: |_| T::Action(BayDoors),
            done: |c| c.a.bay_open || c.leg.best == Some(Store::Gun) || c.leg.over,
        },
        // After the store's steps: the classic camera and weapons pages share the right MFD.
        Step {
            only: all,
            phase: OBJECTIVE,
            title: "Target camera",
            detail: "The tracking camera, ahead: the MFD shows {name} as you close in (a photo \
                     counts with it in the picture)",
            keys: &[CamAhead],
            target: |c| c.page_button(Page::Camera),
            done: |c| {
                c.page(Page::Camera).is_some() && (c.mfd.modern || c.mfd.cam_dir == CamDir::Ahead)
                    || c.leg.over
            },
        },
        Step {
            only: ground,
            phase: OBJECTIVE,
            title: "Designate the {target}",
            detail: "Lock {name}: the HUD's box frames it",
            keys: &[DesignateTarget, NearestTarget, SelectTarget],
            target: |_| T::Action(DesignateTarget),
            done: |c| c.leg.locked || c.leg.over,
        },
        Step {
            only: air,
            phase: OBJECTIVE,
            title: "Lock the {target}",
            detail: "Lock {name}, the nearest aircraft first: the HUD's box frames it",
            keys: &[NearestTarget, SelectTarget],
            target: |_| T::Action(NearestTarget),
            done: |c| c.leg.locked || c.leg.over,
        },
        Step {
            only: destroy,
            phase: OBJECTIVE,
            title: "Fire the {weapon}",
            detail: "In range (the HUD's cue), release it; again until {name} is destroyed",
            keys: &[FireWeapon],
            target: |_| T::Stick,
            done: |c| c.leg.over,
        },
        Step {
            only: air,
            phase: OBJECTIVE,
            title: "Fire the {weapon}",
            detail: "In range (the HUD's cue), launch it; again until {name} goes down",
            keys: &[FireWeapon],
            target: |_| T::Stick,
            done: |c| c.leg.over,
        },
        Step {
            only: photo,
            phase: OBJECTIVE,
            title: "Take the photo",
            detail: "Nose on {name}, close enough (the HUD's cue): the trigger takes a frame",
            keys: &[FireWeapon],
            target: |_| T::Stick,
            done: |c| c.leg.over,
        },
        Step {
            only: supplies,
            phase: OBJECTIVE,
            title: "Drop the supplies",
            detail: "Over the radio beacon, when the HUD says DROP SUPPLIES NOW: the trigger",
            keys: &[FireWeapon],
            target: |_| T::Stick,
            done: |c| c.leg.over,
        },
        Step {
            only: land_jets,
            phase: OBJECTIVE,
            title: "Gear down",
            detail: "For the landing at {name}",
            keys: &[Gear],
            target: |_| T::Action(Gear),
            done: |c| c.a.gear_down || c.leg.over,
        },
        Step {
            only: land_jets,
            phase: OBJECTIVE,
            title: "Land at the {target}",
            detail: "Touch down at {name} and stop until the cargo is unloaded",
            keys: &[Brakes],
            target: |_| T::Display(CanvasRole::Approach),
            done: |c| c.leg.over,
        },
        Step {
            only: land_helicopters,
            phase: OBJECTIVE,
            title: "Land at the {target}",
            detail: "Set down within 500 m of {name} and stay until the troops are out",
            keys: &[ThrottleDown],
            target: |_| T::Throttle,
            done: |c| c.leg.over,
        },
        Step {
            only: land_jets,
            phase: OBJECTIVE,
            title: "Take off again",
            detail: "Flaps down, then full throttle, rotate at about 160 knots and climb",
            keys: &[Flaps, ThrottleMax, PitchUp],
            target: |c| if c.a.flaps { T::Throttle } else { T::Action(Flaps) },
            done: |c| !c.a.on_ground,
        },
        Step {
            only: land_helicopters,
            phase: OBJECTIVE,
            title: "Lift off",
            detail: "Raise the collective: on to the next leg",
            keys: &[ThrottleUp],
            target: |_| T::Throttle,
            done: |c| !c.a.on_ground,
        },
        Step {
            only: land_jets,
            phase: OBJECTIVE,
            title: "Gear up",
            detail: "Once climbing: the gear out is slow and seen by the radars",
            keys: &[Gear],
            target: |_| T::Action(Gear),
            done: |c| !c.a.gear_down,
        },
        Step {
            only: bay,
            phase: OBJECTIVE,
            title: "Close the weapon bay",
            detail: "Back to stealth: open bays are seen by the radars",
            keys: &[BayDoors],
            target: |_| T::Action(BayDoors),
            done: |c| !c.a.bay_open,
        },
        Step {
            only: attack,
            phase: OBJECTIVE,
            title: "Navigation mode",
            detail: "NAV mode: the HUD steers to the next waypoint",
            keys: &[HudMode],
            target: |_| T::Effect(Effect::HudMode(0)),
            done: |c| c.hud.mode == Hud::Nav,
        },
    ]
};

/// From the last objective to the stop after landing (`{name}`: the landing base).
static RETURN_STEPS: &[Step] = {
    use Action::*;
    use Target as T;
    &[
        Step {
            only: all,
            phase: RETURN,
            title: "Waypoint to the landing base",
            detail: "The HUD and the autopilot steer to the current waypoint: step to the one \
                     leading to {name}",
            keys: &[NextWaypoint, LastWaypoint],
            target: |_| T::Action(NextWaypoint),
            done: |c| c.leg.on_route || c.leg.near,
        },
        Step {
            only: jets,
            phase: RETURN,
            title: "Autopilot to the landing base",
            detail: "Flies the waypoints to {name}; any stick input takes the controls back",
            keys: &[Autopilot],
            target: |_| T::Action(Autopilot),
            done: |c| c.a.autopilot_alt > 0.0 || c.a.damage.autopilot_failed || c.leg.near,
        },
        Step {
            only: jets,
            phase: RETURN,
            title: "Autopilot out of order",
            detail: "Avionics damage: fly by hand to {name}, the HUD's \
                     steerpoint diamond shows the way",
            keys: &[],
            target: |_| T::Stick,
            done: |c| !c.a.damage.autopilot_failed || c.leg.near,
        },
        Step {
            only: all,
            phase: RETURN,
            title: "Fly to the landing base",
            detail: "Towards {name}, until about 20 km from it: the tactical page shows the way. \
                     Tip: accelerate time for the cruise (it stops by itself on a threat)",
            keys: &[TimeAccel],
            target: |c| c.way_display(),
            done: |c| c.leg.near,
        },
        Step {
            only: all,
            phase: LANDING,
            title: "ILS",
            detail: "Instrument landing system: the HUD steering bars lead to the runway",
            keys: &[Ils],
            target: |_| T::Action(Ils),
            done: |c| c.mfd.ils,
        },
        Step {
            only: jets,
            phase: LANDING,
            title: "Reduce the throttle",
            detail: "Slow down for the approach, below 250 knots",
            keys: &[ThrottleDown, ThrottleIdle],
            target: |_| T::Throttle,
            done: |c| c.a.set_thrust <= 60.0,
        },
        Step {
            only: jets,
            phase: LANDING,
            title: "Gear down",
            detail: "Three green lamps when it is down",
            keys: &[Gear],
            target: |_| T::Action(Gear),
            done: |c| c.a.gear_down,
        },
        Step {
            only: jets,
            phase: LANDING,
            title: "Flaps down",
            detail: "A lower approach speed",
            keys: &[Flaps],
            target: |_| T::Action(Flaps),
            done: |c| c.a.flaps,
        },
        Step {
            only: jets,
            phase: LANDING,
            title: "Touch down",
            detail: "Keep the green circle (on speed) and a gentle descent to the runway",
            keys: &[],
            target: |_| T::Display(CanvasRole::Approach),
            done: |c| c.a.on_ground && c.a.has_flown,
        },
        Step {
            only: helicopters,
            phase: LANDING,
            title: "Land",
            detail: "Slow to a hover over {name}, then lower the collective gently",
            keys: &[ThrottleDown],
            target: |_| T::Throttle,
            done: |c| c.a.on_ground && c.a.has_flown,
        },
        Step {
            only: jets,
            phase: LANDING,
            title: "Brakes",
            detail: "Throttle to idle and brake to a stop",
            keys: &[Brakes, ThrottleIdle],
            target: |_| T::Action(Brakes),
            done: |c| c.a.on_ground && (c.a.brakes || c.knots() < 5.0),
        },
        Step {
            only: all,
            phase: LANDING,
            title: "Mission complete",
            detail: "End of the tutorial. The in-flight menu switches it off",
            keys: &[Menu],
            target: |_| T::None,
            done: |_| false,
        },
    ]
};

const DANGER: &str = "DANGER";
const THREAT: &str = "THREAT";
/// A lasting state: shown for `WARNING_SECONDS` each time it begins, then the lesson again.
const WARNING: &str = "WARNING";

/// The dangers, threats and warnings, shown before the lesson's step while they last (`done`:
/// over), the first one that applies. A stick input to evade takes the autopilot off: the
/// lesson asks for it again afterwards.
static ALERTS: &[Step] = {
    use Action::*;
    use Target as T;
    &[
        Step {
            only: all,
            phase: DANGER,
            title: "Stall",
            detail: "Too slow to fly: lower the nose and add power until the STALL light goes out",
            keys: &[PitchDown, ThrottleMax],
            target: |_| T::Throttle,
            done: |c| c.a.on_ground || c.a.velocity >= c.a.stall_speed,
        },
        Step {
            only: all,
            phase: THREAT,
            title: "Radar-guided missile: chaff",
            detail: "Release chaff, then turn hard away from the missile; the ECM jams it from \
                     far enough, a decoy draws it away",
            keys: &[Chaff, Ecm, Decoy],
            target: |_| T::Action(Chaff),
            done: |c| !c.lights.rad_threat || c.lights.chaff || c.stores.1 == 0,
        },
        Step {
            only: all,
            phase: THREAT,
            title: "Infrared missile: flares",
            detail: "Release flares, then turn hard away from the missile; the IR jammer jams it \
                     from far enough, a decoy draws it away",
            keys: &[Flare, IrJammer, Decoy],
            target: |_| T::Action(Flare),
            done: |c| !c.lights.ir_threat || c.lights.flare || c.stores.0 == 0,
        },
        Step {
            only: all,
            phase: THREAT,
            title: "Evade the missile",
            detail: "Turn hard away and dive: once its smoke trail ends its motor is out and it \
                     slows down; a decoy draws it away",
            keys: &[Decoy],
            target: |c| {
                if c.stores.2 > 0 && !c.lights.decoy { T::Action(Decoy) } else { T::Stick }
            },
            done: |c| !c.lights.rad_threat && !c.lights.ir_threat,
        },
        Step {
            only: all,
            phase: WARNING,
            title: "Fire on board",
            detail: "It drains fuel while the throttle is above idle: throttle back as far as you \
                     can and head home; with realistic damage it may blow up: be ready to eject",
            keys: &[ThrottleIdle, Eject],
            target: |_| T::Throttle,
            done: |c| !c.fire,
        },
        Step {
            only: all,
            phase: THREAT,
            title: "Tracked by a radar",
            detail: "Descend, slow down, keep the wings level and the bay shut until the radar \
                     loses you (EMV and signal bars apart); the tactical page shows its circle",
            keys: &[ThrottleDown],
            target: |c| match c.page_display(Page::Tactical) {
                T::None => T::Display(CanvasRole::Rwr),
                t => t,
            },
            done: |c| !c.tracked || c.leg.near,
        },
        Step {
            only: all,
            phase: WARNING,
            title: "Low fuel",
            detail: "Under 1,500 lb: leave what is left of the mission and head for the landing \
                     base, the waypoints after the targets lead there",
            keys: &[NextWaypoint],
            target: |_| T::Action(NextWaypoint),
            done: |c| c.a.fuel >= crate::panel::LOW_FUEL || c.a.on_ground,
        },
    ]
};

/// The alert to show (index in `ALERTS`): the first that applies, a warning only for its first
/// `WARNING_SECONDS`. `since`: when each alert began to apply, kept up to date.
fn pick_alert(ctx: &Ctx, since: &mut Vec<Option<f32>>, now: f32) -> Option<usize> {
    since.resize(ALERTS.len(), None);
    let mut pick = None;
    for (i, (s, since)) in ALERTS.iter().zip(since.iter_mut()).enumerate() {
        if (s.done)(ctx) {
            *since = None;
            continue;
        }
        let begun = *since.get_or_insert(now);
        if pick.is_none() && (s.phase != WARNING || now - begun < WARNING_SECONDS) {
            pick = Some(i);
        }
    }
    pick
}

/// A step of the lesson, and its leg.
#[derive(Clone, Copy)]
struct Entry {
    step: &'static Step,
    leg: Leg,
}

/// The lesson for a flight: the departure, each objective (`aims`, in the route's order), the
/// return; only the steps for that flight.
fn lesson(f: Flight, aims: &[Aim]) -> Vec<Entry> {
    let part = |steps: &'static [Step], f: Flight, leg: Leg| {
        steps.iter().filter(move |s| (s.only)(f)).map(move |step| Entry { step, leg })
    };
    let mut plan: Vec<Entry> = part(DEPARTURE, f, Leg::Departure).collect();
    for (i, &aim) in aims.iter().enumerate() {
        plan.extend(part(OBJECTIVE_STEPS, Flight { aim: Some(aim), ..f }, Leg::Objective(i)));
    }
    plan.extend(part(RETURN_STEPS, f, Leg::Return));
    plan
}

/// The leg the current waypoint is on: 0 to the primary target, 1 to the secondary, 2 home.
fn route_leg(w: &Waypoints) -> usize {
    let a = |k: usize| w.anchors.get(k).copied().unwrap_or(k);
    if w.current <= a(1) {
        0
    } else if w.current <= a(2) {
        1
    } else {
        2
    }
}

/// The words of a step's leg.
struct Words<'a> {
    /// The objective ("primary target"), what it is, the store to use and why.
    target: &'a str,
    name: &'a str,
    weapon: String,
    why: &'static str,
    what: &'static str,
}

/// A step's text with its leg's words.
fn fill(text: &str, w: &Words) -> String {
    if !text.contains('{') {
        return text.into();
    }
    text.replace("{why}", w.why)
        .replace("{target}", w.target)
        .replace("{TARGET}", &w.target.to_uppercase())
        .replace("{name}", w.name)
        .replace("{weapon}", &w.weapon)
        .replace("{what}", w.what)
}

/// Where the lesson is.
#[derive(Resource, Default)]
pub(super) struct Tutorial {
    /// Switched on, in a single mission.
    on: bool,
    /// The flight's lesson (made once the flight runs), its objectives and its landing base
    /// (map units, name).
    plan: Vec<Entry>,
    objectives: Vec<Objective>,
    home: (IVec2, String),
    /// Index in `plan`.
    at: usize,
    /// Seconds on the step (not while the menu is open).
    dwell: f32,
    /// The alert shown (index in `ALERTS`) and when it last applied, when each began to apply
    /// (seconds).
    alert: Option<(usize, f32)>,
    alert_since: Vec<Option<f32>>,
    /// The step's phase, title and detail with their words, and what they were filled for
    /// (the threat, the step, the store it names).
    texts: [String; 3],
    filled: Option<(Option<usize>, usize, Option<Store>)>,
    /// What the step shows, and the cockpit part that is.
    target: Target,
    part: Option<Entity>,
    /// The part blinking, and its overlays.
    glowing: Option<Entity>,
    glows: Vec<Entity>,
}

impl Tutorial {
    /// The threat shown, else the lesson's step.
    fn current(&self) -> Option<&'static Step> {
        match self.alert {
            Some((i, _)) => ALERTS.get(i),
            None => self.plan.get(self.at).map(|e| e.step),
        }
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

/// Moves to the first step not done (an objective's steps looked at again from its first one),
/// puts a threat before it while it lasts, says what that shows and fills its texts. The lesson
/// is made once the flight runs.
#[allow(clippy::too_many_arguments)]
pub(super) fn advance(
    time: Res<Time<Real>>,
    settings: Res<Settings>,
    menu: Res<crate::flight_menu::FlightMenu>,
    mission: Option<Res<crate::mission::ActiveMission>>,
    mut tutorial: ResMut<Tutorial>,
    aircraft: Query<(&Aircraft, &WorldPos), With<Player>>,
    (hud, mfd, wps): (Res<HudState>, Res<MfdState>, Res<Waypoints>),
    combat: Option<Res<crate::combat::Combat>>,
    (career, campaign): (
        Option<Res<crate::career::CareerFlight>>,
        Option<Res<crate::campaign::ActiveCampaign>>,
    ),
) {
    let tutorial = &mut *tutorial;
    tutorial.on =
        settings.tutorial && crate::mission::single_mission(career.as_deref(), campaign.as_deref());
    let (true, Ok((a, pos)), Some(combat)) = (tutorial.on, aircraft.single(), combat) else {
        return;
    };
    if !menu.open {
        tutorial.dwell += time.delta_secs();
    }
    if tutorial.plan.is_empty() {
        if let Some(m) = &mission {
            let m = &m.0;
            tutorial.objectives = objectives(m);
            let p = m.landing.position;
            tutorial.home = (IVec2::new(p.x, p.y), m.landing.name.clone());
        }
        let spec = settings.aircraft.spec();
        let f = Flight { helicopter: spec.helicopter, bomb_bay: spec.bomb_bay, aim: None };
        let aims: Vec<Aim> = tutorial.objectives.iter().map(|o| o.aim).collect();
        tutorial.plan = lesson(f, &aims);
    }

    // How the legs stand.
    let (sim, data) = (&combat.sim, &*combat.data);
    let obj = sim.objectives();
    let (carried, cur) = sim.weapon_groups();
    let selected = if sim.gun_selected() {
        Some(Store::Gun)
    } else {
        cur.map(|i| Store::Weapon(carried[i].0))
    };
    let gun = sim.has_gun() && sim.gun_ammo() > 0;
    let lock = sim.targeting();
    let (lights, stores, tracked) = (sim.lights(), sim.stores(), sim.identification().2 > 0);
    let fire = sim.damage_effects().bits & stealth_sim::dmg::FIRE != 0;
    let me = IVec2::from(map_units(pos));
    let within = |p: IVec2, km: f32| (p - me).as_vec2().length() < km * MU_PER_KM;
    let route = (!wps.points.is_empty()).then(|| route_leg(&wps));
    let leg_state = |leg: Leg| match leg {
        Leg::Departure => LegState::default(),
        Leg::Objective(i) => {
            let o = &tutorial.objectives[i];
            let done = if o.k == 0 { obj.primary_done } else { obj.secondary_done };
            let late = o.deadline && obj.deadline > 0 && obj.tick >= obj.deadline;
            // An aircraft to shoot down is where it flies.
            let at =
                o.unit.and_then(|u| sim.flights().get(u)).map_or(o.at, |u| IVec2::new(u.x, u.y));
            LegState {
                over: done || late || (o.unit.is_some() && obj.target_escaped),
                near: within(at, ATTACK_KM),
                on_route: route.is_none_or(|r| r == o.k),
                past: route.is_some_and(|r| r > o.k),
                locked: match o.aim {
                    Aim::Air => o.unit.map_or(lock.air.is_some(), |u| lock.air == Some(u)),
                    _ => lock.ground.is_some(),
                },
                best: best_store(o.aim, o.class, &carried, gun, data),
            }
        }
        Leg::Return => LegState {
            over: false,
            near: a.on_ground || within(tutorial.home.0, HOME_KM),
            on_route: route.is_none_or(|r| r == 2),
            past: false,
            locked: false,
            best: None,
        },
    };

    // An objective's steps from its first one: a step undone shows again.
    let mut at = match tutorial.plan.get(tutorial.at).map(|e| e.leg) {
        Some(leg @ Leg::Objective(_)) => {
            tutorial.plan.iter().position(|e| e.leg == leg).unwrap_or(tutorial.at)
        }
        _ => tutorial.at,
    };
    let mut state: Option<(Leg, LegState)> = None;
    let mut target = Target::None;
    while let Some(e) = tutorial.plan.get(at) {
        let leg = match state {
            Some((l, s)) if l == e.leg => s,
            _ => {
                let s = leg_state(e.leg);
                state = Some((e.leg, s));
                s
            }
        };
        let ctx = Ctx {
            a,
            hud: &hud,
            mfd: &mfd,
            dwell: if at == tutorial.at { tutorial.dwell } else { 0.0 },
            leg,
            selected,
            lights,
            stores,
            tracked,
            fire,
        };
        if !(e.step.done)(&ctx) {
            target = (e.step.target)(&ctx);
            break;
        }
        at += 1;
    }
    let leg = state.map(|(_, s)| s).unwrap_or_default();
    if at != tutorial.at {
        tutorial.at = at;
        tutorial.dwell = 0.0;
    }

    // An alert first, while it lasts and a moment after.
    let ctx =
        Ctx { a, hud: &hud, mfd: &mfd, dwell: 0.0, leg, selected, lights, stores, tracked, fire };
    let now = time.elapsed_secs();
    match pick_alert(&ctx, &mut tutorial.alert_since, now) {
        Some(i) => tutorial.alert = Some((i, now)),
        None if tutorial.alert.is_some_and(|(_, t)| now - t > ALERT_HOLD) => tutorial.alert = None,
        None => {}
    }
    let alert = tutorial.alert.map(|(i, _)| i);
    tutorial.target = alert.map_or(target, |i| (ALERTS[i].target)(&ctx));

    // The texts, filled again when the step or its store changes.
    let Some(e) = tutorial.plan.get(at) else { return };
    let best = leg.best;
    if tutorial.filled == Some((alert, at, best)) {
        return;
    }
    tutorial.filled = Some((alert, at, best));
    let (objective, name, aim, class) = match e.leg {
        Leg::Objective(i) => {
            let o = &tutorial.objectives[i];
            (["primary target", "secondary target"][o.k], o.name.as_str(), Some(o.aim), o.class)
        }
        _ => ("landing base", tutorial.home.1.as_str(), None, 0),
    };
    let words = Words {
        target: objective,
        name,
        weapon: store_name(best, data),
        why: aim.map_or("", |aim| why(aim, best)),
        what: what(class),
    };
    let s = alert.map_or(e.step, |i| &ALERTS[i]);
    tutorial.texts = [fill(s.phase, &words), fill(s.title, &words), fill(s.detail, &words)];
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
            // The F-16 has no NAV cap: A-A again goes back to NAV.
            Target::Effect(Effect::HudMode(0)) => find(&|e| e == Effect::HudMode(0))
                .or_else(|| find(&|e| e == Effect::HudMode(1)))
                .or_else(|| by_action(Action::HudMode)),
            Target::Effect(effect) => {
                find(&|e| e == effect).or_else(|| by_action(effect.action(mfd)?))
            }
            // The classic panel has no bezel buttons: its cap for the command's key.
            Target::Cmd(cmd) => find(&|e| {
                matches!(e, Effect::Osb(side, slot)
                    if osb(side, slot, mfd).is_some_and(|o| o.1 == OsbFn::Cmd(cmd)))
            })
            .or_else(|| by_action(crate::mfd::cmd_action(cmd))),
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
            && !(tutorial.at + 1 == tutorial.plan.len() && tutorial.dwell > LAST_STEP_SECONDS)
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
    let [phase, title, detail] = &tutorial.texts;
    let (number, count) = (tutorial.at + 1, tutorial.plan.len());
    let header = format!("INTERACTIVE TUTORIAL  {number}/{count}  {phase}");
    let (detail, keys) = if tutorial.part.is_some() && !cockpit_view {
        ("Switch to the cockpit view to see it", keys_text(&[Action::ViewCockpit], &input))
    } else {
        (detail.as_str(), keys_text(step.keys, &input))
    };
    let texts = [header, wrap(title), wrap(detail), wrap(&keys)];
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
        Ctx {
            a,
            hud,
            mfd,
            dwell: 0.0,
            leg: LegState::default(),
            selected: None,
            lights: default(),
            stores: (0, 0, 0),
            tracked: false,
            fire: false,
        }
    }

    const AIMS: [Aim; 5] = [Aim::Ground, Aim::Air, Aim::Photo, Aim::Drop, Aim::Land];

    fn flights() -> impl Iterator<Item = Flight> {
        [(false, true), (false, false), (true, false)]
            .into_iter()
            .map(|(helicopter, bomb_bay)| Flight { helicopter, bomb_bay, aim: None })
    }

    #[test]
    fn every_step_says_what_to_do() {
        for s in DEPARTURE.iter().chain(OBJECTIVE_STEPS).chain(RETURN_STEPS).chain(ALERTS) {
            assert!(!s.title.is_empty() && !s.detail.is_empty(), "{}", s.title);
        }
        // The last one stays up (until it times out), whatever the flight.
        let (a, hud, mfd) =
            (Aircraft::new_on_runway(0.0), HudState::default(), MfdState::default());
        for f in flights() {
            let last = lesson(f, &AIMS).last().unwrap().step;
            assert!(std::ptr::eq(last, RETURN_STEPS.last().unwrap()));
            assert!(!(last.done)(&ctx(&a, &hud, &mfd)));
        }
    }

    /// Every flight takes off, flies to each objective and does what it asks for, comes back
    /// and lands.
    #[test]
    fn every_lesson_flies_each_objective() {
        for f in flights() {
            let plan = lesson(f, &AIMS);
            for phase in [TAKEOFF, NAVIGATION, RETURN, LANDING] {
                assert!(plan.iter().any(|e| e.step.phase == phase), "{f:?}: {phase}");
            }
            for (i, aim) in AIMS.into_iter().enumerate() {
                let steps: Vec<&Step> =
                    plan.iter().filter(|e| e.leg == Leg::Objective(i)).map(|e| e.step).collect();
                // The way there, then a store fired, or a landing.
                assert!(steps[0].title.starts_with("Waypoint to"), "{f:?} {aim:?}");
                let fires = steps.iter().any(|s| s.keys.contains(&Action::FireWeapon));
                assert_eq!(fires, aim != Aim::Land, "{f:?} {aim:?}");
                assert_eq!(steps.iter().any(|s| s.title.starts_with("Land at")), aim == Aim::Land);
                // The target camera once there, after the store's page (the same MFD).
                let at = |t: &str| steps.iter().position(|s| s.title.starts_with(t));
                let camera = at("Target camera").unwrap_or_else(|| panic!("{f:?} {aim:?}"));
                assert!(at("Fly to").unwrap() < camera, "{f:?} {aim:?}");
                assert!(at("Select the").is_none_or(|s| s < camera), "{f:?} {aim:?}");
                assert_eq!(
                    steps.iter().any(|s| s.title == "Open the weapon bay"),
                    f.bomb_bay && aim != Aim::Land,
                    "{f:?} {aim:?}"
                );
            }
            // The objectives in order, each one's steps together.
            let mut legs: Vec<Leg> = plan.iter().map(|e| e.leg).collect();
            legs.dedup();
            let objectives = (0..AIMS.len()).map(Leg::Objective);
            let want: Vec<Leg> =
                [Leg::Departure].into_iter().chain(objectives).chain([Leg::Return]).collect();
            assert_eq!(legs, want, "{f:?}");
        }
    }

    /// A store before an objective's steps say it: the camera, the supplies, the missile that
    /// locks the farthest, the weapon that does the most against the target, else the gun.
    #[test]
    fn the_store_fits_the_objective() {
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let data = SimData::load_json(&assets.join("sim")).unwrap();
        let best = |aim, class, carried: &[(i32, i32)], gun| {
            best_store(aim, class, carried, gun, &data).map(|s| store_name(Some(s), &data))
        };
        let name = |s: &str| Some(s.to_string());
        // HARM, Harpoon, Maverick, Sidewinders: against a ship the Harpoon, a radar the HARM.
        let strike = [(2, 1), (4, 1), (5, 2), (0, 4)];
        assert_eq!(best(Aim::Ground, 12, &strike, true), name("Harpoon (AGM-84A)"));
        assert_eq!(best(Aim::Ground, 8, &strike, true), name("HARM (AGM-88A)"));
        // A runway: the Durandal; a bridge: the Paveway.
        let bombs = [(6, 2), (8, 2), (9, 3)];
        assert_eq!(best(Aim::Ground, 1, &bombs, true), name("Durandal (BLU-107)"));
        assert_eq!(best(Aim::Ground, 4, &bombs, true), name("Paveway (GBU-12)"));
        // Spent stores do not count; nothing that fits: the gun if it does anything.
        assert_eq!(best(Aim::Ground, 12, &[(4, 0), (6, 2)], true), name("Paveway (GBU-12)"));
        assert_eq!(best(Aim::Ground, 12, &[(8, 2)], false), None);
        // Aircraft: the AMRAAM before the Sidewinder, the gun once they are spent.
        let missiles = [(0, 4), (1, 3)];
        assert_eq!(best(Aim::Air, 0, &missiles, true), name("AMRAAM (AIM-120)"));
        assert_eq!(best(Aim::Air, 0, &[(0, 0), (6, 2)], true), name("gun"));
        // The camera, the supplies; a landing needs none.
        assert_eq!(best(Aim::Photo, 3, &[(16, 1), (6, 2)], true), name("camera"));
        assert_eq!(best(Aim::Drop, 3, &[(18, 1)], true), name("supplies"));
        assert_eq!(best(Aim::Land, 3, &strike, true), None);
    }

    /// An air mission: the special aircraft first (shot down where it flies), then a target on
    /// the ground.
    #[test]
    fn objectives_follow_the_mission() {
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let gdata = stealth_mission::GameData::load_json(&assets.join("mission")).unwrap();
        let settings = stealth_mission::MissionSettings {
            theater: 0,
            kind: MissionKind::Air,
            ..Default::default()
        };
        let m = stealth_mission::generate(&gdata, &settings, stealth_mission::Seed::Srand(0x1a2b))
            .unwrap();
        let o = objectives(&m);
        assert_eq!(
            o.iter().map(|o| (o.k, o.aim)).collect::<Vec<_>>(),
            [(0, Aim::Air), (1, Aim::Ground)]
        );
        let plane = &m.special_aircraft.as_ref().unwrap().plane_name;
        assert_eq!(o[0].name, format!("the {plane}"));
        assert!(o[0].unit.is_some() && o[1].unit.is_none());
        assert_eq!(o[1].name, format!("{} at {}", m.secondary.shape_name, m.secondary.place_name));
    }

    /// The waypoints lead to the primary, the secondary, then home (a planned route's points
    /// in between).
    #[test]
    fn the_route_has_a_leg_per_objective() {
        let w = |current, anchors: Vec<usize>| Waypoints {
            points: vec![crate::hud::Waypoint { east: 0.0, north: 0.0 }; 6],
            current,
            mission_current: 0,
            anchors,
        };
        assert_eq!([0, 1, 2, 3].map(|c| route_leg(&w(c, vec![]))), [0, 0, 1, 2]);
        // N1 P N2 S N3 L.
        let legs = [0, 1, 2, 3, 4, 5].map(|c| route_leg(&w(c, vec![0, 1, 3, 5])));
        assert_eq!(legs, [0, 0, 1, 1, 2, 2]);
    }

    /// The first threat that applies: a missile's countermeasure while there is some and it is
    /// not out yet, else evading; a radar tracking only away from the objective.
    #[test]
    fn threats_come_first() {
        let (a, hud, mfd) =
            (Aircraft::new_on_runway(0.0), HudState::default(), MfdState::default());
        let mut since = Vec::new();
        let mut alert = |c: &Ctx, now: f32| pick_alert(c, &mut since, now).map(|i| &ALERTS[i]);
        let title = |s: Option<&Step>| s.map(|s| s.title);
        let mut c = ctx(&a, &hud, &mfd);
        assert_eq!(title(alert(&c, 0.0)), None);
        c.stores = (4, 4, 2);
        c.lights.rad_threat = true;
        assert_eq!(title(alert(&c, 0.0)), Some("Radar-guided missile: chaff"));
        c.lights.chaff = true;
        let evade = alert(&c, 0.0).unwrap();
        assert_eq!(evade.title, "Evade the missile");
        assert_eq!((evade.target)(&c), Target::Action(Action::Decoy));
        c.lights = default();
        c.lights.ir_threat = true;
        c.stores.0 = 0;
        assert_eq!(title(alert(&c, 0.0)), Some("Evade the missile"));
        c.lights = default();
        c.tracked = true;
        assert_eq!(title(alert(&c, 0.0)), Some("Tracked by a radar"));
        c.leg.near = true;
        assert_eq!(title(alert(&c, 0.0)), None);
        // A fire is a warning: shown when it starts, then the lesson again until another one.
        c.fire = true;
        assert_eq!(title(alert(&c, 1.0)), Some("Fire on board"));
        assert_eq!(title(alert(&c, 1.0 + WARNING_SECONDS)), None);
        c.fire = false;
        assert_eq!(title(alert(&c, 20.0)), None);
        c.fire = true;
        assert_eq!(title(alert(&c, 21.0)), Some("Fire on board"));
        // In the air: too slow is a stall, before everything; low fuel is a warning.
        let mut flying = Aircraft::new_on_runway(0.0);
        (flying.on_ground, flying.stall_speed, flying.velocity) = (false, 100.0, 90.0);
        flying.fuel = crate::panel::LOW_FUEL - 1.0;
        let mut c = ctx(&flying, &hud, &mfd);
        c.lights.rad_threat = true;
        assert_eq!(title(alert(&c, 30.0)), Some("Stall"));
        flying.velocity = 200.0;
        let c = ctx(&flying, &hud, &mfd);
        assert_eq!(title(alert(&c, 30.0)), Some("Low fuel"));
    }

    #[test]
    fn the_texts_name_the_objective_and_the_store() {
        let words = Words {
            target: "primary target",
            name: "Missile Boat",
            weapon: "Harpoon (AGM-84A)".into(),
            why: why(Aim::Ground, Some(Store::Weapon(4))),
            what: what(12),
        };
        assert_eq!(fill(OBJECTIVE, &words), "PRIMARY TARGET");
        assert_eq!(fill("Select the {weapon}", &words), "Select the Harpoon (AGM-84A)");
        assert_eq!(
            fill("{why}", &words),
            "Against a ship, the weapon on board that does the most damage"
        );
        assert_eq!(fill("Lock {name}", &words), "Lock Missile Boat");
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
        let pages = [Page::Orders, Page::Weapons, Page::Waypoints, Page::Tactical, Page::Map];
        for p in pages.into_iter().chain([Page::Camera]) {
            let Target::Cmd(cmd) = button(&mfd, p) else { panic!("{p:?}") };
            let on_mfd = [MfdSide::Left, MfdSide::Right].into_iter().any(|side| {
                (0..crate::mfd::OSB_COUNT)
                    .any(|slot| osb(side, slot, &mfd).is_some_and(|o| o.1 == OsbFn::Cmd(cmd)))
            });
            assert!(on_mfd, "{p:?}");
        }
    }

    /// The controls the steps and the threats point at are in every cockpit model, whatever
    /// the avionics (else only the box shows): the F-16's NAV is its A-A cap, the classic
    /// panel's display caps stand for the bezel buttons.
    #[test]
    fn step_buttons_are_in_the_cockpits() {
        use super::super::controls::parse_effect;
        use Action::*;
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/models");
        let jet = [Flaps, Autopilot, Gear, Brakes, Ils, BayDoors];
        let heli = [Ils];
        let armed = [Chaff, Flare, Decoy, SelectWeapon, DesignateTarget, NearestTarget];
        let cockpits: [(&str, &[Action], &[Action]); 5] = [
            ("classic", &jet, &armed),
            ("modern", &jet, &armed),
            ("f16", &jet[..5], &armed),
            ("ah1z", &heli, &armed),
            ("uh60", &heli, &armed[..3]),
        ];
        for (model, flight, weapons) in cockpits {
            let glb = std::fs::read(dir.join(format!("cockpit_{model}.glb"))).unwrap();
            let len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
            let json: serde_json::Value = serde_json::from_slice(&glb[20..20 + len]).unwrap();
            let effects: Vec<Effect> = json["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|n| parse_effect(n["extras"]["control"].as_str()?))
                .collect();
            let osbs = effects.iter().any(|e| matches!(e, Effect::Osb(..)));
            for modern in [false, true] {
                let mfd = MfdState { modern, ..default() };
                let has = |a: Action| effects.iter().any(|e| e.action(&mfd) == Some(a));
                for &a in flight.iter().chain(weapons) {
                    assert!(has(a), "{model}: {a:?}");
                }
                for mode in [0, 1, 2] {
                    let cap = |m| effects.contains(&Effect::HudMode(m));
                    assert!(cap(mode) || mode == 0 && cap(1), "{model}: HUD mode {mode}");
                }
                // Without bezel buttons, the page steps' caps.
                if !osbs {
                    let c = ctx(&Aircraft::new_on_runway(0.0), &HudState::default(), &mfd)
                        .page_button(Page::Orders);
                    let a = match c {
                        Target::Cmd(cmd) => crate::mfd::cmd_action(cmd),
                        Target::Action(a) => a,
                        t => panic!("{t:?}"),
                    };
                    assert!(has(a), "{model}: {a:?}");
                    for a in [Ordnance, SelectWaypoint, Maps, CamAhead] {
                        assert!(has(a), "{model}: {a:?}");
                    }
                }
            }
        }
    }
}

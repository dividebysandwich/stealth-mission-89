//! Air traffic control: a military tower, approach and GCA controller at the friendly
//! airfields, on the cockpit's notices panel with a radio click. Fully automatic: the pilot's
//! own calls go out by themselves when the aircraft gets there.
//!
//! - Departure: "ready for departure" on the runway, the tower clears the takeoff (or lines
//!   the aircraft up while traffic is on final), departure control lets it go on its way.
//! - Recovery, VFR: inbound, "report initial", initial, the break, base with the gear down,
//!   cleared to land (or told to go around with the gear up or traffic on the runway).
//! - Recovery, IFR (low cloud or poor visibility at the field): vectors to an 8-mile final and
//!   a GCA talkdown, course and glidepath every few seconds, down to decision height.
//! - Helicopters: straight in to the helipad.
//! - A closed (bombed) or lost field: a diversion to the nearest open one, the landing
//!   waypoint moved there.
//!
//! The AI aircraft don't listen: the controller only works the player around them.

use std::collections::VecDeque;

use bevy::math::DVec2;
use bevy::prelude::*;
use stealth_sim::types::{fflags, oflags};

use crate::flight::{Aircraft, Player};
use crate::settings::AppState;
use crate::world::{WU, WorldPos};

pub struct AtcPlugin;

impl Plugin for AtcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AtcState>()
            .add_systems(OnEnter(AppState::Flight), |mut s: ResMut<AtcState>| {
                *s = AtcState::default();
            })
            .add_systems(Update, control.run_if(in_state(AppState::Flight)));
    }
}

/// Metres in a nautical mile ("miles" on the radio).
const NM: f64 = 1852.0;
/// Seconds before a call made by the pilot and before the controller's answer.
const PILOT_DELAY: f32 = 0.4;
const REPLY_DELAY: f32 = 1.8;
/// The recovery starts this close to the field (metres), closing on it.
const RECOVERY_RANGE: f64 = 40_000.0;
/// Radar service ends beyond this range from the field during an approach (metres).
const SERVICE_RANGE: f64 = 50_000.0;
/// How far from every field an aircraft must have been before it is "coming home" (metres):
/// a jet, a helicopter.
const AWAY_JET: f64 = 12_000.0;
const AWAY_HELI: f64 = 4_000.0;
/// The initial point's distance out on the runway's centreline (metres).
const INITIAL: f64 = 5_500.0;
/// The GCA's final approach starts this far out (metres).
const GCA_FINAL: f64 = 16_000.0;
/// Glide path, degrees.
const GLIDE: f64 = crate::scenery::GLIDE_PATH_DEG;
/// Altitude the GCA descends the aircraft to before the final (ft).
const GCA_ALT: f32 = 2000.0;

/// One end of a runway, as seen from the aircraft landing on it: the touchdown aim point and
/// the unit direction from it out along the approach (ground metres, x east, y north).
#[derive(Debug, Clone, Copy)]
pub struct RunwayEnd {
    pub aim: DVec2,
    pub toward: DVec2,
}

impl RunwayEnd {
    /// Distance out along the approach and offset from the centreline (positive: right of it,
    /// seen from the approach), metres.
    fn along_lateral(&self, p: DVec2) -> (f64, f64) {
        let rel = p - self.aim;
        (rel.dot(self.toward), rel.dot(self.toward.perp()))
    }

    /// The landing heading, degrees.
    fn heading(&self) -> f64 {
        heading_of(-self.toward)
    }

    /// The runway's number, 1..36.
    fn number(&self) -> u32 {
        ((self.heading() / 10.0).round() as u32 + 35) % 36 + 1
    }

    fn name(&self) -> String {
        format!("{:02}", self.number())
    }
}

/// A friendly airfield or carrier.
#[derive(Debug, Clone)]
pub struct Base {
    /// World object index.
    pub id: usize,
    pub name: String,
    pub pos: DVec2,
    /// Not bombed shut.
    pub open: bool,
    pub carrier: bool,
    pub ends: Vec<RunwayEnd>,
}

/// Another aircraft near the fields.
#[derive(Debug, Clone, Copy)]
pub struct Traffic {
    pub pos: DVec2,
    pub alt_ft: f32,
    pub on_ground: bool,
}

/// What the controller sees of the player's aircraft and the world this frame.
pub struct AtcInput<'a> {
    pub dt: f32,
    /// Ground metres (x east, y north), altitude (ft, and metres above the field level) and
    /// height above the ground (ft).
    pub pos: DVec2,
    pub alt_ft: f32,
    pub alt_m: f64,
    pub agl_ft: f32,
    pub knots: f32,
    /// Degrees from north.
    pub heading: f64,
    pub gear_down: bool,
    pub on_ground: bool,
    pub has_flown: bool,
    pub helicopter: bool,
    /// Instrument conditions at the fields (low cloud, poor visibility).
    pub ifr: bool,
    /// The landing waypoint is the one steered to.
    pub homeward: bool,
    /// The radio is still saying the last call (the spoken ATC): the next waits.
    pub hold: bool,
    pub bases: &'a [Base],
    pub traffic: &'a [Traffic],
}

/// A radio call: who says it, what, and whether it is the player's own.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub from: String,
    pub text: String,
    pub own: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Phase {
    /// The first frames: on the runway, or already in the air.
    #[default]
    Start,
    /// Lined up, waiting for traffic on final.
    Holding,
    /// Cleared, climbing out.
    Departing,
    /// Away from the fields (nothing said).
    Enroute,
    /// VFR: heading for the initial point.
    ToInitial,
    /// VFR: the break and the downwind, until the aircraft is on final.
    Pattern,
    /// IFR: vectors to the GCA's final.
    Vectors,
    /// IFR: the talkdown.
    Talkdown,
    /// On final (cleared or waiting for the runway), until touchdown.
    Final,
    /// Down: nothing more to say.
    Landed,
}

/// The controller's state.
#[derive(Debug, Default)]
pub struct Atc {
    callsign: String,
    phase: Phase,
    /// Countdown (s) of the phase's next action.
    wait: f32,
    /// The field being worked and the runway end in use.
    base: Option<usize>,
    end: Option<RunwayEnd>,
    /// Calls waiting to go out, each with its delay after the one before.
    queue: VecDeque<(f32, Call)>,
    /// Has been far from every field (the next approach is a recovery).
    away: bool,
    /// Seconds the range to the nearest field has been shrinking, and that range.
    closing: f32,
    last_range: f64,
    /// Fields announced closed (not chosen again).
    closed: Vec<usize>,
    /// The landing clearance was given, a gear warning was given.
    cleared: bool,
    gear_warned: bool,
    /// GCA: the last heading given, the miles last read out, the descent given, the talkdown's
    /// next call.
    last_heading: Option<i32>,
    last_miles: i32,
    descent_given: bool,
    /// The aircraft came straight in (no initial).
    straight_in: bool,
    /// A diversion to this field: the landing waypoint moves there.
    pub divert: Option<usize>,
}

impl Atc {
    pub fn new(callsign: String) -> Self {
        Self { callsign, wait: 2.0, ..default() }
    }

    fn say(&mut self, from: String, text: String, own: bool) {
        let delay = if own { PILOT_DELAY } else { REPLY_DELAY };
        self.queue.push_back((delay, Call { from, text, own }));
    }

    fn pilot(&mut self, text: String) {
        let cs = self.callsign.clone();
        self.say(cs, text, true);
    }

    /// One frame: the calls going out now.
    pub fn step(&mut self, i: &AtcInput) -> Vec<Call> {
        self.wait -= i.dt;
        self.control(i);
        let mut out = Vec::new();
        if i.hold {
            return out;
        }
        let mut dt = i.dt;
        while let Some((delay, _)) = self.queue.front_mut() {
            *delay -= dt;
            if *delay > 0.0 {
                break;
            }
            dt = -*delay;
            out.push(self.queue.pop_front().unwrap().1);
        }
        out
    }

    /// A call is still waiting or being said.
    fn radio_busy(&self, i: &AtcInput) -> bool {
        i.hold || !self.queue.is_empty()
    }

    fn base<'a>(&self, i: &AtcInput<'a>) -> Option<&'a Base> {
        self.base.and_then(|id| i.bases.iter().find(|b| b.id == id))
    }

    /// The nearest airfield still taking aircraft (not a carrier, not announced closed), and
    /// whether a carrier is nearer (its own deck crew handles that).
    fn nearest<'a>(&self, i: &AtcInput<'a>) -> Option<&'a Base> {
        let d = |b: &&Base| b.pos.distance(i.pos);
        let field = i
            .bases
            .iter()
            .filter(|b| !b.carrier && !self.closed.contains(&b.id))
            .min_by(|a, b| d(a).total_cmp(&d(b)))?;
        let carrier = i.bases.iter().filter(|b| b.carrier).map(|b| d(&b)).fold(f64::MAX, f64::min);
        (carrier > d(&field)).then_some(field)
    }

    fn control(&mut self, i: &AtcInput) {
        match self.phase {
            Phase::Start => self.start(i),
            Phase::Holding => self.holding(i),
            Phase::Departing => self.departing(i),
            Phase::Enroute => self.enroute(i),
            Phase::ToInitial => self.toward_initial(i),
            Phase::Pattern => self.pattern(i),
            Phase::Vectors => self.vectors(i),
            Phase::Talkdown => self.talkdown(i),
            Phase::Final => self.final_approach(i),
            // Off again after rearming: a new departure (unannounced) and recovery.
            Phase::Landed if !i.on_ground && i.agl_ft > 200.0 => {
                self.base = None;
                self.phase = Phase::Enroute;
            }
            Phase::Landed => {}
        }
    }

    fn start(&mut self, i: &AtcInput) {
        if !i.on_ground || i.has_flown {
            // Started in the air: on the way home.
            self.away = true;
            self.phase = Phase::Enroute;
            return;
        }
        let Some(b) = self.nearest(i).filter(|b| b.pos.distance(i.pos) < 3000.0) else {
            self.phase = Phase::Enroute;
            return;
        };
        if self.wait > 0.0 {
            return;
        }
        self.base = Some(b.id);
        // The runway lined up on.
        self.end = b
            .ends
            .iter()
            .min_by(|x, y| {
                let f = |e: &RunwayEnd| angle_diff(e.heading(), i.heading).abs();
                f(x).total_cmp(&f(y))
            })
            .copied();
        let tower = format!("{} Tower", b.name);
        if i.helicopter {
            self.pilot(format!("{tower}, {}, ready for departure from the pad", self.callsign));
        } else {
            let rw = self.runway(b);
            self.pilot(format!("{tower}, {}, ready for departure, runway {rw}", self.callsign));
        }
        if !i.helicopter && traffic_on_final(b, i.traffic) {
            let rw = self.runway(b);
            let cs = self.callsign.clone();
            self.say(
                tower,
                format!("{cs}, runway {rw}, line up and wait, traffic on final"),
                false,
            );
            self.phase = Phase::Holding;
            self.wait = 1.0;
        } else {
            self.takeoff_clearance(b, i.helicopter);
        }
    }

    fn takeoff_clearance(&mut self, b: &Base, helicopter: bool) {
        let cs = self.callsign.clone();
        let text = if helicopter {
            format!("{cs}, cleared for takeoff")
        } else {
            format!("{cs}, runway {}, cleared for takeoff", self.runway(b))
        };
        self.say(format!("{} Tower", b.name), text, false);
        self.phase = Phase::Departing;
    }

    fn holding(&mut self, i: &AtcInput) {
        let Some(b) = self.base(i) else {
            self.phase = Phase::Enroute;
            return;
        };
        if i.has_flown {
            self.phase = Phase::Departing;
        } else if self.wait <= 0.0 {
            self.wait = 1.0;
            if !traffic_on_final(b, i.traffic) {
                self.takeoff_clearance(b, false);
            }
        }
    }

    fn departing(&mut self, i: &AtcInput) {
        let Some(b) = self.base(i) else {
            self.phase = Phase::Enroute;
            return;
        };
        let range = b.pos.distance(i.pos);
        if i.on_ground || (i.agl_ft < 2000.0 && range < 5000.0) {
            return;
        }
        let cs = self.callsign.clone();
        if i.helicopter {
            self.say(
                format!("{} Tower", b.name),
                format!("{cs}, frequency change approved"),
                false,
            );
        } else {
            self.say(
                format!("{} Departure", b.name),
                format!("{cs}, radar contact, climb as filed, resume own navigation"),
                false,
            );
        }
        self.base = None;
        self.phase = Phase::Enroute;
    }

    fn enroute(&mut self, i: &AtcInput) {
        let Some(b) = self.nearest(i) else { return };
        let range = b.pos.distance(i.pos);
        if range > if i.helicopter { AWAY_HELI } else { AWAY_JET } {
            self.away = true;
        }
        if range < self.last_range - 1.0 {
            self.closing += i.dt;
        } else if range > self.last_range + 1.0 {
            self.closing = 0.0;
        }
        self.last_range = range;
        let homeward = self.away || i.homeward;
        if !homeward || i.on_ground || !i.has_flown || range > RECOVERY_RANGE || self.closing < 4.0
        {
            return;
        }
        self.inbound(i, b);
    }

    /// The call inbound to `b`, and the controller's plan for the approach.
    fn inbound(&mut self, i: &AtcInput, b: &Base) {
        let cs = self.callsign.clone();
        let rel = i.pos - b.pos;
        let (range, dir) = (miles(rel.length()), compass(rel));
        let approach = format!("{} Approach", b.name);
        let (station, intent) = if i.helicopter {
            (format!("{} Tower", b.name), "inbound to land")
        } else {
            (approach.clone(), "inbound")
        };
        self.pilot(format!("{station}, {cs}, {range} miles {dir}, {intent}"));
        self.away = false;
        self.closing = 0.0;
        if !b.open {
            self.closed.push(b.id);
            match self.nearest(i) {
                Some(d) => {
                    let to = d.pos - i.pos;
                    self.say(
                        approach,
                        format!(
                            "{cs}, {} is closed, divert {}, heading {}, {} miles",
                            b.name,
                            d.name,
                            heading_text(heading_of(to)),
                            miles(to.length())
                        ),
                        false,
                    );
                    self.divert = Some(d.id);
                    // On the way to the new field: the next approach there is a recovery.
                    self.away = true;
                }
                None => self.say(
                    approach,
                    format!("{cs}, {} is closed, no divert field, land at your discretion", b.name),
                    false,
                ),
            }
            self.phase = Phase::Enroute;
            return;
        }
        self.base = Some(b.id);
        self.cleared = false;
        self.gear_warned = false;
        self.straight_in = false;
        // The runway end facing the aircraft (no wind to choose by).
        self.end = b
            .ends
            .iter()
            .max_by(|x, y| {
                let f = |e: &RunwayEnd| (i.pos - e.aim).normalize_or_zero().dot(e.toward);
                f(x).total_cmp(&f(y))
            })
            .copied();
        let rw = self.runway(b);
        if i.helicopter {
            self.say(
                format!("{} Tower", b.name),
                format!("{cs}, cleared to land at the helipad"),
                false,
            );
            self.cleared = true;
            self.phase = Phase::Final;
        } else if i.ifr {
            let Some(e) = self.end else { return };
            let h = self.vector_heading(i, &e);
            let mut text = format!(
                "{cs}, radar contact, {range} miles {dir}, expect GCA runway {rw}, fly heading {}",
                heading_text(h)
            );
            if i.alt_ft > GCA_ALT + 500.0 {
                text += ", descend and maintain 2,000";
                self.descent_given = true;
            } else {
                self.descent_given = false;
            }
            self.say(approach, text, false);
            self.last_heading = Some(round5(h));
            self.wait = 8.0;
            self.phase = Phase::Vectors;
        } else {
            self.say(
                approach,
                format!("{cs}, radar contact, {range} miles {dir}, expect overhead runway {rw}, report initial"),
                false,
            );
            self.phase = Phase::ToInitial;
        }
    }

    fn runway(&self, b: &Base) -> String {
        self.runway_end(b).map_or_else(|| "36".to_string(), |e| e.name())
    }

    /// The runway end in use: the field's own (its runways come in with the scenery, in no
    /// fixed order), else the one chosen.
    fn runway_end(&self, b: &Base) -> Option<RunwayEnd> {
        let chosen = self.end?;
        b.ends
            .iter()
            .find(|e| e.toward.dot(chosen.toward) > 0.99 && e.aim.distance(chosen.aim) < 3000.0)
            .copied()
            .or(Some(chosen))
    }

    /// The field being approached, else back to en route (radar service ended, or the field
    /// is gone).
    fn approach_field<'a>(&mut self, i: &AtcInput<'a>) -> Option<(&'a Base, RunwayEnd)> {
        let b = self.base(i);
        let lost = b.is_none_or(|b| !b.open);
        let far = b.is_some_and(|b| b.pos.distance(i.pos) > SERVICE_RANGE);
        if lost || far {
            if far {
                let station = format!("{} Approach", b.unwrap().name);
                let cs = self.callsign.clone();
                self.say(station, format!("{cs}, radar service terminated"), false);
            }
            self.base = None;
            self.away = true;
            self.closing = 0.0;
            self.phase = Phase::Enroute;
            return None;
        }
        let b = b.unwrap();
        Some((b, self.runway_end(b)?))
    }

    fn toward_initial(&mut self, i: &AtcInput) {
        let Some((b, e)) = self.approach_field(i) else { return };
        let (along, lateral) = e.along_lateral(i.pos);
        let tower = format!("{} Tower", b.name);
        let cs = self.callsign.clone();
        if on_final(&e, i) {
            if i.gear_down {
                self.straight_in = true;
                self.pilot(format!(
                    "{tower}, {cs}, {} mile straight-in, gear down",
                    miles(along).max(1)
                ));
                self.landing_clearance(i, b, &e);
                self.phase = Phase::Final;
            } else {
                self.gear_check(i, b, &e, along);
            }
            return;
        }
        let aligned = angle_diff(i.heading, e.heading()).abs() < 45.0;
        if (INITIAL * 0.5..INITIAL * 1.6).contains(&along) && lateral.abs() < 1500.0 && aligned {
            let rw = e.name();
            self.pilot(format!("{tower}, {cs}, initial, runway {rw}"));
            let text = if traffic_in_pattern(b, i.traffic) {
                format!(
                    "{cs}, runway {rw}, number two, break at the numbers, report base, gear down"
                )
            } else {
                format!("{cs}, runway {rw}, break at the numbers, report base, gear down")
            };
            self.say(tower, text, false);
            self.phase = Phase::Pattern;
        }
    }

    fn pattern(&mut self, i: &AtcInput) {
        let Some((b, e)) = self.approach_field(i) else { return };
        let (along, _) = e.along_lateral(i.pos);
        if on_final(&e, i) {
            if i.gear_down {
                let cs = self.callsign.clone();
                self.pilot(format!("{cs}, base, gear down"));
                self.landing_clearance(i, b, &e);
                self.phase = Phase::Final;
            } else {
                self.gear_check(i, b, &e, along);
            }
        }
    }

    /// On final with the gear up: a warning, then a go-around.
    fn gear_check(&mut self, i: &AtcInput, b: &Base, e: &RunwayEnd, along: f64) {
        let cs = self.callsign.clone();
        let tower = format!("{} Tower", b.name);
        if along < 1500.0 {
            self.say(tower, format!("{cs}, go around, go around, your gear is up"), false);
            self.go_around(i, b, e);
        } else if along < 3500.0 && !self.gear_warned {
            self.gear_warned = true;
            self.say(tower, format!("{cs}, check gear down"), false);
        }
    }

    /// Back into the pattern (VFR) or to the vectors (IFR).
    fn go_around(&mut self, i: &AtcInput, b: &Base, e: &RunwayEnd) {
        self.cleared = false;
        self.gear_warned = false;
        let cs = self.callsign.clone();
        if i.ifr {
            self.say(
                format!("{} Approach", b.name),
                format!("{cs}, missed approach, fly runway heading, climb and maintain 2,000"),
                false,
            );
            self.last_heading = Some(round5(e.heading()));
            self.descent_given = true;
            self.wait = 20.0;
            self.phase = Phase::Vectors;
        } else {
            self.say(format!("{} Tower", b.name), format!("{cs}, re-enter initial"), false);
            self.phase = Phase::ToInitial;
        }
    }

    /// Cleared to land, or told to continue while the runway is not free.
    fn landing_clearance(&mut self, i: &AtcInput, b: &Base, e: &RunwayEnd) {
        let cs = self.callsign.clone();
        let tower = format!("{} Tower", b.name);
        let (along, _) = e.along_lateral(i.pos);
        if runway_busy(b, e, along, i.traffic) {
            self.say(tower, format!("{cs}, continue, traffic on the runway"), false);
            self.cleared = false;
        } else {
            self.say(
                tower,
                format!("{cs}, runway {}, cleared to land", e.name()),
                false,
            );
            self.cleared = true;
        }
        self.wait = 1.0;
    }

    fn final_approach(&mut self, i: &AtcInput) {
        let Some(b) = self.base(i) else {
            self.phase = Phase::Enroute;
            return;
        };
        let cs = self.callsign.clone();
        let tower = format!("{} Tower", b.name);
        if i.helicopter {
            if i.on_ground && i.knots < 5.0 {
                self.say(tower, format!("{cs}, welcome home"), false);
                self.phase = Phase::Landed;
            }
            return;
        }
        let Some(e) = self.runway_end(b) else { return };
        let (along, lateral) = e.along_lateral(i.pos);
        if i.on_ground {
            if i.knots < 60.0 {
                self.say(tower, format!("{cs}, welcome home, taxi to parking"), false);
                self.phase = Phase::Landed;
            }
            return;
        }
        // Past the runway's far end in the air, or well off the approach: a go-around.
        if along < -1800.0
            || lateral.abs() > 2500.0
            || angle_diff(i.heading, e.heading()).abs() > 90.0
        {
            self.go_around(i, b, &e);
            return;
        }
        if !self.cleared && self.wait <= 0.0 {
            self.wait = 1.0;
            if !runway_busy(b, &e, along, i.traffic) {
                self.say(
                    tower,
                    format!("{cs}, runway {}, cleared to land", e.name()),
                    false,
                );
                self.cleared = true;
            } else if along < 800.0 {
                self.say(tower, format!("{cs}, go around, traffic on the runway"), false);
                self.go_around(i, b, &e);
            }
        }
    }

    /// GCA: the heading towards the final approach course.
    fn vector_heading(&self, i: &AtcInput, e: &RunwayEnd) -> f64 {
        let (along, lateral) = e.along_lateral(i.pos);
        let land = e.heading();
        if along < GCA_FINAL * 0.6 || lateral.abs() > along * 0.7 {
            // Not in front of the runway: out to the start of the final, beside it.
            let side = if lateral >= 0.0 { 1.0 } else { -1.0 };
            let gate = e.aim + e.toward * GCA_FINAL + e.toward.perp() * side * 3000.0;
            heading_of(gate - i.pos)
        } else if lateral.abs() > 600.0 {
            // A 30-degree cut to the centreline.
            land - lateral.signum() * 30.0
        } else {
            land
        }
    }

    fn vectors(&mut self, i: &AtcInput) {
        let Some((b, e)) = self.approach_field(i) else { return };
        let approach = format!("{} Approach", b.name);
        let gca = format!("{} GCA", b.name);
        let cs = self.callsign.clone();
        let (along, lateral) = e.along_lateral(i.pos);
        let established = along > 4000.0
            && lateral.abs() < 600.0
            && angle_diff(i.heading, e.heading()).abs() < 25.0;
        if established {
            self.say(
                gca,
                format!(
                    "{cs}, on course, {} miles from touchdown, wheels should be down, begin descent at the glidepath",
                    miles(along)
                ),
                false,
            );
            self.last_miles = miles(along);
            self.wait = 5.0;
            self.phase = Phase::Talkdown;
            return;
        }
        if self.wait > 0.0 || self.radio_busy(i) {
            return;
        }
        let h = self.vector_heading(i, &e);
        let given = round5(h);
        let off = self.last_heading.map_or(180.0, |l| angle_diff(l as f64, h).abs());
        let flying = angle_diff(h, i.heading);
        if off >= 15.0 || flying.abs() > 20.0 {
            let turn = if flying > 0.0 { "turn right" } else { "turn left" };
            let mut text = format!("{cs}, {turn} heading {}", heading_text(h));
            if !self.descent_given && i.alt_ft > GCA_ALT + 500.0 {
                text += ", descend and maintain 2,000";
                self.descent_given = true;
            }
            self.say(approach, text, false);
            self.last_heading = Some(given);
            self.wait = 12.0;
        } else {
            self.wait = 2.0;
        }
    }

    fn talkdown(&mut self, i: &AtcInput) {
        let Some((b, e)) = self.approach_field(i) else { return };
        let gca = format!("{} GCA", b.name);
        let cs = self.callsign.clone();
        let (along, lateral) = e.along_lateral(i.pos);
        if i.on_ground {
            self.phase = Phase::Final;
            return;
        }
        let loc = lateral.atan2(along.max(1.0)).to_degrees();
        if along < -200.0 || loc.abs() > 10.0 || along > GCA_FINAL * 1.6 {
            self.say(gca, format!("{cs}, too far off course, climb and maintain 2,000"), false);
            self.go_around(i, b, &e);
            return;
        }
        if along < 1300.0 || i.agl_ft < 250.0 {
            self.say(gca, format!("{cs}, at decision height, say runway in sight?"), false);
            self.pilot(format!("{cs}, runway in sight"));
            self.phase = Phase::Final;
            self.wait = 1.0;
            return;
        }
        // Fresh numbers when the radio is free, not old ones queued behind a long call.
        if self.wait > 0.0 || self.radio_busy(i) {
            return;
        }
        let m = miles(along);
        self.wait = if m <= 2 { 3.5 } else { 5.0 };
        let gs = i.alt_m.max(0.0).atan2(along).to_degrees() - GLIDE;
        if gs < -1.0 && i.agl_ft < 700.0 {
            self.say(gca, format!("{cs}, too low, climb immediately"), false);
            return;
        }
        let course = if loc.abs() < 0.5 {
            "on course".to_string()
        } else {
            let side = if loc > 0.0 { "right" } else { "left" };
            let size = if loc.abs() < 1.5 { "slightly " } else { "" };
            let h = e.heading() - (loc * 4.0).clamp(-15.0, 15.0);
            format!("{size}{side} of course, turn heading {}", heading_text(h))
        };
        let path = match gs {
            g if g.abs() < 0.3 => "on glidepath",
            g if g > 1.0 => "well above glidepath",
            g if g > 0.0 => "slightly above glidepath",
            g if g > -1.0 => "slightly below glidepath",
            _ => "below glidepath, adjust rate of descent",
        };
        let mut text = format!("{course}, {path}");
        if m != self.last_miles && m >= 1 {
            self.last_miles = m;
            let unit = if m == 1 { "mile" } else { "miles" };
            text = format!("{m} {unit} from touchdown, {text}");
        }
        if !i.gear_down && m <= 4 && !self.gear_warned {
            self.gear_warned = true;
            text += ", check wheels down";
        }
        if m <= 4 && !self.cleared {
            if runway_busy(b, &e, along, i.traffic) {
                text += ", continue, traffic on the runway";
            } else {
                self.cleared = true;
                text += &format!(", cleared to land runway {}", e.name());
            }
        }
        self.say(gca, format!("{cs}, {text}"), false);
    }
}

/// The aircraft is on final for `e`: in front of it, near the centreline, lined up and low.
fn on_final(e: &RunwayEnd, i: &AtcInput) -> bool {
    let (along, lateral) = e.along_lateral(i.pos);
    (200.0..4500.0).contains(&along)
        && lateral.abs() < along * 0.25 + 200.0
        && angle_diff(i.heading, e.heading()).abs() < 30.0
        && i.agl_ft < 1500.0
}

/// Traffic in the air on final to any of `b`'s runway ends.
fn traffic_on_final(b: &Base, traffic: &[Traffic]) -> bool {
    traffic.iter().filter(|t| !t.on_ground && t.alt_ft < 2000.0).any(|t| {
        b.ends.iter().any(|e| {
            let (along, lateral) = e.along_lateral(t.pos);
            (-500.0..8000.0).contains(&along) && lateral.abs() < 700.0
        })
    })
}

/// Traffic low over the field (in the circuit).
fn traffic_in_pattern(b: &Base, traffic: &[Traffic]) -> bool {
    traffic.iter().any(|t| !t.on_ground && t.alt_ft < 3000.0 && t.pos.distance(b.pos) < 8000.0)
}

/// Something on the runway of `e`, or on final ahead of an aircraft `along` metres out.
fn runway_busy(b: &Base, e: &RunwayEnd, along: f64, traffic: &[Traffic]) -> bool {
    let _ = b;
    traffic.iter().any(|t| {
        let (a, l) = e.along_lateral(t.pos);
        if t.on_ground {
            (-1800.0..400.0).contains(&a) && l.abs() < 60.0
        } else {
            (-300.0..along - 500.0).contains(&a) && l.abs() < 500.0 && t.alt_ft < 1500.0
        }
    })
}

/// Degrees from north of a ground direction (x east, y north), 0..360.
fn heading_of(v: DVec2) -> f64 {
    v.x.atan2(v.y).to_degrees().rem_euclid(360.0)
}

/// `a - b`, -180..180 degrees.
fn angle_diff(a: f64, b: f64) -> f64 {
    (a - b + 540.0).rem_euclid(360.0) - 180.0
}

/// A heading rounded to 5 degrees, 5..360.
fn round5(h: f64) -> i32 {
    let r = ((h.rem_euclid(360.0) / 5.0).round() as i32 * 5) % 360;
    if r == 0 { 360 } else { r }
}

fn heading_text(h: f64) -> String {
    format!("{:03}", round5(h))
}

fn miles(m: f64) -> i32 {
    (m / NM).round() as i32
}

/// Compass point of a direction from the field.
fn compass(v: DVec2) -> &'static str {
    const POINTS: [&str; 8] =
        ["north", "northeast", "east", "southeast", "south", "southwest", "west", "northwest"];
    POINTS[((heading_of(v) + 22.5) / 45.0) as usize % 8]
}

/// The flown aircraft's callsign: its type's, and the pilot's number in coop.
fn callsign(kind: crate::aircraft::AircraftKind, pilot: Option<u8>) -> String {
    use crate::aircraft::AircraftKind as K;
    let name = match kind {
        K::F117 => "Nighthawk",
        K::F16 => "Viper",
        K::UH60 => "Hawk",
        K::AH1Z => "Cobra",
        K::Tornado => "Tornado",
        K::A10 => "Hog",
        K::RAH66 => "Comanche",
        K::AV8B => "Harrier",
        K::OH58 => "Kiowa",
        K::CH47 => "Chinook",
    };
    format!("{name} 1-{}", pilot.map_or(1, |p| p as u32 + 1))
}

/// The controller of this flight (None until the aircraft is there).
#[derive(Resource, Default)]
pub struct AtcState {
    atc: Option<Atc>,
    /// The fields and their runways, rebuilt every second, and the countdown to that.
    bases: Vec<Base>,
    refresh: f32,
}

/// Sound id of the radio's click (`sounds/radio_click.flac`, optional).
pub const RADIO_CLICK: u8 = 112;

#[allow(clippy::too_many_arguments)]
fn control(
    settings: Res<crate::settings::Settings>,
    time: Res<Time>,
    combat: Option<Res<crate::combat::Combat>>,
    mission: Option<Res<crate::mission::ActiveMission>>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    papi: Option<Res<crate::scenery::PapiSites>>,
    weather: Option<Res<crate::weather::WeatherState>>,
    waypoints: Option<ResMut<crate::hud::Waypoints>>,
    player: Query<(&Aircraft, &WorldPos), With<Player>>,
    mut state: ResMut<AtcState>,
    mut notices: ResMut<crate::player_notices::PlayerNotices>,
    mut sounds: MessageWriter<crate::sound::PlaySound>,
    mut tts: ResMut<crate::tts::Tts>,
) {
    let (Some(combat), Ok((a, pos))) = (combat, player.single()) else { return };
    if !settings.atc || a.ended.is_some() || time.delta_secs() <= 0.0 {
        return;
    }
    // In a campaign only a sortie is flown: not the Ops screen (the aircraft parked behind
    // it), nor an air defence console. Each sortie starts afresh.
    if campaign.as_ref().is_some_and(|c| c.phase != crate::campaign::Phase::Sortie) {
        state.atc = None;
        return;
    }
    let sim = &combat.sim;
    let state = &mut *state;
    state.refresh -= time.delta_secs();
    if state.refresh <= 0.0 {
        state.refresh = 1.0;
        state.bases = bases(sim, mission.as_deref(), papi.as_deref());
    }
    let me = sim.active_pilot();
    let atc = state.atc.get_or_insert_with(|| {
        Atc::new(callsign(settings.aircraft, campaign.is_some().then_some(me)))
    });
    let ground = DVec2::new(pos.0.x, -pos.0.z);
    let traffic = traffic(sim, me, ground);
    let ifr = weather.as_ref().is_some_and(|w| instrument_conditions(&w.weather));
    let land = waypoints.as_ref().map_or(3, |w| land_index(w));
    let homeward = waypoints.as_ref().is_some_and(|w| w.current == land && !w.points.is_empty());
    let (h, ..) = a.euler();
    let input = AtcInput {
        dt: time.delta_secs(),
        pos: ground,
        alt_ft: a.altitude,
        alt_m: pos.0.y,
        agl_ft: a.altitude - a.ground_altitude,
        knots: a.knots(),
        heading: (h as f64).to_degrees().rem_euclid(360.0),
        gear_down: a.gear_down,
        on_ground: a.on_ground,
        has_flown: a.has_flown,
        helicopter: settings.aircraft.spec().helicopter,
        ifr,
        homeward,
        hold: tts.busy(),
        bases: &state.bases,
        traffic: &traffic,
    };
    let calls = atc.step(&input);
    if let Some(id) = atc.divert.take()
        && let (Some(mut w), Some(b)) = (waypoints, state.bases.iter().find(|b| b.id == id))
        && let Some(p) = w.points.get_mut(land)
    {
        p.east = b.pos.x / WU;
        p.north = b.pos.y / WU;
        w.current = land;
    }
    for c in calls {
        info!("ATC {}: {}", c.from, c.text);
        notices.chat(&c.from, &c.text, c.own);
        // Spoken, the radio's loudspeaker clicks by itself.
        if !speak(&mut tts, &settings, &c) {
            sounds
                .write(crate::sound::PlaySound { id: RADIO_CLICK, priority: crate::sound::DIRECT });
        }
    }
}

/// The voices' pace on the radio (the Piper voices read at an audiobook's).
pub const RADIO_TEMPO: f32 = 1.15;

/// Says a call in the controllers' or the pilot's voice; false without one.
fn speak(tts: &mut crate::tts::Tts, s: &crate::settings::Settings, c: &Call) -> bool {
    let atc = |station: &str| {
        let speakers = tts.speakers(&s.atc_voice);
        if s.atc_vary_station && speakers > 1 {
            station_speaker(station, s.atc_speaker, speakers)
        } else {
            s.atc_speaker
        }
    };
    if c.own {
        // The same voice as the controller's: a quicker pace tells them apart.
        let same = s.pilot_voice == s.atc_voice && s.pilot_speaker == s.atc_speaker;
        let tempo = if same { RADIO_TEMPO * 1.12 } else { RADIO_TEMPO };
        tts.say(crate::tts::PILOT_TALKER, &s.pilot_voice, s.pilot_speaker, tempo, &c.text)
    } else {
        let speaker = atc(&c.from);
        tts.say(crate::tts::ATC_TALKER, &s.atc_voice, speaker, RADIO_TEMPO, &c.text)
    }
}

/// A station's own speaker of a multi-speaker voice: the chosen one for the tower, others for
/// the rest, the same each time.
fn station_speaker(station: &str, chosen: u32, speakers: u32) -> u32 {
    let role = station.rsplit(' ').next().unwrap_or("");
    if role == "Tower" {
        return chosen;
    }
    let h = station.bytes().fold(2166136261u32, |h, b| (h ^ b as u32).wrapping_mul(16777619));
    (chosen + 1 + h % (speakers - 1)) % speakers
}

/// The landing base's index among the waypoints.
fn land_index(w: &crate::hud::Waypoints) -> usize {
    w.anchors.get(3).copied().unwrap_or(3)
}

/// Low cloud or poor visibility: the approaches are flown on instruments.
fn instrument_conditions(w: &stealth_mission::weather::Weather) -> bool {
    let low = |l: &stealth_mission::weather::Layer| l.cover >= 50 && l.base_ft < 3000;
    w.vis_km < 5 || low(&w.stratus) || low(&w.cumulus)
}

/// The friendly (and neutral) airfields and carriers with their runway ends: those of the
/// scenery's runways near each, else the sim's north-south strip.
fn bases(
    sim: &stealth_sim::Sim,
    mission: Option<&crate::mission::ActiveMission>,
    papi: Option<&crate::scenery::PapiSites>,
) -> Vec<Base> {
    let sites = papi.map_or(&[][..], |p| &p.0[..]);
    sim.objects()
        .iter()
        .enumerate()
        .skip(3)
        .filter(|(_, o)| {
            o.flags & (oflags::AIRFIELD | oflags::CARRIER) != 0
                && o.flags & (oflags::FRIENDLY | oflags::NEUTRAL) != 0
        })
        .map(|(id, o)| {
            let p = crate::sim_render::map_point(o.x as f64, o.y as f64, 0.0);
            let pos = DVec2::new(p.x, -p.z);
            let mut ends: Vec<RunwayEnd> = sites
                .iter()
                .filter(|s| s.aim.distance(pos) < 2500.0)
                .map(|s| RunwayEnd { aim: s.aim, toward: s.toward })
                .collect();
            if ends.is_empty() {
                // The sim's strip: about 2 km north-south, the aim points 300 m in.
                ends = [1.0, -1.0]
                    .map(|s| RunwayEnd {
                        aim: pos + DVec2::new(0.0, s * 700.0),
                        toward: DVec2::new(0.0, s),
                    })
                    .to_vec();
            }
            let name = mission
                .map(|m| crate::map_draw::object_name(&m.0, id))
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| "Base".to_string());
            Base {
                id,
                name,
                pos,
                open: o.flags & (oflags::DISABLED | oflags::DESTROYED) == 0,
                carrier: o.flags & oflags::CARRIER != 0,
                ends,
            }
        })
        .collect()
}

/// The AI aircraft and the other pilots within 20 km.
fn traffic(sim: &stealth_sim::Sim, me: u8, near: DVec2) -> Vec<Traffic> {
    const RANGE: f64 = 20_000.0;
    let ai = sim.flights().iter().filter(|u| {
        u.flags & fflags::EXISTS != 0
            && u.flags & fflags::DESTROYED == 0
            && (u.flags & fflags::AIRBORNE != 0 || u.speed > 0)
            && (u.wx, u.wy) != (0, 0)
    });
    let ai = ai.map(|u| {
        let p = crate::sim_render::map_point(u.wx as f64 / 32.0, u.wy as f64 / 32.0, 0.0);
        Traffic {
            pos: DVec2::new(p.x, -p.z),
            alt_ft: u.alt as f32,
            on_ground: u.alt < 60 || u.flags & fflags::ROLLOUT != 0,
        }
    });
    let pilots =
        sim.remote_pilots().into_iter().filter(|p| p.id != me && p.chute.is_none()).map(|p| {
            Traffic {
                pos: DVec2::new(p.view.wx as f64 * WU, p.view.wy as f64 * WU),
                alt_ft: p.view.alt as f32,
                on_ground: p.on_ground,
            }
        });
    ai.chain(pilots).filter(|t| t.pos.distance(near) < RANGE).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A field at the origin with a north-south runway, its aim points 700 m from the middle.
    fn field(open: bool) -> Base {
        Base {
            id: 5,
            name: "Gutersloh".into(),
            pos: DVec2::ZERO,
            open,
            carrier: false,
            ends: vec![
                RunwayEnd { aim: DVec2::new(0.0, -700.0), toward: DVec2::new(0.0, -1.0) },
                RunwayEnd { aim: DVec2::new(0.0, 700.0), toward: DVec2::new(0.0, 1.0) },
            ],
        }
    }

    fn other() -> Base {
        Base { id: 9, name: "Wildenrath".into(), pos: DVec2::new(60_000.0, 0.0), ..field(true) }
    }

    struct Flight {
        atc: Atc,
        bases: Vec<Base>,
        traffic: Vec<Traffic>,
        calls: Vec<Call>,
        pos: DVec2,
        alt: f32,
        heading: f64,
        gear: bool,
        ground: bool,
        flown: bool,
        ifr: bool,
        knots: f32,
    }

    impl Flight {
        fn new(bases: Vec<Base>) -> Self {
            Self {
                atc: Atc::new("Nighthawk 1-1".into()),
                bases,
                traffic: Vec::new(),
                calls: Vec::new(),
                pos: DVec2::new(0.0, -600.0),
                alt: 0.0,
                heading: 0.0,
                gear: true,
                ground: true,
                flown: false,
                ifr: false,
                knots: 0.0,
            }
        }

        /// Runs `secs` seconds in steps of 0.1 s, moving along the heading at `knots`.
        fn run(&mut self, secs: f32) {
            for _ in 0..(secs * 10.0) as usize {
                let r = self.heading.to_radians();
                self.pos += DVec2::new(r.sin(), r.cos()) * self.knots as f64 * 0.5144 * 0.1;
                let input = AtcInput {
                    dt: 0.1,
                    pos: self.pos,
                    alt_ft: self.alt,
                    alt_m: self.alt as f64 * WU,
                    agl_ft: self.alt,
                    knots: self.knots,
                    heading: self.heading,
                    gear_down: self.gear,
                    on_ground: self.ground,
                    has_flown: self.flown,
                    helicopter: false,
                    ifr: self.ifr,
                    homeward: false,
                    hold: false,
                    bases: &self.bases,
                    traffic: &self.traffic,
                };
                let calls = self.atc.step(&input);
                self.calls.extend(calls);
            }
        }

        fn said(&self, text: &str) -> bool {
            self.calls.iter().any(|c| c.text.contains(text))
        }

        /// In the air at `pos` heading `heading`.
        fn fly(&mut self, pos: DVec2, alt: f32, heading: f64, knots: f32) {
            self.pos = pos;
            self.alt = alt;
            self.heading = heading;
            self.knots = knots;
            self.ground = false;
            self.flown = true;
        }
    }

    #[test]
    fn departure_is_cleared_and_handed_off() {
        let mut f = Flight::new(vec![field(true)]);
        f.run(5.0);
        assert!(f.said("ready for departure, runway 36"), "{:?}", f.calls);
        assert!(f.said("runway 36, cleared for takeoff"));
        f.fly(DVec2::new(0.0, 6000.0), 2500.0, 0.0, 400.0);
        f.run(3.0);
        assert!(f.said("radar contact, climb as filed"));
        assert_eq!(f.atc.phase, Phase::Enroute);
    }

    #[test]
    fn departure_waits_for_traffic_on_final() {
        let mut f = Flight::new(vec![field(true)]);
        f.traffic =
            vec![Traffic { pos: DVec2::new(0.0, -5000.0), alt_ft: 900.0, on_ground: false }];
        f.run(5.0);
        assert!(f.said("line up and wait"));
        assert!(!f.said("cleared for takeoff"));
        f.traffic.clear();
        f.run(3.0);
        assert!(f.said("cleared for takeoff"));
    }

    #[test]
    fn overhead_recovery_to_landing() {
        let mut f = Flight::new(vec![field(true)]);
        // 30 km south of the field, heading home.
        f.fly(DVec2::new(0.0, -30_000.0), 2000.0, 0.0, 300.0);
        f.gear = false;
        f.run(9.0);
        assert!(f.said("inbound"), "{:?}", f.calls);
        assert!(f.said("expect overhead runway 36, report initial"));
        // The scenery's runways come in another order.
        f.bases[0].ends.reverse();
        // Fly up to the initial point.
        f.pos = DVec2::new(0.0, -700.0 - INITIAL - 1000.0);
        f.run(3.0);
        assert!(f.said("initial, runway 36"), "{:?}", f.calls);
        assert!(f.said("break at the numbers"));
        // The break and downwind (skipped), then on final with the gear down.
        f.pos = DVec2::new(0.0, -700.0 - 3000.0);
        f.alt = 800.0;
        f.knots = 150.0;
        f.gear = true;
        f.run(3.0);
        assert!(f.said("base, gear down"));
        assert!(f.said("runway 36, cleared to land"));
        f.ground = true;
        f.knots = 30.0;
        f.alt = 0.0;
        f.run(3.0);
        assert!(f.said("welcome home"));
        assert_eq!(f.atc.phase, Phase::Landed);
    }

    #[test]
    fn gear_up_on_final_is_sent_around() {
        let mut f = Flight::new(vec![field(true)]);
        f.fly(DVec2::new(0.0, -30_000.0), 2000.0, 0.0, 300.0);
        f.run(9.0);
        f.pos = DVec2::new(0.0, -700.0 - INITIAL - 1000.0);
        f.run(3.0);
        f.gear = false;
        f.alt = 800.0;
        f.knots = 160.0;
        f.pos = DVec2::new(0.0, -700.0 - 3400.0);
        f.run(30.0);
        assert!(f.said("check gear down"), "{:?}", f.calls);
        assert!(f.said("go around, go around, your gear is up"));
        assert_eq!(f.atc.phase, Phase::ToInitial);
    }

    #[test]
    fn landing_waits_for_the_runway() {
        let mut f = Flight::new(vec![field(true)]);
        f.fly(DVec2::new(0.0, -700.0 - 4000.0), 900.0, 0.0, 0.0);
        f.atc.phase = Phase::ToInitial;
        f.atc.base = Some(5);
        f.atc.end = Some(f.bases[0].ends[0]);
        f.traffic = vec![Traffic { pos: DVec2::new(0.0, 0.0), alt_ft: 0.0, on_ground: true }];
        f.run(4.0);
        assert!(f.said("continue, traffic on the runway"), "{:?}", f.calls);
        f.traffic.clear();
        f.run(3.0);
        assert!(f.said("cleared to land"));
    }

    #[test]
    fn gca_vectors_and_talkdown() {
        let mut f = Flight::new(vec![field(true)]);
        f.ifr = true;
        // East of the field, heading west.
        f.fly(DVec2::new(30_000.0, -10_000.0), 6000.0, 270.0, 250.0);
        f.run(9.0);
        assert!(f.said("expect GCA runway 36"), "{:?}", f.calls);
        assert!(f.said("descend and maintain 2,000"));
        // Established on the final, drifting left of course, a little high.
        f.alt = 2400.0;
        f.pos = DVec2::new(-300.0, -700.0 - 12_000.0);
        f.heading = 2.0;
        f.knots = 0.0;
        f.run(3.0);
        assert!(f.said("on course"), "{:?}", f.calls);
        assert_eq!(f.atc.phase, Phase::Talkdown);
        f.pos = DVec2::new(-150.0, -700.0 - 9_000.0);
        f.run(6.0);
        assert!(f.said("slightly left of course, turn heading"), "{:?}", f.calls);
        assert!(f.said("5 miles from touchdown"));
        // Down the glidepath to decision height.
        f.pos = DVec2::new(0.0, -700.0 - 7000.0);
        f.alt = ((7000.0 * GLIDE.to_radians().tan()) / WU) as f32;
        f.run(6.0);
        assert!(f.said("on course, on glidepath"), "{:?}", f.calls);
        assert!(f.said("cleared to land runway 36"));
        f.pos = DVec2::new(0.0, -700.0 - 1000.0);
        f.alt = 150.0;
        f.run(3.0);
        assert!(f.said("at decision height"));
        assert_eq!(f.atc.phase, Phase::Final);
    }

    #[test]
    fn closed_field_diverts() {
        let mut f = Flight::new(vec![field(false), other()]);
        f.fly(DVec2::new(0.0, -30_000.0), 3000.0, 0.0, 300.0);
        f.run(9.0);
        assert!(f.said("Gutersloh is closed, divert Wildenrath"), "{:?}", f.calls);
        assert_eq!(f.atc.divert, Some(9));
    }

    #[test]
    fn runway_numbers_and_headings() {
        let f = field(true);
        assert_eq!(f.ends[0].name(), "36");
        assert_eq!(f.ends[1].name(), "18");
        assert_eq!(heading_text(0.0), "360");
        assert_eq!(heading_text(242.0), "240");
        assert_eq!(compass(DVec2::new(1.0, 1.0)), "northeast");
        assert!((angle_diff(10.0, 350.0) - 20.0).abs() < 1e-9);
    }
}

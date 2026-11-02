//! Flight recorder: the player's track with his radar exposure, the spans in which each site
//! tracked him, the enemy aircraft and missiles and the main events of the flight. It feeds the
//! debrief map and the Tacview export; it only reads the simulation.

use std::collections::HashMap;

use stealth_sim::aaa::GroundKind;
use stealth_sim::data::AircraftKind;
use stealth_sim::ground::GroundGun;
use stealth_sim::uav::{Uav, UavMode};
use stealth_sim::{
    Countermeasure, FlightUnit, Launcher, PlayerView, Projectile, Sim, SimEvent, oflags,
};

/// Sim frames per second.
pub const FS: u32 = crate::combat::FS as u32;
/// Aircraft and missiles are kept every this many frames (5 Hz).
const CONTACT_EVERY: u32 = 3;

/// The player in one sim frame. Saved as a row of its values (15 a second make most of a saved
/// game).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(into = "SampleRow", from = "SampleRow")]
pub struct Sample {
    pub frame: u32,
    /// World units (x east, y north).
    pub wx: i32,
    pub wy: i32,
    /// True altitude (ft).
    pub alt: i32,
    /// 16-bit angles.
    pub heading: i32,
    pub pitch: i32,
    pub roll: i32,
    pub knots: i32,
    pub emv: i32,
    /// Best site score + EMV - 100: detected when > 0.
    pub margin: i32,
    /// The site with that score.
    pub site: Option<usize>,
}

type SampleRow = (u32, [i32; 9], Option<usize>);

impl From<Sample> for SampleRow {
    fn from(s: Sample) -> Self {
        let v = [s.wx, s.wy, s.alt, s.heading, s.pitch, s.roll, s.knots, s.emv, s.margin];
        (s.frame, v, s.site)
    }
}

impl From<SampleRow> for Sample {
    fn from((frame, v, site): SampleRow) -> Self {
        let [wx, wy, alt, heading, pitch, roll, knots, emv, margin] = v;
        Self { frame, wx, wy, alt, heading, pitch, roll, knots, emv, margin, site }
    }
}

impl Sample {
    /// Map position (x east, y south, map units).
    pub fn map(&self) -> (f32, f32) {
        (self.wx as f32 / 32.0, 0x8000 as f32 - self.wy as f32 / 32.0)
    }
}

/// Everything else in one frame: the other aircraft and pilots, the ground units, weapons in
/// flight and the player's own state (Tacview).
#[derive(Debug, Clone, Default)]
pub struct Contacts {
    pub frame: u32,
    pub aircraft: Vec<Air>,
    pub missiles: Vec<(usize, Projectile)>,
    /// The low-level air defences and vehicles by `Sim::units` index.
    pub ground: Vec<(usize, GroundSnap)>,
    /// The other pilots (not the player, not hanging under a parachute).
    pub pilots: Vec<PilotSnap>,
    /// Gun rounds in flight.
    pub bullets: Vec<Round>,
    /// The player's flares, chaff and decoys (by slot).
    pub cms: Vec<(usize, Countermeasure)>,
    pub player: Telemetry,
}

/// An AI flight unit (by `Sim::flights` index).
#[derive(Debug, Clone)]
pub struct Air {
    pub index: usize,
    pub unit: FlightUnit,
    pub kind: AircraftKind,
    pub friendly: bool,
    /// The drone it is, if one, and the ground's height (m) under each of its waypoints.
    pub uav: Option<Uav>,
    pub route_ground: Vec<f64>,
    /// A drone the player flies by hand at its command center's console.
    pub flown_by_player: bool,
}

/// A ground unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroundSnap {
    pub kind: GroundKind,
    /// Metres east, north, up (on the rolling hills).
    pub pos: [f64; 3],
    /// Hull heading (rad, clockwise from north).
    pub heading: f32,
    pub blue: bool,
    pub destroyed: bool,
    pub gone: bool,
    /// Its convoy (0: none).
    pub convoy: u32,
}

/// Another pilot.
#[derive(Debug, Clone)]
pub struct PilotSnap {
    pub id: u8,
    pub view: PlayerView,
    pub aircraft: crate::aircraft::AircraftKind,
    pub missiles: Vec<(u8, Projectile)>,
    pub cms: [Countermeasure; 4],
}

/// A gun round: who fired it, what tells it from the next one in its slot, where it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Round {
    pub owner: RoundOwner,
    /// The slot (the player's and the enemy's), and its velocity.
    pub key: (usize, i32, i32),
    /// Map units (x east, y south) and altitude (ft).
    pub pos: (f64, f64, f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundOwner {
    Player,
    Enemy,
    OtherPilot,
}

/// The player's aircraft beyond its track.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Telemetry {
    /// Height above the ground (ft).
    pub agl: f32,
    pub fuel_lb: f32,
    /// Throttle (0..1; 1 full dry) and the afterburner lit (0..1).
    pub throttle: f32,
    pub afterburner: f32,
    pub gear_down: bool,
}

/// What Tacview shows of an event (only kept for the flight being flown).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TacEvent {
    /// An AI aircraft (flight unit) was destroyed or crashed.
    AirDestroyed(usize),
    TakeOff(usize),
    Landed(usize),
    PlayerLanded,
    /// A world object was destroyed: where (map units).
    GroundDestroyed {
        object: usize,
        x: f32,
        y: f32,
    },
    /// A ground gun fired: from, to (metres east, north, up), seconds of flight.
    GroundShot {
        gun: GroundGun,
        from: [f64; 3],
        to: [f64; 3],
        secs: f64,
        blue: bool,
    },
    /// An S-60 shell burst (map units, altitude ft).
    Flak {
        x: f32,
        y: f32,
        alt: f32,
    },
}

/// Something that happened, marked on the map where the player was.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MarkKind {
    /// A site started tracking the player.
    Tracked(usize),
    /// A missile was fired at the player (by a site if known).
    Launch {
        site: Option<usize>,
        spec: i32,
    },
    Hit,
    AirKill {
        spec: i32,
    },
    GroundKill {
        object: usize,
    },
    /// A low-level air defence unit (`Sim::units` index) was destroyed.
    AaaKill(usize),
    Photo,
    Objective {
        primary: bool,
    },
    Ejected,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Mark {
    pub frame: u32,
    /// Map units (x east, y south).
    pub x: f32,
    pub y: f32,
    pub kind: MarkKind,
}

/// A span of frames in which a site tracked the player (`end` None: until the end).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub site: usize,
    pub start: u32,
    pub end: Option<u32>,
}

impl Span {
    pub fn covers(&self, frame: u32) -> bool {
        frame >= self.start && self.end.is_none_or(|e| frame < e)
    }
}

/// A radar site as it was at takeoff.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SiteInfo {
    /// Map units (x east, y south).
    pub x: f32,
    pub y: f32,
    /// Radar type ("SA-6").
    pub kind: String,
    /// Where it is ("Brest").
    pub place: String,
    /// Missile lock range (map units).
    pub lock: f32,
}

/// The whole flight. A saved game keeps it without the contacts (the bulk of it: a saved
/// flight goes on with those from the load on).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FlightRecord {
    pub samples: Vec<Sample>,
    #[serde(skip)]
    pub contacts: Vec<Contacts>,
    pub marks: Vec<Mark>,
    pub spans: Vec<Span>,
    /// The ground units by object index (empty names: not a radar).
    pub sites: Vec<SiteInfo>,
    /// The low-level air defences: what, where (map units) and whether friendly.
    pub aaa: Vec<(stealth_sim::aaa::GroundKind, i32, i32, bool)>,
    /// Open span per site.
    open: Vec<Option<usize>>,
    frame: u32,
    /// Events for Tacview by frame.
    #[serde(skip)]
    pub events: Vec<(u32, TacEvent)>,
    /// The player's state for the next sample (set by the caller before each).
    #[serde(skip)]
    pub telemetry: Telemetry,
    /// Pilot names by id (multiplayer; set by the caller).
    #[serde(skip)]
    pub names: HashMap<u8, String>,
    /// The player's pilot id.
    #[serde(skip)]
    pub pilot: u8,
}

/// Exposure totals for the debrief.
#[derive(Debug, Clone, Copy, Default)]
pub struct Exposure {
    /// Seconds in which at least one site tracked the player.
    pub tracked_s: f32,
    pub peak_margin: i32,
    /// The site that tracked the player longest, and for how many seconds.
    pub worst: Option<(usize, f32)>,
}

/// Best detection margin of all live sites against the player now, and its site.
pub fn margin(sim: &Sim) -> (i32, Option<usize>) {
    let emv = sim.emv();
    (3..sim.ground_unit_count())
        .filter(|&i| sim.objects()[i].unit_type > 0)
        .map(|i| (sim.site_score(i), i))
        .filter(|&(s, _)| s > 0)
        .max_by_key(|&(s, _)| s)
        .map_or((-100, None), |(s, i)| (s + emv - 100, Some(i)))
}

impl FlightRecord {
    pub fn seconds(frame: u32) -> f32 {
        frame as f32 / FS as f32
    }

    /// Frame of the last sample.
    pub fn last_frame(&self) -> u32 {
        self.samples.last().map_or(0, |s| s.frame)
    }

    /// The sample at or just before `frame`.
    pub fn at(&self, frame: u32) -> Option<&Sample> {
        let i = self.samples.partition_point(|s| s.frame <= frame);
        self.samples.get(i.saturating_sub(1))
    }

    fn here(&self) -> (f32, f32) {
        self.samples.last().map_or((0.0, 0.0), Sample::map)
    }

    fn mark(&mut self, kind: MarkKind) {
        let (x, y) = self.here();
        self.marks.push(Mark { frame: self.frame, x, y, kind });
    }

    /// Records the state after a sim step (call once per step, after its events).
    pub fn sample(&mut self, sim: &Sim) {
        if self.sites.is_empty() {
            self.sites = (0..sim.ground_unit_count())
                .map(|i| {
                    let o = sim.objects()[i];
                    let radar = i >= 3 && o.unit_type > 0;
                    SiteInfo {
                        x: o.x as f32,
                        y: o.y as f32,
                        kind: if radar {
                            sim.threat_name(o.unit_type).into()
                        } else {
                            String::new()
                        },
                        place: if radar { sim.place_name(i) } else { String::new() },
                        lock: sim.site_lock_range(o.unit_type) as f32 * 64.0,
                    }
                })
                .collect();
            self.aaa = sim
                .units()
                .iter()
                .enumerate()
                .map(|(k, u)| (u.kind, u.x, u.y, sim.aaa_blue(k)))
                .collect();
        }
        for (i, u) in sim.units().iter().enumerate() {
            if u.destroyed && !u.gone && !self.marks.iter().any(|m| m.kind == MarkKind::AaaKill(i))
            {
                let (x, y) = (u.x as f32, u.y as f32);
                self.marks.push(Mark { frame: self.frame, x, y, kind: MarkKind::AaaKill(i) });
            }
        }
        let p = sim.player();
        let (margin, site) = margin(sim);
        self.samples.push(Sample {
            frame: self.frame,
            wx: p.wx,
            wy: p.wy,
            alt: p.alt,
            heading: p.heading,
            pitch: p.pitch,
            roll: p.roll,
            knots: p.knots,
            emv: sim.emv(),
            margin,
            site,
        });
        let n = sim.ground_unit_count();
        self.open.resize(n, None);
        for i in 0..n {
            let tracking = sim.objects()[i].flags & oflags::TRACKING != 0;
            match (tracking, self.open[i]) {
                (true, None) => {
                    self.open[i] = Some(self.spans.len());
                    self.spans.push(Span { site: i, start: self.frame, end: None });
                    self.mark(MarkKind::Tracked(i));
                }
                (false, Some(k)) => {
                    self.spans[k].end = Some(self.frame);
                    self.open[i] = None;
                }
                _ => {}
            }
        }
        self.pilot = sim.active_pilot();
        if self.frame.is_multiple_of(CONTACT_EVERY) {
            let c = self.contacts(sim);
            self.contacts.push(c);
        }
        self.frame += 1;
    }

    fn contacts(&self, sim: &Sim) -> Contacts {
        let me = sim.active_pilot();
        let manned = sim.station_of(me).map(|st| st.unit);
        let hills = sim.hills();
        // As drawn: not the destroyed ones nor those not yet placed.
        let aircraft = sim
            .active_aircraft()
            .into_iter()
            .filter(|(_, u, _)| {
                u.flags & stealth_sim::fflags::DESTROYED == 0 && (u.wx != 0 || u.wy != 0)
            })
            .map(|(i, u, _)| {
                let uav = sim.uav_of_flight(i).cloned();
                let flown_by_player = uav
                    .as_ref()
                    .is_some_and(|d| Some(d.center) == manned && d.mode == UavMode::Manual);
                let route_ground = uav.as_ref().map_or_else(Vec::new, |d| {
                    d.route
                        .iter()
                        .map(|wp| {
                            let [e, n] = wp.metres();
                            hills.height_m(e, n) as f64
                        })
                        .collect()
                });
                Air {
                    route_ground,
                    index: i,
                    unit: u,
                    kind: sim.aircraft_kind(u.spec),
                    friendly: sim.friendly_unit(i, u.spec),
                    uav,
                    flown_by_player,
                }
            })
            .collect();
        let ground = sim
            .units()
            .iter()
            .enumerate()
            .map(|(k, u)| {
                let snap = GroundSnap {
                    kind: u.kind,
                    pos: u.metres_on(&hills),
                    heading: u
                        .fine
                        .map_or_else(|| stealth_sim::aaa::unit_heading(k), |f| f.heading),
                    blue: sim.aaa_blue(k),
                    destroyed: u.destroyed,
                    gone: u.gone,
                    convoy: sim.unit_convoy(k),
                };
                (k, snap)
            })
            .collect();
        let pilots = sim
            .remote_pilots()
            .into_iter()
            .filter(|p| p.id != me && p.chute.is_none())
            .map(|p| PilotSnap {
                id: p.id,
                view: p.view,
                aircraft: crate::aircraft::AircraftKind::of_remote(&p),
                missiles: p.missiles.clone(),
                cms: p.cms,
            })
            .collect();
        let (rounds, n) = sim.bullets();
        let mut bullets: Vec<Round> = rounds
            .iter()
            .enumerate()
            .filter(|(_, b)| b.x != 0)
            .map(|(k, b)| Round {
                owner: if k < n { RoundOwner::Player } else { RoundOwner::Enemy },
                key: (k, b.vx, b.vy),
                pos: b.map_pos(),
            })
            .collect();
        bullets.extend(sim.other_rounds().iter().map(|b| Round {
            owner: RoundOwner::OtherPilot,
            key: (usize::MAX, b.vx, b.vy),
            pos: b.map_pos(),
        }));
        Contacts {
            frame: self.frame,
            aircraft,
            missiles: sim.active_projectiles().into_iter().map(|(s, m, _, _)| (s, m)).collect(),
            ground,
            pilots,
            bullets,
            cms: sim
                .countermeasures()
                .iter()
                .copied()
                .enumerate()
                .filter(|(_, c)| c.ttl > 0)
                .collect(),
            player: Telemetry { agl: sim.player().height as f32, ..self.telemetry },
        }
    }

    /// Notes for Tacview an event of the step about to be sampled.
    fn tac_event(&mut self, sim: &Sim, e: &SimEvent) {
        let ev = match *e {
            SimEvent::AircraftDestroyed { unit, .. }
            | SimEvent::AircraftCrashed { unit }
            | SimEvent::CampaignFlightDestroyed { unit, .. } => TacEvent::AirDestroyed(unit),
            SimEvent::AircraftTakeoff { unit, .. } => TacEvent::TakeOff(unit),
            SimEvent::CampaignFlightLanded { unit, .. }
            | SimEvent::CampaignFlightDelivered { unit, .. } => TacEvent::Landed(unit),
            SimEvent::LandedAtBase { .. } => TacEvent::PlayerLanded,
            SimEvent::GroundDestroyed { object, .. } => {
                let Some(o) = sim.objects().get(object) else { return };
                TacEvent::GroundDestroyed { object, x: o.x as f32, y: o.y as f32 }
            }
            SimEvent::GroundShot { from, to, gun, hit, miss, .. } => {
                let (Some(a), Some(b)) = (sim.units().get(from), sim.units().get(to)) else {
                    return;
                };
                let hills = sim.hills();
                let (from_m, mut to_m) = (a.metres_on(&hills), b.metres_on(&hills));
                if !hit {
                    to_m[0] += miss.0 as f64;
                    to_m[1] += miss.1 as f64;
                    to_m[2] = hills.height_m(to_m[0], to_m[1]) as f64;
                }
                let d = ((to_m[0] - from_m[0]).powi(2) + (to_m[1] - from_m[1]).powi(2)).sqrt();
                TacEvent::GroundShot {
                    gun,
                    from: from_m,
                    to: to_m,
                    secs: gun.flight_time(d),
                    blue: sim.aaa_blue(from),
                }
            }
            SimEvent::Flak { x, y, alt } => {
                TacEvent::Flak { x: x as f32, y: y as f32, alt: alt as f32 }
            }
            _ => return,
        };
        self.events.push((self.frame, ev));
    }

    /// Notes an event of the step about to be sampled.
    pub fn event(&mut self, sim: &Sim, e: &SimEvent) {
        self.tac_event(sim, e);
        let kind = match *e {
            SimEvent::MissileLaunched { spec, launcher, .. } => match launcher {
                Launcher::Site(i) => MarkKind::Launch { site: Some(i), spec },
                Launcher::Aircraft(_) | Launcher::Manpads(_) => {
                    MarkKind::Launch { site: None, spec }
                }
                Launcher::Player => return,
            },
            SimEvent::PlayerHit { .. } => MarkKind::Hit,
            SimEvent::AircraftDestroyed { spec, .. } => MarkKind::AirKill { spec },
            SimEvent::GroundDestroyed { object, .. } => {
                // Where the object was, not where the player was.
                if let Some(o) = sim.objects().get(object) {
                    let kind = MarkKind::GroundKill { object };
                    self.marks.push(Mark { frame: self.frame, x: o.x as f32, y: o.y as f32, kind });
                }
                return;
            }
            SimEvent::PhotoTaken { .. } => MarkKind::Photo,
            SimEvent::ObjectiveDone { primary } => MarkKind::Objective { primary },
            SimEvent::Ejected { success: true } => MarkKind::Ejected,
            _ => return,
        };
        self.mark(kind);
    }

    pub fn exposure(&self) -> Exposure {
        let end = self.frame;
        let mut per_site: Vec<u32> = Vec::new();
        for s in &self.spans {
            if per_site.len() <= s.site {
                per_site.resize(s.site + 1, 0);
            }
            per_site[s.site] += s.end.unwrap_or(end) - s.start;
        }
        let tracked = (0..end).filter(|&f| self.spans.iter().any(|s| s.covers(f))).count();
        let worst = per_site
            .iter()
            .enumerate()
            .filter(|&(_, &n)| n > 0)
            .max_by_key(|&(_, &n)| n)
            .map(|(i, &n)| (i, Self::seconds(n)));
        Exposure {
            tracked_s: Self::seconds(tracked as u32),
            peak_margin: self.samples.iter().map(|s| s.margin).max().unwrap_or(-100),
            worst,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_and_exposure() {
        let mut r = FlightRecord::default();
        r.spans = vec![
            Span { site: 4, start: 0, end: Some(30) },
            Span { site: 5, start: 15, end: Some(45) },
            Span { site: 4, start: 60, end: None },
        ];
        r.frame = 90;
        let e = r.exposure();
        // Frames 0..45 and 60..90.
        assert_eq!(e.tracked_s, 75.0 / FS as f32);
        assert_eq!(e.worst.map(|w| w.0), Some(4));
        assert!(r.spans[2].covers(89) && !r.spans[0].covers(30));
    }
}

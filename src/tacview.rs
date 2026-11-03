//! Tacview export (`--tacview`, the menu's "Tacview export"): the recorded flight as an ACMI 2.2
//! text file in `<data dir>/stealth-mission-89/tacview`. The theater is laid on the globe at
//! game scale around one real airbase ([`crate::world::geo_origin`]). The same text goes to
//! Tacview while the flight is flown with the real-time telemetry (tacview_real_time.rs).
//!
//! Besides the player it shows the AI aircraft (drones at their exact position, with their call
//! sign, locked target and route as waypoints), the other pilots, the ground units as they move
//! (each unit its own object though `Sim::units` slots are reused), missiles and bombs with who
//! fired them, gun rounds, ground guns' shells (shooter to impact), flak bursts and flares/chaff.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write;
use std::path::PathBuf;

use bevy::log::{info, warn};
use stealth_mission::Mission;
use stealth_mission::world::flags as F;
use stealth_sim::aaa::GroundKind;
use stealth_sim::data::AircraftKind;
use stealth_sim::ground::GroundGun;
use stealth_sim::uav::{UavKind, UavWp};
use stealth_sim::{Countermeasure, Projectile, SimData};

use crate::recorder::{
    Contacts, FS, FlightRecord, GroundSnap, MarkKind, RoundOwner, TacEvent, Telemetry,
};
use crate::world::WU;

const FT: f64 = 0.3048;
const LB: f64 = 0.453_592;
/// Object ids: the player, the other pilots (+ id), AI aircraft (+ flight unit), missiles
/// (counted up), radar sites (+ object), airfields (+ object), buildings (+ object) and the
/// rest (ground units, gun rounds, countermeasures, waypoints, shells: counted up).
const PLAYER: u32 = 0x100;
const PILOTS: u32 = 0x200;
const AIRCRAFT: u32 = 0x1000;
const MISSILES: u32 = 0x2000;
const SITES: u32 = 0x8000;
const AIRFIELDS: u32 = 0x9000;
const BUILDINGS: u32 = 0xC000;
const DYNAMIC: u32 = 0x100000;
/// Dropped equipment (the missile table's).
const EQUIPMENT_SPEC: i32 = 38;

/// Degrees of a 16-bit angle (signed for pitch and roll).
fn deg(bam: i32, signed: bool) -> f64 {
    let v = if signed { bam as i16 as f64 } else { bam as u16 as f64 };
    v / 65536.0 * 360.0
}

/// The altitude (ft) of a viewZ (`stealth_sim::math::view_z` inverted).
fn true_altitude(z: f64) -> f64 {
    if z < 8192.0 {
        z
    } else if z < 12288.0 {
        8192.0 + (z - 8192.0) * 2.0
    } else {
        16384.0 + (z - 12288.0) * 4.0
    }
}

/// The theater on the globe: offsets (degrees) from its south-west corner.
struct Geo {
    lat0: f64,
}

impl Geo {
    /// Longitude and latitude offsets of a point (metres east and north of the corner).
    fn offset(&self, east: f64, north: f64) -> (f64, f64) {
        let lat = north / 111_200.0;
        let lon = east / (111_320.0 * (self.lat0 + lat).to_radians().cos());
        (lon, lat)
    }

    /// `T=` value of a point in metres east, north and up.
    fn metres_t(&self, [e, n, up]: [f64; 3]) -> String {
        let (lon, lat) = self.offset(e, n);
        format!("{lon:.6}|{lat:.6}|{up:.1}")
    }

    /// The same with a heading (rad, clockwise from north).
    fn metres_t_hdg(&self, p: [f64; 3], heading: f32) -> String {
        format!("{}|0|0|{:.1}", self.metres_t(p), (heading as f64).to_degrees().rem_euclid(360.0))
    }

    /// `T=` value of a point in map units (x east, y south) at an altitude (ft).
    fn map_t(&self, x: f64, y: f64, alt_ft: f64) -> String {
        let (lon, lat) = self.offset(x * 32.0 * WU, (0x8000 as f64 - y) * 32.0 * WU);
        format!("{lon:.6}|{lat:.6}|{:.1}", alt_ft * FT)
    }
}

/// Days since 1970-01-01 of a civil date, and back (Howard Hinnant's algorithms).
pub(crate) fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub(crate) fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + (m <= 2) as i64, m, d)
}

/// The briefing date ("June 5, 1991"), else 1 January 1990.
pub(crate) fn briefing_date(text: &str) -> (i64, i64, i64) {
    parse_briefing_date(text).unwrap_or((1990, 1, 1))
}

/// The briefing's date (year, month, day), if it has one.
pub(crate) fn parse_briefing_date(text: &str) -> Option<(i64, i64, i64)> {
    const MONTHS: [&str; 12] =
        ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
    let lower = text.to_ascii_lowercase();
    let words: Vec<&str> = lower.split([' ', ',']).filter(|w| !w.is_empty()).collect();
    let first = words.first()?;
    let m = MONTHS.iter().position(|m| first.starts_with(m))?;
    Some((words.get(2)?.parse().ok()?, m as i64 + 1, words.get(1)?.parse().ok()?))
}

/// ISO 8601 UTC time of the mission start: the briefing date and the local mission clock,
/// shifted by the longitude (local solar time).
fn reference_time(m: &Mission, lon: f64) -> String {
    let (y, mo, d) = briefing_date(&m.briefing.date);
    let local = m.time.base_hour as f64 * 3600.0 + m.time.tod_ticks as f64 * 2.0;
    let utc = (days_from_civil(y, mo, d) as f64 * 86_400.0 + local - lon / 15.0 * 3600.0) as i64;
    let (y, mo, d) = civil_from_days(utc.div_euclid(86_400));
    let s = utc.rem_euclid(86_400);
    format!("{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}Z", s / 3600, s / 60 % 60, s % 60)
}

fn escape(s: &str) -> String {
    s.replace(',', "\\,")
}

/// The ACMI text of a recorded flight.
pub fn acmi(rec: &FlightRecord, m: &Mission, data: &SimData, aircraft: &str) -> String {
    let mut out = String::new();
    let mut writer = AcmiWriter::new(m);
    writer.advance(rec, m, data, aircraft, &mut out);
    writer.start(rec, m, &mut out);
    out
}

/// Writes a flight record as ACMI text while it grows: each call adds what was recorded since
/// the previous one (the whole flight for the export, the last frames for the real-time telemetry).
pub struct AcmiWriter {
    geo: Geo,
    lon0: f64,
    started: bool,
    /// Samples, contacts, marks and events already written.
    samples: usize,
    contacts: usize,
    marks: usize,
    events: usize,
    /// AI aircraft by flight unit.
    planes: HashMap<usize, Plane>,
    /// Missile slot (the other pilots': `REMOTE_SLOT + id * 256 + slot`) -> (object id, spec,
    /// launcher).
    missiles: HashMap<usize, (u32, i32, i32)>,
    next_missile: u32,
    /// The other pilots shown.
    pilots: HashSet<u8>,
    /// Ground units by `Sim::units` slot (slots are reused: each unit gets its own id).
    ground: HashMap<usize, Ground>,
    /// Gun rounds in flight by owner and key.
    rounds: HashMap<(RoundOwner, (usize, i32, i32)), u32>,
    /// Countermeasures by (pilot, slot; the player's pilot 255) -> (id, ttl).
    cms: HashMap<(u8, usize), (u32, i32)>,
    /// Buildings already shown (objectives, destroyed objects).
    buildings: HashSet<usize>,
    /// Lines due at a later frame (shells landing, bursts fading).
    pending: BTreeMap<u32, String>,
    next_id: u32,
}

/// An AI aircraft shown, and a drone's locked target and route.
#[derive(Default)]
struct Plane {
    lock: Option<u16>,
    route: Vec<UavWp>,
    waypoints: Vec<u32>,
}

/// A ground unit shown.
struct Ground {
    id: u32,
    kind: GroundKind,
    last: GroundSnap,
    /// Shown (not destroyed nor gone).
    alive: bool,
}

const REMOTE_SLOT: usize = 0x10000;

impl AcmiWriter {
    pub fn new(m: &Mission) -> Self {
        let (lon0, lat0) = crate::world::geo_origin(&m.theater_code.to_ascii_lowercase());
        Self {
            geo: Geo { lat0 },
            lon0,
            started: false,
            samples: 0,
            contacts: 0,
            marks: 0,
            events: 0,
            planes: HashMap::new(),
            missiles: HashMap::new(),
            next_missile: MISSILES,
            pilots: HashSet::new(),
            ground: HashMap::new(),
            rounds: HashMap::new(),
            cms: HashMap::new(),
            buildings: HashSet::new(),
            pending: BTreeMap::new(),
            next_id: DYNAMIC,
        }
    }

    fn fresh_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    /// The header and the objects of the whole flight (the record has them from its first
    /// sample on), once.
    fn start(&mut self, rec: &FlightRecord, m: &Mission, w: &mut String) {
        if self.started {
            return;
        }
        self.started = true;
        let (lon0, lat0, geo) = (self.lon0, self.geo.lat0, &self.geo);
        let _ = writeln!(w, "FileType=text/acmi/tacview\nFileVersion=2.2");
        let _ = writeln!(w, "0,ReferenceTime={}", reference_time(m, lon0));
        let _ = writeln!(w, "0,ReferenceLongitude={lon0}\n0,ReferenceLatitude={lat0}");
        let _ = writeln!(w, "0,DataSource=Stealth Mission 89");
        let _ = writeln!(
            w,
            "0,Title={}",
            escape(&crate::mission::clean_text(&format!(
                "{} - {}",
                m.theater_name, m.briefing.title
            )))
        );
        let _ = writeln!(w, "#0");
        // The radar sites (with their missiles' reach) and the airfields, for the whole flight.
        for (i, s) in rec.sites.iter().enumerate().filter(|(_, s)| !s.kind.is_empty()) {
            let _ = writeln!(
                w,
                "{:x},T={},Type=Ground+Static+AntiAircraft,Name={},CallSign={},Coalition=Enemies,Color=Red,EngagementRange={:.0}",
                SITES + i as u32,
                geo.map_t(s.x as f64, s.y as f64, 0.0),
                escape(&s.kind),
                escape(&crate::mission::clean_text(&s.place)),
                s.lock as f64 * 32.0 * WU,
            );
        }
        for (i, o) in m.world.objects.iter().enumerate().filter(|(_, o)| o.flags & F::AIRFIELD != 0)
        {
            let name = m.world.names.get(o.name as usize).map_or("", String::as_str);
            let (coalition, colour) =
                if o.flags & F::FRIENDLY != 0 { ("Allies", "Blue") } else { ("Enemies", "Red") };
            let _ = writeln!(
                w,
                "{:x},T={},Type=Ground+Static+Aerodrome,Name={},Coalition={coalition},Color={colour}",
                AIRFIELDS + i as u32,
                geo.map_t(o.x as u16 as f64, o.y as u16 as f64, 0.0),
                escape(&crate::mission::clean_text(name)),
            );
        }
        // The targets (not the radar sites, shown already).
        for (t, what) in [(&m.primary, "Primary target"), (&m.secondary, "Secondary target")] {
            let i = t.object_index as usize;
            if t.object_index < 3
                || rec.sites.get(i).is_some_and(|s| !s.kind.is_empty())
                || !self.buildings.insert(i)
            {
                continue;
            }
            let _ = writeln!(
                w,
                "{:x},T={},Type=Ground+Static+Building,Name={},CallSign={what},Coalition=Enemies,Color=Red",
                BUILDINGS + i as u32,
                self.geo.map_t(t.position.x as f64, t.position.y as f64, 0.0),
                escape(&crate::mission::clean_text(&t.shape_name)),
            );
        }
    }

    /// Whether `rec` started over since the last call (a new campaign sortie in the same flight).
    pub fn restarted(&self, rec: &FlightRecord) -> bool {
        rec.samples.len() < self.samples
    }

    /// Appends to `w` the frames recorded since the last call (nothing before the first sample).
    pub fn advance(
        &mut self,
        rec: &FlightRecord,
        m: &Mission,
        data: &SimData,
        aircraft: &str,
        w: &mut String,
    ) {
        if rec.samples.len() <= self.samples {
            return;
        }
        self.start(rec, m, w);
        for s in &rec.samples[self.samples..] {
            let _ = writeln!(w, "#{:.2}", s.frame as f64 / FS as f64);
            while let Some(e) = self.pending.first_entry().filter(|e| *e.key() <= s.frame) {
                w.push_str(&e.remove());
            }
            let (lon, lat) = self.geo.offset(s.wx as f64 * WU, s.wy as f64 * WU);
            let _ = write!(
                w,
                "{PLAYER:x},T={lon:.6}|{lat:.6}|{:.1}|{:.1}|{:.1}|{:.1}",
                s.alt as f64 * FT,
                deg(s.roll, true),
                deg(s.pitch, true),
                deg(s.heading, false),
            );
            if self.samples == 0 {
                let rotorcraft = crate::aircraft::AircraftKind::ALL
                    .iter()
                    .any(|k| k.name() == aircraft && k.spec().helicopter);
                let me = rec.names.get(&rec.pilot).map_or("Player", String::as_str);
                let _ = write!(
                    w,
                    ",Type=Air+{},Name={},Pilot={},Coalition=Allies,Color=Blue",
                    if rotorcraft { "Rotorcraft" } else { "FixedWing" },
                    escape(aircraft),
                    escape(me),
                );
            }
            self.samples += 1;
            let _ = writeln!(w, ",IAS={:.1}", s.knots as f64 * 0.514_444);

            while let Some(&(frame, ev)) = rec.events.get(self.events).filter(|e| e.0 <= s.frame) {
                self.events += 1;
                self.event(frame, ev, m, rec, w);
            }
            if let Some(c) = rec.contacts.get(self.contacts).filter(|c| c.frame == s.frame) {
                self.contacts += 1;
                self.player(&c.player, w);
                self.aircraft(c, data, w);
                self.missiles(c, data, rec.pilot, w);
                self.pilots(c, &rec.names, w);
                self.ground_units(c, w);
                self.rounds(c, w);
                self.countermeasures(c, s, w);
            }

            for sp in rec.spans.iter().filter(|sp| sp.end == Some(s.frame)) {
                let _ = writeln!(w, "{:x},LockedTarget=", SITES + sp.site as u32);
            }
            while let Some(mk) = rec.marks.get(self.marks).filter(|mk| mk.frame <= s.frame) {
                self.marks += 1;
                match mk.kind {
                    MarkKind::Hit => {
                        let _ = writeln!(w, "0,Event=Message|{PLAYER:x}|Hit");
                    }
                    MarkKind::Tracked(i) => {
                        let id = SITES + i as u32;
                        let _ = writeln!(w, "{id:x},LockedTarget={PLAYER:x}");
                        let _ = writeln!(w, "0,Event=Message|{id:x}|Tracking");
                    }
                    MarkKind::Ejected => {
                        let _ = writeln!(w, "0,Event=Message|{PLAYER:x}|Ejected");
                    }
                    _ => {}
                }
            }
        }
    }

    fn event(&mut self, frame: u32, ev: TacEvent, m: &Mission, rec: &FlightRecord, w: &mut String) {
        match ev {
            TacEvent::AirDestroyed(i) => {
                if self.planes.contains_key(&i) {
                    let _ = writeln!(w, "0,Event=Destroyed|{:x}|", AIRCRAFT + i as u32);
                }
            }
            TacEvent::TakeOff(i) => {
                if self.planes.contains_key(&i) {
                    let _ = writeln!(w, "0,Event=TakenOff|{:x}|", AIRCRAFT + i as u32);
                }
            }
            TacEvent::Landed(i) => {
                if self.planes.contains_key(&i) {
                    let _ = writeln!(w, "0,Event=Landed|{:x}|", AIRCRAFT + i as u32);
                }
            }
            TacEvent::PlayerLanded => {
                let _ = writeln!(w, "0,Event=Landed|{PLAYER:x}|");
            }
            TacEvent::GroundDestroyed { object, x, y } => {
                let id = if rec.sites.get(object).is_some_and(|s| !s.kind.is_empty()) {
                    SITES + object as u32
                } else {
                    let id = BUILDINGS + object as u32;
                    if self.buildings.insert(object) {
                        let name = [&m.primary, &m.secondary]
                            .into_iter()
                            .find(|t| t.object_index as usize == object)
                            .map_or("Building".to_string(), |t| {
                                crate::mission::clean_text(&t.shape_name)
                            });
                        let _ = writeln!(
                            w,
                            "{id:x},T={},Type=Ground+Static+Building,Name={},Coalition=Enemies,Color=Red",
                            self.geo.map_t(x as f64, y as f64, 0.0),
                            escape(&name),
                        );
                    }
                    id
                };
                let _ = writeln!(w, "0,Event=Destroyed|{id:x}|\n-{id:x}");
            }
            TacEvent::GroundShot { gun, from, to, secs, blue } => {
                let id = self.fresh_id();
                let (coalition, colour) = side(blue);
                let name = match gun {
                    GroundGun::Tank => "Tank round",
                    GroundGun::Cannon => "Autocannon round",
                    GroundGun::Flak => "Flak round",
                };
                let _ = writeln!(
                    w,
                    "{id:x},T={},Type=Weapon+Projectile+Shell,Name={name},Coalition={coalition},Color={colour}",
                    self.geo.metres_t(from),
                );
                let land = frame + ((secs * FS as f64).round() as u32).max(1);
                let line = format!("{id:x},T={}\n", self.geo.metres_t(to));
                self.due(land, line);
                self.due(land + 1, format!("-{id:x}\n"));
            }
            TacEvent::Flak { x, y, alt } => {
                let id = self.fresh_id();
                let _ = writeln!(
                    w,
                    "{id:x},T={},Type=Misc+Explosion,Name=Flak burst,Coalition=Enemies,Color=Red",
                    self.geo.map_t(x as f64, y as f64, alt as f64),
                );
                self.due(frame + FS, format!("-{id:x}\n"));
            }
        }
    }

    fn due(&mut self, frame: u32, line: String) {
        self.pending.entry(frame).or_default().push_str(&line);
    }

    /// The player's state beyond the track.
    fn player(&mut self, t: &Telemetry, w: &mut String) {
        let _ = writeln!(
            w,
            "{PLAYER:x},AGL={:.1},Throttle={:.2},Afterburner={:.2},LandingGear={},FuelWeight={:.0}",
            t.agl as f64 * FT,
            t.throttle,
            t.afterburner,
            t.gear_down as u8,
            t.fuel_lb as f64 * LB,
        );
    }

    fn aircraft(&mut self, c: &Contacts, data: &SimData, w: &mut String) {
        let now: Vec<usize> = c.aircraft.iter().map(|a| a.index).collect();
        let mut gone: Vec<usize> =
            self.planes.keys().copied().filter(|i| !now.contains(i)).collect();
        gone.sort_unstable();
        for gone in gone {
            let plane = self.planes.remove(&gone).unwrap_or_default();
            for wp in plane.waypoints {
                let _ = writeln!(w, "-{wp:x}");
            }
            let _ = writeln!(w, "-{:x}", AIRCRAFT + gone as u32);
        }
        for a in &c.aircraft {
            let id = AIRCRAFT + a.index as u32;
            let u = &a.unit;
            match &a.uav {
                // A drone: its exact flight.
                Some(d) => {
                    let (lon, lat) = self.geo.offset(d.pos[0], d.pos[1]);
                    let _ = write!(
                        w,
                        "{id:x},T={lon:.6}|{lat:.6}|{:.1}|{:.1}|{:.1}|{:.1},Throttle={:.2},FuelVolume={:.2}",
                        d.pos[2],
                        d.bank.to_degrees(),
                        d.pitch.to_degrees(),
                        d.heading.to_degrees().rem_euclid(360.0),
                        d.throttle(),
                        d.battery,
                    );
                }
                None => {
                    let alt = true_altitude(u.alt as f64);
                    let (lon, lat) =
                        self.geo.offset(u.wx as f64 * WU, (0x100000 - u.wy) as f64 * WU);
                    let _ = write!(
                        w,
                        "{id:x},T={lon:.6}|{lat:.6}|{:.1}|{:.1}|{:.1}|{:.1}",
                        alt * FT,
                        deg(u.bank, true),
                        deg(u.pitch, true),
                        deg(u.heading, false),
                    );
                }
            }
            let fresh = !self.planes.contains_key(&a.index);
            let plane = self.planes.entry(a.index).or_default();
            if fresh {
                let t = data.aircraft.get(u.spec.max(0) as usize);
                let (coalition, colour) = side(a.friendly);
                let ty = match (&a.uav, a.kind) {
                    (Some(d), _) if d.kind == UavKind::Interceptor => "Air+Rotorcraft",
                    (_, AircraftKind::Helicopter) => "Air+Rotorcraft",
                    _ => "Air+FixedWing",
                };
                let _ = write!(
                    w,
                    ",Type={ty},Name={},Coalition={coalition},Color={colour}",
                    escape(t.map_or("Aircraft", |t| t.name.as_str()))
                );
                if let Some(d) = &a.uav {
                    let _ = write!(w, ",CallSign={},Group=GCS {}", d.label(), d.center);
                }
            }
            if let Some(d) = &a.uav {
                if d.lock != plane.lock || fresh {
                    plane.lock = d.lock;
                    match d.lock {
                        Some(t) => {
                            let _ = write!(w, ",LockedTarget={:x}", AIRCRAFT + t as u32);
                        }
                        None if !fresh => w.push_str(",LockedTarget="),
                        None => {}
                    }
                }
                if a.flown_by_player {
                    w.push_str(",Pilot=Player");
                }
            }
            let _ = writeln!(w);
            if let Some(d) = &a.uav
                && d.route != plane.route
            {
                let old = std::mem::take(&mut plane.waypoints);
                plane.route = d.route.clone();
                for wp in old {
                    let _ = writeln!(w, "-{wp:x}");
                }
                let ids: Vec<u32> = (0..d.route.len()).map(|_| self.fresh_id()).collect();
                let (coalition, colour) = side(a.friendly);
                for (k, wp) in d.route.iter().enumerate() {
                    let [e, n] = wp.metres();
                    let _ = write!(
                        w,
                        "{:x},T={},Type=Navaid+Static+Waypoint,Name={} {},Coalition={coalition},Color={colour}",
                        ids[k],
                        self.geo.metres_t([
                            e,
                            n,
                            wp.alt as f64 + a.route_ground.get(k).copied().unwrap_or(0.0)
                        ]),
                        d.label(),
                        k + 1,
                    );
                    if let Some(next) = ids.get(k + 1) {
                        let _ = write!(w, ",Next={next:x}");
                    }
                    let _ = writeln!(w);
                }
                self.planes.get_mut(&a.index).unwrap().waypoints = ids;
            }
        }
    }

    fn missiles(&mut self, c: &Contacts, data: &SimData, me: u8, w: &mut String) {
        let mut live: Vec<(usize, &Projectile, Option<u8>)> =
            c.missiles.iter().map(|(s, p)| (*s, p, None)).collect();
        for p in &c.pilots {
            live.extend(
                p.missiles
                    .iter()
                    .map(|(s, m)| (REMOTE_SLOT + p.id as usize * 256 + *s as usize, m, Some(p.id))),
            );
        }
        let slots: HashSet<usize> = live.iter().map(|(s, ..)| *s).collect();
        let mut gone: Vec<(usize, u32)> = self
            .missiles
            .iter()
            .filter(|(s, _)| !slots.contains(s))
            .map(|(s, m)| (*s, m.0))
            .collect();
        gone.sort_unstable();
        for (slot, id) in gone {
            self.missiles.remove(&slot);
            let _ = writeln!(w, "-{id:x}");
        }
        for (slot, p, pilot) in live {
            let (x, y, z) = p.map_pos();
            let alt = if p.flight.is_some() { z } else { true_altitude(z) };
            let fresh = match self.missiles.get(&slot) {
                Some(&(id, spec, launcher)) if spec == p.spec && launcher == p.launcher => {
                    let _ = write!(w, "{id:x}");
                    false
                }
                old => {
                    if let Some(&(id, ..)) = old {
                        let _ = writeln!(w, "-{id:x}");
                    }
                    let id = self.next_missile;
                    self.next_missile += 1;
                    self.missiles.insert(slot, (id, p.spec, p.launcher));
                    let _ = write!(w, "{id:x}");
                    true
                }
            };
            let _ = write!(
                w,
                ",T={}|{:.1}|{:.1}|{:.1}",
                self.geo.map_t(x, y, alt),
                deg(p.roll, true),
                deg(p.pitch, true),
                deg(p.heading, false),
            );
            if fresh {
                let blue = pilot.is_some() || slot >= 8;
                let (coalition, colour) = side(blue);
                let t = data.missiles.get(p.spec.max(0) as usize);
                let ty = match t.map_or(0, |t| t.class) {
                    28 => "Weapon+Guided+Bomb",
                    29 | 30 => "Weapon+Bomb",
                    _ if p.spec == EQUIPMENT_SPEC => "Misc+Container",
                    _ => "Weapon+Missile",
                };
                let _ = write!(
                    w,
                    ",Type={ty},Name={},Coalition={coalition},Color={colour}",
                    escape(t.map_or("Missile", |t| t.name.as_str()))
                );
                // Who fired it.
                let parent = match pilot {
                    Some(id) => Some(PILOTS + id as u32),
                    None if slot >= 8 => Some(PLAYER),
                    None if p.launcher > 2 => Some(SITES + p.launcher as u32),
                    None if p.launcher < 0 && self.planes.contains_key(&(-p.launcher as usize)) => {
                        Some(AIRCRAFT + (-p.launcher) as u32)
                    }
                    None => None,
                };
                if let Some(parent) = parent {
                    let _ = write!(w, ",Parent={parent:x}");
                }
                // What it is after.
                let target = match p.target_pilot {
                    Some(id) if id == me => Some(PLAYER),
                    Some(id) => Some(PILOTS + id as u32),
                    None if !blue && p.fall.is_none() => Some(PLAYER),
                    None if p.target >= 0 && self.buildings.contains(&(p.target as usize)) => {
                        Some(BUILDINGS + p.target as u32)
                    }
                    None => None,
                };
                if let Some(t) = target {
                    let _ = write!(w, ",LockedTarget={t:x}");
                }
            }
            let _ = writeln!(w);
        }
    }

    fn pilots(&mut self, c: &Contacts, names: &HashMap<u8, String>, w: &mut String) {
        let mut gone: Vec<u8> = self
            .pilots
            .iter()
            .copied()
            .filter(|id| !c.pilots.iter().any(|p| p.id == *id))
            .collect();
        gone.sort_unstable();
        for gone in gone {
            self.pilots.remove(&gone);
            let _ = writeln!(w, "-{:x}", PILOTS + gone as u32);
        }
        for p in &c.pilots {
            let v = &p.view;
            let (lon, lat) = self.geo.offset(v.wx as f64 * WU, v.wy as f64 * WU);
            let _ = write!(
                w,
                "{:x},T={lon:.6}|{lat:.6}|{:.1}|{:.1}|{:.1}|{:.1},IAS={:.1}",
                PILOTS + p.id as u32,
                v.alt as f64 * FT,
                deg(v.roll, true),
                deg(v.pitch, true),
                deg(v.heading, false),
                v.knots as f64 * 0.514_444,
            );
            if self.pilots.insert(p.id) {
                let rotorcraft = p.aircraft.spec().helicopter;
                let name = names.get(&p.id).cloned().unwrap_or_else(|| format!("Pilot {}", p.id));
                let _ = write!(
                    w,
                    ",Type=Air+{},Name={},Pilot={},Coalition=Allies,Color=Blue",
                    if rotorcraft { "Rotorcraft" } else { "FixedWing" },
                    escape(p.aircraft.name()),
                    escape(&name),
                );
            }
            let _ = writeln!(w);
        }
    }

    fn ground_units(&mut self, c: &Contacts, w: &mut String) {
        for &(k, g) in &c.ground {
            let alive = !g.destroyed && !g.gone;
            let known = self.ground.get(&k).filter(|o| {
                // A slot that holds another unit now.
                o.kind == g.kind && (o.alive || !alive)
            });
            match known {
                Some(o) if o.alive && !alive => {
                    let id = o.id;
                    if g.gone {
                        let _ = writeln!(w, "-{id:x}");
                    } else {
                        let _ = writeln!(w, "0,Event=Destroyed|{id:x}|\n-{id:x}");
                    }
                    let o = self.ground.get_mut(&k).unwrap();
                    o.alive = false;
                    o.last = g;
                }
                Some(o) if alive => {
                    let moved = (o.last.pos[0] - g.pos[0]).hypot(o.last.pos[1] - g.pos[1]) > 1.0
                        || (o.last.heading - g.heading).abs().to_degrees() > 1.0;
                    if moved {
                        let _ =
                            writeln!(w, "{:x},T={}", o.id, self.geo.metres_t_hdg(g.pos, g.heading));
                        self.ground.get_mut(&k).unwrap().last = g;
                    }
                }
                Some(_) => {}
                None => {
                    if let Some(old) = self.ground.remove(&k).filter(|o| o.alive) {
                        let _ = writeln!(w, "-{:x}", old.id);
                    }
                    let id = self.fresh_id();
                    if alive {
                        let (coalition, colour) = side(g.blue);
                        let _ = write!(
                            w,
                            "{id:x},T={},Type={},Name={},Coalition={coalition},Color={colour}",
                            self.geo.metres_t_hdg(g.pos, g.heading),
                            ground_type(g.kind),
                            g.kind.name_on(g.blue),
                        );
                        if g.convoy != 0 {
                            let _ = write!(w, ",Group=Convoy {}", g.convoy);
                        }
                        let _ = writeln!(w);
                    }
                    self.ground.insert(k, Ground { id, kind: g.kind, last: g, alive });
                }
            }
        }
    }

    fn rounds(&mut self, c: &Contacts, w: &mut String) {
        let now: HashSet<_> = c.bullets.iter().map(|r| (r.owner, r.key)).collect();
        let mut gone: Vec<u32> =
            self.rounds.iter().filter(|(k, _)| !now.contains(k)).map(|(_, id)| *id).collect();
        self.rounds.retain(|k, _| now.contains(k));
        gone.sort_unstable();
        for id in gone {
            let _ = writeln!(w, "-{id:x}");
        }
        for r in &c.bullets {
            let (x, y, alt) = r.pos;
            let fresh = !self.rounds.contains_key(&(r.owner, r.key));
            if fresh {
                let id = self.fresh_id();
                self.rounds.insert((r.owner, r.key), id);
            }
            let id = self.rounds[&(r.owner, r.key)];
            let _ = write!(w, "{id:x},T={}", self.geo.map_t(x, y, alt));
            if fresh {
                let (coalition, colour) = side(r.owner != RoundOwner::Enemy);
                let _ = write!(
                    w,
                    ",Type=Weapon+Projectile+Bullet,Name=Gun round,Coalition={coalition},Color={colour}"
                );
            }
            let _ = writeln!(w);
        }
    }

    fn countermeasures(&mut self, c: &Contacts, s: &crate::recorder::Sample, w: &mut String) {
        // Where each pilot is: they drop them where they fly.
        let mut live: Vec<((u8, usize), Countermeasure, f64, bool)> =
            c.cms.iter().map(|&(k, cm)| ((255, k), cm, s.alt as f64, true)).collect();
        for p in &c.pilots {
            live.extend(
                p.cms
                    .iter()
                    .enumerate()
                    .filter(|(_, cm)| cm.ttl > 0)
                    .map(|(k, cm)| ((p.id, k), *cm, p.view.alt as f64, true)),
            );
        }
        // The AI aircraft's, at their own altitude.
        for &(i, k, cm) in &c.ai_cms {
            let blue = c.aircraft.iter().any(|a| a.index == i && a.friendly);
            let m = Countermeasure { x: cm.x, y: cm.y, kind: cm.kind, ttl: cm.ttl };
            live.push(((254, i * 2 + k), m, cm.alt as f64, blue));
        }
        let keys: HashSet<_> = live.iter().map(|(k, ..)| *k).collect();
        let mut gone: Vec<u32> =
            self.cms.iter().filter(|(k, _)| !keys.contains(k)).map(|(_, v)| v.0).collect();
        self.cms.retain(|k, _| keys.contains(k));
        gone.sort_unstable();
        for id in gone {
            let _ = writeln!(w, "-{id:x}");
        }
        for (k, cm, alt, blue) in live {
            match self.cms.get_mut(&k) {
                // The same one burning on (a new one starts with more time).
                Some((_, ttl)) if cm.ttl <= *ttl => *ttl = cm.ttl,
                old => {
                    if let Some((id, _)) = old {
                        let _ = writeln!(w, "-{id:x}");
                    }
                    let id = self.fresh_id();
                    self.cms.insert(k, (id, cm.ttl));
                    let ty = match cm.kind {
                        1 => "Misc+Decoy+Flare",
                        2 => "Misc+Decoy+Chaff",
                        _ => "Misc+Decoy",
                    };
                    let (coalition, colour) =
                        if blue { ("Allies", "Blue") } else { ("Enemies", "Red") };
                    let _ = writeln!(
                        w,
                        "{id:x},T={},Type={ty},Coalition={coalition},Color={colour}",
                        self.geo.map_t(cm.x as f64, cm.y as f64, alt),
                    );
                }
            }
        }
    }
}

fn side(blue: bool) -> (&'static str, &'static str) {
    if blue { ("Allies", "Blue") } else { ("Enemies", "Red") }
}

/// Tacview's type of a ground unit.
fn ground_type(k: GroundKind) -> &'static str {
    match k {
        _ if k.tank() => "Ground+Heavy+Armor+Vehicle+Tank",
        _ if k.carrier() => "Ground+Medium+Armor+Vehicle",
        GroundKind::Manpads => "Ground+Light+Human+Infantry",
        GroundKind::DroneCenter => "Ground+Static+Building",
        _ if k.patriot() => "Ground+Static+AntiAircraft",
        _ => "Ground+Heavy+Armor+AntiAircraft",
    }
}

/// Where recordings go; none in the browser.
fn folder() -> Option<PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    return stealth_host::store::path("tacview");
    #[cfg(target_arch = "wasm32")]
    None
}

/// Writes the recording of a flight; errors are logged.
pub fn export(rec: &FlightRecord, m: &Mission, data: &SimData, aircraft: &str) -> Option<PathBuf> {
    let dir = folder()?;
    let now = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let (y, mo, d) = civil_from_days(now.div_euclid(86_400));
    let t = now.rem_euclid(86_400);
    let name = format!(
        "{y:04}{mo:02}{d:02}-{:02}{:02}{:02}_{}.acmi",
        t / 3600,
        t / 60 % 60,
        t % 60,
        m.theater_code.to_ascii_lowercase()
    );
    let path = dir.join(name);
    let text = acmi(rec, m, data, aircraft);
    match std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, text)) {
        Ok(()) => {
            info!("tacview: {}", path.display());
            Some(path)
        }
        Err(e) => {
            warn!("tacview: {}: {e}", path.display());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip() {
        for (y, m, d) in [(1970, 1, 1), (1991, 1, 17), (2000, 2, 29), (2026, 10, 21)] {
            assert_eq!(civil_from_days(days_from_civil(y, m, d)), (y, m, d));
        }
        assert_eq!(days_from_civil(1970, 1, 2), 1);
        assert_eq!(briefing_date("January 17, 1991"), (1991, 1, 17));
        assert_eq!(briefing_date("Nov 3, 1989"), (1989, 11, 3));
        assert_eq!(briefing_date(""), (1990, 1, 1));
    }

    #[test]
    fn geo_offsets() {
        let g = Geo { lat0: 60.0 };
        // 111.2 km north is a degree; at 60 degrees a degree east is half as long.
        let (lon, lat) = g.offset(55_660.0, 111_200.0);
        assert!((lat - 1.0).abs() < 1e-9);
        assert!((lon - 55_660.0 / (111_320.0 * 61f64.to_radians().cos())).abs() < 1e-9);
        assert_eq!(deg(0x4000, false), 90.0);
        assert_eq!(deg(0xC000, true), -90.0);
        assert_eq!(true_altitude(13192.0), 20000.0);
    }

    #[test]
    fn acmi_objects_spawn_and_go() {
        use crate::recorder::Sample;
        use stealth_sim::FlightUnit;
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let data = SimData::load_json(&assets.join("sim")).unwrap();
        let gdata = stealth_mission::GameData::load_json(&assets.join("mission")).unwrap();
        let m = stealth_mission::generate(
            &gdata,
            &stealth_mission::MissionSettings { theater: 3, ..Default::default() },
            stealth_mission::Seed::Srand(1234),
        )
        .unwrap();
        let mut rec = FlightRecord::default();
        let missile =
            stealth_sim::Projectile { ttl: 10, spec: 2, launcher: 5, ..Default::default() };
        for f in 0..20 {
            rec.samples.push(Sample {
                frame: f,
                wx: 0x80000,
                wy: 0x80000,
                alt: 1000,
                heading: 0x4000,
                pitch: 0,
                roll: 0,
                knots: 400,
                emv: 0,
                margin: -100,
                site: None,
            });
        }
        let route = vec![
            UavWp { x: 0x4000, y: 0x4000, alt: 300.0, kind: Default::default() },
            UavWp { x: 0x4010, y: 0x4000, alt: 300.0, kind: Default::default() },
        ];
        let drone = |kind, lock, route: &Vec<UavWp>| crate::recorder::Air {
            index: 4,
            unit: FlightUnit { spec: 33, wx: 1, wy: 1, ..Default::default() },
            kind: AircraftKind::Drone,
            friendly: true,
            uav: Some(stealth_sim::uav::Uav {
                id: 2,
                center: 7,
                kind,
                flight: 4,
                route: route.clone(),
                next: 0,
                mode: Default::default(),
                battery: 0.5,
                pos: [1000.0, 2000.0, 400.0],
                heading: 0.0,
                pitch: 0.0,
                bank: 0.0,
                speed: 30.0,
                ctrl: [0.0; 3],
                yaw: 0.0,
                climb_out: 0.0,
                lock,
                vel: [0.0; 3],
            }),
            flown_by_player: false,
            route_ground: vec![],
        };
        let tank = |e: f64, destroyed| GroundSnap {
            kind: GroundKind::T90,
            pos: [e, 5000.0, 2.0],
            heading: 0.0,
            blue: false,
            destroyed,
            gone: false,
            convoy: 1,
        };
        let round = |vx| crate::recorder::Round {
            owner: RoundOwner::Player,
            key: (0, vx, 0),
            pos: (100.0, 100.0, 1000.0),
        };
        let bomb = stealth_sim::Projectile { ttl: 10, spec: 30, launcher: 0, ..Default::default() };
        rec.contacts = vec![
            Contacts {
                frame: 0,
                missiles: vec![(1, missile), (9, bomb)],
                aircraft: vec![drone(UavKind::Stork, None, &route)],
                ground: vec![(0, tank(0.0, false))],
                bullets: vec![round(5)],
                ..Default::default()
            },
            Contacts {
                frame: 3,
                aircraft: vec![drone(UavKind::Stork, Some(9), &route[..1].to_vec())],
                ground: vec![(0, tank(100.0, false))],
                bullets: vec![round(6)],
                ..Default::default()
            },
            Contacts {
                frame: 3 * 2 - 1,
                ground: vec![(0, tank(100.0, true))],
                ..Default::default()
            },
        ];
        rec.events = vec![
            (2, TacEvent::AirDestroyed(4)),
            (
                1,
                TacEvent::GroundShot {
                    gun: GroundGun::Tank,
                    from: [0.0, 0.0, 0.0],
                    to: [1650.0, 0.0, 0.0],
                    secs: 1.0,
                    blue: true,
                },
            ),
        ];
        rec.events.sort_by_key(|e| e.0);
        rec.marks.push(crate::recorder::Mark { frame: 2, x: 0.0, y: 0.0, kind: MarkKind::Hit });
        let text = acmi(&rec, &m, &data, "F-117A");
        assert!(text.contains("\n0,Event=Message|100|Hit\n"));
        // Written while it grows, frame by frame (the real-time telemetry), the flight reads the
        // same.
        let mut writer = AcmiWriter::new(&m);
        let mut streamed = String::new();
        for n in 0..=rec.samples.len() {
            let mut part = rec.clone();
            part.samples.truncate(n);
            part.contacts.retain(|c| (c.frame as usize) < n);
            part.marks.retain(|mk| (mk.frame as usize) < n);
            part.events.retain(|e| (e.0 as usize) < n);
            writer.advance(&part, &m, &data, "F-117A", &mut streamed);
        }
        assert_eq!(streamed, text);
        assert!(text.starts_with("FileType=text/acmi/tacview\nFileVersion=2.2\n"));
        assert!(text.contains("0,ReferenceTime="));
        assert!(text.contains(",Type=Air+FixedWing,Name=F-117A,"));
        // The player flies east at 1000 ft (304.8 m), heading 90.
        assert!(text.contains("|304.8|0.0|0.0|90.0"));
        assert_eq!(text.matches("2000,T=").count(), 1);
        assert!(text.contains("2000,T=") && text.contains("Type=Weapon+Missile"));
        assert!(text.contains("\n-2000\n"));
        // A bomb is a bomb, fired by the player.
        assert!(text.contains("Type=Weapon+Bomb,") && text.contains(",Parent=100"));
        // The drone: exact position, its call sign, then a lock and a shorter route.
        assert!(text.contains(",Type=Air+FixedWing,") && text.contains(",CallSign=STK-2,"));
        assert!(text.contains("Type=Navaid+Static+Waypoint") && text.contains(",Next="));
        assert!(text.contains(",LockedTarget=1009"));
        assert!(text.contains("0,Event=Destroyed|1004|"));
        // The tank: shown, moved once, then destroyed and removed.
        assert_eq!(text.matches("Type=Ground+Heavy+Armor+Vehicle+Tank").count(), 1);
        assert!(text.contains(",Group=Convoy 1"));
        // A new round in the same slot is a new object.
        assert_eq!(text.matches("Type=Weapon+Projectile+Bullet").count(), 2);
        // The shell flies a second (15 frames) and goes.
        assert_eq!(text.matches("Type=Weapon+Projectile+Shell").count(), 1);
        // (Ids counted up: the two waypoints, the tank, the round, the shell.)
        let shell = text.find("Type=Weapon+Projectile+Shell").unwrap();
        let landed = text.find("\n#1.07\n").unwrap();
        assert!(shell < landed && text[landed..].contains(&format!("\n-{:x}\n", DYNAMIC + 5)));
        // The tank's destruction.
        assert!(text.contains(&format!("0,Event=Destroyed|{:x}|", DYNAMIC + 3)));
    }
}

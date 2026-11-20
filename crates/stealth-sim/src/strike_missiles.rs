//! Ground attack missiles the campaign fires at the other side's bases, cities and forces: the
//! enemy's Iskander ballistic missiles (from a TEL that drives to its firing point) and Kh-101
//! cruise missiles (from Tu-95 bombers), the friendly side's FP-7 ballistic missiles and Flamingo
//! cruise missiles (from the launch sites the logistics missions build).
//!
//! Each missile is one of the campaign's AI flights (`Role::Missile`), so the air defences, the
//! fighters, the manned Patriot console and the player's radar see it and can shoot it down as
//! any aircraft; [`Sim::fly_strike_missile`] flies it instead of the AI:
//!
//! - A ballistic missile flies a fixed arc from its launch point to its aim point, high and fast;
//!   only a long range SAM (a Patriot, an SA-10/12/21) can engage it.
//! - A cruise missile flies low over the terrain, by a dogleg round the defences it knows of,
//!   then dives onto its aim point. Radars see it late.
//!
//! Where it comes down everything within its warhead's reach is destroyed
//! ([`SimEvent::StrikeMissileImpact`]).
//!
//! The TELs and launchers are ground units ([`GroundKind::IskanderTel`],
//! [`GroundKind::MissileLauncher`]); a TEL drives at its pace from where it was spawned to its
//! firing point ([`SimEvent::LauncherArrived`]).

use serde::{Deserialize, Serialize};

use crate::Sim;
use crate::aaa::{GroundKind, GroundUnit, M_PER_MAP};
use crate::campaign::Role;
use crate::math::{approx, s16};
use crate::types::*;

/// The missiles' AI aircraft types (assets/sim/units.ron).
pub const ISKANDER: i32 = 48;
pub const KH101: i32 = 49;
pub const FP7: i32 = 50;
pub const FLAMINGO: i32 = 51;
/// The Kh-101's carrier: the Tu-95.
pub const TU95: i32 = 15;

const M_PER_WU: f64 = M_PER_MAP / 32.0;
/// Cruise missiles fly this high over the ground (m), and see the ground this far ahead (m).
const CRUISE_AGL: f64 = 60.0;
/// They turn at most this fast (rad/s) and dive at their aim point from this far (m).
const CRUISE_TURN: f64 = 0.06;
const DIVE_FROM: f64 = 1500.0;
/// A TEL's road speed (m/s).
const TEL_SPEED: f64 = 12.0;
/// A carrier launches its salvo this close to its launch point (map units).
const SALVO_REACH: i32 = 0x40;
/// Radars find a cruise missile only this close (m).
pub const CRUISE_DETECT_M: f64 = 25_000.0;

/// What a missile is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MissileKind {
    #[default]
    Iskander,
    Kh101,
    Fp7,
    Flamingo,
}

impl MissileKind {
    /// Its AI aircraft type.
    pub fn spec(self) -> i32 {
        match self {
            MissileKind::Iskander => ISKANDER,
            MissileKind::Kh101 => KH101,
            MissileKind::Fp7 => FP7,
            MissileKind::Flamingo => FLAMINGO,
        }
    }

    pub fn of_spec(spec: i32) -> Option<MissileKind> {
        Some(match spec {
            ISKANDER => MissileKind::Iskander,
            KH101 => MissileKind::Kh101,
            FP7 => MissileKind::Fp7,
            FLAMINGO => MissileKind::Flamingo,
            _ => return None,
        })
    }

    /// It flies a ballistic arc (else it cruises low).
    pub fn ballistic(self) -> bool {
        matches!(self, MissileKind::Iskander | MissileKind::Fp7)
    }

    /// The side that fires it.
    pub fn blue(self) -> bool {
        matches!(self, MissileKind::Fp7 | MissileKind::Flamingo)
    }

    /// Its speed (m/s): a ballistic missile's average along its arc.
    pub fn speed(self) -> f64 {
        match self {
            MissileKind::Iskander => 1500.0,
            MissileKind::Fp7 => 1200.0,
            MissileKind::Kh101 => 220.0,
            MissileKind::Flamingo => 250.0,
        }
    }

    /// Its reach (m).
    pub fn range(self) -> f64 {
        match self {
            MissileKind::Iskander => 450_000.0,
            MissileKind::Fp7 => 200_000.0,
            MissileKind::Kh101 => 1_500_000.0,
            MissileKind::Flamingo => 1_500_000.0,
        }
    }

    /// Its warhead destroys everything within this (m).
    pub fn blast(self) -> f64 {
        match self {
            MissileKind::Iskander => 60.0,
            MissileKind::Fp7 => 50.0,
            MissileKind::Kh101 => 40.0,
            MissileKind::Flamingo => 70.0,
        }
    }
}

/// A missile's flight ([`crate::campaign::CampaignFlight::missile`]): launch and aim points
/// (map units), the object it is aimed at, the dogleg a cruise missile flies by, the seconds
/// since launch and its whole flight's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StrikeMissile {
    pub kind: MissileKind,
    pub from: (i32, i32),
    pub aim: (i32, i32),
    pub object: Option<u16>,
    pub via: Option<(i32, i32)>,
    /// Frames since launch, and the arc's frames (ballistic).
    pub frame: i32,
    pub frames: i32,
    /// Ground height at launch and aim (wu), the arc's top over their line (wu).
    pub ground: (i32, i32),
    pub top: i32,
}

/// A carrier's missiles (its [`crate::campaign::CampaignFlight::salvo`]): what they are, how
/// many, where each goes, and their campaign tags (`tag` + 1 + k).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Salvo {
    pub kind: MissileKind,
    pub count: u8,
    pub aims: [(i32, i32); 6],
    pub objects: [Option<u16>; 6],
    pub tag: u32,
}

/// A TEL on its way to its firing point (metres east, north).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Launcher {
    pub unit: u16,
    pub to: [f64; 2],
    pub arrived: bool,
}

/// A point in map units (east, south) in metres east, north.
fn metres(p: (i32, i32)) -> [f64; 2] {
    [p.0 as f64 * M_PER_MAP, (0x8000 - p.1) as f64 * M_PER_MAP]
}

impl Sim {
    /// The flight unit `j` is a ballistic missile.
    pub fn ballistic_missile(&self, j: usize) -> bool {
        MissileKind::of_spec(self.flights.get(j).map_or(-1, |u| u.spec))
            .is_some_and(|k| k.ballistic())
    }

    /// The flight unit `j` is a cruise missile.
    pub fn cruise_missile(&self, j: usize) -> bool {
        MissileKind::of_spec(self.flights.get(j).map_or(-1, |u| u.spec))
            .is_some_and(|k| !k.ballistic())
    }

    /// A missile of `kind` leaves from `from` (map units; `alt` wu up, `heading`) for `aim`
    /// (the object `object` there, if any): campaign tag `tag`. Returns its flight unit.
    #[allow(clippy::too_many_arguments)]
    pub fn launch_strike_missile(
        &mut self,
        kind: MissileKind,
        from: (i32, i32),
        alt: Option<i32>,
        aim: (i32, i32),
        object: Option<usize>,
        tag: u32,
    ) -> usize {
        if self.campaign.is_none() {
            self.campaign = Some(Box::new(self.campaign_state([0; 4], 0)));
        }
        let i = self.campaign_slot();
        let fs = self.fs.max(1) as f64;
        let [e0, n0] = metres(from);
        let [e1, n1] = metres(aim);
        let d = (e1 - e0).hypot(n1 - n0);
        let g0 = self.hill_alt(from.0, from.1);
        let g1 = self.hill_alt(aim.0, aim.1);
        let frames = ((d / kind.speed()) * fs).round().max(1.0) as i32;
        // The arc's top: a quarter of the range, 10 to 25 km up.
        let top = ((d * 0.25).clamp(10_000.0, 25_000.0) / M_PER_WU) as i32;
        let heading = crate::math::bearing(aim.0 - from.0, s16(from.1 - aim.1));
        // A cruise missile's dogleg: out to one side of the line, a third of the way.
        let via = (!kind.ballistic() && d > 60_000.0).then(|| {
            let side = if self.rng.random(2) == 0 { 1.0 } else { -1.0 };
            let (ue, un) = ((e1 - e0) / d, (n1 - n0) / d);
            let off = d * 0.15 * side;
            let (ve, vn) = (e0 + (e1 - e0) / 3.0 + un * off, n0 + (n1 - n0) / 3.0 - ue * off);
            ((ve / M_PER_MAP) as i32, 0x8000 - (vn / M_PER_MAP) as i32)
        });
        let start_alt = alt.unwrap_or(g0 + 20);
        let wx = (e0 / M_PER_WU) as i32;
        let wy = ((0x8000 as f64 * M_PER_MAP - n0) / M_PER_WU) as i32;
        self.flights[i] = FlightUnit {
            spec: kind.spec(),
            wx,
            wy,
            x: (wx >> 5) & 0xFFFF,
            y: (wy >> 5) & 0xFFFF,
            alt: start_alt,
            heading,
            speed: (kind.speed() / M_PER_WU) as i32,
            flags: fflags::EXISTS | fflags::AIRBORNE,
            timer: 0x7FFF,
            ..Default::default()
        };
        let missile = StrikeMissile {
            kind,
            from,
            aim,
            object: object.map(|o| o as u16),
            via,
            frame: 0,
            frames,
            ground: (start_alt, g1),
            top,
        };
        let home = -1;
        let mut f = self.mission_flight(
            kind.spec(),
            crate::ai_missions::AiTask::Patrol(aim.0, aim.1),
            tag,
            kind.blue(),
            0,
        );
        f.role = Role::Missile;
        f.home = home;
        f.goal = aim;
        f.loadout = Default::default();
        f.missile = Some(missile);
        self.campaign.as_mut().expect("campaign").flights[i] = Some(f);
        i
    }

    /// A Tu-95 (`carrier`, from base `from`) flies to `launch` (map units) and fires `salvo`
    /// there, then turns home. Returns its flight unit.
    pub fn spawn_missile_carrier(
        &mut self,
        from: usize,
        launch: (i32, i32),
        salvo: Salvo,
        blue: bool,
    ) -> usize {
        let i = self.spawn_ai_mission(
            from,
            TU95,
            crate::ai_missions::AiTask::Patrol(launch.0, launch.1),
            salvo.tag,
            blue,
        );
        if let Some(Some(f)) = self.campaign.as_mut().map(|c| &mut c.flights[i]) {
            f.role = Role::Strike;
            f.goal = launch;
            f.strike = None;
            f.salvo = Some(salvo);
        }
        i
    }

    /// A TEL (or another launcher of kind `kind`, side `blue`) drives from `from` to `to` (map
    /// units); `fixed`: it is set up there already. Returns its ground unit.
    pub fn spawn_launcher(
        &mut self,
        kind: GroundKind,
        from: (i32, i32),
        to: (i32, i32),
        blue: bool,
        fixed: bool,
    ) -> usize {
        let at = if fixed { to } else { from };
        let [e, n] = metres(at);
        let unit = GroundUnit {
            kind,
            x: at.0,
            y: at.1,
            fine: Some(crate::aaa::Fine { e: e as f32, n: n as f32, heading: 0.0 }),
            ..Default::default()
        };
        self.units.push(unit);
        let k = self.units.len() - 1;
        if let Some(c) = self.campaign.as_mut() {
            c.aaa_blue.resize(self.units.len(), false);
            c.aaa_target.resize(self.units.len(), None);
            c.aaa_blue[k] = blue;
        }
        self.launchers.push(Launcher { unit: k as u16, to: metres(to), arrived: fixed });
        if fixed {
            self.emit(SimEvent::LauncherArrived { unit: k });
        }
        k
    }

    /// Launcher `unit` fires `count` missiles of `kind` at `aims` (map units; the objects there):
    /// campaign tags `tag` + 1 + k. Returns how many left (none from a destroyed launcher).
    pub fn fire_launcher(
        &mut self,
        unit: usize,
        kind: MissileKind,
        aims: &[((i32, i32), Option<usize>)],
        tag: u32,
    ) -> usize {
        let Some(u) = self.units.get(unit).filter(|u| !u.destroyed && !u.gone).copied() else {
            return 0;
        };
        let from = (u.map_xy().0.round() as i32, u.map_xy().1.round() as i32);
        for (k, &(aim, object)) in aims.iter().enumerate() {
            self.launch_strike_missile(kind, from, None, aim, object, tag + 1 + k as u32);
        }
        self.emit(SimEvent::StrikeMissilesAway {
            tag,
            count: aims.len() as u8,
            x: from.0,
            y: from.1,
            blue: kind.blue(),
        });
        aims.len()
    }

    /// The TELs on their way, and the carriers' salvos (once a frame, the host's).
    pub(crate) fn update_strike_missiles(&mut self) {
        if self.mirror {
            return;
        }
        let dt = 1.0 / self.fs.max(1) as f64;
        let mut arrived = Vec::new();
        for k in 0..self.launchers.len() {
            let l = self.launchers[k];
            if l.arrived {
                continue;
            }
            let Some(u) = self.units.get_mut(l.unit as usize) else { continue };
            if u.destroyed || u.gone {
                continue;
            }
            let Some(f) = u.fine.as_mut() else { continue };
            let (de, dn) = (l.to[0] - f.e as f64, l.to[1] - f.n as f64);
            let d = de.hypot(dn);
            let step = TEL_SPEED * dt;
            if d <= step {
                f.e = l.to[0] as f32;
                f.n = l.to[1] as f32;
                self.launchers[k].arrived = true;
                arrived.push(l.unit as usize);
            } else {
                f.e += (de / d * step) as f32;
                f.n += (dn / d * step) as f32;
                f.heading = de.atan2(dn) as f32;
            }
            let (e, n) = (f.e as f64, f.n as f64);
            u.x = (e / M_PER_MAP).round() as i32;
            u.y = 0x8000 - (n / M_PER_MAP).round() as i32;
        }
        for unit in arrived {
            self.emit(SimEvent::LauncherArrived { unit });
        }
        // Carriers at their launch points let their missiles go and turn home.
        let Some(c) = self.campaign.as_ref() else { return };
        let due: Vec<(usize, Salvo)> = c
            .flights
            .iter()
            .enumerate()
            .filter_map(|(i, f)| {
                let f = f.as_ref()?;
                let s = f.salvo?;
                let u = &self.flights[i];
                (u.flags & fflags::DESTROYED == 0
                    && u.flags & fflags::AIRBORNE != 0
                    && approx(u.x - f.goal.0, u.y - f.goal.1) < SALVO_REACH)
                    .then_some((i, s))
            })
            .collect();
        for (i, s) in due {
            let u = self.flights[i];
            for k in 0..(s.count as usize).min(s.aims.len()) {
                self.launch_strike_missile(
                    s.kind,
                    (u.x, u.y),
                    Some(u.alt),
                    s.aims[k],
                    s.objects[k].map(usize::from),
                    s.tag + 1 + k as u32,
                );
            }
            self.emit(SimEvent::StrikeMissilesAway {
                tag: s.tag,
                count: s.count,
                x: u.x,
                y: u.y,
                blue: s.kind.blue(),
            });
            if let Some(Some(f)) = self.campaign.as_mut().map(|c| &mut c.flights[i]) {
                f.salvo = None;
                let home = f.home;
                self.flights[i].flags |= fflags::RTB;
                self.flights[i].wp = home;
            }
        }
    }

    /// The missile of flight unit `i` (if it is one).
    pub(crate) fn strike_missile_of(&self, i: usize) -> Option<StrikeMissile> {
        self.campaign.as_ref()?.flights.get(i)?.as_ref()?.missile
    }

    /// One frame of missile `i`'s flight; at its aim point it goes off.
    pub(crate) fn fly_strike_missile(&mut self, i: usize) {
        let Some(mut m) = self.strike_missile_of(i) else { return };
        let mut u = self.flights[i];
        let fs = self.fs.max(1) as f64;
        let dt = 1.0 / fs;
        m.frame += 1;
        let [e0, n0] = metres(m.from);
        let [e1, n1] = metres(m.aim);
        let here = [u.wx as f64 * M_PER_WU, (0x10_0000 - u.wy) as f64 * M_PER_WU];
        let arrived;
        if m.kind.ballistic() {
            // Along the arc: its line on the ground, its height a parabola over it.
            let s = (m.frame as f64 / m.frames as f64).min(1.0);
            let (e, n) = (e0 + (e1 - e0) * s, n0 + (n1 - n0) * s);
            let base = m.ground.0 as f64 + (m.ground.1 - m.ground.0) as f64 * s;
            let alt = base + 4.0 * m.top as f64 * s * (1.0 - s);
            let climb = (m.ground.1 - m.ground.0) as f64 + 4.0 * m.top as f64 * (1.0 - 2.0 * s);
            let d = (e1 - e0).hypot(n1 - n0) / M_PER_WU;
            u.pitch = crate::ai_flight::bam(climb.atan2(d));
            u.wx = (e / M_PER_WU) as i32;
            u.wy = 0x10_0000 - (n / M_PER_WU) as i32;
            u.alt = alt as i32;
            arrived = s >= 1.0;
        } else {
            // Low over the ground, by its dogleg, at its speed; it dives at the end.
            let goal = match m.via {
                Some(v) if approx(u.x - v.0, u.y - v.1) > 0x40 => v,
                _ => {
                    m.via = None;
                    m.aim
                }
            };
            let [ge, gn] = metres(goal);
            let want = (ge - here[0]).atan2(gn - here[1]);
            let psi = crate::ai_flight::rad(u.heading);
            let turn = (want - psi + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
                - std::f64::consts::PI;
            let psi = psi + turn.clamp(-CRUISE_TURN * dt, CRUISE_TURN * dt);
            let v = m.kind.speed();
            let (e, n) = (here[0] + psi.sin() * v * dt, here[1] + psi.cos() * v * dt);
            u.heading = crate::ai_flight::bam(psi);
            u.wx = (e / M_PER_WU) as i32;
            u.wy = 0x10_0000 - (n / M_PER_WU) as i32;
            let to_aim = (e1 - e).hypot(n1 - n);
            let ground = self.hill_alt((u.wx >> 5) & 0xFFFF, (u.wy >> 5) & 0xFFFF) as f64;
            let target_alt = if m.via.is_none() && to_aim < DIVE_FROM {
                m.ground.1 as f64 + (CRUISE_AGL / M_PER_WU) * (to_aim / DIVE_FROM)
            } else {
                ground + CRUISE_AGL / M_PER_WU
            };
            let climb = (target_alt - u.alt as f64).clamp(-v * dt / M_PER_WU, v * dt / M_PER_WU);
            u.alt = (u.alt as f64 + climb) as i32;
            u.pitch = crate::ai_flight::bam((climb / (v * dt / M_PER_WU)).clamp(-1.0, 1.0) * 0.6);
            arrived = to_aim < v * dt * 1.5;
        }
        u.x = (u.wx >> 5) & 0xFFFF;
        u.y = (u.wy >> 5) & 0xFFFF;
        u.speed = (m.kind.speed() / M_PER_WU) as i32;
        u.bank = 0;
        self.flights[i] = u;
        if let Some(Some(f)) = self.campaign.as_mut().map(|c| &mut c.flights[i]) {
            f.missile = Some(m);
        }
        if arrived {
            self.strike_missile_impact(i, &m);
        }
    }

    /// Missile `i` goes off at its aim point: what is within its warhead's reach is destroyed;
    /// it is gone.
    fn strike_missile_impact(&mut self, i: usize, m: &StrikeMissile) {
        let tag = self.campaign.as_ref().and_then(|c| c.flights[i]).map_or(0, |f| f.tag);
        let [e, n] = metres(m.aim);
        let r = m.kind.blast();
        let mut kills = 0u8;
        let objects: Vec<usize> = (1..self.objects.len())
            .filter(|&k| {
                let o = &self.objects[k];
                o.flags & oflags::DESTROYED == 0 && {
                    let at = crate::asw::object_metres(o);
                    (at[0] - e).hypot(at[1] - n) < r || m.object == Some(k as u16)
                }
            })
            .collect();
        for k in objects {
            self.destroy_ground(k as i32);
            kills += 1;
        }
        let units: Vec<usize> = (0..self.units.len())
            .filter(|&k| {
                let u = &self.units[k];
                !u.destroyed && !u.gone && {
                    let (x, y) = u.map_xy();
                    (x * M_PER_MAP - e).hypot((0x8000 as f64 - y) * M_PER_MAP - n) < r
                }
            })
            .collect();
        for k in units {
            self.units[k].destroyed = true;
            self.units[k].burst = 0;
            self.emit(SimEvent::AaaDestroyed { unit: k, by: None });
            kills += 1;
        }
        let ground = self.hill_alt(m.aim.0, m.aim.1);
        self.emit(SimEvent::Explosion { x: m.aim.0, y: m.aim.1, alt: ground, kind: -6 });
        self.emit(SimEvent::StrikeMissileImpact { tag, x: m.aim.0, y: m.aim.1, kills });
        self.flights[i] = FlightUnit::default();
        if let Some(c) = self.campaign.as_mut() {
            c.flights[i] = None;
        }
    }

    /// Where launcher `unit` stands (map units), if it is still there.
    pub fn launcher_at(&self, unit: usize) -> Option<(i32, i32)> {
        let u = self.units.get(unit).filter(|u| !u.destroyed && !u.gone)?;
        let (x, y) = u.map_xy();
        Some((x.round() as i32, y.round() as i32))
    }

    /// The TELs and launchers (ground unit, arrived at its firing point).
    pub fn launchers(&self) -> impl Iterator<Item = (usize, bool)> + '_ {
        self.launchers.iter().map(|l| (l.unit as usize, l.arrived))
    }
}

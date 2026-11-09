//! Campaign: the AI flying the missions of the boards (the friendly side in the players'
//! aircraft), and a pilot taking over one of those aircraft in the air or leaving his own to
//! the AI.

use crate::ai_loadout::{AIM9, AIM120, AiLoadout, FighterTask, Seeker, Store};
use crate::campaign::{CampaignFlight, M_PER_WU, Role};
use crate::csar::HIND;
use crate::math::{approx, s16};
use crate::types::*;
use crate::{AircraftKind, Sim};

/// The players' aircraft as AI types (`assets/sim/units.ron`).
pub const F16: i32 = 22;
pub const UH60: i32 = 23;
pub const A10: i32 = 28;
pub const F117: i32 = 36;
pub const F22: i32 = 37;
pub const TORNADO: i32 = 38;
pub const AV8B: i32 = 39;
pub const AH1Z: i32 = 40;
pub const RAH66: i32 = 41;
pub const OH58: i32 = 42;
pub const CH47: i32 = 43;

/// The other side's ground units this close to a CAS flight's point are its targets (map
/// units, 5 km).
const CAS_RADIUS: i32 = 0x140;
/// A recon flight photographs its object this close (map units, 1.5 km).
const RECON_RANGE: i32 = 0x60;
/// Knots in metres a second.
const KNOT: f64 = 0.514_444;

/// What an AI flight of a mission goes for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiTask {
    /// Bomb a world object (an anti-radiation missile at a SAM site's radar).
    Strike(usize),
    /// The other side's ground units around a point (map units): rockets at them.
    Ground(i32, i32),
    /// Photograph a world object.
    Recon(usize),
    /// Patrol over a point and fight the other side's aircraft.
    Patrol(i32, i32),
    /// Wait over a point for the other side's aircraft with a campaign tag (a cargo flight),
    /// then go after it.
    Hunt(u32, i32, i32),
}

/// What the enemy flies a mission in (its era's type of it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedType {
    Strike,
    Fighter,
    Recon,
    Helicopter,
}

/// The true altitude of a viewZ ([`crate::math::view_z`] inverted).
fn true_altitude(z: i32) -> i32 {
    if z < 0x2000 {
        z
    } else if z < 0x3000 {
        0x2000 + (z - 0x2000) * 2
    } else {
        0x4000 + (z - 0x3000) * 4
    }
}

/// A campaign aircraft's record with nothing to do yet.
fn flight(
    tag: u32,
    role: Role,
    blue: bool,
    goal: (i32, i32),
    home: i32,
    anchor: (i32, i32),
) -> CampaignFlight {
    CampaignFlight {
        tag,
        role,
        blue,
        goal,
        target: None,
        cooldown: 0,
        loadout: AiLoadout::default(),
        task: FighterTask::Patrol,
        home,
        anchor,
        quiet: 0,
        burst: 0,
        killed_by_ai: false,
        reported: false,
        killed_by: None,
        killed_by_radar: None,
        strike: None,
        strike_aaa: false,
        kamikaze: false,
        supply: None,
        strike_unit: false,
        defending: false,
        fought: 0,
        passes: 0,
        pass_wait: 0,
        hunt: None,
        iron: false,
    }
}

impl Sim {
    /// Campaign: the enemy's type for a job, flying from base `from` to `to` (map units).
    pub fn red_type(&mut self, job: RedType, from: usize, to: (i32, i32)) -> i32 {
        let late = self.aircraft_types.len() > 32 && self.theater >= 9;
        let types: &[i32] = match job {
            RedType::Strike if late => &[31],
            RedType::Strike => &[11, 0],
            RedType::Fighter if late => &[32, 31, 2],
            RedType::Fighter => &[0, 2, 4, 25],
            RedType::Recon => &[1],
            RedType::Helicopter => &[HIND],
        };
        self.pick_reaching(types, from, to)
    }

    /// Campaign: an aircraft of type `spec` of a side's AI flight takes off from base `from`
    /// for `task`. Returns its flight unit.
    pub fn spawn_ai_mission(
        &mut self,
        from: usize,
        spec: i32,
        task: AiTask,
        tag: u32,
        blue: bool,
    ) -> usize {
        let i = self.campaign_slot();
        let mut f = self.mission_flight(spec, task, tag, blue, from);
        self.flights[i] = FlightUnit { spec, missile: f.classic_missile(), ..Default::default() };
        self.spawn(i, from);
        if f.role == Role::Strike && f.strike_unit {
            f.strike = Some(0);
        }
        self.campaign.as_mut().expect("campaign").flights[i] = Some(f);
        i
    }

    /// The record of an aircraft of type `spec` flying `task` from base `from`.
    fn mission_flight(
        &self,
        spec: i32,
        task: AiTask,
        tag: u32,
        blue: bool,
        from: usize,
    ) -> CampaignFlight {
        let at = |o: usize| self.objects.get(o).map_or((0, 0), |o| (o.x, o.y));
        let base = at(from);
        let home = from as i32;
        // A jet on a ground mission carries missiles to defend itself.
        let own = if self.ac_type(spec).kind == AircraftKind::Plane {
            AiLoadout::self_defence(spec, blue)
        } else {
            AiLoadout::default()
        };
        match task {
            AiTask::Strike(o) => CampaignFlight {
                strike: Some(o as i32),
                loadout: own,
                ..flight(tag, Role::Strike, blue, at(o), home, base)
            },
            AiTask::Ground(x, y) => CampaignFlight {
                strike_unit: true,
                passes: crate::campaign::CAS_PASSES,
                loadout: own,
                ..flight(tag, Role::Strike, blue, (x, y), home, base)
            },
            AiTask::Recon(o) => CampaignFlight {
                strike: Some(o as i32),
                loadout: own,
                ..flight(tag, Role::Recon, blue, at(o), home, base)
            },
            AiTask::Hunt(quarry, x, y) => CampaignFlight {
                hunt: Some(quarry),
                ..self.mission_flight(spec, AiTask::Patrol(x, y), tag, blue, from)
            },
            AiTask::Patrol(x, y) => {
                let store = |missile, count, seeker| Store { missile, count, seeker };
                let heli = self.ac_type(spec).kind == AircraftKind::Helicopter;
                let loadout = if !blue {
                    let missile = match spec {
                        0 | 25 => 19,
                        _ => 21,
                    };
                    AiLoadout::for_task(spec, missile, FighterTask::Patrol)
                } else if heli {
                    AiLoadout {
                        stores: [store(AIM9, 2, Seeker::Ir), Store::default(), Store::default()],
                    }
                } else {
                    AiLoadout::for_task(spec, AIM120, FighterTask::Cap)
                };
                CampaignFlight { loadout, ..flight(tag, Role::Fighter, blue, (x, y), home, base) }
            }
        }
    }

    /// Flight unit of the campaign aircraft with tag `tag` (in the air, alive).
    pub fn flight_of_tag(&self, tag: u32) -> Option<usize> {
        let c = self.campaign.as_ref()?;
        let i = c.flights.iter().position(|f| f.is_some_and(|f| f.tag == tag))?;
        let u = self.flights[i];
        (u.flags & fflags::DESTROYED == 0 && u.flags & fflags::AIRBORNE != 0).then_some(i)
    }

    /// The AI's aircraft `tag` can be taken over: up and away (not taking off, not landing).
    pub fn takeable(&self, tag: u32) -> bool {
        self.flight_of_tag(tag).is_some_and(|i| {
            let u = self.flights[i];
            u.flags & (fflags::TAKEOFF | fflags::FINAL | fflags::ROLLOUT) == 0 && u.speed > 0
        })
    }

    /// Recon flights over their object take its photo and turn for home.
    pub(crate) fn campaign_recon(&mut self) {
        let Some(c) = self.campaign.as_mut() else { return };
        let mut ev = Vec::new();
        for (i, slot) in c.flights.iter_mut().enumerate() {
            let Some(f) = slot.as_mut().filter(|f| f.role == Role::Recon && f.strike.is_some())
            else {
                continue;
            };
            let u = &mut self.flights[i];
            if u.flags & fflags::DESTROYED != 0 || u.flags & fflags::AIRBORNE == 0 {
                continue;
            }
            if approx(u.x - f.goal.0, u.y - f.goal.1) < RECON_RANGE {
                f.strike = None;
                u.flags |= fflags::RTB;
                u.wp = f.home;
                ev.push(SimEvent::CampaignPhotographed { unit: i, tag: f.tag });
            }
        }
        for e in ev {
            self.emit(e);
        }
    }

    /// A CAS aircraft of flight `tag` in reach of its point: the unit of its objective nearest
    /// the point that no rocket of its side is on its way to yet (nor `taken` this frame; any
    /// of them, if all are), else the other side's ground unit nearest it (None: nothing left
    /// there).
    pub(crate) fn cas_target_for(
        &self,
        tag: u32,
        x: i32,
        y: i32,
        blue: bool,
        taken: &[usize],
    ) -> Option<usize> {
        let c = self.campaign.as_ref()?;
        let listed = c.cas_units.iter().find(|(t, _)| *t == tag).map(|(_, u)| u.as_slice());
        let aimed = |k: usize| {
            taken.contains(&k)
                || c.missiles.iter().any(|m| m.ground == Some(k) && m.ground_aaa && m.blue == blue)
        };
        let alive: Vec<usize> = listed
            .unwrap_or_default()
            .iter()
            .map(|&k| k as usize)
            .filter(|&k| {
                self.units.get(k).is_some_and(|u| !u.destroyed)
                    && c.aaa_blue.get(k).copied().unwrap_or(false) != blue
            })
            .collect();
        let near = |k: &usize| approx(self.units[*k].x - x, self.units[*k].y - y);
        alive
            .iter()
            .copied()
            .filter(|&k| !aimed(k))
            .min_by_key(near)
            .or_else(|| alive.iter().copied().min_by_key(near))
            .or_else(|| self.cas_target(x, y, blue))
    }

    /// A CAS aircraft in reach of its point: the other side's ground unit nearest it (None:
    /// nothing left there).
    pub(crate) fn cas_target(&self, x: i32, y: i32, blue: bool) -> Option<usize> {
        let c = self.campaign.as_ref()?;
        (0..self.units.len())
            .filter(|&k| {
                let u = &self.units[k];
                !u.destroyed && c.aaa_blue.get(k).copied().unwrap_or(false) != blue
            })
            .map(|k| (k, approx(self.units[k].x - x, self.units[k].y - y)))
            .filter(|&(_, d)| d < CAS_RADIUS)
            .min_by_key(|&(_, d)| d)
            .map(|(k, _)| k)
    }

    /// Pilot `id` is in the air (not on the ground, not out of the world).
    pub fn pilot_in_the_air(&self, id: u8) -> bool {
        let p = self.pilot_ref(id as usize);
        p.id == id && !p.absent && !p.on_ground && p.eject == 0
    }

    /// Campaign: pilot `id` takes over the AI's aircraft `tag` in the air for a sortie from
    /// base `base` (as [`Sim::respawn_pilot`]). The aircraft leaves the AI without a word (no
    /// landing, no loss). Returns where his flight model starts, None if it is gone.
    pub fn take_control(
        &mut self,
        id: u8,
        tag: u32,
        base: usize,
        objective: Option<(i32, usize)>,
        airframe: Airframe,
    ) -> Option<PlayerState> {
        let i = self.flight_of_tag(tag).filter(|_| self.takeable(tag))?;
        let u = self.flights[i];
        let t = self.ac_type(u.spec);
        let full = (((t.range as i32 * self.fs) << 11) / (t.max_speed as i32).max(1)).max(1);
        // The frames of fuel left count down in 16 bits: a long-ranged type's from past 0x7FFF
        // (negative, from `full` - 0x10000 down), then on through 0 into the negative once
        // the fuel is out.
        let out = u.timer < 0 && u.timer > full - 0x1_0000;
        let left = if out { 0 } else { u.timer & 0xFFFF };
        let fuel = (10000.0 * (left as f64 / full as f64).clamp(0.2, 1.0)) as i32;
        // The AI flies at viewZ (`math::view_z`), the pilot at the true altitude.
        let alt = true_altitude(u.alt.max(0));
        let knots = u.speed.max(0) as f64 * M_PER_WU / KNOT;
        let st = PlayerState {
            x: u.wx,
            y: 0x10_0000 - u.wy,
            altitude: alt.clamp(0, u16::MAX as i32) as u16,
            heading: u.heading as u16,
            pitch: u.pitch as i16,
            roll: u.bank as i16,
            velocity: (knots * 27.0).round().clamp(0.0, u16::MAX as f64) as u16,
            gear_up: true,
            on_ground: false,
            fuel,
            hud_visible: true,
            ..Default::default()
        };
        self.flights[i] = FlightUnit::default();
        if let Some(c) = self.campaign.as_mut() {
            c.flights[i] = None;
        }
        self.respawn_pilot(id, base, objective, airframe);
        let back = self.pl.id as usize;
        self.activate(id as usize);
        if self.pl.id == id {
            self.pl.on_ground = false;
            self.pl.prev_on_ground = false;
            self.ingest(&st);
            self.pl.last_input = st.clone();
        }
        self.activate(back);
        Some(st)
    }

    /// Campaign: pilot `id` in the air leaves his aircraft to the AI: from where it is, an
    /// aircraft of type `spec` (tag `tag`, base `from`) flies `task` on (`home`: flies home),
    /// and his sortie ends ([`OUTCOME_HANDED_OVER`]). False: he is not in the air.
    #[allow(clippy::too_many_arguments)]
    pub fn hand_over(
        &mut self,
        id: u8,
        tag: u32,
        spec: i32,
        task: AiTask,
        from: usize,
        home: bool,
        blue: bool,
    ) -> bool {
        if !self.opts.campaign || !self.pilot_in_the_air(id) {
            return false;
        }
        let p = self.pilot_input(id);
        let alt = crate::math::view_z(p.altitude);
        let speed = (p.velocity as f64 / 27.0 * KNOT / M_PER_WU).round() as i32;
        let t = self.ac_type(spec).clone();
        let i = self.campaign_slot();
        let wy = 0x10_0000 - p.y;
        let mut u = FlightUnit {
            spec,
            wx: p.x,
            wy,
            x: (p.x >> 5) & 0xFFFF,
            y: (wy >> 5) & 0xFFFF,
            alt: alt.max(20),
            heading: s16(p.heading as i32),
            pitch: s16(p.pitch as i32),
            bank: s16(p.roll as i32),
            speed: speed.max(if t.kind == AircraftKind::Helicopter {
                1
            } else {
                t.max_speed as i32 / 2
            }),
            flags: fflags::EXISTS | fflags::AIRBORNE,
            wp: from as i32,
            timer: s16((((t.range as i32 * self.fs) << 11) / (t.max_speed as i32).max(1)) / 2),
            ..Default::default()
        };
        let mut f = self.mission_flight(spec, task, tag, blue, from);
        u.missile = f.classic_missile();
        if home {
            f.strike = None;
            f.strike_unit = false;
            u.flags |= fflags::RTB;
        } else if f.role == Role::Strike && f.strike_unit {
            f.strike = Some(0);
        }
        self.flights[i] = u;
        self.campaign.as_mut().expect("campaign").flights[i] = Some(f);
        self.end_sortie_pilot(id, OUTCOME_HANDED_OVER);
        true
    }
}

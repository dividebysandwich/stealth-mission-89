//! Campaign: helicopters bringing a friendly air defence unit's stockpile up again.
//!
//! * An AI UH-60 ([`Sim::spawn_supply`]) lifts off from the helipad of its airfield (the
//!   theater's, `TheaterTiles::helipads`), flies to the unit, comes down beside it and hovers
//!   there while the load goes down ([`HOVER_MAX`] metres up at most, within [`DROP_RADIUS`]
//!   of the unit, for [`HOVER_SECONDS`]), then flies back to the pad, lands and is gone. The
//!   drop is a [`SimEvent::CampaignFlightDelivered`].
//! * A player's UH-60 delivers the same way: [`Sim::pilot_over_unit`] is the unit he hovers
//!   at; the campaign counts the seconds.
//! * Delivered, the campaign has the unit's stockpile filled ([`Sim::restock`]).

use serde::{Deserialize, Serialize};

use crate::Sim;
use crate::aaa::{AaaKind, M_PER_MAP};
use crate::ai_loadout::{AiLoadout, FighterTask};
use crate::campaign::{CampaignFlight, Role};
use crate::csar::BLACKHAWK;
use crate::math::{bearing, cdiv, clamp_q, s16};
use crate::station::{initial_stock, mannable};
use crate::types::*;

/// A drop counts within this distance of the unit (m), this low over the ground (m), after
/// this long (s).
pub const DROP_RADIUS: f64 = 50.0;
pub const HOVER_MAX: f64 = 30.0;
pub const HOVER_SECONDS: i32 = 10;
/// Metres per world unit.
const M_PER_WU: f64 = M_PER_MAP / 32.0;
/// World units: the AI's hover height (about 5 m), its cruise height (about 150 m), the height
/// it lifts off to before it turns away from the pad.
const HOVER_ALT: i32 = 10;
const CRUISE_ALT: i32 = 300;
const LIFT_ALT: i32 = 60;
/// World units: it hovers this far from the unit, on the pad's side (about 25 m); it slows down
/// within this distance of its goal (about 1 km), and is there within this one (about 4 m).
const STANDOFF: f64 = 51.0;
const SLOW_DOWN: f64 = 2048.0;
const THERE: f64 = 8.0;
/// A helipad this close to the airfield (world units, about 4 km) is the airfield's.
const PAD_RANGE: f64 = 8192.0;

/// A resupply helicopter's run (world units, x east, y south).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyRun {
    /// The air defence unit (`Sim::aaa_units` index).
    pub unit: u16,
    /// Where it hovers for the drop, and the helipad it came from (and lands on).
    pub drop: (i32, i32),
    pub pad: (i32, i32),
    /// Frames hovering at the unit so far.
    pub hover: i32,
    /// The load is down: on the way home.
    pub delivered: bool,
}

/// The middle of AAA unit `u` (world units, x east, y south).
fn unit_centre(u: &crate::aaa::AaaUnit) -> (i32, i32) {
    ((u.x << 5) + 16, (u.y << 5) + 16)
}

impl Sim {
    /// The helipad of base object `base` (world units, x east, y south): the theater's nearest
    /// one, else a spot beside the airfield (where the insertions lift off).
    pub fn helipad(&self, base: usize) -> (i32, i32) {
        let o = self.objects[base];
        let at = (o.x << 5, o.y << 5);
        let d = |p: &(i32, i32)| ((p.0 - at.0) as f64).hypot((p.1 - at.1) as f64);
        self.tiles
            .as_ref()
            .and_then(|t| t.helipads.iter().min_by(|a, b| d(a).total_cmp(&d(b))))
            .filter(|p| d(p) < PAD_RANGE)
            .copied()
            .unwrap_or(((o.x + 6) << 5, o.y << 5))
    }

    /// Campaign: a friendly UH-60 lifts off from the helipad of base `from` with a load for AAA
    /// unit `unit`. Returns its flight unit.
    pub fn spawn_supply(&mut self, from: usize, unit: usize, tag: u32) -> usize {
        let i = self.campaign_slot();
        let pad = self.helipad(from);
        let at = self.aaa.get(unit).map_or(pad, unit_centre);
        // It hovers beside the unit, on its way in.
        let (dx, dy) = ((pad.0 - at.0) as f64, (pad.1 - at.1) as f64);
        let d = dx.hypot(dy).max(1.0);
        let drop = (at.0 + (dx / d * STANDOFF) as i32, at.1 + (dy / d * STANDOFF) as i32);
        let mut u = FlightUnit {
            spec: BLACKHAWK,
            wp: from as i32,
            wx: pad.0,
            wy: pad.1,
            alt: 0,
            speed: 1,
            heading: bearing(at.0 - pad.0, s16(pad.1 - at.1)),
            timer: 0x7FFF,
            ..Default::default()
        };
        u.x = (u.wx >> 5) & 0xFFFF;
        u.y = (u.wy >> 5) & 0xFFFF;
        u.flags = fflags::EXISTS | fflags::AIRBORNE | fflags::TAKEOFF;
        self.flights[i] = u;
        let c = self.campaign.as_mut().expect("campaign");
        c.flights[i] = Some(CampaignFlight {
            tag,
            role: Role::Supply,
            blue: true,
            goal: (at.0 >> 5, at.1 >> 5),
            target: None,
            cooldown: 0,
            loadout: AiLoadout::default(),
            task: FighterTask::Patrol,
            home: from as i32,
            anchor: (pad.0 >> 5, pad.1 >> 5),
            quiet: 0,
            burst: 0,
            killed_by_ai: false,
            killed_by: None,
            reported: false,
            strike: None,
            kamikaze: false,
            supply: Some(SupplyRun { unit: unit as u16, drop, pad, hover: 0, delivered: false }),
        });
        self.emit(SimEvent::AircraftTakeoff { unit: i, base: from });
        i
    }

    /// Flight unit `i` is a resupply helicopter.
    pub(crate) fn supply_run(&self, i: usize) -> Option<SupplyRun> {
        self.campaign_flight(i).and_then(|f| f.supply)
    }

    /// One frame of a resupply helicopter: up from the pad, to the unit, the drop, back to the
    /// pad and down.
    pub(crate) fn fly_supply(&mut self, i: usize) {
        let Some(mut run) = self.supply_run(i) else { return };
        let fs = self.fs.max(1);
        let max = self.ac_type(self.flights[i].spec).max_speed as i32;
        let mut u = self.flights[i];
        let (gx, gy) = if run.delivered { run.pad } else { run.drop };
        let (dx, dy) = (gx - u.wx, gy - u.wy);
        let d = (dx as f64).hypot(dy as f64);
        if u.flags & fflags::TAKEOFF != 0 {
            // Straight up off the pad.
            u.alt += cdiv(40, fs).max(1);
            if u.alt >= LIFT_ALT {
                u.flags &= !fflags::TAKEOFF;
            }
        } else {
            // Speed: full on the way, slowing down into the hover (never 0 in the air: a
            // stopped aircraft counts as parked).
            let want = if d > SLOW_DOWN {
                max
            } else if d < THERE {
                0
            } else {
                ((max as f64 * d / SLOW_DOWN) as i32).clamp(8, max)
            };
            let acc = cdiv(max, 6 * fs).max(1);
            u.speed = (u.speed + clamp_q(want - u.speed, -acc, acc)).max(1);
            if d > THERE {
                let brg = bearing(dx, s16(-dy));
                let turn = clamp_q(s16(brg - u.heading), -cdiv(0x2000, fs), cdiv(0x2000, fs));
                u.heading = s16(u.heading + turn);
            }
            // Height: cruising over the mountains, down near the goal (onto the pad at home).
            let galt = if d > SLOW_DOWN / 2.0 {
                CRUISE_ALT + self.terrain_ahead(&u)
            } else if run.delivered && d < 2.0 * THERE {
                0
            } else if run.delivered {
                LIFT_ALT
            } else {
                HOVER_ALT
            };
            let rate = cdiv(30, fs).max(1);
            u.alt = (u.alt + clamp_q(galt - u.alt, -rate, rate)).max(0);
            // At the goal it holds the spot.
            let mv = if d < THERE { 0 } else { cdiv(u.speed, fs).max(1).min(d as i32) };
            u.wx = u.wx.wrapping_add(self.trig.sin_mul(u.heading, mv));
            u.wy = u.wy.wrapping_sub(self.trig.cos_mul(u.heading, mv));
            u.x = (u.wx >> 5) & 0xFFFF;
            u.y = (u.wy >> 5) & 0xFFFF;
        }
        // Nose down with speed, banked into turns (attitude only).
        u.pitch = -(u.speed * 0x600 / max.max(1));
        u.bank = clamp_q(s16(bearing(dx, s16(-dy)) - u.heading) * 2, -0x1800, 0x1800)
            * (u.speed > 20) as i32;
        // The drop: low and close to the unit for long enough.
        let unit = run.unit as usize;
        if !run.delivered {
            let near = self.aaa.get(unit).filter(|a| !a.destroyed).is_some_and(|a| {
                let c = unit_centre(a);
                let m = ((u.wx - c.0) as f64).hypot((u.wy - c.1) as f64) * M_PER_WU;
                m <= DROP_RADIUS && u.alt as f64 * M_PER_WU <= HOVER_MAX
            });
            run.hover = if near { run.hover + 1 } else { 0 };
            if run.hover >= HOVER_SECONDS * fs {
                run.delivered = true;
                let tag = self.campaign_flight(i).map_or(0, |f| f.tag);
                self.emit(SimEvent::CampaignFlightDelivered { unit: i, tag, aaa: unit });
            }
            // The unit is gone: home with the load.
            if self.aaa.get(unit).is_none_or(|a| a.destroyed) {
                run.delivered = true;
            }
        } else if d < 2.0 * THERE && u.alt == 0 {
            // Down on the pad: done (the campaign's bookkeeping reports it and frees the slot).
            u.flags &= 0x1C1;
            u.speed = 0;
        }
        self.flights[i] = u;
        if let Some(Some(f)) = self.campaign.as_mut().and_then(|c| c.flights.get_mut(i)) {
            f.supply = Some(run);
        }
    }

    /// The friendly Patriot radar or Gepard (AAA unit) pilot `id`'s helicopter hovers at for a
    /// drop now: within [`DROP_RADIUS`] of it, at most [`HOVER_MAX`] over the ground, slow.
    pub fn pilot_over_unit(&self, id: u8) -> Option<usize> {
        let p = self.pilot_ref(id as usize);
        if p.id != id || p.absent || !p.helicopter || p.eject != 0 {
            return None;
        }
        if p.last_input.height as f64 * M_PER_WU > HOVER_MAX || p.p.knots > 15 {
            return None;
        }
        // The pilot's world units run north, a unit's south.
        let (east, south) = (p.p.wx, 0x10_0000 - p.p.wy);
        (0..self.aaa.len()).find(|&k| {
            let a = &self.aaa[k];
            let c = unit_centre(a);
            !a.destroyed
                && mannable(a.kind)
                && self.aaa_blue(k)
                && ((east - c.0) as f64).hypot((south - c.1) as f64) * M_PER_WU <= DROP_RADIUS
        })
    }

    /// Campaign: a load delivered to AAA unit `k` (a Patriot radar's battery, or a Gepard)
    /// fills its stockpile.
    pub fn restock(&mut self, k: usize) {
        if self.campaign.is_none() {
            return;
        }
        self.defence_sizes();
        let Some(kind) = self.aaa.get(k).map(|a| a.kind) else { return };
        if !matches!(kind, AaaKind::PatriotRadar | AaaKind::Gepard) {
            return;
        }
        let d = &mut self.campaign.as_mut().expect("campaign").defence;
        d.stock[k] = d.stock[k].max(initial_stock(kind));
    }
}

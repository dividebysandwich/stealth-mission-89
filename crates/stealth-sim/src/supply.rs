//! Campaign: helicopters bringing a friendly air defence unit's stockpile up again.
//!
//! * An AI UH-60 ([`Sim::spawn_supply`]) lifts off from the helipad of its airfield (the
//!   theater's, `TheaterTiles::helipads`), hovers low over its crate beside the runway while
//!   it is hooked and winched up, flies to the unit with the crate slung under it, comes down
//!   beside it until the crate stands on the ground and hovers there while it is unhooked
//!   ([`HOVER_MAX`] metres up at most, within [`DROP_RADIUS`] of the unit, for
//!   [`HOVER_SECONDS`]), then flies back to the pad, lands and is gone. The drop is a
//!   [`SimEvent::CampaignFlightDelivered`]. A cargo helicopter for a new site does the same
//!   with its load, setting it down by the site's helipad.
//! * A player's UH-60 delivers the same way: [`Sim::pilot_over_unit`] is the unit he hovers
//!   at; the campaign counts the seconds.
//! * Delivered, the campaign has the unit's stockpile filled ([`Sim::restock`]).

use serde::{Deserialize, Serialize};

use crate::Sim;
use crate::aaa::M_PER_MAP;
use crate::ai_loadout::{AiLoadout, FighterTask};
use crate::campaign::{CampaignFlight, Role};
use crate::csar::BLACKHAWK;
use crate::math::{bearing, cdiv, clamp_q, s16};
use crate::station::{initial_stock, initial_stock2, initial_stock3, mannable};
use crate::types::*;

/// A drop counts within this distance of the unit (m), this low over the ground (m), after
/// this long (s).
pub const DROP_RADIUS: f64 = 50.0;
pub const HOVER_MAX: f64 = 30.0;
pub const HOVER_SECONDS: i32 = 10;
/// Metres per world unit.
const M_PER_WU: f64 = M_PER_MAP / 32.0;
/// World units: the AI's height hooking a crate (about 12 m: the hook takes it from there)
/// and setting one down (about 8 m: it hangs 9 m under the hook), its cruise height (about
/// 150 m), the height it lifts off to before it turns away from the pad.
pub const HOOK_ALT: i32 = 24;
pub const SET_ALT: i32 = 16;
const CRUISE_ALT: i32 = 300;
const LIFT_ALT: i32 = 60;
/// Seconds it hovers over its crate, hooked, while the winch lifts it off the ground.
pub const HOOK_SECONDS: i32 = 4;
/// World units: it hovers this far from the unit, on the pad's side (about 25 m); it slows down
/// within this distance of its goal (about 1 km), comes down within this one (about 16 m) and
/// is there within this one (about 4 m).
const STANDOFF: f64 = 51.0;
const SLOW_DOWN: f64 = 2048.0;
const LOW: f64 = 32.0;
const THERE: f64 = 8.0;
/// A helipad this close to the airfield (world units, about 4 km) is the airfield's.
const PAD_RANGE: f64 = 8192.0;

/// A cargo run's "unit": it lands its load on a helipad (the run's drop) instead.
pub const NO_UNIT: u16 = u16::MAX;

/// A resupply helicopter's run (world units, x east, y south).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyRun {
    /// The air defence unit (`Sim::units` index; `NO_UNIT`: cargo for a new site's
    /// helipad).
    pub unit: u16,
    /// Where it hovers for the drop, and the helipad it came from (and lands on).
    pub drop: (i32, i32),
    pub pad: (i32, i32),
    /// Frames hovering at the unit (or over the crate, hooked) so far.
    pub hover: i32,
    /// The load is down: on the way home.
    pub delivered: bool,
    /// Where its crate stands waiting for it (world units; None: off the ground, or no crate).
    #[serde(default)]
    pub pickup: Option<(i32, i32)>,
    /// The crate hangs under it; it set the crate down at the drop.
    #[serde(default)]
    pub hooked: bool,
    #[serde(default)]
    pub set_down: bool,
}

/// The middle of AAA unit `u` (world units, x east, y south).
fn unit_centre(u: &crate::aaa::GroundUnit) -> (i32, i32) {
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

    /// The theater's runways (metres east, north).
    pub fn runways(&self) -> &[crate::Runway] {
        self.tiles.as_ref().map_or(&[], |t| &t.runways)
    }

    /// Campaign: a friendly UH-60 lifts off from the helipad of base `from` to sling the crate
    /// at `pickup` (world units; None: it has the load aboard) to AAA unit `unit`. Returns its
    /// flight unit.
    pub fn spawn_supply(
        &mut self,
        from: usize,
        unit: usize,
        pickup: Option<(i32, i32)>,
        tag: u32,
    ) -> usize {
        let pad = self.helipad(from);
        let at = self.units.get(unit).map_or(pad, unit_centre);
        // It hovers beside the unit, on its way in.
        let (dx, dy) = ((pad.0 - at.0) as f64, (pad.1 - at.1) as f64);
        let d = dx.hypot(dy).max(1.0);
        let drop = (at.0 + (dx / d * STANDOFF) as i32, at.1 + (dy / d * STANDOFF) as i32);
        self.launch_supply(from, unit as u16, pickup, drop, tag)
    }

    /// Campaign: a friendly UH-60 lifts off from the helipad of base `from` to sling the crate
    /// at `pickup` (world units; None: it has the load aboard) to `drop` (world units) by the
    /// helipad of a new air defence site, sets it down there and flies back. Returns its flight
    /// unit.
    pub fn spawn_cargo(
        &mut self,
        from: usize,
        pickup: Option<(i32, i32)>,
        drop: (i32, i32),
        tag: u32,
    ) -> usize {
        self.launch_supply(from, NO_UNIT, pickup, drop, tag)
    }

    /// A resupply or cargo helicopter off the pad of base `from`, by its crate at `pickup`,
    /// towards its drop (world units) for AAA unit `unit` (`NO_UNIT`: a helipad).
    fn launch_supply(
        &mut self,
        from: usize,
        unit: u16,
        pickup: Option<(i32, i32)>,
        drop: (i32, i32),
        tag: u32,
    ) -> usize {
        let i = self.campaign_slot();
        let pad = self.helipad(from);
        let at = pickup.unwrap_or(drop);
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
            goal: (drop.0 >> 5, drop.1 >> 5),
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
            supply: Some(SupplyRun {
                unit,
                drop,
                pad,
                hover: 0,
                delivered: false,
                pickup,
                hooked: false,
                set_down: false,
            }),
        });
        self.emit(SimEvent::AircraftTakeoff { unit: i, base: from });
        i
    }

    /// Flight unit `i` is a resupply helicopter.
    pub(crate) fn supply_run(&self, i: usize) -> Option<SupplyRun> {
        self.campaign_flight(i).and_then(|f| f.supply)
    }

    /// One frame of a resupply helicopter: up from the pad, over its crate to hook it, to the
    /// unit, the drop, back to the pad and down.
    pub(crate) fn fly_supply(&mut self, i: usize) {
        let Some(mut run) = self.supply_run(i) else { return };
        let fs = self.fs.max(1);
        // Slower with a crate swinging under it.
        let full = self.ac_type(self.flights[i].spec).max_speed as i32;
        let max = if run.hooked && run.pickup.is_none() { full * 3 / 4 } else { full };
        let mut u = self.flights[i];
        let fetching = run.pickup.filter(|_| !run.delivered);
        let (gx, gy) = if run.delivered { run.pad } else { fetching.unwrap_or(run.drop) };
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
            // Height: cruising over the mountains, low near the goal, and right over it down
            // onto the pad at home, to the hook's reach over the crate, or until the crate
            // stands on the ground at the drop.
            let galt = if d > SLOW_DOWN / 2.0 {
                CRUISE_ALT + self.terrain_ahead(&u)
            } else if d >= LOW {
                LIFT_ALT
            } else if run.delivered {
                if d < 2.0 * THERE { 0 } else { LIFT_ALT }
            } else if fetching.is_some() {
                HOOK_ALT
            } else {
                SET_ALT
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
        let unit = run.unit as usize;
        let over = d < THERE * 2.0 && u.alt <= HOOK_ALT + 2;
        if fetching.is_some() {
            // Over the crate: hooked, it holds the spot while the winch lifts it.
            run.hooked |= over;
            run.hover += run.hooked as i32;
            if run.hover >= HOOK_SECONDS * fs {
                run.pickup = None;
                run.hover = 0;
            }
        } else if !run.delivered {
            // The drop: the crate on the ground (low over the drop), close to the unit (by a new
            // site's helipad), for long enough.
            let near_point = |c: (i32, i32)| {
                let m = ((u.wx - c.0) as f64).hypot((u.wy - c.1) as f64) * M_PER_WU;
                m <= DROP_RADIUS && u.alt as f64 * M_PER_WU <= HOVER_MAX
            };
            let set_down = d < 2.0 * THERE && u.alt <= SET_ALT;
            let near = set_down
                && if run.unit == NO_UNIT {
                    near_point(run.drop)
                } else {
                    self.units
                        .get(unit)
                        .filter(|a| !a.destroyed)
                        .is_some_and(|a| near_point(unit_centre(a)))
                };
            run.hover = if near { run.hover + 1 } else { 0 };
            if run.hover >= HOVER_SECONDS * fs {
                run.delivered = true;
                run.set_down = run.hooked;
                run.hooked = false;
                let tag = self.campaign_flight(i).map_or(0, |f| f.tag);
                let aaa = if run.unit == NO_UNIT { usize::MAX } else { unit };
                self.emit(SimEvent::CampaignFlightDelivered { unit: i, tag, aaa });
            }
            // The unit is gone: home with the load.
            if run.unit != NO_UNIT && self.units.get(unit).is_none_or(|a| a.destroyed) {
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
        (0..self.units.len()).find(|&k| {
            let a = &self.units[k];
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
        let Some(kind) = self.units.get(k).map(|a| a.kind) else { return };
        if !mannable(kind) {
            return;
        }
        let d = &mut self.campaign.as_mut().expect("campaign").defence;
        d.stock[k] = d.stock[k].max(initial_stock(kind));
        d.stock2[k] = d.stock2[k].max(initial_stock2(kind));
        d.stock3[k] = d.stock3[k].max(initial_stock3(kind));
    }
}

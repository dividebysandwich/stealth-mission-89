//! A single mission's choice of enemy aircraft (`MissionSettings::enemy_aircraft`): none at
//! all (the target of an air mission still flies), half the patrols up at the start and two
//! scrambles at most, the classic game's, or a second patrol beside each and the bases
//! scrambling twice as often.

use stealth_mission::Amount;

use crate::Sim;
use crate::math::s16;
use crate::rng::Rng;
use crate::types::*;

/// Scrambles a base may still make on a low level.
const LOW_SCRAMBLES: u8 = 2;
/// How far (map units) a high level's second patrol starts from the first: 5 to 10 km.
const PAIR_NEAR: i32 = 320;
const PAIR_FAR: i32 = 640;

impl Sim {
    /// The enemy's aircraft as `level` asks (nothing changes on `Normal` or in a campaign).
    pub(crate) fn set_enemy_aircraft(&mut self, level: Amount, seed: u32) {
        if self.opts.campaign || level == Amount::Normal {
            return;
        }
        self.enemy_aircraft = level;
        let n = self.mission_flights.min(self.flights.len());
        // The mission's special aircraft (an air mission's target) stays.
        let special = |i: usize| i == 0 && self.escort_leader == 0;
        let enemy: Vec<usize> = (0..n)
            .filter(|&i| {
                !self.is_traffic(i) && !special(i) && self.flights[i].flags & fflags::EXISTS != 0
            })
            .collect();
        let up = |u: &FlightUnit| u.flags & fflags::AIRBORNE != 0;
        let patrols: Vec<usize> = enemy.iter().copied().filter(|&i| up(&self.flights[i])).collect();
        match level {
            Amount::None => {
                for i in enemy {
                    self.flights[i] = FlightUnit::default();
                }
                self.scrambles_left = Some(0);
            }
            Amount::Low => {
                for i in patrols.into_iter().skip(1).step_by(2) {
                    self.flights[i] = FlightUnit::default();
                }
                self.scrambles_left =
                    Some(self.scrambles_left.map_or(LOW_SCRAMBLES, |s| s.min(LOW_SCRAMBLES)));
            }
            Amount::High => {
                // A parked aircraft (not an escort of the special one) goes up beside each patrol.
                let mut parked = enemy.into_iter().filter(|&i| {
                    let u = &self.flights[i];
                    !up(u) && u.flags & fflags::ESCORT == 0
                });
                let pairs: Vec<(usize, usize)> =
                    patrols.iter().filter_map(|&p| parked.next().map(|q| (p, q))).collect();
                let mut rng = Rng::new(seed ^ 0xE4E3_A1F0);
                for (p, q) in pairs {
                    let a = self.flights[p];
                    let d = PAIR_NEAR + rng.random(PAIR_FAR - PAIR_NEAR);
                    let h = rng.random(3600) as f64 * std::f64::consts::TAU / 3600.0;
                    let (dx, dy) = ((h.sin() * d as f64) as i32, (h.cos() * d as f64) as i32);
                    let (x, y) = ((a.x + dx).clamp(0x400, 0x7BFF), (a.y + dy).clamp(0x400, 0x7BFF));
                    let spec = self.flights[q].spec;
                    self.flights[q] = FlightUnit {
                        x,
                        y,
                        wx: x << 5,
                        wy: y << 5,
                        heading: s16(a.heading + rng.random(0x4000) - 0x2000),
                        spec,
                        ..a
                    };
                }
            }
            Amount::Normal => {}
        }
    }

    /// How often the enemy's bases scramble: a mask on the mission clock (fewer bits, more
    /// often).
    pub(crate) fn scramble_mask(&self) -> i32 {
        if self.enemy_aircraft == Amount::High { 3 } else { 7 }
    }
}

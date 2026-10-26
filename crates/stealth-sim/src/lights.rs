//! Aircraft exterior lights: position and anti-collision lights
//! are on around the aircraft's own airfields, where traffic has to see each other, and off
//! elsewhere (in combat they would give the aircraft away). The gear and the landing lights
//! with it are down on the ground, on the climb out and from the final approach on (a
//! helicopter's landing lights go with its position lights).

use crate::math::approx;
use crate::types::{FlightUnit, GroundObject, oflags};
use crate::{Sim, fflags};

/// Distance from an airfield of the aircraft's own side within which its lights are on (map
/// units: 10 nm).
pub const HOME_LIGHTS_RANGE: i32 = 0x4A0;

/// Whether map point (x, y) is within `HOME_LIGHTS_RANGE` of an airfield or carrier of the
/// given side. Neutral (civil) airfields count for both sides.
pub fn near_home_airfield(objects: &[GroundObject], x: i32, y: i32, friendly: bool) -> bool {
    objects.iter().skip(3).any(|o| {
        o.flags & (oflags::AIRFIELD | oflags::CARRIER) != 0
            && (o.flags & oflags::NEUTRAL != 0 || (o.flags & oflags::FRIENDLY != 0) == friendly)
            && approx(o.x - x, o.y - y) < HOME_LIGHTS_RANGE
    })
}

/// Altitude (ft) a departing aircraft raises its gear at: clear of the runway (12) and of a
/// carrier's deck (140).
pub const GEAR_UP_ALT: i32 = 300;

/// Whether flight unit `u` has its gear down: on the ground (the sim holds it 12 ft up) or
/// parked, taking off until `GEAR_UP_ALT`, and on the final approach and the rollout.
pub fn gear_down(u: &FlightUnit) -> bool {
    u.alt <= 12
        || u.speed == 0
        || u.flags & (fflags::FINAL | fflags::ROLLOUT) != 0
        || (u.flags & fflags::TAKEOFF != 0 && u.alt < GEAR_UP_ALT)
}

impl Sim {
    /// Whether flight unit `i` shows its lights: near an airfield of its side, and not parked
    /// (an unmanned aircraft is dark) or destroyed.
    pub fn aircraft_lights(&self, i: usize) -> bool {
        let Some(u) = self.flights.get(i) else { return false };
        u.flags & fflags::EXISTS != 0
            && u.flags & fflags::DESTROYED == 0
            && u.speed != 0
            && near_home_airfield(&self.objects, u.x, u.y, self.friendly_unit(i, u.spec))
    }

    /// Whether flight unit `i` has its landing lights on: its gear down, moving (not parked)
    /// and not destroyed.
    pub fn aircraft_landing_lights(&self, i: usize) -> bool {
        let Some(u) = self.flights.get(i) else { return false };
        u.flags & fflags::EXISTS != 0
            && u.flags & fflags::DESTROYED == 0
            && u.speed != 0
            && gear_down(u)
    }

    /// Whether the player's aircraft shows its lights (the rule of the friendly aircraft).
    pub fn player_lights(&self) -> bool {
        self.friendly_lights_at(self.pl.p.mx, self.pl.p.my)
    }

    /// Whether a friendly aircraft at map point (x, y) shows its lights (another player's).
    pub fn friendly_lights_at(&self, x: i32, y: i32) -> bool {
        near_home_airfield(&self.objects, x, y, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(x: i32, y: i32, flags: i32) -> GroundObject {
        GroundObject { x, y, flags, ..Default::default() }
    }

    #[test]
    fn lights_near_own_side_airfields() {
        let mut objects = vec![GroundObject::default(); 3];
        objects.push(base(0x1000, 0x1000, oflags::AIRFIELD | oflags::FRIENDLY));
        objects.push(base(0x3000, 0x1000, oflags::AIRFIELD));
        objects.push(base(0x5000, 0x1000, oflags::AIRFIELD | oflags::NEUTRAL));
        objects.push(base(0x7000, 0x1000, oflags::CARRIER | oflags::FRIENDLY));
        // Friendly base: friendly aircraft only, within range.
        assert!(near_home_airfield(&objects, 0x1000 + 0x400, 0x1000, true));
        assert!(!near_home_airfield(&objects, 0x1000 + 0x500, 0x1000, true));
        assert!(!near_home_airfield(&objects, 0x1000, 0x1000, false));
        // Enemy base: enemy aircraft only.
        assert!(near_home_airfield(&objects, 0x3000, 0x1100, false));
        assert!(!near_home_airfield(&objects, 0x3000, 0x1100, true));
        // Neutral airfield: both; friendly carrier: friendly aircraft.
        assert!(
            near_home_airfield(&objects, 0x5000, 0x1000, true)
                && near_home_airfield(&objects, 0x5000, 0x1000, false)
        );
        assert!(near_home_airfield(&objects, 0x7000, 0x1000, true));
        // The first three objects are not airfields.
        let first = vec![base(0x1000, 0x1000, oflags::AIRFIELD | oflags::FRIENDLY); 3];
        assert!(!near_home_airfield(&first, 0x1000, 0x1000, true));
    }

    #[test]
    fn gear_up_after_takeoff_down_on_final() {
        let unit = |alt, speed, flags| FlightUnit { alt, speed, flags, ..Default::default() };
        let air = fflags::EXISTS | fflags::AIRBORNE;
        // Parked and rolling on the runway.
        assert!(gear_down(&unit(12, 0, fflags::EXISTS)));
        assert!(gear_down(&unit(12, 10, air | fflags::TAKEOFF)));
        // Climbing out (from a carrier deck too), then raised.
        assert!(gear_down(&unit(140, 100, air | fflags::TAKEOFF)));
        assert!(!gear_down(&unit(GEAR_UP_ALT, 400, air | fflags::TAKEOFF)));
        assert!(!gear_down(&unit(5000, 400, air)));
        assert!(!gear_down(&unit(2000, 400, air | fflags::RTB)));
        // Lined up for landing and rolling out.
        assert!(gear_down(&unit(2000, 400, air | fflags::RTB | fflags::FINAL)));
        assert!(gear_down(&unit(140, 200, air | fflags::FINAL | fflags::ROLLOUT)));
    }
}

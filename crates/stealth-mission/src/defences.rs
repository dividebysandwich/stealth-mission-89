//! A single mission's choice of enemy air defences, applied to the generated world: no SAM
//! sites, about half of them a generation older, the classic game's, or all of them a
//! generation newer with none left off the air or under repair.

use crate::Amount;
use crate::rng::MscRand;
use crate::tables::MissionTables;
use crate::world::{MissionWorld, flags as F};

/// The enemy's SAM sites in `w` (ground units with a missile radar, ships too) as `level`
/// asks. `original` is the theater's world before the generator's edits (the sites it put
/// under repair); `seed` picks the sites a low level leaves out.
pub fn apply(
    w: &mut MissionWorld,
    level: Amount,
    x: &MissionTables,
    original: &MissionWorld,
    seed: u32,
) {
    if level == Amount::Normal {
        return;
    }
    let sam = |t: i16| t > 0 && x.sam_radars.get(t as usize).copied().unwrap_or(false);
    let step = |t: i16, up: bool| {
        x.threat_steps.get(t as usize).map_or(t, |&(u, d)| if up { u } else { d })
    };
    let mut rng = MscRand::new(seed ^ 0x5A4D_DEF5);
    let n = (w.ground_unit_count as usize).min(w.objects.len());
    for i in 0..n {
        let o = &mut w.objects[i];
        if o.flags & (F::FRIENDLY | F::NEUTRAL) != 0 {
            continue;
        }
        // A site the generator put under repair stands again on a high level.
        if level == Amount::High && o.unit_type == 0 && o.flags & F::UNDER_REPAIR != 0 {
            let was = original.objects.get(i).map_or(0, |o| o.unit_type);
            if sam(was) {
                o.unit_type = was;
                o.flags &= !F::UNDER_REPAIR;
            }
        }
        if !sam(o.unit_type) {
            continue;
        }
        o.unit_type = match level {
            Amount::None => 0,
            Amount::Low if rng.r(2) == 0 => 0,
            Amount::Low => step(o.unit_type, false),
            _ => step(o.unit_type, true),
        };
        if level == Amount::High {
            o.flags &= !F::OFF_THE_AIR;
        }
    }
}

/// The enemy's SAM sites standing in `w` (object indices).
pub fn sam_sites(w: &MissionWorld, x: &MissionTables) -> Vec<usize> {
    let n = (w.ground_unit_count as usize).min(w.objects.len());
    (0..n)
        .filter(|&i| {
            let o = &w.objects[i];
            o.flags & (F::FRIENDLY | F::NEUTRAL) == 0
                && o.unit_type > 0
                && x.sam_radars.get(o.unit_type as usize).copied().unwrap_or(false)
        })
        .collect()
}

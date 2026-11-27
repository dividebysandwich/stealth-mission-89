//! The F-14's two seats: one panel for the crew (a control moved in one seat moves in the
//! other's cockpit, `CrewCmd::Panel`, as the A-6's), the RIO's LAUNCH fired by the pilot's
//! client (whose AWG-9 aims the missiles), and the automatic RIO who works the back seat while
//! nobody sits in it.

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use super::{DDD_RANGES, DddRange, LtsMode, Power, TID_RANGES, TidRange, TomcatPanel, WcsMode};
use crate::aircraft::AircraftKind;
use crate::flight::{Aircraft, Player};
use crate::intruder::crew::{Shared, crew_panel, share_step};
use crate::mfd::{ContactKind, ContactRef, Contacts, MfdState};
use crate::world::{WU, WorldPos};

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (share, remote_launch, rio_buttons, welcome, automatic, log)
            .chain()
            .after(crate::combat::CombatSystems)
            .before(super::avionics::radar_controls)
            .run_if(in_state(crate::settings::AppState::Flight)),
    );
}

/// Seconds between the automatic RIO's looks at the picture.
const LOOK: f32 = 0.5;
/// Seconds between his pushes of the IFF's button.
const IFF_EVERY: f32 = 2.0;
/// The automatic RIO designates a target of the route this close ahead (m, 20 nmi: inside the
/// pod's tracking range).
const DESIGNATE_RANGE: f64 = 37_000.0;
/// ... within this angle of the nose (rad).
const DESIGNATE_CONE: f64 = 1.0;
/// The antenna elevation the automatic RIO sets is whole degrees within this.
const ANTENNA_REACH: f32 = 30.0;

/// The crew's one panel: a control I move goes to the server for the other seat, the other's
/// comes to mine. Alone aboard nothing is sent.
fn share(
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    combat: Option<Res<crate::combat::Combat>>,
    mut campaign: Option<ResMut<crate::campaign::ActiveCampaign>>,
    mut panel: ResMut<TomcatPanel>,
    mut state: Local<Shared>,
) {
    let crew =
        combat.as_deref().filter(|_| settings.aircraft == AircraftKind::F14).and_then(crew_panel);
    let (Some((me, shared)), Some(c)) = (crew, campaign.as_deref_mut()) else {
        *state = Shared::default();
        return;
    };
    let (take, send) = share_step(time.elapsed_secs(), me, shared, panel.pack(), &mut state);
    if let Some(bits) = take {
        panel.unpack(&bits);
    }
    if let Some(cmd) = send {
        c.actions.push(crate::campaign::Action::Crew(cmd));
    }
}

/// Test aid `SM89_F14_LOG`: the panel's shared controls once a second.
fn log(
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    panel: Res<TomcatPanel>,
    hud: Res<crate::hud::HudState>,
    mfd: Res<MfdState>,
    radar: Res<crate::awg9::Awg9>,
    mut since: Local<f32>,
) {
    *since += time.delta_secs();
    if *since < 1.0 || settings.aircraft != AircraftKind::F14 {
        return;
    }
    *since = 0.0;
    if std::env::var_os("SM89_F14_LOG").is_some() {
        // The tracks by what the IFF found: friends, foes, unknown.
        let known = |k: Option<bool>| radar.tracks.iter().filter(|t| t.friendly == k).count();
        eprintln!(
            "f14 tracks friends={} foes={} unknown={}",
            known(Some(true)),
            known(Some(false)),
            known(None)
        );
        eprintln!(
            "f14 panel wcs={:?} tid={:?} hsd={:?} hooks={} launches={} iffs={} acm={} steer={:?} hud={:?} ils={}",
            panel.wcs,
            panel.tid_range,
            panel.hsd_mode,
            panel.hooks,
            panel.launches & 0xF,
            panel.iffs & 0xF,
            panel.acm_cover,
            panel.steer,
            hud.mode,
            mfd.ils
        );
    }
}

/// The selected weapon is a radar missile, the RIO's to launch.
fn radar_missile(sim: &stealth_sim::Sim) -> bool {
    let (bays, cur) = sim.bays();
    !sim.gun_selected()
        && bays.get(cur).is_some_and(|b| {
            b.1 > 0 && matches!(b.0, stealth_sim::PHOENIX_WEAPON | stealth_sim::SPARROW_WEAPON)
        })
}

/// Another player's LAUNCH in my back seat (the panel's press count, shared): my client fires
/// the selected Phoenix or Sparrow, at the track my AWG-9 has for it.
fn remote_launch(
    settings: Res<crate::settings::Settings>,
    combat: Option<Res<crate::combat::Combat>>,
    panel: Res<TomcatPanel>,
    mut last: Local<Option<u8>>,
    mut actions: MessageWriter<crate::input::ActionEvent>,
) {
    let count = panel.launches & 0xF;
    let owner = combat.as_deref().filter(|_| settings.aircraft == AircraftKind::F14).filter(|c| {
        let me = c.sim.active_pilot();
        c.sim.crew_view(me).is_some_and(|v| v.owner == me && v.gunner.is_some_and(|g| g != me))
    });
    let Some(c) = owner else {
        *last = None;
        return;
    };
    if last.is_some_and(|l| l != count) && radar_missile(&c.sim) {
        actions.write(crate::input::ActionEvent(crate::input::Action::FireWeapon));
    }
    *last = Some(count);
}

/// The RIO's WPN SEL (the panel's press count, shared): the pilot's client (mine, whichever
/// seat I sit in) steps to the next weapon.
fn rio_buttons(
    settings: Res<crate::settings::Settings>,
    mut combat: Option<ResMut<crate::combat::Combat>>,
    panel: Res<TomcatPanel>,
    mut last: Local<Option<u8>>,
) {
    let counts = panel.selects & 7;
    let owner =
        combat.as_deref_mut().filter(|_| settings.aircraft == AircraftKind::F14).filter(|c| {
            let me = c.sim.active_pilot();
            c.sim.crew_view(me).is_none_or(|v| v.owner == me)
        });
    let Some(c) = owner else {
        *last = None;
        return;
    };
    if last.is_some_and(|l| l != counts) {
        c.step_weapon();
    }
    *last = Some(counts);
}

/// Seconds after another player took my back seat at which my client pushes the IFF for him:
/// his radar starts with nothing identified, so the pushes (shared like any) bring his picture
/// up to what the pilot's already shows, while his game still loads and his radar finds them.
const WELCOME_IFF: [f32; 3] = [3.0, 10.0, 20.0];

fn welcome(
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    combat: Option<Res<crate::combat::Combat>>,
    mut panel: ResMut<TomcatPanel>,
    mut aboard: Local<Option<f32>>,
) {
    let crewed =
        combat.as_deref().filter(|_| settings.aircraft == AircraftKind::F14).is_some_and(|c| {
            let me = c.sim.active_pilot();
            c.sim.crew_view(me).is_some_and(|v| v.owner == me && v.gunner.is_some_and(|g| g != me))
        });
    if !crewed {
        *aboard = None;
        return;
    }
    let before = aboard.unwrap_or(0.0);
    let now = before + time.delta_secs();
    *aboard = Some(now);
    if WELCOME_IFF.iter().any(|&t| before < t && now >= t) {
        panel.press(super::Key::Iff);
    }
}

/// The DDD's shortest range that shows a contact `metres` away.
pub fn ddd_range_for(metres: f32) -> DddRange {
    DDD_RANGES
        .iter()
        .copied()
        .find(|r| r.miles() as f32 * 1852.0 * 0.9 >= metres)
        .unwrap_or(DddRange::Nm200)
}

/// The TID's shortest range that shows a contact `metres` away (from the middle of the tube).
pub fn tid_range_for(metres: f32) -> TidRange {
    TID_RANGES
        .iter()
        .copied()
        .find(|r| r.miles() as f32 * 1852.0 * 0.9 >= metres)
        .unwrap_or(TidRange::Nm400)
}

/// The WCS mode the automatic RIO wants: a single target track for a Sparrow or a Sidewinder
/// at the pilot's lock (going back to a search would let go of it), else track while scan (the
/// Phoenix's ranks in AUTO).
pub fn wanted_wcs(sparrow_or_sidewinder: bool, locked: bool) -> WcsMode {
    if sparrow_or_sidewinder && locked { WcsMode::PdStt } else { WcsMode::TwsAuto }
}

/// What the automatic RIO last set (he sets a control again only when he wants another
/// setting: a control moved meanwhile stays).
#[derive(Default)]
struct Rio {
    wcs: Option<WcsMode>,
    ddd: Option<DddRange>,
    tid: Option<TidRange>,
    pod: bool,
    /// Seconds until he may push the IFF again.
    iff: f32,
    /// The laser as he set it, the waypoint he lases.
    laser: Option<bool>,
    lased: Option<usize>,
    wait: f32,
}

/// The automatic radar intercept officer, while the back seat is empty (and the pilot has not
/// the ACM cover up): the radar in track while scan, or in a single target track on the pilot's
/// lock for a Sparrow; the IFF pushed for every new aircraft on the radar; the DDD's and the TID's ranges and the antenna on the nearest hostile;
/// with a bomb selected or laser-guided bombs aboard the LANTIRN up and the mission's primary
/// target, then the secondary, designated and lased while it is in reach ahead.
#[allow(clippy::too_many_arguments)]
fn automatic(
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    contacts: Res<Contacts>,
    waypoints: Res<crate::hud::Waypoints>,
    mut panel: ResMut<TomcatPanel>,
    mut mfd: ResMut<MfdState>,
    mut combat: Option<ResMut<crate::combat::Combat>>,
    player: Query<(&Aircraft, &WorldPos), With<Player>>,
    radar: Res<crate::awg9::Awg9>,
    mut rio: Local<Rio>,
) {
    rio.wait -= time.delta_secs();
    rio.iff -= time.delta_secs();
    if settings.aircraft != AircraftKind::F14 || settings.gunner_cockpit || rio.wait > 0.0 {
        return;
    }
    rio.wait = LOOK;
    let (Some(combat), Ok((a, pos))) = (combat.as_deref_mut(), player.single()) else { return };
    let me = combat.sim.active_pilot();
    if combat.sim.crew_view(me).is_some_and(|v| v.gunner.is_some_and(|g| g != me)) {
        return;
    }
    if panel.acm_cover {
        return;
    }
    let (bays, cur) = combat.sim.bays();
    let weapon = if combat.sim.gun_selected() { -1 } else { bays.get(cur).map_or(-1, |b| b.0) };
    let air_weapon = weapon < 0 || crate::loadout::AIR_TO_AIR.contains(&(weapon as u8));
    let locked =
        contacts.locked.is_some_and(|l| matches!(l, ContactRef::Unit(_) | ContactRef::Pilot(_)));
    // The radar's mode.
    // (Weapon 0 is the Sidewinder.)
    let wcs = wanted_wcs(weapon == stealth_sim::SPARROW_WEAPON || weapon == 0, locked);
    if rio.wcs != Some(wcs) {
        rio.wcs = Some(wcs);
        panel.wcs = wcs;
    }
    // The IFF: a push whenever the radar holds an aircraft it has not interrogated yet.
    let unknown = radar.hits.iter().any(|h| h.friendly.is_none())
        || radar.tracks.iter().any(|t| t.friendly.is_none())
        || radar.stt.is_some_and(|t| radar.identity(t.target).is_none());
    if unknown && rio.iff <= 0.0 {
        rio.iff = IFF_EVERY;
        panel.press(super::Key::Iff);
    }
    // The nearest hostile aircraft: the ranges and the antenna on it.
    let own = DVec3::new(pos.0.x, -pos.0.z, pos.0.y);
    let nearest = contacts
        .list
        .iter()
        .filter(|k| matches!(k.kind, ContactKind::Aircraft | ContactKind::Drone) && !k.friendly)
        .map(|k| DVec3::new(k.east * WU, k.north * WU, k.alt as f64 * WU) - own)
        .min_by(|p, q| p.length_squared().total_cmp(&q.length_squared()));
    if let Some(d) = nearest {
        let range = d.length() as f32;
        let ddd = ddd_range_for(range);
        if rio.ddd != Some(ddd) {
            rio.ddd = Some(ddd);
            panel.ddd_range = ddd;
        }
        let tid = tid_range_for(range);
        if rio.tid != Some(tid) {
            rio.tid = Some(tid);
            panel.tid_range = tid;
        }
        if !matches!(panel.wcs, WcsMode::PdStt | WcsMode::PulseStt) {
            let el = (d.z.atan2(d.truncate().length()) as f32).to_degrees();
            mfd.av.fcr_el = el.round().clamp(-ANTENNA_REACH, ANTENNA_REACH);
        }
    }
    // A bomb selected or laser-guided bombs aboard: the LANTIRN up, and the mission's primary
    // target lased while it is in reach ahead, the secondary once the primary is done (with a
    // bomb selected and neither left, the route's current point).
    let lgb = bays.iter().any(|b| b.1 > 0 && b.0 == LGB_WEAPON);
    let pod = !air_weapon || (lgb && !locked);
    if pod != rio.pod {
        rio.pod = pod;
        if pod {
            panel.lts_power = Power::On;
            panel.lts_mode = LtsMode::AirToGround;
        }
    }
    let here = DVec2::new(own.x, own.y);
    let heading = a.euler().0 as f64;
    let anchor = |i: usize| waypoints.anchors.get(i).copied().unwrap_or(i);
    let done = combat.sim.objectives();
    let target = lase_order(done.primary_done, done.secondary_done, !air_weapon)
        .into_iter()
        .flatten()
        .filter(|_| pod)
        .map(|k| if k == CURRENT { waypoints.current } else { anchor(k) })
        .filter_map(|i| waypoints.points.get(i).copied().map(|wp| (i, wp)))
        .find(|(_, wp)| {
            let d = DVec2::new(wp.east * WU, wp.north * WU) - here;
            let off = (d.x.atan2(d.y) - heading + std::f64::consts::PI)
                .rem_euclid(std::f64::consts::TAU)
                - std::f64::consts::PI;
            d.length() < DESIGNATE_RANGE && off.abs() < DESIGNATE_CONE
        });
    // The laser: armed and firing while he has a target, safe again without one.
    let lasing = target.is_some();
    if rio.laser != Some(lasing) {
        rio.laser = Some(lasing);
        panel.laser_armed = lasing;
        panel.lts_oper = lasing;
    }
    let Some((index, wp)) = target else {
        rio.lased = None;
        return;
    };
    // He designates his target once: a point the pilot designates meanwhile stays, until the
    // RIO's target changes.
    if rio.lased != Some(index) && (contacts.ground_lock.is_none() || rio.lased.is_some()) {
        let (x, y) = (wp.east / 32.0, 0x8000 as f64 - wp.north / 32.0);
        combat.designate_at(x.round() as i32, y.round() as i32);
        rio.lased = Some(index);
    }
}

/// The GBU-12's weapon type (the laser-guided bomb).
const LGB_WEAPON: i32 = 6;
/// `lase_order`'s mark for the route's current point (the others are the mission's anchors).
const CURRENT: usize = usize::MAX;

/// The points the automatic RIO lases, the first in reach wins: the mission's primary target
/// (anchor 1) until it is done, then the secondary (anchor 2), and with a bomb selected
/// (`bombing`) the route's current point after them.
pub fn lase_order(primary_done: bool, secondary_done: bool, bombing: bool) -> [Option<usize>; 3] {
    [(!primary_done).then_some(1), (!secondary_done).then_some(2), bombing.then_some(CURRENT)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ranges_show_the_nearest_hostile() {
        assert_eq!(ddd_range_for(15_000.0), DddRange::Nm10);
        assert_eq!(ddd_range_for(80_000.0), DddRange::Nm50);
        assert_eq!(ddd_range_for(1.0e6), DddRange::Nm200);
        assert_eq!(tid_range_for(30_000.0), TidRange::Nm25);
        assert_eq!(tid_range_for(150_000.0), TidRange::Nm100);
    }

    #[test]
    fn a_sparrow_on_a_lock_gets_a_single_target_track() {
        assert_eq!(wanted_wcs(true, true), WcsMode::PdStt);
        assert_eq!(wanted_wcs(true, false), WcsMode::TwsAuto);
        assert_eq!(wanted_wcs(false, true), WcsMode::TwsAuto);
        assert_eq!(wanted_wcs(false, false), WcsMode::TwsAuto);
    }

    #[test]
    fn the_rio_lases_the_primary_then_the_secondary() {
        assert_eq!(lase_order(false, false, false), [Some(1), Some(2), None]);
        assert_eq!(lase_order(true, false, false), [None, Some(2), None]);
        assert_eq!(lase_order(true, true, false), [None, None, None]);
        assert_eq!(lase_order(true, true, true)[2], Some(CURRENT));
    }
}

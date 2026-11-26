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
        (share, remote_launch, automatic, log)
            .chain()
            .after(crate::combat::CombatSystems)
            .before(super::avionics::radar_controls)
            .run_if(in_state(crate::settings::AppState::Flight)),
    );
}

/// Seconds between the automatic RIO's looks at the picture.
const LOOK: f32 = 0.5;
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
    mut since: Local<f32>,
) {
    *since += time.delta_secs();
    if *since < 1.0 || settings.aircraft != AircraftKind::F14 {
        return;
    }
    *since = 0.0;
    if std::env::var_os("SM89_F14_LOG").is_some() {
        eprintln!(
            "f14 panel wcs={:?} tid={:?} hsd={:?} hooks={} launches={} acm={}",
            panel.wcs,
            panel.tid_range,
            panel.hsd_mode,
            panel.hooks,
            panel.launches & 0xF,
            panel.acm_cover
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

/// The WCS mode the automatic RIO wants: a single target track for a Sparrow at the pilot's
/// lock, else track while scan (the Phoenix's ranks in AUTO).
pub fn wanted_wcs(sparrow: bool, locked: bool) -> WcsMode {
    if sparrow && locked { WcsMode::PdStt } else { WcsMode::TwsAuto }
}

/// What the automatic RIO last set (he sets a control again only when he wants another
/// setting: a control moved meanwhile stays).
#[derive(Default)]
struct Rio {
    wcs: Option<WcsMode>,
    ddd: Option<DddRange>,
    tid: Option<TidRange>,
    pod: bool,
    wait: f32,
}

/// The automatic radar intercept officer, while the back seat is empty (and the pilot has not
/// the ACM cover up): the radar in track while scan, or in a single target track on the pilot's
/// lock for a Sparrow; the DDD's and the TID's ranges and the antenna on the nearest hostile;
/// with a bomb selected the LANTIRN up and the route's target ahead designated.
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
    mut rio: Local<Rio>,
) {
    rio.wait -= time.delta_secs();
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
    let wcs = wanted_wcs(weapon == stealth_sim::SPARROW_WEAPON, locked);
    if rio.wcs != Some(wcs) {
        rio.wcs = Some(wcs);
        panel.wcs = wcs;
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
    // A bomb: the LANTIRN up, the route's target ahead designated.
    let pod = !air_weapon;
    if pod != rio.pod {
        rio.pod = pod;
        if pod {
            panel.lts_power = Power::On;
            panel.lts_mode = LtsMode::AirToGround;
        }
    }
    if !pod || contacts.ground_lock.is_some() {
        return;
    }
    let here = DVec2::new(own.x, own.y);
    let heading = a.euler().0 as f64;
    let anchor = |i: usize| waypoints.anchors.get(i).copied().unwrap_or(i);
    let best = [anchor(1), anchor(2), waypoints.current]
        .into_iter()
        .filter_map(|i| waypoints.points.get(i).copied())
        .map(|wp| (wp, DVec2::new(wp.east * WU, wp.north * WU) - here))
        .filter(|(_, d)| {
            let off = (d.x.atan2(d.y) - heading + std::f64::consts::PI)
                .rem_euclid(std::f64::consts::TAU)
                - std::f64::consts::PI;
            d.length() < DESIGNATE_RANGE && off.abs() < DESIGNATE_CONE
        })
        .min_by(|p, q| p.1.length_squared().total_cmp(&q.1.length_squared()));
    if let Some((wp, _)) = best {
        let (x, y) = (wp.east / 32.0, 0x8000 as f64 - wp.north / 32.0);
        combat.designate_at(x.round() as i32, y.round() as i32);
    }
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
    }
}

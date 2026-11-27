//! The A-6's two seats: one panel for the crew (a control moved in one seat moves in the
//! other's cockpit, `CrewCmd::Panel`), and the automatic bombardier/navigator who works the
//! right seat while nobody sits in it.

use bevy::prelude::*;
use stealth_sim::crew::{CrewCmd, SharedPanel};

use super::computer::Computer;
use super::{
    ATTACK_MODES, CLEARANCES, FLIGHT_MODES, IntruderPanel, PHD_MODES, POWERS, PULL_GS, Power,
    RELEASES, SCALES, SEARCH_RANGES, SELECTS, SearchRange, TRACKS, VDI_DISPLAYS, VDI_RANGES,
};
use crate::aircraft::AircraftKind;
use crate::hud::{Waypoint, Waypoints};
use crate::world::WU;

/// Seconds a control I moved stays mine before the server's panel is taken again, and between
/// repeats of mine while the server has another.
const GRACE: f32 = 1.0;
const REPEAT: f32 = 0.5;
/// Seconds between the automatic bombardier/navigator's looks at the panel.
const LOOK: f32 = 1.0;

/// The keyed coordinates go to the other seat in steps of 130 m (twelve bits each); all ones:
/// none keyed.
const KEYED_STEP: u32 = 13;
const KEYED_NONE: u32 = 0xFF_FFFF;

fn index<T: PartialEq>(all: &[T], v: &T) -> u8 {
    all.iter().position(|a| a == v).unwrap_or(0) as u8
}

fn pick<T: Copy>(all: &[T], i: u8) -> T {
    all[(i as usize).min(all.len() - 1)]
}

impl IntruderPanel {
    /// The switches and buttons in 16 bytes (`SharedPanel::bits`), and the target's keyed
    /// coordinates to `KEYED_STEP`; the keyboard's presses and what is being keyed stay in
    /// their seat.
    pub fn pack(&self) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0] = index(&VDI_DISPLAYS, &self.vdi)
            | index(&VDI_RANGES, &self.range) << 2
            | (self.path_bright as u8) << 5
            | (self.one_scan as u8) << 6
            | (self.amti as u8) << 7;
        b[1] = (self.pitch_trim + super::PITCH_TRIM).clamp(0, 6) as u8
            | index(&PHD_MODES, &self.phd_mode) << 3
            | index(&POWERS, &self.phd_power) << 5
            | (self.master_arm as u8) << 7;
        b[2] = index(&CLEARANCES, &self.clearance)
            | index(&TRACKS, &self.track) << 2
            | index(&SCALES, &self.scale) << 4
            | (self.nav_transfer as u8) << 5
            | (self.nav_checkpoint as u8) << 6;
        b[3] = index(&FLIGHT_MODES, &self.flight) | self.nav.min(5) << 3;
        b[4] = self.attack.map_or(0, |m| index(&ATTACK_MODES, &m) + 1)
            | index(&PULL_GS, &self.pull_g) << 3
            | index(&RELEASES, &self.release) << 6;
        let stations = self
            .stations
            .iter()
            .enumerate()
            .fold(0u16, |acc, (i, s)| acc | (index(&SELECTS, s) as u16) << (i * 2));
        b[5..7].copy_from_slice(&stations.to_le_bytes());
        b[7..9].copy_from_slice(&self.interval.to_le_bytes());
        b[9] = index(&POWERS, &self.search_power)
            | index(&SEARCH_RANGES, &self.search_range) << 2
            | (self.photos & 0xF) << 4;
        b[10..12].copy_from_slice(&self.addresses.to_le_bytes());
        b[12] = self.erases;
        let keyed = self.keyed.map_or(KEYED_NONE, |g| {
            let q = |v: u16| (v as u32 + KEYED_STEP / 2) / KEYED_STEP;
            q(g[0]) | q(g[1]) << 12
        });
        b[13..16].copy_from_slice(&keyed.to_le_bytes()[..3]);
        b
    }

    /// Sets the switches and buttons from `pack`'s bytes.
    pub fn unpack(&mut self, b: &[u8; 16]) {
        self.vdi = pick(&VDI_DISPLAYS, b[0] & 3);
        self.range = pick(&VDI_RANGES, b[0] >> 2 & 7);
        self.path_bright = b[0] & 0x20 != 0;
        self.one_scan = b[0] & 0x40 != 0;
        self.amti = b[0] & 0x80 != 0;
        self.pitch_trim = (b[1] & 7).min(6) as i8 - super::PITCH_TRIM;
        self.phd_mode = pick(&PHD_MODES, b[1] >> 3 & 3);
        self.phd_power = pick(&POWERS, b[1] >> 5 & 3);
        self.master_arm = b[1] & 0x80 != 0;
        self.clearance = pick(&CLEARANCES, b[2] & 3);
        self.track = pick(&TRACKS, b[2] >> 2 & 3);
        self.scale = pick(&SCALES, b[2] >> 4 & 1);
        self.nav_transfer = b[2] & 0x20 != 0;
        self.nav_checkpoint = b[2] & 0x40 != 0;
        self.flight = pick(&FLIGHT_MODES, b[3] & 7);
        self.nav = (b[3] >> 3 & 7).min(5);
        self.attack = (b[4] & 7).checked_sub(1).map(|i| pick(&ATTACK_MODES, i));
        self.pull_g = pick(&PULL_GS, b[4] >> 3 & 7);
        self.release = pick(&RELEASES, b[4] >> 6 & 3);
        let stations = u16::from_le_bytes([b[5], b[6]]);
        for (i, s) in self.stations.iter_mut().enumerate() {
            *s = pick(&SELECTS, (stations >> (i * 2) & 3) as u8);
        }
        self.interval = u16::from_le_bytes([b[7], b[8]]).min(super::INTERVAL_MAX);
        self.search_power = pick(&POWERS, b[9] & 3);
        self.search_range = pick(&SEARCH_RANGES, b[9] >> 2 & 3);
        self.photos = (self.photos & !0xF) | b[9] >> 4;
        self.addresses = u16::from_le_bytes([b[10], b[11]]);
        self.erases = b[12];
        let keyed = u32::from_le_bytes([b[13], b[14], b[15], 0]);
        self.keyed = (keyed != KEYED_NONE).then(|| {
            let v = |q: u32| ((q & 0xFFF) * KEYED_STEP).min(super::GRID_MAX) as u16;
            [v(keyed), v(keyed >> 12)]
        });
    }
}

/// The bombardier/navigator's camera button (the panel's press count, shared): the pilot's
/// client (mine, whichever seat I sit in) takes a frame with the camera pod.
pub(super) fn camera(
    settings: Res<crate::settings::Settings>,
    mut combat: Option<ResMut<crate::combat::Combat>>,
    panel: Res<IntruderPanel>,
    mut last: Local<Option<u8>>,
) {
    let count = panel.photos & 0xF;
    let owner =
        combat.as_deref_mut().filter(|_| settings.aircraft == AircraftKind::A6).filter(|c| {
            let me = c.sim.active_pilot();
            c.sim.crew_view(me).is_none_or(|v| v.owner == me)
        });
    let Some(c) = owner else {
        *last = None;
        return;
    };
    if last.is_some_and(|l| l != count) {
        c.photo();
    }
    *last = Some(count);
}

/// What of the panel went where, and when I last moved a control.
#[derive(Default)]
pub(crate) struct Shared {
    /// The panel as it last agreed with the crew's.
    known: Option<[u8; 16]>,
    moved: Option<f32>,
    sent: f32,
}

/// Me and the crew's panel as the server has it (`CrewView::shared_panel`), when another player
/// is aboard my aircraft or I am aboard his (None alone, or not in a crew). Newly in another's
/// seat there is none yet either: the panel is the owner's as he has it (his comes first; mine
/// sent now would swap the two).
pub(crate) fn crew_panel(combat: &crate::combat::Combat) -> Option<(u8, Option<SharedPanel>)> {
    let me = combat.sim.active_pilot();
    let v = combat.sim.crew_view(me)?;
    let other = if v.owner == me { v.gunner } else { Some(v.owner) };
    other
        .filter(|&o| o != me && (v.owner == me || v.shared_panel.is_some()))
        .map(|_| (me, v.shared_panel))
}

/// A frame of the crew's one panel (any aircraft's, packed in 16 bytes): my panel `mine`
/// against the server's `shared`. Returns the other seat's panel to take (unless I am just
/// moving a control) and the command that sends mine (again until the server has it).
pub(crate) fn share_step(
    now: f32,
    me: u8,
    shared: Option<SharedPanel>,
    mine: [u8; 16],
    state: &mut Shared,
) -> (Option<[u8; 16]>, Option<CrewCmd>) {
    if state.known.is_some_and(|k| k != mine) {
        state.moved = Some(now);
    }
    let moving = state.moved.is_some_and(|t| now - t < GRACE);
    match shared {
        // The other seat's, unless I am just moving a control.
        Some(s) if s.by != me && s.bits != mine && !moving => {
            state.known = Some(s.bits);
            (Some(s.bits), None)
        }
        // Mine has not arrived (or a new crew has none yet): again.
        _ if (moving || state.known.is_none())
            && shared.is_none_or(|s| s.bits != mine)
            && now - state.sent >= REPEAT =>
        {
            state.sent = now;
            state.known = Some(mine);
            (None, Some(CrewCmd::Panel(SharedPanel { by: me, bits: mine })))
        }
        _ => {
            state.known = Some(mine);
            (None, None)
        }
    }
}

/// The crew's one panel: a control I move goes to the server for the other seat, the other's
/// comes to mine. Alone aboard nothing is sent.
pub(super) fn share(
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    combat: Option<Res<crate::combat::Combat>>,
    mut campaign: Option<ResMut<crate::campaign::ActiveCampaign>>,
    mut panel: ResMut<IntruderPanel>,
    mut state: Local<Shared>,
) {
    let crew =
        combat.as_deref().filter(|_| settings.aircraft == AircraftKind::A6).and_then(crew_panel);
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

/// The search radar's shortest range that shows a point `metres` away.
pub fn range_for(metres: f32) -> SearchRange {
    SEARCH_RANGES
        .iter()
        .copied()
        .find(|r| r.miles() as f32 * 1852.0 * 0.9 >= metres)
        .unwrap_or(SearchRange::Mi150)
}

/// The automatic bombardier/navigator, while the right seat is empty: with an attack on he
/// turns the search radar on, sets its range for the route's target, and when the target comes
/// onto the scope puts the cursors on it and locks, so that the computer steers and releases.
#[allow(clippy::too_many_arguments)]
pub(super) fn automatic(
    time: Res<Time>,
    settings: Res<crate::settings::Settings>,
    waypoints: Res<Waypoints>,
    computer: Res<Computer>,
    mut panel: ResMut<IntruderPanel>,
    mut combat: Option<ResMut<crate::combat::Combat>>,
    aircraft: Query<&crate::world::WorldPos, With<crate::flight::Player>>,
    mut wait: Local<f32>,
) {
    *wait -= time.delta_secs();
    if settings.aircraft != AircraftKind::A6 || settings.gunner_cockpit || *wait > 0.0 {
        return;
    }
    *wait = LOOK;
    let (Some(combat), Ok(pos)) = (combat.as_deref_mut(), aircraft.single()) else { return };
    let me = combat.sim.active_pilot();
    let aboard = combat.sim.crew_view(me).is_some_and(|v| v.gunner.is_some());
    if aboard || panel.attack.is_none() || computer.designated {
        return;
    }
    // The coordinates keyed, else the mission's targets (the route's primary and secondary
    // points) and the point steered to: the nearest of them ahead.
    let anchor = |i: usize| waypoints.anchors.get(i).copied().unwrap_or(i);
    let own = bevy::math::DVec2::new(pos.0.x, -pos.0.z);
    let steered = panel.keyed_waypoint().or_else(|| waypoints.target());
    let nose = aircraft_heading(&computer, own, steered);
    let candidates: Vec<Waypoint> = match panel.keyed_waypoint() {
        Some(k) => vec![k],
        None => [anchor(1), anchor(2), waypoints.current]
            .into_iter()
            .filter_map(|i| waypoints.points.get(i).copied())
            .collect(),
    };
    let best = candidates
        .iter()
        .map(|wp| {
            let d = bevy::math::DVec2::new(wp.east * WU, wp.north * WU) - own;
            (wp, d.length() as f32, (d.x as f32).atan2(d.y as f32))
        })
        .filter(|(_, _, brg)| nose.is_none_or(|h| super::computer::wrap(brg - h).abs() < 1.0))
        .min_by(|a, b| a.1.total_cmp(&b.1));
    let Some((wp, range, _)) = best else { return };
    if panel.search_power != Power::On {
        panel.search_power = Power::On;
    }
    let wanted = range_for(range);
    if panel.search_range != wanted {
        panel.search_range = wanted;
    }
    if range < wanted.miles() as f32 * 1852.0 {
        let (x, y) = (wp.east / 32.0, 0x8000 as f64 - wp.north / 32.0);
        combat.designate_at(x.round() as i32, y.round() as i32);
    }
}

/// The nose's heading (rad), from the computer's bearing to the point it steers to.
fn aircraft_heading(
    computer: &Computer,
    own: bevy::math::DVec2,
    steered: Option<Waypoint>,
) -> Option<f32> {
    let wp = steered?;
    computer.target.filter(|_| !computer.designated)?;
    let d = bevy::math::DVec2::new(wp.east * WU, wp.north * WU) - own;
    Some(super::computer::wrap((d.x as f32).atan2(d.y as f32) - computer.bearing))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intruder::Key;

    /// Every switch and button survives the packing: each control pressed in turn, the
    /// packed panel unpacks to the same.
    #[test]
    fn panel_packs() {
        let mut p = IntruderPanel::default();
        for (i, key) in Key::all().into_iter().enumerate() {
            p.press(key);
            if i % 3 == 0 {
                p.press(key);
            }
            let mut q = IntruderPanel::default();
            q.unpack(&p.pack());
            q.key_presses = p.key_presses;
            q.entry = p.entry;
            q.keying = p.keying;
            assert_eq!(q, p, "after {key:?}");
        }
    }

    /// Keyed coordinates reach the other seat to within a step, and none stays none.
    #[test]
    fn keyed_coordinates_pack() {
        let mut p = IntruderPanel::default();
        let mut q = IntruderPanel { keyed: Some([1, 1]), ..default() };
        q.unpack(&p.pack());
        assert_eq!(q.keyed, None);
        for g in [[0, 0], [6789, 12345], [51_200, 51_200]] {
            p.keyed = Some(g);
            q.unpack(&p.pack());
            let k = q.keyed.unwrap();
            assert!((0..2).all(|i| k[i].abs_diff(g[i]) as u32 <= KEYED_STEP / 2), "{g:?} {k:?}");
            assert_eq!(q.pack(), p.pack());
        }
    }

    #[test]
    fn radar_range_for_the_target() {
        assert_eq!(range_for(10_000.0), SearchRange::Mi15);
        assert_eq!(range_for(40_000.0), SearchRange::Mi30);
        assert_eq!(range_for(90_000.0), SearchRange::Mi60);
        assert_eq!(range_for(500_000.0), SearchRange::Mi150);
    }
}

//! What the F-14's panel does to the aircraft and what its displays show:
//!
//! - `radar_controls`: the RIO's WCS buttons, DDD range and antenna keys set the AWG-9
//!   (`Awg9Controls`), the pilot's ACM cover takes it into ACM (the antenna keys then step its
//!   modes); the TID's hook button steps through the tracks (in TWS MAN the hooked one is shot
//!   first); a single target track mode locks the hooked track (leaving it lets go).
//! - `scene`: the picture the displays draw each frame ([`TomcatScene`]): the aircraft, our
//!   Phoenixes and Sparrows in flight, the LANTIRN's data.
//! - `pod_video`: the LANTIRN's picture is up while a TV mode shows it (`MfdState::tomcat_tv`).
//! - `station_lights`: the pilot's station lights (`TomcatStatus::stations`).
//! - `steer_mode`: the DISPLAYS panel's steering mode and the HUD's master mode follow each
//!   other; T.O. on the ground, CRUISE once the gear is up, LDG (with the ILS) when it comes
//!   down in flight.

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use super::{
    DddMode, LtsMode, MissileOption, Power, SteerMode, TidMode, TomcatPanel, TomcatStatus, VdiMode,
    WcsMode,
};
use crate::aircraft::AircraftKind;
use crate::awg9::{Awg9, Awg9Controls, Mode};
use crate::cockpit3d::layout::PanelStyle;
use crate::f14_scopes::{InFlight, Lantirn, Own, TidStab};
use crate::flight::{Aircraft, Player};
use crate::hud::HudMode;
use crate::mfd::{ContactRef, Contacts, MfdState};
use crate::settings::{AppState, Settings};
use crate::world::{WU, WorldPos};

pub(super) fn build(app: &mut App) {
    app.init_resource::<TomcatScene>().init_resource::<Cursors>().add_systems(
        Update,
        (
            radar_controls.before(crate::awg9::scan).after(crate::combat::CombatSystems),
            (scene, pod_video, station_lights).after(crate::awg9::scan),
            steer_mode.before(scene),
        )
            .run_if(in_state(AppState::Flight)),
    );
}

/// Metres per nautical mile.
const NM: f64 = 1852.0;
/// The LANTIRN's laser code.
const LASER_CODE: u16 = 1688;
/// Panel lights' stations (`super::STATIONS`) in the loadout's station order.
const STATION_BAYS: [usize; 8] = [0, 1, 3, 4, 5, 6, 8, 9];

/// What the F-14's displays show this frame (beside the radar's own state, `Awg9`).
#[derive(Resource, Debug, Clone, Default)]
pub struct TomcatScene {
    pub own: Own,
    /// The route's points (metres east, north) and the one steered to.
    pub waypoints: Vec<DVec2>,
    pub steer: Option<usize>,
    pub in_flight: Vec<InFlight>,
    pub lantirn: Lantirn,
    /// The TCS camera's picture: the range of what it follows (nmi), its narrow field of view.
    pub tcs: crate::f14_scopes::Tcs,
    /// The hand controller's cursors.
    pub cursor: Cursors,
    /// The HUD's radar and missile cues.
    pub hud: Option<crate::hud::TomcatHud>,
    /// The aircraft on the data link (the TID's DATA LINK button): the friendly ones and those
    /// the other friendly radars see.
    pub link: Vec<crate::f14_scopes::LinkTrack>,
}

/// The display the RIO's hand controller moves a cursor on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cursor {
    /// The DDD's acquisition cursor (the search modes).
    Ddd,
    /// The TID's (track while scan, a single target track).
    Tid,
}

/// The hand controller's cursors: the DDD's (azimuth in radians, 0..1 up the scope), the TID's
/// (`f14_scopes::tid_spot`'s units), and the one the controller moves now (None: it slews the
/// pod, or nobody sits in the back seat).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Cursors {
    pub ddd: Vec2,
    pub tid: Vec2,
    pub active: Option<Cursor>,
}

impl Default for Cursors {
    fn default() -> Self {
        Self { ddd: Vec2::new(0.0, 0.5), tid: Vec2::new(0.0, 0.3), active: None }
    }
}

/// The cursors' speeds at full deflection: the DDD's across (radians a second) and up (scopes
/// a second), the TID's (tube radii a second).
const CURSOR_AZ: f32 = 0.7;
const CURSOR_UP: f32 = 0.5;
const CURSOR_TID: f32 = 0.7;
/// HOOK takes what is this near the cursor (the DDD's: scopes across and up; the TID's: tube
/// radii).
const HOOK_REACH: f32 = 0.14;

/// The TID's picture as its mode knob turns it.
pub fn tid_stab(panel: &TomcatPanel) -> TidStab {
    match panel.tid_mode {
        TidMode::GroundStab => TidStab::Ground,
        TidMode::Attack => TidStab::Attack,
        _ => TidStab::Aircraft,
    }
}

/// The DDD's picture as its display mode buttons and the Vc switch set it.
pub fn ddd_view(panel: &TomcatPanel) -> crate::f14_scopes::DddView {
    use crate::f14_scopes::DddPicture;
    crate::f14_scopes::DddView {
        picture: match panel.ddd_mode {
            DddMode::PulseVideo => DddPicture::Pulse,
            DddMode::Pd => DddPicture::Pd,
            DddMode::AntTrk => DddPicture::Antenna,
            DddMode::Rdrot => DddPicture::Rate,
            DddMode::Jat => DddPicture::Jam,
            DddMode::Irot => DddPicture::Ir,
        },
        vc_top: crate::f14_scopes::VC_TOP * if panel.vc_2x { 2.0 } else { 1.0 },
        cursor: None,
    }
}

/// The cursor the hand controller moves in the radar's mode: the DDD's in the search modes,
/// the TID's otherwise.
pub fn cursor_for(mode: Mode) -> Cursor {
    if matches!(mode, Mode::PulseSearch | Mode::PdSearch | Mode::Rws) {
        Cursor::Ddd
    } else {
        Cursor::Tid
    }
}

/// What HOOK takes: the return nearest the DDD's cursor, or the track nearest the TID's, within
/// `HOOK_REACH`.
pub fn under_cursor(
    radar: &Awg9,
    controls: &Awg9Controls,
    cursor: &Cursors,
    panel: &TomcatPanel,
    own: &Own,
) -> Option<ContactRef> {
    use crate::f14_scopes::{DddPicture, ddd_spot};
    match cursor.active? {
        Cursor::Ddd => {
            // What the picture shows: the returns, the jammers' strobes (any height on their
            // bearing) or the infrared sensor's spots.
            let view = ddd_view(panel);
            let off = |p: Vec2, across_only: bool| {
                let up = if across_only { 0.0 } else { p.y - cursor.ddd.y };
                Vec2::new((p.x - cursor.ddd.x) / (2.0 * crate::awg9::GIMBAL), up).length()
            };
            let marks: Vec<(f32, ContactRef)> = match view.picture {
                DddPicture::Jam => radar
                    .strobes
                    .iter()
                    .map(|s| (off(Vec2::new(s.az, 0.0), true), s.target))
                    .collect(),
                DddPicture::Ir => radar
                    .ir
                    .iter()
                    .map(|s| {
                        let p = ddd_spot(radar, controls, &view, (s.az, s.el), 0.0, 0.0);
                        (off(p, false), s.target)
                    })
                    .collect(),
                _ => radar
                    .hits
                    .iter()
                    .map(|h| {
                        let p = ddd_spot(radar, controls, &view, (h.az, h.el), h.range, h.closure);
                        (off(p, false), h.target)
                    })
                    .collect(),
            };
            marks
                .into_iter()
                .filter(|(d, _)| *d < HOOK_REACH)
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, t)| t)
        }
        Cursor::Tid => radar
            .tracks
            .iter()
            .map(|t| {
                let p = crate::f14_scopes::tid_spot(
                    tid_stab(panel),
                    own,
                    panel.tid_range.miles() as f32,
                    t.now().truncate(),
                );
                (p.distance(cursor.tid), t.target)
            })
            .filter(|(d, _)| *d < HOOK_REACH)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, t)| t),
    }
}

/// A picture is up for the TV modes: the TCS camera's (the VIDEO switch at TCS), or the
/// LANTIRN's with the pod on.
pub fn video_up(panel: &TomcatPanel) -> bool {
    !panel.lts_lantirn || panel.lts_power == Power::On
}

/// The TCS camera's zoom steps (`trackcam`): its wide and narrow fields of view.
const TCS_ZOOM: (u8, u8) = (3, 5);

/// What the TCS looks at: the single target track, else the hooked track, else the next
/// Phoenix's (metres east, north, up from the aircraft); None: along the nose.
pub fn tcs_target(radar: &Awg9, own: DVec3) -> Option<DVec3> {
    let track = |n: u16| radar.track(n).map(|t| t.now() - own);
    radar
        .stt
        .map(|t| {
            let (sa, ca) = (t.az + radar.own_heading()).sin_cos();
            let flat = t.range as f64 * (t.el.cos() as f64);
            DVec3::new(flat * sa as f64, flat * ca as f64, t.alt as f64 - own.z)
        })
        .or_else(|| radar.hooked.and_then(track))
        .or_else(|| {
            let t = radar.next_shot(&[])?;
            radar.tracks.iter().find(|k| k.target == t).map(|k| k.now() - own)
        })
}

/// The display in front of this seat shows the pod's picture: the RIO's TID in TV, the pilot's
/// VDI in TV.
pub fn tv_shown(panel: &TomcatPanel, style: PanelStyle) -> bool {
    video_up(panel)
        && match style {
            PanelStyle::F14Rio => panel.tid_mode == TidMode::Tv,
            PanelStyle::F14 => panel.vdi_mode == VdiMode::Tv,
            _ => false,
        }
}

/// The ACM modes the pilot's antenna keys step through, low to high: the vertical scan low,
/// the wide scan (where the cover puts it), the boresight, the vertical scan high.
pub const ACM_MODES: [Mode; 4] = [Mode::VslLow, Mode::Mrl, Mode::Plm, Mode::VslHigh];

/// The ACM mode after `acm` with the antenna keys pushed `up` (or down).
pub fn step_acm(acm: Mode, up: bool) -> Mode {
    let i = ACM_MODES.iter().position(|&m| m == acm).unwrap_or(1);
    let j = if up { (i + 1).min(ACM_MODES.len() - 1) } else { i.saturating_sub(1) };
    ACM_MODES[j]
}

/// The radar's mode the WCS buttons select, or with the pilot's ACM cover up the ACM mode
/// `acm`.
pub fn mode(panel: &TomcatPanel, acm: Mode) -> Mode {
    if panel.acm_cover {
        return acm;
    }
    match panel.wcs {
        WcsMode::PdSearch => Mode::PdSearch,
        WcsMode::Rws => Mode::Rws,
        WcsMode::TwsManual => Mode::TwsMan,
        WcsMode::TwsAuto => Mode::TwsAuto,
        WcsMode::PdStt => Mode::PdStt,
        WcsMode::PulseSearch => Mode::PulseSearch,
        WcsMode::PulseStt => Mode::PulseStt,
    }
}

/// The track after `hooked` by number (the first after the last), none without tracks.
pub fn next_hook(radar: &Awg9, hooked: Option<u16>) -> Option<u16> {
    let mut numbers: Vec<u16> = radar.tracks.iter().map(|t| t.number).collect();
    numbers.sort_unstable();
    let after = hooked.and_then(|h| numbers.iter().copied().find(|&n| n > h));
    after.or(numbers.first().copied())
}

/// What a single target track locks: the hooked track, else the nearest track, else the
/// nearest return.
fn stt_target(radar: &Awg9) -> Option<ContactRef> {
    if let Some(t) = radar.hook {
        return Some(t);
    }
    let own = radar.own_position().unwrap_or_default();
    let near = |p: DVec3| p.distance_squared(own);
    radar
        .tracks
        .iter()
        .min_by(|a, b| near(a.now()).total_cmp(&near(b.now())))
        .map(|t| t.target)
        .or_else(|| radar.hits.iter().min_by(|a, b| a.range.total_cmp(&b.range)).map(|h| h.target))
}

/// The Sidewinder's weapon type; how far off the nose (radians) and how far away (metres) its
/// seeker, slaved to the radar, takes a target by itself.
const SIDEWINDER_WEAPON: i32 = 0;
const SEEKER_CONE: f64 = 0.26;
const SEEKER_REACH: f64 = 18_500.0;

/// The aircraft not known as a friend the radar has (a track, else a return) nearest the nose within the
/// Sidewinder seeker's cone and reach.
fn seeker_target(radar: &Awg9, a: &Aircraft, pos: &WorldPos) -> Option<ContactRef> {
    let o = own(a, pos);
    // The nose in east, north, up.
    let (hs, hc) = (o.heading as f64).sin_cos();
    let (ps, pc) = (o.pitch as f64).sin_cos();
    let nose = DVec3::new(hs * pc, hc * pc, ps);
    let tracks = radar.tracks.iter().filter(|t| t.friendly != Some(true)).filter_map(|t| {
        let d = t.now() - o.pos;
        let off = nose.angle_between(d);
        (d.length() < SEEKER_REACH && off < SEEKER_CONE).then_some((off, t.target))
    });
    // A return has no elevation: by its azimuth, with the antenna near level.
    let level = radar.elevation.abs() < 0.2 && o.pitch.abs() < 0.2;
    let hits = radar.hits.iter().filter(|h| h.friendly != Some(true) && level).filter_map(|h| {
        let off = h.az.abs() as f64;
        ((h.range as f64) < SEEKER_REACH && off < SEEKER_CONE).then_some((off, h.target))
    });
    tracks.chain(hits).min_by(|a, b| a.0.total_cmp(&b.0)).map(|t| t.1)
}

/// The single target track the MSL OPTIONS knob takes the radar into as a radar missile is
/// launched (NORM and ACT: none, the radar stays in its mode).
pub fn launch_wcs(option: MissileOption) -> Option<WcsMode> {
    match option {
        MissileOption::PdStt => Some(WcsMode::PdStt),
        MissileOption::PulseStt => Some(WcsMode::PulseStt),
        MissileOption::Norm | MissileOption::ActiveStt => None,
    }
}

/// The target of a Phoenix or Sparrow of ours launched since the last call (`flying`: the
/// slots of those in flight then).
fn new_radar_missile(sim: &stealth_sim::Sim, flying: &mut Vec<usize>) -> Option<ContactRef> {
    let mut now = Vec::new();
    let mut new = None;
    for (slot, m) in sim.projectiles().iter().enumerate().skip(stealth_sim::ENEMY_SLOTS) {
        if m.ttl == 0 || !matches!(m.spec, stealth_sim::PHOENIX_SPEC | stealth_sim::SPARROW_SPEC) {
            continue;
        }
        now.push(slot);
        if !flying.contains(&slot) {
            new = match m.target_pilot {
                Some(k) => Some(ContactRef::Pilot(k)),
                None if m.target >= 0 => Some(ContactRef::Unit(m.target as usize)),
                None => new,
            };
        }
    }
    *flying = now;
    new
}

/// The seconds the IFF's button stays lit after a push.
const IFF_SECS: f32 = 1.0;

#[allow(clippy::too_many_arguments)]
pub(super) fn radar_controls(
    settings: Res<Settings>,
    mut panel: ResMut<TomcatPanel>,
    mfd: Res<MfdState>,
    contacts: Res<Contacts>,
    flight: Res<crate::input::FlightControls>,
    time: Res<Time>,
    player: Query<(&Aircraft, &WorldPos), With<Player>>,
    mut cursor: ResMut<Cursors>,
    mut controls: ResMut<Awg9Controls>,
    mut radar: ResMut<Awg9>,
    (mut hooks, mut iffs): (Local<Option<u8>>, Local<Option<u8>>),
    (mut acm, mut was_stt): (Local<Option<(Mode, f32)>>, Local<bool>),
    (combat, hud): (Option<Res<crate::combat::Combat>>, Res<crate::hud::HudState>),
    (mut locks, mut unlocks): (
        MessageWriter<crate::avionics::LockRequest>,
        MessageWriter<crate::avionics::Unlock>,
    ),
    mut actions: MessageReader<crate::input::ActionEvent>,
    mut flying: Local<Vec<usize>>,
) {
    // The lock control (the targeting pod's and the radar's lock: a key, a HOTAS button, a VR
    // controller's): with a cursor up it hooks, as the HOOK button does.
    let lock = actions.read().filter(|a| a.0 == crate::input::Action::LockCursor).count() > 0;
    if settings.aircraft != AircraftKind::F14 {
        return;
    }
    // ACM: the cover puts the radar in the wide scan, the antenna keys (pushed, not held) step
    // the mode up or down.
    let rate = flight.antenna_rate;
    let (mut sub, before) = acm.unwrap_or((Mode::Mrl, 0.0));
    if !panel.acm_cover {
        sub = Mode::Mrl;
    } else if rate.abs() > 0.5 && before.abs() <= 0.5 {
        sub = step_acm(sub, rate > 0.0);
    }
    *acm = Some((sub, rate));
    // MSL OPTIONS at PD STT or PULSE STT: a radar missile leaving the rail takes the radar into
    // that single target track on the missile's target (hooked for it).
    let launched = combat.as_deref().and_then(|c| new_radar_missile(&c.sim, &mut flying));
    if let (Some(target), Some(wcs)) = (launched, launch_wcs(panel.missile_option))
        && !panel.acm_cover
    {
        panel.hook_target = super::hook_code(Some(target));
        panel.wcs = wcs;
    }
    let mode = mode(&panel, sub);
    let want = Awg9Controls {
        mode,
        // In ACM the antenna keys pick the mode: the elevation stays.
        elevation: if panel.acm_cover { controls.elevation } else { mfd.av.fcr_el.to_radians() },
        range: panel.ddd_range.miles() as f32 * NM as f32,
        ph_active: panel.ph_active || panel.missile_option == MissileOption::ActiveStt,
        ..*controls
    };
    controls.set_if_neq(want);
    // The RIO's hand controller: with the TID in TV it slews the pod (`avionics::update_pod`),
    // else it moves the DDD's cursor (the search modes) or the TID's.
    let rio = settings.gunner_cockpit;
    let active = (rio && !tv_shown(&panel, PanelStyle::F14Rio)).then(|| cursor_for(mode));
    let dt = time.delta_secs();
    let mut k = *cursor;
    k.active = active;
    match active {
        Some(Cursor::Ddd) => {
            k.ddd.x = (k.ddd.x + flight.slew.x * CURSOR_AZ * dt)
                .clamp(-crate::awg9::GIMBAL, crate::awg9::GIMBAL);
            k.ddd.y = (k.ddd.y + flight.slew.y * CURSOR_UP * dt).clamp(0.0, 1.0);
        }
        Some(Cursor::Tid) => k.tid = (k.tid + flight.slew * CURSOR_TID * dt).clamp_length_max(0.95),
        None => {}
    }
    cursor.set_if_neq(k);
    // The hook button (this seat's) or the lock control with a cursor up: what is under the
    // cursor, else the next track.
    if hooks.is_some_and(|h| h != panel.hooks) || (lock && active.is_some()) {
        let own = player.single().ok().map(|(a, pos)| own(a, pos)).unwrap_or_default();
        let target = under_cursor(&radar, &controls, &k, &panel, &own).or_else(|| {
            let next = next_hook(&radar, radar.hooked)?;
            radar.track(next).map(|t| t.target)
        });
        let code = super::hook_code(target);
        if panel.hook_target != code {
            panel.hook_target = code;
        }
        // TWS MAN: the hooked track is the next Phoenix's.
        let number =
            target.and_then(|t| radar.tracks.iter().find(|k| k.target == t)).map(|t| t.number);
        if let (Some(n), Mode::TwsMan) = (number, mode) {
            radar.set_priority(n, Some(1));
        }
    }
    *hooks = Some(panel.hooks);
    // The IFF button pushed (in either seat: the push goes across with the panel, so both
    // seats' radars interrogate what they hold and show the same friends and foes).
    let pushes = panel.iffs & 0xf;
    if iffs.is_some_and(|k| k != pushes) {
        panel.interrogating = IFF_SECS;
        radar.interrogate();
    } else if panel.interrogating > 0.0 {
        panel.interrogating = (panel.interrogating - dt).max(0.0);
    }
    *iffs = Some(pushes);
    // The crew's hooked aircraft on this radar: its track, if it has one.
    let hook = super::hook_ref(panel.hook_target);
    let hooked = hook.and_then(|t| radar.tracks.iter().find(|k| k.target == t)).map(|t| t.number);
    if radar.hook != hook || radar.hooked != hooked {
        radar.hook = hook;
        radar.hooked = hooked;
    }
    // A single target track locks; going back to a search lets go.
    let air = contacts.locked.filter(|l| matches!(l, ContactRef::Unit(_) | ContactRef::Pilot(_)));
    // The lock control without a cursor (the pilot's): the hooked aircraft, the track the next
    // shot is for, else the nearest. A Sidewinder's seeker is slaved to the radar: it takes
    // the hostile aircraft the radar has nearest the nose by itself.
    if air.is_none() {
        let sidewinder = combat.as_deref().is_some_and(|c| {
            let (bays, cur) = c.sim.bays();
            !c.sim.gun_selected()
                && hud.mode == crate::hud::HudMode::Air
                && bays.get(cur).is_some_and(|b| b.1 > 0 && b.0 == SIDEWINDER_WEAPON)
        });
        let wanted = if lock && active.is_none() {
            radar.hook.or_else(|| radar.next_shot(&[])).or_else(|| stt_target(&radar))
        } else if sidewinder {
            player.single().ok().and_then(|(a, pos)| seeker_target(&radar, a, pos))
        } else {
            None
        };
        if let Some(t) = wanted {
            locks.write(crate::avionics::LockRequest(t));
        }
    }
    if mode.stt() {
        if air.is_none()
            && let Some(t) = stt_target(&radar)
        {
            locks.write(crate::avionics::LockRequest(t));
        }
    } else if *was_stt && !mode.acm() && air.is_some() {
        unlocks.write(crate::avionics::Unlock);
    }
    *was_stt = mode.stt();
}

/// The game's HUD mode for a steering mode: A/A and A/G theirs, the others NAV.
pub fn hud_mode_for(steer: SteerMode) -> HudMode {
    match steer {
        SteerMode::AirToAir => HudMode::Air,
        SteerMode::AirToGround => HudMode::Gnd,
        SteerMode::TakeOff | SteerMode::Cruise | SteerMode::Landing => HudMode::Nav,
    }
}

/// The steering mode after the HUD's mode was changed to `mode` (its keys) from `steer`: A/A
/// and A/G theirs; NAV keeps T.O., CRUISE and LDG, else CRUISE.
pub fn steer_for(mode: HudMode, steer: SteerMode) -> SteerMode {
    match mode {
        HudMode::Air => SteerMode::AirToAir,
        HudMode::Gnd => SteerMode::AirToGround,
        HudMode::Nav if hud_mode_for(steer) == HudMode::Nav => steer,
        HudMode::Nav => SteerMode::Cruise,
    }
}

/// What the gear does to the steering mode in the front seat: T.O. on the ground, CRUISE
/// from T.O. once the gear is up in the air, LDG from CRUISE when it comes down in flight.
pub fn steer_by_gear(
    steer: SteerMode,
    gear_down: bool,
    was_down: bool,
    on_ground: bool,
) -> SteerMode {
    match steer {
        SteerMode::TakeOff if !gear_down && !on_ground => SteerMode::Cruise,
        SteerMode::Cruise if gear_down && !was_down && !on_ground => SteerMode::Landing,
        s => s,
    }
}

/// The DISPLAYS panel's steering buttons and the HUD's mode keys: one follows the other (a
/// button sets the HUD's master mode, a mode key the button); LDG brings the ILS up and
/// leaving it takes it away. The pilot starts on T.O. on the ground, the gear moves him on.
fn steer_mode(
    settings: Res<Settings>,
    mut panel: ResMut<TomcatPanel>,
    mut hud: ResMut<crate::hud::HudState>,
    mut mfd: ResMut<MfdState>,
    player: Query<&Aircraft, With<Player>>,
    mut before: Local<Option<(SteerMode, HudMode, bool)>>,
) {
    if settings.aircraft != AircraftKind::F14 {
        *before = None;
        return;
    }
    let Ok(a) = player.single() else { return };
    let pilot = !settings.gunner_cockpit;
    let Some((steer, mode, gear)) = *before else {
        // A new flight: T.O. on the ground, else the HUD's mode.
        let start =
            if a.on_ground { SteerMode::TakeOff } else { steer_for(hud.mode, SteerMode::Cruise) };
        if pilot && panel.steer != start {
            panel.steer = start;
        }
        *before = Some((panel.steer, hud.mode, a.gear_down));
        return;
    };
    if panel.steer != steer {
        // A steering button (here or the other seat's panel).
        let want = hud_mode_for(panel.steer);
        if hud.mode != want {
            hud.mode = want;
        }
    } else if hud.mode != mode && pilot {
        // (The buttons are the pilot's: what changes the back seat's HUD mode, a lock there,
        // leaves them.)
        let s = steer_for(hud.mode, panel.steer);
        if panel.steer != s {
            panel.steer = s;
        }
    } else if pilot {
        let s = steer_by_gear(panel.steer, a.gear_down, gear, a.on_ground);
        if panel.steer != s {
            panel.steer = s;
        }
    }
    if panel.steer != steer {
        let landing = panel.steer == SteerMode::Landing;
        if steer == SteerMode::Landing || landing {
            mfd.ils = landing;
        }
    }
    *before = Some((panel.steer, hud.mode, a.gear_down));
}

/// The HUD's cues from the radar `radar` of the aircraft at `own` (metres east, north, up) with
/// the missiles `flying`.
pub fn hud_cues(
    radar: &Awg9,
    own: DVec3,
    flying: &[InFlight],
    clock: f32,
) -> crate::hud::TomcatHud {
    use crate::awg9::{PLM_HALF, VSL_HIGH, VSL_LOW};
    use crate::hud::AcmScan;
    let strip = 2.0 * crate::aa_radar::BEAM_HALF;
    let acm = match radar.mode {
        Mode::Plm => Some(AcmScan::Boresight(PLM_HALF)),
        Mode::VslHigh => Some(AcmScan::Vertical(strip, VSL_HIGH.0, VSL_HIGH.1)),
        Mode::VslLow => Some(AcmScan::Vertical(strip, VSL_LOW.0, VSL_LOW.1)),
        Mode::Mrl => Some(AcmScan::Wide),
        _ => None,
    };
    let targets: Vec<ContactRef> = flying.iter().map(|m| m.target).collect();
    let next = radar.next_shot(&targets).and_then(|t| {
        let track = radar.tracks.iter().find(|k| k.target == t)?;
        let d = track.now() - own;
        // East, north, up to Bevy's world (x east, y up, z south).
        let dir = Vec3::new(d.x as f32, d.z as f32, -d.y as f32);
        Some((dir, (d.length() / 1000.0) as f32, track.number))
    });
    crate::hud::TomcatHud {
        mode: radar.mode.label(),
        acm: acm.filter(|_| radar.stt.is_none()),
        next,
        closure: radar.stt.map(|t| t.closure * 1.943_844),
        in_flight: flying.iter().map(|m| m.to_go).collect(),
        clock,
        target_alt: radar
            .stt
            .map(|t| own.z + t.range as f64 * (t.el as f64).sin())
            .or_else(|| {
                let t = radar.next_shot(&targets)?;
                radar.tracks.iter().find(|k| k.target == t).map(|k| k.now().z)
            })
            .map(|z| (z * 3.280_84 / 1000.0) as f32),
        steer: SteerMode::Cruise,
        glide_deg: crate::scenery::GLIDE_PATH_DEG as f32,
        declutter: false,
    }
}

/// The aircraft's position (metres east, north, up), attitude and speed.
pub fn own(a: &Aircraft, pos: &WorldPos) -> Own {
    let (heading, pitch, roll) = a.euler();
    Own { pos: DVec3::new(pos.0.x, -pos.0.z, pos.0.y), heading, pitch, roll, knots: a.knots() }
}

#[allow(clippy::too_many_arguments)]
fn scene(
    settings: Res<Settings>,
    panel: Res<TomcatPanel>,
    mfd: Res<MfdState>,
    waypoints: Res<crate::hud::Waypoints>,
    clock: Res<crate::mfd::MissionClock>,
    mission: Option<Res<crate::mission::ActiveMission>>,
    combat: Option<Res<crate::combat::Combat>>,
    radar: Res<Awg9>,
    player: Query<(&Aircraft, &WorldPos), With<Player>>,
    time: Res<Time>,
    cursor: Res<Cursors>,
    (contacts, campaign): (Res<Contacts>, Option<Res<crate::campaign::ActiveCampaign>>),
    mut out: ResMut<TomcatScene>,
    mut closing: Local<Vec<(usize, f64, f64)>>,
) {
    if settings.aircraft != AircraftKind::F14 {
        return;
    }
    let Ok((a, pos)) = player.single() else { return };
    let o = own(a, pos);
    out.own = o;
    out.cursor = *cursor;
    out.waypoints =
        waypoints.points.iter().map(|w| DVec2::new(w.east * WU, w.north * WU)).collect();
    out.steer = (!waypoints.points.is_empty())
        .then_some(waypoints.current.min(waypoints.points.len().saturating_sub(1)));
    // Our radar missiles in flight: seconds to their seekers coming on (negative), then to
    // impact, by the rate they close on their targets.
    let dt = time.delta_secs_f64().max(1e-3);
    let mut flying = Vec::new();
    let mut rates = Vec::new();
    if let Some(combat) = combat.as_deref() {
        let sim = &combat.sim;
        for (slot, m) in sim.projectiles().iter().enumerate().skip(stealth_sim::ENEMY_SLOTS) {
            if m.ttl == 0
                || !matches!(m.spec, stealth_sim::PHOENIX_SPEC | stealth_sim::SPARROW_SPEC)
            {
                continue;
            }
            let (target, at) = match m.target_pilot {
                Some(k) => (ContactRef::Pilot(k), (m.tgt_x, m.tgt_y)),
                None if m.target >= 0 => {
                    let u = sim.flights().get(m.target as usize);
                    (
                        ContactRef::Unit(m.target as usize),
                        u.map_or((m.tgt_x, m.tgt_y), |u| (u.x, u.y)),
                    )
                }
                None => continue,
            };
            let range = ((at.0 - m.x) as f64).hypot((at.1 - m.y) as f64);
            let rate = match closing.iter().find(|c| c.0 == slot) {
                Some(&(_, before, rate)) => {
                    rate + ((before - range) / dt - rate) * (dt / 1.0).min(1.0)
                }
                None => 0.0,
            };
            rates.push((slot, range, rate));
            let secs = |d: f64| if rate > 1.0 { (d / rate) as f32 } else { 99.0 };
            let to_go = if m.midcourse.is_some() {
                -secs((range - stealth_sim::PHOENIX_ACTIVE as f64).max(0.0))
            } else {
                secs(range)
            };
            flying.push(InFlight { target, to_go, tti: secs(range) });
        }
    }
    *closing = rates;
    // The data link: the friendly aircraft, and the others where another friendly radar sees
    // them (a jammer: where one burns through, as the contacts have it).
    out.link.clear();
    if panel.tid_option(super::TidOption::DataLink)
        && let Some(combat) = combat.as_deref()
    {
        let sim = &combat.sim;
        let eyes =
            crate::campaign::picture::friendly_eyes(campaign.as_ref().map(|c| c.state()), sim);
        for k in contacts.list.iter().filter(|k| {
            matches!(k.kind, crate::mfd::ContactKind::Aircraft | crate::mfd::ContactKind::Drone)
        }) {
            let Some(target) = k.target else { continue };
            let reported = k.friendly
                || match target {
                    ContactRef::Unit(i) => sim.flights().get(i).is_some_and(|u| {
                        k.seen && crate::campaign::picture::seen_by(&eyes, u.x, u.y)
                    }),
                    _ => false,
                };
            if reported {
                out.link.push(crate::f14_scopes::LinkTrack {
                    target,
                    pos: DVec3::new(k.east * WU, k.north * WU, k.alt as f64 * WU),
                    heading: (k.heading as f64 / 65536.0 * std::f64::consts::TAU) as f32,
                    friendly: k.friendly,
                });
            }
        }
    }
    let now = clock.start_seconds + clock.elapsed.unwrap_or(0.0) * clock.rate;
    out.hud = Some(crate::hud::TomcatHud {
        steer: panel.steer,
        declutter: panel.hud_declutter,
        glide_deg: match panel.hud_awl {
            super::Awl::Ils => crate::scenery::GLIDE_PATH_DEG as f32,
            super::Awl::Acl => stealth_sim::catapult::GLIDE_DEG as f32,
        },
        ..hud_cues(&radar, o.pos, &flying, now as f32)
    });
    out.tcs = crate::f14_scopes::Tcs {
        range_nm: tcs_target(&radar, o.pos).map(|d| (d.length() / NM) as f32),
        narrow: panel.narrow,
        heading: o.heading.to_degrees(),
    };
    out.in_flight = flying;
    // The LANTIRN's data.
    let tgp = &mfd.av.tgp;
    let mut l = Lantirn {
        alt_ft: (o.pos.z * 3.280_84) as f32,
        ground_kt: o.knots * o.pitch.cos(),
        dive: -o.pitch.to_degrees(),
        heading: o.heading.to_degrees(),
        clock: clock.start_seconds + clock.elapsed.unwrap_or(0.0) * clock.rate,
        laser_code: LASER_CODE,
        lasing: panel.laser_armed && panel.lts_oper && tgp.los.is_some(),
        white_hot: !panel.black_hot,
        flir: mfd.flir,
        wide: !panel.narrow,
        air: panel.lts_mode == LtsMode::AirToAir,
        track: tgp.mode.label().to_string(),
        az: 0.0,
        el: -10.0,
        ..default()
    };
    if let Some(los) = tgp.los {
        let local = a.orientation.inverse() * los;
        l.az = local.x.atan2(-local.z).to_degrees();
        l.el = local.y.atan2(local.x.hypot(local.z)).to_degrees();
    }
    let at = tgp.los_ground.map(|g| DVec3::new(g.x, -g.z, g.y)).unwrap_or(o.pos);
    if let Some(g) = tgp.los_ground {
        l.range_nm = (g.distance(pos.0) / NM) as f32;
    }
    let theater = mission.as_deref().map(|m| m.0.theater_code.to_ascii_lowercase());
    let (lon0, lat0) = crate::world::geo_origin(theater.as_deref().unwrap_or(""));
    l.lat = lat0 + at.y / 1000.0 / 111.2;
    l.lon = lon0 + at.x / 1000.0 / (111.2 * l.lat.to_radians().cos());
    out.lantirn = l;
}

/// The pod's picture is rendered while a display in this cockpit shows it (TV or FLIR, the
/// FLIR key; black and white either way: the cockpit's `trackcam` screens are grey). The FLIR
/// picture is white hot or black hot by the POLARITY switch, which also brings it up.
fn pod_video(
    settings: Res<Settings>,
    panel: Res<TomcatPanel>,
    radar: Res<Awg9>,
    player: Query<(&Aircraft, &WorldPos), With<Player>>,
    mut mfd: ResMut<MfdState>,
    mut polarity: Local<Option<bool>>,
) {
    let tomcat = settings.aircraft == AircraftKind::F14;
    // The POLARITY switch is the thermal picture's: moving it (in either seat) brings the
    // pod's FLIR picture up if its television picture was.
    if tomcat && polarity.is_some_and(|p| p != panel.black_hot) && !mfd.flir {
        mfd.flir = true;
    }
    *polarity = tomcat.then_some(panel.black_hot);
    let tv = tomcat.then(|| tv_shown(&panel, settings.cockpit().0));
    if mfd.tomcat_tv != tv {
        mfd.tomcat_tv = tv;
    }
    // The TCS (the VIDEO switch at TCS): along the radar's track, else the nose.
    let tcs =
        player.single().ok().filter(|_| tv == Some(true) && !panel.lts_lantirn).map(|(a, pos)| {
            let own = DVec3::new(pos.0.x, -pos.0.z, pos.0.y);
            let dir = tcs_target(&radar, own)
                .map(|d| Vec3::new(d.x as f32, d.z as f32, -d.y as f32).normalize_or_zero())
                .filter(|d| *d != Vec3::ZERO)
                .unwrap_or(a.orientation * Vec3::NEG_Z);
            let zoom = if panel.narrow { TCS_ZOOM.1 } else { TCS_ZOOM.0 };
            (crate::aircraft::F14_TCS, dir, zoom)
        });
    if mfd.tcs != tcs {
        mfd.tcs = tcs;
    }
    let black_hot = tomcat && panel.black_hot && panel.lts_lantirn;
    if mfd.black_hot != black_hot {
        mfd.black_hot = black_hot;
    }
}

/// The station lights: the stations holding the selected weapon.
fn station_lights(
    settings: Res<Settings>,
    combat: Option<Res<crate::combat::Combat>>,
    mut status: ResMut<TomcatStatus>,
) {
    if settings.aircraft != AircraftKind::F14 {
        return;
    }
    let lit = combat.as_deref().map_or(0, |c| {
        let (bays, cur) = c.sim.bays();
        let selected = bays.get(cur).map_or(-1, |b| b.0);
        STATION_BAYS.iter().enumerate().fold(0u8, |acc, (n, &b)| {
            let ready = selected >= 0 && bays.get(b).is_some_and(|s| s.0 == selected && s.1 > 0);
            acc | ((ready as u8) << n)
        })
    });
    if status.stations != lit {
        status.stations = lit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The steering buttons set the HUD's mode and its keys the buttons; the gear moves T.O. on
    /// to CRUISE in the air and CRUISE to LDG when it comes down there.
    #[test]
    fn steering_modes_follow_the_hud_and_the_gear() {
        use SteerMode::*;
        assert_eq!(hud_mode_for(AirToAir), HudMode::Air);
        assert_eq!(hud_mode_for(AirToGround), HudMode::Gnd);
        assert!([TakeOff, Cruise, Landing].iter().all(|&s| hud_mode_for(s) == HudMode::Nav));
        assert_eq!(steer_for(HudMode::Air, Landing), AirToAir);
        assert_eq!(steer_for(HudMode::Gnd, Cruise), AirToGround);
        assert_eq!(steer_for(HudMode::Nav, AirToAir), Cruise);
        assert_eq!(steer_for(HudMode::Nav, Landing), Landing);
        assert_eq!(steer_for(HudMode::Nav, TakeOff), TakeOff);
        // Rolling with the gear down: still T.O.; airborne with it up: CRUISE.
        assert_eq!(steer_by_gear(TakeOff, true, true, true), TakeOff);
        assert_eq!(steer_by_gear(TakeOff, true, true, false), TakeOff);
        assert_eq!(steer_by_gear(TakeOff, false, true, false), Cruise);
        // The gear lowered in flight: LDG; already down (or on the ground): no change.
        assert_eq!(steer_by_gear(Cruise, true, false, false), Landing);
        assert_eq!(steer_by_gear(Cruise, true, true, false), Cruise);
        assert_eq!(steer_by_gear(Cruise, true, false, true), Cruise);
        // A/A stays A/A whatever the gear does.
        assert_eq!(steer_by_gear(AirToAir, true, false, false), AirToAir);
    }

    #[test]
    fn the_hook_steps_through_the_tracks() {
        let mut radar = Awg9::default();
        assert_eq!(next_hook(&radar, None), None);
        let track = |number| crate::awg9::TrackFile {
            number,
            target: ContactRef::Unit(number as usize),
            pos: DVec3::ZERO,
            vel: DVec3::ZERO,
            friendly: None,
            unseen: 0.0,
            age: 0.0,
            priority: None,
        };
        radar.tracks = vec![track(7), track(3), track(5)];
        assert_eq!(next_hook(&radar, None), Some(3));
        assert_eq!(next_hook(&radar, Some(3)), Some(5));
        assert_eq!(next_hook(&radar, Some(7)), Some(3));
    }

    #[test]
    fn the_wcs_buttons_set_the_mode() {
        let mut p = TomcatPanel::default();
        p.wcs = WcsMode::TwsAuto;
        assert_eq!(mode(&p, Mode::Plm), Mode::TwsAuto);
        p.acm_cover = true;
        assert_eq!(mode(&p, Mode::Plm), Mode::Plm);
        p.acm_cover = false;
        p.wcs = WcsMode::PulseStt;
        assert!(mode(&p, Mode::Mrl).stt() && mode(&p, Mode::Mrl).pulse());
    }

    #[test]
    fn the_missile_options_pick_the_launch_track() {
        assert_eq!(launch_wcs(MissileOption::Norm), None);
        assert_eq!(launch_wcs(MissileOption::ActiveStt), None);
        assert_eq!(launch_wcs(MissileOption::PdStt), Some(WcsMode::PdStt));
        assert_eq!(launch_wcs(MissileOption::PulseStt), Some(WcsMode::PulseStt));
    }

    /// The display mode buttons each give the DDD its picture; the Vc switch doubles the
    /// closure scale.
    #[test]
    fn the_ddd_buttons_pick_its_picture() {
        use crate::f14_scopes::DddPicture;
        let mut p = TomcatPanel::default();
        let mut seen = Vec::new();
        for m in super::super::DDD_MODES {
            p.press(super::super::Key::DddMode(m));
            seen.push(ddd_view(&p).picture);
        }
        assert_eq!(seen.len(), 6);
        assert!(seen.iter().enumerate().all(|(i, a)| seen[..i].iter().all(|b| a != b)));
        assert_eq!(seen[1], DddPicture::Pd);
        let before = ddd_view(&p).vc_top;
        p.press(super::super::Key::VcScale);
        assert_eq!(ddd_view(&p).vc_top, 2.0 * before);
    }

    #[test]
    fn the_antenna_keys_step_the_acm_modes() {
        assert_eq!(step_acm(Mode::Mrl, true), Mode::Plm);
        assert_eq!(step_acm(Mode::Plm, true), Mode::VslHigh);
        assert_eq!(step_acm(Mode::VslHigh, true), Mode::VslHigh);
        assert_eq!(step_acm(Mode::Mrl, false), Mode::VslLow);
        assert_eq!(step_acm(Mode::VslLow, false), Mode::VslLow);
    }

    #[test]
    fn tv_shows_the_tcs_or_the_powered_pod() {
        let mut p = TomcatPanel::default();
        p.tid_mode = TidMode::Tv;
        // The TCS camera's picture needs no pod.
        assert!(tv_shown(&p, PanelStyle::F14Rio));
        // The LANTIRN's needs the pod on.
        p.lts_lantirn = true;
        assert!(!tv_shown(&p, PanelStyle::F14Rio));
        p.lts_power = Power::On;
        assert!(tv_shown(&p, PanelStyle::F14Rio));
        assert!(!tv_shown(&p, PanelStyle::F14));
        p.vdi_mode = VdiMode::Tv;
        assert!(tv_shown(&p, PanelStyle::F14));
    }

    /// The hand controller's cursor: the DDD's in the search modes, the TID's in track while
    /// scan; HOOK takes the return or the track under it and nothing far from it.
    #[test]
    fn hook_takes_what_is_under_the_cursor() {
        assert_eq!(cursor_for(Mode::Rws), Cursor::Ddd);
        assert_eq!(cursor_for(Mode::PdSearch), Cursor::Ddd);
        assert_eq!(cursor_for(Mode::TwsAuto), Cursor::Tid);
        assert_eq!(cursor_for(Mode::PdStt), Cursor::Tid);
        let panel = TomcatPanel::default();
        let controls = Awg9Controls::default();
        let own = Own::default();
        let mut radar = Awg9::default();
        radar.mode = Mode::Rws;
        let hit = |i: usize, az: f32, range: f32| crate::awg9::Hit {
            az,
            el: 0.0,
            range,
            closure: 0.0,
            target: ContactRef::Unit(i),
            friendly: None,
            age: 0.0,
        };
        radar.hits = vec![hit(1, -0.3, controls.range * 0.2), hit(2, 0.2, controls.range * 0.6)];
        let mut k = Cursors { ddd: Vec2::new(0.22, 0.58), active: Some(Cursor::Ddd), ..default() };
        assert_eq!(under_cursor(&radar, &controls, &k, &panel, &own), Some(ContactRef::Unit(2)));
        k.ddd = Vec2::new(0.9, 0.95);
        assert_eq!(under_cursor(&radar, &controls, &k, &panel, &own), None);
        // The TID (aircraft stabilised, 100 nmi): a track 50 nmi ahead is half way up the tube.
        radar.tracks = vec![crate::awg9::TrackFile {
            number: 1,
            target: ContactRef::Unit(9),
            pos: DVec3::new(0.0, 50.0 * NM, 8000.0),
            vel: DVec3::ZERO,
            friendly: None,
            unseen: 0.0,
            age: 0.0,
            priority: None,
        }];
        let k = Cursors { tid: Vec2::new(0.02, 0.5), active: Some(Cursor::Tid), ..default() };
        assert_eq!(under_cursor(&radar, &controls, &k, &panel, &own), Some(ContactRef::Unit(9)));
        let off = Cursors { active: None, ..k };
        assert_eq!(under_cursor(&radar, &controls, &off, &panel, &own), None);
    }

    /// The TCS follows the hooked track, else the next shot's; nothing tracked, the nose.
    #[test]
    fn the_tcs_follows_the_radars_track() {
        let mut radar = Awg9::default();
        assert_eq!(tcs_target(&radar, DVec3::ZERO), None);
        radar.tracks = vec![crate::awg9::TrackFile {
            number: 4,
            target: ContactRef::Unit(4),
            pos: DVec3::new(1000.0, 20_000.0, 500.0),
            vel: DVec3::ZERO,
            friendly: None,
            unseen: 0.0,
            age: 0.0,
            priority: None,
        }];
        radar.hooked = Some(4);
        assert_eq!(tcs_target(&radar, DVec3::ZERO), Some(DVec3::new(1000.0, 20_000.0, 500.0)));
    }
}

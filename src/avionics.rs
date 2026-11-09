//! Modern avionics (on unless `--classic-avionics`): F-16 style MFD
//! formats on either MFD instead of the classic fixed pages.
//!
//! - Formats: HSD (horizontal situation display: route, threat rings, air and ground contacts),
//!   HARM (the radars painting the aircraft, placed by danger like the RWR), TGP (the tracking
//!   camera), MAP (the moving map in muted colours, or the waypoint table) and SMS (stores: the
//!   inventory by station with the bomb release program, ripple count and spacing, and the
//!   ORD page); on the F-117A's third MFD also HSI (`draw_hsi_page`). From the format menu
//!   also RCS (`draw_rcs`): the multipurpose display's threat picture (`crate::mpd`) full size,
//!   with the signature's aspect, the configuration that swells it and a longer strip chart. The bottom bezel
//!   buttons pick the format; picking one another MFD shows swaps the two. The top buttons are the format's own, the top right one makes the
//!   format the sensor of interest (SOI). The buttons' functions are drawn on the display next
//!   to them, like the real ones.
//! - Master modes: NAV, A-A and A-G each keep the formats (and the SOI) the pilot left them
//!   with. The first time NAV has the HSD on the left MFD; with a radar A-A the FCR and A-G
//!   the ground map, as the SOI.
//! - SOI: HSD, HARM and the targeting pod can be the sensor of interest: the "sensor/cursor
//!   slew" controls move its cursor, "lock cursor target" locks what is under it (an aircraft or
//!   a ground object on the HSD, a radar on the HARM display).
//! - Targeting pod (TGP): slaved to the locked target; a target key or a display lock slaves it
//!   again, CZ (cursor zero) turns it to the current steerpoint (point track). As the SOI, slewing moves the pod itself, stabilized against the aircraft's motion (area track);
//!   once the slew stops it holds the ground point it looks at (point track), and "lock cursor
//!   target" locks what is there. SP (snowplow) drops the locks and fixes it ahead of the nose.
//!   The HSD shows where the pod looks.

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use crate::canvas::Canvas;
use crate::cockpit::CockpitArt;
use crate::flight::{Aircraft, Player};
use crate::font::{Align, Fonts, stroke_text, stroke_text_at, upper};
use crate::hud::{HudMode, HudState};
use crate::input::{Action, ActionEvent, FlightControls};
use crate::mfd::{
    ContactKind, ContactRef, Contacts, HIRES, MapRaster, MfdCmd, MfdParams, MfdSide, MfdState,
    OsbFn,
};
use crate::rwr::{EmitterView, Emitters};
use crate::world::{WU, WorldPos, bevy_to_wu};

pub struct AvionicsPlugin;

impl Plugin for AvionicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LockRequest>()
            .add_message::<Unlock>()
            .add_message::<PodDesignate>()
            .add_message::<PodTrack>()
            .add_systems(
                Update,
                slew_and_lock
                    .after(crate::input::InputSystemsSet)
                    .after(crate::flight::FlightSystems)
                    .before(crate::combat::CombatSystems)
                    .run_if(in_state(crate::settings::AppState::Flight)),
            )
            .add_systems(
                Update,
                update_pod
                    .after(crate::combat::CombatSystems)
                    .before(crate::trackcam::update_camera)
                    .run_if(in_state(crate::settings::AppState::Flight)),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Format {
    #[default]
    Hsd,
    Harm,
    Tgp,
    Map,
    Sms,
    /// Horizontal situation indicator: compass rose, course and bearing to the steerpoint.
    Hsi,
    /// The air-to-ground radar's ground map (`crate::ground_radar`), on the aircraft with one.
    Gm,
    /// The same radar's air-to-air search and track (`crate::aa_radar`) on a B-scope.
    Fcr,
    /// The engines, fuel and stores at a glance (after the F-22A's).
    Eng,
    /// Our radar signature among the radars painting us (`draw_rcs`).
    Rcs,
    /// The cargo jet's hold and ramp, and the airdrop (`crate::cargo_page`).
    Cgo,
}

/// The formats on the bottom bezel buttons, left to right.
pub const FORMATS: [Format; 5] = [Format::Hsd, Format::Harm, Format::Tgp, Format::Map, Format::Sms];
/// The third MFD's: the HSI instead of the targeting pod (which belongs on the centre display).
pub const THIRD_FORMATS: [Format; 5] =
    [Format::Hsi, Format::Hsd, Format::Harm, Format::Map, Format::Sms];

/// The left MFD's first row with a radar: its air-to-air and ground map formats instead of the
/// HARM display and the stores (the right MFD keeps those); the pilot can move any of them
/// anywhere with the format menu.
pub const RADAR_FORMATS: [Format; 5] =
    [Format::Hsd, Format::Fcr, Format::Gm, Format::Tgp, Format::Map];

/// The fourth MFD's (the F-22A's, between the knees): the engines first.
pub const FOURTH_FORMATS: [Format; 5] =
    [Format::Eng, Format::Hsi, Format::Hsd, Format::Map, Format::Sms];

/// The cargo jet's rows: the cargo page instead of the HARM display, the centre MFD's first.
pub const CARGO_FORMATS: [Format; 5] =
    [Format::Hsd, Format::Cgo, Format::Tgp, Format::Map, Format::Sms];
pub const CARGO_RIGHT_FORMATS: [Format; 5] =
    [Format::Hsi, Format::Hsd, Format::Cgo, Format::Map, Format::Sms];
pub const CARGO_THIRD_FORMATS: [Format; 5] =
    [Format::Cgo, Format::Hsi, Format::Hsd, Format::Map, Format::Sms];
pub const CARGO_FOURTH_FORMATS: [Format; 5] =
    [Format::Eng, Format::Cgo, Format::Hsi, Format::Hsd, Format::Map];

/// Every format, in the order the format menu lists them down its sides (the radar's only with
/// the radar, the cargo page only on the cargo jet): an aircraft has ten at most.
pub const ALL_FORMATS: [Format; 11] = [
    Format::Hsd,
    Format::Harm,
    Format::Tgp,
    Format::Map,
    Format::Sms,
    Format::Hsi,
    Format::Gm,
    Format::Fcr,
    Format::Eng,
    Format::Rcs,
    Format::Cgo,
];

/// The formats an MFD's bottom bezel buttons start with (`radar`: the aircraft has a ground map
/// radar; `cargo`: it is the cargo jet).
pub fn formats(side: MfdSide, radar: bool, cargo: bool) -> [Format; 5] {
    match side {
        MfdSide::Third if cargo => CARGO_THIRD_FORMATS,
        MfdSide::Fourth if cargo => CARGO_FOURTH_FORMATS,
        MfdSide::Right if cargo => CARGO_RIGHT_FORMATS,
        _ if cargo => CARGO_FORMATS,
        MfdSide::Third => THIRD_FORMATS,
        MfdSide::Fourth => FOURTH_FORMATS,
        MfdSide::Left if radar => RADAR_FORMATS,
        _ => FORMATS,
    }
}

impl Format {
    pub fn label(self) -> &'static str {
        match self {
            Format::Hsd => "HSD",
            Format::Harm => "HARM",
            Format::Tgp => "TGP",
            Format::Map => "MAP",
            Format::Sms => "SMS",
            Format::Hsi => "HSI",
            Format::Gm => "GM",
            Format::Fcr => "FCR",
            Format::Eng => "ENG",
            Format::Rcs => "RCS",
            Format::Cgo => "CGO",
        }
    }

    /// The radar's formats, on the aircraft with one only.
    pub fn radar(self) -> bool {
        matches!(self, Format::Gm | Format::Fcr)
    }

    /// Formats with a cursor (or a pod) the slew controls move.
    pub fn soi_capable(self) -> bool {
        matches!(self, Format::Hsd | Format::Harm | Format::Tgp | Format::Gm | Format::Fcr)
    }

    /// The key that brings the format up (script `press=` lookups).
    pub fn action(self) -> Action {
        match self {
            Format::Hsd
            | Format::Harm
            | Format::Map
            | Format::Gm
            | Format::Fcr
            | Format::Eng
            | Format::Rcs
            | Format::Cgo => Action::Maps,
            Format::Tgp => Action::CamAhead,
            Format::Sms => Action::Ordnance,
            Format::Hsi => Action::SelectWaypoint,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HarmFilter {
    #[default]
    All,
    Sam,
    Air,
}

/// HSD ranges (km from the aircraft to the outer ring).
pub const HSD_RANGES: [f32; 6] = [10.0, 20.0, 40.0, 80.0, 160.0, 320.0];
const HSD_RANGE_LABELS: [&str; 6] = ["10", "20", "40", "80", "160", "320"];
/// Ground map ranges (km from the aircraft to the scope's outer arc).
pub const GM_RANGES: [f32; 5] = [5.0, 10.0, 20.0, 40.0, 80.0];
const GM_RANGE_LABELS: [&str; 5] = ["5", "10", "20", "40", "80"];
/// The A-A radar's ranges (nautical miles to the top of the B-scope).
pub const FCR_RANGES_NM: [f32; 5] = [10.0, 20.0, 40.0, 80.0, 160.0];
const FCR_RANGE_LABELS: [&str; 5] = ["10", "20", "40", "80", "160"];
/// The RCS page's ranges (km to the scope's rim; 0: the shortest of `mpd::RANGES` that holds
/// the farthest radar, as the multipurpose display's).
pub const RCS_RANGES: [f32; 6] = [0.0, 5.0, 10.0, 20.0, 40.0, 80.0];
const RCS_RANGE_LABELS: [&str; 6] = ["AUTO", "5", "10", "20", "40", "80"];
/// The A-A radar's scan widths (A6, A3, A1: degrees either side of the nose) and bars (rows of
/// the scan, stacked `aa_radar::BAR_SPACING` apart).
pub const FCR_AZ_DEG: [f32; 3] = [60.0, 30.0, 10.0];
const FCR_AZ_LABELS: [&str; 3] = ["A6", "A3", "A1"];
pub const FCR_BARS: [u8; 3] = [4, 2, 1];
const FCR_BAR_LABELS: [&str; 3] = ["4B", "2B", "1B"];
/// The A-A radar's antenna elevation over the horizon: steps of `FCR_EL_STEP` degrees, at most
/// `FCR_EL_MAX` steps up or down.
pub const FCR_EL_STEP: f32 = 2.0;
const FCR_EL_MAX: i8 = 10;
const FCR_EL_LABELS: [&str; 21] = [
    "-20", "-18", "-16", "-14", "-12", "-10", "-8", "-6", "-4", "-2", "0", "+2", "+4", "+6", "+8",
    "+10", "+12", "+14", "+16", "+18", "+20",
];

#[derive(Debug, Clone)]
pub struct AvionicsState {
    /// Format on the left, the right, the third and the fourth MFD (never the same twice).
    pub formats: [Format; 4],
    /// The cockpit's MFDs: 2, 3 (the F-117A's third) or 4 (the F-22A's fourth); the formats of
    /// those it lacks are never shown.
    pub mfds: usize,
    /// The formats the pilot put on each MFD's bottom bezel buttons (None: `formats`' own).
    pub rows: [Option<[Format; 5]>; 4],
    /// The format menu on an MFD, for the bottom button (0..5) it was opened from.
    pub menu: [Option<u8>; 4],
    /// The sensor of interest (a format on one of the MFDs).
    pub soi: Option<Format>,
    pub hsd_range: usize,
    /// HSD with the aircraft in the middle instead of low (depressed).
    pub hsd_centred: bool,
    /// Cursors in display pixels.
    pub hsd_cursor: Vec2,
    pub harm_cursor: Vec2,
    /// The HSD view the cursors were last placed in: the HSD cursor stays on its ground point
    /// and the HARM cursor turns with the symbols when the aircraft moves or turns.
    cursor_view: Option<HsdView>,
    /// Seconds "lock cursor target" has been held on the HSD or HARM display; past `MARK_HOLD`
    /// a zoomed window around the cursor shows (`zoom_window`).
    pub mark_held: f32,
    /// "Lock cursor target" pressed on a radar format: it locks when let go (`slew_and_lock`).
    radar_lock: bool,
    pub harm_filter: HarmFilter,
    /// MAP shows the waypoint table.
    pub map_way: bool,
    /// SMS shows the mission orders (else the weapons).
    pub sms_ord: bool,
    /// The aircraft has an air-to-ground radar (the ground map format).
    pub radar: bool,
    /// The aircraft shows its own radar signature (the RCS format: the stealth types).
    pub rcs: bool,
    /// The aircraft is the cargo jet (the CGO format).
    pub cargo: bool,
    /// The aircraft flown: its own engine instruments on the ENG format (`eng_page`).
    pub aircraft: crate::aircraft::AircraftKind,
    pub gm_range: usize,
    /// The radar in standby: not transmitting (either format).
    pub radar_standby: bool,
    /// The ground map's cursor (display pixels) and the view it was last placed in (it stays
    /// on its ground point).
    pub gm_cursor: Vec2,
    gm_view: Option<HsdView>,
    /// The A-A radar's range, scan width and bars (indices into `FCR_RANGES_NM`, `FCR_AZ_DEG`,
    /// `FCR_BARS`), antenna elevation (`FCR_EL_STEP` steps) and cursor (display pixels).
    pub fcr_range: usize,
    pub fcr_az: usize,
    pub fcr_bars: usize,
    pub fcr_el: i8,
    pub fcr_cursor: Vec2,
    /// The RCS page's range (index into `RCS_RANGES`).
    pub rcs_range: usize,
    /// The master mode (NAV, A-A, A-G) the formats are shown for, and the formats and the SOI
    /// each one had when the pilot last left it (None: it comes up with its defaults,
    /// `set_master`).
    master: Option<HudMode>,
    modes: [Option<([Format; 4], Option<Format>)>; 3],
    /// Free-fall and retarded bombs per release (1..=`RIPPLE_MAX`).
    pub ripple: u8,
    /// The ripple's spacing (index into `RIPPLE_SPACING_FT`).
    pub ripple_spacing: usize,
    /// The CBU-105's burst altitude (index into `BURST_FT`).
    pub burst: usize,
    /// Right format before ILS was switched on.
    pub before_ils: Option<Format>,
    pub tgp: TgpState,
    /// The helmet sight (with a helmet-mounted display): where the pilot looks from the
    /// cockpit (world direction; None in the outside views).
    pub helmet: Option<Vec3>,
    /// The smoothed helmet sight in the aircraft's frame (head tremor and tracking jitter
    /// filtered out; the airframe's own motion is not delayed).
    helmet_body: Option<Vec3>,
}

/// Where the targeting pod looks.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TgpMode {
    /// At the locked target (else ahead and down, or the camera key's direction).
    #[default]
    Slaved,
    /// Turned by the slew controls, the aircraft's motion not turning it: the ground point it
    /// looks at (Bevy world position), or above the horizon a point far out on its line of sight.
    Area(DVec3),
    /// Holding a ground point (Bevy world position).
    Point(DVec3),
    /// Snowplow: fixed ahead of the nose (`SP_EL` down), nothing locked, no ground track.
    Snowplow,
}

#[derive(Debug, Clone, Default)]
pub struct TgpState {
    pub mode: TgpMode,
    /// Seconds since the last slew in area track.
    idle: f32,
    /// Seconds the current slew has been held.
    held: f32,
    /// The ground point on the pod's line of sight (Bevy world position), when it is up.
    pub los_ground: Option<DVec3>,
    /// The pod's line of sight (world direction), when it is up: the gunship's turret and
    /// Hellfires follow it.
    pub los: Option<Vec3>,
    /// SP pressed: the simulation is to drop its locks.
    clear: bool,
    /// CZ pressed: the pod is to hold the current steerpoint (`update_pod`, which has the route).
    cz: bool,
    /// The vehicle (`Sim::units` index) the point track follows.
    pub track: Option<usize>,
}

impl TgpMode {
    fn label(self) -> &'static str {
        match self {
            TgpMode::Slaved => "SLV",
            TgpMode::Area(_) => "AREA",
            TgpMode::Point(_) => "PNT",
            TgpMode::Snowplow => "SP",
        }
    }
}

/// Pod turn rate at full slew and 1X (rad/s); halved per zoom step.
const POD_RATE: f32 = 0.5;
/// Seconds without slewing before an area track becomes a point track.
const POD_SETTLE: f32 = 0.15;
/// Farthest ground point the pod tracks (m).
const POD_RANGE: f64 = 40_000.0;
/// How far out an area track above the horizon puts its point (m): far enough that flying
/// does not turn the line of sight.
const POD_SKY: f64 = 1.0e7;
const POD_AZ_LIMIT: f32 = 2.97;
const POD_EL_LIMITS: (f32, f32) = (-1.75, 0.5);
/// Snowplow elevation (rad): ahead and a little down, as the unlocked slaved pod.
const SP_EL: f32 = -0.15;
/// Slew response by the range to the ground point the pod looks at: linear up to `SLEW_NEAR`
/// (m); by `SLEW_FAR` each axis follows `SLEW_EXPO_FAR` (fine near the centre) and a held slew
/// starts at `SLEW_START_FAR` of its rate, building to the full rate over `SLEW_RAMP` seconds
/// (fine taps of the keys). Full deflection held still turns the pod at the full rate.
const SLEW_NEAR: f32 = 3_000.0;
const SLEW_FAR: f32 = 20_000.0;
const SLEW_EXPO_FAR: f32 = 3.0;
const SLEW_START_FAR: f32 = 0.15;
const SLEW_RAMP: f32 = 0.8;
/// Ground slew away from (and toward) the aircraft against slewing across, at the same range.
const SLEW_AWAY: f32 = 1.1;
/// At a shallow angle the away slew moves the line of sight by only the sine of the depression;
/// it is sped up by that sine to this power (so the line of sight follows it to 1 - this), e.g.
/// about 3x at 10 km from 1000 ft.
const SLEW_AWAY_SHALLOW: f64 = 0.32;

/// Direction from the nose (aircraft frame, -Z forward) by azimuth and elevation.
pub fn pod_dir(az: f32, el: f32) -> Vec3 {
    Vec3::new(az.sin() * el.cos(), el.sin(), -az.cos() * el.cos())
}

/// Azimuth and elevation of a direction in the aircraft frame.
pub fn pod_angles(d: Vec3) -> (f32, f32) {
    let d = d.normalize_or(Vec3::NEG_Z);
    (d.x.atan2(-d.z), d.y.clamp(-1.0, 1.0).asin())
}

/// The ground a line of sight meets: the plane at height `y` (Bevy metres: the flat world or a
/// carrier's deck) and the rolling hills over it.
#[derive(Clone, Default)]
pub struct Terrain {
    pub y: f64,
    pub hills: stealth_sim::hills::Hills,
}

impl Terrain {
    #[cfg(test)]
    pub fn flat(y: f64) -> Self {
        Self { y, hills: Default::default() }
    }

    /// The ground's height (Bevy y) at a point (Bevy x, z).
    pub fn at(&self, x: f64, z: f64) -> f64 {
        self.y.max(self.hills.ground_m(x, -z) as f64)
    }

    /// A point put on the ground.
    pub fn on(&self, p: DVec3) -> DVec3 {
        DVec3::new(p.x, self.at(p.x, p.z), p.z)
    }
}

/// Where a line of sight meets the ground, within the pod's range.
pub fn ground_hit(origin: DVec3, dir: Vec3, ground: &Terrain) -> Option<DVec3> {
    let t = ground_distance(origin, dir, ground)?;
    (t <= POD_RANGE).then(|| origin + dir.as_dvec3().normalize_or_zero() * t)
}

/// How far a line of sight is followed over the hills (metres), its first step and its steps'
/// growth with the distance.
const HILLS_RAY: f64 = 100_000.0;
const HILLS_STEP: f64 = 5.0;
const HILLS_STEP_GROWTH: f64 = 0.02;

/// Distance along a line of sight to the ground, at any range; None when it does not meet it.
fn ground_distance(origin: DVec3, dir: Vec3, ground: &Terrain) -> Option<f64> {
    let d = dir.as_dvec3().normalize_or_zero();
    if ground.hills.is_flat() {
        if d.y > -1e-4 {
            return None;
        }
        let t = (ground.y - origin.y) / d.y;
        return (t > 0.0).then_some(t);
    }
    // Over the hills: in steps until below the ground, then halved down to it.
    let below = |t: f64| {
        let p = origin + d * t;
        p.y <= ground.at(p.x, p.z)
    };
    let top = ground.y + stealth_sim::hills::MAX_M as f64;
    let (mut lo, mut t) = (0.0, 0.0);
    loop {
        t += HILLS_STEP + t * HILLS_STEP_GROWTH;
        if t > HILLS_RAY || (d.y >= 0.0 && origin.y + d.y * t > top) {
            return None;
        }
        if below(t) {
            break;
        }
        lo = t;
    }
    let mut hi = t;
    for _ in 0..24 {
        let mid = (lo + hi) * 0.5;
        if below(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Some(hi)
}

/// The slew's share of the full pod rate per axis, with the ground point `range` (m) away
/// (None: the pod looks above the horizon, slewing coarsely) and the slew held `held` seconds.
fn slew_gain(slew: Vec2, range: Option<f64>, held: f32) -> Vec2 {
    let far =
        range.map_or(0.0, |r| ((r as f32 - SLEW_NEAR) / (SLEW_FAR - SLEW_NEAR)).clamp(0.0, 1.0));
    let expo = 1.0 + (SLEW_EXPO_FAR - 1.0) * far;
    let start = 1.0 + (SLEW_START_FAR - 1.0) * far;
    let ramp = start + (1.0 - start) * (held / SLEW_RAMP).min(1.0);
    let curve = |v: f32| v.signum() * v.abs().min(1.0).powf(expo);
    Vec2::new(curve(slew.x), curve(slew.y)) * ramp
}

/// One step of the pod: slewing (`slew`, with the pod the SOI) moves it from where it looks
/// (`looking`, world direction) in area track, stabilized against the aircraft's motion: on the
/// ground it moves the ground point, right and away from the aircraft, at a pace by its range (so
/// at a shallow angle the line of sight rises and falls finely); above the horizon it turns the
/// line of sight. `POD_SETTLE` seconds after the slew stops it holds the ground point it looks
/// at. Returns the ground point on its line of sight.
#[allow(clippy::too_many_arguments)]
pub fn pod_step(
    tgp: &mut TgpState,
    slew: Vec2,
    dt: f32,
    zoom: u8,
    orientation: Quat,
    origin: DVec3,
    looking: Option<Vec3>,
    ground: &Terrain,
) -> Option<DVec3> {
    let to_world = |az: f32, el: f32| orientation * pod_dir(az, el);
    // An area track's ground point (as it is when on the ground, not rounded through its
    // direction).
    let area_ground = |p: DVec3| {
        if p.y == ground.at(p.x, p.z) {
            Some(p).filter(|p| (p - origin).length() <= POD_RANGE)
        } else {
            ground_hit(origin, (p - origin).as_vec3(), ground)
        }
    };
    if slew.length() > 0.05 {
        let dir = match tgp.mode {
            TgpMode::Area(p) | TgpMode::Point(p) => (p - origin).as_vec3(),
            TgpMode::Slaved => looking.unwrap_or_else(|| to_world(0.0, SP_EL)),
            TgpMode::Snowplow => to_world(0.0, SP_EL),
        }
        .normalize_or(orientation * Vec3::NEG_Z);
        let range = ground_distance(origin, dir, ground);
        let slew = slew_gain(slew, range, tgp.held);
        tgp.held += dt;
        let rate = POD_RATE / crate::mfd::cam_magnification(zoom) * dt;
        let on_ground = range.filter(|&r| r <= POD_RANGE).map(|r| origin + dir.as_dvec3() * r);
        let moved = match on_ground {
            Some(p) => {
                let flat = DVec3::new(dir.x as f64, 0.0, dir.z as f64);
                let away = flat.try_normalize().unwrap_or_else(|| {
                    let nose = (orientation * Vec3::NEG_Z).as_dvec3();
                    DVec3::new(nose.x, 0.0, nose.z).normalize_or(DVec3::NEG_Z)
                });
                let right = away.cross(DVec3::Y);
                let r = (p - origin).length();
                let shallow = (-dir.y as f64).max(1e-3).powf(-SLEW_AWAY_SHALLOW);
                let away = away * SLEW_AWAY as f64 * shallow;
                ground.on(p + (right * slew.x as f64 + away * slew.y as f64) * r * rate as f64)
            }
            // Above the horizon (or past the pod's range): turned by heading and elevation, as
            // its picture is upright.
            None => {
                let heading = dir.x.atan2(-dir.z) + slew.x * rate;
                let el = dir.y.clamp(-1.0, 1.0).asin() + slew.y * rate;
                origin + pod_dir(heading, el.clamp(-1.55, 1.55)).as_dvec3() * POD_SKY
            }
        };
        // Within the pod's gimbal limits and range.
        let (az, el) = pod_angles(orientation.inverse() * (moved - origin).as_vec3());
        let (caz, cel) =
            (az.clamp(-POD_AZ_LIMIT, POD_AZ_LIMIT), el.clamp(POD_EL_LIMITS.0, POD_EL_LIMITS.1));
        let on_ground = area_ground(moved).is_some_and(|p| p == moved);
        let above = moved.y > ground.at(moved.x, moved.z);
        tgp.mode = TgpMode::Area(if (caz, cel) == (az, el) && (on_ground || above) {
            moved
        } else {
            let dir = to_world(caz, cel);
            ground_hit(origin, dir, ground).unwrap_or(origin + dir.as_dvec3() * POD_SKY)
        });
        tgp.idle = 0.0;
    } else {
        tgp.held = 0.0;
        if let TgpMode::Area(p) = tgp.mode {
            tgp.idle += dt;
            if tgp.idle >= POD_SETTLE
                && let Some(p) = area_ground(p)
            {
                tgp.mode = TgpMode::Point(p);
            }
        }
    }
    match tgp.mode {
        TgpMode::Point(p) => Some(p),
        TgpMode::Area(p) => area_ground(p),
        TgpMode::Snowplow => ground_hit(origin, to_world(0.0, SP_EL), ground),
        TgpMode::Slaved => looking.and_then(|d| ground_hit(origin, d, ground)),
    }
}

/// The pod's line of sight in area or point track (None: slaved, the tracking camera's own
/// rule), from `origin`.
pub fn pod_direction(tgp: &TgpState, orientation: Quat, origin: DVec3) -> Option<Vec3> {
    match tgp.mode {
        TgpMode::Slaved => None,
        TgpMode::Snowplow => Some(orientation * pod_dir(0.0, SP_EL)),
        TgpMode::Area(p) | TgpMode::Point(p) => (p - origin).as_vec3().try_normalize(),
    }
}

impl Default for AvionicsState {
    fn default() -> Self {
        let centre = Vec2::new(HIRES.0 as f32 / 2.0, HIRES.1 as f32 / 2.0);
        Self {
            formats: [Format::Hsd, Format::Sms, Format::Hsi, Format::Eng],
            mfds: 2,
            rows: [None; 4],
            menu: [None; 4],
            soi: Some(Format::Hsd),
            hsd_range: 2,
            hsd_centred: false,
            hsd_cursor: centre - Vec2::Y * 30.0,
            harm_cursor: centre,
            cursor_view: None,
            mark_held: 0.0,
            radar_lock: false,
            harm_filter: HarmFilter::All,
            map_way: false,
            sms_ord: true,
            radar: false,
            rcs: false,
            cargo: false,
            aircraft: Default::default(),
            gm_range: 2,
            radar_standby: false,
            gm_cursor: centre - Vec2::Y * 20.0,
            gm_view: None,
            fcr_range: 2,
            fcr_az: 0,
            fcr_bars: 0,
            fcr_el: 0,
            fcr_cursor: centre,
            rcs_range: 0,
            master: None,
            modes: [None; 3],
            ripple: 1,
            ripple_spacing: 3,
            burst: 2,
            before_ils: None,
            tgp: default(),
            helmet: None,
            helmet_body: None,
        }
    }
}

/// Most bombs a ripple releases (the simulation flies four of the player's stores at a time).
pub const RIPPLE_MAX: u8 = 4;
const RIPPLE_LABELS: [&str; RIPPLE_MAX as usize] = ["RP 1", "RP 2", "RP 3", "RP 4"];
/// Ripple spacings (ft between the bombs' impacts).
pub const RIPPLE_SPACING_FT: [u16; 9] = [25, 50, 75, 100, 150, 200, 300, 400, 500];
const SPACING_LABELS: [&str; 9] =
    ["25FT", "50FT", "75FT", "100FT", "150FT", "200FT", "300FT", "400FT", "500FT"];

/// The CBU-105's burst altitudes (ft above the ground).
pub const BURST_FT: [u16; 6] = [500, 1000, 1500, 2000, 2500, 3000];
const BURST_LABELS: [&str; 6] = ["500FT", "1000FT", "1500FT", "2000FT", "2500FT", "3000FT"];

/// Seconds "lock cursor target" is held before the zoomed window shows.
const MARK_HOLD: f32 = 0.3;

impl AvionicsState {
    /// The zoomed window around the SOI's cursor is up.
    pub fn zoom_window(&self) -> bool {
        self.mark_held >= MARK_HOLD
    }

    /// The ripple's spacing (ft).
    pub fn ripple_ft(&self) -> u16 {
        RIPPLE_SPACING_FT[self.ripple_spacing.min(RIPPLE_SPACING_FT.len() - 1)]
    }

    /// The CBU-105's burst altitude (ft above the ground).
    pub fn burst_ft(&self) -> u16 {
        BURST_FT[self.burst.min(BURST_FT.len() - 1)]
    }

    /// The ground map's range (km).
    pub fn gm_range_km(&self) -> f32 {
        GM_RANGES[self.gm_range.min(GM_RANGES.len() - 1)]
    }

    /// The radar transmits: it searches the air for the HSD whenever it is not in standby
    /// (mapping the ground too while the GM is up).
    pub fn radar_emitting(&self) -> bool {
        self.radar && !self.radar_standby
    }

    /// The aircraft's radar is in standby: its displays show only the aircraft the friendly
    /// side's other radars see.
    pub fn radar_off(&self) -> bool {
        self.radar && self.radar_standby
    }

    /// The radar maps the ground: its ground map is up, not in standby.
    pub fn gm_on(&self) -> bool {
        self.radar && !self.radar_standby && self.shown(Format::Gm).is_some()
    }

    /// The radar searches the air: its A-A format is up, not in standby.
    pub fn fcr_on(&self) -> bool {
        self.radar && !self.radar_standby && self.shown(Format::Fcr).is_some()
    }

    /// The A-A radar's range (metres).
    pub fn fcr_range_m(&self) -> f32 {
        FCR_RANGES_NM[self.fcr_range.min(FCR_RANGES_NM.len() - 1)] * 1852.0
    }

    /// The A-A radar's scan: its half width, bars and antenna elevation (radians).
    pub fn fcr_scan(&self) -> crate::aa_radar::Scan {
        crate::aa_radar::Scan {
            az_half: FCR_AZ_DEG[self.fcr_az.min(FCR_AZ_DEG.len() - 1)].to_radians(),
            bars: FCR_BARS[self.fcr_bars.min(FCR_BARS.len() - 1)],
            elevation: (self.fcr_el as f32 * FCR_EL_STEP).to_radians(),
        }
    }

    /// The formats on an MFD's bottom bezel buttons.
    pub fn formats(&self, side: MfdSide) -> [Format; 5] {
        self.rows[index(side)].unwrap_or_else(|| formats(side, self.radar, self.cargo))
    }

    /// The formats the format menu offers: all of them, the radar's with the radar only, the
    /// RCS page on the stealth types only.
    pub fn available(&self) -> impl Iterator<Item = Format> + '_ {
        ALL_FORMATS.into_iter().filter(|&f| self.has(f))
    }

    /// The aircraft has format `f`.
    pub fn has(&self, f: Format) -> bool {
        (!f.radar() || self.radar) && (f != Format::Rcs || self.rcs) && (f != Format::Cgo || self.cargo)
    }

    /// F3: the left MFD's next map format (HSD, MAP, HARM, FCR, GM) along its bottom row, one
    /// no other MFD shows (with the default rows HSD -> HARM -> MAP -> HSD, or with a radar
    /// HSD -> FCR -> GM -> MAP -> HSD); None without another.
    pub fn next_left_format(&self) -> Option<Format> {
        let row = self.formats(MfdSide::Left);
        let now = self.format(MfdSide::Left);
        let at = row.iter().position(|&f| f == now).unwrap_or(row.len() - 1);
        (1..=row.len()).map(|k| row[(at + k) % row.len()]).find(|&f| {
            matches!(f, Format::Hsd | Format::Map | Format::Harm | Format::Gm | Format::Fcr)
                && f != now
                && self.shown(f).is_none_or(|s| s == MfdSide::Left)
        })
    }

    /// The bottom button (0..5) an MFD's format menu was opened from, if it is open.
    pub fn menu(&self, side: MfdSide) -> Option<u8> {
        self.menu[index(side)]
    }

    /// Keeps the bottom rows the pilot set up on another flight (`old`), those whose formats
    /// this aircraft has.
    pub fn keep_rows(&mut self, old: &AvionicsState) {
        let kit = self.clone();
        for (row, old) in self.rows.iter_mut().zip(old.rows) {
            *row = old.filter(|r| r.iter().all(|&f| kit.has(f)));
        }
    }

    /// The aircraft changed in flight (a campaign's sortie, an AI aircraft taken over): the
    /// radar's formats come with the radar, an MFD showing one without it takes the HARM
    /// display (they replaced it), else its row's first format no MFD shows. The master modes
    /// come up with their defaults again.
    pub fn set_radar(&mut self, radar: bool) {
        let old = self.clone();
        self.radar = radar;
        self.refit(&old);
    }

    /// The aircraft changed to one with (or without) the RCS page: as `set_radar`.
    pub fn set_rcs(&mut self, rcs: bool) {
        let old = self.clone();
        self.rcs = rcs;
        self.refit(&old);
    }

    /// The aircraft changed to the cargo jet or from it: as `set_radar`, its rows its own.
    pub fn set_cargo(&mut self, cargo: bool) {
        let old = self.clone();
        self.cargo = cargo;
        self.refit(&old);
        if cargo {
            self.rows = [None; 4];
            self.cargo_start();
        }
    }

    /// The cargo jet's displays at the start, left to right: the HSD and the HSI before the
    /// pilot, the engines and the cargo page in the middle (the fourth MFD is left of the
    /// third there).
    pub fn cargo_start(&mut self) {
        self.formats = [Format::Hsd, Format::Hsi, Format::Cgo, Format::Eng];
        self.menu = [None; 4];
        self.fix_soi();
    }

    /// After `set_radar` or `set_rcs`: the formats this aircraft lacks leave the rows and the
    /// MFDs.
    fn refit(&mut self, old: &AvionicsState) {
        self.keep_rows(old);
        for &side in self.sides() {
            if !self.has(self.format(side)) {
                let g = std::iter::once(Format::Harm)
                    .chain(self.formats(side))
                    .chain(self.available())
                    .find(|&g| self.shown(g).is_none())
                    .unwrap_or(Format::Map);
                self.formats[index(side)] = g;
            }
        }
        self.modes = [None; 3];
        self.fix_soi();
    }

    /// The HUD's master mode (NAV, A-A, A-G) changed: the formats and the SOI the pilot left
    /// it with come back, or the first time its default on the left MFD (NAV the HSD; with a
    /// radar A-A the FCR and A-G the ground map, as the SOI). True if the formats changed.
    pub fn set_master(&mut self, mode: HudMode) -> bool {
        if self.master == Some(mode) {
            return false;
        }
        let before = self.formats;
        if let Some(old) = self.master {
            self.modes[master_index(old)] = Some((self.formats, self.soi));
        }
        self.master = Some(mode);
        self.menu = [None; 4];
        match self.modes[master_index(mode)] {
            Some((formats, soi)) => (self.formats, self.soi) = (formats, soi),
            None => {
                let f = match mode {
                    HudMode::Nav => Some(Format::Hsd),
                    HudMode::Air => self.radar.then_some(Format::Fcr),
                    HudMode::Gnd => self.radar.then_some(Format::Gm),
                };
                if let Some(f) = f {
                    self.select(MfdSide::Left, f);
                    if mode != HudMode::Nav {
                        self.soi = Some(f);
                    }
                }
            }
        }
        self.fix_soi();
        self.formats != before
    }
}

fn master_index(mode: HudMode) -> usize {
    match mode {
        HudMode::Nav => 0,
        HudMode::Air => 1,
        HudMode::Gnd => 2,
    }
}

fn index(side: MfdSide) -> usize {
    match side {
        MfdSide::Left => 0,
        MfdSide::Right => 1,
        MfdSide::Third => 2,
        MfdSide::Fourth => 3,
    }
}

impl AvionicsState {
    pub fn format(&self, side: MfdSide) -> Format {
        self.formats[index(side)]
    }

    /// The cockpit's MFDs, left to right on the F-16's panel (the F-117A's third is right of
    /// its centre display, the right MFD; the F-22A's fourth under the centre one).
    pub fn sides(&self) -> &'static [MfdSide] {
        match self.mfds {
            4 => &[MfdSide::Left, MfdSide::Right, MfdSide::Third, MfdSide::Fourth],
            3 => &[MfdSide::Left, MfdSide::Right, MfdSide::Third],
            _ => &[MfdSide::Left, MfdSide::Right],
        }
    }

    /// The MFD showing a format.
    pub fn shown(&self, f: Format) -> Option<MfdSide> {
        self.sides().iter().copied().find(|&s| self.format(s) == f)
    }

    /// The cockpit's MFD count (2..=4). The third and fourth MFDs come up on their bottom row's
    /// first format (the HSI, the engines), or the first one no MFD left of them shows.
    pub fn set_mfds(&mut self, mfds: usize) {
        self.mfds = mfds;
        for i in 2..4 {
            let side = [MfdSide::Third, MfdSide::Fourth][i - 2];
            let shown = self.formats;
            let f = self
                .formats(side)
                .into_iter()
                .chain(self.available())
                .find(|f| !shown[..i].contains(f))
                .unwrap_or(Format::Map);
            self.formats[i] = f;
        }
        self.menu = [None; 4];
        self.modes = [None; 3];
        self.fix_soi();
    }

    /// The sensor of interest if it is on an MFD.
    pub fn soi_format(&self) -> Option<Format> {
        self.soi.filter(|&f| self.shown(f).is_some() && self.soi_capable(f))
    }

    /// Formats that can be the SOI: those with a cursor or a pod, and the MAP while it shows
    /// the waypoint table (the slew keys then move the waypoint being edited).
    pub fn soi_capable(&self, f: Format) -> bool {
        f.soi_capable() || f == Format::Map && self.map_way
    }

    /// Puts a format on an MFD (closing its format menu); the MFD showing the new one gets this
    /// one's format.
    pub fn select(&mut self, side: MfdSide, f: Format) {
        let i = index(side);
        self.menu[i] = None;
        match self.shown(f) {
            Some(o) => self.formats.swap(i, index(o)),
            None => self.formats[i] = f,
        }
        self.fix_soi();
    }

    /// Shows a format, on the right MFD unless it is up already.
    pub fn show(&mut self, f: Format) {
        if self.shown(f).is_none() {
            self.select(MfdSide::Right, f);
        }
    }

    /// The SOI stays on a displayed format that can be one.
    fn fix_soi(&mut self) {
        if self.soi_format().is_none() {
            self.soi = self.sides().iter().map(|&s| self.format(s)).find(|&f| self.soi_capable(f));
        }
    }
}

/// Applies a display command with the modern avionics.
pub fn apply(state: &mut MfdState, raster: &mut MapRaster, cmd: MfdCmd) {
    let av = &mut state.av;
    match cmd {
        MfdCmd::Format(side, f) => {
            av.select(side, f);
            if f == Format::Map {
                raster.invalidate();
            }
        }
        // A bottom button: its format; pressed again on the format it shows, the format menu
        // (pressed on the menu, back to the format).
        MfdCmd::Row(side, k) => {
            let f = av.formats(side)[k as usize];
            match av.menu(side) {
                Some(m) if m == k => av.menu[index(side)] = None,
                None if av.format(side) == f => av.menu[index(side)] = Some(k),
                _ => {
                    av.select(side, f);
                    if f == Format::Map {
                        raster.invalidate();
                    }
                }
            }
        }
        // A format picked in the menu: onto the bottom button it was opened from (swapping
        // places with that format elsewhere on the row), and up on the MFD.
        MfdCmd::Assign(side, f) => {
            let Some(k) = av.menu(side) else { return };
            let mut row = av.formats(side);
            if let Some(j) = row.iter().position(|&g| g == f) {
                row[j] = row[k as usize];
            }
            row[k as usize] = f;
            av.rows[index(side)] = Some(row);
            av.select(side, f);
            if f == Format::Map {
                raster.invalidate();
            }
        }
        MfdCmd::Soi(side) => {
            let f = av.format(side);
            if av.soi_capable(f) {
                av.soi = Some(f);
            }
        }
        // To the next MFD round (left, right, third) with a format that can be the SOI.
        MfdCmd::SwapSoi => {
            let side = av.soi_format().and_then(|f| av.shown(f)).unwrap_or(MfdSide::Right);
            let sides = av.sides();
            let at = sides.iter().position(|&s| s == side).unwrap_or(0);
            let next = (1..sides.len())
                .map(|k| av.format(sides[(at + k) % sides.len()]))
                .find(|&f| av.soi_capable(f));
            if let Some(f) = next {
                av.soi = Some(f);
            }
        }
        MfdCmd::HsdRange(d) => {
            av.hsd_range =
                (av.hsd_range as i32 + d as i32).clamp(0, HSD_RANGES.len() as i32 - 1) as usize
        }
        MfdCmd::HsdCentre => av.hsd_centred = !av.hsd_centred,
        MfdCmd::GmRange(d) => {
            av.gm_range =
                (av.gm_range as i32 + d as i32).clamp(0, GM_RANGES.len() as i32 - 1) as usize
        }
        MfdCmd::RadarStandby => av.radar_standby = !av.radar_standby,
        MfdCmd::FcrRange(d) => {
            av.fcr_range =
                (av.fcr_range as i32 + d as i32).clamp(0, FCR_RANGES_NM.len() as i32 - 1) as usize
        }
        MfdCmd::RcsRange(d) => {
            av.rcs_range =
                (av.rcs_range as i32 + d as i32).clamp(0, RCS_RANGES.len() as i32 - 1) as usize
        }
        MfdCmd::FcrAz => av.fcr_az = (av.fcr_az + 1) % FCR_AZ_DEG.len(),
        MfdCmd::FcrBars => av.fcr_bars = (av.fcr_bars + 1) % FCR_BARS.len(),
        MfdCmd::FcrElevation(d) => av.fcr_el = (av.fcr_el + d).clamp(-FCR_EL_MAX, FCR_EL_MAX),
        MfdCmd::MapZoom(d) => {
            state.map_zoom = (state.map_zoom + d as i32).clamp(2, state.max_map_zoom());
            raster.invalidate();
        }
        // Z/X: the HSD's range when it is the SOI, else the map's zoom.
        MfdCmd::ZoomIn | MfdCmd::ZoomOut => {
            let d = if cmd == MfdCmd::ZoomIn { 1 } else { -1 };
            if av.soi_format() == Some(Format::Gm) {
                av.gm_range =
                    (av.gm_range as i32 - d).clamp(0, GM_RANGES.len() as i32 - 1) as usize;
            } else if av.soi_format() == Some(Format::Fcr) {
                av.fcr_range =
                    (av.fcr_range as i32 - d).clamp(0, FCR_RANGES_NM.len() as i32 - 1) as usize;
            } else if av.soi_format() == Some(Format::Hsd) || av.shown(Format::Map).is_none() {
                av.hsd_range =
                    (av.hsd_range as i32 - d).clamp(0, HSD_RANGES.len() as i32 - 1) as usize;
            } else {
                state.map_zoom = (state.map_zoom + d).clamp(2, state.max_map_zoom());
                raster.invalidate();
            }
        }
        MfdCmd::ToggleOrient => state.map_heading_up = !state.map_heading_up,
        MfdCmd::MapWay => {
            av.map_way = !av.map_way;
            av.fix_soi();
            raster.invalidate();
        }
        MfdCmd::Sms(ord) => {
            av.sms_ord = ord;
            av.show(Format::Sms);
        }
        MfdCmd::Ripple(d) => {
            av.ripple = (av.ripple as i32 + d as i32).clamp(1, RIPPLE_MAX as i32) as u8;
        }
        MfdCmd::Burst(d) => {
            av.burst = (av.burst as i32 + d as i32).clamp(0, BURST_FT.len() as i32 - 1) as usize;
        }
        MfdCmd::RippleSpacing(d) => {
            av.ripple_spacing = (av.ripple_spacing as i32 + d as i32)
                .clamp(0, RIPPLE_SPACING_FT.len() as i32 - 1)
                as usize;
        }
        MfdCmd::HarmFilter => {
            av.harm_filter = match av.harm_filter {
                HarmFilter::All => HarmFilter::Sam,
                HarmFilter::Sam => HarmFilter::Air,
                HarmFilter::Air => HarmFilter::All,
            }
        }
        MfdCmd::Flir => {
            state.flir = !state.flir;
            av.show(Format::Tgp);
        }
        MfdCmd::Cam(d) => {
            state.cam_dir = d;
            av.tgp.mode = TgpMode::Slaved;
            av.show(Format::Tgp);
        }
        // Snowplow, with the pod the SOI.
        MfdCmd::TgpSp => {
            if av.soi_format() == Some(Format::Tgp) {
                av.tgp.mode = TgpMode::Snowplow;
                av.tgp.clear = true;
            }
        }
        // A target key or a display lock: the pod follows the lock again.
        MfdCmd::TgpSlave => av.tgp.mode = TgpMode::Slaved,
        MfdCmd::TgpCz => av.tgp.cz = true,
        // Applied before (`mfd::cam_zoom`).
        MfdCmd::CamZoom(_) | MfdCmd::CamZoomCycle => {}
        // Moves the waypoint (`mfd::edit_waypoints`).
        MfdCmd::WaypointNudge(..) => {}
        MfdCmd::Ils => {
            state.ils = !state.ils;
            if state.ils {
                av.before_ils = Some(av.format(MfdSide::Right));
                av.select(MfdSide::Right, Format::Tgp);
            } else if let Some(f) = av.before_ils.take() {
                av.select(MfdSide::Right, f);
            }
        }
        // The classic pages' keys and caps.
        MfdCmd::LeftPage(crate::mfd::LeftMfd::Map) => av.select(MfdSide::Left, Format::Map),
        MfdCmd::LeftPage(crate::mfd::LeftMfd::Tac) => av.select(MfdSide::Left, Format::Hsd),
        MfdCmd::RightPage(p) => match p {
            crate::mfd::RightMfd::Camera => av.show(Format::Tgp),
            crate::mfd::RightMfd::Way => {
                av.map_way = true;
                av.show(Format::Map);
            }
            crate::mfd::RightMfd::Wpn => {
                av.sms_ord = false;
                av.show(Format::Sms);
            }
            crate::mfd::RightMfd::Ord => {
                av.sms_ord = true;
                av.show(Format::Sms);
            }
        },
    }
}

/// Label, function and lit state of a bezel button with the modern avionics: the formats along
/// the bottom, the format's own functions along the top (SOI top right); the side rows (10..20)
/// are free but for the waypoint table's.
pub fn osb(side: MfdSide, slot: u8, s: &MfdState) -> Option<(&'static str, OsbFn, bool)> {
    use crate::input::Action as A;
    use OsbFn::*;
    let av = &s.av;
    let f = av.format(side);
    let menu = av.menu(side);
    if (5..10).contains(&slot) {
        let k = slot - 5;
        let g = av.formats(side)[k as usize];
        let lit = menu.map_or(f == g, |m| m == k);
        return Some((g.label(), Cmd(MfdCmd::Row(side, k)), lit));
    }
    // The format menu: every format down the sides (the one on the button it came from lit).
    if let Some(k) = menu {
        let g = av.available().nth(slot.checked_sub(10)? as usize)?;
        return Some((g.label(), Cmd(MfdCmd::Assign(side, g)), av.formats(side)[k as usize] == g));
    }
    let soi = ("SOI", Cmd(MfdCmd::Soi(side)), av.soi_format() == Some(f));
    Some(match (f, slot) {
        (Format::Hsd, 0) => {
            (if av.hsd_centred { "CEN" } else { "DEP" }, Cmd(MfdCmd::HsdCentre), false)
        }
        (Format::Hsd, 1) => ("RNG-", Cmd(MfdCmd::HsdRange(-1)), false),
        (Format::Hsd, 2) => (HSD_RANGE_LABELS[av.hsd_range], Readout, false),
        (Format::Hsd, 3) => ("RNG+", Cmd(MfdCmd::HsdRange(1)), false),
        (Format::Harm, 0) => (
            match av.harm_filter {
                HarmFilter::All => "ALL",
                HarmFilter::Sam => "SAM",
                HarmFilter::Air => "AIR",
            },
            Cmd(MfdCmd::HarmFilter),
            false,
        ),
        (Format::Gm, 0) => (
            if av.radar_standby { "STBY" } else { "GM" },
            Cmd(MfdCmd::RadarStandby),
            av.radar_standby,
        ),
        (Format::Gm, 1) => ("RNG-", Cmd(MfdCmd::GmRange(-1)), false),
        (Format::Gm, 2) => (GM_RANGE_LABELS[av.gm_range], Readout, false),
        (Format::Gm, 3) => ("RNG+", Cmd(MfdCmd::GmRange(1)), false),
        // The A-A radar's like the F-16's: the mode on top, the range, the scan's width and
        // bars down the left, the antenna's elevation on the right.
        (Format::Fcr, 0) => (
            if av.radar_standby { "STBY" } else { "CRM" },
            Cmd(MfdCmd::RadarStandby),
            av.radar_standby,
        ),
        (Format::Fcr, 1) => ("RWS", Readout, false),
        (Format::Fcr, 10) => ("RNG+", Cmd(MfdCmd::FcrRange(1)), false),
        (Format::Fcr, 11) => (FCR_RANGE_LABELS[av.fcr_range], Readout, false),
        (Format::Fcr, 12) => ("RNG-", Cmd(MfdCmd::FcrRange(-1)), false),
        (Format::Fcr, 13) => (FCR_AZ_LABELS[av.fcr_az], Cmd(MfdCmd::FcrAz), false),
        (Format::Fcr, 14) => (FCR_BAR_LABELS[av.fcr_bars], Cmd(MfdCmd::FcrBars), false),
        (Format::Fcr, 15) => ("EL+", Cmd(MfdCmd::FcrElevation(1)), false),
        (Format::Fcr, 16) => (FCR_EL_LABELS[(av.fcr_el + FCR_EL_MAX) as usize], Readout, false),
        (Format::Fcr, 17) => ("EL-", Cmd(MfdCmd::FcrElevation(-1)), false),
        (Format::Hsd | Format::Harm | Format::Tgp | Format::Gm | Format::Fcr, 4) => soi,
        // The cargo page: the ramp up or down (the bay doors' key).
        (Format::Cgo, 0) => ("RAMP", OsbFn::Action(crate::input::Action::BayDoors), false),
        (Format::Rcs, 1) => ("RNG-", Cmd(MfdCmd::RcsRange(-1)), false),
        (Format::Rcs, 2) => (RCS_RANGE_LABELS[av.rcs_range], Readout, false),
        (Format::Rcs, 3) => ("RNG+", Cmd(MfdCmd::RcsRange(1)), false),
        (Format::Tgp, 0) => ("FLIR", Cmd(MfdCmd::Flir), s.flir),
        (Format::Tgp, 1) => (
            crate::mfd::cam_zoom_label(s.cam_zoom),
            Cmd(MfdCmd::CamZoomCycle),
            s.cam_zoom.is_some(),
        ),
        (Format::Tgp, 2) => ("SP", Cmd(MfdCmd::TgpSp), av.tgp.mode == TgpMode::Snowplow),
        // CZ: the pod to the current steerpoint, not in A-A.
        (Format::Tgp, 3) if !s.air => ("CZ", Cmd(MfdCmd::TgpCz), false),
        // The waypoint table: the top row has no map functions, the side ones are its own.
        (Format::Map, 0..=2) if av.map_way => return None,
        (Format::Map, 4) if av.map_way => soi,
        (Format::Map, 10..=16 | 19) if av.map_way => return crate::mfd::way_osb(slot, s),
        (Format::Map, 0) => {
            (if s.map_heading_up { "HUP" } else { "NUP" }, Cmd(MfdCmd::ToggleOrient), false)
        }
        (Format::Map, 1) => ("ZM-", Cmd(MfdCmd::MapZoom(-1)), false),
        (Format::Map, 2) => ("ZM+", Cmd(MfdCmd::MapZoom(1)), false),
        (Format::Map, 3) => ("WAY", Cmd(MfdCmd::MapWay), av.map_way),
        (Format::Sms, 0) => ("WPN", Cmd(MfdCmd::Sms(false)), !av.sms_ord),
        (Format::Sms, 1) => ("ORD", Cmd(MfdCmd::Sms(true)), av.sms_ord),
        (Format::Sms, 2) if !av.sms_ord => ("SEL", Action(A::SelectWeapon), false),
        (Format::Sms, 3) if !av.sms_ord => {
            (crate::mfd::bomb_mode_label(s.ccrp), Action(A::BombMode), false)
        }
        // The CBU-105 is released one at a time, opening at the burst altitude on the right.
        (Format::Sms, 10..=12) if !av.sms_ord && s.sfw => return None,
        (Format::Sms, 15) if !av.sms_ord && s.sfw => ("BA+", Cmd(MfdCmd::Burst(1)), false),
        (Format::Sms, 16) if !av.sms_ord && s.sfw => {
            (BURST_LABELS[av.burst.min(BURST_LABELS.len() - 1)], Readout, false)
        }
        (Format::Sms, 17) if !av.sms_ord && s.sfw => ("BA-", Cmd(MfdCmd::Burst(-1)), false),
        // The release program down the sides: ripple count on the left, spacing on the right.
        (Format::Sms, 10) if !av.sms_ord => ("RP+", Cmd(MfdCmd::Ripple(1)), false),
        (Format::Sms, 11) if !av.sms_ord => {
            (RIPPLE_LABELS[(av.ripple.clamp(1, RIPPLE_MAX) - 1) as usize], Readout, false)
        }
        (Format::Sms, 12) if !av.sms_ord => ("RP-", Cmd(MfdCmd::Ripple(-1)), false),
        (Format::Sms, 15) if !av.sms_ord => ("SPC+", Cmd(MfdCmd::RippleSpacing(1)), false),
        (Format::Sms, 16) if !av.sms_ord => {
            (SPACING_LABELS[av.ripple_spacing.min(SPACING_LABELS.len() - 1)], Readout, false)
        }
        (Format::Sms, 17) if !av.sms_ord => ("SPC-", Cmd(MfdCmd::RippleSpacing(-1)), false),
        (Format::Hsi, 0) => ("WP-", Action(A::LastWaypoint), false),
        (Format::Hsi, 1) => ("WP+", Action(A::NextWaypoint), false),
        _ => return None,
    })
}

// ------------------------------------------------------------------------------------------
// Drawing

/// Height of the bezel label bands at the top and bottom of the display.
const BAND: i32 = 16;
/// Colours (palette 0..15, the same at every time of day).
const BLACK: u8 = 5;
const DARK: u8 = 8;
const GREY: u8 = 7;
const NAVY: u8 = 2;
const BLUE: u8 = 9;
const GREEN: u8 = 10;
const CYAN: u8 = 11;
const RED: u8 = 13;
const WHITE: u8 = 15;
const YELLOW: u8 = 17;
const AMBER: u8 = 18;
const DEEP_RED: u8 = 16;

/// Draws an MFD with the modern avionics.
pub fn draw(p: &MfdParams, side: MfdSide, c: &mut Canvas, art: &CockpitArt, fonts: &Fonts) {
    let s = &p.state;
    let av = &s.av;
    let f = av.format(side);
    let ac = p.aircraft.single().ok();
    let soi = av.soi_format() == Some(f);
    // The format menu: just the formats along the sides.
    if av.menu(side).is_some() {
        c.clear(BLACK);
        draw_labels(c, side, s);
        return;
    }
    match f {
        Format::Eng => draw_eng_page(p, c, fonts),
        Format::Hsd => draw_hsd(p, c, ac, soi),
        Format::Harm => draw_harm(p, c, soi),
        Format::Gm => draw_gm(p, c, ac, soi),
        Format::Fcr => draw_fcr(p, c, ac, soi),
        Format::Hsi => draw_hsi_page(p, c, fonts),
        Format::Rcs => draw_rcs_page(p, c),
        Format::Cgo => crate::cargo_page::draw(c, p.cues.drop.as_ref(), crate::hud::blink(&p.time)),
        Format::Sms if !av.sms_ord => draw_sms(p, c),
        _ => {
            // The classic pages, between the label bands.
            let mut page = Canvas::scaled(c.w, c.h - 2 * BAND as usize, c.scale);
            match f {
                Format::Tgp => p.draw_camera_hires(&mut page, fonts),
                Format::Map if av.map_way => p.draw_way_hires(&mut page, fonts, ac),
                Format::Map => p.draw_map_hires(&mut page, art, fonts, ac, true),
                _ => p.draw_ord_hires(&mut page, fonts),
            }
            // See-through around the pod picture, black elsewhere.
            c.clear(if f == Format::Tgp && p.cam.live { 0 } else { BLACK });
            c.copy_rect(&page, 0, 0, page.w as i32, page.h as i32, 0, BAND);
            if f == Format::Tgp {
                // Crosshair over the picture's centre, the pod's mode bottom left of the page.
                let col = p.camera_text_colour();
                fonts.text_scaled(
                    c,
                    6,
                    c.h as i32 - BAND - 42,
                    av.tgp.mode.label().as_bytes(),
                    col,
                    2,
                );
                let o = Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0);
                for d in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
                    c.stroke(o + d * 6.0, o + d * 22.0, 1.3, col);
                }
            }
        }
    }
    draw_labels(c, side, s);
    if soi {
        c.frame(Vec2::splat(1.0), Vec2::new(c.w as f32 - 2.0, c.h as f32 - 2.0), 2.0, WHITE);
    }
}

/// Glyph height of the bezel labels (the stroke font, sharp at the canvas' resolution).
const LABEL_GLYPH: f32 = 7.0;

/// The bezel button labels next to their buttons: lit ones inverted. The top and bottom ones
/// in the label bands, centred over their buttons; the side ones against the edges, kept
/// between the bands.
fn draw_labels(c: &mut Canvas, side: MfdSide, s: &MfdState) {
    use crate::mfd::OsbEdge;
    let h = LABEL_GLYPH as i32;
    for slot in 0..crate::mfd::OSB_COUNT {
        let Some((label, _, lit)) = osb(side, slot, s) else { continue };
        let (edge, k) = crate::mfd::osb_edge(slot);
        let k = k as i32;
        let w = crate::font::stroke_width(label, LABEL_GLYPH).ceil() as i32;
        // Vertical middles: of the label bands, and of the side buttons (clear of the bands).
        let row = ((c.h as i32 * (2 * k + 1)) / 10)
            .clamp(BAND + 2 + h / 2, c.h as i32 - BAND - 2 - h / 2);
        let col = (c.w as i32 * (2 * k + 1)) / 10 - w / 2;
        let (x, mid) = match edge {
            OsbEdge::Top => (col, BAND / 2),
            OsbEdge::Bottom => (col, c.h as i32 - BAND / 2),
            OsbEdge::Left => (4, row),
            OsbEdge::Right => (c.w as i32 - 4 - w, row),
        };
        let y = mid - h / 2;
        if lit {
            c.fill_rect(x - 2, y - 2, w + 4, h + 4, GREEN);
        }
        stroke_text(
            c,
            x as f32,
            y as f32,
            label,
            LABEL_GLYPH,
            1.0,
            if lit { BLACK } else { GREEN },
        );
    }
}

/// The SMS inventory (F-16 style): the master mode, the selected weapon and its rounds, and
/// the stations with what each carries, the selected weapon's highlighted; the bombs' ripple
/// count and spacing by their side buttons.
/// The engine page (after the F-22A's): both engines' speed (%) on dials, the fuel flow
/// (lb a minute), the aircraft from above with its gun rounds and the stores on its stations
/// (side bays either side, the main bay's in the middle), the internal fuel and the chaff and
/// flares left. A fire shows on the left engine, an engine hit on the right one.
pub fn draw_eng_page(p: &MfdParams, c: &mut Canvas, _fonts: &Fonts) {
    c.clear(BLACK);
    let Ok((a, _)) = p.aircraft.single() else { return };
    let k = p.constants.as_deref();
    // The other types have their own instruments.
    let kind = p.state.av.aircraft;
    if kind != crate::aircraft::AircraftKind::F22 {
        crate::eng_page::draw(c, kind, &crate::eng_page::Eng::of(kind, a, k));
        return;
    }
    let (fire, hit) = (a.damage.fuel_leak > 0.0, a.damage.throttle_cap.is_some());
    // Engine speed (%) and fuel flow (lb a minute): the rotor's engines, or a jet's spool.
    let (rpm, flow) = match k.and_then(|k| k.heli.as_ref()) {
        Some(rotor) => {
            let r = crate::mpd::engine_readings(a, rotor.fuel_burn, hit, fire);
            (r.ng, r.flow / 60.0)
        }
        None => {
            let n = if a.fuel > 0.0 { 60.0 + a.thrust.clamp(0.0, 100.0) * 0.4 } else { 0.0 };
            (
                [n, if hit { n.min(60.0 + a.thrust.min(100.0) * 0.3) } else { n }],
                k.map_or(0.0, |k| crate::flight::fuel_flow(a, k)),
            )
        }
    };
    let w = c.w as f32;
    let mid = w / 2.0;
    // The dials: an arc from 0 (up) clockwise to 100% (left), the needle, the reading in the
    // open quarter.
    for (e, cx) in [(0, mid - 66.0), (1, mid + 66.0)] {
        let col = match e {
            0 if fire => RED,
            1 if hit => AMBER,
            _ => GREEN,
        };
        let o = Vec2::new(cx, 44.0);
        let r = 19.0;
        let up = -std::f32::consts::FRAC_PI_2;
        c.arc(o, r, up, std::f32::consts::PI, 1.6, col);
        for t in [up, 0.0, -up] {
            let d = Vec2::from_angle(t);
            c.stroke(o + d * (r - 4.0), o + d * r, 1.4, col);
        }
        let frac = (rpm[e] / 100.0).clamp(0.0, 1.1);
        let d = Vec2::from_angle(up + frac * 1.5 * std::f32::consts::PI);
        c.stroke(o, o + d * (r - 2.0), 1.6, col);
        let text = format!("{:.0} %", rpm[e]);
        stroke_text_at(c, o.x - 12.0, o.y - 16.0, &text, 9.0, 1.2, WHITE, Align::Right);
    }
    stroke_text_at(c, mid, 54.0, "FF", 11.0, 1.5, GREEN, Align::Centre);
    stroke_text_at(c, mid - 30.0, 72.0, "PPM", 9.0, 1.3, GREEN, Align::Centre);
    stroke_text_at(c, mid + 30.0, 72.0, &format!("{flow:.0}"), 9.0, 1.3, WHITE, Align::Centre);
    // The aircraft from above: the wings' leading edges, the fuselage down to the main bay.
    let v = |x: f32, y: f32| Vec2::new(mid + x, y);
    let lead = [(-100.0, 92.0), (-36.0, 88.0), (-24.0, 82.0)];
    let trail = [(-100.0, 94.0), (-42.0, 94.0), (-30.0, 112.0), (-28.0, 118.0)];
    for sx in [-1.0, 1.0] {
        let side = |pts: &[(f32, f32)]| pts.iter().map(|&(x, y)| v(sx * x, y)).collect::<Vec<_>>();
        c.polyline(&side(&lead), 1.2, GREEN);
        c.polyline(&side(&trail), 1.2, GREEN);
    }
    c.stroke(v(-24.0, 82.0), v(24.0, 82.0), 1.2, GREEN);
    // The gun's rounds in a box under the nose.
    if let Some(rounds) = p.stores.gun_rounds {
        c.frame(v(-14.0, 85.0), v(14.0, 97.0), 1.2, GREEN);
        stroke_text_at(c, mid, 91.0, &rounds.to_string(), 8.0, 1.2, WHITE, Align::Centre);
    }
    // The stores: a circled letter a round (the designation's last), the first and the last
    // station in the side bays (with three or more), the others side by side in the main bay.
    let stations = &p.stores.stations;
    let letter = |bay: Option<usize>| {
        bay.and_then(|b| p.stores.bays.get(b))
            .and_then(|b| b.short.chars().rev().find(|ch| ch.is_ascii_alphanumeric()))
            .map(|ch| ch.to_ascii_uppercase().to_string())
    };
    let round = |c: &mut Canvas, at: Vec2, l: &str| {
        c.ring(at, 4.2, 1.0, WHITE);
        stroke_text_at(c, at.x, at.y, l, 6.0, 1.0, WHITE, Align::Centre);
    };
    let sides = stations.len() >= 3;
    if sides {
        for (sx, &(bay, rounds)) in [(-1.0, &stations[0]), (1.0, &stations[stations.len() - 1])] {
            let at = v(sx * 36.0, 102.0);
            let d = 9.0;
            c.polyline(
                &[
                    at - Vec2::X * d,
                    at - Vec2::Y * d,
                    at + Vec2::X * d,
                    at + Vec2::Y * d,
                    at - Vec2::X * d,
                ],
                1.2,
                GREEN,
            );
            if let Some(l) = letter(bay).filter(|_| rounds > 0) {
                round(c, at, &l);
            }
        }
    }
    let main = if sides { &stations[1..stations.len() - 1] } else { &stations[..] };
    let (x0, x1, top) = (mid - 30.0, mid + 30.0, 118.0);
    c.frame(Vec2::new(x0, top), Vec2::new(x1, top + 13.0), 1.2, GREEN);
    let n = main.len().max(1);
    let cell = (x1 - x0) / n as f32;
    for (i, &(bay, rounds)) in main.iter().enumerate() {
        let left = x0 + cell * i as f32;
        if i > 0 {
            c.stroke(Vec2::new(left, top), Vec2::new(left, top + 13.0), 1.2, GREEN);
        }
        let Some(l) = letter(bay) else { continue };
        let fit = ((cell - 2.0) / 9.0).floor().max(1.0) as u32;
        let shown = rounds.min(fit);
        for k in 0..shown {
            let x = left + cell / 2.0 + (k as f32 - (shown - 1) as f32 / 2.0) * 9.0;
            round(c, Vec2::new(x, top + 6.5), &l);
        }
    }
    // The internal fuel (amber under bingo), the chaff and flares.
    let bingo = k.map_or(0.0, |k| k.bingo);
    let fuel_col = if a.fuel < bingo { AMBER } else { WHITE };
    stroke_text_at(c, mid - 26.0, 144.0, "INT FUEL", 9.0, 1.3, GREEN, Align::Centre);
    stroke_text_at(
        c,
        mid + 34.0,
        144.0,
        &format!("{:.0}", a.fuel.max(0.0)),
        9.0,
        1.3,
        fuel_col,
        Align::Left,
    );
    for (k, (label, n)) in [("CH", p.cm.chaff), ("FL", p.cm.flares)].into_iter().enumerate() {
        let y = 128.0 + 16.0 * k as f32;
        stroke_text_at(c, 8.0, y, label, 8.0, 1.2, GREEN, Align::Left);
        stroke_text_at(c, 30.0, y, &n.to_string(), 8.0, 1.2, WHITE, Align::Left);
    }
}

fn draw_sms(p: &MfdParams, c: &mut Canvas) {
    c.clear(BLACK);
    let s = &p.state;
    let st = &p.stores;
    let (w, h) = (c.w as f32, c.h as f32);
    let mid = w / 2.0;
    let sel = st.bays.get(st.selected).filter(|_| !st.gun_selected);
    let master = if s.air {
        "A-A"
    } else if s.ground {
        "A-G"
    } else {
        "NAV"
    };
    let mut y = BAND as f32 + 6.0;
    stroke_text_at(c, mid, y, master, 8.0, 1.2, GREEN, Align::Centre);
    y += 14.0;
    let weapon = match (sel, st.gun_rounds) {
        (Some(b), _) => format!("{} {}", b.count, b.short),
        (None, Some(n)) if st.gun_selected => match st.gun_slave {
            Some(slave) => format!("GUN {n} {slave}"),
            None => format!("GUN {n}"),
        },
        _ => "NO STORES".to_string(),
    };
    stroke_text_at(c, mid, y, &weapon, 11.0, 1.5, WHITE, Align::Centre);
    y += 16.0;
    c.stroke(Vec2::new(44.0, y), Vec2::new(w - 44.0, y), 1.0, DARK);
    y += 6.0;
    // The stations in mirrored pairs from the wingtips in (the first and the last on the top
    // row, numbers outboard), a centreline station between them, then the gun.
    let cell = |k: usize| {
        let (g, n) = st.stations[k];
        match g.and_then(|g| st.bays.get(g).map(|b| (g, b))) {
            Some((g, b)) => (
                format!("{}", k + 1),
                format!("{n} {}", b.short),
                g == st.selected && !st.gun_selected,
                n == 0,
            ),
            None => (format!("{}", k + 1), "----".to_string(), false, true),
        }
    };
    let n = st.stations.len();
    // Each row: (left, centre, right).
    type Cell = (String, String, bool, bool);
    let mut rows: Vec<[Option<Cell>; 3]> =
        (0..n / 2).map(|k| [Some(cell(k)), None, Some(cell(n - 1 - k))]).collect();
    if n % 2 == 1 {
        rows.push([None, Some(cell(n / 2)), None]);
    }
    if let Some(r) = st.gun_rounds {
        rows.push([None, Some(("GUN".to_string(), r.to_string(), st.gun_selected, r == 0)), None]);
    }
    let pitch = ((h - BAND as f32 - 4.0 - y) / rows.len().max(1) as f32).min(14.0);
    let glyph = (pitch - 5.0).min(6.5);
    let (left, right, gap) = (46.0, w - 46.0, 14.0);
    let col_w = (right - left - gap) / 2.0;
    for (i, row) in rows.iter().enumerate() {
        // The row's vertical middle.
        let ry = y + i as f32 * pitch + glyph / 2.0;
        for (slot, cell) in row.iter().enumerate() {
            let Some((label, text, lit, empty)) = cell else { continue };
            let x0 = match slot {
                0 => left,
                1 => mid - col_w / 2.0,
                _ => right - col_w,
            };
            let x1 = x0 + col_w;
            if *lit {
                let top = (ry - (pitch - 1.0) / 2.0).round() as i32;
                c.fill_rect(x0 as i32 - 2, top, col_w as i32 + 4, pitch as i32 - 1, GREEN);
            }
            let col = if *lit {
                BLACK
            } else if *empty {
                GREY
            } else {
                GREEN
            };
            // The number outboard: on the right in the right-hand column.
            let (outer, inner) = if slot == 2 { (x1, x0) } else { (x0, x1) };
            let (oa, ia) =
                if slot == 2 { (Align::Right, Align::Left) } else { (Align::Left, Align::Right) };
            stroke_text_at(c, outer, ry, label, glyph, 1.2, col, oa);
            stroke_text_at(c, inner, ry, text, glyph, 1.2, col, ia);
        }
    }
}

/// HSI course deviation at full scale (km off the course, two dots).
const HSI_FULL_SCALE_KM: f64 = 5.0;

/// The HSI format (and the third MFD's page with the classic pages): a heading-up compass
/// rose with the digital heading over it; the course arrow along the leg to the steerpoint
/// (from the previous one; to the first one, from the aircraft) with its deviation bar, two
/// dots either side (`HSI_FULL_SCALE_KM`) and the TO / FROM arrow; the bearing pointer to the
/// steerpoint (cyan); the steerpoint, distance and ETA left, bearing, course and ground speed
/// right.
pub fn draw_hsi_page(p: &MfdParams, c: &mut Canvas, fonts: &Fonts) {
    c.clear(BLACK);
    let Ok((a, pos)) = p.aircraft.single() else { return };
    let heading = a.euler().0;
    let (e, n, _) = bevy_to_wu(pos.0);
    let own = DVec2::new(e, n);
    let wps = &p.waypoints;
    let steer = wps.target().map(|w| DVec2::new(w.east, w.north));
    // Compass bearing (radians clockwise from north) of a world vector (east, north).
    let compass = |v: DVec2| v.x.atan2(v.y) as f32;
    let bearing = steer.map(|w| compass(w - own));
    let from = match wps.current {
        0 => None,
        i => wps.points.get(i - 1).map(|w| DVec2::new(w.east, w.north)),
    };
    let course_dir = steer.zip(from).map(|(w, f)| w - f).filter(|d| d.length() > 1.0);
    let course = course_dir.map(compass).or(bearing);

    let o = Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0 + 10.0);
    let r = c.h as f32 / 2.0 - BAND as f32 - 14.0;
    // Up the display = the nose.
    let dir = |b: f32| Vec2::new((b - heading).sin(), -(b - heading).cos());
    // The rose: a tick every 5 degrees, longer every 10, labelled every 30.
    for k in 0..72 {
        let b = (k as f32 * 5.0).to_radians();
        let len = if k % 2 == 0 { 7.0 } else { 4.0 };
        c.stroke(o + dir(b) * r, o + dir(b) * (r - len), 1.2, WHITE);
        if k % 6 == 0 {
            let label = match k * 5 {
                0 => "N".to_string(),
                90 => "E".to_string(),
                180 => "S".to_string(),
                270 => "W".to_string(),
                d => (d / 10).to_string(),
            };
            let q = o + dir(b) * (r - 16.0);
            stroke_text_at(c, q.x, q.y, &label, 10.0, 1.3, WHITE, Align::Centre);
        }
    }
    // Fixed index marks every 45 degrees outside the rose, the lubber line at the top.
    for k in 1..8 {
        let d =
            Vec2::from_angle(k as f32 * std::f32::consts::FRAC_PI_4 - std::f32::consts::FRAC_PI_2);
        c.stroke(o + d * (r + 2.0), o + d * (r + 7.0), 1.2, GREY);
    }
    c.triangle(
        [o - Vec2::Y * (r + 1.0), o + Vec2::new(-5.0, -r - 9.0), o + Vec2::new(5.0, -r - 9.0)],
        WHITE,
    );
    // Digital heading in a box over the rose.
    let hdg = format!("{:03}", (heading.to_degrees().rem_euclid(360.0).round() as i32) % 360);
    let w = 2 * fonts.width(hdg.as_bytes()) - 2;
    let top = (o.y - r) as i32 - 24;
    c.frame(
        Vec2::new(o.x - (w / 2 + 4) as f32, top as f32),
        Vec2::new(o.x + (w / 2 + 4) as f32, top as f32 + 14.0),
        1.2,
        WHITE,
    );
    fonts.text_scaled(c, o.x as i32 - w / 2, top + 3, hdg.as_bytes(), WHITE, 2);

    // Course arrow with its deviation bar and dots, the TO / FROM arrow.
    if let (Some(crs), Some(w)) = (course, steer) {
        let d = dir(crs);
        let side = Vec2::new(-d.y, d.x);
        let unit =
            course_dir.map_or(DVec2::new(crs.sin() as f64, crs.cos() as f64), |v| v.normalize());
        // Off the course to its right (km): the course is then left of the aircraft.
        let off_km = (own - w).dot(DVec2::new(unit.y, -unit.x)) * WU / 1000.0;
        let dot = r * 0.17;
        for k in [-2.0, -1.0, 1.0, 2.0] {
            c.ring(o + side * dot * k, 2.0, 1.0, GREY);
        }
        let head = o + d * (r - 20.0);
        c.stroke(o + d * (r * 0.45), head, 1.8, GREEN);
        c.triangle([head + d * 9.0, head + side * 5.0, head - side * 5.0], GREEN);
        c.stroke(o - d * (r * 0.45), o - d * (r - 18.0), 1.8, GREEN);
        let dev = (-off_km / HSI_FULL_SCALE_KM).clamp(-1.0, 1.0) as f32 * 2.0 * dot;
        c.stroke(o + side * dev - d * (r * 0.4), o + side * dev + d * (r * 0.4), 1.8, GREEN);
        let to = (w - own).dot(unit) > 0.0;
        let t = if to { d } else { -d };
        let tip = o + t * (r * 0.32) + side * (r * 0.22);
        c.triangle([tip, tip - t * 8.0 + side * 4.0, tip - t * 8.0 - side * 4.0], GREEN);
    }
    // Bearing pointer to the steerpoint: head at the rose, tail opposite.
    if let Some(b) = bearing {
        let d = dir(b);
        let side = Vec2::new(-d.y, d.x);
        let head = o + d * (r - 1.0);
        c.stroke(o + d * (r - 26.0), head - d * 6.0, 1.6, CYAN);
        c.triangle([head, head - d * 10.0 + side * 5.0, head - d * 10.0 - side * 5.0], CYAN);
        c.stroke(o - d * (r - 26.0), o - d * (r - 2.0), 1.6, CYAN);
    }
    // Aircraft symbol, fixed in the middle.
    c.stroke(o - Vec2::X * 9.0, o + Vec2::X * 9.0, 1.8, WHITE);
    c.stroke(o - Vec2::Y * 6.0, o + Vec2::Y * 9.0, 1.8, WHITE);
    c.stroke(o + Vec2::new(-4.0, 8.0), o + Vec2::new(4.0, 8.0), 1.8, WHITE);

    // Readouts: a caption over each value, left and right of the rose.
    let km = steer.map(|w| ((w - own).length() * WU / 1000.0) as f32);
    let t = p.clock.start_seconds + p.clock.elapsed.unwrap_or(a.time);
    let ms = a.knots() * 0.5144;
    let deg = |b: Option<f32>| {
        b.map_or("---".to_string(), |b| {
            format!("{:03}", (b.to_degrees().rem_euclid(360.0).round() as i32) % 360)
        })
    };
    let left = [
        ("STPT", if steer.is_some() { format!("{}", wps.current + 1) } else { "-".to_string() }),
        (
            "DIS",
            km.map_or("---".to_string(), |k| {
                if k < 100.0 { format!("{k:.1}") } else { format!("{k:.0}") }
            }),
        ),
        (
            "ETA",
            km.filter(|_| ms > 1.0).map_or("--:--".to_string(), |k| {
                let (h, m, _) = crate::mfd::hms(t + k * 1000.0 / ms * p.clock.rate);
                format!("{h:02}:{m:02}")
            }),
        ),
    ];
    let right = [("BRG", deg(bearing)), ("CRS", deg(course)), ("GS", format!("{:.0}", a.knots()))];
    for (row, ((lc, lv), (rc, rv))) in left.iter().zip(&right).enumerate() {
        let y = BAND + 8 + row as i32 * 46;
        for (x, caption, value, align_right) in [(4, lc, lv, false), (c.w as i32 - 4, rc, rv, true)]
        {
            for (dy, text, colour) in [(0, caption.to_string(), GREY), (12, value.clone(), GREEN)] {
                let text = upper(&text);
                let w = 2 * fonts.width(&text) - 2;
                fonts.text_scaled(c, if align_right { x - w } else { x }, y + dy, &text, colour, 2);
            }
        }
    }
}

/// The SOI's cursor: two short vertical lines.
fn draw_cursor(c: &mut Canvas, p: Vec2) {
    for dx in [-3.0, 3.0] {
        c.stroke(p + Vec2::new(dx, -6.0), p + Vec2::new(dx, 6.0), 1.5, WHITE);
    }
}

/// The HSD's projection: heading up, the aircraft low in the display (or centred).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HsdView {
    /// Aircraft position (wu east, north), heading (radians clockwise from north).
    pub own: DVec2,
    pub heading: f64,
    /// Aircraft on the display and display pixels per world unit.
    pub origin: Vec2,
    pub scale: f64,
    /// Outer ring radius (pixels).
    pub radius: f32,
}

impl HsdView {
    pub fn new(own: DVec2, heading: f64, range_km: f32, centred: bool, size: Vec2) -> Self {
        let (origin, radius) = if centred {
            let o = Vec2::new(size.x / 2.0, size.y / 2.0);
            (o, (size.y / 2.0 - BAND as f32 - 6.0).min(size.x / 2.0 - 6.0))
        } else {
            let o = Vec2::new(size.x / 2.0, size.y * 0.72);
            (o, o.y - BAND as f32 - 6.0)
        };
        let scale = radius as f64 / (range_km as f64 * 1000.0 / WU);
        Self { own, heading, origin, scale, radius }
    }

    pub fn to_screen(self, east: f64, north: f64) -> Vec2 {
        let (de, dn) = (east - self.own.x, north - self.own.y);
        let (s, c) = self.heading.sin_cos();
        let right = de * c - dn * s;
        let ahead = de * s + dn * c;
        self.origin + Vec2::new((right * self.scale) as f32, (-ahead * self.scale) as f32)
    }

    /// The ground map's scope: heading up, the aircraft at the apex low in the display, the
    /// outer arc `SCAN_HALF` either side of the nose fitting the display's width.
    pub fn wedge(own: DVec2, heading: f64, range_km: f32, size: Vec2) -> Self {
        let o = Vec2::new(size.x / 2.0, size.y - BAND as f32 - 8.0);
        let side = (size.x / 2.0 - 6.0) / crate::ground_radar::SCAN_HALF.sin();
        let radius = side.min(o.y - BAND as f32 - 10.0);
        let scale = radius as f64 / (range_km as f64 * 1000.0 / WU);
        Self { own, heading, origin: o, scale, radius }
    }

    /// This view magnified `zoom` times around display point `cursor`, which lands on `at`.
    pub fn zoomed(self, cursor: Vec2, at: Vec2, zoom: f32) -> Self {
        Self {
            origin: at + (self.origin - cursor) * zoom,
            scale: self.scale * zoom as f64,
            radius: self.radius * zoom,
            ..self
        }
    }

    pub fn to_world(self, p: Vec2) -> DVec2 {
        let d = (p - self.origin).as_dvec2() / self.scale;
        let (right, ahead) = (d.x, -d.y);
        let (s, c) = self.heading.sin_cos();
        self.own + DVec2::new(right * c + ahead * s, -right * s + ahead * c)
    }

    /// Direction on the display of a heading (radians clockwise from north).
    pub fn dir(self, h: f64) -> Vec2 {
        let a = h - self.heading;
        Vec2::new(a.sin() as f32, -a.cos() as f32)
    }
}

/// The HSD view for the aircraft (None without one).
pub fn hsd_view(av: &AvionicsState, ac: Option<(&Aircraft, &WorldPos)>) -> Option<HsdView> {
    let (a, pos) = ac?;
    let (e, n, _) = bevy_to_wu(pos.0);
    let size = Vec2::new(HIRES.0 as f32, HIRES.1 as f32);
    Some(HsdView::new(
        DVec2::new(e, n),
        a.euler().0 as f64,
        HSD_RANGES[av.hsd_range],
        av.hsd_centred,
        size,
    ))
}

/// The ground map's view for the aircraft (None without one).
pub fn gm_view(av: &AvionicsState, ac: Option<(&Aircraft, &WorldPos)>) -> Option<HsdView> {
    let (a, pos) = ac?;
    let (e, n, _) = bevy_to_wu(pos.0);
    let size = Vec2::new(HIRES.0 as f32, HIRES.1 as f32);
    Some(HsdView::wedge(DVec2::new(e, n), a.euler().0 as f64, av.gm_range_km(), size))
}

/// The HARM display's scope: centre and radius.
fn harm_scope() -> (Vec2, f32) {
    let (w, h) = (HIRES.0 as f32, HIRES.1 as f32);
    (Vec2::new(w / 2.0, h / 2.0), h / 2.0 - BAND as f32 - 4.0)
}

fn draw_hsd(p: &MfdParams, c: &mut Canvas, ac: Option<(&Aircraft, &WorldPos)>, soi: bool) {
    c.clear(BLACK);
    let av = &p.state.av;
    let Some(view) = hsd_view(av, ac) else { return };
    hsd_picture(p, c, view);
    if soi {
        draw_cursor(c, av.hsd_cursor);
        if av.zoom_window() {
            draw_zoom_window(c, av.hsd_cursor, BLACK, |c, at| {
                hsd_picture(p, c, view.zoomed(av.hsd_cursor, at, ZOOM))
            });
        }
    }
}

/// The ground map: the radar's picture (the canvas left at index 0 over it, `backdrop`) in the
/// scope, its range arcs and azimuth marks, the beam's position, the steerpoints, where the pod
/// looks and the locked target, the cursor as the SOI (and the zoomed window while the lock is
/// held).
fn draw_gm(p: &MfdParams, c: &mut Canvas, ac: Option<(&Aircraft, &WorldPos)>, soi: bool) {
    let av = &p.state.av;
    let (Some(view), Some(radar)) = (gm_view(av, ac), p.radar.as_deref()) else {
        c.clear(BLACK);
        return;
    };
    c.clear(0);
    let o = view.origin;
    let r = view.radius;
    let half = crate::ground_radar::SCAN_HALF;
    let at = |az: f32, k: f32| o + Vec2::new(az.sin(), -az.cos()) * r * k;
    // The arcs are drawn from the display's +x axis, clockwise (y down).
    let (a0, a1) = (-std::f32::consts::FRAC_PI_2 - half, -std::f32::consts::FRAC_PI_2 + half);
    c.arc(o, r, a0, a1, 1.0, GREY);
    for k in [0.25, 0.5, 0.75] {
        let steps = 16;
        for i in (0..steps).step_by(2) {
            let t = |i: i32| a0 + (a1 - a0) * i as f32 / steps as f32;
            c.arc(o, r * k, t(i), t(i + 1), 1.0, DARK);
        }
    }
    for az in [-half, half] {
        c.stroke(o, at(az, 1.0), 1.0, GREY);
    }
    for deg in [-30.0f32, 0.0, 30.0] {
        let az = deg.to_radians();
        c.stroke(at(az, 1.0), at(az, 1.0) + Vec2::new(az.sin(), -az.cos()) * 5.0, 1.0, GREY);
    }
    // The beam: a caret over the outer arc.
    if radar.on {
        let q = at(radar.beam, 1.0);
        let d = Vec2::new(radar.beam.sin(), -radar.beam.cos());
        let side = Vec2::new(-d.y, d.x);
        c.triangle([q + d * 2.0, q + d * 8.0 + side * 4.0, q + d * 8.0 - side * 4.0], GREEN);
    } else {
        let label = if av.radar_standby { "STBY" } else { "NO RAD" };
        stroke_text_at(c, o.x, o.y - r * 0.5, label, 10.0, 1.4, GREEN, Align::Centre);
    }
    gm_symbols(p, c, view);
    if soi {
        draw_cursor(c, av.gm_cursor);
        // The radar's picture in the window is `gm_backdrop`'s.
        if av.zoom_window() {
            let cursor = av.gm_cursor;
            draw_zoom_window(c, cursor, 0, |c, at| gm_symbols(p, c, view.zoomed(cursor, at, ZOOM)));
        }
    }
}

/// The ground map's steerpoints, locked target, pod and own aircraft through `view`.
fn gm_symbols(p: &MfdParams, c: &mut Canvas, view: HsdView) {
    let av = &p.state.av;
    let (o, r) = (view.origin, view.radius);
    let half = crate::ground_radar::SCAN_HALF;
    // The steerpoints (the current one ringed).
    let in_scope = |q: Vec2| {
        let d = q - o;
        d.length() <= r && d.y < 0.0 && d.x.abs() <= -d.y * half.tan()
    };
    for (i, w) in p.waypoints.points.iter().enumerate() {
        let q = view.to_screen(w.east, w.north);
        if !in_scope(q) {
            continue;
        }
        c.frame(q - Vec2::splat(2.5), q + Vec2::splat(2.5), 1.0, WHITE);
        if i == p.waypoints.current {
            c.ring(q, 6.0, 1.2, WHITE);
        }
    }
    // The locked ground target (an object, a point, a vehicle), boxed.
    if let Some((e, n)) = p.contacts.ground_lock {
        let q = view.to_screen(e, n);
        if in_scope(q) {
            c.frame(q - Vec2::splat(7.0), q + Vec2::splat(7.0), 1.3, WHITE);
        }
    }
    // Where the pod looks: a diamond.
    if let Some(g) = av.tgp.los_ground.filter(|_| av.shown(Format::Tgp).is_some()) {
        let (e, n, _) = bevy_to_wu(g);
        let q = view.to_screen(e, n);
        if in_scope(q) {
            c.polyline(
                &[
                    q + Vec2::new(0.0, -5.0),
                    q + Vec2::new(5.0, 0.0),
                    q + Vec2::new(0.0, 5.0),
                    q + Vec2::new(-5.0, 0.0),
                    q + Vec2::new(0.0, -5.0),
                ],
                1.3,
                CYAN,
            );
        }
    }
    // Own aircraft at the apex.
    c.triangle([o + Vec2::new(0.0, -5.0), o + Vec2::new(-4.0, 3.0), o + Vec2::new(4.0, 3.0)], CYAN);
}

/// The A-A radar's B-scope: its top left and bottom right corners on the display (azimuth
/// across, `aa_radar::GIMBAL` either side of the nose; range up from the bottom), clear of the
/// side labels and the lock's readouts on top.
pub fn fcr_scope() -> (Vec2, Vec2) {
    let (w, h) = (HIRES.0 as f32, HIRES.1 as f32);
    (Vec2::new(36.0, BAND as f32 + 16.0), Vec2::new(w - 36.0, h - BAND as f32 - 9.0))
}

/// A point on the B-scope: azimuth off the nose (radians) and range as a fraction of the
/// display's.
fn fcr_point(az: f32, k: f32) -> Vec2 {
    let (lo, hi) = fcr_scope();
    let x = lo.x + (az / crate::aa_radar::GIMBAL + 1.0) / 2.0 * (hi.x - lo.x);
    Vec2::new(x, hi.y - k * (hi.y - lo.y))
}

/// Feet per metre.
const FT: f32 = 3.28084;
/// Knots per metre per second.
const KT: f32 = 1.943_84;

/// The A-A radar (after the F-16's FCR in RWS): the B-scope's azimuth and range ticks, the
/// scan's width, the artificial horizon, the antenna's azimuth and elevation carets, the search
/// bricks (old ones dimmer, friendly ones green), the locked aircraft with its heading and
/// altitude and its aspect, heading, closure and speed on top, and the cursor with the altitudes
/// the scan covers at its range as the SOI (and the zoomed window while the lock is held).
fn draw_fcr(p: &MfdParams, c: &mut Canvas, ac: Option<(&Aircraft, &WorldPos)>, soi: bool) {
    use crate::aa_radar::GIMBAL;
    c.clear(BLACK);
    let av = &p.state.av;
    let (lo, hi) = fcr_scope();
    let range = av.fcr_range_m();
    // Range ticks down the sides and at 30 degrees, azimuth ticks along the bottom.
    for k in [0.25, 0.5, 0.75] {
        let y = fcr_point(0.0, k).y;
        c.stroke(Vec2::new(lo.x, y), Vec2::new(lo.x + 4.0, y), 1.0, GREY);
        c.stroke(Vec2::new(hi.x - 4.0, y), Vec2::new(hi.x, y), 1.0, GREY);
        for deg in [-30.0f32, 30.0] {
            let q = fcr_point(deg.to_radians(), k);
            c.stroke(q - Vec2::X * 2.0, q + Vec2::X * 2.0, 1.0, DARK);
        }
    }
    for deg in [-60.0f32, -30.0, 0.0, 30.0, 60.0] {
        let x = fcr_point(deg.to_radians(), 0.0).x;
        c.stroke(Vec2::new(x, hi.y), Vec2::new(x, hi.y - 5.0), 1.0, GREY);
    }
    // The scan's edges inside the gimbals.
    let scan = av.fcr_scan();
    if scan.az_half < GIMBAL - 1e-3 {
        for az in [-scan.az_half, scan.az_half] {
            let x = fcr_point(az, 0.0).x;
            c.dashed(Vec2::new(x, lo.y), Vec2::new(x, hi.y), 3.0, 4.0, 1.0, DARK);
        }
    }
    let Some((a, pos)) = ac else { return };
    // The artificial horizon: banked with the aircraft, down as the nose comes up.
    let (_, pitch, roll) = a.euler();
    let mid = (lo + hi) / 2.0;
    let o = mid + Vec2::Y * (pitch / GIMBAL).clamp(-1.0, 1.0) * (hi.y - lo.y) / 2.0;
    let d = Vec2::new(roll.cos(), -roll.sin());
    let span = (hi.x - lo.x) * 0.42;
    for s in [-1.0, 1.0] {
        c.stroke(o + d * s * 12.0, o + d * s * span, 1.0, GREY);
    }
    let radar = p.aa_radar.as_deref().filter(|r| r.on);
    let Some(radar) = radar else {
        let label = if av.radar_standby { "STBY" } else { "NO RAD" };
        stroke_text_at(c, mid.x, mid.y - 20.0, label, 10.0, 1.4, GREEN, Align::Centre);
        if soi {
            draw_cursor(c, av.fcr_cursor);
        }
        return;
    };
    // The antenna: its azimuth under the scope, its elevation left of it.
    let x = fcr_point(radar.beam, 0.0).x;
    c.stroke(Vec2::new(x, hi.y + 2.0), Vec2::new(x, hi.y + 7.0), 1.5, WHITE);
    let y = mid.y - (radar.elevation / GIMBAL).clamp(-1.0, 1.0) * (hi.y - lo.y) / 2.0;
    c.stroke(Vec2::new(lo.x - 5.0, y), Vec2::new(lo.x - 1.0, y), 1.5, WHITE);
    fcr_contacts(c, radar, range, fcr_point);
    // The locked aircraft's aspect (tens of degrees off its tail, and our side of its nose),
    // heading, closure and ground speed (knots) over the scope.
    if let Some(t) = radar.track {
        let y = BAND as f32 + 8.0;
        let aspect = (t.aspect.to_degrees() / 10.0).round() as i32;
        let side = if aspect == 0 || aspect == 18 {
            ""
        } else if t.right {
            "R"
        } else {
            "L"
        };
        let heading = (t.heading.to_degrees().rem_euclid(360.0).round() as i32 + 359) % 360 + 1;
        let left = format!("{aspect}{side} {heading:03}");
        stroke_text_at(c, lo.x + 10.0, y, &left, 7.0, 1.0, WHITE, Align::Left);
        let closure = (t.closure * KT).round() as i32;
        let right = format!("{closure:+} {}", (t.speed * KT).round() as i32);
        stroke_text_at(c, hi.x - 10.0, y, &right, 7.0, 1.0, WHITE, Align::Right);
    }
    if soi {
        let cursor = av.fcr_cursor;
        draw_cursor(c, cursor);
        if av.zoom_window() {
            draw_zoom_window(c, cursor, BLACK, |c, at| {
                let (lo, hi) = fcr_scope();
                let edge = |p: Vec2| at + (p - cursor) * ZOOM;
                c.frame(edge(lo), edge(hi), 1.0, GREY);
                fcr_contacts(c, radar, range, |az, k| edge(fcr_point(az, k)));
            });
        }
        // The altitudes the scan covers at the cursor's range (thousands of feet).
        let r = ((hi.y - cursor.y) / (hi.y - lo.y)).clamp(0.0, 1.0) * range;
        let (low, high) = scan.coverage();
        let alt = |el: f32| ((pos.0.y as f32 + r * el.tan()) * FT / 1000.0).round() as i32;
        let at = cursor + Vec2::new(6.0, 0.0);
        stroke_text_at(c, at.x, at.y - 5.0, &alt(high).to_string(), 6.0, 1.0, WHITE, Align::Left);
        stroke_text_at(c, at.x, at.y + 5.0, &alt(low).to_string(), 6.0, 1.0, WHITE, Align::Left);
    }
}

/// The A-A radar's search bricks (old ones dimmer, friendly ones green) and the locked
/// aircraft (a filled square with its heading and its altitude in thousands of feet under it),
/// placed by `at` (azimuth, range as a fraction of the display's).
fn fcr_contacts(
    c: &mut Canvas,
    radar: &crate::aa_radar::AaRadar,
    range: f32,
    at: impl Fn(f32, f32) -> Vec2,
) {
    use crate::aa_radar::GIMBAL;
    for b in &radar.bricks {
        let k = b.range / range;
        if k > 1.0 || b.az.abs() > GIMBAL {
            continue;
        }
        let q = at(b.az, k);
        let colour = if b.age > radar.frame {
            GREY
        } else if b.friendly {
            GREEN
        } else {
            WHITE
        };
        c.fill_rect(q.x as i32 - 3, q.y as i32 - 1, 6, 3, colour);
    }
    if let Some(t) = radar.track.filter(|t| t.range <= range) {
        let q = at(t.az, t.range / range);
        c.fill_rect(q.x as i32 - 3, q.y as i32 - 3, 7, 7, WHITE);
        let d = Vec2::new(t.rel_heading.sin(), -t.rel_heading.cos());
        c.stroke(q + d * 5.0, q + d * 14.0, 1.3, WHITE);
        let alt = format!("{}", (t.alt * FT / 1000.0).round() as i32);
        stroke_text_at(c, q.x, q.y + 11.0, &alt, 7.0, 1.0, WHITE, Align::Centre);
    }
}

/// The ground map's picture for an MFD showing it (None for the others): `prev` refilled when
/// it is the right size and not shared, `pw` by `ph` texels of `scale` display pixels.
pub fn gm_backdrop(
    p: &MfdParams,
    side: MfdSide,
    prev: Option<std::sync::Arc<crate::canvas::Picture>>,
    pw: usize,
    ph: usize,
    scale: f32,
) -> Option<std::sync::Arc<crate::canvas::Picture>> {
    let av = &p.state.av;
    if !p.state.modern || av.format(side) != Format::Gm {
        return None;
    }
    let view = gm_view(av, p.aircraft.single().ok())?;
    let radar = p.radar.as_deref()?;
    let mut pic = prev.filter(|pic| pic.w == pw && pic.h == ph).unwrap_or_else(|| {
        std::sync::Arc::new(crate::canvas::Picture { w: pw, h: ph, rgba: vec![0; pw * ph * 4] })
    });
    let pic_mut = match std::sync::Arc::get_mut(&mut pic) {
        Some(m) => m,
        None => {
            pic = std::sync::Arc::new(crate::canvas::Picture {
                w: pw,
                h: ph,
                rgba: vec![0; pw * ph * 4],
            });
            std::sync::Arc::get_mut(&mut pic).unwrap()
        }
    };
    radar.draw(&view, scale, pic_mut);
    // The zoomed window's picture, inside its frame.
    if av.soi_format() == Some(Format::Gm) && av.zoom_window() {
        let cursor = av.gm_cursor;
        let (a, b) = zoom_window_rect(Vec2::new(HIRES.0 as f32, HIRES.1 as f32), cursor);
        let texel = |v: f32| (v * scale).round().max(0.0) as usize;
        let rect = (texel(a.x + 1.0), texel(a.y + 1.0), texel(b.x - 1.0), texel(b.y - 1.0));
        radar.draw_in(&view.zoomed(cursor, (a + b) / 2.0, ZOOM), scale, pic_mut, rect);
    }
    Some(pic)
}

/// Everything on the HSD but the cursor, through `view`.
fn hsd_picture(p: &MfdParams, c: &mut Canvas, view: HsdView) {
    let av = &p.state.av;
    let o = view.origin;
    // Range rings (the inner ones dashed) and the compass rose on the outer one.
    for k in 1..3 {
        let r = view.radius * k as f32 / 3.0;
        let steps = 36;
        for i in (0..steps).step_by(2) {
            let a0 = i as f32 / steps as f32 * std::f32::consts::TAU;
            let a1 = (i + 1) as f32 / steps as f32 * std::f32::consts::TAU;
            c.arc(o, r, a0, a1, 1.0, DARK);
        }
    }
    c.ring(o, view.radius, 1.0, GREY);
    for k in 0..36 {
        let h = (k as f64 * 10.0).to_radians();
        let d = view.dir(h);
        let len = if k % 3 == 0 { 7.0 } else { 4.0 };
        c.stroke(o + d * view.radius, o + d * (view.radius - len), 1.0, GREY);
        if k % 9 == 0 {
            let label = ["N", "E", "S", "W"][k / 9];
            let q = o + d * (view.radius - 14.0);
            stroke_text_at(c, q.x, q.y, label, 9.0, 1.2, GREY, Align::Centre);
        }
    }
    // Route: the steerpoints as small squares, the current one ringed, and a leg from the
    // aircraft to the current one so the route shows with every steerpoint out of range.
    let wps: Vec<Vec2> =
        p.waypoints.points.iter().map(|w| view.to_screen(w.east, w.north)).collect();
    if let Some(&q) = wps.get(p.waypoints.current) {
        c.stroke(o, q, 1.0, WHITE);
    }
    for pair in wps.windows(2) {
        c.stroke(pair[0], pair[1], 1.0, WHITE);
    }
    for (i, &q) in wps.iter().enumerate() {
        c.frame(q - Vec2::splat(2.5), q + Vec2::splat(2.5), 1.0, WHITE);
        if i == p.waypoints.current {
            c.ring(q, 6.0, 1.2, WHITE);
        }
    }
    // Threat rings, ground targets, aircraft, missiles.
    let px_per_km = (view.scale * 1000.0 / WU) as f32;
    for k in p.contacts.shown() {
        let q = view.to_screen(k.east, k.north);
        match k.kind {
            ContactKind::GroundRadar => {
                let colour = if k.friendly { GREEN } else { RED };
                if let Some(t) = k.threat {
                    if t.ring {
                        let ring = if k.friendly { GREEN } else { YELLOW };
                        threat_ring(c, q, t.km * px_per_km, t.doppler, ring);
                    }
                    let text = String::from_utf8_lossy(&t.label).trim_end_matches('\0').to_string();
                    stroke_text_at(c, q.x, q.y, &text, 9.0, 1.3, colour, Align::Centre);
                } else {
                    c.fill_rect(q.x as i32 - 2, q.y as i32 - 2, 4, 4, colour);
                }
            }
            ContactKind::Ship => {
                // A hull with its superstructure.
                let colour = if k.friendly { GREEN } else { RED };
                c.polyline(
                    &[
                        q + Vec2::new(-7.0, -1.0),
                        q + Vec2::new(7.0, -1.0),
                        q + Vec2::new(4.0, 3.0),
                        q + Vec2::new(-4.0, 3.0),
                        q + Vec2::new(-7.0, -1.0),
                    ],
                    1.3,
                    colour,
                );
                c.polyline(
                    &[
                        q + Vec2::new(-2.0, -1.0),
                        q + Vec2::new(-2.0, -4.0),
                        q + Vec2::new(2.0, -4.0),
                        q + Vec2::new(2.0, -1.0),
                    ],
                    1.3,
                    colour,
                );
                if let Some(t) = k.threat {
                    threat_ring(c, q, t.km * px_per_km, t.doppler, YELLOW);
                }
            }
            ContactKind::Target => {
                c.polyline(
                    &[
                        q + Vec2::new(0.0, -5.0),
                        q + Vec2::new(5.0, 0.0),
                        q + Vec2::new(0.0, 5.0),
                        q + Vec2::new(-5.0, 0.0),
                        q + Vec2::new(0.0, -5.0),
                    ],
                    1.3,
                    CYAN,
                );
            }
            ContactKind::Aircraft => {
                let d = view.dir(k.heading as f64 / 65536.0 * std::f64::consts::TAU);
                if k.friendly {
                    c.ring(q, 4.0, 1.3, GREEN);
                    c.stroke(q + d * 4.0, q + d * 9.0, 1.3, GREEN);
                } else {
                    let side = Vec2::new(-d.y, d.x);
                    c.polyline(
                        &[
                            q + d * 5.0,
                            q - d * 3.0 + side * 4.0,
                            q - d * 3.0 - side * 4.0,
                            q + d * 5.0,
                        ],
                        1.3,
                        RED,
                    );
                    c.stroke(q + d * 5.0, q + d * 10.0, 1.3, RED);
                }
            }
            ContactKind::Drone => {
                c.stroke(q - Vec2::splat(3.0), q + Vec2::splat(3.0), 1.2, RED);
                c.stroke(q + Vec2::new(-3.0, 3.0), q + Vec2::new(3.0, -3.0), 1.2, RED);
            }
            ContactKind::Missile => {
                let d = view.dir(k.heading as f64 / 65536.0 * std::f64::consts::TAU);
                let col = if k.own {
                    BLUE
                } else if k.friendly {
                    GREEN
                } else {
                    RED
                };
                c.stroke(q - d * 3.0, q + d * 3.0, 1.5, col);
            }
        }
        if k.target.is_some() && k.target == p.contacts.locked {
            c.frame(q - Vec2::splat(8.0), q + Vec2::splat(8.0), 1.3, WHITE);
        }
    }
    // Where the targeting pod looks: a diamond on its ground point, a dashed line to it.
    if let Some(g) = av.tgp.los_ground.filter(|_| av.shown(Format::Tgp).is_some()) {
        let (e, n, _) = bevy_to_wu(g);
        let q = view.to_screen(e, n);
        c.dashed(o, q, 4.0, 3.0, 1.0, GREEN);
        c.polyline(
            &[
                q + Vec2::new(0.0, -6.0),
                q + Vec2::new(6.0, 0.0),
                q + Vec2::new(0.0, 6.0),
                q + Vec2::new(-6.0, 0.0),
                q + Vec2::new(0.0, -6.0),
            ],
            1.5,
            GREEN,
        );
    }
    // Own aircraft.
    c.stroke(o + Vec2::new(0.0, -6.0), o + Vec2::new(0.0, 6.0), 1.6, CYAN);
    c.stroke(o + Vec2::new(-6.0, -1.0), o + Vec2::new(6.0, -1.0), 1.6, CYAN);
    c.stroke(o + Vec2::new(-3.0, 5.0), o + Vec2::new(3.0, 5.0), 1.6, CYAN);
}

/// A threat ring: solid around a Doppler radar, dotted around a pulse one.
fn threat_ring(c: &mut Canvas, centre: Vec2, r: f32, doppler: bool, colour: u8) {
    if doppler {
        c.ring(centre, r, 1.0, colour);
        return;
    }
    if !c.ring_shows(centre, r, 1.0) {
        return;
    }
    // Dots about 2 px long, 5 px apart (those on the screen).
    let n = ((r * std::f32::consts::TAU / 5.0) as i32).clamp(12, 720);
    let step = std::f32::consts::TAU / n as f32;
    let dot = (2.0 / r.max(1.0)).min(step * 0.5);
    for i in 0..n {
        let a = i as f32 * step;
        if c.shows(centre + Vec2::from_angle(a) * r, 3.0) {
            c.arc(centre, r, a, a + dot, 1.0, colour);
        }
    }
}

/// HARM display: the radars painting the aircraft (filtered), placed by danger like the RWR.
fn draw_harm(p: &MfdParams, c: &mut Canvas, soi: bool) {
    c.clear(BLACK);
    let av = &p.state.av;
    let (centre, r) = harm_scope();
    harm_picture(p, c, centre, r);
    if p.emitters.list.iter().all(|e| !harm_shows(av.harm_filter, e)) {
        stroke_text_at(
            c,
            centre.x,
            centre.y + r * 0.5,
            "NO EMITTERS",
            8.0,
            1.0,
            NAVY.max(DARK),
            Align::Centre,
        );
    }
    if soi {
        draw_cursor(c, av.harm_cursor);
        if av.zoom_window() {
            let cursor = av.harm_cursor;
            draw_zoom_window(c, cursor, BLACK, |c, at| {
                harm_picture(p, c, at + (centre - cursor) * ZOOM, r * ZOOM)
            });
        }
    }
}

/// The HARM scope and its symbols, centred at `centre` with radius `r`.
fn harm_picture(p: &MfdParams, c: &mut Canvas, centre: Vec2, r: f32) {
    let av = &p.state.av;
    c.ring(centre, r, 1.0, GREY);
    for f in [0.33, 0.66] {
        c.ring(centre, r * f, 1.0, DARK);
    }
    for k in 0..12 {
        let a = k as f32 * std::f32::consts::TAU / 12.0;
        let d = Vec2::new(a.sin(), -a.cos());
        c.stroke(centre + d * r, centre + d * (r - 5.0), 1.0, GREY);
    }
    c.stroke(centre - Vec2::X * 4.0, centre + Vec2::X * 4.0, 1.0, GREY);
    c.stroke(centre - Vec2::Y * 4.0, centre + Vec2::Y * 4.0, 1.0, GREY);
    let locked = p.contacts.locked.and_then(emitter_source);
    let filter = av.harm_filter;
    crate::rwr::draw_symbols(
        c,
        &p.emitters.list,
        centre,
        r,
        crate::hud::blink(&p.time),
        locked,
        false,
        |e| harm_shows(filter, e),
    );
}

/// The RCS page: the threat picture of the RAH-66's and F-22A's multipurpose display
/// (`crate::mpd`), full size.
fn draw_rcs_page(p: &MfdParams, c: &mut Canvas) {
    let a = p.aircraft.single().ok().map(|(a, _)| a);
    let (_, pitch, roll) = a.map_or((0.0, 0.0, 0.0), |a| a.euler());
    let th = crate::mpd::Threats {
        emitters: &p.emitters.list,
        emv: &p.emv,
        history: &p.threats,
        bay_open: a.is_some_and(|a| a.bay_open),
        gear_down: a.is_some_and(|a| a.gear_down),
        pitch,
        roll,
    };
    draw_rcs(c, &th, p.state.av.rcs_range, crate::hud::blink(&p.time));
}

/// The RCS page's scope (`crate::mpd::draw_scope`, its range `RCS_RANGES`' `range` entry):
/// the bloom of our radar return among the radars painting us, with the dotted rings where the
/// strongest ground and air radar signal would see it. Right of it the bloom's shape magnified
/// (ASPECT: the parts past the detection ring amber, ticks where the gear, the bays, a bank or
/// a pitch swell it) over the gear, bays, bank and pitch, the signature (RCS), the strongest
/// signal (SIG) and the margin between them (MRG, negative when seen); under them the most
/// dangerous radar, and the last 30 seconds as a strip chart with its time axis.
pub(crate) fn draw_rcs(c: &mut Canvas, th: &crate::mpd::Threats, range: usize, blink: bool) {
    use crate::mpd::{ScopeStyle, bloom, bloom_size, dotted_ring, fill_bloom, pct, pulse_of};
    use std::f32::consts::TAU;
    c.clear(BLACK);
    let range = match RCS_RANGES[range.min(RCS_RANGES.len() - 1)] {
        0.0 => crate::mpd::scope_range(th.emitters),
        km => km,
    };
    let text = |c: &mut Canvas, x: f32, y: f32, s: &str, h: f32, col: u8| {
        stroke_text(c, x, y, s, h, 1.0, col)
    };
    let st = ScopeStyle {
        centre: Vec2::new(80.0, 72.0),
        r: 50.0,
        thin: 0.7,
        pen: 0.8,
        bold: 1.0,
        sym: 3.0,
        received_only: true,
    };
    let (centre, r) = (st.centre, st.r);
    let at = |b: f32, f: f32| centre + Vec2::new(b.sin(), -b.cos()) * r * f;
    crate::mpd::draw_scope(c, th, &st, range, blink, |c, p, label, col| {
        stroke_text(c, p.x + 4.0, p.y - 9.0, label, 6.0, 1.0, col);
    });
    // The scope's range at its rim.
    let s = if range < 10.0 && range.fract() > 0.05 {
        format!("{range:.1}")
    } else {
        format!("{range:.0}")
    };
    let q = at(2.6, 1.0);
    text(c, q.x + 1.0, q.y, &format!("{s}KM"), 5.0, GREY);
    let (emv, signal) = (th.emv.emv, crate::mpd::signal(th.emv));
    let detected = crate::mpd::detected(th.history);
    // The radars whose signal reaches us (the scope shows only those).
    let received: Vec<_> = th
        .emitters
        .iter()
        .filter(|e| {
            e.receiving || e.tracking || e.launch || pulse_of(th.history, e.source).is_some()
        })
        .collect();
    if detected && blink {
        text(c, 6.0, 27.0, "DETECTED", 6.0, RED);
    }
    text(c, 6.0, 19.0, &format!("{} RCVD", received.len()), 5.0, GREY);
    // The aspect: the bloom scaled to fit with its detection ring.
    let (ic, ir) = (Vec2::new(185.0, 50.0), 27.0);
    text(c, 142.0, 19.0, "ASPECT", 5.0, GREY);
    c.ring(ic, ir, 0.7, DARK);
    c.triangle(
        [
            ic - Vec2::Y * (ir + 1.0),
            ic + Vec2::new(-2.0, -ir - 4.0),
            ic + Vec2::new(2.0, -ir - 4.0),
        ],
        WHITE,
    );
    let base = bloom_size(emv, 1.0);
    let limit = (signal > 0.0).then(|| bloom_size(1.0 - signal, 1.0));
    let most = (0..96).map(|i| crate::mpd::lobes(i as f32 * TAU / 96.0, th)).fold(0.0, f32::max);
    let scale = ir * 0.9 / (base * most).max(limit.unwrap_or(0.0)).max(1e-3);
    let pts = bloom(ic, base * scale, ir, th, 96);
    fill_bloom(c, ic, &pts, if detected { DEEP_RED } else { NAVY });
    for p in pts.windows(2) {
        let out = limit.is_some_and(|l| (p[0] - ic).length() > l * scale);
        let col = match (out, detected) {
            (true, true) => RED,
            (true, false) => AMBER,
            (false, true) => RED,
            (false, false) => CYAN,
        };
        c.stroke(p[0], p[1], 0.8, col);
    }
    if let Some(l) = limit {
        dotted_ring(c, ic, l * scale, 0.7, AMBER);
    }
    // Where the radars reaching us look at it from: a tick at each one's bearing, red where
    // the bloom there reaches past the detection ring (that radar sees the most of it).
    for e in &received {
        let d = Vec2::new(e.rel_bearing.sin(), -e.rel_bearing.cos());
        let reach = base * crate::mpd::lobes(e.rel_bearing, th);
        let col = match limit {
            Some(l) if reach > l => RED,
            _ if e.tracking || e.launch => AMBER,
            _ => WHITE,
        };
        c.stroke(ic + d * (ir + 1.5), ic + d * (ir + 6.0), 1.2, col);
    }
    // The configuration.
    let banked = th.roll.to_degrees().abs() >= 10.0;
    let pitched = th.pitch.to_degrees().abs() >= 10.0;
    let tag = |on: bool| if on { AMBER } else { GREY };
    let (x0, x1) = (142.0, 186.0);
    let gear = if th.gear_down { "GEAR DN" } else { "GEAR UP" };
    text(c, x0, 82.0, gear, 5.5, tag(th.gear_down));
    let bay = if th.bay_open { "BAY OPN" } else { "BAY CLS" };
    text(c, x1, 82.0, bay, 5.5, tag(th.bay_open));
    let bank = th.roll.to_degrees();
    let side = if bank.round() == 0.0 {
        ""
    } else if bank > 0.0 {
        "R"
    } else {
        "L"
    };
    text(c, x0, 91.0, &format!("BNK {:02.0}{side}", bank.abs()), 5.5, tag(banked));
    let pitch = th.pitch.to_degrees();
    text(c, x1, 91.0, &format!("PIT {:+03.0}", pitch), 5.5, tag(pitched));
    // The signature, the strongest signal and the margin between them.
    c.hline(x0 as i32, 227, 99, DARK);
    let margin = 1.0 - emv - signal;
    let sig_col = if detected { RED } else { CYAN };
    let mrg_col = if signal <= 0.0 {
        GREY
    } else if margin < 0.0 {
        RED
    } else if margin < 0.15 {
        AMBER
    } else {
        GREEN
    };
    let mrg = if signal > 0.0 {
        let m = (margin.clamp(-1.0, 1.0) * 99.0).round() as i32;
        format!("{m:+}")
    } else {
        "--".into()
    };
    for (k, (name, value, col)) in [
        ("RCS", format!("{}", pct(emv)), sig_col),
        ("SIG", format!("{}", pct(signal)), AMBER),
        ("MRG", mrg, mrg_col),
    ]
    .into_iter()
    .enumerate()
    {
        let x = x0 + k as f32 * 29.0;
        text(c, x, 102.0, name, 5.0, col);
        text(c, x, 110.0, &value, 8.0, col);
    }
    let both = format!("GND {:02}  AIR {:02}", pct(th.emv.egr), pct(th.emv.ear));
    text(c, x0, 124.0, &both, 5.0, GREY);
    // The most dangerous radar reaching us.
    match received.first() {
        Some(e) => {
            let (state, col) = if e.launch {
                ("LNCH", if blink { RED } else { WHITE })
            } else if e.tracking {
                ("TRK", AMBER)
            } else if e.heard {
                ("SRCH", GREEN)
            } else {
                ("", GREY)
            };
            let kind = if e.air { "AIR" } else { "GND" };
            let line = format!("THR {} {kind} {:.0}KM {state}", e.label, e.range_km);
            text(c, 6.0, 124.0, &line, 5.0, col);
        }
        None => {
            text(c, 6.0, 124.0, "NO EMITTERS", 5.0, DARK);
        }
    }
    // The strip chart: a grid every 5 seconds and at half scale under the samples, the
    // legend left of it, the seconds under it.
    let k = c.scale as i32;
    let (cx, cy, cw, ch) = (30, 133, crate::mpd::HISTORY as i32 / k + 2, 20);
    let span = (cw - 2) as f32;
    let right = (cx + cw - 1) as f32;
    let secs = crate::mpd::HISTORY as f32 * crate::mpd::SAMPLE_S;
    for t in (5..30).step_by(5) {
        let x = right - t as f32 / secs * span;
        c.dashed(
            Vec2::new(x, (cy + 1) as f32),
            Vec2::new(x, (cy + ch - 1) as f32),
            1.0,
            2.0,
            0.5,
            DARK,
        );
    }
    let mid = (cy + ch / 2) as f32;
    c.dashed(Vec2::new((cx + 1) as f32, mid), Vec2::new(right, mid), 1.0, 2.0, 0.5, DARK);
    crate::mpd::strip_chart(c, th.history, cx, cy, cw, ch);
    text(c, 6.0, (cy + 2) as f32, "RCS", 5.0, CYAN);
    text(c, 6.0, (cy + 11) as f32, "DET", 5.0, AMBER);
    for t in [30, 20, 10, 0] {
        let x = right - t as f32 / secs * span;
        let s = if t == 0 { "0".to_string() } else { format!("-{t}") };
        stroke_text_at(c, x, (cy + ch + 5) as f32, &s, 5.0, 1.0, GREY, Align::Centre);
    }
}

/// Magnification of the zoomed window and its size (display pixels).
const ZOOM: f32 = 4.0;
const ZOOM_WINDOW: Vec2 = Vec2::new(104.0, 80.0);

/// The zoomed window's corners on a display of `size` pixels: in the corner away from the
/// cursor.
fn zoom_window_rect(size: Vec2, cursor: Vec2) -> (Vec2, Vec2) {
    let x = if cursor.x > size.x / 2.0 { 3.0 } else { size.x - 3.0 - ZOOM_WINDOW.x };
    let y = if cursor.y > size.y / 2.0 {
        BAND as f32 + 3.0
    } else {
        size.y - BAND as f32 - 3.0 - ZOOM_WINDOW.y
    };
    (Vec2::new(x, y), Vec2::new(x, y) + ZOOM_WINDOW)
}

/// The zoomed window shown while "lock cursor target" is held: in the corner away from the
/// cursor, filled with `fill` (the ground map's lets its backdrop through), `draw` paints the
/// page magnified around the cursor, which sits in the middle.
fn draw_zoom_window(c: &mut Canvas, cursor: Vec2, fill: u8, draw: impl FnOnce(&mut Canvas, Vec2)) {
    let (a, b) = zoom_window_rect(Vec2::new(c.w as f32, c.h as f32), cursor);
    c.fill_rect(a.x as i32, a.y as i32, ZOOM_WINDOW.x as i32, ZOOM_WINDOW.y as i32, fill);
    c.set_clip(a.x as i32 + 1, a.y as i32 + 1, b.x as i32 - 2, b.y as i32 - 2);
    let at = (a + b) / 2.0;
    draw(c, at);
    draw_cursor(c, at);
    c.reset_clip();
    c.frame(a, b - Vec2::ONE, 1.5, WHITE);
    stroke_text_at(c, a.x + 5.0, b.y - 10.0, "X4", 9.0, 1.3, WHITE, Align::Left);
}

/// The HARM display shows the radars heard lately ([`EmitterView::heard`]) that pass its filter.
fn harm_shows(filter: HarmFilter, e: &EmitterView) -> bool {
    e.heard
        && match filter {
            HarmFilter::All => true,
            HarmFilter::Sam => !e.air,
            HarmFilter::Air => e.air,
        }
}

fn emitter_source(r: ContactRef) -> Option<stealth_sim::EmitterSource> {
    match r {
        ContactRef::Unit(i) => Some(stealth_sim::EmitterSource::Aircraft(i)),
        ContactRef::Object(i) => Some(stealth_sim::EmitterSource::Site(i)),
        ContactRef::Pilot(_) | ContactRef::Point(..) => None,
    }
}

// ------------------------------------------------------------------------------------------
// Cursor slew and lock

/// A lock on a simulation object picked on a display (`combat` passes it to the simulation).
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockRequest(pub ContactRef);

/// A display lock pressed on what is locked already lets go of it (`combat` clears the
/// simulation's locks).
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unlock;

/// The targeting pod locks what is at this map point (`combat` passes it to the simulation).
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PodDesignate(pub i32, pub i32);

/// The targeting pod locks this ground unit (a vehicle it tracks; `Sim::units` index).
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PodTrack(pub usize);

/// The targeting pod: slewed as the SOI, tracking; "lock cursor target" locks the ground point
/// it holds (then it follows the lock).
#[allow(clippy::too_many_arguments)]
fn update_pod(
    time: Res<Time>,
    controls: Res<FlightControls>,
    mut actions: MessageReader<ActionEvent>,
    mut state: ResMut<MfdState>,
    info: Res<crate::mfd::TrackCam>,
    mut hud: ResMut<HudState>,
    aircraft: Query<(&Aircraft, &WorldPos), With<Player>>,
    camera: Query<&Transform, With<crate::trackcam::TrackCamera>>,
    eyes: HelmetEyes,
    view: Res<crate::camera::ViewState>,
    settings: Res<crate::settings::Settings>,
    mut combat: Option<ResMut<crate::combat::Combat>>,
    mut designate: MessageWriter<PodDesignate>,
    mut track: MessageWriter<PodTrack>,
    mut locks: MessageWriter<LockRequest>,
    (waypoints, theater): (Res<crate::hud::Waypoints>, Option<Res<crate::world::Theater>>),
) {
    let (mut lock, mut clear) = (false, false);
    for ActionEvent(a) in actions.read() {
        lock |= *a == Action::LockCursor;
        clear |= *a == Action::ClearTarget;
    }
    let Ok((a, pos)) = aircraft.single() else { return };
    // The helmet sight, with a helmet-mounted display: the gunship's slaved pod follows it
    // whether its page is up or not (the page shows what it looks at), the Sidewinder seeker
    // too.
    let helmet = settings.aircraft.spec().hmd != crate::aircraft::Hmd::None;
    let sight = helmet_sight(&eyes, view.view).filter(|_| helmet);
    let body = sight.map(|d| {
        let d = a.orientation.inverse() * d;
        state.av.helmet_body.map_or(d, |s| smooth_sight(s, d, time.delta_secs()))
    });
    state.av.helmet_body = body;
    state.av.helmet = body.map(|d| a.orientation * d);
    // The gunship's pod is always up (its turret and Hellfires follow it), as the mast-mounted
    // sight is; that looks from above the rotor.
    let turret = settings.aircraft.spec().slaved_pod();
    if !state.modern && !turret {
        return;
    }
    let origin = pos.0 + (a.orientation * settings.aircraft.spec().pod_origin()).as_dvec3();
    // Snowplow: nothing stays locked (nor is acquired until the next target key).
    if std::mem::take(&mut state.av.tgp.clear)
        && let Some(combat) = &mut combat
    {
        combat.clear_target();
    }
    let shown = state.camera_shown();
    let looking = if turret {
        slaved_sight(combat.as_deref(), origin)
            .or(state.av.helmet)
            .or_else(|| Some(a.orientation * pod_dir(0.0, -0.05)))
    } else {
        camera.single().ok().filter(|_| shown && info.live).map(|t| t.forward().as_vec3())
    };
    // The flat world or the deck under the aircraft, and the hills.
    let ground = Terrain {
        y: crate::world::wu_to_bevy(0.0, 0.0, a.ground_altitude as f64).y,
        hills: theater.map(|t| t.hills.clone()).unwrap_or_default(),
    };
    // Over the hills the aircraft's own ground is theirs: the plane under them is the flat world.
    let ground = if ground.hills.ground_m(pos.0.x, -pos.0.z) > 0.0 {
        Terrain { y: 0.0, ..ground }
    } else {
        ground
    };
    // CZ: point track on the current steerpoint, the pod the SOI to slew on from there.
    let mut cz = false;
    if std::mem::take(&mut state.av.tgp.cz)
        && let Some(wp) = waypoints.target()
    {
        cz = true;
        let p = crate::world::wu_to_bevy(wp.east, wp.north, a.ground_altitude as f64);
        state.av.tgp.mode = TgpMode::Point(ground.on(p));
        state.av.tgp.idle = 0.0;
        if shown {
            state.av.soi = Some(Format::Tgp);
        }
    }
    // Target cleared: the pod holds the current steerpoint (not the gunship's, which follows the
    // helmet), without snapping onto a unit there (that would lock it again).
    if clear
        && !turret
        && let Some(wp) = waypoints.target()
    {
        let p = crate::world::wu_to_bevy(wp.east, wp.north, a.ground_altitude as f64);
        state.av.tgp.mode = TgpMode::Point(ground.on(p));
        state.av.tgp.track = None;
        state.av.tgp.idle = 0.0;
    }
    let soi = state.av.soi_format() == Some(Format::Tgp);
    let zoom = state.cam_zoom.unwrap_or(info.zoom.saturating_sub(1));
    // Not while the slew keys move a waypoint (the classic avionics have no SOI).
    let slew = if soi && shown && !crate::mfd::editing_waypoints(&state, &waypoints) {
        controls.slew
    } else {
        Vec2::ZERO
    };
    let tgp = &mut state.av.tgp;
    // A point track on a vehicle follows it, until slewed off or it is gone.
    if let Some(k) = tgp.track {
        let at = combat.as_deref().and_then(|c| vehicle_at(c, k, &ground));
        match (tgp.mode, at) {
            (TgpMode::Point(_), Some(at)) if slew == Vec2::ZERO => tgp.mode = TgpMode::Point(at),
            _ => tgp.track = None,
        }
    }
    let slewed = matches!(tgp.mode, TgpMode::Area(_));
    let los = pod_step(tgp, slew, time.delta_secs(), zoom, a.orientation, origin, looking, &ground);
    // Settled near a unit, or CZ'd onto a steerpoint on one: the pod snaps onto it, locked as
    // by the lock key (a gun, which cannot be locked, held in point track).
    if (slewed || cz)
        && let TgpMode::Point(p) = tgp.mode
        && let Some(snap) = combat.as_deref().and_then(|c| unit_near(c, p, &ground))
    {
        match snap {
            Snap::Object(object) => {
                tgp.mode = TgpMode::Slaved;
                hud.mode = HudMode::Gnd;
                locks.write(LockRequest(ContactRef::Object(object)));
            }
            Snap::Tile(x, y) => {
                designate.write(PodDesignate(x, y));
                tgp.mode = TgpMode::Slaved;
                hud.mode = HudMode::Gnd;
            }
            Snap::Gun(k, at) => {
                tgp.mode = TgpMode::Point(at);
                let vehicle = combat.as_deref().is_some_and(|c| c.sim.units()[k].fine.is_some());
                tgp.track = vehicle.then_some(k);
            }
        }
    }
    tgp.los_ground = los.filter(|_| shown || turret);
    tgp.los = pod_direction(tgp, a.orientation, origin).or(looking).filter(|_| shown || turret);
    if lock && soi && tgp.mode != TgpMode::Slaved {
        match los {
            // A vehicle: the simulation's point follows it.
            Some(_) if tgp.track.is_some() => {
                track.write(PodTrack(tgp.track.take().unwrap()));
                hud.mode = HudMode::Gnd;
                tgp.mode = TgpMode::Slaved;
            }
            Some(p) => {
                let (e, n, _) = bevy_to_wu(p);
                designate.write(PodDesignate(
                    (e / 32.0).round() as i32,
                    (0x8000 as f64 - n / 32.0).round() as i32,
                ));
                hud.mode = HudMode::Gnd;
                tgp.mode = TgpMode::Slaved;
            }
            None => tgp.mode = TgpMode::Slaved,
        }
    }
}

/// How near a unit the settled pod snaps onto it (metres, across the ground).
const POD_SNAP: f64 = 30.0;

/// What the settled pod snaps onto.
enum Snap {
    /// A world object, locked.
    Object(usize),
    /// A theater tile object at this map point, designated.
    Tile(i32, i32),
    /// An air defence gun or a vehicle (`Sim::units` index): a point track on it.
    Gun(usize, DVec3),
}

/// Where vehicle `k` is now (Bevy world, on the ground), None once destroyed.
fn vehicle_at(c: &crate::combat::Combat, k: usize, ground: &Terrain) -> Option<DVec3> {
    let u = c.sim.units().get(k).filter(|u| !u.destroyed)?;
    let (x, y) = u.map_xy();
    Some(ground.on(crate::sim_render::map_point(x, y, 0.0)))
}

/// The unit nearest the pod's ground point `p` within `POD_SNAP`, friend or foe.
fn unit_near(c: &crate::combat::Combat, p: DVec3, ground: &Terrain) -> Option<Snap> {
    use stealth_sim::oflags;
    let centre = |x: i32, y: i32| {
        let (e, n) = crate::combat::map_wu(x, y);
        ground.on(crate::world::wu_to_bevy(e + 16.0, n - 16.0, 0.0))
    };
    let flat = |q: DVec3| DVec2::new(q.x - p.x, q.z - p.z).length();
    let objects = c.sim.objects().iter().enumerate().skip(3);
    let objects = objects
        .filter(|(_, o)| o.flags & (oflags::DESTROYED | oflags::AIRFIELD) == 0)
        .map(|(i, o)| (Snap::Object(i), centre(o.x, o.y)));
    let guns = c.sim.units().iter().enumerate().filter(|(_, u)| !u.destroyed).map(|(k, u)| {
        let at = vehicle_at(c, k, ground).filter(|_| u.fine.is_some());
        let at = at.unwrap_or_else(|| centre(u.x, u.y));
        (Snap::Gun(k, at), at)
    });
    let (e, n, _) = bevy_to_wu(p);
    let tile = c.sim.target_tile_near(e as i32, n as i32).map(|(x, y)| {
        let at = crate::world::wu_to_bevy(x as f64, y as f64, 0.0);
        let (mx, my) = (x >> 5, 0x8000 - (y >> 5));
        (Snap::Tile(mx, my), ground.on(at))
    });
    objects
        .chain(guns)
        .chain(tile)
        .map(|(r, at)| (r, flat(at)))
        .filter(|&(_, d)| d <= POD_SNAP)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(r, _)| r)
}

/// Where the gunship's slaved pod looks: at the lock (else the helmet sight).
fn slaved_sight(combat: Option<&crate::combat::Combat>, eye: DVec3) -> Option<Vec3> {
    // As the tracking camera: the ground lock, else the aircraft or pilot locked.
    combat.and_then(|c| {
        let (sim, t) = (&c.sim, c.sim.targeting());
        let at = if let Some(o) = t.ground.and_then(|g| sim.objects().get(g)) {
            let (e, n) = crate::combat::map_wu(o.x, o.y);
            crate::world::wu_to_bevy(e + 16.0, n - 16.0, 0.0)
        } else if let Some((i, u)) = t.air.and_then(|i| sim.flights().get(i).map(|u| (i, *u))) {
            let (e, n, z) = crate::combat::unit_position(c.prev_flights.get(i), &u, c.alpha);
            crate::world::wu_to_bevy(e, n, z)
        } else {
            let ((e, n, z), _, _) = t.pilot.and_then(|k| crate::combat::remote_motion(c, k))?;
            crate::world::wu_to_bevy(e, n, z)
        };
        (at - eye).as_vec3().try_normalize()
    })
}

/// What the helmet sight looks along: the eye cameras, and glvr's head (`--gl-vr`, whose only
/// camera is the tracking root, fixed in the cockpit).
type HelmetEyes<'w, 's> = Query<
    'w,
    's,
    (&'static GlobalTransform, Has<crate::camera::XrHead>),
    (Or<(With<Camera3d>, With<crate::camera::XrHead>)>, Without<crate::camera::OffscreenCamera>),
>;

/// Where the pilot looks from the cockpit (the helmet sight: the eye camera, the headset in
/// VR). None in the outside views.
fn helmet_sight(eyes: &HelmetEyes, view: crate::camera::View) -> Option<Vec3> {
    let cockpit = matches!(
        view,
        crate::camera::View::Cockpit
            | crate::camera::View::LookLeft
            | crate::camera::View::LookRight
            | crate::camera::View::LookRear
    );
    let head = eyes.iter().any(|(_, head)| head);
    let sum: Vec3 =
        eyes.iter().filter(|&(_, h)| h == head).map(|(t, _)| t.forward().as_vec3()).sum();
    sum.try_normalize().filter(|_| cockpit)
}

/// Helmet sight smoothing: the time constant (seconds) while the head is nearly still, and
/// the angle (radians) behind the head at which it is halved (a deliberate turn follows fast).
const HELMET_LAG: f32 = 0.12;
const HELMET_FOLLOW: f32 = 0.03;

/// Moves the smoothed sight `s` toward the head's direction `d` (both unit vectors).
fn smooth_sight(s: Vec3, d: Vec3, dt: f32) -> Vec3 {
    let angle = s.angle_between(d);
    let tau = HELMET_LAG / (1.0 + angle / HELMET_FOLLOW);
    let k = 1.0 - (-dt / tau).exp();
    s.slerp(d, k).try_normalize().unwrap_or(d)
}

/// Cursor speed at full slew (display pixels per second).
const SLEW_SPEED: f32 = 110.0;
/// After the HSD's cursor pushed past the bottom edge zoomed in, it is put back at least this
/// far above the edge (pixels), so that holding the slew does not zoom in again at once.
const HSD_EDGE_RESET: f32 = 30.0;
/// A lock picks the nearest symbol within this many pixels of the cursor.
const PICK_RADIUS: f32 = 14.0;

/// Moves the SOI's cursor with the slew controls; "lock cursor target" locks the nearest
/// symbol under it (on the ground map: the nearest radar return, else the ground point),
/// switching the HUD to the matching mode. The radar formats lock when the button is let go,
/// and let go of a lock pressed again on it.
#[allow(clippy::too_many_arguments)]
fn slew_and_lock(
    time: Res<Time>,
    controls: Res<FlightControls>,
    mut actions: MessageReader<ActionEvent>,
    mut state: ResMut<MfdState>,
    (contacts, emitters, radar): (
        Res<Contacts>,
        Res<Emitters>,
        Option<Res<crate::ground_radar::GroundRadar>>,
    ),
    aa: Option<Res<crate::aa_radar::AaRadar>>,
    mut hud: ResMut<HudState>,
    aircraft: Query<(&Aircraft, &WorldPos), With<Player>>,
    mut locks: MessageWriter<LockRequest>,
    (mut designate, mut track, mut unlock): (
        MessageWriter<PodDesignate>,
        MessageWriter<PodTrack>,
        MessageWriter<Unlock>,
    ),
) {
    let lock = actions.read().any(|a| a.0 == Action::LockCursor);
    if !state.modern {
        return;
    }
    let size = Vec2::new(HIRES.0 as f32, HIRES.1 as f32);
    let lo = Vec2::new(4.0, BAND as f32 + 4.0);
    let hi = size - lo;
    let ac = aircraft.single().ok();
    let mut view = hsd_view(&state.av, ac);
    follow_view(&mut state.av, view, lo, hi);
    let mut gm = gm_view(&state.av, ac);
    follow_gm(&mut state.av, gm, lo, hi);
    let f = state
        .av
        .soi_format()
        .filter(|f| matches!(f, Format::Hsd | Format::Harm | Format::Gm | Format::Fcr));
    let av = &mut state.av;
    // Holding the lock shows the zoomed window and slews finely; letting go locks again, now
    // picking only what is right under the cursor in the window.
    let zoomed = av.zoom_window();
    av.mark_held =
        if controls.mark && f.is_some() { av.mark_held + time.delta_secs() } else { 0.0 };
    let release = zoomed && av.mark_held == 0.0;
    let Some(f) = f else {
        av.radar_lock = false;
        return;
    };
    let radius = if release { PICK_RADIUS / ZOOM } else { PICK_RADIUS };
    // The radar formats lock once the button is let go (at once for a press without a hold,
    // a script's), only then knowing whether the window was up.
    let lock = if f.radar() {
        av.radar_lock |= lock;
        let go = (av.radar_lock || release) && !controls.mark;
        av.radar_lock &= !go;
        go
    } else {
        lock
    };
    let fine = if zoomed { ZOOM } else { 1.0 };
    let delta =
        Vec2::new(controls.slew.x, -controls.slew.y) * SLEW_SPEED / fine * time.delta_secs();
    // The HSD's and the ground map's cursors pushed past the top edge zoom out, past the bottom
    // zoom in.
    if f == Format::Hsd && !zoomed && hsd_edge_zoom(av, delta.y, lo, hi, |av| hsd_view(av, ac)) {
        view = hsd_view(av, ac);
    }
    if f == Format::Gm {
        let mut range = av.gm_range;
        let (mut cursor, mut placed) = (av.gm_cursor, av.gm_view);
        let size = GM_RANGES.len();
        let view_at = |range: usize| {
            let mut a = av.clone();
            a.gm_range = range;
            gm_view(&a, ac)
        };
        if edge_zoom(&mut range, size, &mut cursor, &mut placed, delta.y, lo, hi, view_at) {
            (av.gm_range, av.gm_cursor, av.gm_view) = (range, cursor, placed);
            gm = gm_view(av, ac);
        }
    }
    if f == Format::Fcr {
        fcr_slew(av, delta);
        if !lock {
            return;
        }
        let range = av.fcr_range_m();
        let Some(aa) = aa.as_deref().filter(|r| r.on) else { return };
        let track = aa.track.map(|t| (t.az, t.range, t.target));
        let bricks = aa.bricks.iter().map(|b| (b.az, b.range, b.target));
        let near = nearest(
            bricks
                .chain(track)
                .filter(|&(_, r, _)| r <= range)
                .map(|(az, r, target)| (fcr_point(az, r / range), target)),
            av.fcr_cursor,
            radius,
        );
        if near.is_some() && near == contacts.locked {
            unlock.write(Unlock);
        } else if let Some(target) = near {
            hud.mode = HudMode::Air;
            locks.write(LockRequest(target));
        }
        return;
    }
    let cursor = match f {
        Format::Hsd => &mut av.hsd_cursor,
        Format::Gm => &mut av.gm_cursor,
        _ => &mut av.harm_cursor,
    };
    if delta != Vec2::ZERO {
        *cursor = (*cursor + delta).clamp(lo, hi);
    }
    if !lock && !release {
        return;
    }
    let cursor = *cursor;
    if f == Format::Gm {
        let Some(gm) = gm.filter(|_| lock) else { return };
        let returns = radar.as_deref().map_or(&[][..], |r| &r.returns[..]);
        let (pick, d) = gm_pick(returns, &gm, cursor, radius);
        // The locked ground point under the cursor, no return nearer: let go of it.
        let held = contacts.ground_lock.map(|(e, n)| gm.to_screen(e, n).distance(cursor));
        if held.is_some_and(|h| h <= radius && d.is_none_or(|d| h <= d + 2.0)) {
            unlock.write(Unlock);
            return;
        }
        av.tgp.mode = TgpMode::Slaved;
        av.tgp.track = None;
        hud.mode = HudMode::Gnd;
        match pick {
            GmPick::Object(i) => {
                locks.write(LockRequest(ContactRef::Object(i)));
            }
            GmPick::Vehicle(k) => {
                track.write(PodTrack(k));
            }
            GmPick::Point(x, y) => {
                designate.write(PodDesignate(x, y));
            }
        }
        return;
    }
    let picked = match f {
        Format::Hsd => view.and_then(|view| pick_contact(&contacts, &view, cursor, radius)),
        _ => {
            let (centre, r) = harm_scope();
            let filter = av.harm_filter;
            nearest(
                emitters
                    .list
                    .iter()
                    .filter(|e| harm_shows(filter, e))
                    .map(|e| (crate::rwr::scope_point(e, centre, r), emitter_ref(e))),
                cursor,
                radius,
            )
        }
    };
    if let Some(target) = picked {
        av.tgp.mode = TgpMode::Slaved;
        hud.mode = match target {
            ContactRef::Unit(_) | ContactRef::Pilot(_) => HudMode::Air,
            ContactRef::Object(_) | ContactRef::Point(..) => HudMode::Gnd,
        };
        locks.write(LockRequest(target));
    }
}

/// The HSD's range a step out when the cursor is slewed up past the top edge (`lo`..`hi` the
/// cursor's room), a step in past the bottom edge (`edge_zoom`). True if the range changed.
fn hsd_edge_zoom(
    av: &mut AvionicsState,
    dy: f32,
    lo: Vec2,
    hi: Vec2,
    view_of: impl Fn(&AvionicsState) -> Option<HsdView>,
) -> bool {
    let (mut range, mut cursor, mut placed) = (av.hsd_range, av.hsd_cursor, av.cursor_view);
    let view_at = |range: usize| {
        let mut a = av.clone();
        a.hsd_range = range;
        view_of(&a)
    };
    let size = HSD_RANGES.len();
    if !edge_zoom(&mut range, size, &mut cursor, &mut placed, dy, lo, hi, view_at) {
        return false;
    }
    (av.hsd_range, av.hsd_cursor, av.cursor_view) = (range, cursor, placed);
    true
}

/// A display's range (index `range` of `count`, through `view_at`) a step out when its cursor
/// is slewed up past the top edge (`lo`..`hi` the cursor's room), a step in past the bottom
/// edge; the cursor stays on its ground point, after zooming in at least `HSD_EDGE_RESET` above
/// the bottom edge, and `placed` becomes the new view. True if the range changed.
#[allow(clippy::too_many_arguments)]
fn edge_zoom(
    range: &mut usize,
    count: usize,
    cursor: &mut Vec2,
    placed: &mut Option<HsdView>,
    dy: f32,
    lo: Vec2,
    hi: Vec2,
    view_at: impl Fn(usize) -> Option<HsdView>,
) -> bool {
    let y = cursor.y + dy;
    let step = if dy < 0.0 && y < lo.y {
        1
    } else if dy > 0.0 && y > hi.y {
        -1
    } else {
        return false;
    };
    let next = (*range as i32 + step).clamp(0, count as i32 - 1) as usize;
    let Some(prev) = view_at(*range).filter(|_| next != *range) else { return false };
    *range = next;
    let Some(view) = view_at(next) else { return true };
    let g = prev.to_world(*cursor);
    let mut p = view.to_screen(g.x, g.y);
    if step < 0 {
        p.y = p.y.min(hi.y - HSD_EDGE_RESET);
    }
    *cursor = p.clamp(lo, hi);
    *placed = Some(view);
    true
}

/// The A-A radar's cursor stays on the B-scope; pushed past its top the range goes a step
/// out, past its bottom a step in (the cursor keeping its range, back off the bottom edge so
/// that each push is one step), past its left side the scan a step narrower, past its right
/// a step wider (the cursor back off that side).
fn fcr_slew(av: &mut AvionicsState, delta: Vec2) {
    let (lo, hi) = fcr_scope();
    let mut q = av.fcr_cursor + delta;
    let step = if delta.y < 0.0 && q.y < lo.y {
        1
    } else if delta.y > 0.0 && q.y > hi.y {
        -1
    } else {
        0
    };
    let next = (av.fcr_range as i32 + step).clamp(0, FCR_RANGES_NM.len() as i32 - 1) as usize;
    if next != av.fcr_range {
        let k = (hi.y - q.y.clamp(lo.y, hi.y)) / (hi.y - lo.y)
            * (FCR_RANGES_NM[av.fcr_range] / FCR_RANGES_NM[next]);
        q.y = hi.y - k.min(1.0) * (hi.y - lo.y);
        if step < 0 {
            q.y = q.y.min(hi.y - HSD_EDGE_RESET);
        }
        av.fcr_range = next;
    }
    let narrower = if delta.x < 0.0 && q.x < lo.x {
        1
    } else if delta.x > 0.0 && q.x > hi.x {
        -1
    } else {
        0
    };
    let az = (av.fcr_az as i32 + narrower).clamp(0, FCR_AZ_DEG.len() as i32 - 1) as usize;
    if az != av.fcr_az {
        av.fcr_az = az;
        q.x = if narrower > 0 { lo.x + HSD_EDGE_RESET } else { hi.x - HSD_EDGE_RESET };
    }
    av.fcr_cursor = q.clamp(lo, hi);
}

/// Keeps the ground map's cursor on its ground point as the view changes.
fn follow_gm(av: &mut AvionicsState, view: Option<HsdView>, lo: Vec2, hi: Vec2) {
    if let (Some(prev), Some(view)) = (av.gm_view, view)
        && prev != view
    {
        let g = prev.to_world(av.gm_cursor);
        av.gm_cursor = view.to_screen(g.x, g.y).clamp(lo, hi);
    }
    av.gm_view = view;
}

/// What a lock on the ground map designates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GmPick {
    /// A world object, locked.
    Object(usize),
    /// A vehicle or a gun (`Sim::units` index): the simulation's point follows it.
    Vehicle(usize),
    /// A map point (x east, y south): a tile object there, else the ground.
    Point(i32, i32),
}

/// The ground map's designation at `cursor`: the nearest radar return within `radius` where it
/// was painted (objects and vehicles before buildings as near; and its distance), else the
/// ground point under the cursor.
fn gm_pick(
    returns: &[crate::ground_radar::Return],
    view: &HsdView,
    cursor: Vec2,
    radius: f32,
) -> (GmPick, Option<f32>) {
    use crate::ground_radar::RadarTarget;
    let near = returns
        .iter()
        .map(|r| {
            let d = view.to_screen(r.east / WU, r.north / WU).distance(cursor);
            // A tile object counts a few pixels further than a site or vehicle on it.
            let d = if matches!(r.target, RadarTarget::Tile(..)) { d + 3.0 } else { d };
            (d, r.target)
        })
        .filter(|(d, _)| *d <= radius)
        .min_by(|a, b| a.0.total_cmp(&b.0));
    let pick = match near.map(|n| n.1) {
        Some(RadarTarget::Object(i)) => GmPick::Object(i),
        Some(RadarTarget::Unit(k)) => GmPick::Vehicle(k),
        Some(RadarTarget::Tile(x, y)) => GmPick::Point(x, y),
        None => {
            let g = view.to_world(cursor);
            GmPick::Point((g.x / 32.0).round() as i32, (0x8000 as f64 - g.y / 32.0).round() as i32)
        }
    };
    (pick, near.map(|n| n.0))
}

/// Keeps the cursors on what they point at as the view changes: the HSD cursor on its ground
/// point (through turns, flight and range changes), the HARM cursor at its bearing (the HARM
/// display is heading up, so it turns against the heading like the emitter symbols).
fn follow_view(av: &mut AvionicsState, view: Option<HsdView>, lo: Vec2, hi: Vec2) {
    let Some(view) = view else {
        av.cursor_view = None;
        return;
    };
    if let Some(prev) = av.cursor_view.filter(|prev| *prev != view) {
        let g = prev.to_world(av.hsd_cursor);
        av.hsd_cursor = view.to_screen(g.x, g.y).clamp(lo, hi);
        let (centre, _) = harm_scope();
        let turn = (prev.heading - view.heading) as f32;
        let (s, c) = turn.sin_cos();
        let d = av.harm_cursor - centre;
        av.harm_cursor = (centre + Vec2::new(d.x * c - d.y * s, d.x * s + d.y * c)).clamp(lo, hi);
    }
    av.cursor_view = Some(view);
}

fn emitter_ref(e: &EmitterView) -> ContactRef {
    match e.source {
        stealth_sim::EmitterSource::Site(i) => ContactRef::Object(i),
        stealth_sim::EmitterSource::Aircraft(i) => ContactRef::Unit(i),
        // A Shilka: whatever lies where it stands.
        stealth_sim::EmitterSource::Gun(_) => {
            let (east, north) = (e.east, e.north);
            ContactRef::Point((east / 32.0) as i32, (0x8000 as f64 - north / 32.0) as i32)
        }
    }
}

/// The lockable contact nearest the cursor on the HSD: a hostile one if any is in reach,
/// else a friendly one.
pub fn pick_contact(
    contacts: &Contacts,
    view: &HsdView,
    cursor: Vec2,
    radius: f32,
) -> Option<ContactRef> {
    let pick = |friendly: bool| {
        nearest(
            contacts
                .shown()
                .filter(|k| k.friendly == friendly)
                .filter_map(|k| Some((view.to_screen(k.east, k.north), k.target?))),
            cursor,
            radius,
        )
    };
    pick(false).or_else(|| pick(true))
}

fn nearest(
    candidates: impl Iterator<Item = (Vec2, ContactRef)>,
    cursor: Vec2,
    radius: f32,
) -> Option<ContactRef> {
    candidates
        .map(|(q, t)| (q.distance(cursor), t))
        .filter(|(d, _)| *d <= radius)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, t)| t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mfd::Contact;

    #[test]
    fn selecting_the_other_mfds_format_swaps() {
        let mut av = AvionicsState::default();
        assert_eq!(av.formats[..2], [Format::Hsd, Format::Sms]);
        av.select(MfdSide::Right, Format::Hsd);
        assert_eq!(av.formats[..2], [Format::Sms, Format::Hsd]);
        av.select(MfdSide::Left, Format::Harm);
        assert_eq!(av.formats[..2], [Format::Harm, Format::Hsd]);
        // The SOI stays with its format.
        assert_eq!(av.soi_format(), Some(Format::Hsd));
        av.select(MfdSide::Right, Format::Map);
        assert_eq!(
            av.soi_format(),
            Some(Format::Harm),
            "SOI moves to the remaining capable format"
        );
        av.select(MfdSide::Left, Format::Tgp);
        assert_eq!(av.soi_format(), Some(Format::Tgp));
        av.select(MfdSide::Left, Format::Sms);
        assert_eq!(av.soi_format(), None);
    }

    /// A campaign enters the flight in the parked F-117A and gives the sortie's aircraft later:
    /// its radar brings the ground map, and the F-117A again takes it away.
    #[test]
    fn radar_comes_with_the_aircraft() {
        let mut av = AvionicsState::default();
        assert!(av.available().all(|f| f != Format::Gm));
        av.set_radar(true);
        assert!(av.formats(MfdSide::Left).contains(&Format::Gm));
        av.select(MfdSide::Left, Format::Gm);
        assert_eq!(av.format(MfdSide::Left), Format::Gm);
        av.set_radar(false);
        assert_eq!(av.format(MfdSide::Left), Format::Harm);
        assert!(!av.radar_emitting());
    }

    /// With the F-117A's third MFD: it starts on the HSI, swaps with either other MFD, and
    /// without it its format counts as shown nowhere.
    #[test]
    fn three_mfds() {
        let mut av = AvionicsState::default();
        assert_eq!(av.shown(Format::Hsi), None);
        av.set_mfds(3);
        assert_eq!(av.formats[..3], [Format::Hsd, Format::Sms, Format::Hsi]);
        av.select(MfdSide::Third, Format::Hsd);
        assert_eq!(av.formats[..3], [Format::Hsi, Format::Sms, Format::Hsd]);
        assert_eq!(av.soi_format(), Some(Format::Hsd), "the SOI goes with its format");
        av.select(MfdSide::Right, Format::Hsd);
        assert_eq!(av.formats[..3], [Format::Hsi, Format::Hsd, Format::Sms]);
        av.select(MfdSide::Left, Format::Tgp);
        assert_eq!(av.formats[..3], [Format::Tgp, Format::Hsd, Format::Sms]);
        // Removed and back, it shows the HSI again.
        av.select(MfdSide::Third, Format::Hsi);
        av.set_mfds(2);
        av.set_mfds(3);
        assert_eq!(av.format(MfdSide::Third), Format::Hsi);
        // A third MFD whose HSI went to another MFD takes a format not shown elsewhere.
        av.select(MfdSide::Left, Format::Hsi);
        av.set_mfds(2);
        av.set_mfds(3);
        let [l, r, t, _] = av.formats;
        assert!(t != l && t != r, "{:?}", av.formats);
    }

    /// The F-22A's fourth MFD starts on the engines, swaps formats with the others, and the SOI
    /// goes round all four.
    #[test]
    fn four_mfds() {
        let mut s = MfdState { modern: true, ..default() };
        let mut raster = MapRaster::default();
        s.av.set_mfds(4);
        assert_eq!(s.av.formats, [Format::Hsd, Format::Sms, Format::Hsi, Format::Eng]);
        assert_eq!(osb(MfdSide::Fourth, 5, &s).map(|o| o.0), Some("ENG"));
        s.av.select(MfdSide::Left, Format::Eng);
        assert_eq!(s.av.formats, [Format::Eng, Format::Sms, Format::Hsi, Format::Hsd]);
        s.av.select(MfdSide::Third, Format::Tgp);
        assert_eq!(s.av.soi_format(), Some(Format::Hsd));
        apply(&mut s, &mut raster, MfdCmd::SwapSoi);
        assert_eq!(s.av.soi_format(), Some(Format::Tgp));
        apply(&mut s, &mut raster, MfdCmd::SwapSoi);
        assert_eq!(s.av.soi_format(), Some(Format::Hsd));
        // Back to three MFDs: the fourth's format is shown nowhere.
        s.av.set_mfds(3);
        assert_eq!(s.av.shown(s.av.format(MfdSide::Fourth)), None);
    }

    /// A bottom button shows its format; pressed again it opens the format menu down the sides,
    /// where a pick goes onto that button (swapping with its place on the row) and comes up.
    #[test]
    fn format_menu() {
        let mut s = MfdState { modern: true, ..default() };
        let mut raster = MapRaster::default();
        let press = |s: &mut MfdState, raster: &mut MapRaster, side, slot| {
            let Some((_, OsbFn::Cmd(cmd), _)) = osb(side, slot, s) else { panic!("{slot}") };
            apply(s, raster, cmd);
        };
        // HSD is on the left MFD's first button: pressing it opens the menu.
        assert_eq!(s.av.format(MfdSide::Left), Format::Hsd);
        press(&mut s, &mut raster, MfdSide::Left, 5);
        assert_eq!(s.av.menu(MfdSide::Left), Some(0));
        // The menu lists every format but the radar's (no radar) and the RCS page (not a
        // stealth type) down the sides, the button's own lit; the bottom row stays, the button
        // it came from lit.
        let listed: Vec<_> = (10..20).filter_map(|k| osb(MfdSide::Left, k, &s)).collect();
        assert_eq!(
            listed.iter().map(|o| o.0).collect::<Vec<_>>(),
            ["HSD", "HARM", "TGP", "MAP", "SMS", "HSI", "ENG"]
        );
        assert!(listed[0].2 && !listed[1].2);
        assert_eq!(osb(MfdSide::Left, 5, &s).map(|o| o.2), Some(true));
        assert_eq!(osb(MfdSide::Left, 0, &s), None, "no format functions on the menu");
        // Pressed again: back to the format.
        press(&mut s, &mut raster, MfdSide::Left, 5);
        assert_eq!(s.av.menu(MfdSide::Left), None);
        // ENG onto the first button: shown, and on the row.
        press(&mut s, &mut raster, MfdSide::Left, 5);
        press(&mut s, &mut raster, MfdSide::Left, 16);
        assert_eq!(s.av.menu(MfdSide::Left), None);
        assert_eq!(s.av.format(MfdSide::Left), Format::Eng);
        assert_eq!(
            s.av.formats(MfdSide::Left),
            [Format::Eng, Format::Harm, Format::Tgp, Format::Map, Format::Sms]
        );
        // A format already on the row swaps places with the button's.
        press(&mut s, &mut raster, MfdSide::Left, 5);
        press(&mut s, &mut raster, MfdSide::Left, 13);
        assert_eq!(
            s.av.formats(MfdSide::Left),
            [Format::Map, Format::Harm, Format::Tgp, Format::Eng, Format::Sms]
        );
        assert_eq!(s.av.format(MfdSide::Left), Format::Map);
        // Another button selects its format without the menu; the right MFD's row is its own.
        press(&mut s, &mut raster, MfdSide::Left, 8);
        assert_eq!(s.av.format(MfdSide::Left), Format::Eng);
        assert_eq!(s.av.menu(MfdSide::Left), None);
        assert_eq!(s.av.formats(MfdSide::Right), FORMATS);
        // The rows stay for the next flight (the ground map's only on an aircraft with radar).
        let mut next = AvionicsState::default();
        next.keep_rows(&s.av);
        assert_eq!(next.formats(MfdSide::Left), s.av.formats(MfdSide::Left));
    }

    /// The cargo jet starts on the HSD, the HSI, the engines and the cargo page, left to right
    /// (its fourth MFD sits left of the third), and keeps them in NAV; only it has the cargo page.
    #[test]
    fn cargo_jet_starts_on_its_own_displays() {
        assert!(AvionicsState::default().available().all(|f| f != Format::Cgo));
        let mut av = AvionicsState::default();
        av.set_mfds(4);
        av.set_cargo(true);
        let order = [MfdSide::Left, MfdSide::Right, MfdSide::Fourth, MfdSide::Third];
        let shown = |av: &AvionicsState| order.map(|s| av.format(s));
        assert_eq!(shown(&av), [Format::Hsd, Format::Hsi, Format::Eng, Format::Cgo]);
        av.set_master(HudMode::Nav);
        assert_eq!(shown(&av), [Format::Hsd, Format::Hsi, Format::Eng, Format::Cgo]);
        assert!(av.available().any(|f| f == Format::Cgo));
        av.set_cargo(false);
        assert!(order.iter().all(|&s| av.format(s) != Format::Cgo));
    }

    /// The RCS page is in the stealth types' format menus (with or without a radar: the menu's
    /// ten side buttons hold all the formats), not the others'; it leaves the MFDs with an
    /// aircraft without it, and its range steps from AUTO out and back.
    #[test]
    fn rcs_page_in_the_format_menu() {
        let without = AvionicsState::default();
        assert!(without.available().all(|f| f != Format::Rcs));
        for radar in [false, true] {
            let mut s = MfdState { modern: true, ..default() };
            s.av.radar = radar;
            s.av.rcs = true;
            let mut raster = MapRaster::default();
            let k = s.av.formats(MfdSide::Left).iter().position(|&f| f == Format::Hsd).unwrap();
            apply(&mut s, &mut raster, MfdCmd::Row(MfdSide::Left, k as u8));
            let slot = (10..20)
                .find(|&slot| osb(MfdSide::Left, slot, &s).is_some_and(|o| o.0 == "RCS"))
                .expect("RCS in the menu");
            let Some((_, OsbFn::Cmd(cmd), _)) = osb(MfdSide::Left, slot, &s) else { panic!() };
            apply(&mut s, &mut raster, cmd);
            assert_eq!(s.av.format(MfdSide::Left), Format::Rcs);
            assert_eq!(s.av.formats(MfdSide::Left)[k], Format::Rcs);
            s.av.set_rcs(false);
            assert_ne!(s.av.format(MfdSide::Left), Format::Rcs);
            assert!(!s.av.formats(MfdSide::Left).contains(&Format::Rcs));
        }
        let mut s = MfdState { modern: true, ..default() };
        let mut raster = MapRaster::default();
        s.av.select(MfdSide::Right, Format::Rcs);
        assert_eq!(osb(MfdSide::Right, 2, &s).unwrap().0, "AUTO");
        apply(&mut s, &mut raster, MfdCmd::RcsRange(-1));
        assert_eq!(s.av.rcs_range, 0);
        for _ in 0..9 {
            apply(&mut s, &mut raster, MfdCmd::RcsRange(1));
        }
        assert_eq!(osb(MfdSide::Right, 2, &s).unwrap().0, "80");
    }

    /// The RCS page draws with radars painting (one of whose pulses just saw us), at every
    /// range, and with nothing at all.
    #[test]
    fn rcs_page_draws() {
        let (emitters, emv, history) = crate::mpd::threat_fixture();
        for range in 0..RCS_RANGES.len() + 1 {
            let th = crate::mpd::Threats {
                emitters: &emitters,
                emv: &emv,
                history: &history,
                bay_open: range % 2 == 0,
                gear_down: range % 3 == 0,
                pitch: 0.3,
                roll: -0.6,
            };
            let mut c = Canvas::scaled(HIRES.0, HIRES.1, crate::mfd::HIRES_SCALE);
            draw_rcs(&mut c, &th, range, true);
            // The bloom red (seen), the detection ring and line amber, the chart's crossings.
            for col in [DEEP_RED, RED, AMBER, CYAN, WHITE] {
                assert!(c.px.contains(&col), "colour {col} at range {range}");
            }
        }
        let (emv, history) = (crate::panel::Emv::default(), crate::mpd::ThreatHistory::default());
        let th = crate::mpd::Threats {
            emitters: &[],
            emv: &emv,
            history: &history,
            bay_open: false,
            gear_down: false,
            pitch: 0.0,
            roll: 0.0,
        };
        let mut c = Canvas::scaled(HIRES.0, HIRES.1, crate::mfd::HIRES_SCALE);
        draw_rcs(&mut c, &th, 0, false);
        assert!(c.px.contains(&NAVY) && !c.px.contains(&DEEP_RED));
    }

    /// The menu covers the targeting pod's picture (the display is not see-through).
    #[test]
    fn menu_covers_the_pod() {
        let mut s = MfdState { modern: true, ..default() };
        let mut raster = MapRaster::default();
        s.av.select(MfdSide::Right, Format::Tgp);
        assert!(s.camera_on(MfdSide::Right));
        apply(&mut s, &mut raster, MfdCmd::Row(MfdSide::Right, 2));
        assert_eq!(s.av.menu(MfdSide::Right), Some(2));
        assert!(!s.camera_on(MfdSide::Right));
    }

    /// The HSD's cursor slewed past the top zooms out (staying on its ground point, now nearer
    /// the aircraft), past the bottom zooms in and comes back up off the edge; at the range's
    /// ends it only stops.
    #[test]
    fn hsd_cursor_edges_zoom() {
        let mut av = MfdState { modern: true, ..default() }.av;
        let size = Vec2::new(HIRES.0 as f32, HIRES.1 as f32);
        let (lo, hi) =
            (Vec2::new(4.0, BAND as f32 + 4.0), size - Vec2::new(4.0, BAND as f32 + 4.0));
        let view_of = |av: &AvionicsState| {
            Some(HsdView::new(DVec2::ZERO, 0.0, HSD_RANGES[av.hsd_range], av.hsd_centred, size))
        };
        let r0 = av.hsd_range;
        av.hsd_cursor = Vec2::new(100.0, lo.y);
        assert!(!hsd_edge_zoom(&mut av, 1.0, lo, hi, view_of), "moving down from the top");
        assert!(hsd_edge_zoom(&mut av, -1.0, lo, hi, view_of));
        assert_eq!(av.hsd_range, r0 + 1);
        assert!(av.hsd_cursor.y > lo.y + 20.0, "{:?}", av.hsd_cursor);
        av.hsd_cursor.y = hi.y;
        assert!(hsd_edge_zoom(&mut av, 1.0, lo, hi, view_of));
        assert_eq!(av.hsd_range, r0);
        assert!(av.hsd_cursor.y <= hi.y - HSD_EDGE_RESET, "{:?}", av.hsd_cursor);
        av.hsd_range = 0;
        av.hsd_cursor.y = hi.y;
        assert!(!hsd_edge_zoom(&mut av, 1.0, lo, hi, view_of), "closest range");
        av.hsd_range = HSD_RANGES.len() - 1;
        av.hsd_cursor.y = lo.y;
        assert!(!hsd_edge_zoom(&mut av, -1.0, lo, hi, view_of), "farthest range");
    }

    /// The A-A radar's cursor pushed down past the scope's bottom steps the range in once per
    /// push (not on down to the shortest while held), and past its sides the scan's width.
    #[test]
    fn fcr_cursor_edges_step_range_and_azimuth() {
        let (lo, hi) = fcr_scope();
        let mut av = AvionicsState { fcr_range: 3, fcr_az: 1, ..Default::default() };
        av.fcr_cursor = Vec2::new((lo.x + hi.x) / 2.0, hi.y);
        fcr_slew(&mut av, Vec2::new(0.0, 2.0));
        assert_eq!(av.fcr_range, 2);
        assert!(av.fcr_cursor.y <= hi.y - HSD_EDGE_RESET);
        // Held a few frames more: still off the edge, no more steps.
        for _ in 0..5 {
            fcr_slew(&mut av, Vec2::new(0.0, 2.0));
        }
        assert_eq!(av.fcr_range, 2);
        av.fcr_cursor.x = lo.x;
        fcr_slew(&mut av, Vec2::new(-2.0, 0.0));
        assert_eq!(av.fcr_az, 2, "narrower");
        assert!(av.fcr_cursor.x >= lo.x + HSD_EDGE_RESET - 1.0);
        av.fcr_cursor.x = hi.x;
        fcr_slew(&mut av, Vec2::new(2.0, 0.0));
        assert_eq!(av.fcr_az, 1, "wider");
        // Up past the top: a step out.
        av.fcr_cursor.y = lo.y;
        fcr_slew(&mut av, Vec2::new(0.0, -2.0));
        assert_eq!(av.fcr_range, 3);
    }

    /// The ground map's cursor zooms at the edges as the HSD's does, staying on its ground
    /// point.
    #[test]
    fn gm_cursor_edges_zoom() {
        let size = Vec2::new(HIRES.0 as f32, HIRES.1 as f32);
        let (lo, hi) =
            (Vec2::new(4.0, BAND as f32 + 4.0), size - Vec2::new(4.0, BAND as f32 + 4.0));
        let view_at = |range: usize| Some(HsdView::wedge(DVec2::ZERO, 0.0, GM_RANGES[range], size));
        let (mut range, mut placed) = (2, None);
        let mut cursor = Vec2::new(120.0, lo.y);
        let ground = view_at(range).unwrap().to_world(cursor);
        assert!(edge_zoom(
            &mut range,
            GM_RANGES.len(),
            &mut cursor,
            &mut placed,
            -1.0,
            lo,
            hi,
            view_at
        ));
        assert_eq!(range, 3);
        let now = view_at(range).unwrap().to_world(cursor);
        assert!(now.distance(ground) < 2.0, "{now} {ground}");
        cursor.y = hi.y;
        assert!(edge_zoom(
            &mut range,
            GM_RANGES.len(),
            &mut cursor,
            &mut placed,
            1.0,
            lo,
            hi,
            view_at
        ));
        assert_eq!(range, 2);
        assert!(cursor.y <= hi.y - HSD_EDGE_RESET);
        range = 0;
        cursor.y = hi.y;
        assert!(!edge_zoom(
            &mut range,
            GM_RANGES.len(),
            &mut cursor,
            &mut placed,
            1.0,
            lo,
            hi,
            view_at
        ));
    }

    /// The radar aircraft's left MFD has the A-A radar and the ground map instead of HARM, the
    /// right one keeps HARM; the ground map can be the SOI and zooms with the zoom keys.
    #[test]
    fn gm_format_rows_and_soi() {
        let mut s = MfdState { modern: true, ..default() };
        let mut raster = MapRaster::default();
        assert!(!s.av.formats(MfdSide::Left).contains(&Format::Gm));
        s.av.radar = true;
        assert_eq!(s.av.formats(MfdSide::Left)[1], Format::Fcr);
        assert_eq!(s.av.formats(MfdSide::Left)[2], Format::Gm);
        assert_eq!(s.av.formats(MfdSide::Right)[1], Format::Harm);
        assert_eq!(osb(MfdSide::Left, 6, &s).map(|o| o.0), Some("FCR"));
        assert_eq!(osb(MfdSide::Left, 7, &s).map(|o| o.0), Some("GM"));
        assert_eq!(osb(MfdSide::Right, 6, &s).map(|o| o.0), Some("HARM"));
        // It searches the air for the HSD with neither of its formats up.
        assert!(s.av.radar_emitting() && !s.av.gm_on());
        apply(&mut s, &mut raster, MfdCmd::Format(MfdSide::Left, Format::Gm));
        assert!(s.av.radar_emitting() && s.av.gm_on());
        apply(&mut s, &mut raster, MfdCmd::Soi(MfdSide::Left));
        assert_eq!(s.av.soi_format(), Some(Format::Gm));
        let r = s.av.gm_range;
        apply(&mut s, &mut raster, MfdCmd::ZoomOut);
        assert_eq!(s.av.gm_range, r + 1);
        apply(&mut s, &mut raster, MfdCmd::GmRange(-1));
        assert_eq!(s.av.gm_range, r);
        apply(&mut s, &mut raster, MfdCmd::RadarStandby);
        assert!(!s.av.radar_emitting() && !s.av.gm_on() && !s.av.fcr_on() && s.av.radar_off());
        assert_eq!(osb(MfdSide::Left, 0, &s).map(|o| o.0), Some("STBY"));
    }

    /// The A-A radar: its own range, scan and antenna buttons, the SOI with the zoom keys on
    /// its range, sharing the ground map's standby; it transmits while it is up.
    #[test]
    fn fcr_format_buttons() {
        let mut s = MfdState { modern: true, ..default() };
        let mut raster = MapRaster::default();
        s.av.radar = true;
        apply(&mut s, &mut raster, MfdCmd::Format(MfdSide::Left, Format::Fcr));
        assert!(s.av.fcr_on() && !s.av.gm_on() && s.av.radar_emitting());
        let label = |s: &MfdState, slot| osb(MfdSide::Left, slot, s).map(|o| o.0);
        assert_eq!(label(&s, 0), Some("CRM"));
        assert_eq!(label(&s, 11), Some("40"));
        assert_eq!(
            (label(&s, 13), label(&s, 14), label(&s, 16)),
            (Some("A6"), Some("4B"), Some("0"))
        );
        apply(&mut s, &mut raster, MfdCmd::FcrRange(1));
        apply(&mut s, &mut raster, MfdCmd::FcrAz);
        apply(&mut s, &mut raster, MfdCmd::FcrBars);
        apply(&mut s, &mut raster, MfdCmd::FcrElevation(1));
        assert_eq!(
            (label(&s, 11), label(&s, 13), label(&s, 14), label(&s, 16)),
            (Some("80"), Some("A3"), Some("2B"), Some("+2"))
        );
        let scan = s.av.fcr_scan();
        assert!((scan.az_half - 30f32.to_radians()).abs() < 1e-6 && scan.bars == 2);
        assert!((scan.elevation - 2f32.to_radians()).abs() < 1e-6);
        apply(&mut s, &mut raster, MfdCmd::Soi(MfdSide::Left));
        assert_eq!(s.av.soi_format(), Some(Format::Fcr));
        apply(&mut s, &mut raster, MfdCmd::ZoomIn);
        assert_eq!(label(&s, 11), Some("40"));
        apply(&mut s, &mut raster, MfdCmd::RadarStandby);
        assert!(!s.av.radar_emitting());
        assert_eq!(label(&s, 0), Some("STBY"));
        // Without the radar it is not offered.
        s.av.set_radar(false);
        assert!(s.av.available().all(|f| !f.radar()));
        assert!(s.av.sides().iter().all(|&side| !s.av.format(side).radar()));
    }

    /// Each master mode comes up with its own formats: NAV the HSD on the left, A-A the A-A
    /// radar and A-G the ground map (as the SOI), then each as the pilot left it.
    #[test]
    fn master_modes_keep_their_formats() {
        let mut s = MfdState { modern: true, ..default() };
        let mut raster = MapRaster::default();
        s.av.set_radar(true);
        assert!(!s.av.set_master(HudMode::Nav));
        assert_eq!(s.av.format(MfdSide::Left), Format::Hsd);
        assert!(s.av.set_master(HudMode::Air));
        assert_eq!(s.av.format(MfdSide::Left), Format::Fcr);
        assert_eq!(s.av.soi_format(), Some(Format::Fcr));
        assert!(s.av.set_master(HudMode::Gnd));
        assert_eq!(s.av.format(MfdSide::Left), Format::Gm);
        assert_eq!(s.av.soi_format(), Some(Format::Gm));
        // The pilot puts the pod on the right in A-G and the map on the left in NAV.
        apply(&mut s, &mut raster, MfdCmd::Format(MfdSide::Right, Format::Tgp));
        s.av.set_master(HudMode::Nav);
        assert_eq!(s.av.format(MfdSide::Left), Format::Hsd);
        assert_eq!(s.av.format(MfdSide::Right), Format::Sms);
        apply(&mut s, &mut raster, MfdCmd::Format(MfdSide::Left, Format::Map));
        s.av.set_master(HudMode::Gnd);
        assert_eq!(
            (s.av.format(MfdSide::Left), s.av.format(MfdSide::Right)),
            (Format::Gm, Format::Tgp)
        );
        s.av.set_master(HudMode::Air);
        assert_eq!(s.av.format(MfdSide::Left), Format::Fcr);
        s.av.set_master(HudMode::Nav);
        assert_eq!(s.av.format(MfdSide::Left), Format::Map);
        // Without a radar A-A and A-G keep the formats up.
        let mut s = MfdState { modern: true, ..default() };
        s.av.set_master(HudMode::Nav);
        s.av.set_master(HudMode::Air);
        assert_eq!(s.av.format(MfdSide::Left), Format::Hsd);
    }

    /// F3 goes round the map formats on the left MFD's own bottom row: the ground map only while
    /// it is on that row.
    #[test]
    fn f3_follows_the_left_row() {
        let mut s = MfdState { modern: true, ..default() };
        let mut raster = MapRaster::default();
        s.av.radar = true;
        let mut cycle = |s: &mut MfdState| {
            let f = s.av.next_left_format().unwrap();
            apply(s, &mut raster, MfdCmd::Format(MfdSide::Left, f));
            f
        };
        assert_eq!(
            [cycle(&mut s), cycle(&mut s), cycle(&mut s), cycle(&mut s)],
            [Format::Fcr, Format::Gm, Format::Map, Format::Hsd]
        );
        // GM moved to the right MFD's row (onto its HARM button), HARM onto the left's.
        s.av.rows[0] = Some(FORMATS);
        s.av.rows[1] = Some([Format::Hsd, Format::Gm, Format::Tgp, Format::Map, Format::Sms]);
        assert_eq!(
            [cycle(&mut s), cycle(&mut s), cycle(&mut s)],
            [Format::Harm, Format::Map, Format::Hsd]
        );
        // A map format up on the other MFD is skipped.
        s.av.select(MfdSide::Right, Format::Map);
        assert_eq!([cycle(&mut s), cycle(&mut s)], [Format::Harm, Format::Hsd]);
    }

    /// A lock on the ground map takes the return under the cursor where it was painted (a site
    /// or a vehicle before a building as near), else the ground there.
    #[test]
    fn gm_lock_picks_returns() {
        use crate::ground_radar::{RadarTarget, Return};
        let size = Vec2::new(HIRES.0 as f32, HIRES.1 as f32);
        let view = HsdView::wedge(DVec2::ZERO, 0.0, 20.0, size);
        let px = |e: f64, n: f64| view.to_screen(e / WU, n / WU);
        let returns = [
            Return::at(0.0, 10_000.0, RadarTarget::Tile(5, 6)),
            Return::at(30.0, 10_000.0, RadarTarget::Unit(4)),
            Return::at(3000.0, 8000.0, RadarTarget::Object(9)),
        ];
        assert_eq!(gm_pick(&returns, &view, px(0.0, 10_000.0), PICK_RADIUS).0, GmPick::Vehicle(4));
        let at = px(3000.0, 8000.0) + Vec2::X * 3.0;
        assert_eq!(gm_pick(&returns, &view, at, PICK_RADIUS).0, GmPick::Object(9));
        let GmPick::Point(x, y) = gm_pick(&returns, &view, px(-5000.0, 5000.0), PICK_RADIUS).0
        else {
            panic!("a ground point")
        };
        let ((e, n), (we, wn)) =
            ((x as f64 * 32.0 * WU, (0x8000 - y) as f64 * 32.0 * WU), (-5000.0, 5000.0));
        assert!((e - we).abs() < 40.0 && (n - wn).abs() < 40.0, "{e} {n}");
    }

    /// The sensor of interest goes round the three MFDs' capable formats.
    #[test]
    fn soi_round_three_mfds() {
        let mut s = MfdState { modern: true, ..default() };
        let mut raster = MapRaster::default();
        s.av.set_mfds(3);
        s.av.select(MfdSide::Right, Format::Harm);
        s.av.select(MfdSide::Third, Format::Tgp);
        assert_eq!(s.av.soi_format(), Some(Format::Hsd));
        apply(&mut s, &mut raster, MfdCmd::SwapSoi);
        assert_eq!(s.av.soi_format(), Some(Format::Harm));
        apply(&mut s, &mut raster, MfdCmd::SwapSoi);
        assert_eq!(s.av.soi_format(), Some(Format::Tgp));
        apply(&mut s, &mut raster, MfdCmd::SwapSoi);
        assert_eq!(s.av.soi_format(), Some(Format::Hsd));
    }

    #[test]
    fn soi_commands() {
        let mut s = MfdState { modern: true, ..default() };
        let mut raster = MapRaster::default();
        s.av.select(MfdSide::Right, Format::Harm);
        apply(&mut s, &mut raster, MfdCmd::Soi(MfdSide::Right));
        assert_eq!(s.av.soi_format(), Some(Format::Harm));
        apply(&mut s, &mut raster, MfdCmd::SwapSoi);
        assert_eq!(s.av.soi_format(), Some(Format::Hsd));
        // Not on a format without a cursor.
        s.av.select(MfdSide::Right, Format::Sms);
        apply(&mut s, &mut raster, MfdCmd::Soi(MfdSide::Right));
        assert_eq!(s.av.soi_format(), Some(Format::Hsd));
    }

    /// The MAP can be the SOI while it shows the waypoint table, and loses it with the table.
    #[test]
    fn way_page_soi() {
        let mut s = MfdState { modern: true, ..default() };
        let mut raster = MapRaster::default();
        s.av.select(MfdSide::Right, Format::Map);
        apply(&mut s, &mut raster, MfdCmd::Soi(MfdSide::Right));
        assert_eq!(s.av.soi_format(), Some(Format::Hsd), "not the map itself");
        assert_eq!(osb(MfdSide::Right, 4, &s).map(|o| o.0), None);
        apply(&mut s, &mut raster, MfdCmd::MapWay);
        assert_eq!(osb(MfdSide::Right, 4, &s).map(|o| o.0), Some("SOI"));
        apply(&mut s, &mut raster, MfdCmd::Soi(MfdSide::Right));
        assert_eq!(s.av.soi_format(), Some(Format::Map));
        apply(&mut s, &mut raster, MfdCmd::MapWay);
        assert_eq!(s.av.soi_format(), Some(Format::Hsd));
    }

    #[test]
    fn every_button_on_every_format() {
        for side in [MfdSide::Left, MfdSide::Third] {
            for f in formats(side, true, false)
                .into_iter()
                .chain(formats(side, false, false))
                .chain(formats(side, false, true))
            {
                for (way, ord) in [(false, false), (true, true)] {
                    let mut s = MfdState { modern: true, ..default() };
                    s.av.set_mfds(3);
                    s.av.select(side, f);
                    s.av.map_way = way;
                    s.av.sms_ord = ord;
                    for slot in 0..crate::mfd::OSB_COUNT {
                        if let Some((label, _, _)) = osb(side, slot, &s) {
                            assert!(label.len() <= 5, "{f:?} {slot}: {label}");
                        }
                    }
                    assert!(osb(side, 5, &s).is_some());
                }
            }
        }
        // The third MFD's bottom row: the HSI first.
        let s = MfdState { modern: true, ..default() };
        assert_eq!(osb(MfdSide::Third, 5, &s).map(|o| o.0), Some("HSI"));
    }

    /// The ground point of an area track.
    fn area_point(tgp: &TgpState) -> DVec3 {
        let TgpMode::Area(p) = tgp.mode else { panic!("{:?}", tgp.mode) };
        p
    }

    #[test]
    fn pod_area_then_point_track() {
        let origin = DVec3::new(0.0, 1000.0, 0.0);
        let mut tgp = TgpState::default();
        let ahead_down = Vec3::new(0.0, -0.5, -1.0).normalize();
        let start = ground_hit(origin, ahead_down, &Terrain::flat(0.0)).unwrap();
        let r = (start - origin).length();
        // Slewing right from where the slaved pod looks: the ground point moves right.
        let p0 = pod_step(
            &mut tgp,
            Vec2::X,
            0.5,
            0,
            Quat::IDENTITY,
            origin,
            Some(ahead_down),
            &Terrain::flat(0.0),
        );
        let p = area_point(&tgp);
        assert!((p.x - r * 0.25).abs() < 1e-3 && (p.z - start.z).abs() < 1e-3, "{p}");
        assert_eq!(p0, Some(p));
        // Zoomed in, slower.
        let mut z = tgp.clone();
        pod_step(&mut z, Vec2::X, 0.5, 2, Quat::IDENTITY, origin, None, &Terrain::flat(0.0));
        let r2 = (p - origin).length();
        let moved = (area_point(&z) - p).length();
        assert!((moved - r2 * 0.0625).abs() < 1e-3, "{moved}");
        // Released: point track on the ground point after a moment.
        pod_step(&mut tgp, Vec2::ZERO, 0.1, 0, Quat::IDENTITY, origin, None, &Terrain::flat(0.0));
        assert!(matches!(tgp.mode, TgpMode::Area(_)));
        let p = pod_step(
            &mut tgp,
            Vec2::ZERO,
            0.1,
            0,
            Quat::IDENTITY,
            origin,
            None,
            &Terrain::flat(0.0),
        );
        let TgpMode::Point(q) = tgp.mode else { panic!("{:?}", tgp.mode) };
        assert_eq!(p, Some(q));
        // It stays on that point as the aircraft moves.
        assert!(pod_direction(&tgp, Quat::IDENTITY, origin + DVec3::Z * 500.0).unwrap().z < -0.0);
    }

    #[test]
    fn pod_slew_ignores_aircraft_motion() {
        // Slewing right while the aircraft pitches and rolls: the ground point moves as it would
        // in level flight.
        let slew = |pitch: f32, roll: f32| {
            let mut origin = DVec3::new(0.0, 1000.0, 0.0);
            let start = DVec3::new(0.0, 0.0, -5000.0);
            let mut tgp = TgpState { mode: TgpMode::Point(start), ..default() };
            for i in 0..20 {
                let t = i as f32 * 0.05;
                let attitude = Quat::from_rotation_x(pitch * t) * Quat::from_rotation_z(roll * t);
                pod_step(&mut tgp, Vec2::X, 0.05, 0, attitude, origin, None, &Terrain::flat(0.0));
                origin += DVec3::new(0.0, 0.0, -12.0);
            }
            area_point(&tgp)
        };
        let level = slew(0.0, 0.0);
        assert!(level.x > 1000.0, "{level}");
        let rolled = slew(0.3, -0.8);
        assert!((rolled - level).length() < 0.01, "{rolled} {level}");
        // Above the horizon it holds its direction while the aircraft turns.
        let origin = DVec3::new(0.0, 1000.0, 0.0);
        let sky = origin + pod_dir(0.0, 0.2).as_dvec3() * POD_SKY;
        let mut tgp = TgpState { mode: TgpMode::Area(sky), ..default() };
        pod_step(&mut tgp, Vec2::Y, 0.2, 0, Quat::IDENTITY, origin, None, &Terrain::flat(0.0));
        let d = pod_direction(&tgp, Quat::IDENTITY, origin).unwrap();
        assert!(d.y > 0.0, "{d}");
        let pitched = Quat::from_rotation_x(0.2);
        pod_step(&mut tgp, Vec2::ZERO, 0.5, 0, pitched, origin, None, &Terrain::flat(0.0));
        assert!((pod_direction(&tgp, pitched, origin).unwrap() - d).length() < 1e-4);
    }

    #[test]
    fn pod_slew_fine_at_shallow_angles() {
        // 25 km out from 1000 m: up and down move the line of sight far less than left and right
        // (the ground point moves alike either way).
        let origin = DVec3::new(0.0, 1000.0, 0.0);
        let start = DVec3::new(0.0, 0.0, -25_000.0);
        let angles =
            |tgp: &TgpState| pod_angles(pod_direction(tgp, Quat::IDENTITY, origin).unwrap());
        let (az0, el0) = pod_angles((start - origin).as_vec3());
        let mut right = TgpState { mode: TgpMode::Point(start), ..default() };
        pod_step(&mut right, Vec2::X, 0.1, 0, Quat::IDENTITY, origin, None, &Terrain::flat(0.0));
        let mut up = TgpState { mode: TgpMode::Point(start), ..default() };
        pod_step(&mut up, Vec2::Y, 0.1, 0, Quat::IDENTITY, origin, None, &Terrain::flat(0.0));
        let daz = angles(&right).0 - az0;
        let del = angles(&up).1 - el0;
        assert!(daz > 0.0 && del > 0.0 && del < daz / 5.0, "{daz} {del}");
        // Still on the ground (farther out), not above the horizon.
        assert!(area_point(&up).z < start.z && area_point(&up).y.abs() < 1e-6);
    }

    #[test]
    fn pod_slew_finer_far_out() {
        // Up close: linear, at once.
        assert_eq!(slew_gain(Vec2::new(0.5, -1.0), Some(1000.0), 0.0), Vec2::new(0.5, -1.0));
        assert_eq!(slew_gain(Vec2::X, None, 0.0), Vec2::X);
        // Far out: a half deflection much finer, a held slew building up to the full rate.
        let far = Some(30_000.0);
        let half = slew_gain(Vec2::new(0.5, 0.0), far, SLEW_RAMP).x;
        assert!((half - 0.125).abs() < 1e-5, "{half}");
        let tap = slew_gain(Vec2::NEG_Y, far, 0.0).y;
        assert!((tap + SLEW_START_FAR).abs() < 1e-5, "{tap}");
        assert_eq!(slew_gain(Vec2::NEG_Y, far, SLEW_RAMP), Vec2::NEG_Y);
        // In between, in between.
        let mid = slew_gain(Vec2::new(0.5, 0.0), Some(11_500.0), SLEW_RAMP).x;
        assert!(mid < 0.5 && mid > 0.125, "{mid}");
        // A slew held far out ramps up; letting go starts over (moves relative to the range).
        let origin = DVec3::new(0.0, 1000.0, 0.0);
        let mut tgp =
            TgpState { mode: TgpMode::Area(DVec3::new(0.0, 0.0, -25_000.0)), ..default() };
        let step = |tgp: &mut TgpState, s: Vec2| {
            let before = area_point(tgp);
            pod_step(tgp, s, 0.1, 0, Quat::IDENTITY, origin, None, &Terrain::flat(0.0));
            (area_point(tgp) - before).length() / (before - origin).length()
        };
        let first = step(&mut tgp, Vec2::X);
        let later = (0..10).map(|_| step(&mut tgp, Vec2::X)).last().unwrap();
        assert!(later > 4.0 * first, "{first} {later}");
        assert!((later - POD_RATE as f64 * 0.1).abs() < 1e-4, "{later}");
        step(&mut tgp, Vec2::ZERO);
        assert!((step(&mut tgp, Vec2::X) - first).abs() < 1e-4);
    }

    #[test]
    fn camera_zoom_steps() {
        use crate::mfd::{CAM_ZOOM_MAX, cam_zoom, cam_zoom_label};
        let mut s = MfdState::default();
        // From the range's zoom.
        cam_zoom(&mut s, MfdCmd::CamZoom(1), 2);
        assert_eq!(s.cam_zoom, Some(3));
        for _ in 0..9 {
            cam_zoom(&mut s, MfdCmd::CamZoom(1), 2);
        }
        assert_eq!(s.cam_zoom, Some(CAM_ZOOM_MAX));
        cam_zoom(&mut s, MfdCmd::CamZoomCycle, 0);
        assert_eq!((s.cam_zoom, cam_zoom_label(s.cam_zoom)), (None, "AUTO"));
        cam_zoom(&mut s, MfdCmd::CamZoomCycle, 4);
        assert_eq!(cam_zoom_label(s.cam_zoom), "1X");
        cam_zoom(&mut s, MfdCmd::CamZoom(-1), 4);
        assert_eq!(s.cam_zoom, Some(0));
        let labels: Vec<_> = (0..=CAM_ZOOM_MAX).map(|z| cam_zoom_label(Some(z))).collect();
        assert_eq!(labels, ["1X", "2X", "4X", "6X", "10X", "15X", "20X"]);
        // By range (map units): 1X close in, 20X at the last step.
        use crate::combat::camera_zoom;
        assert_eq!(
            [camera_zoom(10), camera_zoom(64), camera_zoom(100), camera_zoom(5000)],
            [0, 2, 3, 6]
        );
    }

    /// With a CBU-105 selected the SMS page sets its burst altitude on the right instead of the
    /// ripple; the classic avionics leave it to the simulation's default.
    #[test]
    fn sms_sets_the_cbu105_burst_altitude() {
        let mut s = MfdState { modern: true, ..default() };
        let raster = &mut MapRaster::default();
        s.av.sms_ord = false;
        s.av.select(MfdSide::Right, Format::Sms);
        let label = |s: &MfdState, slot| osb(MfdSide::Right, slot, s).map(|o| o.0);
        assert_eq!(label(&s, 15), Some("SPC+"));
        s.sfw = true;
        assert_eq!(
            [label(&s, 15), label(&s, 16), label(&s, 17)],
            [Some("BA+"), Some("1500FT"), Some("BA-")]
        );
        assert_eq!(label(&s, 10), None, "no ripple");
        assert_eq!(s.burst_ft(), 1500);
        apply(&mut s, raster, MfdCmd::Burst(1));
        apply(&mut s, raster, MfdCmd::Burst(1));
        apply(&mut s, raster, MfdCmd::Burst(1));
        assert_eq!((s.burst_ft(), label(&s, 16)), (3000, Some("3000FT")));
        for _ in 0..9 {
            apply(&mut s, raster, MfdCmd::Burst(-1));
        }
        assert_eq!(s.burst_ft(), 500);
        s.modern = false;
        assert_eq!(s.burst_ft(), 0);
    }

    #[test]
    fn pod_snowplow_and_slave() {
        let mut s = MfdState { modern: true, ..default() };
        let raster = &mut MapRaster::default();
        s.av.tgp.mode = TgpMode::Point(DVec3::new(100.0, 0.0, -2000.0));
        // SP only with the pod the SOI.
        apply(&mut s, raster, MfdCmd::TgpSp);
        assert!(matches!(s.av.tgp.mode, TgpMode::Point(_)) && !s.av.tgp.clear);
        s.av.select(MfdSide::Right, Format::Tgp);
        apply(&mut s, raster, MfdCmd::Soi(MfdSide::Right));
        apply(&mut s, raster, MfdCmd::TgpSp);
        assert_eq!((s.av.tgp.mode, s.av.tgp.clear), (TgpMode::Snowplow, true));
        assert_eq!(osb(MfdSide::Right, 2, &s).map(|o| (o.0, o.2)), Some(("SP", true)));
        // Fixed ahead of the nose, turning with the aircraft, without holding the ground.
        let origin = DVec3::new(0.0, 1000.0, 0.0);
        let turned = Quat::from_rotation_y(-0.5);
        let p =
            pod_step(&mut s.av.tgp, Vec2::ZERO, 1.0, 0, turned, origin, None, &Terrain::flat(0.0));
        assert_eq!(s.av.tgp.mode, TgpMode::Snowplow);
        let d = pod_direction(&s.av.tgp, turned, origin).unwrap();
        assert!((d - turned * pod_dir(0.0, SP_EL)).length() < 1e-5);
        assert!(p.is_some_and(|p| p.x > 0.0));
        // A slew starts from there.
        pod_step(&mut s.av.tgp, Vec2::Y, 0.5, 0, turned, origin, None, &Terrain::flat(0.0));
        let d = pod_direction(&s.av.tgp, turned, origin).unwrap();
        let (az, el) = pod_angles(turned.inverse() * d);
        // (Farther out on the ground, finer at the shallow angle.)
        assert!(az.abs() < 1e-4 && el > SP_EL && el < SP_EL + 0.1, "{az} {el}");
        // A target key slaves it again; CZ (to the steerpoint), but not in A-A.
        apply(&mut s, raster, MfdCmd::TgpSlave);
        assert_eq!(s.av.tgp.mode, TgpMode::Slaved);
        let cz = osb(MfdSide::Right, 3, &s).map(|o| (o.0, o.1));
        assert_eq!(cz, Some(("CZ", OsbFn::Cmd(MfdCmd::TgpCz))));
        apply(&mut s, raster, MfdCmd::TgpCz);
        assert!(s.av.tgp.cz);
        s.air = true;
        assert_eq!(osb(MfdSide::Right, 3, &s), None);
    }

    #[test]
    fn ground_hits() {
        let o = DVec3::new(10.0, 100.0, 0.0);
        assert_eq!(
            ground_hit(o, Vec3::NEG_Y, &Terrain::flat(0.0)),
            Some(DVec3::new(10.0, 0.0, 0.0))
        );
        assert_eq!(ground_hit(o, Vec3::Y, &Terrain::flat(0.0)), None);
        assert_eq!(ground_hit(o, Vec3::new(1.0, -1e-5, 0.0), &Terrain::flat(0.0)), None);
        let (az, el) = pod_angles(pod_dir(0.7, -0.3));
        assert!((az - 0.7).abs() < 1e-5 && (el + 0.3).abs() < 1e-5);
    }
    /// Over the hills a line of sight stops at the hillside, even looking level.
    #[test]
    fn line_of_sight_meets_the_hills() {
        let hills =
            stealth_sim::hills::Hills::from_bytes(vec![255; stealth_sim::hills::DIM.pow(2)])
                .unwrap();
        let ground = Terrain { y: 0.0, hills };
        let o = DVec3::new(200_000.0, 0.0, -300_000.0);
        let o = DVec3::new(o.x, ground.at(o.x, o.z) + 20.0, o.z);
        let p = ground_hit(o, Vec3::new(0.3, -0.2, -1.0), &ground).expect("meets the ground");
        assert!((p.y - ground.at(p.x, p.z)).abs() < 0.05, "{p} {}", ground.at(p.x, p.z));
        // Straight down: the ground under the eye.
        let p = ground_hit(o, Vec3::NEG_Y, &ground).unwrap();
        assert!((o.y - p.y - 20.0).abs() < 0.05);
    }

    #[test]
    fn hsd_projection_round_trips() {
        let v = HsdView::new(DVec2::new(1000.0, 2000.0), 0.7, 40.0, false, Vec2::new(231.0, 180.0));
        let ahead = DVec2::new(1000.0 + 0.7f64.sin() * 5000.0, 2000.0 + 0.7f64.cos() * 5000.0);
        let p = v.to_screen(ahead.x, ahead.y);
        assert!((p.x - v.origin.x).abs() < 1e-3 && p.y < v.origin.y, "ahead is up: {p}");
        let back = v.to_world(Vec2::new(150.0, 40.0));
        assert!((v.to_screen(back.x, back.y) - Vec2::new(150.0, 40.0)).length() < 1e-3);
    }

    #[test]
    fn cursors_follow_a_turn() {
        let size = Vec2::new(HIRES.0 as f32, HIRES.1 as f32);
        let (lo, hi) = (Vec2::splat(-1e4), Vec2::splat(1e4));
        let mut av = AvionicsState::default();
        let before = HsdView::new(DVec2::new(1000.0, 2000.0), 0.0, 40.0, false, size);
        follow_view(&mut av, Some(before), lo, hi);
        let ground = before.to_world(av.hsd_cursor);
        let (centre, r) = harm_scope();
        av.harm_cursor = centre - Vec2::Y * r * 0.5;
        // A right turn of 90 degrees: the HSD cursor stays on its ground point, the HARM cursor
        // (dead ahead before) moves to the left, like an emitter ahead does.
        let after = HsdView::new(
            DVec2::new(1100.0, 2050.0),
            std::f64::consts::FRAC_PI_2,
            40.0,
            false,
            size,
        );
        follow_view(&mut av, Some(after), lo, hi);
        assert!((av.hsd_cursor - after.to_screen(ground.x, ground.y)).length() < 1e-2);
        assert!(
            (av.harm_cursor - (centre - Vec2::X * r * 0.5)).length() < 1e-3,
            "{}",
            av.harm_cursor
        );
    }

    #[test]
    fn lock_picks_the_nearest_lockable() {
        let v = HsdView::new(DVec2::ZERO, 0.0, 20.0, true, Vec2::new(231.0, 180.0));
        let at = |e: f64, n: f64, target| Contact {
            kind: ContactKind::Aircraft,
            east: e,
            north: n,
            alt: 0.0,
            heading: 0,
            friendly: false,
            target,
            threat: None,
            seen: true,
            own: false,
        };
        let contacts = Contacts {
            list: vec![
                at(0.0, 1000.0, Some(ContactRef::Unit(3))),
                at(0.0, 1200.0, Some(ContactRef::Object(7))),
                at(0.0, 1050.0, None),
            ],
            ..default()
        };
        let p = v.to_screen(0.0, 1040.0);
        assert_eq!(pick_contact(&contacts, &v, p, PICK_RADIUS), Some(ContactRef::Unit(3)));
        assert_eq!(pick_contact(&contacts, &v, p + Vec2::X * 40.0, PICK_RADIUS), None);
    }

    #[test]
    fn lock_prefers_the_enemy_over_a_friendly() {
        let v = HsdView::new(DVec2::ZERO, 0.0, 20.0, true, Vec2::new(231.0, 180.0));
        let at = |n: f64, friendly, unit| Contact {
            kind: ContactKind::Aircraft,
            east: 0.0,
            north: n,
            alt: 0.0,
            heading: 0,
            friendly,
            target: Some(ContactRef::Unit(unit)),
            threat: None,
            seen: true,
            own: false,
        };
        let contacts =
            Contacts { list: vec![at(1000.0, true, 1), at(1100.0, false, 2)], ..default() };
        let p = v.to_screen(0.0, 1000.0);
        assert_eq!(pick_contact(&contacts, &v, p, PICK_RADIUS), Some(ContactRef::Unit(2)));
        // A friendly alone is lockable.
        let contacts = Contacts { list: vec![at(1000.0, true, 1)], ..default() };
        assert_eq!(pick_contact(&contacts, &v, p, PICK_RADIUS), Some(ContactRef::Unit(1)));
        // An enemy no radar sees (the own one in standby) is not on the display to lock.
        let hidden = Contact { seen: false, ..at(1100.0, false, 2) };
        let contacts = Contacts { list: vec![at(1000.0, true, 1), hidden], ..default() };
        assert_eq!(pick_contact(&contacts, &v, p, PICK_RADIUS), Some(ContactRef::Unit(1)));
    }
}

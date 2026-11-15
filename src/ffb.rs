//! Force feedback for FFB joystick bases (MOZA AB6 and AB9): the stick centres harder as the
//! airspeed builds, goes light and shakes approaching the stall, rumbles over the runway, kicks on
//! a touchdown and chatters while the gun fires; a helicopter's cyclic holds a steady force trim
//! spring with the rotor's buzz in it.
//!
//! The bases speak the standard USB HID PID force feedback protocol. On Linux the kernel's
//! hid-pidff driver serves it through the base's evdev node (/dev/input/event*), on Windows
//! Windows.Gaming.Input's force feedback motors do. A worker thread finds the base, uploads the
//! effects (two condition effects, spring and damper, and a sine vibration) and updates them as
//! the flight's forces (`Forces`) change, so a slow USB transfer never holds up a frame.
//!
//! The game's trim is added to the stick's input (`flight::Aircraft::stick_trim`), so the spring
//! always centres at the middle. The base's own spring and damping (MOZA Cockpit) add to these.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::flight::{Aircraft, FlightConstants, Player};
use crate::settings::AppState;

/// FFB bases: USB vendor and product ids, name.
pub const BASES: [(u16, u16, &str); 2] =
    [(0x346e, 0x1002, "MOZA AB6"), (0x346e, 0x1000, "MOZA AB9")];

/// Whether this platform drives FFB bases at all.
pub const SUPPORTED: bool = cfg!(any(target_os = "linux", windows));

pub struct FfbPlugin;

impl Plugin for FfbPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Ffb::start()).init_resource::<Feel>().add_systems(PostUpdate, drive);
    }
}

/// The forces on the stick, each 0..1 of the base's strongest.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Forces {
    /// Centring spring (per full deflection).
    pub spring: f32,
    /// Damping (against the stick's speed).
    pub damper: f32,
    /// Vibration amplitude and frequency (Hz).
    pub shake: f32,
    pub shake_hz: f32,
}

/// The stick with nothing to fly (menus, a console, the flight over): a gentle centring.
const IDLE: Forces = Forces { spring: 0.3, damper: 0.1, shake: 0.0, shake_hz: 0.0 };
/// The spring at rest and at `SPRING_KNOTS` and over (the air load on the control surfaces).
const SPRING: (f32, f32) = (0.15, 0.85);
const SPRING_KNOTS: f32 = 350.0;
/// The damping at rest and at `SPRING_KNOTS`.
const DAMPER: (f32, f32) = (0.08, 0.25);
/// How much lighter the spring is deep in the stall buffet (the controls go mushy).
const STALL_SLACK: f32 = 0.5;
/// The stall buffet: amplitude at its edge and deep in it (`shake::stall_buffet` 1), Hz.
const BUFFET: (f32, f32, f32) = (0.12, 0.5, 9.0);
/// The gun: amplitude, Hz.
const GUN: (f32, f32) = (0.3, 18.0);
/// The ground roll: amplitude at `GROUND_KNOTS` and over, Hz at rest and per knot.
const GROUND: (f32, f32, f32) = (0.12, 5.0, 0.08);
const GROUND_KNOTS: f32 = 150.0;
/// A touchdown's kick: amplitude per ft/s of sink, at most; how fast it dies (per s); Hz.
const TOUCHDOWN: (f32, f32, f32, f32) = (1.0 / 12.0, 0.8, 5.0, 7.0);
/// A helicopter: the cyclic's force trim spring and damping, the rotor's buzz (amplitude, Hz)
/// and its shudder through translational lift (extra amplitude, knots from and to).
const HELI_SPRING: f32 = 0.45;
const HELI_DAMPER: f32 = 0.25;
const HELI_BUZZ: (f32, f32) = (0.05, 17.0);
const HELI_ETL: (f32, f32, f32) = (0.1, 12.0, 26.0);

/// What the stick feels in flight.
#[derive(Debug, Clone, Copy, Default)]
pub struct FlightFeel {
    pub knots: f32,
    pub on_ground: bool,
    /// `shake::stall_buffet`.
    pub buffet: f32,
    pub heli: bool,
    pub gun: bool,
    /// The touchdown kick left (0..1).
    pub kick: f32,
}

/// The stick's forces for a flight's state.
pub fn forces(f: &FlightFeel) -> Forces {
    let speed = (f.knots / SPRING_KNOTS).clamp(0.0, 1.0);
    let mut out = if f.heli {
        let etl = if f.on_ground || !(HELI_ETL.1..HELI_ETL.2).contains(&f.knots) {
            0.0
        } else {
            let x = (f.knots - HELI_ETL.1) / (HELI_ETL.2 - HELI_ETL.1);
            HELI_ETL.0 * (x * std::f32::consts::PI).sin()
        };
        Forces {
            spring: HELI_SPRING,
            damper: HELI_DAMPER,
            shake: HELI_BUZZ.0 + etl,
            shake_hz: HELI_BUZZ.1,
        }
    } else {
        let slack = 1.0 - STALL_SLACK * f.buffet.min(1.0);
        Forces {
            spring: (SPRING.0 + (SPRING.1 - SPRING.0) * speed) * slack,
            damper: DAMPER.0 + (DAMPER.1 - DAMPER.0) * speed,
            shake: 0.0,
            shake_hz: 0.0,
        }
    };
    // The strongest vibration wins (two sines at once would only beat).
    let mut shake = |amp: f32, hz: f32| {
        if amp > out.shake {
            (out.shake, out.shake_hz) = (amp, hz);
        }
    };
    if f.buffet > 0.0 {
        let d = f.buffet.min(1.0);
        shake(BUFFET.0 + (BUFFET.1 - BUFFET.0) * d, BUFFET.2);
    }
    if f.on_ground && f.knots > 2.0 {
        shake(GROUND.0 * (f.knots / GROUND_KNOTS).min(1.0), GROUND.1 + GROUND.2 * f.knots);
    }
    if f.gun {
        shake(GUN.0, GUN.1);
    }
    if f.kick > 0.0 {
        shake(f.kick, TOUCHDOWN.3);
    }
    if out.shake < 0.01 {
        (out.shake, out.shake_hz) = (0.0, 0.0);
    }
    out
}

/// The touchdown kick, from one frame to the next.
#[derive(Resource, Default)]
struct Feel {
    kick: f32,
    on_ground: bool,
    climb: f32,
}

/// What the game asks of the worker, and what it found.
#[derive(Default)]
struct Shared {
    forces: Forces,
    /// The player's strength setting (0: off, the effects stopped).
    gain: f32,
    quit: bool,
}

/// The FFB worker.
#[derive(Resource)]
pub struct Ffb {
    shared: Arc<Mutex<Shared>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Ffb {
    fn start() -> Self {
        let shared = Arc::new(Mutex::new(Shared { forces: IDLE, ..default() }));
        #[cfg(any(target_os = "linux", windows))]
        let worker = (!cfg!(test))
            .then(|| {
                let s = shared.clone();
                std::thread::Builder::new()
                    .name("ffb".into())
                    .spawn(move || backend::run(s))
                    .inspect_err(|e| warn!("force feedback: no worker thread: {e}"))
                    .ok()
            })
            .flatten();
        #[cfg(not(any(target_os = "linux", windows)))]
        let worker = None;
        Self { shared, worker }
    }

    fn set(&self, forces: Forces, gain: f32) {
        if let Ok(mut s) = self.shared.lock() {
            s.forces = forces;
            s.gain = gain;
        }
    }
}

impl Drop for Ffb {
    fn drop(&mut self) {
        if let Ok(mut s) = self.shared.lock() {
            s.quit = true;
        }
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn drive(
    ffb: Res<Ffb>,
    mut feel: ResMut<Feel>,
    config: Res<crate::hotas::InputConfig>,
    state: Res<State<AppState>>,
    time: Res<Time<Virtual>>,
    k: Option<Res<FlightConstants>>,
    combat: Option<Res<crate::combat::Combat>>,
    manning: Option<Res<crate::station::Manning>>,
    aircraft: Query<&Aircraft, With<Player>>,
) {
    if ffb.worker.is_none() {
        return;
    }
    let flying = *state.get() == AppState::Flight
        && !manning.as_deref().is_some_and(|m| m.console() || m.gunner());
    let a = aircraft.single().ok().filter(|a| flying && a.ended.is_none());
    let Some(a) = a else {
        *feel = Feel { on_ground: true, ..default() };
        ffb.set(IDLE, config.ffb_strength);
        return;
    };
    let paused = time.is_paused();
    let dt = time.delta_secs();
    feel.kick = (feel.kick - TOUCHDOWN.2 * dt).max(0.0);
    if a.on_ground && !feel.on_ground && a.has_flown {
        // Climb rate in wu/s: 16 per ft/s (as `shake`'s touchdown).
        let sink = (-feel.climb / 16.0).max(0.0);
        feel.kick = feel.kick.max((sink * TOUCHDOWN.0).min(TOUCHDOWN.1));
    }
    feel.on_ground = a.on_ground;
    feel.climb = a.climb_rate;
    let gun = combat
        .as_deref()
        .is_some_and(|c| c.sim.gun_firing() && c.end.is_none() && !c.sim.player_absent());
    let mut f = forces(&FlightFeel {
        knots: a.velocity / 27.0,
        on_ground: a.on_ground,
        buffet: crate::shake::stall_buffet(a),
        heli: k.is_some_and(|k| k.heli.is_some()),
        gun,
        kick: feel.kick,
    });
    if paused {
        (f.shake, f.shake_hz) = (0.0, 0.0);
    }
    ffb.set(f, config.ffb_strength);
}

/// How much a force must change before the worker sends it again (USB transfers are slow).
#[cfg(any(target_os = "linux", windows, test))]
const STEP: f32 = 0.02;

/// Whether a condition (spring, damper) or the vibration changed enough to send again.
#[cfg(any(target_os = "linux", windows, test))]
fn changed(old: Option<Forces>, new: Forces) -> (bool, bool) {
    let Some(o) = old else { return (true, true) };
    let cond = (o.spring - new.spring).abs() >= STEP || (o.damper - new.damper).abs() >= STEP;
    let shake = (o.shake - new.shake).abs() >= STEP
        || (o.shake_hz - new.shake_hz).abs() >= 0.5
        || ((o.shake == 0.0) != (new.shake == 0.0));
    (cond, shake)
}

#[cfg(any(target_os = "linux", windows))]
mod backend {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use bevy::log::{info, warn};

    use super::{Forces, Shared, changed};

    #[cfg(target_os = "linux")]
    use super::linux::Base;
    #[cfg(windows)]
    use super::windows::Base;

    /// How often the worker looks at the forces, and for a base when there is none.
    const TICK: Duration = Duration::from_millis(10);
    const SCAN: Duration = Duration::from_secs(3);
    /// The effects are sent again this often anyway (a base that dropped one picks it up).
    const REFRESH: Duration = Duration::from_secs(20);

    pub fn run(shared: Arc<Mutex<Shared>>) {
        let mut base: Option<Base> = None;
        let mut last_scan: Option<Instant> = None;
        let mut sent: Option<Forces> = None;
        let mut sent_gain = -1.0f32;
        let mut sent_at = Instant::now();
        let mut failed_quietly = false;
        loop {
            std::thread::sleep(TICK);
            let (forces, gain, quit) = match shared.lock() {
                Ok(s) => (s.forces, s.gain.clamp(0.0, 1.0), s.quit),
                Err(_) => return,
            };
            if quit {
                return;
            }
            if base.is_none() {
                if last_scan.is_some_and(|t| t.elapsed() < SCAN) {
                    continue;
                }
                last_scan = Some(Instant::now());
                match Base::find(!failed_quietly) {
                    Some(b) => {
                        info!("force feedback: using {}", b.name());
                        base = Some(b);
                        (sent, sent_gain) = (None, -1.0);
                    }
                    None => {
                        failed_quietly = true;
                        continue;
                    }
                }
            }
            let Some(b) = base.as_mut() else { continue };
            let result = (|| {
                if gain != sent_gain {
                    b.gain(gain)?;
                    sent_gain = gain;
                }
                // Off: the effects stopped, the base to itself.
                if gain <= 0.0 {
                    if sent.is_some() {
                        b.stop()?;
                        sent = None;
                    }
                    return Ok(());
                }
                let refresh = sent_at.elapsed() >= REFRESH;
                let (cond, shake) = changed(sent, forces);
                if cond || refresh {
                    b.conditions(forces.spring, forces.damper)?;
                }
                if shake || refresh {
                    b.shake(forces.shake, forces.shake_hz)?;
                }
                if cond || shake || refresh {
                    sent = Some(forces);
                    sent_at = Instant::now();
                }
                Ok::<_, String>(())
            })();
            if let Err(e) = result {
                warn!("force feedback: {}: {e}", b.name());
                base = None;
            }
        }
    }
}

/// Linux: the base's evdev node, through the kernel's force feedback interface
/// (Documentation/input/ff.rst).
#[cfg(target_os = "linux")]
mod linux {
    use std::fs::{File, OpenOptions};
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;

    use bevy::log::{info, warn};

    const EV_FF: u16 = 0x15;
    const FF_PERIODIC: u16 = 0x51;
    const FF_SPRING: u16 = 0x53;
    const FF_DAMPER: u16 = 0x55;
    const FF_SQUARE: u16 = 0x58;
    const FF_TRIANGLE: u16 = 0x59;
    const FF_SINE: u16 = 0x5a;
    const FF_GAIN: u16 = 0x60;
    /// FF_MAX 0x7f: the feature bits' bytes.
    const FF_BYTES: usize = 16;
    /// The longest replay (and the HID PID's infinite duration).
    const FOREVER: u16 = 0xffff;
    /// The vibration's direction: up (0x8000), along the pitch axis.
    const SHAKE_DIRECTION: u16 = 0x8000;

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct InputId {
        bustype: u16,
        vendor: u16,
        product: u16,
        version: u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Envelope {
        attack_length: u16,
        attack_level: u16,
        fade_length: u16,
        fade_level: u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Condition {
        right_saturation: u16,
        left_saturation: u16,
        right_coeff: i16,
        left_coeff: i16,
        deadband: u16,
        center: i16,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Periodic {
        waveform: u16,
        period: u16,
        magnitude: i16,
        offset: i16,
        phase: u16,
        envelope: Envelope,
        custom_len: u32,
        custom_data: *mut i16,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    union EffectData {
        condition: [Condition; 2],
        periodic: Periodic,
    }

    /// `struct ff_effect`.
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Effect {
        kind: u16,
        id: i16,
        direction: u16,
        trigger: [u16; 2],
        /// Replay length and delay (ms).
        replay: [u16; 2],
        u: EffectData,
    }

    const fn ioc(dir: u32, nr: u32, size: usize) -> u32 {
        (dir << 30) | ((size as u32) << 16) | ((b'E' as u32) << 8) | nr
    }
    const WRITE: u32 = 1;
    const READ: u32 = 2;
    const EVIOCGID: u32 = ioc(READ, 0x02, size_of::<InputId>());
    const fn eviocgname(len: usize) -> u32 {
        ioc(READ, 0x06, len)
    }
    const EVIOCGBIT_FF: u32 = ioc(READ, 0x20 + EV_FF as u32, FF_BYTES);
    const EVIOCSFF: u32 = ioc(WRITE, 0x80, size_of::<Effect>());
    const EVIOCRMFF: u32 = ioc(WRITE, 0x81, size_of::<libc::c_int>());

    fn bit(bits: &[u8; FF_BYTES], b: u16) -> bool {
        bits[b as usize / 8] & (1 << (b % 8)) != 0
    }

    pub struct Base {
        file: File,
        name: String,
        waveform: Option<u16>,
        has_gain: bool,
        spring: Option<i16>,
        damper: Option<i16>,
        shake: Option<i16>,
        shaking: bool,
    }

    impl Base {
        /// The first FFB base's event node with condition effects, opened for writing. `loud`:
        /// say why a base found can't be used.
        pub fn find(loud: bool) -> Option<Self> {
            let mut paths: Vec<_> = std::fs::read_dir("/dev/input")
                .ok()?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("event")))
                .collect();
            paths.sort();
            for path in paths {
                // Its ids first: read-only works without write access too.
                let Ok(f) = File::open(&path) else { continue };
                let mut id = InputId::default();
                // SAFETY: EVIOCGID fills an input_id.
                if unsafe { libc::ioctl(f.as_raw_fd(), EVIOCGID as _, &mut id) } < 0 {
                    continue;
                }
                let Some(model) = super::BASES
                    .iter()
                    .find(|b| (b.0, b.1) == (id.vendor, id.product))
                    .map(|b| b.2)
                else {
                    continue;
                };
                let mut bits = [0u8; FF_BYTES];
                // SAFETY: EVIOCGBIT fills at most FF_BYTES bytes.
                if unsafe { libc::ioctl(f.as_raw_fd(), EVIOCGBIT_FF as _, bits.as_mut_ptr()) } < 0
                    || !bit(&bits, FF_SPRING)
                {
                    // Another of the base's nodes (a throttle on it, its buttons).
                    continue;
                }
                let file = match OpenOptions::new()
                    .read(true)
                    .write(true)
                    .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
                    .open(&path)
                {
                    Ok(f) => f,
                    Err(e) => {
                        if loud {
                            warn!(
                                "force feedback: {model} at {}: {e} (a udev rule giving the \
                                 player write access to it, uaccess, fixes this)",
                                path.display()
                            );
                        }
                        continue;
                    }
                };
                let mut raw = [0u8; 128];
                // SAFETY: EVIOCGNAME writes at most `raw.len()` bytes.
                let n = unsafe {
                    libc::ioctl(file.as_raw_fd(), eviocgname(raw.len()) as _, raw.as_mut_ptr())
                };
                let kernel_name = (n > 0)
                    .then(|| {
                        let len = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
                        String::from_utf8_lossy(&raw[..len]).trim().to_string()
                    })
                    .filter(|n| !n.is_empty());
                let name = format!(
                    "{} [{:04x}:{:04x}] {}",
                    kernel_name.as_deref().unwrap_or(model),
                    id.vendor,
                    id.product,
                    path.display()
                );
                let waveform = [FF_SINE, FF_TRIANGLE, FF_SQUARE]
                    .into_iter()
                    .find(|&w| bit(&bits, FF_PERIODIC) && bit(&bits, w));
                let effects: Vec<_> = [
                    (FF_SPRING, "spring"),
                    (FF_DAMPER, "damper"),
                    (FF_PERIODIC, "periodic"),
                    (FF_GAIN, "gain"),
                ]
                .into_iter()
                .filter(|&(b, _)| bit(&bits, b))
                .map(|(_, n)| n)
                .collect();
                info!("force feedback: {name}: {}", effects.join(", "));
                return Some(Self {
                    file,
                    name,
                    waveform,
                    has_gain: bit(&bits, FF_GAIN),
                    spring: None,
                    damper: bit(&bits, FF_DAMPER).then_some(-1),
                    shake: None,
                    shaking: false,
                });
            }
            None
        }

        pub fn name(&self) -> &str {
            &self.name
        }

        fn write(&self, code: u16, value: i32) -> Result<(), String> {
            // SAFETY: an all-zero input_event is valid; written whole.
            let mut ev: libc::input_event = unsafe { std::mem::zeroed() };
            ev.type_ = EV_FF;
            ev.code = code;
            ev.value = value;
            let size = size_of::<libc::input_event>();
            // SAFETY: writes `size` bytes from `ev`.
            let n = unsafe { libc::write(self.file.as_raw_fd(), (&raw const ev).cast(), size) };
            if n != size as isize {
                return Err(format!("write: {}", std::io::Error::last_os_error()));
            }
            Ok(())
        }

        /// Uploads (or updates, with its id) an effect and plays it if it is new.
        fn upload(&self, id: &mut Option<i16>, mut e: Effect) -> Result<(), String> {
            e.id = id.unwrap_or(-1);
            // SAFETY: EVIOCSFF reads the ff_effect and writes back its id.
            if unsafe { libc::ioctl(self.file.as_raw_fd(), EVIOCSFF as _, &mut e) } < 0 {
                return Err(format!("upload: {}", std::io::Error::last_os_error()));
            }
            if id.is_none() {
                *id = Some(e.id);
                self.write(e.id as u16, 1)?;
            }
            Ok(())
        }

        pub fn gain(&mut self, gain: f32) -> Result<(), String> {
            if self.has_gain {
                self.write(FF_GAIN, (gain * 65535.0) as i32)?;
            }
            Ok(())
        }

        fn condition(kind: u16, k: f32) -> Effect {
            let coeff = (k.clamp(0.0, 1.0) * 32767.0) as i16;
            let c = Condition {
                right_saturation: 0xffff,
                left_saturation: 0xffff,
                right_coeff: coeff,
                left_coeff: coeff,
                deadband: 0,
                center: 0,
            };
            Effect {
                kind,
                id: -1,
                direction: 0,
                trigger: [0; 2],
                replay: [FOREVER, 0],
                u: EffectData { condition: [c; 2] },
            }
        }

        /// The spring and damper on both axes.
        pub fn conditions(&mut self, spring: f32, damper: f32) -> Result<(), String> {
            let mut id = self.spring;
            self.upload(&mut id, Self::condition(FF_SPRING, spring))?;
            self.spring = id;
            if let Some(d) = self.damper {
                let mut id = (d >= 0).then_some(d);
                self.upload(&mut id, Self::condition(FF_DAMPER, damper))?;
                self.damper = id;
            }
            Ok(())
        }

        pub fn shake(&mut self, amp: f32, hz: f32) -> Result<(), String> {
            let Some(waveform) = self.waveform else { return Ok(()) };
            if amp <= 0.0 || hz <= 0.0 {
                if let (Some(id), true) = (self.shake, self.shaking) {
                    self.write(id as u16, 0)?;
                    self.shaking = false;
                }
                return Ok(());
            }
            let e = Effect {
                kind: FF_PERIODIC,
                id: -1,
                direction: SHAKE_DIRECTION,
                trigger: [0; 2],
                replay: [FOREVER, 0],
                u: EffectData {
                    periodic: Periodic {
                        waveform,
                        period: (1000.0 / hz).round().clamp(1.0, 1000.0) as u16,
                        magnitude: (amp.clamp(0.0, 1.0) * 32767.0) as i16,
                        offset: 0,
                        phase: 0,
                        envelope: Envelope::default(),
                        custom_len: 0,
                        custom_data: std::ptr::null_mut(),
                    },
                },
            };
            let mut id = self.shake;
            self.upload(&mut id, e)?;
            if self.shake.is_some() && !self.shaking {
                self.write(id.unwrap() as u16, 1)?;
            }
            self.shake = id;
            self.shaking = true;
            Ok(())
        }

        /// Stops and removes every effect (the base back to its own spring).
        pub fn stop(&mut self) -> Result<(), String> {
            for id in [self.spring.take(), self.damper.filter(|&d| d >= 0), self.shake.take()]
                .into_iter()
                .flatten()
            {
                // SAFETY: EVIOCRMFF takes the effect id by value.
                unsafe { libc::ioctl(self.file.as_raw_fd(), EVIOCRMFF as _, id as libc::c_int) };
            }
            if self.damper.is_some() {
                self.damper = Some(-1);
            }
            self.shaking = false;
            Ok(())
        }
    }

    impl Drop for Base {
        fn drop(&mut self) {
            let _ = self.stop();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// The structs match the kernel's (x86_64 and aarch64): ff_effect 48 bytes,
        /// EVIOCSFF 0x40304580.
        #[test]
        #[cfg(target_pointer_width = "64")]
        fn ff_effect_layout() {
            assert_eq!(size_of::<Condition>(), 12);
            assert_eq!(size_of::<Periodic>(), 32);
            assert_eq!(size_of::<Effect>(), 48);
            assert_eq!(EVIOCSFF, 0x40304580);
            assert_eq!(EVIOCGID, 0x80084502);
            assert_eq!(EVIOCGBIT_FF, 0x80104535);
            assert_eq!(EVIOCRMFF, 0x40044581);
        }
    }
}

/// Windows: Windows.Gaming.Input's force feedback motor of the base's raw game controller.
#[cfg(windows)]
mod windows {
    use windows::Foundation::TimeSpan;
    use windows::Gaming::Input::ForceFeedback::{
        ConditionForceEffect, ConditionForceEffectKind, ForceFeedbackLoadEffectResult,
        ForceFeedbackMotor, PeriodicForceEffect, PeriodicForceEffectKind,
    };
    use windows::Gaming::Input::RawGameController;
    use windows_numerics::Vector3;

    use bevy::log::warn;

    /// The vibration's longest run (it is sent again sooner, `backend::REFRESH`): 10 minutes.
    const FOREVER: TimeSpan = TimeSpan { Duration: 6_000_000_000 };
    /// The vibration's direction: along the pitch axis.
    const SHAKE_AXIS: Vector3 = Vector3 { X: 0.0, Y: 1.0, Z: 0.0 };
    const AXES: [Vector3; 2] =
        [Vector3 { X: 1.0, Y: 0.0, Z: 0.0 }, Vector3 { X: 0.0, Y: 1.0, Z: 0.0 }];

    fn err(e: windows::core::Error) -> String {
        e.message().to_string()
    }

    pub struct Base {
        motor: ForceFeedbackMotor,
        name: String,
        /// Spring and damper per axis (X, Y), loaded and started on first use.
        springs: Option<[ConditionForceEffect; 2]>,
        dampers: Option<[ConditionForceEffect; 2]>,
        shake: Option<PeriodicForceEffect>,
        shaking: bool,
    }

    // SAFETY: the WinRT objects are agile (free-threaded); only the worker thread uses them.
    unsafe impl Send for Base {}

    impl Base {
        pub fn find(loud: bool) -> Option<Self> {
            // SAFETY: joins the thread to the multithreaded apartment (WinRT needs one); harmless
            // if it already is.
            unsafe {
                let _ = windows::Win32::System::WinRT::RoInitialize(
                    windows::Win32::System::WinRT::RO_INIT_MULTITHREADED,
                );
            }
            let list = RawGameController::RawGameControllers().ok()?;
            for i in 0..list.Size().ok()? {
                let Ok(c) = list.GetAt(i) else { continue };
                let (Ok(v), Ok(p)) = (c.HardwareVendorId(), c.HardwareProductId()) else {
                    continue;
                };
                let Some(model) = super::BASES.iter().find(|b| (b.0, b.1) == (v, p)).map(|b| b.2)
                else {
                    continue;
                };
                let motor = c.ForceFeedbackMotors().ok().and_then(|m| m.GetAt(0).ok());
                let Some(motor) = motor else {
                    if loud {
                        warn!(
                            "force feedback: {model} [{v:04x}:{p:04x}] has no force feedback motor"
                        );
                    }
                    continue;
                };
                if !motor.IsEnabled().unwrap_or(true)
                    && let Ok(op) = motor.TryEnableAsync()
                {
                    let _ = op.join();
                }
                let name = c
                    .DisplayName()
                    .map(|n| n.to_string())
                    .ok()
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| model.to_string());
                return Some(Self {
                    motor,
                    name: format!("{name} [{v:04x}:{p:04x}]"),
                    springs: None,
                    dampers: None,
                    shake: None,
                    shaking: false,
                });
            }
            None
        }

        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn gain(&mut self, gain: f32) -> Result<(), String> {
            self.motor.SetMasterGain(gain as f64).map_err(err)
        }

        fn load(
            &self,
            effect: &windows::Gaming::Input::ForceFeedback::IForceFeedbackEffect,
        ) -> Result<(), String> {
            let r = self.motor.LoadEffectAsync(effect).map_err(err)?.join().map_err(err)?;
            if r != ForceFeedbackLoadEffectResult::Succeeded {
                return Err(format!("loading an effect: {r:?}"));
            }
            Ok(())
        }

        fn conditions_of(
            &self,
            kind: ConditionForceEffectKind,
            slot: &mut Option<[ConditionForceEffect; 2]>,
            k: f32,
        ) -> Result<(), String> {
            let k = k.clamp(0.0, 1.0);
            let new = slot.is_none();
            if new {
                let x = ConditionForceEffect::CreateInstance(kind).map_err(err)?;
                let y = ConditionForceEffect::CreateInstance(kind).map_err(err)?;
                *slot = Some([x, y]);
            }
            let effects = slot.as_ref().unwrap();
            for (e, axis) in effects.iter().zip(AXES) {
                e.SetParameters(axis, k, k, 1.0, 1.0, 0.0, 0.0).map_err(err)?;
                if new {
                    use windows::core::Interface;
                    self.load(&e.cast().map_err(err)?)?;
                    e.Start().map_err(err)?;
                }
            }
            Ok(())
        }

        pub fn conditions(&mut self, spring: f32, damper: f32) -> Result<(), String> {
            let mut springs = self.springs.take();
            let r = self.conditions_of(ConditionForceEffectKind::Spring, &mut springs, spring);
            self.springs = springs;
            r?;
            let mut dampers = self.dampers.take();
            let r = self.conditions_of(ConditionForceEffectKind::Damper, &mut dampers, damper);
            self.dampers = dampers;
            r
        }

        pub fn shake(&mut self, amp: f32, hz: f32) -> Result<(), String> {
            if amp <= 0.0 || hz <= 0.0 {
                if let (Some(e), true) = (&self.shake, self.shaking) {
                    e.Stop().map_err(err)?;
                    self.shaking = false;
                }
                return Ok(());
            }
            let new = self.shake.is_none();
            if new {
                let e = PeriodicForceEffect::CreateInstance(PeriodicForceEffectKind::SineWave)
                    .map_err(err)?;
                self.shake = Some(e);
            }
            let e = self.shake.as_ref().unwrap();
            let v = Vector3 { X: SHAKE_AXIS.X * amp, Y: SHAKE_AXIS.Y * amp, Z: 0.0 };
            e.SetParameters(v, hz, 0.0, 0.0, FOREVER).map_err(err)?;
            if new {
                use windows::core::Interface;
                self.load(&e.cast().map_err(err)?)?;
            }
            if new || !self.shaking {
                e.Start().map_err(err)?;
            }
            self.shaking = true;
            Ok(())
        }

        pub fn stop(&mut self) -> Result<(), String> {
            let springs = self.springs.take().into_iter().flatten();
            let dampers = self.dampers.take().into_iter().flatten();
            for e in springs.chain(dampers) {
                let _ = e.Stop();
                use windows::core::Interface;
                if let Ok(i) =
                    e.cast::<windows::Gaming::Input::ForceFeedback::IForceFeedbackEffect>()
                    && let Ok(op) = self.motor.TryUnloadEffectAsync(&i)
                {
                    let _ = op.join();
                }
            }
            if let Some(e) = self.shake.take() {
                let _ = e.Stop();
                use windows::core::Interface;
                if let Ok(i) =
                    e.cast::<windows::Gaming::Input::ForceFeedback::IForceFeedbackEffect>()
                    && let Ok(op) = self.motor.TryUnloadEffectAsync(&i)
                {
                    let _ = op.join();
                }
            }
            self.shaking = false;
            Ok(())
        }
    }

    impl Drop for Base {
        fn drop(&mut self) {
            let _ = self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jet(knots: f32) -> FlightFeel {
        FlightFeel { knots, ..default() }
    }

    /// The stick stiffens with speed and goes light in the stall buffet.
    #[test]
    fn spring_builds_with_speed() {
        let slow = forces(&jet(120.0));
        let fast = forces(&jet(400.0));
        assert!(slow.spring < fast.spring, "{slow:?} {fast:?}");
        assert!((fast.spring - SPRING.1).abs() < 1e-6);
        assert!(fast.damper > slow.damper);
        let stalling = forces(&FlightFeel { buffet: 1.0, ..jet(120.0) });
        assert!(stalling.spring < slow.spring);
        assert!(stalling.shake > 0.3 && stalling.shake_hz == BUFFET.2, "{stalling:?}");
    }

    /// Cruising: no vibration; on the ground it rumbles with speed, the gun outshakes it.
    #[test]
    fn vibrations() {
        assert_eq!(forces(&jet(300.0)).shake, 0.0);
        let parked = FlightFeel { on_ground: true, ..jet(0.0) };
        assert_eq!(forces(&parked).shake, 0.0);
        let rolling = forces(&FlightFeel { on_ground: true, ..jet(140.0) });
        assert!(rolling.shake > 0.05, "{rolling:?}");
        let gun = forces(&FlightFeel { gun: true, on_ground: true, ..jet(140.0) });
        assert_eq!((gun.shake, gun.shake_hz), GUN);
        let kick = forces(&FlightFeel { kick: 0.6, ..jet(140.0) });
        assert_eq!(kick.shake, 0.6);
    }

    /// A helicopter's cyclic: a steady spring, a buzz, a shudder through translational lift.
    #[test]
    fn helicopter_feel() {
        let hover = forces(&FlightFeel { heli: true, ..jet(0.0) });
        let etl = forces(&FlightFeel { heli: true, ..jet(19.0) });
        let cruise = forces(&FlightFeel { heli: true, ..jet(120.0) });
        assert_eq!(hover.spring, HELI_SPRING);
        assert_eq!(cruise.spring, HELI_SPRING);
        assert!(etl.shake > hover.shake + 0.05, "{hover:?} {etl:?}");
        assert_eq!(cruise.shake, HELI_BUZZ.0);
    }

    /// Small changes are not sent again; the vibration starting or stopping always is.
    #[test]
    fn only_real_changes_are_sent() {
        let a = forces(&jet(200.0));
        assert_eq!(changed(None, a), (true, true));
        assert_eq!(changed(Some(a), Forces { spring: a.spring + 0.005, ..a }), (false, false));
        assert_eq!(changed(Some(a), Forces { spring: a.spring + 0.05, ..a }), (true, false));
        assert_eq!(changed(Some(a), Forces { shake: 0.005, shake_hz: 9.0, ..a }), (false, true));
    }
}

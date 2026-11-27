//! The A-6's instrument panel (`PanelStyle::A6`, the pilot's seat; `PanelStyle::A6Bn`, the
//! bombardier/navigator's), after the A-6A's (NAVAIR 01-85ADA-1, figures FO-2, FO-3, 8-15, 8-28,
//! 8-30 and the armament panel's). One panel across both seats, laid out in millimetres from the
//! cabin's centre line (x right, y down from the panel's middle height), life size
//! (`tools/models/build_cockpit_a6.py`):
//! - The pilot's side (x < 0, his centre line at x -360): the vertical display indicator with
//!   its four lights and the slip ball, the horizontal display's scope in its heading ring under
//!   it with the TIME and RANGE windows; left of them the angle of attack indicator and the
//!   radar altimeter, the engines' four pairs of vertical tapes (RPM, EGT, fuel flow, oil
//!   pressure), the gear lights and the flap, speed brake, autopilot and ILS keys; right of them
//!   the Mach / airspeed indicator, the standby attitude indicator and the accelerometer, the
//!   altimeter, the HSI and the rate of climb indicator, the turn and slip indicator, the clock
//!   and the fuel quantity, the caution lights; the pilot's control box under those (VERTICAL:
//!   the DISPLAY and RANGE MILES pushbuttons, pitch trim and path brightness; HORIZONTAL: mode,
//!   one scan, erase, power; TRACK RADAR: clearance, display scale, function) with the sight's
//!   modes beside it.
//! - Between the seats (0 < x < 240): the armament panel (the station
//!   switches, the attack mode and G pushbuttons, the FT OR SEC counter, master arm, the release
//!   mode), the flight / navigation mode panel under it, and the countermeasures column (the
//!   release keys and counts, the jammers, the threat lights, the radar warning receiver's
//!   round scope under them).
//! - The bombardier/navigator's side (x > 240, his centre line at x 360): the direct view radar
//!   indicator in its heading ring with its lights and range window, the search radar's
//!   controls under it, the sensor scope right of it; the computer keyboard on the pedestal
//!   between his knees (y >= 300: its own surface).
//!
//! Both cockpits have the whole panel, live: from either seat the other's instruments,
//! displays and controls work as in his own.
//!
//! The pushbuttons' and switches' printed legends are label faces beside them, every text at
//! one size (`label_size`), so equal texts share a face of the atlas.

use super::layout::{A6Screens, Dial, Effect, Elem, PanelLayout, PanelStyle, Placed, PxRect, rect};
use crate::input::Action;
use crate::intruder::{
    ACTION_KEYS, ATTACK_MODES, Address, FLIGHT_MODES, Key, Light, NAV_BUTTONS, NAV_LABELS, PULL_GS,
    QUANTITY_KEYS, Readout, STATIONS, VdiDisplay, VdiRange,
};
use crate::panel::{LampId, LedId};

/// A square pushbutton's cap (mm).
const PUSH: f32 = 12.0;
/// A toggle switch's plate.
const SWITCH: (f32, f32) = (16.0, 26.0);
/// White legends on the black panels; black legends on the grey keys.
const LEGEND: (u8, u8) = (15, 5);
const KEYCAP: (u8, u8) = (5, 7);

/// A legend's size by its text alone (so equal texts are one atlas face): 8 mm a line and 4 mm
/// a letter at the font's double size.
fn label_size(text: &str) -> (f32, f32) {
    let lines = text.split('\n').count() as f32;
    let widest = text.split('\n').map(|l| l.chars().count()).max().unwrap_or(1) as f32;
    (4.0 * widest + 3.0, if lines > 1.0 { 6.0 * lines + 1.0 } else { 8.0 })
}

/// Where a legend is anchored across.
#[derive(Clone, Copy)]
enum At {
    Left,
    Centre,
    Right,
}

struct Items(Vec<Placed>);

impl Items {
    fn put(&mut self, elem: Elem, rect: PxRect, cap: Option<Effect>, round: bool) {
        self.0.push(Placed { elem, rect, cap, round });
    }

    /// A legend with its middle height at `y`, its left edge, centre or right edge at `x`.
    fn legend(&mut self, text: &'static str, x: f32, y: f32, at: At) {
        let (w, h) = label_size(text);
        let x = match at {
            At::Left => x,
            At::Centre => x - w / 2.0,
            At::Right => x - w,
        };
        self.put(Elem::Label(text, LEGEND.0, LEGEND.1), rect(x, y - h / 2.0, w, h), None, false);
    }

    /// An A-6 control with its face.
    fn control(&mut self, key: Key, r: PxRect) {
        let round = key.kind() == "knob";
        self.put(Elem::Control(key), r, Some(Effect::Intruder(key)), round);
    }

    /// A square pushbutton with its top left corner at (x, y).
    fn push(&mut self, key: Key, x: f32, y: f32) {
        self.control(key, rect(x, y, PUSH, PUSH));
    }

    /// A toggle switch with its top left corner at (x, y).
    fn switch(&mut self, key: Key, x: f32, y: f32) {
        self.control(key, rect(x, y, SWITCH.0, SWITCH.1));
    }

    /// A switch's positions' legends from its top, right of it (`right`) or left of it.
    fn positions(&mut self, x: f32, y: f32, texts: &[&'static str], right: bool) {
        let step = SWITCH.1 / texts.len() as f32;
        for (i, text) in texts.iter().enumerate() {
            let cy = y + step * (i as f32 + 0.5);
            if right {
                self.legend(text, x + SWITCH.0 + 2.0, cy, At::Left);
            } else {
                self.legend(text, x - 2.0, cy, At::Right);
            }
        }
    }

    /// A key doing a game action, with its legend.
    fn key(&mut self, text: &'static str, r: PxRect, effect: Effect) {
        self.put(Elem::Label(text, KEYCAP.0, KEYCAP.1), r, Some(effect), false);
    }
}

/// Between the seats, in both cockpits: the armament panel, the flight / navigation mode panel
/// under it and the countermeasures column.
fn centre(it: &mut Items) {
    use Effect as E;
    use LampId::*;
    // --- The armament panel (x 4..152, y -196..40).
    it.legend("ARMAMENT", 80.0, -188.0, At::Centre);
    it.legend("BOMBS", 80.0, -171.0, At::Centre);
    for n in 1..=STATIONS {
        it.switch(Key::Station(n), 18.0 + 26.0 * (n - 1) as f32, -164.0);
    }
    it.legend("ROCKETS", 80.0, -131.0, At::Centre);
    it.put(Elem::Light(Light::Reselect), rect(60.0, -123.0, 40.0, 11.0), None, false);
    it.legend("ATTACK", 30.0, -104.0, At::Centre);
    for (i, mode) in ATTACK_MODES.into_iter().enumerate() {
        let y = -96.0 + 17.0 * i as f32;
        let key = Key::Attack(mode);
        it.legend(key.label(), 40.0, y + PUSH / 2.0, At::Right);
        it.push(key, 42.0, y);
    }
    it.legend("G", 134.0, -104.0, At::Centre);
    for (i, g) in PULL_GS.into_iter().enumerate() {
        let y = -96.0 + 17.0 * i as f32;
        let key = Key::PullG(g);
        it.legend(key.label(), 126.0, y + PUSH / 2.0, At::Right);
        it.push(key, 128.0, y);
    }
    it.legend("FT OR SEC", 86.0, -100.0, At::Centre);
    it.put(Elem::Readout(Readout::Interval), rect(66.0, -93.0, 40.0, 18.0), None, false);
    for (i, key) in [Key::IntervalUp, Key::IntervalDown].into_iter().enumerate() {
        it.control(key, rect(72.0, -66.0 + 14.0 * i as f32, 28.0, 12.0));
    }
    it.put(Elem::Hazard, rect(8.0, 8.0, 30.0, 6.0), None, false);
    it.switch(Key::MasterArm, 15.0, 15.0);
    it.legend("MASTER\nARM", 35.0, 28.0, At::Left);
    it.switch(Key::ReleaseMode, 122.0, 12.0);
    it.positions(122.0, 12.0, &["SALVO", "TRN", "STEP"], false);

    // --- The flight / navigation mode panel (x 4..152, y 46..236).
    it.legend("FLT", 46.0, 53.0, At::Centre);
    it.legend("MODE", 79.0, 53.0, At::Centre);
    it.legend("NAV", 110.0, 53.0, At::Centre);
    for (i, mode) in FLIGHT_MODES.into_iter().enumerate() {
        let y = 62.0 + 20.0 * i as f32;
        let key = Key::Flight(mode);
        it.legend(key.label(), 38.0, y + PUSH / 2.0, At::Right);
        it.push(key, 40.0, y);
    }
    for n in 0..NAV_BUTTONS {
        let y = 62.0 + 20.0 * n as f32;
        it.legend(NAV_LABELS[0][n as usize], 102.0, y + PUSH / 2.0, At::Right);
        it.push(Key::Nav(n), 104.0, y);
        it.legend(NAV_LABELS[1][n as usize], 119.0, y + PUSH / 2.0, At::Left);
    }
    it.switch(Key::NavCheckPoint, 40.0, 198.0);
    it.positions(40.0, 198.0, &["CHK PT", "NORM"], false);
    // The transfer switch lies across: its plate turned on its side.
    it.control(Key::NavTransfer, rect(98.0, 203.0, SWITCH.1, SWITCH.0));
    it.legend("XFER", 111.0, 226.0, At::Centre);

    // --- The countermeasures column (x 156..236, y -196..40).
    it.legend("ECM", 195.0, -188.0, At::Centre);
    for (i, (label, action, led, act)) in [
        ("CHAFF", Action::Chaff, LedId::Chaff, ChaffAct),
        ("FLARE", Action::Flare, LedId::Flares, FlareAct),
        ("DECOY", Action::Decoy, LedId::Decoys, DecoyAct),
    ]
    .into_iter()
    .enumerate()
    {
        let y = -178.0 + 16.0 * i as f32;
        it.key(label, rect(158.0, y, 30.0, 13.0), E::Action(action));
        it.put(Elem::Led(led), rect(190.0, y, 24.0, 13.0), None, false);
        it.put(Elem::Lamp(act, None), rect(216.0, y, 16.0, 13.0), None, false);
    }
    for (i, (id, action)) in [(Ecm, Action::Ecm), (Irj, Action::IrJammer)].into_iter().enumerate() {
        let r = rect(158.0 + 38.0 * i as f32, -126.0, 36.0, 13.0);
        it.put(Elem::Lamp(id, None), r, Some(E::Action(action)), false);
    }
    for (i, elem) in [
        Elem::Lamp(Jam, None),
        Elem::Lamp(EmvWarn, Some("EMV")),
        Elem::Lamp(Rad, Some("LAUNCH")),
        Elem::Lamp(Trak, Some("TRACK")),
        Elem::Lamp(Ir, Some("IR")),
        Elem::Lamp(Lock, None),
    ]
    .into_iter()
    .enumerate()
    {
        let r = rect(158.0 + 38.0 * (i % 2) as f32, -110.0 + 15.0 * (i / 2) as f32, 36.0, 13.0);
        it.put(elem, r, None, false);
    }
    it.put(Elem::Gauge, rect(158.0, -62.0, 74.0, 13.0), None, false);
}

/// The pilot's side.
fn pilot(it: &mut Items) {
    use Effect as E;
    use LampId::*;
    // Round the vertical display: its lights at the lower corners, the slip ball under it.
    for (light, x, y) in [
        (Light::VdiAttack, -444.0, -50.0),
        (Light::VdiPlatform, -444.0, -37.0),
        (Light::VdiInRange, -316.0, -50.0),
        (Light::VdiComputer, -316.0, -37.0),
    ] {
        it.put(Elem::Light(light), rect(x, y, 40.0, 11.0), None, false);
    }
    it.put(Elem::Dial(Dial::Slip), rect(-396.0, -47.0, 72.0, 18.0), None, false);
    // The horizontal display's TIME and RANGE windows over its heading ring.
    it.legend("TIME", -438.0, 16.0, At::Left);
    it.put(Elem::Readout(Readout::PhdTime), rect(-416.0, 9.0, 30.0, 14.0), None, false);
    it.legend("RANGE", -282.0, 16.0, At::Right);
    it.put(Elem::Readout(Readout::PhdRange), rect(-346.0, 9.0, 38.0, 14.0), None, false);

    // Left of the displays: angle of attack, radar altimeter, the engines' tapes, the gear
    // lights and the keys under them.
    it.put(Elem::Dial(Dial::AoaUnits), rect(-530.0, -184.0, 58.0, 58.0), None, true);
    it.put(Elem::Dial(Dial::RadarAlt), rect(-548.0, -118.0, 76.0, 76.0), None, true);
    for (i, dial) in
        [Dial::TapeRpm, Dial::TapeEgt, Dial::TapeFlow, Dial::TapeOil].into_iter().enumerate()
    {
        it.put(Elem::Dial(dial), rect(-662.0 + 44.0 * i as f32, -28.0, 38.0, 112.0), None, false);
    }
    it.put(Elem::GearLights, rect(-726.0, 98.0, 40.0, 32.0), None, false);
    for (i, (id, action)) in
        [(Flaps, Action::Flaps), (Brake, Action::Brakes), (Auto, Action::Autopilot)]
            .into_iter()
            .enumerate()
    {
        let r = rect(-662.0 + 44.0 * i as f32, 98.0, 38.0, 14.0);
        it.put(Elem::Lamp(id, None), r, Some(E::Action(action)), false);
    }
    it.key("ILS", rect(-530.0, 98.0, 38.0, 14.0), E::Action(Action::Ils));
    for (i, (label, effect)) in [
        ("MENU", E::Action(Action::Menu)),
        ("NVG", E::Action(Action::Nvg)),
        ("VOL", E::Action(Action::Volume)),
        ("TIME", E::TimeToggle),
    ]
    .into_iter()
    .enumerate()
    {
        it.key(label, rect(-662.0 + 44.0 * i as f32, 118.0, 38.0, 14.0), effect);
    }

    // Right of the displays: the flight instruments in three rows, the caution lights.
    it.put(Elem::Dial(Dial::Mach), rect(-250.0, -188.0, 76.0, 76.0), None, true);
    it.put(Elem::Dial(Dial::GMeter), rect(-88.0, -180.0, 54.0, 54.0), None, true);
    let mc = Some(E::Action(Action::MasterCaution));
    it.put(Elem::Lamp(Mw, Some("MASTER\nCAUTION")), rect(-32.0, -186.0, 30.0, 18.0), mc, false);
    it.put(Elem::Lamp(Stall, None), rect(-32.0, -164.0, 30.0, 12.0), None, false);
    it.put(Elem::Dial(Dial::Altimeter), rect(-250.0, -106.0, 76.0, 76.0), None, true);
    it.put(Elem::Dial(Dial::Climb), rect(-84.0, -100.0, 56.0, 56.0), None, true);
    it.put(Elem::Dial(Dial::TurnSlip), rect(-246.0, -24.0, 58.0, 58.0), None, true);
    it.put(Elem::Dial(Dial::Clock), rect(-180.0, -18.0, 46.0, 46.0), None, true);
    it.put(Elem::Dial(Dial::Fuel), rect(-128.0, -24.0, 58.0, 58.0), None, true);
    const CAUTION: (u8, u8) = (16, 43);
    for (i, elem) in [
        Elem::Lamp(Fire, None),
        Elem::Lamp(Eng, Some("ENGINE")),
        Elem::Lamp(Fc, Some("FLT\nCTL")),
        Elem::Lamp(Fuel, None),
        Elem::Lamp(LowFuel, Some("LOW\nFUEL")),
        Elem::Lamp(Av, None),
        Elem::Label("HYD\nPRESS", CAUTION.0, CAUTION.1),
        Elem::Label("OIL\nPRESS", CAUTION.0, CAUTION.1),
        Elem::Label("CANOPY", CAUTION.0, CAUTION.1),
        Elem::Label("HOOK", CAUTION.0, CAUTION.1),
    ]
    .into_iter()
    .enumerate()
    {
        let r = rect(-64.0 + 31.0 * (i % 2) as f32, -40.0 + 15.0 * (i / 2) as f32, 29.0, 13.0);
        it.put(elem, r, None, false);
    }

    // --- The pilot's control box (x -254..-52, y 42..238).
    it.legend("VERTICAL", -152.0, 49.0, At::Centre);
    it.legend("DISPLAY", -226.0, 49.0, At::Centre);
    for (i, d) in [
        VdiDisplay::ContactAnalog,
        VdiDisplay::TerrainClearance,
        VdiDisplay::Standby,
        VdiDisplay::Off,
    ]
    .into_iter()
    .enumerate()
    {
        let y = 58.0 + 17.0 * i as f32;
        let key = Key::Vdi(d);
        it.legend(key.label(), -219.0, y + PUSH / 2.0, At::Right);
        it.push(key, -217.0, y);
    }
    it.legend("PITCH TRIM", -152.0, 60.0, At::Centre);
    it.control(Key::PitchTrim, rect(-163.0, 66.0, 22.0, 22.0));
    it.legend("PATH BRIGHT", -152.0, 95.0, At::Centre);
    it.control(Key::PathBright, rect(-163.0, 101.0, 22.0, 22.0));
    it.legend("RANGE\nMILES", -76.0, 50.0, At::Centre);
    for (i, r) in [VdiRange::Mi4, VdiRange::Mi3, VdiRange::Mi2, VdiRange::Mi1, VdiRange::Test]
        .into_iter()
        .enumerate()
    {
        let y = 58.0 + 15.0 * i as f32;
        let key = Key::Range(r);
        it.legend(key.label(), -84.0, y + PUSH / 2.0, At::Right);
        it.push(key, -82.0, y);
    }
    it.legend("HORIZONTAL", -152.0, 139.0, At::Centre);
    it.switch(Key::PhdMode, -212.0, 146.0);
    it.positions(-212.0, 146.0, &["TCPPI", "E-SCAN", "SRCH"], false);
    it.control(Key::OneScan, rect(-171.0, 146.0, 38.0, 13.0));
    it.control(Key::Erase, rect(-171.0, 161.0, 38.0, 13.0));
    it.switch(Key::PhdPower, -112.0, 146.0);
    it.positions(-112.0, 146.0, &["ON", "STBY", "OFF"], true);
    it.legend("TRACK RADAR", -152.0, 183.0, At::Centre);
    it.legend("CLEAR", -224.0, 191.0, At::Centre);
    it.switch(Key::Clearance, -212.0, 197.0);
    it.positions(-212.0, 197.0, &["200", "500", "1000"], false);
    it.legend("SCALE MI", -152.0, 192.0, At::Centre);
    it.switch(Key::DisplayScale, -160.0, 198.0);
    it.positions(-160.0, 198.0, &["5", "10"], true);
    it.switch(Key::TrackFunction, -112.0, 197.0);
    it.positions(-112.0, 197.0, &["DISPLAY", "TRACK", "BORE SGT"], true);
    // The sight's modes beside the box (the game's HUD modes).
    it.legend("SIGHT", -24.0, 49.0, At::Centre);
    for (i, (id, label, mode)) in
        [(Nav, None, 0), (Air, Some("A-A"), 1), (Gnd, Some("A-G"), 2)].into_iter().enumerate()
    {
        let r = rect(-42.0, 58.0 + 18.0 * i as f32, 36.0, 14.0);
        it.put(Elem::Lamp(id, label), r, Some(E::HudMode(mode)), false);
    }
}

/// The bombardier/navigator's side.
fn bn(it: &mut Items) {
    use Effect as E;
    // The direct view radar indicator's lights and range window under its heading ring.
    for (light, x, y, w) in [
        (Light::DvriDiscreteLock, 250.0, 76.0, 46.0),
        (Light::DvriGroundLock, 298.0, 76.0, 46.0),
        (Light::DvriAttack, 346.0, 76.0, 40.0),
        (Light::DvriTrack, 388.0, 76.0, 30.0),
        (Light::DvriComputer, 250.0, 91.0, 46.0),
        (Light::DvriEcm, 298.0, 91.0, 46.0),
        (Light::DvriReselect, 346.0, 91.0, 40.0),
    ] {
        it.put(Elem::Light(light), rect(x, y, w, 13.0), None, false);
    }
    it.legend("RANGE", 452.0, 80.0, At::Centre);
    it.put(Elem::Readout(Readout::DvriRange), rect(426.0, 87.0, 52.0, 17.0), None, false);
    // The search radar's controls.
    it.legend("SEARCH RADAR", 365.0, 118.0, At::Centre);
    it.switch(Key::SearchPower, 262.0, 132.0);
    it.positions(262.0, 132.0, &["ON", "STBY", "OFF"], true);
    it.legend("RANGE", 365.0, 129.0, At::Centre);
    it.control(Key::SearchRange, rect(350.0, 136.0, 30.0, 30.0));
    for (text, x, y) in [("15", 342.0, 164.0), ("30", 342.0, 140.0), ("60", 388.0, 140.0)] {
        it.legend(text, x, y, if x < 365.0 { At::Right } else { At::Left });
    }
    it.legend("150", 388.0, 164.0, At::Left);
    it.switch(Key::Amti, 440.0, 132.0);
    it.positions(440.0, 132.0, &["AMTI", "OFF"], false);
    // Under the sensor scope: the master caution and the keys (the pilot's own, the same
    // size, so they share his faces).
    let mc = Some(E::Action(Action::MasterCaution));
    it.put(
        Elem::Lamp(LampId::Mw, Some("MASTER\nCAUTION")),
        rect(508.0, 12.0, 30.0, 18.0),
        mc,
        false,
    );
    for (i, (label, action)) in
        [("MENU", Action::Menu), ("NVG", Action::Nvg), ("VOL", Action::Volume)]
            .into_iter()
            .enumerate()
    {
        it.key(label, rect(542.0 + 40.0 * i as f32, 14.0, 38.0, 14.0), E::Action(action));
    }
    // The camera pod's button under them.
    it.control(Key::Camera, rect(542.0, 32.0, 38.0, 14.0));

    // --- The computer keyboard on the pedestal (x 286..444, y 300..442).
    it.legend("TEMPORARY STORAGE", 344.0, 306.0, At::Centre);
    it.put(Elem::Readout(Readout::Storage), rect(302.0, 312.0, 84.0, 16.0), None, false);
    it.control(Key::Clear, rect(404.0, 308.0, 22.0, 22.0));
    for (i, a) in Address::ALL.into_iter().enumerate() {
        let (col, row) = ((i % 4) as f32, (i / 4) as f32);
        it.control(Key::Address(a), rect(292.0 + 26.0 * col, 334.0 + 21.0 * row, 24.0, 19.0));
    }
    it.control(Key::Release, rect(398.0, 334.0, 14.0, 61.0));
    for n in 0..QUANTITY_KEYS {
        let (col, row) = ((n % 7) as f32, (n / 7) as f32);
        it.control(Key::Quantity(n), rect(292.0 + 20.0 * col, 400.0 + 16.0 * row, 18.0, 14.0));
    }
    for n in 0..ACTION_KEYS {
        it.control(Key::Action(n), rect(292.0 + 23.0 * n as f32, 432.0, 21.0, 12.0));
    }
}

/// The vertical display's tube face and the horizontal display's ring's square (the pilot's);
/// the direct view radar indicator's ring's square and the sensor scope (the
/// bombardier/navigator's).
const VDI: PxRect = rect(-435.0, -168.0, 150.0, 112.0);
const PHD: PxRect = rect(-442.0, 28.0, 164.0, 164.0);
const DVRI: PxRect = rect(255.0, -152.0, 220.0, 220.0);
const SENSOR: PxRect = rect(520.0, -104.0, 124.0, 93.0);
/// The radar warning receiver's round scope between the seats, under the countermeasures.
const RWR: PxRect = rect(160.0, -36.0, 72.0, 72.0);

impl PanelLayout {
    /// The whole panel, both seats' sides and the middle, as either seat sees it: the other
    /// seat's instruments and displays are live too (see the module).
    fn a6_panel(style: PanelStyle) -> Self {
        let mut it = Items(Vec::new());
        pilot(&mut it);
        centre(&mut it);
        bn(&mut it);
        Self {
            style,
            items: it.0,
            adi: rect(-166.0, -186.0, 72.0, 72.0),
            mfds: Vec::new(),
            rwr: Some(RWR),
            hsi: Some(rect(-168.0, -108.0, 80.0, 80.0)),
            tfr: None,
            ded: None,
            lmpd: None,
            rmpd: None,
            a6: Some(A6Screens {
                vdi: Some(VDI),
                phd: Some(PHD),
                dvri: Some(DVRI),
                sensor: Some(SENSOR),
            }),
            f14: None,
        }
    }

    /// The A-6's pilot's cockpit (see the module).
    pub fn a6() -> Self {
        Self::a6_panel(PanelStyle::A6)
    }

    /// The A-6's bombardier/navigator's cockpit (see the module).
    pub fn a6_bn() -> Self {
        Self::a6_panel(PanelStyle::A6Bn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both seats have the whole panel: every control of the A-6 once in each cockpit.
    #[test]
    fn every_key_has_a_control() {
        for l in [PanelLayout::a6(), PanelLayout::a6_bn()] {
            let keys: Vec<Key> = l
                .items
                .iter()
                .filter_map(|p| match p.cap {
                    Some(Effect::Intruder(k)) => Some(k),
                    _ => None,
                })
                .collect();
            for k in Key::all() {
                assert_eq!(keys.iter().filter(|p| **p == k).count(), 1, "{:?}: {k:?}", l.style);
            }
        }
    }

    /// Faces with the same key (equal legends) are the same size: they share a face of the
    /// atlas. Two with a control are one control in each seat's reach (the master caution, the
    /// MENU, NVG and VOL keys).
    #[test]
    fn equal_faces_are_one_size() {
        for l in [PanelLayout::a6(), PanelLayout::a6_bn()] {
            for (i, a) in l.items.iter().enumerate() {
                for b in &l.items[..i] {
                    if a.elem == b.elem {
                        assert_eq!((a.rect.w, a.rect.h), (b.rect.w, b.rect.h), "{:?}", a.elem);
                        assert_eq!(a.cap, b.cap, "{:?} twice", a.elem);
                    }
                }
            }
        }
    }
}

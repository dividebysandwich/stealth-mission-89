//! The F-14B's instrument panels (`PanelStyle::F14`, the pilot's front cockpit;
//! `PanelStyle::F14Rio`, the radar intercept officer's rear one), laid out in millimetres from
//! each panel's centre (x right, y down), life size (`tools/models/build_cockpit_f14.py`).
//!
//! - The pilot's: the ACM panel across the top (the ACM jettison guard, the GUN RATE, SW COOL,
//!   MSL PREP and MSL MODE lit buttons, master arm, the master caution and the station lights)
//!   between the angle of attack indicator's tape and the wing sweep indicator with its mode
//!   switch and handle keys, the caution lights outboard of the one and the threat lights
//!   outboard of the other; the vertical display indicator (VDI) in the middle with the
//!   vertical speed, airspeed, radar and barometric altimeters and the landing checklist left
//!   of it, the standby attitude indicator, the accelerometer, the bearing-distance-heading
//!   indicator and the radar warning scope right of it; under them the horizontal
//!   situation display (HSD) between the engine tapes and the fuel quantity, its BINGO and
//!   TOTAL counters and the clock; the DISPLAYS panel low on the right (the steering MODE and
//!   STEER CMD pushbuttons that turn 45 degrees, the HUD's, VDI's and HSD's switches); the gear
//!   lights, hook handle and countermeasures keys on the left knee panel.
//! - The RIO's: the detail data display (DDD) high in the middle with its range buttons over
//!   it, its display mode buttons either side, the radar's knobs, the antenna elevation
//!   indicator and the CCM buttons left, the thumbwheels and the WCS mode buttons right; the
//!   tactical information display (TID) under it between its symbol buttons, its knobs and hook
//!   button under it; the armament panel left of the TID, the ECM display and the threat lights
//!   right of it; the flight instruments low on the left, the LANTIRN panel low on the right.
//!
//! The switches' printed legends are label faces beside them, every text at one size
//! (`label_size`), so equal texts share a face of the atlas.

use super::layout::{
    Dial, Effect, Elem, F14Screens, PanelLayout, PanelStyle, Placed, PxRect, rect,
};
use crate::input::Action;
use crate::panel::{LampId, LedId};
use crate::tomcat::{
    CCMS, DDD_MODES, DDD_RANGES, Key, LTS_MODES, Light, Readout, RioKnob, STEER_CMDS, STEER_MODES,
    TID_OPTIONS, WCS_MODES,
};

/// A steering pushbutton's cap (the A-6's), a lit button's (mm).
const PUSH: f32 = 12.0;
/// A toggle switch's plate.
const SWITCH: (f32, f32) = (16.0, 26.0);
/// A knob.
const KNOB: f32 = 22.0;
/// White legends on the black panels; black legends on the grey keys.
const LEGEND: (u8, u8) = (15, 5);
const KEYCAP: (u8, u8) = (5, 7);

/// The pilot's landing checklist's placard.
const CHECKLIST: &str = "LAND\nARMT\nFUEL\nHOOK\nANTI SKID\nWINGS\nWHEELS\nFLAPS\nTHROT";

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

    /// An F-14 control with its face.
    fn control(&mut self, key: Key, r: PxRect) {
        let round = key.kind() == "knob";
        self.put(Elem::Tomcat(key), r, Some(Effect::Tomcat(key)), round);
    }

    /// A steering pushbutton with its top left corner at (x, y).
    fn push(&mut self, key: Key, x: f32, y: f32) {
        self.control(key, rect(x, y, PUSH, PUSH));
    }

    /// A lit pushbutton (w x h) with its top left corner at (x, y).
    fn lamp(&mut self, key: Key, x: f32, y: f32, w: f32, h: f32) {
        self.control(key, rect(x, y, w, h));
    }

    /// A toggle switch with its top left corner at (x, y).
    fn switch(&mut self, key: Key, x: f32, y: f32) {
        self.control(key, rect(x, y, SWITCH.0, SWITCH.1));
    }

    /// A knob with its top left corner at (x, y) and its name centred over it.
    fn knob(&mut self, key: Key, x: f32, y: f32) {
        self.control(key, rect(x, y, KNOB, KNOB));
        let (_, h) = label_size(key.label());
        self.legend(key.label(), x + KNOB / 2.0, y - h / 2.0 - 1.5, At::Centre);
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

/// The menu, night vision and volume keys (and the time key), each seat's.
fn keys(it: &mut Items, x: f32, y: f32, time: bool) {
    use Effect as E;
    let mut keys = vec![
        ("MENU", E::Action(Action::Menu)),
        ("NVG", E::Action(Action::Nvg)),
        ("VOL", E::Action(Action::Volume)),
    ];
    if time {
        keys.push(("TIME", E::TimeToggle));
    }
    for (i, (label, effect)) in keys.into_iter().enumerate() {
        it.key(label, rect(x + 42.0 * i as f32, y, 38.0, 14.0), effect);
    }
}

/// The pilot's ACM panel, the AOA and wing sweep indicators either side of it.
fn acm(it: &mut Items) {
    use Effect as E;
    // The ACM jettison button's guard, the four lit buttons with their names over them, master
    // arm with its guard's stripes, the master caution under the buttons, the station lights.
    it.put(Elem::Hazard, rect(-148.0, -270.0, 22.0, 5.0), None, false);
    it.switch(Key::AcmCover, -145.0, -262.0);
    it.legend("ACM\nJETT", -137.0, -224.0, At::Centre);
    for (i, (key, name)) in [
        (Key::GunRate, "GUN RATE"),
        (Key::SwCool, "SW COOL"),
        (Key::MslPrep, "MSL PREP"),
        (Key::MslMode, "MSL MODE"),
    ]
    .into_iter()
    .enumerate()
    {
        let x = -110.0 + 45.0 * i as f32;
        it.legend(name, x + 15.0, -263.0, At::Centre);
        it.lamp(key, x, -256.0, 30.0, 16.0);
    }
    it.put(Elem::Hazard, rect(70.0, -270.0, 22.0, 5.0), None, false);
    it.switch(Key::MasterArm, 73.0, -262.0);
    it.positions(73.0, -262.0, &["ON", "OFF", "TNG"], true);
    it.legend("MASTER ARM", 81.0, -228.0, At::Centre);
    let mc = Some(E::Action(Action::MasterCaution));
    it.put(
        Elem::Lamp(LampId::Mw, Some("MASTER\nCAUTION")),
        rect(-18.0, -234.0, 36.0, 18.0),
        mc,
        false,
    );
    for n in 0..crate::tomcat::STATIONS.len() as u8 {
        let x = -100.0 + 25.0 * n as f32;
        it.put(Elem::TomcatLight(Light::Station(n)), rect(x, -210.0, 20.0, 12.0), None, false);
    }
    // Angle of attack (units, a tape) on the left, the wing sweep indicator on the right with the
    // sweep mode switch and the handle's keys.
    it.put(Elem::Dial(Dial::AoaTape), rect(-192.0, -268.0, 30.0, 86.0), None, false);
    it.put(Elem::Dial(Dial::Sweep), rect(130.0, -268.0, 32.0, 86.0), None, false);
    it.switch(Key::SweepMode, 172.0, -264.0);
    it.positions(172.0, -264.0, &["AUTO", "MAN"], true);
    it.key("AFT", rect(172.0, -232.0, 28.0, 13.0), E::Action(Action::WingSweepAft));
    it.key("FWD", rect(172.0, -216.0, 28.0, 13.0), E::Action(Action::WingSweepForward));
    it.legend("SWEEP", 186.0, -195.0, At::Centre);
}

/// The pilot's flight instruments round the VDI, the cautions and threat lights outboard.
fn flight(it: &mut Items) {
    use LampId::*;
    // Clear of the VDI's housing, which stands out of the panel with its knobs either side of
    // the tube: the small vertical speed indicator, the large radar altimeter.
    it.put(Elem::Dial(Dial::Climb), rect(-272.0, -172.0, 60.0, 60.0), None, true);
    it.put(Elem::Dial(Dial::Mach), rect(-202.0, -180.0, 76.0, 76.0), None, true);
    it.put(Elem::Dial(Dial::RadarAlt), rect(-294.0, -100.0, 82.0, 82.0), None, true);
    it.put(Elem::Dial(Dial::Altimeter), rect(-202.0, -96.0, 76.0, 76.0), None, true);
    it.put(Elem::Dial(Dial::GMeter), rect(206.0, -176.0, 62.0, 62.0), None, true);
    // The landing checklist's placard left of the radar altimeter.
    it.legend(CHECKLIST, -324.0, -62.0, At::Centre);
    // The VDI's housing's trim knobs' names (the knobs are the model's).
    it.legend("HUD\nTRIM", 102.0, -128.0, At::Centre);
    it.legend("VDI\nTRIM", 102.0, -74.0, At::Centre);
    // The caution lights high on the left, the threat lights high on the right, over the fire
    // handles.
    for (i, elem) in [
        Elem::Lamp(Fire, None),
        Elem::Lamp(Eng, Some("ENGINE")),
        Elem::Lamp(Fc, Some("FLT\nCTL")),
        Elem::Lamp(Fuel, None),
        Elem::Lamp(LowFuel, Some("LOW\nFUEL")),
        Elem::Lamp(Stall, None),
    ]
    .into_iter()
    .enumerate()
    {
        let r = rect(-272.0 + 31.0 * (i % 2) as f32, -264.0 + 15.0 * (i / 2) as f32, 29.0, 13.0);
        it.put(elem, r, None, false);
    }
    for (i, elem) in [
        Elem::Lamp(Lock, None),
        Elem::Lamp(Rad, Some("LAUNCH")),
        Elem::Lamp(Trak, Some("TRACK")),
        Elem::Lamp(Ir, Some("IR")),
        Elem::Lamp(Jam, None),
        Elem::Lamp(EmvWarn, Some("EMV")),
    ]
    .into_iter()
    .enumerate()
    {
        let r = rect(226.0 + 31.0 * (i % 2) as f32, -264.0 + 15.0 * (i / 2) as f32, 29.0, 13.0);
        it.put(elem, r, None, false);
    }
}

/// The pilot's lower panel: the engine tapes and the keys under them, the fuel and the clock,
/// the knee panel's gear lights, hook and countermeasures.
fn lower(it: &mut Items) {
    use Effect as E;
    use LampId::*;
    for (i, dial) in [Dial::TapeRpm, Dial::TapeEgt, Dial::TapeFlow].into_iter().enumerate() {
        it.put(Elem::Dial(dial), rect(-250.0 + 42.0 * i as f32, 8.0, 36.0, 112.0), None, false);
    }
    for (i, (id, action)) in
        [(Flaps, Action::Flaps), (Brake, Action::Brakes), (Auto, Action::Autopilot)]
            .into_iter()
            .enumerate()
    {
        let r = rect(-250.0 + 42.0 * i as f32, 128.0, 38.0, 14.0);
        it.put(Elem::Lamp(id, None), r, Some(E::Action(action)), false);
    }
    keys(it, -250.0, 148.0, true);
    it.key("ILS", rect(-250.0, 168.0, 38.0, 14.0), E::Action(Action::Ils));
    // Fuel: the quantity indicator, the BINGO and TOTAL counters, the clock under them.
    it.put(Elem::Dial(Dial::FuelQuantity), rect(88.0, 4.0, 80.0, 80.0), None, false);
    for (i, r) in [Readout::Bingo, Readout::FuelTotal].into_iter().enumerate() {
        let y = 14.0 + 30.0 * i as f32;
        it.legend(r.label(), 205.0, y, At::Centre);
        it.put(Elem::TomcatReadout(r), rect(180.0, y + 6.0, 50.0, 15.0), None, false);
    }
    it.put(Elem::Dial(Dial::Clock), rect(245.0, 12.0, 46.0, 46.0), None, true);
    // The left knee panel: the gear lights, the dispensers and the jammer.
    it.put(Elem::GearLights, rect(-345.0, 6.0, 40.0, 32.0), None, false);
    for (i, (label, action, led)) in
        [("CHAFF", Action::Chaff, LedId::Chaff), ("FLARE", Action::Flare, LedId::Flares)]
            .into_iter()
            .enumerate()
    {
        let y = 100.0 + 16.0 * i as f32;
        it.key(label, rect(-345.0, y, 30.0, 13.0), E::Action(action));
        it.put(Elem::Led(led), rect(-313.0, y, 24.0, 13.0), None, false);
    }
    it.put(
        Elem::Lamp(Ecm, None),
        rect(-345.0, 134.0, 36.0, 13.0),
        Some(E::Action(Action::Ecm)),
        false,
    );
}

/// The arresting hook's panel: where its parts are drawn (the layout's left, top: off the
/// instrument panel; the model sets the panel on the right knee panel over the DISPLAYS panel)
/// and its size.
pub const HOOK_PANEL: (f32, f32, f32, f32) = (500.0, 14.0, 66.0, 80.0);
/// The hook handle's slot on its panel (left, top, width, height from the panel's corner): the
/// control's footprint.
pub const HOOK_SLOT: (f32, f32, f32, f32) = (40.0, 20.0, 14.0, 54.0);
/// The emergency release's placard.
const HOOK_PLACARD: &str = "EMERG\nDOWN\nPULL\nTURN";

/// The pilot's arresting hook panel: the handle in its slot (up: the hook up; down: lowered)
/// between UP and DN, HOOK over it, the emergency release's placard left of the slot.
fn hook(it: &mut Items) {
    let (x, y, w, _) = HOOK_PANEL;
    let (sx, sy, sw, sh) = HOOK_SLOT;
    it.control(Key::Hook, rect(x + sx, y + sy, sw, sh));
    it.legend("HOOK", x + w - 2.0, y + 7.0, At::Right);
    it.legend("UP", x + sx - 3.0, y + 12.0, At::Right);
    it.legend("DN", x + sx - 3.0, y + sy + sh + 1.0, At::Right);
    let (pw, ph) = label_size(HOOK_PLACARD);
    it.put(
        Elem::Label(HOOK_PLACARD, LEGEND.0, LEGEND.1),
        rect(x + 20.0 - pw / 2.0, y + 44.0 - ph / 2.0, pw, ph),
        None,
        false,
    );
}

/// The pilot's DISPLAYS panel (the model sets it on the right knee panel, which is narrow): the
/// steering MODE pushbuttons down its left, the HUD's, the VDI's and the HSD's pairs of
/// switches in three rows right of them, the STEER CMD row and the pitch ladder's knob under
/// them.
fn displays(it: &mut Items) {
    it.legend("DISPLAYS", 170.0, 112.0, At::Centre);
    it.legend("MODE", 136.0, 126.0, At::Centre);
    for (i, m) in STEER_MODES.into_iter().enumerate() {
        let y = 132.0 + 16.0 * i as f32;
        let key = Key::Steer(m);
        it.legend(key.label(), 128.0, y + PUSH / 2.0, At::Right);
        it.push(key, 130.0, y);
    }
    // A row a display: its name, then each switch under its own.
    for (i, (name, left, right)) in [
        (
            "HUD",
            (Key::HudDeclutter, "DCLTR", &["ON", "OFF"][..]),
            (Key::HudAwl, "AWL", &["ILS", "ACL"][..]),
        ),
        (
            "VDI",
            (Key::VdiMode, "MODE", &["TV", "NORM"][..]),
            (Key::VdiAwl, "AWL", &["ILS", "ACL"][..]),
        ),
        (
            "HSD",
            (Key::HsdMode, "MODE", &["NAV", "TID", "ECM"][..]),
            (Key::HsdEcmOverride, "ORIDE", &["ON", "OFF"][..]),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let y = 134.0 + 38.0 * i as f32;
        it.legend(name, 153.0, y + SWITCH.1 / 2.0, At::Centre);
        for (x, (key, title, pos)) in [(164.0, left), (208.0, right)] {
            it.legend(title, x + SWITCH.0 / 2.0, y - 5.0, At::Centre);
            it.switch(key, x, y);
            it.positions(x, y, pos, true);
        }
    }
    // The STEER CMD row.
    for (i, c) in STEER_CMDS.into_iter().enumerate() {
        let x = 98.0 + 27.0 * i as f32;
        let key = Key::SteerCmd(c);
        it.legend(key.label(), x + PUSH / 2.0, 248.0, At::Centre);
        it.push(key, x, 258.0);
    }
    it.legend("STEER CMD", 158.0, 280.0, At::Centre);
    it.control(Key::PitchLadder, rect(224.0, 246.0, KNOB, KNOB));
    it.legend("PITCH LAD", 224.0, 280.0, At::Centre);
}

/// The RIO's radar panel round the DDD.
fn radar(it: &mut Items) {
    // The range buttons over the DDD.
    it.legend("RANGE", 0.0, -266.0, At::Centre);
    for (i, r) in DDD_RANGES.into_iter().enumerate() {
        it.lamp(Key::DddRange(r), -87.5 + 30.0 * i as f32, -292.0, 20.0, 14.0);
    }
    // The display mode buttons either side of it.
    for (i, m) in DDD_MODES.into_iter().enumerate() {
        let x = if i < 3 { -102.0 } else { 74.0 };
        it.lamp(Key::DddMode(m), x, -248.0 + 24.0 * (i % 3) as f32, 28.0, 18.0);
    }
    // The aspect and closing speed switches, the brightness, gain and erase knobs.
    it.switch(Key::Aspect, -132.0, -176.0);
    it.positions(-132.0, -176.0, &["NOSE", "BEAM", "TAIL"], false);
    it.legend("ASPECT", -124.0, -186.0, At::Centre);
    it.switch(Key::VcScale, 116.0, -176.0);
    it.positions(116.0, -176.0, &["2X", "NORM"], true);
    it.legend("Vc", 124.0, -186.0, At::Centre);
    it.knob(Key::Knob(RioKnob::Bright), 104.0, -288.0);
    it.knob(Key::Knob(RioKnob::PulseGain), -100.0, -150.0);
    it.knob(Key::Knob(RioKnob::Erase), 78.0, -150.0);
    // Left: the threshold and jamming knobs, the antenna elevation indicator and its keys, the
    // CCM buttons.
    it.legend("PD THRLD", -296.0, -296.0, At::Centre);
    it.knob(Key::Knob(RioKnob::PdClear), -328.0, -278.0);
    it.knob(Key::Knob(RioKnob::PdClutter), -286.0, -278.0);
    it.knob(Key::Knob(RioKnob::JamJet), -328.0, -232.0);
    it.knob(Key::Knob(RioKnob::AcmThreshold), -286.0, -232.0);
    it.legend("CCM MODES", -290.0, -194.0, At::Centre);
    for (i, c) in CCMS.into_iter().enumerate() {
        it.lamp(Key::Ccm(c), -334.0 + 32.0 * i as f32, -186.0, 28.0, 18.0);
    }
    it.put(Elem::Dial(Dial::AntennaEl), rect(-238.0, -292.0, 30.0, 130.0), None, false);
    it.legend("EL", -223.0, -152.0, At::Centre);
    it.control(Key::AntennaUp, rect(-204.0, -244.0, 28.0, 13.0));
    it.control(Key::AntennaDown, rect(-204.0, -228.0, 28.0, 13.0));
    // Right: the IR gain and volume knobs, the thumbwheels, the WCS mode buttons.
    it.knob(Key::Knob(RioKnob::IrGain), 142.0, -288.0);
    it.knob(Key::Knob(RioKnob::Volume), 180.0, -288.0);
    it.knob(Key::Knob(RioKnob::MslAudio), 218.0, -288.0);
    for (i, key) in [Key::XmtrChan, Key::MslChan].into_iter().enumerate() {
        let x = 268.0 + 34.0 * i as f32;
        it.legend(key.label(), x + 8.0, -290.0, At::Centre);
        it.control(key, rect(x, -278.0, 16.0, 22.0));
    }
    it.legend("WCS MODE", 228.0, -232.0, At::Centre);
    for (i, m) in WCS_MODES.into_iter().enumerate() {
        let (x, y) = (148.0 + 46.0 * (i % 4) as f32, -224.0 + 24.0 * (i / 4) as f32);
        it.lamp(Key::Wcs(m), x, y, 40.0, 20.0);
    }
    // The IFF interrogator in the WCS buttons' free eighth place.
    it.lamp(Key::Iff, 148.0 + 46.0 * 3.0, -200.0, 40.0, 20.0);
}

/// The RIO's TID's buttons and knobs, the armament panel left of it, the ECM panel right of it,
/// the flight instruments and the LANTIRN panel under them.
fn tid(it: &mut Items) {
    use Effect as E;
    use LampId::*;
    for (i, o) in TID_OPTIONS.into_iter().enumerate() {
        let x = if i < 4 { -158.0 } else { 126.0 };
        it.lamp(Key::TidOption(o), x, -86.0 + 30.0 * (i % 4) as f32, 32.0, 20.0);
    }
    it.knob(Key::TidMode, -110.0, 152.0);
    it.knob(Key::TidRange, -40.0, 152.0);
    it.knob(Key::TidReadout, 30.0, 152.0);
    it.lamp(Key::TidHook, 84.0, 152.0, 30.0, 18.0);

    // The armament panel.
    it.legend("ARMAMENT", -264.0, -94.0, At::Centre);
    it.switch(Key::PhAct, -334.0, -84.0);
    it.positions(-334.0, -84.0, &["ON", "OFF"], true);
    it.legend("PH ACT", -326.0, -50.0, At::Centre);
    it.put(Elem::TomcatLight(Light::MslPrep), rect(-280.0, -82.0, 34.0, 18.0), None, false);
    it.knob(Key::MissileOption, -330.0, -26.0);
    it.lamp(Key::Launch, -280.0, -30.0, 40.0, 24.0);
    // The weapon select and the TARPS camera's buttons beside the master caution light.
    it.lamp(Key::WeaponSelect, -288.0, 18.0, 40.0, 20.0);
    it.lamp(Key::Camera, -288.0, 44.0, 40.0, 20.0);
    let mc = Some(E::Action(Action::MasterCaution));
    it.put(Elem::Lamp(Mw, Some("MASTER\nCAUTION")), rect(-334.0, 16.0, 36.0, 18.0), mc, false);
    for (i, (label, action)) in
        [("MENU", Action::Menu), ("NVG", Action::Nvg), ("VOL", Action::Volume)]
            .into_iter()
            .enumerate()
    {
        let y = 40.0 + 18.0 * i as f32;
        it.key(label, rect(-334.0, y, 38.0, 14.0), E::Action(action));
    }

    // The ECM panel: the deception jammer's knob, the threat lights, the dispensers.
    it.knob(Key::Alq126, 300.0, 10.0);
    for (i, elem) in [
        Elem::Lamp(Rad, Some("LAUNCH")),
        Elem::Lamp(Trak, Some("TRACK")),
        Elem::Lamp(Ir, Some("IR")),
        Elem::Lamp(Jam, None),
        Elem::Lamp(Lock, None),
        Elem::Lamp(EmvWarn, Some("EMV")),
    ]
    .into_iter()
    .enumerate()
    {
        let r = rect(196.0 + 31.0 * (i % 2) as f32, 4.0 + 15.0 * (i / 2) as f32, 29.0, 13.0);
        it.put(elem, r, None, false);
    }
    for (i, (label, action, led, act)) in [
        ("CHAFF", Action::Chaff, LedId::Chaff, ChaffAct),
        ("FLARE", Action::Flare, LedId::Flares, FlareAct),
    ]
    .into_iter()
    .enumerate()
    {
        let y = 56.0 + 16.0 * i as f32;
        it.key(label, rect(196.0, y, 30.0, 13.0), E::Action(action));
        it.put(Elem::Led(led), rect(228.0, y, 24.0, 13.0), None, false);
        it.put(Elem::Lamp(act, None), rect(254.0, y, 16.0, 13.0), None, false);
    }
    it.put(
        Elem::Lamp(Ecm, None),
        rect(196.0, 90.0, 36.0, 13.0),
        Some(E::Action(Action::Ecm)),
        false,
    );

    // The flight instruments low on the left.
    it.put(Elem::Dial(Dial::Airspeed), rect(-334.0, 128.0, 60.0, 60.0), None, true);
    it.put(Elem::Dial(Dial::Altimeter), rect(-268.0, 128.0, 60.0, 60.0), None, true);
    it.put(Elem::Dial(Dial::Clock), rect(-200.0, 134.0, 46.0, 46.0), None, true);

    // The LANTIRN panel low on the right.
    it.legend("LANTIRN", 232.0, 122.0, At::Centre);
    it.put(Elem::Hazard, rect(222.0, 128.0, 22.0, 5.0), None, false);
    for (i, (key, pos)) in [
        (Key::LtsPower, &["ON", "STBY", "OFF"][..]),
        (Key::LtsVideo, &["FLIR", "TCS"][..]),
        (Key::LaserArm, &["ARM", "SAFE"][..]),
        (Key::Polarity, &["BHOT", "WHOT"][..]),
        (Key::Fov, &["NAR", "WIDE"][..]),
    ]
    .into_iter()
    .enumerate()
    {
        let x = 130.0 + 44.0 * i as f32;
        it.switch(key, x, 136.0);
        it.positions(x, 136.0, pos, true);
        it.legend(key.label(), x + 12.0, 170.0, At::Centre);
    }
    for (i, m) in LTS_MODES.into_iter().enumerate() {
        it.lamp(Key::LtsMode(m), 136.0 + 36.0 * i as f32, 182.0, 30.0, 18.0);
    }
    it.lamp(Key::LtsOper, 208.0, 182.0, 30.0, 18.0);
    it.put(Elem::TomcatLight(Light::LaserArmed), rect(250.0, 182.0, 36.0, 18.0), None, false);
}

/// The pilot's VDI's tube face and HSD's tube square.
const VDI: PxRect = rect(-75.0, -180.0, 150.0, 150.0);
const HSD: PxRect = rect(-70.0, 0.0, 140.0, 140.0);
/// The RIO's DDD tube face, TID and ECMD tube squares.
const DDD: PxRect = rect(-62.5, -252.0, 125.0, 125.0);
const TID: PxRect = rect(-115.0, -100.0, 230.0, 230.0);
const ECMD: PxRect = rect(200.0, -98.0, 90.0, 90.0);

impl PanelLayout {
    /// The F-14's pilot's cockpit (see the module).
    pub fn f14() -> Self {
        let mut it = Items(Vec::new());
        acm(&mut it);
        flight(&mut it);
        lower(&mut it);
        displays(&mut it);
        hook(&mut it);
        Self {
            style: PanelStyle::F14,
            items: it.0,
            adi: rect(124.0, -178.0, 72.0, 72.0),
            mfds: Vec::new(),
            rwr: Some(rect(210.0, -94.0, 72.0, 72.0)),
            hsi: Some(rect(122.0, -98.0, 80.0, 80.0)),
            tfr: None,
            ded: None,
            lmpd: None,
            rmpd: None,
            a6: None,
            f14: Some(F14Screens { vdi: Some(VDI), hsd: Some(HSD), ..Default::default() }),
        }
    }

    /// The F-14's radar intercept officer's cockpit (see the module).
    pub fn f14_rio() -> Self {
        let mut it = Items(Vec::new());
        radar(&mut it);
        tid(&mut it);
        Self {
            style: PanelStyle::F14Rio,
            items: it.0,
            adi: rect(-334.0, 194.0, 50.0, 50.0),
            mfds: Vec::new(),
            rwr: None,
            hsi: None,
            tfr: None,
            ded: None,
            lmpd: None,
            rmpd: None,
            a6: None,
            f14: Some(F14Screens {
                ddd: Some(DDD),
                tid: Some(TID),
                ecmd: Some(ECMD),
                ..Default::default()
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tomcat::DddMode;

    /// Each cockpit has its own seat's controls, every one of them once.
    #[test]
    fn every_key_has_a_control() {
        for l in [PanelLayout::f14(), PanelLayout::f14_rio()] {
            let keys: Vec<Key> = l
                .items
                .iter()
                .filter_map(|p| match p.cap {
                    Some(Effect::Tomcat(k)) => Some(k),
                    _ => None,
                })
                .collect();
            let pilot = l.style == PanelStyle::F14;
            for k in Key::all() {
                let want = (k.pilot() == pilot) as usize;
                assert_eq!(keys.iter().filter(|p| **p == k).count(), want, "{:?}: {k:?}", l.style);
            }
        }
    }

    /// Faces with the same key (equal legends) are the same size: they share a face of the
    /// atlas; two with a control are the same control.
    #[test]
    fn equal_faces_are_one_size() {
        for l in [PanelLayout::f14(), PanelLayout::f14_rio()] {
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

    /// The display mode buttons are the DDD's six.
    #[test]
    fn ddd_mode_buttons() {
        let l = PanelLayout::f14_rio();
        let n = l
            .items
            .iter()
            .filter(|p| matches!(p.cap, Some(Effect::Tomcat(Key::DddMode(_)))))
            .count();
        assert_eq!(n, DDD_MODES.len());
        assert!(DDD_MODES.contains(&DddMode::Pd));
    }
}

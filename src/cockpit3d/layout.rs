//! Instrument panel layouts: the faces drawn into the instrument atlas (lamps, readouts,
//! labels, bezel buttons), which of them are buttons, and the displays.
//!
//! The cockpit model (`model`, the Blender files) places the faces; their rectangles here set
//! each face's size in the atlas. The classic panel's (`classic`, the DOS-era arrangement
//! with the 77x60 MFDs) are in 320x200 screen pixels, where the faces were in its picture.
//!
//! The F-117A's modern panel (`modern`) and the F-16's (`f16`) are not on that plane: their
//! rectangles are millimetres on their own panels (x right, y down from the panel's centre).
//! Nor is the UH-60A's (`uh60`, the modern faces rearranged). Their models are generated from
//! these layouts (`tools/models/build_cockpit_f117.py`, `build_cockpit_f16.py`,
//! `build_cockpit_uh60.py`, `build_cockpit_ah1z.py`, `build_cockpit_tornado.py`,
//! `build_cockpit_a10.py`, `build_cockpit_rah66.py`, `build_cockpit_av8b.py`, `build_cockpit_oh58.py` and `build_cockpit_ch47.py` read `assets-src/blend/cockpit_<name>_layout.json`, which a test keeps
//! in step with the layouts).

use crate::input::Action;
use crate::mfd::{LeftMfd, MfdSide, OSB_COUNT};
use crate::panel::{LampId, LedId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelStyle {
    Classic,
    Modern,
    F16,
    /// The UH-60A's: the modern faces after the UH-60M's panel, two MFDs.
    Uh60,
    /// The AH-1Z's: the UH-60A's narrowed to a tandem cockpit.
    Ah1z,
    /// The Tornado IDS's modernised front cockpit: three MFDs across the panel.
    Tornado,
    /// The A-10C's: two MFDs either side of the large attitude indicator.
    A10,
    /// The RAH-66's glass cockpit: two large MFDs side by side, a portrait multipurpose display
    /// outboard of each (warnings left, engines right), the keyboard under them.
    Rah66,
    /// The AV-8B Night Attack's: two MFDs either side of the up-front controller.
    Av8b,
    /// The OH-58D's: side by side, one wide panel with two large MFDs either side of the
    /// standby instruments, the CDU down the pedestal.
    Oh58,
    /// The CH-47F's: side by side, one wide panel with two MFDs in front of each pilot (the
    /// copilot's dark) and a large engine display between them.
    Ch47,
}

/// A rectangle in panel pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PxRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub const fn rect(x: f32, y: f32, w: f32, h: f32) -> PxRect {
    PxRect { x, y, w, h }
}

impl PxRect {
    #[cfg(test)]
    fn overlaps(&self, o: &PxRect) -> bool {
        self.x < o.x + o.w && o.x < self.x + self.w && self.y < o.y + o.h && o.y < self.y + self.h
    }
}

/// What a pushable control does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Action(Action),
    /// Select a HUD mode (0 NAV, 1 AIR, 2 GND) by cycling.
    HudMode(u8),
    /// Select a left MFD page.
    LeftPage(LeftMfd),
    /// Next tracking camera direction.
    CamCycle,
    /// Time acceleration on / off.
    TimeToggle,
    /// An MFD bezel button (its function depends on the page, `mfd::osb`).
    Osb(MfdSide, u8),
    /// A button of an air defence console (`station`).
    Console(ConsoleKey),
}

/// The buttons of an air defence console or the drone command center's (`Console(Fire)`,
/// `Console(Launcher, 2)` in the model).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleKey {
    /// The board's tabs: the track list, the site's status.
    Tracks,
    Site,
    /// Lock the selected track, drop the lock.
    Lock,
    Drop,
    /// Missile guidance for the next shots.
    Guided,
    Semi,
    /// A missile (Patriot) or a burst (Gepard) at the lock.
    Fire,
    /// Fire from this launcher next (1..3; again: the readiest).
    Launcher(u8),
    /// Refill this launcher (1..3; the Gepard: 1).
    Refill(u8),
    /// The camera: thermal / colour, zoom.
    Flir,
    ZoomIn,
    ZoomOut,
    /// The scope's range.
    RangeIn,
    RangeOut,
    /// Leave the console (back to the Ops screen).
    Leave,
    /// Drone command center: the drone on its route again, flown by hand; its camera held on
    /// the ground point at its crosshair, back ahead and down.
    Auto,
    Manual,
    Track,
    Slave,
    /// Drone command center: the camera between the TGP and the forward-looking FLIR.
    View,
}

/// A face drawn into the instrument atlas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Elem {
    /// A lamp, with an optional label replacing the standard one.
    Lamp(LampId, Option<&'static str>),
    Led(LedId),
    /// EMV / EGR / EAR gauge.
    Gauge,
    /// Text (lines separated by '\n'), foreground and background colour.
    Label(&'static str, u8, u8),
    /// An arrow glyph, foreground and background colour.
    Arrow(Arrow, u8, u8),
    /// Yellow and black warning stripes.
    Hazard,
    /// Landing gear lights: an aircraft outline with a green lamp at each wheel, lit with the
    /// gear down.
    GearLights,
    /// A round needle instrument (F-16).
    Dial(Dial),
}

/// The glyphs of `Elem::Arrow`: the four directions and the arrow back (reset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrow {
    Up,
    Down,
    Left,
    Right,
    Return,
}

/// The F-16's needle instruments: flight instruments round the attitude indicator and HSI (the
/// AOA and vertical velocity indicators are vertical tapes), engine instruments and fuel on
/// the right, the clock on the right console.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dial {
    Airspeed,
    Altimeter,
    Aoa,
    Vvi,
    Rpm,
    Ftit,
    Oil,
    Nozzle,
    Fuel,
    /// Local time, 12-hour face.
    Clock,
    /// A V/STOL type's nozzle angle down from aft (degrees, `Aircraft::nozzle`; the AV-8B's).
    NozzleAngle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub elem: Elem,
    pub rect: PxRect,
    /// Pushable: the face is a button cap.
    pub cap: Option<Effect>,
    /// Modelled round (the rectangle's inscribed circle shows): dials, round buttons.
    pub round: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MfdUnit {
    pub side: MfdSide,
    pub screen: PxRect,
    /// Labelled bezel buttons around the screen.
    pub osbs: bool,
}

pub struct PanelLayout {
    pub style: PanelStyle,
    pub items: Vec<Placed>,
    /// Attitude ball (square around the disc).
    pub adi: PxRect,
    /// The MFDs: left and right, the third (F-117A, Tornado).
    pub mfds: Vec<MfdUnit>,
    /// Radar warning receiver scope and horizontal situation indicator (F-16, A-10).
    pub rwr: Option<PxRect>,
    pub hsi: Option<PxRect>,
    /// Terrain following radar display (Tornado).
    pub tfr: Option<PxRect>,
    /// Data entry display: the up-front controls' readout (none on the classic panel).
    pub ded: Option<PxRect>,
    /// The portrait multipurpose displays: the left one (warnings; the RAH-66's) and the right
    /// one (engines; the RAH-66's, the CH-47F's centre display).
    pub lmpd: Option<PxRect>,
    pub rmpd: Option<PxRect>,
}

/// Bezel button cap size (millimetres; turned on end down the sides).
const OSB_W: f32 = 12.0;
const OSB_H: f32 = 10.0;
/// The side buttons' centres off the screen's edges.
const OSB_SIDE: f32 = 9.0;

impl PanelStyle {
    /// The cockpit has a third MFD (`MfdSide::Third`).
    pub fn third_mfd(self) -> bool {
        matches!(self, PanelStyle::Modern | PanelStyle::Tornado)
    }
}

impl PanelLayout {
    pub fn for_style(style: PanelStyle) -> Self {
        match style {
            PanelStyle::Classic => Self::classic(),
            PanelStyle::Modern => Self::modern(),
            PanelStyle::F16 => Self::f16(),
            PanelStyle::Uh60 => Self::uh60(),
            PanelStyle::Ah1z => Self::ah1z(),
            PanelStyle::Tornado => Self::tornado(),
            PanelStyle::A10 => Self::a10(),
            PanelStyle::Rah66 => Self::rah66(),
            PanelStyle::Av8b => Self::av8b(),
            PanelStyle::Oh58 => Self::oh58(),
            PanelStyle::Ch47 => Self::ch47(),
        }
    }

    /// Bezel buttons of an MFD: (slot, cap rectangle); 0..5 above the screen, 5..10 below,
    /// 10..15 down its left edge and 15..20 down its right one (`mfd::osb_edge`).
    pub fn osbs(&self, unit: &MfdUnit) -> Vec<(u8, PxRect)> {
        use crate::mfd::OsbEdge;
        if !unit.osbs {
            return Vec::new();
        }
        let s = unit.screen;
        (0..OSB_COUNT)
            .map(|slot| {
                let (edge, k) = crate::mfd::osb_edge(slot);
                let k = k as f32 + 0.5;
                let (cx, cy) = (s.x + s.w * k / 5.0, s.y + s.h * k / 5.0);
                let r = match edge {
                    OsbEdge::Top => rect(cx - OSB_W / 2.0, s.y - OSB_H - 1.5, OSB_W, OSB_H),
                    OsbEdge::Bottom => rect(cx - OSB_W / 2.0, s.y + s.h + 1.5, OSB_W, OSB_H),
                    OsbEdge::Left => {
                        rect(s.x - OSB_SIDE - OSB_H / 2.0, cy - OSB_W / 2.0, OSB_H, OSB_W)
                    }
                    OsbEdge::Right => {
                        rect(s.x + s.w + OSB_SIDE - OSB_H / 2.0, cy - OSB_W / 2.0, OSB_H, OSB_W)
                    }
                };
                (slot, r)
            })
            .collect()
    }

    /// The classic arrangement.
    pub fn classic() -> Self {
        use Effect as E;
        use LampId::*;
        let mut items = Vec::new();
        // Lamps at the classic light rectangles ; rect = (x,y)-(x+w,y+h).
        const LIGHTS: [(f32, f32, f32, f32); 37] = [
            (119.0, 94.0, 18.0, 8.0),
            (139.0, 94.0, 18.0, 8.0),
            (159.0, 94.0, 18.0, 8.0),
            (184.0, 94.0, 21.0, 8.0),
            (38.0, 105.0, 24.0, 8.0),
            (4.0, 117.0, 18.0, 8.0),
            (24.0, 117.0, 18.0, 8.0),
            (44.0, 117.0, 18.0, 8.0),
            (4.0, 127.0, 18.0, 8.0),
            (24.0, 127.0, 18.0, 8.0),
            (44.0, 127.0, 18.0, 8.0),
            (4.0, 137.0, 18.0, 8.0),
            (24.0, 137.0, 18.0, 8.0),
            (44.0, 137.0, 18.0, 8.0),
            (51.0, 149.0, 11.0, 6.0),
            (51.0, 159.0, 11.0, 6.0),
            (51.0, 169.0, 11.0, 6.0),
            (4.0, 179.0, 28.0, 8.0),
            (36.0, 179.0, 26.0, 8.0),
            (4.0, 191.0, 18.0, 8.0),
            (24.0, 191.0, 18.0, 8.0),
            (44.0, 191.0, 18.0, 8.0),
            (80.0, 173.0, 18.0, 8.0),
            (100.0, 173.0, 18.0, 8.0),
            (142.0, 193.0, 14.0, 5.0),
            (232.0, 184.0, 17.0, 8.0),
            (190.0, 181.0, 18.0, 8.0),
            (190.0, 191.0, 38.0, 8.0),
            (170.0, 181.0, 18.0, 8.0),
            (210.0, 181.0, 18.0, 8.0),
            (170.0, 191.0, 18.0, 8.0),
            (257.0, 103.0, 25.0, 8.0),
            (257.0, 113.0, 25.0, 8.0),
            (284.0, 113.0, 25.0, 8.0),
            (257.0, 123.0, 25.0, 8.0),
            (284.0, 123.0, 25.0, 8.0),
            (302.0, 179.0, 13.0, 7.0),
        ];
        let caps: [(LampId, Effect); 20] = [
            (Mw, E::Action(Action::MasterCaution)),
            (Nav, E::HudMode(0)),
            (Air, E::HudMode(1)),
            (Gnd, E::HudMode(2)),
            (Lock, E::Action(Action::DesignateTarget)),
            (Ecm, E::Action(Action::Ecm)),
            (Irj, E::Action(Action::IrJammer)),
            (Map, E::LeftPage(LeftMfd::Map)),
            (Tac, E::LeftPage(LeftMfd::Tac)),
            (Flir, E::Action(Action::Flir)),
            (Wpn, E::Action(Action::Ordnance)),
            (CamDir, E::CamCycle),
            (Way, E::Action(Action::SelectWaypoint)),
            (Ord, E::Action(Action::MissionOrders)),
            (Tc, E::Action(Action::CamAhead)),
            (Bay, E::Action(Action::BayDoors)),
            (Auto, E::Action(Action::Autopilot)),
            (Flaps, E::Action(Action::Flaps)),
            (Gear, E::Action(Action::Gear)),
            (Brake, E::Action(Action::Brakes)),
        ];
        for (i, &(x, y, w, h)) in LIGHTS.iter().enumerate() {
            let id = crate::panel::LAMPS[i].0;
            let cap = caps.iter().find(|(l, _)| *l == id).map(|&(_, e)| e);
            items.push(Placed {
                elem: Elem::Lamp(id, None),
                rect: rect(x, y, w + 1.0, h + 1.0),
                cap,
                round: false,
            });
        }
        // Countermeasure release buttons with their labels, LED readouts and the other texts
        // that were part of the picture.
        for (k, (label, action, led)) in [
            ("CHAFF", Action::Chaff, LedId::Chaff),
            ("FLARE", Action::Flare, LedId::Flares),
            ("DECOY", Action::Decoy, LedId::Decoys),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 147.0 + 10.0 * k as f32;
            items.push(Placed {
                elem: Elem::Label(label, 47, 38),
                rect: rect(4.0, y, 25.0, 9.0),
                cap: Some(E::Action(action)),
                round: false,
            });
            items.push(Placed {
                elem: Elem::Led(led),
                rect: rect(33.0, y, 14.0, 10.0),
                cap: None,
                round: false,
            });
        }
        let fixed = [
            (Elem::Led(LedId::Fuel), rect(259.0, 178.0, 18.0, 9.0)),
            (Elem::Led(LedId::Thrust), rect(259.0, 190.0, 18.0, 9.0)),
            (Elem::Gauge, rect(86.0, 187.0, 51.0, 11.0)),
            (Elem::Label("EGR\nEAR", 47, 33), rect(67.0, 187.0, 16.0, 12.0)),
            (Elem::Label("EMV", 47, 33), rect(141.0, 185.0, 17.0, 7.0)),
            (Elem::Label("FUEL", 47, 79), rect(279.0, 178.0, 20.0, 9.0)),
            (Elem::Label("THRUST", 47, 79), rect(279.0, 190.0, 30.0, 9.0)),
            (Elem::Hazard, rect(307.0, 136.0, 12.0, 40.0)),
        ];
        items.extend(fixed.into_iter().map(|(elem, rect)| Placed {
            elem,
            rect,
            cap: None,
            round: false,
        }));

        let screens = [rect(74.0, 110.0, 77.0, 60.0), rect(169.0, 110.0, 77.0, 60.0)];
        // The top two squares outside each screen are buttons: map zoom on the left MFD,
        // waypoint selection on the right one.
        for (x, y, label, action) in [
            (65.0, 110.0, "+", Action::MapZoomIn),
            (65.0, 123.0, "-", Action::MapZoomOut),
            (249.0, 112.0, "+", Action::NextWaypoint),
            (249.0, 123.0, "-", Action::LastWaypoint),
        ] {
            items.push(Placed {
                elem: Elem::Label(label, 47, 22),
                rect: rect(x, y, 6.0, 6.0),
                cap: Some(E::Action(action)),
                round: false,
            });
        }
        // The WAY page's waypoint editing buttons (up, down, left, right and return arrows down
        // the left edge of the right MFD, MOV at the bottom of its right edge): their faces only, the
        // controls are in the model. Their rectangles stand clear of the others.
        for (k, arrow) in [Arrow::Up, Arrow::Down, Arrow::Left, Arrow::Right, Arrow::Return]
            .into_iter()
            .enumerate()
        {
            items.push(Placed {
                elem: Elem::Arrow(arrow, 47, 22),
                rect: rect(155.0, 104.0 + 15.0 * k as f32, 12.0, 14.0),
                cap: None,
                round: false,
            });
        }
        items.push(Placed {
            elem: Elem::Label("MOV", 47, 22),
            rect: rect(249.0, 150.0, 12.0, 14.0),
            cap: None,
            round: false,
        });
        Self {
            style: PanelStyle::Classic,
            items,
            adi: rect(265.0, 139.0, 32.0, 32.0),
            mfds: vec![
                MfdUnit { side: MfdSide::Left, screen: screens[0], osbs: false },
                MfdUnit { side: MfdSide::Right, screen: screens[1], osbs: false },
            ],
            rwr: None,
            hsi: None,
            tfr: None,
            ded: None,
            lmpd: None,
            rmpd: None,
        }
    }

    /// The F-117A's panel (millimetres as the F-16's, x right and y down, unfolded; the model
    /// is built at the F-117A cockpit's larger scale), after the real one's:
    /// - The centre column (|x| <= 100): the up-front display under the HUD, a data entry
    ///   display between two rows of seven caps (top: the HUD modes, LOCK, DCLT, NVG, ILS;
    ///   bottom: MENU, VOL, TIME, STPT, ORD and two blank ones), then the large centre display
    ///   (the right MFD: where the real one shows the infrared picture).
    /// - The wings (100 < |x| <= 358), their upper edges falling steeply outwards as the
    ///   canopy's (`tools/models/build_cockpit_f117.py`): the left MFD and the third one (the
    ///   HSI at first), the warning lights over their inner corners; the F-16's round
    ///   instruments down the outer edges (airspeed, altimeter, the AOA and vertical velocity
    ///   tapes and the clock left; RPM, FTIT, oil pressure and nozzle position right), the
    ///   attitude indicator and the BAY cap under the left MFD, fuel quantity, the engine
    ///   readouts and the EMV gauge under the right one.
    /// - As in the F-16 the sloped panels from the consoles up to the main panel: the gear
    ///   lights with FLAPS, BRAKE and AUTO left (x <= -600, centred on -700; the gear handle and
    ///   the target buttons are modelled), the caution lights and the countermeasures right
    ///   (x >= 1100, centred on 1200).
    pub fn modern() -> Self {
        use Effect as E;
        use LampId::*;
        let mut items = Vec::new();
        let mut put = |elem: Elem, rect: PxRect, cap: Option<Effect>, round: bool| {
            items.push(Placed { elem, rect, cap, round })
        };
        // Up-front display caps (black on grey as the F-16's keys).
        const KEY: (u8, u8) = (5, 14);
        let ufc = |k: usize, y: f32| rect(-69.0 + 20.0 * k as f32, y, 18.0, 12.0);
        for (k, (elem, cap)) in [
            (Elem::Lamp(Nav, None), Some(E::HudMode(0))),
            (Elem::Lamp(Air, None), Some(E::HudMode(1))),
            (Elem::Lamp(Gnd, None), Some(E::HudMode(2))),
            (Elem::Lamp(Lock, None), Some(E::Action(Action::DesignateTarget))),
            (Elem::Label("DCLT", KEY.0, KEY.1), Some(E::Action(Action::HudDeclutter))),
            (Elem::Label("NVG", KEY.0, KEY.1), Some(E::Action(Action::Nvg))),
            (Elem::Label("ILS", KEY.0, KEY.1), Some(E::Action(Action::Ils))),
        ]
        .into_iter()
        .enumerate()
        {
            put(elem, ufc(k, -135.0), cap, false);
        }
        for (k, (label, cap)) in [
            ("MENU", Some(E::Action(Action::Menu))),
            ("VOL", Some(E::Action(Action::Volume))),
            ("TIME", Some(E::TimeToggle)),
            ("STPT", Some(E::Action(Action::SelectWaypoint))),
            ("ORD", Some(E::Action(Action::MissionOrders))),
            ("COM", None),
            ("IFF", None),
        ]
        .into_iter()
        .enumerate()
        {
            put(Elem::Label(label, KEY.0, KEY.1), ufc(k, -69.0), cap, false);
        }
        // Warning lights over the side MFDs' inner corners: master caution, stall and the
        // threat lights left; fire, engine, flight controls and jammer / avionics damage right.
        let mc = Some(E::Action(Action::MasterCaution));
        put(Elem::Lamp(Mw, Some("MASTER\nCAUTION")), rect(-168.0, -96.0, 40.0, 14.0), mc, false);
        put(Elem::Lamp(Stall, None), rect(-126.0, -96.0, 24.0, 14.0), None, false);
        put(Elem::Lamp(Trak, Some("TRACK")), rect(-168.0, -80.0, 20.0, 11.0), None, false);
        put(Elem::Lamp(Ir, None), rect(-146.0, -80.0, 20.0, 11.0), None, false);
        put(Elem::Lamp(Rad, Some("LAUNCH")), rect(-124.0, -80.0, 22.0, 11.0), None, false);
        put(Elem::Lamp(Fire, Some("ENG\nFIRE")), rect(102.0, -96.0, 24.0, 14.0), None, false);
        put(Elem::Lamp(Eng, Some("ENGINE")), rect(128.0, -96.0, 40.0, 14.0), None, false);
        put(Elem::Lamp(Fc, Some("FLCS")), rect(102.0, -80.0, 22.0, 11.0), None, false);
        put(Elem::Lamp(Jam, None), rect(126.0, -80.0, 20.0, 11.0), None, false);
        put(Elem::Lamp(Av, None), rect(148.0, -80.0, 20.0, 11.0), None, false);
        // Flight instruments down the left edge, engine instruments down the right one.
        for (k, dial) in [Dial::Airspeed, Dial::Altimeter].into_iter().enumerate() {
            put(Elem::Dial(dial), rect(-321.0, -36.0 + 46.0 * k as f32, 42.0, 42.0), None, true);
        }
        put(Elem::Dial(Dial::Aoa), rect(-320.0, 56.0, 18.0, 52.0), None, false);
        put(Elem::Dial(Dial::Vvi), rect(-298.0, 56.0, 18.0, 52.0), None, false);
        put(Elem::Dial(Dial::Clock), rect(-321.0, 114.0, 42.0, 42.0), None, true);
        for (k, dial) in [Dial::Rpm, Dial::Ftit, Dial::Oil, Dial::Nozzle].into_iter().enumerate() {
            put(Elem::Dial(dial), rect(279.0, -36.0 + 46.0 * k as f32, 42.0, 42.0), None, true);
        }
        // Under the left MFD right of the attitude indicator: the bay doors (the master arm
        // panel, its switches modelled).
        put(
            Elem::Lamp(Bay, None),
            rect(-160.0, 92.0, 38.0, 14.0),
            Some(E::Action(Action::BayDoors)),
            false,
        );
        // Under the right MFD: fuel quantity with the fuel and thrust readouts, the EMV gauge.
        put(Elem::Dial(Dial::Fuel), rect(110.0, 86.0, 42.0, 42.0), None, true);
        put(Elem::Led(LedId::Fuel), rect(158.0, 90.0, 30.0, 13.0), None, false);
        put(Elem::Label("FUEL", 47, 79), rect(190.0, 90.0, 34.0, 13.0), None, false);
        put(Elem::Lamp(LowFuel, Some("LOW\nFUEL")), rect(226.0, 90.0, 30.0, 13.0), None, false);
        put(Elem::Led(LedId::Thrust), rect(158.0, 108.0, 30.0, 13.0), None, false);
        put(Elem::Label("THRUST", 47, 79), rect(190.0, 108.0, 34.0, 13.0), None, false);
        put(Elem::Label("EGR\nEAR", 47, 33), rect(110.0, 138.0, 20.0, 13.0), None, false);
        put(Elem::Gauge, rect(132.0, 138.0, 84.0, 13.0), None, false);
        put(Elem::Label("EMV", 47, 33), rect(218.0, 138.0, 20.0, 13.0), None, false);
        put(Elem::Lamp(EmvWarn, None), rect(240.0, 138.0, 18.0, 13.0), None, false);
        // Left slope: the gear lights and under them FLAPS, BRAKE and AUTO, a column clear of
        // the slope's outer edge (it comes in towards the top) and of the gear handle right of it.
        put(Elem::GearLights, rect(-732.0, -110.0, 40.0, 32.0), None, false);
        for (k, (id, action)) in
            [(Flaps, Action::Flaps), (Brake, Action::Brakes), (Auto, Action::Autopilot)]
                .into_iter()
                .enumerate()
        {
            put(
                Elem::Lamp(id, None),
                rect(-731.0, -68.0 + 18.0 * k as f32, 38.0, 14.0),
                Some(E::Action(action)),
                false,
            );
        }
        // Right slope: the caution lights, then the countermeasures (release caps, counts,
        // released lights; the jammers).
        const CAUTION: (u8, u8) = (16, 43);
        for (k, elem) in [
            Elem::Lamp(Fuel, None),
            Elem::Lamp(Bd, None),
            Elem::Lamp(Blank, None),
            Elem::Label("ELEC\nSYS", CAUTION.0, CAUTION.1),
            Elem::Label("HYD\nPRESS", CAUTION.0, CAUTION.1),
            Elem::Label("OXY\nLOW", CAUTION.0, CAUTION.1),
        ]
        .into_iter()
        .enumerate()
        {
            put(
                elem,
                rect(1132.0 + 35.0 * (k % 3) as f32, -118.0 + 17.0 * (k / 3) as f32, 32.0, 14.0),
                None,
                false,
            );
        }
        for (k, (label, action, led, act)) in [
            ("CHAFF", Action::Chaff, LedId::Chaff, ChaffAct),
            ("FLARE", Action::Flare, LedId::Flares, FlareAct),
            ("DECOY", Action::Decoy, LedId::Decoys, DecoyAct),
        ]
        .into_iter()
        .enumerate()
        {
            let y = -70.0 + 17.0 * k as f32;
            put(
                Elem::Label(label, 47, 38),
                rect(1150.0, y, 30.0, 13.0),
                Some(E::Action(action)),
                false,
            );
            put(Elem::Led(led), rect(1183.0, y, 20.0, 13.0), None, false);
            put(Elem::Lamp(act, None), rect(1206.0, y, 16.0, 13.0), None, false);
        }
        put(
            Elem::Lamp(Ecm, None),
            rect(1150.0, -16.0, 34.0, 13.0),
            Some(E::Action(Action::Ecm)),
            false,
        );
        put(
            Elem::Lamp(Irj, None),
            rect(1188.0, -16.0, 34.0, 13.0),
            Some(E::Action(Action::IrJammer)),
            false,
        );

        // The HIRES canvas' shape: the side MFDs 120 mm wide, the centre display 160 (its side
        // buttons inside the centre column).
        let hires = crate::mfd::HIRES.1 as f32 / crate::mfd::HIRES.0 as f32;
        let left = rect(-245.0, -38.0, 120.0, 120.0 * hires);
        let centre = rect(-80.0, -12.0, 160.0, 160.0 * hires);
        let third = rect(125.0, -38.0, 120.0, 120.0 * hires);
        Self {
            style: PanelStyle::Modern,
            items,
            adi: rect(-250.0, 92.0, 68.0, 68.0),
            mfds: vec![
                MfdUnit { side: MfdSide::Left, screen: left, osbs: true },
                MfdUnit { side: MfdSide::Right, screen: centre, osbs: true },
                MfdUnit { side: MfdSide::Third, screen: third, osbs: true },
            ],
            rwr: None,
            hsi: None,
            tfr: None,
            ded: Some(rect(
                -60.0,
                -121.5,
                DED_W,
                DED_W * DED_CANVAS.1 as f32 / DED_CANVAS.0 as f32,
            )),
            lmpd: None,
            rmpd: None,
        }
    }

    /// The UH-60A's panel: the F-117A's faces rearranged after the UH-60M's (millimetres, x
    /// right and y down, life size; `tools/models/build_cockpit_uh60.py`):
    /// - The main panel, its origin in front of the pilot (in the right seat): two large MFDs
    ///   side by side (no third), the warning lights over them; under the left one the gear
    ///   lights with FLAPS, BRAKE and AUTO and the BAY cap, under the right one fuel quantity,
    ///   the engine readouts and the EMV gauge. Between the seats (x about -520, the cabin's
    ///   centre line) the standby attitude indicator over the airspeed indicator and the
    ///   altimeter, the clock and the AOA and vertical velocity tapes; the engine gauges between
    ///   them and the left MFD.
    /// - The centre console between the seats (x >= 1500, centred on x 2000, y growing aft down
    ///   the console): the up-front display with its two rows of caps as the CDU, the caution
    ///   lights and the countermeasures behind it.
    pub fn uh60() -> Self {
        let m = Self::modern();
        // A face's new place: alone, or moved with its group of the F-117A's panel.
        let place = |p: &Placed| {
            let r = p.rect;
            let at = |x: f32, y: f32| rect(x, y, r.w, r.h);
            let by = |dx: f32, dy: f32| rect(r.x + dx, r.y + dy, r.w, r.h);
            let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
            match p.elem {
                Elem::Dial(Dial::Airspeed) => at(-567.0, -45.0),
                Elem::Dial(Dial::Altimeter) => at(-515.0, -45.0),
                Elem::Dial(Dial::Clock) => at(-567.0, 5.0),
                Elem::Dial(Dial::Aoa) => at(-515.0, 0.0),
                Elem::Dial(Dial::Vvi) => at(-493.0, 0.0),
                Elem::Dial(Dial::Rpm) => at(-415.0, -45.0),
                Elem::Dial(Dial::Ftit) => at(-365.0, -45.0),
                Elem::Dial(Dial::Oil) => at(-415.0, 5.0),
                Elem::Dial(Dial::Nozzle) => at(-365.0, 5.0),
                Elem::Lamp(LampId::Bay, _) => at(-180.0, 110.0),
                // The caution lights and countermeasures to the console, behind the CDU.
                _ if cx >= 1100.0 => by(818.0, 150.0),
                // The gear lights, FLAPS, BRAKE and AUTO under the left MFD.
                _ if cx <= -600.0 => by(492.0, 210.0),
                // The up-front display's caps to the console.
                _ if cx.abs() < 80.0 && cy < -50.0 => by(2000.0, 0.0),
                // The warning lights over the MFDs.
                _ if cy < -60.0 => by(10f32.copysign(cx), -29.0),
                // Fuel, the engine readouts and the EMV gauge under the right MFD.
                _ => by(-40.0, 20.0),
            }
        };
        let mut items: Vec<Placed> =
            m.items.iter().map(|p| Placed { rect: place(p), ..*p }).collect();
        // HOVER right of AUTO.
        items.push(Placed {
            elem: Elem::Lamp(LampId::Hover, None),
            rect: rect(-197.0, 178.0, 38.0, 14.0),
            cap: Some(Effect::Action(Action::Hover)),
            round: false,
        });
        let hires = crate::mfd::HIRES.1 as f32 / crate::mfd::HIRES.0 as f32;
        let ded = m.ded.map(|r| rect(r.x + 2000.0, r.y, r.w, r.h));
        Self {
            style: PanelStyle::Uh60,
            items,
            adi: rect(-554.0, -125.0, 68.0, 68.0),
            mfds: vec![
                MfdUnit {
                    side: MfdSide::Left,
                    screen: rect(-240.0, -70.0, 190.0, 190.0 * hires),
                    osbs: true,
                },
                MfdUnit {
                    side: MfdSide::Right,
                    screen: rect(50.0, -70.0, 190.0, 190.0 * hires),
                    osbs: true,
                },
            ],
            rwr: None,
            hsi: None,
            tfr: None,
            ded,
            lmpd: None,
            rmpd: None,
        }
    }

    /// The CH-47F's panel: the UH-60A's (millimetres, x right and y down, life size;
    /// `tools/models/build_cockpit_ch47.py`) on one wide panel across the cockpit after the
    /// CH-47F's, its origin in front of the pilot (in the right seat; the cabin's centre line is
    /// at x -620, the copilot's dark MFDs are only modelled): the pilot's two MFDs side by side
    /// under the warning lights, the large centre display on the centre line showing the engines
    /// (`rmpd`, the RAH-66's engine page), the standby instruments and the engine gauges between
    /// it and the pilot's MFDs, the F-16's RWR scope over the engine gauges left of the left MFD
    /// (as far from them as they are apart), the flight control caps and the fuel readouts under
    /// them (the panel reaches down only between the pilots: the chin windows show under it in
    /// front of each). Fixed wheels and no weapons: no gear lights, gear handle, FLAPS or BAY; no
    /// EMV bar graph. The CDU, the cautions and the countermeasures on the centre
    /// console between the seats, as the UH-60A's.
    pub fn ch47() -> Self {
        let u = Self::uh60();
        // The standby and engine instruments closer to the pilot's MFDs, clear of the centre
        // display.
        const DX: f32 = 45.0;
        let items = u
            .items
            .iter()
            .filter(|p| {
                !matches!(
                    p.elem,
                    Elem::GearLights
                        | Elem::Lamp(LampId::Flaps | LampId::Bay | LampId::EmvWarn, _)
                        | Elem::Gauge
                        | Elem::Label("EGR\nEAR" | "EMV", ..)
                )
            })
            .map(|p| {
                let r = p.rect;
                let by = |dx: f32| rect(r.x + dx, r.y, r.w, r.h);
                let at = |x: f32, y: f32| rect(x, y, r.w, r.h);
                let rect = match p.elem {
                    // BRAKE, AUTO and HOVER in a column left of the fuel readouts, on their plate.
                    Elem::Lamp(LampId::Brake, _) => at(-466.0, 103.0),
                    Elem::Lamp(LampId::Auto, _) => at(-466.0, 119.0),
                    Elem::Lamp(LampId::Hover, _) => at(-466.0, 135.0),
                    // Fuel, the engine readouts and the EMV gauge under the engine gauges.
                    _ if r.x < 1000.0 && r.x >= 60.0 && r.y >= 100.0 => by(-482.0),
                    Elem::Dial(
                        Dial::Airspeed
                        | Dial::Altimeter
                        | Dial::Clock
                        | Dial::Aoa
                        | Dial::Vvi
                        | Dial::Rpm
                        | Dial::Ftit
                        | Dial::Oil
                        | Dial::Nozzle,
                    ) => rect(r.x + DX, r.y, r.w, r.h),
                    _ => r,
                };
                Placed { rect, ..*p }
            })
            .chain(
                // The RWR's priority mode and target separate keys left of its scope (the MFD's
                // raised bezel is right of it), as the F-16's, on its plate widened for them
                // (`build_cockpit_ch47.py`).
                [(LampId::RwrPri, Action::RwrPriority), (LampId::RwrSep, Action::RwrSeparate)]
                    .into_iter()
                    .enumerate()
                    .map(|(k, (id, action))| Placed {
                        elem: Elem::Lamp(id, None),
                        rect: rect(-402.0, -120.0 + 18.0 * k as f32, 20.0, 14.0),
                        cap: Some(Effect::Action(action)),
                        round: false,
                    }),
            )
            .collect();
        Self {
            style: PanelStyle::Ch47,
            items,
            adi: rect(u.adi.x + DX, u.adi.y, u.adi.w, u.adi.h),
            rmpd: Some(rect(-685.0, -95.0, 130.0, 182.0)),
            // Centred over the engine gauges (x -370..-278), its top level with the attitude
            // indicator's and the warning lights', 8 mm above them as they are apart.
            rwr: Some(rect(-360.0, -125.0, 72.0, 72.0)),
            ..u
        }
    }

    /// The AH-1Z's panel: the UH-60A's (millimetres, life size; `tools/models/build_cockpit_ah1z.py`)
    /// narrowed to the pilot's cockpit, after the AH-1Z's: the two MFDs side by side under the
    /// warning lights, the standby airspeed indicator, altimeter and clock with the AOA and
    /// vertical velocity tapes left of them, the engine gauges right of them, the standby
    /// attitude indicator between the groups under them. The CDU, the cautions and the
    /// countermeasures stay on the console (here the pedestal between the knees). No gear
    /// lights (skids); the AUTO cap under BAY.
    pub fn ah1z() -> Self {
        let u = Self::uh60();
        let items = u
            .items
            .iter()
            .filter(|p| !matches!(p.elem, Elem::GearLights))
            .map(|p| {
                let r = p.rect;
                let at = |x: f32, y: f32| rect(x, y, r.w, r.h);
                let rect = match p.elem {
                    Elem::Dial(Dial::Aoa) => at(-372.0, -60.0),
                    Elem::Dial(Dial::Vvi) => at(-372.0, -2.0),
                    Elem::Dial(Dial::Airspeed) => at(-322.0, -60.0),
                    Elem::Dial(Dial::Altimeter) => at(-322.0, -10.0),
                    Elem::Dial(Dial::Clock) => at(-322.0, 40.0),
                    Elem::Dial(Dial::Rpm) => at(280.0, -60.0),
                    Elem::Dial(Dial::Ftit) => at(280.0, -10.0),
                    Elem::Dial(Dial::Oil) => at(330.0, -60.0),
                    Elem::Dial(Dial::Nozzle) => at(330.0, -10.0),
                    Elem::Lamp(LampId::Auto, _) => at(-180.0, 128.0),
                    Elem::Lamp(LampId::Hover, _) => at(-139.0, 128.0),
                    _ => r,
                };
                Placed { rect, ..*p }
            })
            .collect();
        Self { style: PanelStyle::Ah1z, items, adi: rect(-34.0, 110.0, 68.0, 68.0), ..u }
    }

    /// The Tornado IDS's modernised front panel (millimetres, x right and y down, life size;
    /// `tools/models/build_cockpit_tornado.py`), after the upgrade mock-ups' three-MFD panel:
    /// - The centre column (|x| <= 112): the centre MFD (the right MFD) high under the HUD, under
    ///   it the up-front controls: the data entry display between two blocks of six caps (left:
    ///   the HUD modes, LOCK, DCLT, NVG; right: ILS, MENU, VOL, TIME, STPT, ORD) over a row of
    ///   eight keys, then fuel quantity with the fuel and thrust readouts and the EMV gauge.
    /// - The wings (112 < |x| <= 335): the left MFD and the third one (the right MFD's place in the
    ///   cockpit, the HSI at first), a little lower than the centre one. Under the left one the
    ///   standby instruments (airspeed, attitude, altimeter, the AOA and vertical velocity tapes,
    ///   the clock) and the master caution with the threat lights; under the right one the fire,
    ///   engine, flight control and jammer / avionics damage lights and the engine instruments.
    /// - As in the F-117A the sloped panels from the consoles up to the wings: the gear lights
    ///   with FLAPS, BRAKE and AUTO left (x <= -600, centred on -700; the gear handle and the
    ///   target buttons are modelled), the caution lights and the countermeasures right (x >=
    ///   1100, centred on 1200).
    /// - Over the left MFD, in a housing standing on the left wing's top edge: the terrain
    ///   following radar display (`tfr`).
    ///
    /// No BAY cap: the stores hang under the fuselage and the wings.
    pub fn tornado() -> Self {
        use Effect as E;
        use LampId::*;
        let m = Self::modern();
        let mut items = Vec::new();
        let mut put = |elem: Elem, rect: PxRect, cap: Option<Effect>, round: bool| {
            items.push(Placed { elem, rect, cap, round })
        };
        // Up-front controls (black on grey as the F-16's keys): two columns of three caps either
        // side of the data entry display, a row of eight keys under them.
        const KEY: (u8, u8) = (5, 14);
        let caps = [
            (Elem::Lamp(Nav, None), Some(E::HudMode(0))),
            (Elem::Lamp(Air, None), Some(E::HudMode(1))),
            (Elem::Lamp(Gnd, None), Some(E::HudMode(2))),
            (Elem::Lamp(Lock, None), Some(E::Action(Action::DesignateTarget))),
            (Elem::Label("DCLT", KEY.0, KEY.1), Some(E::Action(Action::HudDeclutter))),
            (Elem::Label("NVG", KEY.0, KEY.1), Some(E::Action(Action::Nvg))),
            (Elem::Label("ILS", KEY.0, KEY.1), Some(E::Action(Action::Ils))),
            (Elem::Label("MENU", KEY.0, KEY.1), Some(E::Action(Action::Menu))),
            (Elem::Label("VOL", KEY.0, KEY.1), Some(E::Action(Action::Volume))),
            (Elem::Label("TIME", KEY.0, KEY.1), Some(E::TimeToggle)),
            (Elem::Label("STPT", KEY.0, KEY.1), Some(E::Action(Action::SelectWaypoint))),
            (Elem::Label("ORD", KEY.0, KEY.1), Some(E::Action(Action::MissionOrders))),
        ];
        for (k, (elem, cap)) in caps.into_iter().enumerate() {
            let (block, i) = (k / 6, k % 6);
            let x = [-98.0, 54.0][block] + 24.0 * (i % 2) as f32;
            put(elem, rect(x, -20.0 + 14.0 * (i / 2) as f32, 20.0, 12.0), cap, false);
        }
        for (k, label) in
            ["COM1", "COM2", "IFF", "TCN", "IDM", "RCL", "ENTR", "CLR"].into_iter().enumerate()
        {
            put(
                Elem::Label(label, KEY.0, KEY.1),
                rect(-98.0 + 25.0 * k as f32, 26.0, 21.0, 12.0),
                None,
                false,
            );
        }
        // Standby instruments under the left MFD: airspeed, the attitude indicator (`adi`) and
        // the altimeter in a row with the AOA and vertical velocity tapes, the clock under them.
        put(Elem::Dial(Dial::Airspeed), rect(-307.0, 4.0, 42.0, 42.0), None, true);
        put(Elem::Dial(Dial::Altimeter), rect(-205.0, 4.0, 42.0, 42.0), None, true);
        put(Elem::Dial(Dial::Aoa), rect(-157.0, 0.0, 16.0, 52.0), None, false);
        put(Elem::Dial(Dial::Vvi), rect(-137.0, 0.0, 16.0, 52.0), None, false);
        put(Elem::Dial(Dial::Clock), rect(-307.0, 64.0, 42.0, 42.0), None, true);
        // Engine instruments under the right MFD, outboard of the warning lights.
        for (k, dial) in [Dial::Rpm, Dial::Ftit, Dial::Oil, Dial::Nozzle].into_iter().enumerate() {
            let (col, row) = ((k % 2) as f32, (k / 2) as f32);
            put(
                Elem::Dial(dial),
                rect(205.0 + 48.0 * col, 4.0 + 48.0 * row, 42.0, 42.0),
                None,
                true,
            );
        }
        // The F-117A's warning lights, its fuel group and the faces of its sloped panels: the
        // left warning lights under the left MFD right of the clock, the right ones under the
        // right MFD inboard of the engine instruments, the fuel quantity, readouts and EMV gauge
        // under the up-front controls.
        for p in &m.items {
            let r = p.rect;
            let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
            let (dx, dy) = match p.elem {
                _ if cx <= -600.0 || cx >= 1100.0 => (0.0, 0.0),
                Elem::Lamp(Bay, _) => continue,
                _ if cy < -60.0 && cx < -100.0 => (-80.0, 166.0),
                _ if cy < -60.0 && cx > 100.0 => (26.0, 100.0),
                _ if (100.0..280.0).contains(&cx) && cy > 80.0 => (-184.0, -40.0),
                _ => continue,
            };
            put(p.elem, rect(r.x + dx, r.y + dy, r.w, r.h), p.cap, p.round);
        }

        // The HIRES canvas' shape, 146 mm wide: the centre MFD 22 mm higher than the side ones.
        let (w, h) = (146.0, 146.0 * crate::mfd::HIRES.1 as f32 / crate::mfd::HIRES.0 as f32);
        Self {
            style: PanelStyle::Tornado,
            items,
            adi: rect(-261.0, 0.0, 52.0, 52.0),
            mfds: vec![
                MfdUnit { side: MfdSide::Left, screen: rect(-292.0, -146.0, w, h), osbs: true },
                MfdUnit { side: MfdSide::Right, screen: rect(-73.0, -168.0, w, h), osbs: true },
                MfdUnit { side: MfdSide::Third, screen: rect(146.0, -146.0, w, h), osbs: true },
            ],
            rwr: None,
            hsi: None,
            tfr: Some(rect(-284.0, -250.0, 58.0, 58.0)),
            ded: Some(rect(-46.0, -20.0, 92.0, 92.0 * DED_CANVAS.1 as f32 / DED_CANVAS.0 as f32)),
            lmpd: None,
            rmpd: None,
        }
    }

    /// The A-10C's panel (millimetres, x right and y down, life size, the origin at the attitude
    /// indicator's centre; `tools/models/build_cockpit_a10.py`), after the A-10C's front panel:
    /// - The up-front controller under the HUD (y < -170, on the HUD's body above the glare
    ///   shield): the keypad left (T-ILS, STPT, TIME and MARK work), the data entry display in
    ///   the middle with DCLT, NVG, MENU and COM1 (volume) under it, the master caution and the
    ///   HUD mode lamps (NAV, A-A, A-G) right among the function keys (modelled blank).
    /// - Along the top of the centre column under the glare shield: the engine and APU fire
    ///   handles' lights.
    /// - The centre column (|x| <= 165): the large attitude indicator with the HSI under it;
    ///   over it the countermeasures panel (release caps, counts, released lights; the jammers);
    ///   left of it the RWR scope over the airspeed indicator (the threat lights over the scope),
    ///   the clock and the AOA tape under them; right of it the vertical velocity tape over the
    ///   altimeter (the engine and flight control lights over them).
    /// - The wings (165 < |x| <= 395): the two MFDs, 160 mm screens.
    /// - The lower panel, wider (|x| <= 420, y > 60): left the gear lights over the flap, brake
    ///   and autopilot caps, the downlock override and the gear handle (modelled), the armament
    ///   switches (modelled); right the engine instruments (FTIT, RPM, oil, nozzle), the caution
    ///   lights and the fuel quantity. The target buttons (modelled) under the HSI.
    pub fn a10() -> Self {
        use Effect as E;
        use LampId::*;
        let mut items = Vec::new();
        let mut put = |elem: Elem, rect: PxRect, cap: Option<Effect>, round: bool| {
            items.push(Placed { elem, rect, cap, round })
        };
        // Black on grey keys; unlit legends of the lights the game doesn't drive (red, amber,
        // green).
        const KEY: (u8, u8) = (5, 14);
        const WARN: (u8, u8) = (54, 55);
        const CAUTION: (u8, u8) = (16, 43);
        const ADVISORY: (u8, u8) = (45, 44);
        // Up-front controller: the keypad (as the F-16's), the keys under the data entry
        // display, the master caution and the HUD modes right (the function keys round them are
        // modelled blank).
        const KEYS: [(&str, Option<Effect>); 12] = [
            ("1\nT-ILS", Some(E::Action(Action::Ils))),
            ("2", None),
            ("3", None),
            ("4\nSTPT", Some(E::Action(Action::SelectWaypoint))),
            ("5", None),
            ("6\nTIME", Some(E::TimeToggle)),
            ("7\nMARK", Some(E::Action(Action::DesignateTarget))),
            ("8", None),
            ("9", None),
            ("RCL", None),
            ("0", None),
            ("ENT", None),
        ];
        for (k, (label, cap)) in KEYS.into_iter().enumerate() {
            put(
                Elem::Label(label, KEY.0, KEY.1),
                rect(-126.0 + 18.0 * (k % 3) as f32, -256.0 + 15.0 * (k / 3) as f32, 16.0, 13.0),
                cap,
                false,
            );
        }
        for (k, (label, cap)) in [
            ("DCLT", E::Action(Action::HudDeclutter)),
            ("NVG", E::Action(Action::Nvg)),
            ("MENU", E::Action(Action::Menu)),
            ("COM1", E::Action(Action::Volume)),
        ]
        .into_iter()
        .enumerate()
        {
            put(
                Elem::Label(label, KEY.0, KEY.1),
                rect(-68.0 + 22.0 * k as f32, -211.0, 20.0, 13.0),
                Some(cap),
                false,
            );
        }
        let mc = Some(E::Action(Action::MasterCaution));
        for (k, (elem, cap)) in [
            (Elem::Lamp(Mw, Some("MASTER\nCAUTION")), mc),
            (Elem::Lamp(Nav, None), Some(E::HudMode(0))),
            (Elem::Lamp(Air, Some("A-A")), Some(E::HudMode(1))),
            (Elem::Lamp(Gnd, Some("A-G")), Some(E::HudMode(2))),
        ]
        .into_iter()
        .enumerate()
        {
            let r = rect(24.0 + 34.0 * (k % 2) as f32, -256.0 + 15.0 * (k / 2) as f32, 30.0, 13.0);
            put(elem, r, cap, false);
        }
        // The fire handles' lights across the top of the centre column (both engines' light
        // with an engine fire).
        put(Elem::Lamp(Fire, Some("FIRE\nL ENG")), rect(-150.0, -158.0, 76.0, 13.0), None, false);
        put(Elem::Label("FIRE\nAPU", WARN.0, WARN.1), rect(-38.0, -158.0, 76.0, 13.0), None, false);
        put(Elem::Lamp(Fire, Some("FIRE\nR ENG")), rect(74.0, -158.0, 76.0, 13.0), None, false);
        // Threat lights over the RWR scope; engine and flight control lights over the vertical
        // velocity tape.
        for (k, elem) in [
            Elem::Lamp(Rad, Some("LAUNCH")),
            Elem::Lamp(Trak, Some("TRACK")),
            Elem::Lamp(Ir, Some("IR")),
            Elem::Lamp(Stall, None),
            Elem::Lamp(Eng, Some("ENGINE")),
            Elem::Lamp(Fc, Some("FLT\nCTL")),
            Elem::Label("GUN\nREADY", ADVISORY.0, ADVISORY.1),
            Elem::Label("NWS", ADVISORY.0, ADVISORY.1),
        ]
        .into_iter()
        .enumerate()
        {
            let (side, i) = (k / 4, k % 4);
            let y = -138.0 + 13.0 * (i / 2) as f32;
            let r = match side {
                // Narrower over the RWR scope, room for its keys beside them.
                0 => rect(-158.0 + 28.0 * (i % 2) as f32, y, 26.0, 11.0),
                _ => rect(82.0 + 40.0 * (i % 2) as f32, y, 36.0, 11.0),
            };
            put(elem, r, None, false);
        }
        // The RWR's priority mode and target separate keys, as the F-16's: a third column of the
        // threat lights over the scope (no room beside it between the MFD and the scope's case).
        for (k, (id, action)) in
            [(RwrPri, Action::RwrPriority), (RwrSep, Action::RwrSeparate)].into_iter().enumerate()
        {
            let r = rect(-102.0, -138.0 + 13.0 * k as f32, 26.0, 11.0);
            put(Elem::Lamp(id, None), r, Some(E::Action(action)), false);
        }
        // Countermeasures over the attitude indicator: release caps, counts and released lights
        // left, the jammers and their lights right.
        for (k, (label, action, led, act)) in [
            ("CHAFF", Action::Chaff, LedId::Chaff, ChaffAct),
            ("FLARE", Action::Flare, LedId::Flares, FlareAct),
            ("DECOY", Action::Decoy, LedId::Decoys, DecoyAct),
        ]
        .into_iter()
        .enumerate()
        {
            let y = -138.0 + 16.0 * k as f32;
            put(
                Elem::Label(label, 47, 38),
                rect(-66.0, y, 28.0, 13.0),
                Some(E::Action(action)),
                false,
            );
            put(Elem::Led(led), rect(-35.0, y, 22.0, 13.0), None, false);
            put(Elem::Lamp(act, None), rect(-10.0, y, 14.0, 13.0), None, false);
        }
        put(
            Elem::Lamp(Ecm, None),
            rect(10.0, -138.0, 28.0, 13.0),
            Some(E::Action(Action::Ecm)),
            false,
        );
        put(
            Elem::Lamp(Irj, None),
            rect(10.0, -122.0, 28.0, 13.0),
            Some(E::Action(Action::IrJammer)),
            false,
        );
        put(Elem::Lamp(Jam, None), rect(42.0, -138.0, 24.0, 13.0), None, false);
        put(Elem::Lamp(Av, None), rect(42.0, -122.0, 24.0, 13.0), None, false);
        put(Elem::Lamp(EmvWarn, Some("EMV")), rect(10.0, -106.0, 40.0, 13.0), None, false);
        // Flight instruments round the attitude indicator: airspeed left, altimeter right, the
        // vertical velocity tape over the altimeter; the clock and the AOA tape under the
        // airspeed.
        put(Elem::Dial(Dial::Airspeed), rect(-142.0, -20.0, 56.0, 56.0), None, true);
        put(Elem::Dial(Dial::Altimeter), rect(86.0, -20.0, 56.0, 56.0), None, true);
        put(Elem::Dial(Dial::Vvi), rect(106.0, -104.0, 22.0, 64.0), None, false);
        put(Elem::Dial(Dial::Clock), rect(-148.0, 62.0, 42.0, 42.0), None, true);
        put(Elem::Dial(Dial::Aoa), rect(-92.0, 58.0, 18.0, 56.0), None, false);
        // Lower left: the gear lights over the flap, brake and autopilot caps, the downlock
        // override right of them.
        put(Elem::GearLights, rect(-412.0, 70.0, 40.0, 32.0), None, false);
        for (k, (id, action)) in
            [(Flaps, Action::Flaps), (Brake, Action::Brakes), (Auto, Action::Autopilot)]
                .into_iter()
                .enumerate()
        {
            put(
                Elem::Lamp(id, None),
                rect(-412.0, 112.0 + 18.0 * k as f32, 38.0, 14.0),
                Some(E::Action(action)),
                false,
            );
        }
        put(Elem::Hazard, rect(-364.0, 136.0, 32.0, 20.0), None, false);
        // Lower right: the engine instruments, the caution lights, the fuel quantity.
        for (k, dial) in [Dial::Ftit, Dial::Rpm, Dial::Oil, Dial::Nozzle].into_iter().enumerate() {
            let (col, row) = ((k % 2) as f32, (k / 2) as f32);
            put(
                Elem::Dial(dial),
                rect(82.0 + 52.0 * col, 64.0 + 52.0 * row, 42.0, 42.0),
                None,
                true,
            );
        }
        for (k, elem) in [
            Elem::Lamp(Fuel, None),
            Elem::Lamp(LowFuel, Some("LOW\nFUEL")),
            Elem::Label("ELEC\nGEN", CAUTION.0, CAUTION.1),
            Elem::Label("HYD\nPRESS", CAUTION.0, CAUTION.1),
            Elem::Label("OIL\nPRESS", CAUTION.0, CAUTION.1),
            Elem::Label("ANTI\nSKID", CAUTION.0, CAUTION.1),
            Elem::Label("CANOPY", CAUTION.0, CAUTION.1),
            Elem::Label("SEAT NOT\nARMED", CAUTION.0, CAUTION.1),
            Elem::Lamp(Blank, None),
        ]
        .into_iter()
        .enumerate()
        {
            put(
                elem,
                rect(196.0 + 40.0 * (k % 3) as f32, 70.0 + 17.0 * (k / 3) as f32, 36.0, 14.0),
                None,
                false,
            );
        }
        put(Elem::Dial(Dial::Fuel), rect(340.0, 98.0, 50.0, 50.0), None, true);

        // The HIRES canvas' shape, 160 mm wide (larger than the F-16's).
        let (w, h) = (160.0, 160.0 * crate::mfd::HIRES.1 as f32 / crate::mfd::HIRES.0 as f32);
        Self {
            style: PanelStyle::A10,
            items,
            adi: rect(-58.0, -58.0, 116.0, 116.0),
            mfds: vec![
                MfdUnit { side: MfdSide::Left, screen: rect(-352.0, -100.0, w, h), osbs: true },
                MfdUnit { side: MfdSide::Right, screen: rect(192.0, -100.0, w, h), osbs: true },
            ],
            rwr: Some(rect(-150.0, -106.0, 66.0, 66.0)),
            hsi: Some(rect(-56.0, 72.0, 112.0, 112.0)),
            tfr: None,
            ded: Some(rect(-68.0, -256.0, 86.0, 86.0 * DED_CANVAS.1 as f32 / DED_CANVAS.0 as f32)),
            lmpd: None,
            rmpd: None,
        }
    }

    /// The AV-8B Night Attack's panel (millimetres, x right and y down, life size, the origin at
    /// the panel's centre under the up-front controller; `tools/models/build_cockpit_av8b.py`):
    /// - The up-front controller under the HUD (|x| <= 100, y < -10, raised off the panel): the
    ///   scratchpad (the `ded`) between the COMM 1 / COMM 2 channel windows, two rows of function
    ///   keys (TMR = time, TCN = steerpoint, AWL = ILS, ALT = autopilot), the keypad (as the
    ///   A-10's) with ON/OFF and I/P left of it and CLR / SVE right; the option display unit's
    ///   five windows down its right edge (DCLT, NVG, MENU, VOL, A/A), one more left.
    /// - The two MFDs either side (128 mm screens in square bezels, the right one 14 mm lower).
    /// - Left of the left MFD: the master mode buttons (NAV, VSTOL, A/G), the striped master arm
    ///   and the launch flare switch. Right of the right MFD: the engine display panel (the
    ///   nozzle angle needle, RPM) and the RWR / ECM lights and switches with the countermeasure
    ///   counts.
    /// - Under the controller: the master caution and warning lights, the standby attitude
    ///   indicator with the airspeed indicator and altimeter either side, the AOA and vertical
    ///   velocity tapes and the clock under them; the target buttons (modelled) below.
    /// - The lower panel: left the gear lights (the handle modelled), the flaps panel (CRUISE /
    ///   AUTO / STOL) and the armament panel (MODE, FUZ, QTY, MULT, INTV, the stations 1-7, JETT);
    ///   right the fuel panel (BINGO) and the caution lights.
    pub fn av8b() -> Self {
        use Effect as E;
        use LampId::*;
        let mut items = Vec::new();
        let mut put = |elem: Elem, rect: PxRect, cap: Option<Effect>, round: bool| {
            items.push(Placed { elem, rect, cap, round })
        };
        // Black on grey keys, white on black legends, amber windows; unlit legends of the
        // lights the game doesn't drive (red, amber, green).
        const KEY: (u8, u8) = (5, 14);
        const LEGEND: (u8, u8) = (15, 5);
        const WINDOW: (u8, u8) = (40, 5);
        const WARN: (u8, u8) = (54, 55);
        const CAUTION: (u8, u8) = (16, 43);
        const ADVISORY: (u8, u8) = (45, 44);
        let key = |label| Elem::Label(label, KEY.0, KEY.1);
        let legend = |label| Elem::Label(label, LEGEND.0, LEGEND.1);
        // Up-front controller: the channel windows either side of the scratchpad.
        let ded = rect(-44.0, -146.0, 88.0, 88.0 * DED_CANVAS.1 as f32 / DED_CANVAS.0 as f32);
        let window = |label| Elem::Label(label, WINDOW.0, WINDOW.1);
        put(window("COMM 1\n12"), rect(-72.0, -144.0, 24.0, 14.0), None, false);
        put(window("COMM 2\n3"), rect(48.0, -144.0, 24.0, 14.0), None, false);
        // The function keys in two rows under it.
        for (k, (label, cap)) in [
            ("TMR", Some(E::TimeToggle)),
            ("TOO", None),
            ("IFF", None),
            ("TCN", Some(E::Action(Action::SelectWaypoint))),
            ("AWL", Some(E::Action(Action::Ils))),
            ("WPN", None),
            ("WOF", None),
            ("BCN", None),
            ("ALT", Some(E::Action(Action::Autopilot))),
            ("EMCON", None),
        ]
        .into_iter()
        .enumerate()
        {
            let r = rect(-63.0 + 26.0 * (k % 5) as f32, -104.0 + 15.0 * (k / 5) as f32, 22.0, 12.0);
            put(key(label), r, cap, false);
        }
        // The keypad (the A-10's working keys), ON/OFF and I/P left of it, CLR and SVE right.
        const KEYS: [(&str, Option<Effect>); 12] = [
            ("1\nT-ILS", Some(E::Action(Action::Ils))),
            ("2", None),
            ("3", None),
            ("4\nSTPT", Some(E::Action(Action::SelectWaypoint))),
            ("5", None),
            ("6\nTIME", Some(E::TimeToggle)),
            ("7\nMARK", Some(E::Action(Action::DesignateTarget))),
            ("8", None),
            ("9", None),
            ("-", None),
            ("0", None),
            ("ENT", None),
        ];
        for (k, (label, cap)) in KEYS.into_iter().enumerate() {
            put(
                key(label),
                rect(-28.0 + 20.0 * (k % 3) as f32, -72.0 + 15.0 * (k / 3) as f32, 18.0, 13.0),
                cap,
                false,
            );
        }
        for (k, label) in ["ON/OFF", "I/P", "CLR", "SVE"].into_iter().enumerate() {
            let x = if k < 2 { -60.0 } else { 36.0 };
            put(key(label), rect(x, -72.0 + 15.0 * (k % 2) as f32, 24.0, 12.0), None, false);
        }
        // The option display unit: five windows down the controller's right edge, one left.
        for (k, (label, cap)) in [
            (":DCLT", E::Action(Action::HudDeclutter)),
            (":NVG", E::Action(Action::Nvg)),
            (":MENU", E::Action(Action::Menu)),
            (":VOL", E::Action(Action::Volume)),
            (":A/A", E::HudMode(1)),
        ]
        .into_iter()
        .enumerate()
        {
            put(window(label), rect(76.0, -142.0 + 22.0 * k as f32, 22.0, 14.0), Some(cap), false);
        }
        put(window(":ODU"), rect(-98.0, -98.0, 22.0, 14.0), None, false);
        // Under the controller: the master caution and the warning lights.
        for (k, (elem, cap)) in [
            (Elem::Lamp(Mw, Some("MASTER\nCAUTION")), Some(E::Action(Action::MasterCaution))),
            (Elem::Label("MASTER\nWARNING", WARN.0, WARN.1), None),
            (Elem::Lamp(Fire, None), None),
            (Elem::Lamp(Stall, None), None),
        ]
        .into_iter()
        .enumerate()
        {
            put(elem, rect(-74.0 + 38.0 * k as f32, -3.0, 34.0, 14.0), cap, false);
        }
        // The standby flight instruments round the standby attitude indicator, the AOA and
        // vertical velocity tapes and the clock under them.
        put(Elem::Dial(Dial::Airspeed), rect(-88.0, 24.0, 46.0, 46.0), None, true);
        put(Elem::Dial(Dial::Altimeter), rect(42.0, 24.0, 46.0, 46.0), None, true);
        put(Elem::Dial(Dial::Aoa), rect(-82.0, 80.0, 16.0, 46.0), None, false);
        put(Elem::Dial(Dial::Vvi), rect(66.0, 80.0, 16.0, 46.0), None, false);
        put(Elem::Dial(Dial::Clock), rect(-18.0, 86.0, 36.0, 36.0), None, true);

        // Left of the left MFD: the master modes, the master arm on its stripes, the launch
        // flare switch.
        for (k, (elem, cap)) in [
            (Elem::Lamp(Nav, None), Some(E::HudMode(0))),
            (legend("VSTOL"), None),
            (Elem::Lamp(Gnd, Some("A/G")), Some(E::HudMode(2))),
        ]
        .into_iter()
        .enumerate()
        {
            put(elem, rect(-356.0, -136.0 + 20.0 * k as f32, 44.0, 13.0), cap, false);
        }
        put(Elem::Hazard, rect(-362.0, -70.0, 26.0, 30.0), None, false);
        put(legend("MASTER\nARM"), rect(-362.0, -36.0, 26.0, 12.0), None, false);
        put(
            Elem::Label("LAUNCH\nFLARE", 17, 5),
            rect(-332.0, -70.0, 26.0, 18.0),
            Some(E::Action(Action::Flare)),
            false,
        );
        // Lower left: the gear lights over the brake cap, the flaps panel, the armament panel.
        put(Elem::GearLights, rect(-412.0, 52.0, 40.0, 32.0), None, false);
        put(
            Elem::Lamp(Brake, None),
            rect(-412.0, 170.0, 40.0, 13.0),
            Some(E::Action(Action::Brakes)),
            false,
        );
        put(legend("CRUISE"), rect(-356.0, 56.0, 44.0, 12.0), None, false);
        put(legend("AUTO"), rect(-356.0, 74.0, 44.0, 12.0), None, false);
        put(
            Elem::Lamp(Flaps, Some("STOL")),
            rect(-356.0, 92.0, 44.0, 12.0),
            Some(E::Action(Action::Flaps)),
            false,
        );
        for (k, label) in ["MODE", "FUZ", "QTY", "MULT", "INTV"].into_iter().enumerate() {
            put(legend(label), rect(-280.0 + 35.0 * k as f32, 51.0, 28.0, 10.0), None, false);
        }
        put(Elem::Label("JETT", WARN.0, WARN.1), rect(-170.0, 150.0, 34.0, 13.0), None, false);

        // Right of the right MFD: the engine display panel (the nozzle angle needle over the
        // windows), the RWR and ECM lights and switches outboard of it, the countermeasures
        // under it.
        put(Elem::Dial(Dial::NozzleAngle), rect(312.0, -144.0, 46.0, 46.0), None, true);
        put(legend("RPM"), rect(300.0, -86.0, 20.0, 10.0), None, false);
        put(Elem::Led(LedId::Thrust), rect(324.0, -87.0, 40.0, 12.0), None, false);
        for (k, label) in ["JPT", "STAB", "FLOW", "H2O"].into_iter().enumerate() {
            put(legend(label), rect(300.0, -70.0 + 14.0 * k as f32, 20.0, 10.0), None, false);
        }
        for (k, (elem, cap)) in [
            (Elem::Lamp(Rad, Some("LAUNCH")), None),
            (Elem::Lamp(Trak, Some("TRACK")), None),
            (Elem::Lamp(Ir, None), None),
            (Elem::Lamp(Jam, None), None),
            (Elem::Lamp(Ecm, None), Some(E::Action(Action::Ecm))),
            (Elem::Lamp(Irj, None), Some(E::Action(Action::IrJammer))),
            (Elem::Lamp(Av, None), None),
            (Elem::Lamp(EmvWarn, Some("EMV")), None),
        ]
        .into_iter()
        .enumerate()
        {
            put(elem, rect(372.0, -104.0 + 16.0 * k as f32, 48.0, 13.0), cap, false);
        }
        for (k, (label, action, led, act)) in [
            ("CHAFF", Action::Chaff, LedId::Chaff, ChaffAct),
            ("FLARE", Action::Flare, LedId::Flares, FlareAct),
            ("DECOY", Action::Decoy, LedId::Decoys, DecoyAct),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 16.0 * k as f32;
            let cap = Some(E::Action(action));
            put(Elem::Label(label, 47, 38), rect(294.0, y, 28.0, 13.0), cap, false);
            put(Elem::Led(led), rect(325.0, y, 22.0, 13.0), None, false);
            put(Elem::Lamp(act, None), rect(350.0, y, 16.0, 13.0), None, false);
        }
        // Lower right: the fuel panel under the right MFD, the caution lights outboard.
        put(Elem::Led(LedId::Fuel), rect(128.0, 62.0, 52.0, 14.0), None, false);
        put(Elem::Lamp(Fuel, None), rect(188.0, 62.0, 34.0, 14.0), None, false);
        put(Elem::Lamp(LowFuel, Some("BINGO")), rect(228.0, 62.0, 34.0, 14.0), None, false);
        for (k, elem) in [
            Elem::Lamp(Eng, Some("ENG")),
            Elem::Lamp(Fc, Some("FLT CTL")),
            Elem::Label("H2O", CAUTION.0, CAUTION.1),
            Elem::Label("L PUMP", CAUTION.0, CAUTION.1),
            Elem::Label("R PUMP", CAUTION.0, CAUTION.1),
            Elem::Label("APU\nGEN", ADVISORY.0, ADVISORY.1),
        ]
        .into_iter()
        .enumerate()
        {
            put(
                elem,
                rect(290.0 + 42.0 * (k % 3) as f32, 58.0 + 18.0 * (k / 3) as f32, 38.0, 13.0),
                None,
                false,
            );
        }

        // The HIRES canvas' shape, 144 mm wide, in square bezels level with each other either
        // side of the controller.
        let (w, h) = (144.0, 144.0 * crate::mfd::HIRES.1 as f32 / crate::mfd::HIRES.0 as f32);
        Self {
            style: PanelStyle::Av8b,
            items,
            adi: rect(-30.0, 18.0, 60.0, 60.0),
            mfds: vec![
                MfdUnit { side: MfdSide::Left, screen: rect(-126.0 - w, -124.0, w, h), osbs: true },
                MfdUnit { side: MfdSide::Right, screen: rect(126.0, -124.0, w, h), osbs: true },
            ],
            rwr: None,
            hsi: None,
            tfr: None,
            ded: Some(ded),
            lmpd: None,
            rmpd: None,
        }
    }

    /// The RAH-66's panel (millimetres, x right and y down from the centre), after the
    /// mock-up's: the two large MFDs side by side in the middle (the monochrome one left, the
    /// colour one right), a portrait multipurpose display outboard of each, centred between the
    /// MFD bezels and the panel's ends at +-420 mm (`lmpd`, `rmpd`: the threats and warnings left, the
    /// engines right), the keyboard with its scratchpad (the `ded`) under them. No dials and no
    /// warning lights but the master caution: the landing gear and parking brake keys top left,
    /// the HUD modes and the master caution top centre, the jammers, the laser and the emergency
    /// panel top right; on the pedestal the standby attitude display (a screen) with the
    /// countermeasures under it, and the flight control keys in a column between it and the
    /// keyboard.
    pub fn rah66() -> Self {
        use Effect as E;
        let mut items = Vec::new();
        let mut put = |elem: Elem, rect: PxRect, cap: Option<Effect>| {
            items.push(Placed { elem, rect, cap, round: false })
        };
        // Black on grey keys.
        const KEY: (u8, u8) = (5, 14);
        let key = |label| Elem::Label(label, KEY.0, KEY.1);
        // Top left: the landing gear panel.
        put(key("LDG\nGEAR"), rect(-212.0, -132.0, 40.0, 24.0), Some(E::Action(Action::Gear)));
        put(key("PARK\nBRAKE"), rect(-166.0, -132.0, 40.0, 24.0), Some(E::Action(Action::Brakes)));
        // Top centre: the HUD modes, the master caution.
        for (k, label) in ["NAV", "A-A", "A-G"].into_iter().enumerate() {
            put(
                key(label),
                rect(-104.0 + 32.0 * k as f32, -128.0, 28.0, 16.0),
                Some(E::HudMode(k as u8)),
            );
        }
        put(
            Elem::Lamp(LampId::Mw, Some("MASTER\nCAUTION")),
            rect(16.0, -134.0, 56.0, 28.0),
            Some(E::Action(Action::MasterCaution)),
        );
        // Top right: the jammers and the laser, the emergency panel's stripes.
        for (k, (label, action)) in
            [("ECM", Action::Ecm), ("IRJ", Action::IrJammer), ("LASE", Action::DesignateTarget)]
                .into_iter()
                .enumerate()
        {
            put(
                key(label),
                rect(84.0 + 32.0 * k as f32, -128.0, 28.0, 16.0),
                Some(E::Action(action)),
            );
        }
        put(Elem::Hazard, rect(182.0, -134.0, 30.0, 28.0), None);
        // On the pedestal, between the standby attitude display and the keyboard: the flight
        // control keys in a column, the autopilot's (lit while engaged) at its foot.
        for (k, (label, cap)) in [
            ("DCLT", E::Action(Action::HudDeclutter)),
            ("NVG", E::Action(Action::Nvg)),
            ("MENU", E::Action(Action::Menu)),
            ("VOL", E::Action(Action::Volume)),
            ("TIME", E::TimeToggle),
        ]
        .into_iter()
        .enumerate()
        {
            put(key(label), rect(-121.0, 128.0 + 18.0 * k as f32, 40.0, 16.0), Some(cap));
        }
        put(
            Elem::Lamp(LampId::Auto, None),
            rect(-121.0, 218.0, 40.0, 16.0),
            Some(E::Action(Action::Autopilot)),
        );
        put(
            Elem::Lamp(LampId::Hover, None),
            rect(-164.0, 218.0, 40.0, 16.0),
            Some(E::Action(Action::Hover)),
        );
        // Under the standby attitude display: the countermeasures.
        let adi = rect(-206.0, 120.0, 64.0, 64.0);
        for (k, (label, action)) in
            [("CHAFF", Action::Chaff), ("FLARE", Action::Flare), ("DECOY", Action::Decoy)]
                .into_iter()
                .enumerate()
        {
            put(
                key(label),
                rect(adi.x + adi.w / 2.0 - 46.0 + 32.0 * k as f32, 196.0, 28.0, 16.0),
                Some(E::Action(action)),
            );
        }
        // The keyboard under the scratchpad (the A-10's and the F-16's keys that work).
        let ded = rect(-60.0, 122.0, 120.0, 120.0 * DED_CANVAS.1 as f32 / DED_CANVAS.0 as f32);
        const KEYS: [(&str, Option<Effect>); 12] = [
            ("1\nT-ILS", Some(E::Action(Action::Ils))),
            ("2", None),
            ("3", None),
            ("4\nSTPT", Some(E::Action(Action::SelectWaypoint))),
            ("5", None),
            ("6", None),
            ("7\nMARK", Some(E::Action(Action::DesignateTarget))),
            ("8", None),
            ("9", None),
            ("CLR", None),
            ("0", None),
            ("ENT", None),
        ];
        for (k, (label, cap)) in KEYS.into_iter().enumerate() {
            put(
                key(label),
                rect(
                    -56.0 + 38.0 * (k % 3) as f32,
                    ded.y + ded.h + 6.0 + 16.0 * (k / 3) as f32,
                    34.0,
                    13.0,
                ),
                cap,
            );
        }
        // The HIRES canvas' shape, 200 mm wide (8 inches); the displays 100 by 140 mm.
        let (w, h) = (200.0, 200.0 * crate::mfd::HIRES.1 as f32 / crate::mfd::HIRES.0 as f32);
        Self {
            style: PanelStyle::Rah66,
            items,
            adi,
            mfds: vec![
                MfdUnit { side: MfdSide::Left, screen: rect(-218.0, -76.0, w, h), osbs: true },
                MfdUnit { side: MfdSide::Right, screen: rect(18.0, -76.0, w, h), osbs: true },
            ],
            rwr: None,
            hsi: None,
            tfr: None,
            ded: Some(ded),
            lmpd: Some(rect(-378.0, -44.0, 100.0, 140.0)),
            rmpd: Some(rect(278.0, -44.0, 100.0, 140.0)),
        }
    }

    /// The OH-58D's panel (millimetres, x right and y down, life size;
    /// `tools/models/build_cockpit_oh58.py`), the F-117A's faces rearranged after the OH-58D's
    /// one wide panel under a deep glareshield, spanning both seats, its origin on the cabin's
    /// centre line (the pilot sits in the right seat, about x 360):
    /// - Two large MFDs (200 mm, as the RAH-66's) left and right of the centre column, the right
    ///   one in front of the pilot.
    /// - The centre column (|x| < 178): the data entry display on top with FLAPS, BRAKE, AUTO and
    ///   BAY left of it; under it the standby airspeed indicator, attitude indicator and altimeter,
    ///   the AOA and vertical velocity tapes and the clock; right of them a vertical strip of the
    ///   warning lights over the engine instruments.
    /// - Outboard of the right MFD (x > 422): fuel quantity, the fuel and thrust readouts, the EMV
    ///   gauge.
    /// - The console down the pedestal between the seats (x >= 1500, centred on x 2000, y growing
    ///   aft down it): the up-front display's two rows of caps over the CDU's keyboard, the
    ///   caution lights, the countermeasures (the target buttons, modelled, behind them).
    ///
    /// No gear lights (skids).
    pub fn oh58() -> Self {
        let m = Self::modern();
        let items = m.items.iter().filter(|p| !matches!(p.elem, Elem::GearLights)).map(|p| {
            let r = p.rect;
            let at = |x: f32, y: f32| rect(x, y, r.w, r.h);
            let by = |dx: f32, dy: f32| rect(r.x + dx, r.y + dy, r.w, r.h);
            let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
            use LampId::*;
            let rect = match p.elem {
                // The standby instruments under the data entry display.
                Elem::Dial(Dial::Airspeed) => at(-110.0, -40.0),
                Elem::Dial(Dial::Altimeter) => at(16.0, -40.0),
                Elem::Dial(Dial::Aoa) => at(-110.0, 12.0),
                Elem::Dial(Dial::Vvi) => at(-88.0, 12.0),
                Elem::Dial(Dial::Clock) => at(16.0, 12.0),
                // The engine instruments under the warning strip.
                Elem::Dial(Dial::Rpm) => at(68.0, -22.0),
                Elem::Dial(Dial::Ftit) => at(116.0, -22.0),
                Elem::Dial(Dial::Oil) => at(68.0, 26.0),
                Elem::Dial(Dial::Nozzle) => at(116.0, 26.0),
                // The warning strip: two columns right of the data entry display.
                Elem::Lamp(Mw, _) => at(68.0, -104.0),
                Elem::Lamp(Stall, _) => at(68.0, -88.0),
                Elem::Lamp(Trak, _) => at(68.0, -72.0),
                Elem::Lamp(Ir, _) => at(68.0, -59.0),
                Elem::Lamp(Rad, _) => at(68.0, -46.0),
                Elem::Lamp(Fire, _) => at(114.0, -104.0),
                Elem::Lamp(Eng, _) => at(114.0, -88.0),
                Elem::Lamp(Fc, _) => at(114.0, -72.0),
                Elem::Lamp(Jam, _) => at(114.0, -59.0),
                Elem::Lamp(Av, _) => at(114.0, -46.0),
                // FLAPS, BRAKE, AUTO and BAY left of the data entry display.
                Elem::Lamp(Flaps, _) => at(-168.0, -104.0),
                Elem::Lamp(Brake, _) => at(-126.0, -104.0),
                Elem::Lamp(Auto, _) => at(-168.0, -86.0),
                Elem::Lamp(Bay, _) => at(-126.0, -86.0),
                // Fuel, the engine readouts and the EMV gauge outboard of the right MFD.
                Elem::Dial(Dial::Fuel) => at(436.0, -84.0),
                Elem::Lamp(LowFuel, _) => at(484.0, -70.0),
                Elem::Led(LedId::Fuel) => at(436.0, -34.0),
                Elem::Label("FUEL", ..) => at(470.0, -34.0),
                Elem::Led(LedId::Thrust) => at(436.0, -16.0),
                Elem::Label("THRUST", ..) => at(470.0, -16.0),
                Elem::Label("EGR\nEAR", ..) => at(436.0, 2.0),
                Elem::Gauge => at(460.0, 2.0),
                Elem::Label("EMV", ..) => at(436.0, 20.0),
                Elem::Lamp(EmvWarn, _) => at(460.0, 20.0),
                // The caution lights and countermeasures down the console, behind the CDU.
                _ if cx >= 1100.0 => by(818.0, 110.0),
                // The up-front display's caps to the console's top, two rows.
                _ if cx.abs() < 80.0 && cy < -100.0 => by(2000.0, -5.0),
                _ if cx.abs() < 80.0 && cy < -50.0 => by(2000.0, -55.0),
                _ => r,
            };
            Placed { rect, ..*p }
        });
        // The CDU's keyboard under the caps (its letters and digits; the caps above work).
        let keys = [
            "1", "2", "3", "A", "B", "C", "4", "5", "6", "D", "E", "F", "7", "8", "9", "CLR", "0",
            "ENT",
        ];
        let keyboard = keys.into_iter().enumerate().map(|(k, label)| Placed {
            elem: Elem::Label(label, 5, 14),
            rect: rect(1931.0 + 24.0 * (k % 6) as f32, -102.0 + 16.0 * (k / 6) as f32, 20.0, 13.0),
            cap: None,
            round: false,
        });
        // HOVER under AUTO.
        let hover = Placed {
            elem: Elem::Lamp(LampId::Hover, None),
            rect: rect(-168.0, -68.0, 38.0, 14.0),
            cap: Some(Effect::Action(Action::Hover)),
            round: false,
        };
        let items = items.chain(keyboard).chain([hover]).collect();
        let (w, h) = (200.0, 200.0 * crate::mfd::HIRES.1 as f32 / crate::mfd::HIRES.0 as f32);
        Self {
            style: PanelStyle::Oh58,
            items,
            adi: rect(-60.0, -40.0, 68.0, 68.0),
            mfds: vec![
                MfdUnit { side: MfdSide::Left, screen: rect(-400.0, -80.0, w, h), osbs: true },
                MfdUnit { side: MfdSide::Right, screen: rect(200.0, -80.0, w, h), osbs: true },
            ],
            rwr: None,
            hsi: None,
            tfr: None,
            ded: Some(rect(
                -60.0,
                -104.0,
                DED_W,
                DED_W * DED_CANVAS.1 as f32 / DED_CANVAS.0 as f32,
            )),
            lmpd: None,
            rmpd: None,
        }
    }

    /// The F-16C's panel (millimetres, x right and y down, unfolded), after the Block 50's:
    /// - The centre column (|x| <= 85): the up-front controls on a pedestal under the HUD
    ///   (y < -46): the round override buttons on top (COM1 = volume, MENU = menu, A-A, A-G),
    ///   the keypad with its page labels (T-ILS, STPT, TIME and MARK work) and the declutter
    ///   and NVG keys; under it the airspeed indicator, attitude indicator and altimeter, then
    ///   the AOA indicator, HSI and vertical velocity indicator, and the clock under the HSI.
    /// - The wings (85 < |x| <= 335): the MFDs in wells; top left the flap, brake and
    ///   autopilot caps, the threat lights with the master caution and stall light, and the
    ///   RWR scope next to the centre; top right the data entry display with the oil and
    ///   nozzle gauges right of it; RPM, FTIT and fuel down the right edge.
    /// - The advisory light panel on the right console ahead of the stick (600 <= x < 1100,
    ///   centred on x 700, y 0); the countermeasure panel on the right slope from the console
    ///   up to the panel (x >= 1100, centred on x 1200; y grows down the slope). The left slope
    ///   (x <= -600, centred on x -700) holds the gear lights left of the gear handle, and the
    ///   target buttons (modelled, not faces) below them.
    ///
    /// No EMV gauge: the RWR shows the radars.
    pub fn f16() -> Self {
        use Effect as E;
        use LampId::*;
        let mut items = Vec::new();
        let mut put = |elem: Elem, rect: PxRect, cap: Option<Effect>, round: bool| {
            items.push(Placed { elem, rect, cap, round })
        };
        // Up-front controls: the override buttons (round, black on light grey), the keypad and
        // the keys right of it (black on grey).
        const OVERRIDE: (u8, u8) = (5, 7);
        const KEY: (u8, u8) = (5, 14);
        for (k, (label, cap)) in [
            ("COM1", Some(E::Action(Action::Volume))),
            ("COM2", None),
            ("IFF", None),
            ("MENU", Some(E::Action(Action::Menu))),
            ("A-A", Some(E::HudMode(1))),
            ("A-G", Some(E::HudMode(2))),
        ]
        .into_iter()
        .enumerate()
        {
            let r = rect(-69.0 + 24.0 * k as f32, -157.0, 18.0, 18.0);
            match label {
                "A-A" => put(Elem::Lamp(Air, Some("A-A")), r, cap, true),
                "A-G" => put(Elem::Lamp(Gnd, Some("A-G")), r, cap, true),
                _ => put(Elem::Label(label, OVERRIDE.0, OVERRIDE.1), r, cap, true),
            }
        }
        const KEYS: [(&str, Option<Effect>); 12] = [
            ("1\nT-ILS", Some(E::Action(Action::Ils))),
            ("2\nALOW", None),
            ("3", None),
            ("4\nSTPT", Some(E::Action(Action::SelectWaypoint))),
            ("5\nCRUS", None),
            ("6\nTIME", Some(E::TimeToggle)),
            ("7\nMARK", Some(E::Action(Action::DesignateTarget))),
            ("8\nFIX", None),
            ("9\nA-CAL", None),
            ("RCL", None),
            ("0\nM-SEL", None),
            ("ENTR", None),
        ];
        for (k, (label, cap)) in KEYS.into_iter().enumerate() {
            put(
                Elem::Label(label, KEY.0, KEY.1),
                rect(-63.0 + 24.0 * (k % 3) as f32, -132.0 + 20.0 * (k / 3) as f32, 22.0, 16.0),
                cap,
                false,
            );
        }
        put(
            Elem::Label("DCLT", KEY.0, KEY.1),
            rect(15.0, -132.0, 20.0, 16.0),
            Some(E::Action(Action::HudDeclutter)),
            false,
        );
        put(
            Elem::Label("NVG", KEY.0, KEY.1),
            rect(15.0, -112.0, 20.0, 16.0),
            Some(E::Action(Action::Nvg)),
            false,
        );
        put(Elem::Label("WX", KEY.0, KEY.1), rect(15.0, -92.0, 20.0, 16.0), None, false);
        put(Elem::Label("FLIR", KEY.0, KEY.1), rect(15.0, -72.0, 20.0, 16.0), None, false);
        // Flight instruments as in the F-16C: airspeed and altimeter over the attitude
        // indicator's upper corners, the AOA and vertical velocity tapes either side of it, the
        // HSI under it.
        put(Elem::Dial(Dial::Airspeed), rect(-84.0, -44.0, 42.0, 42.0), None, true);
        put(Elem::Dial(Dial::Altimeter), rect(42.0, -44.0, 42.0, 42.0), None, true);
        put(Elem::Dial(Dial::Aoa), rect(-58.0, 2.0, 18.0, 52.0), None, false);
        put(Elem::Dial(Dial::Vvi), rect(40.0, 2.0, 18.0, 52.0), None, false);
        // Unlit legends of the warning lights the game doesn't drive (red and amber).
        const WARN: (u8, u8) = (54, 55);
        const CAUTION: (u8, u8) = (16, 43);
        // Left eyebrow under the glare shield: master caution, the stall warning and the
        // game's radar track and IR launch warnings.
        let mc = Some(E::Action(Action::MasterCaution));
        put(Elem::Lamp(Mw, Some("MASTER\nCAUTION")), rect(-262.0, -154.0, 50.0, 20.0), mc, false);
        put(Elem::Lamp(Stall, None), rect(-262.0, -131.0, 50.0, 11.0), None, false);
        put(Elem::Lamp(Trak, Some("TRACK")), rect(-262.0, -117.0, 24.0, 11.0), None, false);
        put(Elem::Lamp(Ir, Some("IR")), rect(-236.0, -117.0, 24.0, 11.0), None, false);
        // Threat warning prime: a column of six push-lights left of the RWR, dark keys with
        // light grey legends (LAUNCH lights with a radar missile launch; MODE switches the
        // priority mode, T target separate).
        const PRIME: (u8, u8) = (7, 38);
        for (k, (elem, cap)) in [
            (Elem::Label("HAND\nOFF", PRIME.0, PRIME.1), None),
            (Elem::Lamp(Rad, Some("LAUNCH")), None),
            (Elem::Lamp(RwrPri, None), Some(E::Action(Action::RwrPriority))),
            (Elem::Label("UNK", PRIME.0, PRIME.1), None),
            (Elem::Label("SYS\nTEST", PRIME.0, PRIME.1), None),
            (Elem::Lamp(RwrSep, None), Some(E::Action(Action::RwrSeparate))),
        ]
        .into_iter()
        .enumerate()
        {
            put(elem, rect(-206.0, -154.0 + 12.8 * k as f32, 22.0, 11.0), cap, false);
        }
        // Gear lights left of the gear handle on the left slope (the landing gear panel), the
        // flaps, brakes and autopilot under them.
        put(Elem::GearLights, rect(-746.0, -104.0, 40.0, 32.0), None, false);
        for (k, (id, action)) in
            [(Flaps, Action::Flaps), (Brake, Action::Brakes), (Auto, Action::Autopilot)]
                .into_iter()
                .enumerate()
        {
            put(
                Elem::Lamp(id, None),
                rect(-764.0, -62.0 + 18.0 * k as f32, 38.0, 14.0),
                Some(E::Action(action)),
                false,
            );
        }
        // Clock at the top of the right slope (the right auxiliary console).
        put(Elem::Dial(Dial::Clock), rect(1150.0, -136.0, 44.0, 44.0), None, true);
        // Right eyebrow under the glare shield: the engine fire, engine and flight control
        // warnings among the F-16's others.
        for (k, elem) in [
            Elem::Lamp(Fire, Some("ENG\nFIRE")),
            Elem::Lamp(Eng, Some("ENGINE")),
            Elem::Label("HYD/OIL\nPRESS", WARN.0, WARN.1),
            Elem::Lamp(Fc, Some("FLCS")),
            Elem::Label("DBU\nON", WARN.0, WARN.1),
            Elem::Label("TO/LDG\nCONFIG", WARN.0, WARN.1),
            Elem::Label("CANOPY", WARN.0, WARN.1),
            Elem::Label("OXY\nLOW", WARN.0, WARN.1),
        ]
        .into_iter()
        .enumerate()
        {
            put(
                elem,
                rect(220.0 + 25.0 * (k % 2) as f32, -154.0 + 13.0 * (k / 2) as f32, 23.0, 11.5),
                None,
                false,
            );
        }
        // Engine instruments down the right edge (oil pressure, nozzle position, RPM, FTIT),
        // fuel quantity under them.
        for (k, dial) in
            [Dial::Oil, Dial::Nozzle, Dial::Rpm, Dial::Ftit, Dial::Fuel].into_iter().enumerate()
        {
            put(Elem::Dial(dial), rect(273.0, -127.0 + 45.0 * k as f32, 42.0, 42.0), None, true);
        }
        // The caution light panel on the right console (layout x >= 600, see the doc).
        for (k, elem) in [
            Elem::Label("ELEC\nSYS", CAUTION.0, CAUTION.1),
            Elem::Label("SEAT NOT\nARMED", CAUTION.0, CAUTION.1),
            Elem::Label("ANTI\nSKID", CAUTION.0, CAUTION.1),
            Elem::Lamp(Fuel, None),
            Elem::Lamp(LowFuel, Some("LOW\nFUEL")),
            Elem::Lamp(Jam, None),
            Elem::Lamp(Av, None),
            Elem::Lamp(EmvWarn, Some("EMV")),
            Elem::Lamp(Blank, None),
        ]
        .into_iter()
        .enumerate()
        {
            put(
                elem,
                rect(640.0 + 42.0 * (k % 3) as f32, -38.0 + 19.0 * (k / 3) as f32, 36.0, 14.0),
                None,
                false,
            );
        }
        // Countermeasures on the right slope: release caps, counts, released lights; jammers.
        for (k, (label, action, led, act)) in [
            ("CHAFF", Action::Chaff, LedId::Chaff, ChaffAct),
            ("FLARE", Action::Flare, LedId::Flares, FlareAct),
            ("DECOY", Action::Decoy, LedId::Decoys, DecoyAct),
        ]
        .into_iter()
        .enumerate()
        {
            let y = -70.0 + 17.0 * k as f32;
            put(
                Elem::Label(label, 47, 38),
                rect(1150.0, y, 30.0, 13.0),
                Some(E::Action(action)),
                false,
            );
            put(Elem::Led(led), rect(1183.0, y, 20.0, 13.0), None, false);
            put(Elem::Lamp(act, None), rect(1206.0, y, 16.0, 13.0), None, false);
        }
        put(
            Elem::Lamp(Ecm, None),
            rect(1150.0, -16.0, 34.0, 13.0),
            Some(E::Action(Action::Ecm)),
            false,
        );
        put(
            Elem::Lamp(Irj, None),
            rect(1188.0, -16.0, 34.0, 13.0),
            Some(E::Action(Action::IrJammer)),
            false,
        );

        // The HIRES canvas' shape (231x180), 130 mm wide.
        let (w, h) = (130.0, 130.0 * crate::mfd::HIRES.1 as f32 / crate::mfd::HIRES.0 as f32);
        let screens = [rect(-240.0, -10.0, w, h), rect(110.0, -10.0, w, h)];
        Self {
            style: PanelStyle::F16,
            items,
            adi: rect(-34.0, -30.0, 68.0, 68.0),
            mfds: vec![
                MfdUnit { side: MfdSide::Left, screen: screens[0], osbs: true },
                MfdUnit { side: MfdSide::Right, screen: screens[1], osbs: true },
            ],
            rwr: Some(rect(-176.0, -154.0, 76.0, 76.0)),
            hsi: Some(rect(-34.0, 44.0, 68.0, 68.0)),
            tfr: None,
            ded: Some(rect(95.0, -152.0, DED_W, DED_W * DED_CANVAS.1 as f32 / DED_CANVAS.0 as f32)),
            lmpd: None,
            rmpd: None,
        }
    }
}

/// Width of the data entry display on the F-16 and F-117A panels (mm); its height keeps its canvas' shape.
const DED_W: f32 = 120.0;
/// The data entry display's canvas: 24 columns of 5 lines of the flight font at
/// `DED_SCALE`, with margins.
pub const DED_COLS: i32 = 24;
pub const DED_LINES: i32 = 5;
pub const DED_SCALE: i32 = 3;
pub const DED_CANVAS: (usize, usize) =
    ((DED_COLS * 4 * DED_SCALE + 8) as usize, (DED_LINES * 8 * DED_SCALE + 3) as usize);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mfd::{MfdState, RightMfd, osb};

    fn all() -> [PanelLayout; 11] {
        [
            PanelLayout::classic(),
            PanelLayout::modern(),
            PanelLayout::f16(),
            PanelLayout::uh60(),
            PanelLayout::ah1z(),
            PanelLayout::tornado(),
            PanelLayout::a10(),
            PanelLayout::rah66(),
            PanelLayout::av8b(),
            PanelLayout::oh58(),
            PanelLayout::ch47(),
        ]
    }

    /// Faces and caps do not overlap each other or the screens.
    #[test]
    fn no_overlaps() {
        for l in all() {
            let mut rects: Vec<PxRect> = l.items.iter().map(|p| p.rect).collect();
            rects.push(l.adi);
            rects.extend(l.rwr.iter().chain(&l.hsi).chain(&l.tfr).chain(&l.ded));
            rects.extend(l.lmpd.iter().chain(&l.rmpd));
            for m in &l.mfds {
                rects.push(m.screen);
                rects.extend(l.osbs(m).into_iter().map(|(_, r)| r));
            }
            for (i, a) in rects.iter().enumerate() {
                for b in &rects[i + 1..] {
                    assert!(!a.overlaps(b), "{:?}: {a:?} overlaps {b:?}", l.style);
                }
            }
        }
    }

    #[test]
    fn lamps() {
        let lamps = |l: &PanelLayout| -> Vec<LampId> {
            l.items
                .iter()
                .filter_map(|p| if let Elem::Lamp(id, _) = p.elem { Some(id) } else { None })
                .collect()
        };
        // Every lamp but the helicopters' HOVER and the RWR scope's keys.
        assert_eq!(lamps(&PanelLayout::classic()).len(), crate::panel::LAMP_COUNT - 3);
        let modern = lamps(&PanelLayout::modern());
        for id in [
            LampId::Map,
            LampId::Tac,
            LampId::Flir,
            LampId::Wpn,
            LampId::CamDir,
            LampId::Way,
            LampId::Ord,
            LampId::Tc,
        ] {
            assert!(!modern.contains(&id), "{id:?} is a bezel button in the modern panel");
        }
    }

    /// The modern panels' MFDs have all twenty bezel buttons, the side ones outside the
    /// screen's left and right edges, top to bottom.
    #[test]
    fn side_bezel_buttons() {
        for l in all().into_iter().filter(|l| l.style != PanelStyle::Classic) {
            for m in &l.mfds {
                let caps = l.osbs(m);
                assert_eq!(caps.len(), OSB_COUNT as usize, "{:?}", l.style);
                let s = m.screen;
                for (edge, range) in [(-1.0, 10..15), (1.0, 15..20)] {
                    let side = &caps[range];
                    for (a, b) in side.iter().zip(&side[1..]) {
                        assert!(a.1.y + a.1.h <= b.1.y);
                    }
                    for (_, r) in side {
                        assert!(if edge < 0.0 { r.x + r.w < s.x } else { r.x > s.x + s.w });
                        assert!(r.y >= s.y && r.y + r.h <= s.y + s.h);
                    }
                }
            }
        }
    }

    /// Every bezel function of every page has a button.
    #[test]
    fn bezel_buttons() {
        for l in [
            PanelLayout::modern(),
            PanelLayout::f16(),
            PanelLayout::uh60(),
            PanelLayout::ah1z(),
            PanelLayout::tornado(),
            PanelLayout::a10(),
            PanelLayout::rah66(),
            PanelLayout::av8b(),
            PanelLayout::oh58(),
            PanelLayout::ch47(),
        ] {
            bezel_buttons_of(&l);
        }
    }

    fn bezel_buttons_of(l: &PanelLayout) {
        for right in [RightMfd::Camera, RightMfd::Way, RightMfd::Wpn, RightMfd::Ord] {
            for left in [LeftMfd::Map, LeftMfd::Tac] {
                let s = MfdState { left, right, ..Default::default() };
                for m in &l.mfds {
                    let slots: Vec<u8> = l.osbs(m).into_iter().map(|(s, _)| s).collect();
                    for slot in 0..OSB_COUNT {
                        if osb(m.side, slot, &s).is_some() {
                            assert!(slots.contains(&slot));
                        }
                    }
                }
            }
        }
    }
}

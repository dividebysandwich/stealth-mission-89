//! The F-14's displays' pictures: the pilot's VDI and HSD, the RIO's DDD, TID and ECMD.
//!
//! The pictures are `f14_scopes`'s, from the radar (`Awg9`) and the frame's scene
//! (`avionics::TomcatScene`), in the modes the panel sets. Each canvas is drawn in its tube's
//! phosphor (`colours`: index 0 the dark tube, 1 to 100 a ramp to the phosphor's colour). The
//! round tubes' canvases are their squares (the cockpit model's disc shows the inscribed
//! circle). In TV mode the TID's and the VDI's canvases are see-through (index 0): the pod's
//! picture (`trackcam`) shows behind the LANTIRN's symbology.

use bevy::prelude::*;

use super::avionics::{Cursor, TomcatScene, video_up};
use super::{Alq126, HsdMode, TidMode, TidOption, TomcatPanel, VdiMode};
use crate::awg9::{Awg9, Awg9Controls};
use crate::canvas::Canvas;
use crate::f14_scopes::{self as scopes, Threat, TidView};
use crate::font::Fonts;
use crate::rwr::EmitterView;

/// The displays' canvases (drawing units) and their texels per unit.
pub const VDI_CANVAS: (usize, usize) = (400, 400);
pub const HSD_CANVAS: (usize, usize) = (240, 240);
pub const DDD_CANVAS: (usize, usize) = (240, 240);
pub const TID_CANVAS: (usize, usize) = (320, 320);
pub const ECMD_CANVAS: (usize, usize) = (160, 160);
pub const SCALE: usize = 2;

/// The green phosphor of all five tubes.
pub fn colours() -> std::sync::Arc<[u32; 256]> {
    crate::intruder::phosphor([3, 10, 4], [70, 255, 110])
}

/// The DDD's tube: lit, its marks dark (index 0 the lit screen, the ramp's ink darker the
/// brighter it is drawn), the antenna's line (`f14_scopes::DDD_SWEEP`) white.
pub fn ddd_colours() -> std::sync::Arc<[u32; 256]> {
    let (screen, mark) = ([104.0f32, 176.0, 142.0], [12.0f32, 40.0, 28.0]);
    let rgba = |c: [f32; 3]| u32::from_le_bytes([c[0] as u8, c[1] as u8, c[2] as u8, 255]);
    let mut lut = [rgba(screen); 256];
    for (i, v) in lut.iter_mut().enumerate().skip(1).take(100) {
        let t = i as f32 / 100.0;
        *v = rgba(std::array::from_fn(|k| screen[k] + (mark[k] - screen[k]) * t));
    }
    lut[scopes::DDD_SWEEP as usize] = rgba([232.0, 255.0, 240.0]);
    std::sync::Arc::new(lut)
}

/// What the displays draw from.
pub struct Inputs<'a> {
    pub fonts: &'a Fonts,
    pub panel: &'a TomcatPanel,
    pub scene: &'a TomcatScene,
    pub radar: &'a Awg9,
    pub controls: &'a Awg9Controls,
    pub emitters: &'a [EmitterView],
    /// Seconds (for blinking).
    pub t: f32,
    /// The HUD's frame: the VDI repeats its symbols.
    pub hud: Option<&'a crate::hud::HudFrame>,
}

/// A TV mode with no picture: the dark tube and NO VIDEO.
fn no_video(c: &mut Canvas) {
    c.clear(0);
    let (w, h) = (c.w as f32, c.h as f32);
    crate::font::stroke_text_at(
        c,
        w / 2.0,
        h / 2.0,
        "NO VIDEO",
        h * 0.06,
        1.0,
        crate::intruder::ink(0.6),
        crate::font::Align::Centre,
    );
}

/// The LANTIRN's symbology or the TCS's (the picture shows through), or NO VIDEO with the pod
/// selected and off. On a
/// round tube (`round`, the TID) it fills the square inside the circle.
fn lantirn(c: &mut Canvas, i: &Inputs, round: bool) {
    if !video_up(i.panel) {
        return no_video(c);
    }
    // The VIDEO switch: the LANTIRN's symbology, or the TCS camera's.
    let draw = |c: &mut Canvas| {
        if i.panel.lts_lantirn {
            scopes::draw_lantirn(c, i.fonts, &i.scene.lantirn, i.t);
        } else {
            scopes::draw_tcs(c, &i.scene.tcs);
        }
    };
    if !round {
        return draw(c);
    }
    c.clear(0);
    let side = (c.w.min(c.h) as f32 * INSCRIBED) as usize;
    let mut inner = Canvas::scaled(side, side, c.scale);
    draw(&mut inner);
    let (pw, iw) = (c.pw(), inner.pw());
    let x0 = (c.pw() - iw) / 2;
    let y0 = (c.ph() - inner.ph()) / 2;
    for (row, src) in inner.px.chunks_exact(iw).enumerate() {
        let at = (y0 + row) * pw + x0;
        c.px[at..at + iw].copy_from_slice(src);
    }
}

/// The square inside a round tube (its side over the tube's diameter, a little inside it).
const INSCRIBED: f32 = 0.68;

/// The TID's picture at the TID's range and stabilisation (`tid`: the RIO's tube, else the
/// HSD's repeat).
fn tid_picture(c: &mut Canvas, i: &Inputs, tid: bool) {
    let p = i.panel;
    let cursor = i.scene.cursor;
    scopes::draw_tid(
        c,
        &TidView {
            radar: i.radar,
            controls: i.controls,
            own: i.scene.own,
            range_nm: p.tid_range.miles() as f32,
            stab: super::avionics::tid_stab(p),
            waypoints: &i.scene.waypoints,
            steer: i.scene.steer,
            in_flight: &i.scene.in_flight,
            vel_vectors: p.tid_option(TidOption::VelVector),
            alt_numbers: p.tid_option(TidOption::AltNum),
            cursor: (tid && cursor.active == Some(Cursor::Tid)).then_some(cursor.tid),
            t: i.t,
        },
    );
}

/// The threats on the ECMD, most dangerous first: the radars heard now.
fn threats(emitters: &[EmitterView]) -> Vec<Threat> {
    emitters
        .iter()
        .filter(|e| e.receiving || e.tracking || e.launch)
        .take(12)
        .enumerate()
        .map(|(rank, e)| Threat {
            rel_bearing: e.rel_bearing,
            label: e.label.clone(),
            air: e.air,
            rank,
            tracking: e.tracking,
            launch: e.launch,
        })
        .collect()
}

fn ecm_on(p: &TomcatPanel) -> bool {
    matches!(p.alq126, Alq126::Receive | Alq126::Repeat)
}

/// The pilot's vertical display indicator: the HUD's symbols repeated (`hud::vdig`), or the
/// LANTIRN's picture in TV.
pub fn draw_vdi(c: &mut Canvas, i: &Inputs) {
    if i.panel.vdi_mode == VdiMode::Tv {
        lantirn(c, i, false);
    } else if let Some(f) = i.hud {
        crate::hud::vdig::draw_vdi(c, f, crate::intruder::ink(1.0));
    } else {
        c.clear(0);
    }
    super::dump_png(c, &colours(), "vdi");
}

/// The pilot's horizontal situation display: navigation, the TID's repeat or the threats (the
/// ECM override puts them up whatever the mode).
pub fn draw_hsd(c: &mut Canvas, i: &Inputs) {
    let mode = if i.panel.hsd_ecm_override && !i.emitters.is_empty() {
        HsdMode::Ecm
    } else {
        i.panel.hsd_mode
    };
    match mode {
        HsdMode::Nav => {
            let s = &i.scene;
            let steer = s.steer.and_then(|k| s.waypoints.get(k).copied());
            let label = s.steer.map_or(String::new(), |k| format!("WP{}", k + 1));
            scopes::draw_hsd_nav(c, &s.own, steer, &label);
        }
        HsdMode::Tid => tid_picture(c, i, false),
        HsdMode::Ecm => scopes::draw_ecmd(c, &threats(i.emitters), ecm_on(i.panel), i.t),
    }
    super::dump_png(c, &colours(), "hsd");
}

/// The RIO's detail data display: the AWG-9's returns.
pub fn draw_ddd(c: &mut Canvas, i: &Inputs) {
    let cursor = i.scene.cursor;
    let ddd = (cursor.active == Some(Cursor::Ddd)).then_some(cursor.ddd);
    scopes::draw_ddd(c, i.radar, i.controls, ddd);
    super::dump_png(c, &ddd_colours(), "ddd");
}

/// The RIO's tactical information display: the tactical picture, or the LANTIRN's in TV.
pub fn draw_tid(c: &mut Canvas, i: &Inputs) {
    if i.panel.tid_mode == TidMode::Tv {
        lantirn(c, i, true);
    } else {
        tid_picture(c, i, true);
    }
    super::dump_png(c, &colours(), "tid");
}

/// The RIO's ECM display: the radars painting the aircraft.
pub fn draw_ecmd(c: &mut Canvas, i: &Inputs) {
    scopes::draw_ecmd(c, &threats(i.emitters), ecm_on(i.panel), i.t);
    super::dump_png(c, &colours(), "ecmd");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The TID follows its range knob; in TV it is see-through round the pod's symbology.
    #[test]
    fn the_tid_follows_the_panel() {
        let fonts = Fonts(Vec::new());
        let scene = TomcatScene::default();
        let (radar, controls) = (Awg9::default(), Awg9Controls::default());
        let mut p = TomcatPanel::default();
        let draw = |p: &TomcatPanel| {
            let mut c = Canvas::scaled(TID_CANVAS.0, TID_CANVAS.1, SCALE);
            let i = Inputs {
                fonts: &fonts,
                panel: p,
                scene: &scene,
                radar: &radar,
                controls: &controls,
                emitters: &[],
                t: 0.0,
                hud: None,
            };
            draw_tid(&mut c, &i);
            c.px
        };
        let before = draw(&p);
        p.press(super::super::Key::TidRange);
        assert_ne!(draw(&p), before);
        while p.tid_mode != TidMode::Tv {
            p.press(super::super::Key::TidMode);
        }
        assert_eq!(draw(&p)[0], 0);
    }
}

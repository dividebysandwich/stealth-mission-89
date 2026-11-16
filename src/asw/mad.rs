//! The pilot's MAD page (`avionics::Format::Mad`, the MH-60R's MFDs): the magnetic anomaly
//! detector's trace over the last 30 seconds (newest at the right) against its mark threshold,
//! the towed body's state and the height and speed it needs, the dipping sonar's state. Its
//! buttons: MAD (the body out or in), MARK (a fix here), DIP (the sonar down or up), the gain.

use std::collections::VecDeque;

use bevy::prelude::*;
use stealth_sim::asw::{AswCmd, DipState};

use super::AswUi;
use crate::avionics::AswOsb;
use crate::canvas::Canvas;
use crate::font::{Align, stroke_text_at};
use crate::mfd::{MfdCmd, MfdCommand};

/// Samples a second, seconds shown.
const RATE: f32 = 10.0;
const SECONDS: usize = 30;
/// The trace's full scale (nT) at each gain step, the mark threshold (nT).
const SCALES: [f32; 4] = [20.0, 10.0, 5.0, 2.5];
const THRESHOLD: f32 = 3.0;
/// Limits while the body is out: height (ft), speed (kt).
const MAX_FT: f32 = 300.0;
const MAX_KT: f32 = 120.0;

/// The trace (tenths of nT) and what the page shows besides.
#[derive(Resource, Default)]
pub struct MadTrace {
    pub samples: VecDeque<i16>,
    pub out: bool,
    pub height_ft: f32,
    pub knots: f32,
    pub dip: DipState,
    pub dip_depth: f32,
    pub marks: usize,
}

/// Samples the crew's MAD (the pilot's own aircraft, or the one he is the operator of), and
/// the MFDs' lit buttons.
pub fn sample(
    manning: Option<Res<crate::station::Manning>>,
    combat: Option<Res<crate::combat::Combat>>,
    settings: Res<crate::settings::Settings>,
    mut trace: ResMut<MadTrace>,
    mut mfd: ResMut<crate::mfd::MfdState>,
    time: Res<Time>,
    mut since: Local<f32>,
) {
    let Some(combat) = combat else { return };
    let Some((owner, _)) = super::crew_owner(manning.as_deref(), &combat, &settings) else {
        if !trace.samples.is_empty() {
            *trace = MadTrace::default();
        }
        return;
    };
    let sim = &combat.sim;
    let crew = sim.asw().crew(owner).cloned().unwrap_or_default();
    let dip = matches!(crew.dip.state, DipState::Lowering | DipState::Down);
    if mfd.av.mad_out != crew.mad_down || mfd.av.dip_down != dip {
        mfd.av.mad_out = crew.mad_down;
        mfd.av.dip_down = dip;
    }
    *since += time.delta_secs();
    if *since < 1.0 / RATE {
        return;
    }
    *since = 0.0;
    trace.samples.push_back(crew.mad);
    while trace.samples.len() > SECONDS * RATE as usize {
        trace.samples.pop_front();
    }
    trace.out = crew.mad_down;
    if let Some(v) = super::owner_view(sim, owner) {
        trace.height_ft = (v.height as f64 * super::M_PER_WU / 0.3048) as f32;
        trace.knots = v.knots as f32;
    }
    trace.dip = crew.dip.state;
    trace.dip_depth = crew.dip.depth;
    trace.marks = crew.fixes.iter().filter(|f| f.how == 2).count();
}

/// The MAD page's buttons to the simulation.
pub fn osb(
    combat: Option<Res<crate::combat::Combat>>,
    manning: Option<Res<crate::station::Manning>>,
    settings: Res<crate::settings::Settings>,
    mut cmds: MessageReader<MfdCommand>,
    mut ui: ResMut<AswUi>,
) {
    let Some(combat) = combat else {
        cmds.clear();
        return;
    };
    let Some((owner, _)) = super::crew_owner(manning.as_deref(), &combat, &settings) else {
        cmds.clear();
        return;
    };
    let crew = combat.sim.asw().crew(owner).cloned().unwrap_or_default();
    for MfdCommand(c) in cmds.read() {
        match c {
            MfdCmd::Asw(AswOsb::Mad) => ui.outbox.push(AswCmd::Mad(!crew.mad_down)),
            MfdCmd::Asw(AswOsb::Mark) => ui.outbox.push(AswCmd::Mark),
            MfdCmd::Asw(AswOsb::Dip) => {
                let down = matches!(crew.dip.state, DipState::Lowering | DipState::Down);
                ui.outbox.push(AswCmd::Dip(!down));
            }
            _ => {}
        }
    }
}

/// Draws the page on an MFD canvas (the label bands top and bottom left clear).
pub fn draw(c: &mut Canvas, t: &MadTrace, gain: u8) {
    use crate::station::scope::{BLACK, CYAN, DARK, GREEN, GREY, RED, WHITE, YELLOW};
    c.clear(BLACK);
    let (w, h) = (c.w as f32, c.h as f32);
    let (x0, x1, y0, y1) = (8.0, w - 8.0, 52.0, h - 44.0);
    let full = SCALES[(gain as usize).min(SCALES.len() - 1)];
    let text = |c: &mut Canvas, x: f32, y: f32, s: &str, col: u8, a: Align| {
        stroke_text_at(c, x, y, s, 7.0, 1.2, col, a);
    };
    let bird = if t.out { ("BIRD OUT", GREEN) } else { ("BIRD STOWED", GREY) };
    text(c, x0, 24.0, bird.0, bird.1, Align::Left);
    text(c, x1, 24.0, &format!("+/-{full} NT"), CYAN, Align::Right);
    let ft_col = if t.out && t.height_ft > MAX_FT { YELLOW } else { WHITE };
    let kt_col = if t.out && t.knots > MAX_KT { RED } else { WHITE };
    text(c, x0, 38.0, &format!("{:.0} FT", t.height_ft), ft_col, Align::Left);
    text(c, (x0 + x1) / 2.0, 38.0, &format!("{:.0} KT", t.knots), kt_col, Align::Centre);
    text(c, x1, 38.0, &format!("MARKS {}", t.marks), YELLOW, Align::Right);
    // The trace's box, its zero line and the mark threshold either way.
    c.rect(x0 as i32, y0 as i32, (x1 - x0) as i32, (y1 - y0) as i32, DARK);
    let mid = (y0 + y1) / 2.0;
    let y_of = |nt: f32| mid - (nt / full).clamp(-1.0, 1.0) * (y1 - y0) / 2.0;
    c.line(x0 as i32, mid as i32, x1 as i32, mid as i32, GREY);
    for th in [THRESHOLD, -THRESHOLD] {
        c.dashed(Vec2::new(x0, y_of(th)), Vec2::new(x1, y_of(th)), 4.0, 3.0, 1.0, RED);
    }
    for s in (0..=SECONDS).step_by(10) {
        let x = x1 - (x1 - x0) * s as f32 / SECONDS as f32;
        c.line(x as i32, y1 as i32, x as i32, y1 as i32 + 3, GREY);
        text(c, x, y1 + 9.0, &format!("-{s}"), GREY, Align::Centre);
    }
    let n = SECONDS * RATE as usize;
    let pts: Vec<Vec2> = t
        .samples
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = x1 - (x1 - x0) * (t.samples.len() - 1 - i) as f32 / (n - 1) as f32;
            Vec2::new(x, y_of(v as f32 / 10.0))
        })
        .collect();
    if pts.len() > 1 {
        let peak = t.samples.back().is_some_and(|&v| (v as f32 / 10.0).abs() > THRESHOLD);
        c.polyline(&pts, 1.3, if peak { YELLOW } else { GREEN });
    }
    let dip = match t.dip {
        DipState::Up => "DIP UP".to_string(),
        DipState::Broken => "DIP LOST".to_string(),
        DipState::Lowering => format!("DIP LOWERING {:.0} M", t.dip_depth),
        DipState::Down => format!("DIP DOWN {:.0} M", t.dip_depth),
        DipState::Raising => format!("DIP RAISING {:.0} M", t.dip_depth),
    };
    let col = if t.dip == DipState::Broken { RED } else { CYAN };
    text(c, x0, h - 24.0, &dip, col, Align::Left);
}

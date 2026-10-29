//! The console's board: the track list (the radar's picture, a track clicked there is the
//! selected one on the air picture too) or the site's status (launchers, missiles ready,
//! stockpile, refills; a Gepard's rounds) with the resupply request for a new stockpile.

use bevy::prelude::*;
use stealth_sim::Sim;
use stealth_sim::station::Guidance;

use super::scope::{BLACK, CYAN, DARK, GREEN, GREY, Kind, RED, WHITE, YELLOW};
use super::{ConsoleUi, Manning, Tab};
use crate::canvas::Canvas;
use crate::canvas::CanvasTexture;
use crate::cockpit3d::{BOARD_CANVAS, ConsoleCanvas, ConsoleScreen};
use crate::combat::Combat;
use crate::font::{Align, stroke_text_at};

/// Rows: the tab bar's height, the list's first row and row height, the status line's top.
const TABS_H: f32 = 22.0;
const LIST_TOP: f32 = 44.0;
const ROW_H: f32 = 13.0;
const STATUS_TOP: f32 = 278.0;
/// The site page's rows: first and height.
const SITE_TOP: f32 = 34.0;
const SITE_ROW: f32 = 34.0;
/// The site page's resupply button: top and height.
const RESUPPLY_TOP: f32 = 238.0;
const RESUPPLY_H: f32 = 32.0;
/// Text height and pen.
const TEXT: f32 = 6.0;
const PEN: f32 = 0.9;

/// What a click on the board hits.
pub(super) enum Clicked {
    Track(usize),
    Tab(Tab),
    Launcher(usize),
    Resupply,
}

/// The track list: the latest picture's aircraft, nearest first (missiles are not listed).
fn rows(ui: &ConsoleUi) -> Vec<&super::scope::Track> {
    let Some(snap) = ui.snapshots.front() else { return Vec::new() };
    let mut v: Vec<_> = snap.tracks.iter().filter(|t| t.kind != Kind::Missile).collect();
    v.sort_by(|a, b| a.range_km().total_cmp(&b.range_km()));
    v
}

/// Rows the list shows, and the first one (the selected track stays in view).
fn window(ui: &ConsoleUi, n: usize) -> (usize, usize) {
    let fits = ((STATUS_TOP - LIST_TOP) / ROW_H) as usize;
    let sel = rows(ui).iter().position(|t| t.unit == ui.selected).unwrap_or(0);
    let first = sel.saturating_sub(fits.saturating_sub(1)).min(n.saturating_sub(fits));
    (fits, first)
}

pub(super) fn click(ui: &ConsoleUi, sim: &Sim, launchers: &[usize], uv: Vec2) -> Option<Clicked> {
    let at = uv * Vec2::new(BOARD_CANVAS.0 as f32, BOARD_CANVAS.1 as f32);
    if at.y < TABS_H {
        return Some(Clicked::Tab(if at.x < BOARD_CANVAS.0 as f32 / 2.0 {
            Tab::Tracks
        } else {
            Tab::Site
        }));
    }
    match ui.tab {
        Tab::Tracks => {
            let rows = rows(ui);
            let (fits, first) = window(ui, rows.len());
            let k = ((at.y - LIST_TOP) / ROW_H).floor();
            if k < 0.0 || k as usize >= fits {
                return None;
            }
            rows.get(first + k as usize).and_then(|t| t.unit).map(Clicked::Track)
        }
        Tab::Site if (RESUPPLY_TOP..RESUPPLY_TOP + RESUPPLY_H).contains(&at.y) => {
            Some(Clicked::Resupply)
        }
        Tab::Site => {
            let k = ((at.y - SITE_TOP) / SITE_ROW).floor();
            let l = *launchers.get(k.max(0.0) as usize).filter(|_| k >= 0.0)?;
            (!sim.aaa_units()[l].destroyed).then_some(Clicked::Launcher(l))
        }
    }
}

pub(super) fn draw(
    manning: Option<Res<Manning>>,
    combat: Option<Res<Combat>>,
    c: Option<Res<crate::campaign::ActiveCampaign>>,
    ui: Res<ConsoleUi>,
    mut screens: Query<(&ConsoleCanvas, &mut CanvasTexture)>,
) {
    let (Some(m), Some(combat), Some(c)) = (manning, combat, c) else { return };
    let Some((_, mut tex)) = screens.iter_mut().find(|(s, _)| s.0 == ConsoleScreen::Board) else {
        return;
    };
    tex.dirty = true;
    let cv = &mut tex.canvas;
    cv.reset_clip();
    cv.clear(BLACK);
    let sim = &combat.sim;
    let w = BOARD_CANVAS.0 as f32;
    // The tabs.
    for (i, (tab, label)) in [(Tab::Tracks, "TRACKS"), (Tab::Site, "SITE")].into_iter().enumerate()
    {
        let x0 = i as f32 * w / 2.0;
        let on = ui.tab == tab;
        if on {
            cv.fill_rect(x0 as i32 + 2, 2, (w / 2.0) as i32 - 4, TABS_H as i32 - 4, DARK);
        }
        let col = if on { WHITE } else { GREY };
        stroke_text_at(cv, x0 + w / 4.0, TABS_H / 2.0, label, 8.0, 1.1, col, Align::Centre);
    }
    cv.stroke(Vec2::new(0.0, TABS_H), Vec2::new(w, TABS_H), 1.0, GREEN);
    match ui.tab {
        Tab::Tracks => tracks(cv, &ui),
        Tab::Site if m.gepard() => gepard(cv, sim, m.unit),
        Tab::Site => battery(cv, sim, &ui, m.unit),
    }
    if ui.tab == Tab::Site {
        resupply(cv, sim, c.state(), m.unit);
    }
    // The status line: mode, lock, the last refused command.
    let st = sim.station_of(c.me.0);
    cv.stroke(Vec2::new(0.0, STATUS_TOP), Vec2::new(w, STATUS_TOP), 1.0, GREEN);
    let y = STATUS_TOP + 11.0;
    if !m.gepard() {
        let (mode, col) = match st.map(|s| s.mode) {
            Some(Guidance::SemiActive) => ("SEMI-ACTIVE", YELLOW),
            _ => ("GUIDED", CYAN),
        };
        stroke_text_at(cv, 6.0, y, mode, TEXT, PEN, col, Align::Left);
    }
    let lock = st.and_then(|s| s.lock).map(|l| format!("LOCK {:04}", 1000 + l as usize));
    let (text, col) = lock.map_or(("NO LOCK".to_string(), GREY), |l| (l, WHITE));
    stroke_text_at(cv, w / 2.0, y, &text, TEXT, PEN, col, Align::Centre);
    if let Some((note, _)) = &ui.note {
        stroke_text_at(cv, w - 6.0, y, &note.to_ascii_uppercase(), 5.0, 0.8, RED, Align::Right);
    }
}

fn tracks(cv: &mut Canvas, ui: &ConsoleUi) {
    let cols: [(f32, &str); 8] = [
        (6.0, "TN"),
        (46.0, "IFF"),
        (80.0, "TYPE"),
        (176.0, "BRG"),
        (214.0, "RNG"),
        (258.0, "ALT"),
        (306.0, "SPD"),
        (350.0, "HDG"),
    ];
    let header_y = TABS_H + 11.0;
    for (x, h) in cols {
        stroke_text_at(cv, x, header_y, h, 5.0, 0.8, GREEN, Align::Left);
    }
    let rows = rows(ui);
    let (fits, first) = window(ui, rows.len());
    for (k, t) in rows.iter().skip(first).take(fits).enumerate() {
        let y = LIST_TOP + k as f32 * ROW_H;
        let selected = t.unit.is_some() && t.unit == ui.selected;
        if selected {
            cv.fill_rect(2, y as i32, BOARD_CANVAS.0 as i32 - 4, ROW_H as i32 - 1, DARK);
        }
        let col = t.side.colour();
        let mid = y + ROW_H / 2.0;
        let name: String = t.name.to_ascii_uppercase().chars().take(12).collect();
        let cells = [
            t.number(),
            t.side.label().to_string(),
            name,
            format!("{:03.0}", t.bearing()),
            format!("{:.1}", t.range_km()),
            format!("{:.0}", t.alt),
            format!("{:.0}", t.speed_kt()),
            format!("{:03.0}", t.heading()),
        ];
        for ((x, _), cell) in cols.iter().zip(&cells) {
            stroke_text_at(
                cv,
                *x,
                mid,
                cell,
                TEXT,
                PEN,
                if selected { WHITE } else { col },
                Align::Left,
            );
        }
    }
    if rows.is_empty() {
        stroke_text_at(
            cv,
            BOARD_CANVAS.0 as f32 / 2.0,
            120.0,
            "NO TRACKS",
            8.0,
            1.0,
            GREY,
            Align::Centre,
        );
    }
    if rows.len() > fits {
        let more = format!("{} MORE", rows.len() - fits);
        stroke_text_at(
            cv,
            BOARD_CANVAS.0 as f32 - 6.0,
            header_y,
            &more,
            5.0,
            0.8,
            GREY,
            Align::Right,
        );
    }
}

/// Minutes and seconds.
fn mmss(s: f32) -> String {
    let s = s.ceil() as i32;
    format!("{}:{:02}", s / 60, s % 60)
}

/// A refill's progress bar and time left.
fn refill_bar(cv: &mut Canvas, x: f32, y: f32, left: f32) {
    let w = 110.0;
    let done = 1.0 - left / stealth_sim::station::REFILL_SECONDS;
    cv.frame(Vec2::new(x, y - 4.0), Vec2::new(x + w, y + 4.0), 0.8, YELLOW);
    cv.fill_rect(x as i32 + 1, y as i32 - 3, ((w - 2.0) * done) as i32, 7, YELLOW);
    stroke_text_at(
        cv,
        x + w + 6.0,
        y,
        &format!("RELOAD {}", mmss(left)),
        TEXT,
        PEN,
        YELLOW,
        Align::Left,
    );
}

fn battery(cv: &mut Canvas, sim: &Sim, ui: &ConsoleUi, radar: usize) {
    let launchers = sim.battery_launchers(radar);
    for (i, &l) in launchers.iter().enumerate() {
        let u = sim.aaa_units()[l];
        let y = SITE_TOP + i as f32 * SITE_ROW;
        let mid = y + SITE_ROW / 2.0;
        if ui.launcher == Some(l) {
            cv.frame(
                Vec2::new(2.0, y + 2.0),
                Vec2::new(BOARD_CANVAS.0 as f32 - 2.0, y + SITE_ROW - 2.0),
                1.0,
                WHITE,
            );
        }
        stroke_text_at(cv, 8.0, mid, &format!("L{}", i + 1), 8.0, 1.1, WHITE, Align::Left);
        let (status, col) = if u.destroyed { ("OFFLINE", RED) } else { ("ONLINE", GREEN) };
        stroke_text_at(cv, 34.0, mid, status, TEXT, PEN, col, Align::Left);
        if u.destroyed {
            continue;
        }
        // The missiles ready: four boxes.
        for k in 0..stealth_sim::campaign::PATRIOT_ROUNDS {
            let x = 96.0 + k as f32 * 12.0;
            let a = Vec2::new(x, mid - 4.0);
            if k < u.rounds {
                cv.fill_rect(a.x as i32, a.y as i32, 8, 8, CYAN);
            } else {
                cv.frame(a, a + Vec2::splat(8.0), 0.8, GREY);
            }
        }
        let left = sim.refill_left(l);
        if left > 0.0 {
            refill_bar(cv, 154.0, mid, left);
        } else if u.rounds == 0 {
            stroke_text_at(cv, 154.0, mid, "EMPTY - RELOAD", TEXT, PEN, RED, Align::Left);
        } else if u.cooldown > 0 {
            stroke_text_at(cv, 154.0, mid, "LAUNCHED", TEXT, PEN, GREY, Align::Left);
        } else {
            stroke_text_at(cv, 154.0, mid, "READY", TEXT, PEN, GREEN, Align::Left);
        }
    }
    let y = SITE_TOP + 3.0 * SITE_ROW + 16.0;
    let (status, col) =
        if sim.aaa_units()[radar].destroyed { ("OFFLINE", RED) } else { ("ONLINE", GREEN) };
    stroke_text_at(cv, 8.0, y, "RADAR", TEXT, PEN, WHITE, Align::Left);
    stroke_text_at(cv, 96.0, y, status, TEXT, PEN, col, Align::Left);
    let stock = launchers.first().map_or(0, |&l| sim.stockpile(l));
    stroke_text_at(cv, 8.0, y + 18.0, "STOCKPILE", TEXT, PEN, WHITE, Align::Left);
    stroke_text_at(
        cv,
        96.0,
        y + 18.0,
        &format!("{stock} MSL"),
        TEXT,
        PEN,
        if stock > 0 { CYAN } else { RED },
        Align::Left,
    );
    // The missiles in the air, by guidance.
    let ours = sim.ai_missiles().iter().filter(|m| m.station == Some(radar as u16));
    let (mut guided, mut semi) = (0, 0);
    for m in ours {
        match m.guidance {
            Some(Guidance::Guided) => guided += 1,
            Some(Guidance::SemiActive) => semi += 1,
            None => {}
        }
    }
    stroke_text_at(cv, 8.0, y + 36.0, "IN FLIGHT", TEXT, PEN, WHITE, Align::Left);
    stroke_text_at(
        cv,
        96.0,
        y + 36.0,
        &format!("{guided} GUIDED  {semi} SEMI"),
        TEXT,
        PEN,
        CYAN,
        Align::Left,
    );
}

/// The resupply button, or how the request stands: on the board (the AI's helicopter takes it
/// after a while), or a helicopter on its way.
fn resupply(cv: &mut Canvas, sim: &Sim, st: &stealth_campaign::CampaignState, unit: usize) {
    use stealth_campaign::{Objective, SupplyState};
    let w = BOARD_CANVAS.0 as f32;
    let request = st.supply_requests.iter().find(|r| r.aaa as usize == unit);
    let flown_by = |id: u32| {
        st.players
            .iter()
            .find(|p| {
                p.sortie
                    .as_ref()
                    .is_some_and(|s| s.offer.objective == Objective::Resupply { request: id })
            })
            .map(|p| p.name.to_ascii_uppercase())
    };
    let full = sim
        .aaa_units()
        .get(unit)
        .is_some_and(|u| sim.stockpile(unit) >= stealth_sim::station::initial_stock(u.kind));
    let (text, col, button) = match request.map(|r| (r.id, r.state)) {
        None if full => ("STOCKPILE FULL".to_string(), GREY, true),
        None => ("REQUEST RESUPPLY".to_string(), WHITE, true),
        Some((id, SupplyState::Open { since })) => match flown_by(id) {
            Some(name) => (format!("RESUPPLY: {name} ON THE WAY"), CYAN, false),
            None => {
                let wait = stealth_campaign::tuning::SUPPLY_AI_WAIT;
                let left = (wait - (st.clock - since)).max(0.0) as f32;
                (format!("RESUPPLY ON THE BOARD - AI IN {}", mmss(left)), YELLOW, false)
            }
        },
        Some((_, SupplyState::Ai { .. })) => ("RESUPPLY: AI UH-60 ON THE WAY".into(), CYAN, false),
    };
    let (a, b) = (Vec2::new(6.0, RESUPPLY_TOP), Vec2::new(w - 6.0, RESUPPLY_TOP + RESUPPLY_H));
    if button {
        cv.frame(a, b, 1.0, col);
    }
    stroke_text_at(
        cv,
        w / 2.0,
        RESUPPLY_TOP + RESUPPLY_H / 2.0,
        &text,
        7.0,
        1.0,
        col,
        Align::Centre,
    );
}

fn gepard(cv: &mut Canvas, sim: &Sim, unit: usize) {
    let u = sim.aaa_units()[unit];
    let y = SITE_TOP + 16.0;
    let (status, col) = if u.destroyed { ("OFFLINE", RED) } else { ("ONLINE", GREEN) };
    stroke_text_at(cv, 8.0, y, "GEPARD", 8.0, 1.1, WHITE, Align::Left);
    stroke_text_at(cv, 96.0, y, status, TEXT, PEN, col, Align::Left);
    stroke_text_at(cv, 8.0, y + 22.0, "ROUNDS", TEXT, PEN, WHITE, Align::Left);
    let full = stealth_sim::station::GEPARD_ROUNDS;
    stroke_text_at(
        cv,
        96.0,
        y + 22.0,
        &format!("{} / {full}", u.rounds),
        TEXT,
        PEN,
        CYAN,
        Align::Left,
    );
    let bar = (u.rounds as f32 / full as f32).clamp(0.0, 1.0);
    cv.frame(Vec2::new(180.0, y + 18.0), Vec2::new(380.0, y + 26.0), 0.8, GREY);
    cv.fill_rect(181, (y + 19.0) as i32, (198.0 * bar) as i32, 7, CYAN);
    let bursts = u.rounds / stealth_sim::station::BURST;
    stroke_text_at(cv, 96.0, y + 40.0, &format!("{bursts} BURSTS"), TEXT, PEN, GREY, Align::Left);
    let left = sim.refill_left(unit);
    if left > 0.0 {
        refill_bar(cv, 96.0, y + 62.0, left);
    } else if u.rounds == 0 {
        stroke_text_at(cv, 96.0, y + 62.0, "EMPTY - RELOAD", TEXT, PEN, RED, Align::Left);
    } else if u.burst > 0 {
        stroke_text_at(cv, 96.0, y + 62.0, "FIRING", TEXT, PEN, YELLOW, Align::Left);
    }
    let stock = sim.stockpile(unit);
    stroke_text_at(cv, 8.0, y + 84.0, "STOCKPILE", TEXT, PEN, WHITE, Align::Left);
    stroke_text_at(
        cv,
        96.0,
        y + 84.0,
        &format!("{stock} RDS"),
        TEXT,
        PEN,
        if stock > 0 { CYAN } else { RED },
        Align::Left,
    );
    let reach = stealth_sim::station::GEPARD_REACH / 1000.0;
    stroke_text_at(
        cv,
        8.0,
        y + 106.0,
        &format!("AIRBURST, {reach:.0} KM"),
        TEXT,
        PEN,
        GREY,
        Align::Left,
    );
}

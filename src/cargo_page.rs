//! The C-5's cargo page (`avionics::Format::Cgo`): the hold and the aft ramp seen from the left
//! side, the pallets on the hold's floor (empty places dotted), the ramp at its angle (green up,
//! amber moving, cyan down for an airdrop). A pallet released goes aft along the floor and off
//! the ramp's end, then falls under its chute. Under the picture the ramp's state, the pallets
//! left, a campaign cargo run's load and where it goes, and on an airdrop the drop zone's range,
//! the time to the release point, the drift off the zone's track and the jump light.

use bevy::math::Vec2;

use crate::canvas::Canvas;
use crate::font::stroke_text;
use crate::hud::DropCue;

const BLACK: u8 = 5;
const DARK: u8 = 8;
const GREY: u8 = 7;
const GREEN: u8 = 10;
const CYAN: u8 = 11;
const RED: u8 = 13;
const WHITE: u8 = 15;
const AMBER: u8 = 18;

/// The picture (canvas units of the 231 x 180 page): the hold's floor from `FLOOR_X0` to the
/// ramp's hinge at `HINGE`, the fuselage's top and belly, the tail cone up to `TAIL`.
const FLOOR_X0: f32 = 18.0;
const HINGE: Vec2 = Vec2::new(150.0, 96.0);
const TOP_Y: f32 = 42.0;
const BELLY_Y: f32 = 104.0;
const TAIL: Vec2 = Vec2::new(222.0, 64.0);
/// The ramp's length and its angles up from level (degrees): shut along the belly, down for an
/// airdrop a little below level.
const RAMP_LEN: f32 = 52.0;
const RAMP_UP_DEG: f32 = 30.0;
const RAMP_DOWN_DEG: f32 = -3.0;
/// A pallet's size, the gap between them.
const PALLET: Vec2 = Vec2::new(24.0, 15.0);
const PALLET_GAP: f32 = 5.0;
/// The release's animation (s): along the floor and off the ramp, then the fall.
const SLIDE_SECS: f32 = 1.4;
const FALL_SECS: f32 = 2.6;
/// The falling pallet goes out of the picture here (the text rows under it).
const FALL_FLOOR: f32 = 116.0;

fn v2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn text(c: &mut Canvas, x: f32, y: f32, s: &str, col: u8) {
    stroke_text(c, x, y, s, 6.5, 1.0, col);
}

/// A filled box with its outline.
fn block(c: &mut Canvas, a: Vec2, b: Vec2, fill: u8, edge: u8) {
    c.triangle([a, v2(b.x, a.y), b], fill);
    c.triangle([a, b, v2(a.x, b.y)], fill);
    c.frame(a, b, 1.0, edge);
}

/// The ramp's end at `open` (0 shut .. 1 down).
fn ramp_end(open: f32) -> Vec2 {
    let deg = RAMP_UP_DEG + (RAMP_DOWN_DEG - RAMP_UP_DEG) * open.clamp(0.0, 1.0);
    let a = deg.to_radians();
    HINGE + v2(a.cos(), -a.sin()) * RAMP_LEN
}

/// Where pallet place `k` (0 the foremost) sits on the floor: its lower left corner.
fn place(k: u32, capacity: u32) -> Vec2 {
    let aft = HINGE.x - 4.0 - PALLET.x;
    let x = aft - (capacity.saturating_sub(1) - k) as f32 * (PALLET.x + PALLET_GAP);
    v2(x, HINGE.y)
}

/// Draws the cargo page (None: not the cargo jet, an empty hold).
pub fn draw(c: &mut Canvas, cue: Option<&DropCue>, blink: bool) {
    c.clear(BLACK);
    let empty = DropCue { capacity: 4, ..Default::default() };
    let d = cue.unwrap_or(&empty);
    text(c, 96.0, 28.0, "CARGO", CYAN);
    // The fuselage from the side: its top and belly, the tail cone rising to its end.
    let fus = GREY;
    c.stroke(v2(FLOOR_X0 - 10.0, TOP_Y), v2(178.0, TOP_Y), 1.0, fus);
    c.stroke(v2(178.0, TOP_Y), TAIL, 1.0, fus);
    c.stroke(v2(FLOOR_X0 - 10.0, BELLY_Y), v2(HINGE.x - 6.0, BELLY_Y), 1.0, fus);
    c.stroke(v2(FLOOR_X0 - 10.0, TOP_Y), v2(FLOOR_X0 - 10.0, BELLY_Y), 1.0, DARK);
    // The tail's underside, aft of the opening (the shut ramp closes the outline).
    let shut = ramp_end(0.0);
    c.stroke(shut, TAIL, 1.0, fus);
    // The hold's floor up to the ramp's hinge.
    c.stroke(v2(FLOOR_X0, HINGE.y), HINGE, 1.4, WHITE);
    // The ramp, at its angle; its shut outline dotted while it is open.
    let open = d.ramp;
    let col = if open >= 1.0 {
        CYAN
    } else if open > 0.0 {
        AMBER
    } else {
        GREEN
    };
    if open > 0.0 {
        c.dashed(HINGE, shut, 3.0, 3.0, 0.8, DARK);
    }
    let end = ramp_end(open);
    c.stroke(HINGE, end, 2.4, col);
    c.ring(HINGE, 2.0, 1.0, col);
    // The pallets: the hold's places from the front, the last released going over the ramp.
    let cap = d.capacity.max(d.pallets).max(1);
    for k in 0..cap {
        let p = place(k, cap);
        let (a, b) = (v2(p.x, p.y - PALLET.y), v2(p.x + PALLET.x, p.y));
        if k < d.pallets {
            block(c, a, b, DARK, GREEN);
            c.stroke(v2(a.x + 3.0, a.y + 5.0), v2(b.x - 3.0, a.y + 5.0), 0.7, GREEN);
        } else {
            c.dashed(a, v2(b.x, a.y), 2.0, 2.0, 0.6, DARK);
            c.dashed(v2(b.x, a.y), b, 2.0, 2.0, 0.6, DARK);
            c.dashed(a, v2(a.x, b.y), 2.0, 2.0, 0.6, DARK);
        }
    }
    if let Some(t) = d.since_release.filter(|&t| t < SLIDE_SECS + FALL_SECS) {
        // The pallet that went: from its place along the floor and down the ramp, then falling
        // aft and down, the chute opening over it.
        let from = place(d.pallets.min(cap - 1), cap);
        let off = end;
        let p = if t < SLIDE_SECS {
            let u = (t / SLIDE_SECS).powi(2);
            let floor = HINGE - v2(PALLET.x, 0.0);
            if from.x + (off.x - from.x) * u < floor.x {
                from + v2((off.x - from.x) * u, 0.0)
            } else {
                let k = ((from.x + (off.x - from.x) * u) - floor.x) / (off.x - floor.x);
                floor + (off - PALLET.x * Vec2::X - floor) * k.clamp(0.0, 1.0)
            }
        } else {
            // Falling aft, the chute slowing it.
            let f = t - SLIDE_SECS;
            off - v2(PALLET.x, 0.0) + v2(18.0 * f, 4.0 * f + 1.2 * f * f)
        };
        let (a, b) = (v2(p.x, p.y - PALLET.y), v2(p.x + PALLET.x, p.y));
        if b.y < FALL_FLOOR {
            block(c, a, b, DARK, WHITE);
            if t > SLIDE_SECS + 0.3 {
                let top = v2((a.x + b.x) / 2.0, a.y - 14.0);
                c.arc(top, 12.0, std::f32::consts::PI, std::f32::consts::TAU, 1.0, WHITE);
                for x in [-12.0, 0.0, 12.0] {
                    c.stroke(top + v2(x, 0.0), v2((a.x + b.x) / 2.0, a.y), 0.6, WHITE);
                }
            }
        }
    }
    // The ramp's state and the hold.
    let state = if open >= 1.0 {
        "RAMP DOWN"
    } else if open > 0.0 && d.lowering {
        "RAMP LOWERING"
    } else if open > 0.0 {
        "RAMP RAISING"
    } else {
        "RAMP UP"
    };
    let y0 = 118.0;
    text(c, 6.0, y0, state, col);
    if open > 0.0 && open < 1.0 {
        text(c, 112.0, y0, &format!("{:.0}%", open * 100.0), col);
    }
    let pallets = format!("PLT {}/{}", d.pallets, d.capacity.max(d.pallets));
    text(c, 150.0, y0, &pallets, if d.pallets > 0 { GREEN } else { GREY });
    // A campaign cargo run: its load, where it goes.
    if !d.manifest.is_empty() {
        let to = d.destination.as_deref().unwrap_or("");
        text(c, 6.0, y0 + 11.0, &format!("FOR {}", to.to_uppercase()), CYAN);
        let load = d.manifest.join(", ").to_uppercase();
        text(c, 6.0, y0 + 22.0, &load.chars().take(36).collect::<String>(), GREEN);
        return;
    }
    // The airdrop: the drop zone, the release, the drift, the jump light.
    let Some(km) = d.zone_km else {
        if d.pallets > 0 {
            text(c, 6.0, y0 + 11.0, "NO DROP ZONE", GREY);
        }
        return;
    };
    text(c, 6.0, y0 + 11.0, &format!("DZ {km:.1} KM"), WHITE);
    if let Some(s) = d.release_secs {
        let rel = if s < 0.0 {
            "REL PAST".to_string()
        } else {
            let s = s.round() as i32;
            format!("REL {}:{:02}", s / 60, s % 60)
        };
        text(c, 80.0, y0 + 11.0, &rel, WHITE);
    }
    if let Some(x) = d.cross_m {
        let side = if x > 0.0 { "R" } else { "L" };
        text(c, 150.0, y0 + 11.0, &format!("DFT {:.0}{side}", x.abs()), WHITE);
    }
    // The jump light: green to drop, red otherwise (amber while the ramp moves).
    let (light, lc) = if d.green() {
        ("GREEN LIGHT", GREEN)
    } else if open > 0.0 && open < 1.0 {
        ("RAMP MOVING", AMBER)
    } else {
        ("RED LIGHT", RED)
    };
    let (a, b) = (v2(4.0, y0 + 21.0), v2(80.0, y0 + 33.0));
    if d.green() && blink {
        block(c, a, b, lc, lc);
        text(c, a.x + 3.0, a.y + 3.0, light, BLACK);
    } else {
        c.frame(a, b, 1.0, lc);
        text(c, a.x + 3.0, a.y + 3.0, light, lc);
    }
    text(c, 88.0, y0 + 24.0, &format!("TOF {:.0}", d.fall_secs), GREY);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(d: &DropCue) -> Canvas {
        let mut c = Canvas::scaled(crate::mfd::HIRES.0, crate::mfd::HIRES.1, 2);
        draw(&mut c, Some(d), true);
        c
    }

    /// The page shows the ramp's state in its colours, the pallets, the jump light; a released
    /// pallet goes over the ramp and falls (the page changes as it goes).
    #[test]
    fn the_cargo_page_shows_the_ramp_the_pallets_and_the_drop() {
        let up = DropCue { capacity: 4, pallets: 4, ..Default::default() };
        let c = page(&up);
        assert!(c.px.contains(&GREEN));
        let down = DropCue { ramp: 1.0, zone_km: Some(4.0), in_zone: true, ..up.clone() };
        assert!(down.green());
        let c = page(&down);
        assert!(c.px.contains(&CYAN) && c.px.contains(&GREEN));
        let moving = DropCue { ramp: 0.5, lowering: true, ..up.clone() };
        assert!(page(&moving).px.contains(&AMBER));
        let red = DropCue { ramp: 1.0, zone_km: Some(9.0), ..up.clone() };
        assert!(!red.green() && page(&red).px.contains(&RED));
        // A pallet going: sliding, then under its chute; gone after the animation.
        let gone = DropCue { pallets: 3, ramp: 1.0, ..up.clone() };
        let frames: Vec<Vec<u8>> = [0.3, 1.2, 2.5, 9.0]
            .into_iter()
            .map(|t| page(&DropCue { since_release: Some(t), ..gone.clone() }).px)
            .collect();
        assert_ne!(frames[0], frames[1]);
        assert_ne!(frames[1], frames[2]);
        assert_eq!(frames[3], page(&gone).px);
        if let Ok(dir) = std::env::var("SM89_ENG_PNG") {
            for (k, d) in [up, down, moving, red].iter().enumerate() {
                dump(&page(d), &format!("{dir}/cargo_{k}.png"));
            }
            for t in [0.3, 1.0, 1.6, 2.5] {
                let c = page(&DropCue { since_release: Some(t), ..gone.clone() });
                dump(&c, &format!("{dir}/cargo_drop_{t}.png"));
            }
        }
    }

    fn dump(c: &Canvas, path: &str) {
        let rgb = |i: u8| match i {
            GREEN => [40, 220, 60],
            CYAN => [60, 220, 230],
            RED => [230, 40, 40],
            WHITE => [240, 240, 240],
            AMBER => [240, 170, 30],
            GREY => [150, 150, 150],
            DARK => [70, 70, 70],
            _ => [0, 0, 0],
        };
        let px: Vec<u8> = c.px.iter().flat_map(|&i| rgb(i)).collect();
        image::RgbImage::from_raw(c.pw() as u32, c.ph() as u32, px).unwrap().save(path).unwrap();
    }
}

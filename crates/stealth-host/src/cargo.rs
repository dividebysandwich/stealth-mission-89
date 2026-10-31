//! Where the cargo crates stand: in a row beside the runway of their base, where the pilots
//! and the AI helicopters hook them (the host to send its helicopters there, every client to
//! draw them in the same place).

use stealth_campaign::{CampaignState, CrateState, crate_slot};
use stealth_sim::Sim;

use crate::host::formation;

/// The crates line up beside the runway of an airfield this close to where its aircraft start,
/// this far ahead of them and this far out from the runway's edge (metres).
const ROW_RANGE: f64 = 4000.0;
const ROW_AHEAD: f64 = 150.0;
const ROW_OUT: f64 = 35.0;
/// The crates of a row stand this far apart (metres): well clear of each other's hook reach,
/// so hovering over one never picks its neighbour.
pub const ROW_SPACING: f64 = 30.0;
/// The resupply crates stand in a second row this far behind the first (metres).
const SUPPLY_ROW_BACK: f64 = 25.0;
/// Metres per world unit.
const WU: f64 = 512_000.0 / 1_048_576.0;

/// Metres east, north.
type P = (f64, f64);

fn add(a: P, b: P) -> P {
    (a.0 + b.0, a.1 + b.1)
}

fn mul(a: P, k: f64) -> P {
    (a.0 * k, a.1 * k)
}

fn dot(a: P, b: P) -> f64 {
    a.0 * b.0 + a.1 * b.1
}

fn dist(a: P, b: P) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// The crate row of base `base`: its middle, its direction and the side away from the runway
/// (metres east, north; unit vectors). Beside the runway a little ahead of where its aircraft
/// start, on the far side from the apron.
fn row(sim: &Sim, base: usize) -> (P, P, P) {
    // Where the lead of the take-off formation stands, facing along the runway.
    let st = sim.start_state_at(base);
    let dir = if st.heading == 0 { 1 } else { -1 };
    let start = (st.x as f64 * WU, (st.y + formation(0).0 * dir) as f64 * WU);
    let h = st.heading as f64 / 65536.0 * std::f64::consts::TAU;
    let forward = (h.sin(), h.cos());
    let left = (-forward.1, forward.0);
    let near = sim
        .runways()
        .iter()
        .filter(|r| dist(r.c, start) < ROW_RANGE)
        .min_by(|a, b| dist(a.c, start).total_cmp(&dist(b.c, start)));
    let Some(r) = near else {
        return (add(add(start, mul(forward, ROW_AHEAD)), mul(left, 80.0)), forward, left);
    };
    let ahead = dot(add(start, mul(forward, ROW_AHEAD)), r.axis) - dot(r.c, r.axis);
    let along = ahead.clamp(-r.half_len * 0.8, r.half_len * 0.8);
    let c = add(add(r.c, mul(r.axis, along)), mul(r.out, r.half_width + ROW_OUT));
    (c, r.axis, r.out)
}

/// Slot `k` of `n` in the crate row of base `base`: where the crate stands (metres east,
/// north) and its heading (radians clockwise from north).
pub fn slot(sim: &Sim, base: usize, k: usize, n: usize) -> (P, f32) {
    let (c, axis, _) = row(sim, base);
    let p = add(c, mul(axis, (k as f64 - (n as f64 - 1.0) * 0.5) * ROW_SPACING));
    (p, axis.0.atan2(axis.1) as f32)
}

/// Where load `leg` of new site `id` waits for its helicopter (metres east, north) and its
/// heading: its place in the row of its base, among the loads of every site supplied from there.
pub fn crate_home(sim: &Sim, state: &CampaignState, id: u32, leg: u8) -> Option<(P, f32)> {
    let base = state.deployment(id)?.base?;
    let row: Vec<(u32, u8)> = state
        .deployments
        .iter()
        .filter(|d| !d.built && d.base == Some(base))
        .flat_map(|d| (0..d.loads).map(|l| (d.id, l)))
        .collect();
    let k = row.iter().position(|r| *r == (id, leg))?;
    Some(slot(sim, base as usize, k, row.len()))
}

/// Where the crate of resupply helicopter `tag` from base `base` waits (metres east, north)
/// and its heading: in a short row behind the cargo's.
pub fn supply_crate(sim: &Sim, base: usize, tag: u32) -> (P, f32) {
    let (c, axis, out) = row(sim, base);
    let k = (tag % 4) as f64 - 1.5;
    let p = add(add(c, mul(out, SUPPLY_ROW_BACK)), mul(axis, k * ROW_SPACING));
    (p, axis.0.atan2(axis.1) as f32)
}

/// Where AI helicopter `tag` hooks its load for new site `id` with its helipad at map point
/// (`x`, `y` south), and where it sets it down (world units, x east, y south): its crate in
/// the row of the site's base (None: none there), and the load's place by the helipad.
pub fn cargo_run(
    sim: &Sim,
    state: &CampaignState,
    id: u32,
    tag: u32,
    x: i32,
    y: i32,
) -> (Option<(i32, i32)>, (i32, i32)) {
    let leg = state.deployment(id).and_then(|d| d.ai.iter().find(|a| a.1 == tag)).map(|a| a.0);
    let pickup = leg.and_then(|l| crate_home(sim, state, id, l)).map(|(p, _)| world_units(p));
    let pad = metres(((x << 5) + 16, (y << 5) + 16));
    let drop = match leg.map(crate_slot) {
        Some(CrateState::Delivered { east, north, .. }) => add(pad, (east as f64, north as f64)),
        _ => pad,
    };
    (pickup, world_units(drop))
}

/// Metres east, north to the simulation's world units (x east, y south).
pub fn world_units(p: P) -> (i32, i32) {
    ((p.0 / WU).round() as i32, 0x10_0000 - (p.1 / WU).round() as i32)
}

/// The simulation's world units (x east, y south) to metres east, north.
pub fn metres(w: (i32, i32)) -> P {
    (w.0 as f64 * WU, (0x10_0000 - w.1) as f64 * WU)
}

//! Paratroopers, simplified for the game: the C-5 (two sticks, over the ramp) and the CH-47
//! (one stick, out of its rear) drop them over a drop zone.
//!
//! A stick goes with the fire button, the paratroopers' load selected, within the jump's limits
//! (height over the ground, speed, the C-5's ramp down): one man after another out of the
//! aircraft, each falling a moment until his static line pulls his chute out, then drifting
//! down under it, slowing to the canopy's sink and losing the aircraft's speed. A man landing in
//! the water drowns. When the last of a stick is down, those inside the drop zone (the mission's
//! primary target) count: most of them there is the objective.

use serde::{Deserialize, Serialize};
use stealth_i18n::text;

use crate::Sim;
use crate::aaa::M_PER_MAP;
use crate::types::*;

/// The paratroopers' weapon type (a stick a round) and the mission type of a drop zone in the
/// primary slot (after the raid's 13).
pub const PARA_WEAPON: i32 = 28;
pub const PARA_MISSION: i32 = 14;
/// Men in a stick: the C-5's (two doors, two sticks a load), the CH-47's.
pub const STICK_C5: u8 = 40;
pub const STICK_CH47: u8 = 30;
/// Seconds between men out of the door: the C-5's two doors, the CH-47's ramp.
const EXIT_C5: f64 = 0.25;
const EXIT_CH47: f64 = 0.5;
/// The jump's limits: height over the ground (ft) and speed (kt).
pub const JUMP_FT: (i32, i32) = (500, 1500);
pub const JUMP_KT: i32 = 150;
/// Seconds of fall before the chute opens, the canopy's sink (m/s) and how fast a man loses
/// the aircraft's speed under it (1/s).
pub const STATIC_LINE: f64 = 2.0;
const SINK: f64 = 5.5;
const CANOPY_DRAG: f64 = 0.35;
const GRAVITY: f64 = 9.81;
/// The drop zone: its radius (m), and the share of a stick (percent) landed in it that takes it.
pub const DZ_RADIUS: f64 = 900.0;
const DZ_SHARE: usize = 60;
/// Seconds a man stays on the ground (drawn) after landing.
const ON_GROUND_SECS: f64 = 120.0;

const M_PER_WU: f64 = M_PER_MAP / 32.0;
const FT: f64 = 0.3048;
const KT: f64 = 0.514_444;

/// A paratrooper: his pilot's (whose stick), the stick, where he is (m east, north, up), his
/// velocity (m/s), seconds since he jumped, landed (seconds on the ground), alive.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Jumper {
    pub owner: u8,
    pub stick: u16,
    pub p: [f64; 3],
    pub v: [f64; 3],
    pub t: f32,
    pub landed: Option<f32>,
    pub alive: bool,
}

impl Jumper {
    /// His chute is out.
    pub fn open(&self) -> bool {
        self.t as f64 >= STATIC_LINE
    }
}

/// A stick on its way out of the door: its pilot, its number, men still in the aircraft and
/// seconds to the next.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Stick {
    pub owner: u8,
    pub id: u16,
    pub left: u8,
    pub men: u8,
    pub next: f64,
    pub judged: bool,
}

/// The paratroopers in the air and on the ground.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Paradrop {
    pub jumpers: Vec<Jumper>,
    pub sticks: Vec<Stick>,
    pub next_id: u16,
}

impl Paradrop {
    pub fn is_empty(&self) -> bool {
        self.jumpers.is_empty() && self.sticks.is_empty()
    }
}

impl Sim {
    /// The aircraft (the active pilot's) in metres east, north, up and its velocity (m/s).
    fn para_motion(&self) -> ([f64; 3], [f64; 3]) {
        let p = &self.pl.p;
        let h = (p.heading as u16) as f64 / 65536.0 * std::f64::consts::TAU;
        let v = p.knots as f64 * KT;
        (
            [p.wx as f64 * M_PER_WU, p.wy as f64 * M_PER_WU, p.alt as f64 * FT],
            [h.sin() * v, h.cos() * v, 0.0],
        )
    }

    /// The ground (m) at a point (m east, north).
    fn para_ground(&self, at: [f64; 2]) -> f64 {
        let (x, y) = (at[0] / M_PER_WU, 0x10_0000 as f64 - at[1] / M_PER_WU);
        self.slam_ground(x, y) * FT
    }

    /// The fire button with the paratroopers selected: a stick goes, within the jump's limits.
    pub(crate) fn release_stick(&mut self) {
        let (w, count) = self.selected_weapon();
        if count <= 0 {
            return;
        }
        let agl = self.height_above_ground();
        let fast = self.pl.p.knots > JUMP_KT;
        if !(JUMP_FT.0..=JUMP_FT.1).contains(&agl) || fast {
            self.sound(0x20, 1);
            self.msg(text!("sim-para-limits", low = JUMP_FT.0, high = JUMP_FT.1, kt = JUMP_KT));
            return;
        }
        if self.pl.galaxy && self.pl.pflags & pflags::BAY_OPEN == 0 {
            self.sound(0x20, 1);
            self.msg(text!("sim-ramp-closed"));
            return;
        }
        if self.paradrop.sticks.iter().any(|s| s.owner == self.pl.id && s.left > 0) {
            return;
        }
        let bay = self.firing_station(w);
        if let Some(b) = self.pl.bays.get_mut(bay) {
            b.1 -= 1;
        }
        let men = if self.pl.galaxy { STICK_C5 } else { STICK_CH47 };
        let id = self.paradrop.next_id;
        self.paradrop.next_id = id.wrapping_add(1);
        self.paradrop.sticks.push(Stick {
            owner: self.pl.id,
            id,
            left: men,
            men,
            next: 0.0,
            judged: false,
        });
        self.msg(text!("sim-para-go"));
    }

    /// One frame of the paratroopers: men out of the doors, falling, drifting down, landing;
    /// a stick all down is judged against the drop zone (the host's).
    pub(crate) fn update_paradrop(&mut self, present: &[usize]) {
        if self.paradrop.is_empty() || self.mirror {
            return;
        }
        let dt = 1.0 / self.fs.max(1) as f64;
        // Out of the doors, from each stick's aircraft.
        let back = self.pl.id as usize;
        for k in 0..self.paradrop.sticks.len() {
            let s = self.paradrop.sticks[k];
            if s.left == 0 {
                continue;
            }
            let owner = s.owner as usize;
            if !present.contains(&owner) {
                // Its aircraft gone: the rest stay aboard.
                self.paradrop.sticks[k].left = 0;
                self.paradrop.sticks[k].men -= s.left;
                continue;
            }
            self.activate(owner);
            let gap = if self.pl.galaxy { EXIT_C5 } else { EXIT_CH47 };
            let (p, v) = self.para_motion();
            let st = &mut self.paradrop.sticks[k];
            st.next -= dt;
            while st.next <= 0.0 && st.left > 0 {
                st.next += gap;
                st.left -= 1;
                // Out of the back, a few metres under the aircraft.
                let back_m = 15.0;
                let h = v[0].atan2(v[1]);
                let at = [p[0] - h.sin() * back_m, p[1] - h.cos() * back_m, p[2] - 4.0];
                self.paradrop.jumpers.push(Jumper {
                    owner: s.owner,
                    stick: s.id,
                    p: at,
                    v,
                    t: 0.0,
                    landed: None,
                    alive: true,
                });
            }
        }
        self.activate(back);
        // Falling and drifting down.
        let water = self.water.clone();
        for i in 0..self.paradrop.jumpers.len() {
            let j = self.paradrop.jumpers[i];
            if let Some(t) = j.landed {
                self.paradrop.jumpers[i].landed = Some(t + dt as f32);
                continue;
            }
            let mut j = j;
            j.t += dt as f32;
            if j.open() {
                let k = (1.0 - CANOPY_DRAG * dt).max(0.0);
                j.v = [j.v[0] * k, j.v[1] * k, j.v[2] + (-SINK - j.v[2]) * (2.0 * dt).min(1.0)];
            } else {
                j.v[2] -= GRAVITY * dt;
                j.v[0] *= 1.0 - 0.15 * dt;
                j.v[1] *= 1.0 - 0.15 * dt;
            }
            for a in 0..3 {
                j.p[a] += j.v[a] * dt;
            }
            let ground = self.para_ground([j.p[0], j.p[1]]);
            if j.p[2] <= ground {
                j.p[2] = ground;
                j.landed = Some(0.0);
                let (mx, my) =
                    ((j.p[0] / M_PER_MAP) as i32, (0x8000 as f64 - j.p[1] / M_PER_MAP) as i32);
                j.alive = !water.known() || !water.wet(mx, my);
            }
            self.paradrop.jumpers[i] = j;
        }
        // A stick all down: judged against its pilot's drop zone.
        for k in 0..self.paradrop.sticks.len() {
            let s = self.paradrop.sticks[k];
            if s.judged || s.left > 0 {
                continue;
            }
            let men: Vec<Jumper> =
                self.paradrop.jumpers.iter().filter(|j| j.stick == s.id).copied().collect();
            if men.iter().any(|j| j.landed.is_none()) {
                continue;
            }
            self.paradrop.sticks[k].judged = true;
            self.judge_stick(s.owner, &men);
        }
        // The landed men leave the picture after a while; judged sticks with nobody left go.
        self.paradrop.jumpers.retain(|j| j.landed.is_none_or(|t| (t as f64) < ON_GROUND_SECS));
        let jumpers = &self.paradrop.jumpers;
        self.paradrop.sticks.retain(|s| !s.judged || jumpers.iter().any(|j| j.stick == s.id));
    }

    /// The stick of pilot `owner` is down: how many landed alive in his drop zone, and whether
    /// that takes it.
    fn judge_stick(&mut self, owner: u8, men: &[Jumper]) {
        if !self.pilot_exists(owner as usize) {
            return;
        }
        let back = self.pl.id as usize;
        self.activate(owner as usize);
        let t = self.pl.targets[0];
        let dz = (t.mission_type == PARA_MISSION)
            .then(|| self.objects.get(usize::try_from(t.object).ok()?).copied())
            .flatten()
            .map(|o| crate::asw::object_metres(&o));
        let inside = men
            .iter()
            .filter(|j| {
                j.alive && dz.is_some_and(|d| (j.p[0] - d[0]).hypot(j.p[1] - d[1]) < DZ_RADIUS)
            })
            .count();
        let lost = men.iter().filter(|j| !j.alive).count();
        self.msg(text!(
            "sim-para-down",
            inside = inside as i32,
            men = men.len() as i32,
            lost = lost as i32
        ));
        if dz.is_some() && inside * 100 >= men.len() * DZ_SHARE {
            self.mark_target_reached(0);
        }
        self.activate(back);
    }

    /// The paratroopers in the air and on the ground (for drawing).
    pub fn paradrop(&self) -> &Paradrop {
        &self.paradrop
    }
}

//! Mesh building for the scenery: triangles per palette colour byte (faces shaded a few ramp
//! steps by their direction to the sun), lines, and lamp dots, in block-local Bevy metres
//! (x east, y up, z south).

use std::collections::BTreeMap;
use std::f32::consts::TAU;

use bevy::prelude::*;

#[derive(Default)]
pub struct Geom {
    /// Raised geometry (casts shadows), by colour byte and detail bits (`WINDOWS`, and a
    /// window wall's direction above them: `wall_angle`).
    pub solid: BTreeMap<u32, Vec<Vec3>>,
    /// Ground-level surfaces (roads, fields), by colour byte and detail bits (`FURROW_X`,
    /// `FURROW_Z`).
    pub flat: BTreeMap<u16, Vec<Vec3>>,
    pub lines: BTreeMap<u8, Vec<Vec3>>,
    /// Lights shown at night (street lamps, windows, beacons) with their colours, in drawing
    /// order: where two overlap, the later one is on top.
    pub lamps: Vec<(u8, Vec3)>,
    /// The towns' glowing lamps instead (`Effects::city_lights`): packed words
    /// (`dots::glow_word`) and positions.
    pub glow: Vec<(u32, Vec3)>,
    /// White flashers by sequence phase (approach lighting), lit one phase at a time at night.
    pub strobes: BTreeMap<usize, Vec<Vec3>>,
    /// PAPI light units (red below their angle, white above).
    pub papi: Vec<PapiLight>,
}

/// Detail bits of a flat surface's key (`Geom::flat`, over the colour byte): a field whose crop
/// rows run along x (east) or along z (north), for the ground detail's furrows.
pub const FURROW_X: u16 = 0x100;
pub const FURROW_Z: u16 = 0x200;
/// The scenery's ground (`palette_material::WORD_GROUND`), set on its flat surfaces' meshes.
pub const GROUND: u16 = 0x400;
/// A building's wall with windows (`palette_material::WORD_WINDOWS`), lit at night.
pub const WINDOWS: u16 = 0x800;

/// A window wall's outward direction in a solid key's bits 16.. (`WALL_ANGLES` steps of the
/// angle from Bevy x towards z): the shader lays the windows along it, exactly (its own
/// estimate of a face's direction wobbles from pixel to pixel, which scrambles them).
pub const WALL_ANGLES: u32 = 2048;

fn wall_angle(out: Vec3) -> u32 {
    let a = out.z.atan2(out.x).rem_euclid(TAU) / TAU;
    ((a * WALL_ANGLES as f32).round() as u32 % WALL_ANGLES) << 16
}

/// The colour word bits of a key's detail bits (`palette_material::WORD_FURROW_X`).
pub fn detail_word(key: u32) -> u32 {
    use crate::palette_material::{
        WORD_FURROW_X, WORD_FURROW_Z, WORD_GROUND, WORD_WALL_ANGLE, WORD_WINDOWS,
    };
    let key16 = key as u16;
    (if key16 & FURROW_X != 0 { WORD_FURROW_X } else { 0 })
        | (if key16 & FURROW_Z != 0 { WORD_FURROW_Z } else { 0 })
        | (if key16 & GROUND != 0 { WORD_GROUND } else { 0 })
        | (if key16 & WINDOWS != 0 { WORD_WINDOWS | ((key >> 16) << WORD_WALL_ANGLE) } else { 0 })
}

/// One PAPI light unit: block-local position, horizontal unit vector towards the approaching
/// aircraft, and the elevation angle (radians) above which it shows white.
#[derive(Clone, Copy, Debug)]
pub struct PapiLight {
    pub pos: Vec3,
    pub toward: Vec3,
    pub angle: f32,
}

/// A colour byte darkened up to three steps along its 16-colour ramp for faces turned away
/// from the sun (ramps from 0xA0 get darker towards their end).
fn shaded(c: u8, n: Vec3) -> u8 {
    if c < 0xA0 {
        return c;
    }
    let b = n.dot(crate::lighting::sun_direction());
    let step = if b > 0.45 {
        0
    } else if b > 0.0 {
        1
    } else if b > -0.45 {
        2
    } else {
        3
    };
    (c + step).min(c | 0x0F)
}

impl Geom {
    /// A convex polygon facing `out` (its winding is fixed so the face is front-facing from
    /// that side), shaded by its normal.
    pub fn face(&mut self, c: u8, pts: &[Vec3], out: Vec3) {
        self.face_bits(c, 0, pts, out);
    }

    /// `face` with detail bits over its colour byte (`WINDOWS`).
    fn face_bits(&mut self, c: u8, bits: u16, pts: &[Vec3], out: Vec3) {
        let n = (pts[1] - pts[0]).cross(pts[2] - pts[0]);
        let n = if n.dot(out) < 0.0 { -n } else { n }.normalize_or_zero();
        let angle = if bits & WINDOWS != 0 { wall_angle(n) } else { 0 };
        let tris = self.solid.entry((shaded(c, n) as u16 | bits) as u32 | angle).or_default();
        fan(tris, pts, out);
    }

    /// A flat ground polygon (facing up, unshaded, no shadow).
    pub fn ground(&mut self, c: u8, pts: &[Vec3]) {
        self.ground_detail(c as u16, pts);
    }

    /// A flat ground polygon with detail bits over its colour byte (`FURROW_X`, `FURROW_Z`).
    pub fn ground_detail(&mut self, key: u16, pts: &[Vec3]) {
        fan(self.flat.entry(key).or_default(), pts, Vec3::Y);
    }

    pub fn line(&mut self, c: u8, a: Vec3, b: Vec3) {
        self.lines.entry(c).or_default().extend([a, b]);
    }

    pub fn lamp(&mut self, c: u8, p: Vec3) {
        self.lamps.push((c, p));
    }

    /// A glowing lamp (`dots::glow_word`).
    pub fn glow(&mut self, word: u32, p: Vec3) {
        self.glow.push((word, p));
    }

    pub fn strobe(&mut self, phase: usize, p: Vec3) {
        self.strobes.entry(phase).or_default().push(p);
    }

    /// Vertical walls of a convex footprint from `y0` to `y1`.
    pub fn walls(&mut self, c: u8, foot: &[Vec3], y0: f32, y1: f32) {
        self.walls_bits(c, 0, foot, y0, y1);
    }

    /// Walls with windows lit at night (`WINDOWS`).
    pub fn window_walls(&mut self, c: u8, foot: &[Vec3], y0: f32, y1: f32) {
        self.walls_bits(c, WINDOWS, foot, y0, y1);
    }

    fn walls_bits(&mut self, c: u8, bits: u16, foot: &[Vec3], y0: f32, y1: f32) {
        let centre = foot.iter().copied().sum::<Vec3>() / foot.len() as f32;
        for i in 0..foot.len() {
            let (a, b) = (foot[i], foot[(i + 1) % foot.len()]);
            let out = (a + b) * 0.5 - centre;
            let (up0, up1) = (Vec3::Y * y0, Vec3::Y * y1);
            let pts = [a + up0, b + up0, b + up1, a + up1];
            self.face_bits(c, bits, &pts, Vec3::new(out.x, 0.0, out.z));
        }
    }

    /// A flat roof (or any horizontal cap) over a footprint.
    pub fn cap(&mut self, c: u8, foot: &[Vec3], y: f32) {
        let pts: Vec<Vec3> = foot.iter().map(|p| *p + Vec3::Y * y).collect();
        self.face(c, &pts, Vec3::Y);
    }

    /// A box of a footprint with walls and flat top.
    pub fn block(&mut self, wall: u8, roof: u8, foot: &[Vec3], y0: f32, y1: f32) {
        self.walls(wall, foot, y0, y1);
        self.cap(roof, foot, y1);
    }

    /// Gable roof over a rectangle (`foot` = 4 corners, ridge along the first edge), with its
    /// two gable triangles in the wall colour.
    pub fn gable(&mut self, wall: u8, roof: u8, foot: &[Vec3; 4], y: f32, h: f32) {
        let (a, b, c, d) = (foot[0], foot[1], foot[2], foot[3]);
        let up = Vec3::Y * y;
        let ridge = Vec3::Y * (y + h);
        let (r0, r1) = ((a + d) * 0.5 + ridge, (b + c) * 0.5 + ridge);
        let centre = (a + b + c + d) * 0.25 + up;
        self.face(roof, &[a + up, b + up, r1, r0], ((a + b) * 0.5 + up - centre) + Vec3::Y * 0.01);
        self.face(roof, &[c + up, d + up, r0, r1], ((c + d) * 0.5 + up - centre) + Vec3::Y * 0.01);
        self.face(wall, &[d + up, a + up, r0], (a + d) * 0.5 - (b + c) * 0.5);
        self.face(wall, &[b + up, c + up, r1], (b + c) * 0.5 - (a + d) * 0.5);
    }

    /// Hip roof over a rectangle (ridge along the first edge, shortened by the end slopes).
    pub fn hip(&mut self, roof: u8, foot: &[Vec3; 4], y: f32, h: f32) {
        let (a, b, c, d) = (foot[0], foot[1], foot[2], foot[3]);
        let up = Vec3::Y * y;
        let along = (b - a).normalize_or_zero();
        let inset = ((d - a).length() * 0.5).min((b - a).length() * 0.45);
        let ridge = Vec3::Y * (y + h);
        let (r0, r1) =
            ((a + d) * 0.5 + along * inset + ridge, (b + c) * 0.5 - along * inset + ridge);
        let centre = (a + b + c + d) * 0.25 + up;
        let o = |p: Vec3, q: Vec3| (p + q) * 0.5 + up - centre + Vec3::Y * 0.01;
        self.face(roof, &[a + up, b + up, r1, r0], o(a, b));
        self.face(roof, &[c + up, d + up, r0, r1], o(c, d));
        self.face(roof, &[d + up, a + up, r0], o(d, a));
        self.face(roof, &[b + up, c + up, r1], o(b, c));
    }

    /// Pyramid / cone over a convex footprint up to an apex.
    pub fn spire(&mut self, c: u8, foot: &[Vec3], y: f32, apex: Vec3) {
        let centre = foot.iter().copied().sum::<Vec3>() / foot.len() as f32 + Vec3::Y * y;
        for i in 0..foot.len() {
            let (a, b) = (foot[i] + Vec3::Y * y, foot[(i + 1) % foot.len()] + Vec3::Y * y);
            self.face(c, &[a, b, apex], (a + b) * 0.5 - centre + Vec3::Y * 0.01);
        }
    }
}

/// Fan-triangulates a convex polygon, wound counter-clockwise as seen from `out`.
fn fan(tris: &mut Vec<Vec3>, pts: &[Vec3], out: Vec3) {
    let n = (pts[1] - pts[0]).cross(pts[2] - pts[0]);
    let flip = n.dot(out) < 0.0;
    for i in 1..pts.len() - 1 {
        if flip {
            tris.extend([pts[0], pts[i + 1], pts[i]]);
        } else {
            tris.extend([pts[0], pts[i], pts[i + 1]]);
        }
    }
}

/// Corners of a rectangle centred at `c` (length `l` along `dir`, width `w`), counter-clockwise
/// seen from above, starting with the edge along `dir`.
pub fn rect(c: Vec3, dir: Vec2, l: f32, w: f32) -> [Vec3; 4] {
    let u = Vec3::new(dir.x, 0.0, dir.y) * (l * 0.5);
    let v = Vec3::new(-dir.y, 0.0, dir.x) * (w * 0.5);
    [c - u - v, c + u - v, c + u + v, c - u + v]
}

/// Regular polygon of `n` corners around `c`.
pub fn ring(c: Vec3, r: f32, n: usize, phase: f32) -> Vec<Vec3> {
    (0..n)
        .map(|i| {
            let a = phase + i as f32 / n as f32 * TAU;
            c + Vec3::new(a.cos() * r, 0.0, a.sin() * r)
        })
        .collect()
}

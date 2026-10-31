//! Geometry of one scenery block: the towns' and cities' paved ground, roads with street lamps,
//! buildings, pylons and wires (main tier near the viewpoint, the simpler town tier beyond), and
//! trees (detail tier near the viewpoint, far tier beyond, the thinned distant tier furthest).
//! Everything is generated from the network and per-position hashes, so a block looks the same
//! whenever it is streamed in, and the tree tiers place the same trees (the distant tier a
//! share of them).

use std::f32::consts::TAU;

use bevy::math::DVec2;
use bevy::prelude::*;

use super::Scenery;
use super::fields::{self, Farmland};
use super::geom::{Geom, rect, ring};
use super::network::{
    Building, Kind, Network, PowerLine, Rng, RoadKind, Roof, TownKind, hash, noise, street_spacing,
};
use super::style::{Style, Tree};

/// Block size (metres).
pub const BLOCK: f64 = 1000.0;
/// Block size of the farmland tier (metres).
pub const FIELD_BLOCK: f64 = 4000.0;
/// Block size of the coarse tiers (town, distant trees): 2x2 blocks in one mesh, a quarter of
/// the draws.
pub const COARSE_BLOCK: f64 = 2000.0;
/// Heights of the fields and shelterbelt strips: above the coarsest ground overlays (sea,
/// lakes), below the finer ones (woods, airfields) and the roads.
const FIELD_Y: f32 = 0.08;
const BELT_Y: f32 = 0.11;
const BELT: u8 = 0xAC;
/// Height of road surfaces above the ground overlays (metres).
const ROAD_Y: f32 = 0.9;
/// The built-up ground of towns and cities: concrete in a few shades, between the fields and
/// the roads.
const PAVED_Y: f32 = 0.5;
const PAVED: [u8; 4] = [0xE9, 0xEA, 0xE9, 0xE8];
/// Paved where a town's core (1 at the centre, 0 at the edge) is above this, give or take
/// `PAVED_RAGGED` (the edge of the paving wanders), so the outskirts' houses keep gardens.
const PAVED_EDGE: f64 = 0.22;
const PAVED_RAGGED: f64 = 0.3;
const ROAD: u8 = 0xEC;
const LAMP: u8 = 0x94;
const WINDOW: u8 = 0x95;
const BEACON: u8 = 0x92;
const STEEL: u8 = 0xE7;
const WIRE: u8 = 0xEE;
const WOOD: u8 = 0xCE;
const LAMP_SPACING: f64 = 50.0;
/// Tree grid spacing (metres).
const TREE_GRID: f64 = 24.0;
/// Share of the trees the distant tier draws, each that much wider, so the woods keep their
/// cover.
const DISTANT_TREES: f64 = 0.35;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Tier {
    /// Roads, lamps, buildings, power lines.
    Main,
    /// Main beyond its radius, in `COARSE_BLOCK` blocks: plain boxes and spires for the
    /// buildings, no lamp posts, poles or wires, a window or none per building.
    Town,
    /// Trees.
    Detail,
    /// Distant trees: one small 3-sided cone or double cone per tree (3-6 triangles instead
    /// of 10-30), no bushes or trunks.
    Far,
    /// Far trees beyond the far tier, in `COARSE_BLOCK` blocks: `DISTANT_TREES` of them,
    /// wider.
    Distant,
    /// Farmland: fields and the shelterbelt strips, in `FIELD_BLOCK` blocks.
    Fields,
}

impl Tier {
    pub fn block_size(self) -> f64 {
        match self {
            Tier::Fields => FIELD_BLOCK,
            Tier::Town | Tier::Distant => COARSE_BLOCK,
            _ => BLOCK,
        }
    }
}

/// How a tree is drawn.
#[derive(Clone, Copy, PartialEq)]
enum TreeLod {
    Near,
    Far,
    /// Far, `DISTANT_TREES` of them, wider.
    Distant,
}

/// Block-local frame: ground metres -> Bevy (x east, y up, z south) around the block centre.
pub(super) struct Frame {
    o: DVec2,
    min: DVec2,
    size: f64,
}

impl Frame {
    fn new(bx: i32, by: i32, size: f64) -> Self {
        let min = DVec2::new(bx as f64, by as f64) * size;
        Self { o: min + DVec2::splat(size * 0.5), min, size }
    }

    /// A frame around any origin (whole-airfield meshes).
    pub(super) fn around(o: DVec2) -> Self {
        Self { o, min: o - DVec2::splat(BLOCK * 0.5), size: BLOCK }
    }

    fn max(&self) -> DVec2 {
        self.min + DVec2::splat(self.size)
    }

    pub(super) fn at(&self, p: DVec2, y: f32) -> Vec3 {
        Vec3::new((p.x - self.o.x) as f32, y, -(p.y - self.o.y) as f32)
    }

    fn contains(&self, p: DVec2) -> bool {
        p.x >= self.min.x
            && p.y >= self.min.y
            && p.x < self.min.x + self.size
            && p.y < self.min.y + self.size
    }
}

/// Ground direction -> Bevy xz direction.
fn dir2(d: DVec2) -> Vec2 {
    Vec2::new(d.x as f32, -d.y as f32)
}

/// Bevy horizontal unit vector of a ground direction.
pub(super) fn dir3(d: DVec2) -> Vec3 {
    Vec3::new(d.x as f32, 0.0, -d.y as f32)
}

/// Block (`bx`, `by`) of `tier`; `houses`: the share of the small buildings the town tier
/// draws (the automatic scenery detail thins them).
pub fn build(sc: &Scenery, bx: i32, by: i32, tier: Tier, houses: f64) -> Geom {
    let f = Frame::new(bx, by, tier.block_size());
    let mut g = Geom::default();
    match tier {
        Tier::Main | Tier::Town => {
            let low = tier == Tier::Town;
            paving(sc, &f, &mut g);
            roads(&sc.net, &f, &mut g, low);
            for i in sc.net.building_index.around(f.min, f.max()) {
                let b = &sc.net.buildings[i as usize];
                // Not in a clearing (round the friendly air defence and its new sites).
                if !f.contains(b.p) || sc.cleared(b.p) {
                    continue;
                }
                if low {
                    if small(b) && (hash(b.seed, 0, 0, 7) % 1000) as f64 >= houses * 1000.0 {
                        continue;
                    }
                    low_building(b, &f, &mut g);
                } else {
                    building(b, &f, &mut g);
                }
            }
            power(&sc.net, &f, &mut g, low);
        }
        Tier::Detail => trees(sc, &f, &mut g, TreeLod::Near),
        Tier::Far => trees(sc, &f, &mut g, TreeLod::Far),
        Tier::Distant => trees(sc, &f, &mut g, TreeLod::Distant),
        Tier::Fields => {
            let min = DVec2::new(bx as f64, by as f64) * FIELD_BLOCK;
            farmland(sc, &Frame::around(min + FIELD_BLOCK * 0.5), min, min + FIELD_BLOCK, &mut g);
            // On the hills the fields are drawn with their ground (`skin`).
            return g;
        }
    }
    drape(&sc.hills, &f, &mut g);
    g
}

/// The longest edge of the ground's triangles and lines laid on the hills (metres): well under
/// the drawn ground's grid (`stealth_sim::hills::GRID_M`), so they follow its slopes.
const DRAPE_EDGE: f32 = 16.0;
/// Lines up to this high are on the ground (roads' centre lines), higher ones span between
/// things standing on it (wires).
const DRAPE_LINE_Y: f32 = 2.0;
/// How far a block's geometry may reach past its edge (metres).
const DRAPE_MARGIN: f64 = 300.0;

/// Lays a block's geometry on the rolling hills: the flat parts split to follow the ground,
/// every point raised by the ground under it (so walls stay upright and trees stand on it).
pub(super) fn drape(hills: &stealth_sim::hills::Hills, f: &Frame, g: &mut Geom) {
    let (lo, hi) = (f.min - DRAPE_MARGIN, f.max() + DRAPE_MARGIN);
    let Some(patch) = stealth_sim::hills::Patch::new(hills, (lo.x, lo.y), (hi.x, hi.y)) else {
        return;
    };
    let raise = |v: &mut Vec3| v.y += patch.ground_m(f.o.x + v.x as f64, f.o.y - v.z as f64);
    for tris in g.flat.values_mut() {
        *tris = split_tris(tris);
        tris.iter_mut().for_each(raise);
    }
    for tris in g.solid.values_mut() {
        tris.iter_mut().for_each(raise);
    }
    for lines in g.lines.values_mut() {
        *lines = split_lines(lines);
        lines.iter_mut().for_each(raise);
    }
    g.lamps.iter_mut().for_each(|(_, p)| raise(p));
    g.strobes.values_mut().flatten().for_each(raise);
    g.papi.iter_mut().for_each(|l| raise(&mut l.pos));
}

/// Triangles split at their longest edges until none is longer than `DRAPE_EDGE`.
fn split_tris(tris: &[Vec3]) -> Vec<Vec3> {
    let mut out = Vec::with_capacity(tris.len());
    let mut todo: Vec<[Vec3; 3]> = tris.as_chunks::<3>().0.to_vec();
    while let Some(t) = todo.pop() {
        let len = [0, 1, 2].map(|k| t[k].xz().distance_squared(t[(k + 1) % 3].xz()));
        let k = (0..3).max_by(|&a, &b| len[a].total_cmp(&len[b])).unwrap();
        if len[k] <= DRAPE_EDGE * DRAPE_EDGE {
            out.extend(t);
            continue;
        }
        let (a, b, c) = (t[k], t[(k + 1) % 3], t[(k + 2) % 3]);
        let m = (a + b) * 0.5;
        todo.push([a, m, c]);
        todo.push([m, b, c]);
    }
    out
}

/// Ground lines (`DRAPE_LINE_Y`) split into pieces of up to `DRAPE_EDGE`.
fn split_lines(lines: &[Vec3]) -> Vec<Vec3> {
    let mut out = Vec::with_capacity(lines.len());
    for l in lines.as_chunks::<2>().0 {
        let (a, b) = (l[0], l[1]);
        if a.y.max(b.y) > DRAPE_LINE_Y {
            out.extend(l);
            continue;
        }
        let n = (a.xz().distance(b.xz()) / DRAPE_EDGE).ceil().max(1.0) as usize;
        for i in 0..n {
            out.push(a.lerp(b, i as f32 / n as f32));
            out.push(a.lerp(b, (i + 1) as f32 / n as f32));
        }
    }
    out
}

/// What a farmland section is used for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Land {
    Fields,
    /// Grass with a few trees and copses.
    Meadow,
    /// Left to the trees (the forest noise grows them there).
    Woods,
}

/// The use of section (sx, sy), by the noise at its centre.
fn section_land(style: &Style, land: &Farmland, sx: i64, sy: i64) -> Land {
    let c = (DVec2::new(sx as f64, sy as f64) + 0.5) * land.section;
    if land.woods && noise(style.seed, c, 1600.0) > style.forest {
        Land::Woods
    } else if land.meadow > 0.0 && noise(style.seed ^ 0x3EAD0, c, 2600.0) < land.meadow {
        Land::Meadow
    } else {
        Land::Fields
    }
}

/// The use of the section `p` lies in.
fn land_at(style: &Style, land: &Farmland, p: DVec2) -> Land {
    let s = (p / land.section).floor();
    section_land(style, land, s.x as i64, s.y as i64)
}

/// Whether a rectangle of farmland is open land away from the towns.
fn farmland_ok(sc: &Scenery, min: DVec2, max: DVec2, r: f64) -> bool {
    let d = DVec2::splat(200.0);
    let clear_of_towns = !sc.net.town_index.around(min - d, max + d).any(|i| {
        let t = &sc.net.towns[i as usize];
        let q = t.c.clamp(min, max);
        t.near(q, 1.1, 60.0)
    });
    clear_of_towns && sc.ground.rect_free(min, max, r)
}

/// A shelterbelt's ground rectangle, if it may stand.
fn belt_rect(sc: &Scenery, land: &Farmland, a: DVec2, b: DVec2) -> Option<(DVec2, DVec2)> {
    let h = DVec2::splat(land.belt_width * 0.5);
    let (min, max) = (a.min(b) - h, a.max(b) + h);
    farmland_ok(sc, min, max, 4.0).then_some((min, max))
}

/// The field drawn at `p`, if any (as `farmland` draws it: the whole field or its middle).
fn field_rect(sc: &Scenery, land: &Farmland, seed: u64, p: DVec2) -> Option<(DVec2, DVec2)> {
    let s = (p / land.section).floor();
    if sc
        .style
        .as_ref()
        .is_some_and(|st| section_land(st, land, s.x as i64, s.y as i64) != Land::Fields)
    {
        return None;
    }
    let field = fields::section(land, seed, s.x as i64, s.y as i64)
        .into_iter()
        .find(|f| p.cmpge(f.min).all() && p.cmple(f.max).all())?;
    let (lo, hi) = (field.min, field.max);
    if farmland_ok(sc, lo, hi, 12.0) {
        return Some((lo, hi));
    }
    let inset = (hi - lo) * 0.22;
    let (lo, hi) = (lo + inset, hi - inset);
    farmland_ok(sc, lo, hi, 12.0).then_some((lo, hi))
}

/// Whether `p` lies on a field (no trees grow there).
fn in_field(sc: &Scenery, land: &Farmland, seed: u64, p: DVec2) -> bool {
    field_rect(sc, land, seed, p).is_some_and(|(lo, hi)| p.cmpge(lo).all() && p.cmple(hi).all())
}

/// Fields and shelterbelt strips of the sections whose corner lies in the block.
fn farmland(sc: &Scenery, f: &Frame, min: DVec2, max: DVec2, g: &mut Geom) {
    let Some(style) = &sc.style else { return };
    let Some(land) = style.farmland else { return };
    let quad = |g: &mut Geom, lo: DVec2, hi: DVec2, y: f32, c: u8| {
        g.ground(
            c,
            &[
                f.at(lo, y),
                f.at(DVec2::new(hi.x, lo.y), y),
                f.at(hi, y),
                f.at(DVec2::new(lo.x, hi.y), y),
            ],
        );
    };
    for (sx, sy) in fields::sections(land, min, max - 1.0) {
        let corner = DVec2::new(sx as f64, sy as f64) * land.section;
        if corner.x < min.x || corner.y < min.y {
            continue;
        }
        let kind = section_land(style, land, sx, sy);
        if kind == Land::Woods {
            continue;
        }
        let ploughed = if kind == Land::Fields {
            fields::section(land, style.seed, sx, sy)
        } else {
            Vec::new()
        };
        for field in ploughed {
            let (mut lo, mut hi) = (field.min, field.max);
            if !farmland_ok(sc, lo, hi, 12.0) {
                // A smaller field in the middle, short of the obstacle.
                let inset = (hi - lo) * 0.22;
                (lo, hi) = (lo + inset, hi - inset);
                if !farmland_ok(sc, lo, hi, 12.0) {
                    continue;
                }
            }
            quad(g, lo, hi, FIELD_Y, field.colour);
        }
        for (a, b) in fields::belts(land, style.seed, sx, sy) {
            if let Some((lo, hi)) = belt_rect(sc, land, a, b) {
                quad(g, lo, hi, BELT_Y, BELT);
            }
        }
    }
}

/// The towns' and cities' built-up ground: squares between the streets (half a block each)
/// paved with concrete, but for the parks and the outskirts.
fn paving(sc: &Scenery, f: &Frame, g: &mut Geom) {
    let scale = sc.style.as_ref().map_or(1.0, |s| s.street_scale);
    let (min, max) = (f.min, f.max());
    let mut seen = std::collections::HashSet::new();
    for id in sc.net.town_index.around(min, max) {
        let town = &sc.net.towns[id as usize];
        if town.kind == TownKind::Village || !seen.insert(id) {
            continue;
        }
        let cell = street_spacing(town.kind) * scale * 0.5;
        let (u, v) = (town.u, town.u.perp());
        let n = (town.reach() / cell).ceil() as i64;
        for i in -n..n {
            for j in -n..n {
                let q = town.c + u * ((i as f64 + 0.5) * cell) + v * ((j as f64 + 0.5) * cell);
                if !f.contains(q) {
                    continue;
                }
                let edge = PAVED_EDGE + PAVED_RAGGED * (noise(id as u64 ^ 0x9A7E, q, 300.0) - 0.5);
                if town.core(q) < edge || town.park(q) {
                    continue;
                }
                let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
                    .map(|(a, b)| q + u * (a * cell * 0.5) + v * (b * cell * 0.5));
                if !corners.iter().all(|&p| sc.ground.is_land(p)) {
                    continue;
                }
                let shade = PAVED[(hash(id as u64, i, j, 0x9A) % PAVED.len() as u64) as usize];
                g.ground(shade, &corners.map(|p| f.at(p, PAVED_Y)));
            }
        }
    }
}

/// The roads; `low`: without the lamp posts (the lamps alone).
fn roads(net: &Network, f: &Frame, g: &mut Geom, low: bool) {
    let (min, max) = (f.min, f.max());
    let mut lamps: Vec<DVec2> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for id in net.segment_index.around(min, max) {
        if !seen.insert(id) {
            continue;
        }
        let (ri, s) = net.segments[id as usize];
        let road = &net.roads[ri as usize];
        let s = s as usize;
        let (a, b) = (road.pts[s], road.pts[s + 1]);
        if !f.contains((a + b) * 0.5) {
            continue;
        }
        let half = road.width * 0.5;
        strip(g, f, &road.pts, s, half, ROAD_Y, ROAD);
        // Centre line: keeps distant roads visible like the theaters' line roads.
        g.line(ROAD, f.at(a, ROAD_Y + 0.05), f.at(b, ROAD_Y + 0.05));

        let (s0, s1) = (road.arc[s], road.arc[s + 1]);
        let mid = (a + b) * 0.5;
        let lit = match road.kind {
            RoadKind::Street => true,
            _ => {
                net.town_at(mid, 1.0).is_some() || net.towns.iter().any(|t| t.near(mid, 1.0, 350.0))
            }
        };
        if !lit {
            continue;
        }
        let dir = (b - a).normalize_or_zero();
        let mut k = (s0 / LAMP_SPACING).ceil();
        while k * LAMP_SPACING < s1 {
            let t = (k * LAMP_SPACING - s0) / (s1 - s0);
            // Alternate sides on wide roads.
            let sd = if road.kind == RoadKind::Street || (k as i64) % 2 == 0 { 1.0 } else { -1.0 };
            let foot = a.lerp(b, t) + dir.perp() * sd * (half + 1.5);
            k += 1.0;
            if lamps.iter().any(|l| l.distance(foot) < 14.0) {
                continue;
            }
            lamps.push(foot);
            let top = f.at(foot, 8.5);
            let head = top - dir3(dir.perp()) * sd as f32 * 1.8;
            if !low {
                g.line(STEEL, f.at(foot, 0.0), top);
                g.line(STEEL, top, head);
            }
            g.lamp(LAMP, head - Vec3::Y * 0.2);
        }
    }
}

/// Ground surface of segment `s` of a polyline, `half` wide on each side, with mitred ends so
/// consecutive segments join without gaps.
pub(super) fn strip(
    g: &mut Geom,
    f: &Frame,
    pts: &[DVec2],
    s: usize,
    half: f64,
    y: f32,
    colour: u8,
) {
    let side = |i: usize| -> DVec2 {
        let prev = if i > 0 { (pts[i] - pts[i - 1]).normalize_or_zero() } else { DVec2::ZERO };
        let next =
            if i + 1 < pts.len() { (pts[i + 1] - pts[i]).normalize_or_zero() } else { DVec2::ZERO };
        let n = (prev + next).normalize_or_zero().perp();
        let cos = n.dot((if next != DVec2::ZERO { next } else { prev }).perp()).max(0.5);
        n * half / cos
    };
    let (a, b) = (pts[s], pts[s + 1]);
    let (na, nb) = (side(s), side(s + 1));
    g.ground(colour, &[f.at(a - na, y), f.at(b - nb, y), f.at(b + nb, y), f.at(a + na, y)]);
}

pub(super) fn building(b: &Building, f: &Frame, g: &mut Geom) {
    let c = f.at(b.p, 0.0);
    let d = dir2(b.dir);
    let along = Vec3::new(d.x, 0.0, d.y);
    let mut rng = Rng::new(b.seed);
    let foot = rect(c, d, b.l, b.w);
    match b.kind {
        Kind::House | Kind::Apartment | Kind::Shed | Kind::Barn | Kind::Church | Kind::Tower => {
            g.walls(b.wall, &foot, 0.0, b.h);
            let rise = match b.kind {
                Kind::Shed => b.w * 0.18,
                Kind::Church => b.w * 0.6,
                _ => b.w * 0.42,
            };
            match b.roof {
                Roof::Flat => g.cap(b.roof_colour, &foot, b.h),
                Roof::Gable => g.gable(b.wall, b.roof_colour, &foot, b.h, rise),
                Roof::Hip => g.hip(b.roof_colour, &foot, b.h, rise * 0.85),
            }
            if b.kind == Kind::Church {
                let t = c - along * (b.l * 0.5 + 3.2);
                let tower = rect(t, d, 6.5, 6.5);
                let th = b.h * 2.4;
                g.block(b.wall, b.wall, &tower, 0.0, th);
                g.spire(b.roof_colour, &tower, th, t + Vec3::Y * (th + 11.0));
            }
            if b.kind == Kind::Tower && b.h > 60.0 {
                g.lamp(BEACON, c + Vec3::Y * (b.h + 1.0));
            }
            windows(b, c, d, &mut rng, g);
        }
        Kind::Ruin => ruin(b, c, d, &mut rng, g),
        Kind::Silo => {
            let r = ring(c, b.l * 0.5, 8, 0.0);
            g.walls(b.wall, &r, 0.0, b.h);
            g.spire(b.roof_colour, &r, b.h, c + Vec3::Y * (b.h + 2.5));
        }
        Kind::Chimney => {
            let r = ring(c, 2.4, 8, 0.0);
            g.block(b.wall, b.roof_colour, &r, 0.0, b.h);
            g.lamp(BEACON, c + Vec3::Y * (b.h + 0.5));
        }
        Kind::Tank => {
            let r = ring(c, b.l * 0.5, 12, 0.0);
            g.block(b.wall, b.roof_colour, &r, 0.0, b.h);
        }
        Kind::WaterTower => {
            let tank = b.h - 7.0;
            g.walls(b.wall, &ring(c, 1.6, 6, 0.0), 0.0, tank);
            let r = ring(c, 4.5, 10, 0.0);
            g.walls(b.wall, &r, tank, b.h);
            g.face(b.wall, &r.iter().map(|p| *p + Vec3::Y * tank).collect::<Vec<_>>(), -Vec3::Y);
            g.spire(b.roof_colour, &r, b.h, c + Vec3::Y * (b.h + 2.0));
        }
        Kind::Mast => {
            lattice(g, c, along, &[(0.0, 1.2), (b.h, 0.4)], 3, (b.h / 8.0) as usize);
            g.lamp(BEACON, c + Vec3::Y * (b.h + 0.5));
            g.lamp(BEACON, c + Vec3::Y * (b.h * 0.5));
        }
        Kind::Shelter => {
            // Hardened aircraft shelter: an arched vault along `dir`, door at the front.
            let across = Vec3::new(-d.y, 0.0, d.x);
            let profile: Vec<Vec3> = (0..=6)
                .map(|i| {
                    let a = i as f32 / 6.0 * std::f32::consts::PI;
                    across * (a.cos() * b.w * 0.5) + Vec3::Y * (a.sin() * b.h)
                })
                .collect();
            let (front, back) = (c + along * (b.l * 0.5), c - along * (b.l * 0.5));
            for i in 0..6 {
                let (p, q) = (profile[i], profile[i + 1]);
                let out = (p + q) * 0.5 - Vec3::Y * 0.01;
                g.face(b.wall, &[back + p, back + q, front + q, front + p], out);
            }
            g.face(b.roof_colour, &profile.iter().map(|p| front + *p).collect::<Vec<_>>(), along);
            g.face(b.wall, &profile.iter().map(|p| back + *p).collect::<Vec<_>>(), -along);
        }
        Kind::Radar => {
            // Radome on a square tower.
            let top = b.h - 9.0;
            g.block(b.wall, b.wall, &rect(c, d, 6.0, 6.0), 0.0, top);
            let rings = [(4.6, 0.0), (5.6, 2.5), (5.0, 5.5), (3.2, 7.8)];
            for w in rings.windows(2) {
                let (lo, hi) = (
                    ring(c + Vec3::Y * (top + w[0].1), w[0].0, 10, 0.0),
                    ring(c + Vec3::Y * (top + w[1].1), w[1].0, 10, 0.0),
                );
                for i in 0..10 {
                    let j = (i + 1) % 10;
                    let out = (lo[i] + lo[j] + hi[i] + hi[j]) * 0.25 - (c + Vec3::Y * (top + 4.0));
                    g.face(b.roof_colour, &[lo[i], lo[j], hi[j], hi[i]], out);
                }
            }
            g.spire(b.roof_colour, &ring(c, 3.2, 10, 0.0), top + 7.8, c + Vec3::Y * b.h);
            g.lamp(BEACON, c + Vec3::Y * (b.h + 0.3));
        }
        Kind::Windsock => {
            let top = c + Vec3::Y * b.h;
            g.line(STEEL, c, top);
            // Cone hanging half-inflated along `dir`.
            let across = Vec3::new(-d.y, 0.0, d.x);
            let rim: Vec<Vec3> = (0..6)
                .map(|i| {
                    let a = i as f32 / 6.0 * TAU;
                    top + across * (a.cos() * 0.7) + Vec3::Y * (a.sin() * 0.7 - 0.2)
                })
                .collect();
            let tip = top + along * 5.0 - Vec3::Y * 1.6;
            for i in 0..6 {
                let (p, q) = (rim[i], rim[(i + 1) % 6]);
                // Seen from inside and outside.
                g.face(b.wall, &[p, q, tip], (p + q) * 0.5 - top);
                g.face(b.wall, &[p, q, tip], top - (p + q) * 0.5);
            }
            g.lamp(BEACON, top + Vec3::Y * 0.4);
        }
        Kind::Sign => {
            let foot = rect(c, d, b.l, b.w);
            g.block(b.wall, b.roof_colour, &foot, 0.4, b.h);
            g.face(
                b.roof_colour,
                &foot.iter().map(|p| *p + Vec3::Y * 0.4).collect::<Vec<_>>(),
                -Vec3::Y,
            );
        }
    }
}

/// Houses, sheds and barns: the town tier thins them first.
fn small(b: &Building) -> bool {
    matches!(b.kind, Kind::House | Kind::Shed | Kind::Barn)
}

/// A building of the town tier: its outline as a box (a gabled roof for the pitched ones),
/// towers and tanks as prisms, masts as lines, the beacons kept; small sheds and the
/// windsocks and signs left out; a lit window or none.
fn low_building(b: &Building, f: &Frame, g: &mut Geom) {
    let c = f.at(b.p, 0.0);
    let d = dir2(b.dir);
    let foot = rect(c, d, b.l, b.w);
    match b.kind {
        Kind::House | Kind::Apartment | Kind::Shed | Kind::Barn | Kind::Church | Kind::Tower => {
            if b.l * b.w < 30.0 {
                return;
            }
            match b.roof {
                Roof::Flat => g.block(b.wall, b.roof_colour, &foot, 0.0, b.h),
                Roof::Gable | Roof::Hip => {
                    let rise = match b.kind {
                        Kind::Shed => b.w * 0.18,
                        Kind::Church => b.w * 0.6,
                        _ => b.w * 0.42,
                    };
                    g.walls(b.wall, &foot, 0.0, b.h);
                    g.gable(b.wall, b.roof_colour, &foot, b.h, rise);
                }
            }
            if b.kind == Kind::Church {
                let t = c - Vec3::new(d.x, 0.0, d.y) * (b.l * 0.5 + 3.2);
                let tower = rect(t, d, 6.5, 6.5);
                let th = b.h * 2.4;
                g.walls(b.wall, &tower, 0.0, th);
                g.spire(b.roof_colour, &tower, th, t + Vec3::Y * (th + 11.0));
            }
            if b.kind == Kind::Tower && b.h > 60.0 {
                g.lamp(BEACON, c + Vec3::Y * (b.h + 1.0));
            }
            let mut rng = Rng::new(b.seed);
            if matches!(b.kind, Kind::Apartment | Kind::Tower) && rng.chance(0.6) {
                let floor = (rng.f() * (b.h / 3.0).floor().max(1.0) as f64).floor() as f32;
                let across = Vec3::new(-d.y, 0.0, d.x);
                g.lamp(WINDOW, c + across * (b.w * 0.5 + 0.15) + Vec3::Y * (floor * 3.0 + 1.6));
            }
        }
        Kind::Ruin => g.block(SOOT, RUBBLE, &foot, 0.0, b.h * 0.4),
        Kind::Silo | Kind::Tank => {
            g.block(b.wall, b.roof_colour, &ring(c, b.l * 0.5, 6, 0.0), 0.0, b.h)
        }
        Kind::Chimney => {
            g.walls(b.wall, &ring(c, 2.4, 4, 0.0), 0.0, b.h);
            g.lamp(BEACON, c + Vec3::Y * (b.h + 0.5));
        }
        Kind::WaterTower => {
            g.block(b.wall, b.roof_colour, &ring(c, 4.5, 6, 0.0), b.h - 7.0, b.h);
            g.line(b.wall, c, c + Vec3::Y * (b.h - 7.0));
        }
        Kind::Mast => {
            g.line(STEEL, c, c + Vec3::Y * b.h);
            g.lamp(BEACON, c + Vec3::Y * (b.h + 0.5));
            g.lamp(BEACON, c + Vec3::Y * (b.h * 0.5));
        }
        Kind::Shelter => g.block(b.wall, b.wall, &foot, 0.0, b.h * 0.8),
        Kind::Radar => {
            g.walls(b.wall, &rect(c, d, 6.0, 6.0), 0.0, b.h - 9.0);
            g.spire(b.roof_colour, &ring(c, 5.0, 6, 0.0), b.h - 9.0, c + Vec3::Y * b.h);
            g.lamp(BEACON, c + Vec3::Y * (b.h + 0.3));
        }
        Kind::Windsock | Kind::Sign => {}
    }
}

/// Wall colour of burnt-out walls and of the inside faces of ruins.
const SOOT: u8 = 0xEB;
const RUBBLE: u8 = 0xE8;

/// A gutted building: wall panels broken off at random heights with gaps blown in them, soot
/// grey inside, a floor slab hanging down here and there, rubble mounds in the shell.
fn ruin(b: &Building, c: Vec3, d: Vec2, rng: &mut Rng, g: &mut Geom) {
    let along = Vec3::new(d.x, 0.0, d.y);
    let across = Vec3::new(-d.y, 0.0, d.x);
    let foot = rect(c, d, b.l, b.w);
    // How much still stands: one end usually more than the other.
    let (s0, s1) = (rng.range(0.15, 1.0) as f32, rng.range(0.1, 0.8) as f32);
    let wall = if rng.chance(0.3) { SOOT } else { b.wall };
    let panel = if b.h > 8.0 { 4.0 } else { 3.0 };
    for i in 0..4 {
        let (p, q) = (foot[i], foot[(i + 1) % 4]);
        let len = p.distance(q);
        let n = (len / panel).ceil().max(1.0) as usize;
        let out = (p + q) * 0.5 - c;
        let out = Vec3::new(out.x, 0.0, out.z).normalize_or_zero();
        let mut prev = b.h * rng.range(0.2, 1.0) as f32;
        for k in 0..n {
            let (t0, t1) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
            let (a, e) = (p.lerp(q, t0), p.lerp(q, t1));
            // Standing share along the building's length.
            let x = ((e - c).dot(along) / b.l + 0.5).clamp(0.0, 1.0);
            let stand = s0 + (s1 - s0) * x;
            let h = (b.h * stand * rng.range(0.4, 1.1) as f32).clamp(0.6, b.h);
            if rng.chance(0.15) {
                prev = 0.6;
                continue;
            }
            let (h0, h1) = (prev.min(h + 2.0), h);
            prev = h1;
            g.face(wall, &[a, e, e + Vec3::Y * h1, a + Vec3::Y * h0], out);
            let inset = -out * 0.3;
            g.face(
                SOOT,
                &[a + inset, e + inset, e + inset + Vec3::Y * h1, a + inset + Vec3::Y * h0],
                -out,
            );
        }
    }
    // A slab hanging from a floor that still stands.
    if b.h > 8.0 && rng.chance(0.6) {
        let y = 3.0 * (rng.range(1.0, (b.h * s0 / 3.0).max(1.5) as f64)).floor() as f32;
        let x0 = rng.range(-0.45, 0.1) as f32 * b.l;
        let x1 = x0 + rng.range(0.15, 0.35) as f32 * b.l;
        let drop = rng.range(1.5, 5.0) as f32;
        let at = |x: f32, s: f32, dy: f32| {
            c + along * x + across * s * (b.w * 0.5 - 0.4) + Vec3::Y * (y - dy)
        };
        let pts =
            [at(x0, -1.0, 0.0), at(x1, -1.0, drop), at(x1, 1.0, drop * 0.4), at(x0, 1.0, 0.0)];
        g.face(0xE6, &pts, Vec3::Y);
        g.face(SOOT, &pts, -Vec3::Y);
    }
    // Rubble inside and spilling out.
    let mounds = 2 + (b.l / 25.0) as usize;
    for _ in 0..mounds {
        let p = c
            + along * rng.range(-0.5, 0.5) as f32 * b.l
            + across * rng.range(-0.7, 0.7) as f32 * b.w;
        let r = rng.range(2.5, 6.0) as f32 + b.w * 0.15;
        let h = (b.h * (1.0 - s0.min(s1)) * 0.25 + rng.range(0.8, 2.5) as f32).min(r * 0.9);
        g.spire(
            rng.pick(&[RUBBLE, 0xE7, 0xE9, 0xEA]),
            &ring(p, r, 5, rng.range(0.0, 6.3) as f32),
            0.0,
            p + Vec3::Y * h,
        );
    }
}

/// Lit windows at night on the long sides.
fn windows(b: &Building, c: Vec3, d: Vec2, rng: &mut Rng, g: &mut Geom) {
    let n = match b.kind {
        Kind::Apartment => rng.range(1.0, 7.0) as i32,
        Kind::Tower => rng.range(2.0, 10.0) as i32,
        Kind::House if rng.chance(0.45) => 1,
        _ => 0,
    };
    let along = Vec3::new(d.x, 0.0, d.y);
    let across = Vec3::new(-d.y, 0.0, d.x);
    let floors = (b.h / 3.0).floor().max(1.0);
    for _ in 0..n {
        let side = if rng.chance(0.5) { 1.0 } else { -1.0 };
        let floor = (rng.f() * floors as f64).floor() as f32;
        let x = rng.range(-0.4, 0.4) as f32 * b.l;
        g.lamp(
            WINDOW,
            c + along * x + across * side * (b.w * 0.5 + 0.15) + Vec3::Y * (floor * 3.0 + 1.6),
        );
    }
}

/// Lattice tower: legs through the given (height, half width) sections, rings and bracing.
fn lattice(g: &mut Geom, c: Vec3, along: Vec3, sections: &[(f32, f32)], legs: usize, steps: usize) {
    let corners = |y: f32, hw: f32| -> Vec<Vec3> {
        let phase = along.z.atan2(along.x);
        (0..legs)
            .map(|i| {
                let a = phase
                    + TAU / 8.0 * if legs == 4 { 1.0 } else { 0.0 }
                    + i as f32 * TAU / legs as f32;
                c + Vec3::new(a.cos(), 0.0, a.sin())
                    * hw
                    * if legs == 4 { std::f32::consts::SQRT_2 } else { 1.0 }
                    + Vec3::Y * y
            })
            .collect()
    };
    for w in sections.windows(2) {
        let ((y0, h0), (y1, h1)) = (w[0], w[1]);
        let n = steps.max(1);
        let mut prev = corners(y0, h0);
        for s in 1..=n {
            let t = s as f32 / n as f32;
            let cur = corners(y0 + (y1 - y0) * t, h0 + (h1 - h0) * t);
            for i in 0..legs {
                let j = (i + 1) % legs;
                g.line(STEEL, prev[i], cur[i]);
                g.line(STEEL, cur[i], cur[j]);
                g.line(STEEL, prev[i], cur[j]);
            }
            prev = cur;
        }
    }
}

/// Conductor attachment points of a pylon (relative, `across` = horizontal unit across the line).
pub(super) fn conductors(big: bool, across: Vec3) -> Vec<Vec3> {
    if big {
        vec![
            across * -8.5 + Vec3::Y * 23.0,
            across * 8.5 + Vec3::Y * 23.0,
            across * -5.5 + Vec3::Y * 29.0,
            across * 5.5 + Vec3::Y * 29.0,
        ]
    } else {
        vec![across * -1.1 + Vec3::Y * 8.6, across * 1.1 + Vec3::Y * 8.6]
    }
}

/// Pylons and wires; `low`: the transmission lines' pylons as single lines, their wires
/// straight, no pole lines.
fn power(net: &Network, f: &Frame, g: &mut Geom, low: bool) {
    let (min, max) = (f.min, f.max());
    for id in net.pylon_index.around(min, max) {
        let (li, k) = net.pylons[id as usize];
        let line = &net.lines[li as usize];
        let k = k as usize;
        let p = line.pylons[k];
        if !f.contains(p) {
            continue;
        }
        let dir_at = |k: usize| line_dir(line, k);
        let dir = dir_at(k);
        let across = dir3(dir.perp());
        let c = f.at(p, 0.0);
        if low {
            if line.big {
                g.line(STEEL, c, c + Vec3::Y * 31.0);
                if let Some(&q) = line.pylons.get(k + 1) {
                    g.line(STEEL, c + Vec3::Y * 29.0, f.at(q, 29.0));
                }
            }
            continue;
        }
        if line.big {
            lattice(g, c, dir3(dir), &[(0.0, 4.0), (15.0, 1.6), (31.0, 0.8)], 4, 3);
            for (y, w) in [(23.0, 9.0), (29.0, 6.0)] {
                let base = c + Vec3::Y * y;
                g.line(STEEL, base - across * w, base + across * w);
                g.line(STEEL, base - across * w, base + Vec3::Y * 2.0);
                g.line(STEEL, base + across * w, base + Vec3::Y * 2.0);
            }
        } else {
            g.line(WOOD, c, c + Vec3::Y * 9.0);
            g.line(WOOD, c + across * -1.3 + Vec3::Y * 8.6, c + across * 1.3 + Vec3::Y * 8.6);
        }
        if k + 1 < line.pylons.len() {
            let q = line.pylons[k + 1];
            let cq = f.at(q, 0.0);
            let aq = dir3(dir_at(k + 1).perp());
            let span = p.distance(q) as f32;
            let sag = sag(line.big, span);
            for (pa, pb) in conductors(line.big, across).into_iter().zip(conductors(line.big, aq)) {
                let (a, b) = (c + pa, cq + pb);
                let n = if line.big { 6 } else { 2 };
                let pt = |i: usize| {
                    let t = i as f32 / n as f32;
                    a.lerp(b, t) - Vec3::Y * sag * 4.0 * t * (1.0 - t)
                };
                for i in 0..n {
                    g.line(WIRE, pt(i), pt(i + 1));
                }
            }
        }
    }
}

/// Unit ground direction of a power line at its pylon `k`.
pub(super) fn line_dir(line: &PowerLine, k: usize) -> DVec2 {
    let a = line.pylons[k.saturating_sub(1)];
    let b = line.pylons[(k + 1).min(line.pylons.len() - 1)];
    (b - a).normalize_or_zero()
}

/// How far a wire sags at mid-span (metres).
pub(super) fn sag(big: bool, span: f32) -> f32 {
    if big { span * 0.025 } else { 0.5 }
}

/// The tree or bush growing in tree grid cell (`gx`, `gy`), if one does: where, whether it is
/// a bush, and the cell's random numbers to draw it with.
fn tree_in_cell(sc: &Scenery, style: &Style, gx: i64, gy: i64) -> Option<(DVec2, bool, Rng)> {
    let seed = style.seed;
    let mut rng = Rng::new(hash(seed, gx, gy, 9));
    let p = DVec2::new(
        (gx as f64 + rng.range(0.1, 0.9)) * TREE_GRID,
        (gy as f64 + rng.range(0.1, 0.9)) * TREE_GRID,
    );
    // The theater's own woods are dense, elsewhere the noise decides.
    let wood = sc.ground.is_forest(p, 3.0);
    let cover = noise(seed, p, 1600.0);
    let forest = if wood { 1.0 } else { ((cover - style.forest) / 0.1).clamp(0.0, 1.0) };
    let roll = rng.f();
    let in_town = sc.net.town_at(p, 0.95).is_some();
    // Meadows: lone trees and copses.
    let meadow =
        !wood && style.farmland.is_some_and(|land| land_at(style, land, p) == Land::Meadow);
    let copse = meadow && noise(seed ^ 0xC0B5E, p, 90.0) > 0.66;
    let (tree, bush) = if in_town {
        (roll < 0.05, false)
    } else if meadow {
        (roll < forest * 0.85 || roll > 0.98 || (copse && roll < 0.7), (0.96..0.98).contains(&roll))
    } else {
        (roll < forest * 0.85 || roll > 0.995, forest > 0.0 && forest < 0.5 && roll > 0.97)
    };
    if !tree && !bush {
        return None;
    }
    if !(wood || sc.ground.is_free(p, 3.0))
        || sc.cleared(p)
        || !sc.net.road_clear(p, 4.0)
        || !sc.net.building_clear(p, 3.0)
        || (!wood && style.farmland.is_some_and(|land| in_field(sc, land, seed, p)))
    {
        return None;
    }
    Some((p, bush, rng))
}

/// What grows in a tree grid cell.
enum Grows {
    /// A bush of this radius.
    Bush(f32),
    /// A tree: its kind, height and turn.
    Tree(Tree, f32, f32),
}

/// The tree or bush in tree grid cell (`gx`, `gy`), if one grows there: where, what, and the
/// cell's random numbers left to draw it with.
fn grid_tree(sc: &Scenery, style: &Style, gx: i64, gy: i64) -> Option<(DVec2, Grows, Rng)> {
    let (p, bush, mut rng) = tree_in_cell(sc, style, gx, gy)?;
    let grows = if bush {
        Grows::Bush(rng.range(1.5, 3.0) as f32)
    } else {
        let h = rng.range(8.0, 18.0) as f32;
        let phase = rng.range(0.0, 6.3) as f32;
        Grows::Tree(style.tree(&mut rng), h, phase)
    };
    Some((p, grows, rng))
}

/// A tree or bush (of the tree grid) within `r` metres of `p`.
pub(super) fn tree_near(sc: &Scenery, p: DVec2, r: f64) -> bool {
    let Some(style) = &sc.style else { return false };
    let (lo, hi) = ((p - r) / TREE_GRID, (p + r) / TREE_GRID);
    let cells = (lo.y.floor() as i64..=hi.y.floor() as i64)
        .flat_map(|gy| (lo.x.floor() as i64..=hi.x.floor() as i64).map(move |gx| (gx, gy)));
    cells.filter_map(|(gx, gy)| tree_in_cell(sc, style, gx, gy)).any(|(q, ..)| q.distance(p) < r)
}

/// The distant tier draws this tree (a fixed share of them, by position).
fn distant_keeps(seed: u64, p: DVec2) -> bool {
    hash(seed ^ 0xD157, p.x as i64, p.y as i64, 3) % 1000 < (DISTANT_TREES * 1000.0) as u64
}

fn trees(sc: &Scenery, f: &Frame, g: &mut Geom, lod: TreeLod) {
    let Some(style) = &sc.style else { return };
    let n = (f.size / TREE_GRID) as i64;
    let (gx0, gy0) = ((f.min.x / TREE_GRID) as i64, (f.min.y / TREE_GRID) as i64);
    let far = lod != TreeLod::Near;
    for j in 0..n {
        for i in 0..n {
            let Some((p, grows, mut rng)) = grid_tree(sc, style, gx0 + i, gy0 + j) else {
                continue;
            };
            if lod == TreeLod::Distant && !distant_keeps(style.seed, p) {
                continue;
            }
            let c = f.at(p, 0.0);
            match grows {
                Grows::Bush(_) if far => {}
                Grows::Bush(s) => g.spire(
                    0xAC,
                    &ring(c, s, 5, rng.range(0.0, 6.3) as f32),
                    0.0,
                    c + Vec3::Y * s * 1.2,
                ),
                Grows::Tree(kind, h, phase) => tree_at(g, &mut rng, kind, c, h, phase, lod),
            }
        }
    }
    if let Some(land) = style.farmland {
        belt_trees(sc, land, f, g, lod);
    }
}

/// Two staggered rows of trees along the shelterbelts crossing a block.
fn belt_trees(sc: &Scenery, land: &Farmland, f: &Frame, g: &mut Geom, lod: TreeLod) {
    let seed = sc.style.as_ref().map_or(0, |s| s.seed);
    belt_tree_list(sc, land, f.min, f.max(), |p, kind, h, phase, rng| {
        if lod != TreeLod::Distant || distant_keeps(seed, p) {
            tree_at(g, rng, kind, f.at(p, 0.0), h, phase, lod);
        }
    });
}

/// The shelterbelt trees standing in [`min`, `max`): where, their kind, height and turn, and
/// the random numbers to draw them with.
fn belt_tree_list(
    sc: &Scenery,
    land: &Farmland,
    min: DVec2,
    max: DVec2,
    mut visit: impl FnMut(DVec2, Tree, f32, f32, &mut Rng),
) {
    let Some(style) = &sc.style else { return };
    let inside = |p: DVec2| p.x >= min.x && p.y >= min.y && p.x < max.x && p.y < max.y;
    let m = DVec2::splat(land.section);
    for (sx, sy) in fields::sections(land, min - m, max) {
        if section_land(style, land, sx, sy) == Land::Woods {
            continue;
        }
        for (a, b) in fields::belts(land, style.seed, sx, sy) {
            // Only the part of the belt in the box.
            let (lo, hi) = (a.max(min - 10.0), b.min(max + 10.0));
            if lo.x > hi.x || lo.y > hi.y || belt_rect(sc, land, a, b).is_none() {
                continue;
            }
            let dir = (b - a).normalize();
            let side = dir.perp();
            let (s0, s1) = ((lo - a).dot(dir), (hi - a).dot(dir));
            let mut k = (s0 / 8.0).ceil();
            while k * 8.0 < s1 {
                for row in [-1.0, 1.0] {
                    let mut rng = Rng::new(hash(
                        style.seed,
                        (a.x + k * 8.0 * dir.x) as i64,
                        (a.y + k * 8.0 * dir.y) as i64,
                        row as i64 + 20,
                    ));
                    let p = a
                        + dir * (k * 8.0 + row * 2.0 + rng.range(-1.5, 1.5))
                        + side * row * rng.range(2.0, 4.5);
                    if !inside(p) || rng.chance(0.08) || !sc.net.road_clear(p, 4.0) || sc.cleared(p)
                    {
                        continue;
                    }
                    let kind = if rng.chance(0.9) { Tree::Broadleaf } else { Tree::Conifer };
                    let (h, phase) = (rng.range(9.0, 20.0) as f32, rng.range(0.0, 6.3) as f32);
                    visit(p, kind, h, phase, &mut rng);
                }
                k += 1.0;
            }
        }
    }
}

/// The trees within `r` metres of `p` (and their crowns' reach): where, the crown's radius and
/// the height (metres). Not the bushes.
pub(super) fn trees_near(sc: &Scenery, p: DVec2, r: f64, mut visit: impl FnMut(DVec2, f32, f32)) {
    let Some(style) = &sc.style else { return };
    let r = r + 8.0;
    let (lo, hi) = ((p - r) / TREE_GRID, (p + r) / TREE_GRID);
    for gy in lo.y.floor() as i64..=hi.y.floor() as i64 {
        for gx in lo.x.floor() as i64..=hi.x.floor() as i64 {
            if let Some((q, Grows::Tree(kind, h, _), _)) = grid_tree(sc, style, gx, gy) {
                visit(q, crown(kind, h), h);
            }
        }
    }
    if let Some(land) = style.farmland {
        belt_tree_list(sc, land, p - r, p + r, |q, kind, h, _, _| visit(q, crown(kind, h), h));
    }
}

/// Radius of a tree's crown (`tree_at`'s near shape).
fn crown(kind: Tree, h: f32) -> f32 {
    h * match kind {
        Tree::Conifer => 0.27,
        Tree::Broadleaf => 0.33,
        Tree::Palm => 0.38,
    }
}

/// One tree of height `h` at ground point `c`, turned by `phase`.
fn tree_at(g: &mut Geom, rng: &mut Rng, kind: Tree, c: Vec3, h: f32, phase: f32, lod: TreeLod) {
    let far = lod != TreeLod::Near;
    // The distant tier's fewer trees cover as much ground.
    let w = if lod == TreeLod::Distant { (1.0 / DISTANT_TREES).sqrt() as f32 } else { 1.0 };
    match kind {
        Tree::Conifer if far => {
            let col = rng.pick(&[0xAD, 0xAE]);
            g.spire(col, &ring(c, h * 0.26 * w, 3, phase), h * 0.08, c + Vec3::Y * h);
        }
        Tree::Broadleaf if far => {
            let col = rng.pick(&[0xAB, 0xAC, 0xAD]);
            let crown = ring(c, h * 0.33 * w, 3, phase);
            g.spire(col, &crown, h * 0.58, c + Vec3::Y * h);
            g.spire(col, &crown, h * 0.58, c + Vec3::Y * h * 0.28);
        }
        Tree::Palm if far => {
            let crown = ring(c, h * 0.3 * w, 3, phase);
            g.spire(0xA8, &crown, h * 0.9, c + Vec3::Y * h * 1.02);
            g.spire(0xA8, &crown, h * 0.9, c + Vec3::Y * h * 0.7);
        }
        Tree::Conifer => {
            let col = rng.pick(&[0xAD, 0xAE]);
            g.spire(col, &ring(c, h * 0.27, 5, phase), h * 0.08, c + Vec3::Y * h * 0.72);
            g.spire(col, &ring(c, h * 0.19, 5, phase + 0.6), h * 0.45, c + Vec3::Y * h);
        }
        Tree::Broadleaf => {
            let col = rng.pick(&[0xAB, 0xAC, 0xAD]);
            let crown = ring(c, h * 0.33, 6, phase);
            let mid = h * 0.58;
            g.walls(0xCD, &ring(c, 0.35, 3, phase), 0.0, h * 0.35);
            g.spire(col, &crown, mid, c + Vec3::Y * h);
            g.spire(col, &crown, mid, c + Vec3::Y * h * 0.28);
        }
        Tree::Palm => {
            let lean =
                Vec3::new(rng.range(-1.0, 1.0) as f32, 0.0, rng.range(-1.0, 1.0) as f32) * h * 0.08;
            let top = c + lean + Vec3::Y * h;
            let base = ring(c, 0.35, 3, phase);
            for k in 0..3 {
                let (a, b) = (base[k], base[(k + 1) % 3]);
                let out = (a + b) * 0.5 - c;
                g.face(0xCC, &[a, b, top + (b - c) * 0.5, top + (a - c) * 0.5], out);
            }
            for k in 0..6 {
                let a = phase + k as f32 * TAU / 6.0;
                let d = Vec3::new(a.cos(), 0.0, a.sin());
                let tip = top + d * h * 0.38 - Vec3::Y * h * 0.18;
                let side = Vec3::new(-d.z, 0.0, d.x) * h * 0.06;
                let mid = top + d * h * 0.2 + Vec3::Y * 0.4;
                // Fronds are seen from above and below.
                g.face(0xA8, &[top, mid + side, tip], Vec3::Y);
                g.face(0xA8, &[top, mid + side, tip], -Vec3::Y);
                g.face(0xA8, &[top, tip, mid - side], Vec3::Y);
                g.face(0xA8, &[top, tip, mid - side], -Vec3::Y);
            }
        }
    }
}

//! What an aircraft can fly into among the generated scenery: the buildings (their footprint
//! up to the roof ridge), the trees (the crown as an upright capsule), the power line pylons
//! and the wires sagging between them. The aircraft is a sphere; ground metres (x east, y
//! north), heights above the ground, which is flat wherever scenery stands.

use bevy::math::{DVec2, DVec3};

use super::Scenery;
use super::content;
use super::network::{Building, Kind, Roof, dist_to_segment};

/// What was hit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Obstacle {
    Building,
    Tree,
    Pylon,
    Wire,
}

/// Longest span between two pylons (`Network::pylon_route`), metres.
const MAX_SPAN: f64 = 720.0;
/// Thickness counted for a wire and a pylon's members (metres).
const WIRE: f64 = 0.4;
const MEMBER: f64 = 0.8;

/// Distance from `p` to the segment `a`-`b`.
fn to_segment(p: DVec3, a: DVec3, b: DVec3) -> f64 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// A sphere at `p` (height `h`, radius `r`) against an upright prism from the ground to `top`
/// over a footprint, the footprint's horizontal distance from `p` given.
fn prism(dist: f64, h: f64, r: f64, top: f64) -> bool {
    h - r < top && dist < r
}

/// Horizontal distance from `p` to a `l` x `w` rectangle centred on `c` with its length along
/// `dir`.
fn rect_dist(p: DVec2, c: DVec2, dir: DVec2, l: f64, w: f64) -> f64 {
    let d = p - c;
    let (u, v) = (d.dot(dir).abs() - l * 0.5, d.dot(dir.perp()).abs() - w * 0.5);
    DVec2::new(u.max(0.0), v.max(0.0)).length()
}

/// The building's solid parts hit by the sphere.
fn building_hit(b: &Building, p: DVec2, h: f64, r: f64) -> bool {
    let (l, w, bh) = (b.l as f64, b.w as f64, b.h as f64);
    let round = |radius: f64, top: f64| prism((p.distance(b.p) - radius).max(0.0), h, r, top);
    let boxed = |top: f64| prism(rect_dist(p, b.p, b.dir, l, w), h, r, top);
    match b.kind {
        Kind::House | Kind::Apartment | Kind::Shed | Kind::Barn | Kind::Church | Kind::Tower => {
            let rise = match (b.roof, b.kind) {
                (Roof::Flat, _) => 0.0,
                (_, Kind::Shed) => w * 0.18,
                (_, Kind::Church) => w * 0.6,
                _ => w * 0.42,
            };
            // The church's tower and spire.
            let tower = b.kind == Kind::Church && {
                let t = b.p - b.dir * (l * 0.5 + 3.2);
                prism(rect_dist(p, t, b.dir, 6.5, 6.5), h, r, bh * 2.4 + 11.0)
            };
            tower || boxed(bh + rise)
        }
        Kind::Ruin | Kind::Shelter => boxed(bh),
        Kind::Silo => round(l * 0.5, bh + 2.5),
        Kind::Chimney => round(2.4, bh),
        Kind::Tank => round(l * 0.5, bh),
        Kind::WaterTower => round(4.5, bh + 2.0),
        Kind::Mast => round(1.2, bh),
        Kind::Radar => round(5.6, bh),
        // Too small to matter.
        Kind::Windsock | Kind::Sign => false,
    }
}

/// The top of the building's solid parts within `r` of `p` (metres up), None if none reach.
fn building_top(b: &Building, p: DVec2, r: f64) -> Option<f64> {
    let (l, w, bh) = (b.l as f64, b.w as f64, b.h as f64);
    let round = |radius: f64, top: f64| (p.distance(b.p) - radius < r).then_some(top);
    let boxed = |top: f64| (rect_dist(p, b.p, b.dir, l, w) < r).then_some(top);
    match b.kind {
        Kind::House | Kind::Apartment | Kind::Shed | Kind::Barn | Kind::Church | Kind::Tower => {
            let rise = match (b.roof, b.kind) {
                (Roof::Flat, _) => 0.0,
                (_, Kind::Shed) => w * 0.18,
                (_, Kind::Church) => w * 0.6,
                _ => w * 0.42,
            };
            let tower = (b.kind == Kind::Church)
                .then(|| {
                    let t = b.p - b.dir * (l * 0.5 + 3.2);
                    (rect_dist(p, t, b.dir, 6.5, 6.5) < r).then_some(bh * 2.4 + 11.0)
                })
                .flatten();
            match (tower, boxed(bh + rise)) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            }
        }
        Kind::Ruin | Kind::Shelter => boxed(bh),
        Kind::Silo => round(l * 0.5, bh + 2.5),
        Kind::Chimney => round(2.4, bh),
        Kind::Tank => round(l * 0.5, bh),
        Kind::WaterTower => round(4.5, bh + 2.0),
        Kind::Mast => round(1.2, bh),
        Kind::Radar => round(5.6, bh),
        Kind::Windsock | Kind::Sign => None,
    }
}

impl Scenery {
    /// The highest generated obstacle within `r` metres of ground point `p` (metres up, 0 for
    /// none), for the terrain following radar: the power lines' wires counted at the pylons'
    /// height, the trees only within `tree_r` (they are many and low).
    pub fn top_near(&self, p: DVec2, r: f64, tree_r: f64) -> f64 {
        let near = DVec2::splat(r + 80.0);
        let buildings = self
            .net
            .building_index
            .around(p - near, p + near)
            .map(|i| &self.net.buildings[i as usize])
            .filter(|b| !self.cleared(b.p));
        let airfields = self
            .airfields
            .iter()
            .filter(|a| a.runway.c.distance(p) < a.runway.half_len + 3000.0 + r)
            .flat_map(|a| a.buildings.iter());
        let mut top =
            buildings.chain(airfields).filter_map(|b| building_top(b, p, r)).fold(0.0, f64::max);
        content::trees_near(self, p, tree_r, |q, crown, h| {
            if q.distance(p) < tree_r + crown as f64 * 0.7 {
                top = top.max(h as f64);
            }
        });
        let span = DVec2::splat(MAX_SPAN + r);
        for id in self.net.pylon_index.around(p - span, p + span) {
            let (li, k) = self.net.pylons[id as usize];
            let line = &self.net.lines[li as usize];
            let k = k as usize;
            let height = if line.big { 31.0 } else { 9.0 };
            let c = line.pylons[k];
            let next = line.pylons.get(k + 1).copied().unwrap_or(c);
            if dist_to_segment(p, c, next) < r + 12.0 {
                top = top.max(height);
            }
        }
        top
    }

    /// What a sphere of radius `r` at ground point `p`, `h` metres up, is inside of.
    pub fn obstacle(&self, p: DVec2, h: f64, r: f64) -> Option<Obstacle> {
        // Nothing generated stands taller than the masts.
        if h - r > 320.0 {
            return None;
        }
        let near = DVec2::splat(r + 80.0);
        let buildings = self
            .net
            .building_index
            .around(p - near, p + near)
            .map(|i| &self.net.buildings[i as usize])
            .filter(|b| !self.cleared(b.p));
        let airfields = self
            .airfields
            .iter()
            .filter(|a| a.runway.c.distance(p) < a.runway.half_len + 3000.0)
            .flat_map(|a| a.buildings.iter());
        if buildings.chain(airfields).any(|b| building_hit(b, p, h, r)) {
            return Some(Obstacle::Building);
        }
        if h - r < 20.0 {
            let mut tree = false;
            content::trees_near(self, p, r, |q, crown, top| {
                // The crown as a capsule round the trunk, its tip and lowest leaves spared.
                let (crown, top) = (crown as f64 * 0.7, top as f64);
                let a = DVec3::new(q.x, q.y, top * 0.3);
                let b = DVec3::new(q.x, q.y, top * 0.85);
                tree |= to_segment(DVec3::new(p.x, p.y, h), a, b) < crown + r;
            });
            if tree {
                return Some(Obstacle::Tree);
            }
        }
        self.power_hit(p, h, r)
    }

    /// The pylons and wires of the power lines.
    fn power_hit(&self, p: DVec2, h: f64, r: f64) -> Option<Obstacle> {
        if h - r > 45.0 {
            return None;
        }
        let at = DVec3::new(p.x, p.y, h);
        let up = |q: DVec2, z: f64| DVec3::new(q.x, q.y, z);
        let span = DVec2::splat(MAX_SPAN);
        let mut ids: Vec<u32> = self.net.pylon_index.around(p - span, p + span).collect();
        ids.sort_unstable();
        ids.dedup();
        for id in ids {
            let (li, k) = self.net.pylons[id as usize];
            let line = &self.net.lines[li as usize];
            let k = k as usize;
            let c = line.pylons[k];
            let across = content::line_dir(line, k).perp();
            // The pylon: a lattice tower narrowing from its foot to a member at the top, or a
            // pole; and the cross-arms.
            if c.distance(p) < r + 12.0 {
                let (foot, top, arms): (f64, f64, &[(f64, f64)]) = if line.big {
                    (4.0, 31.0, &[(23.0, 9.0), (29.0, 6.0)])
                } else {
                    (MEMBER, 9.0, &[(8.6, 1.3)])
                };
                let reach = foot + (MEMBER - foot) * (h / top).clamp(0.0, 1.0);
                let mast = to_segment(at, up(c, 0.0), up(c, top)) < reach + r;
                let arm = arms.iter().any(|&(z, half)| {
                    to_segment(at, up(c - across * half, z), up(c + across * half, z)) < MEMBER + r
                });
                if mast || arm {
                    return Some(Obstacle::Pylon);
                }
            }
            // The wires on to the next pylon.
            let Some(&q) = line.pylons.get(k + 1) else { continue };
            if dist_to_segment(p, c, q) > r + 12.0 {
                continue;
            }
            let d = q - c;
            let t = ((p - c).dot(d) / d.length_squared().max(1e-9)).clamp(0.0, 1.0);
            let sag = content::sag(line.big, d.length() as f32) as f64 * 4.0 * t * (1.0 - t);
            let across_q = content::line_dir(line, k + 1).perp();
            let ends = |o: DVec2, across: DVec2| {
                let v = content::dir3(across);
                content::conductors(line.big, v)
                    .into_iter()
                    .map(move |e| up(o, 0.0) + DVec3::new(e.x as f64, -e.z as f64, e.y as f64))
            };
            let wire = ends(c, across)
                .zip(ends(q, across_q))
                .any(|(a, b)| (a.lerp(b, t) - DVec3::Z * sag).distance(at) < WIRE + r);
            if wire {
                return Some(Obstacle::Wire);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::DVec2;

    use super::super::network::{Building, Kind, Network, PowerLine, Roof};
    use super::super::{Scenery, ground::Ground, style::Style};
    use super::Obstacle;

    fn scenery(style: Option<Style>, net: Network) -> Scenery {
        Scenery {
            style,
            ground: Ground::default().into(),
            net: net.into(),
            airfields: vec![].into(),
            clearings: vec![],
        }
    }

    #[test]
    fn houses_and_power_lines_are_solid() {
        let mut net = Network::default();
        let house = Building {
            p: DVec2::new(1000.0, 1000.0),
            dir: DVec2::X,
            kind: Kind::House,
            l: 12.0,
            w: 8.0,
            h: 6.0,
            roof: Roof::Gable,
            wall: 0,
            roof_colour: 0,
            seed: 0,
        };
        net.building_index.insert(house.p - 10.0, house.p + 10.0, 0);
        net.buildings.push(house);
        let line = PowerLine {
            pylons: vec![DVec2::new(2000.0, 0.0), DVec2::new(2000.0, 400.0)],
            big: true,
        };
        for (k, &p) in line.pylons.iter().enumerate() {
            net.pylon_index.insert(p, p, k as u32);
            net.pylons.push((0, k as u32));
        }
        net.lines.push(line);
        let sc = scenery(None, net);
        let hit = |x: f64, y: f64, h: f64| sc.obstacle(DVec2::new(x, y), h, 2.0);
        // Through the house's walls and roof (ridge 6 + 3.4 m), not over it or past its end.
        assert_eq!(hit(1000.0, 1000.0, 5.0), Some(Obstacle::Building));
        assert_eq!(hit(1007.0, 1000.0, 9.0), Some(Obstacle::Building));
        assert_eq!(hit(1000.0, 1000.0, 12.0), None);
        assert_eq!(hit(1009.0, 1000.0, 3.0), None);
        // The tower, its cross-arm tips and the wires; under the wires and above them is clear.
        assert_eq!(hit(2000.0, 0.0, 15.0), Some(Obstacle::Pylon));
        assert_eq!(hit(2008.5, 0.0, 23.0), Some(Obstacle::Pylon));
        assert_eq!(hit(2008.5, 200.0, 23.0 - 10.0), Some(Obstacle::Wire));
        assert_eq!(hit(1994.5, 120.0, 29.0 - 8.4), Some(Obstacle::Wire));
        assert_eq!(hit(2000.0, 200.0, 5.0), None);
        assert_eq!(hit(2000.0, 200.0, 40.0), None);
        assert_eq!(hit(2030.0, 200.0, 15.0), None);
    }

    #[test]
    fn trees_are_solid_up_to_their_crowns() {
        let sc = scenery(Style::for_theater("central_europe", 3), Network::default());
        let mut trees = vec![];
        super::content::trees_near(&sc, DVec2::new(50_000.0, 50_000.0), 400.0, |q, crown, h| {
            trees.push((q, crown as f64, h as f64))
        });
        assert!(!trees.is_empty(), "no trees");
        for (q, crown, h) in trees {
            assert_eq!(sc.obstacle(q, h * 0.6, 1.0), Some(Obstacle::Tree), "{q} {h}");
            assert_eq!(sc.obstacle(q + DVec2::X * crown * 0.5, h * 0.6, 1.0), Some(Obstacle::Tree));
            // Over the tallest trees (neighbours may stand a few metres away).
            assert_eq!(sc.obstacle(q, 22.0, 1.0), None);
        }
    }
}

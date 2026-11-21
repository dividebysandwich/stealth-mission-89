//! A theater's supply roads (remake theaters that have them: Ukraine), as the theater builder
//! writes them (`tools/theater/assemble.py` `roads`): named junctions joined by roads, each a
//! line of points in metres (x east, y north) with the height of a bridge deck over the sea
//! (0 on land). The enemy's supply trucks drive them; the scenery draws them as highways.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadNode {
    pub name: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadEdge {
    /// The junctions it joins (indices into `RoadNet::nodes`); `pts` runs from `a` to `b`.
    pub a: usize,
    pub b: usize,
    pub name: String,
    /// Over a bridge (its own model carries the road; `z` is the deck's height).
    #[serde(default)]
    pub bridge: bool,
    pub pts: Vec<(f64, f64, f64)>,
}

impl RoadEdge {
    pub fn length(&self) -> f64 {
        self.pts.windows(2).map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1)).sum()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RoadNet {
    pub nodes: Vec<RoadNode>,
    pub edges: Vec<RoadEdge>,
    /// Where the trucks come onto the roads (junctions).
    #[serde(default)]
    pub entries: Vec<usize>,
}

impl RoadNet {
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn node(&self, name: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.name == name)
    }

    /// The junction nearest a point (metres), with its distance.
    pub fn nearest_node(&self, x: f64, y: f64) -> Option<(usize, f64)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (i, (n.x - x).hypot(n.y - y)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// The shortest way from junction `from` to `to` over the edges `open` lets through: the
    /// edges in order with the direction each is driven in (true: from `b` to `a`).
    pub fn path(
        &self,
        from: usize,
        to: usize,
        open: impl Fn(usize, &RoadEdge) -> bool,
    ) -> Option<Vec<(usize, bool)>> {
        let n = self.nodes.len();
        if from >= n || to >= n {
            return None;
        }
        let mut dist = vec![f64::INFINITY; n];
        let mut prev: Vec<Option<(usize, bool)>> = vec![None; n];
        let mut done = vec![false; n];
        dist[from] = 0.0;
        loop {
            let Some(u) = (0..n)
                .filter(|&i| !done[i] && dist[i].is_finite())
                .min_by(|&a, &b| dist[a].total_cmp(&dist[b]))
            else {
                break;
            };
            if u == to {
                break;
            }
            done[u] = true;
            for (k, e) in self.edges.iter().enumerate() {
                if !open(k, e) {
                    continue;
                }
                let (v, rev) = if e.a == u {
                    (e.b, false)
                } else if e.b == u {
                    (e.a, true)
                } else {
                    continue;
                };
                let d = dist[u] + e.length();
                if d < dist[v] {
                    dist[v] = d;
                    prev[v] = Some((k, rev));
                }
            }
        }
        if !dist[to].is_finite() {
            return None;
        }
        let mut out = Vec::new();
        let mut at = to;
        while at != from {
            let (k, rev) = prev[at]?;
            out.push((k, rev));
            // Driven from `b` to `a` when reversed: it was entered at `b`.
            at = if rev { self.edges[k].b } else { self.edges[k].a };
            if out.len() > self.edges.len() {
                return None;
            }
        }
        out.reverse();
        Some(out)
    }

    /// The points of a path (from [`RoadNet::path`]), in driving order.
    pub fn points(&self, path: &[(usize, bool)]) -> Vec<(f64, f64, f64)> {
        let mut out: Vec<(f64, f64, f64)> = Vec::new();
        for &(k, rev) in path {
            let pts = &self.edges[k].pts;
            let mut add = |p: (f64, f64, f64)| {
                if out.last().is_none_or(|q| (q.0 - p.0).hypot(q.1 - p.1) > 0.5) {
                    out.push(p);
                }
            };
            if rev {
                pts.iter().rev().for_each(|&p| add(p));
            } else {
                pts.iter().for_each(|&p| add(p));
            }
        }
        out
    }

    /// The deck's height over the sea (metres) at a point on a bridge's road (within `reach`
    /// metres of it), else None.
    pub fn deck_height(&self, x: f64, y: f64, reach: f64) -> Option<f64> {
        let mut best: Option<(f64, f64)> = None;
        for e in self.edges.iter().filter(|e| e.bridge) {
            for w in e.pts.windows(2) {
                let (a, b) = (w[0], w[1]);
                let (dx, dy) = (b.0 - a.0, b.1 - a.1);
                let l2 = dx * dx + dy * dy;
                if l2 <= 0.0 {
                    continue;
                }
                let t = (((x - a.0) * dx + (y - a.1) * dy) / l2).clamp(0.0, 1.0);
                let d = (x - a.0 - dx * t).hypot(y - a.1 - dy * t);
                if d <= reach && best.is_none_or(|(bd, _)| d < bd) {
                    best = Some((d, a.2 + (b.2 - a.2) * t));
                }
            }
        }
        best.map(|(_, z)| z)
    }

    /// The edges on land as lines of points (metres east, north), for the scenery's highways.
    pub fn land_lines(&self) -> impl Iterator<Item = Vec<(f64, f64)>> + '_ {
        self.edges.iter().filter(|e| !e.bridge).map(|e| e.pts.iter().map(|p| (p.0, p.1)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn net() -> RoadNet {
        let node = |name: &str, x, y| RoadNode { name: name.into(), x, y };
        let edge = |a, b, bridge, pts: &[(f64, f64)]| RoadEdge {
            a,
            b,
            name: String::new(),
            bridge,
            pts: pts.iter().map(|&(x, y)| (x, y, if bridge { 30.0 } else { 0.0 })).collect(),
        };
        RoadNet {
            nodes: vec![node("A", 0.0, 0.0), node("B", 1000.0, 0.0), node("C", 1000.0, 1000.0)],
            edges: vec![
                edge(0, 1, true, &[(0.0, 0.0), (1000.0, 0.0)]),
                edge(1, 2, false, &[(1000.0, 0.0), (1000.0, 1000.0)]),
                edge(0, 2, false, &[(0.0, 0.0), (0.0, 3000.0), (1000.0, 1000.0)]),
            ],
            entries: vec![0],
        }
    }

    #[test]
    fn the_shortest_open_way_is_taken() {
        let n = net();
        assert_eq!(n.path(0, 2, |_, _| true), Some(vec![(0, false), (1, false)]));
        assert_eq!(n.path(2, 0, |_, _| true), Some(vec![(1, true), (0, true)]));
        assert_eq!(n.path(0, 2, |_, e| !e.bridge), Some(vec![(2, false)]));
        assert_eq!(n.path(0, 1, |k, _| k == 1), None);
        let pts = n.points(&n.path(0, 2, |_, _| true).unwrap());
        assert_eq!(pts.len(), 3);
    }

    #[test]
    fn a_bridge_has_a_deck() {
        let n = net();
        assert_eq!(n.deck_height(500.0, 5.0, 25.0), Some(30.0));
        assert_eq!(n.deck_height(500.0, 50.0, 25.0), None);
    }
}

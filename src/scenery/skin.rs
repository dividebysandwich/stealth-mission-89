//! The rolling hills' ground (`stealth_sim::hills`): the flat world stays where it is, and
//! where the hills rise a mesh of flat triangles on a grid (finer near the viewpoint) is drawn
//! over it, in chunks. The theater's flat ground polygons (fields, woods, concrete) are cut to
//! the grid's triangles and laid on them in their painter's order, so the hills show what the
//! flat ground below them does. Built on other threads; a chunk whose grid changes keeps its
//! old mesh until the new one is there.

use std::collections::HashMap;

use bevy::light::NotShadowCaster;
use bevy::math::{DVec2, DVec3};
use bevy::mesh::{MeshTag, PrimitiveTopology};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use stealth_sim::hills::GRID_M;

use super::content::{self, FIELD_BLOCK, Tier};
use super::{DetailLevel, Scenery, budget, mesh};
use crate::palette::Palette;
use crate::palette_material::{
    ATTRIBUTE_PALETTE_WORD, TAG_FLAT_LIT, TAG_HILLS, TAG_VERTEX_HAZE, mesh_tag,
};
use crate::world::{LAYER_STEP_M, WU, WorldAnchor, WorldPos, bevy_to_wu};

/// Chunk side (metres).
pub const CHUNK: f64 = 4000.0;
/// Grid spacing (metres) by distance of the chunk's nearest point from the viewpoint, out to
/// the last distance (the hills further out are not drawn).
const STEPS: [(f64, f64); 4] =
    [(3000.0, GRID_M), (8000.0, GRID_M * 2.0), (20_000.0, GRID_M * 4.0), (40_000.0, GRID_M * 8.0)];
/// Lift of each ground polygon above the one below it (metres).
const LIFT: f32 = 0.05;
/// Depth of the skirts round a chunk, hiding the cracks to a neighbour on another grid: at
/// least this, and this share of the grid's spacing (coarser grids cut the hilltops lower).
const SKIRT: f32 = 6.0;
const SKIRT_PER_STEP: f32 = 0.4;
/// Chunks being built at once.
const BUILDING: usize = 4;
/// Size of the buckets the polygons are sorted into (metres).
const BUCKET: f64 = 250.0;

/// What a chunk is built as: its grid level, and whether with the farmland's fields.
type Kind = (usize, bool);

/// A chunk's mesh being built.
type Building = (Kind, Task<Option<ChunkMesh>>);

/// Chunk -> what it is built as and its entity; chunks being built.
#[derive(Resource, Default)]
pub struct Skin {
    chunks: HashMap<(i32, i32), (Kind, Entity)>,
    building: HashMap<(i32, i32), Building>,
}

/// A chunk's triangles (chunk-local Bevy positions) and their model colours (`GROUND`: the
/// ground gradient), and its skirts'.
pub struct ChunkMesh {
    pos: Vec<Vec3>,
    colours: Vec<u16>,
    skirt: Vec<Vec3>,
    skirt_colours: Vec<u16>,
}

/// The colour of the open ground: the ground gradient the flat world shows there (the
/// background's).
const GROUND: u16 = 0x100;
/// Its colour word: OR with nothing.
const GROUND_WORD: u32 = 0xFD00;

/// The model colours of a chunk's vertices, for their words to follow the palette's remap.
#[derive(Component)]
pub(super) struct SkinColours(Vec<u16>);

fn words(palette: &Palette, colours: &[u16]) -> Vec<u32> {
    colours
        .iter()
        .map(|&c| if c == GROUND { GROUND_WORD } else { palette.word(c as u8) as u32 })
        .collect()
}

/// The chunks' colour words after the palette's remap changed.
pub(super) fn remap_words(
    palette: Res<Palette>,
    mut remap: Local<Option<usize>>,
    chunks: Query<(&Mesh3d, &SkinColours)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if remap.replace(palette.remap).is_none_or(|r| r == palette.remap) {
        return;
    }
    for (m, colours) in &chunks {
        if let Some(mut mesh) = meshes.get_mut(&m.0) {
            mesh.insert_attribute(ATTRIBUTE_PALETTE_WORD, words(&palette, &colours.0));
        }
    }
}

/// The grid level for a chunk at `d` metres, None beyond the hills' range.
fn level(d: f64) -> Option<usize> {
    STEPS.iter().position(|&(r, _)| d < r)
}

fn chunk_min(bx: i32, by: i32) -> DVec2 {
    DVec2::new(bx as f64, by as f64) * CHUNK
}

#[allow(clippy::too_many_arguments)]
pub(super) fn stream_skin(
    mut commands: Commands,
    scenery: Res<Scenery>,
    palette: Res<Palette>,
    anchor: Query<&WorldPos, With<WorldAnchor>>,
    detail: Res<DetailLevel>,
    budget: Res<budget::SceneryBudget>,
    mut skin: ResMut<Skin>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let hills = &scenery.hills;
    if hills.is_flat() {
        return;
    }
    let Ok(anchor) = anchor.single() else { return };
    let (east, north, _) = bevy_to_wu(anchor.0);
    let eye = DVec2::new(east, north) * WU;
    let range = STEPS[STEPS.len() - 1].0;
    // The farmland as `stream_blocks` lays it (its fields under the hills are drawn here): in
    // the blocks whose centre is in its range (by the same measure, so the flat fields and
    // the hills' agree).
    let farmland = detail.0 >= 2 && scenery.style.as_ref().is_some_and(|s| s.farmland.is_some());
    let fields_r = budget.ranges().radius(Tier::Fields);
    debug_assert_eq!(CHUNK, FIELD_BLOCK);
    // Chunks wanted: what they are built as and their distance.
    let mut wanted: HashMap<(i32, i32), (Kind, f64)> = HashMap::new();
    let (lo, hi) =
        (((eye - range) / CHUNK).floor().as_ivec2(), ((eye + range) / CHUNK).floor().as_ivec2());
    for by in lo.y..=hi.y {
        for bx in lo.x..=hi.x {
            let min = chunk_min(bx, by);
            let d = eye.clamp(min, min + CHUNK).distance(eye);
            let c = min + CHUNK * 0.5;
            if let Some(l) = level(d)
                && hills.any_in(c.x, c.y, CHUNK * 0.5)
            {
                let fields = farmland && c.distance(eye) < fields_r + CHUNK * 0.5;
                wanted.insert((bx, by), ((l, fields), d));
            }
        }
    }
    // Finished builds replace their chunk's mesh.
    let done: Vec<(i32, i32)> =
        skin.building.iter_mut().filter_map(|(&k, (_, t))| t.is_finished().then_some(k)).collect();
    for key in done {
        let Some((kind, task)) = skin.building.remove(&key) else { continue };
        let Some(built) = block_on(poll_once(task)) else { continue };
        if let Some((_, old)) = skin.chunks.remove(&key) {
            commands.entity(old).try_despawn();
        }
        if wanted.get(&key).is_none_or(|w| w.0 != kind) {
            continue;
        }
        let Some(built) = built else {
            // Nothing to draw (flat after all): remember what it is, no mesh.
            let e = commands.spawn(SkinChunk).id();
            skin.chunks.insert(key, (kind, e));
            continue;
        };
        let m = mesh(PrimitiveTopology::TriangleList, built.pos)
            .with_inserted_attribute(ATTRIBUTE_PALETTE_WORD, words(&palette, &built.colours));
        let c = chunk_min(key.0, key.1) + CHUNK * 0.5;
        let e = commands
            .spawn((
                SkinChunk,
                Mesh3d(meshes.add(m)),
                MeshMaterial3d(palette.per_vertex.clone()),
                SkinColours(built.colours),
                MeshTag(mesh_tag(3, 4, false) | TAG_VERTEX_HAZE | TAG_HILLS),
                NotShadowCaster,
                WorldPos(DVec3::new(c.x, 0.0, -c.y)),
                Transform::default(),
                Visibility::default(),
            ))
            .id();
        // The skirts lit as the flat ground: they only show in the cracks, as ground.
        if !built.skirt.is_empty() {
            let m = mesh(PrimitiveTopology::TriangleList, built.skirt).with_inserted_attribute(
                ATTRIBUTE_PALETTE_WORD,
                words(&palette, &built.skirt_colours),
            );
            let skirt = commands
                .spawn((
                    Mesh3d(meshes.add(m)),
                    MeshMaterial3d(palette.per_vertex.clone()),
                    SkinColours(built.skirt_colours),
                    MeshTag(mesh_tag(3, 4, false) | TAG_VERTEX_HAZE | TAG_HILLS | TAG_FLAT_LIT),
                    NotShadowCaster,
                    Transform::default(),
                    Visibility::default(),
                ))
                .id();
            commands.entity(e).add_child(skirt);
        }
        skin.chunks.insert(key, (kind, e));
    }
    // Chunks out of range go; those on another grid stay until rebuilt.
    skin.chunks.retain(|k, (_, e)| {
        let keep = wanted.contains_key(k);
        if !keep {
            commands.entity(*e).try_despawn();
        }
        keep
    });
    skin.building.retain(|k, (kind, _)| wanted.get(k).is_some_and(|w| w.0 == *kind));
    // Start the nearest builds missing.
    let mut todo: Vec<_> = wanted
        .iter()
        .filter(|(k, (kind, _))| {
            skin.chunks.get(k).is_none_or(|c| c.0 != *kind) && !skin.building.contains_key(k)
        })
        .map(|(&k, &(kind, d))| (d, k, kind))
        .collect();
    todo.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, (bx, by), kind) in todo.into_iter().take(BUILDING.saturating_sub(skin.building.len())) {
        let sc = (*scenery).clone();
        let (l, fields) = kind;
        let task = AsyncComputeTaskPool::get()
            .spawn(async move { build(&sc, bx, by, STEPS[l].1, fields) });
        skin.building.insert((bx, by), (kind, task));
    }
}

/// A hills chunk entity.
#[derive(Component)]
pub struct SkinChunk;

pub(super) fn cleanup(mut commands: Commands, mut skin: ResMut<Skin>) {
    for (_, (_, e)) in skin.chunks.drain() {
        commands.entity(e).try_despawn();
    }
    skin.building.clear();
}

/// Twice the signed area of a polygon.
fn area2(p: &[Vec2]) -> f32 {
    (0..p.len()).map(|i| p[i].perp_dot(p[(i + 1) % p.len()])).sum()
}

/// A polygon cut to the inside of a triangle (counter-clockwise).
fn clip(poly: &[Vec2], tri: [Vec2; 3]) -> Vec<Vec2> {
    let mut out: Vec<Vec2> = poly.to_vec();
    for k in 0..3 {
        if out.is_empty() {
            break;
        }
        let (a, b) = (tri[k], tri[(k + 1) % 3]);
        let side = |p: Vec2| (b - a).perp_dot(p - a);
        let input = std::mem::take(&mut out);
        for i in 0..input.len() {
            let (p, q) = (input[i], input[(i + 1) % input.len()]);
            let (sp, sq) = (side(p), side(q));
            if sp >= 0.0 {
                out.push(p);
            }
            if (sp >= 0.0) != (sq >= 0.0) {
                out.push(p + (q - p) * (sp / (sp - sq)));
            }
        }
    }
    out
}

/// A flat ground polygon chunk-local (metres east, north of its corner), the height its flat
/// copy is drawn at (its painter's order) and its colour.
struct Flat {
    p: [Vec2; 3],
    lift: f32,
    colour: u8,
}

/// A chunk's mesh on a grid of `step` metres, with the farmland's fields if `fields` (None
/// where the hills stay flat).
fn build(sc: &Scenery, bx: i32, by: i32, step: f64, fields: bool) -> Option<ChunkMesh> {
    let hills = &sc.hills;
    let min = chunk_min(bx, by);
    let centre = min + CHUNK * 0.5;
    let n = (CHUNK / step).round() as usize;
    let heights: Vec<f32> = (0..(n + 1) * (n + 1))
        .map(|i| {
            let p = min + DVec2::new((i % (n + 1)) as f64, (i / (n + 1)) as f64) * step;
            hills.height_m(p.x, p.y)
        })
        .collect();
    if heights.iter().all(|&h| h <= 0.0) {
        return None;
    }
    let h = |i: usize, j: usize| heights[j * (n + 1) + i];
    // The ground polygons: the theater's (lifted like `world::stream_tiles` lifts them) and the
    // farmland's, in painter's order, in buckets.
    let mut flats: Vec<Flat> = sc
        .ground
        .overlays(min, min + CHUNK)
        .into_iter()
        .map(|o| {
            let (lod, k) = (4 - (o.order >> 28) as i32, (o.order >> 16) & 0xFFF);
            // Its height as drawn: its object's lift and its own in the model.
            let lift =
                ((4 - lod) as f64 * LAYER_STEP_M * 8.0 + k as f64 * LAYER_STEP_M) as f32 + o.y;
            Flat { p: o.p.map(|v| (v.as_dvec2() - min).as_vec2()), lift, colour: o.colour }
        })
        .collect();
    if fields {
        debug_assert_eq!(CHUNK, FIELD_BLOCK);
        // A field block's fields reach over its edges: the neighbours' too, those over this
        // chunk.
        let size = CHUNK as f32;
        let inside = |p: &[Vec2; 3]| {
            let (lo, hi) = (p[0].min(p[1]).min(p[2]), p[0].max(p[1]).max(p[2]));
            hi.x > 0.0 && hi.y > 0.0 && lo.x < size && lo.y < size
        };
        for (dx, dy) in (-1..=1).flat_map(|dx| (-1..=1).map(move |dy| (dx, dy))) {
            let g = content::build(sc, bx + dx, by + dy, Tier::Fields, 1.0);
            let off = Vec2::new(dx as f32 + 0.5, dy as f32 + 0.5) * size;
            for (&colour, tris) in &g.flat {
                for t in tris.as_chunks::<3>().0 {
                    let p = t.map(|v| Vec2::new(v.x, -v.z) + off);
                    if inside(&p) {
                        flats.push(Flat { p, lift: t[0].y, colour });
                    }
                }
            }
        }
    }
    flats.sort_by(|a, b| a.lift.total_cmp(&b.lift));
    let local: Vec<[Vec2; 3]> = flats.iter().map(|f| f.p).collect();
    let nb = (CHUNK / BUCKET).ceil() as usize;
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); nb * nb];
    let bucket = |v: f32| ((v as f64 / BUCKET).floor() as i64).clamp(0, nb as i64 - 1) as usize;
    for (i, t) in local.iter().enumerate() {
        let (lo, hi) = (t[0].min(t[1]).min(t[2]), t[0].max(t[1]).max(t[2]));
        for y in bucket(lo.y)..=bucket(hi.y) {
            for x in bucket(lo.x)..=bucket(hi.x) {
                buckets[y * nb + x].push(i as u32);
            }
        }
    }
    // Under every polygon: the open ground.
    let base = GROUND;
    let at = |p: Vec2, y: f32| {
        let w = min + p.as_dvec2();
        Vec3::new((w.x - centre.x) as f32, y, -(w.y - centre.y) as f32)
    };
    let mut out = ChunkMesh {
        pos: Vec::new(),
        colours: Vec::new(),
        skirt: Vec::new(),
        skirt_colours: Vec::new(),
    };
    let mut ids: Vec<u32> = Vec::new();
    let mut layers: Vec<(Vec<Vec2>, u16)> = Vec::new();
    let s = step as f32;
    for j in 0..n {
        for i in 0..n {
            let hs = [h(i, j), h(i + 1, j), h(i + 1, j + 1), h(i, j + 1)];
            if hs.iter().all(|&v| v <= 0.0) {
                continue;
            }
            let c = [(i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)]
                .map(|(x, y)| Vec2::new(x as f32 * s, y as f32 * s));
            ids.clear();
            let b = (bucket(c[0].x + 0.01), bucket(c[0].y + 0.01));
            let b1 = (bucket(c[2].x - 0.01), bucket(c[2].y - 0.01));
            for y in b.1..=b1.1 {
                for x in b.0..=b1.0 {
                    ids.extend(&buckets[y * nb + x]);
                }
            }
            ids.sort_unstable();
            ids.dedup();
            for (tri, th) in [
                ([c[0], c[1], c[2]], [hs[0], hs[1], hs[2]]),
                ([c[0], c[2], c[3]], [hs[0], hs[2], hs[3]]),
            ] {
                // The plane through the triangle's corners.
                let (e1, e2) = (tri[1] - tri[0], tri[2] - tri[0]);
                let det = e1.perp_dot(e2);
                let y = |p: Vec2| {
                    let d = p - tri[0];
                    let (u, v) = (d.perp_dot(e2) / det, e1.perp_dot(d) / det);
                    th[0] + (th[1] - th[0]) * u + (th[2] - th[0]) * v
                };
                let full = det.abs() * 0.999;
                // From the top polygon down to the first that covers the triangle.
                layers.clear();
                let mut covered = false;
                for &id in ids.iter().rev() {
                    let t = local[id as usize];
                    let (lo, hi) = (t[0].min(t[1]).min(t[2]), t[0].max(t[1]).max(t[2]));
                    let (tlo, thi) =
                        (tri[0].min(tri[1]).min(tri[2]), tri[0].max(tri[1]).max(tri[2]));
                    if hi.x <= tlo.x || hi.y <= tlo.y || lo.x >= thi.x || lo.y >= thi.y {
                        continue;
                    }
                    let mut p = clip(&t, tri);
                    let a = area2(&p);
                    if a.abs() < 1e-3 {
                        continue;
                    }
                    if a < 0.0 {
                        p.reverse();
                    }
                    covered = a.abs() >= full;
                    layers.push((p, flats[id as usize].colour as u16));
                    if covered {
                        break;
                    }
                }
                if !covered {
                    layers.push((tri.to_vec(), base));
                }
                for (rank, (p, colour)) in layers.iter().rev().enumerate() {
                    let lift = rank as f32 * LIFT;
                    for k in 1..p.len() - 1 {
                        for v in [p[0], p[k], p[k + 1]] {
                            out.pos.push(at(v, y(v) + lift));
                            out.colours.push(*colour);
                        }
                    }
                }
            }
        }
    }
    // Skirts along the chunk's edges, both faces.
    let edges = (0..n).flat_map(|k| {
        [((k, 0), (k + 1, 0)), ((n, k), (n, k + 1)), ((k + 1, n), (k, n)), ((0, k + 1), (0, k))]
    });
    for ((ai, aj), (bi, bj)) in edges {
        let (ha, hb) = (h(ai, aj), h(bi, bj));
        if ha <= 0.0 && hb <= 0.0 {
            continue;
        }
        let pa = Vec2::new(ai as f32 * s, aj as f32 * s);
        let pb = Vec2::new(bi as f32 * s, bj as f32 * s);
        let (a, b) = (at(pa, ha), at(pb, hb));
        let skirt = SKIRT.max(s * SKIRT_PER_STEP);
        let (a1, b1) = (at(pa, ha - skirt), at(pb, hb - skirt));
        for v in [a, a1, b1, a, b1, b, a, b1, a1, a, b, b1] {
            out.skirt.push(v);
            out.skirt_colours.push(base);
        }
    }
    (!out.pos.is_empty()).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips_to_the_triangle() {
        let tri = [Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(0.0, 10.0)];
        // A square over the triangle's corner: a quarter-ish of it inside.
        let sq = [
            Vec2::new(-5.0, -5.0),
            Vec2::new(5.0, -5.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(-5.0, 5.0),
        ];
        let p = clip(&sq, tri);
        assert!((area2(&p) * 0.5 - 25.0).abs() < 1e-3, "{}", area2(&p));
        // A big triangle covers it all.
        let big = [Vec2::new(-100.0, -100.0), Vec2::new(100.0, -100.0), Vec2::new(0.0, 100.0)];
        assert!((area2(&clip(&big, tri)) - area2(&tri)).abs() < 1e-3);
        // One outside: nothing.
        let away = [Vec2::new(20.0, 20.0), Vec2::new(30.0, 20.0), Vec2::new(20.0, 30.0)];
        assert!(clip(&away, tri).len() < 3);
    }
}

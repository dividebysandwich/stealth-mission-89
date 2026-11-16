//! The mesh the waves rise in round the viewpoint: `LEVELS` nested square rings of grid cells,
//! each twice as coarse as the one inside it (a geometry clipmap). A vertex carries its cell
//! coordinates (x, z) and its level (y); the vertex shader (palette.wgsl `ocean_vertex`) puts
//! each level round its own centre (`level_centres`: the viewpoint snapped to twice the
//! level's cell, so a vertex keeps its place on the waves while the viewpoint moves), morphs
//! the vertices near a level's edge onto the coarser level's grid (no cracks between levels)
//! and displaces them by the waves. Where a level overlaps the finer one inside it, its pixels
//! are dropped.

use bevy::asset::RenderAssetUsages;
use bevy::math::DVec2;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// The finest level's cell (metres).
pub const CELL0: f64 = 0.5;
/// Levels: the outermost reaches `HALF * CELL0 * 2^(LEVELS-1)` = 2048 m from its centre.
pub const LEVELS: usize = 8;
/// Cells from a level's centre to its edge.
pub const HALF: i32 = 32;
/// Cells from a level's centre to the edge of its hole (inside the finer level, which may sit
/// up to one and a half of this level's cells off this level's centre).
pub const HOLE: i32 = 14;

/// A level's cell (metres).
pub fn cell(level: usize) -> f64 {
    CELL0 * (1u32 << level) as f64
}

/// The outermost level's reach from its centre (metres).
#[cfg(test)]
pub fn reach() -> f64 {
    HALF as f64 * cell(LEVELS - 1)
}

/// Each level's centre round the viewpoint (absolute metres, Bevy x and z): snapped to twice
/// the level's cell.
pub fn level_centres(eye: DVec2) -> [DVec2; LEVELS] {
    std::array::from_fn(|l| {
        let step = 2.0 * cell(l);
        (eye / step).round() * step
    })
}

/// The grid mesh (positions: cell x, level, cell z).
pub fn mesh() -> Mesh {
    let side = (2 * HALF + 1) as u32;
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for level in 0..LEVELS {
        let base = positions.len() as u32;
        for j in -HALF..=HALF {
            for i in -HALF..=HALF {
                positions.push([i as f32, level as f32, j as f32]);
            }
        }
        for j in -HALF..HALF {
            for i in -HALF..HALF {
                let hole = level > 0 && (-HOLE..HOLE).contains(&i) && (-HOLE..HOLE).contains(&j);
                if hole {
                    continue;
                }
                let at = |i: i32, j: i32| base + (j + HALF) as u32 * side + (i + HALF) as u32;
                let (a, b, c, d) = (at(i, j), at(i + 1, j), at(i, j + 1), at(i + 1, j + 1));
                // Counter-clockwise seen from above (+y): x right, z towards the viewer.
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A level's hole lies inside the finer level's square wherever the viewpoint is.
    #[test]
    fn holes_lie_inside_the_finer_level() {
        for k in 0..2000 {
            let eye = DVec2::new(k as f64 * 3.71 - 3000.0, k as f64 * -2.93 + 1234.5);
            let c = level_centres(eye);
            for l in 1..LEVELS {
                let hole = HOLE as f64 * cell(l);
                let finer = HALF as f64 * cell(l - 1);
                let off = (c[l] - c[l - 1]).abs().max_element();
                assert!(off + hole <= finer + 1e-9, "level {l} at {eye}: {off} + {hole} > {finer}");
            }
            // The finest level is round the viewpoint.
            assert!((eye - c[0]).abs().max_element() <= cell(0) + 1e-9);
        }
    }

    #[test]
    fn mesh_has_every_level() {
        let m = mesh();
        let n = m.count_vertices();
        assert_eq!(n, LEVELS * 65 * 65);
        assert_eq!(reach(), 2048.0);
    }
}

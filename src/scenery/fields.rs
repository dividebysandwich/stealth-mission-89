//! Farmland: the open land is cut into square sections, each ploughed as one or more long
//! fields, with tracks between them and rows of trees (shelterbelts, hedgerows) along some
//! section edges. Everything follows from the section's hash, so any block can be built alone.
//! Fields only go where the ground is open land, clear of water, woods, concrete, raised tile
//! geometry and towns. A theater's `Farmland` sets the section size, the crops and the trees.

use bevy::math::DVec2;

use super::network::{Rng, hash};

/// Track between fields and margin along the section edges (metres).
const TRACK: f64 = 9.0;

/// A theater's farmland.
pub struct Farmland {
    /// Section size (metres).
    pub section: f64,
    /// Weights of one, two, ... fields in a section.
    pub strips: &'static [f64],
    /// Crop colours (model colour bytes, 0: fallow, the ground shows) with their weights.
    pub crops: &'static [(u8, f64)],
    /// Chance of a row of trees along a section edge, and its width (metres).
    pub belt_chance: f64,
    pub belt_width: f64,
}

/// The steppe in autumn: stubble and late wheat, fresh ploughland, dark harrowed soil, green
/// winter wheat, dried sunflowers waiting for the harvest; long fields in 1.4 km sections with
/// shelterbelts.
pub const STEPPE: Farmland = Farmland {
    section: 1400.0,
    strips: &[0.3, 0.45, 0.25],
    crops: &[
        (0xC3, 0.12), // stubble
        (0xC5, 0.07), // late wheat
        (0xCC, 0.12), // harrowed
        (0xCD, 0.15), // ploughed
        (0xCE, 0.08), // fresh ploughland, dark
        (0xA5, 0.12), // winter wheat
        (0xA7, 0.07), // winter rape
        (0xCF, 0.05), // dried sunflowers
        (0x00, 0.08), // fallow
    ],
    belt_chance: 0.4,
    belt_width: 14.0,
};

/// Central Europe in early September: strips of grain stubble and fresh ploughland between
/// maize, sugar beet, pasture and the first rape, in smaller sections with hedgerows.
pub const MIXED: Farmland = Farmland {
    section: 720.0,
    strips: &[0.12, 0.3, 0.33, 0.25],
    crops: &[
        (0xC3, 0.06), // grain stubble
        (0xA2, 0.12), // stubble going grey
        (0xEB, 0.10), // ploughed
        (0xE9, 0.05), // harrowed
        (0xAB, 0.16), // maize
        (0xA9, 0.12), // sugar beet
        (0xA7, 0.12), // pasture and hay meadow
        (0xA5, 0.07), // rape coming up
        (0x00, 0.20), // grassland, the ground shows
    ],
    belt_chance: 0.25,
    belt_width: 8.0,
};

/// Korea in early summer: small rice paddies, newly planted (the flooded ones still show
/// grey water) or green, with dry fields of maize, soy and vegetables between them.
pub const PADDY: Farmland = Farmland {
    section: 320.0,
    strips: &[0.1, 0.25, 0.35, 0.3],
    crops: &[
        (0xA4, 0.22), // young rice
        (0xA6, 0.18), // rice
        (0xF2, 0.12), // flooded, just planted
        (0xA8, 0.1),  // maize
        (0xA3, 0.1),  // soy
        (0xC2, 0.08), // barley stubble
        (0x00, 0.2),  // grass and dikes, the ground shows
    ],
    belt_chance: 0.08,
    belt_width: 6.0,
};

/// Vietnam's lowlands in the wet season: paddies at every stage, from flooded to ripening,
/// in small plots with palm and bamboo lines.
pub const TROPICAL_PADDY: Farmland = Farmland {
    section: 260.0,
    strips: &[0.1, 0.3, 0.35, 0.25],
    crops: &[
        (0xA3, 0.2),  // young rice
        (0xA5, 0.2),  // rice
        (0xA8, 0.1),  // dense rice
        (0xF2, 0.15), // flooded
        (0xC4, 0.08), // ripening
        (0x00, 0.27), // the ground shows
    ],
    belt_chance: 0.15,
    belt_width: 7.0,
};

/// Cuba: big sugar cane estates, pasture and tobacco in large sections with a few tree lines.
pub const CANE: Farmland = Farmland {
    section: 1100.0,
    strips: &[0.35, 0.4, 0.25],
    crops: &[
        (0xA7, 0.3),  // sugar cane
        (0xA9, 0.15), // tall cane
        (0xC3, 0.1),  // cut cane
        (0xCD, 0.08), // ploughed red soil
        (0xA4, 0.12), // tobacco
        (0x00, 0.25), // pasture, the ground shows
    ],
    belt_chance: 0.2,
    belt_width: 10.0,
};

pub struct Field {
    pub min: DVec2,
    pub max: DVec2,
    pub colour: u8,
}

/// The fields of section (sx, sy).
pub fn section(land: &Farmland, seed: u64, sx: i64, sy: i64) -> Vec<Field> {
    let mut rng = Rng::new(hash(seed, sx, sy, 0xF1E1D));
    let min = DVec2::new(sx as f64, sy as f64) * land.section + TRACK;
    let max = min + land.section - 2.0 * TRACK;
    let n = 1 + rng.weighted(land.strips);
    let along_x = rng.chance(0.5);
    // Cut positions across the section.
    let mut cuts = vec![0.0];
    for i in 1..n {
        cuts.push((i as f64 + rng.range(-0.25, 0.25)) / n as f64);
    }
    cuts.push(1.0);
    let weights: Vec<f64> = land.crops.iter().map(|c| c.1).collect();
    let mut out = Vec::new();
    for w in cuts.windows(2) {
        let colour = land.crops[rng.weighted(&weights)].0;
        if colour == 0 {
            continue;
        }
        let (a, b) = (w[0], w[1]);
        let span = max - min;
        let (lo, hi) = if along_x {
            (DVec2::new(min.x, min.y + span.y * a), DVec2::new(max.x, min.y + span.y * b))
        } else {
            (DVec2::new(min.x + span.x * a, min.y), DVec2::new(min.x + span.x * b, max.y))
        };
        let gap = if a > 0.0 { TRACK * 0.5 } else { 0.0 };
        let gap_hi = if b < 1.0 { TRACK * 0.5 } else { 0.0 };
        let (lo, hi) = if along_x {
            (lo + DVec2::new(0.0, gap), hi - DVec2::new(0.0, gap_hi))
        } else {
            (lo + DVec2::new(gap, 0.0), hi - DVec2::new(gap_hi, 0.0))
        };
        out.push(Field { min: lo, max: hi, colour });
    }
    out
}

/// Tree rows on the south and west edges of section (sx, sy) (each edge belongs to the
/// section north or east of it): line segments in metres.
pub fn belts(land: &Farmland, seed: u64, sx: i64, sy: i64) -> Vec<(DVec2, DVec2)> {
    let o = DVec2::new(sx as f64, sy as f64) * land.section;
    let mut out = Vec::new();
    if Rng::new(hash(seed, sx, sy, 0xBE17)).chance(land.belt_chance) {
        out.push((o, o + DVec2::new(land.section, 0.0)));
    }
    if Rng::new(hash(seed, sx, sy, 0xBE18)).chance(land.belt_chance) {
        out.push((o, o + DVec2::new(0.0, land.section)));
    }
    out
}

/// Sections overlapping a rectangle.
pub fn sections(land: &Farmland, min: DVec2, max: DVec2) -> impl Iterator<Item = (i64, i64)> {
    let (a, b) = ((min / land.section).floor(), (max / land.section).floor());
    (a.y as i64..=b.y as i64).flat_map(move |sy| (a.x as i64..=b.x as i64).map(move |sx| (sx, sy)))
}

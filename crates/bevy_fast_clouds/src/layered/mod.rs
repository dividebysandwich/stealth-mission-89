//! Layered clouds (`CloudMode::Layered`): one ray march through the whole sky, in the manner of
//! the volumetric cloud renderers of HDRP / Nubis (and the ports in
//! jiaozi158/UnityVolumetricCloudsURP and clayjohn's godot-volumetric-cloud-demo-v2), drawing a
//! stratus deck, cumulus and cumulonimbus (column and anvil) as volumes one can fly through.
//!
//! - Where the clouds are comes from the application ([`LayeredClouds`]): the deck's heights and
//!   its coverage field (a value noise field over a threshold), and a list of convective clouds
//!   (cylinders: centre, radius, base and top). They are drawn into two maps around the eye: the
//!   weather map (per texel the nearest cloud's distance over its radius, its kind and fill, its
//!   base and top) and the deck's field. Noise only erodes the edges; the bodies stay where
//!   the application put them.
//! - Shapes: Perlin-Worley base noise remapped by coverage and per-kind height profiles
//!   (domed cumulus; a storm cell's column under an anvil spreading to nearly twice its
//!   radius; a flat-based deck with a lumpy top), eroded by Worley detail noise.
//! - Light: Beer-powder, a two-lobe Henyey-Greenstein phase, three multiple scattering octaves,
//!   a short march towards the sun, ambient light from above and below, and the colours of the
//!   application's 16-entry cloud ramp ([`crate::CloudShading`]): sunlit (entry 0) to darkest.
//! - Drawn at a reduced resolution with a jittered start, accumulated over frames (reprojected
//!   through the floating origin's moves), and upsampled over the view, kept off nearer
//!   geometry by the depth at which the clouds start.

pub mod noise;
mod render;

use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
    render::{extract_resource::ExtractResource, render_resource::ShaderType},
};

use crate::{CloudRenderSettings, CloudShading};

pub(crate) use render::build_render;

/// Most convective clouds drawn into the weather map.
pub const MAX_CONVECTIVE: usize = 256;
/// Weather map size (texels per side) and half its span (metres).
pub const MAP_SIZE: u32 = 1024;
pub const MAP_HALF: f32 = 56_000.0;
/// The map is centred again when the eye is this far from its centre (metres).
const MAP_RECENTRE: f64 = 12_000.0;
/// The noise's tiles all divide this (metres): the floating origin is handed over modulo it.
pub const NOISE_PERIOD: f64 = 24_000.0;

/// The deck (a stratus layer): world heights and its coverage field, which is the
/// application's (value noise over a threshold).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CloudDeck {
    /// World heights (metres) of the base and the top.
    pub base: f32,
    pub top: f32,
    /// Covered where the field reaches this, 0..1.
    pub threshold: f32,
    /// The field's seed.
    pub seed: u32,
    /// Metres per unit of the field's coordinates; the field is evaluated at (world x / unit,
    /// 32768 + world z / unit).
    pub unit: f32,
    /// The field's octaves: (cell size in units, weight), three of them.
    pub octaves: [(f32, f32); 3],
    /// The share of the sky it covers on average, 0..1 (drawn far away).
    pub mean_cover: f32,
}

/// A cumulus or a cumulonimbus: a cylinder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConvectiveCloud {
    /// World position of the centre of its base.
    pub base: DVec3,
    pub radius: f32,
    pub height: f32,
    /// A storm cell (column and anvil).
    pub storm: bool,
    /// 0..1: how built up it is (thinner and smaller below 1; 0: not drawn).
    pub fill: f32,
}

/// What the layered clouds draw (set by the application each frame).
#[derive(Resource, Clone, Debug, Default)]
pub struct LayeredClouds {
    /// The world position of the render origin (a floating origin).
    pub origin: DVec3,
    pub deck: Option<CloudDeck>,
    pub clouds: Vec<ConvectiveCloud>,
    /// How much darker the cumulus and the deck are drawn (a storm sky), and the storm cells.
    pub darken: f32,
    pub storm_darken: f32,
    /// The noise's drift (metres, world x / z): the clouds' texture moving with the wind.
    pub drift: Vec2,
    /// The share of the sky the cumulus cover, 0..1: how far their thin tufts spread and how
    /// many wisps drift between them.
    pub cumulus_cover: f32,
    /// Downwind direction (render x, z; unit or zero): storm cells' anvils are blown that way.
    pub wind_dir: Vec2,
    /// The cumulus layer's base and top (world heights, metres): the thin clouds between the
    /// application's cumulus are drawn there.
    pub cumulus_layer: Option<(f32, f32)>,
}

/// The uniform of the march and its passes.
#[derive(Clone, Copy, ShaderType, Default, Debug)]
pub struct LayeredGlobals {
    /// Direction to the sun, haze distance.
    pub sun: Vec4,
    pub flash: Vec4,
    pub line_a: Vec4,
    pub line_b: Vec4,
    pub line_color: Vec4,
    pub fog: Vec4,
    pub ramp: [Vec4; 16],
    /// Deck: base and top (render y), threshold, present.
    pub deck: Vec4,
    /// Deck: mean coverage, darkening, field seed, field unit (metres).
    pub deck_field: Vec4,
    /// Deck field octaves: cell sizes (xyz).
    pub deck_cells: Vec4,
    /// Deck field octave weights (xyz).
    pub deck_weights: Vec4,
    /// Convective clouds: lowest base and highest top (render y), present, storm darkening.
    pub band: Vec4,
    /// Maps: centre x and z (render), half span (m), cumulus darkening.
    pub map: Vec4,
    /// Noise offset: render position plus this, modulo `NOISE_PERIOD`, is the noise position.
    pub noise: Vec4,
    /// Frame counter, resolution scale, primary steps, the render y of the world's zero.
    pub params: Vec4,
    /// Cumulus coverage 0..1, downwind direction (y, z).
    pub layer: Vec4,
    /// The cumulus layer: base and top (render y), present.
    pub cumulus: Vec4,
    /// The sunlit side's colour (`CloudShading::sun_color`).
    pub sun_color: Vec4,
}

/// Everything the render world needs.
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct LayeredFrame {
    pub on: bool,
    pub globals: LayeredGlobals,
    /// Convective clouds relative to the map's centre: (x, z, radius, fill + 2 if a storm),
    /// (base y world, top y world, 0, 0).
    pub instances: Vec<[Vec4; 2]>,
    /// The map's centre (world x, z).
    pub anchor: DVec2,
    /// Bumped when the weather map or the deck's field must be drawn again.
    pub weather_gen: u64,
    pub deck_gen: u64,
    /// The deck field's seed, unit and octaves (what its map depends on).
    pub deck_key: (u32, u32, [u32; 6]),
    pub origin: DVec3,
    pub steps: u32,
    pub resolution_scale: u32,
}

/// The weather map as last built and when to build it again.
#[derive(Default)]
pub(crate) struct MapState {
    anchor: Option<DVec2>,
    instances: Vec<[Vec4; 2]>,
    deck_key: Option<(u32, u32, [u32; 6])>,
}

type LodCameras<'w, 's> =
    Query<'w, 's, &'static GlobalTransform, With<crate::render::CloudCamera>>;

/// Gathers the layered clouds' frame (`PostUpdate`, in `CloudSystems`).
pub(crate) fn upload_layered_frame(
    cameras: LodCameras,
    settings: Res<CloudRenderSettings>,
    shading: Res<CloudShading>,
    clouds: Res<LayeredClouds>,
    mut frame: ResMut<LayeredFrame>,
    mut state: Local<MapState>,
    mut counter: Local<u32>,
) {
    let frame = frame.as_mut();
    frame.on = settings.mode == crate::CloudMode::Layered && noise::ready();
    if settings.mode == crate::CloudMode::Layered {
        noise::start();
    }
    let Some(cam) = cameras.iter().next() else {
        frame.on = false;
        return;
    };
    if !frame.on {
        return;
    }
    *counter = counter.wrapping_add(1);
    let origin = clouds.origin;
    let eye = cam.translation().as_dvec3() + origin;
    let eye_xz = DVec2::new(eye.x, eye.z);
    // The map follows the eye in steps, on a grid of its texels (stable under origin moves).
    let texel = 2.0 * MAP_HALF as f64 / MAP_SIZE as f64;
    let anchor = match state.anchor {
        Some(a) if a.distance(eye_xz) < MAP_RECENTRE => a,
        _ => (eye_xz / texel).round() * texel,
    };
    let rel = |p: DVec3| (DVec2::new(p.x, p.z) - anchor).as_vec2();
    let reach = MAP_HALF + 0.0;
    let mut instances: Vec<[Vec4; 2]> = clouds
        .clouds
        .iter()
        .filter(|c| c.fill > 0.0)
        .filter(|c| {
            let r = rel(c.base);
            r.x.abs() < reach + 2.0 * c.radius && r.y.abs() < reach + 2.0 * c.radius
        })
        .map(|c| {
            let r = rel(c.base);
            // Fills quantized: a fading cloud redraws the map a few dozen times, not each frame.
            let fill = (c.fill.min(1.0) * 64.0).round() / 64.0;
            [
                Vec4::new(r.x, r.y, c.radius, fill + if c.storm { 2.0 } else { 0.0 }),
                Vec4::new(c.base.y as f32, (c.base.y + c.height as f64) as f32, 0.0, 0.0),
            ]
        })
        .collect();
    instances.sort_by(|a, b| {
        a[0].x.hypot(a[0].y).total_cmp(&b[0].x.hypot(b[0].y))
    });
    instances.truncate(MAX_CONVECTIVE);
    if state.anchor != Some(anchor) || state.instances != instances {
        frame.weather_gen += 1;
        state.instances = instances.clone();
    }
    let deck_key = clouds.deck.map(|d| {
        (
            d.seed,
            d.unit.to_bits(),
            [
                d.octaves[0].0.to_bits(),
                d.octaves[0].1.to_bits(),
                d.octaves[1].0.to_bits(),
                d.octaves[1].1.to_bits(),
                d.octaves[2].0.to_bits(),
                d.octaves[2].1.to_bits(),
            ],
        )
    });
    if let Some(key) = deck_key
        && (state.anchor != Some(anchor) || state.deck_key != Some(key))
    {
        frame.deck_gen += 1;
        state.deck_key = Some(key);
        frame.deck_key = key;
    }
    state.anchor = Some(anchor);
    frame.anchor = anchor;
    frame.instances = instances;
    frame.origin = origin;
    frame.steps = settings.step_count.max(8);
    frame.resolution_scale = settings.resolution_scale.max(1);

    let s = &shading;
    let render_y = |y: f32| (y as f64 - origin.y) as f32;
    let deck = clouds.deck;
    let (lo, hi) = clouds
        .clouds
        .iter()
        .filter(|c| c.fill > 0.0)
        .fold((f32::MAX, f32::MIN), |(lo, hi), c| {
            (lo.min(c.base.y as f32), hi.max(c.base.y as f32 + c.height))
        });
    let (lo, hi) = match clouds.cumulus_layer {
        Some((b, t)) => (lo.min(b), hi.max(t)),
        None => (lo, hi),
    };
    let has_band = !frame.instances.is_empty() || clouds.cumulus_layer.is_some();
    let centre = (anchor - DVec2::new(origin.x, origin.z)).as_vec2();
    let noise_off = |v: f64| v.rem_euclid(NOISE_PERIOD) as f32;
    frame.globals = LayeredGlobals {
        sun: s.sun_dir.normalize_or(Vec3::Y).extend(s.haze_distance),
        flash: s.flash.extend(0.0),
        line_a: s.line_a,
        line_b: s.line_b,
        line_color: s.line_color.extend(0.0),
        fog: s.fog,
        ramp: s.ramp,
        deck: deck.map_or(Vec4::ZERO, |d| {
            Vec4::new(render_y(d.base), render_y(d.top), d.threshold, 1.0)
        }),
        deck_field: deck.map_or(Vec4::ZERO, |d| {
            Vec4::new(d.mean_cover, clouds.darken, (d.seed & 0xFFFF) as f32, d.unit)
        }),
        deck_cells: deck.map_or(Vec4::ONE, |d| {
            Vec4::new(d.octaves[0].0, d.octaves[1].0, d.octaves[2].0, 0.0)
        }),
        deck_weights: deck.map_or(Vec4::ZERO, |d| {
            Vec4::new(d.octaves[0].1, d.octaves[1].1, d.octaves[2].1, 0.0)
        }),
        band: if has_band {
            Vec4::new(render_y(lo), render_y(hi), 1.0, clouds.storm_darken)
        } else {
            Vec4::new(0.0, 0.0, 0.0, clouds.storm_darken)
        },
        map: Vec4::new(centre.x, centre.y, MAP_HALF, clouds.darken),
        noise: Vec4::new(
            noise_off(origin.x + clouds.drift.x as f64),
            noise_off(origin.z + clouds.drift.y as f64),
            0.0,
            0.0,
        ),
        params: Vec4::new(
            (*counter % 4096) as f32,
            frame.resolution_scale as f32,
            frame.steps as f32,
            (-origin.y) as f32,
        ),
        layer: {
            let d = clouds.wind_dir.normalize_or_zero();
            Vec4::new(clouds.cumulus_cover.clamp(0.0, 1.0), d.x, d.y, 0.0)
        },
        cumulus: clouds.cumulus_layer.map_or(Vec4::ZERO, |(b, t)| {
            Vec4::new(render_y(b), render_y(t), 1.0, 0.0)
        }),
        sun_color: s.sun_color,
    };
}

#[cfg(test)]
mod tests {
    /// Every noise tile of the march divides `NOISE_PERIOD`: positions wrap there, and a tile
    /// that doesn't fit leaves a straight seam through the clouds.
    #[test]
    fn noise_tiles_divide_the_period() {
        let shader = include_str!("../shaders/layered_cloud.wgsl");
        let tiles: Vec<(String, f64)> = shader
            .lines()
            .filter_map(|l| {
                let l = l.trim().strip_prefix("const ")?;
                let (name, rest) = l.split_once(": f32 = ")?;
                if !name.ends_with("TILE") {
                    return None;
                }
                let v: f64 = rest.trim_end_matches(';').parse().expect("a number");
                Some((name.to_string(), v))
            })
            .collect();
        assert!(tiles.len() >= 8, "{tiles:?}");
        for (name, tile) in tiles {
            let n = super::NOISE_PERIOD / tile;
            assert!((n - n.round()).abs() < 1e-9, "{name} = {tile} doesn't divide the period");
        }
    }
}

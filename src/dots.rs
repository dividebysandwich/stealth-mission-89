//! Fullbright "dot" lights (runway lights, city lights, beacons). In classic rendering these
//! are single pixels (zero-length lines) drawn in a solid palette colour. Here each is a quad
//! that the palette shader expands around the dot's position to a constant size in the
//! pixels of whichever view draws it (main view, XR eyes, tracking camera), drawn with the
//! line material (solid colour, distance haze), so the lights read the same
//! at any resolution, distance or zoom.
//!
//! Dots are drawn in the transparent pass (`dot_materials`): as opaque geometry, which writes
//! depth, tens of thousands of these tiny quads hang the binning pass of the Steam Frame's GPU
//! (Turnip, Adreno 750) at night.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, MeshTag, MeshVertexAttribute, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::VertexFormat;

use crate::palette_material::{FLAG_DOT, FLAG_GLOW, FLAG_NIGHT_DIM, PaletteMaterial, TAG_DOT};

pub struct DotsPlugin;

impl Plugin for DotsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DotMaterials>()
            .add_systems(Startup, setup_dot_assets)
            .add_systems(PostUpdate, dot_materials);
    }
}

/// XR: angular size of a dot in radians (a 320x200 pixel's, a little enlarged); the
/// shader caps the per-view pixel size to it (`Globals.dots.x`).
pub const XR_ANGLE: f32 = 0.6 * 1.5 / (crate::cockpit::VSCALE * crate::cockpit::FOCAL);

#[derive(Resource)]
pub struct DotAssets {
    quad: Handle<Mesh>,
}

#[derive(Component)]
pub struct DotLight;

impl DotLight {
    /// Quad, transform and mesh tag (`tag` with the dot bit set); the caller adds the material.
    pub fn bundle(pos: Vec3, assets: &DotAssets, tag: u32) -> impl Bundle {
        (
            DotLight,
            Mesh3d(assets.quad.clone()),
            MeshTag(tag | TAG_DOT),
            Transform::from_translation(pos),
            bevy::light::NotShadowCaster,
        )
    }
}

/// Per-vertex palette colour of a dot batch with lamps of several colours (otherwise the
/// material's colour).
pub const ATTRIBUTE_DOT_COLOUR: MeshVertexAttribute =
    MeshVertexAttribute::new("DotColour", 0x7D07_0001, VertexFormat::Uint32);

/// A dot batch whose dots have their own palette colours, drawn in the given order.
pub fn dot_batch_coloured(points: &[Vec3], colours: &[u8]) -> Mesh {
    let colours: Vec<u32> = colours.iter().flat_map(|&c| [c as u32; 4]).collect();
    dot_batch(points).with_inserted_attribute(ATTRIBUTE_DOT_COLOUR, colours)
}

/// A glowing lamp's kind (`FLAG_GLOW`; the shader's `GLOW_KIND_*`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GlowKind {
    /// Street and road lamps, porch and yard lights.
    Lamp = 0,
    /// Lit windows: only beyond the range where the walls show theirs (`Globals::city`).
    Window = 1,
    /// Floodlights over yards and car parks.
    Flood = 2,
    /// Red warning beacons on masts, chimneys and towers.
    Beacon = 3,
}

/// A glowing lamp's tint (the shader's `glow_tint`; 0 the palette colour).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GlowTint {
    Palette = 0,
    Sodium = 1,
    Led = 2,
    Mercury = 3,
    WarmWindow = 4,
    CoolWindow = 5,
    Green = 6,
    Blue = 7,
}

/// The packed word of a glowing lamp (`ATTRIBUTE_DOT_COLOUR` of a `FLAG_GLOW` batch): palette
/// colour (bits 0..7, for the FLIR and the tint `Palette`), brightness 0..15 (8..11), kind
/// (12..13), tint (14..17), a seed for its twinkle (18..31).
pub fn glow_word(colour: u8, level: u8, kind: GlowKind, tint: GlowTint, seed: u32) -> u32 {
    colour as u32
        | (level.min(15) as u32) << 8
        | (kind as u32) << 12
        | (tint as u32) << 14
        | (seed & 0x3FFF) << 18
}

/// A dot batch of glowing lamps, each with its packed word (`glow_word`).
pub fn dot_batch_words(points: &[Vec3], words: &[u32]) -> Mesh {
    let words: Vec<u32> = words.iter().flat_map(|&w| [w; 4]).collect();
    dot_batch(points).with_inserted_attribute(ATTRIBUTE_DOT_COLOUR, words)
}

/// Dots drawn as glowing lamps (`FLAG_GLOW`): the towns' lights with `Effects::city_lights`.
#[derive(Component)]
pub struct Glow;

fn setup_dot_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(DotAssets { quad: meshes.add(dot_batch(&[Vec3::ZERO])) });
}

/// Four vertices per dot at its centre; the shader spreads them by the quad corner (UV).
/// Every dot mesh must be built this way: the shader takes each vertex's position as the dot
/// centre, so a real quad would keep its world size and grow up close.
pub fn dot_batch(points: &[Vec3]) -> Mesh {
    let pos: Vec<Vec3> = points.iter().flat_map(|&p| [p; 4]).collect();
    let corners: Vec<[f32; 2]> =
        points.iter().flat_map(|_| [[-0.5, -0.5], [0.5, -0.5], [-0.5, 0.5], [0.5, 0.5]]).collect();
    let idx: Vec<u32> =
        (0..points.len() as u32).flat_map(|i| [0, 1, 3, 0, 3, 2].map(|k| 4 * i + k)).collect();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, corners)
        .with_inserted_indices(Indices::U32(idx))
}

/// The dot variant of each palette material a dot was given (by the material it copies), and
/// the reverse.
#[derive(Resource, Default)]
struct DotMaterials {
    /// By source and whether glowing (`Glow`).
    twins: HashMap<(AssetId<PaletteMaterial>, bool), Handle<PaletteMaterial>>,
    sources: HashMap<AssetId<PaletteMaterial>, AssetId<PaletteMaterial>>,
}

/// Gives every dot the dot variant of its material (spawners and colour changes such as the
/// PAPI's assign ordinary line materials), and keeps the variants' colours in step.
#[allow(clippy::type_complexity)]
fn dot_materials(
    mut dots: ResMut<DotMaterials>,
    mut materials: ResMut<Assets<PaletteMaterial>>,
    mut q: Query<
        (&MeshTag, &mut MeshMaterial3d<PaletteMaterial>, Has<Glow>),
        Changed<MeshMaterial3d<PaletteMaterial>>,
    >,
) {
    for (tag, mut material, glow) in &mut q {
        let source = material.0.id();
        if tag.0 & TAG_DOT == 0 || dots.sources.contains_key(&source) {
            continue;
        }
        let twin = match dots.twins.get(&(source, glow)) {
            Some(h) => h.clone(),
            None => {
                let Some(m) = materials.get(source) else { continue };
                let mut m = m.clone();
                // Dots are lights whatever their colour.
                m.params.flags = (m.params.flags | FLAG_DOT) & !FLAG_NIGHT_DIM;
                if glow {
                    m.params.flags |= FLAG_GLOW;
                }
                let h = materials.add(m);
                dots.twins.insert((source, glow), h.clone());
                dots.sources.insert(h.id(), source);
                h
            }
        };
        material.0 = twin;
    }
    // A source gone (a new theater's palette, the old one's materials freed): its twin goes
    // too, else they'd pile up with each palette (and keep its globals buffer).
    let DotMaterials { twins, sources } = &mut *dots;
    twins.retain(|(source, _), _| materials.contains(*source));
    sources.retain(|_, source| materials.contains(*source));
    for ((source, _), twin) in &dots.twins {
        let Some(word) = materials.get(*source).map(|m| m.params.word) else { continue };
        if let Some(mut m) = materials.get_mut(twin)
            && m.params.word != word
        {
            m.params.word = word;
        }
    }
}

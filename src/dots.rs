//! Fullbright "dot" lights (runway lights, city lights, beacons). In classic rendering these
//! are single pixels (zero-length lines) drawn in a solid palette colour. Here each is a quad
//! that the palette shader expands around the dot's position to a minimum size in the
//! pixels of whichever view draws it (main view, XR eyes, tracking camera), drawn with the
//! line material (solid colour, distance haze). Modern city lights have a small, soft round
//! footprint far away and grow with their projected size on approach or with zoom. Aircraft
//! lamps also grow up close; other lights keep their fixed screen size.
//!
//! Dots are drawn in the transparent pass (`dot_materials`): as opaque geometry, which writes
//! depth, tens of thousands of these tiny quads hang the binning pass of the Steam Frame's GPU
//! (Turnip, Adreno 750) at night.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, MeshTag, MeshVertexAttribute, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::VertexFormat;

use crate::palette_material::{FLAG_DOT, FLAG_NIGHT_DIM, PaletteMaterial, TAG_DOT};

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
    city: HashMap<u8, Handle<Mesh>>,
}

impl DotAssets {
    /// A model lamp with the city style and its palette colour. Other colours are distant
    /// model silhouettes rather than lights, so they keep the ordinary dot mesh.
    pub fn city_mesh(&self, colour: u8) -> Option<&Handle<Mesh>> {
        self.city.get(&colour)
    }
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

/// Per-vertex palette colour (low byte) and flags of a dot batch (otherwise the material's
/// colour). Flags belong to each lamp, so city lights can share a draw with warning beacons.
pub const ATTRIBUTE_DOT_COLOUR: MeshVertexAttribute =
    MeshVertexAttribute::new("DotColour", 0x7D07_0001, VertexFormat::Uint32);

/// Bit above the palette byte in `ATTRIBUTE_DOT_COLOUR`: a city lamp, window or red beacon.
/// Kept separate from mesh tags (which also encode aircraft spotting sizes and light ranges).
pub const DOT_CITY: u32 = 1 << 8;

/// A dot batch whose dots have their own palette colours and flags, drawn in the given order.
pub fn dot_batch_coloured(points: &[Vec3], colours: &[u32]) -> Mesh {
    let colours: Vec<u32> = colours.iter().flat_map(|&c| [c; 4]).collect();
    dot_batch(points).with_inserted_attribute(ATTRIBUTE_DOT_COLOUR, colours)
}

fn setup_dot_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    // Share these tiny meshes across all settlement models. Their materials still carry the
    // plain colour for local illumination; only the vertices carry the city-style flag.
    let city = [15, 146, 147, 148, 149, 150, 151]
        .into_iter()
        .map(|c| (c, meshes.add(dot_batch_coloured(&[Vec3::ZERO], &[c as u32 | DOT_CITY]))))
        .collect();
    commands.insert_resource(DotAssets { quad: meshes.add(dot_batch(&[Vec3::ZERO])), city });
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
    twins: HashMap<AssetId<PaletteMaterial>, Handle<PaletteMaterial>>,
    sources: HashMap<AssetId<PaletteMaterial>, AssetId<PaletteMaterial>>,
}

/// Gives every dot the dot variant of its material (spawners and colour changes such as the
/// PAPI's assign ordinary line materials), and keeps the variants' colours in step.
fn dot_materials(
    mut dots: ResMut<DotMaterials>,
    mut materials: ResMut<Assets<PaletteMaterial>>,
    mut q: Query<
        (&MeshTag, &mut MeshMaterial3d<PaletteMaterial>),
        Changed<MeshMaterial3d<PaletteMaterial>>,
    >,
) {
    for (tag, mut material) in &mut q {
        let source = material.0.id();
        if tag.0 & TAG_DOT == 0 || dots.sources.contains_key(&source) {
            continue;
        }
        let twin = match dots.twins.get(&source) {
            Some(h) => h.clone(),
            None => {
                let Some(m) = materials.get(source) else { continue };
                let mut m = m.clone();
                // Dots are lights whatever their colour.
                m.params.flags = (m.params.flags | FLAG_DOT) & !FLAG_NIGHT_DIM;
                let h = materials.add(m);
                dots.twins.insert(source, h.clone());
                dots.sources.insert(h.id(), source);
                h
            }
        };
        material.0 = twin;
    }
    for (source, twin) in &dots.twins {
        let Some(word) = materials.get(*source).map(|m| m.params.word) else { continue };
        if let Some(mut m) = materials.get_mut(twin)
            && m.params.word != word
        {
            m.params.word = word;
        }
    }
}

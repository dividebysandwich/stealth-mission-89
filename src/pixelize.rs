//! Pixelize (`Effects::pixelize`, OPTIONS > GRAPHICS > Retro, desktop): the 3D view drawn in
//! big square pixels, as on an MS-DOS PC (shader: assets/shaders/pixelize.wgsl).
//!
//! A fullscreen pass on the eye camera (the menus' scene camera too), after the cockpit, the
//! terrain, the aircraft and the sky are drawn and before Bevy's UI pass: the interface (the
//! menus, the tooltips) is drawn over it, sharp. The pass keeps one pixel of each square block
//! of the picture, so edges stay hard: the anti-aliasing is switched off while it is on
//! (`palette::apply_msaa`).

use bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin};
use bevy::prelude::*;
use bevy::render::{extract_component::ExtractComponent, render_resource::ShaderType};
use bevy::shader::ShaderRef;

use crate::camera::EyeCamera;
use crate::menu_scene::SceneCamera;
use crate::settings::Settings;

pub struct PixelizePlugin;

impl Plugin for PixelizePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FullscreenMaterialPlugin::<Pixelize>::default())
            .add_systems(PostUpdate, sync);
    }
}

/// On a camera: its picture is pixelized.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Default)]
pub struct Pixelize {
    /// x: the side of a block in screen pixels (the pad fills the 16 bytes WebGL2 wants a
    /// uniform to be).
    params: Vec4,
}

impl Pixelize {
    fn new(block: u32) -> Self {
        Self { params: Vec4::new(block as f32, 0.0, 0.0, 0.0) }
    }
}

impl FullscreenMaterial for Pixelize {
    fn fragment_shader() -> ShaderRef {
        "shaders/pixelize.wgsl".into()
    }
}

/// The cameras drawing the 3D view.
type ViewCameras = Or<(With<EyeCamera>, With<SceneCamera>)>;

/// Puts the pass on the cameras while the option is on (`palette::pixelized`), with the
/// chosen block size.
fn sync(
    mut commands: Commands,
    settings: Res<Settings>,
    mut cameras: Query<(Entity, Option<&mut Pixelize>), ViewCameras>,
) {
    let want = crate::palette::pixelized(&settings).then(|| {
        let blocks = crate::effects::PIXEL_BLOCKS;
        Pixelize::new(settings.effects.pixel_block.clamp(*blocks.start(), *blocks.end()))
    });
    for (camera, has) in &mut cameras {
        match (want, has) {
            (Some(want), None) => {
                commands.entity(camera).insert(want);
            }
            (Some(want), Some(mut has)) => {
                if has.params != want.params {
                    *has = want;
                }
            }
            (None, Some(_)) => {
                commands.entity(camera).remove::<Pixelize>();
            }
            (None, None) => {}
        }
    }
}

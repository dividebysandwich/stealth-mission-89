//! The HUD combiner glass material (shader: assets/shaders/hud_glass.wgsl): shows the HUD
//! canvas collimated to infinity through the glass, like a real HUD.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

const SHADER: &str = "shaders/hud_glass.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct HudGlassMaterial {
    /// Glass tint where the HUD is blank (linear rgb, alpha).
    #[uniform(0)]
    pub tint: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
    /// Canvas pixel of the boresight (xy) and canvas pixels per unit of the tangent plane
    /// (zw, horizontal and vertical).
    #[uniform(3)]
    pub proj: Vec4,
    /// x: 1 = modern HUD (every set texel drawn in the HUD colour), y: brightness, z: the
    /// colour (`HudColour`).
    #[uniform(4)]
    pub style: Vec4,
}

/// The modern HUD's and the helmet display's symbology colour (the shaders' `HUD_COLOURS`;
/// the classic HUD keeps its palette).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum HudColour {
    #[default]
    Green,
    Amber,
    Cyan,
    White,
}

impl HudColour {
    pub fn next(self) -> Self {
        match self {
            HudColour::Green => HudColour::Amber,
            HudColour::Amber => HudColour::Cyan,
            HudColour::Cyan => HudColour::White,
            HudColour::White => HudColour::Green,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            HudColour::Green => "GREEN",
            HudColour::Amber => "AMBER",
            HudColour::Cyan => "CYAN",
            HudColour::White => "WHITE",
        }
    }
}

/// Steps the HUD colour (remembered between runs when the game started into the menu).
pub(super) fn switch_colour(
    mut actions: MessageReader<crate::input::ActionEvent>,
    mut settings: ResMut<crate::settings::Settings>,
    mut msg: ResMut<crate::hud::HudMessages>,
) {
    let mut changed = false;
    for crate::input::ActionEvent(a) in actions.read() {
        if *a == crate::input::Action::HudColour {
            settings.hud_colour = settings.hud_colour.next();
            msg.show(format!("HUD {}", settings.hud_colour.name()));
            changed = true;
        }
    }
    if changed && settings.initial_state == crate::settings::AppState::Menu {
        crate::prefs::Prefs::save_options(&settings);
    }
}

/// Gives the HUD glass and the helmet display the chosen colour (`style.z`), new ones too.
pub(super) fn recolour(
    settings: Res<crate::settings::Settings>,
    mut huds: ResMut<Assets<HudGlassMaterial>>,
    mut hmds: ResMut<Assets<HmdMaterial>>,
) {
    let z = settings.hud_colour as u8 as f32;
    let stale: Vec<_> = huds.iter().filter(|(_, m)| m.style.z != z).map(|(id, _)| id).collect();
    for id in stale {
        if let Some(mut m) = huds.get_mut(id) {
            m.style.z = z;
        }
    }
    let stale: Vec<_> = hmds.iter().filter(|(_, m)| m.style.z != z).map(|(id, _)| id).collect();
    for id in stale {
        if let Some(mut m) = hmds.get_mut(id) {
            m.style.z = z;
        }
    }
}

impl HudGlassMaterial {
    /// Canvas size, projection and style of the classic or the modern HUD (`scale`: the modern
    /// HUD's canvas pixels per unit; the classic HUD is shrunk alike, `hud::classic_zoom`).
    pub fn layout(classic: bool, scale: f32) -> ((usize, usize), Vec4, Vec4) {
        use crate::hud::modern;
        if classic {
            let f = crate::cockpit::FOCAL * crate::hud::classic_zoom(scale);
            (
                (320, 200),
                Vec4::new(159.5, 52.0, f, f * crate::cockpit::VSCALE),
                Vec4::new(0.0, 1.0, 0.0, 0.0),
            )
        } else {
            (
                (modern::WIDTH, modern::HEIGHT),
                Vec4::new(modern::CX, modern::CY, scale, scale),
                Vec4::new(1.0, 1.0, 0.0, 0.0),
            )
        }
    }
}

impl Material for HudGlassMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout =
            layout.0.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// The helmet-mounted display's visor (`hmd.rs`): the HUD glass's collimated canvas on a quad
/// carried by the head, drawn over everything (no depth test: the cockpit never hides it).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct HmdMaterial {
    #[uniform(0)]
    pub tint: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
    #[uniform(3)]
    pub proj: Vec4,
    #[uniform(4)]
    pub style: Vec4,
}

impl Material for HmdMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout =
            layout.0.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.cull_mode = None;
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_compare = Some(bevy::render::render_resource::CompareFunction::Always);
            depth.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

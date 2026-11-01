//! The HUD combiner glass material (shader: assets/shaders/hud_glass.wgsl): shows the HUD
//! canvas collimated to infinity through the glass, like a real HUD.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
    MaterialPipeline, MaterialPipelineKey,
};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, RenderPipelineDescriptor,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

const SHADER: &str = "shaders/hud_glass.wgsl";

/// Pixelize leaves the HUD sharp: the glass then takes the picture's alpha away where it draws
/// (the alpha 0 of what the pass keeps as it is, pixelize.wgsl), instead of adding to it. It
/// is part of the pipeline: a blend state.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct HudGlassKey {
    sharp: bool,
}

impl From<&HudGlassMaterial> for HudGlassKey {
    fn from(m: &HudGlassMaterial) -> Self {
        Self { sharp: m.sharp }
    }
}

impl From<&HmdMaterial> for HudGlassKey {
    fn from(m: &HmdMaterial) -> Self {
        Self { sharp: m.sharp }
    }
}

impl From<&LeaveSharp> for HudGlassKey {
    fn from(m: &LeaveSharp) -> Self {
        Self { sharp: m.sharp }
    }
}

/// A `StandardMaterial` of something shown with the HUD (the notices panel next to it): sharp
/// over the pixelized view like the glass, with the HUD's switch.
pub type OverlayMaterial = ExtendedMaterial<StandardMaterial, LeaveSharp>;

#[derive(Asset, AsBindGroup, TypePath, Clone, Debug, Default)]
#[bind_group_data(HudGlassKey)]
pub struct LeaveSharp {
    /// Pixelize leaves it sharp (`Effects::sharp_hud`).
    pub sharp: bool,
}

impl MaterialExtension for LeaveSharp {
    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        leave_sharp(descriptor, key.bind_group_data);
        Ok(())
    }
}

/// The glass's blend state, with the alpha channel drawn as the HUD is kept sharp.
fn leave_sharp(descriptor: &mut RenderPipelineDescriptor, key: HudGlassKey) {
    if !key.sharp {
        return;
    }
    if let Some(fragment) = descriptor.fragment.as_mut() {
        for blend in fragment.targets.iter_mut().flatten().filter_map(|t| t.blend.as_mut()) {
            blend.alpha = BlendComponent {
                src_factor: BlendFactor::Zero,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            };
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
#[bind_group_data(HudGlassKey)]
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
    /// Pixelize leaves it sharp (`Effects::sharp_hud`).
    pub sharp: bool,
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
    Red,
    Purple,
}

impl HudColour {
    pub fn next(self) -> Self {
        match self {
            HudColour::Green => HudColour::Amber,
            HudColour::Amber => HudColour::Cyan,
            HudColour::Cyan => HudColour::White,
            HudColour::White => HudColour::Red,
            HudColour::Red => HudColour::Purple,
            HudColour::Purple => HudColour::Green,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            HudColour::Green => "GREEN",
            HudColour::Amber => "AMBER",
            HudColour::Cyan => "CYAN",
            HudColour::White => "WHITE",
            HudColour::Red => "RED",
            HudColour::Purple => "PURPLE",
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

/// Gives the HUD glass and the helmet display the chosen colour (`style.z`), new ones too, and
/// tells them whether pixelize leaves them sharp.
pub(super) fn recolour(
    settings: Res<crate::settings::Settings>,
    mut huds: ResMut<Assets<HudGlassMaterial>>,
    mut hmds: ResMut<Assets<HmdMaterial>>,
    mut overlays: ResMut<Assets<OverlayMaterial>>,
) {
    let sharp = crate::palette::pixel_sharp(&settings, crate::effects::ViewPart::Hud);
    let stale: Vec<_> =
        overlays.iter().filter(|(_, m)| m.extension.sharp != sharp).map(|(id, _)| id).collect();
    for id in stale {
        if let Some(mut m) = overlays.get_mut(id) {
            m.extension.sharp = sharp;
        }
    }
    let stale: Vec<_> = huds.iter().filter(|(_, m)| m.sharp != sharp).map(|(id, _)| id).collect();
    for id in stale {
        if let Some(mut m) = huds.get_mut(id) {
            m.sharp = sharp;
        }
    }
    let stale: Vec<_> = hmds.iter().filter(|(_, m)| m.sharp != sharp).map(|(id, _)| id).collect();
    for id in stale {
        if let Some(mut m) = hmds.get_mut(id) {
            m.sharp = sharp;
        }
    }
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
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout =
            layout.0.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.cull_mode = None;
        leave_sharp(descriptor, key.bind_group_data);
        Ok(())
    }
}

/// The helmet-mounted display's visor (`hmd.rs`): the HUD glass's collimated canvas on a quad
/// carried by the head, drawn over everything (no depth test: the cockpit never hides it).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
#[bind_group_data(HudGlassKey)]
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
    /// Pixelize leaves it sharp (`Effects::sharp_hud`).
    pub sharp: bool,
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
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout =
            layout.0.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.cull_mode = None;
        leave_sharp(descriptor, key.bind_group_data);
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_compare = Some(bevy::render::render_resource::CompareFunction::Always);
            depth.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

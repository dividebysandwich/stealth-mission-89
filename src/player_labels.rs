//! Multiplayer: the other pilots' names over their aircraft, so a wingman is found however far
//! away (on the screen and in the headset alike). Each name is a small sign facing the eye,
//! kept the same size on the screen at any distance and lifted above the aircraft (and above
//! its far-away dot). Switched off with the GAME options' player labels.

use bevy::camera::visibility::{NoFrustumCulling, RenderLayers, VisibilitySystems};
use bevy::image::ImageSampler;
use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::transform::TransformSystems;
use stealth_campaign::PlayerId;

use crate::camera::EyeCamera;
use crate::canvas::Canvas;
use crate::combat::Combat;
use crate::font::{Align, stroke_text_at, stroke_width};
use crate::settings::Settings;
use crate::world::FloatingOrigin;

/// Glyph height, pen width, outline width and margin of the label's canvas (pixels).
const GLYPH: f32 = 40.0;
const PEN: f32 = 5.0;
const OUTLINE: f32 = 13.0;
const PAD: f32 = 10.0;
/// The canvas's height seen from the eye (radians): the same on the screen at any distance.
const ANGLE: f32 = 0.03;
/// The label's middle above the aircraft's: metres, and label heights (clearing the dot).
const LIFT: f32 = 6.0;
const LIFT_HEIGHTS: f32 = 0.8;
/// Canvas inks and their colours (sRGB): the outline and the name (readable on sky and ground).
const SHADE: u8 = 1;
const NAME: u8 = 2;
const COLOURS: [[u8; 4]; 3] = [[0, 0, 0, 0], [0, 10, 40, 230], [235, 242, 255, 255]];

pub struct PlayerLabelsPlugin;

impl Plugin for PlayerLabelsPlugin {
    fn build(&self, app: &mut App) {
        // After the transforms are propagated: the eye is where it is drawn this frame.
        app.add_systems(
            PostUpdate,
            labels
                .run_if(resource_exists::<Combat>)
                .after(TransformSystems::Propagate)
                .before(VisibilitySystems::CheckVisibility),
        );
    }
}

/// A pilot's label: whose, the name drawn and its width per height.
#[derive(Component)]
struct Label {
    id: u8,
    name: String,
    aspect: f32,
}

/// Spawns, names, places and removes the labels of the other pilots in the air or on the
/// ground (not under a parachute).
#[allow(clippy::too_many_arguments)]
fn labels(
    mut commands: Commands,
    combat: Res<Combat>,
    settings: Res<Settings>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    origin: Res<FloatingOrigin>,
    eye: Query<&GlobalTransform, (With<EyeCamera>, Without<Label>)>,
    mut q: Query<(
        Entity,
        &mut Label,
        &mut Transform,
        &mut GlobalTransform,
        &mut Visibility,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut quad: Local<Option<Handle<Mesh>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let sim = &combat.sim;
    let me = sim.active_pilot();
    let others: Vec<_> = if settings.player_labels {
        sim.remote_pilots().into_iter().filter(|p| p.id != me && p.chute.is_none()).collect()
    } else {
        Vec::new()
    };
    for (e, label, ..) in &q {
        if !others.iter().any(|p| p.id == label.id) {
            commands.entity(e).try_despawn();
        }
    }
    let Some(eye) = eye.iter().next() else { return };
    let eye = eye.translation().as_dvec3() + origin.origin;
    let quad = quad.get_or_insert_with(|| meshes.add(Rectangle::new(1.0, 1.0))).clone();
    for p in &others {
        let name = campaign
            .as_ref()
            .and_then(|c| c.state().player_name(PlayerId(p.id)).map(str::to_string))
            .unwrap_or_else(|| format!("Pilot {}", p.id));
        let Some((_, mut label, mut tf, mut gtf, mut vis, material)) =
            q.iter_mut().find(|(_, l, ..)| l.id == p.id)
        else {
            let (image, aspect) = image(&name);
            let material = materials.add(StandardMaterial {
                base_color_texture: Some(images.add(image)),
                unlit: true,
                fog_enabled: false,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            });
            // Shown from the next frame, placed.
            commands.spawn((
                Label { id: p.id, name, aspect },
                Mesh3d(quad.clone()),
                MeshMaterial3d(material),
                Transform::default(),
                Visibility::Hidden,
                // Only the eye cameras (not the targeting pod's or the recon cameras).
                RenderLayers::layer(crate::camera::MAIN_LOD_LAYER),
                NoFrustumCulling,
                NotShadowCaster,
            ));
            continue;
        };
        if label.name != name {
            let (image, aspect) = image(&name);
            if let Some(mut m) = materials.get_mut(&material.0) {
                m.base_color_texture = Some(images.add(image));
            }
            *label = Label { id: p.id, name, aspect };
        }
        let (pos, _) = crate::sim_render::remote_pose(&combat, p);
        let height = (pos - eye).length() as f32 * ANGLE;
        let at = pos + DVec3::Y * (LIFT + height * LIFT_HEIGHTS) as f64;
        let rel = (at - origin.origin).as_vec3();
        // Its front (+Z) towards the eye, upright.
        let away = (at - eye).as_vec3();
        let mut t = Transform::from_translation(rel)
            .with_scale(Vec3::new(height * label.aspect, height, 1.0));
        if away.length_squared() > 1e-6 {
            t = t.looking_to(away, Vec3::Y);
        }
        *tf = t;
        *gtf = GlobalTransform::from(t);
        vis.set_if_neq(Visibility::Visible);
    }
}

/// The name in white on a dark outline, and its width per height.
fn image(name: &str) -> (Image, f32) {
    let w = (stroke_width(name, GLYPH) + 2.0 * PAD).ceil() as usize;
    let h = (GLYPH + 2.0 * PAD).ceil() as usize;
    let mut c = Canvas::new(w, h);
    let (x, y) = (w as f32 / 2.0, h as f32 / 2.0);
    stroke_text_at(&mut c, x, y, name, GLYPH, OUTLINE, SHADE, Align::Centre);
    stroke_text_at(&mut c, x, y, name, GLYPH, PEN, NAME, Align::Centre);
    let size = Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 };
    let data: Vec<u8> = c.px.iter().flat_map(|&i| COLOURS[i as usize]).collect();
    let mut image =
        Image::new(size, TextureDimension::D2, data, TextureFormat::Rgba8UnormSrgb, default());
    image.sampler = ImageSampler::linear();
    (image, w as f32 / h as f32)
}

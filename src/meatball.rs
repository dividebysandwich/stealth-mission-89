//! The carriers' landing lens (an IFLOLS, the "meatball"), on the port deck edge abeam the
//! wires (`stealth_sim::catapult::LENS`): a row of green datum lights each side, the amber
//! "ball" between them showing the pilot where he is on the glide slope (above the datum high,
//! below it low, red at the bottom: very low), and the red wave-off lights flashing when the
//! LSO waves him off (`lso::LsoState::waveoff`). Seen from the approach only, like a PAPI.
//!
//! Drawn larger than the real lens, as the carrier is (1.5 times): it reads from a mile out.

use bevy::mesh::MeshTag;
use bevy::prelude::*;
use stealth_sim::catapult::{DECK_WU, GLIDE_DEG, LENS, deck_point, landing};
use stealth_sim::oflags;

use crate::combat::Combat;
use crate::dots::dot_batch;
use crate::palette::Palette;
use crate::palette_material::{PaletteMaterial, TAG_DOT, dot_size_tenths, mesh_tag};
use crate::settings::AppState;
use crate::world::{Theater, WU, WorldPos, wu_to_bevy};

pub struct MeatballPlugin;

impl Plugin for MeatballPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Lenses>().add_systems(OnExit(AppState::Flight), despawn).add_systems(
            Update,
            (spawn, show)
                .chain()
                .after(crate::sea::SeaSystems)
                .run_if(in_state(AppState::Flight))
                .run_if(resource_exists::<Combat>),
        );
    }
}

/// Light colours, lit in every palette of the day (green and red as the airfields' lights,
/// amber the carrier deck's lit yellow).
const GREEN: u8 = 0x97;
const AMBER: u8 = 0x13;
const RED: u8 = 0x92;
/// The lens's middle above the deck (m), the ball's travel either way of the datum (m) for
/// `BALL_DEG` off the glide slope, and the bottom part where the ball shows red (deg low).
const LENS_UP_M: f32 = 5.0;
const BALL_TRAVEL_M: f32 = 4.5;
const BALL_DEG: f64 = 0.75;
const RED_BALL_DEG: f64 = -0.45;
/// Datum lights: this many each side, from this far out, this far apart (m).
const DATUM_LIGHTS: usize = 5;
const DATUM_FROM_M: f32 = 3.5;
const DATUM_STEP_M: f32 = 1.6;
/// Wave-off lights: a column each side of the ball (m out, heights), flashing this fast (Hz).
const WAVEOFF_OUT_M: f32 = 2.0;
const WAVEOFF_UP_M: [f32; 3] = [-2.5, 0.0, 2.5];
const WAVEOFF_HZ: f32 = 1.6;
/// Seen within this distance (m) and this cosine of the angle off the approach (about ±20°).
const RANGE_M: f32 = 6000.0;
const COS_SPREAD: f32 = 0.94;
const DOT_TENTHS: u32 = 18;
const BALL_TENTHS: u32 = 22;

#[derive(Resource, Default)]
struct Lenses {
    lenses: Vec<Lens>,
    done: bool,
}

struct Lens {
    carrier: usize,
    root: Entity,
    ball: Entity,
    waveoff: Entity,
    /// Its foot on the deck (wu east, north), the landing heading (rad, clockwise from north),
    /// and the touchdown point (wu east, north) the glide slope runs to.
    east: f64,
    north: f64,
    heading: f64,
    touchdown: (f64, f64),
}

fn tag(tenths: u32) -> MeshTag {
    MeshTag(mesh_tag(1, 6, false) | TAG_DOT | dot_size_tenths(tenths))
}

fn spawn(
    mut commands: Commands,
    mut lenses: ResMut<Lenses>,
    combat: Res<Combat>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if lenses.done {
        return;
    }
    let sim = &combat.sim;
    if sim.objects().is_empty() {
        return;
    }
    lenses.done = true;
    for (b, o) in sim.objects().iter().enumerate() {
        if o.flags & oflags::CARRIER == 0 {
            continue;
        }
        let l = landing((o.x, o.y), sim.ns());
        let (east, north) = deck_point((o.x, o.y), sim.ns(), LENS);
        // Across the approach (the lens's local x), up (y).
        let mut datum = Vec::new();
        for k in 0..DATUM_LIGHTS {
            let x = DATUM_FROM_M + k as f32 * DATUM_STEP_M;
            datum.push(Vec3::new(-x, 0.0, 0.0));
            datum.push(Vec3::new(x, 0.0, 0.0));
        }
        let waveoff: Vec<Vec3> = WAVEOFF_UP_M
            .iter()
            .flat_map(|&y| [Vec3::new(-WAVEOFF_OUT_M, y, 0.0), Vec3::new(WAVEOFF_OUT_M, y, 0.0)])
            .collect();
        let dots = |commands: &mut Commands, meshes: &mut Assets<Mesh>, pts: &[Vec3], c, t| {
            commands
                .spawn((
                    Mesh3d(meshes.add(dot_batch(pts))),
                    MeshMaterial3d(palette.line_material(c)),
                    tag(t),
                    Transform::default(),
                    Visibility::Inherited,
                    bevy::light::NotShadowCaster,
                ))
                .id()
        };
        let datum = dots(&mut commands, &mut meshes, &datum, GREEN, DOT_TENTHS);
        let ball = dots(&mut commands, &mut meshes, &[Vec3::ZERO], AMBER, BALL_TENTHS);
        let wave = dots(&mut commands, &mut meshes, &waveoff, RED, DOT_TENTHS);
        let root = commands
            .spawn((
                WorldPos(wu_to_bevy(east, north, DECK_WU)),
                Transform::default(),
                Visibility::Hidden,
            ))
            .add_children(&[datum, ball, wave])
            .id();
        lenses.lenses.push(Lens {
            carrier: b,
            root,
            ball,
            waveoff: wave,
            east,
            north,
            heading: l.heading,
            touchdown: (l.east, l.north),
        });
    }
}

fn despawn(mut commands: Commands, mut lenses: ResMut<Lenses>) {
    for l in lenses.lenses.drain(..) {
        commands.entity(l.root).try_despawn();
    }
    lenses.done = false;
}

/// Where the ball stands (m up from the datum, red) for an eye `dist` m short of the
/// touchdown point and `height` m above the deck.
fn ball_at(dist: f64, height: f64) -> (f32, bool) {
    let err = height.atan2(dist.max(1.0)).to_degrees() - GLIDE_DEG;
    let up = (err / BALL_DEG).clamp(-1.0, 1.0) as f32 * BALL_TRAVEL_M;
    (up, err < RED_BALL_DEG)
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn show(
    time: Res<Time>,
    lenses: Res<Lenses>,
    lso: Res<crate::lso::LsoState>,
    palette: Res<Palette>,
    theater: Option<Res<Theater>>,
    decks: Res<crate::sea::Decks>,
    cameras: Query<
        (&Camera, &GlobalTransform),
        (With<Camera3d>, Without<crate::camera::OffscreenCamera>),
    >,
    globals: Query<&GlobalTransform, Without<Camera3d>>,
    mut q: Query<
        (&mut Transform, &mut Visibility, Option<&mut MeshMaterial3d<PaletteMaterial>>),
        Without<Camera3d>,
    >,
    mut roots: Query<&mut WorldPos, Without<Camera3d>>,
) {
    // The main view (cockpit, external or free camera).
    let Some((_, eye)) = cameras.iter().find(|(c, _)| c.is_active) else { return };
    let eye = eye.translation();
    let deck_list = theater.as_deref().map_or(&[][..], |t| &t.decks[..]);
    let flash = (time.elapsed_secs() * WAVEOFF_HZ).fract() < 0.5;
    for l in &lenses.lenses {
        // The eye from the lens (m; Bevy: x east, y up, -z north), the lens's middle
        // `LENS_UP_M` above the deck.
        let Ok(at) = globals.get(l.root) else { continue };
        let d = (eye - at.translation()).as_dvec3();
        let (de, dn) = (d.x / WU, -d.z / WU);
        let dist = de.hypot(dn);
        let (s, c) = l.heading.sin_cos();
        let seen = dist * WU < RANGE_M as f64
            && dist * WU > 1.0
            && -(de * s + dn * c) > dist * COS_SPREAD as f64;
        if let Ok((_, mut vis, _)) = q.get_mut(l.root) {
            vis.set_if_neq(if seen { Visibility::Inherited } else { Visibility::Hidden });
        }
        // On the deck, riding the sea, facing the approach.
        let mut p = wu_to_bevy(l.east, l.north, DECK_WU);
        let turn = Quat::from_rotation_y(-l.heading as f32);
        let rot = match crate::sea::on_deck(deck_list, &decks.motion, p) {
            Some((lift, tilt)) => {
                p.y += lift;
                tilt * turn
            }
            None => turn,
        };
        p.y += LENS_UP_M as f64;
        if let Ok(mut pos) = roots.get_mut(l.root) {
            pos.0 = p;
        }
        if let Ok((mut tf, ..)) = q.get_mut(l.root) {
            tf.rotation = rot;
        }
        if !seen {
            continue;
        }
        // The eye short of the touchdown point along the approach, and above the deck (m).
        let (te, tn) = (l.touchdown.0 - l.east, l.touchdown.1 - l.north);
        let short = -((de - te) * s + (dn - tn) * c) * WU;
        let (up, red) = ball_at(short, d.y + LENS_UP_M as f64);
        if let Ok((mut tf, _, Some(mut m))) = q.get_mut(l.ball) {
            tf.translation.y = up;
            let want = palette.line_material(if red { RED } else { AMBER });
            if m.0 != want {
                m.0 = want;
            }
        }
        let waving = lso.waveoff.is_some_and(|(b, _)| b == l.carrier) && flash;
        if let Ok((_, mut vis, _)) = q.get_mut(l.waveoff) {
            vis.set_if_neq(if waving { Visibility::Inherited } else { Visibility::Hidden });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ball_rises_above_the_slope_and_reddens_low() {
        let on = 1000.0 * GLIDE_DEG.to_radians().tan();
        let (up, red) = ball_at(1000.0, on);
        assert!(up.abs() < 1e-3 && !red);
        let (up, red) = ball_at(1000.0, on + 10.0);
        assert!(up > 0.0 && up <= BALL_TRAVEL_M && !red);
        let (up, red) = ball_at(1000.0, on - 15.0);
        assert!(up < 0.0 && red);
        assert_eq!(ball_at(1000.0, 0.0).0, -BALL_TRAVEL_M);
    }
}

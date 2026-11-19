//! An aircraft on final approach, followed by a chase plane: to a field's runway
//! (`Shot::Approach`) and to a carrier's angled deck (`Shot::CarrierApproach`). The chase plane
//! holds station off its wing and drifts smoothly about it; a narrow lens shows the gear, the
//! flaps and the landing light close.

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;
use stealth_sim::catapult;

use super::director::{Shot, shot_on};
use super::stage::{self, Camera, Stage, enu};
use super::{SceneCamera, SceneFov, ShotEntity, ShotSet};
use crate::aircraft::AircraftKind as K;
use crate::models::ModelState;
use crate::world::{WU, WorldPos};

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (
            approach::<false>.run_if(shot_on(Shot::Approach)),
            approach::<true>.run_if(shot_on(Shot::CarrierApproach)),
        )
            .in_set(ShotSet),
    );
}

/// The aircraft that land on a runway, and on the carrier.
const FIELD: [K; 8] = [K::F16, K::FA18E, K::F22, K::A10, K::F117, K::Tornado, K::AV8B, K::C5];
const DECK: [K; 2] = [K::FA18E, K::EA18G];
/// Seconds to touchdown at the scene's start (it ends just before it; on the carrier a
/// little earlier, short of the deck's edge).
const TO_TOUCHDOWN: f32 = 5.6;
const TO_TRAP: f32 = 6.4;
/// The chase plane's lens (degrees high).
const LENS: f32 = 13.0;

/// The approach: the touchdown point (Bevy metres), the landing heading (rad), the glide
/// slope (rad), the speed (m/s) and the nose's angle over the path (rad); the aircraft and
/// its length.
#[derive(Component)]
struct Final {
    touchdown: DVec3,
    heading: f32,
    glide: f32,
    speed: f32,
    aoa: f32,
    craft: Entity,
    length: f32,
}

/// The approach speed and angle of attack of a type.
fn approach_of(kind: K) -> (f32, f32) {
    match kind {
        K::C5 => (68.0, 0.08),
        K::A10 => (62.0, 0.12),
        K::AV8B => (65.0, 0.12),
        K::F117 => (85.0, 0.14),
        K::FA18E | K::EA18G => (70.0, 0.14),
        _ => (75.0, 0.13),
    }
}

#[allow(clippy::too_many_arguments)]
fn approach<const CARRIER: bool>(
    mut stage: Stage,
    fin: Query<&Final>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    models: Query<&stage::Model>,
    (tables, landing): (Res<crate::flight::SimTables>, Res<crate::landing_lights::LandingAssets>),
    mut lamps: crate::landing_lights::Lights,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
) {
    let kinds: &[K] = if CARRIER { &DECK } else { &FIELD };
    let kind = kinds[stage.director.pick(kinds.len())];
    if stage.begin(&[stage::AIRCRAFT]) {
        let places = stage.places();
        let (touchdown, heading, glide) = if CARRIER {
            let Some(&obj) =
                places.deck_objects.get(stage.director.pick(places.deck_objects.len().max(1)))
            else {
                return;
            };
            let index = stage.theater.as_ref().map_or(0, |t| t.index);
            let ns = tables.start_dir.get(index).copied().unwrap_or(1) as i32;
            let l = catapult::landing(obj, ns);
            let at = DVec3::new(l.east * WU, catapult::DECK_WU * WU, -l.north * WU);
            (at, l.heading as f32, (catapult::GLIDE_DEG as f32).to_radians())
        } else {
            // One of the scenery's runway ends, else a field's strip: north and south, the
            // aim points 300 m in from its ends (the ATC's when the scenery's runways aren't
            // there), landed on from the scene's side.
            let ends = stage.runway_ends();
            let (aim, toward) = match ends.get(stage.director.pick(ends.len().max(1))) {
                Some(&e) => e,
                None => {
                    let n = places.airfields.len().max(1);
                    let field = places.airfields.get(stage.director.pick(n)).copied();
                    let field = field.unwrap_or(DVec2::splat(256_000.0));
                    let s = if (stage.director.roll >> 4) & 1 == 0 { 1.0 } else { -1.0 };
                    (field + DVec2::new(0.0, s * 700.0), DVec2::new(0.0, s))
                }
            };
            let at = enu(aim.x, 0.0, aim.y);
            let at = DVec3::new(at.x, stage.ground(at), at.z);
            // Landing the other way from the side the aircraft comes from.
            let heading = (-toward.x as f32).atan2(-toward.y as f32);
            (at, heading, 3f32.to_radians())
        };
        let spec = kind.spec();
        // The gear down, the flaps out, the lights on.
        let mut state = ModelState::default();
        (state.0[5], state.0[6], state.0[12]) = (1, 2, 1);
        state.0[crate::aircraft::FLAP_SLOT] = crate::aircraft::flap_slot_value(if kind == K::C5 {
            crate::aircraft::C5_FLAP_ANGLE
        } else {
            30f32.to_radians()
        });
        let Some(craft) = stage.model(stage::AIRCRAFT, spec.model, spec.model_lift, state) else {
            return;
        };
        let (speed, aoa) = approach_of(kind);
        stage.commands.spawn((
            ShotEntity,
            Final { touchdown, heading, glide, speed, aoa, craft, length: stage::length(kind) },
        ));
        return;
    }
    let Ok(f) = fin.single() else { return };
    let t = stage.director.t;
    // Down the glide slope, wings level but for a pilot's small corrections.
    let out = f.speed * (if CARRIER { TO_TRAP } else { TO_TOUCHDOWN } - t);
    let back = stage::forward(f.heading) * out;
    let p = f.touchdown + enu(-back.x as f64, (out * f.glide.tan()) as f64, -back.y as f64);
    let bank = (t * 0.9).sin() * 0.035 + (t * 2.3).sin() * 0.01;
    let rot = stage::attitude(f.heading + (t * 0.6).sin() * 0.01, f.aoa - f.glide, bank);
    stage::put(&mut q, f.craft, p, rot);
    // The landing light on the nose gear (once the model is there).
    if let Ok(m) = models.get(f.craft)
        && lamps.is_empty()
    {
        let at = crate::aircraft::landing_light(kind.spec().model_number());
        let scale = crate::player::AIRCRAFT_MODEL_SCALE as f32;
        let lamp =
            crate::landing_lights::spawn(&mut stage.commands, &landing, m.0, at, scale, false);
        stage.commands.entity(lamp).insert(Visibility::Inherited);
    }
    for mut v in &mut lamps {
        v.set_if_neq(Visibility::Inherited);
    }
    // The chase plane: off the right wing (the nose and its light towards the free side of
    // the screen; at the carrier off the left one, clear of the island, further behind), a
    // little above, drifting slowly about its station (each axis its own slow swing).
    let k = f.length / 18.0;
    let (side, behind) = if CARRIER { (-30.0, 62.0) } else { (44.0, 52.0) };
    let side = (side + 7.0 * (t * 0.45).sin()) * k;
    let behind = (behind + 9.0 * (t * 0.31 + 1.7).sin()) * k;
    let up = (5.0 + 3.0 * (t * 0.53 + 0.6).sin()) * k;
    let eye = stage::behind(p, f.heading, behind, side, up);
    // On its middle: the whole aircraft, its nose and the light's beam ahead in the picture.
    stage::look(&mut cam, eye, p);
    fov.0 = LENS.to_radians();
}

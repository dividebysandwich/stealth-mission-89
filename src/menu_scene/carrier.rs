//! The carrier seen against the sun (by night the moon): a Super Hornet shot off a bow
//! catapult, another trapping on the angled deck, an MH-60R lifting off its spot
//! (`Shot::Carrier`).

use bevy::math::DVec3;
use bevy::prelude::*;
use stealth_sim::catapult;

use super::director::{Shot, shot_on};
use super::stage::{self, Camera, Stage, enu};
use super::{SceneCamera, SceneFov, ShotEntity, ShotSet};
use crate::aircraft::AircraftKind;
use crate::models::ModelState;
use crate::world::{WU, WorldPos};

pub(super) fn build(app: &mut App) {
    app.add_systems(Update, carrier.in_set(ShotSet).run_if(shot_on(Shot::Carrier)));
}

/// The launch: the stroke's start and length (s), the speed it ends at (m/s), the climb out.
const SHOOT_AT: f32 = 0.6;
const STROKE: f32 = 2.2;
const END_SPEED: f32 = 75.0;
const CLIMB: f32 = 0.14;
/// The landing: touchdown (s into the scene), the approach speed (m/s) and the roll out (m).
const TOUCHDOWN_AT: f32 = 2.4;
const APPROACH: f32 = 65.0;
const ROLL_OUT: f32 = 90.0;
/// The helicopter's lift-off (s) and climb (m/s), its turn (rad/s).
const LIFT_AT: f32 = 1.0;
const HELI_CLIMB: f32 = 4.0;
const HELI_TURN: f32 = 0.25;

/// What flies off and onto the deck, and where.
#[derive(Component)]
struct Deck {
    /// The ship's middle and its deck's height (Bevy metres).
    middle: DVec3,
    deck_y: f64,
    launch: catapult::CatPose,
    landing: catapult::LandingPose,
    /// The helicopter's spot (wu east, north) and the ship's bow heading (rad).
    spot: (f64, f64),
    bow: f64,
    jets: [Entity; 2],
    heli: Entity,
}

fn wu_point(e: f64, n: f64, y: f64) -> DVec3 {
    DVec3::new(e * WU, y, -n * WU)
}

#[allow(clippy::too_many_arguments)]
fn carrier(
    mut stage: Stage,
    deck: Query<&Deck>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut states: Query<&mut ModelState>,
    models: Query<&stage::Model>,
    tables: Res<crate::flight::SimTables>,
    sky: Res<crate::sky::Sky>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
) {
    if stage.begin(&[stage::AIRCRAFT]) {
        let places = stage.places();
        let Some(&obj) =
            places.deck_objects.get(stage.director.pick(places.deck_objects.len().max(1)))
        else {
            return;
        };
        let index = stage.theater.as_ref().map_or(0, |t| t.index);
        let ns = tables.start_dir.get(index).copied().unwrap_or(1) as i32;
        let (pe, pn) = catapult::pivot(obj.0, obj.1);
        let deck_y = catapult::DECK_WU * WU;
        let launch = catapult::pose(obj, ns, 0);
        let landing = catapult::landing(obj, ns);
        let spot = catapult::deck_point(obj, ns, catapult::PARKING[0]);
        let bow = if ns < 0 { std::f64::consts::PI } else { 0.0 };
        let mut on_deck = ModelState::default();
        (on_deck.0[5], on_deck.0[6], on_deck.0[12]) = (1, 2, 1);
        on_deck.0[crate::aircraft::FLAP_SLOT] =
            crate::aircraft::flap_slot_value(30f32.to_radians());
        let spec = AircraftKind::FA18E.spec();
        let a = stage.model(stage::AIRCRAFT, spec.model, spec.model_lift, on_deck.clone());
        let b = stage.model(stage::AIRCRAFT, spec.model, spec.model_lift, on_deck.clone());
        let h = AircraftKind::MH60R.spec();
        let mut heli_state = ModelState::default();
        heli_state.0[12] = 1;
        let c = stage.model(stage::AIRCRAFT, h.model, h.model_lift, heli_state);
        let (Some(a), Some(b), Some(c)) = (a, b, c) else { return };
        stage.commands.spawn((
            ShotEntity,
            Deck {
                middle: wu_point(pe, pn, 0.0),
                deck_y,
                launch,
                landing,
                spot,
                bow,
                jets: [a, b],
                heli: c,
            },
        ));
        return;
    }
    let Ok(d) = deck.single() else { return };
    let t = stage.director.t;
    // The launch: held, the stroke, the climb out with the gear coming up.
    let h = d.launch.heading_rad();
    let s = if t < SHOOT_AT {
        0.0
    } else if t < SHOOT_AT + STROKE {
        let tau = t - SHOOT_AT;
        0.5 * END_SPEED / STROKE * tau * tau
    } else {
        let tau = t - SHOOT_AT - STROKE;
        0.5 * END_SPEED * STROKE + END_SPEED * tau
    };
    let climbing = (t - SHOOT_AT - STROKE).max(0.0);
    let (e, n) = d.launch.along(s as f64 / WU);
    let rise = (END_SPEED * climbing * CLIMB.sin()) as f64;
    let pitch = if climbing > 0.0 { CLIMB.min(climbing * 0.3) } else { 0.0 };
    stage::put(
        &mut q,
        d.jets[0],
        wu_point(e, n, d.deck_y + rise),
        stage::attitude(h as f32, pitch, 0.0),
    );
    if climbing > 1.0
        && let Ok(m) = models.get(d.jets[0])
        && let Ok(mut st) = states.get_mut(m.0)
    {
        (st.0[5], st.0[6], st.0[13]) = (0, 0, 1);
    }
    // The landing: down the glide slope to the wire, then the roll out to a stop.
    let lh = d.landing.heading;
    let (out, up) = if t < TOUCHDOWN_AT {
        let out = (APPROACH * (TOUCHDOWN_AT - t)) as f64;
        (out, out * catapult::GLIDE_DEG.to_radians().tan())
    } else {
        let tau = t - TOUCHDOWN_AT;
        let a = APPROACH * APPROACH / (2.0 * ROLL_OUT);
        let stop = APPROACH / a;
        let tau = tau.min(stop);
        (-((APPROACH * tau - 0.5 * a * tau * tau) as f64), 0.0)
    };
    let back = out / WU;
    let (e, n) = (d.landing.east - lh.sin() * back, d.landing.north - lh.cos() * back);
    let flare = if t < TOUCHDOWN_AT { 0.07 } else { 0.0 };
    stage::put(
        &mut q,
        d.jets[1],
        wu_point(e, n, d.deck_y + up),
        stage::attitude(lh as f32, flare, 0.0),
    );
    // The helicopter lifts off and turns away over the side.
    let lift = (t - LIFT_AT).max(0.0);
    let hp = wu_point(d.spot.0, d.spot.1, d.deck_y + (HELI_CLIMB * lift) as f64)
        + enu((lift * lift * 0.8) as f64, 0.0, 0.0);
    let hh = d.bow as f32 - 1.2 + HELI_TURN * lift;
    stage::put(&mut q, d.heli, hp, stage::attitude(hh, -0.05 * lift.min(1.0), 0.0));
    // Against the sun, or the moon by night: the ship between the camera and the light.
    let light = if sky.sun.y > -0.05 { sky.sun } else { sky.moon };
    let flat = Vec2::new(light.x, -light.z).normalize_or(Vec2::Y);
    let side = Vec2::new(flat.y, -flat.x);
    // Between the catapult and the wire, a little over the deck: both aircraft in view.
    let cat = wu_point(d.launch.east, d.launch.north, d.deck_y);
    let wire = wu_point(d.landing.east, d.landing.north, d.deck_y);
    let focus = (cat + wire) * 0.5 + DVec3::Y * 8.0;
    let off = -flat * 250.0 + side * 70.0;
    let eye = focus + enu(off.x as f64, 75.0, off.y as f64);
    stage::look(&mut cam, eye, focus);
    fov.0 = 40f32.to_radians();
}

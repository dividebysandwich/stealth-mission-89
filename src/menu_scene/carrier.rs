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
    app.add_systems(
        Update,
        (carrier.run_if(shot_on(Shot::Carrier)), launch.run_if(shot_on(Shot::Launch)))
            .in_set(ShotSet),
    );
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
/// The helicopter's spot (m, nimitz.blend as `catapult::PARKING`): starboard, aft of the island
/// (which stands at about 18 to 37 m starboard, 39 to 84 m aft) and clear of the angled deck.
const HELI_SPOT: (f64, f64) = (22.0, -105.0);

/// What flies off and onto the deck, and where.
#[derive(Component)]
struct Deck {
    /// The deck's height (Bevy metres).
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
        let deck_y = catapult::DECK_WU * WU;
        let launch = catapult::pose(obj, ns, 0);
        let landing = catapult::landing(obj, ns);
        let spot = catapult::deck_point(obj, ns, HELI_SPOT);
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
            Deck { deck_y, launch, landing, spot, bow, jets: [a, b], heli: c },
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
    // The helicopter lifts off and turns away over the starboard side (away from the island).
    let lift = (t - LIFT_AT).max(0.0);
    let (bs, bc) = d.bow.sin_cos();
    let side = (lift * lift * 0.8) as f64;
    let hp = wu_point(d.spot.0, d.spot.1, d.deck_y + (HELI_CLIMB * lift) as f64)
        + enu(bc * side, 0.0, -bs * side);
    let hh = d.bow as f32 + 1.2 + HELI_TURN * lift;
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

/// The launch seen from behind the shooter: the run-up (his hand circling, the catapult's
/// steam), the lunge, the shot (s into the scene).
const LUNGE_AT: f32 = 2.3;
const STROKE_AT: f32 = 3.0;
/// The shooter's pose and hand slots, his hand's turns a second, its frames
/// (`catapult_crew.rs`); where he stands from the hold-back point (m ahead, out right).
const POSE_SLOT: usize = 23;
const HAND_SLOT: usize = 24;
const HAND_TURNS: f32 = 1.6;
const HAND_FRAMES: f32 = 16.0;
const SHOOTER_AHEAD: f64 = 5.0;
const SHOOTER_ABEAM: f64 = 9.0;

/// The launch: the catapult, the jet and the shooter.
#[derive(Component)]
struct Launch {
    cat: catapult::CatPose,
    deck_y: f64,
    jet: Entity,
    shooter: Entity,
    shooter_at: DVec3,
}

#[allow(clippy::too_many_arguments)]
fn launch(
    mut stage: Stage,
    launch: Query<&Launch>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut states: Query<&mut ModelState>,
    models: Query<&stage::Model>,
    tables: Res<crate::flight::SimTables>,
    mut ex: ResMut<crate::explosions::Explosions>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    mut steam: Local<f32>,
    time: Res<Time>,
) {
    if stage.begin(&[stage::AIRCRAFT, stage::AAA]) {
        let places = stage.places();
        let Some(&obj) =
            places.deck_objects.get(stage.director.pick(places.deck_objects.len().max(1)))
        else {
            return;
        };
        let index = stage.theater.as_ref().map_or(0, |t| t.index);
        let ns = tables.start_dir.get(index).copied().unwrap_or(1) as i32;
        let cat = catapult::pose(obj, ns, 0);
        let deck_y = catapult::DECK_WU * WU;
        let kind =
            if stage.director.roll & 1 == 0 { AircraftKind::FA18E } else { AircraftKind::EA18G };
        let spec = kind.spec();
        let mut state = ModelState::default();
        (state.0[5], state.0[6], state.0[12]) = (1, 2, 1);
        state.0[crate::aircraft::FLAP_SLOT] = crate::aircraft::flap_slot_value(30f32.to_radians());
        let Some(jet) = stage.model(stage::AIRCRAFT, spec.model, spec.model_lift, state) else {
            return;
        };
        // The shooter on the right of the catapult's track (catapult 1's side), across it
        // facing the jet.
        let h = cat.heading_rad();
        let (fe, fnn) = (h.sin(), h.cos());
        let (re, rn) = (fnn, -fe);
        let east = cat.east + (fe * SHOOTER_AHEAD + re * SHOOTER_ABEAM) / WU;
        let north = cat.north + (fnn * SHOOTER_AHEAD + rn * SHOOTER_ABEAM) / WU;
        let shooter_at = wu_point(east, north, deck_y);
        let facing = (h - std::f64::consts::FRAC_PI_2) as f32;
        let Some(shooter) = stage.model(stage::AAA, "aaa_124", 0.0, ModelState::default()) else {
            return;
        };
        stage.commands.entity(shooter).insert((
            WorldPos(shooter_at),
            Transform::from_rotation(Quat::from_rotation_y(-facing)),
        ));
        stage.commands.spawn((ShotEntity, Launch { cat, deck_y, jet, shooter, shooter_at }));
        return;
    }
    let Ok(l) = launch.single() else { return };
    let t = stage.director.t;
    let h = l.cat.heading_rad();
    // Held back, then the stroke and the climb out (as the carrier scene's).
    let tau = (t - STROKE_AT).max(0.0);
    let s = if tau < STROKE {
        0.5 * END_SPEED / STROKE * tau * tau
    } else {
        0.5 * END_SPEED * STROKE + END_SPEED * (tau - STROKE)
    };
    let climbing = (tau - STROKE).max(0.0);
    let (e, n) = l.cat.along(s as f64 / WU);
    let rise = (END_SPEED * climbing * CLIMB.sin()) as f64;
    let pitch = if climbing > 0.0 { CLIMB.min(climbing * 0.3) } else { 0.0 };
    let jet_at = wu_point(e, n, l.deck_y + rise);
    stage::put(&mut q, l.jet, jet_at, stage::attitude(h as f32, pitch, 0.0));
    // The shooter: the run-up signal, his hand circling over his head; then the lunge.
    if let Ok(m) = models.get(l.shooter)
        && let Ok(mut st) = states.get_mut(m.0)
    {
        let pose = if t < LUNGE_AT { 1 } else { 2 };
        let hand = (t * HAND_TURNS).fract() * HAND_FRAMES;
        (st.0[POSE_SLOT], st.0[HAND_SLOT]) = (pose, (hand as i16).min(HAND_FRAMES as i16 - 1));
    }
    if climbing > 1.0
        && let Ok(m) = models.get(l.jet)
        && let Ok(mut st) = states.get_mut(m.0)
    {
        (st.0[5], st.0[6], st.0[13]) = (0, 0, 1);
    }
    // Steam off the catapult's track while it is tensioned, a burst at the shot.
    *steam -= time.delta_secs();
    if t < STROKE_AT + 0.6 && *steam <= 0.0 {
        *steam = if t < STROKE_AT { 0.12 } else { 0.03 };
        let k = (t * 13.7).sin() as f64;
        let (se, sn) = l.cat.along((4.0 + 6.0 * k.abs()) / WU);
        let at = wu_point(se, sn, l.deck_y + 0.3);
        ex.pending.push((at, crate::explosions::Blast::Pop { size: 1.6, shade: 0.95 }));
    }
    // Behind the shooter, low, looking past him at the jet; after the shot it follows the jet.
    let from_jet = (l.shooter_at - wu_point(l.cat.east, l.cat.north, l.deck_y)).normalize();
    let eye = l.shooter_at + from_jet * 7.0 + DVec3::Y * 1.9;
    let hold = wu_point(l.cat.east, l.cat.north, l.deck_y) + DVec3::Y * 2.0;
    let k = ((t - STROKE_AT) / 1.2).clamp(0.0, 1.0) as f64;
    stage::look(&mut cam, eye, hold + (jet_at + DVec3::Y * 2.0 - hold) * k);
    fov.0 = 46f32.to_radians();
}

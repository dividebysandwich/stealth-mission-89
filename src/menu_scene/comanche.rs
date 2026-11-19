//! The Comanche's cockpit, close on a screen: the signature page while it weaves low through
//! the SAMs' country (`Shot::Rcs`), and the targeting pod's picture as the gun fires at a
//! vehicle (`Shot::PodGun`).

use bevy::math::DVec3;
use bevy::prelude::*;

use super::cockpits::{Cabin, eye_camera, spawn_cabin};
use super::director::{Shot, shot_on};
use super::fx::{self, Shell};
use super::pod::{self, PodCamera};
use super::stage::{self, Camera, Stage, Weave, enu};
use super::{SceneCamera, SceneFov, ShotEntity, ShotSet};
use crate::aircraft::AircraftKind;
use crate::cockpit3d::scene::{SceneScreens, Screen, Seat};
use crate::explosions::{Blast, Calibre, Explosions};
use crate::mfd::MfdSide;
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (rcs.run_if(shot_on(Shot::Rcs)), pod_gun.run_if(shot_on(Shot::PodGun))).in_set(ShotSet),
    );
}

/// The scene's flight (its cabin's), and for the gun its target.
#[derive(Component)]
struct Run {
    weave: Weave,
    target: DVec3,
    fired: usize,
}

/// Looks from the cabin's eye at its screen `s`, a little above its middle (the canopy's view
/// over it), with a field of view `fov` degrees high.
#[allow(clippy::too_many_arguments)]
pub(super) fn close_on(
    cam: &mut Camera,
    fov: &mut SceneFov,
    screens: &SceneScreens,
    cabin: &Cabin,
    cabin_at: &GlobalTransform,
    p: DVec3,
    rot: Quat,
    s: Screen,
    deg: f32,
) {
    // The screen in the cabin's frame (fixed in it), then as seen from the eye.
    let look = screens.find(s).map_or(Vec2::new(0.0, -0.4), |(_, g)| {
        let local = cabin_at.affine().inverse().transform_point3(g.translation()) - cabin.eye;
        let up = 0.012;
        let local = local + Vec3::Y * up;
        let yaw = (-local.x).atan2(-local.z);
        let pitch = local.y.atan2(local.xz().length());
        Vec2::new(yaw, pitch)
    });
    eye_camera(cam, cabin, p, rot, look);
    fov.0 = deg.to_radians();
}

/// The Comanche low over the SAMs' country, weaving; the view close on its left screen's
/// signature page, the land streaming past over it.
#[allow(clippy::too_many_arguments)]
fn rcs(
    mut stage: Stage,
    mut screens: SceneScreens,
    run: Query<&Run>,
    mut cabins: Query<
        (&Cabin, &mut WorldPos, &mut Transform, &GlobalTransform),
        Without<SceneCamera>,
    >,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
) {
    if stage.begin(&[]) {
        let places = stage.places();
        let start = match places.sams.get(stage.director.pick(places.sams.len().max(1))) {
            Some(&(p, _)) => {
                let c = enu(p.x, 0.0, p.y);
                DVec3::new(c.x, stage.ground(c), c.z)
            }
            None => stage.land(),
        };
        let heading = (stage.director.roll % 628) as f32 * 0.01;
        let back = stage::forward(heading) * 4000.0;
        let weave = Weave {
            start: start - enu(back.x as f64, -30.0, back.y as f64),
            heading,
            speed: 65.0,
            amp: Weave::amp_for(5.0, 30.0),
            period: 5.0,
            phase: 0.0,
            climb: 0.0,
        };
        spawn_cabin(&mut stage, Cabin::new(Seat::Cockpit(AircraftKind::RAH66), None));
        stage.commands.spawn((ShotEntity, Run { weave, target: start, fired: 0 }));
        return;
    }
    let t = stage.director.t;
    let (Ok(r), Ok((cabin, mut wp, mut tf, gt))) = (run.single(), cabins.single_mut()) else {
        return;
    };
    let (mut p, rot, _) = r.weave.at(t);
    p.y = stage.ground(p) + 30.0;
    (wp.0, tf.rotation) = (p, rot);
    if cabin.fitted.is_none() {
        return;
    }
    if let Some(mut tex) = screens.canvas(Screen::Left) {
        tex.canvas.clear(0);
        super::cockpits::draw_rcs(&mut tex.canvas, t);
        tex.dirty = true;
    }
    if let Some(mut tex) = screens.canvas(Screen::Right) {
        tex.canvas.clear(0);
        crate::eng_page::draw(
            &mut tex.canvas,
            AircraftKind::RAH66,
            &super::cockpits::cruise_engines(AircraftKind::RAH66, t),
        );
        tex.dirty = true;
    }
    close_on(&mut cam, &mut fov, &screens, cabin, gt, p, rot, Screen::Left, 34.0);
}

/// Rounds of the 20 mm a second, the burst's start and end.
const ROUNDS_PER_S: f32 = 12.0;
const BURST: (f32, f32) = (1.6, 3.4);

/// The Comanche hovering low, its pod's picture on the left screen locked on a vehicle; the
/// gun fires and the rounds burst round it in the picture.
#[allow(clippy::too_many_arguments)]
fn pod_gun(
    mut stage: Stage,
    mut screens: SceneScreens,
    mut run: Query<&mut Run>,
    mut cabins: Query<
        (&Cabin, &mut WorldPos, &mut Transform, &GlobalTransform),
        (Without<SceneCamera>, Without<PodCamera>),
    >,
    mut pods: Query<(&mut WorldPos, &mut Transform), (With<PodCamera>, Without<SceneCamera>)>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    palette: Res<crate::palette::Palette>,
    (mut ex, mut materials): (
        ResMut<Explosions>,
        ResMut<Assets<crate::palette_material::PaletteMaterial>>,
    ),
    mut tracer: Local<
        Option<(
            Handle<crate::palette_material::PaletteMaterial>,
            AssetId<bevy::render::storage::ShaderBuffer>,
        )>,
    >,
) {
    if stage.begin(&[stage::AAA]) {
        let ground = stage.land();
        let heading = (stage.director.roll % 628) as f32 * 0.01;
        let f = stage::forward(heading) * 1200.0;
        let target = ground + enu(f.x as f64, 0.0, f.y as f64);
        let target = DVec3::new(target.x, stage.ground(target), target.z);
        for (i, name) in ["aaa_120", "aaa_117"].into_iter().enumerate() {
            let side = stage::right(heading) * (i as f32 * 14.0);
            let at = target + enu(side.x as f64, 0.0, side.y as f64);
            let at = DVec3::new(at.x, stage.ground(at), at.z);
            if let Some(e) =
                stage.model(stage::AAA, name, 0.0, crate::models::ModelState::default())
            {
                stage.commands.entity(e).insert((
                    WorldPos(at),
                    Transform::from_rotation(Quat::from_rotation_y(-heading + 1.2)),
                ));
            }
        }
        let weave = Weave {
            start: ground + DVec3::Y * (stage.ground(ground) - ground.y + 25.0),
            heading,
            speed: 0.0,
            amp: 0.0,
            period: 5.0,
            phase: 0.0,
            climb: 0.0,
        };
        let cabin = spawn_cabin(&mut stage, Cabin::new(Seat::Cockpit(AircraftKind::RAH66), None));
        stage.commands.entity(cabin).insert(pod::WantsPod((480, 360), 2.5f32.to_radians()));
        stage
            .commands
            .spawn((ShotEntity, Run { weave, target: target + DVec3::Y * 1.5, fired: 0 }));
        return;
    }
    let t = stage.director.t;
    let (Ok(mut r), Ok((cabin, mut wp, mut tf, gt))) = (run.single_mut(), cabins.single_mut())
    else {
        return;
    };
    // A hover's small drift.
    let (mut p, _, _) = r.weave.at(t);
    p += DVec3::new(
        (t * 0.9).sin() as f64 * 0.4,
        (t * 1.3).sin() as f64 * 0.2,
        (t * 0.7).cos() as f64 * 0.3,
    );
    let rot =
        stage::attitude(r.weave.heading, 0.02 + (t * 0.8).sin() * 0.01, (t * 1.1).sin() * 0.015);
    (wp.0, tf.rotation) = (p, rot);
    // The pod under the nose, on the target.
    let nose = p + (rot * Vec3::new(0.0, -0.8, -5.0)).as_dvec3();
    pod::aim_pod(&mut pods, nose, r.target);
    // The gun under the nose: rounds at the target, a little spread.
    if (BURST.0..BURST.1).contains(&t) {
        let due = ((t - BURST.0) * ROUNDS_PER_S) as usize;
        let material = fx::tracer_material(&mut tracer, &mut materials, &palette);
        while r.fired <= due {
            r.fired += 1;
            let k = r.fired as f64;
            let spread = DVec3::new((k * 1.7).sin() * 5.0, 0.0, (k * 2.3).cos() * 5.0);
            let range = (r.target - nose).length();
            let speed = 1030.0;
            let tof = range / speed;
            let aim = r.target + spread + DVec3::Y * (0.5 * 9.81 * tof * tof);
            let vel = (aim - nose).normalize() * speed;
            let shell = Shell {
                vel,
                calibre: Calibre::Cannon,
                streak: 0.03,
                glow: None,
                age: 0.0,
                fuse: None,
            };
            fx::fire(
                &mut stage.commands,
                material.clone(),
                &mut ex,
                nose,
                Vec3::ZERO,
                shell,
                r.fired % 3 == 0,
            );
        }
    }
    if (BURST.1 - 0.3..BURST.1 - 0.25).contains(&t) {
        ex.pending.push((r.target, Blast::Kill));
    }
    if cabin.fitted.is_none() {
        return;
    }
    screens.picture(MfdSide::Left, true);
    let range = (r.target - nose).length();
    let firing = (BURST.0..BURST.1).contains(&t);
    if let Some(mut tex) = screens.canvas(Screen::Left) {
        let rounds = format!("{} RNDS", 500usize.saturating_sub(r.fired * 5));
        pod::overlay(
            &mut tex.canvas,
            "TADS",
            if firing { "GUN" } else { "TRK" },
            true,
            (&format!("RNG {:.1}", range / 1000.0), &rounds),
        );
        tex.transparent = true;
        tex.dirty = true;
    }
    if let Some(mut tex) = screens.canvas(Screen::Right) {
        tex.canvas.clear(0);
        crate::eng_page::draw(
            &mut tex.canvas,
            AircraftKind::RAH66,
            &super::cockpits::cruise_engines(AircraftKind::RAH66, t),
        );
        tex.dirty = true;
    }
    close_on(&mut cam, &mut fov, &screens, cabin, gt, p, rot, Screen::Left, 30.0);
}

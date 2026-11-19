//! The Super Hornet's cockpit, close on its left screen: the SLAM-ER's seeker picture, locked
//! on its target, closing on it in its last seconds (`Shot::Slam`).

use bevy::math::DVec3;
use bevy::prelude::*;

use super::cockpits::{Cabin, spawn_cabin};
use super::comanche::close_on;
use super::director::{Shot, shot_on};
use super::pod::{self, PodCamera};
use super::stage::{self, Camera, Stage, Weave, enu};
use super::{SceneCamera, SceneFov, ShotEntity, ShotSet};
use crate::aircraft::AircraftKind;
use crate::cockpit3d::scene::{SceneScreens, Screen, Seat};
use crate::explosions::{Blast, Explosions};
use crate::mfd::MfdSide;
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(Update, slam.in_set(ShotSet).run_if(shot_on(Shot::Slam)));
}

/// The missile's speed (m/s, `stealth_sim::slam::SPEED`'s) and how far out it starts (m); its
/// dive onto the target (rad).
const SPEED: f64 = 240.0;
const START: f64 = 1150.0;
const DIVE: f64 = 0.16;

/// The scene: the jet's flight, the target and the missile's line to it.
#[derive(Component)]
struct Run {
    weave: Weave,
    target: DVec3,
    from: DVec3,
    hit: bool,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn slam(
    mut stage: Stage,
    mut screens: SceneScreens,
    hud: super::cockpits::HudKit,
    mut run: Query<&mut Run>,
    mut cabins: Query<
        (&Cabin, &mut WorldPos, &mut Transform, &GlobalTransform),
        (Without<SceneCamera>, Without<PodCamera>),
    >,
    mut pods: pod::Pods,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    mut ex: ResMut<Explosions>,
) {
    if stage.begin(&[stage::AAA]) {
        // A SAM site of the theater, else a vehicle of our own on open land.
        let places = stage.places();
        let target = match places.sams.get(stage.director.pick(places.sams.len().max(1))) {
            Some(&(p, _)) => {
                let c = enu(p.x, 0.0, p.y);
                DVec3::new(c.x, stage.ground(c), c.z)
            }
            None => {
                let c = stage.land();
                if let Some(e) =
                    stage.model(stage::AAA, "aaa_110", 0.0, crate::models::ModelState::default())
                {
                    stage.commands.entity(e).insert(WorldPos(c));
                }
                c
            }
        };
        let heading = (stage.director.roll % 628) as f32 * 0.01;
        let back = stage::forward(heading);
        let from =
            target + enu(-(back.x as f64) * START, START * DIVE.tan(), -(back.y as f64) * START);
        // The jet a few kilometres off (the land streams round the view), turning away.
        let jet = target + enu(-(back.x as f64) * 3500.0, 1500.0, -(back.y as f64) * 3500.0);
        let weave = Weave {
            start: jet,
            heading: heading + 2.6,
            speed: 230.0,
            amp: Weave::amp_for(12.0, 25.0),
            period: 12.0,
            phase: 0.0,
            climb: 0.0,
        };
        let cabin = spawn_cabin(&mut stage, Cabin::new(Seat::Cockpit(AircraftKind::FA18E), None));
        stage.commands.entity(cabin).insert(pod::WantsPod((480, 360), 9f32.to_radians()));
        stage.commands.spawn((ShotEntity, Run { weave, target, from, hit: false }));
        return;
    }
    let t = stage.director.t;
    let (Ok(mut r), Ok((cabin, mut wp, mut tf, gt))) = (run.single_mut(), cabins.single_mut())
    else {
        return;
    };
    let (p, rot, _) = r.weave.at(t);
    (wp.0, tf.rotation) = (p, rot);
    // The seeker rides the missile down its line; at the end it is there.
    let to = r.target - r.from;
    let gone = (SPEED * t as f64).min(to.length() - 5.0);
    let seeker = r.from + to.normalize() * gone;
    let range = (r.target - seeker).length();
    pod::aim_pod(&mut pods, seeker, r.target + DVec3::Y * 2.0);
    if range < 8.0 && !r.hit {
        r.hit = true;
        ex.pending.push((r.target, Blast::Kill));
    }
    if cabin.fitted.is_none() {
        return;
    }
    screens.picture(MfdSide::Left, !r.hit);
    if let Some(mut tex) = screens.canvas(Screen::Left) {
        if r.hit {
            // The picture gone with the missile.
            tex.canvas.clear(0);
            pod::overlay(&mut tex.canvas, "SLAM A", "IMPACT", false, ("RNG 0.0", "TTI 0"));
            tex.transparent = false;
        } else {
            let rng = format!("RNG {:.1}", range / 1852.0);
            let tti = format!("TTI {:.0}", (range / SPEED).ceil());
            pod::overlay(&mut tex.canvas, "SLAM A", "LOCK", true, (&rng, &tti));
            tex.transparent = true;
        }
        tex.dirty = true;
    }
    if let Some(mut tex) = screens.canvas(Screen::Right) {
        tex.canvas.clear(0);
        let eng = super::cockpits::cruise_engines(AircraftKind::FA18E, t);
        crate::eng_page::draw(&mut tex.canvas, AircraftKind::FA18E, &eng);
        tex.dirty = true;
    }
    let v = (rot * Vec3::NEG_Z) * r.weave.speed;
    let alt = (p.y - stage.ground(p)) as f32;
    let mode = crate::hud::HudMode::Gnd;
    hud.draw(&mut screens, AircraftKind::FA18E, rot, v, alt, mode, Some((1, "SLAM")), t);
    close_on(&mut cam, &mut fov, &screens, cabin, gt, p, rot, Screen::Left, 26.0);
}

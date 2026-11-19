//! One of the enemy's SAM sites of the theater, its launchers and radar, the camera circling it
//! low; a missile leaves a rail and climbs away (`Shot::SamSite`).

use bevy::math::DVec3;
use bevy::prelude::*;

use super::director::{Shot, shot_on};
use super::fx::{self, Aim, Missile};
use super::stage::{self, Camera, Stage, enu};
use super::{SceneFov, ShotEntity, ShotSet};
use crate::explosions::Blast;

pub(super) fn build(app: &mut App) {
    app.add_systems(Update, sam_site.in_set(ShotSet).run_if(shot_on(Shot::SamSite)));
}

/// The camera's circle round the site: radius and height (m), how far round it goes in the
/// scene (radians).
const RADIUS: f64 = 150.0;
const HEIGHT: f64 = 35.0;
const SWEEP: f32 = 0.6;
/// When the missile goes.
const LAUNCH_AT: f32 = 1.6;

/// The scene: the site's centre, where the camera starts round it, whether the missile went.
#[derive(Component)]
struct Site {
    centre: DVec3,
    start: f32,
    launched: bool,
}

fn sam_site(
    mut stage: Stage,
    mut site: Query<&mut Site>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
) {
    if stage.begin(&[stage::AIRCRAFT, stage::AAA]) {
        let places = stage.places();
        let start = (stage.director.roll % 628) as f32 * 0.01;
        let centre = match places.sams.get(stage.director.pick(places.sams.len().max(1))) {
            Some(&(p, _)) => {
                let c = enu(p.x, 0.0, p.y);
                DVec3::new(c.x, stage.ground(c), c.z)
            }
            None => {
                // No site in this theater: a gun and missile position of our own.
                let c = stage.land();
                for (i, name) in ["aaa_110", "aaa_112", "aaa_111"].into_iter().enumerate() {
                    let a = i as f32 * 2.1;
                    let at = c + enu((a.cos() * 40.0) as f64, 0.0, (a.sin() * 40.0) as f64);
                    let at = DVec3::new(at.x, stage.ground(at), at.z);
                    if let Some(e) =
                        stage.model(stage::AAA, name, 0.0, crate::models::ModelState::default())
                    {
                        stage.commands.entity(e).insert(crate::world::WorldPos(at));
                    }
                }
                c
            }
        };
        stage.commands.spawn((ShotEntity, Site { centre, start, launched: false }));
        return;
    }
    let t = stage.director.t;
    let Ok(mut s) = site.single_mut() else { return };
    if t >= LAUNCH_AT && !s.launched {
        s.launched = true;
        // Off a rail at the site's edge, steep, at a point high and far.
        let a = s.start + 2.4;
        let from = s.centre + enu((a.cos() * 30.0) as f64, 3.0, (a.sin() * 30.0) as f64);
        let up = Vec3::new(a.cos() * 0.3, 0.8, -a.sin() * 0.3).normalize();
        let aim = from + (up * 20_000.0).as_dvec3() + DVec3::Y * 4000.0;
        let m = Missile {
            vel: up * 40.0,
            accel: 160.0,
            top: 900.0,
            turn: 0.3,
            straight: 0.3,
            aim: Aim::Point(aim),
            blast: Blast::Flak,
            age: 0.0,
        };
        fx::launch(&mut stage, "aircraft_015", from, up * 40.0, m);
    }
    let a = s.start + SWEEP * t / super::director::SHOT_SECONDS;
    let eye = s.centre + enu(a.cos() as f64 * RADIUS, HEIGHT, a.sin() as f64 * RADIUS);
    // A little over the site's middle, so the missile's climb shows.
    let rise = ((t - LAUNCH_AT).max(0.0) * 12.0) as f64;
    stage::look(&mut cam, eye, s.centre + DVec3::Y * (8.0 + rise));
    fov.0 = 45f32.to_radians();
}

//! One of the enemy's SAM sites of the theater, its launchers and radar, the camera circling it
//! low (`Shot::SamSite`).

use bevy::math::DVec3;
use bevy::prelude::*;

use super::director::{Shot, shot_on};
use super::stage::{self, Camera, Stage, enu};
use super::{SceneFov, ShotEntity, ShotSet};

pub(super) fn build(app: &mut App) {
    app.add_systems(Update, sam_site.in_set(ShotSet).run_if(shot_on(Shot::SamSite)));
}

/// The camera's circle round the site: radius and height (m), how far round it goes in the
/// scene (radians).
const RADIUS: f64 = 150.0;
const HEIGHT: f64 = 35.0;
const SWEEP: f32 = 0.6;

/// The scene: the site's centre, where the camera starts round it.
#[derive(Component)]
struct Site {
    centre: DVec3,
    start: f32,
}

fn sam_site(mut stage: Stage, site: Query<&Site>, mut cam: Camera, mut fov: ResMut<SceneFov>) {
    if stage.begin(&[stage::AAA]) {
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
        stage.commands.spawn((ShotEntity, Site { centre, start }));
        return;
    }
    let t = stage.director.t;
    let Ok(s) = site.single() else { return };
    let a = s.start + SWEEP * t / super::director::SHOT_SECONDS;
    let eye = s.centre + enu(a.cos() as f64 * RADIUS, HEIGHT, a.sin() as f64 * RADIUS);
    stage::look(&mut cam, eye, s.centre + DVec3::Y * 8.0);
    fov.0 = 45f32.to_radians();
}

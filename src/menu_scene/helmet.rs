//! The F-16's cockpit, the pilot's head turned to a MiG-29 flying off the wing: the helmet
//! sight's lock and the Sidewinder's seeker on it, then the missile off the wingtip
//! (`Shot::Helmet`).

use bevy::math::DVec3;
use bevy::prelude::*;

use super::cockpits::{Cabin, spawn_cabin};
use super::director::{Shot, shot_on};
use super::flyers::Flyer;
use super::fx::{self, Aim, Airborne, Missile};
use super::stage::{self, Camera, Stage, Weave, enu};
use super::{SceneCamera, SceneFov, ShotEntity, ShotSet};
use crate::aircraft::AircraftKind;
use crate::cockpit3d::scene::Seat;
use crate::explosions::Blast;
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (helmet, fit_visor).chain().in_set(ShotSet).run_if(shot_on(Shot::Helmet)),
    );
}

/// When the missile goes (s), the jets' speed (m/s).
const FIRE_AT: f32 = 2.0;
const SPEED: f32 = 210.0;
/// The F-16's right wingtip rail (the aircraft's frame, m).
const WINGTIP: Vec3 = Vec3::new(4.8, 0.0, 0.6);

/// The scene: the jets' flights, the MiG, the missile, the visor, the head's turn.
#[derive(Component)]
struct Run {
    weave: Weave,
    mig: Entity,
    missile: Option<Entity>,
    visor: Option<Entity>,
    look: Option<Vec2>,
}

/// The visor on the scene camera, once there is a run.
fn fit_visor(
    mut commands: Commands,
    mut runs: Query<&mut Run>,
    cams: Query<Entity, With<SceneCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<crate::cockpit3d::hud_glass::HmdMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (Ok(mut r), Ok(cam)) = (runs.single_mut(), cams.single()) else { return };
    if r.visor.is_none() {
        let v = crate::hmd::spawn_scene_visor(
            &mut commands,
            cam,
            &mut meshes,
            &mut materials,
            &mut images,
        );
        commands.entity(v).insert(ShotEntity);
        r.visor = Some(v);
    }
}

#[allow(clippy::too_many_arguments)]
fn helmet(
    mut stage: Stage,
    mut screens: crate::cockpit3d::scene::SceneScreens,
    hud: super::cockpits::HudKit,
    mut run: Query<&mut Run>,
    flyers: Query<(Entity, &Flyer)>,
    mut cabins: Query<(&Cabin, &mut WorldPos, &mut Transform), Without<SceneCamera>>,
    things: Query<&WorldPos, (Without<Cabin>, Without<SceneCamera>)>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    time: Res<Time>,
) {
    if stage.begin(&[stage::AIRCRAFT]) {
        let start = stage.land() + DVec3::Y * 3000.0;
        let heading = (stage.director.roll % 628) as f32 * 0.01;
        let weave = Weave {
            start,
            heading,
            speed: SPEED,
            amp: Weave::amp_for(14.0, 20.0),
            period: 14.0,
            phase: 0.0,
            climb: 0.0,
        };
        // Off the right wing, a little ahead and higher, weaving its own way.
        // On the right: the head turns away from the menu.
        let side = 1.0;
        let off = stage::right(heading) * 220.0 * side + stage::forward(heading) * 60.0;
        let mig = Weave {
            start: start + enu(off.x as f64, 45.0, off.y as f64),
            amp: Weave::amp_for(9.0, 35.0),
            period: 9.0,
            phase: 2.0,
            ..weave
        };
        let Some(m) = stage.model(stage::AIRCRAFT, "aircraft_016", 0.0, stage::flying()) else {
            return;
        };
        let (_, _, v) = mig.at(0.0);
        stage.commands.entity(m).insert((Flyer::other(mig, None), Airborne(v)));
        spawn_cabin(&mut stage, Cabin::new(Seat::Cockpit(AircraftKind::F16), None));
        stage
            .commands
            .spawn((ShotEntity, Run { weave, mig: m, missile: None, visor: None, look: None }));
        return;
    }
    let t = stage.director.t;
    let (Ok(mut r), Ok((cabin, mut wp, mut tf))) = (run.single_mut(), cabins.single_mut()) else {
        return;
    };
    let (p, rot, v) = r.weave.at(t);
    (wp.0, tf.rotation) = (p, rot);
    // The MiG flies its weave (gone once the missile breaks it up).
    let mut mig_at = None;
    for (e, f) in &flyers {
        let (mp, mr, _) = f.weave.at(t);
        stage.commands.entity(e).try_insert((WorldPos(mp), Transform::from_rotation(mr)));
        mig_at = Some(mp);
    }
    let eye = p + (rot * cabin.eye).as_dvec3();
    // The head turns to the MiG, smoothly; level with it.
    let want = mig_at.map(|m| {
        let d = rot.inverse() * (m - eye).as_vec3();
        Vec2::new((-d.x).atan2(-d.z).clamp(-2.3, 2.3), d.y.atan2(d.xz().length()))
    });
    let dt = time.delta_secs().min(0.1);
    let look = match (r.look, want) {
        (Some(l), Some(w)) => l + (w - l) * (dt * 3.0).min(1.0),
        (None, Some(w)) => w,
        (l, None) => l.unwrap_or(Vec2::ZERO),
    };
    r.look = Some(look);
    super::cockpits::eye_camera(&mut cam, cabin, p, rot, look);
    let mode = crate::hud::HudMode::Air;
    let alt = 3000.0;
    let aim9 = Some((if t < FIRE_AT { 2 } else { 1 }, "AIM-9"));
    hud.draw(&mut screens, AircraftKind::F16, rot, v, alt, mode, aim9, t);
    // The visor's symbols ride the view: it isn't turned for the menu.
    *fov = SceneFov(50f32.to_radians(), true);
    // The Sidewinder off the wingtip.
    if t >= FIRE_AT && r.missile.is_none() {
        let m = Missile {
            vel: v,
            accel: 300.0,
            top: 850.0,
            // An off-boresight shot: it pulls round hard off the rail.
            turn: 4.5,
            straight: 0.1,
            aim: Aim::Thing(r.mig),
            blast: Blast::Kill,
            age: 0.0,
        };
        let at = p + (rot * WINGTIP).as_dvec3();
        r.missile = fx::launch(&mut stage, "aircraft_050", at, v, m);
        if r.missile.is_none() {
            r.missile = Some(Entity::PLACEHOLDER);
        }
    }
    // The visor: the lock and the seeker on the MiG until the missile gets there.
    let alive = things.contains(r.mig);
    let Some(mut tex) = r.visor.and_then(|v| screens.canvas_of(v)) else { return };
    let head = rot * Quat::from_euler(EulerRot::YXZ, look.x, look.y, 0.0);
    let to_mig = mig_at.filter(|_| alive).map(|m| (m - eye).as_vec3().normalize_or(Vec3::NEG_Z));
    let line =
        if t < FIRE_AT { Some("AIM-9 2  SHOOT".to_string()) } else { Some("AIM-9 1".to_string()) };
    let blink = (time.elapsed_secs() * 3.0) as i32 % 2 == 0;
    tex.canvas.clear(0);
    crate::hmd::draw_scene(
        &mut tex.canvas,
        head,
        rot,
        to_mig.map(|d| rot.inverse() * d),
        to_mig,
        line,
        SPEED * 1.944,
        3000.0 * 3.281,
        blink,
    );
    tex.dirty = true;
}

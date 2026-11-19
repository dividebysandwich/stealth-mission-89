//! The scenes from the cockpit: a random aircraft's forward view as it maneuvers
//! (`Shot::Cockpit`), and the ones closing in on a screen (`slam.rs`, `helmet.rs`, `comanche.rs`).
//! The cockpit's model comes from `cockpit3d::scene`; its screens show what the scene draws.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::director::{Shot, shot_on};
use super::stage::{self, Camera, Stage, Weave};
use super::{SceneFov, ShotEntity, ShotSet};
use crate::aircraft::AircraftKind;
use crate::cockpit3d::scene::{SceneCockpits, SceneScreens, Screen, Seat};
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (
            (super::pod::fit_pods, fit_cockpits)
                .chain()
                .after(ShotSet)
                .run_if(resource_exists::<super::director::Director>),
            forward_view.in_set(ShotSet).run_if(shot_on(Shot::Cockpit)),
        ),
    );
}

/// A scene's own aircraft seen from inside: where it flies (its root, without the outside
/// model) and the seat's model it wants (`fit_cockpits` puts it there), with the camera
/// pictures its screens show.
#[derive(Component)]
pub struct Cabin {
    pub seat: Seat,
    pub pod: Option<Handle<Image>>,
    /// The cockpit's root once it's there.
    pub fitted: Option<Entity>,
    /// The eye point (the aircraft's frame, m).
    pub eye: Vec3,
}

impl Cabin {
    pub fn new(seat: Seat, pod: Option<Handle<Image>>) -> Self {
        Cabin { seat, pod, fitted: None, eye: Vec3::ZERO }
    }
}

/// Spawns a root for the scene's own aircraft (its place set by the scene).
pub fn spawn_cabin(stage: &mut Stage, cabin: Cabin) -> Entity {
    stage
        .commands
        .spawn((
            ShotEntity,
            cabin,
            WorldPos::default(),
            Transform::default(),
            Visibility::default(),
        ))
        .id()
}

/// Fits the cockpits wanted, once their models are loaded.
fn fit_cockpits(
    mut commands: Commands,
    mut kit: SceneCockpits,
    mut cabins: Query<(
        Entity,
        &mut Cabin,
        Option<&super::pod::WantsPod>,
        Option<&super::pod::PodImage>,
    )>,
) {
    for (e, mut c, wants, pod) in &mut cabins {
        // Its camera picture first, if it shows one.
        if c.fitted.is_some() || (wants.is_some() && pod.is_none()) {
            continue;
        }
        if let Some(p) = pod {
            c.pod = Some(p.0.clone());
        }
        c.eye = kit.eye(c.seat);
        c.fitted = kit.spawn(&mut commands, e, c.seat, c.pod.clone());
    }
}

/// The camera at the eye of a cabin flying at `p` turned `rot`, looking `look` (yaw left,
/// pitch up, radians) from straight ahead.
pub fn eye_camera(cam: &mut Camera, cabin: &Cabin, p: DVec3, rot: Quat, look: Vec2) {
    let eye = p + (rot * cabin.eye).as_dvec3();
    if let Ok((mut wp, mut tf)) = cam.single_mut() {
        wp.0 = eye;
        tf.rotation = rot * Quat::from_euler(EulerRot::YXZ, look.x, look.y, 0.0);
    }
}

/// The aircraft that fly the forward view: the fast jets and the attack helicopters.
const FORWARD: [AircraftKind; 9] = [
    AircraftKind::F16,
    AircraftKind::F22,
    AircraftKind::FA18E,
    AircraftKind::F117,
    AircraftKind::Tornado,
    AircraftKind::A10,
    AircraftKind::AV8B,
    AircraftKind::AH64,
    AircraftKind::RAH66,
];

/// A random aircraft's forward view, low over the land and pulling through hard turns.
#[allow(clippy::too_many_arguments)]
fn forward_view(
    mut stage: Stage,
    mut screens: SceneScreens,
    mut cabins: Query<(&Cabin, &mut WorldPos, &mut Transform), Without<super::SceneCamera>>,
    mut weave: Local<Option<Weave>>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
) {
    let kind = FORWARD[stage.director.pick(FORWARD.len())];
    let heli = kind.spec().helicopter;
    if stage.begin(&[]) {
        let start = stage.land() + DVec3::Y * if heli { 60.0 } else { 300.0 };
        let heading = (stage.director.roll % 628) as f32 * 0.01;
        *weave = Some(Weave {
            start,
            heading,
            speed: if heli { 60.0 } else { 220.0 },
            amp: Weave::amp_for(7.0, if heli { 25.0 } else { 50.0 }),
            period: 7.0,
            phase: -0.6,
            climb: if heli { 0.0 } else { 15.0 },
        });
        spawn_cabin(&mut stage, Cabin::new(Seat::Cockpit(kind), None));
        return;
    }
    let Some(w) = *weave else { return };
    let (p, rot, _) = w.at(stage.director.t);
    let Ok((cabin, mut wp, mut tf)) = cabins.single_mut() else { return };
    wp.0 = p;
    tf.rotation = rot;
    eye_camera(&mut cam, cabin, p, rot, Vec2::new(0.0, -0.02));
    fov.0 = 50f32.to_radians();
    if cabin.fitted.is_some() {
        draw_mfds(&mut screens, kind, stage.director.t);
    }
}

/// Engines running at cruise power (the engine page's), `t` seconds into the scene.
pub fn cruise_engines(kind: AircraftKind, t: f32) -> crate::eng_page::Eng {
    let n = match kind {
        AircraftKind::F16 | AircraftKind::F117 | AircraftKind::AV8B => 1,
        AircraftKind::C5 | AircraftKind::AC130 => 4,
        _ => 2,
    };
    let wob = (t * 1.3).sin() * 0.4;
    let mut e = crate::eng_page::Eng {
        engines: n,
        nr: 100.0,
        flow: 6400.0,
        fuel: 7200.0,
        bingo: 2000.0,
        ..Default::default()
    };
    for i in 0..n {
        e.core[i] = 91.0 + wob + i as f32 * 0.3;
        e.fan[i] = 88.0 + wob;
        e.temp[i] = 735.0 + wob * 6.0;
        e.oil[i] = 42.0;
        e.oil_t[i] = 88.0;
        e.nozzle[i] = 35.0;
        e.tq[i] = 72.0 + wob;
    }
    e
}

/// The forward view's MFDs: the engines on the right, on the left the signature page of a
/// stealthy type, else the engines too.
fn draw_mfds(screens: &mut SceneScreens, kind: AircraftKind, t: f32) {
    let eng = cruise_engines(kind, t);
    if let Some(mut tex) = screens.canvas(Screen::Right) {
        tex.canvas.clear(0);
        crate::eng_page::draw(&mut tex.canvas, kind, &eng);
        tex.dirty = true;
    }
    if let Some(mut tex) = screens.canvas(Screen::Left) {
        tex.canvas.clear(0);
        if kind.rcs_page() {
            draw_rcs(&mut tex.canvas, t);
        } else {
            crate::eng_page::draw(&mut tex.canvas, kind, &eng);
        }
        tex.dirty = true;
    }
}

/// The signature page with radars round the aircraft (`mpd::threat_fixture`), its bloom
/// pulsing.
pub fn draw_rcs(c: &mut crate::canvas::Canvas, t: f32) {
    let (emitters, emv, history) = crate::mpd::threat_fixture();
    let th = crate::mpd::Threats {
        emitters: &emitters,
        emv: &emv,
        history: &history,
        bay_open: false,
        gear_down: false,
        pitch: 0.05,
        roll: (t * 1.2).sin() * 0.5,
    };
    crate::avionics::draw_rcs(c, &th, 1, (t * 2.0) as i32 % 2 == 0);
}

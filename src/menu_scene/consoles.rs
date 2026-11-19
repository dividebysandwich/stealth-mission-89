//! The air defence consoles engaging a target (`Shot::Patriot`, `Shot::Gepard`): the operator's
//! view of the console, its air picture, track board and target data drawn from what flies
//! in the world round the site, its camera on the target as the missile reaches it or the
//! gun's bursts close round it.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::cockpits::{Cabin, spawn_cabin};
use super::director::{Shot, shot_on};
use super::fx::{self, Aim, Airborne, Missile, Shell};
use super::pod::{self, PodCamera};
use super::stage::{self, Camera, Stage, enu};
use super::{SceneCamera, SceneFov, ShotEntity, ShotSet};
use crate::cockpit3d::scene::{SceneScreens, Seat};
use crate::explosions::{Blast, Calibre, Explosions};
use crate::station::scene::{self as console, AirPicture, Kind, Side};
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (
            air_defence::<false>.run_if(shot_on(Shot::Patriot)),
            air_defence::<true>.run_if(shot_on(Shot::Gepard)),
            sonar.run_if(shot_on(Shot::Sonar)),
        )
            .in_set(ShotSet),
    );
}

/// The target's track number (flight unit) and the others'.
const TARGET: usize = 4;
/// The battery's missile: off the rail (s) and its speed (m/s).
const LAUNCH_AT: f32 = 0.3;
const MISSILE_SPEED: f32 = 1500.0;
/// The Gepard's bursts: from, to (s), rounds a second shown, the rounds' speed (m/s).
const BURST: (f32, f32) = (0.8, 3.6);
const ROUNDS_PER_S: f32 = 14.0;
const GEPARD_V0: f64 = 1175.0;

/// The scene: the site (the console's place, the launcher's or gun's), the target's flight
/// (start, velocity), what has been fired.
#[derive(Component)]
struct Site {
    at: DVec3,
    gun: DVec3,
    start: DVec3,
    vel: DVec3,
    target: Entity,
    missile: Option<Entity>,
    fired: usize,
    killed: bool,
}

/// Others in the picture: (unit, start east/north/alt from the site, velocity east/north, side,
/// kind, name).
type Other = (usize, [f32; 3], [f32; 2], Side, Kind, &'static str);
const OTHERS: [Other; 3] = [
    (2, [-18_000.0, 9_000.0, 6_000.0], [180.0, 40.0], Side::Friend, Kind::Fixed, "F-16C"),
    (7, [12_000.0, 21_000.0, 4_000.0], [-150.0, -60.0], Side::Hostile, Kind::Fixed, "Su-24"),
    (9, [6_000.0, -14_000.0, 300.0], [-30.0, 50.0], Side::Unknown, Kind::Rotary, "Mi-8"),
];

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn air_defence<const GEPARD: bool>(
    mut stage: Stage,
    mut screens: SceneScreens,
    mut site: Query<&mut Site>,
    mut cabins: Query<
        (&Cabin, &mut WorldPos, &mut Transform),
        (Without<SceneCamera>, Without<PodCamera>),
    >,
    mut pods: pod::Pods,
    things: Query<&WorldPos, (Without<Cabin>, Without<SceneCamera>, Without<PodCamera>)>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    (mut ex, mut materials, palette): (
        ResMut<Explosions>,
        ResMut<Assets<crate::palette_material::PaletteMaterial>>,
        Res<crate::palette::Palette>,
    ),
    mut tracer: Local<fx::TracerCache>,
) {
    if stage.begin(&[stage::AIRCRAFT, stage::AAA, stage::DRONES]) {
        let at = stage.land();
        let a = (stage.director.roll % 628) as f64 * 0.01;
        // Inbound: from out there towards a point beside the site.
        let (range, alt, speed) =
            if GEPARD { (3800.0, 250.0, 70.0) } else { (5200.0, 1600.0, 260.0) };
        let start = at + enu(a.sin() * range, alt, a.cos() * range);
        let aim = at + enu(a.cos() * 600.0, alt * 0.8, -a.sin() * 600.0);
        let vel = (aim - start).normalize() * speed;
        let (file, name) =
            if GEPARD { (stage::DRONES, "drones_102") } else { (stage::AIRCRAFT, "aircraft_022") };
        let Some(target) = stage.model(file, name, 0.0, stage::flying()) else { return };
        let heading = (vel.x as f32).atan2(-vel.z as f32);
        stage.commands.entity(target).insert((
            WorldPos(start),
            Transform::from_rotation(stage::attitude(heading, 0.0, 0.0)),
            Airborne(vel.as_vec3()),
        ));
        // The launcher (and the radar) or the Gepard a little off the console's place.
        let gun = at + enu(a.cos() * 60.0, 0.0, -a.sin() * 60.0);
        let gun = DVec3::new(gun.x, stage.ground(gun), gun.z);
        let names: &[&str] = if GEPARD { &["aaa_115"] } else { &["aaa_113", "aaa_114"] };
        for (i, n) in names.iter().enumerate() {
            let p = gun + enu(i as f64 * 25.0, 0.0, i as f64 * 15.0);
            let p = DVec3::new(p.x, stage.ground(p), p.z);
            if let Some(e) = stage.model(stage::AAA, n, 0.0, crate::models::ModelState::default()) {
                stage.commands.entity(e).insert((
                    WorldPos(p),
                    Transform::from_rotation(stage::attitude(a as f32, 0.0, 0.0)),
                ));
            }
        }
        let seat = Seat::Console(crate::cockpit3d::CONSOLE_MODEL);
        let cabin = spawn_cabin(&mut stage, Cabin::new(seat, None));
        // A lens that shows the target a fair size at its range.
        let lens = if GEPARD { 0.9f32 } else { 0.6 };
        stage.commands.entity(cabin).insert(pod::WantsPod((616, 480), lens.to_radians()));
        stage.commands.spawn((
            ShotEntity,
            Site { at, gun, start, vel, target, missile: None, fired: 0, killed: false },
        ));
        return;
    }
    let t = stage.director.t;
    let (Ok(mut s), Ok((cabin, mut wp, mut tf))) = (site.single_mut(), cabins.single_mut()) else {
        return;
    };
    // The console's room under the site.
    let face = (s.start - s.at).as_vec3();
    let yaw = (face.x).atan2(-face.z);
    (wp.0, tf.rotation) = (s.at + DVec3::Y * 2.0, stage::attitude(yaw, 0.0, 0.0));
    let alive = !s.killed && things.contains(s.target);
    let target_at = s.start + s.vel * t as f64;
    if alive {
        stage.commands.entity(s.target).try_insert(WorldPos(target_at));
    }
    // The battery's missile, or the Gepard's bursts.
    if !GEPARD && t >= LAUNCH_AT && s.missile.is_none() {
        let up = Vec3::new(0.0, 1.0, 0.0);
        let m = Missile {
            vel: up * 80.0,
            accel: 1000.0,
            top: MISSILE_SPEED,
            turn: 1.6,
            straight: 0.3,
            aim: Aim::Thing(s.target),
            blast: Blast::Kill,
            age: 0.0,
        };
        let from = s.gun + DVec3::Y * 4.0;
        s.missile = fx::launch(&mut stage, "aircraft_015", from, up * 80.0, m)
            .or(Some(Entity::PLACEHOLDER));
    }
    if GEPARD && (BURST.0..BURST.1).contains(&t) && alive {
        let material = fx::tracer_material(&mut tracer, &mut materials, &palette);
        let due = ((t - BURST.0) * ROUNDS_PER_S) as usize;
        while s.fired <= due {
            s.fired += 1;
            let muzzle =
                s.gun + DVec3::Y * 3.5 + DVec3::new((s.fired % 2) as f64 * 1.6 - 0.8, 0.0, 0.0);
            let range = (target_at - muzzle).length();
            let tof = range / GEPARD_V0;
            let lead = s.vel * tof + DVec3::Y * (0.5 * 9.81 * tof * tof);
            let k = s.fired as f64;
            let spread = DVec3::new((k * 1.9).sin(), (k * 2.7).cos(), (k * 1.3).sin())
                * (6.0 - 1.4 * t as f64).max(1.0);
            let vel = (target_at + lead + spread - muzzle).normalize() * GEPARD_V0;
            let shell = Shell {
                vel,
                calibre: Calibre::Small,
                streak: 0.04,
                glow: None,
                age: 0.0,
                fuse: Some((tof - 0.02, Blast::Flak)),
            };
            fx::fire(
                &mut stage.commands,
                material.clone(),
                &mut ex,
                muzzle,
                Vec3::ZERO,
                shell,
                s.fired % 2 == 0,
            );
        }
        if t >= BURST.1 - 0.5 && !s.killed {
            s.killed = true;
            stage.commands.entity(s.target).insert(fx::Hit);
            ex.pending.push((target_at, Blast::Kill));
        }
    }
    if !GEPARD && !s.killed && !things.contains(s.target) {
        s.killed = true;
    }
    // The camera on the target from the site.
    let eye = s.at + DVec3::Y * 12.0;
    pod::aim_pod(
        &mut pods,
        eye,
        if alive { target_at } else { s.start + s.vel * (t as f64).min(4.0) },
    );
    // The picture: the target, the others, our missile, made from where they are now and were.
    let picture = |at: f32| {
        let mut tracks = Vec::new();
        let rel = |p: DVec3| (p - s.at).as_vec3();
        let tp = rel(s.start + s.vel * at as f64);
        if alive || at < t - 0.3 {
            let (name, kind) =
                if GEPARD { ("Mi-24", Kind::Rotary) } else { ("Su-27", Kind::Fixed) };
            tracks.push(console::track(
                Some(TARGET),
                tp.x,
                -tp.z,
                tp.y,
                s.vel.x as f32,
                -s.vel.z as f32,
                Side::Hostile,
                kind,
                name,
            ));
        }
        for &(u, p, v, side, kind, name) in &OTHERS {
            let (e, n) = (p[0] + v[0] * at, p[1] + v[1] * at);
            tracks.push(console::track(Some(u), e, n, p[2], v[0], v[1], side, kind, name));
        }
        if !GEPARD && at > LAUNCH_AT && !s.killed {
            let k = ((at - LAUNCH_AT) * MISSILE_SPEED * 0.8 / tp.length().max(1.0)).min(1.0);
            let m = tp * k;
            tracks.push(console::track(
                None,
                m.x,
                -m.z,
                m.y,
                tp.x,
                -tp.z,
                Side::Friend,
                Kind::Missile,
                "PAC-2",
            ));
        }
        console::snapshot(tracks)
    };
    let history: Vec<_> = (0..6).map(|k| picture((t - k as f32 * 0.5).max(0.0))).collect();
    let rel = (target_at - eye).as_vec3();
    let camera = alive.then(|| {
        let brg = rel.x.atan2(-rel.z).to_degrees().rem_euclid(360.0);
        let elv = rel.y.atan2(rel.xz().length()).to_degrees();
        (rel.length(), brg, elv)
    });
    if cabin.fitted.is_some() {
        let p = AirPicture {
            title: if GEPARD { "GEPARD" } else { "PATRIOT" },
            gepard: GEPARD,
            history,
            lock: Some(TARGET),
            launchers: if GEPARD {
                vec![]
            } else {
                vec![Vec2::new(60.0, 0.0), Vec2::new(-40.0, 50.0), Vec2::new(30.0, -70.0)]
            },
            reach: if GEPARD { 4500.0 } else { 70_000.0 },
            range_km: if GEPARD { 10.0 } else { 40.0 },
            camera,
            zoom: "8X",
            firing: if GEPARD {
                (BURST.0..BURST.1).contains(&t)
            } else {
                (LAUNCH_AT..LAUNCH_AT + 0.6).contains(&t)
            },
        };
        console::draw_air_defence(&mut screens, &p);
        screens.all_pictures();
    }
    // The operator's view of the console, a little down and to its middle.
    super::cockpits::eye_camera(&mut cam, cabin, wp.0, tf.rotation, Vec2::new(-0.05, -0.12));
    fov.0 = 58f32.to_radians();
}

/// The MH-60R's sensor operator's console in its cabin, close on the sonar page: the buoys'
/// and the dip's waterfalls full of traces, filling on.
#[allow(clippy::too_many_arguments)]
fn sonar(
    mut stage: Stage,
    mut screens: SceneScreens,
    mut cabins: Query<(&Cabin, &mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut page: Local<Option<crate::asw::scene::SonarScene>>,
    mut colours: Local<Option<std::sync::Arc<[u32; 256]>>>,
    palette: Res<crate::palette::Palette>,
    time: Res<Time>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
) {
    if stage.begin(&[]) {
        let places = stage.places();
        let deck = places.decks.first().copied();
        let at = deck.map_or_else(|| stage.land(), |d| enu(d.x + 3000.0, 0.0, d.y));
        let model = crate::aircraft::AircraftKind::MH60R
            .spec()
            .console
            .as_ref()
            .map_or("cockpit_asw", |c| c.model);
        let cabin = spawn_cabin(&mut stage, Cabin::new(Seat::Console(model), None));
        stage.commands.entity(cabin).insert((WorldPos(at + DVec3::Y * 30.0), Transform::default()));
        *page = Some(crate::asw::scene::SonarScene::new(stage.director.roll));
        *colours = Some(crate::asw::mad::console_colours(&palette));
        return;
    }
    let Ok((cabin, wp, tf)) = cabins.single_mut() else { return };
    let Some(page) = page.as_mut() else { return };
    page.step(time.delta_secs().min(0.1));
    if cabin.fitted.is_some()
        && let Some(mut tex) = screens
            .canvas(crate::cockpit3d::scene::Screen::Console(crate::cockpit3d::ConsoleScreen::Asw))
        && let Some(c) = colours.clone()
    {
        page.draw(&mut tex, c);
    }
    // The operator looking at the screen, close.
    let look = screens
        .find(crate::cockpit3d::scene::Screen::Console(crate::cockpit3d::ConsoleScreen::Asw))
        .map(|_| Vec2::new(0.0, -0.18))
        .unwrap_or(Vec2::new(0.0, -0.18));
    super::cockpits::eye_camera(&mut cam, cabin, wp.0, tf.rotation, look);
    fov.0 = 42f32.to_radians();
}

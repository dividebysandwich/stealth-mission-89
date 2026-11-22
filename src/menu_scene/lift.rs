//! Helicopters and transports at work: the MH-60R hovering over the sea with its dipping sonar
//! down (`Shot::Dip`), the C-5 dropping pallets over its ramp (`Shot::Airdrop`), a Black Hawk
//! coming to a hover over a crate, hooking it on its sling and leaving with it (`Shot::Sling`).

use bevy::math::DVec3;
use bevy::prelude::*;

use super::director::{Shot, shot_on};
use super::stage::{self, Camera, Stage, Weave, enu};
use super::{SceneCamera, SceneFov, ShotEntity, ShotSet};
use crate::aircraft::AircraftKind;
use crate::models::ModelState;
use crate::world::WorldPos;

pub(super) fn build(app: &mut App) {
    app.add_systems(
        Update,
        (
            dip.run_if(shot_on(Shot::Dip)),
            airdrop.run_if(shot_on(Shot::Airdrop)),
            (sling, sling_props).chain().run_if(shot_on(Shot::Sling)),
        )
            .in_set(ShotSet),
    );
}

/// A hover as a pilot holds it: the drift (m) to the side, fore and aft and up at `t` seconds,
/// and the bank and pitch it takes (rad).
fn sway(t: f32) -> (Vec3, f32, f32) {
    let wave = |amp: f32, period: f32, phase: f32| {
        let w = std::f32::consts::TAU / period;
        let s = (w * t + phase).sin();
        (amp * s, -amp * w * w * s)
    };
    let (side, side_acc) = wave(0.8, 7.3, 0.0);
    let (fore, fore_acc) = wave(0.5, 8.9, 1.3);
    let (up, _) = wave(0.3, 5.3, 0.7);
    (Vec3::new(side, up, fore), -side_acc / 9.81, -fore_acc / 9.81)
}

/// The root of a scene's helicopter, and where it hovers (Bevy metres) and its heading.
#[derive(Component)]
struct Hover {
    at: DVec3,
    heading: f32,
}

/// The MH-60R in a hover over the sea (its downwash beating the water), its sonar's cable down into the water (the model's
/// dip slot), a little way off a carrier; the camera low over the waves, circling slowly.
fn dip(
    mut stage: Stage,
    heli: Query<(Entity, &Hover)>,
    mut q: Query<(&mut WorldPos, &mut Transform), Without<SceneCamera>>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
) {
    const HEIGHT: f64 = 18.0;
    if stage.begin(&[stage::AIRCRAFT]) {
        let places = stage.places();
        let deck = places.decks.get(stage.director.pick(places.decks.len().max(1))).copied();
        let a = (stage.director.roll % 628) as f64 * 0.01;
        let sea = deck.map_or_else(|| stage.land(), |d| enu(d.x, 0.0, d.y))
            + enu(a.cos() * 3000.0, HEIGHT, a.sin() * 3000.0);
        let spec = AircraftKind::MH60R.spec();
        let mut state = ModelState::default();
        (state.0[12], state.0[crate::aircraft::SEAHAWK_DIP_SLOT]) = (1, 1);
        if let Some(e) = stage.model(stage::AIRCRAFT, spec.model, spec.model_lift, state) {
            stage.commands.entity(e).insert(Hover { at: sea, heading: a as f32 });
        }
        return;
    }
    let t = stage.director.t;
    let Ok((e, h)) = heli.single() else { return };
    let (off, bank, pitch) = sway(t);
    let r = Quat::from_rotation_y(-h.heading) * Vec3::new(off.x, off.y, -off.z);
    let p = h.at + r.as_dvec3();
    stage::put(&mut q, e, p, stage::attitude(h.heading, 0.03 + pitch, bank));
    // Low over the water, swinging round; the helicopter and its cable's way down in view.
    let a = h.heading + 1.9 + 0.08 * t;
    let eye = h.at + enu((a.sin() * 55.0) as f64, -HEIGHT + 6.0, (a.cos() * 55.0) as f64);
    stage::look(&mut cam, eye, p - DVec3::Y * 12.0);
    fov.0 = 45f32.to_radians();
}

/// A pallet out of the ramp (on its rig's load): its velocity, age, tumble, heading and rig.
#[derive(Component)]
struct Pallet {
    vel: DVec3,
    age: f32,
    tumble: Quat,
    heading: Quat,
    rig: crate::pallet_chute::CargoRig,
}

/// The camera's stations behind the C-5 (m: behind, to the right, above), the point it looks
/// at (the aircraft's frame, m) and its lens (degrees high).
const DROP_VIEWS: [(f32, f32, f32, Vec3, f32); 5] = [
    // Behind its left side, a little below.
    (95.0, -60.0, -12.0, Vec3::new(0.0, -8.0, 30.0), 50.0),
    // Behind its right side, a little below.
    (90.0, 65.0, -8.0, Vec3::new(0.0, -8.0, 30.0), 50.0),
    // High behind, looking down on the pallets falling away.
    (120.0, 30.0, 45.0, Vec3::new(0.0, -20.0, 50.0), 55.0),
    // Low behind, the chutes opening over the camera.
    (150.0, -20.0, -55.0, Vec3::new(0.0, -15.0, 40.0), 55.0),
    // Straight behind, on a long lens: the ramp and the pallets leaving it.
    (180.0, 6.0, -6.0, Vec3::new(0.0, -4.0, 25.0), 28.0),
];

/// The C-5, its ramp open, pallets going out one after another and their chutes opening
/// behind it; the camera behind it at the scene's station, drifting slowly.
#[allow(clippy::too_many_arguments)]
fn airdrop(
    mut stage: Stage,
    flyers: Query<(Entity, &super::flyers::Flyer)>,
    mut pallets: Query<(Entity, &mut Pallet)>,
    mut q: Query<(&mut WorldPos, &mut Transform, &mut Visibility), Without<SceneCamera>>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    time: Res<Time>,
    mut dropped: Local<usize>,
) {
    const SPEED: f32 = 70.0;
    /// The first pallet goes, and the next ones every so often.
    const FIRST: f32 = 0.4;
    const EVERY: f32 = 0.55;
    const PALLETS: usize = 6;
    /// The ramp's lip in the aircraft's frame (m).
    const RAMP: Vec3 = Vec3::new(0.0, 0.5, 30.0);
    if stage.begin(&[stage::AIRCRAFT]) {
        *dropped = 0;
        let start = stage.land() + DVec3::Y * 350.0;
        let heading = (stage.director.roll % 628) as f32 * 0.01;
        let weave =
            Weave { start, heading, speed: SPEED, amp: 0.0, period: 10.0, phase: 0.0, climb: 0.0 };
        let spec = AircraftKind::C5.spec();
        let mut state = stage::flying();
        // The ramp down for the airdrop (slot 8), the flaps out.
        state.0[8] = 2;
        state.0[crate::aircraft::FLAP_SLOT] =
            crate::aircraft::flap_slot_value(crate::aircraft::C5_FLAP_ANGLE);
        if let Some(e) = stage.model(stage::AIRCRAFT, spec.model, spec.model_lift, state) {
            stage.commands.entity(e).insert(super::flyers::Flyer::lead(weave, None));
        }
        return;
    }
    let t = stage.director.t;
    let Some((_, p, rot, v)) = ({
        let mut q2 = q.transmute_lens::<(&mut WorldPos, &mut Transform)>();
        super::flyers::fly(&stage, &flyers, &mut q2.query())
    }) else {
        return;
    };
    // Pallets out over the lip, tumbling back in the slipstream, then hanging under their
    // canopies.
    while *dropped < PALLETS && t >= FIRST + *dropped as f32 * EVERY {
        *dropped += 1;
        let at = p + (rot * RAMP).as_dvec3();
        let rig = stage.cargo_rig();
        let vel = (v - rot * Vec3::NEG_Z * 4.0).as_dvec3();
        let ahead = rot * Vec3::NEG_Z;
        let heading = Quat::from_rotation_y((-ahead.x).atan2(-ahead.z));
        stage.commands.entity(rig.load).insert((
            WorldPos(at),
            Transform::from_rotation(rot),
            Pallet { vel, age: 0.0, tumble: rot, heading, rig },
        ));
    }
    let dt = time.delta_secs().min(0.1);
    for (e, mut pal) in &mut pallets {
        pal.age += dt;
        let open = ((pal.age - crate::pallet_chute::DEPLOY_SECS) / crate::pallet_chute::OPEN_SECS)
            .clamp(0.0, 1.0);
        // Air drag: small for the pallet, large under the canopies.
        let k = 0.002 + 0.06 * open as f64;
        let drag = -pal.vel * pal.vel.length() * k;
        pal.vel += (drag - DVec3::Y * 9.81) * dt as f64;
        let Ok((pos, _, _)) = q.get(e) else { continue };
        let at = pos.0 + pal.vel * dt as f64;
        // Tumbling in the slipstream until the canopies take it.
        let turn = (1.0 - open) * pal.age * 1.5;
        pal.tumble = pal.tumble.slerp(pal.heading * Quat::from_rotation_x(-turn), 0.1);
        let pose = crate::pallet_chute::CargoRig::pose(
            at,
            pal.tumble,
            pal.heading,
            pal.vel.as_vec3(),
            pal.age,
        );
        pal.rig.place(&pose, &mut q);
    }
    let heading = v.x.atan2(-v.z);
    let (back, side, up, at, lens) =
        DROP_VIEWS[(stage.director.roll >> 8) as usize % DROP_VIEWS.len()];
    let back = back + 8.0 * (t * 0.37).sin();
    let side = side + 6.0 * (t * 0.29 + 1.1).sin();
    let up = up + 4.0 * (t * 0.43 + 0.4).sin();
    let eye = stage::behind(p, heading, back, side, up);
    stage::look(&mut cam, eye, p + (rot * at).as_dvec3());
    fov.0 = lens.to_radians();
}

/// The sling load: the crate (its root), the rope and how it swings once hooked.
#[derive(Component)]
struct Load {
    /// Made by `sling_props` (the meshes are its).
    crate_root: Option<Entity>,
    rope: Option<Entity>,
    ground: DVec3,
    heading: f32,
    swing: Option<crate::sling::Swing>,
}

/// The Black Hawk comes in over a crate on the ground, settles into a hover over it, hooks it
/// on and climbs away with it swinging under it; the camera on the ground nearby.
#[allow(clippy::too_many_arguments)]
fn sling(
    mut stage: Stage,
    heli: Query<Entity, With<Hover>>,
    mut load: Query<&mut Load>,
    mut q: Query<(&mut WorldPos, &mut Transform, &mut Visibility), Without<SceneCamera>>,
    mut cam: Camera,
    mut fov: ResMut<SceneFov>,
    time: Res<Time>,
) {
    /// The approach ends (s), the hook-up, the lift; the hover's height over the crate's top
    /// and how much higher it comes in (m), its speed coming in (m/s).
    const ARRIVE: f32 = 3.2;
    const HOOK: f32 = 4.0;
    const LIFT: f32 = 4.8;
    const HOVER: f32 = 12.0;
    const DESCENT: f32 = 4.0;
    const SPEED: f32 = 7.0;
    /// The hook's height over the wheels (uh60.py: the belly 0.58 m up, the model lifted).
    const BELLY: f32 = 0.6;
    if stage.begin(&[stage::AIRCRAFT]) {
        let ground = stage.land();
        let heading = (stage.director.roll % 628) as f32 * 0.01;
        let spec = AircraftKind::UH60.spec();
        let mut state = stage::flying();
        state.0[13] = 1;
        let Some(h) = stage.model(stage::AIRCRAFT, spec.model, spec.model_lift, state) else {
            return;
        };
        stage.commands.entity(h).insert(Hover { at: ground, heading });
        stage.commands.spawn((
            ShotEntity,
            Load { crate_root: None, rope: None, ground, heading, swing: None },
        ));
        return;
    }
    let t = stage.director.t;
    let (Ok(e), Ok(mut l)) = (heli.single(), load.single_mut()) else { return };
    let top = crate::scenery::CRATE_SIZE.z as f64;
    let f = stage::forward(l.heading);
    // In over the crate, slowing smoothly (the nose up as much as it brakes) and settling;
    // the hover; then the load taken up slowly, and away, the nose eased down.
    let smooth = |u: f32| {
        let u = u.clamp(0.0, 1.0);
        u * u * (3.0 - 2.0 * u)
    };
    let (along, up, flare) = if t < ARRIVE {
        let u = t / ARRIVE;
        // Its speed falls from `SPEED` as 1 - smoothstep: the braking eases in and out.
        let gone = SPEED * ARRIVE * (u - u * u * u + 0.5 * u * u * u * u);
        let brake = SPEED / ARRIVE * 6.0 * u * (1.0 - u);
        (
            gone - SPEED * ARRIVE * 0.5,
            HOVER + DESCENT * (1.0 - smooth(u)),
            (brake / 9.81).atan() * 0.7,
        )
    } else if t < LIFT {
        (0.0, HOVER, 0.0)
    } else {
        let tau = t - LIFT;
        let away = (tau - 0.8).max(0.0);
        (0.9 * away * away, HOVER + 0.7 * tau * tau, -0.1 * smooth((tau - 0.5) / 1.5))
    };
    let (off, bank, pitch) = sway(t);
    let base = l.ground + enu((f.x * along) as f64, top + up as f64, (f.y * along) as f64);
    let p = base + DVec3::new(off.x as f64 * 0.5, off.y as f64, off.z as f64 * 0.5);
    let rotation = stage::attitude(l.heading, flare + pitch * 0.5, bank * 0.5);
    if let Ok((mut wp, mut tf, _)) = q.get_mut(e) {
        wp.0 = p;
        tf.rotation = rotation;
    }
    // The hook in the cabin's belly under the mast (the model stands on its wheels at the
    // root); hooked up, the crate swings under it.
    let hook = p + (rotation * Vec3::Y * BELLY).as_dvec3();
    let crate_mid = l.ground + DVec3::Y * (top * 0.5);
    if t >= HOOK && l.swing.is_none() {
        l.swing = Some(crate::sling::Swing::new(crate_mid, hook));
    }
    let ground_y = stage.ground(l.ground) + top * 0.5;
    let at = if let Some(s) = l.swing.as_mut() {
        s.advance(hook, time.delta_secs().min(0.05) as f64, ground_y);
        s.pos
    } else {
        crate_mid
    };
    let (Some(crate_root), Some(rope)) = (l.crate_root, l.rope) else { return };
    if let Ok((mut wp, _, _)) = q.get_mut(crate_root) {
        wp.0 = at;
    }
    if let Ok((mut wp, mut tf, mut vis)) = q.get_mut(rope) {
        let top_of = at + DVec3::Y * (top * 0.5);
        wp.0 = hook;
        let d = (top_of - hook).as_vec3();
        tf.rotation = Quat::from_rotation_arc(Vec3::NEG_Y, d.normalize_or(Vec3::NEG_Y));
        tf.scale = Vec3::new(1.0, d.length().max(0.01), 1.0);
        vis.set_if_neq(if t >= HOOK { Visibility::Inherited } else { Visibility::Hidden });
    }
    // On the ground off to the side, looking up at the hover.
    let side = stage::right(l.heading) * 45.0;
    let eye = l.ground + enu((side.x - f.x * 20.0) as f64, 3.0, (side.y - f.y * 20.0) as f64);
    let eye = DVec3::new(eye.x, stage.ground(eye) + 3.0, eye.z);
    stage::look(&mut cam, eye, l.ground + DVec3::Y * (top + 6.0) + (p - l.ground) * 0.4);
    fov.0 = 50f32.to_radians();
}

/// The sling's crate and rope.
fn sling_props(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    palette: Res<crate::palette::Palette>,
    mut loads: Query<&mut Load>,
) {
    for mut l in &mut loads {
        if l.crate_root.is_some() {
            continue;
        }
        // Centred on its root, shown where the scene puts it.
        let root = crate::scenery::spawn_crate(&mut commands, &mut meshes, &palette);
        commands.entity(root).insert((
            ShotEntity,
            WorldPos(l.ground),
            Transform::from_rotation(Quat::from_rotation_y(-l.heading)),
            Visibility::Inherited,
        ));
        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::LineList,
            bevy::asset::RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::ZERO, Vec3::NEG_Y]);
        let rope = commands
            .spawn((
                ShotEntity,
                WorldPos(l.ground),
                Transform::default(),
                Visibility::Hidden,
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(palette.line_material(0xEE)),
                bevy::light::NotShadowCaster,
                bevy::camera::visibility::NoFrustumCulling,
            ))
            .id();
        (l.crate_root, l.rope) = (Some(root), Some(rope));
    }
}

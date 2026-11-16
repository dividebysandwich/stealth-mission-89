//! The carriers' catapult officers, the "shooters" (`aaa_124`): one beside each catapult
//! (`stealth_sim::catapult`), level with the nose of the aircraft held on it and facing it.
//! While its engines run up he circles two raised fingers over his head; at full power he drops
//! into the launch signal, a deep lunge towards the bow, and holds it while the aircraft goes.
//! The aircraft are the player's (`Aircraft::catapult`), the other pilots' (their reports) and
//! the AI's (`FlightUnit::cat`).
//!
//! The player waiting behind a busy catapult is taxied up to it once it is free, and the HUD
//! tells him what the catapult waits for.

use bevy::prelude::*;
use stealth_i18n::tf;
use stealth_sim::catapult::{AiLaunch, CATAPULTS, OCCUPIED_WU, Phase, TAXI_WU_S, from_report};
use stealth_sim::{fflags, oflags};

use crate::combat::Combat;
use crate::flight::catapult::CatPhase;
use crate::flight::{Aircraft, Player};
use crate::hud::HudMessages;
use crate::models::{ModelAssets, ModelLibrary, ModelState};
use crate::settings::AppState;
use crate::world::{Theater, WU, WorldPos, bevy_to_wu, wu_to_bevy};

pub struct CatapultCrewPlugin;

impl Plugin for CatapultCrewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Crew>().add_systems(OnExit(AppState::Flight), despawn).add_systems(
            Update,
            (taxi_up, hud_hint, (spawn, pose).chain().after(crate::sea::SeaSystems))
                .after(crate::flight::FlightSystems)
                .run_if(in_state(AppState::Flight))
                .run_if(resource_exists::<Combat>),
        );
    }
}

/// The shooter's model (tools/models/air_defence.py `SHOOTER`).
const SHOOTER_MODEL: &str = "aaa_124";
/// His pose (`_s23vN`: idle, run-up, lunge to his right, lunge to his left) and his signalling
/// hand's circling (BAM).
const POSE_SLOT: usize = 23;
const HAND_SLOT: usize = 24;
/// Where he stands from the aircraft's hold-back point (m: ahead along the track, out to the
/// side).
const AHEAD_M: f64 = 5.0;
const ABEAM_M: f64 = 9.0;
/// Turns of his hand a second, giving the run-up signal.
const HAND_TURNS: f32 = 1.6;
/// Seconds he holds the launch signal after the shot.
const FOLLOW_S: f32 = 2.0;

/// The shooters drawn.
#[derive(Resource, Default)]
struct Crew {
    shooters: Vec<Shooter>,
    /// The carriers looked for (none on this theater, or the model not loaded yet: again).
    done: bool,
}

struct Shooter {
    entity: Entity,
    carrier: usize,
    cat: usize,
    /// Where he stands (wu east, north on the deck) and the way he faces (rad).
    east: f64,
    north: f64,
    facing: f32,
    /// He stands right of the aircraft (the bow on his right).
    right: bool,
    hand: f32,
    /// Seconds he goes on holding the launch signal.
    follow: f32,
}

/// The side of its aircraft each catapult's shooter stands on: away from the neighbouring
/// catapult (right of 1 and 3, left of 2 and 4).
fn stands_right(cat: usize) -> bool {
    cat.is_multiple_of(2)
}

fn spawn(
    mut commands: Commands,
    mut crew: ResMut<Crew>,
    combat: Res<Combat>,
    mut library: ResMut<ModelLibrary>,
    server: Res<AssetServer>,
    assets: ModelAssets,
) {
    if crew.done {
        return;
    }
    let sim = &combat.sim;
    // The world not there yet (a client's before the host sends it).
    if sim.objects().is_empty() {
        return;
    }
    let carriers: Vec<usize> = (0..sim.objects().len())
        .filter(|&b| sim.objects()[b].flags & oflags::CARRIER != 0)
        .collect();
    let file = library.load(&server, "aaa");
    let mut shooters = Vec::new();
    for &b in &carriers {
        for cat in 0..CATAPULTS {
            let Some(entity) = assets.spawn(
                &mut commands,
                &file,
                SHOOTER_MODEL,
                crate::palette_material::LEVEL_AIRCRAFT,
                false,
            ) else {
                // Not loaded yet: again next frame.
                for s in shooters.drain(..) {
                    let s: Shooter = s;
                    commands.entity(s.entity).try_despawn();
                }
                return;
            };
            let p = sim.catapult(b, cat);
            let h = p.heading_rad();
            let right = stands_right(cat);
            let side = if right { 1.0 } else { -1.0 };
            let (fe, fn_) = (h.sin(), h.cos());
            let (re, rn) = (fn_, -fe);
            let (ahead, abeam) = (AHEAD_M / WU, side * ABEAM_M / WU);
            let east = p.east + fe * ahead + re * abeam;
            let north = p.north + fn_ * ahead + rn * abeam;
            // Facing the aircraft, across the track.
            let facing = (h - side * std::f64::consts::FRAC_PI_2) as f32;
            commands.entity(entity).insert((
                WorldPos(wu_to_bevy(east, north, crate::sea::DECK_WU as f64)),
                Transform::from_scale(Vec3::splat(crate::player::AIRCRAFT_MODEL_SCALE as f32)),
                Visibility::Inherited,
            ));
            shooters.push(Shooter {
                entity,
                carrier: b,
                cat,
                east,
                north,
                facing,
                right,
                hand: 0.0,
                follow: 0.0,
            });
        }
    }
    crew.shooters = shooters;
    crew.done = true;
}

fn despawn(mut commands: Commands, mut crew: ResMut<Crew>) {
    for s in crew.shooters.drain(..) {
        commands.entity(s.entity).try_despawn();
    }
    crew.done = false;
}

/// The phase of the launch on catapult `cat` of carrier `b`: the player's, another pilot's or
/// an AI aircraft's (the furthest along when there are more).
pub(crate) fn launch_phase(
    sim: &stealth_sim::Sim,
    own: Option<&Aircraft>,
    b: usize,
    cat: usize,
) -> Option<Phase> {
    let mut phases = Vec::new();
    if let Some(c) =
        own.and_then(|a| a.catapult).filter(|c| c.carrier == b && c.cat as usize == cat)
    {
        phases.push(match c.phase {
            CatPhase::Wait => Phase::Wait,
            CatPhase::Tension => Phase::Tension,
            CatPhase::Shoot(_) => Phase::Shoot,
            CatPhase::Stroke(..) => Phase::Stroke,
        });
    }
    let me = sim.active_pilot();
    let pose = sim.catapult(b, cat);
    for p in sim.remote_pilots() {
        if p.id == me {
            continue;
        }
        let near = (p.view.wx as f64 - pose.east).abs().max((p.view.wy as f64 - pose.north).abs())
            < pose.length;
        if let Some((_, phase)) = from_report(p.catapult).filter(|&(c, _)| c == cat && near) {
            phases.push(phase);
        }
    }
    for u in sim.flights() {
        if u.flags & fflags::TAKEOFF == 0 {
            continue;
        }
        if let Some(l) = AiLaunch::decode(u.cat).filter(|l| l.carrier == b && l.cat == cat) {
            phases.push(l.phase);
        }
    }
    let rank = |p: &Phase| match p {
        Phase::Wait => 0,
        Phase::Tension => 1,
        Phase::Shoot => 2,
        Phase::Stroke => 3,
    };
    phases.into_iter().filter(|p| *p != Phase::Wait).max_by_key(rank)
}

fn pose(
    time: Res<Time>,
    mut crew: ResMut<Crew>,
    combat: Res<Combat>,
    theater: Option<Res<Theater>>,
    decks: Res<crate::sea::Decks>,
    player: Query<&Aircraft, With<Player>>,
    mut q: Query<(&mut WorldPos, &mut Transform, &mut ModelState)>,
) {
    let dt = time.delta_secs();
    let own = player.single().ok();
    let deck_list = theater.as_deref().map_or(&[][..], |t| &t.decks[..]);
    for s in &mut crew.shooters {
        let phase = launch_phase(&combat.sim, own, s.carrier, s.cat);
        let lunge = if s.right { 2 } else { 3 };
        let v = match phase {
            Some(Phase::Tension) => {
                s.follow = 0.0;
                s.hand = (s.hand + HAND_TURNS * dt).fract();
                1
            }
            Some(Phase::Shoot | Phase::Stroke) => {
                s.follow = FOLLOW_S;
                lunge
            }
            _ if s.follow > 0.0 => {
                s.follow -= dt;
                lunge
            }
            _ => 0,
        };
        let Ok((mut pos, mut tf, mut state)) = q.get_mut(s.entity) else { continue };
        // On the deck, riding the sea with it.
        let mut at = wu_to_bevy(s.east, s.north, crate::sea::DECK_WU as f64);
        let turn = Quat::from_rotation_y(-s.facing);
        let rot = match crate::sea::on_deck(deck_list, &decks.motion, at) {
            Some((lift, tilt)) => {
                at.y += lift;
                tilt * turn
            }
            None => turn,
        };
        pos.0 = at;
        tf.rotation = rot;
        state.0[POSE_SLOT] = v as i16;
        state.0[HAND_SLOT] = (s.hand * 65536.0) as i32 as i16;
    }
}

/// The player queued behind a busy catapult: taxied up to it once nobody is on it.
fn taxi_up(
    time: Res<Time>,
    combat: Res<Combat>,
    mut player: Query<(&mut Aircraft, &mut WorldPos), With<Player>>,
) {
    let Ok((mut a, mut pos)) = player.single_mut() else { return };
    let Some(mut c) = a.catapult.filter(|c| c.phase == CatPhase::Wait) else { return };
    let sim = &combat.sim;
    let p = sim.catapult(c.carrier, c.cat as usize);
    let near = |e: f64, n: f64| (e - p.east).abs().max((n - p.north).abs()) < OCCUPIED_WU;
    let me = sim.active_pilot();
    let busy = sim
        .remote_pilots()
        .iter()
        .any(|o| o.id != me && o.on_ground && near(o.view.wx as f64, o.view.wy as f64))
        || sim.flights().iter().any(|u| {
            u.flags & fflags::EXISTS != 0
                && match AiLaunch::decode(u.cat).filter(|_| u.flags & fflags::TAKEOFF != 0) {
                    Some(l) => {
                        l.carrier == c.carrier && l.cat == c.cat as usize && l.phase != Phase::Wait
                    }
                    None => u.alt < 400 && near(u.wx as f64, (0x8000 * 32 - u.wy) as f64),
                }
        });
    if busy {
        return;
    }
    let (east, north, alt) = bevy_to_wu(pos.0);
    let (de, dn) = (p.east - east, p.north - north);
    let d = de.hypot(dn);
    let step = TAXI_WU_S * time.delta_secs() as f64;
    if d <= step {
        pos.0 = wu_to_bevy(p.east, p.north, alt);
        c.phase = CatPhase::Tension;
        a.catapult = Some(c);
    } else {
        pos.0 = wu_to_bevy(east + de / d * step, north + dn / d * step, alt);
    }
}

/// The HUD says what the catapult waits for: full power, or a free catapult.
fn hud_hint(mut msgs: ResMut<HudMessages>, player: Query<&Aircraft, With<Player>>) {
    let Ok(a) = player.single() else { return };
    let Some(c) = a.catapult else { return };
    let text = match c.phase {
        CatPhase::Wait => tf!("hud-catapult-wait"),
        CatPhase::Tension => tf!("hud-catapult-power"),
        _ => return,
    };
    // Kept up (not over another message).
    if msgs.timer <= 0.0 || (msgs.text == text && msgs.timer < 0.5) {
        msgs.show(text);
    }
}

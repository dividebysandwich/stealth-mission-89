//! Cargo for the new air defence sites (campaign deployments): a crate per load beside the
//! runway of its base. A helicopter hovering low over one hooks it; it swings on its rope under
//! the helicopter (a damped pendulum, simulated on every client: the carrier's says where it is
//! set down), and set down by its site's helipad it is delivered and stays there until the site
//! is built. The AI's cargo and resupply helicopters sling theirs the same way (the simulation
//! flies them: hooked, carried, set down).

use std::collections::HashMap;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::math::{DVec2, DVec3};
use bevy::mesh::{MeshTag, PrimitiveTopology};
use bevy::prelude::*;
use stealth_campaign::CrateState;
use stealth_campaign::tuning::CRATE_DROP_METRES;

use crate::campaign::{Action, ActiveCampaign, Phase};
use crate::combat::{Combat, CombatSystems, unit_position};
use crate::flight::{Aircraft, FlightConstants, Player};
use crate::hud::HudMessages;
use crate::palette::Palette;
use crate::palette_material::{LEVEL_AIRCRAFT, mesh_tag};
use crate::scenery::{CRATE_SIZE, map_metres, spawn_crate};
use crate::settings::AppState;
use crate::sim_render::attitude;
use crate::world::{WorldPos, wu_to_bevy};
use stealth_sim::hills::Hills;

pub struct SlingPlugin;

impl Plugin for SlingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Sling>().add_systems(OnExit(AppState::Flight), cleanup).add_systems(
            Update,
            sling
                .after(CombatSystems)
                .run_if(resource_exists::<Combat>)
                .run_if(in_state(AppState::Flight)),
        );
    }
}

/// Metres from the hook to the crate's middle once the winch has taken the slack in, and how
/// fast it does (m/s).
const ROPE: f64 = 9.0;
const WINCH: f64 = 2.0;
/// The hook above the helicopter's lowest point (metres).
const HOOK_UP: f32 = 1.0;
/// It hooks a crate this close (metres, horizontally) hovering this low (ft above the ground)
/// and this slowly (knots).
const HOOK_REACH: f64 = 8.0;
const HOOK_FEET: f32 = 50.0;
const HOOK_KNOTS: f32 = 15.0;
/// The four strops meet this far above the crate's top (metres).
const APEX: f32 = 2.0;
/// Physics step (s); the swing's damping (1/s, relative to the hook), the air's drag and the
/// ground's friction (1/s).
const STEP: f64 = 1.0 / 120.0;
const SWING_DAMP: f64 = 0.8;
const DRAG: f64 = 0.05;
const FRICTION: f64 = 6.0;
const GRAVITY: f64 = 9.81;
/// A hook or a drop the campaign has not taken up in this long (s) is given up.
const CONFIRM: f32 = 3.0;
const ROPE_COLOUR: u8 = 0xEE;

/// The crate's middle above the ground at rest (metres).
fn rest_height() -> f64 {
    CRATE_SIZE.z as f64 * 0.5
}

/// A slung crate: its middle as a point mass on a rope from the hook (Bevy metres).
#[derive(Debug, Clone, Copy)]
pub struct Swing {
    pub pos: DVec3,
    pub vel: DVec3,
    hook: DVec3,
    /// The rope paid out (the winch takes it in to `ROPE`).
    rope: f64,
}

impl Swing {
    pub fn new(pos: DVec3, hook: DVec3) -> Self {
        Self { pos, vel: DVec3::ZERO, hook, rope: pos.distance(hook).max(ROPE) }
    }

    /// The hook moved to `hook` in `dt` seconds: the crate follows on its rope, swinging less
    /// and less, and rests on the ground (at height `ground`).
    pub fn advance(&mut self, hook: DVec3, dt: f64, ground: f64) {
        // A jump (a respawn, a long stall) does not fling the crate: it is carried along.
        if hook.distance(self.hook) > 200.0 || dt > 0.5 {
            self.pos += hook - self.hook;
            self.vel = DVec3::ZERO;
            self.hook = hook;
            return;
        }
        let from = self.hook;
        let hook_vel = if dt > 0.0 { (hook - from) / dt } else { DVec3::ZERO };
        // The whole frame in equal steps of at most `STEP`: the crate is where the hook is in
        // time (a remainder carried over left it behind by a different bit every frame, a
        // jitter at speed).
        let n = (dt / STEP).ceil().max(1.0) as usize;
        for k in 1..=n {
            self.step(from.lerp(hook, k as f64 / n as f64), hook_vel, dt / n as f64, ground);
        }
        self.hook = hook;
    }

    fn step(&mut self, hook: DVec3, hook_vel: DVec3, dt: f64, ground: f64) {
        self.rope = (self.rope - WINCH * dt).max(ROPE);
        self.vel.y -= GRAVITY * dt;
        self.vel *= (-DRAG * dt).exp();
        // On a taut rope the swing dies down.
        if self.pos.distance(hook) >= self.rope * 0.98 {
            self.vel = hook_vel + (self.vel - hook_vel) * (-SWING_DAMP * dt).exp();
        }
        self.pos += self.vel * dt;
        let d = self.pos - hook;
        let len = d.length();
        if len > self.rope {
            let n = d / len;
            self.pos = hook + n * self.rope;
            let out = (self.vel - hook_vel).dot(n);
            if out > 0.0 {
                self.vel -= n * out;
            }
        }
        let bottom = ground + rest_height();
        if self.pos.y < bottom {
            self.pos.y = bottom;
            self.vel.y = self.vel.y.max(0.0);
            let f = (-FRICTION * dt).exp();
            self.vel.x *= f;
            self.vel.z *= f;
        }
    }

    /// The crate stands on the ground (at height `ground`).
    pub fn grounded(&self, ground: f64) -> bool {
        self.pos.y <= ground + rest_height() + 0.05
    }
}

/// A crate as drawn: its entity, its rope's entity and mesh, where it is and its heading
/// (radians clockwise from north), and its swing while slung.
struct Shown {
    root: Entity,
    rope: Entity,
    mesh: Handle<Mesh>,
    at: DVec3,
    heading: f32,
    swing: Option<Swing>,
}

/// The crates shown (by site and load), the one under the own helicopter (since hooked, s;
/// whether the campaign has it carried yet) and the own ones set down until the campaign has
/// them delivered (where, for how long, s).
#[derive(Resource, Default)]
struct Sling {
    crates: HashMap<(u32, u8), Shown>,
    mine: Option<((u32, u8), f32, bool)>,
    dropped: Vec<((u32, u8), DVec3, f32)>,
}

/// Where a crate is shown: at rest there (heading), or hung from a hook (the helicopter's
/// heading).
#[derive(Clone, Copy)]
enum Place {
    Rest(DVec3, f32),
    Hung(DVec3, f32),
}

/// A helicopter's heading (radians clockwise from north) from its attitude.
fn heading_of(rot: Quat) -> f32 {
    let f = rot * Vec3::NEG_Z;
    f.x.atan2(-f.z)
}

/// Metres east, north to Bevy metres at height `y`.
fn bevy_at(p: DVec2, y: f64) -> DVec3 {
    DVec3::new(p.x, y, -p.y)
}

/// A spot of a crate row (metres east, north, and a heading) as a crate standing there, on the
/// rolling hills.
fn standing(hills: &Hills, ((east, north), heading): ((f64, f64), f32)) -> (DVec3, f32) {
    (resting(hills, DVec2::new(east, north)), heading)
}

/// Where a crate stands at a ground point (metres east, north): on the ground there.
fn resting(hills: &Hills, p: DVec2) -> DVec3 {
    bevy_at(p, hills.ground_m(p.x, p.y) as f64 + rest_height())
}

/// The ground (Bevy y) under a Bevy point.
fn ground_under(hills: &Hills, p: DVec3) -> f64 {
    hills.ground_m(p.x, -p.z) as f64
}

/// The turn from `a` towards `b` (radians, the short way round).
fn turn(a: f32, b: f32) -> f32 {
    (b - a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

#[allow(clippy::too_many_arguments)]
fn sling(
    mut commands: Commands,
    campaign: Option<ResMut<ActiveCampaign>>,
    combat: Res<Combat>,
    k: Res<FlightConstants>,
    player: Query<(&Aircraft, &WorldPos), With<Player>>,
    mut msgs: ResMut<HudMessages>,
    time: Res<Time>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sling: ResMut<Sling>,
    mut q: Query<(&mut WorldPos, &mut Transform, &mut Visibility), Without<Player>>,
) {
    let sling = &mut *sling;
    let Some(mut c) = campaign else {
        for (_, s) in sling.crates.drain() {
            commands.entity(s.root).try_despawn();
            commands.entity(s.rope).try_despawn();
        }
        sling.mine = None;
        sling.dropped.clear();
        return;
    };
    let dt = time.delta_secs();
    let me = c.me;
    let state = c.state();
    let on_sortie = state.players.iter().any(|p| p.id == me && p.sortie.is_some());
    // The own helicopter: its hook, heading, height above the ground (ft) and speed (knots).
    let own = player
        .single()
        .ok()
        .filter(|(a, _)| {
            c.phase == Phase::Sortie && on_sortie && k.heli.is_some() && a.ended.is_none()
        })
        .map(|(a, pos)| {
            let hook = pos.0 + (a.orientation * Vec3::Y * HOOK_UP).as_dvec3();
            (hook, heading_of(a.orientation), a.altitude - a.ground_altitude, a.knots())
        });
    // The other pilots' hooks and headings.
    let sim = &combat.sim;
    let hills = sim.hills();
    let remote: HashMap<u8, (DVec3, f32)> = sim
        .remote_pilots()
        .iter()
        .filter(|p| p.id != sim.active_pilot() && p.chute.is_none())
        .map(|p| {
            let (pos, rot) = crate::sim_render::remote_pose(&combat, p);
            (p.id, (pos + (rot * Vec3::Y * HOOK_UP).as_dvec3(), heading_of(rot)))
        })
        .collect();

    // Where every crate of the sites waiting for cargo is.
    let mut want: Vec<((u32, u8), Place, DVec2, bool)> = Vec::new();
    for d in state.deployments.iter().filter(|d| !d.built) {
        let pad = map_metres(d.x, d.y);
        for leg in 0..d.loads {
            let key = (d.id, leg);
            let at_home = || {
                stealth_host::cargo::crate_home(sim, state, d.id, leg).map(|c| standing(&hills, c))
            };
            let cs = d.crate_state(leg);
            let delivered = matches!(cs, CrateState::Delivered { .. });
            let dropped = sling.dropped.iter().find(|x| x.0 == key);
            let place = match cs {
                _ if sling.mine.is_some_and(|m| m.0 == key) => own.map(|o| Place::Hung(o.0, o.1)),
                _ if dropped.is_some() && !delivered => {
                    let shown = sling.crates.get(&key).map_or(0.0, |s| s.heading);
                    dropped.map(|x| Place::Rest(x.1, shown))
                }
                CrateState::AtBase if d.crate_waiting(leg) => {
                    at_home().map(|(p, h)| Place::Rest(p, h))
                }
                CrateState::AtBase => None,
                CrateState::Carried(p) if p == me => own.map(|o| Place::Hung(o.0, o.1)),
                CrateState::Carried(p) => remote.get(&p.0).map(|r| Place::Hung(r.0, r.1)),
                CrateState::Delivered { east, north, heading } => Some(Place::Rest(
                    resting(&hills, pad + DVec2::new(east as f64, north as f64)),
                    heading,
                )),
            };
            let waiting = matches!(cs, CrateState::AtBase) && d.crate_waiting(leg);
            if let Some(place) = place {
                want.push((key, place, pad, waiting));
            }
        }
    }
    // The AI helicopters' crates: waiting for them in the row of their base, slung under them,
    // set down at the drop (a new site's until the campaign has it delivered: then it is the
    // site's).
    for (i, u) in sim.flights().iter().enumerate() {
        let Some(f) = sim.campaign_flight(i) else { continue };
        let Some(run) = f.supply.filter(|r| r.pickup.is_some() || r.hooked || r.set_down) else {
            continue;
        };
        if u.flags & stealth_sim::fflags::EXISTS == 0 {
            continue;
        }
        let key = if run.unit == stealth_sim::supply::NO_UNIT {
            let leg = state
                .deployments
                .iter()
                .filter(|d| !d.built)
                .find_map(|d| d.ai.iter().find(|a| a.1 == f.tag).map(|a| (d.id, a.0)));
            let Some(key) = leg else { continue };
            key
        } else {
            (f.tag, u8::MAX)
        };
        if want.iter().any(|w| w.0 == key) {
            continue;
        }
        let shown = sling.crates.get(&key);
        let place = if run.hooked {
            let (east, north, alt) = unit_position(combat.prev_flights.get(i), u, combat.alpha);
            let rot = attitude(u.heading, u.pitch, u.bank);
            let hook = wu_to_bevy(east, north, alt) + (rot * Vec3::Y * HOOK_UP).as_dvec3();
            Place::Hung(hook, heading_of(rot))
        } else if let Some(p) = run.pickup {
            let (at, _) = standing(&hills, (stealth_host::cargo::metres(p), 0.0));
            let row = if run.unit == stealth_sim::supply::NO_UNIT {
                stealth_host::cargo::crate_home(sim, state, key.0, key.1)
            } else {
                Some(stealth_host::cargo::supply_crate(sim, f.home as usize, f.tag))
            };
            Place::Rest(at, row.map_or(0.0, |r| r.1))
        } else {
            // Where it came down.
            let (at, h) = standing(&hills, (stealth_host::cargo::metres(run.drop), 0.0));
            Place::Rest(
                shown.map_or(at, |s| DVec3::new(s.at.x, at.y, s.at.z)),
                shown.map_or(h, |s| s.heading),
            )
        };
        want.push((key, place, DVec2::ZERO, false));
    }
    // The own crate: the campaign carries it (a host's at once, a server's a moment later), or
    // the hook did not count; set down, it is delivered.
    if let Some((key, since, confirmed)) = &mut sling.mine {
        *since += dt;
        let d = state.deployment(key.0);
        let cs = d.map(|d| d.crate_state(key.1));
        if cs == Some(CrateState::Carried(me)) {
            *confirmed = true;
        } else if *confirmed || *since > CONFIRM || own.is_none() {
            sling.mine = None;
        }
    }
    sling.dropped.retain_mut(|(key, _, age)| {
        *age += dt;
        let delivered = state
            .deployment(key.0)
            .is_some_and(|d| matches!(d.crate_state(key.1), CrateState::Delivered { .. }));
        !delivered && *age < CONFIRM * 2.0
    });
    if sling.mine.is_none()
        && let Some(&((id, leg), ..)) = want.iter().find(|(key, place, ..)| {
            matches!(place, Place::Hung(..))
                && state
                    .deployment(key.0)
                    .is_some_and(|d| d.crate_state(key.1) == CrateState::Carried(me))
        })
    {
        sling.mine = Some(((id, leg), 0.0, true));
    }

    // Draw them.
    sling.crates.retain(|key, s| {
        let keep = want.iter().any(|w| w.0 == *key);
        if !keep {
            commands.entity(s.root).try_despawn();
            commands.entity(s.rope).try_despawn();
        }
        keep
    });
    for &(key, place, _, _) in &want {
        let s = sling.crates.entry(key).or_insert_with(|| {
            let root = spawn_crate(&mut commands, &mut meshes, &palette);
            let mesh = meshes.add(
                Mesh::new(PrimitiveTopology::LineList, bevy::asset::RenderAssetUsages::default())
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::ZERO; 2]),
            );
            let rope = commands
                .spawn((
                    WorldPos::default(),
                    Transform::default(),
                    Visibility::Hidden,
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(palette.line_material(ROPE_COLOUR)),
                    MeshTag(mesh_tag(LEVEL_AIRCRAFT, 0, false)),
                    bevy::light::NotShadowCaster,
                    NoFrustumCulling,
                ))
                .id();
            let (at, heading) = match place {
                Place::Rest(p, h) => (p, h),
                Place::Hung(hook, h) => (hook - DVec3::Y * ROPE, h),
            };
            Shown { root, rope, mesh, at, heading, swing: None }
        });
        let ease = 1.0 - (-6.0 * dt as f64).exp();
        let mut tilt = Quat::IDENTITY;
        let hook = match place {
            Place::Rest(p, h) => {
                s.swing = None;
                // Set down by another pilot (or put back): it goes there.
                s.at = if s.at.distance(p) > 300.0 { p } else { s.at.lerp(p, ease) };
                s.heading += turn(s.heading, h) * ease as f32;
                None
            }
            Place::Hung(hook, h) => {
                let w = s.swing.get_or_insert_with(|| Swing::new(s.at, hook));
                let ground = ground_under(&hills, w.pos);
                w.advance(hook, dt as f64, ground);
                s.at = w.pos;
                // It turns slowly with the helicopter, and leans along its rope.
                s.heading += turn(s.heading, h) * (1.0 - (-0.5 * dt).exp());
                // (Lifting off, it leans in gradually: no flicker as it touches and leaves.)
                let up = (hook - s.at).as_vec3().normalize_or(Vec3::Y);
                let off = ((w.pos.y - ground - rest_height()) / 0.5).clamp(0.0, 1.0) as f32;
                tilt = Quat::IDENTITY.slerp(Quat::from_rotation_arc(Vec3::Y, up), off);
                Some(hook)
            }
        };
        let rot = tilt * Quat::from_rotation_y(-s.heading);
        if let Ok((mut pos, mut tf, mut vis)) = q.get_mut(s.root) {
            pos.0 = s.at;
            tf.rotation = rot;
            vis.set_if_neq(Visibility::Inherited);
        }
        if let Ok((mut pos, _, mut vis)) = q.get_mut(s.rope) {
            vis.set_if_neq(if hook.is_some() { Visibility::Inherited } else { Visibility::Hidden });
            if let Some(hook) = hook {
                pos.0 = hook;
                let rel = |p: Vec3| (s.at - hook).as_vec3() + rot * p;
                let (w, h, l) = (CRATE_SIZE.y * 0.5, CRATE_SIZE.z * 0.5, CRATE_SIZE.x * 0.5);
                let apex = rel(Vec3::Y * (h + APEX));
                let mut v = vec![Vec3::ZERO, apex];
                for (x, z) in [(-w, -l), (w, -l), (w, l), (-w, l)] {
                    v.extend([apex, rel(Vec3::new(x, h, z))]);
                }
                if let Some(mut m) = meshes.get_mut(&s.mesh) {
                    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, v);
                }
            }
        }
    }

    // The own helicopter hooks a crate hovering low over it, and sets it down by its helipad.
    let mut actions = Vec::new();
    match (sling.mine, own) {
        (None, Some((hook, _, feet, knots))) if feet <= HOOK_FEET && knots < HOOK_KNOTS => {
            let near = want
                .iter()
                .filter(|w| w.3)
                .filter_map(|w| match w.1 {
                    Place::Rest(p, _) => Some((w.0, (p - hook).xz().length())),
                    Place::Hung(..) => None,
                })
                .filter(|(_, d)| *d < HOOK_REACH)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some(((id, leg), _)) = near {
                sling.mine = Some(((id, leg), 0.0, false));
                actions.push(Action::HookCrate { id, leg });
                msgs.show("Cargo hooked");
            }
        }
        (Some((key, ..)), Some(_)) => {
            let pad = want.iter().find(|w| w.0 == key).map(|w| w.2);
            if let (Some(pad), Some(s)) = (pad, sling.crates.get_mut(&key))
                && s.swing.is_some_and(|w| w.grounded(ground_under(&hills, w.pos)))
            {
                let (east, north) = (s.at.x - pad.x, -s.at.z - pad.y);
                if (east.hypot(north) as f32) <= CRATE_DROP_METRES {
                    s.swing = None;
                    sling.mine = None;
                    sling.dropped.push((key, s.at, 0.0));
                    actions.push(Action::DropCrate {
                        id: key.0,
                        leg: key.1,
                        east: east as f32,
                        north: north as f32,
                        heading: s.heading,
                    });
                    msgs.show("Cargo set down");
                }
            }
        }
        _ => {}
    }
    c.actions.extend(actions);
}

fn cleanup(mut commands: Commands, mut sling: ResMut<Sling>) {
    for (_, s) in sling.crates.drain() {
        commands.entity(s.root).try_despawn();
        commands.entity(s.rope).try_despawn();
    }
    *sling = Sling::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The swing's angle from the vertical under the hook (radians).
    fn angle(s: &Swing, hook: DVec3) -> f64 {
        let d = s.pos - hook;
        (d.xz().length()).atan2(-d.y)
    }

    #[test]
    fn a_swing_dies_down_and_the_rope_holds() {
        let hook = DVec3::new(0.0, 40.0, 0.0);
        let t = 0.5f64;
        let mut s = Swing::new(hook + DVec3::new(t.sin(), -t.cos(), 0.0) * ROPE, hook);
        let a0 = angle(&s, hook);
        for _ in 0..(20 * 60) {
            s.advance(hook, 1.0 / 60.0, 0.0);
            assert!(s.pos.distance(hook) <= ROPE + 1e-6);
        }
        assert!(angle(&s, hook) < a0 * 0.1, "{} of {a0}", angle(&s, hook));
    }

    #[test]
    fn a_crate_follows_a_steady_flight_smoothly_at_uneven_frame_rates() {
        let mut hook = DVec3::new(0.0, 40.0, 0.0);
        let mut s = Swing::new(hook - DVec3::Y * ROPE, hook);
        let v = DVec3::new(30.0, 0.0, 0.0);
        let mut offsets = Vec::new();
        for k in 0..3000 {
            let dt = if k % 3 == 0 { 1.0 / 45.0 } else { 1.0 / 75.0 };
            hook += v * dt;
            s.advance(hook, dt, 0.0);
            if k > 2500 {
                offsets.push(s.pos - hook);
            }
        }
        for w in offsets.windows(2) {
            assert!(w[0].distance(w[1]) < 0.01, "{:?} -> {:?}", w[0], w[1]);
        }
    }

    #[test]
    fn a_crate_is_lifted_off_the_ground_and_never_goes_through_it() {
        // Hooked hovering at 50 ft: the winch lifts it.
        let mut hook = DVec3::new(0.0, 15.2 + HOOK_UP as f64, 0.0);
        let mut s = Swing::new(DVec3::new(2.0, rest_height(), 0.0), hook);
        assert!(s.grounded(0.0));
        for _ in 0..(15 * 60) {
            s.advance(hook, 1.0 / 60.0, 0.0);
        }
        assert!(!s.grounded(0.0));
        assert!((s.pos.distance(hook) - ROPE).abs() < 0.1);
        // Flown off and down to the ground: it trails, and rests on it.
        for k in 0..(20 * 60) {
            hook += DVec3::new(10.0 / 60.0, -0.4 / 60.0, 0.0);
            s.advance(hook, 1.0 / 60.0, 0.0);
            assert!(s.pos.y >= rest_height() - 1e-9, "step {k}: {}", s.pos.y);
        }
        assert!(s.grounded(0.0));
    }

    #[test]
    fn the_ais_heights_lift_a_crate_off_and_set_it_down() {
        use stealth_sim::supply::{HOOK_ALT, HOOK_SECONDS, SET_ALT};
        let hook = |alt: i32| DVec3::new(0.0, alt as f64 * crate::world::WU + HOOK_UP as f64, 0.0);
        // Hooked from its hover over the crate: off the ground before it flies away.
        let mut s = Swing::new(DVec3::new(1.0, rest_height(), 0.0), hook(HOOK_ALT));
        for _ in 0..(HOOK_SECONDS * 60) {
            s.advance(hook(HOOK_ALT), 1.0 / 60.0, 0.0);
        }
        assert!(!s.grounded(0.0), "{}", s.pos.y);
        // Down to its set-down height: the crate stands on the ground.
        for _ in 0..(5 * 60) {
            s.advance(hook(SET_ALT), 1.0 / 60.0, 0.0);
        }
        assert!(s.grounded(0.0), "{}", s.pos.y);
    }
}

//! The AC-130's side guns as seen and heard (`SimEvent::SideGunFired`,
//! `SimEvent::SideShellImpact`): the muzzle's flash and grey smoke, the flash lighting the
//! fuselage round the gun port (with the local lights on), each shell's glowing tracer on its
//! way down, and the burst where it lands (the 105 mm's blast, the 40 mm's sharp one), the guns'
//! reports and the bursts heard when their sound gets there.
//!
//! The shells are flown here from the muzzle the way the simulation flies them (gravity, no
//! drag), so their tracers show from any view; each ends at the burst the simulation reports.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::combat::Combat;
use crate::explosions::{Blast, Calibre, Explosions};
use crate::flight::{Aircraft, Player};
use crate::palette_material::{FLAG_LINE, PaletteMaterial, PaletteParams};
use crate::sim_render::SimObject;
use crate::world::WorldPos;

/// Seconds of a shell's flight its tracer streak shows: the 40 mm's, the 105 mm's.
const STREAK: [f64; 2] = [0.05, 0.035];
/// A shell's tracer glow: puffs left along its way, their size (m) and life (s).
const GLOW: [(f32, f32); 2] = [(0.9, 0.06), (1.4, 0.05)];
/// The muzzle flash's light: strength and seconds; it shines from this far outboard of the
/// muzzle (m), so it lights the fuselage round the port.
const LIGHT: [(f32, f32); 2] = [(1.4, 0.10), (3.0, 0.22)];
const LIGHT_OUT: f32 = 1.5;
/// Shells drawn at most.
const MAX_SHELLS: usize = 64;
/// Gravity (m/s^2), as the simulation's.
const G: f64 = 9.81;
/// A tracer colour (palette): the 40 mm's bright tracer.
const TRACER: u8 = crate::sim_render::TRACER;

pub fn calibre(gun: u8) -> Calibre {
    if gun == 0 { Calibre::Mm40 } else { Calibre::Mm105 }
}

/// Simulation metres (east, north, up) to the Bevy world.
fn bevy_of(m: [f64; 3]) -> DVec3 {
    DVec3::new(m[0], m[2], -m[1])
}

struct Shell {
    pos: DVec3,
    vel: DVec3,
    gun: u8,
    age: f64,
}

/// A muzzle flash's light: on the own aircraft it rides along with the gun port.
struct Flash {
    entity: Entity,
    age: f32,
    life: f32,
    strength: f32,
    /// The gun (on the own aircraft, the muzzle in the aircraft frame), else where it fired.
    own: Option<usize>,
}

#[derive(Resource, Default)]
pub struct SideGunFx {
    shells: Vec<Shell>,
    lines: Vec<Entity>,
    flashes: Vec<Flash>,
    material: Option<Handle<PaletteMaterial>>,
}

pub struct SideGunsPlugin;

impl Plugin for SideGunsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SideGunFx>()
            .add_systems(OnExit(crate::settings::AppState::Flight), cleanup)
            .add_systems(
                Update,
                side_gun_fx
                    .after(crate::combat::CombatSystems)
                    .run_if(resource_exists::<Combat>)
                    .run_if(in_state(crate::settings::AppState::Flight)),
            );
    }
}

fn cleanup(mut commands: Commands, mut fx: ResMut<SideGunFx>) {
    let flashes: Vec<Entity> = fx.flashes.drain(..).map(|f| f.entity).collect();
    for e in fx.lines.drain(..).chain(flashes) {
        commands.entity(e).try_despawn();
    }
    *fx = SideGunFx::default();
}

/// The muzzle of gun `gun` on the own aircraft as drawn (Bevy world), and the barrel's
/// direction.
fn own_muzzle(a: &Aircraft, pos: &WorldPos, gun: usize) -> (DVec3, Vec3) {
    let at = pos.0 + (a.orientation * crate::aircraft::C130_MUZZLES[gun]).as_dvec3();
    (at, a.orientation * crate::aircraft::C130_BARREL)
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn side_gun_fx(
    mut commands: Commands,
    combat: Res<Combat>,
    mut fx: ResMut<SideGunFx>,
    mut ex: ResMut<Explosions>,
    mut reports: MessageWriter<crate::sound::GunReport>,
    mut bursts: MessageWriter<crate::sound::ShellBurst>,
    time: Res<Time>,
    mut materials: ResMut<Assets<PaletteMaterial>>,
    palette: Option<Res<crate::palette::Palette>>,
    player: Query<(&Aircraft, &WorldPos), (With<Player>, Without<SimObject>)>,
    mut q: Query<(&mut WorldPos, &mut Transform, &mut Visibility), With<SimObject>>,
) {
    let Some(palette) = palette else { return };
    let fx = &mut *fx;
    let own_id = combat.sim.active_pilot();
    let own = player.single().ok();
    let dm = |v: [i32; 3]| [v[0] as f64 / 10.0, v[1] as f64 / 10.0, v[2] as f64 / 10.0];
    for e in &combat.side_gun_fx {
        match *e {
            stealth_sim::SimEvent::SideGunFired { gun, pilot, from, vel } => {
                let g = gun as usize;
                let vel = bevy_of(dm(vel));
                // The own gun's flash and smoke at the muzzle as drawn; another's where his
                // simulation fired it.
                let mine = pilot == own_id && own.is_some();
                let (at, dir) = match own.filter(|_| mine) {
                    Some((a, p)) => own_muzzle(a, p, g),
                    None => (bevy_of(dm(from)), vel.as_vec3().normalize_or(Vec3::NEG_Y)),
                };
                // The own aircraft's velocity, else the shell's less its muzzle speed.
                let carry = match own.filter(|_| mine) {
                    Some((a, _)) => a.path_dir().normalize_or_zero() * a.knots() * 0.5144,
                    None => (vel - dir.as_dvec3() * stealth_sim::gunship::SHELL_SPEED[g]).as_vec3(),
                };
                ex.pending.push((at, Blast::AirMuzzle(calibre(gun), dir, carry)));
                reports.write(crate::sound::GunReport { at, calibre: calibre(gun) });
                if fx.shells.len() < MAX_SHELLS {
                    fx.shells.push(Shell { pos: at, vel, gun, age: 0.0 });
                }
                let (strength, life) = LIGHT[g];
                let light_at = at + (dir * LIGHT_OUT).as_dvec3();
                let entity = commands
                    .spawn((
                        SimObject,
                        WorldPos(light_at),
                        Transform::default(),
                        Visibility::Inherited,
                    ))
                    .id();
                fx.flashes.push(Flash { entity, age: 0.0, life, strength, own: mine.then_some(g) });
            }
            stealth_sim::SimEvent::SideShellImpact { gun, at, armour } => {
                let at = bevy_of(dm(at));
                let c = calibre(gun);
                ex.pending.push((at, if armour { Blast::ArmourHit(c) } else { Blast::Strike(c) }));
                bursts.write(crate::sound::ShellBurst { at, calibre: c });
                // The shell of that gun nearest the burst lands there.
                if let Some(k) = fx
                    .shells
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.gun == gun)
                    .min_by(|a, b| a.1.pos.distance(at).total_cmp(&b.1.pos.distance(at)))
                    .map(|(k, _)| k)
                {
                    fx.shells.swap_remove(k);
                }
            }
            _ => {}
        }
    }

    let dt = time.delta_secs() as f64;
    // The shells: on their way down, a glow left behind each frame; lost once well below the
    // ground or old (the simulation's burst ends them).
    for s in &mut fx.shells {
        s.pos += s.vel * dt - DVec3::Y * (0.5 * G * dt * dt);
        s.vel.y -= G * dt;
        s.age += dt;
    }
    fx.shells.retain(|s| s.age < 45.0 && s.pos.y > -200.0);
    for s in &fx.shells {
        let (size, life) = GLOW[s.gun as usize];
        ex.pending.push((s.pos, Blast::Tracer { size, life }));
    }
    let material = fx
        .material
        .get_or_insert_with(|| {
            materials.add(PaletteMaterial {
                params: PaletteParams {
                    word: 0xFF00 | TRACER as u32,
                    flags: FLAG_LINE,
                    ..default()
                },
                globals: palette.globals.clone(),
            })
        })
        .clone();
    while fx.lines.len() < fx.shells.len() {
        fx.lines.push(crate::sim_render::spawn_tracer(&mut commands, material.clone()));
    }
    for (k, &e) in fx.lines.iter().enumerate() {
        let Ok((mut pos, mut tf, mut vis)) = q.get_mut(e) else { continue };
        let Some(s) = fx.shells.get(k) else {
            vis.set_if_neq(Visibility::Hidden);
            continue;
        };
        pos.0 = s.pos;
        let streak = (-s.vel * STREAK[s.gun as usize].min(s.age)).as_vec3();
        tf.rotation = Quat::from_rotation_arc(Vec3::Z, streak.normalize_or(Vec3::Z));
        tf.scale = Vec3::new(1.0, 1.0, streak.length().max(0.01));
        vis.set_if_neq(Visibility::Inherited);
    }

    // The muzzle flashes' lights: fading, the own one riding along with its gun port.
    let dt = dt as f32;
    fx.flashes.retain_mut(|f| {
        f.age += dt;
        if f.age >= f.life {
            commands.entity(f.entity).try_despawn();
            return false;
        }
        if let (Some(g), Some((a, p))) = (f.own, own)
            && let Ok((mut pos, ..)) = q.get_mut(f.entity)
        {
            let (at, dir) = own_muzzle(a, p, g);
            pos.0 = at + (dir * LIGHT_OUT).as_dvec3();
        }
        let t = f.age / f.life;
        let s = (1.0 - t).powi(2) * f.strength;
        commands.entity(f.entity).insert(crate::lighting::LightEmitter {
            color: Vec3::new(1.0, 0.78 - 0.3 * t, 0.45 - 0.35 * t) * s,
            range: 60.0,
        });
        true
    });
}

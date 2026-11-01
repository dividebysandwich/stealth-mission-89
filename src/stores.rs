//! The stores the flyable jets hang on their pylons and rails (`assets/models/stores.json`,
//! written by tools/models/aircraft.py with the models of the stores and their racks): drawn
//! on the own aircraft (outside and from the cockpit) as loaded, each round gone once it has
//! left. The simulation launches each from where it hangs (`Sim::set_hardpoints`), and a store
//! that has just left is drawn from its rail at first, going over to its flight path (no jump
//! as it changes from the aircraft's model to its own).

use std::collections::HashMap;
use std::sync::LazyLock;

use bevy::math::Affine3A;
use bevy::prelude::*;
use serde::Deserialize;

use crate::combat::Combat;
use crate::flight::Player;
use crate::models::{ModelAssets, ModelLibrary, ModelState};
use crate::player::{AIRCRAFT_MODEL_SCALE, OwnModel, OwnModelInterior};
use crate::world::WorldPos;

/// Seconds a store that has left is drawn going over from its rail to its flight path.
const RELEASE_BLEND: f32 = 0.5;
/// State slot hiding the AI's own wingtip Sidewinders on the flyable F-16 (its stores are hung
/// here instead); 1 on the own aircraft.
pub const STORES_SLOT: usize = 28;
/// The camera pod's and the fuel tank's weapon types: never released.
const CAMERA: i32 = 16;
const FUEL_TANK: i32 = 17;
/// State slot showing a missile's motor flame (`_s14v1`).
const MOTOR_SLOT: usize = 14;

#[derive(Deserialize)]
struct Table {
    /// Weapon type -> its store's model number.
    models: HashMap<u8, i32>,
    /// Jet model number -> its stations, in loadout order.
    jets: HashMap<i32, Vec<Station>>,
}

#[derive(Deserialize)]
struct Station {
    station: usize,
    /// Suffix of the model's `_lod0` node the stores hang from ("" the airframe, else a
    /// part's: the Tornado's pylons turning against the sweep).
    parent: String,
    /// Model units in the parent's frame (Blender's: x right, y forward, z up) and degrees
    /// (the store's top turned right).
    at: [f32; 3],
    roll: f32,
    /// Weapon type -> its rack's model and its rounds' lugs (x, y, z, roll), the last one
    /// first to leave.
    loads: HashMap<u8, Load>,
}

#[derive(Deserialize)]
struct Load {
    rack: Option<i32>,
    rounds: Vec<[f32; 4]>,
}

static TABLE: LazyLock<Table> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../assets/models/stores.json"))
        .expect("assets/models/stores.json")
});

/// The model drawn for a store of weapon type `w` that has left a station.
pub fn weapon_model(w: i32) -> Option<i32> {
    u8::try_from(w).ok().and_then(|w| TABLE.models.get(&w).copied())
}

/// The stations of aircraft model `model` (e.g. "aircraft_023"), if it hangs stores.
fn stations(model: &str) -> Option<&'static [Station]> {
    let n: i32 = model.strip_prefix("aircraft_")?.parse().ok()?;
    TABLE.jets.get(&n).map(Vec::as_slice)
}

/// Blender's frame (x right, y forward, z up) to the models' glTF frame (x right, y up, z aft).
fn gltf(p: [f32; 3]) -> Vec3 {
    Vec3::new(p[0], p[2], -p[1])
}

/// A store's attitude rolled `roll` degrees (its top to the right) about its axis.
fn rolled(roll: f32) -> Quat {
    Quat::from_rotation_z(-roll.to_radians())
}

pub struct StoresPlugin;

impl Plugin for StoresPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                aim_hardpoints
                    .before(crate::combat::CombatSystems)
                    .run_if(resource_exists::<Combat>),
                (
                    (own_loads, hang_stores.after(crate::sim_render::sync_remote_pilots)).chain(),
                    // After the loads are this frame's: the round that left is the one past
                    // those still on its station.
                    release_stores
                        .after(own_loads)
                        .after(crate::sim_render::sync_objects)
                        .after(crate::sim_render::sync_remote_pilots),
                )
                    .in_set(crate::sim_render::SimRenderSystems)
                    .after(crate::combat::CombatSystems)
                    .run_if(resource_exists::<Combat>),
            )
                .run_if(in_state(crate::settings::AppState::Flight)),
        );
    }
}

/// The stores a jet's model hangs (the own aircraft's, outside and seen from the cockpit, and
/// the other pilots'): its model, and per station the weapon type (-1 none) and the rounds
/// left on it.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct StoreLoads {
    pub model: &'static str,
    pub loads: Vec<(i32, i32)>,
}

impl StoreLoads {
    /// Another pilot's, as sent (`RemotePilot::stores`); None if his model hangs none or none
    /// were sent (an older game's).
    pub fn remote(model: &'static str, stores: &[(u8, u8)]) -> Option<Self> {
        if stores.is_empty() {
            return None;
        }
        stations(model)?;
        let loads = stores.iter().map(|&(w, n)| (if w == 255 { -1 } else { w as i32 }, n as i32));
        Some(Self { model, loads: loads.collect() })
    }
}

/// What a model has hung: per station, the weapon type, its rounds' and its rack's entities.
#[derive(Component, Default)]
pub struct Hung {
    stations: Vec<Option<(i32, Vec<Entity>, Option<Entity>)>>,
    /// Frames spent looking for a station's node not spawned yet.
    waited: u32,
}

/// Frames to wait for a model's nodes before giving up on the stations hung from them.
const NODE_WAIT: u32 = 600;

/// The own aircraft's models hang what the simulation has loaded.
#[allow(clippy::type_complexity)]
fn own_loads(
    mut commands: Commands,
    combat: Res<Combat>,
    settings: Res<crate::settings::Settings>,
    mut holders: Query<
        (Entity, Option<&mut StoreLoads>),
        Or<(With<OwnModel>, With<OwnModelInterior>)>,
    >,
) {
    let model = settings.aircraft.spec().model;
    if stations(model).is_none() {
        return;
    }
    let loads = combat.sim.bays().0.to_vec();
    for (e, has) in &mut holders {
        match has {
            Some(mut l) => {
                if l.model != model || l.loads != loads {
                    *l = StoreLoads { model, loads: loads.clone() };
                }
            }
            None => {
                commands.entity(e).insert(StoreLoads { model, loads: loads.clone() });
            }
        }
    }
}

/// The stores hung on the models as loaded, each round shown while it is still on its station.
#[allow(clippy::type_complexity)]
fn hang_stores(
    mut commands: Commands,
    mut library: ResMut<ModelLibrary>,
    server: Res<AssetServer>,
    assets: ModelAssets,
    mut holders: Query<(Entity, &StoreLoads, Option<&mut Hung>, Has<OwnModelInterior>)>,
    names: Query<(Entity, &Name)>,
    parents: Query<&ChildOf>,
    mut shown: Query<&mut Visibility>,
) {
    let file = library.load(&server, "aircraft");
    for (holder, loads, hung, interior) in &mut holders {
        let Some(stations) = stations(loads.model) else { continue };
        let Some(mut hung) = hung else {
            commands.entity(holder).insert(Hung::default());
            continue;
        };
        hung.stations.resize_with(stations.len(), || None);
        for (k, st) in stations.iter().enumerate() {
            let (w, left) = loads.loads.get(st.station).copied().unwrap_or((-1, 0));
            let load = u8::try_from(w).ok().and_then(|w| st.loads.get(&w));
            if hung.stations[k].as_ref().map(|h| h.0) != load.map(|_| w) {
                if let Some((_, rounds, rack)) = hung.stations[k].take() {
                    for e in rounds.into_iter().chain(rack) {
                        commands.entity(e).try_despawn();
                    }
                }
                let Some(load) = load else { continue };
                if hung.waited > NODE_WAIT {
                    continue;
                }
                // The node the station hangs from (the cockpit view's airframe is the holder).
                let node = format!("{}_lod0{}", loads.model, st.parent);
                let parent = if interior && st.parent.is_empty() {
                    Some(holder)
                } else {
                    names.iter().find_map(|(e, n)| {
                        (n.as_str() == node && parents.iter_ancestors(e).any(|a| a == holder))
                            .then_some(e)
                    })
                };
                // Not spawned yet: next frame.
                let Some(parent) = parent else {
                    hung.waited += 1;
                    continue;
                };
                let mut spawn = |model: i32, at: [f32; 3], roll: f32| {
                    let e = assets.spawn(
                        &mut commands,
                        &file,
                        &format!("aircraft_{model:03}"),
                        crate::palette_material::LEVEL_AIRCRAFT,
                        true,
                    )?;
                    commands.entity(e).insert((
                        Transform::from_translation(gltf(at)).with_rotation(rolled(roll)),
                        Visibility::Inherited,
                    ));
                    commands.entity(parent).add_child(e);
                    Some(e)
                };
                let model = TABLE.models[&(w as u8)];
                let rounds: Option<Vec<Entity>> =
                    load.rounds.iter().map(|r| spawn(model, [r[0], r[1], r[2]], r[3])).collect();
                let Some(rounds) = rounds else { continue };
                let rack = load.rack.and_then(|m| spawn(m, st.at, st.roll));
                hung.stations[k] = Some((w, rounds, rack));
            }
            // The rounds still there: the first `left` of them (the camera pod and a tank used
            // up stay on).
            let stays = matches!(w, CAMERA | FUEL_TANK);
            if let Some((_, rounds, _)) = &hung.stations[k] {
                for (i, &e) in rounds.iter().enumerate() {
                    if let Ok(mut v) = shown.get_mut(e) {
                        v.set_if_neq(if stays || (i as i32) < left {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        });
                    }
                }
            }
        }
    }
}

/// Where each station's next store hangs on the own aircraft (from the model as last drawn),
/// for the simulation to launch it from there.
fn aim_hardpoints(
    mut combat: ResMut<Combat>,
    settings: Res<crate::settings::Settings>,
    model: Query<(&GlobalTransform, &Hung), (With<OwnModel>, Without<OwnModelInterior>)>,
    rounds: Query<&GlobalTransform>,
) {
    let spec = settings.aircraft.spec();
    let Some(stations) = stations(spec.model) else { return };
    let Ok((root, hung)) = model.single() else { return };
    let to_model = root.affine().inverse();
    let lift = spec.model_lift;
    let s = AIRCRAFT_MODEL_SCALE as f32;
    let (bays, _) = combat.sim.bays();
    let mut points = vec![[0.0, 0.0, 0.0]; bays.len()];
    for (k, st) in stations.iter().enumerate() {
        let Some(point) = points.get_mut(st.station) else { continue };
        let left = bays[st.station].1;
        let next = hung
            .stations
            .get(k)
            .and_then(|h| h.as_ref())
            .and_then(|(_, r, _)| rounds.get(*r.get(usize::try_from(left - 1).ok()?)?).ok());
        let p = next.map_or(gltf(st.at), |g| to_model.transform_point3(g.translation()));
        *point = [p.x * s, -p.z * s, p.y * s + lift];
    }
    combat.sim.set_hardpoints(&points);
}

/// A store that has left: drawn from where it hung on the aircraft (in the model's frame),
/// going over to its flight path.
struct Release {
    entity: Entity,
    /// The model it left (the own aircraft's or another pilot's).
    holder: Entity,
    hung: Affine3A,
    age: f32,
}

/// The stores that have just left (their models taking over from the hung rounds), the own and
/// the other pilots', and the motor flames of those in flight.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn release_stores(
    combat: Res<Combat>,
    time: Res<Time>,
    vis: Res<crate::sim_render::SimVisuals>,
    remote: Res<crate::sim_render::RemoteVisuals>,
    own: Query<Entity, (With<OwnModel>, Without<OwnModelInterior>)>,
    holders: Query<(&GlobalTransform, &Hung, &StoreLoads)>,
    player: Query<(&WorldPos, &Transform), (With<Player>, Without<crate::sim_render::SimObject>)>,
    settings: Res<crate::settings::Settings>,
    rounds: Query<&GlobalTransform, Without<crate::sim_render::SimObject>>,
    mut proj: Query<
        (&mut WorldPos, &mut Transform, Option<&mut ModelState>),
        (With<crate::sim_render::SimObject>, Without<Player>),
    >,
    mut releases: Local<Vec<Release>>,
    mut seen: Local<HashMap<(u8, u8), Entity>>,
) {
    // Every store in flight that left a station: (key, its model, the model it left).
    let own_model = own.single().ok();
    let me = combat.sim.active_pilot();
    let mut flying: Vec<((u8, u8), Entity, Option<Entity>, stealth_sim::Projectile)> = Vec::new();
    for (s, m) in combat.sim.projectiles().iter().enumerate() {
        if let Some((e, ..)) = vis.proj[s] {
            flying.push(((me, s as u8), e, own_model, *m));
        }
    }
    for p in combat.sim.remote_pilots().iter().filter(|p| p.id != me) {
        let holder = remote.plane(p.id);
        for &(slot, m) in &p.missiles {
            if let Some(e) = remote.missile((p.id, slot)) {
                flying.push(((p.id, slot), e, holder, m));
            }
        }
    }
    let mut fresh = Vec::new();
    for &(key, e, holder, m) in &flying {
        if m.station.is_none() {
            continue;
        }
        if let Ok((_, _, Some(mut state))) = proj.get_mut(e) {
            let v = (crate::trails::is_missile(m.spec) && m.motor_visible()) as i16;
            if state.0[MOTOR_SLOT] != v {
                state.0[MOTOR_SLOT] = v;
            }
        }
        if seen.get(&key) != Some(&e) {
            fresh.push((key, e, holder, m));
        }
    }
    seen.clear();
    seen.extend(flying.iter().map(|&(key, e, ..)| (key, e)));
    // Rounds of each model's station gone this frame (a ripple can release several at once).
    let mut gone: HashMap<(Entity, u8), i32> = HashMap::new();
    for (_, e, holder, m) in fresh {
        let (Some(holder), Some(st)) = (holder, m.station) else { continue };
        let Ok((root, hung, loads)) = holders.get(holder) else { continue };
        // The round that left: the one after those still on the station.
        let n = gone.entry((holder, st)).or_insert(0);
        let round = loads.loads.get(st as usize).map_or(0, |l| l.1) + *n;
        *n += 1;
        let Some(k) =
            stations(loads.model).and_then(|sts| sts.iter().position(|x| x.station == st as usize))
        else {
            continue;
        };
        let Some(g) = hung
            .stations
            .get(k)
            .and_then(|h| h.as_ref())
            .and_then(|(_, r, _)| r.get(usize::try_from(round).ok()?.min(r.len().checked_sub(1)?)))
            .and_then(|&r| rounds.get(r).ok())
        else {
            continue;
        };
        // The round in the model's frame.
        let hung = root.affine().inverse() * g.affine();
        releases.push(Release { entity: e, holder, hung, age: 0.0 });
    }
    let dt = time.delta_secs();
    let lift = settings.aircraft.spec().model_lift;
    let own_pose = player
        .single()
        .ok()
        .map(|(p, tf)| (p.0 + (tf.rotation * Vec3::Y * lift).as_dvec3(), tf.rotation));
    releases.retain_mut(|r| {
        // The model it left where it is now (the own: on the flight model's pose).
        let pose = if Some(r.holder) == own_model {
            own_pose
        } else {
            proj.get(r.holder).ok().map(|(p, tf, _)| (p.0, tf.rotation))
        };
        let Some((origin, attitude)) = pose else { return false };
        let Ok((mut p, mut drawn, _)) = proj.get_mut(r.entity) else {
            // Not spawned yet.
            return r.age == 0.0;
        };
        let (_, rot, at) = r.hung.to_scale_rotation_translation();
        let hung = origin + (attitude * at * AIRCRAFT_MODEL_SCALE as f32).as_dvec3();
        let t = (r.age / RELEASE_BLEND).clamp(0.0, 1.0);
        let w = 1.0 - t * t * (3.0 - 2.0 * t);
        p.0 = p.0.lerp(hung, w as f64);
        drawn.rotation = drawn.rotation.slerp(attitude * rot, w);
        r.age += dt;
        r.age < RELEASE_BLEND
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aircraft::AircraftKind;

    /// Every station of the jets hanging stores, and every store it takes, is hung somewhere
    /// (tools/models/aircraft.py rebuilt after a change to src/loadout.rs).
    #[test]
    fn every_station_hangs_its_stores() {
        for kind in
            [AircraftKind::F16, AircraftKind::Tornado, AircraftKind::A10, AircraftKind::AV8B]
        {
            let hung = stations(kind.spec().model).unwrap_or_else(|| panic!("{kind:?}"));
            let lay = crate::loadout::layout(kind, false);
            assert_eq!(hung.len(), lay.hardpoints.len(), "{kind:?}");
            for (i, (h, st)) in lay.hardpoints.iter().zip(hung).enumerate() {
                assert_eq!(st.station, i, "{kind:?}");
                for w in h.loads {
                    let load = st.loads.get(w).unwrap_or_else(|| panic!("{kind:?} {} {w}", h.name));
                    assert_eq!(load.rounds.len(), h.rounds(*w) as usize, "{kind:?} {} {w}", h.name);
                    assert!(weapon_model(*w as i32).is_some());
                }
            }
        }
    }
}

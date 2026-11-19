//! The scenes behind the main menu (`director.rs` takes them in turn, a few seconds each, under
//! a weather and a time of day that change every few scenes): the night on Tabor's apron
//! (`apron.rs`), aircraft, helicopters, drones and ships in action seen from outside, cockpits
//! and the operators' consoles. In VR the player stands on the apron, life size, with the
//! menu's panel between the tower and the aircraft, and the scene stays.

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;

use crate::palette::{Palette, TimeOfDay};
use crate::palette_material::PaletteMaterial;
use crate::settings::{AppState, Settings};
use crate::world::{WU, WorldAnchor, WorldPos};

mod apron;
mod carrier;
mod cockpits;
mod comanche;
mod consoles;
pub mod director;
mod drones;
mod flyers;
mod fx;
mod gunship;
mod helmet;
mod lift;
mod pod;
mod sam;
mod slam;
mod stage;
mod strike;

use director::Director;

pub struct MenuScenePlugin;

/// The director's work (the scene changes, the look applied), before the scenes'.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectorSet;

/// The scenes' systems (spawning their things, moving them and the camera).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShotSet;

impl Plugin for MenuScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), spawn_scene)
            .add_systems(OnExit(AppState::Menu), (despawn_scene, crate::world::unload_theater))
            .configure_sets(
                Update,
                (DirectorSet, ShotSet.after(DirectorSet)).run_if(resource_exists::<Director>),
            )
            .add_systems(
                Update,
                (director::direct, apply_look, |mut f: ResMut<SceneFov>| f.1 = false)
                    .chain()
                    .in_set(DirectorSet),
            )
            .add_systems(
                Update,
                (
                    (fit_view, offset_view).chain(),
                    track_viewpoint,
                    match_overlay_msaa,
                    director::snap.run_if(|| std::env::var_os("SM89_MENU_SNAP").is_some()),
                )
                    .after(ShotSet)
                    .run_if(resource_exists::<MenuScene>),
            );
        apron::build(app);
        flyers::build(app);
        carrier::build(app);
        cockpits::build(app);
        comanche::build(app);
        consoles::build(app);
        drones::build(app);
        fx::build(app);
        gunship::build(app);
        helmet::build(app);
        lift::build(app);
        strike::build(app);
        sam::build(app);
        slam::build(app);
    }
}

/// The scene is drawn behind the menu.
#[derive(Resource)]
pub struct MenuScene;

/// Everything spawned for the scenes (gone when the menu closes).
#[derive(Component)]
struct SceneEntity;

/// What one scene spawned (gone when the next one comes).
#[derive(Component)]
pub(crate) struct ShotEntity;

/// The background sphere (made again with each look's palette).
#[derive(Component)]
struct SceneBackground;

#[derive(Component)]
pub(crate) struct SceneCamera;

/// The scene camera's vertical field of view (radians) the scene asks for; narrow windows
/// widen it (`fit_view`). The view turns to frame its subject right of the menu unless the
/// scene holds it still (`held`: a helmet's sight drawn on the view).
#[derive(Resource)]
pub(crate) struct SceneFov(pub f32, pub bool);

impl Default for SceneFov {
    fn default() -> Self {
        Self(std::f32::consts::FRAC_PI_4, false)
    }
}

/// The narrowest horizontal field of view (degrees), so the scene stays on the screen in
/// narrow windows.
const MIN_WIDTH_FOV: f32 = 72.0;

fn spawn_scene(
    mut commands: Commands,
    settings: Res<Settings>,
    root: Option<Res<crate::camera::XrRoot>>,
    (mut materials, mut buffers): (ResMut<Assets<PaletteMaterial>>, ResMut<Assets<ShaderBuffer>>),
    tod: Res<TimeOfDay>,
) {
    let director = Director::new(settings.vr);
    // The menu's other systems want a palette from the start (`apply_look` makes the look's).
    commands.insert_resource(Palette::new(
        director.look.theater,
        &tod,
        &mut materials,
        &mut buffers,
    ));
    commands.insert_resource(director);
    commands.insert_resource(SceneFov::default());
    crate::lighting::spawn_lights(&mut commands, &settings);
    if settings.vr {
        apron::spawn_standing_point(&mut commands, root.and_then(|r| r.0));
        return;
    }
    commands.spawn((
        Camera3d::default(),
        Camera { order: 0, ..default() },
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        Projection::Perspective(PerspectiveProjection { near: 0.5, far: 600_000.0, ..default() }),
        SceneCamera,
        SceneEntity,
        WorldAnchor,
        WorldPos(DVec3::ZERO),
        Transform::default(),
        bevy::camera::visibility::RenderLayers::from_layers(&[
            0,
            crate::camera::MAIN_BACKGROUND_LAYER,
            crate::camera::MAIN_LOD_LAYER,
        ]),
    ));
}

/// The palette's assets.
type PaletteAssets<'w> = (
    ResMut<'w, Assets<PaletteMaterial>>,
    ResMut<'w, Assets<ShaderBuffer>>,
    ResMut<'w, Assets<Mesh>>,
);

/// A new look: its theater (loaded if another), the palette and the sky for its time of day,
/// the background sphere in the new palette.
#[allow(clippy::too_many_arguments)]
fn apply_look(
    mut commands: Commands,
    mut director: ResMut<Director>,
    settings: Res<Settings>,
    theater: Option<Res<crate::world::Theater>>,
    (mut streamed, mut overrides): (
        ResMut<crate::world::StreamedCells>,
        ResMut<crate::world::TileOverrides>,
    ),
    (mut materials, mut buffers, mut meshes): PaletteAssets,
    (mut tod, mut sky): (ResMut<TimeOfDay>, ResMut<crate::sky::Sky>),
    cameras: Query<Entity, With<SceneCamera>>,
    backgrounds: Query<Entity, With<SceneBackground>>,
    root: Option<Res<crate::camera::XrRoot>>,
) {
    if !director.look_changed {
        return;
    }
    director.look_changed = false;
    let look = director.look.clone();
    if theater.is_none_or(|t| t.id != look.theater) {
        streamed.despawn_all(&mut commands);
        crate::world::insert_theater(
            &mut commands,
            look.theater,
            None,
            &mut streamed,
            &mut overrides,
        );
    }
    *tod = TimeOfDay::default();
    tod.frozen = true;
    let mut palette;
    if look.classic {
        // A night in the classic palette, held still, the moon over the field.
        tod.night_mission = true;
        tod.seed = apron::TOD_SEED;
        palette = Palette::new(look.theater, &tod, &mut materials, &mut buffers);
        apron::classic_sky(&mut sky, settings.vr);
    } else {
        tod.night_mission = !(6.0..18.0).contains(&look.hours);
        palette = Palette::new(look.theater, &tod, &mut materials, &mut buffers);
        *sky = crate::sky::Sky::of(look.theater, None);
        let shade = if look.weather.stratus.cover >= 90 { 0.8 } else { 0.0 };
        sky.at(look.hours, look.phase, shade, Some(&mut palette));
    }
    for e in &backgrounds {
        commands.entity(e).despawn();
    }
    let eye = cameras.iter().next().or_else(|| root.and_then(|r| r.0));
    if let Some(eye) = eye {
        let sphere = crate::camera::spawn_background(
            &mut commands,
            eye,
            &mut meshes,
            &palette,
            crate::camera::MAIN_BACKGROUND_LAYER,
        );
        commands.entity(sphere).insert((SceneEntity, SceneBackground));
    }
    commands.insert_resource(palette);
    // The world is drawn from now on (it needs the palette).
    commands.insert_resource(MenuScene);
}

/// The palette's sky and ground gradients follow the camera's height.
fn track_viewpoint(
    cams: Query<&WorldPos, With<SceneCamera>>,
    mut viewpoint: ResMut<crate::palette::Viewpoint>,
) {
    if let Ok(p) = cams.single() {
        let alt = (p.0.y / WU) as f32;
        if viewpoint.altitude_wu != alt {
            viewpoint.altitude_wu = alt;
        }
    }
}

/// The scene's field of view; a narrow window widens it (Bevy keeps the vertical field of
/// view).
fn fit_view(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    want: Res<SceneFov>,
    mut cams: Query<&mut Projection, With<SceneCamera>>,
) {
    let Ok(w) = windows.single() else { return };
    let aspect = w.width() / w.height().max(1.0);
    let half = (MIN_WIDTH_FOV.to_radians() / 2.0).tan();
    let fov = if want.0 < std::f32::consts::FRAC_PI_4 {
        want.0
    } else {
        want.0.max(2.0 * (half / aspect).atan())
    };
    for mut p in &mut cams {
        if let Projection::Perspective(p) = &mut *p
            && (p.fov - fov).abs() > 1e-4
        {
            p.fov = fov;
        }
    }
}

/// The main menu's right edge (logical pixels from the window's left edge, `menu.rs`).
const MENU_RIGHT: f32 = 460.0;

/// The scenes frame their subject in the middle of the view: turned left, the view shows it in
/// the middle of what the main menu leaves free on its right.
fn offset_view(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    fov: Res<SceneFov>,
    mut cams: Query<(&Projection, &mut Transform), With<SceneCamera>>,
    mut done: Local<Option<Quat>>,
) {
    let Ok(w) = windows.single() else { return };
    let Ok((Projection::Perspective(p), mut tf)) = cams.single_mut() else { return };
    // Turned already (the scene hasn't aimed the camera again this frame), or held.
    if *done == Some(tf.rotation) || fov.1 {
        return;
    }
    // The free part's middle, in normalised device coordinates.
    let k = (MENU_RIGHT / w.width().max(1.0)).min(0.45);
    let half = (p.fov / 2.0).tan() * w.width() / w.height().max(1.0);
    tf.rotation *= Quat::from_rotation_y((k * half).atan());
    *done = Some(tf.rotation);
}

/// The menu's egui camera draws over the scene: its MSAA has to be the scene camera's (else
/// it gets a picture of its own that nothing clears).
fn match_overlay_msaa(
    scene: Query<&Msaa, (With<SceneCamera>, Without<crate::menu::MenuCamera>)>,
    mut overlay: Query<&mut Msaa, With<crate::menu::MenuCamera>>,
) {
    let Ok(&want) = scene.single() else { return };
    for mut m in &mut overlay {
        if *m != want {
            *m = want;
        }
    }
}

#[allow(clippy::type_complexity)]
fn despawn_scene(
    mut commands: Commands,
    q: Query<Entity, Or<(With<SceneEntity>, With<ShotEntity>)>>,
    root: Option<Res<crate::camera::XrRoot>>,
) {
    // VR: the tracking root leaves the standing point before it goes.
    if let Some(r) = root.and_then(|r| r.0) {
        commands
            .entity(r)
            .remove::<(ChildOf, crate::camera::EyeCamera)>()
            .insert(Transform::IDENTITY);
    }
    for e in &q {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<MenuScene>();
    commands.remove_resource::<Director>();
    commands.remove_resource::<Palette>();
    apron::cleanup(&mut commands);
}

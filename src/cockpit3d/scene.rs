//! Cockpits and consoles for the menu's scenes (`menu_scene`): the model spawned under any
//! root (not the player's aircraft), its screens found so the scene can draw them with its own
//! made-up pictures (the flight's display systems don't run in the menu).

use bevy::app::Propagate;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::displays::{self, CanvasRole, TrackCamScreen};
use super::hud_glass::HudGlassMaterial;
use super::{CockpitMaterial, CockpitMaterials, layout, model};
use crate::aircraft::AircraftKind;
use crate::canvas::CanvasTexture;
use crate::mfd::MfdSide;
use crate::settings::Settings;

/// A screen of a scene's cockpit or console.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Left,
    Right,
    Third,
    Fourth,
    Hud,
    /// The RAH-66's multipurpose displays.
    Lmpd,
    Rmpd,
    Console(displays::ConsoleScreen),
}

impl Screen {
    fn role(self) -> CanvasRole {
        match self {
            Screen::Left => CanvasRole::Left,
            Screen::Right => CanvasRole::Right,
            Screen::Third => CanvasRole::Third,
            Screen::Fourth => CanvasRole::Fourth,
            Screen::Hud => CanvasRole::Hud,
            Screen::Lmpd => CanvasRole::Lmpd,
            Screen::Rmpd => CanvasRole::Rmpd,
            Screen::Console(s) => CanvasRole::Console(s),
        }
    }
}

/// Which model: an aircraft's front cockpit, or a console (`CONSOLE_MODEL`,
/// `DRONE_CONSOLE_MODEL`, the MH-60R's cabin console).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seat {
    Cockpit(AircraftKind),
    Console(&'static str),
}

/// What spawning a scene's cockpit takes.
#[derive(SystemParam)]
pub struct SceneCockpits<'w> {
    settings: Res<'w, Settings>,
    mats: Res<'w, CockpitMaterials>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<CockpitMaterial>>,
    hud_materials: ResMut<'w, Assets<HudGlassMaterial>>,
    images: ResMut<'w, Assets<Image>>,
    fonts: Res<'w, crate::font::Fonts>,
    models: Res<'w, model::CockpitModels>,
    server: Res<'w, AssetServer>,
    assets: model::ModelAssets<'w>,
}

impl SceneCockpits<'_> {
    /// The eye point of a seat (the aircraft's frame, m), the cockpit's root sits there.
    pub fn eye(&self, seat: Seat) -> Vec3 {
        match seat {
            Seat::Cockpit(kind) => {
                let s = Settings {
                    aircraft: kind,
                    gunner_cockpit: false,
                    asw_console: false,
                    vr: false,
                    ..self.settings.clone()
                };
                s.cockpit_origin()
            }
            Seat::Console(_) => Vec3::ZERO,
        }
    }

    /// The model is loaded.
    pub fn ready(&self, seat: Seat) -> bool {
        let (_, stem, _) = self.choice(seat);
        self.models.0.get(stem).is_some_and(|f| self.server.is_loaded_with_dependencies(f))
    }

    fn choice(&self, seat: Seat) -> (layout::PanelStyle, &'static str, f32) {
        match seat {
            Seat::Cockpit(kind) => {
                let s = Settings { aircraft: kind, gunner_cockpit: false, ..self.settings.clone() };
                let (style, stem) = s.cockpit();
                (style, stem, kind.spec().cockpit_model_scale())
            }
            Seat::Console(stem) => (layout::PanelStyle::Modern, stem, super::CONSOLE_MODEL_SCALE),
        }
    }

    /// Spawns the seat's model under `parent` at the eye point (`eye`); with `pod` its screens'
    /// camera pictures show that image. The cockpit's root, or None if the model isn't loaded.
    pub fn spawn(
        &mut self,
        commands: &mut Commands,
        parent: Entity,
        seat: Seat,
        pod: Option<Handle<Image>>,
    ) -> Option<Entity> {
        if !self.ready(seat) {
            return None;
        }
        let (style, stem, scale) = self.choice(seat);
        let file = self.models.0.get(stem)?.clone();
        let eye = self.eye(seat);
        let layout = layout::PanelLayout::for_style(style);
        let root = commands
            .spawn((
                Transform::from_translation(eye).with_scale(Vec3::splat(1.0 / scale)),
                Visibility::default(),
                Propagate(super::eye_only()),
            ))
            .id();
        commands.entity(parent).add_child(root);
        let panel = displays::panel_materials(
            commands,
            root,
            &layout,
            &mut self.materials,
            &mut self.images,
        );
        let tc = pod.map(crate::trackcam::TrackCamImage);
        let mut spawner = model::Spawner::new(
            &self.mats,
            &panel,
            &layout,
            &self.fonts,
            tc.as_ref(),
            false,
            false,
            self.settings.hires_displays(),
            self.settings.hud_scale(),
            &mut self.meshes,
            &mut self.materials,
            &mut self.hud_materials,
            &mut self.images,
        );
        spawner.tc_wide = tc.clone();
        spawner.spawn(commands, &self.assets, &file, root);
        Some(root)
    }
}

/// The scene cockpits' screens: their canvases to draw, where they are, and the camera
/// pictures behind them.
#[derive(SystemParam)]
pub struct SceneScreens<'w, 's> {
    roles: Query<'w, 's, (Entity, &'static CanvasRole, &'static GlobalTransform)>,
    canvases: Query<'w, 's, &'static mut CanvasTexture>,
    pictures: Query<'w, 's, (&'static TrackCamScreen, &'static mut Visibility)>,
}

impl SceneScreens<'_, '_> {
    /// A screen's entity and where it is (its mesh's middle).
    pub fn find(&self, s: Screen) -> Option<(Entity, &GlobalTransform)> {
        let role = s.role();
        self.roles.iter().find(|(_, r, _)| **r == role).map(|(e, _, g)| (e, g))
    }

    /// A screen's canvas, to draw.
    pub fn canvas(&mut self, s: Screen) -> Option<Mut<'_, CanvasTexture>> {
        let (e, _) = self.find(s)?;
        self.canvases.get_mut(e).ok()
    }

    /// Shows or hides the camera picture behind an MFD.
    pub fn picture(&mut self, side: MfdSide, shown: bool) {
        for (screen, mut v) in &mut self.pictures {
            if screen.0 == side {
                v.set_if_neq(if shown { Visibility::Inherited } else { Visibility::Hidden });
            }
        }
    }
}

//! Mission planner: the theater map with the enemy's radar and missile coverage, and the route
//! to fly. The targets and the landing base are fixed; the player adds, moves and removes
//! navigation points between them. The map's LAYERS panel hides and shows what it draws (the
//! radar types, their rings, the campaign's units).

use std::collections::HashMap;
use std::sync::Arc;

use stealth_i18n::t;

use bevy::gltf::Gltf;
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_egui::{EguiContexts, egui};

use stealth_mission::Mission;
use stealth_mission::world::flags as F;
use stealth_sim::SimData;
use stealth_sim::hills::Hills;
use stealth_sim::relief::Relief;

use crate::aircraft::AircraftKind;
use crate::canvas::Canvas;
use crate::layout::screen_window;
use crate::map_draw::{
    self, FRIENDLY_BLUE, Layer, MISSILE_RED, MapLayers, MapTips, MapView, MapWeather, UAV_PINK,
    drone_cross, missile_circle,
};
use crate::menu::{AMBER, BRIGHT, DIM, GREEN, MenuState, big_button, icon, section};
use crate::mfd::{MapModel, map_models, rasterise_map};
use crate::mission::{PlannedRoute, RouteKind, RoutePoint};
use crate::models::ModelLibrary;
use crate::palette::{VgaPalette, load_theater_palettes};
use crate::world::Theater;

/// Side of the rendered theater map (pixels; 32 map units, 500 m, each).
const MAP_PX: usize = 1024;
/// The whole theater (map units).
const THEATER: egui::Rect =
    egui::Rect { min: egui::Pos2::ZERO, max: egui::pos2(0x8000 as f32, 0x8000 as f32) };
/// Closer than this (map units per screen pixel), the part of the map in view is drawn at the
/// screen's resolution: the map picture's pixels blur.
const DETAIL_BELOW: f32 = 24.0;
/// Map legend colours of the scenery: the built-up area of a town, its streets, the roads.
const MAP_TOWN: u8 = 8;
/// The map legend's open land.
const MAP_LAND: u8 = 4;
const MAP_STREET: u8 = 7;
const MAP_ROAD: u8 = 0;
/// Most route points.
const MAX_POINTS: usize = stealth_campaign::tuning::ROUTE_POINTS;
/// Cruise speed for the leg times (knots).
const CRUISE_KTS: f64 = 480.0;
/// Map units per km.
const PER_KM: f64 = 64.0;

/// Flight profile the radar coverage is drawn for (doors shut, wings level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Profile {
    #[default]
    Low,
    Medium,
    High,
}

impl Profile {
    const ALL: [Profile; 3] = [Profile::Low, Profile::Medium, Profile::High];

    fn feet(self) -> u16 {
        match self {
            Profile::Low => 500,
            Profile::Medium => 5000,
            Profile::High => 20000,
        }
    }

    fn name(self) -> &'static str {
        t!(match self {
            Profile::Low => "planner-low",
            Profile::Medium => "planner-medium",
            Profile::High => "planner-high",
        })
    }
}

/// Planner state (the route itself lives in the menu state).
#[derive(Resource, Default)]
pub struct Planner {
    pub open: bool,
    /// Theater whose map is being prepared.
    theater: Option<Theater>,
    gltf: Option<Handle<Gltf>>,
    /// What the map picture was drawn from, for drawing the part in view closer up.
    source: Option<Arc<MapSource>>,
    /// The part of the map in view drawn at the screen's resolution, and the one being drawn.
    detail: Option<MapDetail>,
    drawing: Option<(DetailKey, Task<egui::ColorImage>)>,
    /// The map picture: theater, texture, and the egui context (camera) it was loaded into (a
    /// texture only exists in its own context: the menu, the debrief and the campaign's Ops
    /// screen each have theirs).
    texture: Option<(String, egui::TextureHandle, Entity)>,
    /// The map picture itself, kept for loading into the next context (rendering it again
    /// takes seconds).
    image: Option<(String, std::sync::Arc<egui::ColorImage>)>,
    sim: Option<SimData>,
    /// View: map centre (map units) and map units per screen point.
    centre: egui::Vec2,
    scale: f32,
    /// Route point being dragged, and its offset from the pointer (screen points).
    drag: Option<(usize, egui::Vec2)>,
    /// The view was fitted to this route's mission.
    fitted: Option<(String, u8, u32)>,
    profile: Profile,
    /// The theater's mountains and hills (theater id), for the vertical profile.
    terrain: Option<(String, Relief, Hills)>,
    /// The vertical profile of the route, and what it was worked out for.
    vertical: Option<(u64, VerticalProfile)>,
    /// The aircraft the route is flown in, and whether in a campaign: its radar signature.
    aircraft: (AircraftKind, bool),
    /// The route point clicked (its card over the map), and where it was drawn last.
    selected: Option<usize>,
    selected_at: Option<egui::Pos2>,
    /// Large targets for the mouse and the headset's laser (the campaign's briefing), and how
    /// much of the map's right the LAYERS button stays clear of (a panel over it).
    big: bool,
    inset: f32,
}

impl Planner {
    pub fn opened(open: bool) -> Self {
        Self { open, ..default() }
    }

    /// The next route shown is fitted to the view again (a new campaign sortie: its route
    /// key is the campaign's).
    pub fn refit(&mut self) {
        self.fitted = None;
        self.drag = None;
    }

    /// The route is flown in `kind` (`campaign`: a campaign sortie): the radar coverage and
    /// the vertical profile are worked out for its radar signature.
    pub fn fly(&mut self, kind: AircraftKind, campaign: bool) {
        self.aircraft = (kind, campaign);
    }

    /// A route point is being dragged.
    /// Zooms the view in (`f` < 1) or out around its centre.
    pub fn zoom(&mut self, f: f32) {
        self.scale = (self.scale * f).clamp(1.0, 64.0);
    }

    pub fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// The map picture of theater `id`, once prepared.
    pub fn map_texture(&self, id: &str) -> Option<&egui::TextureHandle> {
        self.texture.as_ref().filter(|(t, ..)| t == id).map(|(_, t, _)| t)
    }

    /// The map picture of theater `id` in `view` (or a note while it is being prepared) and,
    /// closer up, the part in view drawn at the screen's resolution over it: every zoomable map
    /// (the planner's, the debrief's, the campaign's war and air defence maps).
    pub fn draw_map(&mut self, painter: &egui::Painter, view: &MapView, id: &str) {
        map_draw::picture(painter, view, self.map_texture(id));
        if self.map_texture(id).is_some()
            && let Some((tex, region)) = self.detail(painter.ctx(), view)
        {
            let screen = egui::Rect::from_min_max(
                view.to_screen(region.min.x, region.min.y),
                view.to_screen(region.max.x, region.max.y),
            );
            let uv = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
            painter.image(tex, screen, uv, egui::Color32::WHITE);
        }
    }

    /// The part of the map in `view` drawn at the screen's resolution, to draw over the map
    /// picture: its texture and the map units it covers. Drawn on another thread, one drawing
    /// at a time, each for the view of the moment it starts: while the view moves (a zoom, a
    /// pan), it follows as fast as they are drawn. While far out, none.
    fn detail(
        &mut self,
        ctx: &egui::Context,
        view: &MapView,
    ) -> Option<(egui::TextureId, egui::Rect)> {
        let id = self.texture.as_ref()?.0.clone();
        if self.source.as_ref().is_none_or(|s| s.theater.id != id) {
            return None;
        }
        // Map units per screen pixel, and the theater's part in view.
        let px = view.scale / ctx.pixels_per_point();
        let seen = egui::Rect::from_min_max(
            view.to_map(view.rect.min).to_pos2(),
            view.to_map(view.rect.max).to_pos2(),
        )
        .intersect(THEATER);
        // (To a map unit: the drawn region's float rounding.)
        let fits = |k: &DetailKey| {
            k.id == id && k.region.contains_rect(seen.shrink(1.0)) && (k.px / px - 1.0).abs() < 0.1
        };
        if let Some((_, task)) = &mut self.drawing
            && let Some(image) = bevy::tasks::futures::check_ready(task)
            && let Some((key, _)) = self.drawing.take()
        {
            match &mut self.detail {
                // A drawing for a view since left, the shown one back in view: kept.
                Some(d) if d.ctx == *ctx && fits(&d.key) && !fits(&key) => {}
                // (Into the texture already there: a new one each drawing would churn.)
                Some(d) if d.ctx == *ctx => {
                    d.texture.set(image, egui::TextureOptions::LINEAR);
                    d.key = key;
                }
                _ => {
                    let texture =
                        ctx.load_texture("planner_detail", image, egui::TextureOptions::LINEAR);
                    self.detail = Some(MapDetail { key, texture, ctx: ctx.clone() });
                }
            }
        }
        let current = self.detail.as_ref().is_some_and(|d| d.ctx == *ctx && fits(&d.key));
        // A drawing under way is let finish, even for a view already left: cancelling it for
        // each new view, nothing would be drawn till the view stops. The next one starts from
        // the view of then.
        if px < DETAIL_BELOW
            && seen.is_positive()
            && !current
            && self.drawing.is_none()
            && let Some(source) = self.source.clone()
        {
            // With a margin round the view: panning a little needs no new drawing.
            let region = seen.expand2(seen.size() * 0.2).intersect(THEATER);
            let side = ctx.input(|i| i.max_texture_side) as f32;
            let scale = px.max(region.width().max(region.height()) / side);
            // (Not over the texture size limit, whatever the float rounding.)
            let side_px = |l: f32| (l / scale).ceil().min(side);
            let (w, h) = (side_px(region.width()), side_px(region.height()));
            let region = egui::Rect::from_min_size(region.min, egui::vec2(w, h) * scale);
            let (w, h) = (w as usize, h as usize);
            let task = AsyncComputeTaskPool::get().spawn(async move {
                let start = bevy::platform::time::Instant::now();
                let image = source.render(region, w, h);
                debug!("planner: map detail {w}x{h} in {:.2?}", start.elapsed());
                image
            });
            self.drawing = Some((DetailKey { id: id.clone(), region, px }, task));
        }
        // Each frame till it is drawn: shown the frame it is (and the next one started).
        if self.drawing.is_some() {
            ctx.request_repaint();
        }
        // Much finer than the screen, it would shimmer: the map picture is enough.
        self.detail
            .as_ref()
            .filter(|d| d.key.id == id && d.ctx == *ctx && px < d.key.px * 2.0)
            .map(|d| (d.texture.id(), d.key.region))
    }
}

/// What a theater's map pictures are drawn from (gathering it, the scenery's layout above all,
/// takes seconds; drawing from it, a fraction of one).
struct MapSource {
    theater: Theater,
    models: HashMap<u8, MapModel>,
    scenery: crate::scenery::MapScenery,
    /// The day palette (the MFD map's colours), and the map16 remap of its indices.
    pal: VgaPalette,
    remap: Vec<u8>,
}

/// A part of the map drawn closer up: the theater, the map units it covers, and the map units
/// per screen pixel of the view it was drawn for.
struct DetailKey {
    id: String,
    region: egui::Rect,
    px: f32,
}

/// A part of the map drawn closer up, and the egui context its texture lives in.
struct MapDetail {
    key: DetailKey,
    texture: egui::TextureHandle,
    ctx: egui::Context,
}

/// Prepares the map texture of the menu's theater while the planner is open.
#[allow(clippy::too_many_arguments)]
pub fn prepare_map(
    mut planner: ResMut<Planner>,
    state: Res<MenuState>,
    mut library: ResMut<ModelLibrary>,
    server: Res<AssetServer>,
    assets: crate::models::ModelAssets,
    mut contexts: EguiContexts,
    debrief: Option<Res<crate::debrief::Debrief>>,
    app_state: Res<State<crate::settings::AppState>>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    egui_context: Query<Entity, With<bevy_egui::PrimaryEguiContext>>,
    built: Option<Res<crate::scenery::Scenery>>,
    flight_theater: Option<Res<Theater>>,
    mut layouts: ResMut<crate::scenery::LayoutCache>,
) {
    // The planner's mission, the one flown on the debrief's flight map, or the campaign's
    // theater (its war map).
    let code = if *app_state.get() == crate::settings::AppState::Flight {
        campaign.map(|c| c.mission.theater_code.clone())
    } else if *app_state.get() == crate::settings::AppState::Debrief {
        debrief.as_ref().and_then(|d| d.mission.as_ref()).map(|m| m.theater_code.clone())
    } else if planner.open {
        match &state.mission {
            Some(Ok(m)) => Some(m.theater_code.clone()),
            _ => None,
        }
    } else {
        None
    };
    let Some(code) = code else { return };
    let id = code.to_ascii_lowercase();
    let Ok(context) = egui_context.single() else { return };
    if planner.texture.as_ref().is_some_and(|(t, _, c)| *t == id && *c == context) {
        return;
    }
    if planner.sim.is_none() {
        planner.sim = stealth_host::theater::sim_tables().map_err(|e| warn!("planner: {e:#}")).ok();
    }
    if let Some((_, image)) = planner.image.as_ref().filter(|(t, _)| *t == id) {
        let image = image.clone();
        let Ok(ctx) = contexts.ctx_mut() else { return };
        let tex = ctx.load_texture(format!("planner_{id}"), image, egui::TextureOptions::LINEAR);
        planner.texture = Some((id, tex, context));
        return;
    }
    // Once the scenery layout being made (in a flight on this theater, the flight's) is there:
    // laying it out again takes seconds.
    let in_flight = *app_state.get() == crate::settings::AppState::Flight
        && flight_theater.is_some_and(|t| t.id == id);
    if layouts.making(&id) || (in_flight && built.is_none()) {
        return;
    }
    if planner.theater.as_ref().is_none_or(|t| t.id != id) {
        planner.theater = Some(Theater::load(&id));
        planner.gltf = Some(library.load(&server, &id));
    }
    let Some(g) = planner.gltf.as_ref().and_then(|h| assets.gltfs.get(h)) else { return };
    let models = map_models(&id, g, &assets.nodes, &assets.meshes, &assets.mesh_data);
    let Some(theater) = planner.theater.take() else { return };
    let source = MapSource::new(theater, models, &assets, &library.files, &mut layouts);
    let image = Arc::new(source.render(THEATER, MAP_PX, MAP_PX));
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let tex =
        ctx.load_texture(format!("planner_{id}"), image.clone(), egui::TextureOptions::LINEAR);
    planner.texture = Some((id.clone(), tex, context));
    planner.image = Some((id, image));
    // The previous theater's part drawn closer up goes with its source.
    planner.source = Some(Arc::new(source));
    planner.detail = None;
    planner.drawing = None;
}

/// `--export-maps`: the theaters still to write, the one loading, and where to.
#[derive(Resource)]
pub struct MapExport {
    dir: std::path::PathBuf,
    next: usize,
    loading: Option<(Theater, Handle<Gltf>)>,
}

impl MapExport {
    pub fn new(dir: String) -> Self {
        Self { dir: dir.into(), next: 0, loading: None }
    }
}

/// Side of an exported map picture (pixels).
const EXPORT_PX: u32 = 512;

/// Writes each theater's map picture as `<dir>/<id>.png`, then exits.
#[allow(clippy::too_many_arguments)]
pub fn export_maps(
    mut export: ResMut<MapExport>,
    mut library: ResMut<ModelLibrary>,
    server: Res<AssetServer>,
    assets: crate::models::ModelAssets,
    mut exit: MessageWriter<AppExit>,
) {
    let theaters = stealth_host::theater::THEATERS;
    if export.loading.is_none() {
        let Some((id, _)) = theaters.get(export.next) else {
            exit.write(AppExit::Success);
            return;
        };
        export.loading = Some((Theater::load(id), library.load(&server, id)));
    }
    let Some((_, handle)) = export.loading.as_ref() else { return };
    let Some(g) = assets.gltfs.get(handle) else { return };
    let Some((theater, _)) = export.loading.take() else { return };
    let models = map_models(&theater.id, g, &assets.nodes, &assets.meshes, &assets.mesh_data);
    let path = export.dir.join(format!("{}.png", theater.id));
    let source = MapSource::new(
        theater,
        models,
        &assets,
        &library.files,
        &mut crate::scenery::LayoutCache::default(),
    );
    let image = source.render(THEATER, MAP_PX, MAP_PX);
    let rgba: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    let saved = image::RgbaImage::from_raw(MAP_PX as u32, MAP_PX as u32, rgba)
        .map(|i| {
            image::imageops::resize(&i, EXPORT_PX, EXPORT_PX, image::imageops::FilterType::Triangle)
        })
        .ok_or_else(|| "bad picture".to_string())
        .and_then(|i| i.save(&path).map_err(|e| e.to_string()));
    match saved {
        Ok(()) => info!("map picture {}", path.display()),
        Err(e) => error!("map picture {}: {e}", path.display()),
    }
    export.next += 1;
}

impl MapSource {
    fn new(
        theater: Theater,
        models: HashMap<u8, MapModel>,
        assets: &crate::models::ModelAssets,
        files: &HashMap<String, Handle<Gltf>>,
        layouts: &mut crate::scenery::LayoutCache,
    ) -> Self {
        let remaps = crate::palette::remap_tables();
        let map16 =
            remaps.iter().find(|t| t.name == "map16").map(|t| &t.entries[..]).unwrap_or(&[]);
        let remap =
            (0..=255u8).map(|c| map16.get(c as usize).map_or(c, |w| (*w & 0xFF) as u8)).collect();
        let pal: VgaPalette = load_theater_palettes(&theater.id).day;
        let scenery = crate::scenery::map_scenery(&theater, assets, files, &pal, MAP_PX, layouts);
        Self { theater, models, scenery, pal, remap }
    }

    /// The map picture of `region` (map units) on `w` x `h` square pixels: the theater top-down
    /// in the day palette, with the scenery's towns and roads.
    fn render(&self, region: egui::Rect, w: usize, h: usize) -> egui::ColorImage {
        let mut c = Canvas::new(w, h);
        // Map units per pixel.
        let px = region.width() as f64 / w as f64;
        let (x0, y0) = (region.min.x as f64, region.min.y as f64);
        let to_px = |east: f64, north: f64| {
            Vec2::new(
                ((east / 32.0 - x0) / px) as f32,
                ((0x8000 as f64 - north / 32.0 - y0) / px) as f32,
            )
        };
        // The MFD's zoom of this scale, 2^(10 - zoom) map units a pixel (the whole theater on
        // MAP_PX pixels: 5): closer, its finer tiles too.
        let zoom = (10.0 - px.log2()).floor().clamp(5.0, 9.0) as i32;
        let centre = region.center();
        let visible = (
            centre.x as f64 * 32.0,
            (0x8000 as f64 - centre.y as f64) * 32.0,
            region.width() as f64 * 16.0,
            region.height() as f64 * 16.0,
        );
        let remap = |col: u8| self.remap[col as usize];
        rasterise_map(&mut c, &self.models, &self.theater, &to_px, visible, zoom, &remap);
        let at = |p: DVec2| {
            let q = to_px(p.x / crate::world::WU, p.y / crate::world::WU);
            (q.x.floor() as i32, q.y.floor() as i32)
        };
        let (cw, ch) = (w as i32, h as i32);
        let px_m = px * 32.0 * crate::world::WU;
        for &(centre, r) in &self.scenery.towns {
            let (x, y) = at(centre);
            let rp = (r / px_m).max(1.2) as f32;
            let n = rp.ceil() as i32;
            if x + n < 0 || y + n < 0 || x - n >= cw || y - n >= ch {
                continue;
            }
            for dy in -n..=n {
                for dx in -n..=n {
                    if ((dx * dx + dy * dy) as f32) <= rp * rp {
                        c.set(x + dx, y + dy, MAP_TOWN);
                    }
                }
            }
        }
        for (lines, colour) in
            [(&self.scenery.streets, MAP_STREET), (&self.scenery.roads, MAP_ROAD)]
        {
            for pts in lines {
                for seg in pts.windows(2) {
                    let ((x0, y0), (x1, y1)) = (at(seg[0]), at(seg[1]));
                    // Off the picture (the line would still be walked pixel by pixel).
                    if x0.max(x1) < 0 || y0.max(y1) < 0 || x0.min(x1) >= cw || y0.min(y1) >= ch {
                        continue;
                    }
                    c.line(x0, y0, x1, y1, colour);
                }
            }
        }
        // Woods: the land's own colour, darker. The forest grid (MAP_PX cells a side) is
        // blended between its cells' centres: closer up, its edges don't show its cells.
        let forest = &self.scenery.forest;
        let has_forest = forest.len() == MAP_PX * MAP_PX;
        let cell = 0x8000 as f64 / MAP_PX as f64;
        let wooded = |k: usize| {
            let gx = (x0 + ((k % w) as f64 + 0.5) * px) / cell - 0.5;
            let gy = (y0 + ((k / w) as f64 + 0.5) * px) / cell - 0.5;
            let (fx, fy) = (gx - gx.floor(), gy - gy.floor());
            let at = |dx: f64, dy: f64| {
                let i = ((gx.floor() + dx) as i64).clamp(0, MAP_PX as i64 - 1) as usize;
                let j = ((gy.floor() + dy) as i64).clamp(0, MAP_PX as i64 - 1) as usize;
                forest[j * MAP_PX + i] as u8 as f64
            };
            let top = at(0.0, 0.0) * (1.0 - fx) + at(1.0, 0.0) * fx;
            let bottom = at(0.0, 1.0) * (1.0 - fx) + at(1.0, 1.0) * fx;
            top * (1.0 - fy) + bottom * fy > 0.5
        };
        let rgba: Vec<u8> =
            c.px.iter()
                .enumerate()
                .flat_map(|(k, &i)| {
                    let [r, g, b] = self.pal.colors.get(i as usize).copied().unwrap_or([0, 0, 0]);
                    let [r, g, b] = [r * 4 + r / 16, g * 4 + g / 16, b * 4 + b / 16];
                    if i == MAP_LAND && has_forest && wooded(k) {
                        [r * 3 / 5, g * 3 / 5, b * 3 / 5, 255]
                    } else {
                        [r, g, b, 255]
                    }
                })
                .collect();
        egui::ColorImage::from_rgba_unmultiplied([w, h], &rgba)
    }
}

/// A site's coverage: where it detects the player flying the profile, and its missile range
/// (map units).
struct Ring {
    x: f32,
    y: f32,
    detect: f32,
    launch: f32,
    /// Where it opens fire on the player: inside both its detection and its missile range.
    engage: f32,
    name: String,
    /// What it is (a SAM site, a radar, a ship) and where (its place, a ship's class).
    kind: &'static str,
    place: String,
    active: bool,
    /// Doppler radar (solid detection circle; pulse radars dotted).
    doppler: bool,
    /// Its radar type, and the type's (lethality, tier, flags) for [`stealth_sim::detect`].
    typ: i32,
    threat: (i32, i32, i32),
}

/// The radar signature of the aircraft flown, as the simulation works it out (its `Pilot::rcs`,
/// `low_level_ft` and `compute_emv`), cruising level with the gear up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Signature {
    /// Radar return in 32nds of the usual one.
    rcs: i32,
    /// The RAH-66: the return and the EMV fall with its speed.
    comanche: bool,
    /// Ground radars lose it in the clutter below this height (ft).
    low_level_ft: i32,
    /// Stores on pylons: seen as an open bay.
    bay_open: bool,
    lockheed: bool,
    diff: i32,
}

impl Signature {
    fn of(m: &Mission, (kind, campaign): (AircraftKind, bool)) -> Self {
        let spec = kind.spec();
        let lockheed = m.settings.plane == stealth_mission::Plane::Lockheed;
        let bay_open = !spec.bomb_bay;
        let rcs = if kind == AircraftKind::F22 {
            if campaign {
                stealth_sim::F22_RCS
            } else {
                32 * stealth_sim::F22_RCS / stealth_sim::F117_RCS
            }
        } else if !campaign || lockheed || bay_open || spec.helicopter {
            32
        } else {
            stealth_sim::F117_RCS
        };
        Self {
            rcs,
            comanche: kind == AircraftKind::RAH66,
            low_level_ft: if kind == AircraftKind::Tornado {
                stealth_sim::TORNADO_LOW_LEVEL_FT
            } else {
                stealth_sim::LOW_LEVEL_FT
            },
            bay_open,
            lockheed,
            diff: m.settings.opponents as i32,
        }
    }

    /// The EMV cruising `feet` above the ground.
    fn emv(&self, feet: u16) -> f32 {
        use stealth_sim::types::pflags;
        let agl = stealth_sim::math::view_z(feet);
        let velocity = (CRUISE_KTS * 27.0) as i32;
        let flags = pflags::GEAR_UP | if self.bay_open { pflags::BAY_OPEN } else { 0 };
        (if self.comanche {
            let base = stealth_sim::emv(agl, velocity, 0, pflags::GEAR_UP, self.diff, false);
            let scale = 15 + stealth_sim::comanche_signature(velocity);
            base * scale / 47 + stealth_sim::emv(0, 0, 0, flags, self.diff, false)
        } else {
            stealth_sim::emv(agl, velocity, 0, flags, self.diff, self.lockheed)
        }) as f32
    }

    /// What it leaves of a radar's score.
    fn return_factor(&self) -> f32 {
        let comanche = if self.comanche {
            stealth_sim::comanche_signature((CRUISE_KTS * 27.0) as i32) as f32 / 32.0
        } else {
            1.0
        };
        self.rcs as f32 / 32.0 * comanche
    }
}

/// The game's radar detection ([`stealth_sim::radar`]): a site detects the player when its
/// score plus his EMV passes 100; the score falls linearly with range from
/// `terrain * (2 * opponents + tier + 1)` (an average aspect, less for a smaller radar return)
/// to 0 at the radar's lethality.
fn threat_rings(
    m: &Mission,
    sim: &SimData,
    profile: Profile,
    sig: &Signature,
    known: &dyn Fn(usize) -> bool,
) -> Vec<Ring> {
    let w = &m.world;
    let diff = m.settings.opponents as i32;
    let lockheed = m.settings.plane == stealth_mission::Plane::Lockheed;
    let emv = sig.emv(profile.feet());
    (3..(w.ground_unit_count as usize).min(w.objects.len()))
        .filter_map(|i| {
            let o = w.objects[i];
            if o.unit_type <= 0 || o.flags & (F::FRIENDLY | F::NEUTRAL) != 0 || !known(i) {
                return None;
            }
            let t = sim.threat(o.unit_type as i32)?;
            let missile = sim.missile(t.missile.map_or(o.unit_type as i32, i32::from));
            let (x, y) = (o.x as u16 as i32, o.y as u16 as i32);
            let terrain = (w.grid[((y >> 11).clamp(0, 15) * 16 + (x >> 11).clamp(0, 15)) as usize]
                & 0xC) as f32;
            let peak = terrain * (2 * diff + t.tier as i32 + 1) as f32
                / if lockheed { 2.0 } else { 1.0 }
                * sig.return_factor();
            let detect = if peak > 0.0 {
                (t.lethality as f32 * (1.0 - (100.0 - emv) / peak)).max(0.0)
            } else {
                0.0
            };
            let launch = missile.lock_range as f32 * PER_KM as f32;
            let ship = o.flags & F::SHIP != 0;
            let place = crate::mission::clean_text(w.name(o.name as i32));
            Some(Ring {
                x: x as f32,
                y: y as f32,
                detect: detect * PER_KM as f32,
                launch,
                engage: (detect * PER_KM as f32).min(launch),
                name: t.name.clone(),
                kind: if ship {
                    t!("mt-enemy-ship")
                } else if launch > 0.0 {
                    t!("mt-sam-site")
                } else {
                    t!("mt-enemy-radar")
                },
                place: match ship {
                    true => map_draw::object_name(m, i),
                    false if place.is_empty() => String::new(),
                    false => t!("mt-near", place = place.as_str()),
                },
                active: o.flags & (F::OFF_THE_AIR | F::UNDER_REPAIR) == 0,
                doppler: t.flags & 1 != 0,
                typ: o.unit_type as i32,
                threat: (t.lethality as i32, t.tier as i32, t.flags as i32),
            })
        })
        .collect()
}

/// Km of the route inside any of the circles of `radius`.
fn exposure(route: &[RoutePoint], rings: &[Ring], radius: impl Fn(&Ring) -> f32) -> f64 {
    let mut km = 0.0;
    for w in route.windows(2) {
        let (a, b) =
            (egui::vec2(w[0].x as f32, w[0].y as f32), egui::vec2(w[1].x as f32, w[1].y as f32));
        let len = (b - a).length() as f64 / PER_KM;
        let n = len.ceil().max(1.0) as usize;
        for k in 0..n {
            let p = a + (b - a) * ((k as f32 + 0.5) / n as f32);
            if rings.iter().any(|r| r.active && (p - egui::vec2(r.x, r.y)).length() < radius(r)) {
                km += len / n as f64;
            }
        }
    }
    km
}

fn leg_time(km: f64) -> String {
    let s = (km / (CRUISE_KTS * 1.852) * 3600.0) as i32;
    format!("{}:{:02}", s / 60, s % 60)
}

/// The route from the takeoff base (the base is not one of the INS waypoints).
fn with_base(m: &Mission, route: &[RoutePoint]) -> Vec<RoutePoint> {
    let base =
        RoutePoint { x: m.takeoff.position.x, y: m.takeoff.position.y, kind: RouteKind::Nav };
    std::iter::once(base).chain(route.iter().copied()).collect()
}

fn point_label(route: &[RoutePoint], i: usize) -> String {
    match route[i].kind {
        RouteKind::Primary => "PRI".into(),
        RouteKind::Secondary => "SEC".into(),
        RouteKind::Land => "LAND".into(),
        RouteKind::Nav => {
            format!("N{}", route[..=i].iter().filter(|p| p.kind == RouteKind::Nav).count())
        }
    }
}

/// The planner window (over the menu).
pub fn planner_ui(
    ctx: &egui::Context,
    planner: &mut Planner,
    state: &mut MenuState,
    aircraft: AircraftKind,
) {
    if !planner.open {
        return;
    }
    let Some(Ok(m)) = state.mission.clone() else {
        planner.open = false;
        return;
    };
    let route = state.route.get_or_insert_with(|| PlannedRoute::of(&m));
    if !route.fits(&m) {
        *route = PlannedRoute::of(&m);
    }
    planner.fly(aircraft, false);
    (planner.big, planner.inset) = (false, 0.0);
    let mut open = true;
    let window = screen_window(ctx, "Mission planner", PLANNER_SIZE);
    let note = crate::mission::clean_text(&m.briefing.title).to_ascii_uppercase();
    crate::menu::Panel::new(t!("planner-title")).note(note).window(window, ctx, |ui| {
        let size = ui.available_size();
        route_panel(ui, planner, &m, route, &|_| true, None, None, size, true, |ui| {
            ui.add_space(12.0);
            if ui.add(big_button(t!("loadout-done")).icon(icon::DONE).primary()).clicked() {
                open = false;
            }
        });
    });
    if !open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        planner.open = false;
        planner.drag = None;
    }
}

/// The mission planner window's size, frame included.
const PLANNER_SIZE: egui::Vec2 = egui::vec2(1632.0, 1032.0);

/// Drawn over the route editor's map instead of the mission's airfields, with the legend lines
/// (and layer checkboxes) of what it draws.
pub struct Overlay<'a> {
    pub draw: &'a dyn Fn(&egui::Painter, &MapView, &MapLayers, &mut MapTips),
    pub legend: &'a dyn Fn(&mut egui::Ui, &mut MapLayers),
}
pub type MapOverlay<'a> = Option<Overlay<'a>>;

/// The route editor: the map with the known radars' coverage, the route's legs and its
/// exposure (the menu's planner, the campaign's briefing). `known`: the radar sites (world
/// object indices) to show; `overlay`: drawn over the map instead of the mission's airfields;
/// the legend's lines hide and show the map's layers and the radar types;
/// `editable`: the route can be changed (else only looked at); `buttons`: below the side column.
#[allow(clippy::too_many_arguments)]
pub fn route_panel(
    ui: &mut egui::Ui,
    planner: &mut Planner,
    m: &Mission,
    route: &mut PlannedRoute,
    known: &dyn Fn(usize) -> bool,
    overlay: MapOverlay,
    weather: Option<MapWeather>,
    size: egui::Vec2,
    editable: bool,
    buttons: impl FnOnce(&mut egui::Ui),
) {
    let rings = planner
        .sim
        .as_ref()
        .map(|s| threat_rings(m, s, planner.profile, &Signature::of(m, planner.aircraft), known))
        .unwrap_or_default();
    let off = [&[Layer::EngagementRange][..], &map_draw::WEATHER_LAYERS].concat();
    let mut layers = MapLayers::of(ui.ctx(), "route", &off);
    // The mission's weather, unless the caller's (a campaign's changes).
    let weather = weather.unwrap_or_else(|| {
        let sky = crate::sky::Sky::of(&m.theater_code.to_ascii_lowercase(), Some(m));
        MapWeather::new(m.weather, sky.freezing_level())
    });
    ui.allocate_ui(size, |ui| {
        ui.horizontal_top(|ui| {
            // Wide enough for the radar coverage's three profiles on a line; the gap between the map
            // and the column is the window's margin left of the map.
            let gap = 12.0;
            let side = (size.x * 0.45).clamp(260.0, 350.0);
            let map_size =
                egui::vec2(ui.available_width() - side - gap, ui.available_height() - 8.0);
            // The map, and the route's vertical profile under it (drawn after the map: a point
            // dragged, added or removed this frame shows in it at once).
            let profile_h = (map_size.y * 0.24).clamp(150.0, 240.0);
            let profile_gap = 8.0;
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                map_view(
                    ui,
                    planner,
                    m,
                    route,
                    &rings,
                    &mut layers,
                    overlay.as_ref(),
                    &weather,
                    egui::vec2(map_size.x, map_size.y - profile_h - profile_gap),
                    editable,
                );
                ui.add_space(profile_gap);
                vertical_view(ui, planner, m, route, &rings, egui::vec2(map_size.x, profile_h));
            });
            ui.add_space(gap - ui.spacing().item_spacing.x);
            ui.vertical(|ui| {
                ui.set_width(side);
                // The column scrolls where the window is lower than it; the buttons under it stay.
                let key = ui.id().with("route buttons height");
                let below = ui.data(|d| d.get_temp::<f32>(key)).unwrap_or(0.0);
                egui::ScrollArea::vertical()
                    .id_salt("route_side")
                    .max_height((size.y - 8.0 - below).max(100.0))
                    .show(ui, |ui| {
                        section(ui, t!("planner-route"));
                        let pts = with_base(m, &route.points);
                        let label = |i: usize| {
                            if i == 0 {
                                t!("planner-base").to_string()
                            } else {
                                point_label(&route.points, i - 1)
                            }
                        };
                        egui::Grid::new("legs").num_columns(3).spacing([12.0, 3.0]).show(
                            ui,
                            |ui| {
                                for i in 1..pts.len() {
                                    let km = (((pts[i].x - pts[i - 1].x) as f64).powi(2)
                                        + ((pts[i].y - pts[i - 1].y) as f64).powi(2))
                                    .sqrt()
                                        / PER_KM;
                                    ui.label(format!("{} > {}", label(i - 1), label(i)));
                                    ui.label(format!("{km:5.0} km"));
                                    ui.label(leg_time(km));
                                    ui.end_row();
                                }
                            },
                        );
                        let length = |r: &[RoutePoint]| {
                            PlannedRoute { mission: Default::default(), points: with_base(m, r) }
                                .length()
                                / PER_KM
                        };
                        let total = length(&route.points);
                        let default = length(&PlannedRoute::of(m).points);
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(t!(
                                "planner-total",
                                km = format!("{total:.0}"),
                                time = leg_time(total)
                            ))
                            .color(BRIGHT),
                        );
                        ui.label(
                            egui::RichText::new(t!(
                                "planner-briefed",
                                km = format!("{default:.0}"),
                                fuel = m.fuel_lbs
                            ))
                            .color(DIM),
                        );
                        section(ui, t!("planner-coverage"));
                        ui.horizontal(|ui| {
                            for p in Profile::ALL {
                                ui.radio_value(
                                    &mut planner.profile,
                                    p,
                                    egui::RichText::new(p.name()).small(),
                                );
                            }
                        });
                        let detect = exposure(&pts, &rings, |r| r.detect);
                        ui.label(
                            egui::RichText::new(t!(
                                "planner-detected",
                                km = format!("{detect:.0}")
                            ))
                            .color(if detect > 0.0 {
                                AMBER
                            } else {
                                GREEN
                            }),
                        );
                        ui.label(
                            egui::RichText::new(t!("planner-coverage.note")).small().color(DIM),
                        );
                        section(ui, t!("debrief-section-map"));
                        map_draw::symbology_switch(ui);
                        legend(ui, Legend::Pulse, t!("planner-pulse"));
                        legend(ui, Legend::Doppler, t!("planner-doppler"));
                        legend(ui, Legend::Missile, t!("planner-missile"));
                        if m.drones.is_some() {
                            legend(ui, Legend::Drone, t!("planner-uav"));
                        }
                        if editable {
                            ui.label(
                                egui::RichText::new(t!("planner-edit-hint")).small().color(DIM),
                            );
                            ui.add_space(12.0);
                            if ui.add(big_button(t!("planner-reset")).icon(icon::RESET)).clicked() {
                                *route = PlannedRoute::of(m);
                            }
                        } else {
                            ui.label(
                                egui::RichText::new(t!("planner-view-hint")).small().color(DIM),
                            );
                        }
                    });
                let now = ui.scope(buttons).response.rect.height();
                if (now - below).abs() > 0.5 {
                    ui.data_mut(|d| d.insert_temp(key, now));
                    ui.ctx().request_repaint();
                }
            });
        });
    });
    layers.keep(ui.ctx(), "route");
}

/// The route editor over a whole screen (the campaign's briefing): the map fills `rect`, the
/// vertical profile along its bottom (folded away unless `profile`), the route's legs, fuel,
/// radar coverage and exposure in a panel at the right `panel` wide, the clicked point's card
/// with DELETE and ADD AFTER over the map (the headset has no right click), and the zoom.
#[allow(clippy::too_many_arguments)]
pub fn route_screen(
    ui: &mut egui::Ui,
    planner: &mut Planner,
    m: &Mission,
    route: &mut PlannedRoute,
    known: &dyn Fn(usize) -> bool,
    overlay: MapOverlay,
    weather: MapWeather,
    rect: egui::Rect,
    panel: f32,
    profile: &mut bool,
    editable: bool,
    chrome: bool,
    take_off: Option<bool>,
) -> bool {
    use crate::campaign::shell;
    let mut pressed = false;
    planner.big = true;
    planner.inset = if chrome { panel + shell::GAP } else { 0.0 };
    let rings = planner
        .sim
        .as_ref()
        .map(|s| threat_rings(m, s, planner.profile, &Signature::of(m, planner.aircraft), known))
        .unwrap_or_default();
    let off = [&[Layer::EngagementRange][..], &map_draw::WEATHER_LAYERS].concat();
    let mut layers = MapLayers::of(ui.ctx(), "route", &off);
    let gap = shell::GAP;
    // Under a briefing step's panel: the map alone, to look at.
    if !chrome {
        planner.selected = None;
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.set_clip_rect(rect);
            map_view(
                ui,
                planner,
                m,
                route,
                &rings,
                &mut layers,
                overlay.as_ref(),
                &weather,
                rect.size(),
                false,
            );
        });
        layers.keep(ui.ctx(), "route");
        return false;
    }
    let strip = if *profile { 190.0 } else { 40.0 };
    let map_rect =
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.right(), rect.bottom() - strip));
    ui.scope_builder(egui::UiBuilder::new().max_rect(map_rect), |ui| {
        ui.set_clip_rect(map_rect);
        map_view(
            ui,
            planner,
            m,
            route,
            &rings,
            &mut layers,
            overlay.as_ref(),
            &weather,
            map_rect.size(),
            editable,
        );
    });
    // The profile strip: its fold, then the profile.
    let strip_rect = egui::Rect::from_min_max(egui::pos2(rect.left(), map_rect.bottom()), rect.max);
    ui.painter().rect_filled(strip_rect, 0.0, crate::menu::BG);
    ui.painter().hline(
        strip_rect.x_range(),
        strip_rect.top(),
        egui::Stroke::new(2.0, crate::menu::FRAME),
    );
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(strip_rect.shrink2(egui::vec2(8.0, 4.0))),
        |ui| {
            let fold = if *profile { icon::COLLAPSE } else { icon::EXPAND };
            let head = ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(20.0, 30.0), egui::Sense::hover());
                shell::paint_icon(ui.painter(), r.center(), fold, 16.0, DIM);
                // (Unfolded, the profile has its own title.)
                if !*profile {
                    ui.label(egui::RichText::new(t!("planner-vprofile")).color(DIM));
                }
            });
            let head = ui.interact(
                head.response.rect.with_max_x(strip_rect.right() - 16.0),
                ui.id().with("profile fold"),
                egui::Sense::click(),
            );
            if head.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                *profile = !*profile;
            }
            if *profile {
                let size = egui::vec2(ui.available_width(), ui.available_height());
                vertical_view(ui, planner, m, route, &rings, size);
            }
        },
    );
    // The route panel at the right.
    let panel_rect = egui::Rect::from_min_max(
        egui::pos2(map_rect.right() - panel - gap, map_rect.top() + gap),
        egui::pos2(map_rect.right() - gap, map_rect.bottom() - gap),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(panel_rect), |ui| {
        shell::overlay_frame().show(ui, |ui| {
            ui.set_width(panel_rect.width() - 28.0);
            ui.label(egui::RichText::new(t!("planner-route")).size(20.0).color(BRIGHT));
            ui.add_space(6.0);
            let pts = with_base(m, &route.points);
            let label = |i: usize| {
                if i == 0 {
                    t!("planner-base").to_string()
                } else {
                    point_label(&route.points, i - 1)
                }
            };
            egui::ScrollArea::vertical()
                .id_salt("route_legs")
                .max_height((panel_rect.height() * 0.35).max(90.0))
                .show(ui, |ui| {
                    egui::Grid::new("legs_big").num_columns(3).spacing([16.0, 6.0]).show(
                        ui,
                        |ui| {
                            for i in 1..pts.len() {
                                let km = (((pts[i].x - pts[i - 1].x) as f64).powi(2)
                                    + ((pts[i].y - pts[i - 1].y) as f64).powi(2))
                                .sqrt()
                                    / PER_KM;
                                let lit = planner.selected == Some(i - 1);
                                let ink = if lit { BRIGHT } else { GREEN };
                                ui.label(
                                    egui::RichText::new(format!("{} > {}", label(i - 1), label(i)))
                                        .color(ink),
                                );
                                ui.label(egui::RichText::new(format!("{km:4.0} km")).color(ink));
                                ui.label(egui::RichText::new(leg_time(km)).color(ink));
                                ui.end_row();
                            }
                        },
                    );
                });
            let length = |r: &[RoutePoint]| {
                PlannedRoute { mission: Default::default(), points: with_base(m, r) }.length()
                    / PER_KM
            };
            let total = length(&route.points);
            let briefed = length(&PlannedRoute::of(m).points);
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(t!(
                    "planner-total",
                    km = format!("{total:.0}"),
                    time = leg_time(total)
                ))
                .size(17.0)
                .color(BRIGHT),
            );
            ui.label(
                egui::RichText::new(t!(
                    "planner-briefed",
                    km = format!("{briefed:.0}"),
                    fuel = m.fuel_lbs
                ))
                .small()
                .color(DIM),
            );
            ui.add_space(8.0);
            section(ui, t!("planner-coverage"));
            let names: Vec<(Profile, String)> =
                Profile::ALL.iter().map(|&p| (p, p.name().to_string())).collect();
            let options: Vec<(Profile, Option<char>, &str)> =
                names.iter().map(|(p, n)| (*p, None, n.as_str())).collect();
            if let Some(p) = shell::segmented(ui, planner.profile, &options, 44.0) {
                planner.profile = p;
            }
            let detect = exposure(&pts, &rings, |r| r.detect);
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(t!("planner-detected", km = format!("{detect:.0}")))
                    .size(18.0)
                    .color(if detect > 0.0 { AMBER } else { GREEN }),
            );
            ui.label(egui::RichText::new(t!("planner-coverage.note")).small().color(DIM));
            if !editable {
                ui.label(egui::RichText::new(t!("planner-view-hint")).small().color(DIM));
            }
            // RESET at the left, TAKE OFF (the briefing's) at the bottom right.
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if editable
                    && shell::button(ui, Some(icon::RESET), t!("planner-reset"), false, 52.0, None)
                        .clicked()
                {
                    *route = PlannedRoute::of(m);
                    planner.selected = None;
                }
                if let Some(enabled) = take_off {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let button = big_button(t!("ops-take-off"))
                            .icon(icon::FLY)
                            .primary()
                            .min_size(egui::vec2(170.0, 52.0));
                        pressed = ui.add_enabled(enabled, button).clicked();
                    });
                }
            });
        });
    });
    // How to change the route, at the top left.
    let hint = egui::Rect::from_min_size(
        map_rect.min + egui::vec2(gap, gap),
        egui::vec2(map_rect.width() * 0.5, 40.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(hint), |ui| {
        shell::chip_frame().show(ui, |ui| {
            let text = if editable { t!("planner-edit-hint-big") } else { t!("planner-view-hint") };
            ui.label(egui::RichText::new(text).color(BRIGHT));
        });
    });
    // The zoom, left of the route panel at the bottom.
    let zoom = egui::Rect::from_min_max(
        egui::pos2(
            panel_rect.left() - gap - shell::SQUARE,
            map_rect.bottom() - gap - 3.0 * shell::SQUARE - 16.0,
        ),
        egui::pos2(panel_rect.left() - gap, map_rect.bottom() - gap),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(zoom), |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        if shell::square_button(ui, icon::ZOOM_IN, t!("map-zoom-in")).clicked() {
            planner.zoom(0.7);
        }
        if shell::square_button(ui, icon::ZOOM_OUT, t!("map-zoom-out")).clicked() {
            planner.zoom(1.0 / 0.7);
        }
        if shell::square_button(ui, icon::FIT, t!("map-fit")).clicked() {
            planner.refit();
        }
    });
    // The clicked point's card, above it.
    if let (Some(i), Some(at)) = (planner.selected, planner.selected_at)
        && map_rect.contains(at)
    {
        let w = 340.0;
        let left = (at.x - 40.0).clamp(map_rect.left() + gap, panel_rect.left() - gap - w);
        let r = egui::Rect::from_min_size(egui::pos2(left, at.y - 190.0), egui::vec2(w, 160.0));
        let r =
            if r.top() < map_rect.top() + gap { r.translate(egui::vec2(0.0, 220.0)) } else { r };
        ui.scope_builder(egui::UiBuilder::new().max_rect(r), |ui| {
            shell::overlay_frame().show(ui, |ui| {
                ui.set_width(w - 28.0);
                let p = route.points[i];
                let kind = match p.kind {
                    RouteKind::Nav => t!("mt-nav-point"),
                    RouteKind::Primary => t!("sim-primary-target"),
                    RouteKind::Secondary => t!("mt-secondary-target"),
                    RouteKind::Land => t!("mt-landing-base"),
                };
                let mut close = false;
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(point_label(&route.points, i)).size(18.0).color(BRIGHT),
                    );
                    ui.label(egui::RichText::new(kind.to_uppercase()).color(DIM));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        close = shell::square_button(ui, icon::CANCEL, "").clicked();
                    });
                });
                let (from, a) = match i {
                    0 => (
                        t!("planner-base").to_string(),
                        (m.takeoff.position.x, m.takeoff.position.y),
                    ),
                    _ => (
                        point_label(&route.points, i - 1),
                        (route.points[i - 1].x, route.points[i - 1].y),
                    ),
                };
                let km =
                    (((p.x - a.0) as f64).powi(2) + ((p.y - a.1) as f64).powi(2)).sqrt() / PER_KM;
                ui.label(
                    egui::RichText::new(t!(
                        "mt-leg",
                        km = format!("{km:.0}"),
                        from = from.as_str(),
                        time = leg_time(km)
                    ))
                    .small()
                    .color(DIM),
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let nav = p.kind == RouteKind::Nav;
                    let can_delete = editable && nav && route.points.len() > 2;
                    let can_add =
                        editable && i + 1 < route.points.len() && route.points.len() < MAX_POINTS;
                    let w = (ui.available_width() - 8.0) / 2.0;
                    if ui
                        .add_enabled_ui(can_delete, |ui| {
                            shell::button(
                                ui,
                                Some(icon::DELETE),
                                t!("delete"),
                                false,
                                48.0,
                                Some(w),
                            )
                        })
                        .inner
                        .clicked()
                    {
                        route.points.remove(i);
                        close = true;
                    }
                    if ui
                        .add_enabled_ui(can_add, |ui| {
                            shell::button(
                                ui,
                                Some(icon::ADD),
                                t!("planner-add-after"),
                                false,
                                48.0,
                                Some(w),
                            )
                        })
                        .inner
                        .clicked()
                    {
                        let b = route.points[i + 1];
                        let mid = RoutePoint {
                            x: (p.x + b.x) / 2,
                            y: (p.y + b.y) / 2,
                            kind: RouteKind::Nav,
                        };
                        route.points.insert(i + 1, mid);
                        planner.selected = Some(i + 1);
                    }
                });
                if close {
                    planner.selected = None;
                }
            });
        });
    }
    layers.keep(ui.ctx(), "route");
    pressed
}

#[allow(clippy::too_many_arguments)]
fn map_view(
    ui: &mut egui::Ui,
    planner: &mut Planner,
    m: &Mission,
    route: &mut PlannedRoute,
    rings: &[Ring],
    layers: &mut MapLayers,
    overlay: Option<&Overlay>,
    weather: &MapWeather,
    size: egui::Vec2,
    editable: bool,
) {
    let (resp, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = resp.rect;
    painter.rect_filled(rect, 0.0, egui::Color32::BLACK);
    // Fit the route on first showing.
    if planner.fitted.as_ref() != Some(&route.mission) || planner.scale <= 0.0 {
        let (mut lo, mut hi) = (egui::vec2(f32::MAX, f32::MAX), egui::vec2(f32::MIN, f32::MIN));
        for p in &route.points {
            lo = lo.min(egui::vec2(p.x as f32, p.y as f32));
            hi = hi.max(egui::vec2(p.x as f32, p.y as f32));
        }
        (planner.centre, planner.scale) = MapView::fit(rect, lo, hi);
        planner.fitted = Some(route.mission.clone());
    }
    let view = MapView { rect, centre: planner.centre, scale: planner.scale };
    let scale = view.scale;
    let to_screen = |x: f32, y: f32| view.to_screen(x, y);
    let to_map = |p: egui::Pos2| view.to_map(p);
    let painter = painter.with_clip_rect(rect);
    let mut tips = MapTips::default();

    planner.draw_map(&painter, &view, &m.theater_code.to_ascii_lowercase());
    map_draw::weather(&painter, &view, "route", weather, layers);
    // Threat coverage: filled radar detection circles (solid Doppler, dotted pulse), dashed
    // missile range circles; dim when off the air.
    for ring in rings.iter().filter(|r| layers.shows_type(&r.name)) {
        let c = to_screen(ring.x, ring.y);
        let alpha = if ring.active { 1.0 } else { 0.35 };
        if layers.shows(Layer::MissileRange) {
            painter.extend(missile_circle(c, ring.launch / scale, alpha));
        }
        if ring.engage > 0.0 && layers.shows(Layer::EngagementRange) {
            painter.add(map_draw::engagement_circle(c, ring.engage / scale, alpha));
        }
        let detection = if ring.doppler { Layer::DopplerRadar } else { Layer::PulseRadar };
        if ring.detect > 0.0 && layers.shows(detection) {
            painter.extend(detection_circle(c, ring.detect / scale, ring.doppler, alpha));
        }
        if layers.shows(Layer::Sams) {
            let tag = if ring.doppler { "D" } else { "P" };
            map_draw::site(&painter, &view, c, &format!("{} {tag}", ring.name), alpha, 1.0);
            ring_tip(&mut tips, c, ring, planner.profile);
        }
    }
    // (The mission's airfields' tooltips name the takeoff base themselves.)
    let airfield_tips = overlay.is_none() && layers.shows(Layer::Airfields);
    match overlay {
        Some(o) => (o.draw)(&painter, &view, layers, &mut tips),
        None => {
            map_draw::landmark_tips(&view, m, &mut tips);
            if layers.shows(Layer::Airfields) {
                map_draw::airfields(&painter, &view, m, &mut tips);
            }
        }
    }
    // Drones on their way (drone defense).
    if let Some(d) = m.drones.as_ref().filter(|_| layers.shows(Layer::Drones)) {
        let t = &m.world.objects[d.target as usize];
        for f in m.world.flights.iter().skip(d.first_unit).take(d.count) {
            let p = to_screen(f.x as u16 as f32, f.y as u16 as f32);
            drone_cross(&painter, p);
            let target = crate::mission::clean_text(&d.target_name);
            tips.add(p, t!("planner-uav"), "", UAV_PINK)
                .line(t!("mt-flying-at", target = target))
                .symbol(drone_cross);
            painter.extend(egui::Shape::dashed_line(
                &[p, to_screen(t.x as u16 as f32, t.y as u16 as f32)],
                egui::Stroke::new(0.8, UAV_PINK.gamma_multiply(0.5)),
                3.0,
                4.0,
            ));
        }
    }
    // The route, from the takeoff base.
    let pts: Vec<egui::Pos2> =
        route.points.iter().map(|p| to_screen(p.x as f32, p.y as f32)).collect();
    let base = to_screen(m.takeoff.position.x as f32, m.takeoff.position.y as f32);
    let legs: Vec<egui::Pos2> = std::iter::once(base).chain(pts.iter().copied()).collect();
    painter.add(egui::Shape::line(legs.clone(), egui::Stroke::new(2.0, BRIGHT)));
    for (i, &p) in pts.iter().enumerate() {
        let (colour, r) = match route.points[i].kind {
            RouteKind::Nav => (GREEN, 5.0),
            RouteKind::Primary => (egui::Color32::from_rgb(255, 255, 80), 7.0),
            RouteKind::Secondary => (AMBER, 7.0),
            RouteKind::Land => (egui::Color32::from_rgb(90, 160, 255), 7.0),
        };
        painter.circle(
            p,
            r,
            egui::Color32::BLACK.gamma_multiply(0.5),
            egui::Stroke::new(2.0, colour),
        );
        painter.text(
            p + egui::vec2(9.0, -9.0),
            egui::Align2::LEFT_BOTTOM,
            point_label(&route.points, i),
            egui::FontId::monospace(13.0),
            colour,
        );
        let tip = match route.points[i].kind {
            RouteKind::Nav => {
                tips.add(p, t!("mt-nav-point"), point_label(&route.points, i), colour)
            }
            RouteKind::Primary => {
                target_tip(&mut tips, p, t!("sim-primary-target"), &m.primary, colour)
            }
            RouteKind::Secondary => {
                target_tip(&mut tips, p, t!("mt-secondary-target"), &m.secondary, colour)
            }
            RouteKind::Land => tips.add(
                p,
                t!("mt-landing-base"),
                crate::mission::clean_text(&m.landing.name),
                colour,
            ),
        };
        let (from, a) = match i {
            0 => (t!("planner-base").to_string(), (m.takeoff.position.x, m.takeoff.position.y)),
            _ => {
                (point_label(&route.points, i - 1), (route.points[i - 1].x, route.points[i - 1].y))
            }
        };
        let b = &route.points[i];
        let km = (((b.x - a.0) as f64).powi(2) + ((b.y - a.1) as f64).powi(2)).sqrt() / PER_KM;
        tip.line(t!("mt-leg", km = format!("{km:.0}"), from = from.as_str(), time = leg_time(km)))
            .symbol(move |p, c| {
                p.circle(
                    c,
                    r,
                    egui::Color32::BLACK.gamma_multiply(0.5),
                    egui::Stroke::new(2.0, colour),
                );
            });
    }
    if !airfield_tips {
        let owner = m
            .world
            .objects
            .get(m.takeoff.object_index as usize)
            .map_or(crate::app6::Affiliation::Friend, |o| map_draw::affiliation_of(o.flags));
        tips.add(
            base,
            t!("mt-takeoff-base"),
            crate::mission::clean_text(&m.takeoff.name),
            FRIENDLY_BLUE,
        )
        .symbol(move |p, c| map_draw::place_symbol(p, c, owner, true, false));
    }

    // Input.
    let reach = if planner.big { 20.0 } else { 9.0 };
    let hit_point = |pos: egui::Pos2| {
        pts.iter()
            .enumerate()
            .map(|(i, &p)| (i, p.distance(pos)))
            .filter(|(_, d)| *d < reach)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    };
    // Leg i ends at route point i (leg 0 starts at the base).
    let hit_leg = |pos: egui::Pos2| {
        (1..legs.len()).map(|i| i - 1).find(|&i| {
            let (a, b) = (legs[i], legs[i + 1]);
            let ab = b - a;
            let t = ((pos - a).dot(ab) / ab.length_sq().max(1e-3)).clamp(0.0, 1.0);
            (a + ab * t).distance(pos) < if planner.big { 12.0 } else { 6.0 }
        })
    };
    if let Some(pos) = resp.hover_pos().filter(|_| editable) {
        if hit_point(pos).is_some_and(|i| route.points[i].kind == RouteKind::Nav) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        } else if hit_leg(pos).is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
    }
    // A drag starts once the pointer has moved a little: what it grabs is where the button went
    // down (where the grab cursor was shown).
    if resp.drag_started()
        && editable
        && let Some(pos) = ui.input(|i| i.pointer.press_origin()).or(resp.interact_pointer_pos())
    {
        planner.drag = hit_point(pos)
            .filter(|&i| route.points[i].kind == RouteKind::Nav)
            .map(|i| (i, pts[i] - pos));
    }
    match planner.drag {
        Some((i, offset)) => {
            if resp.dragged()
                && let Some(pos) = resp.interact_pointer_pos()
            {
                let p = to_map(pos + offset);
                route.points[i].x = p.x.clamp(0x100 as f32, 0x7F00 as f32) as i32;
                route.points[i].y = p.y.clamp(0x100 as f32, 0x7F00 as f32) as i32;
            }
        }
        // Wheel zoom around the cursor; a drag that grabbed no point pans.
        None => view.pan_zoom(ui, &resp, &mut planner.centre, &mut planner.scale),
    }
    if resp.drag_stopped() {
        planner.drag = None;
    }
    if resp.clicked()
        && let Some(pos) = resp.interact_pointer_pos()
    {
        // A point: its card; a leg: a new point there; elsewhere: the card closes.
        planner.selected = hit_point(pos);
        if planner.selected.is_none()
            && editable
            && let Some(i) = hit_leg(pos)
            && route.points.len() < MAX_POINTS
        {
            let p = to_map(pos);
            route
                .points
                .insert(i, RoutePoint { x: p.x as i32, y: p.y as i32, kind: RouteKind::Nav });
            planner.selected = Some(i);
        }
    }
    if resp.drag_started() && planner.drag.is_some() {
        planner.selected = planner.drag.map(|d| d.0);
    }
    if resp.secondary_clicked()
        && editable
        && let Some(pos) = resp.interact_pointer_pos()
        && let Some(i) = hit_point(pos).filter(|&i| route.points[i].kind == RouteKind::Nav)
        && route.points.len() > 2
    {
        route.points.remove(i);
        planner.selected = None;
    }
    planner.selected = planner.selected.filter(|&i| i < route.points.len());
    planner.selected_at =
        planner.selected.map(|i| to_screen(route.points[i].x as f32, route.points[i].y as f32));
    if let Some(at) = planner.selected_at {
        painter.circle(at, 18.0, GREEN.gamma_multiply(0.15), egui::Stroke::new(3.0, GREEN));
    }
    let panel = layers_panel(
        ui,
        rect.with_max_x(rect.right() - planner.inset),
        layers,
        planner.big,
        |ui, layers| {
            layers.legend(ui, Layer::Sams, t!("planner-layer-sams"), |p, c| {
                map_draw::site_symbol(p, c, 1.0);
            });
            layers.types(
                ui,
                t!("planner-radar-types"),
                rings.iter().map(|r| r.name.as_str()).collect(),
            );
            layer_legend(ui, layers, Legend::Pulse, t!("planner-layer-pulse"));
            layer_legend(ui, layers, Legend::Doppler, t!("planner-layer-doppler"));
            layer_legend(ui, layers, Legend::Missile, t!("planner-missile"));
            layer_legend(ui, layers, Legend::Engage, t!("pic-engagement-range"));
            if m.drones.is_some() {
                layer_legend(ui, layers, Legend::Drone, t!("planner-uav"));
            }
            match overlay {
                Some(o) => (o.legend)(ui, layers),
                None => {
                    layers.legend(ui, Layer::Airfields, t!("planner-layer-airfield"), |p, c| {
                        if map_draw::app6() {
                            let side = crate::app6::Affiliation::Friend;
                            return map_draw::app6_place(
                                p,
                                c + egui::vec2(0.0, 1.0),
                                side,
                                true,
                                0.8,
                            );
                        }
                        p.rect_stroke(
                            egui::Rect::from_center_size(c, egui::vec2(8.0, 8.0)),
                            0.0,
                            egui::Stroke::new(1.5, map_draw::FRIENDLY_BLUE),
                            egui::StrokeKind::Middle,
                        );
                    })
                }
            }
            map_draw::weather_legend(ui, layers, weather.snow);
        },
    );
    // The tooltip of what is under the pointer (not over the LAYERS panel, nor while a point is
    // dragged), with what a click does there.
    if let Some(pos) = resp.hover_pos().filter(|&p| !panel.contains(p) && planner.drag.is_none()) {
        let hint = if !editable {
            None
        } else if hit_point(pos).is_some_and(|i| route.points[i].kind == RouteKind::Nav) {
            Some(t!("mt-drag-hint"))
        } else if hit_leg(pos).is_some() {
            Some(t!("mt-click-hint"))
        } else {
            None
        };
        tips.show(&resp, hint);
    }
}

/// A radar or SAM site's tooltip: its type, place, radar and ranges for the flight `profile`.
fn ring_tip(tips: &mut MapTips, c: egui::Pos2, ring: &Ring, profile: Profile) {
    let tip = tips.add(c, ring.kind, &ring.name, MISSILE_RED);
    tip.symbol(|p, c| map_draw::site_symbol(p, c, 1.0));
    if !ring.place.is_empty() {
        tip.line(&ring.place);
    }
    tip.line(if ring.doppler { t!("mt-doppler") } else { t!("mt-pulse") });
    let km = |r: f32| r as f64 / PER_KM;
    if ring.detect > 0.0 {
        tip.line(t!(
            "mt-detects",
            km = format!("{:.0}", km(ring.detect)),
            profile = profile.name()
        ));
    } else {
        tip.line(t!("mt-no-detect", profile = profile.name()));
    }
    if ring.launch > 0.0 {
        tip.line(t!("mt-missile-range", km = format!("{:.0}", km(ring.launch))));
    }
    if !ring.active {
        tip.line(t!("mt-off-air"));
    }
}

/// A route target's tooltip: its name, the mission type and what it is.
fn target_tip<'t>(
    tips: &'t mut MapTips,
    p: egui::Pos2,
    kind: &str,
    t: &stealth_mission::Target,
    colour: egui::Color32,
) -> &'t mut map_draw::Tip {
    let named = !t.place_name.is_empty();
    let name = if named { &t.place_name } else { &t.shape_name };
    let tip = tips.add(p, kind, crate::mission::clean_text(name), colour);
    if named && !t.shape_name.is_empty() {
        tip.line(crate::mission::clean_text(&t.shape_name));
    }
    let mut what = t.type_name.clone();
    if let Some(first) = what.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    if !what.is_empty() {
        tip.line(what);
    }
    tip
}

/// The map's LAYERS button in its top-right corner, opening a panel over the map with the
/// checkboxes `lines` draws; where it is (the button and the open panel).
pub(crate) fn layers_panel(
    ui: &mut egui::Ui,
    map: egui::Rect,
    layers: &mut MapLayers,
    big: bool,
    lines: impl FnOnce(&mut egui::Ui, &mut MapLayers),
) -> egui::Rect {
    const WIDTH: f32 = 280.0;
    let gap = if big { 16.0 } else { 8.0 };
    let area = egui::Rect::from_min_max(
        egui::pos2(map.right() - WIDTH - gap, map.top() + gap),
        egui::pos2(map.right() - gap, map.bottom() - gap),
    );
    // A child that takes no room in the parent: over the map, not after it (the side column).
    let ui = &mut ui.new_child(
        egui::UiBuilder::new().max_rect(area).layout(egui::Layout::top_down(egui::Align::Max)),
    );
    let clicked = if big {
        let glyph = Some(crate::menu::icon::LAYERS);
        crate::campaign::shell::button(ui, glyph, t!("planner-layers"), layers.panel, 56.0, None)
            .clicked()
    } else {
        let text = format!("{}  {}", t!("planner-layers"), if layers.panel { "^" } else { "v" });
        ui.add(egui::Button::new(egui::RichText::new(text).small()).selected(layers.panel))
            .clicked()
    };
    if clicked {
        layers.panel = !layers.panel;
    }
    if !layers.panel {
        return ui.min_rect();
    }
    egui::Frame::new()
        .fill(egui::Color32::from_black_alpha(220))
        .stroke(egui::Stroke::new(1.0, DIM))
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.set_width(WIDTH - 14.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| lines(ui, layers));
            });
        });
    ui.min_rect()
}

/// A radar's detection circle: filled, its edge solid for a Doppler radar, dotted for a pulse
/// one.
fn detection_circle(c: egui::Pos2, r: f32, doppler: bool, alpha: f32) -> Vec<egui::Shape> {
    let stroke = egui::Stroke::new(1.5, AMBER.gamma_multiply(alpha));
    let fill = AMBER.gamma_multiply(alpha * 0.18);
    if doppler {
        return vec![
            egui::Shape::circle_filled(c, r, fill),
            egui::Shape::circle_stroke(c, r, stroke),
        ];
    }
    let n = ((r * std::f32::consts::TAU / 2.0) as usize).clamp(24, 720);
    let pts: Vec<egui::Pos2> = (0..=n)
        .map(|k| c + egui::Vec2::angled(k as f32 / n as f32 * std::f32::consts::TAU) * r)
        .collect();
    let mut shapes = vec![egui::Shape::circle_filled(c, r, fill)];
    shapes.extend(egui::Shape::dotted_line(&pts, stroke.color, 4.0, 0.9));
    shapes
}

#[derive(Clone, Copy)]
enum Legend {
    Pulse,
    Doppler,
    Missile,
    Engage,
    Drone,
}

/// A map legend line: the symbol as drawn on the map, then its meaning.
fn legend(ui: &mut egui::Ui, symbol: Legend, text: &str) {
    map_draw::legend(ui, text, |painter, c| symbol.draw(painter, c));
}

/// A legend line with its layer's checkbox, in the LAYERS panel.
fn layer_legend(ui: &mut egui::Ui, layers: &mut MapLayers, symbol: Legend, text: &str) {
    let layer = match symbol {
        Legend::Pulse => Layer::PulseRadar,
        Legend::Doppler => Layer::DopplerRadar,
        Legend::Missile => Layer::MissileRange,
        Legend::Engage => Layer::EngagementRange,
        Legend::Drone => Layer::Drones,
    };
    layers.legend(ui, layer, text, |painter, c| symbol.draw(painter, c));
}

impl Legend {
    fn draw(self, painter: &egui::Painter, c: egui::Pos2) {
        match self {
            Legend::Pulse => painter.extend(detection_circle(c, 6.0, false, 1.0)),
            Legend::Doppler => painter.extend(detection_circle(c, 6.0, true, 1.0)),
            Legend::Missile => painter.extend(missile_circle(c, 6.0, 1.0)),
            Legend::Engage => {
                painter.add(map_draw::engagement_circle(c, 6.0, 1.0));
            }
            Legend::Drone => drone_cross(painter, c),
        }
    }
}

/// Columns (along the route) and rows (heights) of the vertical profile's detection grid.
const PROFILE_COLS: usize = 192;
const PROFILE_ROWS: usize = 56;
/// The vertical profile's ceiling (altitude units, feet), unless the mountains reach higher.
const PROFILE_TOP: f32 = 30000.0;
/// The vertical profile's background: black at the top, dark blue at the bottom.
const PROFILE_SKY: egui::Color32 = egui::Color32::BLACK;
const PROFILE_DEEP: egui::Color32 = egui::Color32::from_rgb(6, 16, 64);
const PROFILE_EARTH: egui::Color32 = egui::Color32::from_rgb(40, 32, 24);
const PROFILE_RIDGE: egui::Color32 = egui::Color32::from_rgb(150, 122, 82);

/// The route's vertical profile: the ground under it and how well the known radars see each
/// height over it, by the game's own detection ([`stealth_sim::detect`]: range, aspect, the
/// cone over a site, the ground clutter, the mountains and hills in between).
struct VerticalProfile {
    /// Distance of each route point from the base (map units); the last, the route's length.
    marks: Vec<f32>,
    /// The height of the top row (altitude units).
    top: f32,
    /// The ground under each column (altitude units).
    ground: Vec<f32>,
    /// Row by row from the bottom: the best radar score plus the EMV there (over 100: detected);
    /// NaN under the ground.
    heat: Vec<f32>,
    /// The highest of them.
    max: f32,
}

/// Height (altitude units) of a point `f` of the way up the profile: on a square-root scale,
/// flying low gets the room.
fn profile_alt(f: f32, top: f32) -> f32 {
    top * f.clamp(0.0, 1.0).powi(2)
}

/// How far up the profile a height is (0 the bottom, 1 the top).
fn profile_frac(alt: f32, top: f32) -> f32 {
    (alt.max(0.0) / top).sqrt().min(1.0)
}

impl VerticalProfile {
    fn new(
        m: &Mission,
        sig: &Signature,
        pts: &[RoutePoint],
        rings: &[Ring],
        relief: &Relief,
        hills: &Hills,
    ) -> Self {
        let p: Vec<egui::Vec2> = pts.iter().map(|p| egui::vec2(p.x as f32, p.y as f32)).collect();
        let mut marks = vec![0.0f32];
        for w in p.windows(2) {
            marks.push(marks[marks.len() - 1] + (w[1] - w[0]).length());
        }
        let length = marks[marks.len() - 1].max(1.0);
        let ground_at = |x: i32, y: i32| relief.ground(x, y).max(hills.ground_alt(x, y));
        // Each column's map point and the heading of its leg.
        let cols: Vec<(i32, i32, i32)> = (0..PROFILE_COLS)
            .map(|c| {
                let d = length * c as f32 / (PROFILE_COLS - 1) as f32;
                let leg = (1..marks.len()).find(|&i| d <= marks[i]).unwrap_or(marks.len() - 1);
                let (a, b) = (p[leg.saturating_sub(1)], p[leg]);
                let span = marks[leg] - marks[leg.saturating_sub(1)];
                let at =
                    if span > 0.0 { a + (b - a) * ((d - (marks[leg] - span)) / span) } else { b };
                let dir = b - a;
                let k = 4096.0 / dir.x.abs().max(dir.y.abs()).max(1.0);
                // (Map y runs south; a bearing's y north.)
                let heading = stealth_sim::math::bearing((dir.x * k) as i32, (-dir.y * k) as i32);
                (at.x as i32, at.y as i32, heading)
            })
            .collect();
        let ground: Vec<f32> = cols.iter().map(|&(x, y, _)| ground_at(x, y) as f32).collect();
        let top = PROFILE_TOP.max(ground.iter().fold(0.0f32, |a, &g| a.max(g)) * 1.5);
        let row_alt = |r: usize| profile_alt(r as f32 / (PROFILE_ROWS - 1) as f32, top);
        let rows: Vec<usize> = (0..PROFILE_ROWS).collect();
        let mut heat = vec![f32::NAN; PROFILE_COLS * PROFILE_ROWS];
        for (c, &(x, y, heading)) in cols.iter().enumerate() {
            let g = ground[c];
            let mut best = [0.0f32; PROFILE_ROWS];
            for ring in rings.iter().filter(|r| r.active) {
                let (sx, sy) = (ring.x as i32, ring.y as i32);
                if stealth_sim::math::approx(x - sx, y - sy) >> 6 >= ring.threat.0 {
                    continue;
                }
                // The mountains and hills hide the low rows from the site's antenna: a line of
                // sight open to a height is open to every height above it.
                let site = (sx, sy, ground_at(sx, sy) + stealth_sim::MAST);
                let sees = |r: usize| {
                    let to = (x, y, row_alt(r) as i32);
                    relief.sight_open(site, to) && hills.sight_open(site, to)
                };
                let first = rows.partition_point(|&r| !sees(r));
                for (r, b) in best.iter_mut().enumerate().skip(first) {
                    let alt = row_alt(r);
                    if alt < g {
                        continue;
                    }
                    let ctx = stealth_sim::DetectCtx {
                        px: x,
                        py: y,
                        alt: alt as i32,
                        view_z: stealth_sim::math::view_z(alt.min(65535.0) as u16),
                        height: (alt - g) as i32,
                        heading,
                        velocity: (CRUISE_KTS * 27.0) as i32,
                        diff: sig.diff,
                        lockheed: sig.lockheed,
                        comanche: sig.comanche,
                        rcs: sig.rcs,
                        low_level_ft: sig.low_level_ft,
                        aspect: None,
                        grid: &m.world.grid,
                        relief: None,
                        hills: None,
                    };
                    let s = stealth_sim::detect(sx, sy, 0, ring.typ, Some(ring.threat), &ctx).0;
                    *b = b.max(s as f32);
                }
            }
            for (r, b) in best.iter().enumerate() {
                let alt = row_alt(r);
                if alt >= g {
                    let agl = (alt - g).min(65535.0) as u16;
                    heat[r * PROFILE_COLS + c] = b + sig.emv(agl);
                }
            }
        }
        let max = heat.iter().copied().filter(|h| !h.is_nan()).fold(0.0, f32::max);
        Self { marks, top, ground, heat, max }
    }
}

/// The colour of a radar score plus EMV on the profile, over its whole range up to the
/// profile's highest (`max`): green well under detection, yellow at it (100), red at the
/// highest. Semi-transparent: the background shows through.
fn heat_colour(s: f32, max: f32) -> egui::Color32 {
    if s.is_nan() {
        return egui::Color32::TRANSPARENT;
    }
    let lerp = |a: [f32; 3], b: [f32; 3], t: f32, alpha: f32| {
        let c = |i: usize| (a[i] + (b[i] - a[i]) * t) as u8;
        egui::Color32::from_rgba_unmultiplied(c(0), c(1), c(2), alpha as u8)
    };
    let (green, yellow, red) = ([40.0, 190.0, 110.0], [255.0, 215.0, 40.0], [255.0, 40.0, 30.0]);
    if s < 100.0 {
        let t = (s / 100.0).clamp(0.0, 1.0);
        lerp(green, yellow, t, 20.0 + 60.0 * t)
    } else {
        let t = ((s - 100.0) / (max - 100.0).max(1.0)).min(1.0);
        lerp(yellow, red, t, 80.0 + 50.0 * t)
    }
}

/// The route's vertical profile under the map: the ground along it, the radar detection over
/// it, the route points and the profile's flying height.
fn vertical_view(
    ui: &mut egui::Ui,
    planner: &mut Planner,
    m: &Mission,
    route: &PlannedRoute,
    rings: &[Ring],
    size: egui::Vec2,
) {
    let (resp, painter) = ui.allocate_painter(size, egui::Sense::hover());
    let rect = resp.rect;
    let painter = painter.with_clip_rect(rect);
    let mut sky = egui::Mesh::default();
    for (p, c) in [
        (rect.left_top(), PROFILE_SKY),
        (rect.right_top(), PROFILE_SKY),
        (rect.right_bottom(), PROFILE_DEEP),
        (rect.left_bottom(), PROFILE_DEEP),
    ] {
        sky.colored_vertex(p, c);
    }
    sky.add_triangle(0, 1, 2);
    sky.add_triangle(0, 2, 3);
    painter.add(sky);

    // Worked out again when the route or the radars change.
    let id = m.theater_code.to_ascii_lowercase();
    if planner.terrain.as_ref().is_none_or(|(t, ..)| *t != id) {
        let (relief, hills) =
            (stealth_host::theater::relief(&id), stealth_host::theater::hills(&id));
        planner.terrain = Some((id.clone(), relief, hills));
    }
    let pts = with_base(m, &route.points);
    let sig = Signature::of(m, planner.aircraft);
    let key = {
        use std::hash::{Hash, Hasher};
        let mut h = std::hash::DefaultHasher::new();
        id.hash(&mut h);
        sig.hash(&mut h);
        pts.iter().for_each(|p| (p.x, p.y).hash(&mut h));
        for r in rings.iter().filter(|r| r.active) {
            (r.x as i32, r.y as i32, r.typ, r.threat).hash(&mut h);
        }
        h.finish()
    };
    if planner.vertical.as_ref().is_none_or(|(k, _)| *k != key)
        && let Some((_, relief, hills)) = planner.terrain.as_ref()
    {
        planner.vertical = Some((key, VerticalProfile::new(m, &sig, &pts, rings, relief, hills)));
    }
    let Some((_, v)) = planner.vertical.as_ref() else { return };

    let plot = egui::Rect::from_min_max(
        rect.min + egui::vec2(52.0, 36.0),
        rect.max - egui::vec2(28.0, 18.0),
    );
    let length = v.marks[v.marks.len() - 1].max(1.0);
    let col_x = |c: usize| plot.left() + plot.width() * c as f32 / (PROFILE_COLS - 1) as f32;
    let alt_y = |alt: f32| plot.bottom() - plot.height() * profile_frac(alt, v.top);
    let dist_x = |d: f32| plot.left() + plot.width() * d / length;
    let font = egui::FontId::proportional(11.0);
    let grid_line = egui::Stroke::new(1.0, egui::Color32::from_white_alpha(45));
    let minor_line = egui::Stroke::new(1.0, egui::Color32::from_white_alpha(22));

    // Detection: each grid point's colour, blended between them.
    let mut heat = egui::Mesh::default();
    for r in 0..PROFILE_ROWS {
        let y = plot.bottom() - plot.height() * r as f32 / (PROFILE_ROWS - 1) as f32;
        for c in 0..PROFILE_COLS {
            heat.colored_vertex(
                egui::pos2(col_x(c), y),
                heat_colour(v.heat[r * PROFILE_COLS + c], v.max),
            );
        }
    }
    for r in 0..PROFILE_ROWS as u32 - 1 {
        for c in 0..PROFILE_COLS as u32 - 1 {
            let i = r * PROFILE_COLS as u32 + c;
            let up = i + PROFILE_COLS as u32;
            heat.add_triangle(i, i + 1, up + 1);
            heat.add_triangle(i, up + 1, up);
        }
    }
    painter.add(heat);

    // A faint grid over the detection: the labelled heights and distances, and fainter lines
    // between them.
    for ft in [1000.0, 3000.0, 4000.0, 6000.0, 7000.0, 8000.0, 9000.0, 15000.0, 25000.0] {
        if ft <= v.top {
            painter.hline(plot.x_range(), alt_y(ft), minor_line);
        }
    }
    // Heights.
    for ft in [500.0, 2000.0, 5000.0, 10000.0, 20000.0, 30000.0, 40000.0, 60000.0] {
        if ft > v.top {
            break;
        }
        let y = alt_y(ft);
        painter.hline(plot.x_range(), y, grid_line);
        let label =
            if ft < 1000.0 { format!("{ft:.0} ft") } else { format!("{}k ft", ft / 1000.0) };
        painter.text(
            egui::pos2(plot.left() - 6.0, y),
            egui::Align2::RIGHT_CENTER,
            label,
            font.clone(),
            DIM,
        );
    }
    // Distances.
    let km = length as f64 / PER_KM;
    let step = [10.0, 20.0, 50.0, 100.0, 200.0, 500.0]
        .into_iter()
        .find(|s| km / s <= 10.0)
        .unwrap_or(1000.0);
    let mut d = 0.0;
    while d <= km {
        let x = dist_x((d * PER_KM) as f32);
        painter.vline(x, plot.y_range(), grid_line);
        let half = dist_x(((d + step / 2.0) * PER_KM) as f32);
        if half < plot.right() {
            painter.vline(half, plot.y_range(), minor_line);
        }
        painter.text(
            egui::pos2(x, plot.bottom() + 3.0),
            egui::Align2::CENTER_TOP,
            format!("{d:.0} km"),
            font.clone(),
            DIM,
        );
        d += step;
    }

    // The ground, over the detection (whose rows straddle it).
    let ridge: Vec<egui::Pos2> =
        v.ground.iter().enumerate().map(|(c, &g)| egui::pos2(col_x(c), alt_y(g))).collect();
    let mut earth = egui::Mesh::default();
    for &p in &ridge {
        earth.colored_vertex(p, PROFILE_EARTH);
        earth.colored_vertex(egui::pos2(p.x, plot.bottom()), PROFILE_EARTH);
    }
    for c in 0..PROFILE_COLS as u32 - 1 {
        let i = c * 2;
        earth.add_triangle(i, i + 1, i + 3);
        earth.add_triangle(i, i + 3, i + 2);
    }
    painter.add(earth);
    painter.add(egui::Shape::line(ridge, egui::Stroke::new(1.5, PROFILE_RIDGE)));

    // The flying height of the coverage's profile over the ground.
    let feet = planner.profile.feet() as f32;
    let flown: Vec<egui::Pos2> =
        v.ground.iter().enumerate().map(|(c, &g)| egui::pos2(col_x(c), alt_y(g + feet))).collect();
    painter.extend(egui::Shape::dashed_line(&flown, egui::Stroke::new(1.5, BRIGHT), 6.0, 4.0));

    // The route points.
    for (i, &d) in v.marks.iter().enumerate().skip(1) {
        let x = dist_x(d);
        let colour = match route.points.get(i - 1).map(|p| p.kind) {
            Some(RouteKind::Primary) => egui::Color32::from_rgb(255, 255, 80),
            Some(RouteKind::Secondary) => AMBER,
            Some(RouteKind::Land) => FRIENDLY_BLUE,
            _ => GREEN,
        };
        painter.extend(egui::Shape::dashed_line(
            &[egui::pos2(x, plot.top()), egui::pos2(x, plot.bottom())],
            egui::Stroke::new(1.0, colour.gamma_multiply(0.6)),
            3.0,
            3.0,
        ));
        if i - 1 < route.points.len() {
            painter.text(
                egui::pos2(x, plot.top() - 2.0),
                egui::Align2::CENTER_BOTTOM,
                point_label(&route.points, i - 1),
                egui::FontId::monospace(11.0),
                colour,
            );
        }
    }

    painter.text(
        rect.left_top() + egui::vec2(8.0, 4.0),
        egui::Align2::LEFT_TOP,
        t!("planner-vprofile"),
        font.clone(),
        BRIGHT,
    );

    // Where the pointer is: the distance and height.
    if let Some(pos) = resp.hover_pos().filter(|p| plot.contains(*p)) {
        painter.vline(
            pos.x,
            plot.y_range(),
            egui::Stroke::new(1.0, egui::Color32::from_white_alpha(60)),
        );
        let d = (pos.x - plot.left()) / plot.width() * length;
        let alt = profile_alt((plot.bottom() - pos.y) / plot.height(), v.top);
        let text = format!("{:.0} km  {:.0} ft", d as f64 / PER_KM, alt);
        let right = pos.x < plot.right() - 120.0;
        let anchor = if right { egui::Align2::LEFT_BOTTOM } else { egui::Align2::RIGHT_BOTTOM };
        let off = if right { egui::vec2(8.0, -6.0) } else { egui::vec2(-8.0, -6.0) };
        painter.text(pos + off, anchor, text, font, BRIGHT);
    }
}

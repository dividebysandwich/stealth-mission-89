//! Single mission saves. SAVE MISSION in the in-flight menu writes the flight as it is to
//! `saves/<theater>-<time>.json` in the player's folder (`stealth_host::store`, docs/files.md):
//! the mission with the route and stores planned for it, the combat simulation, the aircraft and
//! its flight model, the waypoint and HUD modes, the time of day and the flight record. LOAD on
//! the single mission screen lists the saves; one loaded takes the menu's choices, the aircraft
//! and the realism switches it was flown with, enters the flight as the mission's own start
//! would, then puts the saved state in place of the start's, the in-flight menu open and the
//! flight paused.
//!
//! Not kept: the MFD pages, the view, the kneeboard, fires and smoke, the time acceleration, and
//! the other aircraft and missiles the flight record keeps for Tacview before the save. Career
//! sorties, campaigns and coop games are not saved here.

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy_egui::egui;
use serde::{Deserialize, Serialize};
use stealth_i18n::t;
use stealth_mission::Mission;
use stealth_sim::Sim;

use crate::aircraft::AircraftKind;
use crate::combat::Combat;
use crate::flight::{Aircraft, Player};
use crate::flight_menu::FlightMenu;
use crate::hud::{HudState, Waypoints};
use crate::loadout::PlannedLoadout;
use crate::menu::{AMBER, BRIGHT, DIM, MenuState, THEATERS, big_button, menu_window};
use crate::mission::{ActiveMission, PlannedRoute, clean_text};
use crate::palette::{Palette, TimeOfDay};
use crate::recorder::FlightRecord;
use crate::settings::{AppState, MissionSetup, Settings, SetupSet};
use crate::world::WorldPos;

pub struct SaveGamePlugin;

impl Plugin for SaveGamePlugin {
    fn build(&self, app: &mut App) {
        let mut saves = SavesWindow::default();
        // `--open-saves`: start with the list of saves open (screenshots).
        if std::env::args().any(|a| a == "--open-saves") {
            saves.show();
        }
        app.insert_resource(saves)
            .add_systems(
                OnEnter(AppState::Flight),
                restore.after(SetupSet::Spawn).after(crate::hud::reset_hud),
            )
            .add_systems(Update, save.run_if(in_state(AppState::Flight)));
    }
}

/// The store directory of the saves.
const DIR: &str = "saves";

/// The saves' format; a save of another one is listed as unreadable.
const VERSION: u32 = 1;

/// What the list of saves shows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub version: u32,
    /// The primary order ("STRIKE THE SS MISSILE AT LIBEREC").
    pub title: String,
    /// The theater's name ("Central Europe").
    pub theater: String,
    pub aircraft: AircraftKind,
    /// The mission clock when saved (s).
    pub mission_seconds: i32,
    /// When saved (s since the Unix epoch).
    pub saved_at: u64,
}

/// The realism switches of the single mission screen the flight was flown with.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Realism {
    pub no_crashes: bool,
    pub realistic_damage: bool,
    pub realistic_guidance: bool,
    pub air_defence: bool,
}

/// A saved flight.
#[derive(Serialize, Deserialize)]
pub struct SavedFlight {
    pub summary: Summary,
    /// The single mission screen's mission number (`MenuState::seed`).
    pub seed: u16,
    pub realism: Realism,
    pub mission: Mission,
    pub route: PlannedRoute,
    pub loadout: PlannedLoadout,
    pub sim: Sim,
    pub aircraft: Aircraft,
    pub position: DVec3,
    /// The waypoint flown to and the one the mission logic chose last.
    pub waypoint: (usize, usize),
    pub hud: HudState,
    /// `TimeOfDay::saved`.
    pub time_of_day: (u32, f32),
    pub record: FlightRecord,
}

/// The start of a save's file, read for the list.
#[derive(Deserialize)]
struct Head {
    summary: Summary,
}

/// A save loaded on the single mission screen, put in place when the flight starts.
#[derive(Resource)]
pub struct RestoreFlight(Option<Box<SavedFlight>>);

fn key(name: &str) -> String {
    let safe: String =
        name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
    format!("{DIR}/{safe}.json")
}

fn now() -> u64 {
    web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// The saves, the latest first: (name, its summary; None if this game can't read it).
fn list() -> Vec<(String, Option<Summary>)> {
    let mut found: Vec<(f64, String, Option<Summary>)> = stealth_host::store::list(DIR)
        .into_iter()
        .filter_map(|(file, modified)| {
            let name = file.strip_suffix(".json")?.to_string();
            let summary = stealth_host::store::read(&key(&name))
                .ok()
                .flatten()
                .and_then(|t| serde_json::from_str::<Head>(&t).ok())
                .map(|h| h.summary)
                .filter(|s| s.version == VERSION);
            Some((modified, name, summary))
        })
        .collect();
    found.sort_by(|a, b| b.0.total_cmp(&a.0));
    found.into_iter().map(|(_, n, s)| (n, s)).collect()
}

fn read(name: &str) -> anyhow::Result<SavedFlight> {
    let key = key(name);
    let text =
        stealth_host::store::read(&key)?.ok_or_else(|| anyhow::anyhow!("the save is gone"))?;
    let save: SavedFlight = serde_json::from_str(&text)?;
    anyhow::ensure!(save.summary.version == VERSION, "a save of another version of the game");
    Ok(save)
}

/// "2026-10-25 14:03 UTC".
fn saved_at(secs: u64) -> String {
    let (y, m, d) = crate::tacview::civil_from_days((secs / 86_400) as i64);
    let s = secs % 86_400;
    format!("{y}-{m:02}-{d:02} {:02}:{:02} UTC", s / 3600, s % 3600 / 60)
}

/// Every frame in flight: whether SAVE MISSION can save now (a single mission under way: not a
/// career sortie, a campaign or a coop game, the pilot still in the aircraft), and the save when
/// it was chosen.
#[allow(clippy::too_many_arguments)]
fn save(
    mut menu: ResMut<FlightMenu>,
    combat: Option<Res<Combat>>,
    player: Query<(&Aircraft, &WorldPos), With<Player>>,
    mission: Option<Res<ActiveMission>>,
    flown: (Option<Res<PlannedRoute>>, Option<Res<PlannedLoadout>>),
    cockpit: (Res<Waypoints>, Res<HudState>, Res<TimeOfDay>),
    settings: Res<Settings>,
    state: Res<MenuState>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    link: Option<Res<crate::net::NetLink>>,
    career: Option<Res<crate::career::CareerFlight>>,
) {
    let single = campaign.is_none() && link.is_none() && career.is_none();
    let (Some(c), Some(m), Ok((a, pos))) = (combat, mission, player.single()) else {
        if menu.can_save {
            menu.can_save = false;
        }
        return;
    };
    let flying = c.end.is_none()
        && c.sim.mission_end().is_none()
        && c.sim.chute().is_none()
        && a.ended.is_none();
    let can_save = single && flying && !menu.exiting;
    if menu.can_save != can_save {
        menu.can_save = can_save;
    }
    if !menu.save_request {
        return;
    }
    menu.save_request = false;
    if !can_save {
        return;
    }
    let m = &m.0;
    let (wps, hud, tod) = cockpit;
    let (route, loadout) = flown;
    let saved = SavedFlight {
        summary: Summary {
            version: VERSION,
            title: clean_text(&m.orders.primary),
            theater: THEATERS
                .get(m.settings.theater)
                .map_or(m.theater_name.as_str(), |t| t.1)
                .to_string(),
            aircraft: settings.aircraft,
            mission_seconds: c.sim.mission_tick(),
            saved_at: now(),
        },
        seed: state.seed,
        realism: Realism {
            no_crashes: settings.no_crashes,
            realistic_damage: settings.realistic_damage,
            realistic_guidance: settings.realistic_guidance,
            air_defence: settings.air_defence,
        },
        route: route.filter(|r| r.fits(m)).map_or_else(|| PlannedRoute::of(m), |r| r.clone()),
        loadout: loadout
            .filter(|l| l.fits(m, settings.aircraft))
            .map_or_else(|| PlannedLoadout::of(m, settings.aircraft), |l| l.clone()),
        mission: m.clone(),
        sim: c.sim.clone(),
        aircraft: a.clone(),
        position: pos.0,
        waypoint: (wps.current, wps.mission_current),
        hud: hud.clone(),
        time_of_day: tod.saved(),
        record: c.record.clone(),
    };
    let name = format!("{}-{}", m.theater_code.to_ascii_lowercase(), saved.summary.saved_at);
    let written = serde_json::to_string(&saved).map_err(anyhow::Error::from).and_then(|text| {
        stealth_host::store::write(&key(&name), &text)?;
        Ok(text.len())
    });
    menu.save_note = Some(match written {
        Ok(bytes) => {
            info!("mission saved: {name} ({} kB)", bytes / 1024);
            stealth_i18n::tf!("fm-saved")
        }
        Err(e) => {
            warn!("saving the mission: {e:#}");
            stealth_i18n::tf!("fm-not-saved")
        }
    });
}

/// OnEnter(Flight), once the flight is set up from the save's mission: the saved state in place
/// of the start's, the menu open.
#[allow(clippy::too_many_arguments)]
fn restore(
    mut commands: Commands,
    pending: Option<ResMut<RestoreFlight>>,
    combat: Option<ResMut<Combat>>,
    mut player: Query<(&mut Aircraft, &mut WorldPos), With<Player>>,
    mut wps: ResMut<Waypoints>,
    mut hud: ResMut<HudState>,
    mut tod: ResMut<TimeOfDay>,
    palette: Option<ResMut<Palette>>,
    mut menu: ResMut<FlightMenu>,
) {
    let Some(save) = pending.and_then(|mut p| p.0.take()) else { return };
    commands.remove_resource::<RestoreFlight>();
    let save = *save;
    let Some(mut combat) = combat else {
        warn!("no combat simulation: the save is not restored");
        return;
    };
    combat.restore(save.sim, save.record);
    if let Ok((mut a, mut pos)) = player.single_mut() {
        *a = save.aircraft;
        pos.0 = save.position;
    }
    let last = wps.points.len().saturating_sub(1);
    (wps.current, wps.mission_current) = (save.waypoint.0.min(last), save.waypoint.1.min(last));
    *hud = save.hud;
    if let Some(mut palette) = palette {
        tod.restore(&mut palette, save.time_of_day);
    }
    menu.open_on_start = true;
}

/// Flies a save on: the single mission screen takes its mission and choices, the aircraft and the
/// realism switches it was flown with; the rest is put in place when the flight starts.
pub fn fly(
    commands: &mut Commands,
    save: SavedFlight,
    state: &mut MenuState,
    settings: &mut Settings,
    setup: &mut MissionSetup,
    start_dir: &[i16],
) {
    let m = &save.mission;
    state.show_saved(m, save.seed, save.route.clone(), save.loadout.clone());
    settings.aircraft = save.summary.aircraft;
    let r = save.realism;
    settings.no_crashes = r.no_crashes;
    settings.realistic_damage = r.realistic_damage;
    settings.realistic_guidance = r.realistic_guidance;
    settings.air_defence = r.air_defence;
    *setup = crate::mission::setup_from_mission(m, start_dir);
    commands.insert_resource(ActiveMission(m.clone()));
    commands.insert_resource(save.route.clone());
    commands.insert_resource(save.loadout.clone());
    commands.remove_resource::<crate::career::CareerFlight>();
    commands.insert_resource(RestoreFlight(Some(Box::new(save))));
}

/// `--load <save>`: straight into a save's flight (its name: the file name without `.json`).
pub fn from_args(app: &mut App) {
    let Some(name) = std::env::args().skip_while(|a| a != "--load").nth(1) else { return };
    app.add_systems(
        Startup,
        move |mut commands: Commands,
              mut state: ResMut<MenuState>,
              mut settings: ResMut<Settings>,
              mut setup: ResMut<MissionSetup>,
              tables: Res<crate::flight::SimTables>,
              mut next: ResMut<NextState<AppState>>| match read(&name) {
            Ok(save) => {
                fly(&mut commands, save, &mut state, &mut settings, &mut setup, &tables.start_dir);
                next.set(AppState::Flight);
            }
            Err(e) => warn!("--load {name}: {e:#}"),
        },
    );
}

/// The single mission screen's list of saves (LOAD).
#[derive(Resource, Default)]
pub struct SavesWindow {
    pub open: bool,
    /// Read when the window opens: (name, summary; None if unreadable).
    saves: Vec<(String, Option<Summary>)>,
    error: Option<String>,
    /// The save whose DELETE was clicked: (name, label), awaiting confirmation.
    confirm_delete: Option<(String, String)>,
}

impl SavesWindow {
    pub fn show(&mut self) {
        self.open = true;
        self.saves = list();
        self.error = None;
        self.confirm_delete = None;
    }
}

/// The list of saves: LOAD returns the save to fly, DELETE asks first.
pub fn saves_ui(ctx: &egui::Context, w: &mut SavesWindow) -> Option<SavedFlight> {
    if !w.open {
        return None;
    }
    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    let mut close = escape && w.confirm_delete.is_none();
    let mut load = None;
    let width = crate::menu::wide_width(ctx).min(900.0);
    menu_window(ctx, "Saved missions", width).show(ctx, |ui| {
        ui.label(egui::RichText::new(t!("saves-title")).heading().color(BRIGHT));
        ui.label(egui::RichText::new(t!("saves-subtitle")).color(DIM));
        ui.separator();
        if w.saves.is_empty() {
            ui.label(egui::RichText::new(t!("saves-none")).color(DIM));
        }
        egui::ScrollArea::vertical()
            .id_salt("mission_saves")
            .max_height(crate::campaign::SAVES_HEIGHT * 1.5)
            .show(ui, |ui| {
                for (name, summary) in &w.saves {
                    ui.horizontal(|ui| {
                        let label = match summary {
                            Some(s) => {
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&s.title).color(BRIGHT));
                                    let small = |ui: &mut egui::Ui, text: String| {
                                        ui.label(egui::RichText::new(text).small().color(DIM));
                                    };
                                    let time = format!(
                                        "{}:{:02}",
                                        s.mission_seconds / 60,
                                        s.mission_seconds % 60
                                    );
                                    small(
                                        ui,
                                        t!(
                                            "saves-line",
                                            theater = s.theater.as_str(),
                                            aircraft = s.aircraft.name(),
                                            time = time
                                        ),
                                    );
                                    small(ui, t!("saves-saved-at", date = saved_at(s.saved_at)));
                                });
                                format!("{}, {}", s.theater, s.title)
                            }
                            None => {
                                ui.label(egui::RichText::new(t!("saves-unreadable")).color(AMBER));
                                name.clone()
                            }
                        };
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(big_button(t!("delete")))
                                .on_hover_text(t!("saves-delete.facts"))
                                .clicked()
                            {
                                w.confirm_delete = Some((name.clone(), label));
                            }
                            if summary.is_some() && ui.add(big_button(t!("menu-load"))).clicked() {
                                load = Some(name.clone());
                            }
                        });
                    });
                    ui.separator();
                }
            });
        if let Some(e) = &w.error {
            ui.colored_label(AMBER, e);
        }
        ui.add_space(6.0);
        if ui.add(big_button(t!("back"))).on_hover_text(t!("saves-back.facts")).clicked() {
            close = true;
        }
    });
    if let Some((name, label)) = w.confirm_delete.clone() {
        let answer = crate::campaign::delete_popup(ctx, "save", &label, escape);
        if answer == Some(true) {
            if let Err(e) = stealth_host::store::remove(&key(&name)) {
                w.error = Some(t!("cm-save-undeletable", error = format!("{e:#}")));
            }
            w.saves = list();
        }
        if answer.is_some() {
            w.confirm_delete = None;
        }
    }
    if close {
        w.open = false;
    }
    let save = match read(&load?) {
        Ok(s) => s,
        Err(e) => {
            w.error = Some(t!("cm-save-unreadable", error = format!("{e:#}")));
            return None;
        }
    };
    w.open = false;
    Some(save)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved() -> SavedFlight {
        let data = crate::mission::MissionData::load().expect("mission data");
        let s = crate::mission::settings(
            3,
            1,
            stealth_mission::MissionKind::Strike,
            1,
            false,
            None,
            None,
        );
        let m = crate::mission::generate(&data, &s, 7).unwrap();
        let tables = stealth_host::theater::sim_tables().expect("sim tables");
        let sim = Sim::with_options(&tables, &m, default());
        let mut aircraft = Aircraft::new_on_runway(9000.0);
        aircraft.orientation = Aircraft::from_euler(1.0, 0.1, -0.3);
        aircraft.velocity = 450.0 * 27.0;
        aircraft.gear_down = false;
        let mut record = FlightRecord::default();
        for _ in 0..4 {
            record.sample(&sim);
        }
        SavedFlight {
            summary: Summary {
                version: VERSION,
                title: clean_text(&m.orders.primary),
                theater: THEATERS[3].1.into(),
                aircraft: AircraftKind::F16,
                mission_seconds: 754,
                saved_at: 1_792_937_000,
            },
            seed: 7,
            realism: Realism {
                no_crashes: false,
                realistic_damage: true,
                realistic_guidance: true,
                air_defence: true,
            },
            route: PlannedRoute::of(&m),
            loadout: PlannedLoadout::of(&m, AircraftKind::F16),
            mission: m,
            sim,
            aircraft,
            position: DVec3::new(1234.5, 600.25, -98765.125),
            waypoint: (2, 1),
            hud: HudState { declutter: 2, ccrp: true, ..default() },
            time_of_day: (300, 0.5),
            record,
        }
    }

    /// A save reads back as written (the list reads its summary alone), without the record's
    /// contacts.
    #[test]
    fn a_save_reads_back() {
        let save = saved();
        assert!(!save.record.contacts.is_empty());
        let text = serde_json::to_string(&save).unwrap();
        let back: SavedFlight = serde_json::from_str(&text).unwrap();
        assert!(back.record.contacts.is_empty());
        assert_eq!(back.record.samples.len(), 4);
        assert_eq!(serde_json::to_string(&back).unwrap(), text);
        assert_eq!(back.mission, save.mission);
        assert_eq!(back.position, save.position);
        assert_eq!(back.aircraft.orientation, save.aircraft.orientation);
        let head: Head = serde_json::from_str(&text).unwrap();
        assert_eq!(head.summary.title, save.summary.title);
        assert_eq!(head.summary.aircraft, AircraftKind::F16);
    }

    #[test]
    fn saved_at_reads_as_a_date() {
        assert_eq!(saved_at(0), "1970-01-01 00:00 UTC");
        assert_eq!(saved_at(1_792_937_000), "2026-10-25 14:03 UTC");
    }
}

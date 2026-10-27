//! The menu's campaign window: choose the options and start a dynamic campaign, or continue a
//! saved one.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use stealth_campaign::CampaignSetup;

use super::ActiveCampaign;
use crate::menu::{AMBER, BRIGHT, DIM, MenuState, THEATERS, big_button, dropdown, group, section};
use crate::mission::MissionData;
use crate::settings::{AppState, MissionSetup};
use stealth_i18n::{t, t_lines};

#[derive(Resource)]
pub struct CampaignWindow {
    pub open: bool,
    /// Saved campaigns (name, theater, hours played), read when the window opens.
    saves: Vec<(String, String, f64)>,
    /// Sky speed: game time per real time.
    sky_ratio: f32,
    start_hour: f32,
    /// No weather: the classic clear sky.
    clear_sky: bool,
    error: Option<String>,
    /// The save whose DELETE was clicked: (name, label), awaiting confirmation.
    confirm_delete: Option<(String, String)>,
}

impl Default for CampaignWindow {
    fn default() -> Self {
        Self {
            open: false,
            saves: Vec::new(),
            sky_ratio: 24.0,
            start_hour: 8.0,
            clear_sky: false,
            error: None,
            confirm_delete: None,
        }
    }
}

impl CampaignWindow {
    pub fn opened(open: bool) -> Self {
        let mut w = Self::default();
        if open {
            w.show();
        }
        w
    }

    pub fn show(&mut self) {
        self.open = true;
        self.saves = stealth_host::save::list_saves();
        self.error = None;
        self.confirm_delete = None;
    }
}

/// How tall the list of saved campaigns grows before it scrolls.
pub(crate) const SAVES_HEIGHT: f32 = 240.0;

/// The "delete this ...?" popup over the menu (`what`: "campaign", "save"): Some(true) to
/// delete, Some(false) to keep it (CANCEL, or `cancel` such as Escape), None while it waits.
pub(crate) fn delete_popup(
    ctx: &egui::Context,
    what: &str,
    label: &str,
    cancel: bool,
) -> Option<bool> {
    let mut answer = cancel.then_some(false);
    egui::Window::new(format!("Delete the {what}"))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .order(egui::Order::Foreground)
        .frame(egui::Frame::window(&ctx.global_style()).inner_margin(18.0))
        .show(ctx, |ui| {
            let question = t!(&format!("delete-{what}"));
            ui.label(egui::RichText::new(question).heading().color(BRIGHT));
            ui.label(egui::RichText::new(label).color(BRIGHT));
            ui.label(egui::RichText::new(t!("delete-permanent")).color(DIM));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.add(big_button(t!("delete")).fill(super::ops::EXIT_RED)).clicked() {
                    answer = Some(true);
                }
                if ui.add(big_button(t!("cancel"))).clicked() {
                    answer = Some(false);
                }
            });
        });
    answer
}

#[allow(clippy::too_many_arguments)]
pub fn campaign_window(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut w: ResMut<CampaignWindow>,
    mut state: ResMut<MenuState>,
    data: Option<Res<MissionData>>,
    tables: Res<crate::flight::SimTables>,
    mut setup: ResMut<MissionSetup>,
    mut next: ResMut<NextState<AppState>>,
    mut settings: ResMut<crate::settings::Settings>,
    vr: Res<crate::menu::VrAvailable>,
    mut exit: MessageWriter<AppExit>,
    mut options: ResMut<crate::options::OptionsWindow>,
) -> Result {
    // Hidden while OPTIONS is open over it.
    if !w.open || options.open {
        return Ok(());
    }
    let Some(data) = data else { return Ok(()) };
    let ctx = contexts.ctx_mut()?;
    let code = THEATERS[state.theater.min(THEATERS.len() - 1)].0;
    let theater = crate::i18n::theater_name(state.theater.min(THEATERS.len() - 1));
    let mut start: Option<ActiveCampaign> = None;
    let mut vr_name: Option<String> = None;
    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    // Escape cancels the delete confirmation first, then closes the window.
    let mut close = escape && w.confirm_delete.is_none();
    let width = crate::menu::wide_width(ctx);
    crate::menu::menu_window(ctx, "Dynamic campaign", width).show(ctx, |ui| {
        ui.label(egui::RichText::new(t!("menu-campaign")).heading().color(BRIGHT));
        ui.separator();
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(210.0);
                group(ui, t!("cm-group-campaign"), |ui| {
                    crate::menu::theater_options(ui, &mut state.theater);
                    crate::menu::situation_options(ui, &mut state.roe);
                    crate::menu::opponent_options(ui, &mut state.opponents);
                });
            });
            ui.vertical(|ui| {
                ui.set_width(210.0);
                group(ui, t!("group-conditions"), |ui| {
                    section(ui, t!("section-weather"));
                    dropdown(
                        ui,
                        "campaign_weather",
                        &mut w.clear_sky,
                        [
                            (false, t!("cm-weather-dynamic"), t_lines!("cm-weather-dynamic.facts")),
                            (true, t!("cm-weather-clear"), t_lines!("cm-weather-clear.facts")),
                        ],
                    );
                    section(ui, t!("section-start-time"));
                    ui.add(
                        egui::Slider::new(&mut w.start_hour, 0.0..=23.0)
                            .step_by(1.0)
                            .custom_formatter(|v, _| format!("{v:02.0}:00")),
                    );
                    section(ui, t!("cm-section-day-night"));
                    dropdown(
                        ui,
                        "campaign_sky_ratio",
                        &mut w.sky_ratio,
                        [
                            (1.0, t!("cm-day-real"), t_lines!("cm-day-real.facts")),
                            (12.0, t!("cm-day-2h"), t_lines!("cm-day-2h.facts")),
                            (24.0, t!("cm-day-1h"), t_lines!("cm-day-1h.facts")),
                            (48.0, t!("cm-day-30min"), t_lines!("cm-day-30min.facts")),
                        ],
                    );
                });
                ui.label(egui::RichText::new(t!("cm-conditions-note")).small().color(DIM));
            });
            ui.vertical(|ui| {
                ui.set_width(220.0);
                crate::menu::simulation_options(ui, &mut settings, Some(t!("cm-rules-note")));
            });
            ui.vertical(|ui| {
                for line in t_lines!("cm-about") {
                    ui.label(egui::RichText::new(*line).color(DIM));
                }
                ui.add_space(6.0);
                section(ui, t!("cm-section-new"));
                ui.label(t!(
                    "cm-new-line",
                    theater = theater,
                    situation = crate::i18n::roe_name(state.roe),
                    opponents = crate::i18n::opponents_name(state.opponents).to_lowercase()
                ));
                if ui.add(big_button(t!("cm-start"))).clicked() {
                    let secs = web_time::SystemTime::now()
                        .duration_since(web_time::UNIX_EPOCH)
                        .map_or(0, |d| d.as_secs());
                    let setup = CampaignSetup {
                        theater: state.theater,
                        theater_code: code.to_string(),
                        roe: state.roe,
                        opponents: state.opponents,
                        seed: rand::random(),
                        sky_ratio: w.sky_ratio,
                        start_hour: w.start_hour,
                        dynamic_weather: !w.clear_sky,
                    };
                    let rules = super::rules_of(
                        &settings,
                        state.realistic(crate::aircraft::AircraftKind::F117),
                    );
                    match ActiveCampaign::new(format!("{code}-{secs}"), setup, &data, rules) {
                        Ok(c) => start = Some(c),
                        Err(e) => w.error = Some(format!("{e:#}")),
                    }
                }
                ui.add_space(6.0);
                section(ui, t!("cm-section-continue"));
                if w.saves.is_empty() {
                    ui.label(egui::RichText::new(t!("cm-no-saves")).color(DIM));
                }
                egui::ScrollArea::vertical()
                    .id_salt("campaign_saves")
                    .max_height(SAVES_HEIGHT)
                    .show(ui, |ui| {
                        for (name, code, hours) in w.saves.clone() {
                            let theater = crate::i18n::theater_name_of(&code);
                            let played = t!("cm-played", hours = format!("{hours:.1}"));
                            ui.horizontal(|ui| {
                                ui.label(format!("{theater:<24} {played:>12}"));
                                if ui.add(big_button(t!("continue"))).clicked() {
                                    let rules = super::rules_of(
                                        &settings,
                                        state.realistic(crate::aircraft::AircraftKind::F117),
                                    );
                                    match stealth_host::save::load(&name) {
                                        Ok(Some(st)) => match ActiveCampaign::resume(
                                            name.clone(),
                                            st,
                                            &data,
                                            rules,
                                        ) {
                                            Ok(c) => start = Some(c),
                                            Err(e) => w.error = Some(format!("{e:#}")),
                                        },
                                        Ok(None) => w.error = Some(t!("cm-save-gone").into()),
                                        Err(e) => {
                                            w.error = Some(t!(
                                                "cm-save-unreadable",
                                                error = format!("{e:#}")
                                            ))
                                        }
                                    }
                                }
                                if !settings.vr
                                    && vr.0
                                    && ui.add(big_button(t!("cm-in-vr"))).clicked()
                                {
                                    vr_name = Some(name.clone());
                                }
                                if ui
                                    .add(big_button(t!("delete")))
                                    .on_hover_text(t!("cm-delete.facts"))
                                    .clicked()
                                {
                                    w.confirm_delete =
                                        Some((name.clone(), format!("{theater}, {played}")));
                                }
                            });
                        }
                    });
                if let Some(e) = &w.error {
                    ui.colored_label(AMBER, e);
                }
            });
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui.add(big_button(t!("back"))).on_hover_text(t!("menu-back-main")).clicked() {
                close = true;
            }
            if ui
                .add(big_button(t!("menu-options")))
                .on_hover_text(t!("menu-options.short"))
                .clicked()
            {
                options.open = true;
            }
        });
    });
    if let Some((name, label)) = w.confirm_delete.clone() {
        let answer = delete_popup(ctx, "campaign", &label, escape);
        if answer == Some(true) {
            if let Err(e) = stealth_host::save::delete(&name) {
                w.error = Some(t!("cm-save-undeletable", error = format!("{e:#}")));
            }
            w.saves = stealth_host::save::list_saves();
        }
        if answer.is_some() {
            w.confirm_delete = None;
        }
    }
    if let Some(c) = start {
        c.save();
        super::start(&mut commands, c, &mut setup, &tables.start_dir);
        w.open = false;
        next.set(AppState::Flight);
    }
    if let Some(name) = vr_name {
        match crate::menu::relaunch_campaign_in_vr(&name, &settings) {
            Ok(()) => {
                exit.write(AppExit::Success);
            }
            Err(e) => w.error = Some(t!("menu-vr-failed", error = e.to_string())),
        }
    }
    if close {
        w.open = false;
    }
    Ok(())
}

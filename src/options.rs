//! The main menu's OPTIONS window: the player's own settings, saved in the preferences with the
//! menu's other choices (`prefs.rs`), on four tabs:
//!
//! - GAME: the language, cockpit, HUD, avionics and flight model style, auto trim, the cockpit
//!   tooltips, the player labels, the VR cockpit size and Tacview.
//! - GRAPHICS: shaders, anti-aliasing, shadows, point lights, time of day.
//! - SOUND: the master, engine, effects, warning and radar warning volumes, which radar warning
//!   tones sound, the engine low-pass inside the aircraft.
//! - CONTROLS: the key and joystick bindings (the CONTROLS dialog, `input_dialog.rs`) and head
//!   tracking.
//!
//! The realism switches and crash protection belong to each game: they stay in the single
//! mission, career and campaign setups. The in-flight menu has the same pages, but for the
//! flight model and Tacview, which take effect at the next flight (`flight_menu.rs`).

use bevy::prelude::*;
use bevy_egui::egui;
use stealth_i18n::{Language, list_name, t, t_lines};

use crate::menu::{
    AMBER, BRIGHT, DIM, GREEN, big_button, facts, menu_window, section, slide_switch,
};
use crate::settings::{PLAYER_LABEL_SIZE_RANGE, Settings, VR_COCKPIT_SCALE_RANGE};

#[derive(Resource, Default)]
pub struct OptionsWindow {
    pub open: bool,
    tab: Tab,
    /// The largest tab's size: every tab gets it, so the window keeps its size across tabs.
    body: egui::Vec2,
}

impl OptionsWindow {
    /// `--open-options <tab>`: open on that tab (screenshots).
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let Some(i) = args.iter().position(|a| a == "--open-options") else {
            return Self::default();
        };
        let name = args.get(i + 1).map_or("", |s| s.as_str()).to_ascii_uppercase();
        let tab = Tab::ALL.iter().find(|(_, n, _)| *n == name).map_or(Tab::Game, |t| t.0);
        Self { open: true, tab, ..default() }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Game,
    Graphics,
    Sound,
    Controls,
}

impl Tab {
    /// The tab, its `--open-options` name and its title's message.
    const ALL: [(Tab, &'static str, &'static str); 4] = [
        (Tab::Game, "GAME", "options-tab-game"),
        (Tab::Graphics, "GRAPHICS", "options-tab-graphics"),
        (Tab::Sound, "SOUND", "options-tab-sound"),
        (Tab::Controls, "CONTROLS", "options-tab-controls"),
    ];
}

type SettingsMut<'a> = ResMut<'a, Settings>;

/// The window (while open and the bindings dialog is not).
pub fn options_ui(
    ctx: &egui::Context,
    w: &mut OptionsWindow,
    settings: &mut SettingsMut,
    head: Option<crate::headtrack::Source>,
    dialog: &mut crate::input_dialog::InputDialog,
) {
    if !w.open || dialog.open {
        return;
    }
    let mut back = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    menu_window(ctx, "Options", 500.0).show(ctx, |ui| {
        ui.label(egui::RichText::new(t!("options-title")).heading().color(BRIGHT));
        ui.horizontal(|ui| {
            for (tab, _, name) in Tab::ALL {
                ui.selectable_value(&mut w.tab, tab, egui::RichText::new(t!(name)).size(16.0));
            }
        });
        ui.separator();
        // The other tabs are laid out unseen for their size.
        for (tab, ..) in Tab::ALL.into_iter().filter(|t| t.0 != w.tab) {
            let builder =
                egui::UiBuilder::new().id_salt(("measure", tab as u8)).invisible().sizing_pass();
            let mut hidden = ui.new_child(builder);
            tab_options(&mut hidden, tab, settings, head, dialog);
            w.body = w.body.max(hidden.min_rect().size());
        }
        let shown = ui.scope(|ui| {
            ui.set_min_size(w.body);
            tab_options(ui, w.tab, settings, head, dialog);
        });
        w.body = w.body.max(shown.response.rect.size());
        ui.add_space(8.0);
        ui.separator();
        ui.label(egui::RichText::new(t!("options-footer")).small().color(DIM));
        if facts(ui.add(big_button(t!("back"))), t_lines!("back.facts")).clicked() {
            back = true;
        }
    });
    if back {
        w.open = false;
    }
}

fn tab_options(
    ui: &mut egui::Ui,
    tab: Tab,
    settings: &mut SettingsMut,
    head: Option<crate::headtrack::Source>,
    dialog: &mut crate::input_dialog::InputDialog,
) {
    match tab {
        Tab::Game => game_options(ui, settings),
        Tab::Graphics => graphics_options(ui, settings),
        Tab::Sound => sound_options(ui, settings),
        Tab::Controls => control_options(ui, settings, head, dialog),
    }
}

/// CLASSIC / MODERN switches (the message of each name has the facts): true if one is flipped.
fn switches<const N: usize>(ui: &mut egui::Ui, list: [(&mut bool, &str); N]) -> bool {
    let mut toggled = false;
    for (modern, id) in list {
        toggled |=
            facts(slide_switch(ui, t!(id), modern), t_lines!(&format!("{id}.facts"))).changed();
    }
    toggled
}

/// The language of the menus and messages, in OPTIONS and in the in-flight menu: true if it
/// changed (it is shown from the next frame).
pub fn language_option(ui: &mut egui::Ui, settings: &mut SettingsMut) -> bool {
    section(ui, t!("options-section-language"));
    let mut lang = settings.language;
    // Alphabetical by the language's own name, accents ignored; the other scripts follow the
    // Latin one.
    let mut languages = Language::ALL;
    languages.sort_by_cached_key(|l| stealth_i18n::fold_accents(l.native_name()).to_lowercase());
    let options = languages.map(|l| (l, list_name(l), t_lines!("language.facts")));
    crate::menu::dropdown(ui, "language", &mut lang, options);
    if lang == settings.language {
        return false;
    }
    settings.language = lang;
    stealth_i18n::set_language(lang);
    true
}

fn game_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    language_option(ui, settings);
    section(ui, t!("options-section-classic-modern"));
    let s = settings.bypass_change_detection();
    let (mut cockpit, mut hud) = (!s.classic_cockpit, !s.classic_hud);
    let (mut avionics, mut flight) = (!s.classic_avionics, !s.classic_flight);
    let mut toggled = switches(
        ui,
        [
            (&mut cockpit, "options-cockpit"),
            (&mut hud, "options-hud"),
            (&mut avionics, "options-avionics"),
            (&mut flight, "options-flight-model"),
        ],
    );
    let s = settings.bypass_change_detection();
    s.classic_cockpit = !cockpit;
    s.classic_hud = !hud;
    s.classic_avionics = !avionics;
    s.classic_flight = !flight;
    section(ui, t!("options-section-flying"));
    toggled |= facts(
        ui.checkbox(&mut s.auto_trim, t!("options-auto-trim")),
        t_lines!("options-auto-trim.facts"),
    )
    .changed();
    toggled |= facts(
        ui.checkbox(&mut s.tooltips, t!("options-tooltips")),
        t_lines!("options-tooltips.facts"),
    )
    .changed();
    toggled |= facts(
        ui.checkbox(&mut s.player_labels, t!("options-player-labels")),
        t_lines!("options-player-labels.facts"),
    )
    .changed();
    let mut percent = s.player_label_size * 100.0;
    let range = PLAYER_LABEL_SIZE_RANGE.start() * 100.0..=PLAYER_LABEL_SIZE_RANGE.end() * 100.0;
    ui.indent("label_size", |ui| {
        let slider = egui::Slider::new(&mut percent, range).step_by(10.0).fixed_decimals(0);
        let resp = facts(
            ui.add_enabled(s.player_labels, slider.suffix("%")),
            t_lines!("options-label-size.facts"),
        );
        if resp.changed() {
            s.player_label_size = percent / 100.0;
            toggled = true;
        }
    });
    section(ui, t!("options-section-vr-size"));
    let mut percent = s.vr_cockpit_scale * 100.0;
    let range = VR_COCKPIT_SCALE_RANGE.start() * 100.0..=VR_COCKPIT_SCALE_RANGE.end() * 100.0;
    if facts(
        ui.add(egui::Slider::new(&mut percent, range).step_by(5.0).fixed_decimals(0).suffix("%")),
        t_lines!("options-vr-size.facts"),
    )
    .changed()
    {
        s.vr_cockpit_scale = percent / 100.0;
        toggled = true;
    }
    if toggled {
        settings.set_changed();
    }
    crate::menu::tacview_options(ui, settings);
    gif_options(ui, settings);
}

/// The GIF recording's size (with the resolution it gives) and frame rate.
fn gif_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    // The browser build writes no files.
    if cfg!(target_arch = "wasm32") {
        return;
    }
    section(ui, t!("options-section-gif"));
    let s = settings.bypass_change_detection();
    let before = (s.gif_scale, s.gif_fps);
    let window = ui.ctx().content_rect().size() * ui.ctx().pixels_per_point();
    let size =
        crate::gif_recorder::gif_size(UVec2::new(window.x as u32, window.y as u32), s.gif_scale);
    // The choices' notes: the message of a choice's value, where it has one.
    let noted =
        |id: String, f: &'static [&'static str]| if f.is_empty() { f } else { t_lines!(&id) };
    let scales = crate::gif_recorder::SCALES
        .map(|(v, name, f)| (v, name, noted(format!("options-gif-scale-{v}.facts"), f)));
    let rates = crate::gif_recorder::FRAME_RATES
        .map(|(v, name, f)| (v, name, noted(format!("options-gif-rate-{v}.facts"), f)));
    // The names padded to the longer (the font is monospace).
    let names = [t!("options-gif-size"), t!("options-gif-rate")];
    let pad = names.iter().map(|n| n.chars().count()).max().unwrap_or(0) + 1;
    ui.horizontal(|ui| {
        ui.label(format!("{:<pad$}", names[0]));
        ui.allocate_ui(egui::vec2(90.0, 22.0), |ui| {
            crate::menu::dropdown(ui, "gif_scale", &mut s.gif_scale, scales)
        });
        ui.colored_label(GREEN, format!("{} x {}", size.x, size.y));
    });
    ui.horizontal(|ui| {
        ui.label(format!("{:<pad$}", names[1]));
        ui.allocate_ui(egui::vec2(90.0, 22.0), |ui| {
            crate::menu::dropdown(ui, "gif_fps", &mut s.gif_fps, rates)
        });
    });
    ui.label(egui::RichText::new(t!("options-gif-hint")).small().color(DIM));
    if (s.gif_scale, s.gif_fps) != before {
        settings.set_changed();
    }
}

fn graphics_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    section(ui, t!("options-section-rendering"));
    let s = settings.bypass_change_detection();
    let mut toggled = switches(
        ui,
        [
            (&mut s.modern_shading, "options-shaders"),
            (&mut s.shadows, "options-shadows"),
            (&mut s.local_lights, "options-point-lights"),
            (&mut s.continuous_time, "options-time-of-day"),
        ],
    );
    if !crate::WEBGL2 {
        toggled |=
            facts(ui.checkbox(&mut s.msaa, t!("options-msaa")), t_lines!("options-msaa.facts"))
                .changed();
    }
    if !crate::WEBGL2 {
        toggled |= facts(
            ui.checkbox(&mut s.vr_auto_detail, t!("options-auto-detail")),
            t_lines!("options-auto-detail.facts"),
        )
        .changed();
    }
    // The classic time of day can still show the sun and the moon.
    if !s.continuous_time {
        ui.indent("sun_moon", |ui| {
            toggled |= facts(
                ui.checkbox(&mut s.sun_moon, t!("options-sun-moon")),
                t_lines!("options-sun-moon.facts"),
            )
            .changed();
        });
    }
    for (title, retro) in
        [(t!("options-section-effects"), false), (t!("options-section-retro"), true)]
    {
        section(ui, title);
        // Two columns: the effects are many.
        let column = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
        egui::Grid::new(title).num_columns(2).min_col_width(column).show(ui, |ui| {
            let list = crate::effects::Effect::ALL.into_iter().filter(|e| e.retro() == retro);
            for (i, e) in list.enumerate() {
                let mut on = e.get(&s.effects);
                if facts(ui.checkbox(&mut on, e.label()), e.facts()).changed() {
                    e.set(&mut s.effects, on);
                    toggled = true;
                }
                if i % 2 == 1 {
                    ui.end_row();
                }
            }
        });
    }
    if toggled {
        settings.set_changed();
    }
}

fn sound_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    section(ui, t!("options-section-volume"));
    let s = settings.bypass_change_detection();
    let mut changed = false;
    let volumes = [
        (&mut s.master_volume, "options-volume-master"),
        (&mut s.engine_volume, "options-volume-engine"),
        (&mut s.effects_volume, "options-volume-effects"),
        (&mut s.warning_volume, "options-volume-warnings"),
        (&mut s.rwr_volume, "options-volume-rwr"),
    ];
    // The names padded to the longest (the font is monospace).
    let pad = volumes.iter().map(|(_, id)| t!(id).chars().count()).max().unwrap_or(0) + 1;
    for (v, id) in volumes {
        let mut percent = (*v * 100.0).round();
        let f = t_lines!(&format!("{id}.facts"));
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 220.0;
            ui.label(format!("{:<pad$}", t!(id).to_uppercase()));
            let slider = egui::Slider::new(&mut percent, 0.0..=100.0)
                .step_by(5.0)
                .fixed_decimals(0)
                .suffix("%");
            if facts(ui.add(slider), f).changed() {
                *v = percent / 100.0;
                changed = true;
            }
        });
    }
    section(ui, t!("options-section-rwr-tones"));
    let tones = s.rwr_tones;
    ui.scope(|ui| {
        ui.set_width(220.0);
        crate::menu::dropdown(
            ui,
            "rwr_tones",
            &mut s.rwr_tones,
            crate::sound::RwrTones::ALL.map(|t| (t, t.name(), t.facts())),
        );
    });
    changed |= s.rwr_tones != tones;
    section(ui, t!("options-section-cockpit"));
    changed |= facts(
        ui.checkbox(&mut s.engine_low_pass, t!("options-low-pass")),
        t_lines!("options-low-pass.facts"),
    )
    .changed();
    if changed {
        settings.set_changed();
    }
}

fn control_options(
    ui: &mut egui::Ui,
    settings: &mut SettingsMut,
    head: Option<crate::headtrack::Source>,
    dialog: &mut crate::input_dialog::InputDialog,
) {
    section(ui, t!("options-section-bindings"));
    if facts(
        ui.add(big_button(t!("options-keys-joystick")).min_size(egui::vec2(300.0, 44.0))),
        t_lines!("options-keys-joystick.facts"),
    )
    .clicked()
    {
        dialog.open = true;
    }
    // The browser build hears no tracking program.
    if cfg!(target_arch = "wasm32") {
        return;
    }
    section(ui, t!("options-section-head-tracking"));
    // Read every frame by `headtrack`: no change detection needed.
    let s = settings.bypass_change_detection();
    facts(
        ui.add_enabled(
            !s.vr,
            egui::Checkbox::new(&mut s.head_tracking, t!("options-head-tracking")),
        ),
        t_lines!("options-head-tracking.facts"),
    );
    if s.head_tracking && !s.vr {
        match head {
            Some(src) => ui.colored_label(GREEN, t!("options-tracking", source = src.name())),
            None => ui.colored_label(AMBER, t!("options-no-tracker")),
        };
    }
}

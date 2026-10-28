//! The main menu's OPTIONS window: the player's own settings, saved in the preferences with the
//! menu's other choices (`prefs.rs`), on four tabs:
//!
//! - GAME: the language, cockpit, HUD, avionics and flight model style, auto trim, the cockpit
//!   tooltips, the VR cockpit size and Tacview.
//! - GRAPHICS: shaders, anti-aliasing, shadows, point lights, time of day.
//! - SOUND: the master, engine, effects and warning volumes, the engine low-pass inside the
//!   aircraft.
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
use crate::settings::{Settings, VR_COCKPIT_SCALE_RANGE};

#[derive(Resource, Default)]
pub struct OptionsWindow {
    pub open: bool,
    tab: Tab,
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
        Self { open: true, tab }
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
    menu_window(ctx, "Options", 460.0).show(ctx, |ui| {
        ui.label(egui::RichText::new(t!("options-title")).heading().color(BRIGHT));
        ui.horizontal(|ui| {
            for (tab, _, name) in Tab::ALL {
                ui.selectable_value(&mut w.tab, tab, egui::RichText::new(t!(name)).size(16.0));
            }
        });
        ui.separator();
        match w.tab {
            Tab::Game => game_options(ui, settings),
            Tab::Graphics => graphics_options(ui, settings),
            Tab::Sound => sound_options(ui, settings),
            Tab::Controls => control_options(ui, settings, head, dialog),
        }
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
    for (title, retro) in [(t!("options-section-effects"), false), (t!("options-section-retro"), true)] {
        section(ui, title);
        for e in crate::effects::Effect::ALL.into_iter().filter(|e| e.retro() == retro) {
            let mut on = e.get(&s.effects);
            if facts(ui.checkbox(&mut on, e.label()), e.facts()).changed() {
                e.set(&mut s.effects, on);
                toggled = true;
            }
        }
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

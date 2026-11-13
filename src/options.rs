//! The main menu's OPTIONS window: the player's own settings, saved in the preferences with the
//! menu's other choices (`prefs.rs`), on four tabs:
//!
//! - GAME: the language, cockpit, HUD, avionics and flight model style, auto trim, the cockpit
//!   tooltips, the player labels, the VR cockpit size, Tacview and the GIF recording (in two
//!   columns).
//! - GRAPHICS: shaders, anti-aliasing, shadows, point lights, time of day.
//! - SOUND: the master, engine, effects, warning, radar warning, radio and ATC voice volumes, which radar
//!   warning tones sound, the engine low-pass inside the aircraft, the coop radio's microphone.
//! - CONTROLS: the key and joystick bindings (the CONTROLS dialog, `input_dialog.rs`) and head
//!   tracking.
//!
//! The realism switches and crash protection belong to each game: they stay in the single
//! mission, career and campaign setups. The in-flight menu has the same pages, but for the
//! flight model and Tacview, which take effect at the next flight (`flight_menu.rs`).

use bevy::prelude::*;
use bevy_egui::egui;
use stealth_i18n::{Language, list_name, t, t_lines};

use crate::layout::{Flow, Span, Split, screen_window};
use crate::menu::{
    AMBER, DIM, GREEN, Panel, big_button, facts, footer_rule, icon, section, slide_switch,
};
use crate::settings::{
    MIC_GAIN_RANGE, PLAYER_LABEL_SIZE_RANGE, SEAT_HEIGHT_RANGE, Settings, VR_COCKPIT_SCALE_RANGE,
};

#[derive(Resource, Default)]
pub struct OptionsWindow {
    pub open: bool,
    /// The VOICES dialog is open over it (`voice_dialog.rs`).
    pub voices: bool,
    tab: Tab,
}

impl OptionsWindow {
    /// `--open-options <tab>`: open on that tab; `--open-voices`: the VOICES dialog
    /// (screenshots).
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|a| a == "--open-voices") {
            return Self { open: true, voices: true, tab: Tab::Sound, ..default() };
        }
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

/// The window's size, frame included. It is as high as the single mission screen, so the borders
/// show wherever that screen's do; on a smaller screen the tabs scroll.
const SIZE: egui::Vec2 = egui::vec2(572.0, 700.0);
/// The GAME tab's left column (the switches and the flying options).
const GAME_LEFT_WIDTH: f32 = 220.0;
/// The least width of its right column (the sliders and the recording's size).
const GAME_RIGHT_LEAST: f32 = 240.0;

/// The window (while open and the bindings dialog is not).
pub fn options_ui(
    ctx: &egui::Context,
    w: &mut OptionsWindow,
    settings: &mut SettingsMut,
    head: Option<crate::headtrack::Source>,
    dialog: &mut crate::input_dialog::InputDialog,
) {
    if !w.open || dialog.open || w.voices {
        return;
    }
    let mut back = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    Panel::new(t!("options-title")).window(screen_window(ctx, "Options", SIZE), ctx, |ui| {
        ui.horizontal(|ui| {
            for (tab, _, name) in Tab::ALL {
                ui.selectable_value(&mut w.tab, tab, egui::RichText::new(t!(name)).size(16.0));
            }
        });
        ui.separator();
        let split = Split::new(ui, "options");
        split.body(ui, |ui| tab_options(ui, w.tab, settings, head, dialog, &mut w.voices));
        split.footer(ui, |ui| {
            ui.add_space(4.0);
            footer_rule(ui);
            ui.label(egui::RichText::new(t!("options-footer")).small().color(DIM));
            if facts(ui.add(big_button(t!("back")).icon(icon::BACK)), t_lines!("back.facts"))
                .clicked()
            {
                back = true;
            }
        });
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
    voices: &mut bool,
) {
    match tab {
        Tab::Game => game_options(ui, settings),
        Tab::Graphics => graphics_options(ui, settings),
        Tab::Sound => sound_options(ui, settings, voices),
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
    // Two columns keep the tab as low as the others: the window is as high as the single
    // mission screen.
    let mut flow = Flow::new(ui);
    flow.card(ui, Span::Fixed(GAME_LEFT_WIDTH), |ui| {
        language_option(ui, settings);
        style_options(ui, settings);
        flying_options(ui, settings);
    });
    flow.card(ui, Span::Fill(GAME_RIGHT_LEAST), |ui| {
        vr_size_options(ui, settings);
        crate::menu::tacview_options(ui, settings);
        gif_options(ui, settings);
    });
    flow.finish(ui);
}

fn style_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    section(ui, t!("options-section-classic-modern"));
    let s = settings.bypass_change_detection();
    let (mut cockpit, mut hud) = (!s.classic_cockpit, !s.classic_hud);
    let (mut avionics, mut flight) = (!s.classic_avionics, !s.classic_flight);
    let toggled = switches(
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
    if toggled {
        settings.set_changed();
    }
}

fn flying_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    let s = settings.bypass_change_detection();
    let mut toggled = false;
    section(ui, t!("options-section-flying"));
    toggled |= facts(
        ui.checkbox(&mut s.auto_trim, t!("options-auto-trim")),
        t_lines!("options-auto-trim.facts"),
    )
    .changed();
    toggled |= facts(
        ui.checkbox(&mut s.keep_time_accel, t!("options-keep-time-accel")),
        t_lines!("options-keep-time-accel.facts"),
    )
    .changed();
    toggled |= facts(
        ui.checkbox(&mut s.tooltips, t!("options-tooltips")),
        t_lines!("options-tooltips.facts"),
    )
    .changed();
    toggled |=
        facts(ui.checkbox(&mut s.atc, t!("options-atc")), t_lines!("options-atc.facts")).changed();
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
    let mut cm = s.seat_height * 100.0;
    let range = SEAT_HEIGHT_RANGE.start() * 100.0..=SEAT_HEIGHT_RANGE.end() * 100.0;
    let slider = egui::Slider::new(&mut cm, range)
        .step_by(1.0)
        .fixed_decimals(0)
        .suffix(" cm")
        .text(t!("options-seat-height"));
    if facts(ui.add(slider), t_lines!("options-seat-height.facts")).changed() {
        s.seat_height = cm / 100.0;
        toggled = true;
    }
    if toggled {
        settings.set_changed();
    }
}

fn vr_size_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    let s = settings.bypass_change_detection();
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
        settings.set_changed();
    }
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
        // Off while the pixelize look is on (Retro).
        let forced = crate::palette::pixelized(s);
        let mut msaa = s.msaa && !forced;
        let check = ui.add_enabled(!forced, egui::Checkbox::new(&mut msaa, t!("options-msaa")));
        let check = if forced {
            check.on_disabled_hover_text(t!("options-msaa-forced"))
        } else {
            facts(check, t_lines!("options-msaa.facts"))
        };
        if check.changed() {
            s.msaa = msaa;
            toggled = true;
        }
    }
    if !crate::WEBGL2 {
        toggled |= facts(
            ui.checkbox(&mut s.vr_auto_detail, t!("options-auto-detail")),
            t_lines!("options-auto-detail.facts"),
        )
        .changed();
    }
    let clouds = s.cloud_quality;
    ui.horizontal(|ui| {
        ui.label(t!("options-cloud-quality"));
        ui.allocate_ui(egui::vec2(140.0, 22.0), |ui| {
            crate::menu::dropdown(
                ui,
                "cloud_quality",
                &mut s.cloud_quality,
                crate::weather::CloudQuality::ALL.map(|q| (q, q.name(), q.facts())),
            )
        });
    });
    toggled |= s.cloud_quality != clouds;
    for (distance, id) in [
        (&mut s.scenery_distance, "options-scenery-distance"),
        (&mut s.city_light_distance, "options-city-light-distance"),
    ] {
        let range = crate::settings::SCENERY_DISTANCE_RANGE;
        let mut km = *distance / 1000.0;
        let slider = egui::Slider::new(&mut km, range.start() / 1000.0..=range.end() / 1000.0)
            .step_by(1.0)
            .fixed_decimals(0)
            .suffix(" km")
            .text(t!(id));
        if facts(ui.add(slider), t_lines!(&format!("{id}.facts"))).changed() {
            *distance = km * 1000.0;
            toggled = true;
        }
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
        // Two columns: the effects are many; one where a label would not fit its column.
        let column = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
        let font = egui::TextStyle::Body.resolve(ui.style());
        let box_width = ui.spacing().icon_width + ui.spacing().icon_spacing;
        let widest = crate::effects::Effect::ALL
            .into_iter()
            .filter(|e| e.retro() == retro && !e.pixel_part())
            .map(|e| {
                let label = ui.painter().layout_no_wrap(
                    e.label().to_string(),
                    font.clone(),
                    egui::Color32::WHITE,
                );
                // Pixelize has its slider beside it.
                let slider = if e == crate::effects::Effect::Pixelize { 150.0 } else { 0.0 };
                box_width + label.size().x + slider + 2.0 * ui.spacing().item_spacing.x
            })
            .fold(0.0, f32::max);
        let columns = if widest > column { 1 } else { 2 };
        let column = if columns == 1 { ui.available_width() } else { column };
        egui::Grid::new(title).num_columns(columns).min_col_width(column).show(ui, |ui| {
            let list = crate::effects::Effect::ALL
                .into_iter()
                .filter(|e| e.retro() == retro && !e.pixel_part());
            for (i, e) in list.enumerate() {
                let mut on = e.get(&s.effects);
                ui.horizontal(|ui| {
                    if facts(ui.checkbox(&mut on, e.label()), e.facts()).changed() {
                        e.set(&mut s.effects, on);
                        toggled = true;
                    }
                    // Pixelize: the size of its blocks, beside it.
                    if e == crate::effects::Effect::Pixelize {
                        let blocks = crate::effects::PIXEL_BLOCKS;
                        let slider = egui::Slider::new(&mut s.effects.pixel_block, blocks)
                            .integer()
                            .suffix(" px");
                        let resp = facts(
                            ui.add_enabled(on && !s.vr, slider),
                            t_lines!("options-pixel-block.facts"),
                        );
                        toggled |= resp.changed();
                    }
                });
                if i % columns == columns - 1 {
                    ui.end_row();
                }
            }
        });
        // The parts pixelize leaves sharp: one a line under it, indented, and only with it on.
        if retro {
            ui.indent("pixelize_sharp", |ui| {
                let usable = s.effects.pixelize && !s.vr;
                for e in crate::effects::Effect::ALL.into_iter().filter(|e| e.pixel_part()) {
                    let mut on = e.get(&s.effects);
                    let check = ui.add_enabled(usable, egui::Checkbox::new(&mut on, e.label()));
                    let check = if usable {
                        facts(check, e.facts())
                    } else {
                        check.on_disabled_hover_text(t!("options-needs-pixelize"))
                    };
                    if check.changed() {
                        e.set(&mut s.effects, on);
                        toggled = true;
                    }
                }
            });
        }
    }
    if toggled {
        settings.set_changed();
    }
}

fn sound_options(ui: &mut egui::Ui, settings: &mut SettingsMut, voices: &mut bool) {
    section(ui, t!("options-section-volume"));
    let s = settings.bypass_change_detection();
    let mut changed = false;
    let volumes = [
        (&mut s.master_volume, "options-volume-master"),
        (&mut s.engine_volume, "options-volume-engine"),
        (&mut s.effects_volume, "options-volume-effects"),
        (&mut s.warning_volume, "options-volume-warnings"),
        (&mut s.rwr_volume, "options-volume-rwr"),
        (&mut s.voice_volume, "options-volume-radio"),
        (&mut s.atc_volume, "options-volume-atc"),
        (&mut s.music_volume, "options-volume-music"),
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
    section(ui, t!("options-section-atc"));
    atc_voices(ui, s, voices);
    section(ui, t!("options-section-radio"));
    changed |= facts(
        ui.checkbox(&mut s.voice, t!("options-push-to-talk")),
        t_lines!("options-push-to-talk.facts"),
    )
    .changed();
    ui.add_enabled_ui(s.voice, |ui| {
        let mut percent = (s.mic_gain * 100.0).round();
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 220.0;
            ui.label(format!("{:<9}", t!("options-mic-gain")));
            let range = MIC_GAIN_RANGE.0 * 100.0..=MIC_GAIN_RANGE.1 * 100.0;
            let slider =
                egui::Slider::new(&mut percent, range).step_by(5.0).fixed_decimals(0).suffix("%");
            if facts(ui.add(slider), t_lines!("options-mic-gain.facts")).changed() {
                s.mic_gain = percent / 100.0;
                changed = true;
            }
        });
        let devices = mic_devices();
        if !devices.is_empty() {
            let shown = if s.mic_device.is_empty() {
                t!("options-mic-default")
            } else {
                s.mic_device.as_str()
            };
            let before = s.mic_device.clone();
            egui::ComboBox::from_id_salt("mic_device").width(220.0).selected_text(shown).show_ui(
                ui,
                |ui| {
                    ui.selectable_value(
                        &mut s.mic_device,
                        String::new(),
                        t!("options-mic-default"),
                    );
                    for d in devices {
                        ui.selectable_value(&mut s.mic_device, d.clone(), d.as_str());
                    }
                },
            );
            changed |= s.mic_device != before;
        }
    });
    if changed {
        settings.set_changed();
    }
}

/// The spoken ATC's voices, and the button to the VOICES dialog.
#[cfg(not(target_arch = "wasm32"))]
fn atc_voices(ui: &mut egui::Ui, s: &Settings, voices: &mut bool) {
    let name = |n: &str| {
        if n.is_empty() { t!("options-voice-text-only").to_string() } else { n.to_string() }
    };
    ui.label(
        egui::RichText::new(t!(
            "options-voices-line",
            controllers = name(&s.atc_voice),
            pilot = name(&s.pilot_voice)
        ))
        .color(DIM),
    );
    if facts(
        ui.add(
            big_button(t!("options-voices")).icon(icon::DOWNLOAD).min_size(egui::vec2(300.0, 44.0)),
        ),
        t_lines!("options-voices.facts"),
    )
    .clicked()
    {
        *voices = true;
    }
}

/// The browser build: the calls are text.
#[cfg(target_arch = "wasm32")]
fn atc_voices(ui: &mut egui::Ui, _s: &Settings, _voices: &mut bool) {
    ui.label(egui::RichText::new(t!("options-voices-web")).color(DIM));
}

/// The microphones (looked up once: it takes a moment).
fn mic_devices() -> &'static [String] {
    static DEVICES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    DEVICES.get_or_init(crate::voice::devices)
}

fn control_options(
    ui: &mut egui::Ui,
    settings: &mut SettingsMut,
    head: Option<crate::headtrack::Source>,
    dialog: &mut crate::input_dialog::InputDialog,
) {
    section(ui, t!("options-section-bindings"));
    if facts(
        ui.add(
            big_button(t!("options-keys-joystick"))
                .icon(icon::KEYS)
                .min_size(egui::vec2(300.0, 44.0)),
        ),
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
    facts(
        ui.add_enabled(
            s.head_tracking && !s.vr,
            egui::Checkbox::new(&mut s.tracking_mouse_look, t!("options-tracking-mouse")),
        ),
        t_lines!("options-tracking-mouse.facts"),
    );
    if s.head_tracking && !s.vr {
        match head {
            Some(src) => ui.colored_label(GREEN, t!("options-tracking", source = src.name())),
            None => ui.colored_label(AMBER, t!("options-no-tracker")),
        };
    }
}

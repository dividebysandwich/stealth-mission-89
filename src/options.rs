//! The main menu's OPTIONS window: the player's own settings, saved in the preferences with the
//! menu's other choices (`prefs.rs`), on four tabs:
//!
//! - GAME: cockpit, HUD, avionics and flight model style, auto trim, the cockpit tooltips, the
//!   player labels, the VR cockpit size and Tacview.
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

use crate::menu::{
    AMBER, DIM, GREEN, Panel, big_button, facts, footer_rule, icon, menu_window, section,
    slide_switch,
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
        let tab = Tab::ALL.iter().find(|(_, n)| *n == name).map_or(Tab::Game, |t| t.0);
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
    const ALL: [(Tab, &'static str); 4] = [
        (Tab::Game, "GAME"),
        (Tab::Graphics, "GRAPHICS"),
        (Tab::Sound, "SOUND"),
        (Tab::Controls, "CONTROLS"),
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
    Panel::new("OPTIONS").window(menu_window(ctx, "Options", 500.0), ctx, |ui| {
        ui.horizontal(|ui| {
            for (tab, name) in Tab::ALL {
                ui.selectable_value(&mut w.tab, tab, egui::RichText::new(name).size(16.0));
            }
        });
        ui.separator();
        // The other tabs are laid out unseen for their size.
        for (tab, _) in Tab::ALL.into_iter().filter(|t| t.0 != w.tab) {
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
        ui.add_space(4.0);
        footer_rule(ui);
        ui.label(
            egui::RichText::new(
                "Saved for the next flights; the realism switches are chosen with each game",
            )
            .small()
            .color(DIM),
        );
        if facts(ui.add(big_button("BACK").icon(icon::BACK)), &["Back to the previous screen"])
            .clicked()
        {
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

/// CLASSIC / MODERN switches with their facts: true if one is flipped.
fn switches<const N: usize>(ui: &mut egui::Ui, list: [(&mut bool, &str, &[&str]); N]) -> bool {
    let mut toggled = false;
    for (modern, name, f) in list {
        toggled |= facts(slide_switch(ui, name, modern), f).changed();
    }
    toggled
}

fn game_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    section(ui, "Classic / Modern");
    let s = settings.bypass_change_detection();
    let (mut cockpit, mut hud) = (!s.classic_cockpit, !s.classic_hud);
    let (mut avionics, mut flight) = (!s.classic_avionics, !s.classic_flight);
    let mut toggled = switches(
        ui,
        [
            (
                &mut cockpit,
                "Cockpit",
                &[
                    "Modern: high-resolution displays with bezel buttons",
                    "Classic: the DOS-era instrument panel layout and 77x60 displays (F-117A)",
                ][..],
            ),
            (
                &mut hud,
                "HUD",
                &[
                    "Modern: high-resolution F-16 style HUD in green with NAV, AA and AG modes",
                    "Classic: the DOS-era HUD",
                ][..],
            ),
            (
                &mut avionics,
                "Avionics",
                &[
                    "Modern: F-16 style MFD formats on either MFD (situation display, HARM, targeting pod, map, stores) with a slewable cursor",
                    "Classic: map and TAC left, camera, WAY, WPN and ORD right",
                ][..],
            ),
            (
                &mut flight,
                "Flight model",
                &[
                    "Modern: aerodynamic flight model with fly-by-wire: the stick commands pitch and roll rate; released, the nose stays put and the flight path is held",
                    "Classic: a DOS-era flight model",
                ][..],
            ),
        ],
    );
    let s = settings.bypass_change_detection();
    s.classic_cockpit = !cockpit;
    s.classic_hud = !hud;
    s.classic_avionics = !avionics;
    s.classic_flight = !flight;
    section(ui, "Flying");
    toggled |= facts(
        ui.checkbox(&mut s.auto_trim, "Auto trim"),
        &[
            "With the stick released, the trim holds the attitude where you left it",
            "Helicopters: no more nose rising with speed; jets: trims out damaged control surfaces",
            "Manual trim: Alt + arrow keys, Alt + Home centres it",
        ],
    )
    .changed();
    toggled |= facts(
        ui.checkbox(&mut s.tooltips, "Cockpit tooltips"),
        &["What a cockpit control or display is, under the mouse or a VR fingertip"],
    )
    .changed();
    toggled |= facts(
        ui.checkbox(&mut s.player_labels, "Player labels"),
        &["Multiplayer: the other pilots' names over their aircraft, however far away"],
    )
    .changed();
    let mut percent = s.player_label_size * 100.0;
    let range = PLAYER_LABEL_SIZE_RANGE.start() * 100.0..=PLAYER_LABEL_SIZE_RANGE.end() * 100.0;
    ui.indent("label_size", |ui| {
        let slider = egui::Slider::new(&mut percent, range).step_by(10.0).fixed_decimals(0);
        let resp = facts(
            ui.add_enabled(s.player_labels, slider.suffix("%")),
            &["Size of the player labels", "50%: the default"],
        );
        if resp.changed() {
            s.player_label_size = percent / 100.0;
            toggled = true;
        }
    });
    section(ui, "VR cockpit size");
    let mut percent = s.vr_cockpit_scale * 100.0;
    let range = VR_COCKPIT_SCALE_RANGE.start() * 100.0..=VR_COCKPIT_SCALE_RANGE.end() * 100.0;
    if facts(
        ui.add(egui::Slider::new(&mut percent, range).step_by(5.0).fixed_decimals(0).suffix("%")),
        &["Size of the cockpit (and the world) in the headset", "100%: life size"],
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
    section(ui, "GIF recording");
    let s = settings.bypass_change_detection();
    let before = (s.gif_scale, s.gif_fps);
    let window = ui.ctx().content_rect().size() * ui.ctx().pixels_per_point();
    let size =
        crate::gif_recorder::gif_size(UVec2::new(window.x as u32, window.y as u32), s.gif_scale);
    ui.horizontal(|ui| {
        ui.label(format!("{:<6}", "SIZE"));
        ui.allocate_ui(egui::vec2(90.0, 22.0), |ui| {
            crate::menu::dropdown(ui, "gif_scale", &mut s.gif_scale, crate::gif_recorder::SCALES)
        });
        ui.colored_label(GREEN, format!("{} x {}", size.x, size.y));
    });
    ui.horizontal(|ui| {
        ui.label(format!("{:<6}", "RATE"));
        ui.allocate_ui(egui::vec2(90.0, 22.0), |ui| {
            crate::menu::dropdown(ui, "gif_fps", &mut s.gif_fps, crate::gif_recorder::FRAME_RATES)
        });
    });
    ui.label(
        egui::RichText::new("F12 (unless rebound) starts and stops; saved in your Pictures folder")
            .small()
            .color(DIM),
    );
    if (s.gif_scale, s.gif_fps) != before {
        settings.set_changed();
    }
}

fn graphics_options(ui: &mut egui::Ui, settings: &mut SettingsMut) {
    section(ui, "Rendering");
    let s = settings.bypass_change_detection();
    let mut toggled = false;
    for (modern, name, f) in [
        (
            &mut s.modern_shading,
            "Shaders",
            &["Modern: subtle directional shading, smooth distance haze", "Classic: flat shading"]
                [..],
        ),
        (
            &mut s.shadows,
            "Shadows",
            &[
                "Modern: sun shadows in the cockpit and the world (costs GPU time)",
                "Classic: no shadows",
            ][..],
        ),
        (
            &mut s.local_lights,
            "Point lights",
            &[
                "Modern: cockpit flood and display lights; explosions, plumes and runway lights light the scene",
                "Classic: sun and sky light only",
            ][..],
        ),
        (
            &mut s.continuous_time,
            "Time of day",
            &[
                "Modern: the sun and the moon move with the mission clock, the sky darkens and brightens with the sun; stars and the moon's phase at night",
                "Modern: moonlight sets how far enemy pilots see you at night (2 km on a moonless night, 4 km under a full moon)",
                "Classic: a slow change of colours and a fixed sun",
            ][..],
        ),
    ] {
        toggled |= facts(slide_switch(ui, name, modern), f).changed();
    }
    if !crate::WEBGL2 {
        toggled |= facts(
            ui.checkbox(&mut s.msaa, "Anti-aliasing (MSAA)"),
            &["Smooth edges: 4x MSAA, 2x in VR (costs GPU time)", "Off: hard pixel edges"],
        )
        .changed();
    }
    if !crate::WEBGL2 {
        toggled |= facts(
            ui.checkbox(&mut s.vr_auto_detail, "Automatic scenery detail (VR)"),
            &[
                "In the headset, the range of towns, forests and fields shrinks while frames run late and grows back when there is time to spare",
                "The graphics settings stay as chosen",
            ],
        )
        .changed();
    }
    // The classic time of day can still show the sun and the moon.
    if !s.continuous_time {
        ui.indent("sun_moon", |ui| {
            toggled |= facts(
                ui.checkbox(&mut s.sun_moon, "Sun and moon"),
                &[
                    "The sun in the sky by day, the moon and the stars by night",
                    "The moon shows the mission's phase",
                ],
            )
            .changed();
        });
    }
    for (title, retro) in [("Effects", false), ("Retro", true)] {
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
    section(ui, "Volume");
    let s = settings.bypass_change_detection();
    let mut changed = false;
    for (v, name, f) in [
        (&mut s.master_volume, "Master", &["Every sound"][..]),
        (
            &mut s.engine_volume,
            "Engine",
            &["Your engine or rotor, and the aircraft heard around you"][..],
        ),
        (&mut s.effects_volume, "Effects", &["Weapons, explosions, gear, radio and the rest"][..]),
        (
            &mut s.warning_volume,
            "Warnings",
            &["Stall horn, radar and missile warnings, the fire siren, the Sidewinder tones"][..],
        ),
        (
            &mut s.rwr_volume,
            "RWR",
            &["The radar warning tones, as a part of the warnings' volume"][..],
        ),
    ] {
        let mut percent = (*v * 100.0).round();
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 220.0;
            ui.label(format!("{:<9}", name.to_ascii_uppercase()));
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
    section(ui, "Radar warning tones");
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
    section(ui, "Cockpit");
    changed |= facts(
        ui.checkbox(&mut s.engine_low_pass, "Engine low-pass"),
        &[
            "Inside the aircraft the engine sounds muffled (the canopy and helmet take the highs off)",
            "Off: the same as outside",
        ],
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
    section(ui, "Bindings");
    if facts(
        ui.add(big_button("KEYS AND JOYSTICK").icon(icon::KEYS).min_size(egui::vec2(300.0, 44.0))),
        &["Map keys, joystick and HOTAS axes and buttons", "Saved for the next flights"],
    )
    .clicked()
    {
        dialog.open = true;
    }
    // The browser build hears no tracking program.
    if cfg!(target_arch = "wasm32") {
        return;
    }
    section(ui, "Head/eye tracking");
    // Read every frame by `headtrack`: no change detection needed.
    let s = settings.bypass_change_detection();
    facts(
        ui.add_enabled(!s.vr, egui::Checkbox::new(&mut s.head_tracking, "Head/Eye Tracking")),
        &[
            "Your head turns and moves the view in the cockpit (desktop; in VR the headset does)",
            "Tracking program (OpenTrack...): FreeTrack output, or UDP to 127.0.0.1 port 4242",
            "Home: the current head position becomes the centre",
        ],
    );
    if s.head_tracking && !s.vr {
        match head {
            Some(src) => ui.colored_label(GREEN, format!("Tracking ({})", src.name())),
            None => ui.colored_label(AMBER, "No tracker heard: start the tracking program"),
        };
    }
}

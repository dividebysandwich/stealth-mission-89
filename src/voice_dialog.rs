//! The VOICES dialog (from OPTIONS > SOUND): picks the spoken ATC's voices, the controllers'
//! and the pilot's (and a multi-speaker voice's speaker), each with a preview through the
//! radio; lists the voices installed, with their licence and credit, to delete; downloads
//! others from the catalogue (`tts::catalog`), a progress window with CANCEL over it meanwhile.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::layout::{Split, screen_window};
use crate::menu::{AMBER, BRIGHT, DIM, GREEN, Panel, big_button, facts, icon, icon_text, section};
use crate::settings::{AppState, Settings};
use crate::tts::download::{Download, State};
use crate::tts::{ATC_TALKER, PILOT_TALKER, Tts, catalog, voice};

const ATC_SAMPLE: &str = "Viper 1-1, Gutersloh Tower, runway 27, wind calm, cleared for takeoff";
const PILOT_SAMPLE: &str = "Gutersloh Tower, Viper 1-1, ready for departure, runway 27";
/// The window's size, frame included; on a smaller screen the lists scroll.
const SIZE: egui::Vec2 = egui::vec2(760.0, 700.0);

pub struct VoiceDialogPlugin;

impl Plugin for VoiceDialogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoiceDialog>().add_systems(
            EguiPrimaryContextPass,
            voice_dialog.after(crate::menu::menu_ui).run_if(in_state(AppState::Menu)),
        );
    }
}

#[derive(Resource, Default)]
pub struct VoiceDialog {
    download: Option<Download>,
    /// The last download's or delete's outcome.
    status: Option<(String, bool)>,
    /// The voice whose DELETE was pressed once (asks again).
    confirm: Option<String>,
    /// The voices were looked up since the dialog opened.
    looked: bool,
}

fn voice_dialog(
    mut contexts: EguiContexts,
    mut options: ResMut<crate::options::OptionsWindow>,
    mut dialog: ResMut<VoiceDialog>,
    mut tts: ResMut<Tts>,
    mut settings: ResMut<Settings>,
) -> Result {
    if !options.voices {
        dialog.looked = false;
        return Ok(());
    }
    let Ok(ctx) = contexts.ctx_mut() else { return Ok(()) };
    let dialog = &mut *dialog;
    if !dialog.looked {
        dialog.looked = true;
        dialog.status = None;
        tts.refresh();
    }
    let s = settings.bypass_change_detection();
    let mut changed = false;
    // A download over.
    if let Some(d) = &dialog.download {
        let state = d.state();
        if state != State::Running {
            let name = d.voice.name;
            dialog.status = Some(match state.clone() {
                State::Done => (format!("{name} installed"), true),
                State::Cancelled => (format!("{name}: download cancelled"), false),
                State::Failed(e) => (format!("{name}: download failed: {e}"), false),
                State::Running => unreachable!(),
            });
            if state == State::Done {
                tts.refresh();
                // The first voice: everybody speaks.
                let id = d.voice.id.to_string();
                if s.atc_voice.is_empty() {
                    s.atc_voice = id.clone();
                    changed = true;
                }
                if s.pilot_voice.is_empty() {
                    s.pilot_voice = id;
                    // Another speaker of a multi-speaker voice than the controllers'.
                    s.pilot_speaker = if d.voice.speakers > 1 { 1 } else { 0 };
                    changed = true;
                }
            }
            dialog.download = None;
        }
    }
    let running = dialog.download.is_some();
    let mut back = !running && ctx.input(|i| i.key_pressed(egui::Key::Escape));
    Panel::new("VOICES").window(screen_window(ctx, "Voices", SIZE), ctx, |ui| {
        let split = Split::new(ui, "voices");
        split.body(ui, |ui| {
            ui.add_enabled_ui(!running, |ui| {
                changed |= choices(ui, s, &mut tts);
                installed(ui, s, &mut tts, dialog, &mut changed);
                available(ui, &tts, dialog);
                if let Some((text, good)) = &dialog.status {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(text).color(if *good { GREEN } else { AMBER }));
                }
                folder(ui);
            });
        });
        split.footer(ui, |ui| {
            ui.add_space(6.0);
            ui.add_enabled_ui(!running, |ui| {
                if facts(ui.add(big_button("BACK").icon(icon::BACK)), &["Back to the options"])
                    .clicked()
                {
                    back = true;
                }
            });
        });
    });
    progress(ctx, dialog);
    if changed {
        settings.set_changed();
    }
    if back {
        tts.stop();
        options.voices = false;
    }
    Ok(())
}

/// The controllers' and the pilot's voice, speaker and preview: true if one changed.
fn choices(ui: &mut egui::Ui, s: &mut Settings, tts: &mut Tts) -> bool {
    section(ui, "Radio voices");
    if tts.installed.is_empty() {
        ui.label(
            egui::RichText::new("No voice yet: download one below. Until then the calls are text.")
                .color(DIM),
        );
    }
    let mut changed = false;
    egui::Grid::new("voice_choices").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
        for pilot in [false, true] {
            let (label, talker, sample) = if pilot {
                ("PILOT", PILOT_TALKER, PILOT_SAMPLE)
            } else {
                ("CONTROLLERS", ATC_TALKER, ATC_SAMPLE)
            };
            let (name, speaker) = if pilot {
                (&mut s.pilot_voice, &mut s.pilot_speaker)
            } else {
                (&mut s.atc_voice, &mut s.atc_speaker)
            };
            ui.label(label);
            let before = name.clone();
            let shown = tts
                .installed
                .iter()
                .find(|v| v.name == *name)
                .map_or("None (text only)".to_string(), title);
            egui::ComboBox::from_id_salt(("voice", pilot))
                .width(280.0)
                .selected_text(shown)
                .show_ui(ui, |ui| {
                    ui.selectable_value(name, String::new(), "None (text only)");
                    for v in &tts.installed {
                        ui.selectable_value(name, v.name.clone(), title(v));
                    }
                });
            if *name != before {
                *speaker = 0;
                changed = true;
            }
            let v = tts.installed.iter().find(|v| v.name == *name);
            match v.filter(|v| v.speakers > 1) {
                Some(v) => {
                    let max = v.speakers - 1;
                    let corpus =
                        v.speaker_names.get(*speaker as usize).cloned().unwrap_or_default();
                    let drag = egui::DragValue::new(speaker).range(0..=max).prefix("speaker ");
                    changed |= facts(
                        ui.add(drag),
                        &[
                            "One of the voice's speakers (drag, or click and type)",
                            &format!("{corpus} in the recordings"),
                            "Preview to hear it",
                        ],
                    )
                    .changed();
                }
                None => {
                    ui.label("");
                }
            }
            let can = v.is_some();
            if facts(
                ui.add_enabled(can, egui::Button::new(icon_text(ui, icon::START, "PREVIEW"))),
                &["Hear it on the radio"],
            )
            .clicked()
            {
                tts.preview(talker, name, *speaker, sample);
            }
            ui.end_row();
        }
    });
    if tts.speakers(&s.atc_voice) > 1 {
        changed |= facts(
            ui.checkbox(&mut s.atc_vary_station, "A different controller at each station"),
            &[
                "Tower, approach, departure and GCA each talk with another of the voice's speakers",
                "The tower's is the one chosen above",
            ],
        )
        .changed();
    }
    if let Some(e) = tts.error() {
        ui.label(egui::RichText::new(e).color(AMBER).small());
    }
    changed
}

/// A voice's name in the lists.
fn title(v: &voice::InstalledVoice) -> String {
    match v.catalog {
        Some(c) => format!("{} ({})", c.name, c.accent),
        None => v.name.clone(),
    }
}

fn megabytes(bytes: u64) -> String {
    format!("{:.0} MB", bytes as f64 / 1e6)
}

/// The voices on disk, to delete.
fn installed(
    ui: &mut egui::Ui,
    s: &mut Settings,
    tts: &mut Tts,
    dialog: &mut VoiceDialog,
    changed: &mut bool,
) {
    if tts.installed.is_empty() {
        return;
    }
    section(ui, "Installed");
    let mut delete = None;
    for v in &tts.installed {
        let credit =
            v.catalog.map_or("Licence unknown: see the voice's MODEL_CARD".to_string(), |c| {
                format!("{}, {}", c.licence, c.credit)
            });
        let kind = if v.speakers > 1 {
            format!("{} speakers", v.speakers)
        } else {
            v.catalog.map_or("", |c| c.kind).to_string()
        };
        row(ui, &title(v), &credit, &kind, v.size, |ui| {
            if !v.removable {
                facts(
                    ui.label(egui::RichText::new("beside the game").color(DIM)),
                    &["Found in a voices folder next to the game: remove it there"],
                );
            } else if dialog.confirm.as_deref() == Some(v.name.as_str()) {
                if ui.button(icon_text(ui, icon::CANCEL, "KEEP")).clicked() {
                    dialog.confirm = None;
                }
                let sure = egui::Button::new(icon_text(ui, icon::DELETE, "DELETE"))
                    .fill(egui::Color32::from_rgb(110, 20, 20));
                if ui.add(sure).clicked() {
                    delete = Some(v.clone());
                }
            } else if facts(
                ui.button(icon_text(ui, icon::DELETE, "DELETE")),
                &["Removes its files (asks again)"],
            )
            .clicked()
            {
                dialog.confirm = Some(v.name.clone());
            }
        });
    }
    if let Some(v) = delete {
        dialog.confirm = None;
        tts.stop();
        tts.unload(&v.name);
        let failed: Vec<String> = v
            .files()
            .iter()
            .filter_map(|p| std::fs::remove_file(p).err().map(|e| format!("{}: {e}", p.display())))
            .collect();
        dialog.status = Some(if failed.is_empty() {
            (format!("{} deleted", title(&v)), true)
        } else {
            (failed.join("; "), false)
        });
        tts.refresh();
        for (name, speaker) in
            [(&mut s.atc_voice, &mut s.atc_speaker), (&mut s.pilot_voice, &mut s.pilot_speaker)]
        {
            if *name == v.name {
                name.clear();
                *speaker = 0;
                *changed = true;
            }
        }
    }
}

/// The catalogue's voices not installed, to download.
fn available(ui: &mut egui::Ui, tts: &Tts, dialog: &mut VoiceDialog) {
    let missing: Vec<&'static catalog::CatalogVoice> =
        catalog::CATALOG.iter().filter(|c| !tts.has(c.id)).collect();
    if missing.is_empty() {
        return;
    }
    section(ui, "Download");
    let Some(dir) = voice::download_dir() else {
        ui.label(
            egui::RichText::new("No folder to keep voices in (no home directory)").color(AMBER),
        );
        return;
    };
    for c in missing {
        let kind =
            if c.speakers > 1 { format!("{} speakers", c.speakers) } else { c.kind.to_string() };
        let credit = format!("{}, {}", c.licence, c.credit);
        row(ui, &format!("{} ({})", c.name, c.accent), &credit, &kind, c.size(), |ui| {
            if facts(
                ui.button(icon_text(ui, icon::DOWNLOAD, "DOWNLOAD")),
                &["From huggingface.co/rhasspy/piper-voices", "Checked before it is used"],
            )
            .clicked()
            {
                dialog.status = None;
                dialog.download = Some(Download::start(c, dir.clone()));
            }
        });
    }
}

/// One voice in a list: its name, what it is and its size, the buttons at the right; its
/// licence and credit under it.
fn row(
    ui: &mut egui::Ui,
    name: &str,
    credit: &str,
    kind: &str,
    size: u64,
    buttons: impl FnOnce(&mut egui::Ui),
) {
    let cell = |ui: &mut egui::Ui, width: f32, text: egui::RichText| {
        let layout = egui::Layout::left_to_right(egui::Align::Center);
        ui.allocate_ui_with_layout(egui::vec2(width, 22.0), layout, |ui| {
            ui.set_width(width);
            ui.add(egui::Label::new(text).truncate());
        });
    };
    ui.horizontal(|ui| {
        cell(ui, 320.0, egui::RichText::new(name).color(BRIGHT));
        cell(ui, 130.0, egui::RichText::new(kind));
        cell(ui, 70.0, egui::RichText::new(megabytes(size)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), buttons);
    });
    ui.label(egui::RichText::new(credit).small().color(DIM));
    ui.add_space(4.0);
}

/// Where the voices are kept, and a button to open it.
fn folder(ui: &mut egui::Ui) {
    let Some(dir) = voice::download_dir() else { return };
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if facts(
            ui.small_button(icon_text(ui, icon::LOAD, "OPEN")),
            &[
                "Opens the voices folder",
                "Piper voices (.onnx with its .onnx.json) put here are found too",
            ],
        )
        .clicked()
        {
            let _ = std::fs::create_dir_all(&dir);
            let opener = if cfg!(windows) {
                "explorer"
            } else if cfg!(target_os = "macos") {
                "open"
            } else {
                "xdg-open"
            };
            if let Err(e) = std::process::Command::new(opener).arg(&dir).spawn() {
                warn!("{opener} {}: {e}", dir.display());
            }
        }
        let text =
            egui::RichText::new(format!("Voices folder: {}", dir.display())).small().color(DIM);
        ui.add(egui::Label::new(text).truncate());
    });
}

/// The download's progress over the dialog, with CANCEL (Esc too).
fn progress(ctx: &egui::Context, dialog: &mut VoiceDialog) {
    let Some(d) = &dialog.download else { return };
    let (done, total) = (d.done(), d.total.max(1));
    let mut cancel = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    egui::Modal::new(egui::Id::new("voice_download")).show(ctx, |ui| {
        ui.set_width(420.0);
        ui.label(egui::RichText::new(format!("Downloading {}", d.voice.name)).color(BRIGHT));
        ui.add_space(6.0);
        let part = done as f32 / total as f32;
        let text = format!(
            "{:.1} / {:.1} MB  {:.0}%",
            done as f64 / 1e6,
            total as f64 / 1e6,
            part * 100.0
        );
        ui.add(egui::ProgressBar::new(part).text(text));
        ui.add_space(6.0);
        if ui.add(big_button("CANCEL").icon(icon::CANCEL)).clicked() {
            cancel = true;
        }
    });
    if cancel {
        d.cancel();
    }
    ctx.request_repaint();
}

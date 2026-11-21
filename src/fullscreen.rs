//! The display: fullscreen (Alt Enter, F11 or OPTIONS > GRAPHICS): the window goes borderless
//! fullscreen and back, in any screen. The choice is remembered (`window.json`); new players
//! start in fullscreen. The browser build keeps its canvas, and `--window-size` (screenshots)
//! keeps a window of that size. Also the vertical sync, the frame rate limit and the menus' size
//! of OPTIONS > GRAPHICS (`Settings::vsync`, `fps_cap`, `ui_scale`).

use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};
use serde::{Deserialize, Serialize};

use crate::settings::Settings;

pub struct FullscreenPlugin;

impl Plugin for FullscreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_scale);
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, (toggle, apply).chain()).add_systems(Last, cap_frame_rate);
    }
}

/// The window's own settings are shown and changed on the desktop only (not in the browser nor
/// on Android).
pub const DISPLAY_OPTIONS: bool = !cfg!(any(target_arch = "wasm32", target_os = "android"));

const FILE: &str = "window.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
struct WindowPrefs {
    fullscreen: bool,
}

impl Default for WindowPrefs {
    fn default() -> Self {
        Self { fullscreen: true }
    }
}

fn mode(fullscreen: bool) -> WindowMode {
    if fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    }
}

/// The primary window's mode at startup.
pub fn start_mode() -> WindowMode {
    mode(starts_fullscreen())
}

/// Whether the window starts fullscreen (`Settings::fullscreen`).
pub fn starts_fullscreen() -> bool {
    if cfg!(target_arch = "wasm32") || std::env::args().any(|a| a == "--window-size") {
        return false;
    }
    crate::prefs::load_json::<WindowPrefs>(FILE).unwrap_or_default().fullscreen
}

/// Alt Enter is held: the fullscreen key, not Enter (fire, confirm).
pub fn alt_enter(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
        && keys.any_pressed([KeyCode::Enter, KeyCode::NumpadEnter])
}

fn pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.just_pressed(KeyCode::F11)
        || (keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
            && keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]))
}

/// Alt Enter, F11: the other window mode (applied by `apply`).
fn toggle(keys: Res<ButtonInput<KeyCode>>, mut settings: ResMut<Settings>) {
    if pressed(&keys) {
        settings.fullscreen = !settings.fullscreen;
    }
}

/// The settings' window mode and vertical sync on the window; a window mode the player chose is
/// remembered.
#[cfg(not(target_arch = "wasm32"))]
fn apply(
    settings: Res<Settings>,
    mut window: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    mut saved: Local<Option<bool>>,
) {
    if !settings.is_changed() {
        return;
    }
    let Ok(mut window) = window.single_mut() else { return };
    let fullscreen = settings.fullscreen;
    let windowed = matches!(window.mode, WindowMode::Windowed);
    if windowed == fullscreen {
        window.mode = mode(fullscreen);
    }
    // The mode the window started in is not saved again (`--window-size` keeps a window).
    let started = *saved.get_or_insert(fullscreen);
    if started != fullscreen {
        *saved = Some(fullscreen);
        if let Err(e) = crate::prefs::save_json(FILE, &WindowPrefs { fullscreen }) {
            warn!("{FILE}: {e:#}");
        }
    }
    use bevy::window::PresentMode;
    let present = if settings.vsync { PresentMode::AutoVsync } else { PresentMode::AutoNoVsync };
    if window.present_mode != present {
        window.present_mode = present;
    }
}

/// The frame rate limit: the frame waits out the rest of its time (on the desktop, outside VR,
/// where the headset's runtime paces the frames).
#[cfg(not(target_arch = "wasm32"))]
fn cap_frame_rate(settings: Res<Settings>, mut last: Local<Option<std::time::Instant>>) {
    let now = std::time::Instant::now();
    if settings.fps_cap == 0 || settings.vr {
        *last = None;
        return;
    }
    let frame = std::time::Duration::from_secs_f64(1.0 / f64::from(settings.fps_cap));
    if let Some(prev) = *last {
        let spent = now.duration_since(prev);
        if spent < frame {
            std::thread::sleep(frame - spent);
        }
    }
    *last = Some(std::time::Instant::now());
}

/// The menus' size.
fn ui_scale(settings: Res<Settings>) {
    if settings.is_changed() {
        crate::menu::set_ui_scale(settings.ui_scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_players_start_fullscreen() {
        assert!(WindowPrefs::default().fullscreen);
        let p: WindowPrefs = serde_json::from_str("{}").unwrap();
        assert!(p.fullscreen);
    }

    #[test]
    fn alt_enter_and_f11_toggle() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Enter);
        assert!(!pressed(&keys), "Enter alone fires");
        keys.release_all();
        keys.clear();
        keys.press(KeyCode::AltLeft);
        keys.press(KeyCode::Enter);
        assert!(pressed(&keys) && alt_enter(&keys));
        keys.release_all();
        keys.clear();
        keys.press(KeyCode::F11);
        assert!(pressed(&keys));
    }
}

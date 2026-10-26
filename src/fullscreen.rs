//! Fullscreen (Alt Enter or F11): the window goes borderless fullscreen and back, in any screen.
//! The choice is remembered (`window.json`); new players start in fullscreen. The browser build
//! keeps its canvas, and `--window-size` (screenshots) keeps a window of that size.

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};
use serde::{Deserialize, Serialize};

pub struct FullscreenPlugin;

impl Plugin for FullscreenPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, toggle);
    }
}

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
    if cfg!(target_arch = "wasm32") || std::env::args().any(|a| a == "--window-size") {
        return WindowMode::Windowed;
    }
    mode(crate::prefs::load_json::<WindowPrefs>(FILE).unwrap_or_default().fullscreen)
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

fn toggle(keys: Res<ButtonInput<KeyCode>>, mut window: Query<&mut Window, With<PrimaryWindow>>) {
    if !pressed(&keys) {
        return;
    }
    let Ok(mut window) = window.single_mut() else { return };
    let fullscreen = matches!(window.mode, WindowMode::Windowed);
    window.mode = mode(fullscreen);
    if let Err(e) = crate::prefs::save_json(FILE, &WindowPrefs { fullscreen }) {
        warn!("{FILE}: {e:#}");
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

//! The Android app's entry point: the activity starts the game here, in the menu, flat.
#![cfg(target_os = "android")]

use bevy::prelude::*;

#[bevy_main]
fn main() {
    // The player's files in the app's private storage (/data/data/<package>/files).
    if let Some(dir) = bevy::android::ANDROID_APP.get().and_then(|app| app.internal_data_path()) {
        // SAFETY: before any other thread starts.
        unsafe { std::env::set_var("SM89_HOME", dir) };
    }
    sm89::run();
}

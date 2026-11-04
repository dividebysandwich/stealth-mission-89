//! The Android soft keyboard for the menus' text fields (pilot names, addresses, the Ops chat).
//! bevy_egui shows it when a field takes the focus and hides it when the field loses it, through
//! winit's `set_ime_allowed` (patched in vendor/winit to ask InputMethodManager: the NDK's call
//! does nothing on recent Android). The back gesture hides the keyboard while the field keeps
//! the focus, and bevy_egui asks again only when the focus changes: a tap on the field shows it
//! again. The keys arrive as key events, as a hardware keyboard's.

use bevy::ecs::system::NonSendMarker;
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContext, EguiPostUpdateSet};

pub struct AndroidKeyboardPlugin;

impl Plugin for AndroidKeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, reshow.after(EguiPostUpdateSet::ProcessOutput));
    }
}

/// A finger lifted with a text field focused: the keyboard up (again).
fn reshow(
    touches: Res<Touches>,
    mut contexts: Query<&mut EguiContext>,
    window: Query<Entity, With<PrimaryWindow>>,
    _main_thread: NonSendMarker,
) {
    if touches.iter_just_released().next().is_none()
        || !contexts.iter_mut().any(|mut c| c.get_mut().text_edit_focused())
    {
        return;
    }
    let Ok(entity) = window.single() else { return };
    bevy::winit::WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(window) = windows.get_window(entity) {
            window.set_ime_allowed(true);
        }
    });
}

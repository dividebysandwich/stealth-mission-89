//! Linux: a Vulkan GPU, checked before Bevy starts. Without a Vulkan driver (a virtual machine,
//! a desktop without Mesa's Vulkan drivers) wgpu falls back to OpenGL, and Bevy can't present
//! to a window with that (it makes the OpenGL instance without the window's display, so the
//! surface offers no present modes): it panics in `create_surfaces` before showing anything.
//! Instead a plain window, drawn on the CPU, says what is missing, and the game exits.

use std::num::NonZeroU32;
use std::rc::Rc;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const TITLE: &str = "Stealth Mission 89 needs a Vulkan GPU";

const MESSAGE: &str = "\
No GPU with a Vulkan driver was found.

Install your distribution's Vulkan driver (Mesa's vulkan-radeon,
vulkan-intel or vulkan-nouveau, or NVIDIA's driver), then start
the game again. A virtual machine needs 3D acceleration.

Press Escape or close this window to quit.";

/// Shows the message and exits when there is no Vulkan adapter. `WGPU_BACKEND` picks the
/// backend by hand, so it skips the check.
pub fn exit_if_missing() {
    if std::env::var_os("WGPU_BACKEND").is_some() || has_vulkan() {
        return;
    }
    eprintln!("{TITLE}\n\n{MESSAGE}");
    if let Ok(event_loop) = EventLoop::new() {
        let _ = event_loop.run_app(&mut Message::default());
    }
    std::process::exit(1);
}

fn has_vulkan() -> bool {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    !bevy::tasks::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN)).is_empty()
}

#[derive(Default)]
struct Message {
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
}

impl ApplicationHandler for Message {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title(TITLE)
            .with_inner_size(LogicalSize::new(620.0, 230.0));
        let Ok(window) = event_loop.create_window(attributes) else {
            event_loop.exit();
            return;
        };
        let window = Rc::new(window);
        let surface = softbuffer::Context::new(window.clone())
            .and_then(|context| softbuffer::Surface::new(&context, window.clone()));
        match surface {
            Ok(surface) => self.surface = Some(surface),
            Err(_) => event_loop.exit(),
        }
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape | NamedKey::Enter),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::Resized(_) => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }
}

impl Message {
    fn draw(&mut self) {
        let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else {
            return;
        };
        let size = window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return;
        };
        if surface.resize(w, h).is_err() {
            return;
        }
        let Ok(mut buffer) = surface.buffer_mut() else {
            return;
        };
        let (w, h) = (size.width as usize, size.height as usize);
        const BACKGROUND: [f32; 3] = [16.0, 20.0, 18.0];
        const TEXT: [f32; 3] = [120.0, 230.0, 140.0];
        buffer.fill(pack(BACKGROUND));
        let font = FontRef::try_from_slice(epaint_default_fonts::HACK_REGULAR).unwrap();
        let scale = window.scale_factor() as f32;
        let font = font.as_scaled(PxScale::from(16.0 * scale));
        let margin = 20.0 * scale;
        let mut y = margin + font.ascent();
        for line in std::iter::once(TITLE).chain([""]).chain(MESSAGE.lines()) {
            let mut x = margin;
            for c in line.chars() {
                let glyph = font.scaled_glyph(c);
                let advance = font.h_advance(glyph.id);
                let positioned = glyph.id.with_scale_and_position(font.scale(), (x, y));
                if let Some(outline) = font.outline_glyph(positioned) {
                    let bounds = outline.px_bounds();
                    outline.draw(|gx, gy, coverage| {
                        let (px, py) =
                            (bounds.min.x as i32 + gx as i32, bounds.min.y as i32 + gy as i32);
                        if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                            let mix =
                                |i: usize| BACKGROUND[i] + (TEXT[i] - BACKGROUND[i]) * coverage;
                            buffer[py as usize * w + px as usize] = pack([mix(0), mix(1), mix(2)]);
                        }
                    });
                }
                x += advance;
            }
            y += font.height() + font.line_gap();
        }
        let _ = buffer.present();
    }
}

fn pack([r, g, b]: [f32; 3]) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

//! The OpenXR session of the headset renderer: EGL's OpenGL context (`egl.rs`), two swapchains
//! of two layers (one per eye, both drawn in one pass), the local space, and the frame loop's
//! calls (as rust-dos's `display/stage/xr/{mod,gl_frames}.rs`).

use super::controllers::{Controller, Controllers};
use super::egl::{Context, Egl};
use bevy::log::{info, warn};
use openxr as xr;

/// GL_SRGB8_ALPHA8 / GL_RGBA8.
const SRGB8_ALPHA8: u32 = 0x8C43;
const RGBA8: u32 = 0x8058;

pub struct Headset {
    _instance: xr::Instance,
    controllers: Option<Controllers>,
    session: xr::Session<Egl>,
    waiter: xr::FrameWaiter,
    stream: xr::FrameStream<Egl>,
    space: xr::Space,
    /// Two swapchains drawn in turn (one frame's image is drawn while the last one's is still
    /// held, to be shown once the GPU is done with it; a swapchain holds one at a time), each
    /// with its images: 2D array textures of two layers.
    swapchains: Vec<(xr::Swapchain<Egl>, Vec<glow::Texture>)>,
    /// The swapchain drawn next.
    next: usize,
    /// The size of an eye's image.
    pub size: (u32, u32),
    /// The images encode linear light as sRGB themselves.
    pub srgb: bool,
    blend: xr::EnvironmentBlendMode,
    running: bool,
    events: xr::EventDataBuffer,
}

/// A frame to draw: when it is shown, and the eyes' and the controllers' poses then (local
/// space).
pub struct Frame {
    pub time: xr::Time,
    pub views: Vec<xr::View>,
    pub controllers: [Controller; 2],
}

fn err(what: &'static str) -> impl Fn(xr::sys::Result) -> String {
    move |e| format!("{what}: {e}")
}

impl Headset {
    /// The session on `context` (current on this thread).
    pub fn new(context: &Context) -> Result<Self, String> {
        // SAFETY: loads the OpenXR loader library (beside the executable, see build.rs).
        let entry = unsafe { xr::Entry::load() }.map_err(|e| format!("no OpenXR loader: {e}"))?;
        let offered = entry.enumerate_extensions().map_err(err("the runtime's extensions"))?;
        if !offered.mndx_egl_enable || !offered.khr_opengl_enable {
            return Err("the OpenXR runtime has no OpenGL through EGL (XR_MNDX_egl_enable)".into());
        }
        let mut exts = xr::ExtensionSet::default();
        exts.mndx_egl_enable = true;
        exts.khr_opengl_enable = true;
        let instance = entry
            .create_instance(
                &xr::ApplicationInfo {
                    application_name: "Stealth Mission 89",
                    application_version: 1,
                    engine_name: "sm89-gl",
                    engine_version: 1,
                    api_version: xr::Version::new(1, 0, 0),
                },
                &exts,
                &[],
            )
            .map_err(err("the OpenXR instance"))?;
        let system =
            instance.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY).map_err(err("the headset"))?;
        // (The runtime wants to be asked before a session is made.)
        let _ = <Egl as xr::Graphics>::requirements(&instance, system);
        let blend = instance
            .enumerate_environment_blend_modes(system, xr::ViewConfigurationType::PRIMARY_STEREO)
            .map_err(err("the blend modes"))?
            .first()
            .copied()
            .unwrap_or(xr::EnvironmentBlendMode::OPAQUE);
        let views = instance
            .enumerate_view_configuration_views(system, xr::ViewConfigurationType::PRIMARY_STEREO)
            .map_err(err("the eyes' views"))?;
        let size = (views[0].recommended_image_rect_width, views[0].recommended_image_rect_height);
        let binding = context.binding()?;
        // SAFETY: the binding is the context current on this thread.
        let (session, waiter, stream) = unsafe { instance.create_session::<Egl>(system, &binding) }
            .map_err(err("the session"))?;
        let space = session
            .create_reference_space(xr::ReferenceSpaceType::LOCAL, xr::Posef::IDENTITY)
            .map_err(err("the local space"))?;
        let formats = session.enumerate_swapchain_formats().map_err(err("the formats"))?;
        let (format, srgb) = if formats.contains(&SRGB8_ALPHA8) {
            (SRGB8_ALPHA8, true)
        } else if formats.contains(&RGBA8) {
            (RGBA8, false)
        } else {
            return Err(format!("the headset takes no RGBA8 format ({formats:x?})"));
        };
        let mut swapchains = Vec::new();
        for _ in 0..2 {
            let swapchain = session
                .create_swapchain(&xr::SwapchainCreateInfo {
                    create_flags: xr::SwapchainCreateFlags::EMPTY,
                    usage_flags: xr::SwapchainUsageFlags::COLOR_ATTACHMENT,
                    format,
                    sample_count: 1,
                    width: size.0,
                    height: size.1,
                    face_count: 1,
                    array_size: 2,
                    mip_count: 1,
                })
                .map_err(err("the swapchain"))?;
            let images = swapchain
                .enumerate_images()
                .map_err(err("the swapchain's images"))?
                .into_iter()
                .map(|image| {
                    std::num::NonZeroU32::new(image)
                        .map(glow::NativeTexture)
                        .ok_or("an empty swapchain image")
                })
                .collect::<Result<Vec<_>, _>>()?;
            swapchains.push((swapchain, images));
        }
        let controllers = Controllers::new(&instance, &session)
            .inspect_err(|e| warn!("glvr: no controllers: {e}"))
            .ok();
        Ok(Headset {
            _instance: instance,
            controllers,
            session,
            waiter,
            stream,
            space,
            swapchains,
            next: 0,
            size,
            srgb,
            blend,
            running: false,
            events: xr::EventDataBuffer::new(),
        })
    }

    /// Handles the runtime's events: false once the session is over.
    pub fn poll(&mut self) -> Result<bool, String> {
        while let Some(event) = self
            .session
            .instance()
            .poll_event(&mut self.events)
            .map_err(err("the runtime's events"))?
        {
            match event {
                xr::Event::SessionStateChanged(e) => {
                    info!("glvr: XR state {:?}", e.state());
                    match e.state() {
                        xr::SessionState::READY => {
                            self.session
                                .begin(xr::ViewConfigurationType::PRIMARY_STEREO)
                                .map_err(err("beginning the session"))?;
                            self.running = true;
                        }
                        xr::SessionState::STOPPING => {
                            self.session.end().map_err(err("ending the session"))?;
                            self.running = false;
                        }
                        xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => {
                            return Ok(false);
                        }
                        _ => {}
                    }
                }
                xr::Event::InstanceLossPending(_) => return Ok(false),
                _ => {}
            }
        }
        Ok(true)
    }

    pub fn running(&self) -> bool {
        self.running
    }

    /// Waits for the next frame and begins it: the eyes to draw, or None for a frame not shown
    /// (ended at once).
    pub fn begin(&mut self) -> Result<Option<Frame>, String> {
        let state = self.waiter.wait().map_err(err("waiting for the frame"))?;
        self.stream.begin().map_err(err("beginning the frame"))?;
        if !state.should_render {
            self.stream
                .end(state.predicted_display_time, self.blend, &[])
                .map_err(err("ending the frame"))?;
            return Ok(None);
        }
        // The image drawn now is shown a frame later (`present`): the eyes and the hands
        // where they will be then.
        let time = state.predicted_display_time;
        let shown = xr::Time::from_nanos(
            time.as_nanos() + state.predicted_display_period.as_nanos().max(0),
        );
        let (_, views) = self
            .session
            .locate_views(xr::ViewConfigurationType::PRIMARY_STEREO, shown, &self.space)
            .map_err(err("the eyes' poses"))?;
        let controllers = self
            .controllers
            .as_ref()
            .map_or([Controller::default(); 2], |c| c.read(&self.session, &self.space, shown));
        Ok(Some(Frame { time, views, controllers }))
    }

    /// The image to draw the frame into, waited for: its swapchain (for `present`) and texture.
    pub fn acquire(&mut self) -> Result<(usize, glow::Texture), String> {
        let k = self.next;
        self.next = (k + 1) % self.swapchains.len();
        let (swapchain, images) = &mut self.swapchains[k];
        let image = swapchain.acquire_image().map_err(err("acquiring the image"))?;
        swapchain.wait_image(xr::Duration::INFINITE).map_err(err("waiting for the image"))?;
        let texture = *images.get(image as usize).ok_or("the headset's image")?;
        Ok((k, texture))
    }

    /// A short vibration of `amount` (0..1) on hand `i`.
    pub fn pulse(&self, i: usize, amount: f32) {
        if let Some(c) = &self.controllers {
            c.pulse(&self.session, i, amount);
        }
    }

    /// Ends the frame with nothing drawn (no picture yet).
    pub fn end_empty(&mut self, frame: &Frame) -> Result<(), String> {
        self.stream.end(frame.time, self.blend, &[]).map_err(err("ending the frame"))
    }

    /// Releases the image of swapchain `chain` (drawn last frame, done on the GPU) and ends
    /// `frame` with it, shown with the eyes' poses it was drawn with (`views`).
    pub fn present(
        &mut self,
        frame: &Frame,
        chain: usize,
        views: &[xr::View],
    ) -> Result<(), String> {
        let swapchain = &mut self.swapchains[chain].0;
        swapchain.release_image().map_err(err("releasing the image"))?;
        let swapchain = &self.swapchains[chain].0;
        let rect = xr::Rect2Di {
            offset: xr::Offset2Di { x: 0, y: 0 },
            extent: xr::Extent2Di { width: self.size.0 as i32, height: self.size.1 as i32 },
        };
        let views: Vec<_> = views
            .iter()
            .enumerate()
            .map(|(i, view)| {
                xr::CompositionLayerProjectionView::new().pose(view.pose).fov(view.fov).sub_image(
                    xr::SwapchainSubImage::new()
                        .swapchain(swapchain)
                        .image_array_index(i as u32)
                        .image_rect(rect),
                )
            })
            .collect();
        let layer = xr::CompositionLayerProjection::new().space(&self.space).views(&views);
        self.stream.end(frame.time, self.blend, &[&layer]).map_err(err("ending the frame"))
    }
}

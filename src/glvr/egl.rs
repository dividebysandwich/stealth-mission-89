//! The OpenGL context of the headset renderer: desktop OpenGL through EGL, with no window
//! (surfaceless), and the OpenXR binding for it (XR_MNDX_egl_enable), which the openxr crate
//! has none of (as rust-dos's `display/stage/xr/egl.rs`). On the Steam Frame OpenGL is Zink,
//! Vulkan in this process, which Valve's density map layers foveate (`foveation.rs`).

use khronos_egl as egl;
use openxr as xr;
use openxr::sys;
use openxr::sys::Handle as _;
use std::ffi::c_void;
use std::ptr;

/// EGL_PLATFORM_SURFACELESS_MESA: a display without a window system.
const PLATFORM_SURFACELESS: egl::Enum = 0x31DD;

/// The context, current on the thread that made it.
pub struct Context {
    pub egl: egl::DynamicInstance<egl::EGL1_5>,
    pub display: egl::Display,
    pub config: egl::Config,
    pub context: egl::Context,
    /// libEGL for eglGetProcAddress, kept loaded while the session uses it.
    library: libloading::Library,
}

impl Context {
    /// A desktop OpenGL 4.5 (or later) core context without a surface, made current.
    pub fn new() -> Result<Self, String> {
        // SAFETY: libEGL's functions with EGL's signatures.
        let egl = unsafe { egl::DynamicInstance::<egl::EGL1_5>::load_required() }
            .map_err(|e| format!("EGL can't be loaded: {e}"))?;
        // SAFETY: the surfaceless platform takes no native display.
        let display = unsafe {
            egl.get_platform_display(PLATFORM_SURFACELESS, ptr::null_mut(), &[egl::ATTRIB_NONE])
        }
        .map_err(|e| format!("no surfaceless EGL display: {e}"))?;
        egl.initialize(display).map_err(|e| format!("EGL: {e}"))?;
        egl.bind_api(egl::OPENGL_API).map_err(|e| format!("EGL has no desktop OpenGL: {e}"))?;
        let attributes = [
            egl::RENDERABLE_TYPE,
            egl::OPENGL_BIT,
            egl::SURFACE_TYPE,
            0,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::NONE,
        ];
        let config = egl
            .choose_first_config(display, &attributes)
            .map_err(|e| format!("EGL: {e}"))?
            .ok_or("EGL has no OpenGL configuration")?;
        let mut last = String::new();
        for (major, minor) in [(4, 6), (4, 5)] {
            let attributes = [
                egl::CONTEXT_MAJOR_VERSION,
                major,
                egl::CONTEXT_MINOR_VERSION,
                minor,
                egl::CONTEXT_OPENGL_PROFILE_MASK,
                egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
                egl::NONE,
            ];
            match egl.create_context(display, config, None, &attributes) {
                Ok(context) => {
                    egl.make_current(display, None, None, Some(context))
                        .map_err(|e| format!("EGL can't make the context current: {e}"))?;
                    // SAFETY: the library EGL was just loaded from.
                    let library = unsafe { libloading::Library::new("libEGL.so.1") }
                        .map_err(|e| format!("EGL can't be loaded: {e}"))?;
                    return Ok(Context { egl, display, config, context, library });
                }
                Err(e) => last = format!("OpenGL {major}.{minor}: {e}"),
            }
        }
        Err(format!("EGL can't make an OpenGL 4.5 core context ({last})"))
    }

    /// OpenGL's functions, for glow.
    pub fn gl(&self) -> glow::Context {
        // SAFETY: the context is current on this thread.
        unsafe {
            glow::Context::from_loader_function(|name| {
                self.egl.get_proc_address(name).map_or(ptr::null(), |f| f as *const c_void)
            })
        }
    }

    /// The session's binding to this context.
    pub fn binding(&self) -> Result<Handles, String> {
        // SAFETY: eglGetProcAddress with the signature XR_MNDX_egl_enable gives it.
        let get_proc_address = unsafe {
            *self
                .library
                .get::<sys::platform::EglGetProcAddressMNDX>(b"eglGetProcAddress\0")
                .map_err(|e| e.to_string())?
        };
        Ok(Handles {
            get_proc_address,
            display: self.display.as_ptr(),
            config: self.config.as_ptr(),
            context: self.context.as_ptr(),
        })
    }
}

/// The EGL binding, for `xr::Session<Egl>`.
pub enum Egl {}

/// The context's EGL handles.
pub struct Handles {
    get_proc_address: sys::platform::EglGetProcAddressMNDX,
    display: *mut c_void,
    config: *mut c_void,
    context: *mut c_void,
}

impl xr::Graphics for Egl {
    type Requirements = xr::opengl::Requirements;
    type SessionCreateInfo = Handles;
    type Format = u32;
    type SwapchainImage = u32;

    fn raise_format(x: i64) -> u32 {
        x as _
    }

    fn lower_format(x: u32) -> i64 {
        x.into()
    }

    /// OpenGL's, which XR_MNDX_egl_enable asks for too.
    fn requirements(
        instance: &xr::Instance,
        system: xr::SystemId,
    ) -> xr::Result<Self::Requirements> {
        <xr::OpenGL as xr::Graphics>::requirements(instance, system)
    }

    unsafe fn create_session(
        instance: &xr::Instance,
        system: xr::SystemId,
        info: &Handles,
    ) -> xr::Result<sys::Session> {
        let binding = sys::GraphicsBindingEGLMNDX {
            ty: sys::GraphicsBindingEGLMNDX::TYPE,
            next: ptr::null(),
            get_proc_address: Some(info.get_proc_address),
            display: info.display,
            config: info.config,
            context: info.context,
        };
        let info = sys::SessionCreateInfo {
            ty: sys::SessionCreateInfo::TYPE,
            next: &binding as *const _ as *const _,
            create_flags: Default::default(),
            system_id: system,
        };
        let mut out = sys::Session::NULL;
        // SAFETY: the binding is the context current on this thread.
        let result = unsafe { (instance.fp().create_session)(instance.as_raw(), &info, &mut out) };
        if result.into_raw() < 0 { Err(result) } else { Ok(out) }
    }

    fn enumerate_swapchain_images(swapchain: &xr::Swapchain<Self>) -> xr::Result<Vec<u32>> {
        let enumerate =
            |capacity: u32, count: &mut u32, images: *mut sys::SwapchainImageOpenGLKHR| {
                // SAFETY: `images` has room for `capacity` of them.
                unsafe {
                    (swapchain.instance().fp().enumerate_swapchain_images)(
                        swapchain.as_raw(),
                        capacity,
                        count,
                        images.cast(),
                    )
                }
            };
        let empty = sys::SwapchainImageOpenGLKHR {
            ty: sys::SwapchainImageOpenGLKHR::TYPE,
            next: ptr::null_mut(),
            image: 0,
        };
        let mut count = 0;
        let result = enumerate(0, &mut count, ptr::null_mut());
        if result.into_raw() < 0 {
            return Err(result);
        }
        let mut images = vec![empty; count as usize];
        let result = enumerate(count, &mut count, images.as_mut_ptr());
        if result.into_raw() < 0 {
            return Err(result);
        }
        images.truncate(count as usize);
        Ok(images.into_iter().map(|i| i.image).collect())
    }
}

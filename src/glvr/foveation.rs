//! Foveated rendering on the Steam Frame through Valve's density map layers, as rust-dos asks
//! for them (`display/stage/foveation.rs`): their OpenXR half (implicit, always loaded) learns
//! the size of the headset's images and the eyes' gaze, their Vulkan half (explicit, asked for
//! here) lays a density map over the render passes drawing into those images. OpenGL is Zink
//! on the Frame, so the layers reach it when Zink makes its Vulkan instance: the environment is
//! set before EGL starts.

/// Valve's Vulkan layers that foveate render passes into the headset's images.
pub const FDM_LAYERS: &str = "VK_LAYER_VALVE_rpo:VK_LAYER_VALVE_fdm_injection";

/// What Steam turns the layers off with.
pub const FDM_DISABLE: &str = "DISABLE_VULKAN_FDM_INJECTION_LAYER";

/// The environment for the layers to foveate at `level` (low, medium or high; None: nothing),
/// keeping what is set already; `extra` ("K=V;K=V") replaces or adds variables.
pub fn fdm_env(
    level: Option<&str>,
    existing: impl Fn(&str) -> Option<String>,
    extra: &str,
) -> Vec<(String, String)> {
    let Some(level) = level else { return Vec::new() };
    if existing(FDM_DISABLE).is_some_and(|v| !v.is_empty() && v != "0") {
        return Vec::new();
    }
    let mut env: Vec<(String, String)> = Vec::new();
    match existing("VK_INSTANCE_LAYERS").filter(|l| !l.is_empty()) {
        Some(asked) if asked.contains("VK_LAYER_VALVE_fdm_injection") => {}
        Some(asked) => env.push(("VK_INSTANCE_LAYERS".into(), format!("{asked}:{FDM_LAYERS}"))),
        None => env.push(("VK_INSTANCE_LAYERS".into(), FDM_LAYERS.into())),
    }
    // `yflip`: OpenGL's images go bottom up, the gaze's vertical offset the other way.
    for (key, value) in [("FDM_DEBUG", "enable,yflip"), ("FOVE_LEVEL", level)] {
        if existing(key).is_none() {
            env.push((key.into(), value.into()));
        }
    }
    for pair in extra.split(';').filter(|p| !p.trim().is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = key.trim().to_string();
        env.retain(|(k, _)| *k != key);
        env.push((key, value.trim().to_string()));
    }
    env
}

/// Asks Valve's layers for foveation: `SM89_GL_FOVEATION` = off, low, medium (default) or
/// high; `SM89_FDM_ENV="K=V;K=V"` sets others. Returns what was set, for the log.
pub fn enable() -> String {
    let level = match std::env::var("SM89_GL_FOVEATION").as_deref() {
        Ok("off") => None,
        Ok(l @ ("low" | "medium" | "high")) => Some(l.to_string()),
        _ => Some("medium".to_string()),
    };
    let extra = std::env::var("SM89_FDM_ENV").unwrap_or_default();
    let env = fdm_env(level.as_deref(), |key| std::env::var(key).ok(), &extra);
    for (key, value) in &env {
        // SAFETY: called first in `main`, before any other thread starts.
        unsafe { std::env::set_var(key, value) };
    }
    let set: Vec<String> = env.iter().map(|(k, v)| format!("{k}={v}")).collect();
    match level {
        None => "foveation off".into(),
        Some(level) if set.is_empty() => format!("foveation {level}: all set already"),
        Some(level) => format!("foveation {level}: set {}", set.join(" ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_what_steam_would_and_keeps_what_is_set() {
        let none = |_: &str| None;
        assert!(fdm_env(None, none, "").is_empty());
        let pairs = |list: &[(&str, &str)]| {
            list.iter().map(|&(k, v)| (k.to_string(), v.to_string())).collect::<Vec<_>>()
        };
        assert_eq!(
            fdm_env(Some("medium"), none, ""),
            pairs(&[
                ("VK_INSTANCE_LAYERS", FDM_LAYERS),
                ("FDM_DEBUG", "enable,yflip"),
                ("FOVE_LEVEL", "medium")
            ])
        );
        let some = |key: &str| match key {
            "VK_INSTANCE_LAYERS" => Some("VK_LAYER_MESA_overlay".to_string()),
            "FOVE_LEVEL" => Some("low".to_string()),
            _ => None,
        };
        assert_eq!(
            fdm_env(Some("high"), some, ""),
            pairs(&[
                ("VK_INSTANCE_LAYERS", &format!("VK_LAYER_MESA_overlay:{FDM_LAYERS}")),
                ("FDM_DEBUG", "enable,yflip")
            ])
        );
        let off = |key: &str| (key == FDM_DISABLE).then(|| "1".to_string());
        assert!(fdm_env(Some("low"), off, "").is_empty());
        assert_eq!(
            fdm_env(Some("low"), none, "FDM_DEBUG=enable,debug; X = 1")[1..],
            pairs(&[("FOVE_LEVEL", "low"), ("FDM_DEBUG", "enable,debug"), ("X", "1")])[..]
        );
    }
}

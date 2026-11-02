//! The player's files, all in one folder (`root`) with the same layout on every system, so that
//! Steam Cloud can sync them with one rule per subfolder (docs/files.md):
//!
//! * Windows: `Documents\My Games\Stealth Mission 89` (the Documents folder wherever it was
//!   moved, OneDrive included);
//! * macOS: `~/Library/Application Support/stealth-mission-89`;
//! * Linux: `$XDG_DATA_HOME/stealth-mission-89`, by default `~/.local/share/stealth-mission-89`
//!   (where the AppImage also copies the game's assets).
//!
//! Keys are paths in it with `/` as separator: `config/` this computer's settings (`prefs.json`,
//! `input.json`), `pilots/`, `campaigns/` and `saves/` the player's progress, e.g.
//! `campaigns/ce-1.json`. The browser build keeps them in localStorage.

use anyhow::Result;

/// The contents of `key`, None if there is none.
pub fn read(key: &str) -> Result<Option<String>> {
    imp::read(key)
}

/// Stores `text` under `key` (a crash mid-write keeps the previous contents).
pub fn write(key: &str, text: &str) -> Result<()> {
    imp::write(key, text)
}

/// Removes `key`; one that is already gone is no error.
pub fn remove(key: &str) -> Result<()> {
    imp::remove(key)
}

/// The names in directory `dir` (`pilots`), each with when it was last written (seconds since
/// the Unix epoch, 0 if unknown).
pub fn list(dir: &str) -> Vec<(String, f64)> {
    imp::list(dir)
}

#[cfg(not(target_arch = "wasm32"))]
pub use imp::{path, root};

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::path::PathBuf;

    use anyhow::{Context, Result};

    /// The player's folder (made when first written to), found once; None without a home
    /// directory. Windows: in `Documents\My Games`, as many games keep theirs (in the data
    /// directory without a Documents folder); elsewhere the data directory: macOS asks the player
    /// before a game reads Documents, Linux has no such convention. `SM89_HOME` replaces it
    /// (test runs that must not touch the player's settings).
    pub fn root() -> Option<PathBuf> {
        static ROOT: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
        ROOT.get_or_init(|| {
            if let Some(home) = std::env::var_os("SM89_HOME").filter(|h| !h.is_empty()) {
                return Some(PathBuf::from(home));
            }
            if cfg!(windows)
                && let Some(d) = dirs::document_dir()
            {
                let root = d.join("My Games").join("Stealth Mission 89");
                migrate(&root);
                return Some(root);
            }
            let root = dirs::data_dir()?.join("stealth-mission-89");
            migrate(&root);
            Some(root)
        })
        .clone()
    }

    /// Up to 0.7 the settings were in the config directory and the pilots and campaigns in the
    /// data directory (each under `stealth-mission-89/`, both `%APPDATA%` on Windows): copied
    /// into the folder on the first start without its `config/` (the old files stay).
    fn migrate(root: &std::path::Path) {
        let config = root.join("config");
        if config.exists() {
            return;
        }
        let old = |d: Option<PathBuf>| d.map(|d| d.join("stealth-mission-89"));
        let copy = |from: PathBuf, to: PathBuf| {
            if from.is_file() && !to.exists() {
                let _ = std::fs::create_dir_all(to.parent().unwrap_or(root));
                let _ = std::fs::copy(from, to);
            }
        };
        if let Some(old) = old(dirs::config_dir()) {
            for f in ["prefs.json", "input.json"] {
                copy(old.join(f), config.join(f));
            }
        }
        if let Some(old) = old(dirs::data_dir()).filter(|d| d != root) {
            for dir in ["pilots", "campaigns"] {
                let Ok(files) = std::fs::read_dir(old.join(dir)) else { continue };
                for f in files.flatten() {
                    copy(f.path(), root.join(dir).join(f.file_name()));
                }
            }
        }
        // Once only, also with nothing to copy.
        let _ = std::fs::create_dir_all(&config);
    }

    /// Where `key` is on disk; None without a home directory.
    pub fn path(key: &str) -> Option<PathBuf> {
        Some(key.split('/').filter(|s| !s.is_empty()).fold(root()?, |p, s| p.join(s)))
    }

    fn need_path(key: &str) -> Result<PathBuf> {
        path(key).with_context(|| format!("{key}: no home directory"))
    }

    pub fn read(key: &str) -> Result<Option<String>> {
        let path = need_path(key)?;
        match std::fs::read_to_string(&path) {
            Ok(t) => Ok(Some(t)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| path.display().to_string()),
        }
    }

    pub fn write(key: &str, text: &str) -> Result<()> {
        let path = need_path(key)?;
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d)?;
        }
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        std::fs::write(&tmp, text).with_context(|| path.display().to_string())?;
        std::fs::rename(&tmp, &path).with_context(|| path.display().to_string())
    }

    pub fn remove(key: &str) -> Result<()> {
        let path = need_path(key)?;
        match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                Err(e).with_context(|| path.display().to_string())
            }
            _ => Ok(()),
        }
    }

    pub fn list(dir: &str) -> Vec<(String, f64)> {
        let Some(Ok(entries)) = path(dir).map(std::fs::read_dir) else { return Vec::new() };
        entries
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
            .filter_map(|e| {
                let name = e.file_name().into_string().ok()?;
                let modified = e
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(0.0, |d| d.as_secs_f64());
                Some((name, modified))
            })
            .collect()
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use anyhow::{Result, anyhow};
    use web_sys::Storage;

    /// The contents are under `sm89/<key>`, when they were written under `sm89-time/<key>`.
    const DATA: &str = "sm89/";
    const TIME: &str = "sm89-time/";

    fn storage() -> Result<Storage> {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or_else(|| anyhow!("the browser keeps no local storage"))
    }

    fn js(e: web_sys::wasm_bindgen::JsValue) -> anyhow::Error {
        anyhow!("local storage: {e:?}")
    }

    pub fn read(key: &str) -> Result<Option<String>> {
        storage()?.get_item(&format!("{DATA}{key}")).map_err(js)
    }

    pub fn write(key: &str, text: &str) -> Result<()> {
        let s = storage()?;
        // Full (the quota is a few MB): the previous contents stay.
        s.set_item(&format!("{DATA}{key}"), text).map_err(js)?;
        let _ = s.set_item(&format!("{TIME}{key}"), &(js_sys::Date::now() / 1000.0).to_string());
        Ok(())
    }

    pub fn remove(key: &str) -> Result<()> {
        let s = storage()?;
        s.remove_item(&format!("{DATA}{key}")).map_err(js)?;
        s.remove_item(&format!("{TIME}{key}")).map_err(js)
    }

    pub fn list(dir: &str) -> Vec<(String, f64)> {
        let Ok(s) = storage() else { return Vec::new() };
        let prefix = format!("{DATA}{}/", dir.trim_end_matches('/'));
        let n = s.length().unwrap_or(0);
        (0..n)
            .filter_map(|i| s.key(i).ok().flatten())
            .filter_map(|k| {
                let name = k.strip_prefix(&prefix)?;
                if name.contains('/') {
                    return None;
                }
                let key = &k[DATA.len()..];
                let time = s
                    .get_item(&format!("{TIME}{key}"))
                    .ok()
                    .flatten()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(0.0);
                Some((name.to_string(), time))
            })
            .collect()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::{path, root};

    /// The keys are paths in the player's folder, `My Games` on Windows.
    #[test]
    fn keys_are_paths_in_the_folder() {
        let Some(root) = root() else { return };
        assert_eq!(path("saves/ce-1.json"), Some(root.join("saves").join("ce-1.json")));
        assert_eq!(path("pilots"), Some(root.join("pilots")));
        let end = if cfg!(windows) { "My Games/Stealth Mission 89" } else { "stealth-mission-89" };
        assert!(root.ends_with(end), "{}", root.display());
    }
}

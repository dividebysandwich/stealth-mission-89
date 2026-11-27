//! The game's log file, written on every start (from a terminal too, which still shows the
//! log), where the system keeps programs' logs:
//!
//! - Windows: `%LOCALAPPDATA%\Stealth Mission 89\logs` (not Documents, which OneDrive may sync)
//! - macOS: `~/Library/Logs/Stealth Mission 89`
//! - Linux, SteamOS: `$XDG_STATE_HOME/stealth-mission-89`, by default `~/.local/state/...`
//! - with `SM89_HOME` (test runs): `$SM89_HOME/logs`
//!
//! `stealth-mission-89.log` is the last start's, `stealth-mission-89-vr.log` the last VR start's
//! ("Fly in VR" relaunches with `--vr`, so the menu's log is kept); the starts before are kept
//! as `.1.log` to `.4.log`. None in the browser and on Android (logcat has the log).
//!
//! Started from a launcher (Windows Explorer or Steam, which give the game no console; the
//! AppImage's AppRun without a terminal, through `SM89_LOG_STDERR=1`), the standard output and
//! error go to the file: the log lines, and what drivers and libraries print there (Mesa's
//! "DEVICE LOST"). Otherwise the log lines are written to the file besides the terminal.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicI32;
use std::sync::{Mutex, OnceLock};

use bevy::log::{BoxedFmtLayer, BoxedLayer};
use bevy::log::tracing_subscriber::fmt;

/// Earlier starts' logs kept (`.1.log`, the last, and on).
const KEEP: u32 = 4;

/// The open log file's descriptor (-1: none), for the fatal signal handler's last line
/// (`crash_log`), which may not take locks.
pub static LOG_FD: AtomicI32 = AtomicI32::new(-1);

static PATH: OnceLock<PathBuf> = OnceLock::new();
/// The opened file, until `layer` takes it; None after a redirect.
static FILE: Mutex<Option<File>> = Mutex::new(None);
/// The standard output and error go to the file.
static REDIRECTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The folder the logs are written to; None without a home folder (and on Android).
pub fn dir() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("SM89_HOME").filter(|h| !h.is_empty()) {
        return Some(PathBuf::from(home).join("logs"));
    }
    #[cfg(windows)]
    return Some(dirs::data_local_dir()?.join("Stealth Mission 89").join("logs"));
    #[cfg(target_os = "macos")]
    return Some(dirs::home_dir()?.join("Library").join("Logs").join("Stealth Mission 89"));
    #[cfg(target_os = "android")]
    return None;
    #[cfg(not(any(windows, target_os = "macos", target_os = "android")))]
    {
        let state = dirs::state_dir().or_else(|| dirs::home_dir().map(|h| h.join(".local/state")));
        Some(state?.join("stealth-mission-89"))
    }
}

/// This start's log file, once opened.
pub fn path() -> Option<&'static Path> {
    PATH.get().map(PathBuf::as_path)
}

/// The log's name without `.log`: a VR start's has its own.
fn stem() -> &'static str {
    if std::env::args().any(|a| a == "--vr" || a == "--gl-vr") {
        "stealth-mission-89-vr"
    } else {
        "stealth-mission-89"
    }
}

/// Moves the earlier logs one number up (the oldest goes) and creates this start's. Another
/// instance still writing the log (a second client on this computer) keeps writing to its file
/// under the new name; where that can't be renamed, this one writes `<stem>-<pid>.log`.
fn open(dir: &Path, stem: &str) -> std::io::Result<(PathBuf, File)> {
    std::fs::create_dir_all(dir)?;
    let numbered = |n: u32| dir.join(format!("{stem}.{n}.log"));
    let current = dir.join(format!("{stem}.log"));
    for n in (1..KEEP).rev() {
        let from = numbered(n);
        if from.exists() {
            let _ = std::fs::rename(&from, numbered(n + 1));
        }
    }
    let path = if !current.exists() || std::fs::rename(&current, numbered(1)).is_ok() {
        current
    } else {
        dir.join(format!("{stem}-{}.log", std::process::id()))
    };
    let file = File::create(&path)?;
    Ok((path, file))
}

/// Opens this start's log file (a line on stderr if it can't), with the standard output and
/// error sent to it when `redirect` (started from a launcher). Call first, before any thread.
pub fn start(redirect: bool) {
    let Some(dir) = dir() else { return };
    let (path, file) = match open(&dir, stem()) {
        Ok(opened) => opened,
        Err(e) => {
            eprintln!("log file in {}: {e}", dir.display());
            return;
        }
    };
    #[cfg(unix)]
    if let Ok(copy) = file.try_clone() {
        use std::os::fd::IntoRawFd;
        LOG_FD.store(copy.into_raw_fd(), std::sync::atomic::Ordering::Relaxed);
    }
    let _ = PATH.set(path);
    if redirect && redirect_std(file.try_clone()) {
        REDIRECTED.store(true, std::sync::atomic::Ordering::Relaxed);
    } else if let Ok(mut f) = FILE.lock() {
        *f = Some(file);
    }
}

/// Whether the launcher asks for the standard output and error in the log (AppRun).
pub fn asked_to_redirect() -> bool {
    std::env::var_os("SM89_LOG_STDERR").is_some_and(|v| v == "1")
}

/// Points the standard output and error at the log file; false if that failed.
fn redirect_std(file: std::io::Result<File>) -> bool {
    let Ok(file) = file else { return false };
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        // SAFETY: dup2 onto the standard descriptors of an open file; `file` is then closed,
        // the duplicates stay open for the life of the process.
        unsafe { libc::dup2(file.as_raw_fd(), 1) >= 0 && libc::dup2(file.as_raw_fd(), 2) >= 0 }
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::IntoRawHandle;
        use windows_sys::Win32::System::Console::{
            STD_ERROR_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle,
        };
        // Rust's stdout and stderr look the handles up on every write.
        let handle = file.into_raw_handle();
        // SAFETY: `handle` is an open file handle that is never closed.
        unsafe {
            SetStdHandle(STD_OUTPUT_HANDLE, handle) != 0
                && SetStdHandle(STD_ERROR_HANDLE, handle) != 0
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        drop(file);
        false
    }
}

/// `LogPlugin::custom_layer`'s part: the log lines written to the file (without colour codes),
/// unless the standard error already goes there.
pub fn layer() -> Option<BoxedLayer> {
    let file = FILE.lock().ok()?.take()?;
    Some(Box::new(fmt::layer().with_ansi(false).with_writer(Mutex::new(file))))
}

/// `LogPlugin::fmt_layer`: Bevy's own (stderr), but without colour codes once stderr is the
/// file.
pub fn console_layer(_app: &mut bevy::app::App) -> Option<BoxedFmtLayer> {
    REDIRECTED
        .load(std::sync::atomic::Ordering::Relaxed)
        .then(|| Box::new(fmt::layer().with_ansi(false).with_writer(std::io::stderr)) as _)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each start moves the earlier logs one number up and keeps `KEEP` of them.
    #[test]
    fn starts_keep_the_earlier_logs() {
        let dir = std::env::temp_dir().join(format!("sm89-log-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for run in 0..KEEP + 2 {
            let (path, _) = open(&dir, "game").unwrap();
            assert_eq!(path, dir.join("game.log"));
            std::fs::write(&path, run.to_string()).unwrap();
        }
        let read = |name: &str| std::fs::read_to_string(dir.join(name)).ok();
        assert_eq!(read("game.log").as_deref(), Some("5"));
        assert_eq!(read("game.1.log").as_deref(), Some("4"));
        assert_eq!(read(&format!("game.{KEEP}.log")).as_deref(), Some("1"));
        assert_eq!(read(&format!("game.{}.log", KEEP + 1)), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

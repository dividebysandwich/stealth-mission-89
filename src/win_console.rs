//! Windows: the console window of a game started from Explorer, a Start menu shortcut or Steam.
//! The game is a console program, so Windows gives it a console of its own then, which only it
//! is attached to (or none at all on Windows 11 24H2 and later, through the manifest's
//! consoleAllocationPolicy, packaging/windows/stealth-mission-89.manifest). It closes that
//! console, and the standard output and error go to the log file (`log_file`). Started from a
//! terminal, it shares the terminal's console and keeps writing to it.

use windows_sys::Win32::System::Console::{FreeConsole, GetConsoleProcessList};

/// Call first in `main`, before the log is set up: whether the game has no terminal (started
/// from Explorer or Steam).
pub fn leave_explorer_console() -> bool {
    let mut pids = [0u32; 2];
    // SAFETY: the list has room for the two entries it is given.
    let attached = unsafe { GetConsoleProcessList(pids.as_mut_ptr(), pids.len() as u32) };
    if attached == 1 {
        // SAFETY: nothing holds on to the console's handles yet.
        unsafe { FreeConsole() };
    }
    attached <= 1
}

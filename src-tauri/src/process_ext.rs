//! Child process helpers. On Windows, `CREATE_NO_WINDOW` avoids flashing cmd.exe consoles.
use std::ffi::OsStr;
use std::process::Command;

/// Build a [`Command`] for running an external program (FFmpeg, `where.exe`, etc.).
pub fn command(program: impl AsRef<OsStr>) -> Command {
    let mut c = Command::new(program);
    hide_console(&mut c);
    c
}

fn hide_console(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
}

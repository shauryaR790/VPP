//! Live re-run on save  -  instant feedback without leaving the terminal.

use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::driver;
use crate::error::{VppError, VppResult};

fn file_mtime(path: &Path) -> VppResult<SystemTime> {
    std::fs::metadata(path)
        .map(|m| m.modified().unwrap_or(SystemTime::UNIX_EPOCH))
        .map_err(|e| VppError::Other {
            message: format!("cannot watch `{}`: {e}", path.display()),
        })
}

fn run_once(path: &Path) -> VppResult<()> {
    driver::run("", path)
}

/// Poll the file and re-run whenever it is saved.
pub fn watch_file(path: &Path, debounce_ms: u64) -> VppResult<()> {
    if !path.exists() {
        return Err(VppError::Other {
            message: format!("cannot watch missing file `{}`", path.display()),
        });
    }

    println!("v++ watch v0.7  -  live run on save");
    println!("  File: {}", path.display());
    println!("  Save in your editor to re-run. Ctrl+C to stop.\n");

    let mut last_mtime = file_mtime(path)?;
    run_once(path)?;

    loop {
        std::thread::sleep(Duration::from_millis(debounce_ms));
        let mtime = file_mtime(path)?;
        if mtime > last_mtime {
            last_mtime = mtime;
            println!("\n── saved  -  re-running {} ──", path.file_name().unwrap_or_default().to_string_lossy());
            if let Err(e) = run_once(path) {
                eprintln!("{e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detects_mtime_change() {
        let dir = std::env::temp_dir().join("vpp_watch_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tick.vpp");
        std::fs::write(&path, "import std.io\nfn main() { print(1) }").unwrap();
        let t0 = file_mtime(&path).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        let mut f = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
        writeln!(f, "// touch").unwrap();
        drop(f);
        let t1 = file_mtime(&path).unwrap();
        assert!(t1 > t0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

//! Compare interpreter speed  -  proves the parity path is fast enough to iterate.

use std::path::Path;
use std::time::{Duration, Instant};

use crate::driver;
use crate::error::{VppError, VppResult};

pub const DEFAULT_RUNS: u32 = 5;

/// Time repeated interpreter runs (same engine as `vpp run` / `vpp repl`).
pub fn bench_file(path: &Path, runs: u32) -> VppResult<()> {
    if !path.exists() {
        return Err(VppError::Other {
            message: format!("cannot bench missing file `{}`", path.display()),
        });
    }
    if runs == 0 {
        return Err(VppError::Other {
            message: "runs must be at least 1".to_string(),
        });
    }

    println!("v++ bench  -  interpreter path (same as run/repl/watch)");
    println!("  File: {}", path.display());
    println!("  Runs: {runs}\n");

    // Warmup  -  populates OS cache, first typecheck
    let _ = driver::run("", path);

    let mut total = Duration::ZERO;
    for i in 0..runs {
        let start = Instant::now();
        driver::run("", path)?;
        let elapsed = start.elapsed();
        total += elapsed;
        println!("  run {}: {:.2} ms", i + 1, elapsed.as_secs_f64() * 1000.0);
    }

    let avg = total / runs;
    println!(
        "\n  average: {:.2} ms  |  use `vpp watch` for live feedback, `vpp build` when ready to ship native",
        avg.as_secs_f64() * 1000.0
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn benches_hello() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("hello.vpp");
        bench_file(&path, 2).unwrap();
    }
}

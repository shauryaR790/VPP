//! Environment diagnostics for v++ tooling.

use std::path::Path;
use std::process::Command;

use crate::error::{VppError, VppResult};

pub fn run_doctor(project_root: Option<&Path>) -> VppResult<()> {
    println!("v++ doctor");
    println!("==========");
    println!();
    println!(
        "Platform: {} / {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!("v++ compiler: v{}", env!("CARGO_PKG_VERSION"));
    println!();

    check_rust();
    check_llvm();
    check_git();
    if let Some(root) = project_root {
        check_project(root)?;
    }

    println!();
    println!("Doctor checks complete.");
    Ok(())
}

fn check_rust() {
    print!("Rust toolchain ... ");
    match Command::new("rustc").arg("--version").output() {
        Ok(out) if out.status.success() => {
            println!("ok ({})", String::from_utf8_lossy(&out.stdout).trim());
        }
        _ => {
            println!("not installed (optional  -  only needed to compile v++ from source)");
        }
    }
}

fn clang_version() -> Option<String> {
    for program in ["clang", "clang.exe"] {
        if let Ok(out) = Command::new(program).arg("--version").output() {
            if out.status.success() {
                let line = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .next()
                    .unwrap_or("clang")
                    .to_string();
                return Some(line);
            }
        }
    }
    None
}

fn check_llvm() {
    print!("LLVM (clang) ... ");
    if let Some(line) = clang_version() {
        println!("ok ({line})");
        return;
    }
    println!("not installed (optional  -  only needed for `vpp build`)");
    if cfg!(target_os = "windows") {
        println!("  Install: winget install LLVM.LLVM");
    } else if cfg!(target_os = "macos") {
        println!("  Install: brew install llvm@22  (or use a release tarball)");
    } else {
        println!("  Install: apt install clang-22 lld-22  (or use a release tarball)");
    }
    println!("  Or use a release bundle with llvm included.");
}

fn check_git() {
    print!("git ... ");
    match Command::new("git").arg("--version").output() {
        Ok(out) if out.status.success() => {
            println!("ok ({})", String::from_utf8_lossy(&out.stdout).trim());
        }
        _ => {
            println!("not installed (optional  -  only needed for git-based packages)");
        }
    }
}

fn check_project(root: &Path) -> VppResult<()> {
    print!("vpp.toml ... ");
    let manifest = crate::project::load_manifest(root)?;
    println!("ok ({}, v{})", manifest.name, manifest.version);

    print!("entry `{}` ... ", manifest.entry.display());
    let entry = root.join(&manifest.entry);
    if entry.exists() {
        println!("ok");
    } else {
        println!("MISSING");
        return Err(VppError::Other {
            message: format!("entry file `{}` not found", entry.display()),
        });
    }

    if root.join("vpp.lock").exists() {
        println!("vpp.lock ... ok");
    } else if !manifest.dependencies.is_empty() {
        println!("vpp.lock ... missing (run `vpp update`)");
    }

    Ok(())
}

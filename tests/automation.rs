//! Automation API parity (interpreter vs native).

#![cfg(feature = "codegen")]

use std::path::{Path, PathBuf};
use std::process::Command;

use vpp::driver::CompileOptions;

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn run_interpreter(path: &Path) {
    let source = std::fs::read_to_string(path).unwrap();
    vpp::driver::run(&source, path).unwrap_or_else(|e| {
        panic!("interpreter failed for {}: {e}", path.display());
    });
}

fn run_native(path: &Path, exe: &Path) {
    let source = std::fs::read_to_string(path).unwrap();
    vpp::compile(
        &source,
        path,
        CompileOptions {
            output: Some(exe.to_path_buf()),
            emit_ir: None,
        },
    )
    .unwrap_or_else(|e| panic!("native compile failed for {}: {e}", path.display()));

    let output = Command::new(exe).output().expect("run native");
    assert!(
        output.status.success(),
        "native failed for {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn automation_smoke_interpreter() {
    run_interpreter(&example("automation_smoke.vpp"));
}

#[test]
fn automation_smoke_native() {
    let path = example("automation_smoke.vpp");
    let exe = std::env::temp_dir().join(format!("vpp-auto-smoke-{}.exe", std::process::id()));
    run_native(&path, &exe);
    let _ = std::fs::remove_file(&exe);
}

#[test]
fn workflow_smoke_interpreter() {
    run_interpreter(&example("workflow_smoke.vpp"));
}

// Native workflow: deferred until struct-array for-loop codegen is fixed (v2.0.1).

#[test]
fn process_exec_parity() {
    let dir = std::env::temp_dir().join(format!("vpp-auto-exec-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let (program, args) = if cfg!(windows) {
        ("cmd", r#"["/C", "echo", "vpp-exec"]"#)
    } else {
        ("echo", r#"["vpp-exec"]"#)
    };

    let source = format!(
        r#"import std.process

fn main() -> int {{
    let r = process.exec("{program}", {args}, process.default_options())
    match r {{
        Ok(out) => {{
            print(out.exit_code)
            print(out.stdout)
            return 0
        }}
        Err(e) => {{
            print(e)
            return 1
        }}
    }}
}}
"#
    );

    let file = dir.join("exec_test.vpp");
    std::fs::write(&file, &source).unwrap();
    let exe = dir.join("exec_test.exe");

    run_interpreter(&file);
    run_native(&file, &exe);

    let _ = std::fs::remove_dir_all(&dir);
}

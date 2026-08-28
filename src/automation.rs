//! Interpreter-side automation helpers (v1.0.5).

use std::cell::RefCell;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use crate::error::{VppError, VppResult};

thread_local! {
    static LAST_CMD_IO: RefCell<(String, String)> = RefCell::new((String::new(), String::new()));
}

pub(crate) fn take_last_cmd_io() -> (String, String) {
    LAST_CMD_IO.with(|cell| {
        let guard = cell.borrow_mut();
        (guard.0.clone(), guard.1.clone())
    })
}

/// Run a program with argv. Returns exit code, or -1 spawn error, -2 timeout.
pub(crate) fn command_run(
    program: &str,
    args: &[Rc<String>],
    cwd: &str,
    timeout_ms: i64,
) -> VppResult<i64> {
    let program = program.to_string();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let cwd = cwd.to_string();

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut cmd = Command::new(&program);
        cmd.args(&args);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        if !cwd.is_empty() {
            cmd.current_dir(&cwd);
        }
        let result = cmd.output();
        let _ = tx.send(result);
    });

    let output = if timeout_ms > 0 {
        match rx.recv_timeout(Duration::from_millis(timeout_ms as u64)) {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => {
                return Err(VppError::Other {
                    message: format!("command_run failed: {e}"),
                });
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                set_last_cmd_io(String::new(), String::new());
                return Ok(-2);
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(VppError::Other {
                    message: "command_run thread exited unexpectedly".to_string(),
                });
            }
        }
    } else {
        rx.recv()
            .map_err(|_| VppError::Other {
                message: "command_run thread exited unexpectedly".to_string(),
            })?
            .map_err(|e| VppError::Other {
                message: format!("command_run failed: {e}"),
            })?
    };

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    set_last_cmd_io(stdout, stderr);
    Ok(output.status.code().unwrap_or(-1) as i64)
}

pub(crate) fn set_last_cmd_io(stdout: String, stderr: String) {
    LAST_CMD_IO.with(|cell| *cell.borrow_mut() = (stdout, stderr));
}

#[derive(Debug, Clone)]
pub(crate) struct TaskSpec {
    pub name: String,
    pub program: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub timeout_ms: i64,
}

/// Run tasks concurrently. Returns 0 if all succeeded, 1 if any failed.
pub(crate) fn parallel_tasks(specs: Vec<TaskSpec>) -> VppResult<i64> {
    if specs.is_empty() {
        return Ok(0);
    }

    use std::sync::mpsc;
    use std::thread;

    let count = specs.len();
    let (tx, rx) = mpsc::channel();
    for spec in specs {
        let tx = tx.clone();
        thread::spawn(move || {
            let code = command_run(
                &spec.program,
                &spec.args.iter().map(|s| Rc::new(s.clone())).collect::<Vec<_>>(),
                &spec.cwd,
                spec.timeout_ms,
            )
            .unwrap_or(-1);
            let ok = code == 0;
            let _ = tx.send((spec.name, ok));
        });
    }
    drop(tx);

    let mut failed = false;
    for _ in 0..count {
        match rx.recv() {
            Ok((_name, ok)) if !ok => failed = true,
            Ok(_) => {}
            Err(_) => {
                failed = true;
                break;
            }
        }
    }

    Ok(if failed { 1 } else { 0 })
}

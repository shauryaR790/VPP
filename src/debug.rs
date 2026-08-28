//! Line debugger for the v++ interpreter (same engine as run/repl/watch).

use std::collections::HashSet;
use std::io::{self, BufRead, Write};
use std::path::Path;

use serde_json::{json, Value};

use crate::driver::check_path;
use crate::error::{VppError, VppResult};
use crate::interp::{Interpreter, StepMode, stmt_line};
use crate::types::{TypedProgram, TypedStmt};

/// Run an interactive CLI debugger on a `.vpp` file.
pub fn debug_file(path: &Path, breakpoints: &[u32]) -> VppResult<()> {
    let source = std::fs::read_to_string(path).map_err(|e| VppError::Other {
        message: format!("failed to read `{}`: {e}", path.display()),
    })?;
    let program = check_path(path)?;
    let mut session = DebugSession::new(source, program, breakpoints)?;
    session.run_cli()
}

/// Debug Adapter Protocol over stdio (used by VS Code).
pub fn debug_dap(path: &Path) -> VppResult<()> {
    let source = std::fs::read_to_string(path).map_err(|e| VppError::Other {
        message: format!("failed to read `{}`: {e}", path.display()),
    })?;
    let program = check_path(path)?;
    let mut session = DebugSession::new(source, program, &[])?;
    session.run_dap(path)
}

struct DebugSession {
    source: String,
    lines: Vec<String>,
    program: TypedProgram,
    interp: Interpreter,
    breakpoints: HashSet<u32>,
    step_mode: StepMode,
    paused_line: Option<u32>,
    top_ip: usize,
    finished: bool,
    dap_print_rx: Option<std::sync::mpsc::Receiver<String>>,
}

impl DebugSession {
    fn new(source: String, program: TypedProgram, breakpoints: &[u32]) -> VppResult<Self> {
        let lines: Vec<String> = source.lines().map(str::to_string).collect();
        let mut bp: HashSet<u32> = breakpoints.iter().copied().collect();
        bp.insert(1);
        let interp = Interpreter::new_debug(program.functions.clone(), source.clone(), bp.clone());
        Ok(Self {
            source,
            lines,
            program,
            interp,
            breakpoints: bp,
            step_mode: StepMode::Continue,
            paused_line: None,
            top_ip: 0,
            finished: false,
            dap_print_rx: None,
        })
    }

    fn flush_dap_output(&mut self, stdout: &mut io::Stdout) -> VppResult<()> {
        if let Some(rx) = &self.dap_print_rx {
            while let Ok(line) = rx.try_recv() {
                send_event(
                    stdout,
                    "output",
                    json!({
                        "category": "stdout",
                        "output": format!("{line}\n"),
                    }),
                )?;
            }
        }
        Ok(())
    }

    fn run_cli(&mut self) -> VppResult<()> {
        println!("v++ debugger  -  same interpreter as run/repl/watch");
        println!("  break N | b N   set breakpoint on line N");
        println!("  continue | c     run until next breakpoint");
        println!("  step | s          step one line");
        println!("  next | n          step over calls");
        println!("  locals | l        show variables in scope");
        println!("  print EXPR | p     evaluate expression");
        println!("  list | ls          show source around current line");
        println!("  quit | q           exit\n");

        loop {
            if self.paused_line.is_none() && !self.finished {
                self.run_until_pause()?;
            }
            if self.finished {
                println!("Program finished.");
                return Ok(());
            }
            if let Some(line) = self.paused_line {
                self.print_location(line);
            }
            print!("(v++ dbg)> ");
            io::stdout().flush().ok();
            let mut input = String::new();
            if io::stdin().read_line(&mut input).ok().filter(|&n| n > 0).is_none() {
                break;
            }
            if !self.handle_command(input.trim())? {
                break;
            }
        }
        Ok(())
    }

    fn handle_command(&mut self, cmd: &str) -> VppResult<bool> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(true);
        }
        match parts[0] {
            "c" | "continue" => {
                self.step_mode = StepMode::Continue;
                self.paused_line = None;
                self.interp.debug_mut().step_mode = StepMode::Continue;
                self.interp.debug_mut().paused_line = None;
                self.interp.debug_mut().resume_pending = true;
            }
            "s" | "step" => {
                self.step_mode = StepMode::StepInto;
                self.paused_line = None;
                self.interp.debug_mut().step_mode = StepMode::StepInto;
                self.interp.debug_mut().paused_line = None;
                self.interp.debug_mut().resume_pending = true;
            }
            "n" | "next" => {
                self.step_mode = StepMode::StepOver(self.interp.debug_call_depth());
                self.paused_line = None;
                self.interp.debug_mut().step_mode = self.step_mode.clone();
                self.interp.debug_mut().paused_line = None;
                self.interp.debug_mut().resume_pending = true;
            }
            "b" | "break" if parts.len() >= 2 => {
                if let Ok(n) = parts[1].parse::<u32>() {
                    self.breakpoints.insert(n);
                    self.interp.debug_mut().breakpoints.insert(n);
                    println!("Breakpoint set on line {n}");
                }
            }
            "l" | "locals" => self.print_locals(),
            "ls" | "list" => {
                let line = self.paused_line.unwrap_or(1);
                self.list_source(line);
            }
            "p" | "print" if parts.len() >= 2 => {
                let expr = parts[1..].join(" ");
                self.print_expr(&expr)?;
            }
            "q" | "quit" => return Ok(false),
            "h" | "help" => {
                println!("  c/continue  s/step  n/next  b N  locals  list  print EXPR  quit");
            }
            _ => println!("Unknown command. Type help."),
        }
        Ok(true)
    }

    fn run_until_pause(&mut self) -> VppResult<()> {
        self.interp.debug_mut().step_mode = self.step_mode.clone();
        loop {
            if let Some(saved) = self.interp.take_saved() {
                self.interp.exec_saved(saved)?;
            } else if self.top_ip < self.program.top_level.len() {
                let stmt = self.program.top_level[self.top_ip].clone();
                if self.interp.maybe_pause(stmt_line(&self.source, &stmt))? {
                    self.paused_line = Some(stmt_line(&self.source, &stmt));
                    return Ok(());
                }
                self.interp.exec_stmt(&stmt)?;
                if self.interp.is_debug_paused() {
                    self.paused_line = self.interp.debug_paused_line();
                    return Ok(());
                }
                self.top_ip += 1;
            } else if self.program.functions.contains_key("main") && !self.finished {
                self.interp.call_function("main", &[])?;
                self.finished = true;
                return Ok(());
            } else {
                self.finished = true;
                return Ok(());
            }
            if self.interp.is_debug_paused() {
                self.paused_line = self.interp.debug_paused_line();
                return Ok(());
            }
        }
    }

    fn print_location(&self, line: u32) {
        println!("→ paused at {}:{} ", self.program.source_file.display(), line);
        if line > 0 && (line as usize) <= self.lines.len() {
            println!("  {:4} | {}", line, self.lines[line as usize - 1]);
        }
    }

    fn list_source(&self, center: u32) {
        let start = center.saturating_sub(3).max(1) as usize;
        let end = (center as usize + 3).min(self.lines.len());
        for i in start..=end {
            let mark = if i as u32 == center { ">>" } else { "  " };
            println!("{mark} {:4} | {}", i, self.lines[i - 1]);
        }
    }

    fn print_locals(&self) {
        let locals = self.interp.debug_locals();
        if locals.is_empty() {
            println!("(no locals in scope)");
        } else {
            for (name, val) in locals {
                println!("  {name} = {val}");
            }
        }
    }

    fn print_expr(&mut self, text: &str) -> VppResult<()> {
        let wrapped = format!("({text})");
        let typed = crate::driver::check(&wrapped)?;
        if let Some(expr) = typed.top_level.first().and_then(|s| match s {
            TypedStmt::Expr(e) => Some(e),
            _ => None,
        }) {
            let val = self.interp.eval_expr_display(expr)?;
            println!("{val}");
        } else {
            println!("Could not evaluate `{text}`");
        }
        Ok(())
    }

    fn run_dap(&mut self, path: &Path) -> VppResult<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.interp.set_dap_print_tx(tx);
        self.dap_print_rx = Some(rx);

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut seq = 1u64;
        let threads = json!([{ "id": 1, "name": "v++ interpreter" }]);

        for line in stdin.lock().lines() {
            let line = line.map_err(|e| VppError::Other {
                message: format!("dap read: {e}"),
            })?;
            if line.trim().is_empty() {
                continue;
            }
            let req: Value = serde_json::from_str(&line).map_err(|e| VppError::Other {
                message: format!("dap json: {e}"),
            })?;
            let cmd = req["command"].as_str().unwrap_or("");
            let req_seq = req["seq"].as_u64().unwrap_or(0);

            let body = match cmd {
                "initialize" => {
                    json!({
                        "supportsConfigurationDoneRequest": true,
                        "supportsEvaluateForHovers": true,
                        "supportsStepBack": false,
                    })
                }
                "launch" => {
                    self.step_mode = StepMode::Continue;
                    self.paused_line = None;
                    json!({})
                }
                "setBreakpoints" => {
                    if let Some(src) = req["arguments"]["source"]["path"].as_str() {
                        if src == path.to_string_lossy() {
                            if let Some(arr) = req["arguments"]["breakpoints"].as_array() {
                                for bp in arr {
                                    if let Some(ln) = bp["line"].as_u64() {
                                        self.breakpoints.insert(ln as u32);
                                        self.interp.debug_mut().breakpoints.insert(ln as u32);
                                    }
                                }
                            }
                        }
                    }
                    json!({ "breakpoints": [] })
                }
                "configurationDone" => {
                    self.run_until_pause()?;
                    self.flush_dap_output(&mut stdout)?;
                    send_event(&mut stdout, "stopped", json!({ "reason": "entry", "threadId": 1 }))?;
                    json!({})
                }
                "threads" => json!({ "threads": threads }),
                "stackTrace" => {
                    let line = self.paused_line.unwrap_or(1);
                    json!({
                        "stackFrames": [{
                            "id": 1,
                            "name": "main",
                            "line": line,
                            "column": 1,
                            "source": { "path": path.to_string_lossy() }
                        }],
                        "totalFrames": 1
                    })
                }
                "scopes" => json!({
                    "scopes": [{ "name": "Locals", "variablesReference": 1, "expensive": false }]
                }),
                "variables" => {
                    let mut vars = Vec::new();
                    for (i, (name, val)) in self.interp.debug_locals().into_iter().enumerate() {
                        vars.push(json!({
                            "name": name,
                            "value": val,
                            "variablesReference": 0,
                            "evaluateName": name,
                            "indexedVariables": i
                        }));
                    }
                    json!({ "variables": vars })
                }
                "continue" | "next" | "stepIn" => {
                    self.step_mode = match cmd {
                        "stepIn" => StepMode::StepInto,
                        "next" => StepMode::StepOver(self.interp.debug_call_depth()),
                        _ => StepMode::Continue,
                    };
                    self.paused_line = None;
                    self.interp.debug_mut().step_mode = self.step_mode.clone();
                    self.interp.debug_mut().paused_line = None;
                    self.interp.debug_mut().resume_pending = true;
                    self.run_until_pause()?;
                    self.flush_dap_output(&mut stdout)?;
                    if self.finished {
                        send_event(&mut stdout, "terminated", json!({}))?;
                    } else {
                        send_event(
                            &mut stdout,
                            "stopped",
                            json!({ "reason": "breakpoint", "threadId": 1 }),
                        )?;
                    }
                    json!({ "allThreadsContinued": true })
                }
                "disconnect" => return Ok(()),
                _ => json!({}),
            };

            let resp = json!({
                "seq": seq,
                "type": "response",
                "request_seq": req_seq,
                "success": true,
                "command": cmd,
                "body": body,
            });
            seq += 1;
            writeln!(stdout, "{}", resp).ok();
            stdout.flush().ok();
        }
        Ok(())
    }
}

fn send_event(stdout: &mut io::Stdout, event: &str, body: Value) -> VppResult<()> {
    let msg = json!({ "seq": 1, "type": "event", "event": event, "body": body });
    writeln!(stdout, "{msg}").ok();
    stdout.flush().ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn debug_hello_steps() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("hello.vpp");
        let source = std::fs::read_to_string(&path).unwrap();
        let program = check_path(&path).unwrap();
        let mut session = DebugSession::new(source, program, &[1]).unwrap();
        session.run_until_pause().unwrap();
        assert!(session.paused_line.is_some() || session.finished);
    }
}

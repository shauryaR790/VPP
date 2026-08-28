use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast::{BinOp, UnOp};
use crate::error::{VppError, VppResult};
use crate::span::line_col;
use crate::types::{
    FunctionInfo, TypedExpr, TypedPattern, TypedProgram, TypedStmt, Type,
};

#[derive(Debug, Clone)]
pub enum StepMode {
    Continue,
    StepInto,
    StepOver(u32),
}

#[derive(Debug, Clone)]
pub enum SavedExec {
    Block { stmts: Vec<TypedStmt>, ip: usize },
}

#[derive(Debug, Clone)]
pub struct DebugState {
    pub source: String,
    pub breakpoints: HashSet<u32>,
    pub step_mode: StepMode,
    pub call_depth: u32,
    pub paused_line: Option<u32>,
    pub resume_pending: bool,
    pub saved: Option<SavedExec>,
}

impl DebugState {
    fn should_pause(&mut self, line: u32) -> bool {
        if self.resume_pending {
            self.resume_pending = false;
            return false;
        }
        let pause = match self.step_mode {
            StepMode::Continue => self.breakpoints.contains(&line),
            StepMode::StepInto => true,
            StepMode::StepOver(depth) => self.call_depth <= depth,
        };
        if pause {
            self.paused_line = Some(line);
            self.step_mode = StepMode::Continue;
        }
        pause
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(Rc<String>),
    Array(Rc<Vec<Value>>),
    Struct {
        name: String,
        fields: HashMap<String, Value>,
    },
    Variant {
        enum_name: String,
        variant: String,
        payload: Vec<Value>,
    },
    Void,
}

impl Value {
    fn as_int(&self) -> VppResult<i64> {
        match self {
            Value::Int(n) => Ok(*n),
            other => Err(VppError::Other {
                message: format!("expected int at runtime, found {other:?}"),
            }),
        }
    }

    fn as_bool(&self) -> VppResult<bool> {
        match self {
            Value::Bool(b) => Ok(*b),
            other => Err(VppError::Other {
                message: format!("expected bool at runtime, found {other:?}"),
            }),
        }
    }

    pub(crate) fn display_string(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            Value::String(s) => s.to_string(),
            Value::Struct { name, fields } => {
                let inner: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", v.display_string()))
                    .collect();
                format!("{name} {{ {} }}", inner.join(", "))
            }
            Value::Variant {
                enum_name,
                variant,
                payload,
            } => {
                if payload.is_empty() {
                    if enum_name == "Option" || enum_name == "Result" {
                        variant.clone()
                    } else {
                        format!("{enum_name}.{variant}")
                    }
                } else if payload.len() == 1 {
                    format!("{variant}({})", payload[0].display_string())
                } else {
                    format!(
                        "{variant}({})",
                        payload
                            .iter()
                            .map(Value::display_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            Value::Array(items) => {
                let inner: Vec<String> = items.iter().map(Value::display_string).collect();
                format!("[{}]", inner.join(", "))
            }
            Value::Void => "void".to_string(),
        }
    }
}

pub(crate) struct Interpreter {
    functions: HashMap<String, FunctionInfo>,
    scopes: Vec<HashMap<String, Value>>,
    return_value: Option<Value>,
    returning: bool,
    breaking: bool,
    continuing: bool,
    debug: Option<DebugState>,
    dap_print_tx: Option<std::sync::mpsc::Sender<String>>,
}

pub fn run(program: &TypedProgram) -> VppResult<()> {
    let mut interp = Interpreter::new(program.functions.clone());

    for stmt in &program.top_level {
        interp.exec_stmt(stmt)?;
        if interp.returning {
            return Ok(());
        }
    }

    if program.functions.contains_key("main") {
        interp.call_function("main", &[])?;
    }
    Ok(())
}

pub fn run_tests(program: &TypedProgram) -> VppResult<usize> {
    if program.tests.is_empty() {
        return Err(VppError::Other {
            message: "no `test` blocks found in this file".to_string(),
        });
    }

    let mut passed = 0usize;
    for test in &program.tests {
        print!("  {} ... ", test.name);
        let mut interp = Interpreter::new(program.functions.clone());
        match interp.exec_block(&test.body) {
            Ok(()) => {
                println!("ok");
                passed += 1;
            }
            Err(e) => {
                println!("FAILED");
                return Err(VppError::Other {
                    message: format!("test `{}`: {e}", test.name),
                });
            }
        }
    }
    Ok(passed)
}

impl Interpreter {
    pub fn new_debug(
        functions: HashMap<String, FunctionInfo>,
        source: String,
        breakpoints: HashSet<u32>,
    ) -> Self {
        Self {
            functions,
            scopes: vec![HashMap::new()],
            return_value: None,
            returning: false,
            breaking: false,
            continuing: false,
            debug: Some(DebugState {
                source,
                breakpoints,
                step_mode: StepMode::Continue,
                call_depth: 0,
                paused_line: None,
                resume_pending: false,
                saved: None,
            }),
            dap_print_tx: None,
        }
    }

    pub fn set_dap_print_tx(&mut self, tx: std::sync::mpsc::Sender<String>) {
        self.dap_print_tx = Some(tx);
    }

    pub fn debug_mut(&mut self) -> &mut DebugState {
        self.debug.as_mut().expect("debug session")
    }

    pub fn debug_call_depth(&self) -> u32 {
        self.debug.as_ref().map(|d| d.call_depth).unwrap_or(0)
    }

    pub fn is_debug_paused(&self) -> bool {
        self.debug
            .as_ref()
            .and_then(|d| d.paused_line)
            .is_some()
    }

    pub fn debug_paused_line(&self) -> Option<u32> {
        self.debug.as_ref().and_then(|d| d.paused_line)
    }

    pub fn maybe_pause(&mut self, line: u32) -> VppResult<bool> {
        if let Some(dbg) = &mut self.debug {
            if dbg.should_pause(line) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn take_saved(&mut self) -> Option<SavedExec> {
        self.debug.as_mut()?.saved.take()
    }

    pub fn exec_saved(&mut self, saved: SavedExec) -> VppResult<()> {
        match saved {
            SavedExec::Block { stmts, ip } => self.exec_block_from(&stmts, ip),
        }
    }

    pub fn debug_locals(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for scope in self.scopes.iter().rev() {
            for (name, val) in scope {
                if !out.iter().any(|(n, _)| n == name) {
                    out.push((name.clone(), val.display_string()));
                }
            }
        }
        out
    }

    pub fn eval_expr_display(&mut self, expr: &TypedExpr) -> VppResult<String> {
        Ok(self.eval_expr(expr)?.display_string())
    }

    fn new(functions: HashMap<String, FunctionInfo>) -> Self {
        Self {
            functions,
            scopes: vec![HashMap::new()],
            return_value: None,
            returning: false,
            breaking: false,
            continuing: false,
            debug: None,
            dap_print_tx: None,
        }
    }

    fn stmt_line(&self, stmt: &TypedStmt) -> u32 {
        let Some(src) = self.debug.as_ref().map(|d| d.source.as_str()) else {
            return 1;
        };
        stmt_line(src, stmt)
    }

    pub(crate) fn exec_stmt(&mut self, stmt: &TypedStmt) -> VppResult<()> {
        let line = self.stmt_line(stmt);
        if self.maybe_pause(line)? {
            return Ok(());
        }
        self.exec_stmt_inner(stmt)
    }

    fn exec_stmt_inner(&mut self, stmt: &TypedStmt) -> VppResult<()> {
        match stmt {
            TypedStmt::Let { name, value, .. } => {
                let val = self.eval_expr(value)?;
                self.define(name, val);
            }
            TypedStmt::Expr(expr) => {
                self.eval_expr(expr)?;
            }
            TypedStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                if self.eval_expr(condition)?.as_bool()? {
                    self.exec_block(then_block)?;
                } else if let Some(block) = else_block {
                    self.exec_block(block)?;
                }
            }
            TypedStmt::While { condition, body } => {
                while self.eval_expr(condition)?.as_bool()? {
                    self.exec_block(body)?;
                    if self.returning {
                        break;
                    }
                    if self.breaking {
                        self.breaking = false;
                        break;
                    }
                    if self.continuing {
                        self.continuing = false;
                        continue;
                    }
                }
            }
            TypedStmt::ForInt { var, start, end, body } => {
                let mut i = *start;
                while i < *end {
                    self.push_scope();
                    self.define(var, Value::Int(i));
                    self.exec_block(body)?;
                    self.pop_scope();
                    if self.returning {
                        break;
                    }
                    if self.breaking {
                        self.breaking = false;
                        break;
                    }
                    if self.continuing {
                        self.continuing = false;
                        i += 1;
                        continue;
                    }
                    i += 1;
                }
            }
            TypedStmt::ForArray { var, array, body, .. } => {
                let arr = self.eval_expr(array)?;
                if let Value::Array(items) = arr {
                    for item in items.iter() {
                        self.push_scope();
                        self.define(var, item.clone());
                        self.exec_block(body)?;
                        self.pop_scope();
                        if self.returning {
                            break;
                        }
                        if self.breaking {
                            self.breaking = false;
                            break;
                        }
                        if self.continuing {
                            self.continuing = false;
                            continue;
                        }
                    }
                }
            }
            TypedStmt::Return { value } => {
                self.return_value = if let Some(expr) = value {
                    Some(self.eval_expr(expr)?)
                } else {
                    Some(Value::Void)
                };
                self.returning = true;
            }
            TypedStmt::Match { scrutinee, arms, .. } => {
                let val = self.eval_expr(scrutinee)?;
                for arm in arms {
                    if let Some(bindings) = self.match_pattern(&arm.pattern, &val)? {
                        self.push_scope();
                        for (name, bound) in bindings {
                            self.define(&name, bound);
                        }
                        self.exec_block(&arm.body)?;
                        self.pop_scope();
                        if self.returning {
                            break;
                        }
                        break;
                    }
                }
            }
            TypedStmt::Block(stmts) => {
                self.exec_block(stmts)?;
            }
            TypedStmt::Break => {
                self.breaking = true;
            }
            TypedStmt::Continue => {
                self.continuing = true;
            }
        }
        Ok(())
    }

    fn exec_block(&mut self, stmts: &[TypedStmt]) -> VppResult<()> {
        self.exec_block_from(stmts, 0)
    }

    fn exec_block_from(&mut self, stmts: &[TypedStmt], start: usize) -> VppResult<()> {
        self.push_scope();
        let mut i = start;
        while i < stmts.len() {
            let stmt = &stmts[i];
            let line = self.stmt_line(stmt);
            if self.maybe_pause(line)? {
                if let Some(dbg) = &mut self.debug {
                    dbg.saved = Some(SavedExec::Block {
                        stmts: stmts.to_vec(),
                        ip: i,
                    });
                }
                return Ok(());
            }
            self.exec_stmt_inner(stmt)?;
            if self.is_debug_paused() {
                if let Some(dbg) = &mut self.debug {
                    dbg.saved = Some(SavedExec::Block {
                        stmts: stmts.to_vec(),
                        ip: i,
                    });
                }
                return Ok(());
            }
            if self.returning || self.breaking || self.continuing {
                break;
            }
            i += 1;
        }
        self.pop_scope();
        Ok(())
    }

    fn match_pattern(
        &self,
        pattern: &TypedPattern,
        value: &Value,
    ) -> VppResult<Option<Vec<(String, Value)>>> {
        match pattern {
            TypedPattern::Wildcard => Ok(Some(Vec::new())),
            TypedPattern::Literal(expr) => {
                let lit = self.eval_expr_standalone(expr)?;
                Ok((lit == *value).then_some(Vec::new()))
            }
            TypedPattern::Variant {
                enum_name,
                variant,
                bindings,
                ..
            } => {
                if let Value::Variant {
                    enum_name: en,
                    variant: vn,
                    payload,
                } = value
                {
                    if en == enum_name && vn == variant && payload.len() == bindings.len() {
                        let mut out = Vec::new();
                        for (name, val) in bindings.iter().zip(payload.iter()) {
                            out.push((name.clone(), val.clone()));
                        }
                        return Ok(Some(out));
                    }
                }
                Ok(None)
            }
            TypedPattern::Struct {
                struct_name,
                fields,
            } => {
                if let Value::Struct { name, fields: vals } = value {
                    if name != struct_name {
                        return Ok(None);
                    }
                    let mut out = Vec::new();
                    for (field, binding, _) in fields {
                        if let Some(val) = vals.get(field) {
                            out.push((binding.clone(), val.clone()));
                        } else {
                            return Ok(None);
                        }
                    }
                    Ok(Some(out))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn eval_expr_standalone(&self, expr: &TypedExpr) -> VppResult<Value> {
        match expr {
            TypedExpr::Int(n) => Ok(Value::Int(*n)),
            TypedExpr::Float(n) => Ok(Value::Float(*n)),
            TypedExpr::Bool(b) => Ok(Value::Bool(*b)),
            TypedExpr::String(s) => Ok(Value::String(Rc::new(s.clone()))),
            _ => Err(VppError::Other {
                message: "complex literal patterns not supported yet".to_string(),
            }),
        }
    }

    pub(crate) fn call_function(&mut self, name: &str, args: &[TypedExpr]) -> VppResult<Value> {
        if name == "print" {
            for arg in args {
                let val = self.eval_expr(arg)?;
                let line = val.display_string();
                if let Some(tx) = &self.dap_print_tx {
                    let _ = tx.send(line.clone());
                }
                println!("{line}");
            }
            return Ok(Value::Void);
        }

        if name == "len" {
            let val = self.eval_expr(&args[0])?;
            let len = match val {
                Value::String(s) => s.len() as i64,
                Value::Array(items) => items.len() as i64,
                other => {
                    return Err(VppError::Other {
                        message: format!("len() expects array or string, found {other:?}"),
                    });
                }
            };
            return Ok(Value::Int(len));
        }

        if name == "assert" {
            let cond = self.eval_expr(&args[0])?.as_bool()?;
            if !cond {
                return Err(VppError::Other {
                    message: "assertion failed".to_string(),
                });
            }
            return Ok(Value::Void);
        }

        if name == "assert_eq" {
            let left = self.eval_expr(&args[0])?;
            let right = self.eval_expr(&args[1])?;
            if left != right {
                return Err(VppError::Other {
                    message: format!(
                        "assertion failed: {} != {}",
                        left.display_string(),
                        right.display_string()
                    ),
                });
            }
            return Ok(Value::Void);
        }

        if name == "read_file" {
            let path = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("read_file expects string path, found {other:?}"),
                    });
                }
            };
            let text = std::fs::read_to_string(path.as_str()).map_err(|e| VppError::Other {
                message: format!("read_file failed: {e}"),
            })?;
            return Ok(Value::String(Rc::new(text)));
        }

        if name == "write_file" {
            let path = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("write_file expects string path, found {other:?}"),
                    });
                }
            };
            let contents = match self.eval_expr(&args[1])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("write_file expects string contents, found {other:?}"),
                    });
                }
            };
            std::fs::write(path.as_str(), contents.as_str()).map_err(|e| VppError::Other {
                message: format!("write_file failed: {e}"),
            })?;
            return Ok(Value::Void);
        }

        if name == "file_exists" {
            let path = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("file_exists expects string path, found {other:?}"),
                    });
                }
            };
            return Ok(Value::Bool(std::path::Path::new(path.as_str()).exists()));
        }

        if name == "json_parse" {
            let raw = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("json_parse expects string, found {other:?}"),
                    });
                }
            };
            serde_json::from_str::<serde_json::Value>(raw.as_str()).map_err(|e| VppError::Other {
                message: format!("json_parse failed: {e}"),
            })?;
            return Ok(Value::String(raw));
        }

        if name == "json_stringify" {
            let raw = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("json_stringify expects string, found {other:?}"),
                    });
                }
            };
            let value = if raw.starts_with('{') || raw.starts_with('[') {
                serde_json::from_str(raw.as_str()).map_err(|e| VppError::Other {
                    message: format!("json_stringify failed: {e}"),
                })?
            } else {
                serde_json::Value::String(raw.to_string())
            };
            return Ok(Value::String(Rc::new(value.to_string())));
        }

        if name == "process_run" {
            let cmd = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("process_run expects string command, found {other:?}"),
                    });
                }
            };
            let status = if cfg!(windows) {
                std::process::Command::new("cmd")
                    .args(["/C", cmd.as_str()])
                    .status()
            } else {
                std::process::Command::new("sh")
                    .args(["-c", cmd.as_str()])
                    .status()
            }
            .map_err(|e| VppError::Other {
                message: format!("process_run failed: {e}"),
            })?;
            return Ok(Value::Int(status.code().unwrap_or(1) as i64));
        }

        if name == "command_run" {
            let program = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("command_run expects string program, found {other:?}"),
                    });
                }
            };
            let argv = match self.eval_expr(&args[1])? {
                Value::Array(items) => items,
                other => {
                    return Err(VppError::Other {
                        message: format!("command_run expects array[string] args, found {other:?}"),
                    });
                }
            };
            let mut arg_strings = Vec::new();
            for item in argv.iter() {
                match item {
                    Value::String(s) => arg_strings.push(s.clone()),
                    other => {
                        return Err(VppError::Other {
                            message: format!(
                                "command_run args must be strings, found {other:?}"
                            ),
                        });
                    }
                }
            }
            let cwd = match self.eval_expr(&args[2])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("command_run expects string cwd, found {other:?}"),
                    });
                }
            };
            let timeout = self.eval_expr(&args[3])?.as_int()?;
            let code = crate::automation::command_run(
                program.as_str(),
                &arg_strings,
                cwd.as_str(),
                timeout,
            )?;
            return Ok(Value::Int(code));
        }

        if name == "command_stdout" {
            let (out, _) = crate::automation::take_last_cmd_io();
            return Ok(Value::String(Rc::new(out)));
        }

        if name == "command_stderr" {
            let (_, err) = crate::automation::take_last_cmd_io();
            return Ok(Value::String(Rc::new(err)));
        }

        if name == "env_get" {
            let key = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("env_get expects string key, found {other:?}"),
                    });
                }
            };
            let val = std::env::var(key.as_str()).unwrap_or_default();
            return Ok(Value::String(Rc::new(val)));
        }

        if name == "env_set" {
            let key = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("env_set expects string key, found {other:?}"),
                    });
                }
            };
            let val = match self.eval_expr(&args[1])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("env_set expects string value, found {other:?}"),
                    });
                }
            };
            std::env::set_var(key.as_str(), val.as_str());
            return Ok(Value::Void);
        }

        if name == "dir_exists" {
            let path = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("dir_exists expects string path, found {other:?}"),
                    });
                }
            };
            let ok = std::fs::metadata(path.as_str())
                .map(|m| m.is_dir())
                .unwrap_or(false);
            return Ok(Value::Bool(ok));
        }

        if name == "dir_create" {
            let path = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("dir_create expects string path, found {other:?}"),
                    });
                }
            };
            std::fs::create_dir_all(path.as_str()).map_err(|e| VppError::Other {
                message: format!("dir_create failed: {e}"),
            })?;
            return Ok(Value::Void);
        }

        if name == "dir_list" {
            let path = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("dir_list expects string path, found {other:?}"),
                    });
                }
            };
            let mut names = Vec::new();
            for entry in std::fs::read_dir(path.as_str()).map_err(|e| VppError::Other {
                message: format!("dir_list failed: {e}"),
            })? {
                let entry = entry.map_err(|e| VppError::Other {
                    message: format!("dir_list failed: {e}"),
                })?;
                names.push(Value::String(Rc::new(
                    entry.file_name().to_string_lossy().into_owned(),
                )));
            }
            return Ok(Value::Array(Rc::new(names)));
        }

        if name == "log_line" {
            let level = match self.eval_expr(&args[0])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("log_line expects string level, found {other:?}"),
                    });
                }
            };
            let message = match self.eval_expr(&args[1])? {
                Value::String(s) => s,
                other => {
                    return Err(VppError::Other {
                        message: format!("log_line expects string message, found {other:?}"),
                    });
                }
            };
            eprintln!("[{}] {}", level.as_str(), message.as_str());
            return Ok(Value::Void);
        }

        if name == "workflow_parallel_tasks" {
            let tasks = match self.eval_expr(&args[0])? {
                Value::Array(items) => items,
                other => {
                    return Err(VppError::Other {
                        message: format!(
                            "workflow_parallel_tasks expects array[Task], found {other:?}"
                        ),
                    });
                }
            };
            let mut specs = Vec::new();
            for item in tasks.iter() {
                specs.push(parse_task_spec(item)?);
            }
            let code = crate::automation::parallel_tasks(specs)?;
            return Ok(Value::Int(code));
        }

        let func = self
            .functions
            .get(name)
            .cloned()
            .ok_or_else(|| VppError::Other {
                message: format!("undefined function `{name}`"),
            })?;

        if self.debug.is_some() && !matches!(name, "print" | "len" | "assert" | "assert_eq" | "read_file" | "write_file" | "file_exists" | "json_parse" | "json_stringify" | "process_run" | "command_run" | "command_stdout" | "command_stderr" | "env_get" | "env_set" | "dir_list" | "dir_exists" | "dir_create" | "log_line" | "workflow_parallel_tasks") {
            if let Some(dbg) = &mut self.debug {
                dbg.call_depth += 1;
            }
        }

        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_expr(arg)?);
        }

        self.push_scope();
        for ((param, _), value) in func.params.iter().zip(arg_values) {
            self.define(param, value);
        }

        let saved_returning = self.returning;
        let saved_return = self.return_value.take();
        self.returning = false;

        for stmt in &func.body {
            self.exec_stmt(stmt)?;
            if self.returning {
                break;
            }
        }

        let result = if self.returning {
            self.return_value.take().unwrap_or(Value::Void)
        } else if func.ret == Type::Void {
            Value::Void
        } else {
            Value::Int(0)
        };

        self.returning = saved_returning;
        self.return_value = saved_return;
        self.pop_scope();

        if self.debug.is_some() && !matches!(name, "print" | "len" | "assert" | "assert_eq" | "read_file" | "write_file" | "file_exists" | "json_parse" | "json_stringify" | "process_run" | "command_run" | "command_stdout" | "command_stderr" | "env_get" | "env_set" | "dir_list" | "dir_exists" | "dir_create" | "log_line" | "workflow_parallel_tasks") {
            if let Some(dbg) = &mut self.debug {
                dbg.call_depth = dbg.call_depth.saturating_sub(1);
            }
        }

        Ok(result)
    }

    fn eval_expr(&mut self, expr: &TypedExpr) -> VppResult<Value> {
        match expr {
            TypedExpr::Int(n) => Ok(Value::Int(*n)),
            TypedExpr::Float(n) => Ok(Value::Float(*n)),
            TypedExpr::Bool(b) => Ok(Value::Bool(*b)),
            TypedExpr::String(s) => Ok(Value::String(Rc::new(s.clone()))),
            TypedExpr::Ident { name, .. } => self.lookup(name).ok_or_else(|| VppError::Other {
                message: format!("undefined variable `{name}` at runtime"),
            }),
            TypedExpr::Binary { op, left, right, .. } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binary(*op, l, r)
            }
            TypedExpr::Unary { op, expr, .. } => {
                let val = self.eval_expr(expr)?;
                self.eval_unary(*op, val)
            }
            TypedExpr::Call { name, args, .. } => self.call_function(name, args),
            TypedExpr::Index { target, index, .. } => {
                let target_val = self.eval_expr(target)?;
                let idx = self.eval_expr(index)?.as_int()? as usize;
                match target_val {
                    Value::Array(items) => items.get(idx).cloned().ok_or_else(|| VppError::Other {
                        message: format!("array index out of bounds: {idx}"),
                    }),
                    other => Err(VppError::Other {
                        message: format!("cannot index non-array value {other:?}"),
                    }),
                }
            }
            TypedExpr::Field { target, field, .. } => {
                let target_val = self.eval_expr(target)?;
                match target_val {
                    Value::Struct { fields, .. } => fields.get(field).cloned().ok_or_else(|| {
                        VppError::Other {
                            message: format!("struct has no field `{field}`"),
                        }
                    }),
                    other => Err(VppError::Other {
                        message: format!("field access on non-struct value {other:?}"),
                    }),
                }
            }
            TypedExpr::Array { elements, .. } => {
                let mut items = Vec::new();
                for elem in elements {
                    items.push(self.eval_expr(elem)?);
                }
                Ok(Value::Array(Rc::new(items)))
            }
            TypedExpr::StructLit { name, fields, .. } => {
                let mut map = HashMap::new();
                for (field, expr) in fields {
                    map.insert(field.clone(), self.eval_expr(expr)?);
                }
                Ok(Value::Struct {
                    name: name.clone(),
                    fields: map,
                })
            }
            TypedExpr::Variant {
                enum_name,
                variant,
                payload,
                ..
            } => {
                let mut vals = Vec::new();
                for expr in payload {
                    vals.push(self.eval_expr(expr)?);
                }
                Ok(Value::Variant {
                    enum_name: enum_name.clone(),
                    variant: variant.clone(),
                    payload: vals,
                })
            }
            TypedExpr::Assign { name, value, .. } => {
                let val = self.eval_expr(value)?;
                if self.assign(name, val.clone()) {
                    Ok(val)
                } else {
                    Err(VppError::Other {
                        message: format!("undefined variable `{name}` for assignment"),
                    })
                }
            }
            TypedExpr::Match { scrutinee, arms, .. } => {
                let val = self.eval_expr(scrutinee)?;
                for arm in arms {
                    if let Some(bindings) = self.match_pattern(&arm.pattern, &val)? {
                        self.push_scope();
                        for (name, bound) in bindings {
                            self.define(&name, bound);
                        }
                        let mut last = Value::Void;
                        for stmt in &arm.body {
                            self.exec_stmt(stmt)?;
                            if let TypedStmt::Expr(expr) = stmt {
                                last = self.eval_expr(expr)?;
                            }
                            if self.returning {
                                break;
                            }
                        }
                        self.pop_scope();
                        return Ok(last);
                    }
                }
                Err(VppError::Other {
                    message: "non-exhaustive match at runtime".to_string(),
                })
            }
        }
    }

    fn eval_binary(&self, op: BinOp, left: Value, right: Value) -> VppResult<Value> {
        match op {
            BinOp::Add => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::String(a), Value::String(b)) => {
                    Ok(Value::String(Rc::new(format!("{}{}", a, b))))
                }
                (a, b) => Err(VppError::Other {
                    message: format!("invalid operands for +: {a:?} and {b:?}"),
                }),
            },
            BinOp::Sub => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (a, b) => Err(VppError::Other {
                    message: format!("invalid operands for -: {a:?} and {b:?}"),
                }),
            },
            BinOp::Mul => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (a, b) => Err(VppError::Other {
                    message: format!("invalid operands for *: {a:?} and {b:?}"),
                }),
            },
            BinOp::Div => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (a, b) => Err(VppError::Other {
                    message: format!("invalid operands for /: {a:?} and {b:?}"),
                }),
            },
            BinOp::Mod => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
                (a, b) => Err(VppError::Other {
                    message: format!("invalid operands for %: {a:?} and {b:?}"),
                }),
            },
            BinOp::Eq => Ok(Value::Bool(left == right)),
            BinOp::NotEq => Ok(Value::Bool(left != right)),
            BinOp::Lt => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                (a, b) => Err(VppError::Other {
                    message: format!("invalid comparison operands: {a:?} and {b:?}"),
                }),
            },
            BinOp::LtEq => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                (a, b) => Err(VppError::Other {
                    message: format!("invalid comparison operands: {a:?} and {b:?}"),
                }),
            },
            BinOp::Gt => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                (a, b) => Err(VppError::Other {
                    message: format!("invalid comparison operands: {a:?} and {b:?}"),
                }),
            },
            BinOp::GtEq => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                (a, b) => Err(VppError::Other {
                    message: format!("invalid comparison operands: {a:?} and {b:?}"),
                }),
            },
            BinOp::And => Ok(Value::Bool(left.as_bool()? && right.as_bool()?)),
            BinOp::Or => Ok(Value::Bool(left.as_bool()? || right.as_bool()?)),
        }
    }

    fn eval_unary(&self, op: UnOp, val: Value) -> VppResult<Value> {
        match op {
            UnOp::Not => Ok(Value::Bool(!val.as_bool()?)),
            UnOp::Neg => match val {
                Value::Int(n) => Ok(Value::Int(-n)),
                Value::Float(n) => Ok(Value::Float(-n)),
                other => Err(VppError::Other {
                    message: format!("cannot negate {other:?}"),
                }),
            },
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: &str, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), value);
        }
    }

    fn assign(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    fn lookup(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }
}

pub fn stmt_line(source: &str, stmt: &TypedStmt) -> u32 {
    match stmt {
        TypedStmt::Let { span, .. } => line_col(source, span.start).0 as u32,
        TypedStmt::Expr(expr) => expr_line(source, expr),
        TypedStmt::If { condition, .. } => expr_line(source, condition),
        TypedStmt::While { condition, .. } => expr_line(source, condition),
        TypedStmt::ForInt { .. } => 1,
        TypedStmt::ForArray { array, .. } => expr_line(source, array),
        TypedStmt::Return { value: Some(v) } => expr_line(source, v),
        TypedStmt::Return { value: None } => 1,
        TypedStmt::Match { scrutinee, .. } => expr_line(source, scrutinee),
        TypedStmt::Break | TypedStmt::Continue | TypedStmt::Block(_) => 1,
    }
}

fn expr_line(source: &str, expr: &TypedExpr) -> u32 {
    match expr {
        TypedExpr::Ident { span, .. } => line_col(source, span.start).0 as u32,
        TypedExpr::Binary { left, .. } => expr_line(source, left),
        TypedExpr::Unary { expr, .. } => expr_line(source, expr),
        TypedExpr::Call { .. } => 1,
        TypedExpr::Index { target, .. } => expr_line(source, target),
        TypedExpr::Field { target, .. } => expr_line(source, target),
        TypedExpr::Assign { value, .. } => expr_line(source, value),
        TypedExpr::Match { scrutinee, .. } => expr_line(source, scrutinee),
        _ => 1,
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Struct { name: n1, fields: f1 }, Value::Struct { name: n2, fields: f2 }) => {
                n1 == n2 && f1 == f2
            }
            (
                Value::Variant {
                    enum_name: e1,
                    variant: v1,
                    payload: p1,
                },
                Value::Variant {
                    enum_name: e2,
                    variant: v2,
                    payload: p2,
                },
            ) => e1 == e2 && v1 == v2 && p1 == p2,
            (Value::Void, Value::Void) => true,
            _ => false,
        }
    }
}

/// Persistent REPL state: accumulates accepted source and runs only new statements.
pub struct ReplSession {
    source: String,
    executed_stmts: usize,
    interp: Interpreter,
    session_path: std::path::PathBuf,
}

impl ReplSession {
    pub fn new() -> crate::VppResult<Self> {
        let cwd = std::env::current_dir().map_err(|e| VppError::Other {
            message: format!("repl needs a working directory: {e}"),
        })?;
        let session_path = cwd.join(".vpp").join("repl_session.vpp");
        if let Some(parent) = session_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| VppError::Other {
                message: format!("failed to create {}: {e}", parent.display()),
            })?;
        }
        let session = Self {
            source: String::from("import std.io\n"),
            executed_stmts: 0,
            interp: Interpreter::new(HashMap::new()),
            session_path,
        };
        std::fs::write(&session.session_path, &session.source).ok();
        Ok(session)
    }

    pub fn reset(&mut self) -> crate::VppResult<()> {
        self.source = String::from("import std.io\n");
        self.executed_stmts = 0;
        self.interp = Interpreter::new(HashMap::new());
        std::fs::write(&self.session_path, &self.source).ok();
        Ok(())
    }

    pub fn eval(&mut self, input: &str) -> crate::VppResult<()> {
        let line = input.trim();
        if line.is_empty() {
            return Ok(());
        }
        self.source.push_str(line);
        self.source.push('\n');
        std::fs::write(&self.session_path, &self.source).map_err(|e| VppError::Other {
            message: format!("failed to write {}: {e}", self.session_path.display()),
        })?;
        let typed = crate::driver::check_path(&self.session_path)?;
        self.interp.functions = typed.functions.clone();
        for stmt in typed.top_level.iter().skip(self.executed_stmts) {
            self.interp.exec_stmt(stmt)?;
            if self.interp.returning {
                self.interp.returning = false;
                break;
            }
        }
        self.executed_stmts = typed.top_level.len();
        Ok(())
    }
}

pub fn run_repl() -> crate::VppResult<()> {
    use std::io::{self, Write};

    println!("v++ REPL v0.6  -  readable code, instant feedback");
    println!("  Same language as `vpp run` and `vpp build`. Type :help for commands.\n");

    let mut session = ReplSession::new()?;
    let stdin = io::stdin();
    loop {
        print!("v++> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        let n = stdin.read_line(&mut line).map_err(|e| VppError::Other {
            message: format!("repl read error: {e}"),
        })?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        match trimmed {
            "" => continue,
            ":quit" | ":exit" | ":q" => break,
            ":reset" => {
                session.reset()?;
                println!("(session reset)");
            }
            ":help" | ":h" => {
                println!("  :quit     exit repl");
                println!("  :reset    clear session");
                println!("  :help     this message");
                println!("  print(x)  show a value");
                println!("  fn/let/…  definitions persist across lines");
            }
            _ => match session.eval(trimmed) {
                Ok(()) => {}
                Err(e) => {
                    if session.source.ends_with(&format!("{trimmed}\n")) {
                        session.source.truncate(session.source.len() - trimmed.len() - 1);
                        let _ = std::fs::write(&session.session_path, &session.source);
                        if let Ok(typed) = crate::driver::check_path(&session.session_path) {
                            session.executed_stmts = typed.top_level.len();
                        }
                    }
                    eprintln!("{e}");
                }
            },
        }
    }
    Ok(())
}

fn parse_task_spec(value: &Value) -> VppResult<crate::automation::TaskSpec> {
    let fields = match value {
        Value::Struct { fields, .. } => fields,
        other => {
            return Err(VppError::Other {
                message: format!("expected Task struct, found {other:?}"),
            });
        }
    };
    let field_str = |key: &str| -> VppResult<String> {
        match fields.get(key) {
            Some(Value::String(s)) => Ok(s.to_string()),
            other => Err(VppError::Other {
                message: format!("Task.{key} expects string, found {other:?}"),
            }),
        }
    };
    let args = match fields.get("args") {
        Some(Value::Array(items)) => {
            let mut out = Vec::new();
            for item in items.iter() {
                match item {
                    Value::String(s) => out.push(s.to_string()),
                    other => {
                        return Err(VppError::Other {
                            message: format!("Task.args must be strings, found {other:?}"),
                        });
                    }
                }
            }
            out
        }
        other => {
            return Err(VppError::Other {
                message: format!("Task.args expects array[string], found {other:?}"),
            });
        }
    };
    let timeout_ms = match fields.get("timeout_ms") {
        Some(Value::Int(n)) => *n,
        other => {
            return Err(VppError::Other {
                message: format!("Task.timeout_ms expects int, found {other:?}"),
            });
        }
    };
    Ok(crate::automation::TaskSpec {
        name: field_str("name")?,
        program: field_str("program")?,
        args,
        cwd: field_str("cwd")?,
        timeout_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::driver::{check, check_path};

    #[test]
    fn runs_hello() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("hello.vpp");
        let typed = check_path(&path).unwrap();
        run(&typed).unwrap();
    }

    #[test]
    fn runs_structs() {
        let source = include_str!("../../examples/structs.vpp");
        run(&check(source).unwrap()).unwrap();
    }

    #[test]
    fn ok_val_function_has_body() {
        let source = include_str!("../../examples/match_test.vpp");
        let typed = check(source).unwrap();
        let func = typed.functions.get("ok_val").expect("ok_val missing");
        assert!(!func.body.is_empty(), "function body should not be empty");
        assert!(matches!(func.ret, Type::Result { .. }));
        assert!(matches!(
            &func.body[0],
            TypedStmt::Return {
                value: Some(TypedExpr::Variant { .. }),
                ..
            }
        ));
        assert_eq!(func.body.len(), 1);
    }

    #[test]
    fn repl_session_eval() {
        let mut session = ReplSession::new().unwrap();
        session.eval("fn double(n: int) -> int { return n + n }").unwrap();
        session.eval("print(double(21))").unwrap();
    }

    #[test]
    fn calls_ok_val() {
        let source = include_str!("../../examples/match_test.vpp");
        let typed = check(source).unwrap();
        let call = TypedExpr::Call {
            name: "ok_val".to_string(),
            args: vec![],
            ty: typed.functions["ok_val"].ret.clone(),
        };
        let mut interp = Interpreter::new(typed.functions.clone());
        let val = interp.eval_expr(&call).unwrap();
        assert!(matches!(val, Value::Variant { .. }));
    }
}

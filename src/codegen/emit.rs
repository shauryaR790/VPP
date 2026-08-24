//! Lower v++ IR to LLVM IR and link the C runtime.

#[path = "struct_enum.rs"]
mod struct_enum;

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::basic_block::BasicBlock;
use inkwell::types::StructType;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FunctionValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::FloatPredicate;
use inkwell::IntPredicate;

use crate::ast::BinOp;
use crate::builtins::BuiltinKind;
use crate::error::{VppError, VppResult};
use crate::ir::{
    IrFunction, IrModule, IrStmt, IrType, IrValue,
};

pub(super) struct Emit<'ctx> {
    pub(super) context: &'ctx Context,
    pub(super) module: Module<'ctx>,
    pub(super) builder: Builder<'ctx>,
    pub(super) functions: HashMap<String, FunctionValue<'ctx>>,
    pub(super) locals_stack: Vec<HashMap<String, PointerValue<'ctx>>>,
    pub(super) types_stack: Vec<HashMap<String, IrType>>,
    pub(super) heap_names_stack: Vec<Vec<String>>,
    pub(super) struct_defs: HashMap<String, Vec<(String, IrType)>>,
    pub(super) enum_defs: HashMap<String, Vec<(String, Vec<IrType>)>>,
    pub(super) struct_types: HashMap<String, StructType<'ctx>>,
    pub(super) enum_types: HashMap<String, StructType<'ctx>>,
    pub(super) variant_tags: HashMap<(String, String), i64>,
    pub(super) loop_stack: Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,
    pub(super) i64_type: inkwell::types::IntType<'ctx>,
    pub(super) f64_type: inkwell::types::FloatType<'ctx>,
    pub(super) i1_type: inkwell::types::IntType<'ctx>,
    pub(super) i8_ptr_type: inkwell::types::PointerType<'ctx>,
    pub(super) string_ptr_type: inkwell::types::PointerType<'ctx>,
    pub(super) array_ptr_type: inkwell::types::PointerType<'ctx>,
    pub(super) void_type: inkwell::types::VoidType<'ctx>,
}

pub fn compile_module(
    ir: &IrModule,
    source_path: &Path,
    output: Option<&Path>,
    emit_ir: Option<&Path>,
) -> VppResult<()> {
    crate::ensure_llvm_stubs_linked();
    let context = Context::create();
    let module = context.create_module("vpp_module");
    let mut emit = Emit::new(&context, module);

    emit.declare_runtime()?;
    emit.init_types_from_module(ir);
    emit.build_all_types()?;
    emit.compile_functions(ir)?;
    emit.compile_main(ir)?;

    if let Some(ir_path) = emit_ir {
        emit.module.print_to_file(ir_path).map_err(|e| VppError::Other {
            message: format!("failed to write IR: {e}"),
        })?;
    }

    let Some(output) = output else {
        let _ = source_path;
        return Ok(());
    };

    link_executable(&emit.module, output)?;
    let _ = source_path;
    Ok(())
}

fn link_executable(module: &Module, output: &Path) -> VppResult<()> {
    static BUILD_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = BUILD_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temp_dir = std::env::temp_dir().join(format!("vpp-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&temp_dir).map_err(|source| VppError::Io { source })?;

    let ll_path = temp_dir.join("out.ll");
    let obj_path = temp_dir.join("out.o");
    let runtime_c = temp_dir.join("vpp_runtime.c");
    let runtime_o = temp_dir.join("vpp_runtime.o");

    crate::codegen::runtime::emit_runtime_c(&runtime_c)?;

    module.print_to_file(&ll_path).map_err(|e| VppError::Other {
        message: format!("failed to write IR: {e}"),
    })?;

    let ll = ll_path.to_string_lossy();
    let obj = obj_path.to_string_lossy();
    let rt_c = runtime_c.to_string_lossy();
    let rt_o = runtime_o.to_string_lossy();

    run_cmd("clang", &["-c", &ll, "-o", &obj, "-O1"])?;
    run_cmd("clang", &["-c", &rt_c, "-o", &rt_o])?;
    let staged = temp_dir.join(if cfg!(windows) { "linked.exe" } else { "linked" });
    let staged_str = staged.to_string_lossy();
    run_cmd("clang", &[&obj, &rt_o, "-o", &staged_str])?;
    std::fs::copy(&staged, output).map_err(|source| VppError::Io { source })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(output) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(output, perms);
        }
    }
    Ok(())
}

fn tool_in_llvm_bin(name: &str) -> String {
    if let Ok(prefix) = std::env::var("LLVM_SYS_221_PREFIX") {
        let bin = std::path::PathBuf::from(prefix).join("bin");
        for candidate in [name.to_string(), format!("{name}-22")] {
            let path = bin.join(&candidate);
            if path.exists() {
                return path.to_string_lossy().into_owned();
            }
        }
    }
    name.to_string()
}

fn run_cmd(program: &str, args: &[&str]) -> VppResult<()> {
    let program = if program == "clang" {
        tool_in_llvm_bin("clang")
    } else {
        program.to_string()
    };
    let output = Command::new(&program).args(args).output().map_err(|e| VppError::Other {
        message: format!("failed to run `{program}`: {e}. Ensure clang/LLVM is installed."),
    })?;
    if !output.status.success() {
        return Err(VppError::Other {
            message: format!(
                "`{program}` failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }
    Ok(())
}

impl<'ctx> Emit<'ctx> {
    fn new(context: &'ctx Context, module: Module<'ctx>) -> Self {
        let string_struct = context.opaque_struct_type("VppString");
        let array_struct = context.opaque_struct_type("VppArray");
        let _ = (&string_struct, &array_struct);
        let ptr = context.ptr_type(AddressSpace::default());

        Self {
            context,
            module,
            builder: context.create_builder(),
            functions: HashMap::new(),
            locals_stack: Vec::new(),
            types_stack: Vec::new(),
            heap_names_stack: Vec::new(),
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            struct_types: HashMap::new(),
            enum_types: HashMap::new(),
            variant_tags: HashMap::new(),
            loop_stack: Vec::new(),
            i64_type: context.i64_type(),
            f64_type: context.f64_type(),
            i1_type: context.bool_type(),
            i8_ptr_type: ptr,
            string_ptr_type: ptr,
            array_ptr_type: ptr,
            void_type: context.void_type(),
        }
    }

    pub(super) fn enter_scope(&mut self) {
        self.locals_stack.push(HashMap::new());
        self.types_stack.push(HashMap::new());
        self.heap_names_stack.push(Vec::new());
    }

    pub(super) fn exit_scope(&mut self) {
        self.release_current_scope_heap();
        self.heap_names_stack.pop();
        self.locals_stack.pop();
        self.types_stack.pop();
    }

    fn release_current_scope_heap(&mut self) {
        if let Some(names) = self.heap_names_stack.last() {
            for name in names.clone() {
                if let Some((ptr, ty)) = self.lookup_local(&name) {
                    if ty == IrType::String {
                        let loaded = self
                            .builder
                            .build_load(self.string_ptr_type, ptr, &name)
                            .unwrap();
                        self.emit_string_release(loaded.into_pointer_value());
                    } else if ty.is_array() {
                        let loaded = self
                            .builder
                            .build_load(self.array_ptr_type, ptr, &name)
                            .unwrap();
                        self.emit_array_release(loaded.into_pointer_value());
                    }
                }
            }
        }
    }

    fn clear_scope_state(&mut self) {
        self.locals_stack.clear();
        self.types_stack.clear();
        self.heap_names_stack.clear();
    }

    pub(super) fn define_local(&mut self, name: &str, ptr: PointerValue<'ctx>, ty: IrType) {
        self.locals_stack
            .last_mut()
            .expect("scope")
            .insert(name.to_string(), ptr);
        self.types_stack
            .last_mut()
            .expect("scope")
            .insert(name.to_string(), ty);
    }

    fn lookup_local(&self, name: &str) -> Option<(PointerValue<'ctx>, IrType)> {
        for (locals, types) in self.locals_stack.iter().zip(self.types_stack.iter()).rev() {
            if let Some(ptr) = locals.get(name) {
                let ty = types.get(name).cloned().unwrap_or(IrType::Int);
                return Some((*ptr, ty));
            }
        }
        None
    }

    fn declare_runtime(&self) -> VppResult<()> {
        let i64 = self.i64_type;
        let f64 = self.f64_type;
        let i32 = self.context.i32_type();
        let i8_ptr = self.i8_ptr_type;
        let str_ptr = self.string_ptr_type;
        let arr_ptr = self.array_ptr_type;
        let void = self.void_type;

        self.module.add_function("vpp_print_int", void.fn_type(&[i64.into()], false), None);
        self.module.add_function("vpp_print_float", void.fn_type(&[f64.into()], false), None);
        self.module.add_function("vpp_print_bool", void.fn_type(&[i32.into()], false), None);
        self.module.add_function("vpp_print_str", void.fn_type(&[str_ptr.into()], false), None);
        self.module.add_function(
            "vpp_string_new",
            str_ptr.fn_type(&[i8_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_string_concat",
            str_ptr.fn_type(&[str_ptr.into(), str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_string_retain",
            str_ptr.fn_type(&[str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_string_release",
            void.fn_type(&[str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_make_array",
            arr_ptr.fn_type(&[i64.into(), i64.into()], false),
            None,
        );
        self.module.add_function("vpp_array_len", i64.fn_type(&[arr_ptr.into()], false), None);
        self.module.add_function(
            "vpp_array_data",
            i8_ptr.fn_type(&[arr_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_array_index_ptr",
            i8_ptr.fn_type(&[arr_ptr.into(), i64.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_array_retain",
            arr_ptr.fn_type(&[arr_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_array_release",
            void.fn_type(&[arr_ptr.into()], false),
            None,
        );
        self.module.add_function("vpp_strlen", i64.fn_type(&[str_ptr.into()], false), None);
        self.module.add_function(
            "vpp_assert_fail",
            void.fn_type(&[i8_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_assert_eq_fail",
            void.fn_type(&[i8_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_read_file",
            str_ptr.fn_type(&[str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_write_file",
            void.fn_type(&[str_ptr.into(), str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_file_exists",
            i32.fn_type(&[str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_json_parse",
            str_ptr.fn_type(&[str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_json_stringify",
            str_ptr.fn_type(&[str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_process_run",
            i64.fn_type(&[str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_command_run",
            i64.fn_type(
                &[str_ptr.into(), arr_ptr.into(), str_ptr.into(), i64.into()],
                false,
            ),
            None,
        );
        self.module.add_function(
            "vpp_command_stdout",
            str_ptr.fn_type(&[], false),
            None,
        );
        self.module.add_function(
            "vpp_command_stderr",
            str_ptr.fn_type(&[], false),
            None,
        );
        self.module.add_function("vpp_env_get", str_ptr.fn_type(&[str_ptr.into()], false), None);
        self.module.add_function(
            "vpp_env_set",
            void.fn_type(&[str_ptr.into(), str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_dir_list",
            arr_ptr.fn_type(&[str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_dir_exists",
            i32.fn_type(&[str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_dir_create",
            void.fn_type(&[str_ptr.into()], false),
            None,
        );
        self.module.add_function(
            "vpp_log_line",
            void.fn_type(&[str_ptr.into(), str_ptr.into()], false),
            None,
        );
        Ok(())
    }

    fn compile_functions(&mut self, ir: &IrModule) -> VppResult<()> {
        for func in &ir.functions {
            let param_types: Vec<IrType> = func.params.iter().map(|(_, t)| t.clone()).collect();
            let fn_type = self.function_type(&param_types, &func.ret);
            let llvm_name = if func.name == "main" {
                "vpp_user_main"
            } else {
                &func.name
            };
            let function = self.module.add_function(llvm_name, fn_type, None);
            for (i, (name, _)) in func.params.iter().enumerate() {
                function.get_nth_param(i as u32).unwrap().set_name(name);
            }
            self.functions.insert(func.name.clone(), function);
        }

        for func in &ir.functions {
            self.compile_function_body(func)?;
        }
        Ok(())
    }

    fn compile_function_body(&mut self, func: &IrFunction) -> VppResult<()> {
        let function = *self.functions.get(&func.name).unwrap();
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        self.enter_scope();
        for (i, (name, ty)) in func.params.iter().enumerate() {
            let alloca = self.builder.build_alloca(self.llvm_value_type(ty), name).unwrap();
            let param = function.get_nth_param(i as u32).unwrap();
            self.builder.build_store(alloca, param).unwrap();
            self.define_local(name, alloca, ty.clone());
        }

        for stmt in &func.body {
            self.compile_stmt(stmt)?;
        }

        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.exit_scope();
            self.builder
                .build_return(Some(&self.default_value(&func.ret)))
                .unwrap();
        } else {
            self.clear_scope_state();
        }

        function.verify(true);
        Ok(())
    }

    fn compile_main(&mut self, ir: &IrModule) -> VppResult<()> {
        let has_user_main = ir.functions.iter().any(|f| f.name == "main");
        let fn_type = self.i64_type.fn_type(&[], false);
        let function = self
            .module
            .add_function("main", fn_type, Some(Linkage::External));
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        self.enter_scope();
        for stmt in &ir.top_level {
            self.compile_stmt(stmt)?;
        }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.exit_scope();
            if has_user_main {
                let user_main = *self.functions.get("main").unwrap();
                let ret = self
                    .builder
                    .build_call(user_main, &[], "user_main")
                    .unwrap();
                self.builder
                    .build_return(Some(&self.call_value(ret)))
                    .unwrap();
            } else {
                self.builder
                    .build_return(Some(&self.i64_type.const_int(0, false)))
                    .unwrap();
            }
        } else {
            self.clear_scope_state();
        }
        function.verify(true);
        Ok(())
    }

    pub(super) fn compile_stmt(&mut self, stmt: &IrStmt) -> VppResult<()> {
        match stmt {
            IrStmt::Let { name, ty, value } => {
                let val = self.compile_value(value)?;
                let alloca = self
                    .builder
                    .build_alloca(self.llvm_value_type(ty), name)
                    .unwrap();
                self.builder.build_store(alloca, val).unwrap();
                if ty.is_heap() {
                    self.heap_names_stack
                        .last_mut()
                        .expect("scope")
                        .push(name.clone());
                }
                self.define_local(name, alloca, ty.clone());
            }
            IrStmt::Expr(v) => {
                self.compile_value(v)?;
            }
            IrStmt::If { cond, then_body, else_body } => {
                let cond_v = self.compile_value(cond)?;
                let cond_i1 = self.to_bool(cond_v);
                let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                let then_bb = self.context.append_basic_block(function, "then");
                let else_bb = self.context.append_basic_block(function, "else");
                let merge_bb = self.context.append_basic_block(function, "merge");
                self.builder
                    .build_conditional_branch(cond_i1, then_bb, else_bb)
                    .unwrap();

                self.builder.position_at_end(then_bb);
                self.enter_scope();
                for s in then_body {
                    self.compile_stmt(s)?;
                }
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.exit_scope();
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                self.builder.position_at_end(else_bb);
                self.enter_scope();
                if let Some(stmts) = else_body {
                    for s in stmts {
                        self.compile_stmt(s)?;
                    }
                }
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.exit_scope();
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                self.builder.position_at_end(merge_bb);
            }
            IrStmt::While { cond, body } => {
                let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                let cond_bb = self.context.append_basic_block(function, "while.cond");
                let body_bb = self.context.append_basic_block(function, "while.body");
                let end_bb = self.context.append_basic_block(function, "while.end");
                self.builder.build_unconditional_branch(cond_bb).unwrap();

                self.push_loop(cond_bb, end_bb);

                self.builder.position_at_end(cond_bb);
                let cond_val = self.compile_value(cond)?;
                let c = self.to_bool(cond_val);
                self.builder
                    .build_conditional_branch(c, body_bb, end_bb)
                    .unwrap();

                self.builder.position_at_end(body_bb);
                self.enter_scope();
                for s in body {
                    self.compile_stmt(s)?;
                }
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.exit_scope();
                    self.builder.build_unconditional_branch(cond_bb).unwrap();
                }
                self.builder.position_at_end(end_bb);
                self.pop_loop();
            }
            IrStmt::ForInt { var, start, end, body } => {
                let alloca = self
                    .builder
                    .build_alloca(self.i64_type, var)
                    .unwrap();
                self.builder
                    .build_store(alloca, self.i64_type.const_int(*start as u64, true))
                    .unwrap();
                self.define_local(var, alloca, IrType::Int);

                let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                let cond_bb = self.context.append_basic_block(function, "for.cond");
                let body_bb = self.context.append_basic_block(function, "for.body");
                let inc_bb = self.context.append_basic_block(function, "for.inc");
                let end_bb = self.context.append_basic_block(function, "for.end");
                self.builder.build_unconditional_branch(cond_bb).unwrap();

                self.push_loop(inc_bb, end_bb);

                self.builder.position_at_end(cond_bb);
                let cur = self.builder.build_load(self.i64_type, alloca, var).unwrap();
                let cmp = self
                    .builder
                    .build_int_compare(
                        IntPredicate::SLT,
                        cur.into_int_value(),
                        self.i64_type.const_int(*end as u64, true),
                        "for.cmp",
                    )
                    .unwrap();
                self.builder
                    .build_conditional_branch(cmp, body_bb, end_bb)
                    .unwrap();

                self.builder.position_at_end(body_bb);
                self.enter_scope();
                for s in body {
                    self.compile_stmt(s)?;
                }
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.exit_scope();
                    self.builder.build_unconditional_branch(inc_bb).unwrap();
                }

                self.builder.position_at_end(inc_bb);
                let cur = self.builder.build_load(self.i64_type, alloca, var).unwrap();
                let next = self
                    .builder
                    .build_int_add(cur.into_int_value(), self.i64_type.const_int(1, true), "inc")
                    .unwrap();
                self.builder.build_store(alloca, next).unwrap();
                self.builder.build_unconditional_branch(cond_bb).unwrap();
                self.builder.position_at_end(end_bb);
                self.pop_loop();
            }
            IrStmt::ForArray { var, array, elem_ty, body } => {
                self.compile_for_array(var, array, elem_ty, body)?;
            }
            IrStmt::Return { value } => {
                while self.locals_stack.len() > 1 {
                    self.exit_scope();
                }
                let ret_val = if let Some(v) = value {
                    Some(self.compile_value(v)?)
                } else {
                    None
                };
                self.release_current_scope_heap();
                if let Some(compiled) = ret_val {
                    self.builder.build_return(Some(&compiled)).unwrap();
                } else {
                    self.builder
                        .build_return(Some(&self.i64_type.const_int(0, true)))
                        .unwrap();
                }
            }
            IrStmt::Block(stmts) => {
                self.enter_scope();
                for s in stmts {
                    self.compile_stmt(s)?;
                }
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.exit_scope();
                }
            }
            IrStmt::Break => self.compile_break()?,
            IrStmt::Continue => self.compile_continue()?,
            IrStmt::Match { scrutinee, arms, .. } => {
                self.compile_match_stmt(scrutinee, arms)?;
            }
        }
        Ok(())
    }

    fn compile_for_array(
        &mut self,
        var: &str,
        array: &IrValue,
        elem_ty: &IrType,
        body: &[IrStmt],
    ) -> VppResult<()> {
        let arr_val = self.compile_value(array)?;
        let arr_ptr = self.as_array_ptr(arr_val)?;

        let len_fn = self.module.get_function("vpp_array_len").unwrap();
        let len = self.call_value(
            self.builder
                .build_call(len_fn, &[arr_ptr.into()], "len")
                .unwrap(),
        );

        let idx_alloca = self.builder.build_alloca(self.i64_type, "idx").unwrap();
        self.builder
            .build_store(idx_alloca, self.i64_type.const_int(0, true))
            .unwrap();

        let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let cond_bb = self.context.append_basic_block(function, "farr.cond");
        let body_bb = self.context.append_basic_block(function, "farr.body");
        let inc_bb = self.context.append_basic_block(function, "farr.inc");
        let end_bb = self.context.append_basic_block(function, "farr.end");
        self.builder.build_unconditional_branch(cond_bb).unwrap();

        self.push_loop(inc_bb, end_bb);

        self.builder.position_at_end(cond_bb);
        let idx = self.builder.build_load(self.i64_type, idx_alloca, "idx").unwrap();
        let cmp = self
            .builder
            .build_int_compare(
                IntPredicate::SLT,
                idx.into_int_value(),
                len.into_int_value(),
                "farr.cmp",
            )
            .unwrap();
        self.builder
            .build_conditional_branch(cmp, body_bb, end_bb)
            .unwrap();

        self.builder.position_at_end(body_bb);
        let idx = self.builder.build_load(self.i64_type, idx_alloca, "idx").unwrap();
        let elem_val = self.load_array_elem(arr_ptr, idx.into_int_value(), elem_ty)?;
        let elem_alloca = self
            .builder
            .build_alloca(self.llvm_value_type(elem_ty), var)
            .unwrap();
        self.builder.build_store(elem_alloca, elem_val).unwrap();
        self.enter_scope();
        self.define_local(var, elem_alloca, elem_ty.clone());
        for s in body {
            self.compile_stmt(s)?;
        }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.exit_scope();
            self.builder.build_unconditional_branch(inc_bb).unwrap();
        }

        self.builder.position_at_end(inc_bb);
        let idx = self.builder.build_load(self.i64_type, idx_alloca, "idx").unwrap();
        let next = self
            .builder
            .build_int_add(idx.into_int_value(), self.i64_type.const_int(1, true), "inc")
            .unwrap();
        self.builder.build_store(idx_alloca, next).unwrap();
        self.builder.build_unconditional_branch(cond_bb).unwrap();
        self.builder.position_at_end(end_bb);
        self.pop_loop();
        Ok(())
    }

    pub(super) fn compile_value(&mut self, value: &IrValue) -> VppResult<BasicValueEnum<'ctx>> {
        match value {
            IrValue::Int(v) => Ok(self.i64_type.const_int(*v as u64, true).into()),
            IrValue::Float(v) => Ok(self.f64_type.const_float(*v).into()),
            IrValue::Bool(v) => Ok(self
                .i1_type
                .const_int(if *v { 1 } else { 0 }, false)
                .into()),
            IrValue::String(s) => self.compile_string_lit(s),
            IrValue::Local { name, ty } => {
                let (ptr, _) = self.lookup_local(name).ok_or_else(|| VppError::Other {
                    message: format!("codegen: undefined local `{name}`"),
                })?;
                Ok(self
                    .builder
                    .build_load(self.llvm_value_type(ty), ptr, name)
                    .unwrap())
            }
            IrValue::Binary { op, left, right, ty } => {
                let l = self.compile_value(left)?;
                let r = self.compile_value(right)?;
                self.compile_binary(*op, l, r, &left.ty(), &right.ty(), ty)
            }
            IrValue::Unary { op, expr, ty } => {
                let val = self.compile_value(expr)?;
                match op {
                    crate::ast::UnOp::Not => Ok(self.builder.build_not(self.to_bool(val), "not").unwrap().into()),
                    crate::ast::UnOp::Neg => {
                        if val.is_int_value() {
                            Ok(self
                                .builder
                                .build_int_neg(val.into_int_value(), "neg")
                                .unwrap()
                                .into())
                        } else {
                            Ok(self
                                .builder
                                .build_float_neg(val.into_float_value(), "neg")
                                .unwrap()
                                .into())
                        }
                    }
                }
            }
            IrValue::Call { name, args, ty } => self.compile_call(name, args, ty),
            IrValue::Index { target, index, ty } => {
                let target_val = self.compile_value(target)?;
                let arr_ptr = self.as_array_ptr(target_val)?;
                let idx = self.compile_value(index)?.into_int_value();
                self.load_array_elem(arr_ptr, idx, ty.elem_type())
            }
            IrValue::Array { elements, ty } => self.compile_array_lit(elements, ty),
            IrValue::Assign { name, value, .. } => {
                let val = self.compile_value(value)?;
                let (ptr, _) = self.lookup_local(name).ok_or_else(|| VppError::Other {
                    message: format!("codegen: undefined local `{name}`"),
                })?;
                self.builder.build_store(ptr, val).unwrap();
                Ok(val)
            }
            IrValue::Field { target, field, ty } => self.compile_field(target, field, ty),
            IrValue::StructLit { name, fields, ty } => self.compile_struct_lit(name, fields, ty),
            IrValue::Variant {
                enum_name,
                variant,
                payload,
                ty,
            } => self.compile_variant(enum_name, variant, payload, ty),
            IrValue::Match {
                scrutinee,
                arms,
                ty,
            } => self.compile_match_expr(scrutinee, arms, ty),
        }
    }

    fn compile_string_lit(&mut self, s: &str) -> VppResult<BasicValueEnum<'ctx>> {
        let cstr = self
            .builder
            .build_global_string_ptr(s, "strlit")
            .unwrap()
            .as_pointer_value();
        let new_fn = self.module.get_function("vpp_string_new").unwrap();
        Ok(self.call_value(
            self.builder
                .build_call(new_fn, &[cstr.into()], "str")
                .unwrap(),
        ))
    }

    fn compile_array_lit(&mut self, elements: &[IrValue], ty: &IrType) -> VppResult<BasicValueEnum<'ctx>> {
        let IrType::Array(elem_ty) = ty else {
            return Err(VppError::Other {
                message: "expected array type".to_string(),
            });
        };
        let len = elements.len() as i64;
        let elem_size = self.type_size(elem_ty) as i64;
        let make_fn = self.module.get_function("vpp_make_array").unwrap();
        let arr = self
            .call_value(
                self.builder
                    .build_call(
                        make_fn,
                        &[
                            self.i64_type.const_int(len as u64, true).into(),
                            self.i64_type.const_int(elem_size as u64, true).into(),
                        ],
                        "arr",
                    )
                    .unwrap(),
            )
            .into_pointer_value();

        let data_fn = self.module.get_function("vpp_array_data").unwrap();
        let data = self
            .call_value(
                self.builder
                    .build_call(data_fn, &[arr.into()], "data")
                    .unwrap(),
            )
            .into_pointer_value();
        let typed_data = self
            .builder
            .build_pointer_cast(
                data,
                self.llvm_elem_type(elem_ty)
                    .ptr_type(AddressSpace::default()),
                "arr_data",
            )
            .unwrap();

        for (i, elem) in elements.iter().enumerate() {
            let mut val = self.compile_value(elem)?;
            if **elem_ty == IrType::String {
                let retain_fn = self.module.get_function("vpp_string_retain").unwrap();
                val = self.call_value(
                    self.builder
                        .build_call(retain_fn, &[val.into()], "retain")
                        .unwrap(),
                );
            }
            let ptr = unsafe {
                self.builder
                    .build_gep(
                        self.llvm_elem_type(elem_ty),
                        typed_data,
                        &[self.i64_type.const_int(i as u64, true)],
                        "slot",
                    )
                    .unwrap()
            };
            self.builder.build_store(ptr, val).unwrap();
        }
        Ok(arr.into())
    }

    fn compile_call(&mut self, name: &str, args: &[IrValue], ret_ty: &IrType) -> VppResult<BasicValueEnum<'ctx>> {
        if let Some(builtin) = crate::builtins::lookup(name) {
            return match builtin.kind {
                BuiltinKind::Print => self.emit_print(args),
                BuiltinKind::Len => self.emit_len(args),
                BuiltinKind::Assert => self.emit_assert(args),
                BuiltinKind::AssertEq => self.emit_assert_eq(args),
                BuiltinKind::ReadFile => self.emit_unary_runtime("vpp_read_file", args, ret_ty),
                BuiltinKind::WriteFile => self.emit_binary_runtime_void("vpp_write_file", args),
                BuiltinKind::FileExists => self.emit_file_exists(args),
                BuiltinKind::JsonParse => self.emit_unary_runtime("vpp_json_parse", args, ret_ty),
                BuiltinKind::JsonStringify => self.emit_unary_runtime("vpp_json_stringify", args, ret_ty),
                BuiltinKind::ProcessRun => self.emit_unary_runtime("vpp_process_run", args, ret_ty),
                BuiltinKind::CommandRun => self.emit_command_run(args),
                BuiltinKind::CommandStdout => {
                    self.emit_nullary_runtime("vpp_command_stdout", ret_ty)
                }
                BuiltinKind::CommandStderr => {
                    self.emit_nullary_runtime("vpp_command_stderr", ret_ty)
                }
                BuiltinKind::EnvGet => self.emit_unary_runtime("vpp_env_get", args, ret_ty),
                BuiltinKind::EnvSet => self.emit_binary_runtime_void("vpp_env_set", args),
                BuiltinKind::DirList => self.emit_unary_runtime("vpp_dir_list", args, ret_ty),
                BuiltinKind::DirExists => self.emit_dir_exists(args),
                BuiltinKind::DirCreate => self.emit_unary_runtime_void("vpp_dir_create", args),
                BuiltinKind::LogLine => self.emit_binary_runtime_void("vpp_log_line", args),
                _ => Err(VppError::Other {
                    message: format!("native codegen: `{name}` not supported yet"),
                }),
            };
        }

        let function = *self.functions.get(name).ok_or_else(|| VppError::Other {
            message: format!("undefined function `{name}`"),
        })?;
        let compiled: Vec<BasicMetadataValueEnum> = args
            .iter()
            .map(|a| {
                let mut v = self.compile_value(a)?;
                if a.ty().is_array() {
                    let retain = self.module.get_function("vpp_array_retain").unwrap();
                    v = self.call_value(
                        self.builder
                            .build_call(retain, &[v.into()], "arr_retain")
                            .unwrap(),
                    );
                }
                Ok(v.into())
            })
            .collect::<VppResult<_>>()?;
        Ok(self.call_value(
            self.builder
                .build_call(function, &compiled, "call")
                .unwrap(),
        ))
    }

    fn emit_print(&mut self, args: &[IrValue]) -> VppResult<BasicValueEnum<'ctx>> {
        for arg in args {
            match arg.ty() {
                IrType::Int => {
                    let v = self.compile_value(arg)?.into_int_value();
                    let f = self.module.get_function("vpp_print_int").unwrap();
                    self.builder.build_call(f, &[v.into()], "print").unwrap();
                }
                IrType::Float => {
                    let v = self.compile_value(arg)?.into_float_value();
                    let f = self.module.get_function("vpp_print_float").unwrap();
                    self.builder.build_call(f, &[v.into()], "print").unwrap();
                }
                IrType::Bool => {
                    let compiled = self.compile_value(arg)?;
                    let v = self.to_i32_bool(compiled);
                    let f = self.module.get_function("vpp_print_bool").unwrap();
                    self.builder.build_call(f, &[v.into()], "print").unwrap();
                }
                IrType::String => {
                    let v = self.compile_value(arg)?.into_pointer_value();
                    let f = self.module.get_function("vpp_print_str").unwrap();
                    self.builder.build_call(f, &[v.into()], "print").unwrap();
                }
                other => {
                    return Err(VppError::Other {
                        message: format!("cannot print type {}", other.name()),
                    });
                }
            }
        }
        Ok(self.i64_type.const_int(0, true).into())
    }

    fn emit_len(&mut self, args: &[IrValue]) -> VppResult<BasicValueEnum<'ctx>> {
        let arg = &args[0];
        match arg.ty() {
            IrType::String => {
                let ptr = self.compile_value(arg)?.into_pointer_value();
                let f = self.module.get_function("vpp_strlen").unwrap();
                Ok(self.call_value(
                    self.builder.build_call(f, &[ptr.into()], "len").unwrap(),
                ))
            }
            IrType::Array(_) => {
                let compiled = self.compile_value(arg)?;
                let ptr = self.as_array_ptr(compiled)?;
                let f = self.module.get_function("vpp_array_len").unwrap();
                Ok(self.call_value(
                    self.builder.build_call(f, &[ptr.into()], "len").unwrap(),
                ))
            }
            other => Err(VppError::Other {
                message: format!("len unsupported for {}", other.name()),
            }),
        }
    }

    fn emit_unary_runtime(
        &mut self,
        runtime_fn: &str,
        args: &[IrValue],
        _ret_ty: &IrType,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let arg = self.compile_value(&args[0])?;
        let f = self.module.get_function(runtime_fn).unwrap();
        Ok(self.call_value(
            self.builder.build_call(f, &[arg.into()], runtime_fn).unwrap(),
        ))
    }

    fn emit_binary_runtime_void(
        &mut self,
        runtime_fn: &str,
        args: &[IrValue],
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let a = self.compile_value(&args[0])?;
        let b = self.compile_value(&args[1])?;
        let f = self.module.get_function(runtime_fn).unwrap();
        self.builder
            .build_call(f, &[a.into(), b.into()], runtime_fn)
            .unwrap();
        Ok(self.i64_type.const_int(0, true).into())
    }

    fn emit_unary_runtime_void(
        &mut self,
        runtime_fn: &str,
        args: &[IrValue],
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let a = self.compile_value(&args[0])?;
        let f = self.module.get_function(runtime_fn).unwrap();
        self.builder.build_call(f, &[a.into()], runtime_fn).unwrap();
        Ok(self.i64_type.const_int(0, true).into())
    }

    fn emit_nullary_runtime(
        &mut self,
        runtime_fn: &str,
        _ret_ty: &IrType,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let f = self.module.get_function(runtime_fn).unwrap();
        Ok(self.call_value(
            self.builder.build_call(f, &[], runtime_fn).unwrap(),
        ))
    }

    fn emit_command_run(&mut self, args: &[IrValue]) -> VppResult<BasicValueEnum<'ctx>> {
        let program = self.compile_value(&args[0])?;
        let mut argv = self.compile_value(&args[1])?;
        if args[1].ty().is_array() {
            let retain = self.module.get_function("vpp_array_retain").unwrap();
            argv = self.call_value(
                self.builder
                    .build_call(retain, &[argv.into()], "arr_retain")
                    .unwrap(),
            );
        }
        let cwd = self.compile_value(&args[2])?;
        let timeout = self.compile_value(&args[3])?;
        let f = self.module.get_function("vpp_command_run").unwrap();
        Ok(self.call_value(
            self.builder
                .build_call(
                    f,
                    &[program.into(), argv.into(), cwd.into(), timeout.into()],
                    "command_run",
                )
                .unwrap(),
        ))
    }

    fn emit_dir_exists(&mut self, args: &[IrValue]) -> VppResult<BasicValueEnum<'ctx>> {
        let raw = self.emit_unary_runtime("vpp_dir_exists", args, &IrType::Int)?;
        Ok(self
            .builder
            .build_int_compare(
                IntPredicate::NE,
                raw.into_int_value(),
                self.context.i32_type().const_int(0, false),
                "dir_exists",
            )
            .unwrap()
            .into())
    }

    fn emit_file_exists(&mut self, args: &[IrValue]) -> VppResult<BasicValueEnum<'ctx>> {
        let raw = self.emit_unary_runtime("vpp_file_exists", args, &IrType::Int)?;
        Ok(self
            .builder
            .build_int_compare(
                IntPredicate::NE,
                raw.into_int_value(),
                self.context.i32_type().const_int(0, false),
                "file_exists",
            )
            .unwrap()
            .into())
    }

    fn emit_assert(&mut self, args: &[IrValue]) -> VppResult<BasicValueEnum<'ctx>> {
        let cond_val = self.compile_value(&args[0])?;
        let cond = self.to_bool(cond_val);
        let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let ok_bb = self.context.append_basic_block(function, "assert.ok");
        let fail_bb = self.context.append_basic_block(function, "assert.fail");
        self.builder
            .build_conditional_branch(cond, ok_bb, fail_bb)
            .unwrap();
        self.builder.position_at_end(fail_bb);
        let msg = self.builder.build_global_string_ptr("condition is false", "assert_msg").unwrap();
        let f = self.module.get_function("vpp_assert_fail").unwrap();
        self.builder
            .build_call(f, &[msg.as_pointer_value().into()], "fail")
            .unwrap();
        self.builder.build_unreachable().unwrap();
        self.builder.position_at_end(ok_bb);
        Ok(self.i64_type.const_int(0, true).into())
    }

    fn emit_assert_eq(&mut self, args: &[IrValue]) -> VppResult<BasicValueEnum<'ctx>> {
        let left = self.compile_value(&args[0])?;
        let right = self.compile_value(&args[1])?;
        let eq = self.values_equal(left, right, &args[0].ty())?;
        let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let ok_bb = self.context.append_basic_block(function, "assert_eq.ok");
        let fail_bb = self.context.append_basic_block(function, "assert_eq.fail");
        self.builder
            .build_conditional_branch(eq, ok_bb, fail_bb)
            .unwrap();
        self.builder.position_at_end(fail_bb);
        let msg = self
            .builder
            .build_global_string_ptr("values not equal", "assert_eq_msg")
            .unwrap();
        let f = self.module.get_function("vpp_assert_eq_fail").unwrap();
        self.builder
            .build_call(f, &[msg.as_pointer_value().into()], "fail")
            .unwrap();
        self.builder.build_unreachable().unwrap();
        self.builder.position_at_end(ok_bb);
        Ok(self.i64_type.const_int(0, true).into())
    }

    pub(super) fn values_equal(
        &self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        ty: &IrType,
    ) -> VppResult<inkwell::values::IntValue<'ctx>> {
        Ok(match ty {
            IrType::Float => self
                .builder
                .build_float_compare(
                    FloatPredicate::OEQ,
                    left.into_float_value(),
                    right.into_float_value(),
                    "feq",
                )
                .unwrap(),
            IrType::Bool => self
                .builder
                .build_int_compare(
                    IntPredicate::EQ,
                    self.to_bool(left),
                    self.to_bool(right),
                    "beq",
                )
                .unwrap(),
            IrType::String => {
                // Compare C strings inside VppString  -  MVP: len+byte compare via runtime helper later
                // For v0.2 bootstrap: pointer identity for assert_eq on strings is weak; use int compare of data ptr
                let l = left.into_pointer_value();
                let r = right.into_pointer_value();
                self.builder
                    .build_int_compare(IntPredicate::EQ, l, r, "seq")
                    .unwrap()
            }
            _ => self
                .builder
                .build_int_compare(
                    IntPredicate::EQ,
                    left.into_int_value(),
                    right.into_int_value(),
                    "ieq",
                )
                .unwrap(),
        })
    }

    fn compile_binary(
        &self,
        op: BinOp,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        left_ty: &IrType,
        right_ty: &IrType,
        result_ty: &IrType,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let operand_ty = if *left_ty == IrType::Float || *right_ty == IrType::Float {
            IrType::Float
        } else if *left_ty == IrType::Bool && *right_ty == IrType::Bool {
            IrType::Bool
        } else if *left_ty == IrType::String && *right_ty == IrType::String {
            IrType::String
        } else {
            IrType::Int
        };

        match op {
            BinOp::Add if *result_ty == IrType::String => {
                let a = left.into_pointer_value();
                let b = right.into_pointer_value();
                let f = self.module.get_function("vpp_string_concat").unwrap();
                Ok(self.call_value(
                    self.builder.build_call(f, &[a.into(), b.into()], "concat").unwrap(),
                ))
            }
            BinOp::Add if left.is_int_value() => Ok(self
                .builder
                .build_int_add(left.into_int_value(), right.into_int_value(), "add")
                .unwrap()
                .into()),
            BinOp::Add => Ok(self
                .builder
                .build_float_add(left.into_float_value(), right.into_float_value(), "add")
                .unwrap()
                .into()),
            BinOp::Sub if left.is_int_value() => Ok(self
                .builder
                .build_int_sub(left.into_int_value(), right.into_int_value(), "sub")
                .unwrap()
                .into()),
            BinOp::Sub => Ok(self
                .builder
                .build_float_sub(left.into_float_value(), right.into_float_value(), "sub")
                .unwrap()
                .into()),
            BinOp::Mul if left.is_int_value() => Ok(self
                .builder
                .build_int_mul(left.into_int_value(), right.into_int_value(), "mul")
                .unwrap()
                .into()),
            BinOp::Mul => Ok(self
                .builder
                .build_float_mul(left.into_float_value(), right.into_float_value(), "mul")
                .unwrap()
                .into()),
            BinOp::Div if left.is_int_value() => Ok(self
                .builder
                .build_int_signed_div(left.into_int_value(), right.into_int_value(), "div")
                .unwrap()
                .into()),
            BinOp::Div => Ok(self
                .builder
                .build_float_div(left.into_float_value(), right.into_float_value(), "div")
                .unwrap()
                .into()),
            BinOp::Mod => Ok(self
                .builder
                .build_int_signed_rem(left.into_int_value(), right.into_int_value(), "mod")
                .unwrap()
                .into()),
            BinOp::Eq => Ok(self.values_equal(left, right, &operand_ty)?.into()),
            BinOp::NotEq => {
                let eq = self.values_equal(left, right, &operand_ty)?;
                Ok(self.builder.build_not(eq, "ne").unwrap().into())
            }
            BinOp::Lt => self.build_cmp(IntPredicate::SLT, left, right),
            BinOp::LtEq => self.build_cmp(IntPredicate::SLE, left, right),
            BinOp::Gt => self.build_cmp(IntPredicate::SGT, left, right),
            BinOp::GtEq => self.build_cmp(IntPredicate::SGE, left, right),
            BinOp::And => Ok(self
                .builder
                .build_and(self.to_bool(left), self.to_bool(right), "and")
                .unwrap()
                .into()),
            BinOp::Or => Ok(self
                .builder
                .build_or(self.to_bool(left), self.to_bool(right), "or")
                .unwrap()
                .into()),
        }
    }

    fn build_cmp(
        &self,
        pred: IntPredicate,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        if left.is_float_value() || right.is_float_value() {
            let pred_f = match pred {
                IntPredicate::SLT => FloatPredicate::OLT,
                IntPredicate::SLE => FloatPredicate::OLE,
                IntPredicate::SGT => FloatPredicate::OGT,
                IntPredicate::SGE => FloatPredicate::OGE,
                _ => FloatPredicate::OEQ,
            };
            return Ok(self
                .builder
                .build_float_compare(pred_f, left.into_float_value(), right.into_float_value(), "fcmp")
                .unwrap()
                .into());
        }
        if left.is_int_value() {
            return Ok(self
                .builder
                .build_int_compare(pred, left.into_int_value(), right.into_int_value(), "icmp")
                .unwrap()
                .into());
        }
        Ok(self
            .builder
            .build_int_compare(
                pred,
                self.to_bool(left),
                self.to_bool(right),
                "bcmp",
            )
            .unwrap()
            .into())
    }

    fn emit_string_release(&self, ptr: PointerValue<'ctx>) {
        let f = self.module.get_function("vpp_string_release").unwrap();
        self.builder.build_call(f, &[ptr.into()], "release").unwrap();
    }

    fn emit_array_release(&self, ptr: PointerValue<'ctx>) {
        let f = self.module.get_function("vpp_array_release").unwrap();
        self.builder.build_call(f, &[ptr.into()], "release").unwrap();
    }

    fn load_array_elem(
        &mut self,
        arr_ptr: PointerValue<'ctx>,
        idx: inkwell::values::IntValue<'ctx>,
        elem_ty: &IrType,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let index_fn = self.module.get_function("vpp_array_index_ptr").unwrap();
        let raw = self
            .call_value(
                self.builder
                    .build_call(index_fn, &[arr_ptr.into(), idx.into()], "idx_ptr")
                    .unwrap(),
            )
            .into_pointer_value();
        let elem_ptr = self
            .builder
            .build_pointer_cast(
                raw,
                self.llvm_elem_type(elem_ty)
                    .ptr_type(AddressSpace::default()),
                "elem_ptr",
            )
            .unwrap();
        Ok(self
            .builder
            .build_load(self.llvm_elem_type(elem_ty), elem_ptr, "elem")
            .unwrap())
    }

    fn as_array_ptr(&self, val: BasicValueEnum<'ctx>) -> VppResult<PointerValue<'ctx>> {
        if val.is_pointer_value() {
            Ok(val.into_pointer_value())
        } else {
            Err(VppError::Other {
                message: "expected array pointer".to_string(),
            })
        }
    }

    fn to_bool(&self, val: BasicValueEnum<'ctx>) -> inkwell::values::IntValue<'ctx> {
        if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 1 {
            val.into_int_value()
        } else if val.is_int_value() {
            self.builder
                .build_int_compare(
                    IntPredicate::NE,
                    val.into_int_value(),
                    self.i64_type.const_int(0, true),
                    "tobool",
                )
                .unwrap()
        } else {
            val.into_int_value()
        }
    }

    fn to_i32_bool(&self, val: BasicValueEnum<'ctx>) -> inkwell::values::IntValue<'ctx> {
        self.builder
            .build_int_z_extend(self.to_bool(val), self.context.i32_type(), "b32")
            .unwrap()
    }

    pub(super) fn call_value(&self, call: CallSiteValue<'ctx>) -> BasicValueEnum<'ctx> {
        call.try_as_basic_value().unwrap_basic()
    }

    pub(super) fn default_value(&self, ty: &IrType) -> BasicValueEnum<'ctx> {
        match ty {
            IrType::Float => self.f64_type.const_float(0.0).into(),
            IrType::Bool => self.i1_type.const_int(0, false).into(),
            IrType::String => self.string_ptr_type.const_zero().into(),
            IrType::Array(_) => self.array_ptr_type.const_zero().into(),
            IrType::Struct { name } => self
                .struct_types
                .get(name)
                .map(|st| st.const_zero().into())
                .unwrap_or_else(|| self.i64_type.const_int(0, true).into()),
            IrType::Enum { name } => self
                .enum_types
                .get(name)
                .map(|st| st.const_zero().into())
                .unwrap_or_else(|| self.i64_type.const_int(0, true).into()),
            IrType::Void => self.i64_type.const_int(0, true).into(),
            _ => self.i64_type.const_int(0, true).into(),
        }
    }

    /// LLVM type for a v++ value stored in a local, parameter, or return slot.
    pub(super) fn llvm_value_type(&self, ty: &IrType) -> BasicTypeEnum<'ctx> {
        match ty {
            IrType::Int => self.i64_type.into(),
            IrType::Float => self.f64_type.into(),
            IrType::Bool => self.i1_type.into(),
            IrType::String => self.string_ptr_type.into(),
            IrType::Array(_) => self.array_ptr_type.into(),
            IrType::Struct { name } => self
                .struct_types
                .get(name)
                .copied()
                .map(BasicTypeEnum::from)
                .unwrap_or(self.i64_type.into()),
            IrType::Enum { name } => self
                .enum_types
                .get(name)
                .copied()
                .map(BasicTypeEnum::from)
                .unwrap_or(self.i64_type.into()),
            IrType::Void => self.i64_type.into(),
            IrType::Unknown => self.i64_type.into(),
        }
    }

    /// LLVM type for elements inside array storage.
    fn llvm_elem_type(&self, ty: &IrType) -> BasicTypeEnum<'ctx> {
        match ty {
            IrType::Array(inner) => self.llvm_elem_type(inner),
            _ => self.llvm_value_type(ty),
        }
    }

    fn function_type(&self, params: &[IrType], ret: &IrType) -> FunctionType<'ctx> {
        let param_types: Vec<BasicMetadataTypeEnum> = params
            .iter()
            .map(|t| self.llvm_value_type(t).into())
            .collect();
        self.llvm_value_type(ret).fn_type(&param_types, false)
    }

    fn type_size(&self, ty: &IrType) -> u64 {
        match ty {
            IrType::Int | IrType::Float | IrType::String => 8,
            IrType::Bool => 1,
            IrType::Array(inner) => self.type_size(inner),
            IrType::Struct { .. } | IrType::Enum { .. } => 8,
            _ => 8,
        }
    }
}

//! Struct, enum, match, break/continue codegen.

use inkwell::basic_block::BasicBlock;
use inkwell::types::BasicTypeEnum;
use inkwell::types::StructType;
use inkwell::values::BasicValueEnum;
use inkwell::values::IntValue;
use inkwell::IntPredicate;

use crate::error::{VppError, VppResult};
use crate::ir::{IrMatchArm, IrModule, IrPattern, IrStmt, IrType, IrValue};

use super::Emit;

impl<'ctx> Emit<'ctx> {
    pub(super) fn init_types_from_module(&mut self, ir: &IrModule) {
        self.struct_defs = ir.struct_defs.clone();
        self.enum_defs = ir.enum_defs.clone();
    }

    pub(super) fn build_all_types(&mut self) -> VppResult<()> {
        for (name, fields) in &self.struct_defs.clone() {
            let field_tys: Vec<BasicTypeEnum<'ctx>> = fields
                .iter()
                .map(|(_, t)| self.llvm_value_type(t))
                .collect();
            let st = self.context.struct_type(&field_tys, false);
            self.struct_types.insert(name.clone(), st);
        }

        for (name, variants) in &self.enum_defs.clone() {
            let st = self.build_enum_struct_type(name, variants);
            self.enum_types.insert(name.clone(), st);
            for (i, (vname, _)) in variants.iter().enumerate() {
                self.variant_tags
                    .insert((name.clone(), vname.clone()), i as i64);
            }
        }

        Ok(())
    }

    fn build_enum_struct_type(
        &self,
        enum_name: &str,
        variants: &[(String, Vec<IrType>)],
    ) -> StructType<'ctx> {
        let mut fields: Vec<BasicTypeEnum<'ctx>> = vec![self.i64_type.into()];

        if enum_name.starts_with("Result<") {
            let ok_ty = variants
                .iter()
                .find(|(n, _)| n == "Ok")
                .and_then(|(_, p)| p.first())
                .cloned()
                .unwrap_or(IrType::Int);
            let err_ty = variants
                .iter()
                .find(|(n, _)| n == "Err")
                .and_then(|(_, p)| p.first())
                .cloned()
                .unwrap_or(IrType::String);
            fields.push(self.llvm_value_type(&ok_ty));
            fields.push(self.llvm_value_type(&err_ty));
        } else {
            let max_payloads = variants.iter().map(|(_, p)| p.len()).max().unwrap_or(0);
            for i in 0..max_payloads {
                let mut payload_ty = IrType::Int;
                for (_, payloads) in variants {
                    if let Some(t) = payloads.get(i) {
                        payload_ty = t.clone();
                        break;
                    }
                }
                fields.push(self.llvm_value_type(&payload_ty));
            }
        }

        self.context.struct_type(&fields, false)
    }

    pub(super) fn compile_break(&mut self) -> VppResult<()> {
        let (_, break_bb) = self.loop_stack.last().ok_or_else(|| VppError::Other {
            message: "`break` outside loop".to_string(),
        })?;
        self.builder.build_unconditional_branch(*break_bb).unwrap();
        Ok(())
    }

    pub(super) fn compile_continue(&mut self) -> VppResult<()> {
        let (cont_bb, _) = self.loop_stack.last().ok_or_else(|| VppError::Other {
            message: "`continue` outside loop".to_string(),
        })?;
        self.builder.build_unconditional_branch(*cont_bb).unwrap();
        Ok(())
    }

    pub(super) fn compile_match_stmt(
        &mut self,
        scrutinee: &IrValue,
        arms: &[IrMatchArm],
    ) -> VppResult<()> {
        self.compile_match_internal(scrutinee, arms, None)?;
        Ok(())
    }

    pub(super) fn compile_match_expr(
        &mut self,
        scrutinee: &IrValue,
        arms: &[IrMatchArm],
        ty: &IrType,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        self.compile_match_internal(scrutinee, arms, Some(ty))
    }

    fn compile_match_internal(
        &mut self,
        scrutinee: &IrValue,
        arms: &[IrMatchArm],
        result_ty: Option<&IrType>,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let scrutinee_val = self.compile_value(scrutinee)?;
        let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let merge_bb = self.context.append_basic_block(function, "match.end");
        let fail_bb = self.context.append_basic_block(function, "match.fail");

        let mut prev_check_bb = self.builder.get_insert_block().unwrap();
        let mut phi_incoming: Vec<(BasicValueEnum<'ctx>, BasicBlock<'ctx>)> = Vec::new();

        for (i, arm) in arms.iter().enumerate() {
            let arm_bb = self.context.append_basic_block(function, &format!("match.arm{i}"));
            let next_check_bb = self.context.append_basic_block(function, &format!("match.chk{i}"));

            self.builder.position_at_end(prev_check_bb);
            let cond = self.emit_pattern_cond(scrutinee, &scrutinee_val, &arm.pattern)?;
            self.builder
                .build_conditional_branch(cond, arm_bb, next_check_bb)
                .unwrap();

            self.builder.position_at_end(arm_bb);
            self.enter_scope();
            self.bind_pattern_vars(&scrutinee_val, &arm.pattern)?;
            let mut arm_result = self.default_value(result_ty.unwrap_or(&IrType::Void));
            for stmt in &arm.body {
                if let IrStmt::Expr(v) = stmt {
                    arm_result = self.compile_value(v)?;
                } else {
                    self.compile_stmt(stmt)?;
                }
            }
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                self.exit_scope();
                self.builder.build_unconditional_branch(merge_bb).unwrap();
                if result_ty.is_some() {
                    phi_incoming.push((arm_result, self.builder.get_insert_block().unwrap()));
                }
            }

            prev_check_bb = next_check_bb;
        }

        self.builder.position_at_end(prev_check_bb);
        self.builder.build_unconditional_branch(fail_bb).unwrap();
        self.builder.position_at_end(fail_bb);
        let msg = self
            .builder
            .build_global_string_ptr("non-exhaustive match", "match_fail")
            .unwrap();
        let f = self.module.get_function("vpp_assert_fail").unwrap();
        self.builder
            .build_call(f, &[msg.as_pointer_value().into()], "fail")
            .unwrap();
        self.builder.build_unreachable().unwrap();

        self.builder.position_at_end(merge_bb);
        if let Some(ty) = result_ty {
            if phi_incoming.is_empty() {
                return Ok(self.default_value(ty));
            }
            let phi = self.builder.build_phi(self.llvm_value_type(ty), "match_phi").unwrap();
            for (val, bb) in phi_incoming {
                phi.add_incoming(&[(&val, bb)]);
            }
            Ok(phi.as_basic_value())
        } else {
            Ok(self.i64_type.const_int(0, true).into())
        }
    }

    fn emit_pattern_cond(
        &mut self,
        scrutinee: &IrValue,
        val: &BasicValueEnum<'ctx>,
        pattern: &IrPattern,
    ) -> VppResult<IntValue<'ctx>> {
        match pattern {
            IrPattern::Wildcard => Ok(self.i1_type.const_int(1, false)),
            IrPattern::Literal(lit) => {
                let expected = self.compile_value(lit)?;
                self.values_equal(val.clone(), expected, &lit.ty())
            }
            IrPattern::Variant {
                enum_name,
                variant,
                ..
            } => {
                let enum_ty = scrutinee.ty();
                let key_owned = enum_ty
                    .enum_key()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| enum_name.clone());
                let tag = self.variant_tag(&key_owned, variant);
                let loaded_tag = self.extract_enum_tag(&key_owned, val)?;
                Ok(self
                    .builder
                    .build_int_compare(IntPredicate::EQ, loaded_tag, tag, "vcmp")
                    .unwrap())
            }
            IrPattern::Struct {
                struct_name, ..
            } => {
                let _ = struct_name;
                Ok(self.i1_type.const_int(1, false))
            }
        }
    }

    fn bind_pattern_vars(
        &mut self,
        val: &BasicValueEnum<'ctx>,
        pattern: &IrPattern,
    ) -> VppResult<()> {
        match pattern {
            IrPattern::Wildcard | IrPattern::Literal(_) => Ok(()),
            IrPattern::Struct {
                struct_name,
                fields,
            } => {
                for (i, (field, binding, fty)) in fields.iter().enumerate() {
                    let fv = self.extract_struct_field(struct_name, val, field, i, fty)?;
                    let alloca = self
                        .builder
                        .build_alloca(self.llvm_value_type(fty), binding)
                        .unwrap();
                    self.builder.build_store(alloca, fv).unwrap();
                    if fty.is_heap() {
                        self.heap_names_stack
                            .last_mut()
                            .expect("scope")
                            .push(binding.clone());
                    }
                    self.define_local(binding, alloca, fty.clone());
                }
                Ok(())
            }
            IrPattern::Variant {
                enum_name,
                variant,
                bindings,
                payload_types,
            } => {
                for (i, (binding, pty)) in bindings.iter().zip(payload_types.iter()).enumerate() {
                    let pv = self.extract_variant_payload(enum_name, val, variant, i, pty)?;
                    let alloca = self
                        .builder
                        .build_alloca(self.llvm_value_type(pty), binding)
                        .unwrap();
                    self.builder.build_store(alloca, pv).unwrap();
                    if pty.is_heap() {
                        self.heap_names_stack
                            .last_mut()
                            .expect("scope")
                            .push(binding.clone());
                    }
                    self.define_local(binding, alloca, pty.clone());
                }
                Ok(())
            }
        }
    }

    pub(super) fn compile_field(
        &mut self,
        target: &IrValue,
        field: &str,
        _ty: &IrType,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let val = self.compile_value(target)?;
        let target_ty = target.ty();
        let struct_name = target_ty.struct_name().ok_or_else(|| VppError::Other {
            message: "field access on non-struct".to_string(),
        })?;
        let struct_name = struct_name.to_string();
        let idx = self
            .struct_defs
            .get(&struct_name)
            .and_then(|fields| fields.iter().position(|(n, _)| n == field))
            .ok_or_else(|| VppError::Other {
                message: format!("struct `{struct_name}` has no field `{field}`"),
            })?;
        let fty = self.struct_defs[&struct_name][idx].1.clone();
        self.extract_struct_field(&struct_name, &val, field, idx, &fty)
    }

    pub(super) fn compile_struct_lit(
        &mut self,
        name: &str,
        fields: &[(String, IrValue)],
        _ty: &IrType,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let def = self
            .struct_defs
            .get(name)
            .cloned()
            .ok_or_else(|| VppError::Other {
                message: format!("unknown struct `{name}`"),
            })?;
        let st = *self.struct_types.get(name).ok_or_else(|| VppError::Other {
            message: format!("struct type `{name}` not built"),
        })?;
        let mut agg = st.const_zero();

        for (field_name, field_val_ir) in fields {
            let idx = def
                .iter()
                .position(|(n, _)| n == field_name)
                .ok_or_else(|| VppError::Other {
                    message: format!("struct `{name}` has no field `{field_name}`"),
                })?;
            let fty = &def[idx].1;
            let mut val = self.compile_value(field_val_ir)?;
            if *fty == IrType::String {
                let retain = self.module.get_function("vpp_string_retain").unwrap();
                val = self.call_value(
                    self.builder
                        .build_call(retain, &[val.into()], "retain")
                        .unwrap(),
                );
            }
            agg = self
                .builder
                .build_insert_value(agg, val, idx as u32, field_name)
                .unwrap()
                .into_struct_value();
        }
        Ok(agg.into())
    }

    pub(super) fn compile_variant(
        &mut self,
        enum_name: &str,
        variant: &str,
        payload: &[IrValue],
        ty: &IrType,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let key = ty.enum_key().unwrap_or(enum_name);
        if !self.enum_defs.contains_key(key) {
            return Err(VppError::Other {
                message: format!("unknown enum `{key}`"),
            });
        }
        let tag = self.variant_tag(key, variant);
        let st = *self.enum_types.get(key).ok_or_else(|| VppError::Other {
            message: format!("enum type `{key}` not built"),
        })?;

        let mut agg = st.get_undef();
        agg = self
            .builder
            .build_insert_value(agg, tag, 0u32, "tag")
            .unwrap()
            .into_struct_value();

        if key.starts_with("Result<") {
            if variant == "Ok" {
                let mut v = self.compile_value(&payload[0])?;
                if payload[0].ty() == IrType::String {
                    v = self.retain_string(v);
                }
                agg = self
                    .builder
                    .build_insert_value(agg, v, 1u32, "ok")
                    .unwrap()
                    .into_struct_value();
            } else if variant == "Err" {
                let mut v = self.compile_value(&payload[0])?;
                if payload[0].ty() == IrType::String {
                    v = self.retain_string(v);
                }
                agg = self
                    .builder
                    .build_insert_value(agg, v, 2u32, "err")
                    .unwrap()
                    .into_struct_value();
            }
        } else if variant == "None" {
            // tag 0 only
        } else if variant == "Some" {
            let v = self.compile_value(&payload[0])?;
            agg = self
                .builder
                .build_insert_value(agg, v, 1u32, "some")
                .unwrap()
                .into_struct_value();
        } else {
            for (i, p) in payload.iter().enumerate() {
                let mut v = self.compile_value(p)?;
                if p.ty() == IrType::String {
                    v = self.retain_string(v);
                }
                agg = self
                    .builder
                    .build_insert_value(agg, v, (i + 1) as u32, "payload")
                    .unwrap()
                    .into_struct_value();
            }
        }

        Ok(agg.into())
    }

    fn variant_tag(&self, enum_name: &str, variant: &str) -> IntValue<'ctx> {
        let tag = self
            .variant_tags
            .get(&(enum_name.to_string(), variant.to_string()))
            .copied()
            .unwrap_or(0);
        self.i64_type.const_int(tag as u64, false)
    }

    fn extract_enum_tag(&self, enum_name: &str, val: &BasicValueEnum<'ctx>) -> VppResult<IntValue<'ctx>> {
        let _st = self.enum_types.get(enum_name).ok_or_else(|| VppError::Other {
            message: format!("unknown enum `{enum_name}`"),
        })?;
        Ok(self
            .builder
            .build_extract_value(val.into_struct_value(), 0, "tag")
            .unwrap()
            .into_int_value())
    }

    fn extract_variant_payload(
        &self,
        enum_name: &str,
        val: &BasicValueEnum<'ctx>,
        variant: &str,
        payload_index: usize,
        _pty: &IrType,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let idx = if enum_name.starts_with("Result<") {
            if variant == "Ok" {
                1u32
            } else {
                2u32
            }
        } else {
            (payload_index + 1) as u32
        };
        Ok(self
            .builder
            .build_extract_value(val.into_struct_value(), idx, "payload")
            .unwrap())
    }

    fn extract_struct_field(
        &self,
        struct_name: &str,
        val: &BasicValueEnum<'ctx>,
        _field: &str,
        index: usize,
        _fty: &IrType,
    ) -> VppResult<BasicValueEnum<'ctx>> {
        let _ = self.struct_types.get(struct_name);
        Ok(self
            .builder
            .build_extract_value(val.into_struct_value(), index as u32, "field")
            .unwrap())
    }

    pub(super) fn push_loop(&mut self, continue_bb: BasicBlock<'ctx>, break_bb: BasicBlock<'ctx>) {
        self.loop_stack.push((continue_bb, break_bb));
    }

    pub(super) fn pop_loop(&mut self) {
        self.loop_stack.pop();
    }

    fn retain_string(&self, val: BasicValueEnum<'ctx>) -> BasicValueEnum<'ctx> {
        let retain = self.module.get_function("vpp_string_retain").unwrap();
        self.call_value(
            self.builder
                .build_call(retain, &[val.into()], "retain")
                .unwrap(),
        )
    }
}

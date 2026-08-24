//! Runtime shim for v++ native compilation.
//! Heap strings/arrays use ARC  -  see MEMORY_MODEL.md.

use std::path::Path;

use crate::error::{span_to_source, VppError, VppResult};
use crate::types::TypedProgram;

#[cfg(feature = "codegen")]
pub fn generate_and_build(
    program: &TypedProgram,
    source_path: &Path,
    output: &Path,
    emit_ir: Option<&Path>,
) -> VppResult<()> {
    crate::codegen::compile(program, source_path, Some(output), emit_ir)
}

#[cfg(not(feature = "codegen"))]
pub fn generate_and_build(
    _program: &TypedProgram,
    _source_path: &Path,
    _output: &Path,
    _emit_ir: Option<&Path>,
) -> VppResult<()> {
    Err(VppError::Other {
        message: "codegen is disabled; rebuild with `--features codegen`".to_string(),
    })
}

pub fn runtime_c_source() -> &'static str {
    concat!(
        include_str!("../../runtime/vpp_runtime.c"),
        "\n",
        include_str!("../../runtime/vpp_automation.c"),
    )
}

pub fn emit_runtime_c(path: &Path) -> VppResult<()> {
    std::fs::write(path, runtime_c_source()).map_err(|source| VppError::Io { source })
}

pub fn codegen_error(source: &str, message: impl Into<String>, offset: usize) -> VppError {
    VppError::Codegen {
        message: message.into(),
        span: span_to_source(source, crate::span::Span::new(offset, offset + 1)),
    }
}

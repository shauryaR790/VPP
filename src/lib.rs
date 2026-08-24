pub mod debug;
pub mod watch;
pub mod bench;
pub mod automation;
pub mod ast;
pub mod builtins;
pub mod codegen;
pub mod doctor;
pub mod driver;
pub mod error;
pub mod fmt;
pub mod interp;
pub mod ir;
pub mod lexer;
pub mod pkg;
pub mod project;
#[cfg(feature = "lsp")]
pub mod lsp;
pub mod modules;
pub mod parser;
pub mod span;
pub mod symbols;
pub mod types;

#[cfg(all(feature = "codegen", target_os = "windows"))]
#[link(name = "vpp_llvm_stubs", kind = "static")]
extern "C" {
    fn vpp_force_llvm_stubs();
}

#[cfg(feature = "codegen")]
pub fn ensure_llvm_stubs_linked() {
    // Windows LLVM-C.dll omits target-init exports; stubs live in vpp_llvm_stubs.
    // Unix/macOS link real symbols via llvm-config in build.rs.
    #[cfg(target_os = "windows")]
    unsafe {
        vpp_force_llvm_stubs();
    }
}

pub use driver::{
    check, check_file, check_path, check_with_index, compile, format_source, init_project,
    list_project_tests, parse, project_entry, run, run_tests_in_project, emit_ir, CompileOptions,
    TestListing,
};
pub use pkg::{
    add_dependency, parse_manifest_toml, remove_dependency, resolve_and_lock, resolve_dependencies,
    resolve_from_registry, search_registry, update_dependencies, DependencySpec, Lockfile, Manifest,
    RegistryPackage,
};
pub use project::find_project_root;
pub use doctor::run_doctor;
pub use watch::watch_file;
pub use bench::bench_file;
pub use debug::{debug_dap, debug_file};
pub use error::{VppError, VppResult};

use std::path::{Path, PathBuf};

use crate::error::{VppError, VppResult};
use crate::modules::{LoadContext, LoadedProgram};
use crate::pkg::Manifest;
use crate::project::{find_project_root, load_manifest, std_search_paths};
use crate::types::{TypeChecker, TypedProgram};

pub struct CompileOptions {
    pub emit_ir: Option<PathBuf>,
    pub output: Option<PathBuf>,
}

pub fn load_context_for(path: &Path) -> LoadContext {
    let project_root = find_project_root(path);
    LoadContext {
        std_paths: std_search_paths(project_root.as_deref()),
        is_entry: false,
    }
}

pub fn project_entry(start: &Path) -> VppResult<(PathBuf, Manifest)> {
    let root = find_project_root(start).ok_or_else(|| VppError::Other {
        message: "not in a v++ project (no vpp.toml found)".to_string(),
    })?;
    let manifest = load_manifest(&root)?;
    Ok((root.join(&manifest.entry), manifest))
}

pub fn parse(source: &str) -> VppResult<crate::ast::Program> {
    let tokens = crate::lexer::Lexer::new(source).tokenize()?;
    crate::parser::Parser::new(source, tokens).parse_program()
}

pub fn typecheck(source: &str, program: &crate::ast::Program, source_file: &Path) -> VppResult<TypedProgram> {
    TypeChecker::with_file(source, source_file.to_path_buf()).check(program)
}

pub fn typecheck_with_modules(
    source: &str,
    program: &crate::ast::Program,
    source_file: &Path,
    modules: crate::modules::ModuleGraph,
) -> VppResult<TypedProgram> {
    TypeChecker::with_modules(source, source_file.to_path_buf(), modules).check(program)
}

pub fn check(source: &str) -> VppResult<TypedProgram> {
    check_file(source, Path::new("<source>"))
}

pub fn check_file(source: &str, source_path: &Path) -> VppResult<TypedProgram> {
    let program = parse(source)?;
    typecheck(source, &program, source_path)
}

pub fn check_path(source_path: &Path) -> VppResult<TypedProgram> {
    let ctx = load_context_for(source_path);
    let loaded = crate::modules::load_with_context(source_path, ctx)?;
    typecheck_with_modules(
        &loaded.source,
        &loaded.program,
        &loaded.entry_path,
        loaded.modules,
    )
}

pub fn load_program(path: &Path) -> VppResult<LoadedProgram> {
    let ctx = load_context_for(path);
    crate::modules::load_with_context(path, ctx)
}

#[cfg(feature = "codegen")]
pub fn emit_ir(_source: &str, source_path: &Path, ir_path: &Path) -> VppResult<()> {
    let typed = check_path(source_path)?;
    crate::codegen::compile(&typed, source_path, None, Some(ir_path))
}

#[cfg(not(feature = "codegen"))]
pub fn emit_ir(source: &str, _source_path: &Path, _ir_path: &Path) -> VppResult<()> {
    let _ = source;
    Err(VppError::Other {
        message: "codegen is disabled; rebuild with `--features codegen` to emit LLVM IR".to_string(),
    })
}

#[cfg(feature = "codegen")]
pub fn compile(source: &str, source_path: &Path, options: CompileOptions) -> VppResult<()> {
    let _ = source;
    let typed = check_path(source_path)?;

    if let Some(ir_path) = &options.emit_ir {
        if options.output.is_none() {
            return crate::codegen::compile(&typed, source_path, None, Some(ir_path));
        }
    }

    let output = options.output.unwrap_or_else(|| {
        source_path.with_extension(if cfg!(windows) { "exe" } else { "" })
    });

    crate::codegen::compile(
        &typed,
        source_path,
        Some(&output),
        options.emit_ir.as_deref(),
    )?;

    Ok(())
}

#[cfg(not(feature = "codegen"))]
pub fn compile(source: &str, source_path: &Path, _options: CompileOptions) -> VppResult<()> {
    let _ = (source, source_path);
    Err(VppError::Other {
        message: "native compile requires codegen; use `vpp run` (interpreter) or rebuild with `--features codegen`".to_string(),
    })
}

pub fn run(source: &str, source_path: &Path) -> VppResult<()> {
    let typed = if source_path.exists() {
        check_path(source_path)?
    } else {
        check_file(source, source_path)?
    };
    crate::interp::run(&typed)
}

#[derive(serde::Serialize)]
pub struct TestListing {
    pub file: PathBuf,
    pub tests: Vec<String>,
}

pub fn list_project_tests(start: &Path) -> VppResult<Vec<TestListing>> {
    let root = find_project_root(start).ok_or_else(|| VppError::Other {
        message: "not in a v++ project (no vpp.toml found)".to_string(),
    })?;

    let mut files = Vec::new();
    collect_vpp_files(&root.join("tests"), &mut files)?;
    if files.is_empty() {
        collect_vpp_files(&root.join("src"), &mut files)?;
    }

    let mut listings = Vec::new();
    for file in files {
        let typed = check_path(&file)?;
        if typed.tests.is_empty() {
            continue;
        }
        listings.push(TestListing {
            file: file.strip_prefix(&root).unwrap_or(&file).to_path_buf(),
            tests: typed.tests.iter().map(|t| t.name.clone()).collect(),
        });
    }
    Ok(listings)
}

pub fn run_tests_in_project(start: &Path) -> VppResult<()> {
    let root = find_project_root(start).ok_or_else(|| VppError::Other {
        message: "not in a v++ project (no vpp.toml found)".to_string(),
    })?;

    let mut files = Vec::new();
    collect_vpp_files(&root.join("tests"), &mut files)?;
    if files.is_empty() {
        collect_vpp_files(&root.join("src"), &mut files)?;
    }

    if files.is_empty() {
        return Err(VppError::Other {
            message: "no test files found in tests/".to_string(),
        });
    }

    let mut passed = 0usize;
    let mut failed = 0usize;

    for file in files {
        let typed = check_path(&file).map_err(|e| {
            VppError::Other {
                message: format!("{}: {e}", file.display()),
            }
        })?;
        match crate::interp::run_tests(&typed) {
            Ok(n) => {
                passed += n;
                println!("✓ {} ({} tests)", file.strip_prefix(&root).unwrap_or(&file).display(), n);
            }
            Err(e) => {
                failed += 1;
                eprintln!("✗ {}: {e}", file.display());
            }
        }
    }

    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        return Err(VppError::Other {
            message: format!("{failed} test file(s) failed"),
        });
    }
    Ok(())
}

fn collect_vpp_files(dir: &Path, out: &mut Vec<PathBuf>) -> VppResult<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir).map_err(|e| VppError::Other {
        message: format!("failed to read `{}`: {e}", dir.display()),
    })? {
        let entry = entry.map_err(|e| VppError::Other {
            message: e.to_string(),
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_vpp_files(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("vpp") {
            out.push(path);
        }
    }
    out.sort();
    Ok(())
}

pub fn format_source(source: &str) -> VppResult<String> {
    crate::fmt::format(source)
}

pub fn check_with_index(source: &str, source_path: &Path) -> VppResult<TypedProgram> {
    let ctx = load_context_for(source_path);
    if source_path.exists() {
        let disk = std::fs::read_to_string(source_path).unwrap_or_default();
        if disk == source {
            return check_path(source_path);
        }
        let entry = parse(source)?;
        let (program, modules) = crate::modules::resolve_imports(source_path, entry, ctx)?;
        typecheck_with_modules(source, &program, source_path, modules)
    } else {
        check_file(source, source_path)
    }
}

pub use crate::project::init_project;

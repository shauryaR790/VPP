use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::{ImportDecl, ImportSpec, Item, Program};
use crate::error::{VppError, VppResult};
use crate::lexer::Lexer;
use crate::parser::Parser;

#[derive(Debug, Clone, Default)]
pub struct LoadContext {
    pub std_paths: Vec<PathBuf>,
    pub is_entry: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleExports {
    pub functions: HashSet<String>,
    pub structs: HashSet<String>,
    pub enums: HashSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleGraph {
    pub namespaces: HashMap<String, ModuleExports>,
    /// Functions that must be called with a module prefix (`math.add`, not `add`).
    pub scoped_functions: HashSet<String>,
}

pub struct LoadedProgram {
    pub program: Program,
    pub source: String,
    pub entry_path: PathBuf,
    pub modules: ModuleGraph,
    pub sources: HashMap<PathBuf, String>,
}

pub fn load(entry_path: &Path) -> VppResult<LoadedProgram> {
    load_with_context(entry_path, LoadContext::default())
}

pub fn resolve_imports(
    entry_path: &Path,
    mut entry_program: Program,
    ctx: LoadContext,
) -> VppResult<(Program, ModuleGraph)> {
    let entry_path = entry_path.canonicalize().map_err(|e| VppError::Other {
        message: format!("cannot resolve path `{}`: {e}", entry_path.display()),
    })?;
    let mut visited = HashSet::new();
    let mut sources = HashMap::new();
    let mut modules = ModuleGraph::default();
    resolve_imports_recursive(
        &entry_path,
        &entry_path,
        &mut entry_program,
        &mut visited,
        &mut sources,
        &mut modules,
        &ctx,
    )?;
    Ok((entry_program, modules))
}

fn resolve_imports_recursive(
    path: &Path,
    entry_path: &Path,
    program: &mut Program,
    visited: &mut HashSet<PathBuf>,
    sources: &mut HashMap<PathBuf, String>,
    modules: &mut ModuleGraph,
    ctx: &LoadContext,
) -> VppResult<()> {
    let path = path.canonicalize().map_err(|e| VppError::Other {
        message: format!("cannot read `{}`: {e}", path.display()),
    })?;

    if !visited.insert(path.clone()) {
        return Err(VppError::ImportCycle {
            path: path.display().to_string(),
        });
    }

    let imports: Vec<ImportDecl> = program
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Import(import) = item {
                Some(import.clone())
            } else {
                None
            }
        })
        .collect();
    program.items.retain(|item| !matches!(item, Item::Import(_)));

    let base_dir = path.parent().unwrap_or(Path::new("."));
    for import in imports {
        match &import.spec {
            ImportSpec::FilePath(spec) => {
                let import_path = resolve_file_import(base_dir, spec, ctx)?;
                let child = load_file_program(&import_path)?;
                let mut imported = child;
                resolve_imports_recursive(
                    &import_path,
                    entry_path,
                    &mut imported,
                    visited,
                    sources,
                    modules,
                    ctx,
                )?;
                merge_legacy(program, imported);
            }
            ImportSpec::Module(segments) => {
                let import_path = resolve_module_import(base_dir, segments, ctx)?;
                if modules.namespaces.contains_key(&module_alias(segments)) {
                    let text = sources.get(&path).map(String::as_str).unwrap_or("");
                    return Err(VppError::DuplicateImport {
                        module: module_canonical_name(segments),
                        span: crate::error::span_to_source(text, import.span),
                    });
                }
                let mut imported = load_file_program(&import_path)?;
                resolve_imports_recursive(
                    &import_path,
                    entry_path,
                    &mut imported,
                    visited,
                    sources,
                    modules,
                    ctx,
                )?;
                let exports = collect_exports(&imported, false);
                for fname in &exports.functions {
                    modules.scoped_functions.insert(fname.clone());
                }
                modules
                    .namespaces
                    .insert(module_alias(segments), exports);
                merge_module_exports(program, &imported, false);
            }
        }
    }

    visited.remove(&path);
    Ok(())
}

fn load_file_program(path: &Path) -> VppResult<Program> {
    let text = std::fs::read_to_string(path).map_err(|e| VppError::Other {
        message: format!("failed to read `{}`: {e}", path.display()),
    })?;
    let tokens = Lexer::new(&text).tokenize()?;
    Parser::new(text, tokens).parse_program()
}

pub fn load_with_context(entry_path: &Path, mut ctx: LoadContext) -> VppResult<LoadedProgram> {
    let entry_path = entry_path.canonicalize().map_err(|e| VppError::Other {
        message: format!("cannot resolve path `{}`: {e}", entry_path.display()),
    })?;

    let mut visited = HashSet::new();
    let mut sources = HashMap::new();
    let mut modules = ModuleGraph::default();
    ctx.is_entry = true;
    let program = load_recursive(
        &entry_path,
        &entry_path,
        &mut visited,
        &mut sources,
        &mut modules,
        &ctx,
    )?;

    let source = sources.get(&entry_path).cloned().unwrap_or_default();

    Ok(LoadedProgram {
        program,
        source,
        entry_path,
        modules,
        sources,
    })
}

fn load_recursive(
    path: &Path,
    entry_path: &Path,
    visited: &mut HashSet<PathBuf>,
    sources: &mut HashMap<PathBuf, String>,
    modules: &mut ModuleGraph,
    ctx: &LoadContext,
) -> VppResult<Program> {
    let path = path.canonicalize().map_err(|e| VppError::Other {
        message: format!("cannot read `{}`: {e}", path.display()),
    })?;

    if !visited.insert(path.clone()) {
        return Err(VppError::ImportCycle {
            path: path.display().to_string(),
        });
    }

    let text = std::fs::read_to_string(&path).map_err(|e| VppError::Other {
        message: format!("failed to read `{}`: {e}", path.display()),
    })?;
    sources.insert(path.clone(), text.clone());

    let tokens = Lexer::new(&text).tokenize()?;
    let mut program = Parser::new(text.clone(), tokens).parse_program()?;

    let imports: Vec<ImportDecl> = program
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Import(import) = item {
                Some(import.clone())
            } else {
                None
            }
        })
        .collect();

    program.items.retain(|item| !matches!(item, Item::Import(_)));

    let base_dir = path.parent().unwrap_or(Path::new("."));

    for import in imports {
        match &import.spec {
            ImportSpec::FilePath(spec) => {
                let import_path = resolve_file_import(base_dir, spec, ctx)?;
                let child_ctx = LoadContext {
                    std_paths: ctx.std_paths.clone(),
                    is_entry: false,
                };
                let imported = load_recursive(
                    &import_path,
                    entry_path,
                    visited,
                    sources,
                    modules,
                    &child_ctx,
                )?;
                merge_legacy(&mut program, imported);
            }
            ImportSpec::Module(segments) => {
                let import_path = resolve_module_import(base_dir, segments, ctx)?;
                let canonical = module_canonical_name(segments);
                if modules.namespaces.contains_key(&module_alias(segments)) {
                    return Err(VppError::DuplicateImport {
                        module: canonical,
                        span: crate::error::span_to_source(&text, import.span),
                    });
                }
                let child_ctx = LoadContext {
                    std_paths: ctx.std_paths.clone(),
                    is_entry: false,
                };
                let imported = load_recursive(
                    &import_path,
                    entry_path,
                    visited,
                    sources,
                    modules,
                    &child_ctx,
                )?;
                let exports = collect_exports(&imported, false);
                modules
                    .namespaces
                    .insert(module_alias(segments), exports.clone());
                merge_module_exports(&mut program, &imported, false);
                for fname in &exports.functions {
                    modules.scoped_functions.insert(fname.clone());
                }
            }
        }
    }

    visited.remove(&path);
    Ok(program)
}

fn module_alias(segments: &[String]) -> String {
    segments.last().cloned().unwrap_or_else(|| "mod".to_string())
}

fn module_canonical_name(segments: &[String]) -> String {
    segments.join(".")
}

fn collect_exports(program: &Program, legacy: bool) -> ModuleExports {
    let mut exports = ModuleExports::default();
    for item in &program.items {
        match item {
            Item::Function(f) if legacy || f.public => {
                exports.functions.insert(f.name.clone());
            }
            Item::Struct(s) if legacy || s.public => {
                exports.structs.insert(s.name.clone());
            }
            Item::Enum(e) if legacy || e.public => {
                exports.enums.insert(e.name.clone());
            }
            _ => {}
        }
    }
    exports
}

fn merge_legacy(into: &mut Program, from: Program) {
    for item in from.items {
        if let Some(name) = item_name(&item) {
            if into_has_name(into, &name) {
                continue;
            }
        }
        into.items.push(item);
    }
}

fn merge_module_exports(into: &mut Program, from: &Program, legacy: bool) {
    for item in &from.items {
        let include = match item {
            Item::Function(f) => legacy || f.public,
            Item::Struct(s) => legacy || s.public,
            Item::Enum(e) => legacy || e.public,
            Item::Statement(_) | Item::Test(_) | Item::Trait(_) | Item::Impl(_) => false,
            Item::Import(_) => false,
        };
        if !include {
            continue;
        }
        if let Some(name) = item_name(item) {
            if into_has_name(into, &name) {
                continue;
            }
        }
        into.items.push(item.clone());
    }
}

fn item_name(item: &Item) -> Option<String> {
    match item {
        Item::Function(f) => Some(f.name.clone()),
        Item::Struct(s) => Some(s.name.clone()),
        Item::Enum(e) => Some(e.name.clone()),
        _ => None,
    }
}

fn into_has_name(into: &Program, name: &str) -> bool {
    into.items.iter().any(|item| item_name(item).as_deref() == Some(name))
}

fn resolve_file_import(base_dir: &Path, spec: &str, ctx: &LoadContext) -> VppResult<PathBuf> {
    let candidates = file_import_candidates(base_dir, spec, ctx);
    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }
    Err(VppError::ImportNotFound {
        spec: spec.to_string(),
        hint: "use a path relative to the importing file or std/…".to_string(),
    })
}

fn resolve_module_import(
    base_dir: &Path,
    segments: &[String],
    ctx: &LoadContext,
) -> VppResult<PathBuf> {
    let spec = segments.join("/");
    let candidates = module_import_candidates(base_dir, &spec, ctx);
    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }
    Err(VppError::ImportNotFound {
        spec: module_canonical_name(segments),
        hint: "module paths use dots, e.g. `import std.io` -> std/io.vpp".to_string(),
    })
}

fn file_import_candidates(base_dir: &Path, spec: &str, ctx: &LoadContext) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let file_name = if spec.ends_with(".vpp") {
        spec.to_string()
    } else {
        format!("{spec}.vpp")
    };

    paths.push(base_dir.join(&file_name));

    if spec.starts_with("std/") || spec.starts_with("std\\") {
        let rel = spec
            .strip_prefix("std/")
            .or_else(|| spec.strip_prefix("std\\"))
            .unwrap_or(spec);
        let rel_file = if rel.ends_with(".vpp") {
            rel.to_string()
        } else {
            format!("{rel}.vpp")
        };
        for std_root in &ctx.std_paths {
            paths.push(std_root.join(&rel_file));
        }
    }

    paths
}

fn module_import_candidates(base_dir: &Path, spec: &str, ctx: &LoadContext) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let rel = if spec.starts_with("std/") {
        spec.strip_prefix("std/").unwrap_or(spec)
    } else {
        spec
    };
    let file = if rel.ends_with(".vpp") {
        rel.to_string()
    } else {
        format!("{rel}.vpp")
    };

    if spec.starts_with("std/") {
        for std_root in &ctx.std_paths {
            paths.push(std_root.join(&file));
        }
    } else {
        paths.push(base_dir.join(&file));
    }

    paths
}

pub fn canonical_module_path(path: &Path, std_roots: &[PathBuf]) -> String {
    for root in std_roots {
        if let Ok(rel) = path.strip_prefix(root) {
            let s = rel.with_extension("").to_string_lossy().replace('\\', "/");
            return format!("std.{s}").replace('/', ".");
        }
    }
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_alias_uses_last_segment() {
        assert_eq!(
            module_alias(&["std".to_string(), "io".to_string()]),
            "io"
        );
    }
}

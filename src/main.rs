use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use miette::{Context, IntoDiagnostic};

#[derive(Parser)]
#[command(name = "vpp", version, about = "The v++ programming language")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Type-check a v++ file without generating code
    Check {
        file: PathBuf,
    },
    /// Compile a v++ project or file to an executable
    Build {
        file: Option<PathBuf>,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Compile and run a v++ file (or project entry when no file is given)
    Run {
        file: Option<PathBuf>,
    },
    /// Emit LLVM IR for debugging
    Compile {
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Format a v++ source file in place
    Fmt {
        file: PathBuf,
    },
    /// Start the v++ language server (stdio)
    Lsp,
    /// Run tests in the current v++ project
    Test {
        path: Option<PathBuf>,
        /// JSON test listing for IDE Test Explorer
        #[arg(long)]
        list: bool,
    },
    /// Create a new v++ project
    New {
        name: Option<String>,
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Create a new v++ project (alias for `new`)
    Init {
        name: Option<String>,
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Add a dependency to vpp.toml and update the lockfile
    Add {
        name: String,
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        git: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        branch: Option<String>,
        #[arg(long)]
        version: Option<String>,
    },
    /// Remove a dependency from vpp.toml
    Remove {
        name: String,
    },
    /// Resolve dependencies and refresh vpp.lock
    Update,
    /// Check toolchain and project health
    Doctor,
    /// Interactive read-eval-print loop (interpreter; same language as run/build)
    Repl,
    /// Re-run a file automatically whenever you save it (live dev loop)
    Watch {
        file: PathBuf,
        /// Debounce milliseconds between save detection and re-run
        #[arg(long, default_value_t = 300)]
        debounce_ms: u64,
    },
    /// Time repeated interpreter runs (same engine as run/repl/watch)
    Bench {
        file: PathBuf,
        #[arg(short, long, default_value_t = vpp::bench::DEFAULT_RUNS)]
        runs: u32,
    },
    /// Line debugger (interpreter; breakpoints, step, locals)
    Debug {
        file: PathBuf,
        /// Breakpoint line numbers (repeatable)
        #[arg(short, long = "break")]
        breakpoints: Vec<u32>,
        /// Debug Adapter Protocol on stdio (VS Code)
        #[arg(long, hide = true)]
        dap: bool,
    },
    /// Search the v++ package registry
    Search {
        query: String,
    },
}

fn read_source(path: &Path) -> miette::Result<String> {
    std::fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("failed to read `{}`", path.display()))
}

fn resolve_user_file(path: PathBuf) -> miette::Result<PathBuf> {
    if path.exists() {
        return Ok(path);
    }

    let cwd = std::env::current_dir().into_diagnostic()?;
    let name = path
        .file_name()
        .map(|s| s.to_owned())
        .unwrap_or_else(|| path.as_os_str().to_owned());

    for candidate in [
        cwd.join(&path),
        cwd.join("examples").join(&name),
        cwd.join("src").join(&name),
        cwd.join("tests").join(&name),
    ] {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(miette::miette!(
        "cannot find `{}`\n  hint: try `vpp run examples\\{}` or open the file and press F5",
        path.display(),
        name.to_string_lossy()
    ))
}

fn resolve_run_path(file: &Option<PathBuf>) -> miette::Result<PathBuf> {
    if let Some(path) = file {
        return resolve_user_file(path.clone());
    }
    let cwd = std::env::current_dir().into_diagnostic()?;
    if let Ok((entry, _)) = vpp::project_entry(&cwd) {
        return Ok(entry);
    }
    Err(miette::miette!(
        "no file given and no vpp.toml in this directory.\n\
         Usage: vpp run path/to/file.vpp\n\
         Or: vpp init  then  vpp run  (runs src/main.vpp)"
    ))
}

fn project_root() -> miette::Result<PathBuf> {
    let cwd = std::env::current_dir().into_diagnostic()?;
    vpp::find_project_root(&cwd).ok_or_else(|| {
        miette::miette!("not in a v++ project (no vpp.toml found)")
    })
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Check { file } => cmd_check(&file),
        Commands::Build { file, output } => cmd_build(file.as_ref(), output),
        Commands::Run { file } => cmd_run(file.as_ref()),
        Commands::Compile { file, output } => cmd_compile(&file, output),
        Commands::Fmt { file } => cmd_fmt(&file),
        Commands::Lsp => cmd_lsp(),
        Commands::Test { path, list } => cmd_test(path.as_deref(), list),
        Commands::New { name, path } | Commands::Init { name, path } => {
            cmd_init(name.as_deref(), path.as_deref())
        }
        Commands::Add {
            name,
            path,
            git,
            tag,
            branch,
            version,
        } => cmd_add(&name, path.as_deref(), git.as_deref(), tag.as_deref(), branch.as_deref(), version.as_deref()),
        Commands::Remove { name } => cmd_remove(&name),
        Commands::Update => cmd_update(),
        Commands::Doctor => cmd_doctor(),
        Commands::Repl => cmd_repl(),
        Commands::Watch { file, debounce_ms } => cmd_watch(&file, debounce_ms),
        Commands::Bench { file, runs } => cmd_bench(&file, runs),
        Commands::Debug {
            file,
            breakpoints,
            dap,
        } => cmd_debug(&file, &breakpoints, dap),
        Commands::Search { query } => cmd_search(&query),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(report) => {
            eprintln!("{report:?}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_check(file: &PathBuf) -> miette::Result<()> {
    let path = resolve_user_file(file.clone())?;
    vpp::check_path(&path)
        .map_err(|e| {
            if let Ok(source) = std::fs::read_to_string(&path) {
                e.with_source(source)
            } else {
                miette::Report::new(e)
            }
        })?;
    println!("✓ `{}` type-checks successfully", path.display());
    Ok(())
}

fn cmd_build(file: Option<&PathBuf>, output: Option<PathBuf>) -> miette::Result<()> {
    let path = resolve_run_path(&file.cloned())?;
    let source = read_source(&path)?;
    vpp::compile(
        &source,
        &path,
        vpp::CompileOptions {
            emit_ir: None,
            output,
        },
    )
    .map_err(|e| e.with_source(source))?;
    Ok(())
}

fn cmd_run(file: Option<&PathBuf>) -> miette::Result<()> {
    let path = resolve_run_path(&file.cloned())?;
    let source = read_source(&path)?;
    vpp::run(&source, &path).map_err(|e| e.with_source(source))
}

fn cmd_lsp() -> miette::Result<()> {
    #[cfg(feature = "lsp")]
    {
        tokio::runtime::Runtime::new()
            .into_diagnostic()?
            .block_on(vpp::lsp::run_server());
        Ok(())
    }
    #[cfg(not(feature = "lsp"))]
    {
        Err(miette::miette!(
            "LSP requires the `lsp` feature. Rebuild with: cargo build --features lsp\n\
             On Windows, use the MSVC toolchain (Visual Studio Build Tools) for best compatibility."
        ))
    }
}

fn cmd_test(path: Option<&Path>, list: bool) -> miette::Result<()> {
    let start = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    if list {
        let tests = vpp::list_project_tests(&start).map_err(miette::Report::new)?;
        println!("{}", serde_json::to_string_pretty(&tests).into_diagnostic()?);
        return Ok(());
    }
    vpp::run_tests_in_project(&start).map_err(miette::Report::new)
}

fn cmd_search(query: &str) -> miette::Result<()> {
    let root = project_root()?;
    let hits = vpp::search_registry(&root, query).map_err(miette::Report::new)?;
    if hits.is_empty() {
        println!("No packages matching `{query}`");
        return Ok(());
    }
    println!("v++ registry  -  {} hit(s) for `{query}`\n", hits.len());
    for pkg in hits {
        println!("  {} {}", pkg.name, pkg.version);
    }
    Ok(())
}

fn cmd_init(name: Option<&str>, path: Option<&Path>) -> miette::Result<()> {
    let dir = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    let name = name.unwrap_or(
        dir.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("myapp"),
    );
    vpp::init_project(&dir, name).map_err(miette::Report::new)?;
    println!("Created v++ project `{name}` in `{}`", dir.display());
    println!();
    println!("  cd {}", dir.display());
    println!("  vpp run          # run src/main.vpp");
    println!("  vpp test         # run tests in tests/");
    Ok(())
}

fn cmd_add(
    name: &str,
    path: Option<&Path>,
    git: Option<&str>,
    tag: Option<&str>,
    branch: Option<&str>,
    version: Option<&str>,
) -> miette::Result<()> {
    let root = project_root()?;
    let spec = if let Some(path) = path {
        vpp::DependencySpec::from_path(path)
    } else if let Some(git) = git {
        vpp::DependencySpec::from_git(
            git,
            tag.map(str::to_string),
            branch.map(str::to_string),
        )
    } else if let Some(version) = version {
        vpp::DependencySpec::Version(version.to_string())
    } else {
        return Err(miette::miette!(
            "specify --path, --git, or --version for dependency `{name}`"
        ));
    };
    vpp::add_dependency(&root, name, spec).map_err(miette::Report::new)?;
    println!("Added dependency `{name}` and updated vpp.lock");
    Ok(())
}

fn cmd_remove(name: &str) -> miette::Result<()> {
    let root = project_root()?;
    vpp::remove_dependency(&root, name).map_err(miette::Report::new)?;
    println!("Removed dependency `{name}`");
    Ok(())
}

fn cmd_update() -> miette::Result<()> {
    let root = project_root()?;
    vpp::update_dependencies(&root).map_err(miette::Report::new)?;
    println!("Updated vpp.lock");
    Ok(())
}

fn cmd_doctor() -> miette::Result<()> {
    let root = vpp::find_project_root(&std::env::current_dir().unwrap_or_default());
    vpp::run_doctor(root.as_deref()).map_err(miette::Report::new)
}

fn cmd_repl() -> miette::Result<()> {
    vpp::interp::run_repl().map_err(miette::Report::new)
}

fn cmd_watch(file: &PathBuf, debounce_ms: u64) -> miette::Result<()> {
    let path = resolve_user_file(file.clone())?;
    vpp::watch_file(&path, debounce_ms).map_err(miette::Report::new)
}

fn cmd_bench(file: &PathBuf, runs: u32) -> miette::Result<()> {
    let path = resolve_user_file(file.clone())?;
    vpp::bench_file(&path, runs).map_err(miette::Report::new)
}

fn cmd_debug(file: &PathBuf, breakpoints: &[u32], dap: bool) -> miette::Result<()> {
    let path = resolve_user_file(file.clone())?;
    if dap {
        vpp::debug_dap(&path).map_err(miette::Report::new)
    } else {
        vpp::debug_file(&path, breakpoints).map_err(miette::Report::new)
    }
}

fn cmd_compile(file: &PathBuf, output: Option<PathBuf>) -> miette::Result<()> {
    let source = read_source(file)?;
    let ir_path = output.unwrap_or_else(|| file.with_extension("ll"));
    vpp::emit_ir(&source, file, &ir_path)
        .map_err(|e| e.with_source(source))?;
    println!("Wrote LLVM IR to `{}`", ir_path.display());
    Ok(())
}

fn cmd_fmt(file: &PathBuf) -> miette::Result<()> {
    vpp::fmt::format_file(file).map_err(|e| {
        if let Ok(source) = std::fs::read_to_string(file) {
            e.with_source(source)
        } else {
            miette::Report::new(e)
        }
    })?;
    println!("Formatted `{}`", file.display());
    Ok(())
}

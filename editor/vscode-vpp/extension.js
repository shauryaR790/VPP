const vscode = require("vscode");
const { execFile, exec } = require("child_process");
const path = require("path");
const fs = require("fs");
const os = require("os");

/** @type {import("vscode-languageclient/node").LanguageClient | undefined} */
let languageClient;
/** @type {vscode.OutputChannel | undefined} */
let outputChannel;
/** @type {vscode.StatusBarItem | undefined} */
let toolchainStatus;
/** @type {vscode.StatusBarItem | undefined} */
let lspStatus;
/** @type {{ root: string, runner: { kind: string, path: string } } | undefined} */
let runnerCache;
let lspStarted = false;

function getOutput() {
  if (!outputChannel) {
    outputChannel = vscode.window.createOutputChannel("v++");
  }
  return outputChannel;
}

/** @returns {string | undefined} */
function workspaceRoot() {
  const folders = vscode.workspace.workspaceFolders;
  if (!folders || folders.length === 0) {
    return undefined;
  }
  return folders[0].uri.fsPath;
}

function invalidateRunnerCache() {
  runnerCache = undefined;
}

/** @param {string} root */
function resolveRunner(root) {
  if (runnerCache && runnerCache.root === root) {
    return runnerCache.runner;
  }

  const config = vscode.workspace.getConfiguration("vpp");
  const configured = config.get("compilerPath", "");
  if (configured && fs.existsSync(configured)) {
    runnerCache = { root, runner: { kind: "exe", path: configured } };
    return runnerCache.runner;
  }

  const ps1 = path.join(root, "vpp.ps1");
  if (fs.existsSync(ps1)) {
    runnerCache = { root, runner: { kind: "ps1", path: ps1 } };
    return runnerCache.runner;
  }

  const cmd = path.join(root, "vpp.cmd");
  if (fs.existsSync(cmd)) {
    runnerCache = { root, runner: { kind: "cmd", path: cmd } };
    return runnerCache.runner;
  }

  for (const sub of [
    "target/release/vpp.exe",
    "target/debug/vpp.exe",
    "target/release/vpp",
    "target/debug/vpp",
  ]) {
    const candidate = path.join(root, sub);
    if (fs.existsSync(candidate)) {
      runnerCache = { root, runner: { kind: "exe", path: candidate } };
      return runnerCache.runner;
    }
  }

  return undefined;
}

/** @param {string} root @returns {string | undefined} */
function resolveLanguageServer(root) {
  const config = vscode.workspace.getConfiguration("vpp");
  const configured = config.get("languageServerPath", "vppls");

  if (path.isAbsolute(configured)) {
    return fs.existsSync(configured) ? configured : undefined;
  }

  for (const sub of [
    `target/debug/${configured}.exe`,
    `target/release/${configured}.exe`,
    `target/debug/${configured}`,
    `target/release/${configured}`,
  ]) {
    const candidate = path.join(root, sub);
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  return undefined;
}

/** @param {{ kind: string, path: string }} runner @param {string[]} args */
function runRunner(runner, args) {
  if (runner.kind === "ps1") {
    return {
      command: "powershell",
      argv: ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", runner.path, ...args],
    };
  }
  if (runner.kind === "cmd") {
    return {
      command: "cmd",
      argv: ["/c", runner.path, ...args],
    };
  }
  return {
    command: runner.path,
    argv: args,
  };
}

/** @param {string} subcommand @param {string} filePath */
async function runVpp(subcommand, filePath) {
  const root = workspaceRoot();
  if (!root) {
    vscode.window.showErrorMessage("Open the v++ project folder first.");
    return false;
  }

  if (!filePath || !filePath.endsWith(".vpp")) {
    vscode.window.showErrorMessage("Open a .vpp file first.");
    return false;
  }

  let runner = resolveRunner(root);
  if (!runner) {
    const choice = await vscode.window.showInformationMessage(
      "Build v++ compiler now? (one time, ~30 seconds)",
      "Build",
      "Cancel"
    );
    if (choice !== "Build") {
      return false;
    }
    await buildCompiler(root);
    invalidateRunnerCache();
    runner = resolveRunner(root);
    if (!runner) {
      vscode.window.showErrorMessage("Build finished but vpp was not found. Run .\\setup.ps1 in the project folder.");
      return false;
    }
  }

  const output = getOutput();
  output.clear();
  output.show(true);
  output.appendLine(`> vpp ${subcommand} ${filePath}`);
  output.appendLine("");

  const { command, argv } = runRunner(runner, [subcommand, filePath]);
  const execOpts = { cwd: root, maxBuffer: 10 * 1024 * 1024 };

  return new Promise((resolve) => {
    execFile(command, argv, execOpts, (err, stdout, stderr) => {
      if (stdout) {
        output.append(stdout);
      }
      if (stderr) {
        output.append(stderr);
      }
      if (err) {
        output.appendLine("");
        output.appendLine(`Exit code: ${err.code ?? 1}`);
        vscode.window.showErrorMessage(
          `v++ ${subcommand} failed  -  see the "v++" output panel`
        );
        resolve(false);
      } else {
        if (subcommand === "run" && !stdout.trim()) {
          output.appendLine("(program finished with no stdout)");
        } else if (subcommand === "check") {
          vscode.window.showInformationMessage("File type-checks successfully");
        }
        resolve(true);
      }
    });
  });
}

/** @param {vscode.TextDocument} document */
async function formatDocument(document) {
  const filePath = document.uri.fsPath;
  if (!filePath.endsWith(".vpp")) {
    return [];
  }

  const root = workspaceRoot();
  if (!root) {
    return [];
  }

  const runner = resolveRunner(root);
  if (!runner) {
    vscode.window.showWarningMessage("v++ compiler not found  -  cannot format.");
    return [];
  }

  // Format via a temp copy so we never touch the open file on disk (avoids save conflicts).
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "vpp-fmt-"));
  const tmpFile = path.join(tmpDir, path.basename(filePath));
  try {
    fs.writeFileSync(tmpFile, document.getText(), "utf8");

    const { command, argv } = runRunner(runner, ["fmt", tmpFile]);
    const execOpts = { cwd: root, maxBuffer: 10 * 1024 * 1024 };

    const ok = await new Promise((resolve) => {
      execFile(command, argv, execOpts, (err, stdout, stderr) => {
        const output = getOutput();
        if (stdout || stderr || err) {
          output.appendLine(`> vpp fmt ${path.basename(filePath)}`);
          if (stdout) {
            output.append(stdout);
          }
          if (stderr) {
            output.append(stderr);
          }
        }
        resolve(!err);
      });
    });

    if (!ok) {
      vscode.window.showErrorMessage("v++ fmt failed  -  see the v++ output panel");
      return [];
    }

    const formatted = fs.readFileSync(tmpFile, "utf8");
    if (formatted === document.getText()) {
      return [];
    }

    const fullRange = new vscode.Range(
      document.positionAt(0),
      document.positionAt(document.getText().length)
    );
    return [vscode.TextEdit.replace(fullRange, formatted)];
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
}

/** @param {string} root */
function buildCompiler(root) {
  return new Promise((resolve, reject) => {
    const cargoBin = path.join(process.env.USERPROFILE || process.env.HOME || "", ".cargo", "bin");
    const env = { ...process.env };
    if (fs.existsSync(cargoBin)) {
      env.PATH = `${cargoBin};${env.PATH || ""}`;
    }

    exec("cargo build --features lsp,codegen", { cwd: root, env }, (err, _stdout, stderr) => {
      if (err) {
        vscode.window.showErrorMessage(`cargo build failed: ${stderr || err.message}`);
        reject(err);
        return;
      }
      invalidateRunnerCache();
      resolve(undefined);
    });
  });
}

function updateStatusBar() {
  const root = workspaceRoot();
  if (!toolchainStatus) {
    return;
  }

  if (!root) {
    toolchainStatus.text = "$(symbol-method) v++";
    toolchainStatus.tooltip = "Open a v++ workspace";
    if (lspStatus) {
      lspStatus.hide();
    }
    return;
  }

  const runner = resolveRunner(root);
  if (runner) {
    const name = path.basename(runner.path);
    toolchainStatus.text = `$(zap) ${name}`;
    toolchainStatus.tooltip = `v++ compiler: ${runner.path}\nClick to open settings`;
  } else {
    toolchainStatus.text = "$(warning) v++ not found";
    toolchainStatus.tooltip = "v++ compiler not found  -  click to configure";
  }
  toolchainStatus.show();

  if (lspStatus) {
    if (languageClient && lspStarted) {
      lspStatus.text = "$(check) LSP";
      lspStatus.tooltip = "v++ language server running";
      lspStatus.show();
    } else if (vscode.workspace.getConfiguration("vpp").get("enableLanguageServer", true)) {
      lspStatus.text = "$(circle-outline) LSP";
      lspStatus.tooltip = "Language server idle or unavailable";
      lspStatus.show();
    } else {
      lspStatus.hide();
    }
  }
}

function startLanguageServer(context) {
  if (lspStarted) {
    return;
  }

  const root = workspaceRoot();
  if (!root) {
    return;
  }

  const config = vscode.workspace.getConfiguration("vpp");
  if (!config.get("enableLanguageServer", true)) {
    return;
  }

  const serverPath = resolveLanguageServer(root);
  if (!serverPath) {
    return;
  }

  let LanguageClient;
  let TransportKind;
  try {
    ({ LanguageClient, TransportKind } = require("vscode-languageclient/node"));
  } catch {
    return;
  }

  lspStarted = true;
  languageClient = new LanguageClient(
    "vppLanguageServer",
    "v++ Language Server",
    {
      command: serverPath,
      args: [],
      transport: TransportKind.stdio,
      options: { cwd: root },
    },
    {
      documentSelector: [{ scheme: "file", language: "vpp" }],
      synchronize: {
        fileEvents: vscode.workspace.createFileSystemWatcher("**/*.vpp"),
      },
      outputChannel: getOutput(),
    }
  );

  languageClient.start().then(
    () => updateStatusBar(),
    () => {
      lspStarted = false;
      updateStatusBar();
    }
  );

  context.subscriptions.push({
    dispose: () => {
      if (languageClient) {
        return languageClient.stop();
      }
    },
  });
}

function ensureLanguageServer(context) {
  startLanguageServer(context);
  updateStatusBar();
}

function setupTestExplorer(context) {
  if (!vscode.tests?.createTestController) {
    return;
  }

  const controller = vscode.tests.createTestController("vppTests", "v++ Tests");
  context.subscriptions.push(controller);

  /** @type {Map<string, import("vscode").TestItem>} */
  const itemsById = new Map();

  async function refreshTests() {
    controller.items.replace([]);
    itemsById.clear();
    const root = workspaceRoot();
    if (!root) {
      return;
    }
    const runner = resolveRunner(root);
    if (!runner) {
      return;
    }
    const { command, argv } = runRunner(runner, ["test", "--list"]);
    const json = await new Promise((resolve) => {
      execFile(command, argv, { cwd: root, maxBuffer: 4 * 1024 * 1024 }, (err, stdout) => {
        resolve(err ? "[]" : stdout);
      });
    });
    /** @type {{ file: string, tests: string[] }[]} */
    let listings = [];
    try {
      listings = JSON.parse(json);
    } catch {
      listings = [];
    }
    for (const entry of listings) {
      const fileUri = vscode.Uri.file(path.join(root, entry.file));
      let fileItem = itemsById.get(entry.file);
      if (!fileItem) {
        fileItem = controller.createTestItem(entry.file, path.basename(entry.file), fileUri);
        fileItem.canResolveChildren = false;
        controller.items.add(fileItem);
        itemsById.set(entry.file, fileItem);
      }
      for (const name of entry.tests) {
        const id = `${entry.file}::${name}`;
        const testItem = controller.createTestItem(id, name, fileUri);
        testItem.range = new vscode.Range(0, 0, 0, 0);
        fileItem.children.add(testItem);
        itemsById.set(id, testItem);
      }
    }
  }

  controller.createRunProfile(
    "Run v++ tests",
    vscode.TestRunProfileKind.Run,
    async (request, token) => {
      const root = workspaceRoot();
      if (!root) {
        return;
      }
      const runner = resolveRunner(root);
      if (!runner) {
        vscode.window.showErrorMessage("v++ compiler not found.");
        return;
      }
      const run = controller.createTestRun(request);
      const queue = [];
      if (request.include) {
        request.include.forEach((t) => queue.push(t));
      } else {
        controller.items.forEach((t) => queue.push(t));
      }
      for (const test of queue) {
        if (token.isCancellationRequested) {
          break;
        }
        if (test.children.size > 0) {
          test.children.forEach((c) => queue.push(c));
          continue;
        }
        run.started(test);
        const { command, argv } = runRunner(runner, ["test"]);
        const ok = await new Promise((resolve) => {
          execFile(command, argv, { cwd: root }, (err) => resolve(!err));
        });
        if (ok) {
          run.passed(test);
        } else {
          run.failed(test, new vscode.TestMessage("vpp test failed  -  see terminal"));
        }
      }
      run.end();
    }
  );

  refreshTests();
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (doc.languageId === "vpp") {
        refreshTests();
      }
    }),
    vscode.commands.registerCommand("vpp.refreshTests", refreshTests)
  );
}

function activate(context) {
  toolchainStatus = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 50);
  toolchainStatus.command = "vpp.openSettings";
  context.subscriptions.push(toolchainStatus);

  lspStatus = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 50);
  context.subscriptions.push(lspStatus);

  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory("vpp", {
      createDebugAdapterDescriptor(session) {
        const root = workspaceRoot();
        if (!root) {
          vscode.window.showErrorMessage("Open a v++ workspace to debug.");
          return undefined;
        }
        const runner = resolveRunner(root);
        if (!runner) {
          vscode.window.showErrorMessage("v++ compiler not found  -  run setup or set vpp.compilerPath.");
          return undefined;
        }
        const program = session.configuration.program;
        if (!program) {
          vscode.window.showErrorMessage("Debug configuration needs a program path.");
          return undefined;
        }
        const { command, argv } = runRunner(runner, ["debug", "--dap", program]);
        return new vscode.DebugAdapterExecutable(command, argv, { cwd: root });
      },
    })
  );

  updateStatusBar();

  const iconTheme = vscode.workspace.getConfiguration("workbench").get("iconTheme");
  if (!iconTheme) {
    vscode.workspace.getConfiguration("workbench").update("iconTheme", "vpp-icons", true);
  }

  setupTestExplorer(context);

  const maybeStartLsp = () => ensureLanguageServer(context);
  if (vscode.window.activeTextEditor?.document.languageId === "vpp") {
    maybeStartLsp();
  }

  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor?.document.languageId === "vpp") {
        maybeStartLsp();
      }
      updateStatusBar();
    }),
    vscode.workspace.onDidOpenTextDocument((doc) => {
      if (doc.languageId === "vpp") {
        maybeStartLsp();
      }
    }),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("vpp")) {
        invalidateRunnerCache();
        updateStatusBar();
      }
    }),
    vscode.workspace.onDidChangeWorkspaceFolders(() => {
      invalidateRunnerCache();
      updateStatusBar();
    }),
    vscode.languages.registerDocumentFormattingEditProvider("vpp", {
      provideDocumentFormattingEdits: (document) => formatDocument(document),
    }),
    vscode.workspace.onWillSaveTextDocument((event) => {
      if (event.document.languageId !== "vpp") {
        return;
      }
      const formatOnSave = vscode.workspace.getConfiguration("vpp").get("formatOnSave", true);
      if (formatOnSave) {
        event.waitUntil(formatDocument(event.document));
      }
    }),
    vscode.commands.registerCommand("vpp.debugFile", () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "vpp") {
        vscode.window.showErrorMessage("Open a .vpp file to debug.");
        return;
      }
      vscode.debug.startDebugging(undefined, {
        type: "vpp",
        request: "launch",
        name: "Debug v++ file",
        program: editor.document.uri.fsPath,
        stopOnEntry: true,
      });
    }),
    vscode.commands.registerCommand("vpp.runFile", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No file open.");
        return;
      }
      await runVpp("run", editor.document.uri.fsPath);
    }),
    vscode.commands.registerCommand("vpp.checkFile", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No file open.");
        return;
      }
      await runVpp("check", editor.document.uri.fsPath);
    }),
    vscode.commands.registerCommand("vpp.formatDocument", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "vpp") {
        return;
      }
      const edits = await formatDocument(editor.document);
      if (edits.length > 0) {
        await editor.edit((builder) => {
          for (const edit of edits) {
            builder.replace(edit.range, edit.newText);
          }
        });
      }
    }),
    vscode.commands.registerCommand("vpp.startRepl", () => {
      const root = workspaceRoot();
      if (!root) {
        vscode.window.showErrorMessage("Open the v++ project folder first.");
        return;
      }
      const runner = resolveRunner(root);
      if (!runner) {
        vscode.window.showErrorMessage("v++ compiler not found. Run .\\setup.ps1 once.");
        return;
      }
      const { command, argv } = runRunner(runner, ["repl"]);
      const term = vscode.window.createTerminal({ name: "v++ REPL", cwd: root });
      term.show(true);
      term.sendText([command, ...argv].join(" "));
    }),
    vscode.commands.registerCommand("vpp.watchFile", () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "vpp") {
        return;
      }
      const root = workspaceRoot();
      if (!root) {
        return;
      }
      const runner = resolveRunner(root);
      if (!runner) {
        vscode.window.showErrorMessage("v++ compiler not found.");
        return;
      }
      const file = editor.document.uri.fsPath;
      const { command, argv } = runRunner(runner, ["watch", file]);
      const term = vscode.window.createTerminal({ name: "v++ watch", cwd: root });
      term.show(true);
      term.sendText([command, ...argv].join(" "));
      vscode.window.showInformationMessage("Watching  -  save the file to re-run (Ctrl+C in terminal to stop)");
    }),
    vscode.commands.registerCommand("vpp.benchFile", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "vpp") {
        return;
      }
      await runVpp("bench", editor.document.uri.fsPath);
    }),
    vscode.commands.registerCommand("vpp.testProject", () => {
      const root = workspaceRoot();
      if (!root) {
        return;
      }
      const runner = resolveRunner(root);
      if (!runner) {
        vscode.window.showErrorMessage("v++ compiler not found. Run .\\setup.ps1 once.");
        return;
      }
      const { command, argv } = runRunner(runner, ["test"]);
      const term = vscode.window.createTerminal({ name: "vpp test", cwd: root });
      term.show(true);
      term.sendText([command, ...argv].join(" "));
    }),
    vscode.commands.registerCommand("vpp.showOutput", () => {
      getOutput().show(true);
    }),
    vscode.commands.registerCommand("vpp.openSettings", () => {
      vscode.commands.executeCommand("workbench.action.openSettings", "vpp");
    }),
    vscode.commands.registerCommand("vpp.searchPackages", async () => {
      const root = workspaceRoot();
      if (!root) {
        return;
      }
      const query = await vscode.window.showInputBox({
        prompt: "Search v++ registry",
        placeHolder: "package name",
      });
      if (!query) {
        return;
      }
      const runner = resolveRunner(root);
      if (!runner) {
        vscode.window.showErrorMessage("v++ compiler not found.");
        return;
      }
      const { command, argv } = runRunner(runner, ["search", query]);
      const term = vscode.window.createTerminal({ name: "v++ registry", cwd: root });
      term.show(true);
      term.sendText([command, ...argv].join(" "));
    }),
    vscode.commands.registerCommand("vpp.openDocs", () => {
      vscode.env.openExternal(vscode.Uri.parse("https://github.com/shauryaR790/V-/tree/main/docs"));
    })
  );

  const welcomeKey = "vpp.welcomeShown";
  if (!context.globalState.get(welcomeKey)) {
    context.globalState.update(welcomeKey, true);
    vscode.window
      .showInformationMessage(
        "v++ Language 1.0.0  -  stable. F5 debug, Test Explorer, registry search. Same .vpp for run/repl/watch/build.",
        "Open docs",
        "Settings"
      )
      .then((choice) => {
        if (choice === "Open docs") {
          vscode.commands.executeCommand("vpp.openDocs");
        } else if (choice === "Settings") {
          vscode.commands.executeCommand("vpp.openSettings");
        }
      });
  }
}

function deactivate() {
  if (languageClient) {
    return languageClient.stop();
  }
}

module.exports = { activate, deactivate };

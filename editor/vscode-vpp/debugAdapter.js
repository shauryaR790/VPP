// Spawns `vpp debug --dap`  -  DAP is implemented in the Rust compiler.
// VS Code talks JSON to vpp on stdio; no Node debug logic needed here.

const { spawn } = require("child_process");

/**
 * @param {string} command
 * @param {string[]} args
 */
function createAdapterProcess(command, args) {
  return spawn(command, args, { stdio: "pipe" });
}

module.exports = { createAdapterProcess };

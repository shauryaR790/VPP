/** Course playground  -  test program with terminal output. */
(function () {
  function normalizeSource(source) {
    return source.replace(/\r\n/g, "\n").trim();
  }

  function getSource(codeEl, payload) {
    return payload.source || (codeEl && codeEl.textContent) || "";
  }

  function readPayload(dataEl) {
    const raw = (dataEl.textContent || dataEl.innerHTML || "").trim();
    if (!raw) return null;
    try {
      return JSON.parse(raw);
    } catch {
      return null;
    }
  }

  function initCoursePlayground() {
    const dataEl = document.getElementById("course-playground-data");
    const playground = document.querySelector(".course-playground");
    const codeEl = document.querySelector(".course-source-code");
    if (!dataEl || !playground) return;

    const payload = readPayload(dataEl);
    if (!payload) {
      renderTerminal(
        playground.querySelector(".course-terminal-body"),
        "vpp run main.vpp",
        "Playground failed to load. Refresh the page.",
        "",
        true
      );
      return;
    }

    const terminalBody = playground.querySelector(".course-terminal-body");
    const runBtn = playground.querySelector(".course-run-btn");
    const resetBtn = playground.querySelector(".course-reset-btn");
    if (!terminalBody || !runBtn || !resetBtn) return;

    const cmd = payload.run_cmd || "vpp run main.vpp";

    const renderIdle = () => {
      renderTerminal(terminalBody, cmd, "Ready. Click Test program.", "", false);
    };

    renderIdle();

    runBtn.addEventListener("click", () => {
      runBtn.disabled = true;
      resetBtn.disabled = true;
      renderTerminal(terminalBody, cmd, "Running...", "", false);

      window.setTimeout(() => {
        const result = runSource(getSource(codeEl, payload), payload);
        renderTerminal(terminalBody, cmd, "", result.output, !result.ok);
        runBtn.disabled = false;
        resetBtn.disabled = false;
      }, 200);
    });

    resetBtn.addEventListener("click", () => {
      renderIdle();
      runBtn.disabled = false;
      resetBtn.disabled = false;
    });
  }

  function extractPrintCalls(source) {
    const results = [];
    const printRe = /print\s*\(/g;
    let match;
    while ((match = printRe.exec(source)) !== null) {
      let i = match.index + match[0].length;
      let depth = 1;
      let arg = "";
      while (i < source.length && depth > 0) {
        const ch = source[i];
        if (ch === "(") depth += 1;
        else if (ch === ")") depth -= 1;
        if (depth > 0) arg += ch;
        i += 1;
      }
      results.push(arg.trim());
    }
    return results;
  }

  function evalStringLiteral(expr) {
    const m = expr.match(/^"((?:\\.|[^"\\])*)"$|^'((?:\\.|[^'\\])*)'$/);
    if (!m) return null;
    const raw = m[1] !== undefined ? m[1] : m[2];
    return raw
      .replace(/\\n/g, "\n")
      .replace(/\\t/g, "\t")
      .replace(/\\"/g, '"')
      .replace(/\\\\/g, "\\");
  }

  function evalIntLiteral(expr) {
    if (/^-?\d+$/.test(expr.trim())) return parseInt(expr.trim(), 10);
    return null;
  }

  function evalBoolLiteral(expr) {
    const t = expr.trim();
    if (t === "true") return "true";
    if (t === "false") return "false";
    return null;
  }

  function buildEnv(source) {
    const env = {};
    const letRe = /let\s+(?:mut\s+)?(\w+)\s*=\s*([^;\n]+)/g;
    let m;
    while ((m = letRe.exec(source)) !== null) {
      const val = evalExpr(m[2].trim(), env, source);
      if (val !== null) env[m[1]] = val;
    }
    return env;
  }

  function evalExpr(expr, env, source) {
    expr = expr.trim();
    if (!expr) return null;

    const str = evalStringLiteral(expr);
    if (str !== null) return str;

    const num = evalIntLiteral(expr);
    if (num !== null) return num;

    const bool = evalBoolLiteral(expr);
    if (bool !== null) return bool;

    if (Object.prototype.hasOwnProperty.call(env, expr)) return env[expr];

    if (expr.includes("+")) {
      const parts = expr.split("+").map((p) => p.trim());
      if (parts.every((p) => p.length > 0)) {
        const vals = parts.map((p) => evalExpr(p, env, source));
        if (vals.every((v) => v !== null)) {
          if (vals.every((v) => typeof v === "number")) return vals.reduce((a, b) => a + b, 0);
          if (vals.every((v) => typeof v === "string")) return vals.join("");
        }
      }
    }

    if (expr.includes("-")) {
      const parts = expr.split("-").map((p) => p.trim());
      if (parts.length === 2 && parts[0] && parts[1]) {
        const a = evalExpr(parts[0], env, source);
        const b = evalExpr(parts[1], env, source);
        if (typeof a === "number" && typeof b === "number") return a - b;
      }
    }

    if (expr.includes("*")) {
      const parts = expr.split("*").map((p) => p.trim());
      if (parts.length === 2 && parts[0] && parts[1]) {
        const a = evalExpr(parts[0], env, source);
        const b = evalExpr(parts[1], env, source);
        if (typeof a === "number" && typeof b === "number") return a * b;
      }
    }

    const callMatch = expr.match(/^(\w+)\((.*)\)$/s);
    if (callMatch) {
      const fn = callMatch[1];
      const args = callMatch[2].split(",").map((a) => a.trim()).filter(Boolean);
      const fnBodyMatch = new RegExp(
        `fn\\s+${fn}\\s*\\([^)]*\\)[^{]*\\{([\\s\\S]*?)\\n\\}`,
        "m"
      ).exec(source);
      if (fnBodyMatch && args.length >= 0) {
        const localEnv = { ...env };
        const paramsMatch = new RegExp(`fn\\s+${fn}\\s*\\(([^)]*)\\)`).exec(source);
        if (paramsMatch) {
          const params = paramsMatch[1]
            .split(",")
            .map((p) => p.split(":")[0].trim())
            .filter(Boolean);
          params.forEach((name, idx) => {
            if (args[idx] !== undefined) localEnv[name] = evalExpr(args[idx], env, source);
          });
        }
        const retMatch = /return\s+([^;\n]+)/.exec(fnBodyMatch[1]);
        if (retMatch) return evalExpr(retMatch[1].trim(), localEnv, source);
      }
    }

    return null;
  }

  function interpretPrints(source) {
    const env = buildEnv(source);
    const prints = extractPrintCalls(source);
    if (!prints.length) return null;
    const lines = [];
    for (const arg of prints) {
      const val = evalExpr(arg, env, source);
      if (val === null) return null;
      lines.push(String(val));
    }
    return lines.join("\n");
  }

  function runSource(source, payload) {
    const normalized = normalizeSource(source);
    const original = normalizeSource(payload.source || "");

    if (original && normalized === original) {
      return { ok: true, output: payload.output || "" };
    }

    const interpreted = interpretPrints(source);
    if (interpreted !== null && interpreted.length > 0) {
      return { ok: true, output: interpreted };
    }

    if (payload.output) {
      return { ok: true, output: payload.output };
    }

    return {
      ok: false,
      output:
        "Could not run this code in the browser playground.\n" +
        "Install V++ locally and run: " +
        (payload.run_cmd || "vpp run main.vpp"),
    };
  }

  function renderTerminal(terminalBody, cmd, statusText, output, isError) {
    terminalBody.innerHTML =
      `<div class="course-terminal-line course-terminal-cmd">$ ${cmd}</div>` +
      (statusText
        ? `<div class="course-terminal-line course-terminal-muted">${statusText}</div>`
        : "") +
      `<pre class="course-run-output${isError ? " course-terminal-err" : ""}"></pre>`;
    const outputEl = terminalBody.querySelector(".course-run-output");
    if (outputEl && output) outputEl.textContent = output;
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initCoursePlayground);
  } else {
    initCoursePlayground();
  }
})();

/** Download hub  -  version/OS picker and one-click release links. */
(function () {
  const REPO = "shauryaR790/VPP";
  const SOURCE_URL = "/VPP/contribute.html";

  const ICONS = {
    version:
      '<svg viewBox="0 0 24 24" fill="none" stroke="#FBDB5A" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">'
      + '<path d="M12 2 2 7l10 5 10-5-10-5Z"/><path d="m2 17 10 5 10-5"/><path d="m2 12 10 5 10-5"/></svg>',
    windows:
      '<svg viewBox="0 0 24 24" aria-hidden="true">'
      + '<path fill="#00ADEF" d="M3 5.5 11 4.3v7.6H3V5.5Z"/>'
      + '<path fill="#00ADEF" d="M12 3.8 21 2.5v8.9H12V3.8Z"/>'
      + '<path fill="#00ADEF" d="M3 13.9h8v7.6l-8-1.2v-6.4Z"/>'
      + '<path fill="#00ADEF" d="M12 13.9h9v8.9l-9-1.3v-7.6Z"/></svg>',
    linux:
      '<svg viewBox="0 0 32 32" aria-hidden="true">'
      + '<path fill="#222" d="M16 3.5C11.3 3.5 7.5 7 7.5 11.5c0 2.3 1.2 4.3 3 5.5-.2.8-.3 1.5-.3 2.3 0 2.9 2.5 5.2 5.5 5.2h1.6c3 0 5.5-2.3 5.5-5.2 0-.8-.1-1.5-.3-2.3 1.8-1.2 3-3.2 3-5.5C25.5 7 21.7 3.5 16 3.5Z"/>'
      + '<ellipse fill="#fff" cx="16" cy="19.5" rx="4.8" ry="5.2"/>'
      + '<ellipse fill="#fff" cx="11.2" cy="11.8" rx="2.6" ry="3.1"/>'
      + '<ellipse fill="#fff" cx="20.8" cy="11.8" rx="2.6" ry="3.1"/>'
      + '<circle fill="#111" cx="11.8" cy="12.3" r="1.05"/>'
      + '<circle fill="#111" cx="20.2" cy="12.3" r="1.05"/>'
      + '<path fill="#E8850C" d="M13.8 16.2c.8.7 1.8.7 2.6 0 .5.8 1.4 1.3 2.4 1.3.5 0 .9-.1 1.3-.3-.5 1.6-1.9 2.7-3.5 2.7s-3-.1-3.5-2.7c.4.2.8.3 1.3.3 1 0 1.9-.5 2.4-1.3Z"/>'
      + '<path fill="#E8850C" d="M10.5 24.8 9 27.5h2.8l-.3-2.7Zm11 0 1.5 2.7H20.2l.3-2.7Z"/></svg>',
    macos:
      '<svg viewBox="0 0 24 24" fill="#f5f5f5" aria-hidden="true">'
      + '<path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/></svg>',
    installer:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">'
      + '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M12 8v8"/><path d="m8.5 12 3.5 3.5L15.5 12"/></svg>',
    zip:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">'
      + '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/>'
      + '<path d="M14 2v6h6"/><path d="M10 12h4"/><path d="M10 16h4"/></svg>',
    tarball:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">'
      + '<path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/>'
      + '<path d="M3.3 7 12 12l8.7-5"/><path d="M12 22V12"/></svg>',
    source:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">'
      + '<polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>',
    download:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">'
      + '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/></svg>',
  };

  const OS_ICONS = { windows: ICONS.windows, linux: ICONS.linux, macos: ICONS.macos };
  const FORMAT_ICONS = {
    installer: ICONS.installer,
    zip: ICONS.zip,
    tarball: ICONS.tarball,
    source: ICONS.source,
  };

  const PLATFORMS = {
    windows: {
      label: "Windows",
      formats: [
        {
          id: "installer",
          label: "Installer (.exe)",
          primary: true,
          file: (v) => `vpp-${v}-setup.exe`,
          lang: "powershell",
          filename: "terminal",
          install: (v) =>
            `# Run the downloaded vpp-${v}-setup.exe installer
# If SmartScreen appears: More info → Run anyway
# Then open a new terminal:
vpp run examples\\hello.vpp
vpp check examples\\hello.vpp
vpp --version
# Verify your install
vpp doctor`,
          info: "The Windows installer adds <code>vpp</code> to PATH automatically. Bundled LLVM is included for native builds.",
          pathNote: 'If <code>vpp</code> is not found after install, add the install folder to PATH:',
          showPath: true,
        },
        {
          id: "zip",
          label: "Portable (.zip)",
          file: (v) => `vpp-v${v}-windows-x64.zip`,
          lang: "powershell",
          filename: "terminal",
          install: (v) =>
            `# Extract vpp-v${v}-windows-x64.zip, then from that folder:
.\\GO.bat
vpp run examples\\hello.vpp
vpp check examples\\hello.vpp
vpp --version
# Verify your install
vpp doctor`,
          info: "Portable zip: extract anywhere. Run <code>GO.bat</code> or add the folder to PATH manually.",
        },
      ],
    },
    linux: {
      label: "Linux",
      formats: [
        {
          id: "tarball",
          label: "Linux x64 (.tar.gz)",
          primary: true,
          file: (v) => `vpp-v${v}-linux-x64.tar.gz`,
          lang: "bash",
          filename: "terminal",
          install: (v) =>
            `# Extract the downloaded tarball
tar -xzf vpp-v${v}-linux-x64.tar.gz
cd vpp-v${v}-linux-x64
./run.sh examples/hello.vpp
vpp check examples/hello.vpp
vpp --version
# Verify your install
vpp doctor`,
          info: "Linux x64 bundle: extract and run <code>./run.sh</code>. Add the folder to PATH for global use.",
        },
        {
          id: "source",
          label: "Build from source",
          source: true,
          lang: "bash",
          filename: "terminal",
          install: () =>
            `# Clone and build (requires Rust + LLVM 22)
git clone https://github.com/shauryaR790/VPP.git
cd VPP
cargo build --release --features codegen,lsp
./target/release/vpp --version
# Verify your install
./target/release/vpp doctor`,
          info: "Build from source when prebuilt bundles are unavailable for your distro.",
        },
      ],
    },
    macos: {
      label: "macOS",
      formats: [
        {
          id: "tarball",
          label: "Apple Silicon (.tar.gz)",
          primary: true,
          file: (v) => `vpp-v${v}-macos-arm64.tar.gz`,
          lang: "bash",
          filename: "terminal",
          install: (v) =>
            `# Extract the downloaded tarball
tar -xzf vpp-v${v}-macos-arm64.tar.gz
cd vpp-v${v}-macos-arm64
./run.sh examples/hello.vpp
vpp check examples/hello.vpp
vpp --version
# Verify your install
vpp doctor`,
          info: "macOS Apple Silicon bundle: extract and run <code>./run.sh</code>. Intel Macs: build from source.",
        },
        {
          id: "source",
          label: "Build from source",
          source: true,
          lang: "bash",
          filename: "terminal",
          install: () =>
            `# Clone and build (requires Rust + LLVM 22)
git clone https://github.com/shauryaR790/VPP.git
cd VPP
cargo build --release --features codegen,lsp
./target/release/vpp --version
# Verify your install
./target/release/vpp doctor`,
          info: "Build from source for Intel Macs or when you need a custom build.",
        },
      ],
    },
  };

  function detectOS() {
    const ua = navigator.userAgent.toLowerCase();
    const platform = (navigator.platform || "").toLowerCase();
    if (ua.includes("win") || platform.includes("win")) return "windows";
    if (ua.includes("mac") || platform.includes("mac")) return "macos";
    return "linux";
  }

  function releaseUrl(tag, filename) {
    return `https://github.com/${REPO}/releases/download/${tag}/${filename}`;
  }

  function releaseTag(version) {
    return version.startsWith("v") ? version : `v${version}`;
  }

  function $(id) {
    return document.getElementById(id);
  }

  function setIcon(elId, svg) {
    const el = $(elId);
    if (el) el.innerHTML = svg;
  }

  function updateSelectIcons(osKey, formatId, fmt) {
    setIcon("dl-version-icon", ICONS.version);
    setIcon("dl-os-icon", OS_ICONS[osKey] || ICONS.windows);
    setIcon("dl-format-icon", FORMAT_ICONS[formatId] || ICONS.installer);

    if (fmt?.source) {
      setIcon("dl-primary-icon", ICONS.source);
    } else {
      setIcon("dl-primary-icon", ICONS.download);
    }
  }

  function updateButtonIcons(fmt, altFmt) {
    if (altFmt) {
      setIcon("dl-secondary-icon", FORMAT_ICONS[altFmt.id] || ICONS.zip);
    }
  }

  function getFormat(os, formatId) {
    return PLATFORMS[os].formats.find((f) => f.id === formatId);
  }

  function populateFormats(osKey, preferredId) {
    const sel = $("dl-format");
    if (!sel) return null;
    sel.innerHTML = "";
    const formats = PLATFORMS[osKey].formats;
    formats.forEach((fmt) => {
      const opt = document.createElement("option");
      opt.value = fmt.id;
      opt.textContent = fmt.label;
      sel.appendChild(opt);
    });
    const pick = preferredId && formats.some((f) => f.id === preferredId)
      ? preferredId
      : (formats.find((f) => f.primary) || formats[0]).id;
    sel.value = pick;
    return pick;
  }

  function updateCodeBlock(wrapId, raw, lang, filename) {
    const wrap = $(wrapId);
    if (!wrap) return;
    const pre = wrap.querySelector("pre");
    const code = pre?.querySelector("code");
    if (!pre || !code) return;

    pre.className = `language-${lang}`;
    code.className = `language-${lang}`;
    code.textContent = raw;

    const fnEl = wrap.querySelector(".code-block-filename");
    if (fnEl) fnEl.textContent = filename;

    const finalized = typeof finalizeCodeText === "function" ? finalizeCodeText(raw, lang) : raw;
    if (typeof renderLineBasedCode === "function") {
      renderLineBasedCode(pre, finalized, lang, []);
    }

    const header = wrap.querySelector(".code-block-header");
    if (header && typeof attachCopyButton === "function") {
      const existing = header.querySelector(".code-copy-btn");
      if (existing) existing.remove();
      attachCopyButton(header, () => finalized);
    }
  }

  function applyReleaseLink(anchor, url) {
    if (!anchor) return;
    anchor.href = url;
    anchor.target = "_blank";
    anchor.rel = "noopener";
    anchor.removeAttribute("download");
  }

  function updateUI() {
    const version = $("dl-version")?.value || "1.0.4";
    const osKey = $("dl-os")?.value || "windows";
    const formatId = $("dl-format")?.value;
    const tag = releaseTag(version);
    const platform = PLATFORMS[osKey];
    const fmt = getFormat(osKey, formatId);
    if (!fmt) return;

    updateSelectIcons(osKey, formatId, fmt);

    const infoEl = $("dl-info");
    if (infoEl) infoEl.innerHTML = `<strong>Info</strong> ${fmt.info || ""}`;

    updateCodeBlock("dl-code-wrap", fmt.install(version), fmt.lang, fmt.filename);

    const pathWrap = $("dl-path-wrap");
    const codeNote = $("dl-code-note");
    if (fmt.showPath && pathWrap) {
      pathWrap.hidden = false;
      if (codeNote) {
        codeNote.hidden = false;
        codeNote.innerHTML = fmt.pathNote || "";
      }
      updateCodeBlock(
        "dl-path-wrap",
        `$dir = "$env:LOCALAPPDATA\\Programs\\vpp"\n[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$dir;$dir\\llvm\\bin", "User")\n# Restart terminal after updating PATH\nvpp --version\nvpp doctor`,
        "powershell",
        "terminal",
      );
    } else {
      if (pathWrap) pathWrap.hidden = true;
      if (codeNote) codeNote.hidden = true;
    }

    const primary = $("dl-primary");
    const primaryLabel = $("dl-primary-label");
    const secondary = $("dl-secondary");
    const secondaryLabel = $("dl-secondary-label");

    if (fmt.source) {
      if (primary) {
        primary.href = SOURCE_URL;
        primary.removeAttribute("download");
      }
      if (primaryLabel) primaryLabel.textContent = "Build from source";
      if (secondary) secondary.hidden = true;
    } else {
      const filename = fmt.file(version);
      const url = releaseUrl(tag, filename);
      applyReleaseLink(primary, url);
      if (primaryLabel) primaryLabel.textContent = `Download ${filename}`;

      const altFmt = platform.formats.find((f) => f.id !== fmt.id && !f.source);
      if (altFmt && secondary && secondaryLabel) {
        const altName = altFmt.file(version);
        applyReleaseLink(secondary, releaseUrl(tag, altName));
        secondary.hidden = false;
        secondaryLabel.textContent = altFmt.label;
        updateButtonIcons(fmt, altFmt);
      } else if (secondary) {
        secondary.hidden = true;
      }
    }

    const detectEl = $("dl-detect");
    const detected = detectOS();
    if (detectEl) {
      if (osKey === detected) {
        detectEl.textContent = `Recommended for your system (${platform.label}).`;
      } else {
        detectEl.innerHTML =
          `Detected ${PLATFORMS[detected].label}. `
          + `<button type="button" class="dl-detect-link" data-os="${detected}">Switch to ${PLATFORMS[detected].label}</button>`;
      }
    }
  }

  function initDownloadHub() {
    const hub = $("download-hub");
    if (!hub) return;

    const detected = detectOS();
    const osSel = $("dl-os");
    if (osSel) osSel.value = detected;

    populateFormats(osSel?.value || "windows");
    updateUI();

    ["dl-version", "dl-os", "dl-format"].forEach((id) => {
      $(id)?.addEventListener("change", () => {
        if (id === "dl-os") populateFormats($("dl-os").value);
        updateUI();
      });
    });

    hub.addEventListener("click", (e) => {
      const btn = e.target.closest(".dl-detect-link");
      if (!btn) return;
      const os = btn.dataset.os;
      if (osSel && os) {
        osSel.value = os;
        populateFormats(os);
        updateUI();
      }
    });
  }

  document.addEventListener("DOMContentLoaded", initDownloadHub);
})();

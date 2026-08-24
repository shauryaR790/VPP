/** Minimal download links for GitHub Releases. */
(function () {
  const REPO = "shauryaR790/VPP";
  const SOURCE_URL = `https://github.com/${REPO}`;

  const PLATFORMS = {
    windows: {
      label: "Windows",
      formats: [
        {
          id: "installer",
          label: "Installer (.exe)",
          primary: true,
          file: (v) => `vpp-${v}-setup.exe`,
          install: (v) =>
            `# Run vpp-${v}-setup.exe, then open a new terminal:\nvpp run examples\\hello.vpp\nvpp --version\nvpp doctor`,
          info: "Adds vpp to PATH. Bundled LLVM included for native builds.",
        },
        {
          id: "zip",
          label: "Portable (.zip)",
          file: (v) => `vpp-v${v}-windows-x64.zip`,
          install: (v) =>
            `# Extract vpp-v${v}-windows-x64.zip, then:\n.\\GO.bat\nvpp run examples\\hello.vpp\nvpp doctor`,
          info: "Portable zip. Run GO.bat or add the folder to PATH.",
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
          install: (v) =>
            `tar -xzf vpp-v${v}-linux-x64.tar.gz\ncd vpp-v${v}-linux-x64\n./run.sh examples/hello.vpp\nvpp doctor`,
          info: "Extract and run ./run.sh.",
        },
        {
          id: "source",
          label: "Build from source",
          source: true,
          install: () =>
            `git clone https://github.com/${REPO}.git\ncd VPP\ncargo build --release --features codegen,lsp\n./target/release/vpp doctor`,
          info: "When no prebuilt bundle matches your system.",
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
          install: (v) =>
            `tar -xzf vpp-v${v}-macos-arm64.tar.gz\ncd vpp-v${v}-macos-arm64\n./run.sh examples/hello.vpp\nvpp doctor`,
          info: "Apple Silicon bundle. Intel Macs: build from source.",
        },
        {
          id: "source",
          label: "Build from source",
          source: true,
          install: () =>
            `git clone https://github.com/${REPO}.git\ncd VPP\ncargo build --release --features codegen,lsp\n./target/release/vpp doctor`,
          info: "Build from source for Intel Macs or custom builds.",
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
    const pick =
      preferredId && formats.some((f) => f.id === preferredId)
        ? preferredId
        : (formats.find((f) => f.primary) || formats[0]).id;
    sel.value = pick;
    return pick;
  }

  function updateUI() {
    const version = $("dl-version")?.value || "1.0.5";
    const osKey = $("dl-os")?.value || "windows";
    const formatId = $("dl-format")?.value;
    const tag = releaseTag(version);
    const platform = PLATFORMS[osKey];
    const fmt = getFormat(osKey, formatId);
    if (!fmt) return;

    const infoEl = $("dl-info");
    if (infoEl) infoEl.textContent = fmt.info || "";

    const codeEl = $("dl-code");
    if (codeEl) codeEl.textContent = fmt.install(version);

    const primary = $("dl-primary");
    const primaryLabel = primary;
    const secondary = $("dl-secondary");

    if (fmt.source) {
      if (primary) {
        primary.href = SOURCE_URL;
        primary.textContent = "View source on GitHub";
      }
      if (secondary) secondary.hidden = true;
    } else {
      const filename = fmt.file(version);
      const url = releaseUrl(tag, filename);
      if (primary) {
        primary.href = url;
        primary.textContent = `Download ${filename}`;
      }
      const altFmt = platform.formats.find((f) => f.id !== fmt.id && !f.source);
      if (altFmt && secondary) {
        const altName = altFmt.file(version);
        secondary.href = releaseUrl(tag, altName);
        secondary.hidden = false;
        secondary.textContent = altFmt.label;
      } else if (secondary) {
        secondary.hidden = true;
      }
    }

    const detectEl = $("dl-detect");
    const detected = detectOS();
    if (detectEl) {
      if (osKey === detected) {
        detectEl.textContent = `Suggested for your system (${platform.label}).`;
      } else {
        detectEl.innerHTML =
          `Detected ${PLATFORMS[detected].label}. `
          + `<button type="button" class="dl-detect-link" data-os="${detected}">Switch</button>`;
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

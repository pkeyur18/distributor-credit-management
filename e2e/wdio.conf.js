// T-QA.3-1 — tauri-driver + WebdriverIO harness. `tauri-driver` is a Rust
// binary (`cargo install tauri-driver`), not an npm package, so it isn't a
// devDependency here — this config spawns it as a subprocess and proxies
// every WebDriver session through it, per Tauri's own documented pattern.
//
// D-8: `tauri-driver` speaks to WebView2 (Windows) and webkit2gtk (Linux)
// only — WKWebView (macOS) exposes no WebDriver, so this suite cannot run
// on macOS at all, by construction, not by omission. `T-QA.3-3`'s manual
// checklist (documents/qa/macos-manual-verification-checklist.md) is that
// platform's actual coverage; this file was authored and reviewed on macOS
// but has not been executed anywhere in this repository's history yet — it
// needs a Windows or Linux machine (or CI runner) with `tauri-driver`
// installed to run for the first time.
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.join(__dirname, "..");
const binaryName = process.platform === "win32" ? "bvconsole.exe" : "bvconsole";
const applicationPath = path.join(repoRoot, "src-tauri", "target", "release", binaryName);
const runStamp = new Date().toISOString().replace(/[:.]/g, "-");

let tauriDriver;

export const config = {
  // No browserName in capabilities (tauri:options only), so wdio needs an
  // explicit target — this is where tauri-driver listens by default.
  hostname: "127.0.0.1",
  port: 4444,
  specs: ["./specs/**/*.e2e.js"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": { application: applicationPath },
    },
  ],
  reporters: ["spec"],
  framework: "mocha",
  // The first test of the run does cold-start Setup (observed ~30s+ just
  // for the webview to appear under CI's xvfb/no-GPU webkit2gtk) on top of
  // its own onboarding flow — 60s left it no room.
  mochaOpts: { ui: "bdd", timeout: 120_000 },

  // Each spec starts from a fresh, unseeded app-data directory — set once
  // per suite run, not per test, since `create_root_member` is callable
  // exactly once (AC-7) and most specs build on a prior spec's state.
  onPrepare: () => {
    // Plain `cargo build` never enables `tauri`'s `custom-protocol` feature
    // (only the `tauri` CLI does that) — without it the binary always runs
    // in dev mode, per tauri's own build.rs (`dev = !custom_protocol`), and
    // tries to load http://localhost:1420 instead of the bundled frontend.
    spawnSync("cargo", ["build", "--release", "--features", "tauri/custom-protocol"], {
      cwd: path.join(repoRoot, "src-tauri"),
      stdio: "inherit",
    });
  },
  beforeSession: () => {
    tauriDriver = spawn("tauri-driver", [], {
      stdio: [null, process.stdout, process.stderr],
    });
  },
  afterSession: () => {
    tauriDriver?.kill();
  },

  // T-QA.3-4: one screenshot per failing test, kept under a per-run
  // timestamped directory — never overwritten by the next run, unlike a
  // fixed filename would be.
  afterTest: async function (test, _context, { passed }) {
    if (passed) return;
    const dir = path.join(__dirname, "screenshots", runStamp);
    fs.mkdirSync(dir, { recursive: true });
    const safeName = test.title.replace(/[^a-z0-9]+/gi, "-").toLowerCase();
    await browser.saveScreenshot(path.join(dir, `${safeName}.png`));
  },
};

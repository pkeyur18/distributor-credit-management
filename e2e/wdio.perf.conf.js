// Separate config from wdio.conf.js, deliberately: this suite needs the
// real OS app-data directory pre-seeded with 25,000 members *before* the
// app process launches (see tests/e2e_seed.rs), not the fresh-unseeded-
// per-suite-run assumption wdio.conf.js's specs rely on. Kept in its own
// `specs-perf/` directory so the regular `npm run test:e2e` never picks
// this up — it's slow (builds a 25,000-member dataset) and belongs on the
// weekly schedule (see .github/workflows/perf-ceiling.yml), not every PR.
//
// Same platform constraint as wdio.conf.js: Windows/Linux only (WKWebView
// has no WebDriver), and only ever safe to run on a throwaway CI runner —
// the seed step writes into the real app-data directory.
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
  specs: ["./specs-perf/**/*.e2e.js"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": { application: applicationPath },
    },
  ],
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: { ui: "bdd", timeout: 120_000 },

  onPrepare: () => {
    spawnSync("cargo", ["build", "--release"], {
      cwd: path.join(repoRoot, "src-tauri"),
      stdio: "inherit",
    });
    const seed = spawnSync(
      "cargo",
      [
        "test",
        "--release",
        "--test",
        "e2e_seed",
        "seed_the_real_app_data_directory_for_e2e_perf_at_scale",
        "--",
        "--ignored",
        "--nocapture",
      ],
      { cwd: path.join(repoRoot, "src-tauri"), stdio: "inherit" },
    );
    if (seed.status !== 0) {
      throw new Error("seeding the real app-data directory failed — see output above");
    }
  },
  beforeSession: () => {
    tauriDriver = spawn("tauri-driver", [], {
      stdio: [null, process.stdout, process.stderr],
    });
  },
  afterSession: () => {
    tauriDriver?.kill();
  },

  afterTest: async function (test, _context, { passed }) {
    if (passed) return;
    const dir = path.join(__dirname, "screenshots", runStamp);
    fs.mkdirSync(dir, { recursive: true });
    const safeName = test.title.replace(/[^a-z0-9]+/gi, "-").toLowerCase();
    await browser.saveScreenshot(path.join(dir, `${safeName}.png`));
  },
};

// Testing-only tool. Wipes this app's OS app-data directory — console.db,
// auth.json (the PIN/password sidecar), backups-manifest.json, and the
// default backups/ folder — so the next launch starts at first-run setup
// again. No test-only reset command exists in the app itself by design
// (see e2e/helpers/seed.js's header comment); this operates on the real
// on-disk files the same way uninstalling and reinstalling would.
//
// Quit the app before running this — the db file is open/locked while the
// app is running.
//
// Not wired into any app code. Delete this file to remove the tool.
import { existsSync, rmSync } from "node:fs";
import path from "node:path";
import os from "node:os";

const IDENTIFIER = "com.siddharthpatel.bvconsole"; // src-tauri/tauri.conf.json "identifier"

function appDataDir() {
  const home = os.homedir();
  switch (process.platform) {
    case "darwin":
      return path.join(home, "Library", "Application Support", IDENTIFIER);
    case "win32":
      return path.join(process.env.APPDATA ?? path.join(home, "AppData", "Roaming"), IDENTIFIER);
    default:
      return path.join(process.env.XDG_DATA_HOME ?? path.join(home, ".local", "share"), IDENTIFIER);
  }
}

function main() {
  const dir = appDataDir();
  const targets = ["console.db", "console.db-wal", "console.db-shm", "auth.json", "backups-manifest.json", "backups"].map(
    (name) => path.join(dir, name)
  );
  const existing = targets.filter(existsSync);

  if (existing.length === 0) {
    console.log(`nothing to reset — no app data found at ${dir}`);
    return;
  }

  console.log(`about to delete:\n${existing.map((p) => `  ${p}`).join("\n")}`);

  if (!process.argv.includes("--yes")) {
    console.log("\nre-run with --yes to actually delete. Make sure the app is closed first.");
    return;
  }

  for (const p of existing) {
    rmSync(p, { recursive: true, force: true });
  }
  console.log(`\ndone — ${existing.length} item(s) removed. Next launch starts at first-run setup.`);
}

main();

// Single source of truth: package.json's "version". This script propagates
// it into src-tauri/tauri.conf.json and src-tauri/Cargo.toml so the three
// files never drift (T-REL.1-1).
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

export function updateTauriConfVersion(json, version) {
  const parsed = JSON.parse(json);
  parsed.version = version;
  return JSON.stringify(parsed, null, 2) + "\n";
}

export function updateCargoTomlVersion(toml, version) {
  const lines = toml.split("\n");
  let inPackageSection = false;
  let replaced = false;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (/^\[.*\]$/.test(line.trim())) {
      inPackageSection = line.trim() === "[package]";
      continue;
    }
    if (inPackageSection && !replaced && /^version\s*=/.test(line)) {
      lines[i] = `version = "${version}"`;
      replaced = true;
    }
  }
  if (!replaced) {
    throw new Error("no [package] version line found in Cargo.toml");
  }
  return lines.join("\n");
}

// Cargo.lock pins the root crate's own version in its `[[package]] name = "bvconsole"`
// block. sync-version bumps Cargo.toml but Cargo would otherwise leave this entry
// stale until the next local `cargo build`, which is exactly the diff-on-every-pull
// nuisance this keeps in sync instead.
export function updateCargoLockVersion(lock, version) {
  const lines = lock.split("\n");
  let inBvconsole = false;
  let replaced = false;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line === "[[package]]") {
      inBvconsole = lines[i + 1]?.trim() === 'name = "bvconsole"';
      continue;
    }
    if (inBvconsole && !replaced && /^version\s*=/.test(line)) {
      lines[i] = `version = "${version}"`;
      replaced = true;
      inBvconsole = false;
    }
  }
  if (!replaced) {
    throw new Error('no bvconsole [[package]] version line found in Cargo.lock');
  }
  return lines.join("\n");
}

function main() {
  const root = path.resolve(fileURLToPath(import.meta.url), "..", "..");
  const checkOnly = process.argv.includes("--check");

  const pkg = JSON.parse(readFileSync(path.join(root, "package.json"), "utf8"));
  const version = pkg.version;

  const tauriConfPath = path.join(root, "src-tauri", "tauri.conf.json");
  const cargoTomlPath = path.join(root, "src-tauri", "Cargo.toml");
  const cargoLockPath = path.join(root, "src-tauri", "Cargo.lock");

  const nextTauriConf = updateTauriConfVersion(readFileSync(tauriConfPath, "utf8"), version);
  const nextCargoToml = updateCargoTomlVersion(readFileSync(cargoTomlPath, "utf8"), version);
  const nextCargoLock = updateCargoLockVersion(readFileSync(cargoLockPath, "utf8"), version);

  if (checkOnly) {
    const dirty =
      readFileSync(tauriConfPath, "utf8") !== nextTauriConf ||
      readFileSync(cargoTomlPath, "utf8") !== nextCargoToml ||
      readFileSync(cargoLockPath, "utf8") !== nextCargoLock;
    if (dirty) {
      console.error(`version out of sync: package.json says ${version}`);
      process.exit(1);
    }
    console.log(`versions in sync at ${version}`);
    return;
  }

  writeFileSync(tauriConfPath, nextTauriConf);
  writeFileSync(cargoTomlPath, nextCargoToml);
  writeFileSync(cargoLockPath, nextCargoLock);
  console.log(`synced tauri.conf.json, Cargo.toml, and Cargo.lock to ${version}`);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

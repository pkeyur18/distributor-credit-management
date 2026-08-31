import { test } from "node:test";
import assert from "node:assert/strict";
import { updateTauriConfVersion, updateCargoTomlVersion, updateCargoLockVersion } from "./sync-version.mjs";

test("updateTauriConfVersion replaces only the top-level version field", () => {
  const input = JSON.stringify(
    { productName: "Business Volume Console", version: "0.1.0", identifier: "x" },
    null,
    2,
  );
  const output = updateTauriConfVersion(input, "0.2.0");
  const parsed = JSON.parse(output);
  assert.equal(parsed.version, "0.2.0");
  assert.equal(parsed.productName, "Business Volume Console");
});

test("updateCargoTomlVersion replaces the [package] version line only", () => {
  const input = [
    "[package]",
    'name = "bvconsole"',
    'version = "0.1.0"',
    'description = "Business Volume Console"',
    "",
    "[dependencies]",
    'tauri = { version = "2.11.5", features = [] }',
    "",
  ].join("\n");

  const output = updateCargoTomlVersion(input, "0.2.0");

  assert.match(output, /^\[package\]\nname = "bvconsole"\nversion = "0\.2\.0"/);
  // the tauri dependency's own "version" field must not be touched
  assert.match(output, /tauri = \{ version = "2\.11\.5", features = \[\] \}/);
});

test("updateCargoLockVersion replaces only the bvconsole [[package]] version", () => {
  const input = [
    "[[package]]",
    'name = "argon2"',
    'version = "0.5.3"',
    "",
    "[[package]]",
    'name = "bvconsole"',
    'version = "0.1.0"',
    "dependencies = [",
    ' "argon2",',
    "]",
    "",
  ].join("\n");

  const output = updateCargoLockVersion(input, "0.2.0");

  assert.match(output, /name = "bvconsole"\nversion = "0\.2\.0"/);
  // an unrelated dependency's own version must not be touched
  assert.match(output, /name = "argon2"\nversion = "0\.5\.3"/);
});

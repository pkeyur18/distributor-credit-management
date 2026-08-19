# Client test-data tool — design spec

Date: 2026-08-19
Branch: `feature/client-test-tool` (permanent — never merged to `develop` or `main`, see Git workflow section)

## Problem

The client tests the app on Windows with no dev toolchain (no node, no cargo). Two testing-only helpers already exist in the repo — `src-tauri/src/bin/import_test_data.rs` and `scripts/dev-reset-app-data.mjs` — but both require running through `cargo`/`node`, which the client doesn't have either. The client needs a way to reset app data and import their own test CSV without any of that, using only a double-click.

## Goals

- Client can reset test data and import a CSV without a terminal, without installing node/cargo/anything.
- Reuses the app's own real logic (`bvconsole_lib`'s `m1_members`/`m2_entries`/`m5_close`) so imported data can never drift from the app's own validation/calc rules — same reasoning `import_test_data.rs` already documents.
- Throwaway: this is a testing-only tool, discarded later. Minimum code that works.

## Non-goals

- Not a general-purpose admin tool — just reset + import, nothing else.
- Not distributed as an installer — a single .exe handed to the client directly.
- No support for the app's normal (non-testing) workflows.

## Components

Single new Rust binary, `src-tauri/src/bin/test_tool.rs`, built as a small native GUI (egui/eframe — pure Rust, no webview, single static binary, no runtime deps on the client machine). Two buttons:

**Reset Test Data**
1. Confirm dialog + reminder to close the main app first (db file is locked while it's running).
2. Deletes `console.db`, `console.db-wal`, `console.db-shm`, `backups-manifest.json`, `backups/` from the app-data dir. Keeps `auth.json`.
3. On next launch of the real app, the client logs in with their existing PIN/password as normal — `db::open_encrypted` (`src-tauri/src/db/mod.rs:15-25`) auto-creates a fresh, migrated, seeded blank DB when `console.db` is missing, so no first-run redo and no schema code needed in the tool. (Confirmed by code inspection — this path already exists and already runs unconditionally on open, not something the tool has to implement.)

**Import CSV...**
1. PIN/password entry field (to unwrap the master key, same as `import_test_data.rs` does via `unwrap_master_key`).
2. Native file picker (`rfd` crate) to choose the CSV.
3. Optional "closed months" text field (comma-separated `YYYY-MM`), same semantics as `import_test_data.rs`'s `--closed-months` — client does need this for period-close testing.
4. Runs the same import logic `import_test_data.rs` already has (CSV parsing, member/entry creation, closed-month snapshot handling) in-process against the unlocked db.
5. Result (counts, or a plain-English error) shown in a message box — not a Rust panic/backtrace.

Both actions call into `bvconsole_lib` directly, same as the main app and `import_test_data.rs` — no reimplemented business logic.

## Error handling

The reused import logic currently uses `panic!`/`expect` (fine for a CLI tool run by a developer, wrong for a GUI a non-technical client uses). The GUI wrapper converts failures to a plain message-box string instead of a crash/backtrace — either by making the reused functions return `Result` at the call sites the tool uses, or wrapping calls in `std::panic::catch_unwind` and formatting the panic payload.

## CSV format

Unchanged from `import_test_data.rs`'s existing documented format (header row: `member_name, phone, address, email, consent, introducer_phone, amount, entry_date`). Client authors their own CSV.

## Git workflow (exception to normal repo flow)

- All work lives on `feature/client-test-tool`, branched from `develop`.
- This branch is **never merged** into `develop` or `main` — permanent exception to the repo's normal feature-branch → PR → develop flow.
- The CI workflow that builds the Windows `.exe` (see below) is added **only on this branch** — its workflow file never lands on `develop`/`main` either, since GitHub Actions runs workflow files as they exist on the ref that triggered the event, not just what's on the default branch.

## Build & distribution

- **macOS (your own testing):** `cargo run --bin test_tool` / `cargo build --bin test_tool` locally, same as any other bin target.
- **Windows (client):** cross-compiling the bundled SQLCipher/vendored-OpenSSL build from macOS is fragile (per the existing `Cargo.toml` pin comment on that dependency), so the Windows exe is built natively via a GitHub Actions workflow on a `windows-latest` runner, triggered by pushes to `feature/client-test-tool` (and `workflow_dispatch` for on-demand builds). You download the built `.exe` artifact from the Actions run and hand it to the client directly — no installer.

## Testing

Manual: run Reset then Import against a real app-data directory on macOS (first-run the main app once to create `auth.json`/`console.db`, then exercise both buttons) before shipping the Windows build to the client.

## Open items

- Exact egui window layout/styling — left to implementation, not load-bearing.
- Whether the tool needs its own icon/product name shown in the window — cosmetic, default to something plain like "BV Console — Test Tool".

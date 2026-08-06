# Testing Strategy

## 1. Foundation: the five worked scenarios are the project's golden test set

`requirement-spec.md` §5 re-derives all five of the client's original worked examples independently from Rules 6–12 alone, with no reference to the draft's stated answers, and all five reconcile exactly:

| Scenario | Differential | Royalty | Total | Golden value |
|---|---|---|---|---|
| 1 — basic differential | 35 | 0 | 35 | 35 |
| 2 — differential collapses to zero on equal slab | 22 | 0 | 22 | 22 |
| 3 — multi-depth rollup | 450 | 0 | 450 | 450 |
| 4 — pure royalty | 0 | 1,000 | 1,000 | 1,000 |
| 5 — differential and royalty together | 580 | 400 | 980 | 980 |

These five scenarios, plus `client-requirements-validation.md`'s AC-1–AC-36 (§13), are the primary acceptance-test source — not invented from scratch. Every critical requirement in this project already has at least one verification mechanism defined by the client/architect before this analysis phase began; this strategy organizes and extends that coverage.

## 2. Test levels

### Unit tests — Calculation engine (M3)
- Slab lookup (Rule-3): boundary values exactly at a threshold land in the higher slab (Scenario 2's C at 3,000 → 8%; Scenario 4's A at 10,000 → 14%).
- TBV rollup (Rule-6): one-level-deep sum, own-BV term never omitted even when derived (Scenario 3: A's own BV = 500).
- Differential (Rule-8): base is child's TBV not own-BV; only direct children contribute; own BV never earns.
- Differential non-negativity (Rule-9): property-based test — for any valid slab table and any tree, no differential term is ever negative (this test should explicitly document that it assumes a monotonic slab table, since Rule-41 removes that guarantee at the input layer — see the edge-case test below).
- Royalty qualification (Rule-10, Rule-25): exactly `royalty_min_children` boundary (2 vs. 3 top-slab children); royalty stacking at multiple levels of the same chain (the P/Q/R worked illustration in requirement-spec.md §4.2, double-royalty on the same underlying volume).
- Rewards ledger isolation (Rule-13): after computing Rewards, re-run and confirm the earner's own TBV/slab are unchanged by their own Rewards.
- Fixed-point precision (Rule-22): sum a column of ×100 integers, confirm no drift vs. hand calculation across a long chain.

### Unit tests — Other modules
- Member ID allocation (Rule-35): random, excludes 100000, never reuses a taken ID (including deactivated members' IDs).
- Phone uniqueness + reactivation offer (Rule-34): duplicate against active → error; duplicate against inactive → reactivation payload, not an error.
- Inactive-member calculation neutrality (Rule-28, the corrected rule): deactivating a mid-tree member with active descendants must **not** change any ancestor's TBV/Rewards — this is the single highest-value regression test in the whole suite, since implementing the original (superseded) spec wording here would silently corrupt the ledger.
- Zero/negative Business Volume refusal (Rule-16a).
- Empty-month exclusion from yearly averaging (RQ-16).

### Integration tests — Transactional correctness
- Chain-upward recalculation (ADR-005): trigger a `record_entry` deep in a multi-level tree, confirm every ancestor up to root is recalculated **in one transaction**, and confirm that a sibling's differential term changes too when the parent's slab shifts (the specific detail `architecture.md` §9.2 calls out — re-scanning **all** direct children of an ancestor, not just the changed leaf).
- Monthly close atomicity (Rule-18): simulate a backup-verification failure mid-close, confirm zero data is mutated (no partial zeroing, no orphaned snapshot row).
- Closed-month correction versioning (Rule-39): edit an entry in a closed month, confirm a new `monthly_snapshots`/`backups` version is created, confirm the original version's row is byte-identical to before, confirm `redownload_backup` and `export_yearly_average` both read the new version.
- Settings-change mid-period recalculation: change the royalty rate mid-month, confirm only the current open period recalculates and no closed-month snapshot is touched.

### API / contract tests — IPC command surface
One test per command in [04-api-specification.md](04-api-specification.md) verifying: request/response shape, validation rules, authorization (every command except `login`/`setup_first_run`/`use_recovery_code` requires an authenticated session), and the documented error responses. Given there are 32 commands and no HTTP layer, these are Tauri command-invocation tests run against the Rust application container directly (no browser/network mocking needed).

### E2E tests — Full user workflows, driven through the actual UI
- First-run setup → root member creation → first Business Volume entry → visible Rewards on screen, no manual recalculation step anywhere (Rule-26).
- Full monthly-close wizard: backup confirm → commit → figures zeroed → alert clears → (if applicable) next outstanding month's alert appears.
- Entry-lock enforcement: let a month elapse without closing, confirm the BV Entry screen shows the locked empty state and the persistent banner appears with no dismiss control.
- Closed-month correction end-to-end: correct an entry in a closed month via the "Correct a closed month" panel, confirm the warning copy is shown, confirm the toast, confirm the exported closed-month snapshot reflects the correction.
- Phone-duplicate-on-inactive reactivation flow, end-to-end through the Add Member modal.
- Auth: setup wizard (recovery-code reveal + mandatory "I have saved this" gate), login with each credential type, lockout countdown, recovery flow (old credential invalidated, new one issued).

### Security tests
- Lockout: exactly 5 failed attempts triggers lockout; countdown timing; attempts do not reset early.
- Encryption at rest: confirm the raw `.sqlite` file is unreadable without the derived key (attempt to open with a plain SQLite client, expect failure).
- Session key lifecycle: confirm the decryption key is not retrievable from process memory after `lock_session` (as far as testable without specialized memory-forensics tooling — at minimum, confirm re-authentication is required and cannot be bypassed by any command).
- Filesystem capability allowlist: confirm the WebView cannot invoke any command outside the 32 documented ones (Tauri capability config test).

### Performance tests
Targets per NFR-1: screens <2s, recalculation <2s, extracts <30s, at the 25,000-member/200,000-entries-per-year design ceiling (client's actual scale is 500–5,000 members — test at both the realistic scale and the design ceiling).
- Chain-upward recalculation time vs. tree depth/width, confirming the claimed O(depth × average width) complexity — independent of total member count (`architecture.md` §9.4).
- Search response time at 25,000 members.
- Monthly export generation time at 25,000 members / a full year of entries.

### UAT — the five worked scenarios, reconciled by the client
Per SC-1–SC-8/AC-1–AC-36 (`client-requirements-validation.md` §12–13): re-run all five scenarios through the actual built UI (not just the calculation engine in isolation) and have the client confirm the on-screen figures match their own hand-worked numbers. This is the single most important acceptance gate in the entire test strategy — it is the client's own stated bar for trusting the system ("recalculating the client's five worked examples reproduces their stated totals exactly").

## 3. Coverage mapping — every critical requirement has a verification mechanism

| Requirement class | Verification |
|---|---|
| Calculation correctness (Rules 3–13, 25) | Unit tests + the five golden scenarios + UAT |
| Recalculation trigger/scope (Rule-26) | Integration tests (chain-upward, transactional) |
| Entry/correction rules (Rules 16, 16a, 39) | Unit + E2E |
| Monthly close/backup gate (Rules 17–20, 31, 38) | Integration (atomicity) + E2E (wizard) |
| Member lifecycle/hierarchy integrity (Rules 28, 30, 34, 35, 37, 40) | Unit + E2E |
| Exports/reporting (Rules 19, 23, 24, 33) | Integration + E2E |
| Auth/security (Rule-29, NFR-4) | Security tests + E2E |
| Performance/scale (NFR-1, NFR-2) | Performance tests |
| Accepted risks (Rule-41 monotonicity) | A deliberate **negative** test: confirm the system does *not* block a non-monotonic slab table save, and confirm the resulting (possibly negative) differential is computed and displayed as-is, not silently clamped — this proves the accepted-risk decision is correctly implemented as "no validation," not accidentally half-implemented. |

## 4. What is explicitly not tested (and why)

- Concurrent-user/multi-writer scenarios — structurally inapplicable (single-user, single-session system).
- Network failure/retry handling — no network layer exists.
- Cross-browser/device compatibility — desktop-only, no browser/phone/tablet target (NFR-15).
- Data migration — no migration tooling is in scope (NFR-16).
- Monitoring/alerting on a silently-failed close — explicitly declined by the client (NFR-12); do not add a test for a feature that was deliberately not built.

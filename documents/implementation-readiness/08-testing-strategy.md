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
- Empty-month exclusion from yearly averaging (RQ-16) — still required after CR-2, which makes an empty month less likely but not impossible.
- **Period isolation on recalculation (CR-2):** with two periods holding live figures, record into the older one and assert every ancestor chain in **that** period recalculates while the other period's `member_period_totals` rows are byte-identical to before.
- **TEST-R42 — members are never removed, and never omitted.** Two assertions: no code path deletes a `members` row (there is no delete command to call), and every export includes deactivated members. This is a client requirement, so the test exists to stop a future change quietly introducing an erasure path or an "active only" export filter.
- Last slab row cannot be removed (LOW-2): with one row remaining, the remove handler refuses and the row count is unchanged.
- **TEST-R44 — phone matching (Rule-44).** A member stored as `+91 98765 43210` is found by `9876543210`, by `98765 43210`, by `+919876543210`, by `09876543210` and by the mid-number fragment `4321`. **The reverse direction is the case that actually catches a bare digit-strip implementation:** a member stored as plain `9876543210` must be found by `+91 98765 43210`. A 3-digit query matches on phone not at all; a query matching one member's name and another's phone returns both. The same function is asserted to back every search box, so behaviour cannot drift between screens. The Rule-34 duplicate check is asserted against the same canonical key, so one person cannot be added twice under two spellings of one number.
- **TEST-R36 — entry eligibility by period (Rule-36 as amended).** Given June `awaiting_close` and today in August: a June-dated entry is accepted; an August-dated entry is refused naming June; a May-dated entry (closed) is refused and directed to the correction path; a future-dated entry is refused. After June closes, the August-dated entry is accepted. This is the test that fails loudly if the superseded "lock everything" wording is implemented.
- **TEST-R45 — full-tree layout (Rule-45).** The single post-order pass places every node without overlap and produces identical geometry for identical input across two runs; a single-node structure and a single-chain structure both lay out correctly; the emitted connector path matches the node positions it was computed alongside.
- `preview_settings_impact` (API-33) leaves nothing behind: call it with candidate settings, then confirm the live settings and all live figures are byte-identical to before. The temporarily-swapped settings must be restored even when the computation throws — test that path deliberately by feeding it a candidate that panics the engine.

### Integration tests — Transactional correctness
- Chain-upward recalculation (ADR-005): trigger a `record_entry` deep in a multi-level tree, confirm every ancestor up to root is recalculated **in one transaction**, and confirm that a sibling's differential term changes too when the parent's slab shifts (the specific detail `architecture.md` §9.2 calls out — re-scanning **all** direct children of an ancestor, not just the changed leaf).
- Monthly close atomicity (Rule-18): simulate a backup-verification failure mid-close, confirm zero data is mutated (no partial zeroing, no orphaned snapshot row).
- Closed-month correction versioning (Rule-39): edit an entry in a closed month, confirm a new `monthly_snapshots`/`backups` version is created, confirm the original version's row is byte-identical to before, confirm `redownload_backup` and `export_yearly_average` both read the new version.
- Settings-change mid-period recalculation: change the royalty rate mid-month, confirm only the current open period recalculates and no closed-month snapshot is touched.
- **The preview must equal what actually lands (MEDIUM-1).** The highest-value test of the settings warning, and already proven against the prototype: capture the Rewards figure the warning predicts, confirm the save, then re-open the warning with no further edits and confirm the settled "before" figure equals the earlier prediction exactly. A preview that can disagree with reality is worse than showing no preview at all, because the admin would have approved a change on the strength of a number that was never true.
- Restore from backup (LOW-3): against a deliberately corrupted database file, confirm the app enters the recovery state rather than crashing; confirm a restore whose checksum fails is refused and leaves the corrupt file untouched; confirm a successful restore leaves the app at sign-in with the credential still required.

### API / contract tests — IPC command surface
One test per command in [04-api-specification.md](04-api-specification.md) verifying: request/response shape, validation rules, authorization, and the documented error responses. Given there are 36 commands and no HTTP layer, these are Tauri command-invocation tests run against the Rust application container directly (no browser/network mocking needed).

**Authorization is tested as a closed set, not per-command.** Exactly six commands may run unauthenticated — `login`, `setup_first_run`, `use_recovery_code`, `check_data_readable`, `list_restore_points`, `restore_from_backup`. The test asserts that list exactly: every other command must refuse without a session, and no seventh command may join the set without the test failing. That is the point — this is the assertion that catches an unauthenticated command being added by accident.

### E2E tests — Full user workflows, driven through the actual UI
- First-run setup → root member creation → first Business Volume entry → visible Rewards on screen, no manual recalculation step anywhere (Rule-26).
- Full monthly-close wizard: backup confirm → commit → figures zeroed → alert clears → (if applicable) next outstanding month's alert appears.
- **Late-entry flow (CR-2):** let a month elapse without closing; confirm the BV Entry screen still renders its form, names the outstanding month it is recording into, accepts a figure dated in that month, and shows the recalculated figures immediately. Confirm the persistent banner appears with no dismiss control and states that entries dated in the outstanding month are still accepted.
- **Current-month refusal (CR-2):** in the same state, attempt a figure dated in the current month; confirm it is refused naming the blocking month, that nothing else on the form is disabled, and that after completing the close the same figure saves.
- **Multiple outstanding months (CR-2):** with two months outstanding, confirm the month selector appears on the entry screen and the figure screens and that switching changes the figures shown; with one month in play, confirm no selector is rendered anywhere.
- **Phone search (CR-1):** find a member by phone from Home, confirm the phone column renders in the results, then repeat from the Structure, Volume Entry and Correction search boxes and confirm identical behaviour.
- **Full hierarchy (CR-3):** from Structure, activate "View full hierarchy" on a network above 60 members; confirm the confirmation names the real count and Cancel opens nothing; confirm the window opens with every branch expanded and three fields per node; confirm zoom, fit-width, in-window search and print all work; confirm the main console stays usable throughout; record an entry in the console afterwards and confirm the open window is unchanged.
- Closed-month correction end-to-end: correct an entry in a closed month via the "Correct a closed month" panel, confirm the warning copy is shown, confirm the toast, confirm the exported closed-month snapshot reflects the correction.
- Settings warning end-to-end (MEDIUM-1): save the slab table → warning names the open month, shows Rewards before → after and the affected members → Cancel leaves both the settings and the admin's typed values untouched → re-save and confirm applies. Repeat for royalty, where the list shows members starting/stopping royalty instead. Confirm a duplicate threshold is refused outright, with no warning offered.
- Data-recovery screen (LOW-3): launch against an unreadable database, confirm the recovery screen appears in place of sign-in, restore points are listed newest-first by the month each holds, and the screen states that anything recorded after the chosen backup will need entering again.
- Phone-duplicate-on-inactive reactivation flow, end-to-end through the Add Member modal.
- Auth: setup wizard (recovery-code reveal + mandatory "I have saved this" gate), login with each credential type, lockout countdown, recovery flow (old credential invalidated, new one issued).

### Security tests
- Lockout: exactly 5 failed attempts triggers lockout; countdown timing; attempts do not reset early.
- Encryption at rest: confirm the raw `.sqlite` file is unreadable without the derived key (attempt to open with a plain SQLite client, expect failure).
- Session key lifecycle: confirm the decryption key is not retrievable from process memory after `lock_session` (as far as testable without specialized memory-forensics tooling — at minimum, confirm re-authentication is required and cannot be bypassed by any command).
- Filesystem capability allowlist: confirm the WebView cannot invoke any command outside the 36 documented ones (Tauri capability config test).
- Pre-authentication surface: confirm `restore_from_backup` — the only destructive unauthenticated command — refuses a checksum mismatch, and that a restored database still requires the credential to open (restoring must not grant access to anything).

### Performance tests
Targets per NFR-1: screens <2s, recalculation <2s, extracts <30s, at the 25,000-member/200,000-entries-per-year design ceiling (client's actual scale is 500–5,000 members — test at both the realistic scale and the design ceiling).
- Chain-upward recalculation time vs. tree depth/width, confirming the claimed O(depth × average width) complexity — independent of total member count (`architecture.md` §9.4).
- Search response time at 25,000 members — including a **phone** query, whose canonical-key substring match is a scan rather than an index seek (Rule-44).
- Monthly export generation time at 25,000 members / a full year of entries.
- **Full hierarchy draw time at 25,000 members**, measured in the separate window — **with the main console's responsiveness measured at the same time.** The second measurement is the one that gates: the draw itself is a known, accepted cost (TR-7), while any slowdown of the main console is a defect against the client's binding constraint on CR-3.

### UAT — the five worked scenarios, reconciled by the client
Per SC-1–SC-8/AC-1–AC-36 (`client-requirements-validation.md` §12–13): re-run all five scenarios through the actual built UI (not just the calculation engine in isolation) and have the client confirm the on-screen figures match their own hand-worked numbers. This is the single most important acceptance gate in the entire test strategy — it is the client's own stated bar for trusting the system ("recalculating the client's five worked examples reproduces their stated totals exactly").

## 3. Coverage mapping — every critical requirement has a verification mechanism

| Requirement class | Verification |
|---|---|
| Calculation correctness (Rules 3–13, 25) | Unit tests + the five golden scenarios + UAT |
| Recalculation trigger/scope (Rule-26) | Integration tests (chain-upward, transactional) |
| Entry/correction rules (Rules 16, 16a, 39) | Unit + E2E |
| Monthly close/backup gate (Rules 17–20, 31, 38) | Integration (atomicity) + E2E (wizard) |
| Entry eligibility by period (Rule-36 as amended) | TEST-R36 — unit (the state matrix) + integration (recalculation confined to one period) + E2E (late entry, current-month refusal, month switcher) |
| Phone as a search key (Rule-44) | TEST-R44 — unit (digit normalisation, four-digit floor) + E2E (every search box) + performance (scan at 25,000) |
| Full hierarchy view (Rule-45) | TEST-R45 — unit (layout pass) + E2E (size gate, draw, print, read-only) + performance (draw time **and** main-console responsiveness) |
| Member lifecycle/hierarchy integrity (Rules 28, 30, 34, 35, 37, 40) | Unit + E2E |
| Members never removed, never omitted (Rule-42) | TEST-R42 — unit (no delete path) + integration (exports include deactivated members) |
| Exports/reporting (Rules 19, 23, 24, 33) | Integration + E2E |
| Settings pre-save warning (RQ-18/V7.6, MEDIUM-1) | Unit (`preview_settings_impact` leaves nothing behind) + integration (prediction equals what lands) + E2E (both sections, Cancel is a no-op) |
| Slab table cannot be emptied (LOW-2) | Unit + E2E |
| Data recovery at launch (LOW-3) | Integration (corrupt file, checksum refusal) + E2E (screen, restore, still requires credential) |
| Auth/security (Rule-29, NFR-4) | Security tests + E2E |
| Pre-authentication command set is exactly six | Contract test asserting the closed set |
| Performance/scale (NFR-1, NFR-2) | Performance tests |
| Accepted risks (Rule-41 monotonicity) | A deliberate **negative** test: confirm the system does *not* block a non-monotonic slab table save, and confirm the resulting (possibly negative) differential is computed and displayed as-is, not silently clamped — this proves the accepted-risk decision is correctly implemented as "no validation," not accidentally half-implemented. |

## 4. What is explicitly not tested (and why)

- Concurrent-user/multi-writer scenarios — structurally inapplicable (single-user, single-session system).
- Network failure/retry handling — no network layer exists.
- Cross-browser/device compatibility — desktop-only, no browser/phone/tablet target (NFR-15).
- Data migration — no migration tooling is in scope (NFR-16).
- Monitoring/alerting on a silently-failed close — explicitly declined by the client (NFR-12); do not add a test for a feature that was deliberately not built.

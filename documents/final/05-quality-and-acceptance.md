# 05 — Quality & Acceptance

Every error/edge case, the full test strategy, the golden regression scenarios, all 39 acceptance criteria and 8 success criteria, and the Definition of Done. Nothing is claimed complete until it passes what is written here.

---

## 1. Golden regression set — reproduce these five totals exactly, always

| Scenario | Differential | Royalty | **Total Rewards** |
|---|---|---|---|
| 1 — basic differential | 35 | 0 | **35** |
| 2 — differential collapses on an equal slab | 22 | 0 | **22** |
| 3 — multi-depth rollup | 450 | 0 | **450** |
| 4 — pure royalty | 0 | 1,000 | **1,000** |
| 5 — differential and royalty together | 580 | 400 | **980** |

Full trees: [02](02-business-rules.md) §5. These are the client's own hand-worked numbers, re-derived independently from the rules and confirmed to match exactly. **If any total moves during development, a rule was implemented wrongly — stop and find it before continuing.**

---

## 2. Error & edge-case matrix — 46 scenarios, organised by workflow

"Behaviour" is stated as defined by a source document; where a rule leaves something undefined, the gap is named rather than filled with an invented default. See [06](06-decision-log-and-open-items.md) §3 for the five items still genuinely open.

### Add / Edit / Deactivate / Reactivate Member (M1)

| Scenario | Behaviour | Rule |
|---|---|---|
| Reference ID does not resolve to an existing member | Rejected at entry, clear message | Rule-30 |
| Reference ID resolves to an **inactive** member | Rejected — must be existing **and active** | Rule-30 |
| Phone matches an active member | Rejected — "already in use" | Rule-34 |
| Phone matches an inactive member | **Not an error** — named reactivation offer, ID/position/history preserved | Rule-34 |
| Attempt to create a second root member | Rejected — one-time, first-run-only action | Rule-30 |
| Attempt to move a member beneath its own descendant | Blocked, reason shown | Rule-30 (structurally unreachable given Rule-37, guard remains) |
| Attempt to change a member's introducer after creation | **Blocked outright, no override** | Rule-37 |
| Attempt to hard-delete a member | **Not offered anywhere** | Rule-28, Rule-42 |
| Attempt to deactivate the root member | Rejected/disabled | Structural consequence of "exactly one root, fixed permanently" |
| Onboarding exceeds level-width or depth guidance | **Not an error** — warns, allows | Rule-1, Rule-32 |
| Consent checkbox not ticked | Save disabled — action simply unavailable, not a submit-then-reject error | Rule-40 |

### Business Volume Entry (M2)

| Scenario | Behaviour | Rule |
|---|---|---|
| Amount is zero | Refused | Rule-16a |
| Amount is negative | Refused | Rule-16a |
| Amount has more than 2 decimal places | Refused, field-level | Rule-16 |
| Entry attempted while a reset is outstanding | **Entire screen locked** — no entry form rendered, locked state names the outstanding month | Rule-36 |
| Entry date outside the current period's bounds | Refused, date clamped to period bounds | Rule-21 |
| Correcting an entry in the current **open** month | Permitted, standard edit | Rule-39 |
| Correcting an entry in a **closed** month | Permitted — new snapshot/backup version, original never overwritten, warning shown before the edit | Rule-39 |
| Concurrent entry conflict (two writes to the same member at once) | **Not applicable** — single-user, single-session by design; no concurrency-control mechanism needed | ADR-001, OC-1 |

### Monthly Close (M5)

| Scenario | Behaviour | Rule |
|---|---|---|
| Backup generation fails | **Abort entirely** — nothing zeroed, nothing touched, alert stays up, admin may retry | Rule-18 |
| Backup generation is cancelled by the admin | Same as failure — abort, no partial state | Rule-18 |
| Admin attempts to close a month that is not the oldest outstanding | Rejected — oldest-first enforced | Rule-20 |
| Multiple months outstanding | All listed; each closes separately with its own backup and snapshot; never merged | Rule-20 |
| A calendar month elapses with **zero entries** | **No snapshot produced**, excluded from the yearly-averaging denominator | RQ-16 |
| External (physically-separate-medium) backup fails to write | **Does not block the close** — internal retained copy is the actual gate; external failure re-prompts/reminds | Rule-31 |
| Internal retained backup copy write fails | **Blocks the close** — this is the real gate | Rule-18, Rule-31 |
| Admin loses/never takes the external backup copy at all | **Not defended** — documented single point of failure; accepted risk, not solved | TR-4 |

### Settings / Slab Table (M7)

| Scenario | Behaviour | Rule |
|---|---|---|
| Duplicate slab threshold on add/edit | Rejected — duplicate-threshold guard on save | Rule-4 |
| Slab percentages configured non-monotonically vs. thresholds | **Not validated, not blocked** — explicitly declined by the client | Rule-41 |
| Attempt to remove the last remaining slab row | **Refused.** Remove control disabled with an `aria-label` and hint; handler refuses if reached another way | Rule-27 (LOW-2, built) |
| Settings change alters the current open period (e.g. royalty rate) | Recalculates the current open period only; closed months never affected | RQ-18 |
| Settings change that re-works the open month (slab table, royalty) | **Pre-save warning.** Names the open month, states closed months unaffected, shows Rewards before → after, lists affected members. Cancel is a true no-op. A duplicate threshold is refused outright *before* the warning is offered | RQ-18 (variant C, built) |

### Authentication (M8)

| Scenario | Behaviour | Rule |
|---|---|---|
| Wrong PIN or password entered | Generic "incorrect" message — does not reveal which credential type or which part was wrong; failed-attempt counter increments | Rule-29 |
| 5 consecutive failed attempts | Timed lockout with countdown; no login possible until it elapses. **Ladder beyond this point is undefined — see [06](06-decision-log-and-open-items.md) O4** | Rule-29 |
| Recovery code entered is invalid or already used | Refused — codes are single-use | ADR-008 |
| Both credential **and** all recovery codes are lost | **Permanently unrecoverable by design** — no vendor backdoor, no email flow | ADR-008 |
| Session inactivity timeout elapses | Auto-locks; encryption key dropped from memory; re-authentication genuinely re-derives the key | NFR-4 |
| Device stolen or accessed while the session is unlocked | **Not defended** by the application — client's own physical-security responsibility | §8.8 threat model |

### Exports / Reports (M6)

| Scenario | Behaviour | Rule |
|---|---|---|
| Export requested for a period with no snapshot (empty month) | Excluded from yearly-average calculation and its denominator; not shown as a closed-month option | RQ-16, Rule-23 |
| Export requested for a corrected (multi-version) closed month | Always returns the **latest** version; the original stays in the audit trail, not in the export | Rule-39 |
| Low-contribution report with no members below threshold | Empty state shown, not an error | Confirmed in prototype |
| Export column selection is empty (no optional columns ticked) | Not an error — mandatory columns always included regardless | Rule-19 |

### Downstream / System-level failures

| Scenario | Behaviour | Rule |
|---|---|---|
| Database file corrupted or unreadable at launch | **Full-screen data-recovery state**, shown in place of sign-in. States nothing has been lost, lists retained backups by month held (marking corrected months), offers restore and retry, states plainly what will need re-entering | LOW-3 (built, design D) |
| Application crash mid-transaction | Protected by SQLite's own transactional guarantees — a crash mid-transaction rolls back cleanly; no partial recalculation or partial close state can persist | [04](04-technical-architecture.md) §5.1 |
| Disk full during backup write | Covered by the same write-verify mechanism as any backup failure — verification fails, close aborts | Rule-18 |
| Timeout / retry | **Not applicable** — no network calls exist anywhere; every operation is local disk I/O | ADR-001 |

---

## 3. Testing strategy

### 3.1 Test levels

**Unit — calculation engine (M3)**
- Slab lookup boundary values exactly at a threshold land in the higher slab (Scenario 2's C at 3,000 → 8%; Scenario 4's A at 10,000 → 14%).
- TBV rollup: one-level-deep sum, own-BV term never omitted even when derived (Scenario 3: A's own BV = 500).
- Differential: base is child's TBV not own-BV; only direct children contribute; own BV never earns.
- Differential non-negativity: property-based test — for any *monotonic* slab table and any tree, no differential term is ever negative. Explicitly document the monotonicity assumption, since Rule-41 removes that guarantee at the input layer.
- Royalty qualification: exactly `royalty_min_children` boundary (2 vs 3 top-slab children); royalty stacking at multiple levels of the same chain (§5.6's P/Q/R illustration).
- Rewards ledger isolation: after computing Rewards, re-run and confirm the earner's own TBV/slab are unchanged.
- Fixed-point precision: sum a column of ×100 integers, confirm no drift vs. hand calculation across a long chain.

**Unit — other modules**
- Member ID allocation: random, excludes 100000, never reuses a taken ID (including deactivated members').
- Phone uniqueness + reactivation offer: duplicate against active → error; duplicate against inactive → reactivation payload, not an error.
- **Inactive-member calculation neutrality — the single highest-value regression test in the suite.** Deactivating a mid-tree member with active descendants must **not** change any ancestor's TBV/Rewards. Implementing the superseded spec wording here silently corrupts the ledger.
- Zero/negative Business Volume refusal.
- Empty-month exclusion from yearly averaging.
- **Members never removed, never omitted:** no code path deletes a `members` row (there is no delete command to call); every export includes deactivated members.
- Last slab row cannot be removed: with one row remaining, the remove handler refuses and the row count is unchanged.
- `preview_settings_impact` leaves nothing behind: call with candidate settings, confirm live settings and all live figures are byte-identical afterward. Test the restore-on-panic path deliberately, by feeding a candidate that panics the engine.

**Integration — transactional correctness**
- Chain-upward recalculation: trigger `record_entry` deep in a multi-level tree, confirm every ancestor up to root recalculates **in one transaction**, and confirm a sibling's differential term changes too when the parent's slab shifts.
- Monthly close atomicity: simulate a backup-verification failure mid-close, confirm zero data is mutated.
- Closed-month correction versioning: edit an entry in a closed month, confirm a new version is created, the original version's row is byte-identical to before, and `redownload_backup`/`export_yearly_average` both read the new version.
- Settings-change mid-period recalculation: change the royalty rate mid-month, confirm only the current open period recalculates and no closed-month snapshot is touched.
- **The settings preview must equal what actually lands:** capture the Rewards figure the warning predicts, confirm the save, then re-open the warning with no further edits and confirm the settled "before" figure equals the earlier prediction exactly.
- Restore from backup: against a deliberately corrupted database, confirm the recovery state appears rather than a crash; confirm a checksum-mismatch restore is refused and leaves the corrupt file untouched; confirm a successful restore leaves the app at sign-in with the credential still required.

**API / contract — the 40-command IPC surface**
One test per command in [04](04-technical-architecture.md) §6: request/response shape, validation rules, authorization, documented error responses. No HTTP layer — these are direct Tauri command-invocation tests against the Rust core.

**Authorization is tested as a closed set, not per-command.** Exactly **seven** commands may run unauthenticated (see [06](06-decision-log-and-open-items.md) C3): `login`, `setup_first_run`, `use_recovery_code`, `check_data_readable`, `list_restore_points`, `restore_from_backup`, `restore_from_backup_file`. The test asserts that list exactly — every other command must refuse without a session, and no eighth command may join the set without the test failing.

**E2E — full user workflows through the actual built UI**
- First-run setup → root member creation → first Business Volume entry → visible Rewards on screen, no manual recalculation step anywhere.
- Full monthly-close wizard: backup confirm → commit → figures zeroed → alert clears → (if applicable) next outstanding month's alert appears.
- Entry-lock enforcement: let a month elapse without closing, confirm the BV Entry screen shows the locked empty state and the persistent banner appears with no dismiss control.
- Closed-month correction end-to-end: correct an entry via the correction panel, confirm the warning copy, confirm the toast, confirm the exported closed-month snapshot reflects the correction.
- Settings warning end-to-end: save the slab table → warning shows before → after and affected members → Cancel leaves settings and typed values untouched → re-save applies. Repeat for royalty. Confirm a duplicate threshold is refused outright, no warning offered.
- Data-recovery screen: launch against an unreadable database, confirm the recovery screen appears in place of sign-in, restore points listed newest-first, states that anything after the chosen backup needs re-entering.
- Phone-duplicate-on-inactive reactivation flow, end-to-end through Add Member.
- Auth: setup wizard (recovery-code reveal + mandatory confirmation gate), login with each credential type, lockout countdown, recovery flow (old codes invalidated, new set issued).
- Whole-console backup/restore: schedule a due backup, confirm it fires at next login; restore on a fresh install from a file, confirm the same credential works unchanged.

**Security**
- Lockout: exactly 5 failed attempts triggers it; countdown timing; attempts do not reset early.
- Encryption at rest: confirm the raw `.sqlite` file is unreadable without the derived key (attempt to open with a plain SQLite client).
- Session key lifecycle: confirm the decryption key is not retrievable after `lock_session`, as far as testable without memory-forensics tooling.
- Filesystem capability allowlist: confirm the WebView cannot invoke any command outside the 40 documented ones.
- Pre-authentication surface: confirm `restore_from_backup`/`restore_from_backup_file` refuse a checksum mismatch, and a restored database still requires the credential to open.

**Performance**
Targets: screens < 2s, recalculation < 2s, extracts < 30s, at the **25,000-member design ceiling**. Client's actual scale: 500–5,000 members, ~1,000 entries/month.
- Chain-upward recalculation time vs. tree depth/width — confirm *O*(depth × average width) complexity, independent of total member count.
- Search response time at 25,000 members.
- Monthly export generation time at 25,000 members / a full year of entries.

**UAT — the five worked scenarios, reconciled by the client**
Re-run all five scenarios through the actual built UI (not just the isolated engine) and have the client confirm the on-screen figures match their own hand-worked numbers. **This is the single most important acceptance gate in the entire test strategy** — the client's own stated bar for trust (R-9).

### 3.2 What is explicitly not tested

| Item | Why |
|---|---|
| Concurrent-user/multi-writer scenarios | Structurally inapplicable — single-user, single-session |
| Network failure/retry handling | No network layer exists |
| Cross-browser/device compatibility | Desktop-only (NFR-15) |
| Data migration | No migration tooling in scope (NFR-16) |
| Monitoring/alerting on a silently-failed close | Explicitly declined by the client (NFR-12) — do not add a test for a feature deliberately not built |

### 3.3 Coverage map

| Requirement class | Verification |
|---|---|
| Calculation correctness (Rules 3–13, 25) | Unit + five golden scenarios + UAT |
| Recalculation trigger/scope (Rule-26) | Integration (chain-upward, transactional) |
| Entry/correction (Rules 16, 16a, 39) | Unit + E2E |
| Monthly close/backup gate (Rules 17–20, 31, 38) | Integration (atomicity) + E2E (wizard) |
| Member lifecycle/hierarchy integrity (Rules 28, 30, 34, 35, 37, 40) | Unit + E2E |
| Members never removed, never omitted (Rule-42) | Unit (no delete path) + integration (exports include deactivated members) |
| Exports/reporting (Rules 19, 23, 24, 33) | Integration + E2E |
| Settings pre-save warning | Unit (leaves nothing behind) + integration (prediction equals landed) + E2E |
| Slab table cannot be emptied | Unit + E2E |
| Data recovery at launch | Integration (corrupt file, checksum refusal) + E2E |
| Auth/security | Security tests + E2E |
| Pre-authentication command set is exactly seven | Contract test asserting the closed set |
| Performance/scale | Performance tests |
| Accepted risk — slab monotonicity | A deliberate **negative** test: confirm a non-monotonic table save is **not** blocked, and the resulting (possibly negative) differential computes and displays as-is, not silently clamped |

---

## 4. Acceptance criteria — AC-1 to AC-39

### 4.1 Calculation — the five worked examples

| # | Scenario | Total Rewards | Must match |
|---|---|---|---|
| **AC-1** | D with A, B, C beneath (300/50/1,000, D holds 500) | **35** | ✅ |
| **AC-2** | As AC-1 but C holds 3,000 | **22** | ✅ |
| **AC-3** | A with six people beneath at 1,250 each, three more beneath D | **450** | ✅ |
| **AC-4** | P with four people beneath, all top band | **1,000** | ✅ |
| **AC-5** | P with seven beneath — four top band, three lower | **980** | ✅ |
| **AC-6** | Scenario 3 also demonstrates the three people beneath D contribute **nothing directly** to A's reward — already inside D's team total | ✅ (this is what keeps the scheme self-limiting) |

### 4.2 Structure and members

| # | Criterion |
|---|---|
| **AC-7** | Exactly one top-level member exists; a second cannot be created by any route |
| **AC-8** | A member cannot be added with an introducer number that does not exist or belongs to an inactive member |
| **AC-9** | A member cannot be added on a contact number already in use; where it belongs to an inactive member, the system names them and offers reactivation |
| **AC-10** | A reactivated member keeps their original number, position and full history; no second record created |
| **AC-11** | Member numbers are six digits, random, within **100001–999999**, never reissued |
| **AC-12** | No route through the system changes an existing member's introducer |
| **AC-13** | No route through the system permanently removes a member |
| **AC-14** | Exceeding a level width or the depth setting warns and allows |

### 4.3 Recording and precision

| # | Criterion |
|---|---|
| **AC-15** | The entry screen accepts one Business Volume figure, up to two decimal places, no currency field anywhere |
| **AC-16** | Two decimal places held throughout; rounding only at display. A total of many terms matches a calculator |
| **AC-17** | On save, every affected figure to the top of the structure is correct with no further action, and no recalculate control exists |

### 4.4 Monthly close

| # | Criterion |
|---|---|
| **AC-18** | Once a month ends, an undismissable banner appears on every screen naming it, plus a notification entry |
| **AC-19** | All recording is locked while any month is outstanding; the entry screen names the month waiting |
| **AC-20** | The alert clears only on a completed close — not navigation, logout, or acknowledgement |
| **AC-21** | With several months outstanding, all are listed and only the oldest can be closed |
| **AC-22** | A failed or cancelled backup abandons the close — nothing is cleared, the alert stays up |
| **AC-23** | The permanent record is written before anything is cleared, capturing all six specified fields per member |
| **AC-24** | After a close, every live figure is zero and the month's record is retrievable in full |
| **AC-25** | Every backup is both downloaded and retained in the system; any past month can be re-downloaded |

### 4.5 Reporting

| # | Criterion |
|---|---|
| **AC-26** | The monthly extract carries the four defaults and any chosen columns |
| **AC-27** | The yearly average divides by the count of months holding a record, and displays that count next to every average |
| **AC-28** | The low-contribution report filters on the yearly average of the member's **own** Business Volume |
| **AC-29** | Every extract carries the member's basic details, contact number, volume, and Business Volume regardless of selection *(see [06](06-decision-log-and-open-items.md) O1 — this is Rule-19's five-field wording, awaiting reconciliation against V6.1's four)* |
| **AC-30** | All extracts open correctly in a standard spreadsheet application |

### 4.6 Settings, access and language

| # | Criterion |
|---|---|
| **AC-31** | Every setting in the inventory is editable by the client, unaided |
| **AC-32** | Slab rows can be added and removed; the top slab is always the highest-percentage row, and the royalty trigger follows automatically |
| **AC-33** | Both threshold examples work: 2% moved to 200, 6% moved to 1,000 |
| **AC-34** | Exactly one login exists — no member login, no second account |
| **AC-35** | Repeated failed attempts lock the account |
| **AC-36** | No excluded term appears in any screen label, button, column heading, extract filename, error message or tooltip |

### 4.7 Console backup & restore

| # | Criterion |
|---|---|
| **AC-37** | The whole console — every member, entry, monthly record and setting — can be backed up on a schedule (off/daily/weekly/monthly) or on demand; the most recent backups (default 10, adjustable) are kept, older pruned automatically |
| **AC-38** | Installing on a different computer and restoring from a backup file brings it to exactly the state the original held, no separate setup step, same login credential working unchanged |
| **AC-39** | Restoring always names what will be replaced and requires deliberate confirmation; the console backs up its own current state immediately beforehand |

---

## 5. Success criteria — SC-1 to SC-8

How the client and architect jointly judge whether the system succeeded.

| # | Criterion | How measured |
|---|---|---|
| **SC-1** | The client no longer performs any reward calculation by hand | Client's own confirmation after 3 months of live use |
| **SC-2** | Every figure the system produces matches a hand-worked check | All five worked examples reproduce exactly, plus spot checks during the first live month |
| **SC-3** | A member's question about their figure can be answered from one screen | Client demonstrates it, without leaving Member Detail |
| **SC-4** | No month is ever lost | Every month since go-live holds a permanent record and a retained backup |
| **SC-5** | Recording a figure takes under 15 seconds for a known member | Timed, during acceptance |
| **SC-6** | The client changes a scheme setting themselves, unaided | Done once during acceptance without the architect's help |
| **SC-7** | No commercial vocabulary appears anywhere visible | Full review of every screen, message, and extract filename |
| **SC-8** | Nobody but the client has ever accessed the system | Confirmed at review |

---

## 6. Definition of Done

Adapted to an offline, single-user, Tauri/React/Rust/SQLCipher desktop application with no server, no CI infrastructure yet, and no existing code.

### 6.1 Per user story

A story is **Done** only when all of the following are true:

1. **Implementation complete** — matches [delivery-plan.md](delivery-plan.md)'s acceptance criteria exactly, including every Given/When/Then case, not just the happy path.
2. **Code review complete** — this is currently a solo-maintainer project; review may be a self-review checklist pass at minimum, but must explicitly re-check the story against [02](02-business-rules.md) and §2 of this file for the rules it touches — not just against the acceptance criteria as written.
3. **Unit tests** written and passing for any calculation, validation, or state-transition logic touched (§3.1).
4. **Integration tests** written and passing for any change touching a transaction boundary (chain-upward recalculation, monthly close, closed-month correction versioning).
5. **API/contract tests** written and passing for any new or modified IPC command, matching its [04](04-technical-architecture.md) §6 contract exactly.
6. **E2E test** added or updated for any story that changes a user-facing workflow, run against the actual built UI, not a mock.
7. **Security validation** for any story touching auth, encryption, or the audit log: confirm no plaintext credential is logged/stored; confirm the WebView capability allowlist is unchanged unless the story explicitly extends it (and if so, the extension is justified in the commit description).
8. **Static analysis** — `clippy` (Rust) and the project's ESLint config (TypeScript, once Sprint 0 establishes it) pass with no new warnings.
9. **Dependency vulnerability check** — any new Rust crate or npm package checked against known advisories (`cargo audit`/`npm audit`) before being added. This matters here because a compromised dependency sits inside the same process as the encryption key.
10. **Database migration** — any schema change ships with a versioned migration, even though the system starts empty at launch — once real client data exists post-launch, ad-hoc schema edits are unacceptable.
11. **Logging** — any new mutating command writes an `audit_log` entry per its [04](04-technical-architecture.md) §6 contract; read-only commands correctly do **not** write one.
12. **Documentation** — this document set is updated if the story changes an API contract, data model entity, or business rule interpretation.
13. **Acceptance criteria verified** — manually walked through in the built application, not just asserted by automated tests, for any story with a UI component.
14. **Build passes** — the full Tauri build succeeds on both target platforms (Windows, macOS), not just on the developer's machine.

### 6.2 Per module (M1–M9)

A module is **Done** only when, in addition to every story within it meeting the per-story bar:

- Every rule attributed to that module ([02](02-business-rules.md) §7's map) shows a passing test, not just "documented."
- Any open item in [06](06-decision-log-and-open-items.md) §3 attributed to that module is resolved — not merely noted and deferred. As of this consolidation, **O1 gates only M6**; O2–O5 are build decisions that should be taken deliberately before the module ships, not defaults slipped in silently.
- The prototype-approved behaviours ported into that module — the settings recalculation warning, the last-slab-row refusal, the data-recovery screen, the console backup schedule/restore flows — match their approved reference behaviour exactly, not a re-interpretation.
- The five worked scenarios still reproduce their golden totals through the real UI (not just a unit test) once M2/M3/M4 are all Done together.

### 6.3 Project-level (pre-handover)

- All nine modules meet the module-level bar.
- **Full UAT pass:** the client reconciles all five scenarios and confirms the on-screen figures match their own hand-worked numbers (SC-2).
- Performance targets verified at the **25,000-member design ceiling**, not only at the client's actual 500–5,000-member scale.
- A full monthly-close cycle (backup → snapshot → zero → alert clears) exercised end-to-end at least once against realistic data volume.
- Handover deliverables match [01](01-product-and-scope.md) §12 exactly: installable desktop app, three working exports, backups verified working before anything is cleared, a working recovery-code path, an audit log that can explain any figure.
- CI pipeline (once Sprint 0 establishes one) is green — the repository currently has no CI configuration at all, so "CI passing" is only meaningful once Epic 0 creates it; do not silently skip this by treating its absence as satisfaction.

### 6.4 Explicitly not required — do not gold-plate against this DoD

- Cross-browser or mobile/tablet testing — out of scope by design (NFR-15).
- Load testing for concurrent users — structurally inapplicable, single-user system.
- Localization testing beyond English/Indian date format — no other locale is in scope (NFR-9).
- A monitoring/alerting test — the capability itself was explicitly declined by the client (NFR-12); do not build or test it "for completeness."

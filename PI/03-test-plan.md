# Test Plan — Tooling, Data, Schedule, Acceptance

**This is the test plan, not the test strategy.** The strategy — what to test and why — is `documents/refinement/05-quality-and-acceptance.md` §3, and is cited here, never restated. This file covers what that document does not: which tools, which data, which environments, in which sprint, entered and exited on what criteria, and how a defect is triaged.

---

## 1. Tooling

| Level | Tool | Rationale |
|---|---|---|
| **Unit — Rust** | Built-in `#[test]`, plus `proptest` for property-based cases | No framework needed. `proptest` earns its place for exactly one job: differential non-negativity across arbitrary trees and slab tables |
| **Unit — TypeScript** | Vitest | Matches the Vite toolchain Tauri v2 scaffolds with. Used only for pure frontend logic — formatting, canonicalisation mirrors, chart aggregation |
| **Integration** | Rust `#[test]` against a real temp SQLCipher database | Transaction boundaries are the thing under test, so a real database is the only honest fixture. No mocking layer |
| **API / contract** | Direct Tauri command invocation against the Rust core | **No HTTP layer exists**, so there is nothing to mock — no browser, no network stub, no test server |
| **E2E** | `tauri-driver` + WebdriverIO | The only WebDriver path into a Tauri application |
| **E2E — macOS** | ⚠️ **Manual, scripted checklist** | `tauri-driver` has no macOS support — WKWebView exposes no WebDriver endpoint. This is a platform limitation, not a tooling choice (D-8) |
| **Performance** | Purpose-built timing harness (US-QA.6) | The measurements needed — recalculation complexity against depth and width, and **main-console responsiveness during a concurrent draw** — are not what a generic benchmark tool measures |
| **Vocabulary** | Purpose-built grep (US-QA.4) | Nothing off the shelf knows this project's closed word list |
| **Static analysis** | `clippy`, ESLint | DoD item 8 |
| **Dependency advisories** | `cargo audit`, `npm audit` | DoD item 9 — a hard gate here, because a compromised dependency sits inside the same process as the encryption key |

**Nothing else gets added.** No coverage-percentage tool, no mutation testing, no snapshot-testing framework. The coverage bar in this project is the traceability matrix — *every rule attributed to a module has a passing test* — not a percentage.

---

## 2. Test data

Three datasets, all produced by the generator in US-QA.5, all **deterministically seeded** so a performance regression is measured against an identical tree rather than a fresh random one.

| Dataset | Members | Entries | Used for |
|---|---|---|---|
| **D-small** | 500 | ~1,000/month | Everyday E2E runs, the developer loop |
| **D-real** | 5,000 | ~1,000/month, one full year | The client's actual upper scale. UAT rehearsal, close-cycle rehearsal |
| **D-ceiling** | 25,000 | 200,000/year | NFR-2 design ceiling. Performance only |

Plus **D-golden**: the six worked-scenario trees from `02-business-rules.md` §5, held as data fixtures rather than test code (`T-QA.1-2`), so a seventh scenario is a row and not a new test.

⚠️ **Every generated name, address and string passes the vocabulary grep.** A fixture that fails it is a fixture that can never appear in a screenshot or a demo — and mock data does appear in both.

⚠️ **Every generated Business Volume amount is strictly `> 0`.** Zero is refused by Rule-16a; a dataset containing zero entries would be testing a state the system does not permit.

---

## 3. Environments

| Environment | What it is | Used for |
|---|---|---|
| **Dev-Win** | Windows development machine | Primary development, automated E2E, packaging |
| **Dev-Mac** | macOS development machine | macOS builds, manual verification checklist. ⚠️ **Required, not optional** — Tauri cannot cross-compile a macOS bundle from Windows |
| **Clean-Win** | Fresh Windows install, no toolchain | Installer verification, first-run path, certificate trust install (S15) |
| **Clean-Mac** | Fresh macOS install, no toolchain | Installer verification, Gatekeeper first-open step (S15) |
| **Baseline** | ⚠️ A deliberately **modest** machine | Argon2id cost-parameter tuning (TR-2). Parameters tuned on fast hardware feel sluggish on the client's |

No staging, no shared environment, no test server — there is nothing to host. Every environment is a local machine.

---

## 4. Test work by sprint

| Sprint | Test work | Entry criteria | Exit criteria |
|---|---|---|---|
| **S1** | Database foundation verification | Schema implemented | Fresh launch produces 10 tables, 7 slab rows, 16 settings; a plain SQLite client cannot read the file |
| **S2** | Vocabulary grep operational | Grep implemented | A planted excluded term fails the build; the specification-document allowlist works |
| **S4** | Golden fixtures + contract harness | US-0.2 Done | D-golden encoded as data; harness asserts **exactly seven** unauthenticated commands and **40** total, with the capability allowlist matching |
| **S5** | Member-lifecycle units | M1 stories implemented | ⚠️ **Deactivation neutrality passes** — the highest-value single test in the suite. ID allocation excludes 100000 and never reuses. TEST-R44 passes in **both** directions |
| **S6** | ⚠️ **Calculation-engine suite** | Engine implemented | **65 / 62 / 510 / 1,000 / 980 / 10 exactly.** Slab boundaries land in the higher slab. Royalty `min_children` boundary. Royalty stacking. OwnReward zero when own BV is zero. Ledger isolation. Fixed-point no-drift across a long chain. Property-based non-negativity **with its monotonicity assumption documented** |
| **S7** | Chain-upward integration; E2E rig live; correction versioning | M3 Done | One transaction per recalculation; a sibling's differential moves when the parent's slab moves; a closed-month edit leaves version 1 byte-identical |
| **S8** | ⚠️ **Golden scenarios through the real UI** | M4 views built | **Go/no-go for PI-2.** All six totals visible and correct on screen |
| **S9** | Full-hierarchy layout + responsiveness | Window built | Identical geometry across two runs on identical input; single-node and single-chain layouts correct; size gate names the real count; Cancel opens nothing; **console responsive during the draw** |
| **S10** | Settings tests incl. the negative monotonicity test; datasets generated | M7 implemented | ⚠️ A non-monotonic table saves **unblocked** and its (possibly negative) differential displays **as-is, not clamped**. Last slab row cannot be removed. D-small/D-real/D-ceiling exist and pass the grep |
| **S11** | ⚠️ **Close atomicity**; settings-preview equivalence | M5.1, M7.3 implemented | A mid-close verification failure mutates **zero** data. **The preview equals what actually lands.** API-33 leaves nothing behind, including on the panic path |
| **S12** | ⚠️ **Period catch-up**; ⚠️ **TEST-R36 full matrix**; period isolation; banner | M5.5/M5.2/M5.3/M2.2 implemented | ⚠️ **The application unopened across three month boundaries creates all three periods, all at `awaiting_close`, queued oldest-first, each accepting entries dated within itself; a second login creates nothing.** Every state in the TEST-R36 matrix behaves; recording into one live period leaves every other period's rows byte-identical; the banner has **no dismissal route of any kind** |
| **S13** | Empty-month close; export tests | M5.4, M6 implemented | ⚠️ **A month with zero entries raises its alert, closes through the full backup gate, writes no snapshot, and is absent from the yearly-average denominator** — the close succeeds with nothing to zero. Five mandatory columns always present; yearly average divides by the real snapshot count; low-contribution filters on **own** Business Volume; extracts open in a spreadsheet application; deactivated members are **included** |
| **S14** | Restore, audit completeness, performance harness | M8.2, M9 implemented | Checksum mismatch refused with the existing file untouched; a restore still requires the credential; every mutating command audits and every read-only one does not |
| **S15** | ⚠️ **Full performance run at D-ceiling**; security suite; clean-machine installs | Release candidate built | Screens < 2s, recalculation < 2s, extracts < 30s at 25,000 members; lockout ladder verified end to end; encryption-at-rest verified; the pre-authentication surface refuses a bad checksum |
| **S16** | ⚠️ **UAT** | S15 exit met | The client confirms all six scenarios match their own numbers |

---

## 5. Security test set

Run in full at S15, and per-story as auth work lands.

1. **Lockout** — exactly 5 failures triggers it; the countdown is accurate; ⚠️ **attempts do not reset early**; the ladder escalates (30s → 2min → 10min → 30min → 1h); ⚠️ **a process kill does not clear the state**.
2. **Encryption at rest** — the raw `.sqlite` file cannot be opened by a plain SQLite client.
3. **Session key lifecycle** — after `lock_session`, re-authentication is required and cannot be bypassed by any command. Key non-retrievability is asserted as far as testable without memory-forensics tooling; the limit is stated rather than pretended away.
4. **Capability allowlist** — the WebView cannot invoke any command outside the documented 40.
5. **Pre-authentication surface** — ⚠️ `restore_from_backup` and `restore_from_backup_file`, the only destructive unauthenticated commands, **refuse a checksum mismatch**, and a restored database **still requires the credential**. Restoring must not grant access to anything.
6. **Closed unauthenticated set** — asserted as **exactly seven**, by name. An eighth cannot join without the test failing. That is the point of the test.
7. **No plaintext credential** in the audit log, the technical log, or anywhere on disk.

---

## 6. UAT script

**The single most important acceptance gate in the project** (SC-2, R-9). Executed with the client, at S16, on a clean installed build against D-real.

### Part A — the six scenarios *(the gate)*

For each of Scenario 1–6 from `02-business-rules.md` §5: build the tree through the real UI, record the figures, and have the client compare the on-screen Rewards against their own hand-worked number.

| Scenario | Expected total |
|---|---|
| 1 — basic differential | 65 |
| 2 — differential collapses on an equal slab | 62 |
| 3 — multi-depth rollup | 510 |
| 4 — pure royalty | 1,000 |
| 5 — differential and royalty together | 980 |
| 6 — own-Business-Volume reward | 10 |

⚠️ **Through the built UI, not the engine in isolation.** A unit test passing is not this gate; the client seeing their own number on screen is.

### Part B — AC-1 to AC-47

Walked with the client, pass/fail recorded per criterion. Grouped as `05-quality-and-acceptance.md` §4 groups them: calculation, structure and members, recording and precision, monthly close, reporting, settings and access and language, console backup and restore, the 7 August change requests, the 8 August change requests.

### Part C — the eight success criteria

| # | Criterion | How measured at UAT |
|---|---|---|
| SC-1 | No hand calculation | Confirmed after 3 months of live use — hypercare, not UAT day |
| SC-2 | Every figure matches a hand check | Part A, plus spot checks during the first live month |
| SC-3 | A member's question answered from one screen | Client demonstrates it **without leaving Member Detail** |
| SC-4 | No month is ever lost | Every month since go-live holds a record and a retained backup — hypercare |
| SC-5 | Recording takes under 15 seconds | ⚠️ **Timed**, for a known member |
| SC-6 | The client changes a setting unaided | ⚠️ Done once, during acceptance, **without the architect's help** |
| SC-7 | No commercial vocabulary anywhere visible | Full human review of every screen, message and extract filename — alongside the automated grep, not instead of it |
| SC-8 | Nobody but the client has accessed the system | Confirmed at review |

---

## 7. Defect triage

| Severity | Definition | Action |
|---|---|---|
| **S1 — Ledger** | Any figure is wrong, or any golden total has moved | ⚠️ **Stop other work.** A moved total means a rule is implemented wrongly — find it before continuing, and never adjust the expected value to match |
| **S2 — Safety** | The close, backup gate, restore, or audit trail can lose or corrupt data | Fix before the sprint closes. No workaround is acceptable — NFR-12 means nothing will detect a silent failure in production |
| **S3 — Functional** | A rule or acceptance criterion is not met, but no data is at risk | Fix within the sprint, or move it with an explicit note against the story |
| **S4 — Presentation** | Visual or copy defect against `DESIGN.md` or the vocabulary constraint | Batch. ⚠️ Except vocabulary violations, which are **S3** — they are a binding client requirement (BC-4, SC-7), not a nicety |

**Before filing a defect, check `06-decision-log-and-open-items.md` §6.** Several behaviours that look like bugs are recorded client decisions:

- A negative differential is most likely the **accepted** non-monotonic slab table (Rule-41), not a code defect — check the slab configuration before the engine.
- The full hierarchy window not updating while open is Rule-45's **defining property**.
- The chart being extremely wide at scale is **TR-7**, accepted.
- The absence of any monitoring on a failed close is **NFR-12**, declined by the client.

---

## 8. Explicitly not tested

Carried forward from `05-quality-and-acceptance.md` §3.2 so nobody gold-plates against it.

| Item | Why |
|---|---|
| Concurrent-user / multi-writer scenarios | Structurally inapplicable — single-user, single-session (OC-1) |
| Network failure and retry handling | No network layer exists (ADR-001) |
| Cross-browser and device compatibility | Desktop-only (NFR-15) |
| Data migration | No import tooling is in scope (NFR-16) |
| Monitoring on a silently-failed close | ⚠️ Explicitly declined by the client (NFR-12). **Do not add a test for a feature deliberately not built** |
| Coverage percentage thresholds | The bar is the traceability matrix, not a number |

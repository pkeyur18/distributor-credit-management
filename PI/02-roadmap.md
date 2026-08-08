# Roadmap — PIs, Sprints, Gates and Risks

**2 programme increments · 16 two-week sprints · 161.0 ideal solo days.**

Relative numbering only. `S1` is whenever the first sprint starts; nothing in this file implies a calendar.

---

## 1. Shape of the plan

| | PI-1 — Foundation & Core Calculation | PI-2 — Configuration, Close, Reporting & Release |
|---|---|---|
| **Sprints** | S1–S9 | S10–S16 |
| **Effort** | 87.0d | 74.0d |
| **Delivers** | A console that can be set up, hold a hierarchy, record activity, and calculate every figure correctly on screen | Everything that depends on the core being stable — settings, close, reporting, console backup/restore, audit — then packaging, acceptance and handover |
| **Proves** | The six golden totals reproduce through the real UI | A month can be closed safely and the client accepts the system |

The two PIs are split at the point where correctness stops being the risk and safety starts being it. Everything in PI-1 can be re-done cheaply if it is wrong. Almost nothing in PI-2 can — the close is irreversible by design, and the backup gate before it is the only thing standing between a mistake and a lost month.

---

## 2. Sprint plan

### PI-1 — Foundation & Core Calculation

| Sprint | Load | Stories | Goal | Exit gate |
|---|---|---|---|---|
| **S1** | 9.0d | US-0.1, US-0.2, US-REL.1 | Stack scaffolding, encrypted database with the full 10-entity schema and both seed sets | A fresh launch creates the encrypted file with all 10 tables, 7 slab rows and **16** settings; a plain SQLite client cannot read it. **The `backups` generalization (ADR-012) has landed** |
| **S2** | 8.0d | US-UI.1, US-UI.2, US-UI.5, US-QA.4 | Design tokens, app shell, typed IPC layer, vocabulary grep | The shell renders in both themes from `DESIGN.md`'s tokens; the vocabulary grep runs and fails a build on a planted violation |
| **S3** | 8.5d | US-UI.3, US-UI.4 | The component library, built once | Every component named in `DESIGN.md` §Components exists, including the Structure Tree Node and the bar-list chart that both Home charts will share |
| **S4** | 10.25d | US-M1.1, US-QA.1, US-QA.2 | Member onboarding; the test harnesses that everything after this depends on | A member can be added with a random ID in 100001–999999; the golden-scenario fixtures exist as data; the contract harness asserts **exactly seven** unauthenticated commands and **40** total |
| **S5** | 12.75d | US-M1.2, US-M1.3, US-M1.4, US-M8.1, US-M8.2 | Member lifecycle and base authentication, in parallel — M8 has no data dependency on M1 | ⚠️ **Deactivating a mid-tree member with active descendants changes no ancestor's figures.** Lockout ladder (D-2) escalates and survives a process kill. Phone search behaves identically from every search box |
| **S6** | 11.0d | US-M3.1, US-M3.2 | **The calculation engine, alone.** Nothing else ships this sprint | ⚠️ **All six golden totals — 65 / 62 / 510 / 1,000 / 980 / 10 — reproduce exactly in unit test**, before any UI is built on top |
| **S7** | 11.0d | US-M2.1, US-M2.2, US-M8.3, US-QA.3 | Entry, correction, session lock, the E2E rig | An entry recalculates the ancestor chain on save with no recalculate control anywhere; a closed-month correction writes a new version and leaves version 1 byte-identical |
| **S8** | 9.5d | US-M4.1, US-M4.2, US-M4.4, US-M8.4 | Member detail, hierarchy chart, both Home charts, credential recovery | ⚠️ **All six golden totals reproduce through the real UI** — the project's go/no-go, and the client's own stated bar for trust (SC-2, R-9) |
| **S9** | 7.0d | US-M4.3 | The full hierarchy window | The size gate names the exact count above 60 descendants and **Cancel opens nothing**; the window draws every branch with three fields per node; **the main console stays responsive while it draws** |

### PI-2 — Configuration, Close, Reporting & Release

| Sprint | Load | Stories | Goal | Exit gate |
|---|---|---|---|---|
| **S10** | 12.5d | US-M7.1, US-M7.2, US-M7.4, US-QA.5 | Settings, console backup schedule, the synthetic dataset generator | All 16 settings editable; a non-monotonic slab table saves **without being blocked**; datasets exist at 500 / 5,000 / 25,000 members and pass the vocabulary grep |
| **S11** | 11.0d | US-M7.3, US-M5.1 | The settings pre-save warning, and the gated close | ⚠️ **A backup-verification failure mid-close mutates zero data** — no partial zeroing, no orphaned snapshot row |
| **S12** | 11.0d | **US-M5.5**, US-M5.2, US-M5.3, US-M2.3, US-M2.4 | The period state machine, the outstanding-month alert, and the entry-eligibility contract — built together because none is testable without the others | ⚠️ **The application unopened across three month boundaries produces all three periods, queued oldest-first, each accepting entries dated within itself.** ⚠️ **The full TEST-R36 matrix passes**: outstanding-month entry accepted, current-month refused naming the blocker, closed-month directed to correction, future refused, and after close the current-month figure saves. The banner has no dismissal route of any kind |
| **S13** | 12.0d | US-M5.4, US-M2.5, US-M6.5, US-M6.1, US-M6.2, US-M6.3, US-M6.4 | Empty-month handling, the month switcher, and the three extracts plus snapshot re-download | An empty month raises its alert and closes through the full backup gate while writing **no snapshot** and staying out of the averaging denominator; all five mandatory columns present regardless of selection (D-1); the yearly average divides by the real snapshot count with that count displayed; the low-contribution filter uses **own** Business Volume |
| **S14** | 11.5d | US-M8.5, US-M8.6, US-M9.1, US-QA.6 | Console backup/restore, audit-log completion, the performance harness | A restore on a second machine reaches the original's exact state with the same credential; every mutating command writes exactly one audit entry and every read-only command writes none |
| **S15** | 8.5d | US-REL.2, US-REL.3, US-REL.4, US-REL.5 | Pre-release gate, packaging, signing, clean-machine verification | ⚠️ **Performance targets met at the 25,000-member ceiling**; installers verified on clean Windows and macOS machines; cross-device restore proven (AC-38) |
| **S16** | 7.5d | US-REL.6, US-REL.7, US-REL.8 | UAT, handover, hypercare | ⚠️ **The client reconciles all six scenarios against their own hand-worked numbers and confirms the match.** Handover pack complete against `01-product-and-scope.md` §12 |

---

## 3. Dependency graph

```
S1  E0 (scaffolding) ──────────────────────────────────────────────┐
      └─ T-0.2-3 `backups` generalization (ADR-012) ───────────────┼──> M7.4, M8.5, M8.6
                                                                    │
S2-S3  E-UI (tokens, shell, components, IPC layer) ────────────────┤
S2     QA.4 vocabulary grep ───────────────────────────> REL.2      │
                                                                    │
S4     M1.1 ──┐                     QA.1, QA.2 (harnesses) ────────┤
S5     M1.2/M1.3/M1.4 ─┤            M8.1 ──> M8.2 ──> M8.3, M8.4    │
                       │            (base auth: no data dependency  │
                       │             on M1 — genuinely parallel)    │
                       ▼                                            │
S6                  M3.1 ──> M3.2   [the engine — blocks everything downstream]
                       │
        ┌──────────────┼──────────────────────┬─────────────────────┐
        ▼              ▼                      ▼                     ▼
S7    M2.1 ──> M2.2  S8 M4.1 ──> M4.2 ──> S9 M4.3    S10 M7.1/M7.2/M7.3
        │                    └─> M4.4                      (M7.3 needs M3)
        │
        ▼
S11   M5.1 ──> S12 M5.5 ──> M5.2 ──> M5.3        [M5.5 is the state machine
                     │                            M5.2/M5.3/M5.4 assume exists:
                     │                            without it nothing ever reaches
                     │                            awaiting_close]
                     │
                     └──> M2.3 ──> M2.4   [CR-2: the entry side of the
                                           same contract — cannot exist
                                           before the outstanding-period
                                           state does]
                     │
S13                  ├──> M5.4, M2.5
                     └──> M6.5 ──> M6.1 ──> M6.2 ──> M6.3, M6.4

S14   M8.5 ──> M8.6      QA.5 ──> QA.6      M9.1 (completeness check)
S15   REL.2/REL.3/REL.4 ──> REL.5
S16   REL.6 ──> REL.7 ──> REL.8

M9 (audit)  — cross-cutting. Wired into every mutating command from S4 onward.
```

---

## 4. What must not move

Six constraints. The first four are carried forward verbatim from `documents/refinement/delivery-plan.md` §4; the last two are added by this plan.

1. **S6 cannot start before S1–S5 are Done.** The calculation engine needs the schema and a member hierarchy to test against.
2. **S8's exit gate — all six golden totals reproducing through the real UI — is a genuine go/no-go**, not a nice-to-have. It is the client's own stated trust bar (SC-2, R-9). If it fails, nothing in PI-2 starts.
3. **US-M7.4, US-M8.5 and US-M8.6 cannot start before the `backups` generalization lands.** That is why `T-0.2-3` is in S1 — it is a schema decision, not a feature, and there is no reason to defer it to the sprint that consumes it.
4. **US-M9.1 is cross-cutting.** Wire the audit-log call into every mutating command *as it is built*, from S4. S14's US-M9.1 is a completeness check, not the first time audit logging is written.
5. **`T-M7.3-1` (`preview_settings_impact`) is built before the warning UI.** The prototype hid this dependency by running everything in one JavaScript scope; the real engine is Rust-side and the frontend cannot dry-run it.
6. **Feature M2.2 ships in S12 with Epic M5, not in S7 with the rest of M2.** The entry-eligibility rules and the outstanding-period state are two halves of one contract (CR-2) and neither is testable alone.
7. **`US-M5.5` runs before US-M5.2, US-M5.3 and US-M5.4.** It builds the period state machine those three assume already exists — until it runs, no period ever reaches `awaiting_close`, so the alert has nothing to display and entry eligibility has nothing to gate on. The source specification never assigned this work to anything; it is new as of D-9.

---

## 5. Milestones

| # | Milestone | Sprint | Evidence it happened |
|---|---|---|---|
| **M-1** | Encrypted database foundation | S1 | Fresh launch creates 10 tables + 16 settings + 7 slab rows; plain SQLite client fails to read the file |
| **M-2** | Design system built once | S3 | Every `DESIGN.md` component exists; the bar-list chart is a single component both Home charts will use |
| **M-3** | Hierarchy and access | S5 | A hierarchy can be built; login, lockout ladder and session lock work; **the deactivation-neutrality regression test passes** |
| **M-4** | **Calculation engine correct** | S6 | Six golden totals in unit test |
| **M-5** | **Six golden totals through the real UI** | S8 | Go/no-go for PI-2 |
| **M-6** | Whole network viewable without slowing the console | S9 | Size gate, full draw, main console responsive |
| **M-7** | A month can be closed safely | S11–S12 | Backup-gate atomicity test; period catch-up across three unopened months; TEST-R36 matrix |
| **M-8** | Reporting complete | S13 | Three extracts open in a spreadsheet application |
| **M-9** | Console movable between machines | S14 | Restore on a second machine, same credential, no re-setup |
| **M-10** | Release candidate | S15 | Pre-release gate green, installers verified on clean machines, performance met at ceiling |
| **M-11** | **Client acceptance** | S16 | Six scenarios reconciled by the client; AC-1–AC-47 walked; handover pack delivered |

---

## 6. Where the schedule risk actually lives

Not evenly spread. Four sprints carry almost all of it.

| Sprint | Why it is the risk | Mitigation |
|---|---|---|
| **S6** — the engine | Fixed-point bugs are subtle and silently misstate every downstream figure (TR-5). Nothing else in the project has this blast radius | The six-scenario suite exists **before** the engine (S4's US-QA.1), not after. Nothing else ships in S6 — the sprint is deliberately single-purpose |
| **S9** — full hierarchy | A layout pass over 25,000 nodes with a hard responsiveness constraint (AC-45) and an accepted-but-severe width behaviour (TR-7) | Separate window by design, so the cost never lands on the console. Single post-order pass, never DOM measurement. The named fallback is the indented outline — **raise it as a change request, do not switch layouts unilaterally** |
| **S11** — the gated close | Irreversible by design. A partial close is unrecoverable, and NFR-12 means **nothing will detect it** — monitoring was explicitly declined | Atomicity test before the UI. Strict ordering: verify backup → snapshot → zero → mark closed, with verification failure never reaching the zeroing phase |
| **S12** — period state + entry eligibility | Rule-36 was reversed by CR-2 and the superseded wording appears throughout the 6 August documents. Implementing the stale version is a live risk. **Compounded by US-M5.5**: the period state machine had no trigger anywhere in the source specification until D-9, so there is no reference implementation to check against | TEST-R36's full matrix is the exit gate, plus the three-unopened-months catch-up test. The schema value is `awaiting_close` precisely so the code cannot state the opposite of the behaviour |

S2, S3, S9, S15 and S16 have slack by comparison. If something must give, take it from there first.

---

## 7. Capacity honesty

161.0 ideal days over 16 sprints assumes **10 fully productive days per sprint**. That is optimistic for a solo maintainer who is also the reviewer, the tester and the release engineer.

**Realistic range: 18–21 sprints.** Treat 16 as the floor.

If the schedule must compress, these are the only defensible cuts, in order — each with what it costs:

| Cut | Saves | Costs |
|---|---|---|
| US-M2.5 (month selector for multiple outstanding months) | 1.5d | The client has stated this will not arise in practice. Cutting it means the system is *wrong* if it ever does |
| US-QA.6 + ceiling-scale performance testing | 2.0d+ | Performance verified only at the client's actual 500–5,000 scale, not the 25,000 ceiling. **Violates DoD §6.3** |
| Full manual macOS checklist depth (`T-QA.3-3`) | 0.5d | macOS is already E2E-uncovered by D-8. Cutting the checklist too leaves it genuinely untested |

**These are not cuttable, at any price:** the six-scenario suite, the deactivation-neutrality test, the close atomicity test, TEST-R36, the backup gate, the vocabulary grep, UAT. Each one is either a client requirement or the only thing standing between a silent defect and a corrupted ledger.

---

## 8. Risk register

TR-1 to TR-7 carried forward from `04-technical-architecture.md` §11, plus the four risks this plan introduces.

| ID | Risk | Likelihood | Impact | Mitigation | Sprint |
|---|---|---|---|---|---|
| **TR-1** | SQLCipher's bundled build is slow to compile and fragile across toolchain updates | Medium | Low (build-time) | Pin exact versions; document the known-good toolchain | S1 |
| **TR-2** | Argon2id parameters tuned on a fast machine feel sluggish on the client's | Low | Medium | Tune against a deliberately modest baseline before handover | S5 |
| **TR-3** | Tauri v2's plugin ecosystem may not cover a needed capability | Medium | Low (more time, not a design flaw) | Accepted trade-off of ADR-002; budgeted | ongoing |
| **TR-4** | Single-machine data loss if the client never takes the external-medium copy | Medium | **Critical** | Prompted and reminded at every close, then **taught explicitly at handover** (`T-REL.7-3`). Ultimately client process discipline — stated plainly, not solved | S16 |
| **TR-5** | Fixed-point bugs silently misstate every downstream figure | Low *(if the six-scenario suite is followed)* | **Critical** | The suite exists specifically to catch this class of bug **before any UI is built on top** | S6 |
| **TR-6** | Solo maintainer, no second reviewer — a design flaw could ship unnoticed | Medium | Medium–High | The specification set is the primary mitigation. DoD item 2 requires a self-review pass **against the rules a story touches**, not just its acceptance criteria | ongoing |
| **TR-7** | The full hierarchy chart's width grows with leaf count; extreme at 25,000 members | Medium (large networks only) | Medium (one view's usability, no data risk) | **Chosen deliberately by the client** over a width-stable outline. Mitigations agreed at the same time: 10% zoom floor, fit-width, in-window search, size gate, separate window | S9 |
| **PR-1** | **No CI (D-7).** DoD §6.3 explicitly warns against treating CI's absence as satisfaction | High | Medium | The scripted local pre-release gate (US-REL.2) runs the same checks. Recorded as a dated deviation, not dropped. **The residual risk is discipline: nothing forces the gate to run** | S15 |
| **PR-2** | **macOS is E2E-untested (D-8).** `tauri-driver` cannot drive WKWebView | High | Medium | A scripted manual verification checklist, run every release (`T-QA.3-3`, `T-REL.5-4`) | S7, S15 |
| **PR-3** | **The self-signed certificate is trusted only where it is installed (D-6)** | Certain | Low *for this deployment* | One machine, one user, offline hand-delivery. A second machine — including the cross-device restore target — needs the same one-time trust install. **Named in the runbook so it is not discovered at the client's desk** | S15 |
| **PR-4** | **Estimates assume 10 productive days per sprint** | High | Medium | §7's realistic 18–21 range and its ordered cut list. Re-baseline after S3, when three sprints of actual velocity exist |

---

## 9. Definition of Done, per level

Not restated — `documents/refinement/05-quality-and-acceptance.md` §6 owns all three tiers. Two additions this plan makes explicit:

- **Item 8 (static analysis) and item 9 (dependency vulnerability check)** run inside the pre-release gate (US-REL.2), since no CI exists to run them per commit. They are still per-story requirements; the gate is the backstop, not the substitute.
- **Item 14 (build passes on both platforms)** is verified at the gate and again on clean machines at S15, not on the developer's machine.

The three deviations — D-6, D-7, D-8 — are recorded in [05-decisions-and-gaps.md](05-decisions-and-gaps.md) and must be **accepted explicitly at project close**, not quietly passed over.

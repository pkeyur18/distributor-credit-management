# Decisions, Deviations & Remaining Gaps

**Fourteen decisions** taken on **8 August 2026**. They are newer than every document in `documents/refinement/`, so **until §4's propagation tasks are executed, this file is the authority for these fourteen items alone** — and for nothing else.

They arrived in two passes:

- **D-1 to D-8, during PI planning.** Five close O1–O5, the open register that `06-decision-log-and-open-items.md` §3 deliberately left unfilled. Three are new, arising from release and deployment planning that no source document covered.
- **D-9 to D-14, during the pre-development readiness audit** — the pass that traced the four ID namespaces the first one skipped (`V1.1–V8.5`, `M1.1–M8.7`, `RQ-1–23`, the 63-scenario error matrix). ⚠️ **Three of these are defects in the specification itself, not gaps in the plan** — two documents contradicting each other, and two behaviours nobody had assigned to any code. They would have stalled M3 or M5 mid-sprint.

---

## 1. Decisions closing the open register

### D-1 — O1: mandatory export columns → **five**

| | |
|---|---|
| **The question** | Rule-19 and AC-29 name five things carried by every extract — basic details, contact number, volume, Business Volume — reading *volume* as Total Business Volume. Rule-33, V6.1 and US-M6.1 name only **four**: name, member number, phone, Business Volume, with Total Business Volume in the *optional* list |
| **Decision** | **Five mandatory columns: name, member number, phone, Business Volume, Total Business Volume.** Untickable on all three extracts |
| **Decided by** | Architect, 8 Aug 2026, on the client's own Rule-19 wording |
| **Why** | Rule-19 is the client's own statement of what an extract must always carry. Rule-33 governs what *starts ticked*, which is a different question the prototype conflated — a *default* is not a *mandatory* column. Resolving toward the client's words rather than the prototype's convenience |
| **Consequence** | Rule-19 and AC-29 become literally correct. **V6.1's "the four default columns are always present and cannot be removed" is wrong by one.** Total Business Volume leaves the optional list entirely. The seeded default column set changes from four entries to five. Implemented by `US-M6.5`, which **blocks US-M6.1** |
| **⚠️ Worth confirming with the client** | This is architect-resolved, not client-confirmed. It resolves *toward* the client's own wording and is therefore safe to build, but the next client conversation should note it |

### D-2 — O4: the lockout ladder

| | |
|---|---|
| **The question** | Every source agrees lockout is mandatory at **5 consecutive failures**. Beyond that, sources disagree and none is complete: two say "exponential backoff" without a ladder; the prototype implements a **flat 20 seconds and then resets the failure counter to zero** |
| **Decision** | Lockout at 5 failures and at every 5 further failures, with durations **30s → 2min → 10min → 30min → 1h (capped)**. The counter resets **only on a successful login**, never on lockout expiry. State persisted in the database, surviving a process kill |
| **Decided by** | Architect, 8 Aug 2026, as a security design decision |
| **Why** | This is the only control between a 6-digit PIN — one million combinations — and an attacker with the machine. ⚠️ **The prototype's behaviour was demo pacing, not a security decision**: a flat 20 seconds with a counter reset gives a patient attacker unlimited batches of five, roughly 4 seconds per attempt sustained, which is not meaningful protection over hours. The cap at 1 hour keeps a fat-fingering client from locking themselves out for a day with recovery codes as the only way back |
| **Consequence** | **No schema change** — the tier derives from `auth.failed_attempts`, and `locked_until` already exists. `05-quality-and-acceptance.md` §2's M8 row ("Ladder beyond this point is undefined") is now answerable. The test `08-testing-strategy.md` prescribes — "countdown timing; attempts do not reset early" — becomes writable |

### D-3 — O2: hierarchy depth default → **4**

Advisory only (Rule-1, Rule-32 — depth overflow warns, never blocks), so a wrong value cannot break anything; it only changes when the advisory warning fires. **4** matches the approved prototype and the level-width coverage, which is specified for levels 2, 3 and 4 only. Flagged on the Settings screen as guidance the client should set to their own intended shape. Architect decision, 8 Aug 2026.

### D-4 — O3: session inactivity timeout → **15 minutes**

Matches the approved prototype, editable on Settings. ⚠️ **This contradicts `architecture.md` Appendix B's "set at setup" wording** — the prototype puts it on the Settings screen, not in the setup wizard, and the built behaviour wins (tier 4). Security-relevant: it bounds the window in which a stolen unlocked machine is exposed, a threat explicitly out of the model and defended only by this timer. Architect decision, 8 Aug 2026.

### D-5 — O5: advisory level widths above level 4 → **suppress**

Widths are configured for levels 2, 3 and 4 (defaults 9 / 6 / 3); depth is separately configurable and can exceed 4. **The width warning is suppressed entirely for any level with no configured width.** The depth warning (Rule-32) still fires independently. ⚠️ **Do not silently reuse level 4's width for level 5+** — that would produce a warning the client never configured. Nothing is ever blocked either way, so the consequence is cosmetic. Architect decision, 8 Aug 2026.

---

## 2. Decisions and deviations from release planning

These three cover ground no source document addressed. Each is a **deviation from a stated requirement**, recorded as one rather than quietly absorbed.

### D-6 — Code signing: Windows self-signed, macOS unsigned

| | |
|---|---|
| **The requirement** | `04-technical-architecture.md` §10: "Code signing: required on both platforms to avoid 'unknown publisher' warnings that would confuse a low-technical user" |
| **Decision** | **Windows:** self-signed code-signing certificate, with a one-time install into the client's Trusted Root / Trusted Publishers store. **macOS:** unsigned and un-notarized, with a documented one-time Gatekeeper first-open step. **Paid CA certificates deferred**, not refused |
| **Why** | Windows Authenticode from a trusted CA costs money (~$200–600/year) and since June 2023 requires the private key on FIPS 140-2 L2 hardware even for OV. macOS notarization requires a paid Apple Developer Program membership. Neither buys anything **this** deployment needs: one machine, one user, installer hand-delivered on physical media. The self-signed path removes the "unknown publisher" dialog completely and costs nothing; a USB-copied file carries no Mark-of-the-Web, so SmartScreen never engages either — and SmartScreen reputation is independent of signature validity, so even a paid OV certificate would not reliably avoid it |
| **⚠️ Ceiling** | **The certificate is trusted only on machines where it has been installed.** This is not a general distribution solution. The cross-device restore target (AC-38) is a second machine and needs the same one-time install — named in the runbook so it is not discovered at the client's desk |
| **Reversal trigger** | The application ever needs distributing beyond machines the maintainer physically touches |

### D-7 — No CI pipeline

| | |
|---|---|
| **The requirement** | `05-quality-and-acceptance.md` §6.3: "CI pipeline (once Sprint 0 establishes one) is green — the repository currently has no CI configuration at all, so 'CI passing' is only meaningful once Epic 0 creates it; **do not silently skip this by treating its absence as satisfaction**" |
| **Decision** | **No CI.** Replaced by a scripted local pre-release gate (`US-REL.2`) running the same twelve checks: clippy, ESLint, cargo audit, npm audit, unit + integration suite, golden-scenario re-verification, contract suite, E2E suite, vocabulary grep, both-platform builds, macOS manual checklist |
| **Why** | Solo project, offline product, no shared repository or deployment target for a pipeline to serve. macOS runners would be the main cost driver and would exist only to run a build one developer already runs locally |
| **⚠️ Residual risk (PR-1)** | **Nothing forces the gate to run.** CI's real value is not the checks, it is that they are unskippable. A script is skippable. Running it is a release-checklist item and the checklist is the only enforcement — this is a genuine reduction in assurance, stated as one |
| **Recorded as** | An explicit, dated deviation from DoD §6.3 — exactly what §6.3 asks for instead of silent omission |

### D-8 — E2E automated on Windows only

| | |
|---|---|
| **The requirement** | DoD §6.1 item 6 (E2E against the actual built UI) read together with item 14 (build passes on both platforms) |
| **Decision** | E2E automated on **Windows only**. macOS covered by a scripted manual verification checklist (`T-QA.3-3`), run every release |
| **Why** | ⚠️ **Not a tooling preference — a platform limitation.** `tauri-driver` has no macOS support because WKWebView exposes no WebDriver endpoint. There is no alternative WebDriver path into a Tauri application on macOS |
| **⚠️ Residual risk (PR-2)** | macOS regressions are caught by a human following a list, or not at all. The checklist is the deliverable that stops this becoming "untested" |

---

## 3. Decisions from the pre-development readiness audit

Found by tracing the four namespaces the first planning pass did not: validation rules, module functions, client-answered questions, and the error/edge matrix. Each is a **defect in the source specification**, recorded as one.

### D-9 — The period lifecycle has no trigger ⚠️ *would have blocked M5*

| | |
|---|---|
| **The defect** | `04-technical-architecture.md` §7.1: a period row is created *"as soon as the calendar month begins."* `05-data-model-specification.md`: created *"implicitly, when the first entry of a new month is recorded, **or** explicitly at month-start."* The two disagree, the second is an unresolved either/or, and **neither names the code that runs**. There is no background service while the application is closed — already established and accepted for the backup schedule (RQ-23, 7 Aug 2026) |
| **Why it blocks** | Without a trigger, `open → awaiting_close` never fires. Rule-20's undismissable alert never appears, `get_period_lock_status` has no `awaiting_close` period to report, and Rule-36's entry eligibility has nothing to gate on. Three stories — US-M5.2, US-M5.3, US-M5.4 — assume this state machine already runs |
| **Decision** | **A catch-up routine at successful login.** It creates a period row for every calendar month up to and including the current one that has none, then transitions every elapsed `open` period to `awaiting_close`. It runs **before** the backup-schedule check and **before** the UI takes over, so the banner and entry eligibility are correct on the first frame the operator sees |
| **Decided by** | Architect, 8 Aug 2026 |
| **Why login** | ⚠️ It is the only point the application is reliably running — **the same design constraint already accepted for the whole-console backup schedule, and equally not a gap to "fix" with a background timer.** A missed month catches up at the next login exactly as a missed backup does |
| **Consequence** | New story **US-M5.5** (S12, 2.5d, 6 tasks), blocking US-M5.2/M5.3/M5.4. Must handle the application being unopened across several month boundaries — every intervening month gets a row, none is skipped, all queue oldest-first, and each accepts entries dated within itself. Must be idempotent: a second login creates nothing |

### D-10 — An empty month closes like any other

| | |
|---|---|
| **The question** | `US-M5.4`'s criterion reads *"when it becomes eligible for close"*, implying a month with zero entries is closable. But nothing states whether Rule-18's backup gate applies to a month with nothing to zero, whether Rule-20 raises its alert for it, or whether Rule-36 blocks the current month behind it |
| **Decision** | **Exactly like any other month.** Period row, undismissable alert, full close wizard **including the backup gate** — and **no snapshot written** (RQ-16), so it stays out of the yearly-average denominator |
| **Decided by** | Architect, 8 Aug 2026 |
| **Why** | It is the reading `US-M5.4` already carries, so nothing else needs amending. And the backup is a copy of the **whole database**, not of that month — so it is a genuine restore point whether or not the month it closes holds anything. Skipping the gate would have meant writing an exception into Rule-18's otherwise unqualified wording, for no safety gain |
| **Consequence** | `T-M5.5-5` and US-M5.4. The operator returning after a quiet month works through a close wizard with nothing in it — accepted ceremony, in exchange for one close path rather than two |

### D-11 — Slab lookup is ordered by threshold, not by `sort_order` ⚠️ *highest blast radius in the project*

| | |
|---|---|
| **The defect** | Rule-3: *"Slab table queried in **threshold-descending order**; first match wins."* `05-data-model-specification.md`, on `slab_table.sort_order`: *"Determines **lookup order**, not necessarily equal to threshold order if misconfigured."* A direct contradiction about how every slab in the system is resolved |
| **Why it matters** | Rule-41 is the accepted risk that the table may be non-monotonic — which is precisely the configuration where threshold order and `sort_order` diverge. Under the data model's reading, dragging a row on the Settings screen would silently change every computed figure |
| **Decision** | **Threshold order governs.** The engine scans thresholds descending and takes the first ≤ Total Business Volume. **`sort_order` is display-only** — it orders rows on the Settings screen and is read by nothing else |
| **Authority** | Rule-3 is tier-2; the data model is tier-3. Precedence settles it without needing a judgement call. **The data model is corrected, not the rule** |
| **Consequence** | `T-M3.1-1` states it explicitly. P-9 corrects `05-data-model-specification.md`. ⚠️ Related and easily confused: **Rule-10's "top slab" means the highest-*percentage* row, whatever its threshold** — also not the highest threshold, and also divergent under a non-monotonic table. `T-M3.1-4` now says so |

### D-12 — `audit_log.entity_type` extended

| | |
|---|---|
| **The defect** | The enum is `member \| entry \| setting \| period`. But an audit entry is **required** for `manual_backup_current_period` (API-15), `run_console_backup_now` (API-39), `setup_first_run` (API-26), `use_recovery_code` (API-30), and restore success (API-36/API-40). **None of those five has a valid `entity_type` to write.** M9's completeness check would fail against a schema that cannot represent the events |
| **Decision** | Extend to **`member \| entry \| setting \| period \| backup \| auth`** |
| **Consequence** | `T-0.2-2`. No implementation exists, so this costs nothing now and a migration later |

### D-13 — `audit_log.cause` extended, and `reversal` retired

| | |
|---|---|
| **The defect** | The enum stops at `manual_backup`, but API-39 specifies a **`console_backup`** cause, and restore success must be audited with no cause to use |
| **Decision** | Add **`console_backup`** and **`restore`**. **Retire `reversal`** — `reverse_entry` was confirmed dead and dropped; `edit`/`correction` cover every case, and the data model itself recommends retiring it |
| **Consequence** | `T-0.2-2`, `T-M9.1-3` |

### D-14 — Session timeout has one home

| | |
|---|---|
| **The defect** | `05-data-model-specification.md` gives `auth.session_timeout_minutes` as *"From settings, duplicated/cached here **or** read from `settings`"* — an unresolved either/or. `06-security-authorization-matrix.md` §3 names `settings.session_timeout_minutes`. Two places that can drift, for a security-relevant value |
| **Decision** | **`settings` only.** `auth.session_timeout_minutes` is not created |
| **Why** | It bounds the window in which a stolen unlocked machine is exposed — a threat defended by nothing else. A cached second copy buys one avoided lookup per idle tick and risks the timer running to a stale value |
| **Consequence** | `T-0.2-2`, `T-M8.3-2` |

### Corrections applied without a decision

Each traces to a source document or to a decision above; none needed a judgement call.

| # | Correction | Where |
|---|---|---|
| 1 | ⚠️ **`03-functional-specification.md` §6 says "the complete set of 49" validation rules. The actual count is 50** — V4.6 was appended by CR-5 and the count was never bumped. `00-master-index.md` §4 already says 50 | P-11 |
| 2 | **Six validation rules had no task**: V1.1 (name required), V1.4 (email valid if given), V6.3 and V7.1 (positive thresholds), **V7.2 (percentages 0–100)**, V7.4 (royalty count a positive whole number) | `T-M1.1-10`, `T-M7.1-7`, `T-M7.2-8` |
| 3 | **No task for the deactivate confirmation modal** — `confirmDeactivate` exists in the approved prototype | `T-M1.3-7` |
| 4 | ~~P-1 did not clear Rule-19's own ⚠️ marker~~ — **executed in full during S13** (`T-M6.5-3`), including Rule-19's marker and the `04-technical-architecture.md` API-16 row P-1's original file list omitted | P-1, done |
| 5 | **`PI/06-traceability.md` omitted four namespaces** — V-rules, module functions, RQ, error matrix | [06-traceability.md](06-traceability.md) §§6–9 |

⚠️ **On correction 2:** V7.1 and V7.2 are **per-field range checks** and are *not* the cross-row monotonicity check Rule-41 forbids. Refusing a percentage of 400 is not the same as refusing a table whose percentages fall as thresholds rise. Do not let one be removed in the belief it violates the other.

---

## 4. Propagation tasks

`00-master-index.md` §8 requires that a decision be propagated, not merely recorded: *"An open item (O1–O5) is answered — move it out of §3 into §2 with the answer, its date, and who gave it. Update every file that depends on it."* These are those updates. **Until they are done, `documents/refinement/` still shows five open items and a four-column export set.**

| ID | Task | Files | Owner | Due before |
|---|---|---|---|---|
| **P-1** | ✅ **Done, S13.** Recorded D-1 (as C9) and amended the four-column statements to five, including Rule-19's own ⚠️ marker. Also amended `04-technical-architecture.md`'s API-16 row and `05-quality-and-acceptance.md`'s AC-26/AC-29, which the original file list below omitted — see correction 4 above | `06-decision-log-and-open-items.md` (moved O1 §3 → §2 as C9), `02-business-rules.md` **Rule-19's warning line**, Rule-33 and §6 row 12, `03-functional-specification.md` V6.1 and §5.8, `delivery-plan.md` US-M6.1 AC, `04-technical-architecture.md` API-16 row, `05-quality-and-acceptance.md` AC-26/AC-29 | Architect | **S13** (`T-M6.5-3`) |
| **P-2** | Record D-2 and the ladder | `06-decision-log` (O4 §3 → §2), `05-quality-and-acceptance.md` §2 M8 row, `03-functional-specification.md` §5.10 Login | Architect | **S5** |
| **P-3** | Record D-3, D-4, D-5 | `06-decision-log` (O2/O3/O5 §3 → §2), `02-business-rules.md` §6 rows 4 and 13 (drop the "no source default" annotations) | Architect | **S1** (D-3, D-4 are seed values), **S4** (D-5) |
| **P-4** | Record D-6, D-7, D-8 as deviations | `04-technical-architecture.md` §10, `05-quality-and-acceptance.md` §6.3 | Architect | **S15** |
| **P-5** | ⚠️ **Fix the documentation path drift.** The refinement set cites `documents/final/` throughout its cross-references, but the folder is `documents/refinement/`. Every internal link is stale by path — including the "read this first" pointers a future session depends on | All of `documents/refinement/`, plus `PRODUCT.md` §Evidence and `documents/implementation-readiness/` | Architect | **S1** — a future build session following a broken link is exactly the failure mode `00-master-index.md` exists to prevent |
| **P-6** | Fold the three new epics into the source delivery plan, or mark it superseded by `PI/` for sequencing | `delivery-plan.md` §4 (its 8-sprint proposal predates E-UI/E-QA/E-REL) | Architect | **S1** |
| **P-7** | ⚠️ Record D-9 — **name the login catch-up as the period lifecycle's trigger.** Resolve the data model's "implicitly on first entry **or** explicitly at month-start" either/or in favour of the catch-up routine | `04-technical-architecture.md` §7.1, `05-data-model-specification.md` (`periods` Lifecycle) | Architect | **S11** — before US-M5.5 is built |
| **P-8** | Record D-10 — empty months raise the alert and close through the full backup gate, writing no snapshot | `02-business-rules.md` Rule-18/Rule-20 notes, `05-quality-and-acceptance.md` §2 M5 empty-month row | Architect | **S12** |
| **P-9** | ⚠️ Record D-11 — **correct `slab_table.sort_order` from "determines lookup order" to display-only.** As written it contradicts Rule-3 about how every slab in the system resolves | `05-data-model-specification.md` (`slab_table`) | Architect | **S6** — before the engine is built |
| **P-10** | Record D-12, D-13, D-14 — the two audit enums and the dropped `auth` column | `05-data-model-specification.md` (`audit_log`, `auth`), `04-technical-architecture.md` §4.4 DDL | Architect | **S1** — the schema lands there |
| **P-11** | Correct the validation-rule count **49 → 50** (V4.6 was appended by CR-5 and never counted) | `03-functional-specification.md` §6 opening line | Architect | **S1** |

---

## 5. Gaps this plan does not close

Stated with an owner and a trigger, rather than left implicit.

| ID | Gap | Owner | Trigger | Consequence if unaddressed |
|---|---|---|---|---|
| **G-1** | **D-1 is architect-resolved, not client-confirmed.** It resolves toward the client's own Rule-19 wording, so it is safe to build — but the client has not said so | Architect → client | Next client conversation, before **S13** | Low. If the client wants four, `US-M6.5` reverses in under a day |
| **G-2** | **No second reviewer** (TR-6). DoD item 2 permits a self-review pass, which is a mitigation, not a solution | Architect | Ongoing | A design or security flaw ships unnoticed. The specification set is the primary defence |
| **G-3** | **Dev-Mac is assumed available.** Tauri cannot cross-compile a macOS bundle, so without it the macOS target is undeliverable, not merely untested | Architect | **S1** | The macOS target silently becomes out of scope |
| **G-4** | **Clean-Win and Clean-Mac machines assumed available** for S15's installer verification. A virtual machine satisfies this; a developer machine with a toolchain does not | Architect | **S15** | Installer verification becomes theatre — the one check that catches "works on my machine" |
| **G-5** | **The client's own machine and platform are not recorded anywhere** in the source set. Windows and macOS are both targets, but which one the client actually uses daily determines which platform's untested paths matter most — and D-8 leaves macOS E2E-uncovered | Architect → client | Before **S15** | If the client is on macOS, the platform with no automated E2E is the only one in production |
| **G-6** | **`period_close` and `pre_restore_safety` backups are never pruned** (Rule-43) and retention is permanent (Rule-31, OC-5). Disk growth is unbounded over years | Architect | Post-handover | Not a defect — permanent retention is the requirement. Worth naming at handover so it is not a surprise in year three |
| **G-7** | **No rollback path for a bad release.** Without auto-update, recovery from a defective installer is: deliver the previous installer and restore from a console backup | Architect | **S15** | Named in the maintainer runbook (`T-REL.7-6`) rather than solved |
| **G-8** | ⚠️ **Period state is only as current as the last login (D-9).** If the client does not open the console for two months, no period transitions and no outstanding-month alert exists until they do — the alert cannot warn someone who is not looking at the screen | Architect → client | Handover (`T-REL.7-2`) | **Inherent, not solvable**: there is no background service, by the same constraint that governs the backup schedule. UN-19 ("impossible to lose a month by forgetting") is satisfied at the point the operator returns, not before. Worth saying plainly at handover so it is not mistaken for a defect |

---

## 6. Do not re-raise

Beyond `06-decision-log-and-open-items.md` §6's own list, three things about **this plan** will look like mistakes and are not:

| Item | Why it is deliberate |
|---|---|
| **16 sprints, not 8** | `delivery-plan.md` §4's eight-sprint proposal predates the three epics this plan adds — UI foundation, test foundation, and release/handover. Absorbing them into eight sprints without moving the boundary would be a schedule fiction, not a tighter plan |
| **`documents/refinement/delivery-plan.md`'s 36 stories are cited, not restated** | Their acceptance criteria live there and only there. Copying them here would create a second copy to keep in sync — the precise failure this project already paid for once, across nineteen source documents |
| **Estimates total 157.5 days against 160 sprint-days** | Deliberately not padded to look comfortable. [02-roadmap.md](02-roadmap.md) §7 states the realistic range is 18–21 sprints and gives an ordered cut list. A plan that fits by construction is a plan that has hidden something |

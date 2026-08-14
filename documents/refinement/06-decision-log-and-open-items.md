# 06 — Decision Log & Open Items

The place to look when something in this specification appears wrong, contradictory, or missing.

Three kinds of thing live here:

- **§2 — Conflicts (C1–C9):** places where two source documents disagree, and which one wins. All resolved. The resolution is already applied throughout the rest of this set; §2 records *why*, so nobody re-litigates it.
- **§3 — Open items (O2–O6):** genuinely undecided. **No default has been invented for any of them.**
- **§6 — Do not re-raise:** decisions that look like oversights and are not.

**Change requests are recorded in §5, by date.** The most recent are **CR-4 and CR-5 of 8 August 2026**, which add Rule-46 and amend Rule-12. CR-4 in particular **reverses the 3 August 2026 decision that a member earns nothing on their own Business Volume** — if you find a document still describing that as an absolute rule, or citing the pre-CR-4 golden totals (35/22/450/1,000/980), that document is stale and §5 is right. CR-1, CR-2 and CR-3 of 7 August 2026 (amending Rule-36, adding Rule-44 and Rule-45) remain in force alongside CR-4/CR-5.

---

## 1. The precedence rule

Restated from [00](00-master-index.md) §3 because every resolution below applies it:

> **`client-requirements-validation.md` and `user-needs-document.md` (tier 1) beat `03-business-rules.md` (tier 2), which beats `architecture.md` and the readiness technical docs (tier 3), which beat the approved prototype (tier 4), which beats `requirement-spec.md` and `open-questions-checklist.md` (tier 5, historical).**
>
> Within a tier, the **later dated** statement wins.

A conflict that precedence does not settle belongs in §3, not in code.

---

## 2. Conflicts resolved — C1 to C9

### C1 — Settings count: 13 or 16?

| | |
|---|---|
| **Disagreement** | `05-data-model-specification.md` §`settings` says "the 13 configurable items in Appendix B". `09-implementation-backlog.md` US-0.2's acceptance criteria says "13 default settings". `12-implementation-context.md` §6 says "13 configurable items". `architecture.md` Appendix B lists **16 rows**. |
| **Cause** | Rows 14–16 (whole-console backup schedule, retention count, folder) were added on 7 August 2026 for RQ-23. The three readiness documents were written on 6 August and only partially updated. |
| **Resolution** | **16 settings.** Appendix B is authoritative and is restated in full in [02](02-business-rules.md) §6. |
| **Authority** | `architecture.md` Appendix B, revised 7 Aug 2026 (tier 3, later date). Corroborated by `05-data-model-specification.md`'s own "Seed/reference data" line, which *does* say 16 — the document contradicts itself. |
| **Build consequence** | The first-run seed inserts **16** settings rows, not 13. |

### C2 — IPC command count: 32, 36, or 40?

| | |
|---|---|
| **Disagreement** | `12-implementation-context.md` §4 says "32-command IPC surface"; §7 of the same file says "36 Tauri IPC commands"; `08-testing-strategy.md` says "there are 36 commands"; `04-api-specification.md` enumerates **API-01 … API-40**. |
| **Cause** | Three additions in sequence: the original 26 in `architecture.md` Appendix C, minus `reverse_entry` (dropped), plus API-33 and API-34–36 on 6 August, plus API-37–40 on 7 August. Each document froze at a different point. |
| **Resolution** | **42 commands**, API-01 to API-42, with no gaps (amended 14 Aug 2026 — `get_ancestor_chain`, API-42 — see "14 August 2026" below). Full contracts in [04](04-technical-architecture.md) §6. |
| **Authority** | `04-api-specification.md` command-surface summary, 14 Aug 2026 (tier 3, later date, and the document that owns the surface). |
| **Build consequence** | The Tauri capability allowlist has 42 entries. The contract-test suite has 42 tests, not 36. |

### C3 — Unauthenticated commands: six or seven?

| | |
|---|---|
| **Disagreement** | `12-implementation-context.md` §7 and `08-testing-strategy.md` both name **six**: `login`, `setup_first_run`, `use_recovery_code`, `check_data_readable`, `list_restore_points`, `restore_from_backup`. `04-api-specification.md` and `06-security-authorization-matrix.md` §3 name **seven** — the same six plus `restore_from_backup_file` (API-40). |
| **Cause** | API-40 was added 7 August 2026 for RQ-23, and is unauthenticated of necessity: a brand-new install has nothing to authenticate against. |
| **Resolution** | **Seven**, and the set is closed: `login`, `setup_first_run`, `use_recovery_code`, `check_data_readable`, `list_restore_points`, `restore_from_backup`, `restore_from_backup_file`. |
| **Authority** | `06-security-authorization-matrix.md` §3 and `04-api-specification.md`, both 7 Aug 2026. |
| **Build consequence** | `08-testing-strategy.md` prescribes a contract test asserting the unauthenticated set *exactly*, so that an eighth cannot be added by accident. **That test must assert seven, not six** — written against the stale number it would fail on correct code. An eighth must not be added without revisiting both documents. |

### C4 — Member ID range: does it start at 100000 or 100001?

| | |
|---|---|
| **Disagreement** | `requirement-spec.md` Rule 35 says "100000–999999". `client-requirements-validation.md` §10.3 and Rule 35's own confirmation line say the usable range starts at **100001**. |
| **Resolution** | **100001–999999.** 100000 itself is never assigned. |
| **Authority** | `client-requirements-validation.md`, 4 Aug 2026 (tier 1). |
| **Build consequence** | ID allocation excludes 100000 from the candidate pool. `architecture.md`'s DDL and `05-data-model-specification.md` already reflect this. AC-11 tests it. |

### C5 — Does deactivating a member change any calculation? ⚠️ *highest-risk conflict in the project*

| | |
|---|---|
| **Disagreement** | `requirement-spec.md` Rule 28 (line 422) says an inactive member "stop[s] appearing in new periods". `client-requirements-validation.md` V3.5 / RQ-2 says inactive status has **no effect on any calculation at all** — the member's own Business Volume still counts fully toward their introducer's figure, and their downline still rolls up through them exactly as before. Deactivation is purely a display flag. |
| **Cause** | RQ-2 asked the question directly. The client's answer (4 Aug 2026) was **stricter than the architect's own recommendation**, which had proposed that an inactive member's own figure stop contributing. The validation document flags the wording gap against the spec explicitly. |
| **Resolution** | **`is_active` is a display-only flag with zero computational effect.** |
| **Authority** | `client-requirements-validation.md` V3.5 / RQ-2, 4 Aug 2026 (tier 1). |
| **Build consequence** | Module M3 must **not** filter, zero, or special-case inactive members anywhere in the calculation path. Implementing the original spec wording would silently corrupt every ancestor's Total Business Volume and Rewards the moment any member is deactivated. The display layer applies a distinct colour plus a labelled pill (never colour alone) in the chart, member lists, and every extract row. This has a dedicated regression test — see [05](05-quality-and-acceptance.md) §2. |

### C6 — Two questions marked ☐ open that were answered days earlier

| | |
|---|---|
| **Disagreement** | `requirement-spec.md` and `open-questions-checklist.md` both still carry ☐ open markers for (a) whether an elapsed month with no entries produces a zero snapshot, and (b) whether the credential is a PIN *or* a password. |
| **Cause** | The two draft documents were never updated after the answers landed. The checklist's own summary says all 22 of its questions are closed while leaving the empty-month question visibly unticked. |
| **Resolution** | **(a)** An empty month produces **no snapshot at all** and is excluded from the yearly-averaging denominator — RQ-16, 3 Aug 2026. **(b)** A PIN and a complex password may **both** be configured simultaneously; **either** authenticates — M8.5, 4 Aug 2026. |
| **Authority** | `client-requirements-validation.md` RQ-16 and M8.5 (tier 1). |
| **Build consequence** | Cite RQ-16 and M8.5, never the stale ☐ markers. The `auth` table carries `pin_hash` and `password_hash` as independently-nullable columns, with at least one required. |

### C7 — Business Volume entries per month: supplied or not?

| | |
|---|---|
| **Disagreement** | `11-open-questions-and-decisions.md` LOW-4 says "still not supplied by the client… deferred by decision, 6 August 2026". `12-implementation-context.md` §17 repeats it: "Only the Business Volume entries-per-month sizing figure remains unsupplied." `requirement-spec.md` §10 and `open-questions-checklist.md` Question 15 both leave the field blank. But `client-requirements-validation.md` §10.3 and §11.1 both record it as **confirmed on 4 August 2026: approximately 1,000 entries per month, explicitly variable**. `user-needs-document.md` UN-14 says the same. |
| **Cause** | The readiness analysis (6 Aug) missed the answer already sitting in the tier-1 document from two days earlier. |
| **Resolution** | **The figure is supplied: ~1,000 Business Volume entries per month, explicitly approximate and variable** — the client warned it could run well above or below. |
| **Authority** | `client-requirements-validation.md` §10.3 and §11.1, 4 Aug 2026 (tier 1). |
| **Build consequence** | Performance testing has a realistic data-volume figure to rehearse against and is no longer blocked on client input. **Note carefully:** NFR-1's targets (2 s / 2 s / 30 s) are fixed *regardless* of volume and were never tuned to 1,000 — this is a confirmation of scale, not a change to the design. Test at both the realistic scale (5,000 members / 1,000 entries a month) and the 25,000-member design ceiling. |

### C8 — Two compliance constraints marked "needs a client decision" that were decided

| | |
|---|---|
| **Disagreement** | `user-needs-document.md` §7.4 marks **CC-2** (no retention limit defined) and **CC-3** (no erasure route) as 🔴 "Needs a client decision". `06-security-authorization-matrix.md` §6 and `11-open-questions-and-decisions.md` HIGH-2 both record them as settled. |
| **Cause** | Raised on 3 August as open compliance questions; answered on 6 August via the architect on the client's stated requirement; §7.4's markers were never updated. |
| **Resolution** | **Permanent, complete retention is the explicit client requirement, not an unresolved gap.** Members are never removed from the application, and all data persists throughout — including in every export. Correction of a member's own record is supported (`edit_member`, fully audited). **There is no erasure path and none is to be built.** This is Rule-42. |
| **Authority** | Client requirement confirmed via the architect, 6 Aug 2026; recorded in `06-security-authorization-matrix.md` §6 and `03-business-rules.md` Rule-42. |
| **Build consequence** | No delete path in the schema, the API, or the UI. No "erasure requested" flag. No export filter that would omit a member. HIGH-2 is marked **"not to be re-raised in future analysis"** — see §6 below. |

### C9 — Is Total Business Volume a fifth mandatory export column?

| | |
|---|---|
| **Disagreement** | Rule-19 says every extract carries "the member's basic details, contact number, **volume** and Business Volume, regardless of which optional columns are chosen" — which reads as five things, with *volume* meaning Total Business Volume. AC-29 repeats the same five. But Rule-33, V6.1, and US-M6.1's acceptance criteria all named only **four** mandatory columns: name, ID, phone, Business Volume, with Total Business Volume in Rule-33's *optional* list. |
| **Cause** | Rule-33/V6.1 conflated *what starts ticked* (a default) with *what can never be unticked* (mandatory) — the approved prototype's default column set (`['name', 'id', 'phone', 'bv']`) settles the former, not the latter, but the two subsequent documents copied it as though it settled both. |
| **Resolution** | **Five mandatory columns: name, member number, phone, Business Volume, Total Business Volume.** Untickable on all three extracts. Resolved toward Rule-19's own wording, the client's own statement of what an extract must always carry. |
| **Authority** | Architect, 8 Aug 2026 (`PI/05-decisions-and-gaps.md` D-1) — resolved *toward* Rule-19's client-sourced wording, not client-confirmed directly. Worth a client conversation to confirm; if the client wants four, this reverses in under a day. |
| **Build consequence** | V6.1's "the four default columns are always present and cannot be removed" is corrected to five. Total Business Volume leaves the optional list entirely. The seeded default column set changes from four entries to five (US-M6.5, S13). |

---

## 3. Open items — O2 to O6

**Nothing below has a default invented for it.** Each states the question, why it matters, what the closest available evidence is, and who has to answer.

### O2 — Hierarchy depth setting has no stated default

| | |
|---|---|
| **The question** | `architecture.md` Appendix B row 4 gives the default as "—". `client-requirements-validation.md` §6.7 setting 5 says "Not specified". `requirement-spec.md` §7 row 4 says "not specified". No source states a number. |
| **Why it matters** | The first-run seed has to write *some* value. The setting is advisory only (Rule-1, Rule-32 — depth overflow warns, never blocks), so a wrong value cannot break anything; it only changes when the advisory warning fires. |
| **Closest evidence** | The approved prototype uses `structureDepthGuidance: 4`, consistent with level widths being specified for levels 2, 3 and 4 only. |
| **Who answers** | **The client**, or the architect as a build decision — it is advisory. |
| **Until answered** | Seed **4**, matching the approved prototype and the level-width coverage. Flag it on the settings screen as guidance the client should set to their own intended shape. |

### O3 — Session inactivity timeout has no stated default

| | |
|---|---|
| **The question** | Appendix B row 13 gives the default as "— (set at setup)". NFR-4 and `06-security-authorization-matrix.md` §3 both call it "configurable" without naming a number. |
| **Why it matters** | The setting is security-relevant — it bounds the window in which a stolen unlocked machine is exposed (a threat explicitly out of scope, defended only by this timer). The first-run seed needs a value, and "set at setup" implies the setup wizard should ask, which no screen currently does. |
| **Closest evidence** | The approved prototype uses `sessionTimeoutMinutes: 15` and exposes it on the Settings → Access section, not in the setup wizard. |
| **Who answers** | **The architect** as a build decision, unless the client wants the setup wizard to ask. |
| **Until answered** | Seed **15 minutes**, matching the approved prototype, editable on the Settings screen. Note that this contradicts Appendix B's "set at setup" wording — the prototype puts it in Settings, not the wizard. |

### O4 — The lockout ladder is undefined beyond its first step

| | |
|---|---|
| **The question** | Every source agrees lockout is mandatory and triggers at **5 consecutive failed attempts**. Beyond that they disagree and none is complete: `architecture.md` §11.2 and `06-security-authorization-matrix.md` §3 both say "exponential backoff"; the approved prototype implements a **flat 20-second lockout and resets the failure counter to zero afterward**, so a patient attacker gets unlimited batches of five. `07-error-edge-case-matrix.md` says "timed lockout with countdown (exponential backoff)". No source states the ladder — what the second, third and *n*th lockouts are, whether the counter resets on lockout expiry or only on a successful login, or whether there is a ceiling. |
| **Why it matters** | This is the only control standing between a 6-digit PIN — one million combinations — and an attacker with the machine. A flat 20 seconds with a counter reset is roughly 4 seconds per attempt sustained, which is not meaningful protection over hours. `08-testing-strategy.md` prescribes a test for "countdown timing; attempts do not reset early" that cannot be written against an undefined ladder. |
| **Who answers** | **The architect**, as a security design decision. The client's requirement ("lock the account after repeated failed attempts", V8.1) is satisfied by any reasonable ladder. |
| **Until answered** | Do **not** ship the prototype's flat-20-seconds-with-reset behaviour as the production ladder — it was demo pacing, not a security decision. Define the ladder before M8 is built. Whatever is chosen must satisfy: the counter does not reset merely because a lockout elapsed, and the ladder grows. |

### O5 — Advisory level widths are defined only for levels 2, 3 and 4

| | |
|---|---|
| **The question** | Settings carry `level_widths` for levels 2, 3 and 4 (defaults 9 / 6 / 3). Depth is separately configurable and can exceed 4. No source says what advisory width applies at level 5 and below. |
| **Why it matters** | Only to the advisory warning in `add_member` (Rule-1, V1.7) — the warning cannot fire, or must be suppressed, for a level with no configured width. Nothing is ever blocked either way, so the consequence is cosmetic. |
| **Who answers** | **The architect**, as a build decision, unless the client wants per-level widths for a deeper structure. |
| **Until answered** | Suppress the width warning for any level with no configured width; the depth warning (Rule-32) still fires independently. Do not silently reuse level 4's width for level 5+ — that would produce a warning the client never configured. |

### O6 — `login`'s lockout-transition audit and `use_recovery_code`'s audit are architecturally unrealizable as documented **[NEW — found in S14's US-M9.1 completeness pass]**

| | |
|---|---|
| **The question** | `04-technical-architecture.md` §6 documents `login`/`unlock_session` as audited "only on failed-lockout transitions" and `use_recovery_code` as audited with cause "credential recovery". Neither is built, and neither can be with the architecture as it stands: `audit_log` lives inside the SQLCipher database, and both these paths run — by design, per Rule-29's closed set of seven unauthenticated commands — with no key and no open connection. `setup_first_run` had the identical problem and S14 could close it (a connection opens moments later, once the credential is created); `login`'s lockout bookkeeping and `use_recovery_code`'s credential reset both live entirely in the unencrypted `auth.json` sidecar and never reach a point where a database connection exists. |
| **Why it matters** | NFR-5's "an audit log that can explain any figure" doesn't strictly need these — neither touches a member's figures — but a security-relevant event (repeated failed logins escalating a lockout; a recovery code consuming itself and replacing the credential) going unrecorded anywhere is a real gap for an admin trying to reconstruct what happened after the fact. Rule-43/S14 solved the structurally similar "restore" case with an unencrypted manifest, but that mechanism was justified because backup metadata was already deemed safe to reveal pre-auth (§8.6). Recording lockout/recovery *events* in a similar unencrypted file is a different risk profile — it starts to reveal security-relevant activity to anyone with filesystem access, before any credential has been proven. |
| **Who answers** | **The architect.** Either accept that these two paths are simply never audited (correct the API table to say so plainly, matching how `lock_session`/most reads already say "Not audited") or decide what a pre-auth-safe security event log would need to look like and whether that's worth building. |
| **Until answered** | Left exactly as S10 left it — neither path writes anything. The API table (§6) now states this explicitly as an open gap rather than a built behaviour, so a future reader doesn't assume it exists. |

---

## 4. Source-document contradictions — INC-1 to INC-5, all closed 3 August 2026

Five places where the two draft documents contradicted each other. All were put to the client and closed. Recorded here so a reader encountering the stale text in `requirement-spec.md` or `open-questions-checklist.md` knows it was found and settled, not missed.

| ID | The contradiction | Resolution |
|---|---|---|
| **INC-1** | The spec says a member's introducer is fixed forever with no override (Rule 37). The checklist's Question 16 confirmation box still reads "Moving is allowed, and already-closed months stay frozen", marked confirmed | ☑ **Transfers are prohibited.** Rule-37 stands; Question 16's box is superseded |
| **INC-2** | Both documents carry a rule blocking any move that would place a member beneath their own team — but moves are now prohibited entirely, so no such move can be attempted | ☑ **The structure is sound by design** because positions never change. The loop check is retained as a belt-and-braces safeguard that can never fire in normal use. Keep it; do not remove provably-unreachable defensive code |
| **INC-3** | Both documents still describe figures recorded between the 1st and the close counting into the month being closed. Rule-36's entry lock makes this window unreachable | ☑ **No late-recording window exists.** Rule-21's third bullet is struck through and retained for the record only — do not implement it |
| **INC-4** | The spec's Q-I7 answer and its change-log entry both still record "moves permitted with closed months frozen" with no supersession marker | 🟢 **No client decision needed** — Rule-37 already settled it; only the paper trail lagged. The supersession marker has been applied |
| **INC-5** | The spec's export column list includes *active/inactive status*; the list the client actually read and ticked in the checklist does not | ☑ **Include active/inactive status.** The full list is in Rule-33 |

---

## 5. Decision history

Chronological record of what was decided, when, and where it differed from the recommendation. A ⚠️ marks a reversal of a previously agreed position; a 🔷 marks a client answer that differed from the architect's recommendation.

### 3 August 2026 — the 22 original questions closed

| Decision | Note |
|---|---|
| Differential applies to the child's **Total Business Volume** | Only Scenario 3 disambiguates it |
| A member earns **nothing** on their own Business Volume | Confirms all five scenarios' behaviour as deliberate |
| Own Business Volume is **always** included in a member's own Total Business Volume | Scenarios 4 and 5 omitted it as a write-up simplification, not a different rule |
| 🔷 The reset zeroes **everything** — Business Volume, Total Business Volume, Rewards, royalty | Differs from the recommendation, which proposed keeping Rewards live. Makes the backup gate load-bearing — and, as it stood on 3 Aug 2026, the total entry lock too. That lock was narrowed on 7 Aug 2026 (CR-2); the backup gate is untouched and still load-bearing |
| A period is a **calendar month**; the close closes whichever month it belongs to | Plus a client-added requirement: a persistent, undismissable alert |
| Yearly average divides by the count of months that **have a snapshot**, with that count displayed | Protects late joiners |
| 🔷 The low-contribution report filters on the yearly average of **own** Business Volume | Differs from the recommendation of Total Business Volume |
| 🔷 Two decimal places throughout, rounded only at display. **Rupee entry removed entirely** | ⚠️ Reverses the original "two entry modes" decision locked at the start of the engagement |
| The **Business Volume** family of terms is used throughout; "purchase volume" dropped | *Purchase* is on the client's excluded list |
| 🔷 The chart node shows **name, ID and own Business Volume** — nothing else | Differs from the recommendation of Total Business Volume. Consequence accepted: the chart alone cannot explain a member's slab |
| The royalty rate is **configurable**, like everything else | |
| Royalty **stacks at every qualifying level** | Re-confirmed with the payout consequence understood — the same volume can attract royalty twice in one chain |
| Slab rows can be **added and removed**; the top slab is always the highest-percentage row | |
| Recalculation is **immediate** on every entry; no recalculate control | Network size 500–5,000 |
| Edit freely; deactivate rather than delete | ⚠️ The "moving allowed" half was reversed the same day by Rule-37 |
| "Final discounts" means **final Rewards** | No missing feature. Rupee conversion stays manual, outside the software |
| Reference must resolve to an existing active member; the root is created once at setup; loop-creating moves blocked | |
| **One administrator login only**; members never log in | Failed-attempt lockout mandatory |
| All export fields offered, four defaults pre-ticked | |
| Backups downloaded **and** retained permanently in the system | |
| Depth overflow **warns but allows** | Consistent with level widths being advisory |
| ⚠️ **Phone numbers unique** across active and inactive; a match on an inactive member offers reactivation | New client requirement |
| ⚠️ **Member IDs randomly allocated**, never sequential, never released | New client requirement |
| ⚠️ **The reset is enforced** — all entry locks once a month ends | Reverses the earlier decision to keep the alert non-blocking, which was explicitly offered and rejected at the time |
| ⚠️ **Transfers prohibited outright** | Reverses Rule-28's move provision. Makes the hierarchy a tree by construction |
| 🔶 **No slab-table monotonicity validation** (RQ-1) | The recommended safeguard was explicitly declined. Accepted risk R-2 |
| 🔷 **Neither zero nor negative** Business Volume is accepted (RQ-17) | Stricter than the recommendation, which proposed accepting zero |
| A settings change applies immediately to the open month, behind a pre-save warning (RQ-18) | Closed months never affected |
| The **internal retained copy** is the backup gate; the download is a convenience (RQ-6) | A download cannot be reliably observed; the gate must be verifiable |
| The backup carries the permanent record's fields **plus the slab table, royalty rate and qualifying count in force that month** (RQ-5) | Otherwise a past month cannot be re-derived from it |
| A monthly extract for a closed month reads from the **permanent record** (RQ-4) | Otherwise it would return zeros |
| Recovery codes issued at setup are the recovery path (RQ-10) | |
| An audit log will be built (RQ-9) | Closes R-3 |
| The recording lock is a **hard stop, no grace period** (RQ-11) | Consequence understood: if the client is away over a month end, nothing can be recorded |
| The reference unit value is labelled as the value of one **Reward** (RQ-12) | |
| Reward detail = one line per direct child, then royalty lines, then the total (RQ-13) | |
| Past months on screen are **out of scope**; extracts only (RQ-14) | Deferred, not refused |
| Joining date captured automatically, editable afterward (RQ-15) | |
| An **empty month produces no record** and is excluded from the average (RQ-16) | |
| Retention permanent; the client takes their own advice on notification (RQ-8) | |
| NFR targets confirmed: 2 s screens, 2 s recalculation, 30 s extracts; 25,000-member design ceiling; offline desktop only; no migration | |
| 🔶 **Monitoring declined** (NFR-12) | Nothing will detect a close silently failing to write its record |

### 4 August 2026 — validation-document decisions

| Decision | Note |
|---|---|
| 🔷 ⚠️ **Inactive status has zero calculation effect** (RQ-2) | Stricter than the recommendation. **The highest-risk item in the build** — see C5 |
| The **root member cannot be deactivated** (RQ-3) | |
| 🔷 ⚠️ **An entry is editable at any time, including in an already-closed month** (RQ-7) | Broader than the recommendation. Reverses the "permanent once closed" framing, which the validation document states was the architect's gloss on Rule-38, not the client's requirement |
| **The original backup for a corrected month is never touched**; a new dated version is created and retained alongside it (RQ-20) | Sharper than the recommendation, which proposed only a log entry |
| An entry's date is editable **only within its own month** (RQ-21, option a) | Moving an entry between months is deferred as an explicit future action |
| The downloaded backup copy goes to a **physically separate medium** (RQ-19) | Closes R-12 |
| **Consent captured in the system** — mandatory checkbox plus auto-captured date (RQ-22) | Closes R-14; DPDP Act 2023 |
| **PIN and password may both be set**; either authenticates (M8.5) | Strengthens R-5 |
| Member ID range starts at **100001** | See C4 |
| Business Volume entries per month: **~1,000, explicitly variable** | See C7 — the readiness documents missed this |
| Inactive members shown in a **distinct colour** in the chart, lists and every extract row (M4.5, M6.5) | Informational only |
| BA-1, BA-3, BA-4, BA-7, BA-10, BA-11 confirmed (blanket confirmation) | |

### 6 August 2026 — readiness-analysis decisions

| Decision | Note |
|---|---|
| **HIGH-1 closed:** the prototype's "Closed month snapshot" export card maps to `redownload_backup` (API-20) | No new backend command |
| **HIGH-2 closed — not an issue:** members are never removed and all data persists, including in exports | Became **Rule-42**. Marked *not to be re-raised* |
| **MEDIUM-1 built:** the settings mid-period recalculation warning, variant C | Names the open month, states closed months are unaffected, shows Rewards before → after, lists affected members. Fires on slab-table and royalty saves only |
| **LOW-2 built:** the last slab row cannot be removed | Control disabled with an explanatory `aria-label`; the handler also refuses if reached another way |
| **LOW-3 built:** a full-screen data-recovery state when the database cannot be opened at launch, design D | Not in any source document — a genuine gap the analysis found and the architect approved filling |
| **`reverse_entry` dropped** | No requirement described a reversal functionally distinct from an edit; the prototype implements only editing. `edit_entry` is the complete mechanism |
| **`preview_settings_impact` (API-33) added** | The settings warning needs a Rust-side dry run; the frontend cannot compute it |
| **API-34–36 added** — the three pre-flight/recovery commands | Unauthenticated of necessity |

### 7 August 2026 — RQ-23, whole-console backup and cross-device restore

| Decision | Note |
|---|---|
| The backup is a **verified copy of the whole encrypted database file**, credentials included | A restored machine needs no re-setup |
| The schedule (off/daily/weekly/monthly) is checked **once, at successful login** | There is no background service while the app is closed. A missed day catches up at the next login. **This is a design constraint, not a gap to "fix" with a background timer** |
| Retention: keep the most recent backups, default **10**, client-adjustable; oldest pruned | `period_close` and `pre_restore_safety` rows are never pruned by this |
| Restore is reachable from two places: a **plain link** on the ordinary first-run setup screen, and a **Restore card** in Settings | No separate welcome/choice screen. The first-run link leads to the same recovery screen the db-error path uses, reworded rather than duplicated |
| Restore confirmation is a **checklist modal** — the same weight already given to closing a month | Not heavier, not lighter |
| The console takes a **`pre_restore_safety` backup of its own current state** immediately before any restore | A restore is never a true one-way door |
| Any authenticated session is **dropped immediately after any restore** | The restored file may carry a different credential |
| `backups` table **generalized**, not duplicated (ADR-012) | `period_id` nullable, plus `kind` and `schedule_kind` columns |
| **API-37–40 added** | `restore_from_backup_file` joins the unauthenticated set — see C3 |

**Four small prototype fixes, made alongside the above (not requirement changes, carry forward as build notes when porting the UI):** Escape now closes a dismissable modal (add/edit member modals still ignore it, by design — see [07](07-design-system.md) §6.6); `role="dialog"`/`aria-modal`/`aria-labelledby` added to the modal primitive; toast icons given an explicit size rule (`.toast svg`) after rendering at the SVG default; a `hashchange` listener added so the recovery-screen trigger fires correctly (a hash appended to an already-open page is a same-document navigation, so `init()` would otherwise never re-run). Source: `11-open-questions-and-decisions.md`, "Improvements made while building the above."

### 7 August 2026 — CR-1, CR-2, CR-3: three client change requests

Raised by the client after reviewing this approved set. All three change behaviour that was already frozen, so each is recorded here in full: what was asked, what was decided, and what it reverses.

#### CR-1 — Phone number as a search key

| | |
|---|---|
| **Requested** | The home page should let the client search by phone number as well as by member ID and name, "since phone number is unique to member so it is easy to search member by mobile number" |
| **Decided** | Every search box in the console accepts a phone number. Both sides are reduced to a canonical key (non-digits stripped, then a country prefix or trunk zero dropped) so a number is found however it was written, and the phone clause engages only from **four digits** upward so short queries do not sweep in unrelated members. Search results gain a **phone column** |
| **Rule** | **Rule-44** (new). Also FR-1, M2.1, M4.6, V4.4, AC-40, AC-41, UN-29 |
| **Notes** | Applied to *all* search boxes rather than only Home, because one shared search function serves them all — differing behaviour between screens would be a defect, not a feature. Phone is personal data under the DPDP Act 2023; it now appears on the landing screen, visible only to the single administrator role that already sees it on Member Detail and in exports. Recorded in [04](04-technical-architecture.md) §8.7 |
| **Rejected alternative** | Home-only phone search, and "match on phone but do not display it". The client chose full scope and display |

#### CR-2 — Grace period for entry into an unclosed month

| | |
|---|---|
| **Requested** | Remove the hard lock on Business Volume entry when the previous month is not closed. The client's rationale: a member who purchases on the last day of a month often reports it two or three days later, and under the frozen rule that figure could not be recorded at all. The client's stated condition, verbatim in substance: *while the previous month is unclosed I can add entries for the previous month, but I cannot add current-month entries; to add current-month entries the previous month must be closed* |
| **Decided** | Rule-36 is **narrowed, not removed**. A month that has ended but is not closed continues to accept entries, indefinitely, for as long as it stays unclosed. The current month accepts entries only when no earlier month is outstanding. Closed months remain reachable only through the correction path (Rule-39) |
| **Reverses** | **RQ-11's answer of 3 Aug 2026** ("hard stop kept, no grace period"), and with it OC-2, OC-6's severity, R-4's position, AC-19, V2.3 and V2.5 |
| **Rule** | **Rule-36 amended.** Also M2.3/M2.6/M2.7, M5.2, V2.3/V2.5/V2.6/V2.7, AC-19/AC-42/AC-43, UN-30 |
| **Grace has no clock** | No day limit, no configurable grace window, no countdown, no seventeenth setting. The grace lasts exactly as long as the month stays unclosed. The client considered and declined a configurable N-day window |
| **Why it is safe** | Live figures belong to a period. An entry dated in the ended-but-unclosed month is already an entry against **that** period's figures. Only a current-month entry would mix into a period not yet snapshotted and zeroed — and that is what stays blocked |
| **Schema consequence** | `periods.status` value `ended_locked` renamed **`awaiting_close`**; `member_period_totals` widened from "the current open period only" to "any not-yet-closed period". Both are documentation-only — no implementation exists yet |
| **Multiple outstanding months** | Any outstanding month accepts entries, not merely the oldest. The client noted this situation will not arise in practice and chose the permissive behaviour for the hypothetical case. Figure-showing screens display the **oldest** outstanding month, and a month switcher is rendered **only** when more than one month is outstanding — so nothing new appears on screen in the ordinary case |

#### CR-3 — "View full hierarchy" in a separate window

| | |
|---|---|
| **Requested** | A "View Full Hierarchy" button on the Structure screen that opens a new window with the full hierarchy expanded, with the explicit constraint that *"our original software should not be affected by performance — it just opens new window with expanded full hierarchy with all data and forgets"* |
| **Decided** | A separate, read-only window rooted always at the **top member**, drawing the whole structure once and never updating, carrying an "as at" timestamp. Zoom (down to 10%), fit-width, in-window search-and-highlight, and print. Gated above **60 descendants** by a confirmation naming the exact member count |
| **Rule** | **Rule-45** (new). Also FR-10, M4.7, V4.5, AC-44, AC-45, UN-31, §5.3a |
| **Root choice** | Always the top member, not the currently-viewed member — the client's explicit choice. A full view is a view of the network, not of a branch |
| **Layout choice** | **Top-down chart, fully expanded**, chosen by the client over a width-stable indented outline after being shown that a top-down chart's width grows with leaf count and becomes tens of thousands of pixels wide at the 25,000-member ceiling. Recorded as **TR-7**, accepted, with the 10% zoom floor, fit-width, in-window search and the size gate as the agreed mitigations. The indented outline is the named fallback, listed under deferred scope |
| **No new API** | `get_direct_children_chart` (API-11) already carried a `full_tree` parameter in the readiness spec but had lost it in [04](04-technical-architecture.md); the parameter is restored and put to work. The count for the gate and the draw itself are both cheap local reads — the cost of this feature is rendering, not fetching, which is exactly why the rendering happens somewhere else |
| **Closes a standing gap** | The ">60 descendants confirm-before-render" gate had been specified in [03](03-functional-specification.md) §5.3 and [07](07-design-system.md) §8 but never built, and was tracked as untraced prototype behaviour. It now has a rule, a validation row and a home |

**Prototype drift found and corrected during this work (build notes, not requirement changes):** the ">60 descendants" gate was documented but absent from `ui-prototype-v2.html`; the Structure screen's zoom, fit-width, collapse-all and search controls existed in the prototype but were documented nowhere; V2.5 stated the entry screen carries no date field while the prototype has always shown one. All three are now resolved in favour of the built behaviour, with V2.5 amended.

### 8 August 2026 — CR-4, CR-5: two client change requests, raised the day before implementation starts

Raised by the client just before implementation begins. CR-4 changes a calculation rule already frozen since 3 August 2026; CR-5 adds a new screen element. Both are recorded here in full, same format as CR-1/2/3.

#### CR-4 — Reward on own Business Volume

| | |
|---|---|
| **Requested** | A member's own Business Volume should also earn a reward, at the member's own slab — worked example supplied: A with children B, C, D each holding 100 Business Volume (2% slab), A's own Business Volume also 100. A's Total Business Volume becomes 400 (4% slab). Differential from B/C/D = 2 + 2 + 2 = 6 (unchanged formula). A's own 100 Business Volume, at A's own 4% slab, adds a further 4. Total Rewards for A = **10** |
| **Decided** | A third, additive term: `OwnReward(x) = slab%(x) × BusinessVolume(x)` (own Business Volume only, own slab from Rule-7, unchanged). `Rewards(x) = Differential(x) + Royalty(x) + OwnReward(x)`. Differential (Rule-8) and Royalty (Rule-10) are **not redefined** — same formulas, same exclusions, same golden trees |
| **Reverses** | The 3 August 2026 decision that **"a member earns nothing on their own Business Volume"** (§5 above). That decision is superseded specifically by this addition — it no longer holds as an absolute statement, only as a description of the differential term in isolation |
| **Rule** | **Rule-46** (new). Rule-12 amended. Also M3.5 (new), V4.3 (amended), AC-46 |
| **Golden regression set** | Scenarios 1–3's totals move (35→65, 22→62, 450→510 — each gains the top member's own-slab reward on their own Business Volume); Scenarios 4–5 are unchanged (1,000, 980) because the top member's own Business Volume is 0 in both, a pre-existing write-up simplification, not evidence the rule doesn't apply there. The client's own worked example above is added as **Scenario 6** (total 10) specifically because it is the only one of the six with a nonzero own-BV reward on a mid-tree member, giving Rule-46 real regression coverage |
| **Reward-detail screen** | The own-Business-Volume reward line is shown **first**, before the per-leg differential rows, then royalty, then the total — "your own contribution, then your team's" |
| **Notes** | No change to Rule-6 (TBV rollup), Rule-7 (slab lookup), Rule-9 (differential non-negativity), Rule-10/11 (royalty), or Rule-13 (Rewards stay a separate ledger, never feed back into Business Volume) |

#### CR-5 — "Rewards by slab" chart on Home

| | |
|---|---|
| **Requested** | Alongside the existing "Members by slab" chart on Home, a second chart showing total accumulated Rewards per slab — "how simplified format we can showcase this data... user friendly and easy to understand in one glance" |
| **Decided** | Reuse the members-by-slab card's exact visual pattern (horizontal bar per slab row) rather than a new chart type — the client already reads that shape. Placed directly below the existing card. Each bar's value is the sum of Rewards (all three components) across every member currently on that slab; the label reads `<slab's Rewards total> / <Rewards total across all members>`, mirroring the existing `<count> / <total members>` label exactly. Current live period only, computed the same way the existing chart already is |
| **Rule** | No new rule number — a display aggregation of already-defined figures, same as the members-by-slab chart itself has none. FR-1 (extended), V4.6 (new), AC-47 |
| **Rejected alternative** | A single combined chart showing both member count and Rewards total per bar. Two simple, familiar charts read faster at a glance than one denser dual-value one — the client asked for "easy to understand in one glance," and reuse of an already-understood pattern serves that directly |
| **No new API/data model change** | Computed from the same in-memory period figures the existing chart already reads; no dedicated backend command exists for either chart today, and none is being added for this one |

### 13 August 2026 — Volume Entry period table and summary nodes

| | |
|---|---|
| **Requested** | Volume Entry's entry list should show the real period's entries (every member, not just the current session), sorted by recorded date descending, paginated (10/25/50, default 10), titled `<Month> <Year> entries`. Two summary nodes above the lock-status banner: total entries recorded in that month, and entries recorded today |
| **Which month** | Reuses the existing recording-month rule (T-M2.3-2/T-M2.3-4, CR-2) rather than a new one: the outstanding month while one exists, otherwise the current month |
| **Decided** | **API-41 `list_period_entries`** added — the closed 40-command surface (C2) becomes 41. Returns the full month's entry list (member name blended in server-side); both summary nodes and the table's pagination derive from this one fetch client-side, no separate aggregate commands |
| **Entry-count definition** | Raw entry-record count, not distinct members — a member with two entries in the month counts as two |
| **Rule** | No new business rule — a read-only listing of already-recorded data, same status as `get_audit_log` (API-32) |

### 14 August 2026 — Back-navigation breadcrumbs (Structure / Member Detail / Volume Entry)

| | |
|---|---|
| **Requested** | Structure, Member Detail, and Volume Entry need the back-link/breadcrumb navigation the client-approved prototype already ships: a dynamic "back to whatever screen you came from" link on all three, plus Structure's root-to-current ancestor trail and Member Detail's fixed Home crumb |
| **Which command** | Structure's ancestor trail needs a root-to-member path that no existing command returns (`get_direct_children_chart` only walks downward) |
| **Decided** | **API-42 `get_ancestor_chain`** added — the closed 41-command surface (C2) becomes 42. Returns the ancestor path root-first, the requested member last; the back-link labels themselves are computed client-side from navigation history, no backend involvement |
| **Rule** | No new business rule — a read-only structural lookup, same status as `get_direct_children_chart` (API-11) |

---

## 6. Do not re-raise

Each of these looks like an oversight on first encounter and is not. Each was considered, put to the client, and deliberately decided. Re-raising them costs the client's time and, in two cases, would deliver scope they explicitly turned down.

| Item | Why it is not a gap |
|---|---|
| **Slab-table monotonicity is not validated** | The safeguard was recommended and **explicitly declined** (RQ-1, 3 Aug 2026; ADR-009; Rule-41). If you encounter what looks like a negative-differential bug, check the slab table configuration first — it may be this accepted risk manifesting, not a code defect. The Settings screen carries an on-screen disclaimer instead of a code guard |
| **No data-subject erasure route** | Permanent, complete retention is the client's **stated requirement**, not an oversight (Rule-42, C8). HIGH-2 is explicitly marked "not to be re-raised in future analysis" |
| **No monitoring of a silently-failed close** | Declined by the client (NFR-12). Do not build it, and do not write a test for it |
| **No member login or member-facing screen** | Permanently out of scope (OS-1), not deferred. ADR-001's single-process design has no socket to attach one to without a genuine architecture change |
| **No currency figure anywhere** | Out of scope (OS-2, OS-3). The reference unit value setting exists but is read by nothing except the settings screen itself |
| **No auto-update mechanism** | It would require exactly the network capability the offline requirement forbids (ADR-011). Upgrades are a new installer, run manually |
| **The loop-prevention check can never fire** | Correct, and it stays anyway (INC-2). Do not remove provably-unreachable defensive code |
| **The `reversal` cause value in `audit_log` is unused** | `reverse_entry` was dropped; `edit`/`correction` cover every case. Retire the enum value unless the client specifically wants that word preserved in the log |
| **The external-medium backup copy is not enforced** | The internal retained copy is the real gate; the external copy is prompted and reminded, never blocking (RQ-6, RQ-19). Forcing the close to block on it would contradict a documented design decision. The residual single-medium risk is stated plainly (TR-4), not solved |
| **No concurrency control** | Single-user, single-machine, single-session by client confirmation (BA-3, OC-1). SQLite's own file locking is sufficient. Do not add an optimistic/pessimistic locking scheme |
| **Loss of credential *and* recovery codes is unrecoverable** | The direct, accepted cost of "nobody but the client can ever get in" with no vendor backdoor (ADR-008). Must be communicated plainly at setup, not buried in settings |
| **The entry grace period has no time limit** | Deliberate (CR-2, 7 Aug 2026). A configurable "grace days" setting was offered and declined — the grace lasts exactly as long as the month stays unclosed. Do not add a countdown, a cutoff, or a seventeenth setting |
| **The full hierarchy window is rooted at the top member, not the current one** | The client's explicit choice (CR-3). Do not "improve" it by rooting it where the user happens to be — a second action for that was offered and declined |
| **The full hierarchy chart gets very wide on large networks** | Known, measured and accepted (TR-7). The width-stable indented outline was offered and the client chose the top-down chart. Do not silently switch layouts; if it becomes unusable in practice, raise it as a change, not as a bug fix |
| **The full hierarchy window does not update while open** | The defining property of Rule-45, not an oversight. It is a point-in-time draw carrying its own timestamp. Do not add live refresh — it would reintroduce exactly the main-console cost the separate window exists to avoid |

---

## 7. Readiness position

| | |
|---|---|
| **Blocking issues** | **None.** No item, at any point in this analysis, met the bar of "implementation should not begin until resolved" |
| **Conflicts** | 9, all resolved by precedence and applied throughout this set (C9 added 8 Aug 2026 — see `PI/05-decisions-and-gaps.md` D-1) |
| **Open items** | 5 — O2–O5 are build decisions to be taken deliberately rather than by default; O6 (added S14) is an architectural limitation found by US-M9.1's completeness audit, not a build decision — `login`'s lockout-transition audit and `use_recovery_code`'s audit cannot be built as documented without a new security tradeoff |
| **Modules gated** | None. No open item blocks any module from starting |
| **Prototype behaviours, approved reference** | 5 — the settings recalculation warning, the last-slab-row refusal, the data-recovery screen, the whole-console backup schedule/retention, and the restore flows. All now ported — S14 closed the last of them (the data-recovery screen and the restore flows, M8.6) |

C9's mandatory-column resolution is architect-resolved, not client-confirmed — worth raising in the next client conversation, but it does not block M6, which is already built against it.

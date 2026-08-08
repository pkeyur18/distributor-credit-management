# Master Specification — Index

## Distributor Business Volume & Beneficiary Management System

| | |
|---|---|
| **Client** | Siddharth Patel |
| **Solution Architect / Developer** | Keyur Patel |
| **Status** | **Build reference — supersedes all source documents** |
| **Compiled** | 7 August 2026 |
| **Baseline** | All requirements client-confirmed. Last decisions: **CR-1, CR-2, CR-3, 7 August 2026** (phone search; entry into an unclosed month; full hierarchy window) — see [06](06-decision-log-and-open-items.md) §5. |
| **Repository state** | Documentation-only. Zero application code exists. |

---

## 1. What this document set is

Requirements for this system were settled across nineteen documents written over five days, in which **later documents silently correct earlier ones**. `requirement-spec.md` Rule 28 says an inactive member stops contributing to calculations; `client-requirements-validation.md` V3.5 says the opposite and is the client's actual decision. Building from the wrong file would corrupt every ancestor's figures in the tree.

This set resolves that. It is **the single build reference**. Every rule, contract, column, screen, error case and acceptance criterion is restated here in its corrected, current form. **No source document needs to be opened during implementation.**

Where a source document is cited, it is cited as *provenance* — evidence of who decided what and when — never as a place you have to go to find the actual content.

---

## 2. The document set

| File | Contains | Read when |
|---|---|---|
| **[00-master-index.md](00-master-index.md)** *(this file)* | Reading order, ID namespaces, source precedence, change control | First, once |
| **[01-product-and-scope.md](01-product-and-scope.md)** | Purpose, users, glossary, scope boundary, vocabulary constraint, all 16 NFRs, constraints, risks | Before anything else — it defines the words everything else uses |
| **[02-business-rules.md](02-business-rules.md)** | All 47 business rules in corrected form, the calculation model, six worked scenarios, the 16-row settings inventory | Building M1, M2, M3, M5, M6, M7 — the heart of the system |
| **[03-functional-specification.md](03-functional-specification.md)** | FR-1–10, UN-01–31, RQ-1–23 coverage; screen-by-screen specification of every view, modal, window and flow | Building any UI |
| **[04-technical-architecture.md](04-technical-architecture.md)** | ADR-001–012, modules M1–M9, full DDL for 10 entities, all 40 IPC contracts, state machines, security, backup/restore | Building any backend |
| **[05-quality-and-acceptance.md](05-quality-and-acceptance.md)** | 63 error/edge cases, test strategy, golden scenarios, AC-1–AC-47, SC-1–SC-8, Definition of Done | Before claiming any story done |
| **[06-decision-log-and-open-items.md](06-decision-log-and-open-items.md)** | Every resolved conflict with its authority, the open-items register, superseded-decision history, the CR-1–5 change requests | When something looks wrong, contradictory, or missing |
| **[07-design-system.md](07-design-system.md)** | Colour tokens (light/dark), typography scale, layout, elevation, shape rules, and every component's exact spec — buttons, pills, modals, the Structure Tree Node, Full Hierarchy Window, Impact Summary, Restore Option List | Building any screen, alongside 03 |
| **[delivery-plan.md](delivery-plan.md)** | 36 user stories, dependency graph, proposed PI and sprint breakdown | Planning and sequencing the build |

---

## 3. Source precedence — the rule that makes this set trustworthy

When two source documents disagree, the higher tier wins. This ordering was applied to produce every statement in this set.

| Tier | Document | Dates | Authority |
|---|---|---|---|
| **1** | `documents/business/client-requirements-validation.md` | 3–7 Aug 2026 | **Highest.** The client's own confirmations, each dated. Overrides everything below it. |
| **1** | `documents/business/user-needs-document.md` | 3–7 Aug 2026 | Same tier — the client-facing statement of need, kept in step with the validation document. |
| **2** | `documents/implementation-readiness/03-business-rules.md` | 6–7 Aug 2026 | Corrected rule text. Already applies tier 1 over tier 4. |
| **3** | `04-technical-architecture.md`, `implementation-readiness/04`, `05`, `06` | 6–8 Aug 2026 | Technical contracts — schema, commands, security. (`documents/design/architecture.md` is an earlier draft superseded by `04-technical-architecture.md` — not used for implementation.) |
| **4** | `documents/design/ui-prototype-v2.html`, `ui-theme.md` | 6–7 Aug 2026 | Client-signed UI behaviour of record. Authoritative for anything visual or interactional the written documents leave unstated. |
| **5** | `documents/draft/requirement-spec.md`, `open-questions-checklist.md` | 3 Aug 2026 | **Historical.** Accurate where nothing later contradicts them; stale in six known places (see [06](06-decision-log-and-open-items.md) §2). |
| — | `documents/draft/requirement-draft.md` | — | The client's original notes. Deliberately untouched, cited for line references only. Uses retired vocabulary. |

**Two documents are themselves stale despite being recent:** `implementation-readiness/11-open-questions-and-decisions.md` and `12-implementation-context.md` carry figures that were correct on 6 August and were overtaken on 7 August, plus one item they record as unanswered which the client had already answered on 4 August. Corrections are in [06](06-decision-log-and-open-items.md) §2 (C1, C2, C3, C7).

---

## 4. ID namespaces — the complete map

Identifier schemes in use. **One namespace was added on 7 August 2026 — `CR-N`, the client change requests.** Every other ID below appears in full somewhere in these documents.

| Prefix | Range | Count | What it identifies | Defined in |
|---|---|---|---|---|
| `Rule-N` | Rule-1 … Rule-46, plus **Rule-16a** | **47** | A business rule | [02](02-business-rules.md) §3 |
| `FR-N` | FR-1 … FR-10 | 10 | A functional requirement area | [03](03-functional-specification.md) §2 |
| `UN-NN` | UN-01 … UN-31 | 31 | A user need | [03](03-functional-specification.md) §3 |
| `RQ-N` | RQ-1 … RQ-23 | 23 | A client-answered validation question | [03](03-functional-specification.md) §4 |
| `CR-N` | CR-1 … CR-5 | 5 | A client change request raised after this set was approved | [06](06-decision-log-and-open-items.md) §5 |
| `NFR-N` | NFR-1 … NFR-16 | 16 | A non-functional requirement | [01](01-product-and-scope.md) §7 |
| `ADR-NNN` | ADR-001 … ADR-012 | 12 | An architecture decision | [04](04-technical-architecture.md) §2 |
| `API-NN` | API-01 … API-40 | **40** | A Tauri IPC command | [04](04-technical-architecture.md) §6 |
| `M-N` | M1 … M9 | 9 | An application module | [04](04-technical-architecture.md) §3 |
| `M-N.N` | M1.1 … M8.7 | 57 | A module function | [03](03-functional-specification.md) §1 |
| `V-N.N` | V1.1 … V8.5 | 50 | A validation rule | [03](03-functional-specification.md) §6 |
| `AC-NN` | AC-1 … AC-47 | 47 | An acceptance criterion | [05](05-quality-and-acceptance.md) §4 |
| `SC-N` | SC-1 … SC-8 | 8 | A success criterion | [05](05-quality-and-acceptance.md) §5 |
| `US-*` | US-0.1 … US-M9.1 | **36** | A user story | [delivery-plan.md](delivery-plan.md) §3 |
| `OS-NN` | OS-1 … OS-15 | 15 | An out-of-scope item | [01](01-product-and-scope.md) §5 |
| `R-N` | R-1 … R-14 | 14 | A business risk | [01](01-product-and-scope.md) §10 |
| `TR-N` | TR-1 … TR-7 | 7 | A technical risk | [04](04-technical-architecture.md) §11 |
| `BA-N` | BA-1 … BA-11 | 11 | A business assumption (all resolved) | [01](01-product-and-scope.md) §9 |
| `INC-N` | INC-1 … INC-5 | 5 | A source-document contradiction (all closed) | [06](06-decision-log-and-open-items.md) §4 |
| `C-N` | C1 … C8 | 8 | A conflict resolved by this set | [06](06-decision-log-and-open-items.md) §2 |
| `O-N` | O1 … O5 | 5 | An item genuinely still open | [06](06-decision-log-and-open-items.md) §3 |

`Rule-16a` is not a typo. It was inserted between Rule-16 and Rule-17 rather than renumbering 17–43, because those numbers are referenced across every other document.

**Everything added on 7 August 2026 was appended, never renumbered** — Rule-44/45, FR-10, UN-29/30/31, M2.6/M2.7/M4.6/M4.7, V2.6/V2.7/V4.4/V4.5, AC-40–45, TR-7, US-M2.3/M2.4/M2.5/M4.3. **Rule-36, V2.3, V2.5, AC-19, OC-2, OC-6, R-4, US-M1.4, US-M2.1 and US-M5.3 were amended in place**, each carrying its superseded wording so the reversal is visible rather than silently overwritten. **No API command was added** — API-06, API-07, API-08 and API-11 were amended.

**8 August 2026 (CR-4, CR-5) — appended:** Rule-46, M3.5 (new numbering, M3.6/M3.7 shifted), V4.6, AC-46/47. **Rule-12 amended in place** — Rewards gains a third additive term; Differential (Rule-8) and Royalty (Rule-10) are untouched. **No API command was added.**

---

## 5. The six numbers that must never move

Every calculation change in this project is validated against the client's own worked examples — five original, plus a sixth added 8 August 2026 for Rule-46 (CR-4). These are the golden regression values.

| Scenario | Differential | Royalty | OwnReward | **Total Rewards** |
|---|---|---|---|---|
| 1 — basic differential | 35 | 0 | 30 | **65** |
| 2 — differential collapses on an equal slab | 22 | 0 | 40 | **62** |
| 3 — multi-depth rollup | 450 | 0 | 60 | **510** |
| 4 — pure royalty | 0 | 1,000 | 0 | **1,000** |
| 5 — differential and royalty together | 580 | 400 | 0 | **980** |
| 6 — own-Business-Volume reward | 6 | 0 | 4 | **10** |

Full trees and derivations: [02](02-business-rules.md) §5. If any of these six totals moves, a rule has been implemented wrongly — start there, not in the UI.

---

## 6. The thirteen things most likely to be got wrong

Collected here because each one is a place where the obvious implementation is the wrong one.

1. **An inactive member still contributes fully to every calculation.** `is_active` is a display flag with zero computational effect. The original spec wording says otherwise and is superseded. Implementing the original silently corrupts every ancestor's total. — Rule-28, [02](02-business-rules.md).
2. **Differential re-scans *all* of an ancestor's direct children**, not just the one on the changed chain. When an ancestor's own slab moves, every sibling's term moves with it. — [04](04-technical-architecture.md) §5.2.
3. **The slab table is not validated for monotonicity.** The client was offered the safeguard and declined it. Do not add it. — Rule-41, ADR-009.
4. **Zero is refused, not just negatives.** A member with no activity has no entry, not a zero entry. — Rule-16a.
5. **Member IDs start at 100001, not 100000.** — Rule-35.
6. **There is no delete, anywhere, for anything.** No member, entry, snapshot or backup is ever removed. Do not propose an erasure path. — Rule-42.
7. **A closed month is correctable.** It writes a *new snapshot version*; version 1 is never touched. Reporting reads `MAX(version)`. — Rule-39, ADR-006.
8. **There is no recalculate button and there must never be one.** No command triggers a calculation. — Rule-26.
9. **A month with no entries produces no snapshot at all** and is excluded from the yearly-average denominator. — RQ-16, Rule-23.
10. **Backup verification gates the close transactionally.** Not a prompt, not a warning — nothing is zeroed until the internal retained copy is written *and* verified. — Rule-18.
11. **An outstanding month is not locked for entry — the *current* month is.** Rule-36 was amended on 7 August 2026 and now blocks the opposite of what its earlier wording said. A figure dated in the ended-but-unclosed month must be accepted; a figure dated in the current month must be refused while that older month waits. Any document, comment or variable name still describing a total entry lock is stale. The schema value is `awaiting_close`, not `ended_locked`, for exactly this reason. — Rule-36, [06](06-decision-log-and-open-items.md) §5 CR-2.
12. **An entry belongs to the month its own date falls in** — never to "the month currently being closed". `period_month` is derived from `entry_date` and fixed there. This is the trap Rule-21's struck-through third bullet describes, and it became reachable again when Rule-36 was narrowed. — Rule-21, Rule-36.
13. **Showing more of the tree never means showing more per node.** The full hierarchy window (FR-10) expands every branch, and each node still shows exactly name, ID and own Business Volume — never Total Business Volume. FR-2's constraint belongs to the node component, not to the screen. — FR-2, Rule-45.

---

## 7. Reading order for a build session

**Starting the project:** [01](01-product-and-scope.md) → [02](02-business-rules.md) → [04](04-technical-architecture.md) → [delivery-plan.md](delivery-plan.md).

**Picking up a story:** [delivery-plan.md](delivery-plan.md) for the story → [02](02-business-rules.md) for the rules it names → [04](04-technical-architecture.md) for its API contract and schema → [03](03-functional-specification.md) for its screen → **if the story touches UI, [07](07-design-system.md) for the exact colour/type/component spec** → [05](05-quality-and-acceptance.md) for its tests and Definition of Done.

**Something looks wrong:** [06](06-decision-log-and-open-items.md) first. If a decision looks like an oversight, it is more likely a recorded client choice.

**Answering "is this in scope?":** [01](01-product-and-scope.md) §5. Fifteen items are explicitly and permanently out of scope; five more are deferred.

---

## 8. Change control

This set is the build reference, so it must not drift.

- **A client decision changes something.** Update the affected file, add a dated row to [06](06-decision-log-and-open-items.md) §5, and update the source-document reference. Never edit a source document to match — sources are the historical record.
- **A client change request arrives after approval.** Give it the next `CR-N`, record it in [06](06-decision-log-and-open-items.md) §5 with what was asked, what was decided and what it reverses, then propagate. **Append new IDs; amend existing ones in place, keeping the superseded wording visible.** Never renumber. The 7 August 2026 batch (CR-1/2/3) is the worked example to copy. The tier-1 business documents get an appended, dated addendum — they are never rewritten.
- **An open item (O1–O5) is answered.** Move it out of [06](06-decision-log-and-open-items.md) §3 into §2 with the answer, its date, and who gave it. Update every file that depends on it.
- **An implementation detail changes a contract.** Update [04](04-technical-architecture.md), then check [05](05-quality-and-acceptance.md) for the test that asserts it.
- **Never** resolve a contradiction by picking whichever reading is easier to build. The precedence table in §3 decides it; if precedence does not settle it, it belongs in the open register, not in code.

---

## 9. What is not here

- **Code.** The repository contains no `package.json`, `Cargo.toml`, `tsconfig.json` or `src-tauri/` tree. Everything from Sprint 0 onward is greenfield.
- **A second opinion on settled decisions.** Items the client explicitly decided against — slab monotonicity validation, monitoring, member logins, currency conversion — are recorded as decisions, not as gaps. See [06](06-decision-log-and-open-items.md) §6, "Do not re-raise".
- **Invented defaults.** Where no source settles a value, it is in the open register with the question stated, not filled in with a plausible guess.

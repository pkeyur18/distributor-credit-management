# Implementation Context

**Read this file first in any future Claude Code session picking up implementation work on this project.** It is a condensed summary, not a replacement for the source documents — every section links to the full deliverable or source artifact it was distilled from.

> ⚠️ **Superseded by `documents/final/` (8 August 2026).** That set is the build reference; where this file disagrees with it, it is wrong. This file was written on 6 August and has been patched — not rewritten — for the three change requests of 7 August (**CR-1** phone search, **CR-2** entry into an ended-but-unclosed month, **CR-3** the full hierarchy window) and the two change requests of 8 August (**CR-4** own-Business-Volume reward/Rule-46, **CR-5** Rewards-by-slab chart). **CR-2 reverses a previously frozen rule**: recording is no longer locked outright when a month ends. **CR-4 adds a third additive term to Rewards** without changing Differential or Royalty. Start at [../final/00-master-index.md](../final/00-master-index.md).

---

## 1. Project Purpose

A single-admin, offline desktop application for Siddharth Patel to manage a referral-network hierarchy (500–5,000 members today, designed to scale to 25,000). Each month he records a Business Volume figure against members who were active; the system rolls those figures up the hierarchy and computes each member's Rewards from the percentage-slab differential between them and their direct children, plus a royalty bonus. Today this is done entirely by hand. The system replaces that with instant, correct-on-save calculation, a permanent historical record, and simple Excel exports. **No money ever appears anywhere in the software** — figures are unitless "Business Volume"/"Rewards," converted to rupees by the client manually, outside the application.

Full narrative: `documents/business/project-confirmation-summary.html`.

## 2. Repository State (as of this analysis)

**Documentation-only.** Zero application code, no `package.json`/`Cargo.toml`/`tsconfig.json`, no `src-tauri/` tree, no tests, no CI. Everything from Sprint 0 (stack scaffolding) onward is greenfield. See [09-implementation-backlog.md](09-implementation-backlog.md) Epic 0.

## 3. Technology Stack

- **Framework:** Tauri v2 — React + TypeScript frontend (WebView), Rust application core. No server, no client-server split, no network layer at all (ADR-001, ADR-002).
- **Database:** SQLCipher-encrypted SQLite, accessed via `rusqlite`.
- **UI:** shadcn/ui + Tailwind CSS. Inter font bundled locally (no CDN, offline constraint). Design tokens in `documents/design/ui-theme.md` — one accent colour, flat design, status never colour-only, `tabular-nums` for all figures.
- **Excel generation:** `rust_xlsxwriter`, Rust-side only — the WebView never touches raw file content or paths.
- **Auth hashing:** Argon2id.
- **Packaging:** Native Tauri bundler, Windows (.msi/.exe) + macOS (.dmg/.app), no auto-update, ~10–20MB installer.
- **Precision:** All volume/reward figures stored as fixed-point integers (× 100), never floats (ADR-004) — this is what makes "no intermediate rounding" (Rule-22) achievable.

Full detail: `../final/04-technical-architecture.md` (12 ADRs). (`documents/design/architecture.md` is an earlier draft, superseded — not used for implementation.)

## 4. Architecture

Three containers: **Presentation** (React/TS in WebView) → **Tauri IPC** (typed, allowlisted commands — no general filesystem/shell/network capability exposed) → **Application** (Rust, 9 modules M1–M9) → **rusqlite/SQLCipher** → **Data** (one encrypted file + versioned backups). Security boundary sits between Presentation and Application — the WebView is fully sandboxed.

Modules: M1 Member Directory, M2 Business Volume Entry, M3 Calculation Engine (no exposed commands — purely internal), M4 Member Detail & Chart, M5 Monthly Close, M6 Reports & Exports, M7 Settings, M8 Authentication, M9 Audit Log.

Full detail: [05-data-model-specification.md](05-data-model-specification.md) (entities), [04-api-specification.md](04-api-specification.md) (42-command IPC surface).

## 5. The Calculation Model — the project's core logic

```
TotalBusinessVolume(x) = BusinessVolume(x) + Σ TotalBusinessVolume(c)   for every DIRECT child c
slab%(x) = lookup(TotalBusinessVolume(x))                                 [highest threshold ≤ TBV]
Differential(x) = Σ [(slab%(x) − slab%(c)) × TotalBusinessVolume(c)]      for every DIRECT child c
Royalty(x) = Σ royalty_rate × TotalBusinessVolume(c)   if ≥3 direct children on the top slab, else 0
OwnReward(x) = slab%(x) × BusinessVolume(x)                               [Rule-46, added 8 Aug 2026, CR-4]
Rewards(x) = Differential(x) + Royalty(x) + OwnReward(x)
```

Key structural guarantees: differential is never negative (parent's TBV always ≥ child's, by construction — Rule-9); royalty and differential never double-pay the same leg (Rule-11); a member earns nothing on their own Business Volume through the differential term, only on the gap to their children (Rule-8) — but earns separately through OwnReward (Rule-46), at their own slab, on their own Business Volume; royalty **stacks** — the same underlying volume can earn royalty at multiple levels of the same chain (Rule-25). Recalculation is immediate on every entry, chain-upward only (only the ancestor chain recomputes, not the whole tree — ADR-005), inside one DB transaction.

**Golden regression values** (reproduce exactly, always): Scenario 1 = 65, Scenario 2 = 62, Scenario 3 = 510, Scenario 4 = 1,000, Scenario 5 = 980, Scenario 6 = 10. Scenarios 1–3 and 6 recomputed/added 8 Aug 2026 for Rule-46/CR-4; 4–5 unchanged (own BV = 0 in both). Full worked trees: `documents/final/02-business-rules.md` §5.

Full rule text: [03-business-rules.md](03-business-rules.md).

## 6. Data Model Summary

10 entities: `members` (the hierarchy, self-referencing via `introducer_member_id`; `phone` is unique **and** a search key since CR-1), `business_volume_entries` (append-only ledger; `period_month` derived from `entry_date`), `slab_table` (editable %-band config), `member_period_totals` (live cache — **any not-yet-closed period**, amended by CR-2; was "current open period only"), `periods` (**open/awaiting_close/closed** lifecycle — `ended_locked` renamed by CR-2), `monthly_snapshots` (permanent, **versioned** historical record — all yearly reporting reads this, never live values), `backups` (metadata, internal-retained-copy is the actual close-gate), `settings` (16 configurable items — 13 scheme settings plus 3 backup settings, Rule-43), `auth` (single-row credentials), `audit_log` (append-only who-changed-what).

Full attribute-level detail: [05-data-model-specification.md](05-data-model-specification.md).

## 7. API Summary

40 Tauri IPC commands, fully specified in [04-api-specification.md](04-api-specification.md). Four things worth knowing before you touch the surface:

- **`reverse_entry` was dropped** — confirmed dead by the architect; `edit_entry` alone handles all corrections, including in closed months.
- **M3 (the calculation engine) exposes one command, not none.** No command *triggers* a calculation — there is no recalculate button (Rule-26) — but `preview_settings_impact` (API-33) asks what the engine *would* produce, without committing, to back the settings pre-save warning. The frontend cannot compute this itself.
- **Seven commands run unauthenticated**, and the set is closed: `login`, `setup_first_run`, `use_recovery_code`, `check_data_readable`, `list_restore_points`, `restore_from_backup`, `restore_from_backup_file`. The last four back the data-recovery screen and whole-console restore, and are unauthenticated of necessity — the credential hashes live in the database that could not be opened, or the restore is bringing a database onto a machine with nothing set up yet. `restore_from_backup` and `restore_from_backup_file` are the only destructive ones; both must verify the backup's checksum first.
- **No delete command exists for any entity**, by client requirement (Rule-42). Do not add one.

## 8. Authorization Summary

One role: Administrator. Full access to everything. Network members have **zero** system access — no login, no screen, ever. Dual-credential auth (PIN and/or complex password, either authenticates), mandatory failed-attempt lockout regardless of credential type, Argon2id hashing, one-time local recovery codes as the sole recovery path (no network exists for a "forgot password" email flow — loss of both credential and codes is permanently unrecoverable by design). Full detail: [06-security-authorization-matrix.md](06-security-authorization-matrix.md).

## 9. Key Workflows

1. **Onboarding** — search a Reference ID, fill name/phone/address/consent, save → random 6-digit ID (100001–999999) assigned.
2. **Recording** — search member (by name, 6-digit ID **or phone**, per Rule-44), enter one Business Volume figure, save → entire ancestor chain's TBV/slab/Rewards visible correct on screen instantly, no recalculate button anywhere. The entry always names the month it is recording into: the oldest month awaiting close if there is one, otherwise the current month (Rule-36 as amended).
3. **Monthly close** — outstanding-month alert (undismissable) → admin triggers close on the oldest outstanding month → backup generated and verified → **only then** are figures zeroed and a permanent snapshot written → alert clears; repeats for any further outstanding months, oldest-first, never merged.
4. **Correction** — any entry, in any month including closed ones, can be edited; a closed-month edit writes a new snapshot/backup **version**, the original is never overwritten, reporting always reads the latest version.
5. **Reporting** — three export types (monthly data, yearly average [divided by actual snapshot count, not a fixed 12], low-contribution [filtered on own-BV yearly average]) plus re-download of any past closed-month snapshot.

## 10. UI Behavior Notes

Vocabulary is restricted everywhere (screens, exports, error messages, tooltips, filenames): only *member, Business Volume, Rewards, royalty, volume, slab, level, leg* — never sale/purchase/order/cash/payment/commission/invoice. Hierarchy chart nodes show name/ID/**own**-Business-Volume only, never TBV (client re-confirmed this over the architect's TBV recommendation). Status is never colour-only (labelled pills). The outstanding-month banner has no dismiss control of any kind. Full theme reference: `documents/design/ui-theme.md`; full screen inventory: the cross-check section of this analysis's exploration (see [01](01-implementation-readiness-assessment.md) §8) and the approved prototype itself, `documents/design/ui-prototype-v2.html`.

## 11. Error Handling Summary

Full matrix: [07-error-edge-case-matrix.md](07-error-edge-case-matrix.md). Headline patterns: validation failures are inline, non-blocking where the rule is advisory (level-width/depth guidance always warns, never blocks); backup failure during close **aborts everything, mutates nothing**; once a month ends it **keeps accepting entries dated within it** while the **current** month is refused until it closes (Rule-36 as amended by CR-2, 7 Aug 2026 — this reverses the earlier "recording is fully locked" wording, which appears throughout the 6 August documents and is stale); inactive-member deactivation has **zero calculation effect** (display-only) — this is the single highest-risk correction to implement faithfully, since the original spec wording (superseded) would silently corrupt ancestor totals if followed.

## 12. Testing Strategy Summary

The six worked scenarios are the project's golden regression set — any calculation-engine change must continue to reproduce 65/62/510/1,000/980/10 exactly. Full strategy across unit/integration/API/E2E/security/performance/UAT levels: [08-testing-strategy.md](08-testing-strategy.md).

## 13. Project Conventions

- **No abbreviations for the three core quantities** — "Business Volume," "Total Business Volume," "Rewards" are always spelled out; `ICP`/`BV` are retired terms and must never reappear in code, comments, or UI strings (a prior rename made `BV` dangerously ambiguous).
- **Vocabulary constraint** (§10 above) applies to every user-visible string without exception, including test fixtures and mock data if they're ever shown in a screenshot/demo.
- **No rupee/currency figure anywhere** except the settings screen's reference-only conversion rate, which is never used in any calculation and never displayed elsewhere.

## 14. Constraints

- **Offline-only** — no network capability exists in the Tauri config, structurally, not by policy. No cloud sync, no remote auth, no telemetry.
- **Single-user, single-machine, single-session** — no concurrency-control design is needed or should be added.
- **Data protection** — consent is captured at onboarding (Rule-40). Permanent, complete retention is an explicit client requirement: members are **never** removed and all data persists throughout, including in exports. There is no erasure path by design.
- **Desktop only** — no browser, phone, or tablet target, ever, by design.

## 15. Assumptions Carried Forward

See [01-implementation-readiness-assessment.md](01-implementation-readiness-assessment.md) §5 for the full list. The most consequential: where `client-requirements-validation.md` and `requirement-spec.md` conflict, the validation document's later, explicitly-dated decision wins (four such conflicts identified and resolved — inactive-member calc effect, member ID range, dual-credential auth, entries-editable-anytime).

## 16. Decisions

See [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md) "Resolved This Session" — `reverse_entry` dropped, `edit_entry` is the sole correction mechanism.

## 17. Unresolved Items

**None.** As of 6 August 2026 every item raised by the readiness analysis is closed — see [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md). Three points worth carrying into any build session:

- **Members are never removed, and all data persists throughout — including in exports.** This is an explicit client requirement, not an oversight. There is no erasure path and none is to be built or proposed.
- **Three behaviours were added to the approved prototype after the original analysis** and are now approved reference behaviour to port: the settings mid-period recalculation warning (names the open month, Rewards before → after, lists affected members; fires on slab-table and royalty saves only), the refusal to remove the last slab row, and the data-recovery screen shown when the data file cannot be opened at launch.
- Only the Business Volume entries-per-month sizing figure remains unsupplied, deliberately deferred to the later performance-testing phase. It affects test realism, not architecture.

## 18. Traceability References

| Need | Document |
|---|---|
| "What does Rule X actually require?" | [03-business-rules.md](03-business-rules.md) |
| "What screens/fields does this touch?" | [02-requirements-traceability-matrix.md](02-requirements-traceability-matrix.md) |
| "What's the exact IPC contract?" | [04-api-specification.md](04-api-specification.md) |
| "What's the DB schema?" | [05-data-model-specification.md](05-data-model-specification.md) |
| "Who can do what?" | [06-security-authorization-matrix.md](06-security-authorization-matrix.md) |
| "What happens if X fails?" | [07-error-edge-case-matrix.md](07-error-edge-case-matrix.md) |
| "How do I test this?" | [08-testing-strategy.md](08-testing-strategy.md) |
| "What's the next story to build?" | [09-implementation-backlog.md](09-implementation-backlog.md) |
| "Is this story actually done?" | [10-definition-of-done.md](10-definition-of-done.md) |
| "Is this thing I'm unsure about already answered?" | [11-open-questions-and-decisions.md](11-open-questions-and-decisions.md) |
| "Is the project ready to build?" | [01-implementation-readiness-assessment.md](01-implementation-readiness-assessment.md) |

When in doubt about a business rule, **[03-business-rules.md](03-business-rules.md)'s full Rule-1…Rule-46 set (correcting and extending `requirement-spec.md`'s original Rules 1–38) is the canonical text** — everything else, including this file, is a derived summary.

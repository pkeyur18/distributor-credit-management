# PI Backlog — Epics, Features, User Stories, Tasks

**13 epics · 24 features · 57 user stories · 309 tasks · 161.0 ideal solo days.**

Acceptance criteria for the 36 pre-existing stories (`US-0.1` … `US-M9.1`) live in `documents/refinement/delivery-plan.md` §3 and are **not restated here**. This file adds the layer that document stops short of: the tasks that deliver each story, their dependencies, and their estimates.

**Story header format**

> **US-ID — Title.** `Sprint` · `estimate`
> *Refs:* rule and requirement IDs · *AC:* acceptance-criteria IDs · *API:* command IDs · *Screen:* the specification section and prototype view
> *Depends:* upstream stories

**Estimates** are ideal solo days. **`⚠️`** marks a task where the obvious implementation is the wrong one — the note says why.

---

## Epic index

| Epic | Name | Features | Stories | Tasks | Est (d) | Sprints |
|---|---|---|---|---|---|---|
| **E0** | Project Scaffolding | 2 | 2 | 13 | 8.0 | S1 |
| **E-UI** | UI Foundation & Design System | 5 | 5 | 28 | 15.5 | S2–S3 |
| **E-QA** | Test Foundation | 2 | 6 | 23 | 13.5 | S2, S4, S7, S10, S14 |
| **M1** | Member Directory | 2 | 4 | 28 | 12.5 | S4–S5 |
| **M8** | Authentication & Console Backup/Restore | 2 | 6 | 29 | 14.5 | S5, S7–S8, S14 |
| **M3** | Calculation Engine | 1 | 2 | 15 | 11.0 | S6 |
| **M2** | Business Volume Entry | 2 | 5 | 26 | 12.5 | S7, S12–S13 |
| **M4** | Member Detail & Hierarchy | 2 | 4 | 28 | 15.0 | S8–S9 |
| **M7** | Settings | 1 | 4 | 27 | 14.0 | S10–S11 |
| **M5** | Monthly Close | 1 | 5 | 28 | 15.0 | S11–S13 |
| **M6** | Reports & Exports | 1 | 5 | 23 | 9.0 | S13 |
| **M9** | Audit Log | 1 | 1 | 6 | 3.5 | S14 (wired from S4) |
| **E-REL** | Release, Deployment & Handover | 3 | 8 | 35 | 17.0 | S1, S15–S16 |
| | **Total** | **24** | **57** | **309** | **161.0** | |

**161.0 ideal solo days ≈ 16 two-week sprints at 10 working days each.** That is the honest number, not the one that fits a preferred shape — the eight-sprint proposal in `documents/refinement/delivery-plan.md` §4 predates the three epics this plan adds (E-UI, E-QA, E-REL) and cannot absorb them.

⚠️ **Treat 16 sprints as the floor, not the estimate.** A sprint rarely yields 10 fully productive days once defect-fixing, re-work and the Definition-of-Done pass are counted. [02-roadmap.md](02-roadmap.md) §7 states the realistic range and what to cut if it has to compress.

---

# Epic E0 — Project Scaffolding

*No `package.json`, `Cargo.toml`, `tsconfig.json` or `src-tauri/` tree exists. Genuinely greenfield.*

## Feature E0.1 — Stack scaffolding

> **US-0.1 — Initialize the Tauri v2 project.** `S1` · 3.0d
> *Refs:* ADR-002, ADR-011 · *DoD:* builds and runs an empty shell on Windows and macOS
> *Depends:* none — first work in the project

- `T-0.1-1` — Initialize Tauri v2 with the React + TypeScript template; set the product name, identifier and window defaults. — 0.5d
- `T-0.1-2` — Establish the folder structure exactly as `04-technical-architecture.md` §12 specifies: `src/screens`, `src/windows`, `src/components`, `src/lib`, and `src-tauri/src/m1_members` … `m9_audit`, `db/`, `error.rs`, `capabilities/`. Module boundaries mirror §3.1 so the rule→module map doubles as a map of the code. — 0.5d
- `T-0.1-3` — Configure Tailwind CSS and initialize shadcn/ui. — 0.5d
- `T-0.1-4` — ⚠️ Bundle the **Inter** font locally as a static asset. No CDN, no `@font-face` pointing at a network URL, ever — the offline constraint is structural (ADR-001). `DESIGN.md` uses the system-ui stack as a CSP-safe *prototype placeholder*; Inter is the real decision. — 0.5d
- `T-0.1-5` — ⚠️ Configure `src-tauri/capabilities/` with **no** network, shell or general filesystem capability. The absence of network capability is the offline guarantee — it must not exist to be misconfigured later. — 0.5d
- `T-0.1-6` — Set up ESLint + Prettier (TypeScript) and `clippy` + `rustfmt` (Rust); add the `AppError` shared type skeleton in `src-tauri/src/error.rs`. — 0.5d

## Feature E0.2 — Database & encryption foundation

> **US-0.2 — Database & encryption foundation.** `S1` · 5.0d
> *Refs:* ADR-003, ADR-004, ADR-012 · *AC:* — · *Schema:* `04-technical-architecture.md` §4.4 (full DDL)
> *Depends:* US-0.1

- `T-0.2-1` — Wire `rusqlite` with SQLCipher (bundled build). ⚠️ Pin exact dependency versions and record the known-good toolchain — TR-1 names this build as fragile across OS/toolchain updates. — 1.0d
- `T-0.2-2` — Implement the full DDL for all 10 entities from §4.4, with three corrections taken 8 Aug 2026 because no implementation exists yet and they are expensive to change later. ⚠️ `periods.status` takes the value **`awaiting_close`**, never `ended_locked` (CR-2). ⚠️ **D-12:** `audit_log.entity_type` is `member|entry|setting|period|`**`backup`**`|`**`auth`** — the published enum cannot represent the backup, auth-setup, recovery or restore events that API-15/26/30/36/39/40 all require an audit entry for. ⚠️ **D-13:** `audit_log.cause` gains **`console_backup`** and **`restore`**, and drops the unused **`reversal`** value (`reverse_entry` was dropped; `edit`/`correction` cover every case). ⚠️ **D-14:** `auth.session_timeout_minutes` is **not created** — the timeout lives only in `settings`, one source of truth. — 1.0d
- `T-0.2-3` — ⚠️ Land the **`backups` table generalization (ADR-012)** now, in Sprint 1 — nullable `period_id`, plus `kind` and `schedule_kind`. It is a schema decision, not a feature, and three later stories (US-M7.4, US-M8.5, US-M8.6) cannot start until it exists. — 0.5d
- `T-0.2-4` — Implement the versioned migration runner. Required by DoD item 10 even though the system launches empty — once client data exists, ad-hoc schema edits are unacceptable. — 1.0d
- `T-0.2-5` — Implement first-run seeding: 7 default slab rows and **16** settings rows (not 13 — conflict C1). Seeds hierarchy depth **4** (D-3), session timeout **15 minutes** (D-4), and the **five**-column mandatory export set (D-1). — 0.5d
- `T-0.2-6` — ⚠️ Implement the fixed-point convention: every volume and reward figure stored as a ×100 integer, never a float (ADR-004). This is what makes Rule-22's "no intermediate rounding" achievable; a float anywhere in the chain defeats it silently. — 0.5d
- `T-0.2-7` — Verification test: a fresh launch creates the encrypted file with all 10 tables and both seed sets; the same file opened by a plain SQLite client is unreadable. — 0.5d

---

# Epic E-UI — UI Foundation & Design System

*New epic. The design-system port is currently implicit inside every UI story; building it once, first, removes that duplication and makes `DESIGN.md`'s named rules enforceable rather than aspirational.*

## Feature UI.1 — Tokens and theme

> **US-UI.1 — Design tokens and light/dark theme.** `S2` · 2.5d
> *Refs:* `DESIGN.md` (frontmatter + all named rules), `07-design-system.md`
> *Depends:* US-0.1

- `T-UI.1-1` — Translate `DESIGN.md`'s frontmatter into CSS custom properties and a Tailwind theme extension: colours (indigo, indigo-weak, slate-bg, white-surface, slate-border, ink, slate-muted, ledger-green, amber, amber-text, red, and each `-weak` tint), typography scale (headline / title / title-sm / body / label / numeric / numeric-lg / caption), radius tiers, spacing scale. — 1.0d
- `T-UI.1-2` — Implement the dark-theme token pairs from `DESIGN.md` §Colors and the theme toggle with persistence. — 0.5d
- `T-UI.1-3` — ⚠️ Implement **The Tabular Rule** as a base style: `font-variant-numeric: tabular-nums` on every figure that could sit in a column, and the monospace face on six-digit member numbers only. This is not decoration — the system is read as a ledger. — 0.5d
- `T-UI.1-4` — Implement the two shadow tokens (`--shadow-modal`, control-lift) and enforce **The Flat-By-Default Rule** — no shadow on any card, row or sidebar. — 0.25d
- `T-UI.1-5` — Implement `prefers-reduced-motion` clamping for the 0.14s modal rise-and-settle. — 0.25d

## Feature UI.2 — App shell

> **US-UI.2 — Application shell and navigation.** `S2` · 2.5d
> *Refs:* `03-functional-specification.md` §5.11, `DESIGN.md` §Layout/§Navigation
> *Depends:* US-UI.1

- `T-UI.2-1` — Layout grid: fixed 236px sidebar + fluid content column, sticky at full viewport height, 32px horizontal padding, sticky top bar. — 0.5d
- `T-UI.2-2` — Sidebar navigation — Home, Structure, Business Volume Entry, Settings, Reports, Audit — with the active-item treatment (indigo-weak fill, indigo text, 600 weight, full-opacity icon). Footer: theme toggle, lock session, sign out. — 0.5d
- `T-UI.2-3` — Client-side routing across the nine primary views plus the auth phases. — 0.5d
- `T-UI.2-4` — ⚠️ Outstanding-month banner slot, present on **every** screen. **No dismiss control of any kind, not even a disguised one** — no close icon, no auto-hide, no acknowledge action (Rule-20). It clears only on a completed close. — 0.5d
- `T-UI.2-5` — Notification-list surface mirroring the outstanding-month alert as a persistent entry, not a dismissable toast. — 0.5d

## Feature UI.3 — Core components

> **US-UI.3 — Core component library.** `S3` · 4.0d
> *Refs:* `DESIGN.md` §Components, `07-design-system.md`
> *Depends:* US-UI.1

- `T-UI.3-1` — Button variants: primary, secondary, ghost, danger (two-stage hover), and `.btn-commit` — taller and bolder, reserved for the single irreversible action in the system. ⚠️ Weight communicates stakes here, never a new colour; the One Accent Rule holds. — 0.5d
- `T-UI.3-2` — ⚠️ Status pills: Active, Inactive, Slab/band, Locked, Neutral. **The label text is always present** — the dot reinforces the colour, it never substitutes for the word (the Color-Plus-Label Rule, a hard §11.8 requirement, not a preference). — 0.5d
- `T-UI.3-3` — Input fields with the focus treatment (indigo border + 3px indigo-weak glow, no outline ring on top), error state with the hint line, disabled state. — 0.5d
- `T-UI.3-4` — Cards and containers: 8px radius, white surface, 1px hairline, no shadow. — 0.25d
- `T-UI.3-5` — Modal primitive: 480px / 640px `.wide`, `max-height: 88vh` with the body scrolling. ⚠️ Includes the four fixes made 7 Aug 2026 — Escape closes a dismissable modal (add/edit member opt out by design), plus `role="dialog"`, `aria-modal` and `aria-labelledby`. **Cancel first, then the action, never reversed; Cancel takes focus on open, never the confirming button.** — 1.0d
- `T-UI.3-6` — Toast: bottom-right stack, `aria-live="polite"`, ~3.4s lifetime. ⚠️ Includes the `.toast svg` size rule (a pre-existing defect fixed 7 Aug 2026). Toasts confirm; they never carry anything the operator must act on. — 0.5d
- `T-UI.3-7` — In-modal alert notes (`.modal-warn`, `.modal-danger-note`) implementing the **Blended Alert Border Rule** — 35% status colour mixed into the neutral border. ⚠️ Amber copy uses `--warning-text` (`#92400e`), not `--warning`, which fails AA on white. — 0.5d
- `T-UI.3-8` — Table primitive at ~40px row density with uppercase-tracked muted column headers. — 0.25d

## Feature UI.4 — Specialised components

> **US-UI.4 — Specialised components.** `S3` · 4.5d
> *Refs:* `DESIGN.md` §Components, `07-design-system.md`
> *Depends:* US-UI.3

- `T-UI.4-1` — Structure Tree Node (signature component): 172px card (190px root), 8px radius, 1.5px border, indigo root distinction, `translateY(-1px)` hover lift. ⚠️ **Exactly three fields — name, member number, own Business Volume. Never Total Business Volume** (FR-2's constraint belongs to the component, not the screen). — 1.0d
- `T-UI.4-2` — SVG connector rendering: 1.5px, `--border` coloured. ⚠️ Never the accent colour, never thicker than the node borders it connects — the diagram's data must always outweigh its scaffolding. — 0.5d
- `T-UI.4-3` — Bar-list chart component (track + fill + label rows), implementing the **Half-Height Bar Rule**. ⚠️ Built once and reused verbatim by both Home charts — "Members by slab" and "Rewards by slab" (CR-5). No second chart component is introduced for the second chart. — 0.75d
- `T-UI.4-4` — Impact-summary component: bordered container, hairline-separated rows, before → after with the old figure muted and the new at 650 weight. ⚠️ The unchanged state shows a single figure followed by a muted "unchanged", never an identical pair either side of an arrow. — 0.5d
- `T-UI.4-5` — Restore-option-list component: full-width card rows, custom 15px radio, selection reusing the **input-focus treatment** rather than inventing a second selection language. Primary line names the thing in the operator's terms (the month a backup holds, or when it was taken), muted line for provenance. — 0.75d
- `T-UI.4-6` — Segmented control with the control-lift shadow on the active segment and the **Nested-Radius Rule** (outer radius minus padding). — 0.5d
- `T-UI.4-7` — PIN entry dots (1.5px border, fully round) and the auth-brand mark container at the Large-Icon-Container radius formula (~25% of box width). — 0.5d

## Feature UI.5 — IPC integration layer

> **US-UI.5 — Typed IPC wrappers and error surfacing.** `S2` · 2.0d
> *Refs:* `04-technical-architecture.md` §6, `05-quality-and-acceptance.md` §2
> *Depends:* US-0.1

- `T-UI.5-1` — TypeScript types generated from or mirrored against the 40 command contracts, one wrapper per command in `src/lib`. — 1.0d
- `T-UI.5-2` — Map every typed Rust error to its user-facing presentation per the error matrix. ⚠️ Includes `PeriodNotAcceptingEntries { month, blocking_month }` and `PeriodClosed { month }`. **The retired `PeriodLocked` variant must not be reintroduced under a new meaning.** — 0.5d
- `T-UI.5-3` — Loading and empty states as shared primitives, so no screen invents its own. — 0.5d

---

# Epic E-QA — Test Foundation

*New epic. `05-quality-and-acceptance.md` §3 defines a thorough strategy but no harness exists to run it, no fixtures exist to run it against, and no dataset exists to run it at scale.*

## Feature QA.1 — Harnesses

> **US-QA.1 — Rust test harness and golden-scenario fixtures.** `S4` · 2.5d
> *Refs:* `05-quality-and-acceptance.md` §1, §3.1
> *Depends:* US-0.2

- `T-QA.1-1` — Test harness with in-memory/temp encrypted database fixtures and a per-test teardown. — 0.5d
- `T-QA.1-2` — ⚠️ Encode all six golden scenarios as **data fixtures, not test code** — the input trees from `02-business-rules.md` §5.1–5.6 with their expected Differential / Royalty / OwnReward / Total. Data means a seventh scenario is a row, not a new test. — 1.0d
- `T-QA.1-3` — Assertion helper that reports which of the three terms diverged when a total moves, not just that the total is wrong. — 0.5d
- `T-QA.1-4` — Add `proptest` for the property-based differential non-negativity test. ⚠️ The test must **explicitly document its monotonic-slab-table assumption**, since Rule-41 removes that guarantee at the input layer. — 0.5d

> **US-QA.2 — Contract-test harness for the IPC surface.** `S4` · 2.5d
> *Refs:* `04-technical-architecture.md` §6, `06-decision-log-and-open-items.md` C2/C3
> *Depends:* US-QA.1

- `T-QA.2-1` — Direct Tauri command-invocation harness — no HTTP layer, no browser or network mocking needed. — 1.0d
- `T-QA.2-2` — ⚠️ Authorization test asserting the unauthenticated set is **exactly seven**: `login`, `setup_first_run`, `use_recovery_code`, `check_data_readable`, `list_restore_points`, `restore_from_backup`, `restore_from_backup_file`. Not six — the stale count in two older documents would fail against correct code (C3). The test asserts the list *exactly*, so an eighth command cannot join it by accident. — 0.5d
- `T-QA.2-3` — ⚠️ Test that the surface holds **40** commands, API-01 to API-40 with no gaps (C2), and that the Tauri capability allowlist has exactly 40 entries. — 0.5d
- `T-QA.2-4` — Per-command test template: request/response shape, validation, authorization, documented error responses. Filled in per command as each module ships. — 0.5d

> **US-QA.3 — E2E rig.** `S7` · 3.0d
> *Refs:* `05-quality-and-acceptance.md` §3.1, D-8
> *Depends:* US-UI.2

- `T-QA.3-1` — `tauri-driver` + WebdriverIO harness, with app build/launch/teardown per suite. — 1.0d
- `T-QA.3-2` — Test-data seeding helpers so an E2E suite can start from a known hierarchy and period state. — 0.5d
- `T-QA.3-3` — ⚠️ **macOS manual verification checklist**, scripted step by step. `tauri-driver` has no macOS support — WKWebView exposes no WebDriver — so macOS coverage is manual by necessity (D-8). The checklist is the deliverable that stops that becoming "untested". — 1.0d
- `T-QA.3-4` — Screenshot capture on failure, retained per run. — 0.5d

## Feature QA.2 — Tooling and data

> **US-QA.4 — Vocabulary grep tool.** `S2` · 1.0d
> *Refs:* `01-product-and-scope.md` §3, UN-27, BC-4, AC-36, SC-7
> *Depends:* US-0.1

- `T-QA.4-1` — ⚠️ Scanner over every literal string in the build — screen labels, buttons, column headings, toasts, tooltips, placeholder and empty-state copy, error messages, extract filenames, **and test fixtures and mock data** — against the excluded-word list. Mock data counts: it can appear in a screenshot or a demo. — 0.5d
- `T-QA.4-2` — Wire it to fail the build on any hit, and into the pre-release gate (`US-REL.2`). — 0.25d
- `T-QA.4-3` — Allowlist mechanism for the legitimate cases. Two exist: the specification and planning documents, which must quote the excluded terms in order to forbid them; and ⚠️ **ordinary English uses of *order*** in code and comments — post-order traversal, sort order, "closing out of order". The constraint binds **user-visible product strings**, so the scan targets those; a naive whole-repository word match would flag the calculation engine's own layout pass. — 0.25d

> **US-QA.5 — Synthetic dataset generator.** `S10` · 2.5d
> *Refs:* NFR-1, NFR-2, C7 (~1,000 entries/month, variable)
> *Depends:* US-M1.1, US-M2.1

- `T-QA.5-1` — Generator producing realistic hierarchies at three scales: 500, 5,000 (client's actual range) and 25,000 members (design ceiling), with configurable depth and branching. — 1.0d
- `T-QA.5-2` — ⚠️ Names, addresses and all mock content drawn from a **vocabulary-clean** corpus, so the generated dataset passes `US-QA.4` — a fixture that fails the grep is a fixture that can never be screenshotted. — 0.5d
- `T-QA.5-3` — Entry generator: ~1,000 Business Volume entries per month across a full year (200,000 entries at ceiling scale), all amounts strictly `> 0` per Rule-16a. — 0.5d
- `T-QA.5-4` — Deterministic seeding so a performance regression is measured against an identical dataset, not a fresh random one. — 0.5d

> **US-QA.6 — Performance measurement harness.** `S14` · 2.0d
> *Refs:* NFR-1, NFR-2, TR-7, AC-45
> *Depends:* US-QA.5

- `T-QA.6-1` — Timing harness for the three NFR-1 targets: screen render < 2s, recalculation < 2s, extract < 30s. — 0.5d
- `T-QA.6-2` — Complexity measurement: recalculation time against tree depth and width, confirming *O*(depth × average width) and its independence from total member count. — 0.5d
- `T-QA.6-3` — ⚠️ **Main-console responsiveness probe measured concurrently** with the full hierarchy draw. The draw time itself is a known accepted cost (TR-7); the console slowing down is a defect against the client's binding constraint on CR-3 (AC-45). The second measurement is the one that gates. — 0.75d
- `T-QA.6-4` — Search timing including a **phone** query, whose canonical-key match is a scan rather than an index seek (Rule-44). — 0.25d

---

# Epic M1 — Member Directory

## Feature M1.1 — Onboard and maintain members

> **US-M1.1 — Add a new member.** `S4` · 5.25d
> *Refs:* FR-4, Rule-30/34/35/40, UN-02/03 · *AC:* AC-8, AC-9, AC-11, AC-14 · *API:* API-01, API-02 · *Screen:* §5.1 Add Member modal
> *Depends:* US-0.2, US-UI.3

- `T-M1.1-1` — ⚠️ Member ID allocator: random, six digits, range **100001–999999** — 100000 is never assigned (C4). Draws only from currently-unallocated numbers; a deactivated member's ID stays taken forever. Never sequential, never released. — 1.0d
- `T-M1.1-2` — `create_root_member` (API-01): callable exactly once, no Reference ID, guarded so a second root cannot be created by any route (AC-7). — 0.5d
- `T-M1.1-3` — `add_member` (API-02) with Reference-ID resolution — must resolve to an existing **and active** member (Rule-30). — 0.75d
- `T-M1.1-4` — ⚠️ Phone uniqueness across **active and inactive** members (Rule-34). A match on an active member is an error; a match on an inactive member is **not an error** — it returns a named reactivation offer. Uses the same canonical key as Rule-44's search, so one person cannot be added twice under two spellings of one number. — 0.75d
- `T-M1.1-5` — Consent capture: mandatory checkbox plus auto-captured date (Rule-40). Save is **disabled** until ticked — the action is simply unavailable, not a submit-then-reject error. — 0.25d
- `T-M1.1-6` — ⚠️ Level-width and depth guidance: **warns, never blocks** (Rule-1, Rule-32). Per D-5, the width warning is **suppressed** for any level with no configured width; the depth warning still fires independently. Never reuse level 4's width for level 5+. — 0.5d
- `T-M1.1-7` — Add Member modal UI, including the reference lookup (active-only filter) and the inline duplicate-phone check. — 0.5d
- `T-M1.1-8` — Unit tests: ID allocation (random, excludes 100000, never reused), phone uniqueness + reactivation payload, consent gating, advisory warnings. — 0.5d
- `T-M1.1-9` — Contract test for API-01 and API-02; audit-log wiring (`entry` cause). — 0.25d
- `T-M1.1-10` — Field-level validation: **V1.1** name required, refused naming the field; **V1.4** email optional, but if given must be a valid address — the field alone is refused, not the save. — 0.25d

> **US-M1.2 — Edit an existing member.** `S5` · 1.5d
> *Refs:* Rule-28, Rule-34, Rule-37 · *AC:* AC-12 · *API:* API-03 · *Screen:* Edit Member modal
> *Depends:* US-M1.1

- `T-M1.2-1` — ⚠️ `edit_member` (API-03). The **introducer field is never accepted as editable input, locked at the API layer** — not merely disabled in the UI (Rule-37). No route through the system changes an introducer. — 0.5d
- `T-M1.2-2` — Re-check phone uniqueness on edit against the same canonical key. — 0.25d
- `T-M1.2-3` — Edit modal UI: introducer displayed but visibly not editable. — 0.25d
- `T-M1.2-4` — Audit wiring: one entry per changed field, cause `edit`. — 0.25d
- `T-M1.2-5` — Unit + contract tests, including an attempt to set an introducer through the API directly. — 0.25d

> **US-M1.3 — Deactivate and reactivate a member.** `S5` · 2.25d ⚠️ *Highest-risk regression in the project*
> *Refs:* Rule-28 (corrected, C5) · *AC:* AC-10, AC-13 · *API:* API-04, API-05
> *Depends:* US-M1.1

- `T-M1.3-1` — ⚠️⚠️ `deactivate_member` (API-04) sets `is_active` **and triggers no recalculation whatsoever**. `is_active` is a display flag with **zero computational effect** — the member's own Business Volume still counts fully toward their introducer's figure and their downline still rolls up through them exactly as before. Implementing the superseded spec wording ("stops appearing in new periods") silently corrupts every ancestor's Total Business Volume. — 0.5d
- `T-M1.3-2` — Root member cannot be deactivated: control unavailable and the command refuses. — 0.25d
- `T-M1.3-3` — `reactivate_member` (API-05): original ID, hierarchy position and full history preserved unchanged; no second record created. — 0.5d
- `T-M1.3-4` — ⚠️ Display treatment for inactive members: a distinct colour **plus a labelled pill** in the chart, member lists and every extract row (M4.5, M6.5). Never colour alone. — 0.25d
- `T-M1.3-5` — ⚠️ **The single highest-value regression test in the suite**: deactivate a mid-tree member with active descendants, assert every ancestor's Total Business Volume and Rewards are byte-identical to before. — 0.25d
- `T-M1.3-6` — ⚠️ TEST-R42: no code path deletes a `members` row — assert there is no delete command to call — and every export includes deactivated members. The test exists to stop a future change quietly introducing an erasure path or an "active only" filter. — 0.25d
- `T-M1.3-7` — Deactivate confirmation modal (`confirmDeactivate` in the approved prototype), stating that the member's figures keep contributing exactly as before. ⚠️ Disabled outright for the root member — the action is unavailable, not refused after the fact. — 0.25d

## Feature M1.2 — Search

> **US-M1.4 — Search by name, ID or phone.** `S5` · 3.5d *(amended 7 Aug 2026, CR-1)*
> *Refs:* FR-1, Rule-44, UN-15, UN-29 · *AC:* AC-40, AC-41 · *API:* API-06 · *Screen:* §5.1
> *Depends:* US-M1.1

- `T-M1.4-1` — ⚠️ **One shared search function backing every search box in the console** — Home, Structure, Business Volume Entry, Correction panel, and the Add-Member reference lookup (which keeps its active-only filter per Rule-30). Behaviour differing between screens would be a defect, not a feature. — 1.0d
- `T-M1.4-2` — ⚠️ Phone canonicalisation: reduce **both sides** to a canonical key — strip non-digits, then drop an international prefix or trunk zero. Not a bare digit-strip: a member stored as plain `9876543210` must be found when `+91 98765 43210` is typed, which is the direction a naive implementation fails. **The stored value is never rewritten.** — 0.75d
- `T-M1.4-3` — Four-digit floor on the phone clause (V4.4): a shorter query matches on phone not at all, while name and ID matching are unaffected — and this is **not an error**, just no phone match. — 0.25d
- `T-M1.4-4` — Name-substring and six-digit-ID matching. Empty query returns **no results** — not an error, not "all members" (V4.1). — 0.5d
- `T-M1.4-5` — Search-results component: name, ID, **phone**, Total Business Volume, slab, status pill. Inactive members appear normally, in the distinct colour with a labelled pill. — 0.5d
- `T-M1.4-6` — ⚠️ TEST-R44: stored `+91 98765 43210` found by `9876543210`, `98765 43210`, `+919876543210`, `09876543210`, and the mid-number fragment `4321`; **and the reverse direction**; a 3-digit query matches on phone not at all; a query matching one member's name and another's phone returns both. Asserted against the shared function, so behaviour cannot drift between screens. — 0.5d

---

# Epic M8 — Authentication & Console Backup/Restore

*Feature M8.1 has no data dependency on M1 and runs in parallel with it.*

## Feature M8.1 — Setup, login, lockout, recovery

> **US-M8.1 — First-run setup.** `S5` · 3.0d
> *Refs:* Rule-29, ADR-008, UN-26 · *AC:* AC-34 · *API:* API-26 · *Screen:* §5.10 Setup
> *Depends:* US-0.2, US-UI.3

- `T-M8.1-1` — ⚠️ Argon2id credential hashing. **PIN and password may both be set; either authenticates** (M8.5, C6) — `pin_hash` and `password_hash` are independently nullable with at least one required. PIN is 6 numeric digits; password ≥8 chars with a letter and a number. — 1.0d
- `T-M8.1-2` — ⚠️ Tune Argon2id cost parameters against a **deliberately modest baseline machine**, not the development machine (TR-2) — parameters tuned on fast hardware feel sluggish on the client's. — 0.5d
- `T-M8.1-3` — Recovery-code generation: one-time codes, hashed at rest, displayed exactly once. — 0.5d
- `T-M8.1-4` — Setup wizard UI: step 0 credential mode + entry, step 1 recovery-code reveal with the mandatory "I have saved this…" checkbox gating the "Enter the console" action. — 0.75d
- `T-M8.1-5` — ⚠️ The "Restore from a backup file instead" **plain link** on the setup screen — not a competing button, and no separate welcome/choice screen. Wired in US-M8.6. — 0.25d

> **US-M8.2 — Login with lockout.** `S5` · 2.5d
> *Refs:* Rule-29, D-2 · *AC:* AC-35 · *API:* API-27 · *Screen:* §5.10 Login
> *Depends:* US-M8.1

- `T-M8.2-1` — `login` (API-27) verifying against either credential. ⚠️ A wrong credential returns a **generic** "incorrect" message that never reveals which credential type or which part was wrong. — 0.5d
- `T-M8.2-2` — ⚠️ **Lockout ladder per D-2**: locks at 5 consecutive failures, then at every 5 further failures, with durations **30s → 2min → 10min → 30min → 1h (capped)**. The tier is derived from `auth.failed_attempts` — no schema change; `locked_until` already exists. — 0.75d
- `T-M8.2-3` — ⚠️ **The counter resets only on a successful login — never on lockout expiry.** The prototype's flat-20s-with-reset was demo pacing, not a security decision: it gives a patient attacker unlimited batches of five against a one-million-combination PIN. Do not port it. — 0.25d
- `T-M8.2-4` — ⚠️ Persist lockout state in the database so quitting and relaunching the application does **not** clear it. — 0.25d
- `T-M8.2-5` — Lockout screen with a live countdown and an attempts-remaining count before the threshold. — 0.5d
- `T-M8.2-6` — Security tests: exactly 5 attempts triggers lockout; countdown timing; attempts do not reset early; the ladder escalates; a process kill does not reset it. — 0.25d

> **US-M8.3 — Session lock and inactivity timeout.** `S7` · 1.5d
> *Refs:* NFR-4, D-4 · *API:* API-28, API-29 · *Screen:* §5.10 Locked
> *Depends:* US-M8.2

- `T-M8.3-1` — `lock_session` (API-28) / `unlock_session` (API-29). ⚠️ The encryption key is **genuinely dropped from memory**, not hidden behind an overlay; re-authentication genuinely re-derives it. — 0.75d
- `T-M8.3-2` — Inactivity timer at the configured timeout, default **15 minutes** (D-4), editable on Settings — not asked in the setup wizard. ⚠️ Read from **`settings` only** (D-14); there is no cached copy on `auth` to drift out of step. — 0.25d
- `T-M8.3-3` — Lock screen UI. — 0.25d
- `T-M8.3-4` — Security test: after `lock_session`, no command can be invoked without re-authentication, and the key is not retrievable as far as testable without memory-forensics tooling. — 0.25d

> **US-M8.4 — Credential recovery.** `S8` · 1.5d
> *Refs:* Rule-29, ADR-008 · *API:* API-30 · *Screen:* §5.10 Recovery
> *Depends:* US-M8.1

- `T-M8.4-1` — `use_recovery_code` (API-30): verify against the hashed codes, single-use enforcement. — 0.5d
- `T-M8.4-2` — On success, set the new credential, **invalidate all prior codes and issue a fresh set**. — 0.5d
- `T-M8.4-3` — Recovery screen UI. ⚠️ The consequence — loss of credential **and** all recovery codes is permanently unrecoverable, no vendor backdoor, no email flow — must be **communicated plainly at setup**, not buried in settings. — 0.25d
- `T-M8.4-4` — Tests: invalid code refused, used code refused, old codes dead after recovery. — 0.25d

## Feature M8.2 — Whole-console backup & cross-device restore

*New 7 Aug 2026 (Rule-43, RQ-23, ADR-012). Approved reference behaviour, prototyped — port it, do not redesign it.*

> **US-M8.5 — Take a whole-console backup, scheduled or on demand.** `S14` · 2.5d
> *Refs:* Rule-43 · *AC:* AC-37 · *API:* API-39
> *Depends:* T-0.2-3 (`backups` generalization), US-M7.4

- `T-M8.5-1` — `run_console_backup_now` (API-39): copy and checksum the whole live encrypted database file, credentials included. — 1.0d
- `T-M8.5-2` — ⚠️ Schedule check **at successful login only**. There is no background service while the application is closed — this is a design constraint, **not a gap to "fix" with a background timer**. A missed day catches up at the next login. — 0.5d
- `T-M8.5-3` — ⚠️ Retention pruning to `console_backup_retention_count` (default 10), oldest `scheduled`/`manual` row first. **`period_close` and `pre_restore_safety` rows are never pruned by this.** — 0.5d
- `T-M8.5-4` — Tests: due schedule fires silently at login before the UI takes over; manual backup appears at the top of the Restore list; pruning respects the protected kinds. — 0.5d

> **US-M8.6 — Restore the console from a backup file.** `S14` · 3.5d
> *Refs:* Rule-43 · *AC:* AC-38, AC-39 · *API:* API-36, API-40 (API-35/36 widened to every `kind`)
> *Depends:* US-M8.5

- `T-M8.6-1` — `check_data_readable` (API-34) and `list_restore_points` (API-35), unauthenticated of necessity — the credential hashes live in the database that could not be opened. API-35 widened to read **every** `backups.kind`, not only `period_close`. — 0.75d
- `T-M8.6-2` — ⚠️ `restore_from_backup` (API-36) and `restore_from_backup_file` (API-40): **verify the stored checksum before overwriting anything**. A mismatch is refused and leaves the existing file untouched. These are the only destructive unauthenticated commands in the system. — 1.0d
- `T-M8.6-3` — ⚠️ Write a **`pre_restore_safety` backup of the current live state before every restore, on every entry path**. A restore is never a true one-way door. — 0.5d
- `T-M8.6-4` — ⚠️ Drop any authenticated session immediately after any restore — the restored file may carry a different credential — and land on sign-in, which still requires it. **Restoring must not grant access to anything.** — 0.25d
- `T-M8.6-5` — Data-recovery screen (LOW-3, design D): full-screen state in place of sign-in when the database cannot be opened. States nothing has been lost, lists retained backups by the month or occasion each holds (marking corrected months), offers restore and retry, and states plainly what will need re-entering. — 0.75d
- `T-M8.6-6` — ⚠️ The **same screen, reworded**, serves the voluntary first-run restore path — no internal restore-points list there, since a brand-new machine has none of its own. One screen, not a duplicate. — 0.25d

---

# Epic M3 — Calculation Engine

*Pure backend logic. No UI. One exposed command — `preview_settings_impact` — which asks what the engine would produce and writes nothing. **No command triggers a calculation.***

## Feature M3.1 — Core calculation

> **US-M3.1 — TBV / slab / differential / royalty / own-Business-Volume-reward computation.** `S6` · 6.0d
> *Refs:* Rule-3, Rule-5–13, Rule-25, Rule-46 · *AC:* AC-1–AC-6, AC-46 · *API:* — (internal)
> *Depends:* US-0.2, US-QA.1

- `T-M3.1-1` — Slab lookup (Rule-3, Rule-7): highest threshold ≤ Total Business Volume, driven by the **team** figure not the personal one. ⚠️ A value exactly at a threshold lands in the **higher** slab. ⚠️ **The scan is threshold-descending, first match wins (D-11).** `slab_table.sort_order` is **display-only** — it orders the Settings screen and nothing else. The data model's claim that `sort_order` determines lookup order is wrong and is corrected by P-9; under Rule-41's accepted risk the two orderings can diverge, and Rule-3 governs. — 0.5d
- `T-M3.1-2` — Total Business Volume rollup (Rule-6): own Business Volume + the Total Business Volume of every **direct** child. ⚠️ The own-BV term is never omitted, even where a worked example's write-up simplified it away. — 1.0d
- `T-M3.1-3` — ⚠️ Differential (Rule-8): `Σ (slab%(x) − slab%(c)) × TotalBusinessVolume(c)` over **direct children only**. The base is the child's **Total** Business Volume, not their own. A member earns nothing on their own Business Volume through this term. — 1.0d
- `T-M3.1-4` — Royalty (Rule-10, Rule-25): applies when at least `royalty_min_children` direct children sit on the top slab. ⚠️ **"Top slab" means the highest-*percentage* row, whatever its threshold** — Rule-10's own wording, and **not** the highest threshold. Under Rule-41's accepted risk those two rows can be different, which is exactly where a silent royalty defect would live. ⚠️ **Royalty stacks at every qualifying level** — the same underlying volume can attract royalty twice in one chain, confirmed with the payout consequence understood. — 1.0d
- `T-M3.1-5` — ⚠️ OwnReward (Rule-46, CR-4): `slab%(x) × BusinessVolume(x)` — own Business Volume at the member's **own** slab. A **third additive term**: `Rewards = Differential + Royalty + OwnReward`. Rule-8 and Rule-10 are **not redefined**. This reverses the 3 Aug 2026 position that own Business Volume earns nothing — for this term only. — 0.75d
- `T-M3.1-6` — ⚠️ Rule-11: differential and royalty never both pay on the same leg. Rule-13: Rewards are a separate ledger and never feed back into Business Volume or Total Business Volume. — 0.5d
- `T-M3.1-7` — ⚠️ Fixed-point arithmetic throughout, ×100 integers, **no per-child-term rounding before summing** (Rule-22). Rounding happens only at display. — 0.5d
- `T-M3.1-8` — ⚠️ **The six golden scenarios reproduce 65 / 62 / 510 / 1,000 / 980 / 10 exactly.** Plus the boundary tests (Scenario 2's C at 3,000 → 8%; Scenario 4's A at 10,000 → 14%), the royalty `min_children` boundary (2 vs 3), the P/Q/R stacking illustration, ledger isolation, and fixed-point drift across a long chain. — 0.75d

> **US-M3.2 — Chain-upward incremental recalculation.** `S6` · 5.0d
> *Refs:* Rule-26, ADR-005 · *AC:* AC-17
> *Depends:* US-M3.1

- `T-M3.2-1` — ⚠️ Ancestor-chain walk: recompute **only** the chain from the changed member to the root, never the full tree. — 1.0d
- `T-M3.2-2` — ⚠️ **Re-scan every direct child of every ancestor on that chain** for its differential term — not just the changed leaf. When an ancestor's own slab moves, every sibling's term moves with it. This is the detail §5.2 exists to call out, and the one most likely to be optimised away. — 1.0d
- `T-M3.2-3` — ⚠️ The whole recalculation runs **inside one transaction**. A crash mid-transaction rolls back cleanly; no partial recalculation can persist. — 0.75d
- `T-M3.2-4` — ⚠️ **Period isolation (CR-2):** a recalculation is confined to the period its triggering entry belongs to. `member_period_totals` may now hold rows for more than one not-yet-closed period; the composite primary key already supports it, so there is no schema change. — 0.75d
- `T-M3.2-5` — ⚠️ Inactive members are **not** filtered, zeroed or special-cased anywhere in the calculation path (Rule-28 corrected, C5). — 0.25d
- `T-M3.2-6` — Integration tests: a deep write recalculates every ancestor in one transaction; a sibling's differential changes when the parent's slab shifts; with two live periods, writing into the older leaves the other's rows byte-identical. — 0.75d
- `T-M3.2-7` — Performance test confirming *O*(depth × average width), independent of total member count. — 0.5d

---

# Epic M2 — Business Volume Entry

*Feature M2.2 depends on M5's outstanding-period state and therefore lands with Epic M5 in S7, not with the rest of M2.*

## Feature M2.1 — Record and correct entries

> **US-M2.1 — Record a Business Volume entry.** `S7` · 3.5d
> *Refs:* FR-5, Rule-15, Rule-16, Rule-16a · *AC:* AC-15, AC-16, AC-17 · *API:* API-08 · *Screen:* §5.4
> *Depends:* US-M1.1, US-M3.2, US-UI.3

- `T-M2.1-1` — `record_entry` (API-08): insert the entry and trigger the chain-upward recalculation in one transaction. ⚠️ `period_month` is derived from `entry_date` and fixed there — **never from "the period being closed"**. — 1.0d
- `T-M2.1-2` — ⚠️ Validation: amount `> 0` — **zero is refused, not just negatives** (Rule-16a, stricter than the architect's own recommendation, explicitly overridden by the client). Maximum 2 decimal places. A member with no activity has no entry, not a zero entry. — 0.5d
- `T-M2.1-3` — Entry screen: search (shared function), selected-member context, **one amount field**. ⚠️ No currency field, no mode toggle, no second field on this fast path — the target is under 15 seconds for a known member (SC-5). — 1.0d
- `T-M2.1-4` — Date field bounded to the recording month; defaults to today when recording into the current month, otherwise to the last day of that month. — 0.25d
- `T-M2.1-5` — ⚠️ Immediate on-screen update of every affected figure after save, with **no recalculate control anywhere** (Rule-26, TC-1). — 0.25d
- `T-M2.1-6` — "This period's entries" list beneath the form, showing the recording month's entries. — 0.25d
- `T-M2.1-7` — Unit + contract + E2E tests; audit wiring (`entry` cause). — 0.25d

> **US-M2.2 — Correct an entry, including in a closed month.** `S7` · 3.0d
> *Refs:* Rule-39 (extends Rule-38), UN-21 · *API:* API-09 · *Screen:* §5.5 Correction Panel
> *Depends:* US-M2.1

- `T-M2.2-1` — ⚠️ `edit_entry` (API-09) — **the sole correction mechanism**. `reverse_entry` was confirmed dead and dropped; no separate reverse or void action exists, and none is to be added. — 0.75d
- `T-M2.2-2` — ⚠️ Closed-month correction writes a **new `monthly_snapshots` / `backups` version**; version 1 is never touched. Reporting reads `MAX(version)`. — 1.0d
- `T-M2.2-3` — ⚠️ An entry's date is editable **only within its own month** (RQ-21). Moving an entry between months is deferred as an explicit future action and must not fall out of a date-field behaviour. — 0.25d
- `T-M2.2-4` — Correction panel UI with the pre-save warning: "Editing a record recalculates the affected chain and writes a new snapshot version — the original record is never overwritten." — 0.5d
- `T-M2.2-5` — Integration test: edit in a closed month → new version created, **original version's row byte-identical to before**, and `redownload_backup` plus `export_yearly_average` both read the new version. — 0.5d

## Feature M2.2 — Entry eligibility by period

*New 7 Aug 2026 (CR-2). **Reverses RQ-11's answer of 3 Aug 2026.** Read Rule-36 as amended before touching anything here.*

> **US-M2.3 — Record into a month that has ended but is not closed.** `S12` · 2.5d
> *Refs:* Rule-36 (amended), UN-30, V2.6 · *AC:* AC-19, AC-42 · *API:* API-07, API-08
> *Depends:* US-M2.1, US-M5.2

- `T-M2.3-1` — ⚠️ Target-period derivation from `entry_date`. An ended-but-unclosed month **accepts entries indefinitely, for as long as it stays unclosed**. There is no day limit, no configurable grace window, no countdown and no seventeenth setting — a configurable grace was offered and declined. — 0.75d
- `T-M2.3-2` — `get_period_lock_status` (API-07) returns the **list of recordable periods plus the blocking month — not a boolean**. The command name is retained for continuity; the semantics are entry eligibility. — 0.5d
- `T-M2.3-3` — Recording-month note heading the form: names the month being recorded into and states plainly when the current month unlocks. — 0.5d
- `T-M2.3-4` — Date bounds re-derived from the recording month (first to last day; capped at today for the current month). — 0.25d
- `T-M2.3-5` — ⚠️ Test: saving into the outstanding month recalculates **that** month's chains and leaves every other period's `member_period_totals` rows byte-identical. — 0.5d

> **US-M2.4 — Refuse a current-month entry while an earlier month is outstanding.** `S12` · 2.0d
> *Refs:* Rule-36 (amended), V2.3, V2.7 · *AC:* AC-43 · *API:* API-08
> *Depends:* US-M2.3

- `T-M2.4-1` — ⚠️ Typed errors `PeriodNotAcceptingEntries { month, blocking_month }` and `PeriodClosed { month }`. **The retired `PeriodLocked` variant must not be reintroduced under a new meaning.** — 0.5d
- `T-M2.4-2` — ⚠️ Refusal **names the blocking month**, and **only that date is rejected** — the form stays available, nothing else is disabled. There is no locked empty state; the superseded design replaced the whole form and must not be ported. — 0.5d
- `T-M2.4-3` — A date in an already-closed month is not offered here — a link points to the correction panel (Rule-39), shown as a route, not an error. — 0.25d
- `T-M2.4-4` — Future-dated entries refused. — 0.25d
- `T-M2.4-5` — ⚠️ TEST-R36, the full state matrix: June `awaiting_close`, today in August — June-dated accepted; August-dated refused naming June; May-dated (closed) directed to correction; September-dated refused as future; after June closes, the August-dated figure saves. **This is the test that fails loudly if the superseded "lock everything" wording is implemented.** — 0.5d

> **US-M2.5 — Month selector for multiple outstanding months.** `S13` · 1.5d
> *Refs:* Rule-36 (amended), Rule-20 · *Screen:* §5.1, §5.4
> *Depends:* US-M2.3

- `T-M2.5-1` — ⚠️ Selector rendered **only** when more than one month is outstanding. With a single month in play — the ordinary case — **nothing new appears on screen anywhere**. This is the client's explicit preference. — 0.5d
- `T-M2.5-2` — Selector defaults to the **oldest** outstanding month; changing it re-bounds the date field. — 0.25d
- `T-M2.5-3` — Figure screens (Home, Member Detail, Structure) show the **oldest** outstanding month by default, with the same conditional switcher. — 0.5d
- `T-M2.5-4` — E2E test with two outstanding months, and the negative case with one. *Note: the client has stated this situation will not arise in practice. It is built so the system is correct if it ever does.* — 0.25d

---

# Epic M4 — Member Detail & Hierarchy

## Feature M4.1 — Views

> **US-M4.1 — Member detail view.** `S8` · 3.0d
> *Refs:* FR-3, UN-17, Rule-46 · *AC:* AC-46 · *API:* API-10 · *Screen:* §5.2
> *Depends:* US-M3.1, US-UI.3

- `T-M4.1-1` — `get_member_detail` (API-10): contact block, figures block, Rewards breakdown, direct children at one depth, Total Business Volume, leg count. — 1.0d
- `T-M4.1-2` — ⚠️ Rewards breakdown in RQ-13's confirmed order, amended by CR-4: **own-Business-Volume reward line first** — "your own contribution, then your team's" — then one row per direct child (name, ID, their Total Business Volume, their slab, this member's slab, the difference, the resulting amount), then royalty lines, then the total. — 1.0d
- `T-M4.1-3` — The note that differential and royalty never both pay on the same leg (Rule-11). — 0.25d
- `T-M4.1-4` — Direct-children list, **one depth only** (FR-3). — 0.25d
- `T-M4.1-5` — Actions: open Structure rooted here; record an entry. ⚠️ The entry action is **always available** — it opens the entry screen, which names the month it will record into. It is never disabled by an outstanding month. — 0.25d
- `T-M4.1-6` — ⚠️ SC-3 check: a member's question about their figure is answerable **without leaving this screen**. — 0.25d

> **US-M4.2 — Hierarchy chart.** `S8` · 3.5d
> *Refs:* FR-2, UN-16 · *API:* API-11 (`full_tree: false`) · *Screen:* §5.3
> *Depends:* US-M4.1, US-UI.4

- `T-M4.2-1` — `get_direct_children_chart` (API-11) with `full_tree: false` — the member and its direct children. — 0.5d
- `T-M4.2-2` — ⚠️ Chart rendering, one generation at a time. Each node shows **exactly name, ID and own Business Volume — never Total Business Volume**. The client re-confirmed this over the architect's recommendation, accepting the consequence that the chart alone cannot explain a member's slab. — 1.0d
- `T-M4.2-3` — Connector rendering per `T-UI.4-2`. — 0.5d
- `T-M4.2-4` — Toolbar: member search (same component and matching rules as Home), zoom out / level / in, Fit width, Collapse all, View full hierarchy. ⚠️ These controls exist in the prototype and were documented nowhere until 7 Aug 2026 — they are specified behaviour, not optional polish. — 0.75d
- `T-M4.2-5` — ⚠️ **No size gate on this screen.** Its node count is bounded by a single generation; the gate belongs to the full hierarchy window, which is the only view drawing an unbounded number of nodes. — 0.25d
- `T-M4.2-6` — E2E test including the inactive-node treatment. — 0.5d

> **US-M4.4 — Rewards-by-slab chart on Home.** `S8` · 1.5d *(new 8 Aug 2026, CR-5)*
> *Refs:* FR-1 (extended), Rule-46, V4.6 · *AC:* AC-47 · *API:* none — client-side aggregation · *Screen:* §5.1
> *Depends:* US-M3.1, US-UI.4

- `T-M4.4-1` — Members-by-slab chart: one bar per slab row, count on that slab out of the total. — 0.5d
- `T-M4.4-2` — ⚠️ Rewards-by-slab chart directly below it, **reusing the same bar-list component verbatim** — same track, fill, label and shapes, summing Rewards (all three components) instead of counting members. **No new chart component.** A combined dual-value chart was offered and rejected: two familiar charts read faster at a glance than one denser one. — 0.5d
- `T-M4.4-3` — Label mirrors the sibling exactly: `<slab's Rewards total> / <Rewards total across all members>`. Current live period only. — 0.25d
- `T-M4.4-4` — ⚠️ Client-side aggregation of the same not-yet-closed `member_period_totals` rows the sibling chart already reads. **No new IPC command** — matching the existing pattern, since neither chart has a dedicated one. — 0.25d

## Feature M4.2 — Full hierarchy window

*New 7 Aug 2026 (CR-3). The client's binding constraint: the main console must not be slowed down.*

> **US-M4.3 — Full hierarchy window.** `S9` · 7.0d
> *Refs:* FR-10, Rule-45, UN-31, V4.5 · *AC:* AC-44, AC-45 · *API:* API-11 (`full_tree: true`) · *Screen:* §5.3a
> *Depends:* US-M4.2, US-M3.1

- `T-M4.3-1` — ⚠️ `get_direct_children_chart` with `full_tree: true` — **no new command**; the parameter was always in the contract. Both the gate's count and the draw are cheap local reads: the cost of this feature is rendering, not fetching, which is exactly why the rendering happens elsewhere. — 0.5d
- `T-M4.3-2` — ⚠️ A **separate top-level window** — not a modal, not a route. Its own root, its own render, sharing only the node component and design tokens with the main app, **never live state**. — 1.0d
- `T-M4.3-3` — ⚠️ Rooted **always at the top member**, whatever the Structure screen is currently rooted at. The client's explicit choice; a second action for "root here" was offered and declined. — 0.25d
- `T-M4.3-4` — ⚠️ **Single post-order layout pass** placing every node and emitting connectors as one pre-computed path — **never measured back out of the rendered DOM** as the main Structure screen does. This is what keeps the draw off the main thread's critical path. — 1.5d
- `T-M4.3-5` — ⚠️ Size gate above **60 descendants**: a confirmation naming the **exact** count — never an estimate, never rounded. **Cancel opens nothing at all.** At or below 60 it opens immediately with no confirmation. — 0.5d
- `T-M4.3-6` — Header: top member's name, total member count, and an **"as at &lt;date, time&gt;"** stamp. ⚠️ The stamp is what makes a printed copy honest about when it was true, and it must survive printing. — 0.25d
- `T-M4.3-7` — Toolbar: zoom **10%–150%** (the floor drops far below the main chart's 50% because a whole network must be takeable-in at once), fit-width, in-window search with a 2px indigo ring and scroll-to-centre, and Print. — 1.0d
- `T-M4.3-8` — ⚠️ Print stylesheet: drops the toolbar, keeps header and stamp on the first page, lets the chart break across pages. **Do not scale a wide chart to fit one page** — that makes the node text unreadable. — 0.5d
- `T-M4.3-9` — ⚠️ **Read-only, visibly.** No node links, no writes, and specifically **no `translateY(-1px)` hover lift** — that affordance means "opens a branch", and there are no branches left to open. The absence of affordances is the design. — 0.25d
- `T-M4.3-10` — ⚠️ Point-in-time: subscribes to nothing, polls nothing, holds no handle on live state. It does **not** update when a figure is recorded in the console, and does not follow later theme changes. This is Rule-45's defining property, not an oversight — do not add live refresh. — 0.25d
- `T-M4.3-11` — Empty case: the top member with nobody beneath shows the single root node and states plainly there is nothing beneath it — **not an error**. — 0.25d
- `T-M4.3-12` — ⚠️ TEST-R45 and the performance gate: identical geometry across two runs on identical input; single-node and single-chain structures lay out correctly; **main console measurably responsive while the window draws** at 25,000 members. *Known accepted limit TR-7 — the top-down layout's width grows with leaf count. Chosen deliberately over a width-stable indented outline. **Do not switch layouts unilaterally**; raise it as a change request.* — 0.75d

---

# Epic M7 — Settings

## Feature M7.1 — Configuration

> **US-M7.1 — Edit the slab table.** `S10` · 3.25d
> *Refs:* Rule-4, Rule-27, Rule-41 · *AC:* AC-32, AC-33 · *API:* API-23, API-24, API-25 · *Screen:* §5.7
> *Depends:* US-M3.1

- `T-M7.1-1` — `add_slab_row` / `update_slab_row` / `remove_slab_row` (API-23/25/24), each recalculating the current open period only. — 1.0d
- `T-M7.1-2` — Duplicate-threshold guard: **refused outright, before any warning is offered**. — 0.25d
- `T-M7.1-3` — ⚠️ **No monotonicity validation** (Rule-41, ADR-009). The safeguard was recommended and **explicitly declined by the client**, who accepts the residual risk of a silently negative reward. A static on-screen disclaimer stands in for a code guard. Do not "fix" this. — 0.25d
- `T-M7.1-4` — ⚠️ Last remaining row cannot be removed (LOW-2, built in the prototype): control `disabled` with an explanatory `aria-label` and an on-screen hint, **and** the handler refuses with a named message if reached another way. — 0.5d
- `T-M7.1-5` — Slab table section UI. — 0.5d
- `T-M7.1-6` — ⚠️ A **deliberate negative test**: confirm a non-monotonic table save is **not** blocked and the resulting (possibly negative) differential computes and displays **as-is, not silently clamped**. This proves the accepted risk is implemented as "no validation", not accidentally half-implemented. — 0.5d
- `T-M7.1-7` — Field-level validation: **V7.1** thresholds must be positive numbers; **V7.2** percentages must be between 0 and 100. ⚠️ These are *per-field* range checks and are **not** the cross-row monotonicity check Rule-41 forbids — refusing a percentage of 400 is not the same as refusing a table whose percentages fall as thresholds rise. — 0.25d

> **US-M7.2 — Royalty, structure guidance, reporting settings, session timeout.** `S10` · 3.25d
> *Refs:* `02-business-rules.md` §6 (all 16 settings) · *AC:* AC-31 · *API:* API-21, API-22
> *Depends:* US-0.2

- `T-M7.2-1` — `get_settings` / `update_settings` (API-21/22) across all 16 rows. ⚠️ **16, not 13** (C1). — 0.75d
- `T-M7.2-2` — Royalty section: qualifying-child count, royalty rate — recalculating the current open period on change. — 0.5d
- `T-M7.2-3` — Structure guidance section: hierarchy depth (default **4**, D-3) and level 2/3/4 widths, **explicitly labelled as guidance, never enforced**. — 0.5d
- `T-M7.2-4` — Reporting section: yearly cycle bounds, low-contribution threshold, default export columns. — 0.5d
- `T-M7.2-5` — Reference unit value: labelled per RQ-12, **display-only, read by nothing else in the system**, and the only place any conversion figure appears. — 0.25d
- `T-M7.2-6` — Access section: session inactivity timeout (default **15 minutes**, D-4). — 0.25d
- `T-M7.2-7` — ⚠️ AC-31 check: every setting editable by the client **unaided** — this is SC-6, measured during acceptance. — 0.25d
- `T-M7.2-8` — Field-level validation: **V7.4** the royalty qualifying count must be a positive whole number; **V6.3** the low-contribution threshold must be a positive number. — 0.25d

> **US-M7.3 — Mid-period recalculation warning.** `S11` · 4.0d *(approved reference behaviour — port it)*
> *Refs:* RQ-18, V7.6 · *API:* API-33 · *Screen:* §5.7
> *Depends:* US-M3.1, US-UI.4

- `T-M7.3-1` — ⚠️ `preview_settings_impact` (API-33) — **build this before the UI**. The prototype hid the need because everything there ran in one JavaScript scope; the real engine is Rust-side and the frontend cannot dry-run it. — 1.5d
- `T-M7.3-2` — ⚠️ The dry run reuses the live Total Business Volume (slab and royalty settings never feed the rollup) and re-runs the Rewards computation alone against temporarily-swapped settings, **restored in a `finally` block**. A panic must never leave live settings holding uncommitted values. — 0.5d
- `T-M7.3-3` — Variant-C warning modal: names the open month, states closed months are unaffected, shows Rewards before → after via the impact-summary component, and lists the members actually affected. — 1.0d
- `T-M7.3-4` — ⚠️ Fires on **Slab table and Royalty saves only**. The other three sections change nothing already calculated and still save silently. On a royalty save no member's slab can move, so the list shows who **starts or stops earning royalty**, with a "Members earning royalty: before → after" row. — 0.5d
- `T-M7.3-5` — ⚠️ Cancel is a **true no-op** — nothing saved, and the admin's typed values remain exactly as they were. — 0.25d
- `T-M7.3-6` — ⚠️ **The preview must equal what actually lands** (MEDIUM-1): capture the predicted figure, confirm the save, re-open the warning with no further edits, and assert the settled "before" equals the earlier prediction exactly. A preview that can disagree with reality is worse than no preview, because the admin approved a change on a number that was never true. Plus: API-33 leaves nothing behind, tested including the restore-on-panic path. — 0.25d

> **US-M7.4 — Whole-console backup schedule and retention.** `S10` · 3.5d *(approved reference behaviour — port it)*
> *Refs:* Rule-43, RQ-23 · *AC:* AC-37 · *API:* API-37, API-38 · *Screen:* §5.7
> *Depends:* T-0.2-3 (`backups` generalization)

- `T-M7.4-1` — `get_console_backup_settings` / `update_console_backup_settings` (API-37/38): schedule ∈ {off, daily, weekly, monthly}, retention ≥ 1. — 1.0d
- `T-M7.4-2` — Backup schedule card: segmented control saving **immediately, with no separate Save step**, matching the segmented-control pattern used elsewhere. — 0.75d
- `T-M7.4-3` — ⚠️ Retention count takes effect **on the next prune, not immediately** — existing excess backups are pruned then. — 0.25d
- `T-M7.4-4` — "Back up now" action producing a `manual` backup that appears at the top of the Restore card's list. — 0.5d
- `T-M7.4-5` — Restore card listing retained backups of **every** kind, via the shared restore-option-list component — `period_close` labelled by the month it holds, `scheduled`/`manual` by when it was taken. Plus a "Restore from a file…" action. — 0.75d
- `T-M7.4-6` — ⚠️ Restore confirmation reuses the **month-close wizard's checklist pattern exactly** — a `.modal-warn` note naming what will be replaced, one checkbox, Cancel first, then a disabled-until-checked danger action. No new confirmation pattern: this action earns the same weight already given to closing a month, not more and not less. — 0.25d

---

# Epic M5 — Monthly Close

*Carries the project's highest safety requirement — the backup gate.*

## Feature M5.1 — Gated close flow

> **US-M5.1 — Close the oldest outstanding month.** `S11` · 7.0d
> *Refs:* Rule-17, Rule-18, Rule-20, Rule-21, Rule-38 · *AC:* AC-21–AC-25 · *API:* API-12, API-13, API-14, API-15 · *Screen:* §5.6
> *Depends:* US-M3.2, US-M2.1

- `T-M5.1-1` — `get_outstanding_periods` (API-12) and `begin_close` (API-13). ⚠️ **Only the oldest outstanding period may begin** — closing out of order is rejected, and months are never merged into a combined period. — 0.75d
- `T-M5.1-2` — ⚠️ Backup generation and **verification** — exists, checksum matches, readable — as a genuine transactional precondition. — 1.0d
- `T-M5.1-3` — ⚠️ `confirm_backup_and_close` (API-14) in one transaction, strictly ordered: **write and verify backup → write snapshots → zero live figures → mark period closed. A verification failure never begins the zeroing phase.** — 1.5d
- `T-M5.1-4` — ⚠️ Snapshot written for **every** member before anything is cleared, carrying the slab table, royalty rate and qualifying count **in force that month** (RQ-5) — otherwise a past month cannot be re-derived from it. — 0.75d
- `T-M5.1-5` — ⚠️ Zeroing covers **everything** — Business Volume, Total Business Volume, Rewards, royalty (Rule-38). This differed from the architect's recommendation to keep Rewards live, and it is what makes the backup gate load-bearing. — 0.5d
- `T-M5.1-6` — ⚠️ Failure and cancellation paths: **abort entirely — nothing zeroed, nothing touched**, the alert stays up, the admin may retry. Includes disk-full during the backup write, which fails verification like any other failure. — 0.5d
- `T-M5.1-7` — ⚠️ External-medium copy: prompted and reminded, **never blocking**. The internal retained copy is the real gate (RQ-6, RQ-19). Forcing the close to block on the external copy would contradict a documented decision; the residual single-medium risk (TR-4) is stated, not solved. — 0.25d
- `T-M5.1-8` — `manual_backup_current_period` (API-15): on-demand backup of the in-progress month, no zeroing, same write-verify mechanism. — 0.25d
- `T-M5.1-9` — Close wizard UI: confirmation step naming the month unambiguously, backup step, commit step reachable only after a verified backup, completion. — 1.0d
- `T-M5.1-10` — ⚠️ Integration test for close atomicity: simulate a backup-verification failure mid-close and confirm **zero data is mutated** — no partial zeroing, no orphaned snapshot row. — 0.5d

> **US-M5.2 — Persistent outstanding-month alert.** `S12` · 2.0d
> *Refs:* Rule-20 · *AC:* AC-18, AC-20 · *API:* API-31 · *Screen:* §5.11
> *Depends:* US-M5.1, US-M5.5, US-UI.2

- `T-M5.2-1` — `get_outstanding_alert` (API-31) plus period-lifecycle transition to `awaiting_close` when a calendar month ends. — 0.5d
- `T-M5.2-2` — ⚠️ Banner on **every** screen, naming the month(s), stating that entries dated in the outstanding month can still be recorded, and naming the month that unlocks on close. **Undismissable — no close icon, no auto-hide, no acknowledge, not even a disguised dismissal.** — 0.5d
- `T-M5.2-3` — Matching persistent notification-list entry — not a dismissable toast. — 0.25d
- `T-M5.2-4` — ⚠️ Clears **only** on a completed close — not on navigation, logout or acknowledgement (AC-20). — 0.25d
- `T-M5.2-5` — Multiple outstanding months all listed, oldest first, only the oldest closable. — 0.25d
- `T-M5.2-6` — E2E test asserting the absence of any dismissal route. — 0.25d

> **US-M5.3 — Entry eligibility by period (M5's side of the contract).** `S12` · 2.0d
> *Refs:* Rule-36 (amended) · *API:* API-07
> *Depends:* US-M5.2

- `T-M5.3-1` — Publish which periods are recordable via `get_period_lock_status`, and release the current month on close. — 1.0d
- `T-M5.3-2` — Period state machine: `open` → `awaiting_close` → `closed`. ⚠️ The value is **`awaiting_close`**; `ended_locked` described a total lock Rule-36 no longer imposes and would make the schema state the opposite of the behaviour. — 0.5d
- `T-M5.3-3` — ⚠️ Integration test: with the close outstanding, entries dated in that month are accepted and current-month entries refused naming it; after the close, the current-month entry is accepted. *Superseded criterion, do not implement: "no entry of any kind is accepted until the close completes."* — 0.5d

> **US-M5.4 — Empty-month handling.** `S13` · 1.5d
> *Refs:* RQ-16, Rule-23 · *API:* API-12, API-14
> *Depends:* US-M5.1, US-M5.5

- `T-M5.4-1` — ⚠️ A calendar month elapsing with **zero entries produces no snapshot at all** — not a zero snapshot. — 0.5d
- `T-M5.4-2` — Excluded from the yearly-averaging denominator, and not offered as a closed-month export option. — 0.5d
- `T-M5.4-3` — Test. ⚠️ Still required after CR-2, which makes an empty month **less likely but not impossible** — a month awaiting close now keeps accepting entries, so it no longer goes empty merely because an earlier month was outstanding. — 0.5d

> **US-M5.5 — Period lifecycle catch-up.** `S12` · 2.5d *(new 8 Aug 2026, D-9/D-10)*
> *Refs:* Rule-17, Rule-20, Rule-21, Rule-36, RQ-16 · *AC:* AC-18, AC-21 · *API:* API-07, API-12 · *State machine:* `04-technical-architecture.md` §7.1
> *Depends:* US-M5.1
> *Blocks:* US-M5.2, US-M5.3, US-M5.4

*Closes a genuine gap in the source specification, not in the plan. `04-technical-architecture.md` §7.1 says a period row is created "as soon as the calendar month begins"; `05-data-model-specification.md` says "implicitly when the first entry is recorded, **or** explicitly at month-start". Neither names the code that runs, and there is no background service. Without this story, `open → awaiting_close` never fires and Rule-20's alert never appears.*

- `T-M5.5-1` — ⚠️ **Catch-up routine, run once at successful login** — the only point the application is reliably running, exactly as the whole-console backup schedule is (Rule-43). **This is a design constraint, not a gap to "fix" with a background timer.** — 0.75d
- `T-M5.5-2` — ⚠️ Create a period row (`open`) for every calendar month up to and including the current one that has no row yet. **Must handle the application being unopened across several month boundaries** — every intervening month gets a row, none is skipped. — 0.5d
- `T-M5.5-3` — Transition every `open` period whose calendar month has elapsed to **`awaiting_close`**, setting `ended_at`. Multiple periods may sit at `awaiting_close` simultaneously; each accepts entries, and only the oldest is closable. — 0.5d
- `T-M5.5-4` — ⚠️ **Ordering inside login**: catch-up runs **before** the backup-schedule check and **before** the UI takes over, so the outstanding-month banner and entry eligibility are already correct on the first frame the operator sees. — 0.25d
- `T-M5.5-5` — ⚠️ **Empty-month path (D-10):** a month with zero entries is treated exactly like any other — it gets its row, raises the undismissable alert, and goes through the full close wizard **including the backup gate** — but **writes no snapshot** (RQ-16) and stays out of the yearly-average denominator. The backup is a whole-database copy, so it is a real restore point regardless of that month holding nothing. — 0.25d
- `T-M5.5-6` — ⚠️ Test: with the application unopened across three month boundaries, confirm all three periods are created, all three sit at `awaiting_close`, they are queued **oldest-first**, only the oldest is closable, and every one of them accepts entries dated within itself. Plus the idempotency case — logging in twice in a row creates nothing the second time. — 0.25d

---

# Epic M6 — Reports & Exports

## Feature M6.1 — Exports

> **US-M6.5 — Mandatory export column set.** `S13` · 1.0d *(new 8 Aug 2026, D-1)*
> *Refs:* Rule-19, Rule-33 · *AC:* AC-29 · *Depends:* US-0.2
> *Blocks:* US-M6.1

- `T-M6.5-1` — ⚠️ Implement the **five** mandatory columns per D-1: name, member number, phone, Business Volume, **Total Business Volume**. All five are untickable on all three extracts. — 0.5d
- `T-M6.5-2` — Remove Total Business Volume from the optional column list and change the seeded default column set from four entries to five. — 0.25d
- `T-M6.5-3` — Execute the propagation task `P-1` in [05-decisions-and-gaps.md](05-decisions-and-gaps.md) — amend V6.1, Rule-33 and US-M6.1's acceptance criteria in `documents/refinement/`, and move O1 from `06-decision-log` §3 to §2. — 0.25d

> **US-M6.1 — Monthly data export.** `S13` · 3.0d
> *Refs:* Rule-19, Rule-33, UN-22 · *AC:* AC-26, AC-29, AC-30 · *API:* API-16 · *Screen:* §5.8
> *Depends:* US-M5.1, US-M6.5

- `T-M6.1-1` — `export_monthly` (API-16) via `rust_xlsxwriter`. ⚠️ **Rust-side only** — the WebView never touches raw file content or paths (ADR-007). — 1.0d
- `T-M6.1-2` — ⚠️ The five mandatory columns are present regardless of selection; an empty optional selection is **not an error**. — 0.25d
- `T-M6.1-3` — Column picker with the full optional list from Rule-33, including active/inactive status (INC-5). — 0.5d
- `T-M6.1-4` — ⚠️ A closed month's extract reads from the **permanent snapshot** (RQ-4), never live values — which would return zeros. — 0.5d
- `T-M6.1-5` — ⚠️ Filenames name the **period, never a member** (NFR-4), and pass the vocabulary grep. — 0.25d
- `T-M6.1-6` — ⚠️ Every export includes deactivated members, in the same distinct colour plus the textual status column (M6.5, Rule-42). **No "active only" filter exists.** — 0.25d
- `T-M6.1-7` — Test: opens correctly in a standard spreadsheet application (AC-30). — 0.25d

> **US-M6.2 — Yearly average export.** `S13` · 2.0d
> *Refs:* Rule-23, UN-23 · *AC:* AC-27 · *API:* API-17
> *Depends:* US-M6.1

- `T-M6.2-1` — ⚠️ `export_yearly_average` (API-17): the divisor is the count of periods that **have a snapshot** — never a fixed 12. This protects late joiners. — 0.75d
- `T-M6.2-2` — ⚠️ The count is **displayed alongside every average**, not just used. — 0.25d
- `T-M6.2-3` — Extract carries both Total Business Volume and own-Business-Volume averages, each with its month count. — 0.5d
- `T-M6.2-4` — ⚠️ Reads snapshots at `MAX(version)`, so a corrected month is reflected. — 0.25d
- `T-M6.2-5` — Test including empty-month exclusion. — 0.25d

> **US-M6.3 — Low-contribution report.** `S13` · 1.5d
> *Refs:* Rule-24, UN-24 · *AC:* AC-28 · *API:* API-18
> *Depends:* US-M6.2

- `T-M6.3-1` — ⚠️ `export_low_contribution` (API-18) filtering on the yearly average of **own** Business Volume, **not Total Business Volume**. The client's answer differed from the architect's recommendation and was deliberately re-confirmed. — 0.75d
- `T-M6.3-2` — Threshold from settings (default 100), overridable per run. — 0.25d
- `T-M6.3-3` — Empty result shown as a plain empty state, **not an error**. — 0.25d
- `T-M6.3-4` — Test. — 0.25d

> **US-M6.4 — Closed-month snapshot re-download.** `S13` · 1.5d
> *Refs:* Rule-31, Rule-39 · *AC:* AC-25 · *API:* API-19, API-20
> *Depends:* US-M6.1

- `T-M6.4-1` — `list_backups` (API-19) and `redownload_backup` (API-20). ⚠️ This is the command behind the prototype's "Closed month snapshot" card — **no separate export command exists** (HIGH-1). — 0.75d
- `T-M6.4-2` — ⚠️ Always returns the **latest** version for a corrected month; the original stays in the audit trail, not in the export. — 0.25d
- `T-M6.4-3` — Reports-screen card UI. — 0.25d
- `T-M6.4-4` — Test with a multi-version closed month. — 0.25d

---

# Epic M9 — Audit Log

*Cross-cutting. **Wired into every mutating command as that command is built**, from S2 onward. This story is the completeness check, not the first time audit logging is written.*

## Feature M9.1 — Change history

> **US-M9.1 — Record and display audit entries.** `S14` · 3.5d
> *Refs:* NFR-5, NFR-11 · *API:* API-32 · *Screen:* §5.9
> *Depends:* every mutating command

- `T-M9.1-1` — `audit_log` write helper: entity, field, value before, value after, timestamp, cause. Append-only — no entry is ever edited or removed. — 0.75d
- `T-M9.1-2` — ⚠️ Completeness audit across every mutating command: each produces exactly one entry — or one per changed field for `record_entry` / `edit_entry` — and **every read-only command correctly produces none**. — 1.0d
- `T-M9.1-3` — Cause taxonomy: `entry`, `edit`, `correction`, `settings_change`, `period_close`, `manual_backup`, `console_backup`. ⚠️ The `reversal` value is unused — `reverse_entry` was dropped. Retire it unless the client specifically wants the word preserved in the log. — 0.25d
- `T-M9.1-4` — Audit screen: chronological, read-only, filterable by member name, ID or phone. — 0.75d
- `T-M9.1-5` — ⚠️ Technical logging as a **separate rotating file with no UI surface** (NFR-11) — distinct from the audit log and never visible to the client. — 0.5d
- `T-M9.1-6` — ⚠️ Security check: no plaintext credential is ever written to either log. — 0.25d

---

# Epic E-REL — Release, Deployment & Handover

*New epic. No packaging, signing, verification, UAT-execution, handover or support work exists anywhere in the source set.*

## Feature REL.1 — Build and release engineering

> **US-REL.1 — Versioning scheme and build configuration.** `S1` · 1.0d
> *Refs:* ADR-011 · *Depends:* US-0.1

- `T-REL.1-1` — Semantic version scheme, single source of truth across `Cargo.toml`, `package.json` and `tauri.conf.json`. — 0.5d
- `T-REL.1-2` — Release/debug build profiles; strip debug symbols from release. — 0.25d
- `T-REL.1-3` — ⚠️ Confirm **no auto-update mechanism exists** (ADR-011) — it would require exactly the network capability the offline requirement forbids. Upgrades are a new installer, run manually, with the maintainer notifying the client. — 0.25d

> **US-REL.2 — Local pre-release gate.** `S15` · 2.5d
> *Refs:* D-7, `05-quality-and-acceptance.md` §6.1/§6.3 · *Depends:* US-QA.4

- `T-REL.2-1` — ⚠️ Single scripted gate replacing CI (D-7): `clippy` + ESLint (no new warnings) → `cargo audit` + `npm audit` → full unit/integration/contract suite → E2E suite → vocabulary grep → both-platform build. **All must pass; the script exits non-zero on any failure.** — 1.5d
- `T-REL.2-2` — ⚠️ Dependency vulnerability check as a **hard gate, not advisory** — a compromised dependency here sits inside the same process as the encryption key. — 0.25d
- `T-REL.2-3` — Golden-scenario re-verification as an explicit named step, so a moved total is impossible to miss in the output. — 0.25d
- `T-REL.2-4` — ⚠️ Record the CI omission as a **dated deviation** from `05-quality-and-acceptance.md` §6.3 in [05-decisions-and-gaps.md](05-decisions-and-gaps.md), rather than treating CI's absence as satisfaction — which §6.3 explicitly warns against. — 0.5d

## Feature REL.2 — Packaging and signing

> **US-REL.3 — Windows packaging and signing.** `S15` · 2.5d
> *Refs:* `04-technical-architecture.md` §10, D-6 · *Depends:* US-REL.1

- `T-REL.3-1` — Windows `.msi`/`.exe` bundle via the native Tauri bundler; verify the ~10–20MB footprint with no bundled browser runtime. — 0.75d
- `T-REL.3-2` — ⚠️ Generate a **self-signed code-signing certificate** and sign the installer (D-6). Windows Authenticode from a trusted CA costs money and, since June 2023, requires the private key on FIPS 140-2 L2 hardware — deferred. — 0.5d
- `T-REL.3-3` — ⚠️ Runbook: install the certificate into the client's **Trusted Root / Trusted Publishers** store, once, on the single target machine. This is what removes the "unknown publisher" dialog. **Ceiling: the certificate is trusted only on machines where it is installed** — this is not a general distribution solution, and it does not need to be, since there is exactly one machine. — 0.5d
- `T-REL.3-4` — ⚠️ Deliver the installer on physical media, not via download — a USB-copied file carries no Mark-of-the-Web, so SmartScreen never engages. — 0.25d
- `T-REL.3-5` — Application icon and window mark. ⚠️ A plain visual mark only — **no company name, no commercial branding anywhere** (BC-4). — 0.5d

> **US-REL.4 — macOS packaging.** `S15` · 1.5d
> *Refs:* `04-technical-architecture.md` §10, D-6 · *Depends:* US-REL.1

- `T-REL.4-1` — macOS `.dmg`/`.app` bundle. — 0.75d
- `T-REL.4-2` — ⚠️ Ships **unsigned and un-notarized** (D-6, deviation from §10). Notarization requires a paid Apple Developer Program membership, deferred with the rest of the paid-certificate decision. — 0.25d
- `T-REL.4-3` — Documented one-time Gatekeeper first-open step for the client, written for someone who will not read documentation — screenshots, not prose. — 0.5d

> **US-REL.5 — Installer verification on clean machines.** `S15` · 2.0d
> *Refs:* DoD §6.1 item 14 · *Depends:* US-REL.3, US-REL.4

- `T-REL.5-1` — Windows clean-machine install: first-run setup wizard completes, root member created, first entry recorded, figures visible. — 0.75d
- `T-REL.5-2` — macOS clean-machine install, same path, including the Gatekeeper step. — 0.5d
- `T-REL.5-3` — ⚠️ Cross-device restore verification (AC-38): install on a **second, different machine**, restore from a backup file, and confirm it reaches exactly the state the original held — **no separate setup step, the same login credential working unchanged**. — 0.5d
- `T-REL.5-4` — Run the macOS manual verification checklist (`T-QA.3-3`) in full. — 0.25d

## Feature REL.3 — Acceptance and handover

> **US-REL.6 — UAT execution.** `S16` · 3.0d
> *Refs:* SC-1–SC-8, AC-1–AC-47 · *Depends:* US-REL.5

- `T-REL.6-1` — ⚠️ **The single most important acceptance gate in the project**: the client re-runs all six worked scenarios **through the actual built UI**, not the engine in isolation, and confirms the on-screen figures match their own hand-worked numbers (SC-2, R-9). — 1.0d
- `T-REL.6-2` — Walk AC-1 to AC-47 with the client, recording pass/fail per criterion. — 1.0d
- `T-REL.6-3` — SC-5 measurement: **time** recording a figure for a known member; the target is under 15 seconds. — 0.25d
- `T-REL.6-4` — ⚠️ SC-6 measurement: the client changes a scheme setting themselves, **unaided**, once, during acceptance. — 0.25d
- `T-REL.6-5` — SC-7: full review of every screen, message and extract filename for excluded vocabulary — the human pass alongside the automated grep. — 0.25d
- `T-REL.6-6` — Defect log with triage against the convention in [03-test-plan.md](03-test-plan.md) §7. — 0.25d

> **US-REL.7 — Handover pack and client training.** `S16` · 3.0d
> *Refs:* `01-product-and-scope.md` §12 · *Depends:* US-REL.6

- `T-REL.7-1` — Handover pack mapped one-to-one against the five promised deliverables: the installable desktop application; three working extracts; backups verified before anything is cleared; a working recovery-code path; an audit log that can explain any figure. — 0.75d
- `T-REL.7-2` — ⚠️ Training session on the monthly close, walking the backup gate explicitly — the client must understand that a failed backup **aborts the close entirely and nothing is lost**. — 0.5d
- `T-REL.7-3` — ⚠️ Training on the **external-medium backup discipline**. TR-4 leaves this unenforced by design, and it is the single point of failure the software does not defend against. It is a client process, and it must be taught as one. — 0.5d
- `T-REL.7-4` — ⚠️ Recovery-code custody: where the codes live, and the plain statement that **losing both the credential and the codes is permanently unrecoverable** — no vendor backdoor, no email flow. Delivered at handover, not buried in settings. — 0.25d
- `T-REL.7-5` — Training on settings self-service (SC-6) and on restore, including that a restore always names what it replaces and takes a safety backup first. — 0.5d
- `T-REL.7-6` — Maintainer runbook: how to build, sign and deliver an upgrade installer, given there is no auto-update. — 0.5d

> **US-REL.8 — Hypercare.** `S16` · 1.5d
> *Refs:* SC-1, SC-4 · *Depends:* US-REL.7

- `T-REL.8-1` — ⚠️ Support the client's **first live monthly close** end to end — the first time the irreversible action runs against real data. — 0.5d
- `T-REL.8-2` — Spot-check figures against hand calculations during the first live month (SC-2's second half). — 0.5d
- `T-REL.8-3` — ⚠️ Support window covering SC-1's **three-month** measurement period, after which the client confirms they no longer calculate by hand. — 0.25d
- `T-REL.8-4` — SC-4 check: every month since go-live holds a permanent record **and** a retained backup. — 0.25d

---

## Dependency exceptions worth restating

Four sequencing constraints that are **not optional** — a story cannot start before what it depends on is Done (`05-quality-and-acceptance.md` §6.1):

1. **`T-0.2-3` (the `backups` generalization) lands in S1.** US-M7.4, US-M8.5 and US-M8.6 all block on it. It is a schema decision, not a feature — there is no reason to defer it to the sprint that consumes it.
2. **Feature M2.2 (US-M2.3/M2.4/M2.5) ships with Epic M5, not with Epic M2.** It cannot be built or tested before the outstanding-period state exists.
3. **`T-M7.3-1` (`preview_settings_impact`) is built before the warning UI**, not alongside it. The prototype hid this dependency by running everything in one JavaScript scope.
4. **Audit logging is wired as each mutating command is built**, from S4. US-M9.1 in S14 is a completeness check.
5. **`US-M5.5` (period lifecycle catch-up) precedes US-M5.2, US-M5.3 and US-M5.4.** Without it no period ever reaches `awaiting_close`, so the outstanding-month alert has no state to read and entry eligibility has nothing to gate on. It is the state machine those three stories assume already exists.

# Traceability — Requirement → Work Item → Verification

`documents/implementation-readiness/02-requirements-traceability-matrix.md` already closes requirement → design → API → database. **This file closes the other half — requirement → work item → sprint** — which is what makes the module-level Definition of Done checkable: *"every rule attributed to that module shows a passing test, not just 'documented'."*

**Orphan check result: 0 orphans across eleven namespaces.** Every Rule-1–46 (plus Rule-16a), FR-1–10, UN-01–31, NFR-1–16, AC-1–47, API-01–40, **V1.1–V8.5, M1.1–M8.7, RQ-1–23** and every one of the **63 error/edge scenarios** maps to at least one task. Four items map to a deliberate non-task and are marked **‡** — each is a recorded client decision that nothing be built.

⚠️ **The last four namespaces were added by the pre-development audit of 8 August 2026.** Tracing them is what surfaced D-9 (nothing triggered the period lifecycle), D-11 (two documents contradicting each other on slab lookup order), and D-12/D-13 (audit enums that could not represent half the events requiring an audit entry). A namespace nobody traces is where a defect survives a full specification review.

---

## 1. Business rules → stories

| Rule | Story | Key task | Sprint |
|---|---|---|---|
| Rule-1 — level widths advisory | US-M1.1, US-M7.2 | T-M1.1-6, T-M7.2-3 | S4, S10 |
| Rule-2 — unique 6-digit member ID | US-M1.1 | T-M1.1-1 | S4 |
| Rule-3 — slab lookup | US-M3.1 | T-M3.1-1 | S6 |
| Rule-4 — slab thresholds/percentages configurable | US-M7.1 | T-M7.1-1 | S10 |
| Rule-5 — bottom-up calculation order | US-M3.1 | T-M3.1-2 | S6 |
| Rule-6 — Total Business Volume formula | US-M3.1 | T-M3.1-2 | S6 |
| Rule-7 — slab driven by Total Business Volume | US-M3.1 | T-M3.1-1 | S6 |
| Rule-8 — differential earnings | US-M3.1 | T-M3.1-3 | S6 |
| Rule-9 — differential never negative | US-M3.1, US-QA.1 | T-M3.1-8, T-QA.1-4 | S4, S6 |
| Rule-10 — royalty qualification | US-M3.1 | T-M3.1-4 | S6 |
| Rule-11 — royalty and differential never double-pay | US-M3.1, US-M4.1 | T-M3.1-6, T-M4.1-3 | S6, S8 |
| Rule-12 — Rewards *(amended, CR-4)* | US-M3.1 | T-M3.1-5 | S6 |
| Rule-13 — Rewards are a separate ledger | US-M3.1 | T-M3.1-6 | S6 |
| Rule-14 — unit value, reference only | US-M7.2 | T-M7.2-5 | S10 |
| Rule-15 — Business Volume entry flow | US-M2.1 | T-M2.1-3 | S7 |
| Rule-16 — points-only entry, 2 decimals | US-M2.1 | T-M2.1-2 | S7 |
| Rule-16a — zero **and** negative refused | US-M2.1 | T-M2.1-2 | S7 |
| Rule-17 — manual reset only | US-M5.1 | T-M5.1-1 | S11 |
| Rule-18 — reset gated by backup | US-M5.1 | T-M5.1-2, -3, -6, -10 | S11 |
| Rule-19 — every export carries basic fields | US-M6.5, US-M6.1 | T-M6.5-1, T-M6.1-2 | S13 |
| Rule-20 — persistent reset alert | US-M5.2 | T-M5.2-2, -4 | S12 |
| Rule-21 — period boundaries | US-M2.1, US-M5.1 | T-M2.1-1 | S7, S11 |
| Rule-22 — precision | US-0.2, US-M3.1 | T-0.2-6, T-M3.1-7 | S1, S6 |
| Rule-23 — yearly average method | US-M6.2 | T-M6.2-1, -2 | S13 |
| Rule-24 — low-threshold report metric | US-M6.3 | T-M6.3-1 | S13 |
| Rule-25 — royalty stacks at every level | US-M3.1 | T-M3.1-4 | S6 |
| Rule-26 — recalculation trigger | US-M3.2, US-M2.1 | T-M3.2-1, T-M2.1-5 | S6, S7 |
| Rule-27 — slab rows addable/removable | US-M7.1 | T-M7.1-1, -4 | S10 |
| Rule-28 — member lifecycle *(corrected, C5)* | US-M1.3 | **T-M1.3-1, -5** | S5 |
| Rule-29 — access control | US-M8.1, US-M8.2, US-M8.4 | T-M8.1-1, T-M8.2-2 | S5, S8 |
| Rule-30 — reference and hierarchy integrity | US-M1.1 | T-M1.1-2, -3 | S4 |
| Rule-31 — backup storage and retention | US-M5.1, US-M6.4 | T-M5.1-7, T-M6.4-1 | S11, S13 |
| Rule-32 — depth overflow | US-M1.1 | T-M1.1-6 | S4 |
| Rule-33 — configurable export columns | US-M6.5, US-M6.1 | T-M6.5-2, T-M6.1-3 | S13 |
| Rule-34 — phone number uniqueness | US-M1.1 | T-M1.1-4 | S4 |
| Rule-35 — member ID allocation | US-M1.1 | T-M1.1-1 | S4 |
| Rule-36 — reset enforcement *(amended, CR-2)* | US-M2.3, US-M2.4, US-M5.3 | **T-M2.4-5** | S12 |
| Rule-37 — transfers prohibited | US-M1.2 | T-M1.2-1 | S5 |
| Rule-38 — reset scope | US-M5.1 | T-M5.1-4, -5 | S11 |
| Rule-39 — entries editable, incl. closed months | US-M2.2 | T-M2.2-2, -5 | S7 |
| Rule-40 — consent capture | US-M1.1 | T-M1.1-5 | S4 |
| Rule-41 — monotonicity **not** validated | US-M7.1 | T-M7.1-3, **T-M7.1-6** *(negative test)* | S10 |
| Rule-42 — members never removed | US-M1.3, US-M6.1 | T-M1.3-6, T-M6.1-6 | S5, S13 |
| Rule-43 — whole-console backup & restore | US-M7.4, US-M8.5, US-M8.6 | T-M7.4-1, T-M8.5-1, T-M8.6-2 | S10, S14 |
| Rule-44 — phone number is a search key | US-M1.4 | T-M1.4-2, **T-M1.4-6** | S5 |
| Rule-45 — full hierarchy point-in-time draw | US-M4.3 | T-M4.3-5, -10, **-12** | S9 |
| Rule-46 — reward on own Business Volume | US-M3.1, US-M4.1, US-M4.4 | T-M3.1-5, T-M4.1-2, T-M4.4-2 | S6, S8 |

**Bold tasks are the tests that would catch a silent, ledger-corrupting regression.** Those five are not cuttable.

---

## 2. Functional requirements → stories

| FR | Story | Sprint |
|---|---|---|
| FR-1 — search, member detail entry point | US-M1.4, US-M4.4 | S5, S8 |
| FR-2 — hierarchy chart, three fields per node | US-M4.2 | S8 |
| FR-3 — member detail | US-M4.1 | S8 |
| FR-4 — add / edit / deactivate member | US-M1.1, US-M1.2, US-M1.3 | S4–S5 |
| FR-5 — Business Volume entry | US-M2.1 | S7 |
| FR-6 — settings | US-M7.1, US-M7.2, US-M7.3, US-M7.4 | S10–S11 |
| FR-7 — monthly reset | US-M5.1, US-M5.2, US-M5.3, US-M5.4 | S11–S12 |
| FR-8 — exports | US-M6.1, US-M6.2, US-M6.3, US-M6.4, US-M6.5 | S13 |
| FR-9 — authentication | US-M8.1, US-M8.2, US-M8.3, US-M8.4 | S5, S7, S8 |
| FR-10 — full hierarchy view | US-M4.3 | S9 |

---

## 3. Acceptance criteria → verifying task

| AC | Verified by | Sprint |
|---|---|---|
| AC-1 – AC-6 | T-M3.1-8 (unit), T-REL.6-1 (client, through the UI) | S6, S16 |
| AC-7 | T-M1.1-2 | S4 |
| AC-8 | T-M1.1-3 | S4 |
| AC-9 | T-M1.1-4 | S4 |
| AC-10 | T-M1.3-3 | S5 |
| AC-11 | T-M1.1-1, T-M1.1-8 | S4 |
| AC-12 | T-M1.2-1, T-M1.2-5 | S5 |
| AC-13 | T-M1.3-6 | S5 |
| AC-14 | T-M1.1-6 | S4 |
| AC-15 | T-M2.1-3 | S7 |
| AC-16 | T-M3.1-7, T-M3.1-8 | S6 |
| AC-17 | T-M2.1-5, T-M3.2-6 | S6, S7 |
| AC-18 | T-M5.2-2 | S12 |
| AC-19 | T-M2.3-3, T-M2.4-5 | S12 |
| AC-20 | T-M5.2-4, T-M5.2-6 | S12 |
| AC-21 | T-M5.1-1, T-M5.2-5 | S11, S12 |
| AC-22 | T-M5.1-6, **T-M5.1-10** | S11 |
| AC-23 | T-M5.1-4 | S11 |
| AC-24 | T-M5.1-5 | S11 |
| AC-25 | T-M5.1-8, T-M6.4-2 | S11, S13 |
| AC-26 | T-M6.1-2, T-M6.1-3 | S13 |
| AC-27 | T-M6.2-1, T-M6.2-2 | S13 |
| AC-28 | T-M6.3-1 | S13 |
| AC-29 | T-M6.5-1 *(five columns, D-1)* | S13 |
| AC-30 | T-M6.1-7 | S13 |
| AC-31 | T-M7.2-7, T-REL.6-4 | S10, S16 |
| AC-32 | T-M7.1-1, T-M7.1-4 | S10 |
| AC-33 | T-M7.1-1 | S10 |
| AC-34 | T-M8.1-4 | S5 |
| AC-35 | T-M8.2-6 | S5 |
| AC-36 | T-QA.4-1 (automated), T-REL.6-5 (human) | S2, S16 |
| AC-37 | T-M7.4-1, T-M8.5-1, T-M8.5-3 | S10, S14 |
| AC-38 | T-M8.6-2, **T-REL.5-3** | S14, S15 |
| AC-39 | T-M7.4-6, T-M8.6-3 | S10, S14 |
| AC-40 | **T-M1.4-6** | S5 |
| AC-41 | T-M1.4-5 | S5 |
| AC-42 | T-M2.3-5 | S12 |
| AC-43 | T-M2.4-5 | S12 |
| AC-44 | T-M4.3-5, T-M4.3-6 | S9 |
| AC-45 | T-M4.3-12, **T-QA.6-3** | S9, S14 |
| AC-46 | T-M3.1-5, T-M3.1-8 | S6 |
| AC-47 | T-M4.4-2, T-M4.4-3 | S8 |

---

## 4. IPC commands → stories

All 40, API-01 to API-40, no gaps (conflict C2). Each also carries a contract test from the `T-QA.2-4` template.

| API | Command | Story | Sprint |
|---|---|---|---|
| API-01 | `create_root_member` | US-M1.1 | S4 |
| API-02 | `add_member` | US-M1.1 | S4 |
| API-03 | `edit_member` | US-M1.2 | S5 |
| API-04 | `deactivate_member` | US-M1.3 | S5 |
| API-05 | `reactivate_member` | US-M1.3 | S5 |
| API-06 | `search_members` | US-M1.4 | S5 |
| API-07 | `get_period_lock_status` | US-M2.3, US-M5.3 | S12 |
| API-08 | `record_entry` | US-M2.1, US-M2.4 | S7, S12 |
| API-09 | `edit_entry` | US-M2.2 | S7 |
| API-10 | `get_member_detail` | US-M4.1 | S8 |
| API-11 | `get_direct_children_chart` | US-M4.2 (`full_tree: false`), US-M4.3 (`true`) | S8, S9 |
| API-12 | `get_outstanding_periods` | US-M5.1 | S11 |
| API-13 | `begin_close` | US-M5.1 | S11 |
| API-14 | `confirm_backup_and_close` | US-M5.1 | S11 |
| API-15 | `manual_backup_current_period` | US-M5.1 | S11 |
| API-16 | `export_monthly` | US-M6.1 | S13 |
| API-17 | `export_yearly_average` | US-M6.2 | S13 |
| API-18 | `export_low_contribution` | US-M6.3 | S13 |
| API-19 | `list_backups` | US-M6.4 | S13 |
| API-20 | `redownload_backup` | US-M6.4 | S13 |
| API-21 | `get_settings` | US-M7.2 | S10 |
| API-22 | `update_settings` | US-M7.2 | S10 |
| API-23 | `add_slab_row` | US-M7.1 | S10 |
| API-24 | `remove_slab_row` | US-M7.1 | S10 |
| API-25 | `update_slab_row` | US-M7.1 | S10 |
| API-26 | `setup_first_run` ° | US-M8.1 | S5 |
| API-27 | `login` ° | US-M8.2 | S5 |
| API-28 | `lock_session` | US-M8.3 | S7 |
| API-29 | `unlock_session` | US-M8.3 | S7 |
| API-30 | `use_recovery_code` ° | US-M8.4 | S8 |
| API-31 | `get_outstanding_alert` | US-M5.2 | S12 |
| API-32 | `get_audit_log` | US-M9.1 | S14 |
| API-33 | `preview_settings_impact` | US-M7.3 | S11 |
| API-34 | `check_data_readable` ° | US-M8.6 | S14 |
| API-35 | `list_restore_points` ° | US-M8.6 | S14 |
| API-36 | `restore_from_backup` ° | US-M8.6 | S14 |
| API-37 | `get_console_backup_settings` | US-M7.4 | S10 |
| API-38 | `update_console_backup_settings` | US-M7.4 | S10 |
| API-39 | `run_console_backup_now` | US-M8.5 | S14 |
| API-40 | `restore_from_backup_file` ° | US-M8.6 | S14 |

**°** = one of the **seven** unauthenticated commands. ⚠️ The set is closed and asserted exactly by `T-QA.2-2` — **seven, not six**; two older documents carry the stale count and the test would fail against correct code if written to it.

---

## 5. Non-functional requirements → work

| NFR | Work item | Sprint |
|---|---|---|
| NFR-1 — performance | US-QA.6, T-M3.2-7 | S6, S14 |
| NFR-2 — scalability to 25,000 | US-QA.5, US-QA.6 | S10, S14 |
| NFR-3 — availability | T-0.1-5 (no network capability exists), verified by US-REL.5 | S1, S15 |
| NFR-4 — security | T-0.2-1, US-M8.2, US-M8.3, §5 of [03-test-plan.md](03-test-plan.md) | S1, S5, S7, S15 |
| NFR-5 — auditability | US-M9.1 | S14 |
| NFR-6 — maintainability | US-M7.1, US-M7.2 *(settings-driven, no hardcoded constants)* | S10 |
| NFR-7 — compliance | T-M1.1-5 (consent), T-M1.3-6 (permanent retention) | S4, S5 |
| NFR-8 — accessibility | T-UI.3-2 (labelled pills), T-UI.3-5 (aria on modals), T-UI.3-7 (contrast) | S3 |
| NFR-9 — localisation | US-UI.1, T-QA.4-1 *(no currency anywhere)* | S2 |
| NFR-10 — reporting | US-M6.1, US-M6.2, US-M6.3 | S13 |
| NFR-11 — technical logging | T-M9.1-5 *(separate rotating file, no UI surface)* | S14 |
| NFR-12 — monitoring | **‡ No work item.** Explicitly declined by the client. Do not build it; do not test for it | — |
| NFR-13 — backup & recovery | US-M5.1, US-M7.4, US-M8.5, US-M8.6 | S11, S10, S14 |
| NFR-14 — hosting & deployment | US-REL.1, US-REL.3, US-REL.4 | S1, S15 |
| NFR-15 — browser & device support | **‡ No work item.** No web deployment target exists — T-0.1-5 makes it structural | S1 |
| NFR-16 — data migration | **‡ No import tooling in scope.** T-0.2-4 builds the migration runner for *future* schema changes only, per DoD item 10 | S1 |

---

## 6. User needs → stories

| UN | Story | UN | Story |
|---|---|---|---|
| UN-01 | US-M1.1 | UN-17 | US-M4.1 |
| UN-02 | US-M1.1 | UN-18 | US-M5.1, US-M5.2 |
| UN-03 | US-M1.1 | UN-19 | US-M5.1 |
| UN-04 | US-M1.2 | UN-20 | US-M6.4 |
| UN-05 | US-M1.1, US-M1.3 | UN-21 | US-M2.2 |
| UN-06 | US-M1.4 | UN-22 | US-M6.1 |
| UN-07 | US-M2.1 | UN-23 | US-M6.2, US-M5.4 |
| UN-08 | US-M2.1, US-M3.1 | UN-24 | US-M6.3 |
| UN-09 | US-M3.1, US-M3.2 | UN-25 | US-M7.1, US-M7.2 |
| UN-10 – UN-13 | US-M3.1 | UN-26 | US-M8.1, US-M8.2, US-M8.4 |
| UN-14 | US-M3.2 | UN-27 | US-QA.4 |
| UN-15 | US-M1.4 | UN-28 | US-M7.4, US-M8.5, US-M8.6 |
| UN-16 | US-M4.2 | UN-29 | US-M1.4 |
| | | UN-30 | US-M2.3 |
| | | UN-31 | US-M4.3 |

---

## 7. Validation rules → verifying task

All **50** — V1.1 to V8.5. ⚠️ `03-functional-specification.md` §6 opens with *"the complete set of 49"*; the actual count is 50 (V4.6 was appended by CR-5 and never counted). `00-master-index.md` §4 already says 50. Corrected by **P-11**.

| V | Rule | Task | Sprint |
|---|---|---|---|
| V1.1 | Name required | T-M1.1-10 | S4 |
| V1.2 | Phone required, unique across active **and** inactive; inactive match offers reactivation | T-M1.1-4 | S4 |
| V1.3 | Reference ID required, must be existing **and active** | T-M1.1-3 | S4 |
| V1.4 | Email optional; valid if given | T-M1.1-10 | S4 |
| V1.5 | Assigned member number must not be in use | T-M1.1-1 | S4 |
| V1.6 | A second top-level member can never be created | T-M1.1-2 | S4 |
| V1.7 | Level width / depth exceeded → **warn, allow** | T-M1.1-6 | S4 |
| V1.8 | Any attempt to change an introducer → **refuse outright** | T-M1.2-1 | S5 |
| V1.9 | Consent unticked → Save disabled | T-M1.1-5 | S4 |
| V2.1 | A member must be selected before recording | T-M2.1-3 | S7 |
| V2.2 | Figure numeric, ≤ 2 decimals | T-M2.1-2 | S7 |
| V2.3 | Current-month entry refused while an earlier month is outstanding | T-M2.4-2 | S12 |
| V2.4 | Neither zero nor negative permitted | T-M2.1-2 | S7 |
| V2.5 | Date field present, pre-filled, bounded to the recording month | T-M2.1-4, T-M2.3-4 | S7, S12 |
| V2.6 | Date must fall within the month recorded into | T-M2.3-4 | S12 |
| V2.7 | Closed month not offered here → correction panel | T-M2.4-3 | S12 |
| V3.1 | Own Business Volume always in own Total — structural | T-M3.1-2 | S6 |
| V3.2 | Only direct children give a differential term — structural | T-M3.1-3 | S6 |
| V3.3 | Only direct children counted/paid for royalty — structural | T-M3.1-4 | S6 |
| V3.4 | 🔶 Nothing prevents a non-monotonic slab table — **not built** | T-M7.1-3, T-M7.1-6 | S10 |
| V3.5 | Inactive status has no calculation effect | **T-M1.3-1, T-M3.2-5** | S5, S6 |
| V4.1 | A search returning nothing says so clearly | T-M1.4-4 | S5 |
| V4.2 | Detail and home search show one level only | T-M4.1-4 | S8 |
| V4.3 | Reward detail order: own-BV line first, then per-leg, then royalty, then total | T-M4.1-2 | S8 |
| V4.4 | Phone matching engages only at 4+ digits | T-M1.4-3 | S5 |
| V4.5 | Full hierarchy gated above 60 descendants; Cancel opens nothing | T-M4.3-5 | S9 |
| V4.6 | Home shows a Rewards-by-slab chart alongside members-by-slab | T-M4.4-2 | S8 |
| V5.1 | Only the oldest outstanding month may be closed | T-M5.1-1 | S11 |
| V5.2 | Nothing cleared until the backup is confirmed | T-M5.1-3 | S11 |
| V5.3 | Nothing cleared until the permanent record is written | T-M5.1-3, T-M5.1-4 | S11 |
| V5.4 | The confirmation screen names the month | T-M5.1-9 | S11 |
| V5.5 | The retained in-system copy is the gate; the download is a convenience | T-M5.1-7 | S11 |
| V5.6 | An empty month produces no record, excluded from the average | T-M5.5-5, T-M5.4-1 | S12, S13 |
| V5.7 | Closed-month edit shows an explicit warning naming that month | T-M2.2-4 | S7 |
| V6.1 | Mandatory columns always present, never removable — **five** per D-1 | T-M6.5-1 | S13 |
| V6.2 | Every yearly average shown with its month count | T-M6.2-2 | S13 |
| V6.3 | Low-contribution threshold must be positive | T-M7.2-8 | S10 |
| V6.4 | A past month's extract reads the permanent record | T-M6.1-4 | S13 |
| V6.5 | The backup carries the threshold table in force that month | T-M5.1-4 | S11 |
| V7.1 | Thresholds must be positive | T-M7.1-7 | S10 |
| V7.2 | Percentages must be between 0 and 100 | T-M7.1-7 | S10 |
| V7.3 | At least one slab row must exist | T-M7.1-4 | S10 |
| V7.4 | Royalty qualifying count must be a positive whole number | T-M7.2-8 | S10 |
| V7.5 | 🔶 Percentages must rise as thresholds rise — **not built** | T-M7.1-3 | S10 |
| V7.6 | A settings change re-works the open month, behind a pre-save warning | US-M7.3 | S11 |
| V8.1 | Repeated failed attempts lock the account | T-M8.2-2 | S5 |
| V8.2 | The alert cannot be dismissed by navigating, logging out, or acknowledging | T-M5.2-4, T-M5.2-6 | S12 |
| V8.3 | Recovery codes are the route back in | T-M8.4-1 | S8 |
| V8.4 | PIN and password coexist; either unlocks | T-M8.1-1 | S5 |
| V8.5 | A restore names what it replaces, requires confirmation, backs up current state first | T-M7.4-6, T-M8.6-3 | S10, S14 |

⚠️ **V3.4 and V7.5 are the two rules that must be verified as *not built*** — `T-M7.1-6`'s deliberate negative test is their verification. ⚠️ **V7.1/V7.2 are per-field range checks and are not V7.5's cross-row monotonicity check** — do not let one be removed in the belief it violates the other.

---

## 8. Module functions → stories

All **57**, M1.1 to M8.7. M9 has no numbered functions — it is cross-cutting.

| Module | Functions | Stories |
|---|---|---|
| **M1** | M1.1 create root · M1.2 add member · M1.3 assign random number · M1.4 edit · M1.5 deactivate/reactivate by phone · M1.6 refuse introducer change · M1.7 consent | US-M1.1 (M1.1–M1.3, M1.7), US-M1.2 (M1.4, M1.6), US-M1.3 (M1.5) |
| **M2** | M2.1 search · M2.2 record · M2.3 name the recording month · M2.4 edit/correct any month · M2.5 dated, bounded entry · M2.6 accept into ended-unclosed · M2.7 refuse current month | US-M1.4 (M2.1), US-M2.1 (M2.2, M2.5), US-M2.2 (M2.4), US-M2.3 (M2.3, M2.6), US-M2.4 (M2.7) |
| **M3** | M3.1 Total Business Volume · M3.2 slab · M3.3 differential · M3.4 royalty · M3.5 own-BV reward · M3.6 combine into Rewards · M3.7 immediate, chain-only | US-M3.1 (M3.1–M3.6), US-M3.2 (M3.7) |
| **M4** | M4.1 home search · M4.2 open detail · M4.3 branch chart · M4.4 full detail · M4.5 inactive colour everywhere · M4.6 phone search + display · M4.7 full hierarchy window | US-M1.4 (M4.1, M4.6), US-M4.1 (M4.2, M4.4), US-M4.2 (M4.3), US-M1.3 (M4.5), US-M4.3 (M4.7) |
| **M5** | M5.1 alert on month end · M5.2 outstanding open / current closed for entry · M5.3 list outstanding, oldest closable · M5.4 backup gate · M5.5 permanent record · M5.6 zero everything · M5.7 retain backups · M5.8 manual backup · M5.9 closed-month edit → new version · M5.10 original never touched | **US-M5.5 (M5.1's trigger)**, US-M5.2 (M5.1), US-M5.3 (M5.2), US-M5.1 (M5.3–M5.8), US-M2.2 (M5.9, M5.10) |
| **M6** | M6.1 monthly extract · M6.2 yearly average with count · M6.3 low-contribution · M6.4 re-download · M6.5 inactive colour in extracts | US-M6.1 (M6.1, M6.5), US-M6.2, US-M6.3, US-M6.4 |
| **M7** | M7.1 slab rows · M7.2 depth and widths · M7.3 royalty · M7.4 yearly cycle and threshold · M7.5 reference unit value · M7.6 default export columns · M7.7 console backup schedule and retention | US-M7.1 (M7.1), US-M7.2 (M7.2–M7.6), US-M7.4 (M7.7) |
| **M8** | M8.1 one login · M8.2 lockout · M8.3 banner on every screen · M8.4 notification list · M8.5 PIN and password coexist · M8.6 console backup · M8.7 restore anywhere | US-M8.1 (M8.1, M8.5), US-M8.2 (M8.2), US-M5.2 (M8.3, M8.4), US-M8.5 (M8.6), US-M8.6 (M8.7) |

⚠️ **M5.1's trigger is US-M5.5, not US-M5.2.** US-M5.2 renders the alert; nothing raised it until D-9 assigned the period transition to the login catch-up.

---

## 9. Client-answered questions → where the answer lives in the build

All **23**. Each was answered by the client; this maps the answer to the work that honours it.

| RQ | Story / task | RQ | Story / task |
|---|---|---|---|
| RQ-1 no monotonicity check | T-M7.1-3, **T-M7.1-6** | RQ-13 reward-detail layout | T-M4.1-2 |
| RQ-2 inactive = zero effect | **T-M1.3-1, T-M3.2-5** | RQ-14 no past month on screen | *(out of scope — no task)* |
| RQ-3 root cannot deactivate | T-M1.3-2 | RQ-15 joining date auto, editable | T-M1.1-7, T-M1.2-3 |
| RQ-4 past extract from the record | T-M6.1-4 | RQ-16 empty month, no record | **T-M5.5-5**, T-M5.4-1 |
| RQ-5 backup carries the table in force | T-M5.1-4 | RQ-17 zero and negative refused | T-M2.1-2 |
| RQ-6 internal copy is the gate | T-M5.1-7 | RQ-18 mid-month change, warned | US-M7.3 |
| RQ-7 editable any time, incl. closed | US-M2.2 | RQ-19 physically separate medium | T-M5.1-7, **T-REL.7-3** |
| RQ-8 retention permanent | T-M1.3-6 | RQ-20 original backup never touched | T-M2.2-2 |
| RQ-9 change log | US-M9.1 | RQ-21 date stays in its own month | T-M2.2-3 |
| RQ-10 recovery codes | US-M8.4 | RQ-22 consent captured | T-M1.1-5 |
| RQ-11 ⚠️ **reversed by CR-2** | US-M2.3, US-M2.4 | RQ-23 whole console portable | US-M7.4, US-M8.5, US-M8.6 |
| RQ-12 unit value = one Reward | T-M7.2-5 | | |

⚠️ **RQ-14 has no task deliberately** — viewing a past month on screen is deferred scope (`01-product-and-scope.md` §5.3), not an omission.

---

## 10. Error & edge-case matrix → coverage

All 63 scenarios in `05-quality-and-acceptance.md` §2, by workflow. Each workflow's scenarios are covered by the tasks listed; the ⚠️ rows are the ones where the obvious implementation is the wrong one.

| Workflow | Scenarios | Covered by | ⚠️ The one to get wrong |
|---|---|---|---|
| Add / edit / deactivate / reactivate (M1) | 11 | T-M1.1-3/-4/-5/-6, T-M1.2-1, T-M1.3-1/-2/-6/-7, T-M1.4-3 | Phone matching an **inactive** member is **not an error** — it offers reactivation |
| Business Volume entry (M2) | 11 | T-M2.1-1/-2, T-M2.2-1/-2/-3, T-M2.3-1/-5, T-M2.4-1/-2/-3/-4 | An entry dated in an ended-but-unclosed month is **accepted**; the **current** month is the one refused |
| Search & structure (M4) | 9 | T-M1.4-3/-4, T-M4.3-5/-10/-11, T-M4.3-8, T-QA.6-3 | Cancelling the size gate **opens nothing at all** |
| Monthly close (M5) | 8 | T-M5.1-1/-6/-7/-10, T-M5.2-5, **T-M5.5-5**, T-M5.4-1 | A **failed external** copy does not block; a **failed internal** copy does |
| Settings / slab table (M7) | 5 | T-M7.1-2/-3/-4/-6, US-M7.3 | A non-monotonic table saves **unblocked**, and its negative differential displays **as-is** |
| Authentication (M8) | 6 | T-M8.2-1/-2/-3/-4, T-M8.3-1, T-M8.4-4 | The counter resets **only on successful login** |
| Exports / reports (M6) | 4 | T-M6.1-2/-6, T-M6.2-1, T-M6.3-3, T-M6.4-2 | A corrected month always returns the **latest** version |
| Downstream / system-level | 4 | T-M8.6-2/-5, T-M5.1-6, T-0.2-1 | A checksum mismatch leaves the corrupt file **untouched** |

---

## 11. Prototype behaviours awaiting port

Five behaviours exist in `ui-prototype-v2.html` as **approved reference behaviour** — to be ported exactly, not reinterpreted.

| Behaviour | Ported by | Sprint |
|---|---|---|
| Settings mid-period recalculation warning (variant C) | US-M7.3 | S11 |
| Last slab row cannot be removed | T-M7.1-4 | S10 |
| Data-recovery screen (design D) | T-M8.6-5 | S14 |
| Console backup schedule and retention | US-M7.4 | S10 |
| Restore flows, both entry points | US-M8.6 | S14 |

Plus four small prototype fixes made 7 Aug 2026 that carry forward as build notes: Escape closes a dismissable modal (`T-UI.3-5`), modal aria attributes (`T-UI.3-5`), the toast icon size rule (`T-UI.3-6`), and the `hashchange` listener for the recovery-screen trigger (`T-M8.6-5`).

⚠️ **Three behaviours specified after the prototype was approved are not in it and must be built from the specification, not copied:** the >60-descendant size gate (`T-M4.3-5`), the full hierarchy window itself (US-M4.3), and CR-4/CR-5's own-Business-Volume reward and Rewards-by-slab chart (`T-M3.1-5`, US-M4.4).

---

## 12. Coverage check

Eleven namespaces. The first pass traced seven; the pre-development audit added the four marked **†** — and found the specification defects behind D-9, D-11, D-12, D-13 and D-14 in the process.

| Namespace | Count | Traced | Orphans |
|---|---|---|---|
| `Rule-N` (incl. Rule-16a) | 47 | 47 | 0 |
| `FR-N` | 10 | 10 | 0 |
| `UN-NN` | 31 | 31 | 0 |
| `NFR-N` | 16 | 13 + **3 ‡** | 0 |
| `AC-NN` | 47 | 47 | 0 |
| `API-NN` | 40 | 40 | 0 |
| `SC-N` | 8 | 8 *(via [03-test-plan.md](03-test-plan.md) §6 Part C)* | 0 |
| **†** `V-N.N` | **50** *(not 49 — see §7)* | 50 | 0 |
| **†** `M-N.N` module functions | 57 | 57 | 0 |
| **†** `RQ-N` | 23 | 22 + **1 ‡** | 0 |
| **†** Error/edge scenarios | 63 | 63 | 0 |
| `US-*` (pre-existing) | 36 | 36 *(all IDs unchanged)* | 0 |
| `US-*` (new) | 21 | 21 | — |

**‡ Four items map to a deliberate non-task**, each a recorded client decision that nothing be built: **NFR-12** (monitoring, declined), **NFR-15** (no browser or device target), **NFR-16** (no migration tooling), **RQ-14** (past month on screen, deferred). Each is named rather than left as a silent gap — the distinction `00-master-index.md` §9 exists to preserve.

**Re-run this check whenever a story is added, split or cut.** An orphan appearing here is a requirement that has quietly left the plan — which is exactly how the D-9 and D-11 defects survived a full specification review.

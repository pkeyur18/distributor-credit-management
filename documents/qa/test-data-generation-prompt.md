# Refined ChatGPT prompt — CSV test-data + expected-results generator

## Context

Goal: verify M3 calculation engine (business rules doc `documents/refinement/02-business-rules.md`) actually matches spec, by generating a year of synthetic member+BV data, importing via `src-tauri/src/bin/import_test_data.rs`, and comparing software output against an independently-computed expected-results CSV. No code touched — this plan's only deliverable is a refined ChatGPT prompt text block plus the gaps found while producing it.

Authoritative source: `documents/refinement/00-master-index.md` (index — "supersedes all source documents") → `02-business-rules.md` (47 rules + calc model + 6 golden scenarios). Older doc sets (`documents/draft/`, `documents/implementation-readiness/`) are explicitly superseded, not used.

Decisions locked in (user-confirmed):
- Settings assumed = documented defaults (7-row slab table, royalty_min_children=3, royalty_rate=1%, low-threshold=100). **If the app's Settings screen has ever been edited from fresh-install defaults, the expected-results numbers will be wrong** — verify Settings screen matches Section "Settings assumed" below before importing.
- Every past month gets closed via `--closed-months`; only the current month stays open — exercises M5 close/snapshot + Rule-23 yearly average, not just live M3 recalculation.
- Rule-28 (inactive member still contributes) and Rule-39 (closed-month correction → new snapshot version) are **out of scope** this round — `import_test_data.rs`'s CSV has no column to deactivate a member and the tool never calls `edit_entry`, so neither is reachable through CSV import alone. Would need a follow-up manual UI step.

## CSV contract the import script enforces (from its own header comment + code, `import_test_data.rs:17-29`)

Columns, any order, header row required: `member_name, phone, address, email, consent, introducer_phone, amount, entry_date`

- One row = one Business Volume entry, **not** one member. A member's identity fields (name/address/email/consent/introducer_phone) are read only on that phone's *first* appearance; later rows for the same phone contribute only another entry (confirmed: `m3_calc` sums all `business_volume_entries` for a member+period via `SUM(amount)` — multiple rows per member per month is normal and they add up, not overwrite).
- `introducer_phone` empty ⇒ root member — **exactly one** such row allowed in the whole file.
- **Strict top-down order**: every introducer's own row must appear somewhere earlier in the file than any member they introduce. The tool is a single forward pass, not a resolver — it panics on an unresolved introducer.
- `consent`: yes/no/true/false/1/0, case-insensitive. Real app requires this ticked to save (Rule-40), so generate `yes` for every row.
- `amount`: plain decimal rupees, **> 0**, max 2 decimals (Rule-16/16a — zero and negative are both refused; there's no such thing as a zero-activity entry, only *no entry*).
- `entry_date`: `YYYY-MM-DD`. `period_month` is derived from this, not supplied.
- Member IDs are not in the CSV — auto-assigned randomly in 100001–999999 by the app (Rule-35); irrelevant to generation.
- Level widths (9/6/3 defaults) and hierarchy depth are **advisory only** (Rule-1, Rule-32) — generation doesn't need to strictly cap children per node, just stay internally consistent.

## Calculation model the expected-results CSV must implement (Rules 3, 6–12, 46 — `02-business-rules.md` §4.1)

```
BusinessVolume(x, period)      = SUM(amount) over that member's entries in that period_month
TotalBusinessVolume(x, period) = BusinessVolume(x) + Σ TotalBusinessVolume(c) for every DIRECT child c
slab%(x, period)                = highest slab threshold ≤ TotalBusinessVolume(x)   [see default table below]
Differential(x, period)         = Σ (slab%(x) − slab%(c)) × TotalBusinessVolume(c)   over DIRECT children c
Royalty(x, period)              = Σ royalty_rate × TotalBusinessVolume(c)  for direct children c on the TOP slab,
                                   only if count(such children) ≥ royalty_min_children, else 0
OwnReward(x, period)            = slab%(x) × BusinessVolume(x)
Rewards(x, period)              = Differential(x) + Royalty(x) + OwnReward(x)
```

Notes the prompt must carry (these are the "13 most likely to be got wrong" items that actually bite a CSV-driven test):
- Differential's base is the child's **TBV**, not the child's own BV (Rule-8) — only Scenario 3 in the docs disambiguates this, so any generated tree with depth ≥ 3 will exercise it.
- Grandchildren contribute nothing as a separate differential term — already folded into their parent's TBV (Rule-8).
- Two decimal places throughout, no intermediate rounding — round only the final displayed/compared number (Rule-22).
- A period with zero entries for a member produces no contribution *and* (at the period level) no snapshot at all if it has zero entries app-wide — excluded from yearly-average denominators (RQ-16/Rule-23). Practically: make sure every closed month in the generated year has at least one entry somewhere, or explicitly note which months are intentionally empty.
- Royalty and differential are structurally disjoint for the same child — never double-pay (Rule-11), useful as a self-check on the generated expected values.
- Rewards never feed back into BV/TBV — each period's calculation is independent, no compounding across months (Rule-13).

## Settings assumed (default slab table — `02-business-rules.md` §4.3 / §6)

| Slab | Threshold | TBV range |
|---|---|---|
| 0% | — | 0–99 |
| 2% | 100 | 100–399 |
| 4% | 400 | 400–1,199 |
| 6% | 1,200 | 1,200–2,999 |
| 8% | 3,000 | 3,000–4,999 |
| 10% | 5,000 | 5,000–6,999 |
| 12% | 7,000 | 7,000–9,999 |
| 14% | 10,000 | ≥10,000 (top slab, triggers royalty) |

royalty_min_children = 3, royalty_rate = 1%, low-contribution threshold = 100 (own-BV yearly-average filter, Rule-24 — informational, not required for the core BV/rewards check).

## The refined ChatGPT prompt (deliverable — copy/paste as-is, fill in the four `{{...}}` placeholders)

Self-contained by design — everything below (CSV rules, formulas, slab table) is inlined so it works standalone in a fresh ChatGPT session with zero access to this repo or its docs.

~~~
This prompt is fully self-contained — no other files, documents, or repo access are needed or assumed. Everything you need is below.

You are generating synthetic test data for a private, single-user business-volume and rewards calculation console app, to validate its calculation engine end-to-end. It works entirely in this restricted vocabulary: member, Business Volume, Rewards, royalty, volume, slab, level, leg. Do not use commercial/network-marketing terms (sale, purchase, order, cash, payment, commission, invoice) anywhere in either output file, including column names, sample data, or your own reasoning. Produce TWO CSV files.

INPUTS (fill in before running):
- Time range: {{START_DATE}} to {{END_DATE}} (inclusive, YYYY-MM-DD) — must cover roughly 12 calendar months
- Target member count: {{MEMBER_COUNT}}
- Hierarchy depth: {{HIERARCHY_DEPTH}} levels (root = level 1)
- Branching factor: {{CHILDREN_PER_NODE}} (rough max direct children per member — advisory only, not a hard cap)

============================================================
FILE 1 — test_data.csv (the file that gets imported into the app)
============================================================
Columns, exact header, any order: member_name, phone, address, email, consent, introducer_phone, amount, entry_date

Rules — follow exactly, they mirror a real bulk-import tool's parsing:
1. One row = one Business Volume entry, not one member. `phone` must be filled in on every single row, including continuation rows — it is the lookup key that ties each entry back to its member. Only the OTHER identity fields (name/address/email/consent/introducer_phone) are meaningful solely on that phone's FIRST row in the file; on later rows for the same phone (additional entries), those other fields may be left blank or repeated — the importer only reads them once. Never blank `phone` itself.
2. Exactly ONE row in the whole file has an empty introducer_phone — that member is the tree root.
3. STRICT top-down ordering: a member's own first-appearance row must come before ANY row that names their phone number as introducer_phone. Build the hierarchy first, then interleave BV entry rows, preserving this order.
4. consent = "yes" on every row.
5. amount: decimal rupees, strictly greater than 0, at most 2 decimal places. Never emit 0 or a negative number — a member with no activity in a month simply gets NO row for that member+month, not a zero-amount row.
6. entry_date: YYYY-MM-DD, must fall within {{START_DATE}}..{{END_DATE}}.
7. A member MAY have zero, one, or multiple entry rows within the same calendar month — if multiple, they represent separate Business Volume entries that SUM together for that member's that-month Business Volume. Vary this realistically (most members: 0-2 entries/month; don't force exactly one entry per member per month everywhere).
8. Build a single tree, {{HIERARCHY_DEPTH}} levels deep, roughly {{CHILDREN_PER_NODE}} children per node, totaling approximately {{MEMBER_COUNT}} members. Make sure at least one branch reaches full depth with enough Business Volume that some parent ends up on the TOP slab (see slab table below) with 3+ direct children also on the top slab, to exercise royalty. Make sure at least one 3+ level chain has differing child slabs to exercise differential meaningfully (not everyone landing on the same slab).
9. Generate realistic but fake Indian-style names, 10-digit phone numbers (unique per member), addresses, emails.

============================================================
FILE 2 — expected_results.csv (what to verify the software's output against)
============================================================
One row per (member, calendar month) that has at least one entry OR at least one descendant with an entry (since a member's TBV/slab/rewards depend on their subtree even in a month they personally didn't transact).

Columns: member_phone, period_month (YYYY-MM), business_volume, total_business_volume, slab_pct, differential, royalty, own_reward, rewards

Compute using EXACTLY this model (no shortcuts, no simplifications):

  business_volume(x, month)      = SUM of that member's own entry amounts in that month (0 if none)
  total_business_volume(x, month)= business_volume(x, month) + SUM(total_business_volume(c, month)) for every DIRECT child c
  slab_pct(x, month)             = highest slab threshold <= total_business_volume(x, month), from the table below
  differential(x, month)         = SUM[ (slab_pct(x) - slab_pct(c)) * total_business_volume(c) ] over DIRECT children c
  royalty(x, month)              = if count(direct children c on the TOP slab, 14%) >= 3:
                                        SUM(0.01 * total_business_volume(c)) over those top-slab direct children c
                                    else 0
  own_reward(x, month)           = slab_pct(x, month) * business_volume(x, month)
  rewards(x, month)              = differential(x, month) + royalty(x, month) + own_reward(x, month)

Default slab table (threshold → percentage):
  0 → 0%, 100 → 2%, 400 → 4%, 1200 → 6%, 3000 → 8%, 5000 → 10%, 7000 → 12%, 10000 → 14% (top slab)

Compute bottom-up (leaves first, root last) for every month independently — nothing carries over between months, rewards never add back into business_volume or total_business_volume for any member including the earner. Round only the FINAL values in the output to 2 decimal places; do not round any intermediate term.

Also append a THIRD file, closed_months.txt: a single line, comma-separated, listing every calendar month in {{START_DATE}}..{{END_DATE}} EXCEPT the one containing {{END_DATE}} itself (that one stays "open"/current), e.g. `2026-06,2026-07,2026-08`. This is the exact value to pass to the import tool's --closed-months flag.

Output all three files ready to download.
~~~

## Verification approach (once CSVs exist — not part of this analysis task, just so the plan is complete)

1. `cargo run --bin import_test_data -- --csv test_data.csv --credential <pin-or-password> --closed-months $(cat closed_months.txt)`
2. For each closed month: compare `expected_results.csv` rows for that month against `monthly_snapshots` (version = MAX(version) per member/period) in `console.db`.
3. For the current open month: compare against live `member_period_totals`.
4. Cross-check the six golden scenarios in `02-business-rules.md` §5 still reconcile (65/62/510/1,000/980/10) independently of the generated data — regression guard, not part of the generated set.

## Open items / things to double check before running this

- **Settings screen**: open Settings in the actual app instance being tested and confirm the slab table / royalty rate / royalty min-children still match the defaults above. If they were ever edited, the expected-results formula block in the prompt must be updated with the real values first — do not run the prompt as-is against a customized instance.
- **Out of scope, confirmed**: Rule-28 (inactive member) and Rule-39 (closed-month correction/new snapshot version) are not exercised by this CSV-driven round. If you want them covered, that's a separate manual test (deactivate a member / edit a closed-month entry in the running app after this import) — not something to fold into the CSV.

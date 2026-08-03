# Distributor Credit Points & Beneficiary Management System
## Requirement Specification — v1.0 (for review)

| | |
|---|---|
| **Client** | Siddharth Patel |
| **Architect / Developer** | Keyur Patel |
| **Source** | [requirement-draft.md](requirement-draft.md) |
| **Status** | Draft for client review — contains open questions that block implementation |

> **How to read this document.** Everything stated as a **Rule** is confirmed and implementable. Everything marked **[DERIVED]** was inferred from the draft's worked examples because the draft never states it in prose — these are correct against the numbers but need the client's explicit "yes". Everything marked **[CONFIRMED]** is a derived item the client has since agreed, carrying the date it was settled. Everything in [§9 Open Questions](#9-open-questions) is genuinely unanswered and nothing was assumed in its place; answered items stay there, marked, rather than being deleted.
>
> **Status badges** used throughout:
>
> | Badge | Meaning |
> |---|---|
> | ✅ **CONFIRMED** | Settled. Build on it. |
> | 🟡 **PROVISIONAL** | Agreed, but being re-checked with the client. Design against it; do not treat as final. |
> | ⏸️ **DEFERRED** | Parked pending client input. |
> | ☐ **OPEN** | No answer yet. |

---

## 1. Overview

A single-admin dashboard for managing a hierarchy of members, tracking credit points earned by each member, rolling those points up the hierarchy, and computing each member's earned score from the slab differential between them and the members directly beneath them.

### 1.1 Glossary — four quantities the draft treats as one

The draft uses the phrase "credit points" for four different things. Every calculation below depends on keeping them apart. This is the single most important clarification in this document.

| Term | Definition | How it changes |
|---|---|---|
| **Individual Credit Points** (ICP) | Points recorded directly against one member by the admin on the points-add screen. | Manual entry only. Reset to 0 at monthly reset. |
| **Business Volume** (BV) | A member's own ICP plus the **already-computed BV of each direct child** — one level of addition only. Because each child's figure is itself complete, the total transitively covers every depth without re-walking the tree. | Derived. Recomputed whenever any ICP beneath the member changes. |
| **Slab %** | The percentage band a member falls into, looked up from their **BV** (not their ICP). | Derived from BV. |
| **Earned Points** | The member's score for the period = Differential + Royalty. | Derived. **A separate ledger** — see Rule 6. |

> **Naming — ✅ [CONFIRMED — client, 2026-08-03].** **"Business volume" is the term everywhere** — screens, column headers, exports and this document alike. The draft's one use of *"total purchase volume"* (line 39) is replaced by it, because *purchase* is on the client's own forbidden list (draft line 45) while *business volume* is not and is already the term used throughout the rest of the draft. No renaming to *Group Volume* or *Team Points* is needed.

### 1.2 Vocabulary constraint

No user-visible string — screen label, button, column header, export filename, error message, tooltip — may use *sale*, *purchase*, *order*, *cash*, *payment*, *commission*, *invoice*, or equivalents. Permitted vocabulary: *member, credit points, rewards, earned points, royalty, volume, slab, level, leg*.

---

## 2. Hierarchy Model

### 2.1 Structure

- A single tree. Exactly **one root member** at Level 1. This is fixed permanently and can never increase.
- Every non-root member has exactly **one parent**, set at onboarding via a mandatory **Reference ID**.
- Depth is **configurable** in settings.

### 2.2 Level widths — soft defaults, not enforced

**Rule 1.** The per-level width figures (Level 2 = 9, Level 3 = 6, Level 4 = 3) are stored in settings as **informational defaults only**. Onboarding does **not** reject a member for exceeding them.

*Rationale:* the draft's own scenarios contradict a hard limit — Scenario 3 gives a Level-2 member 6 children while Scenario 5 gives a member 7 children. Confirmed with the client that these are advisory. The UI may show a soft warning; it must not block.

### 2.3 Member identity

**Rule 2.** Every member receives a unique **6-digit ID** at onboarding. IDs are the primary lookup key for search, points entry and reference linking.

### 2.4 Hierarchy tree

```mermaid
flowchart TD
    R["Root — Level 1<br/>exactly 1, fixed"]
    L2A["Level 2"]
    L2B["Level 2"]
    L2C["Level 2 — default width 9"]
    L3A["Level 3"]
    L3B["Level 3 — default width 6"]
    L4A["Level 4 — default width 3"]
    R --> L2A
    R --> L2B
    R --> L2C
    L2A --> L3A
    L2A --> L3B
    L3A --> L4A
```

---

## 3. Slab Table

### 3.1 Default slabs

**Rule 3.** A member's slab is the **highest** slab whose point threshold is **less than or equal to** the member's **Business Volume**. A member below the lowest threshold is on 0%.

| Slab | Threshold (points) | Applies when BV is | Notes |
|---|---|---|---|
| 0% | — | 0 – 99 | Implicit base slab, not in the draft's list |
| 2% | 100 | 100 – 399 | |
| 4% | 400 | 400 – 1,199 | |
| 6% | 1,200 | 1,200 – 2,999 | |
| 8% | 3,000 | 3,000 – 4,999 | |
| 10% | 5,000 | 5,000 – 6,999 | |
| 12% | 7,000 | 7,000 – 9,999 | |
| 14% | 10,000 | ≥ 10,000 | **Top slab** — triggers royalty eligibility |

The `>=` boundary rule is proved by the draft: Scenario 2 places C on 8% at exactly 3,000 points, and Scenario 4 places A on 14% at exactly 10,000 points.

### 3.2 Configurability

**Rule 4.** Every threshold in the table is editable in settings. The examples given by the client — moving the 2% slab to 200 points, moving the 6% slab to 1,000 points — must both be supported.

The **top slab** (the royalty trigger) is defined as the row with the highest percentage, whatever its threshold currently is. It is not hard-coded to 14% or to 10,000 points.

**Rule 27 — Slab rows are addable and removable.** ✅ **[CONFIRMED — client, 2026-08-03]** The admin may **add and remove** slab rows, not merely re-threshold the existing seven. The slab table can grow to eight rows or shrink to five. The top slab is always recomputed as the highest-percentage row, so the royalty trigger stays correct without anyone having to update it separately.

---

## 4. Calculation Logic (CRITICAL)

### 4.1 Order of evaluation

**Rule 26 — Recalculation trigger.** ✅ **[CONFIRMED — client, 2026-08-03]** The system recalculates **immediately on every points entry**. The moment an entry is saved, every affected member's BV, slab and earned points are correct on screen. There is no manual "recalculate" button and no batch-only mode.

> **Scale:** expected size is **500 to 5,000 members**. Immediate recalculation is comfortable at that size, but the implementation should update **only the affected chain upward** from the member who received points, rather than rebuilding the whole tree on every entry. Internal design note — no visible difference to the admin.
>
> ☐ **Still needed:** expected number of points entries per month. Not yet supplied.

**Rule 5.** Calculation runs **bottom-up** — a post-order traversal of the tree. A member's BV cannot be computed until each **direct child's** BV is final; the deeper levels are already folded into those figures by the same rule applied one level lower. Results propagate to the root.

### 4.2 The rules

**Rule 6 — Business Volume.**
```
BV(x) = ICP(x) + Σ BV(c)   for every direct child c of x
```
**The sum is one level deep.** Only direct children appear in it, and each contributes their *already-computed* BV. Full-depth coverage is a consequence of that, not a separate step: because the same rule was applied one level lower, each child's figure already carries their own team. Nothing is double-counted and nothing is missed. **[CONFIRMED — client, 2026-08-03]** — proved by Scenario 3: A's BV is 8,000 while six children at 1,250 sum to 7,500, so A's own ICP is 500, and D's contribution of 1,250 had already absorbed p1/p2/p3. The `ICP(x)` term — a member's own points always counting toward their own BV — is confirmed without exception.

**Rule 7 — Slab.** `slab%(x) = lookup(BV(x))` per Rule 3. Driven by BV, never by ICP.

**Rule 8 — Differential earnings.**
```
Differential(x) = Σ [ (slab%(x) − slab%(c)) × BV(c) ]   for every DIRECT child c of x
```
Three things this says, each of which the draft leaves implicit:
- The base is the child's **BV**, not the child's ICP. **[CONFIRMED — client, 2026-08-03]** — Scenarios 1, 2, 4 and 5 use leaf children where BV and ICP are identical, so only Scenario 3 disambiguates: it applies 6% to B's *volume* of 1,250. Confirmed as the record of why, not as a request.
- Only **direct children** contribute a term. Grandchildren are already inside the child's BV. **[DERIVED]** — consistent across all five scenarios.
- A member earns **nothing on their own ICP**. In every scenario the member's own points inflate their BV (and therefore their slab) but never appear as an earning term. **[CONFIRMED — client, 2026-08-03]**

**Rule 9 — The differential can never be negative.** Because `BV(parent) ≥ BV(child)` by construction (Rule 6), `slab%(parent) ≥ slab%(child)` always holds. No clamping, no negative-earnings case, no error state. This is a structural guarantee, not a check.

**Rule 10 — Royalty qualification.** Let `Q` = the set of **direct children** of x whose slab is the **top slab**.
```
if |Q| >= ROYALTY_MIN_CHILDREN   (default 3, configurable)
    Royalty(x) = Σ [ ROYALTY_RATE × BV(c) ]   for every c in Q      (rate default 1%)
else
    Royalty(x) = 0
```
Confirmed with the client: **direct children only**, both for counting and for paying.

**Rule 11 — Royalty and differential never double-pay.** If a child is on the top slab, the parent's BV is at least the child's BV, so the parent is on the top slab too, so that child's differential term is exactly 0 (Rule 9). The two mechanisms are automatically disjoint — no explicit exclusion logic is needed.

> This is what the draft's confusing sentence on line 133 — *"since P has 0 earned points from his descendants, hence he can now start earning royalty"* — is actually describing. It reads like a precondition but it is a **consequence**. Scenario 5 proves it is not a precondition: P there has 580 points of differential from E, F and G and still collects royalty from A–D. The only real precondition is the `|Q| >= 3` count.

**Rule 25 — Royalty stacks at every level.** 🟡 **[PROVISIONAL — client, 2026-08-03]** Each member is assessed **independently** against their own direct children. If a member qualifies under Rule 10, their upline may also qualify on that member's BV, and so on to the root — the same underlying volume can therefore attract royalty at several levels of the same chain.

> Worked illustration: A, B and C each hold BV 10,000 under P. BV(P) = 30,000, three top-slab children, so P collects 1% × 30,000 = **300**. P, Q and R are identical siblings under T. BV(T) = 90,000, three top-slab children, so T collects 1% × 90,000 = **900**. Total paid across the chain is 1,800, and A's original 10,000 has attracted royalty twice.
>
> Being re-checked with the client, because it directly increases total payout.

**Rule 12 — Earned Points.**
```
Earned(x) = Differential(x) + Royalty(x)
```

**Rule 13 — Earned Points are a separate ledger.** Confirmed with the client. Earned Points are **never** added to any member's ICP. They do not raise the earner's own slab, do not enter any ancestor's BV, and do not compound into the next period. The draft's line 60 — *"Always pay Royalty in credit points, not in cash"* — means royalty is **denominated** in points, not that it credits the ICP balance.

**Rule 14 — Point value.** ✅ **[CONFIRMED — client, 2026-08-03]** `1 point = 500 Rs`, configurable, retained on the settings screen per draft L56. It is **reference only**: no rupee figure is displayed on any screen, report or export, and it plays no part in any calculation. All calculation, entry, storage and display happen in points.

> **Why the setting is kept.** The client converts **final earned points** into rupees at this rate **by hand, outside the application** (see Q-I8). The setting is their reference figure for doing that sum. Building the conversion into the software is explicitly **not** wanted now, and may be added later if asked.

### 4.3 Per-member calculation flow

```mermaid
flowchart TD
    A["Start at member x<br/>(all descendants already computed)"] --> B["BV(x) = ICP(x) + Σ BV(children)"]
    B --> C["slab%(x) = lookup BV(x) in slab table"]
    C --> D["For each DIRECT child c:<br/>diff += (slab%(x) − slab%(c)) × BV(c)"]
    D --> E{"Count direct children<br/>on TOP slab ≥ 3?"}
    E -- "No" --> F["Royalty(x) = 0"]
    E -- "Yes" --> G["Royalty(x) = Σ 1% × BV(c)<br/>for each top-slab direct child"]
    F --> H["Earned(x) = diff + Royalty(x)"]
    G --> H
    H --> I["Store as separate ledger.<br/>Does NOT modify ICP or BV."]
    I --> J["Move to parent of x"]
```

---

## 5. Worked Scenarios — re-derived from the rules above

Each scenario below was recomputed from Rules 6–12 alone. All five totals match the draft.

### 5.1 Scenario 1 — basic differential

```mermaid
flowchart TD
    D["D — ICP 500<br/>BV 1,850 → 6%"]
    A["A — ICP 300<br/>BV 300 → 2%"]
    B["B — ICP 50<br/>BV 50 → 0%"]
    C["C — ICP 1,000<br/>BV 1,000 → 4%"]
    D --> A
    D --> B
    D --> C
```

BV(D) = 500 + 300 + 50 + 1,000 = **1,850** → 6% slab.

| Child | Child BV | Child slab | D slab | Differential % | Earned |
|---|---|---|---|---|---|
| A | 300 | 2% | 6% | 4% | **12** |
| B | 50 | 0% | 6% | 6% | **3** |
| C | 1,000 | 4% | 6% | 2% | **20** |
| | | | | **Total** | **35** |

Royalty: 0 direct children on the top slab → not eligible.
**Earned(D) = 35** ✅ matches draft.

### 5.2 Scenario 2 — differential collapses to zero on an equal slab

Identical to Scenario 1 except C's ICP is 3,000.

BV(D) = 500 + 300 + 50 + 3,000 = **3,850** → 8% slab.

| Child | Child BV | Child slab | D slab | Differential % | Earned |
|---|---|---|---|---|---|
| A | 300 | 2% | 8% | 6% | **18** |
| B | 50 | 0% | 8% | 8% | **4** |
| C | 3,000 | 8% | 8% | 0% | **0** |
| | | | | **Total** | **22** |

Royalty: C is on 8%, which is not the top slab. 0 qualifying children → not eligible.
**Earned(D) = 22** ✅ matches draft.

### 5.3 Scenario 3 — multi-depth rollup

```mermaid
flowchart TD
    A["A — ICP 500 (derived)<br/>BV 8,000 → 12%"]
    B["B — BV 1,250 → 6%"]
    C["C — BV 1,250 → 6%"]
    D["D — BV 1,250 → 6%"]
    E["E — BV 1,250 → 6%"]
    F["F — BV 1,250 → 6%"]
    G["G — BV 1,250 → 6%"]
    P1["p1"]
    P2["p2"]
    P3["p3"]
    A --> B
    A --> C
    A --> D
    A --> E
    A --> F
    A --> G
    D --> P1
    D --> P2
    D --> P3
```

BV(A) = ICP(A) + 6 × 1,250 = ICP(A) + 7,500 = **8,000** → so **ICP(A) = 500**. The draft never states this figure; it was derived here and the client has since confirmed that a member's own points always count toward their own business volume.

8,000 falls in 7,000–9,999 → **12% slab**.

| Child | Child BV | Child slab | A slab | Differential % | Earned |
|---|---|---|---|---|---|
| B – G (six members) | 1,250 each | 6% | 12% | 6% | **75 each** |
| | | | | **Total** | **450** |

**Key point:** p1, p2 and p3 contribute nothing directly to A's earnings. Their points are already absorbed into D's BV of 1,250, and A earns on D's BV. This is what makes the differential model self-limiting.

Royalty: no direct child on the top slab → not eligible.
**Earned(A) = 450** ✅ matches draft.

### 5.4 Scenario 4 — pure royalty

```mermaid
flowchart TD
    P["P — BV 100,000 → 14%<br/>4 top-slab children ✓"]
    A["A — BV 10,000 → 14%"]
    B["B — BV 20,000 → 14%"]
    C["C — BV 30,000 → 14%"]
    D["D — BV 40,000 → 14%"]
    P --> A
    P --> B
    P --> C
    P --> D
```

BV(P) = 10,000 + 20,000 + 30,000 + 40,000 = **100,000** → 14% (top slab).

> **Note — settled 2026-08-03.** The draft computes `A + B + C + D` and omits ICP(P) entirely, unlike Scenarios 1 and 3 which include the parent's own points. The client has confirmed this was a **simplification in the write-up, not a different rule**: own points are always counted. The example therefore stands as shown, with ICP(P) = 0.

**Differential:** every child is on 14%, P is on 14% → all four terms are 0. Total **0**.

**Royalty:** 4 direct children on the top slab ≥ 3 → **eligible**.

| Child | Child BV | Royalty @ 1% |
|---|---|---|
| A | 10,000 | **100** |
| B | 20,000 | **200** |
| C | 30,000 | **300** |
| D | 40,000 | **400** |
| | **Total** | **1,000** |

**Earned(P) = 0 + 1,000 = 1,000** ✅ matches draft.

### 5.5 Scenario 5 — differential and royalty together

```mermaid
flowchart TD
    P["P — BV 49,000 → 14%<br/>4 top-slab children ✓"]
    A["A — 10,000 → 14%"]
    B["B — 10,000 → 14%"]
    C["C — 10,000 → 14%"]
    D["D — 10,000 → 14%"]
    E["E — 2,000 → 6%"]
    F["F — 3,000 → 8%"]
    G["G — 4,000 → 8%"]
    P --> A
    P --> B
    P --> C
    P --> D
    P --> E
    P --> F
    P --> G
```

BV(P) = (4 × 10,000) + 2,000 + 3,000 + 4,000 = **49,000** → 14% (top slab). ICP(P) is again omitted by the draft, for the same reason settled in §5.4 — a simplification in the example, not a different rule. Taken as ICP(P) = 0 here.

**Differential:**

| Child | Child BV | Child slab | P slab | Differential % | Earned |
|---|---|---|---|---|---|
| A | 10,000 | 14% | 14% | 0% | 0 |
| B | 10,000 | 14% | 14% | 0% | 0 |
| C | 10,000 | 14% | 14% | 0% | 0 |
| D | 10,000 | 14% | 14% | 0% | 0 |
| E | 2,000 | 6% | 14% | 8% | **160** |
| F | 3,000 | 8% | 14% | 6% | **180** |
| G | 4,000 | 8% | 14% | 6% | **240** |
| | | | | **Subtotal** | **580** |

**Royalty:** A, B, C, D on the top slab = 4 ≥ 3 → eligible. 1% × 10,000 = 100 each → **400**.

**Earned(P) = 580 + 400 = 980** ✅ matches draft.

This scenario is the one that settles the royalty rule: P earns a non-zero differential *and* royalty in the same period, so "zero differential" is not a royalty precondition.

---

## 6. Functional Requirements

### FR-1 — Home / Search
Search by member **name** or **6-digit ID**. Selecting a result opens that member's detail view with their hierarchy shown to **one depth only** (direct children).

### FR-2 — Hierarchy chart
Visual tree of members under a chosen member. Each node shows exactly three fields: **name, ID, credit points**. Nothing else.

> 🟡 **PROVISIONAL — client, 2026-08-03.** The node shows the member's **own credit points**, not their Business Volume. ⚠️ This **differs from the recommendation**, which proposed BV on the grounds that the chart exists to show volume building upward. Being re-checked with the client before it is treated as final.
>
> One factual consequence to be aware of: because the slab is driven by BV and not by own points, a node can display a small own-points figure while the member sits on a high slab. The chart will therefore not, on its own, explain why someone is on the slab they are on.

### FR-3 — Member detail
Shows: name, phone number, address, all point details earned, direct children (1 depth only) with their points, total **business volume**, and **number of legs** = count of direct children.

### FR-4 — Add member
Captures name, phone number, email (optional), **Reference ID (mandatory)**, address, and remaining basic fields. Reference ID must resolve to an existing member. On save, assigns a unique 6-digit ID.

**Rule 30 — Reference and hierarchy integrity.** ✅ **[CONFIRMED — client, 2026-08-03]**
- The Reference ID must resolve to an **existing, active** member. Anything else is rejected at entry with a clear message.
- The single root member is created **once, during initial setup**, as a special step with no Reference ID. The option is never available again — the top level can never grow beyond one person (Rule 1).
- Any move that would place a member **beneath their own descendant** is blocked, with the reason shown.

**Rule 32 — Depth overflow.** ✅ **[CONFIRMED — client, 2026-08-03]** If onboarding would exceed the configured maximum depth, the system **warns but allows**. Consistent with Rule 1, where the per-level widths are advisory rather than enforced — a real member is never blocked by a settings value.

**Rule 28 — Member lifecycle.** ✅ **[CONFIRMED — client, 2026-08-03]**
- **Edit** — permitted at any time (name, phone, address and so on).
- **Removal** — a member may be marked **inactive**, so they stop appearing in new periods. They are **never hard-deleted**; their history stays intact.
- **Move to a different sponsor** — permitted. **Already-closed months are frozen exactly as they were** and are never recalculated; only the current and future periods use the new position.

> Hard deletion is prohibited because it would silently change past reports, leaving no way to explain why last year's figures no longer match what was seen at the time. Freezing closed months means a past report renders identically every time it is opened.

### FR-5 — Points add screen
**Rule 15.** Admin searches by name or ID, selects a member, records credit points against them.

**Rule 16 — Points-only entry.** ✅ **[CONFIRMED — client, 2026-08-03]** The admin enters **credit points directly**, and nothing else. The field accepts up to **two decimal places** — `250` or `250.50` are both valid. There is no rupee entry mode, no currency conversion, and no rupee field anywhere on this screen.

> **Supersedes an earlier decision.** This replaces the original "two entry modes, admin's choice" rule (rupee mode plus points mode), which was locked at the start of this work and has now been reversed by the client. Recorded here deliberately rather than rewritten silently.

**Rule 22 — Precision.** ✅ **[CONFIRMED — client, 2026-08-03]** Credit points and earned points carry **two decimal places** throughout storage and calculation. Rounding happens **only at the point of display**, never at an intermediate step — no per-child-term rounding before summing, so totals always reconcile against a calculator.

### FR-6 — Settings
All values in [§7](#7-settings-inventory) are editable here.

### FR-7 — Monthly reset (manual, backup-gated)
**Rule 17.** Reset is **manual only** — never automatic. The admin is **prompted** on the 1st of each month but may act later.

**Rule 21 — Period boundaries.** ✅ **[CONFIRMED — client, 2026-08-03]**
- A period is a **calendar month**, 1st to last day.
- The reset closes **whichever month it belongs to**, whenever it is actually pressed. Pressing it on 5 September still closes August.
- Points entered between the 1st and the moment of reset count into the **month being closed**, not the new one. The confirmation screen must name the month it is about to close, explicitly and unambiguously.

**Rule 20 — Persistent reset alert.** ✅ **[CONFIRMED — client, 2026-08-03]** — client-added requirement, not present in the original draft.
- Raised as soon as the month being closed has ended.
- Appears as **both** an undismissable banner on every screen, naming the outstanding month, **and** an entry in the notification list.
- **Clears only on successful completion of the reset.** Not on navigation, not on logout, not on acknowledgement. There is no snooze and no dismiss control.
- Where **several months are outstanding**, the alert lists every one of them. Only the **oldest** can be closed; the next unlocks once it completes.
- Each outstanding month is closed **separately**, keeping its own backup and its own snapshot. Months are never merged into a combined period.

**Rule 18.** Reset flow is strictly gated:
```mermaid
flowchart LR
    Z["Month ends →<br/>persistent alert raised"] --> A["Admin triggers reset<br/>(oldest outstanding month)"]
    A --> B["Popup: back up this month's data as Excel?"]
    B --> C{"Backup file<br/>successfully generated?"}
    C -- "No / cancelled / failed" --> D["ABORT — no data is reset.<br/>Alert stays up."]
    D --> Z
    C -- "Yes" --> E["Zero the points for all members"]
    E --> F["Reset complete, new period begins.<br/>Alert clears for this month."]
    F --> G{"Any older months<br/>still outstanding?"}
    G -- "Yes" --> Z
    G -- "No" --> H["No alert"]
```
**A reset must never proceed without a confirmed successful backup.** A failed or cancelled backup leaves the alert in place. Exactly which values are zeroed, and how the period is archived, is unresolved → [Q-B5](#blocking).

**Rule 31 — Backup storage and retention.** ✅ **[CONFIRMED — client, 2026-08-03]** Each backup is **downloaded to the administrator's computer and also retained permanently inside the system**, where any past month can be re-downloaded at any time. Nothing is auto-deleted.

> Two independent copies, because the reset is gated on this backup. If the only copy were a file in a downloads folder, a lost or overwritten download would defeat the gate the client deliberately asked for.

### FR-8 — Exports

| Export | Contents |
|---|---|
| **Monthly data** | Default columns: name, ID, phone number, credit points. Admin-configurable additional columns. |
| **Yearly average** | Per member: yearly average of volume **and** of own credit points, plus the month count each average is based on (Rule 23). Yearly cycle defaults to 1 Jan – 31 Dec, configurable. |
| **Low-threshold report** | Members whose yearly average of **own credit points** falls below a configurable threshold (default 100 points) — Rule 24. |

**Rule 19.** Every exported report includes the member's basic details, phone number, volume and credit points, regardless of which optional columns are selected.

**Rule 23 — Yearly average method.** ✅ **[CONFIRMED — client, 2026-08-03]** Sum the member's figures across the periods that **actually have a snapshot**, and divide by the **count of those periods** — not by a fixed 12. The report must **display that month count** next to each average, so a figure based on three months is never mistaken for one based on twelve. This protects members who joined part-way through the year, and protects everybody if a reset is ever late.

**Rule 24 — Low-threshold report metric.** ✅ **[CONFIRMED — client, 2026-08-03]** The report filters on the yearly average of the member's **own credit points**, not their Business Volume.

> **Client answer differs from the recommendation.** This specification originally recommended filtering on Business Volume, reading the 100-point threshold as "never reached the lowest slab". The client instead wants the report to reflect what each person personally brought in, independent of the team beneath them. The **yearly-average export still carries both figures** (draft L46) — only the *filter metric* is own credit points.

All exports are Excel format.

**Rule 33 — Configurable export columns.** ✅ **[CONFIRMED — client, 2026-08-03]** Every field is offered, with the client's four defaults pre-ticked. Available columns: **name, ID, phone number, credit points** (defaults), plus email address, address, reference number, name of the person they work under, level in the hierarchy, number of direct legs, business volume, slab percentage, earned points, royalty earned, joining date, and active/inactive status.

### FR-9 — Authentication

**Rule 29 — Access control.** ✅ **[CONFIRMED — client, 2026-08-03]**
- **One administrator account**, used solely by the client. There are no other user accounts and no roles.
- **Members never log in** and have no access of any kind to the system.
- Protected by either a **6-digit PIN or a complex password** — ⏸️ the client will decide which. Both must be supported in design until that choice is made.

> **Security requirement, applying to either choice.** A 6-digit PIN is one million combinations and is trivially brute-forced if attempts are unlimited. **Failed-attempt limiting with lockout is mandatory, not optional** — this single account guards every member's personal details and phone number. If the PIN route is chosen, this is the only thing standing between the PIN and an attacker.

---

## 7. Settings Inventory

Everything the draft describes as configurable, in one place.

| # | Setting | Default | Source |
|---|---|---|---|
| 1 | Slab thresholds (7 rows) | 100 / 400 / 1,200 / 3,000 / 5,000 / 7,000 / 10,000 | draft L21–32 |
| 2 | Slab percentages | 2 / 4 / 6 / 8 / 10 / 12 / 14 | draft L21–27 |
| 3 | Point value in Rs — **reference only, never displayed elsewhere** (Rule 14) | 500 | draft L56 |
| 4 | Hierarchy depth | not specified | draft L15 |
| 5 | Level 2 width (advisory) | 9 | draft L12 |
| 6 | Level 3 width (advisory) | 6 | draft L13 |
| 7 | Level 4 width (advisory) | 3 | draft L14 |
| 8 | Royalty min. qualifying direct children | 3 | draft L59 |
| 9 | Royalty rate — ✅ confirmed configurable | 1% | draft L59 |
| 9a | Slab rows — add / remove permitted (Rule 27) | 7 rows | ✅ confirmed 2026-08-03 |
| 10 | Yearly cycle start / end | 1 Jan – 31 Dec | draft L46 |
| 11 | Low-average threshold | 100 points | draft L47 |
| 12 | Export column selection | name, ID, phone, credit points | draft L44 |

---

## 8. Verification of this Document

Every scenario in [§5](#5-worked-scenarios--re-derived-from-the-rules-above) was recomputed from Rules 6–12 alone, without reference to the draft's stated answers. All five totals reconcile:

| Scenario | Differential | Royalty | Total | Draft says | Match |
|---|---|---|---|---|---|
| 1 | 35 | 0 | 35 | 35 | ✅ |
| 2 | 22 | 0 | 22 | 22 | ✅ |
| 3 | 450 | 0 | 450 | 450 | ✅ |
| 4 | 0 | 1,000 | 1,000 | 1,000 | ✅ |
| 5 | 580 | 400 | 980 | 980 | ✅ |

The calculation model in §4 is therefore **arithmetically consistent with every example the client provided**. What remains is confirming the model's *interpretation*, which is what §9 asks.

> **Change log — 2026-08-03 (Q-I7, Q-I8 and all Minor questions answered — every question now has an answer).** The final seven landed. Position is now **19 confirmed, 2 provisional, 1 deferred, 0 open**.
> - **Q-I8 — the suspected missing feature does not exist.** The client corrected the wording: draft line 7's *"final discounts"* means **final earned points**, already modelled by Rule 12. No discount feature, nothing extra to build. The rupee conversion is done **by hand, outside the application**, which retrospectively explains why Rule 14's point-value setting is kept but never displayed.
> - **New Rule 29 — authentication.** One administrator account (the client only); members never log in. PIN or complex password, ⏸️ choice still with the client. ⚠️ **Failed-attempt lockout recorded as mandatory either way** — a 6-digit PIN is one million combinations and this account guards every member's personal details.
> - **New Rule 28** — member lifecycle: edit freely, deactivate but never hard-delete, moves permitted with closed months frozen so past reports never change retrospectively.
> - **New Rule 30** — reference must resolve to an existing active member; root created once at setup; loop-creating moves blocked.
> - **New Rule 31** — backups downloaded locally *and* retained permanently in the system.
> - **New Rule 32** — depth overflow warns but allows, consistent with Rule 1.
> - **New Rule 33** — full configurable export column list.
> - **§10** — authentication removed (now answered); the data-volume line narrowed to entries-per-month only.
>
> No calculation, formula or scenario total changed. Rules 1–27 were not renumbered. **Q-B5 remains the sole blocker.**
>
> **Change log — 2026-08-03 (Q-I1 to Q-I6 answered; Q-B5 deferred; status badges introduced).** Six of the eight "Important" questions landed, and a four-state badge model was introduced (confirmed / provisional / deferred / open) because two answers are subject to a client re-check:
> - **Naming** — "business volume" everywhere; draft L39's "total purchase volume" replaced. §1.1 note rewritten as settled.
> - **Chart value** 🟡 — shows **own credit points**. ⚠️ Differs from the recommendation of BV; provisional pending client re-check. FR-2 rewritten, with a note that a node may show small own points while sitting on a high slab.
> - **Royalty rate** — confirmed configurable; settings row 9 caveat dropped.
> - **Royalty stacking** 🟡 — allowed at every level, each assessed independently. New **Rule 25**, provisional, with a worked illustration showing the same volume attracting royalty twice in one chain.
> - **Slab rows** — add and remove permitted; top slab always the highest-percentage row. New **Rule 27**, replacing the unresolved paragraph in §3.2.
> - **Recalculation** — immediate on every entry. New **Rule 26**, with expected scale 500–5,000 members and a note to update only the affected chain upward. Entries per month still outstanding.
>
> **Q-B5 is deferred**, not answered, and remains the sole blocker on architecture. No calculation, formula or scenario total changed. Rules 1–24 were not renumbered.
>
> **Change log — 2026-08-03 (Q-B6 closed; Q-B7 and Q-B8 answered; rupee removed).** Three answers landed:
> - **Yearly average** — divide by the count of periods that actually have a snapshot, displaying that count alongside (new **Rule 23**). This closes the second half of Q-B6; the partial flag is removed and Q-B6 now names its two checklist questions explicitly, so the split cannot cause confusion again.
> - **Low-threshold report** — filters on the yearly average of **own credit points** (new **Rule 24**). ⚠️ Differs from the recommendation, which proposed Business Volume. The export still carries both figures; only the filter metric changed.
> - **Rupee entry removed entirely** — points are the only thing ever entered, decimals accepted on the field (**Rule 16** rewritten), two decimal places throughout with rounding only at display (new **Rule 22**). ⚠️ This **reverses the original "both entry modes" decision** locked at the start of this work. The `1 point = 500 Rs` setting is retained on the settings screen per draft L56 but is reference only and never displayed elsewhere (**Rule 14** rewritten).
>
> No calculation, formula or scenario total changed. Rules 1–21 were not renumbered. **Seven of eight blocking questions are now answered** — only Q-B5 remains.
>
> **Change log — 2026-08-03 (Q-B6 period boundaries answered; new alert requirement).** The client confirmed the calendar-month period rule, and **added a requirement not present in the original draft**: a persistent, undismissable alert — banner on every screen plus a notification entry — that stays up until the reset actually completes. Where several months are outstanding, all are listed and the **oldest closes first**, each keeping its own backup and snapshot. Captured as new **Rules 20 and 21** in FR-7; no existing rule was renumbered. This supersedes the earlier assumption that a skipped month would simply have no snapshot. The **averaging-denominator half of Q-B6 remains open** (checklist Question 7). No calculation, formula or scenario total changed.
>
> **Change log — 2026-08-03 (Q-B3 and Q-B4 answered).** The client confirmed that a member's own credit points are **always** counted in their Business Volume, and that Person P's absence from the Scenario 4 and 5 sums was a simplification in the draft's write-up rather than a different rule. **Rule 6 is unchanged and no scenario total moved**; its marker moved from `[DERIVED]` to `[CONFIRMED]`, and the explanatory notes on §5.4 and §5.5 moved from open queries to settled statements. This closes the Business Volume definition end to end — formula, one-level rollup and own-points term are all now client-confirmed. Four blocking questions remain.
>
> **Change log — 2026-08-03 (Q-B2 answered).** The client confirmed that a member earns **nothing on their own credit points** — own points still feed Business Volume and therefore still set the member's slab, but never produce an earning term. This matched the recommendation already carried, so **Rule 8 is unchanged and no scenario total moved**; the marker on that bullet moved from `[DERIVED]` to `[CONFIRMED]`. Six blocking questions remain.
>
> **Change log — 2026-08-03 (Q-B1 answered).** The client confirmed that the differential percentage applies to the child's **Business Volume**, not their individually-added credit points. This matched the recommendation the specification already carried, so **Rule 8 is unchanged and no scenario total moved** — the marker on that bullet moved from `[DERIVED]` to `[CONFIRMED]`. Seven blocking questions remain.
>
> **Change log — 2026-08-03 (business volume wording).** The definition of Business Volume was reworded to lead with the method rather than the effect: the sum is **one level deep**, taking each direct child's already-computed figure. Full-depth coverage follows transitively because each child's figure is itself complete. Confirmed with the client. **The formula in Rule 6 is unchanged and no scenario total moved** — the table above still holds.
>
> **Change log — 2026-08-03.** Three mislabelled slab percentages in the draft's Scenario 1, 2 and 3 working (draft lines 74, 89 and 109–113) were corrected by the client. All three were labelling errors in the intermediate steps; **no scenario total changed** and no rule in §4 was affected. The draft is now internally consistent throughout, and the table above still holds.

---

## 9. Open Questions

Nothing below has been assumed. These are ordered by how much they block work. **Answered questions stay here, marked, rather than being deleted** — the reasoning is the record of why a rule is what it is.

### Blocking
*Calculation cannot be built until these are answered. **7 of 8 answered, 1 deferred** — Q-B5 (what the monthly reset clears) is parked pending client input and is the sole remaining blocker.*

**Q-B1 — Differential base.** ✅ **ANSWERED — 2026-08-03: use the child's Business Volume.** Confirmed by the client; no longer open. Original question, retained for the record: confirm the differential applies to the child's **Business Volume** (their own points plus each of *their* direct children's business volume), not the child's individually-added points. Only Scenario 3 distinguishes the two, and it points to BV. If the client had meant ICP, every multi-level result in §5 would have changed.

**Q-B2 — Self-earning.** ✅ **ANSWERED — 2026-08-03: no, a member does not earn on their own credit points.** Confirmed by the client; no longer open. Original question, retained for the record: does a member earn a differential on their **own** credit points? No scenario shows it — a member's own points raise their slab but never generate an earning term. Confirm this is intended and not an omission from the examples.

**Q-B3 — Parent's own points in BV.** ✅ **ANSWERED — 2026-08-03: yes, a member's own points are always counted in their Business Volume.** Confirmed by the client; no longer open. Original question, retained for the record: confirm BV always includes the member's own ICP. Scenarios 1 and 3 include it (D's 500, A's derived 500); Scenarios 4 and 5 compute `P = A+B+C+D…` with no P term at all. Was P's contribution zero, or was it simply left out of the write-up?

**Q-B4 — Scenario 4 & 5 arithmetic.** ✅ **ANSWERED — 2026-08-03: Person P's own points were left out of the write-up for simplicity. The rule stands — own points are always counted.** Confirmed by the client; no longer open. Original question, retained for the record: directly following from Q-B3 — should BV(P) in Scenario 4 be 100,000 (as written) or 100,000 + ICP(P)? If P holds points, P's slab is unchanged (already top) but the numbers in any report differ.

**Q-B5 — What the monthly reset zeroes.** ⏸️ **DEFERRED — 2026-08-03.** Parked; being checked with the client separately. **This is the last blocking question, and architecture cannot be finalised until it is answered** — the whole historical-snapshot model and every yearly report depend on it. Original question, retained: individual credit points only? Earned points too? Are Business Volumes recomputed to zero as a consequence, or archived? Precisely what is written to the mandatory backup before zeroing — a point-in-time snapshot of every member's ICP, BV, slab and earned points?

**Q-B6 — Period boundaries and the yearly average denominator.** ✅ **ANSWERED — 2026-08-03.** This entry bundles two separate things, which the client checklist splits into **Question 6** and **Question 7**. Both are now settled:
> - **Period boundaries** (checklist Question 6) — a period is a calendar month; the reset closes the month it belongs to; points entered before the reset count into the month being closed. The client additionally required a **persistent undismissable alert** until the reset completes, and **oldest-first closing** when several months are outstanding — see Rules 20 and 21. This supersedes the earlier assumption that a skipped month would simply have no snapshot: outstanding months stay outstanding and are eventually closed with their own backup and snapshot.
> - **Yearly average denominator** (checklist Question 7) — divide by the count of periods that actually have a snapshot, and display that count alongside each average. See Rule 23.
>
> Original question, retained for the record: since reset is manual, a "month" is not necessarily a calendar month. Is a period the calendar month, or the interval between two resets? What happens if the admin resets on the 12th, resets twice in one month, or skips a month entirely? And is the yearly average divided by a fixed 12, by the number of periods that actually have a snapshot, or by the months since the member joined? Yearly reporting is unbuildable without this.

**Q-B7 — Low-performer metric.** ✅ **ANSWERED — 2026-08-03: the yearly average of the member's own credit points.** ⚠️ **This differs from the recommendation**, which proposed Business Volume; the client wants the report to reflect what each person personally brought in. See Rule 24. Original question, retained for the record: "Yearly average below 100 points" — the average of **which** value? Credit points, business volume, or earned points? The three give very different member lists.

**Q-B8 — Rounding.** ✅ **ANSWERED — 2026-08-03: two decimal places throughout, rounded only for display (Rule 22). Rupee entry is removed entirely** — points are the only thing ever entered, decimals accepted on the field itself (Rule 16). ⚠️ **The rupee half differs from the recommendation** and reverses the original "both entry modes" decision; the point-value setting is retained but never displayed (Rule 14). Original question, retained for the record: Are earned points stored with decimals or rounded? At what precision, and rounded at which step — per child term, or on the total? Separately: when a rupee amount is not a multiple of 500, does the system round the resulting points down, to nearest, or reject the entry?

### Important
*Needed before design is finalised. **8 of 8 answered**, 2 of those provisional.*

> 🟡 **Awaiting final sign-off:** Q-I2 (chart value) and Q-I4 (royalty stacking). Both are being re-checked with the client. Design against them, but treat them as movable.

**Q-I1 — Naming.** ✅ **ANSWERED — 2026-08-03: use "business volume" everywhere.** The draft's "total purchase volume" (L39) is replaced by it; *purchase* is on the client's forbidden list, *business volume* is not. See §1.1. Original question, retained: what generic term replaces "business volume" / "purchase volume" in the UI and exports, given the client's no-trade-vocabulary rule? Suggested: *Group Volume* or *Team Points*.

**Q-I2 — Hierarchy chart value.** 🟡 **PROVISIONAL — 2026-08-03: show the member's own credit points.** ⚠️ **Differs from the recommendation** of BV. Being re-checked with the client. See FR-2. Original question, retained: the chart node shows "credit points" — ICP or BV? BV is more informative; ICP is more literal.

**Q-I3 — Royalty rate configurability.** ✅ **ANSWERED — 2026-08-03: yes, the 1% rate is configurable** alongside the qualifying-child count. Original question, retained: the draft confirms the qualifying-child count (3) is configurable. Is the 1% rate configurable too?

**Q-I4 — Royalty stacking up the chain.** 🟡 **PROVISIONAL — 2026-08-03: yes, allowed at every level**, each assessed independently against its own direct children. See Rule 25. Being re-checked with the client, as it directly increases total payout. Original question, retained: if P qualifies for royalty, and P's own parent has 3+ top-slab direct children including P, that parent also earns 1% of P's BV — and so on to the root. Confirm this compounding at every level is intended, or whether royalty is capped at some level count.

**Q-I5 — Slab table editing.** ✅ **ANSWERED — 2026-08-03: rows can be added and removed**, and the top slab is always the highest-percentage row. See Rule 27. Original question, retained: can the admin **add or remove** slab rows, or only change thresholds and percentages on the existing seven? If rows can be added, is the top slab always the highest-percentage row?

**Q-I6 — Recalculation trigger.** ✅ **ANSWERED — 2026-08-03: recalculate immediately on every points entry.** Expected scale is 500–5,000 members, which supports this comfortably; implementation should update only the affected chain upward. See Rule 26. ☐ Points entries per month still not supplied. Original question, retained: is the whole tree recomputed live on every point entry, on demand via a button, or only at period close? This drives the entire data model — live recomputation on a deep tree is very different work from a batch job.

**Q-I7 — Member lifecycle.** ✅ **ANSWERED — 2026-08-03: edit freely; deactivate but never hard-delete; moves permitted with closed months frozen.** See Rule 28. Original question, retained: can a member be edited, deactivated, deleted, or **moved to a different sponsor**? If a member moves, are past periods recalculated, or frozen as they were?

**Q-I8 — "Final discounts."** ✅ **ANSWERED — 2026-08-03: it is not a discount. The draft means "final earned points".** The client has corrected the wording: draft line 7's *"final discounts"* refers to the **final earned points** already modelled by Rule 12. **No discount feature exists and nothing extra is to be built.** The client separately converts final earned points into rupees at 1 point = 500 Rs **manually, outside the application** — that conversion must not appear anywhere in the software, though it may be added later if requested (see Rule 14).
>
> This closes the one item previously flagged as a possible missing feature. There is no missing feature.
>
> Original question, retained for the record: line 7 of the draft says the system will *"calculate final discounts based on calculated points and user hierarchy"*, but no scenario ever produces a discount. Is the slab % itself the discount, or is a discount a separate output that hasn't been specified at all?

### Minor
*Can be settled during design. **5 of 5 answered.***

**Q-M1** — ✅ **ANSWERED — 2026-08-03.** Reference must resolve to an existing active member; the single root is created once at setup without one; moves creating a loop are blocked. See Rule 30. Original question, retained: reference ID validation: must the referenced member already exist? How is the root member created, given the Reference ID is mandatory? How are cycles prevented if members can be re-parented?

**Q-M2** — ✅ **ANSWERED — 2026-08-03: one administrator account (the client only); members never log in.** Protected by a 6-digit PIN or a complex password, ⏸️ the client to decide which. **Failed-attempt lockout is mandatory either way.** See Rule 29. Original question, retained: authentication: the draft never mentions login. Is this a single admin account, or multiple users with roles? Do members ever log in to see their own data?

**Q-M3** — ✅ **ANSWERED — 2026-08-03: all fields offered, four defaults pre-ticked.** Full list in Rule 33. Original question, retained: full list of fields available as configurable export columns.

**Q-M4** — ✅ **ANSWERED — 2026-08-03: downloaded locally and retained permanently in the system; nothing auto-deleted.** See Rule 31. Original question, retained: where backup files are written, and how long they are retained.

**Q-M5** — ✅ **ANSWERED — 2026-08-03: warn but allow.** See Rule 32. Original question, retained: behaviour when hierarchy depth exceeds the configured maximum at onboarding: block, warn, or allow?

---

## 10. Not Covered by the Draft

Deliberately out of scope for this document because the draft says nothing about them. Each needs its own decision before architecture:

- Hosting, deployment, backup infrastructure
- Points entries per month — still not supplied. Member count is settled at 500–5,000 (Rule 26); only the entry rate remains unknown
- Audit trail / who-changed-what
- Non-functional targets: response time, concurrent users, browser support
- Mobile / responsive requirements
- Localisation and currency beyond INR
- Migration of any existing member or points data

---

*Every question now has an answer. **Q-B5 — what the monthly reset clears — is the sole remaining blocker on architecture.** Q-I2 and Q-I4 are provisional and awaiting a final word from the client. Two small inputs are still outstanding: points entries per month, and the PIN-versus-password choice.*
